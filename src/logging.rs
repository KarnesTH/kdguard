use std::{
    env::consts::{ARCH, OS},
    fs::{self, File},
    path::PathBuf,
};

use chrono::Local;
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use sysinfo::System;

use crate::errors::LoggingError;

const MAX_LOG_FILES: usize = 10;

pub struct LoggingManager;

impl LoggingManager {
    /// Initialize Logging
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub fn init() -> Result<(), LoggingError> {
        let logging_path = Self::get_logging_path()?;

        let system_info_path = logging_path.join("system_info.log");
        if !system_info_path.exists() {
            let system_info = Self::collect_system_info()?;
            fs::write(&system_info_path, system_info)
                .map_err(|e| LoggingError::WriteSystemInfo(e.to_string()))?;
        }

        let datetime = Local::now().format("%Y-%m-%dT%H-%M-%S");
        let log_file_name = format!("kdguard_{}.log", datetime);
        let log_file_path = logging_path.join(&log_file_name);

        let log_file =
            File::create(&log_file_path).map_err(|e| LoggingError::CreateFile(e.to_string()))?;

        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            log_file,
        )])
        .map_err(|e| LoggingError::Initialize(e.to_string()))?;

        Self::cleanup_old_logs(&logging_path, MAX_LOG_FILES)?;

        Ok(())
    }

    /// Get the logging path
    ///
    /// # Returns
    ///
    /// Returns the logging path if successful, otherwise an error
    fn get_logging_path() -> Result<PathBuf, LoggingError> {
        if OS == "Windows" || OS == "Darwin" {
            let local_app_data = dirs::data_local_dir().ok_or(LoggingError::GetDirectory(
                "Failed to get logging directory".to_string(),
            ))?;
            let logging_dir = local_app_data.join("kdguard").join("logs");
            fs::create_dir_all(&logging_dir)
                .map_err(|e| LoggingError::CreateDirectory(e.to_string()))?;
            Ok(logging_dir)
        } else {
            let log_dir = dirs::state_dir().ok_or(LoggingError::GetDirectory(
                "Failed to get logging directory".to_string(),
            ))?;
            let logging_dir = log_dir.join("kdguard").join("logs");
            fs::create_dir_all(&logging_dir)
                .map_err(|e| LoggingError::CreateDirectory(e.to_string()))?;
            Ok(logging_dir)
        }
    }

    /// Collect system information
    ///
    /// # Returns
    ///
    /// Returns the system information if successful, otherwise an error
    fn collect_system_info() -> Result<String, LoggingError> {
        let mut system = System::new_all();
        system.refresh_all();

        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

        let cpu_count = system.cpus().len();
        let cpu_name = if let Some(cpu) = system.cpus().first() {
            cpu.brand().to_string()
        } else {
            "Unknown".to_string()
        };

        let total_memory = system.total_memory();
        let available_memory = system.available_memory();

        let os_name = System::name().unwrap_or_else(|| OS.to_string());
        let os_version = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());

        let info = format!(
            "System Information\n\
            ===================\n\
            OS: {} {}\n\
            Architecture: {}\n\
            Hostname: {}\n\
            CPU: {} ({} cores)\n\
            Total Memory: {} MB\n\
            Available Memory: {} MB\n\
            ===================\n",
            os_name,
            os_version,
            ARCH,
            hostname,
            cpu_name,
            cpu_count,
            total_memory / 1024 / 1024,
            available_memory / 1024 / 1024
        );

        Ok(info)
    }

    /// Cleanup old logs
    ///
    /// # Arguments
    ///
    /// * `logging_path`: The path to the logging directory
    /// * `max_count`: The maximum number of logs to keep
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    fn cleanup_old_logs(logging_path: &PathBuf, max_count: usize) -> Result<(), LoggingError> {
        let mut log_files: Vec<_> = fs::read_dir(logging_path)
            .map_err(|e| LoggingError::Cleanup(e.to_string()))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("kdguard_") && n.ends_with(".log"))
                        .unwrap_or(false)
                {
                    let metadata = fs::metadata(&path).ok()?;
                    Some((path, metadata.modified().ok()?))
                } else {
                    None
                }
            })
            .collect();

        log_files.sort_by(|a, b| b.1.cmp(&a.1));

        if log_files.len() > max_count {
            for (path, _) in log_files.iter().skip(max_count) {
                fs::remove_file(path).map_err(|e| LoggingError::Cleanup(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Log an info message
    ///
    /// # Arguments
    ///
    /// * `msg`: The message to log
    pub fn info(msg: &str) {
        log::info!("{}", msg);
    }

    /// Log a debug message
    ///
    /// # Arguments
    ///
    /// * `msg`: The message to log
    pub fn debug(msg: &str) {
        log::debug!("{}", msg);
    }

    /// Log a warning message
    ///
    /// # Arguments
    ///
    /// * `msg`: The message to log
    pub fn warn(msg: &str) {
        log::warn!("{}", msg);
    }

    /// Log an error message
    ///
    /// # Arguments
    ///
    /// * `msg`: The message to log
    pub fn error(msg: &str) {
        log::error!("{}", msg);
    }
}
