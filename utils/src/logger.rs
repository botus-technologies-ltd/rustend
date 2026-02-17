use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logger - auto-detects crate name from module path
/// 
/// # Returns
/// A guard that must be held for the duration of the program to keep logging active
/// 
/// # Example
/// ```ignore
/// // In app crate - automatically uses "app" as log file name
/// let _guard = utils::init!();
/// 
/// // With custom log level
/// let _guard = utils::init!("debug");
/// ```
#[macro_export]
macro_rules! init {
    () => {
        utils::logger::init(module_path!())
    };
    ($level:expr) => {
        utils::logger::init_with_level(module_path!(), $level)
    };
}

/// Initialize a logger with custom log level
pub fn init_with_level(crate_name: &str, level: &str) -> WorkerGuard {
    // Create logs directory path
    let log_dir = PathBuf::from("logs");
    
    // Create crate-specific log file path
    let log_file = log_dir.join(format!("{}.log", crate_name));
    
    // Create file appender
    let file_appender = tracing_appender::rolling::daily(&log_dir, format!("{}.log", crate_name));
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    // Initialize subscriber with both stdout and file output
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_target(true),
        )
        .init();
    
    tracing::info!("Logger initialized for crate: {}", crate_name);
    tracing::info!("Log file: {:?}", log_file);
    
    guard
}

/// Initialize logger that writes only to stdout (useful for testing)
pub fn init_stdout() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_target(true),
        )
        .init();
}

/// Get the log file path for a specific crate
pub fn get_log_path(crate_name: &str) -> PathBuf {
    PathBuf::from("logs").join(format!("{}.log", crate_name))
}
