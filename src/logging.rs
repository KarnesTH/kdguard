use std::{
    env::consts::{ARCH, OS},
    fs::{self, File},
    path::{Path, PathBuf},
};

use chrono::Local;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use crate::errors::LoggingError;

const MAX_LOG_FILES: usize = 10;

pub struct LoggingManager;

impl LoggingManager {
    /// Creates `kdguard_<timestamp>.log` in the data directory, registers it as the log target and
    /// keeps only the newest log files.
    pub fn init() -> Result<(), LoggingError> {
        let dir = Self::logs_path().ok_or(LoggingError::NoDirectory)?;
        fs::create_dir_all(&dir).map_err(|e| LoggingError::CreateDirectory(e.to_string()))?;

        let name = format!("kdguard_{}.log", Local::now().format("%Y-%m-%dT%H-%M-%S"));
        let file =
            File::create(dir.join(name)).map_err(|e| LoggingError::CreateFile(e.to_string()))?;
        WriteLogger::init(LevelFilter::Info, Config::default(), file)
            .map_err(|e| LoggingError::Initialize(e.to_string()))?;

        Self::cleanup_old_logs(&dir, MAX_LOG_FILES);
        Self::info(&format!(
            "kdguard {} on {OS} {ARCH}",
            env!("CARGO_PKG_VERSION")
        ));
        Ok(())
    }

    /// Returns `<state dir>/kdguard` (Linux), otherwise `<local data dir>/kdguard`.
    pub fn data_path() -> Option<PathBuf> {
        dirs::state_dir()
            .or_else(dirs::data_local_dir)
            .map(|dir| dir.join("kdguard"))
    }

    /// Returns `<data path>/logs`.
    pub fn logs_path() -> Option<PathBuf> {
        Self::data_path().map(|dir| dir.join("logs"))
    }

    /// Deletes all but the `max_count` newest `kdguard_*.log` files. Failures are ignored.
    fn cleanup_old_logs(dir: &Path, max_count: usize) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        let mut logs: Vec<_> = entries
            .flatten()
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                name.starts_with("kdguard_") && name.ends_with(".log")
            })
            .filter_map(|entry| Some((entry.path(), entry.metadata().ok()?.modified().ok()?)))
            .collect();
        logs.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));
        for (path, _) in logs.iter().skip(max_count) {
            let _ = fs::remove_file(path);
        }
    }

    pub fn info(message: &str) {
        log::info!("{message}");
    }

    pub fn warn(message: &str) {
        log::warn!("{message}");
    }

    pub fn error(message: &str) {
        log::error!("{message}");
    }
}
