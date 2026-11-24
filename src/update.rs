use std::{
    env::consts::{ARCH, OS},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use ring::digest;

use crate::errors::UpdateError;
use crate::kd_cli::confirm::Confirm;
use crate::kd_cli::output::Output;
use crate::kd_cli::terminal::Terminal;
use crate::logging::LoggingManager;
use crate::translation::Translation;

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/KarnesTH/kdguard/releases/latest";
const DOWNLOAD_URL: &str = "https://github.com/KarnesTH/kdguard/releases/download";
const CHECK_INTERVAL_SECONDS: u64 = 24 * 60 * 60;

pub struct UpdateManager;

impl UpdateManager {
    /// Checks GitHub for a newer release at most once a day and only in an interactive terminal.
    /// A newer release is installed after confirmation. Failures are printed as warnings and never
    /// stop the program.
    pub fn check() {
        if !Terminal::interactive() || !Self::check_due() {
            return;
        }
        Self::record_check();

        let latest = match Self::latest_tag() {
            Ok(tag) => tag,
            Err(error) => {
                LoggingManager::warn(&format!("Update check failed: {error}"));
                Output::warn(&Translation::get("update.check_failed"), &error.to_string());
                return;
            }
        };
        let version = latest.trim_start_matches('v');
        if Self::compare_versions(version, env!("CARGO_PKG_VERSION")) <= 0 {
            return;
        }

        let question = Translation::with("update.confirm", &[("version", version)]);
        if !Confirm::new(&question, false).ask().unwrap_or(false) {
            return;
        }
        match Self::install(&latest) {
            Ok(()) => Output::success(
                &Translation::with("update.done", &[("version", version)]),
                &Translation::get("update.restart"),
            ),
            Err(error) => {
                LoggingManager::error(&format!("Update failed: {error}"));
                Output::error(&Translation::get("update.failed"), &error.to_string());
            }
        }
    }

    /// Returns `<data path>/update_check`.
    fn state_file() -> Option<PathBuf> {
        LoggingManager::data_path().map(|dir| dir.join("update_check"))
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs())
    }

    /// True when the last recorded check is older than a day or missing.
    fn check_due() -> bool {
        let last = Self::state_file()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| content.trim().parse::<u64>().ok())
            .unwrap_or(0);
        Self::now().saturating_sub(last) >= CHECK_INTERVAL_SECONDS
    }

    fn record_check() {
        if let Some(path) = Self::state_file() {
            if let Some(dir) = path.parent() {
                let _ = fs::create_dir_all(dir);
            }
            let _ = fs::write(path, Self::now().to_string());
        }
    }

    /// Runs `curl` over HTTPS only and returns stdout.
    fn curl(args: &[&str]) -> Result<Vec<u8>, UpdateError> {
        let output = Command::new("curl")
            .args([
                "-fsSL",
                "--proto",
                "=https",
                "--tlsv1.2",
                "-A",
                "kdguard-updater",
            ])
            .args(args)
            .output()
            .map_err(|e| UpdateError::Curl(e.to_string()))?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(UpdateError::Curl(message));
        }
        Ok(output.stdout)
    }

    /// Reads `tag_name` of the latest release. Tags with characters other than letters, digits,
    /// `.` and `-` are rejected.
    fn latest_tag() -> Result<String, UpdateError> {
        let body = Self::curl(&[
            "--max-time",
            "10",
            "-H",
            "Accept: application/vnd.github+json",
            LATEST_RELEASE_URL,
        ])?;
        let json: serde_json::Value =
            serde_json::from_slice(&body).map_err(|_| UpdateError::InvalidResponse)?;
        let tag = json
            .get("tag_name")
            .and_then(|value| value.as_str())
            .ok_or(UpdateError::InvalidResponse)?;
        if tag.is_empty()
            || !tag
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return Err(UpdateError::InvalidResponse);
        }
        Ok(tag.to_string())
    }

    /// Returns 1 if `v1` is newer than `v2`, -1 if older and 0 if equal, comparing the dot
    /// separated numbers from left to right.
    fn compare_versions(v1: &str, v2: &str) -> i32 {
        let parse = |version: &str| -> Vec<u32> {
            version
                .split('.')
                .map(|part| part.parse().unwrap_or(0))
                .collect()
        };
        let (a, b) = (parse(v1), parse(v2));
        for i in 0..a.len().max(b.len()) {
            let (x, y) = (
                a.get(i).copied().unwrap_or(0),
                b.get(i).copied().unwrap_or(0),
            );
            if x != y {
                return if x > y { 1 } else { -1 };
            }
        }
        0
    }

    /// Returns the platform part of the release file name.
    fn platform() -> Result<&'static str, UpdateError> {
        match (OS, ARCH) {
            ("linux", "x86_64") => Ok("linux-x86_64"),
            ("macos", "x86_64") => Ok("macos-x86_64"),
            ("macos", "aarch64") => Ok("macos-aarch64"),
            ("windows", "x86_64") => Ok("windows-x86_64"),
            _ => Err(UpdateError::UnsupportedPlatform(format!("{OS}-{ARCH}"))),
        }
    }

    /// Downloads the release binary next to the running executable, checks it against the
    /// published `.sha256` file and replaces the executable with it. On Windows a `kdg.exe` that is a
    /// copy instead of a symlink is replaced as well.
    fn install(tag: &str) -> Result<(), UpdateError> {
        let platform = Self::platform()?;
        let version = tag.trim_start_matches('v');
        let extension = if cfg!(windows) { ".exe" } else { "" };
        let asset = format!("kdguard_{version}-{platform}{extension}");
        let url = format!("{DOWNLOAD_URL}/{tag}/{asset}");

        let executable = std::env::current_exe()
            .and_then(fs::canonicalize)
            .map_err(|e| UpdateError::Install(e.to_string()))?;
        let dir = executable
            .parent()
            .ok_or_else(|| UpdateError::Install(executable.display().to_string()))?;
        let file_name = executable
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| UpdateError::Install(executable.display().to_string()))?;
        let new_path = dir.join(format!("{file_name}.new"));
        let old_path = dir.join(format!("{file_name}.old"));

        Output::info(
            &Translation::with("update.downloading", &[("version", version)]),
            "",
        );
        LoggingManager::info(&format!("Downloading {url}"));
        let new_path_text = new_path.to_string_lossy().into_owned();
        Self::curl(&["--max-time", "300", "-o", &new_path_text, &url])?;

        if let Err(error) = Self::verify(&new_path, &format!("{url}.sha256")) {
            let _ = fs::remove_file(&new_path);
            return Err(error);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&new_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| UpdateError::Install(e.to_string()))?;
        }

        let _ = fs::remove_file(&old_path);
        fs::rename(&executable, &old_path).map_err(|e| UpdateError::Install(e.to_string()))?;
        if let Err(error) = fs::rename(&new_path, &executable) {
            let _ = fs::rename(&old_path, &executable);
            return Err(UpdateError::Install(error.to_string()));
        }
        let _ = fs::remove_file(&old_path);

        #[cfg(windows)]
        {
            let alias = dir.join("kdg.exe");
            let is_copy =
                fs::symlink_metadata(&alias).is_ok_and(|meta| !meta.file_type().is_symlink());
            if is_copy {
                fs::copy(&executable, &alias).map_err(|e| UpdateError::Install(e.to_string()))?;
            }
        }

        LoggingManager::info(&format!("Updated to {tag}"));
        Ok(())
    }

    /// Compares the SHA-256 of the file with the first word of the checksum file at `url`.
    fn verify(path: &Path, url: &str) -> Result<(), UpdateError> {
        let checksum =
            Self::curl(&["--max-time", "30", url]).map_err(|_| UpdateError::ChecksumMissing)?;
        let expected = String::from_utf8_lossy(&checksum)
            .split_whitespace()
            .next()
            .map(str::to_lowercase)
            .filter(|hash| hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .ok_or(UpdateError::ChecksumMissing)?;

        let content = fs::read(path).map_err(|e| UpdateError::Install(e.to_string()))?;
        let actual: String = digest::digest(&digest::SHA256, &content)
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        if actual != expected {
            return Err(UpdateError::ChecksumMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(UpdateManager::compare_versions("0.7.0", "0.6.5"), 1);
        assert_eq!(UpdateManager::compare_versions("0.6.5", "0.7.0"), -1);
        assert_eq!(UpdateManager::compare_versions("0.7.0", "0.7.0"), 0);
        assert_eq!(UpdateManager::compare_versions("1.0", "0.9.9"), 1);
        assert_eq!(UpdateManager::compare_versions("0.10.0", "0.9.0"), 1);
    }

    #[test]
    fn test_platform_is_known_on_ci() {
        if OS == "linux" && ARCH == "x86_64" {
            assert_eq!(UpdateManager::platform().unwrap(), "linux-x86_64");
        }
    }
}
