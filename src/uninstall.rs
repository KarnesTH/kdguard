use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::commands::CommandResult;
use crate::config::Config;
use crate::errors::UninstallError;
use crate::kd_cli::confirm::Confirm;
use crate::kd_cli::output::Output;
use crate::logging::LoggingManager;
use crate::translation::Translation;

pub struct UninstallManager;

impl UninstallManager {
    /// After confirmation removes the config directory, the data directory with the logs, the
    /// `kdg` alias next to the executable and the executable itself. Only these known paths are
    /// removed.
    pub fn uninstall() -> CommandResult {
        let executable = std::env::current_exe()
            .and_then(fs::canonicalize)
            .map_err(|e| UninstallError::Executable(e.to_string()))?;
        let dir = executable
            .parent()
            .ok_or_else(|| UninstallError::Executable(executable.display().to_string()))?
            .to_path_buf();

        if !Confirm::new(&Translation::get("uninstall.confirm"), false).ask()? {
            Output::warn(&Translation::get("uninstall.cancelled"), "");
            return Ok(());
        }
        LoggingManager::info("Uninstalling kdguard");

        if let Some(config_dir) = Config::path()?.parent() {
            Self::remove_dir(config_dir)?;
        }
        let data_dir = LoggingManager::data_path();
        #[cfg(unix)]
        if let Some(data_dir) = &data_dir {
            Self::remove_dir(data_dir)?;
        }
        Self::remove_alias(&dir, &executable)?;
        Self::remove_executable(&executable, &dir, data_dir.as_deref())?;

        Output::success(&Translation::get("uninstall.done"), "");
        Ok(())
    }

    fn remove_dir(path: &Path) -> Result<(), UninstallError> {
        if !path.exists() {
            return Ok(());
        }
        fs::remove_dir_all(path)
            .map_err(|e| UninstallError::Remove(path.to_path_buf(), e.to_string()))?;
        Output::success(
            &Translation::get("uninstall.removed"),
            &path.display().to_string(),
        );
        Ok(())
    }

    /// Removes `kdg` when it is a symlink to the executable (Unix) or `kdg.exe` (Windows).
    fn remove_alias(dir: &Path, executable: &Path) -> Result<(), UninstallError> {
        let alias: PathBuf = dir.join(if cfg!(windows) { "kdg.exe" } else { "kdg" });
        let Ok(metadata) = fs::symlink_metadata(&alias) else {
            return Ok(());
        };
        let ours = if metadata.file_type().is_symlink() {
            fs::canonicalize(&alias).is_ok_and(|target| target == executable)
        } else {
            cfg!(windows)
        };
        if !ours {
            return Ok(());
        }
        fs::remove_file(&alias)
            .map_err(|e| UninstallError::Remove(alias.clone(), e.to_string()))?;
        Output::success(
            &Translation::get("uninstall.removed"),
            &alias.display().to_string(),
        );
        Ok(())
    }

    /// Unix: deletes the executable. Windows: the running executable and the open log file cannot
    /// be deleted, so a detached `cmd` waits two seconds, deletes the executable and the data
    /// directory and removes the install directory when it is empty.
    fn remove_executable(
        executable: &Path,
        dir: &Path,
        data_dir: Option<&Path>,
    ) -> Result<(), UninstallError> {
        #[cfg(unix)]
        {
            let _ = (dir, data_dir);
            fs::remove_file(executable)
                .map_err(|e| UninstallError::Remove(executable.to_path_buf(), e.to_string()))?;
        }
        #[cfg(windows)]
        {
            let mut script = format!(
                "timeout /T 2 /NOBREAK >NUL & del /F /Q \"{}\"",
                executable.display()
            );
            if let Some(data_dir) = data_dir {
                script += &format!(" & rmdir /S /Q \"{}\"", data_dir.display());
            }
            script += &format!(" & rmdir \"{}\"", dir.display());
            use std::os::windows::process::CommandExt;
            std::process::Command::new("cmd")
                .raw_arg(format!("/C {script}"))
                .spawn()
                .map_err(|e| UninstallError::Remove(executable.to_path_buf(), e.to_string()))?;
        }
        Output::success(
            &Translation::get("uninstall.removed"),
            &executable.display().to_string(),
        );
        Ok(())
    }
}
