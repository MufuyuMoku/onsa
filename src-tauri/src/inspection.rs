//! Whether anything outside Onsa may reach into its window.
//!
//! Neither WebView2 nor WebKitGTK needs Onsa's say to open a port that
//! hands the page to whoever connects: WebView2 reads extra browser
//! arguments from `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, and WebKitGTK
//! starts its remote inspector when `WEBKIT_INSPECTOR_SERVER` or
//! `WEBKIT_INSPECTOR_HTTP_SERVER` is set. Anything on the machine that can
//! set an environment variable for Onsa could then drive the player, read
//! the library and call every command.
//!
//! So this is decided here, once, before the window exists: a build made
//! with the `test-harness` feature — the builds the layout and motion
//! checkers drive — keeps what the environment asks for; every other build,
//! the released installer and packages among them, takes those requests out
//! of its own environment first and says so in the log. CI starts the
//! release build with the variable set and fails if the port answers.

use std::sync::OnceLock;

/// Whether this build may open a debugging port at all.
pub const ALLOWED: bool = cfg!(feature = "test-harness");

/// What was taken out at start, for the log once it exists.
static REFUSED: OnceLock<Vec<String>> = OnceLock::new();

/// Takes every request for a debugging port out of the environment, unless
/// this is a test build. Call first thing, before any thread or window.
pub fn close() {
    if ALLOWED {
        return;
    }
    let mut refused = Vec::new();
    if let Ok(value) = std::env::var(WEBVIEW2_ARGUMENTS) {
        let (kept, dropped) = without_debugging(&value);
        if !dropped.is_empty() {
            refused.extend(dropped);
            if kept.is_empty() {
                std::env::remove_var(WEBVIEW2_ARGUMENTS);
            } else {
                std::env::set_var(WEBVIEW2_ARGUMENTS, kept);
            }
        }
    }
    for name in WEBKIT_INSPECTOR {
        if std::env::var_os(name).is_some() {
            std::env::remove_var(name);
            refused.push(name.to_string());
        }
    }
    let _ = REFUSED.set(refused);
}

/// Says in the log what `close` refused; once logging is running.
pub fn report() {
    if ALLOWED {
        tracing::warn!("test build: a debugging port asked for in the environment stays open");
        return;
    }
    if let Some(refused) = REFUSED.get().filter(|refused| !refused.is_empty()) {
        tracing::warn!(
            refused = %refused.join(" "),
            "debugging port refused: this build never opens one"
        );
    }
}

/// The browser arguments a test build hands its window, when the
/// environment asks for a debugging port; `None` otherwise.
///
/// The environment alone is not enough: Tauri gives WebView2 browser
/// arguments of its own, and the WebView2 on CI's Windows runner (154)
/// then never saw the port asked for in `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`.
/// So the test build passes them the way Tauri does, with the arguments
/// wry would otherwise pass by itself. It does not exist in any other
/// build.
#[cfg(feature = "test-harness")]
pub fn test_browser_args() -> Option<String> {
    let asked = std::env::var(WEBVIEW2_ARGUMENTS).ok()?;
    let (_, debugging) = without_debugging(&asked);
    if debugging.is_empty() {
        return None;
    }
    Some(format!("{WRY_DEFAULTS} {}", asked.trim()))
}

/// What wry passes when nothing else is given (wry `webview2/mod.rs`).
#[cfg(feature = "test-harness")]
const WRY_DEFAULTS: &str = concat!(
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
    " --autoplay-policy=no-user-gesture-required"
);

/// The variable WebView2 reads its extra browser arguments from.
const WEBVIEW2_ARGUMENTS: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";

/// The variables that start WebKitGTK's remote inspector.
const WEBKIT_INSPECTOR: [&str; 2] = ["WEBKIT_INSPECTOR_SERVER", "WEBKIT_INSPECTOR_HTTP_SERVER"];

/// Arguments that open the page to something outside it.
const DEBUGGING: [&str; 3] = [
    "--remote-debugging-port",
    "--remote-debugging-pipe",
    "--remote-debugging-address",
];

/// The arguments with every debugging one taken out: what is kept, joined
/// again, and what was taken.
fn without_debugging(value: &str) -> (String, Vec<String>) {
    let (dropped, kept): (Vec<&str>, Vec<&str>) = value.split_whitespace().partition(|argument| {
        DEBUGGING
            .iter()
            .any(|name| argument.eq_ignore_ascii_case(name) || has_value(argument, name))
    });
    (
        kept.join(" "),
        dropped.into_iter().map(str::to_string).collect(),
    )
}

/// `--name=value`, in any case.
fn has_value(argument: &str, name: &str) -> bool {
    argument.len() > name.len()
        && argument.as_bytes()[name.len()] == b'='
        && argument[..name.len()].eq_ignore_ascii_case(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_debugging_port_is_taken_out_and_nothing_else() {
        let (kept, dropped) =
            without_debugging("--disable-gpu --remote-debugging-port=9241 --lang=id");
        assert_eq!(kept, "--disable-gpu --lang=id");
        assert_eq!(dropped, ["--remote-debugging-port=9241"]);
    }

    #[test]
    fn every_way_of_asking_is_caught() {
        for asked in [
            "--remote-debugging-port=1",
            "--REMOTE-DEBUGGING-PORT=1",
            "--remote-debugging-pipe",
            "--remote-debugging-address=0.0.0.0",
        ] {
            let (kept, dropped) = without_debugging(asked);
            assert!(kept.is_empty(), "{asked} was kept");
            assert_eq!(dropped.len(), 1, "{asked}");
        }
    }

    /// The other place a port could be asked for: the window's own browser
    /// arguments in `tauri.conf.json`, which every build carries.
    #[test]
    fn the_configuration_asks_for_no_debugging_port() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid JSON");
        let windows = config["app"]["windows"].as_array().expect("windows");
        assert!(!windows.is_empty());
        for window in windows {
            let args = window["additionalBrowserArgs"].as_str().unwrap_or("");
            assert!(
                without_debugging(args).1.is_empty(),
                "tauri.conf.json asks for a debugging port: {args}"
            );
        }
        assert!(
            config["app"]["windows"][0]["devtools"].as_bool() != Some(true),
            "devtools are not turned on for every build"
        );
    }

    #[test]
    fn a_similar_name_is_not_mistaken_for_one() {
        let (kept, dropped) = without_debugging("--remote-debugging-portal=1 --disable-gpu");
        assert_eq!(kept, "--remote-debugging-portal=1 --disable-gpu");
        assert!(dropped.is_empty());
    }
}
