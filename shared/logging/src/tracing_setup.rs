use chrono::{Duration, NaiveDate, Utc};
use std::path::{Path, PathBuf};
use tracing::Level;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer,
    filter::LevelFilter,
    fmt,
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

#[derive(Default)]
struct FieldVisitor {
    message: String,
    context: Option<String>,
    fields: Vec<(String, String)>,
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}").trim_matches('"').to_string();
        } else if field.name() == "context" {
            self.context = Some(format!("{value:?}").trim_matches('"').to_string());
        } else {
            self.fields.push((
                field.name().to_string(),
                format!("{value:?}").trim_matches('"').to_string(),
            ));
        }
    }
}

pub struct RustMicroServiceFormatter {
    pub app_name: String,
}

fn format_rust_message(target: &str, raw_message: &str) -> String {
    if target == "LoggerInterceptor" {
        // e.g. "200 | [GET] /api/v1/ping - 2ms"
        let parts: Vec<&str> = raw_message.splitn(4, ' ').collect();
        if parts.len() >= 4 && parts[1] == "|" {
            let status = parts[0];
            let method = parts[2];
            let rest = parts[3];

            let colored_status = if status.starts_with('2') {
                format!("\x1b[32m{status}\x1b[0m") // green
            } else if status.starts_with('3') || status.starts_with('4') {
                format!("\x1b[33m{status}\x1b[0m") // yellow
            } else {
                format!("\x1b[1;31m{status}\x1b[0m") // bold red
            };

            let colored_method = format!("\x1b[32m{method}\x1b[0m");

            if let Some((path, duration)) = rest.rsplit_once(" - ") {
                return format!(
                    "{colored_status} | {colored_method} \x1b[1m{path}\x1b[0m \x1b[33m+{duration}\x1b[0m"
                );
            } else {
                return format!("{colored_status} | {colored_method} {rest}");
            }
        }
    }
    raw_message.to_string()
}

impl<S, N> FormatEvent<S, N> for RustMicroServiceFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        let now = chrono::Local::now().format("%-m/%-d/%Y, %-I:%M:%S %p");
        let pid = std::process::id();

        let (level_color, level_str) = match *event.metadata().level() {
            Level::ERROR => ("\x1b[1;31m", "  ERROR"),
            Level::WARN => ("\x1b[33m", "   WARN"),
            Level::INFO => ("\x1b[32m", "    LOG"),
            Level::DEBUG => ("\x1b[35m", "  DEBUG"),
            Level::TRACE => ("\x1b[36m", "  TRACE"),
        };

        let target = visitor
            .context
            .as_deref()
            .unwrap_or_else(|| event.metadata().target());
        let formatted_msg = format_rust_message(target, &visitor.message);

        write!(
            writer,
            "\x1b[32m[{}]\x1b[0m {} {} {level_color}{level_str}\x1b[0m \x1b[33m[{}]\x1b[0m {}",
            self.app_name, pid, now, target, formatted_msg
        )?;

        if !visitor.fields.is_empty() {
            write!(writer, " \x1b[2m(")?;
            for (i, (k, v)) in visitor.fields.iter().enumerate() {
                if i > 0 {
                    write!(writer, ", ")?;
                }
                write!(writer, "\x1b[36m{k}\x1b[0m: \x1b[37m{v}\x1b[0m")?;
            }
            write!(writer, "\x1b[2m)\x1b[0m")?;
        }

        writeln!(writer)
    }
}

/// RAII guards that keep the non-blocking background writer threads alive.
///
/// If these guards are dropped, any buffered log lines waiting to be written to disk
/// will be flushed immediately, and no further file logs will be written.
/// Therefore, `main()` should hold onto `LogGuards` for the entire process lifetime.
pub struct LogGuards {
    pub _worker_guards: Vec<WorkerGuard>,
}

/// Initializes structured logging for a service.
///
/// Features:
/// - **Console Output:** Colored, human-readable stdout output with timestamps and target names.
/// - **File Output (optional):** When `CREATE_LOG_FILES="true"`, non-blocking daily rolling JSON
///   log files are written to `logs/error/`, `logs/warn/`, `logs/info/`, and `logs/debug/`, replicating
///   Virat's Winston file rotation strategy with 30-day retention.
/// - **Environment Filtering:** Configurable via the `RUST_LOG` environment variable, defaulting to
///   `info,{service_name}=debug,actix_web=info`.
pub fn init_tracing(service_name: &str) -> LogGuards {
    // Load environment variables via shared config loader
    config::loader::load_env_file(service_name);

    let default_filter = format!("info,{service_name}=debug,actix_web=info");
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let mut guards = Vec::new();

    // 1. Colorful Console Formatted Output
    let console_layer = fmt::layer().event_format(RustMicroServiceFormatter {
        app_name: "Virat".to_string(),
    });

    // 2. Check if file logging is enabled (matches Virat's CREATE_LOG_FILES flag)
    let create_log_files = std::env::var("CREATE_LOG_FILES")
        .map(|val| val.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if create_log_files {
        let logs_base = Path::new("logs");

        // Error log appender
        let (error_appender, error_guard) = tracing_appender::non_blocking(
            tracing_appender::rolling::daily(logs_base.join("error"), "error.log"),
        );
        guards.push(error_guard);
        let error_file_layer = fmt::layer()
            .json()
            .with_ansi(false)
            .with_writer(error_appender)
            .with_filter(LevelFilter::from_level(Level::ERROR));

        // Warn log appender
        let (warn_appender, warn_guard) = tracing_appender::non_blocking(
            tracing_appender::rolling::daily(logs_base.join("warn"), "warn.log"),
        );
        guards.push(warn_guard);
        let warn_file_layer = fmt::layer()
            .json()
            .with_ansi(false)
            .with_writer(warn_appender)
            .with_filter(LevelFilter::from_level(Level::WARN));

        // Info log appender
        let (info_appender, info_guard) = tracing_appender::non_blocking(
            tracing_appender::rolling::daily(logs_base.join("info"), "info.log"),
        );
        guards.push(info_guard);
        let info_file_layer = fmt::layer()
            .json()
            .with_ansi(false)
            .with_writer(info_appender)
            .with_filter(LevelFilter::from_level(Level::INFO));

        // Debug log appender
        let (debug_appender, debug_guard) = tracing_appender::non_blocking(
            tracing_appender::rolling::daily(logs_base.join("debug"), "debug.log"),
        );
        guards.push(debug_guard);
        let debug_file_layer = fmt::layer()
            .json()
            .with_ansi(false)
            .with_writer(debug_appender)
            .with_filter(LevelFilter::from_level(Level::DEBUG));

        // Combine all layers with the registry
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(error_file_layer)
            .with(warn_file_layer)
            .with(info_file_layer)
            .with(debug_file_layer)
            .try_init();

        // 30-day retention: prune old logs on startup and spawn 24-hour task
        prune_old_log_files(&logs_base.join("error"), 30);
        prune_old_log_files(&logs_base.join("warn"), 30);
        prune_old_log_files(&logs_base.join("info"), 30);
        prune_old_log_files(&logs_base.join("debug"), 30);

        if tokio::runtime::Handle::try_current().is_ok() {
            spawn_retention_task(logs_base.to_path_buf(), 30);
        }
        tracing::info!("File logs are enabled");
    } else {
        // Only console output
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .try_init();
    }

    LogGuards {
        _worker_guards: guards,
    }
}

pub fn prune_old_log_files(dir: &Path, retention_day: i64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let cutoff_date = Utc::now().date_naive() - Duration::days(retention_day);

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(date_str) = file_name.split('.').last() {
                if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    if file_date < cutoff_date {
                        tracing::info!(
                            file = %path.display(),
                            date = %file_date.to_string(),
                            "pruning expired log file (>30 days old)"
                        );
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

pub fn spawn_retention_task(logs_dir: PathBuf, retention_days: i64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 60 * 60));
        loop {
            interval.tick().await;

            prune_old_log_files(&logs_dir.join("error"), retention_days);
            prune_old_log_files(&logs_dir.join("warn"), retention_days);
            prune_old_log_files(&logs_dir.join("info"), retention_days);
            prune_old_log_files(&logs_dir.join("debug"), retention_days);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_does_not_panic() {
        let _guards = init_tracing("test-service");
        tracing::info!(service = "test-service", "Telemetry test event logged");
        let logger = crate::logger::Logger::new("AuthService");
        logger.log("User successfully logged in");
    }

    #[test]
    fn test_prune_old_log_files() {
        let test_dir = std::env::temp_dir().join("virat_test_log_pruning");
        let _ = std::fs::create_dir_all(&test_dir);

        // 1. Create a dummy expired file from 40 days ago
        let old_file = test_dir.join("info.log.2026-07-01");
        std::fs::write(&old_file, "old log entry").unwrap();

        // 2. Create a dummy recent file
        let today_str = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let new_file = test_dir.join(format!("info.log.{}", today_str));
        std::fs::write(&new_file, "new log entry").unwrap();

        // Verify both exist initially
        assert!(old_file.exists());
        assert!(new_file.exists());

        // 3. Prune files older than 30 days
        prune_old_log_files(&test_dir, 30);

        // 4. Old file must be deleted, new file must remain!
        assert!(!old_file.exists(), "Old log file should have been pruned!");
        assert!(new_file.exists(), "Recent log file must NOT be pruned!");

        // Cleanup test directory
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
