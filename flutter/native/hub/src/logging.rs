//! Logging infrastructure for the Rust hub.
//!
//! Uses eprintln! as backend — flutter run captures stderr on macOS.
//! If this doesn't work, we'll switch to oslog crate.

static LOGGER: StderrLogger = StderrLogger;

struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "[RUST/{}] {} — {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Initialize logging. Call once at hub startup.
pub fn init_logging() {
    let level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(log::LevelFilter::Info);

    let _ = log::set_logger(&LOGGER);
    log::set_max_level(level);
}

/// Install a global panic hook that logs via eprintln + writes crash file.
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>");

        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let log_msg = format!(
            "RUST PANIC on thread '{thread_name}' at {location}: {message}"
        );

        // Print to stderr (captured by flutter run if eprintln works)
        eprintln!("!!! {log_msg}");

        // Also write to crash log file
        if let Err(e) = write_crash_log(&log_msg) {
            eprintln!("Failed to write crash log: {e}");
        }
    }));
}

fn write_crash_log(message: &str) -> std::io::Result<()> {
    use std::io::Write;

    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("wrenflow");
    std::fs::create_dir_all(&dir)?;

    let path = dir.join("crash.log");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    writeln!(file, "[{timestamp}] {message}")?;
    Ok(())
}

/// Convert a panic payload to a human-readable string.
pub fn panic_payload_to_string(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}
