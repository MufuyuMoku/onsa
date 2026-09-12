//! Logging (SPEC §13): records go to a daily file in the application's log
//! folder, keeping a week of files, and to standard error. The About screen
//! can switch the Onsa crates to debug level while the application runs, so
//! a user can reproduce a problem and send the log.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, reload, EnvFilter, Registry};

/// Level while debug logging is off; `ONSA_LOG` overrides it.
const NORMAL: &str = "info";
/// Level while debug logging is on: Onsa's own crates in detail, the rest
/// as usual.
const DEBUG: &str = "info,onsa=debug,onsa_audio=debug,onsa_library=debug";
/// Daily files kept.
const KEEP_FILES: usize = 7;

/// The installed logger.
pub struct Logging {
    filter: reload::Handle<EnvFilter, Registry>,
    dir: PathBuf,
    normal: String,
    debug: AtomicBool,
    /// Flushes the file writer when the application ends.
    _guard: Option<WorkerGuard>,
}

impl Logging {
    /// Installs the logger. A log folder that cannot be created leaves only
    /// standard error, with a warning; logging never stops the application.
    pub fn init(dir: &Path) -> Self {
        let normal = std::env::var("ONSA_LOG")
            .ok()
            .filter(|value| EnvFilter::try_new(value).is_ok())
            .unwrap_or_else(|| NORMAL.to_string());
        let (filter, handle) = reload::Layer::new(EnvFilter::new(&normal));

        let appender = std::fs::create_dir_all(dir)
            .map_err(|error| error.to_string())
            .and_then(|()| {
                Builder::new()
                    .rotation(Rotation::DAILY)
                    .filename_prefix("onsa")
                    .filename_suffix("log")
                    .max_log_files(KEEP_FILES)
                    .build(dir)
                    .map_err(|error| error.to_string())
            });
        let (file_layer, guard, failure) = match appender {
            Ok(appender) => {
                let (writer, guard) = tracing_appender::non_blocking(appender);
                let layer = fmt::layer().with_writer(writer).with_ansi(false);
                (Some(layer), Some(guard), None)
            }
            Err(error) => (None, None, Some(error)),
        };

        let installed = tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(fmt::layer().with_writer(std::io::stderr))
            .try_init();
        if installed.is_err() {
            eprintln!("Onsa: a logger was already installed");
        }
        if let Some(error) = failure {
            tracing::warn!(dir = %dir.display(), "no log file, logging to standard error only: {error}");
        }
        install_panic_hook();

        Self {
            filter: handle,
            dir: dir.to_path_buf(),
            normal,
            debug: AtomicBool::new(false),
            _guard: guard,
        }
    }

    /// The folder the log files are in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Whether debug logging is on.
    pub fn debug(&self) -> bool {
        self.debug.load(Ordering::Relaxed)
    }

    /// Switches debug logging on or off.
    pub fn set_debug(&self, on: bool) {
        let directive = if on { DEBUG } else { self.normal.as_str() };
        match self.filter.reload(EnvFilter::new(directive)) {
            Ok(()) => {
                self.debug.store(on, Ordering::Relaxed);
                tracing::info!(debug = on, "log level changed");
            }
            Err(error) => tracing::warn!("log level cannot change: {error}"),
        }
    }
}

/// A panic lands in the log file too, not only on a console nobody sees.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("panic: {info}");
        previous(info);
    }));
}
