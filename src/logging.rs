//! Logging: daily-rotated files in the platform log directory, plus stderr.
//!
//! Log location:
//! - macOS:   ~/Library/Logs/Kodular Starter/
//! - Windows: %LOCALAPPDATA%\Kodular Starter\logs\
//! - Linux:   $XDG_STATE_HOME/kodular-starter/logs/ (~/.local/state/...)
//!
//! The level defaults to `info` (with noisy dependencies quietened) and can be
//! overridden with `RUST_LOG`, e.g. `RUST_LOG=debug` or `RUST_LOG=starter=trace`.

use std::path::PathBuf;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::format::{DefaultFields, Writer};
use tracing_subscriber::fmt::{self, FormatFields};
use tracing_subscriber::prelude::*;

const DEFAULT_FILTER: &str = "info,adb_client=warn";
const MAX_LOG_FILES: usize = 7;

pub fn log_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return dirs::home_dir().map(|home| home.join("Library/Logs/Kodular Starter"));

    #[cfg(target_os = "windows")]
    return dirs::data_local_dir().map(|dir| dir.join("Kodular Starter").join("logs"));

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .map(|dir| dir.join("kodular-starter").join("logs"));
}

/// Installs the global subscriber. Call once, first thing in `main`.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    // Writes are blocking (no background worker), so nothing is lost when the
    // app exits via `process::exit` or aborts on panic.
    let file_appender = log_dir().and_then(|dir| {
        RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("starter")
            .filename_suffix("log")
            .max_log_files(MAX_LOG_FILES)
            .build(dir)
            .ok()
    });
    let file_layer = file_appender.map(|appender| {
        fmt::layer()
            .with_ansi(false)
            .fmt_fields(PlainFields::default())
            .with_writer(appender)
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(file_layer)
        .init();

    // Release builds use `panic = "abort"`; record the panic before the process dies.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("panic: {info}");
        default_hook(info);
    }));

    match log_dir() {
        Some(dir) => tracing::info!(
            version = env!("CARGO_PKG_VERSION"),
            os = std::env::consts::OS,
            arch = std::env::consts::ARCH,
            log_dir = %dir.display(),
            "Kodular Starter starting"
        ),
        None => tracing::warn!("no log directory available; logging to stderr only"),
    }
}

/// `DefaultFields` under a distinct type.
///
/// fmt layers cache each span's formatted fields in the span's extensions,
/// keyed by the field formatter's type. With both layers on `DefaultFields`,
/// the file layer would reuse the stderr layer's colored span fields, leaving
/// ANSI escapes in the log file. A separate type gives it its own plain cache.
#[derive(Default)]
struct PlainFields(DefaultFields);

impl<'writer> FormatFields<'writer> for PlainFields {
    fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> std::fmt::Result {
        self.0.format_fields(writer, fields)
    }
}
