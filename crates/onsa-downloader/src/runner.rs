//! Running a program and reading it as it goes (SPEC §7.2, §7.3).
//!
//! [`crate::programs::Programs::run`] waits for a program and hands back
//! what it said. That is right for `fpcalc`, which answers in a moment. It
//! is wrong for a download, which takes minutes and has something to say the
//! whole time.
//!
//! So this streams instead: every line the program prints goes to the caller
//! as it arrives, and the caller can say "stop" between lines. Stopping ends
//! the program **and everything it started** — yt-dlp starts ffmpeg, and a
//! cancel that leaves ffmpeg running is not a cancel.
//!
//! Nothing here knows what the lines mean. What yt-dlp's progress looks like
//! is `src-tauri`'s business; this only makes sure the lines arrive.

use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::platform::{self, Group};
use crate::{Error, Result};

/// What came of a program that ran to its end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    /// Its exit code, when it had one.
    pub code: Option<i32>,
    /// Whether it said it succeeded.
    pub ok: bool,
    /// The last few lines it wrote to standard error, for the log.
    pub complaints: Vec<String>,
    /// Whether it was stopped rather than allowed to finish.
    pub stopped: bool,
}

/// Most lines of standard error to keep. A program that fails usually says
/// why in the last few; the rest is repetition.
const KEEP_COMPLAINTS: usize = 20;

/// Runs a program, handing every line of its output to `on_line` as it
/// arrives, until it finishes or `stop` is set.
///
/// The arguments are passed as an array and never through a shell, so
/// nothing in a URL can become a command (SPEC §7.3).
pub fn stream<S: AsRef<OsStr>>(
    path: &Path,
    args: &[S],
    stop: Arc<AtomicBool>,
    mut on_line: impl FnMut(&str),
) -> Result<Finished> {
    let mut command = Command::new(path);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    platform::quietly(&mut command);
    let mut child = command.spawn()?;

    // Everything this program starts belongs to the same group, so one stop
    // reaches all of it.
    let group = Group::new();
    if let Some(group) = &group {
        group.take(&child);
    }

    // Standard error is drained on a thread of its own: a program that fills
    // that pipe while Onsa reads the other one would otherwise stop for good.
    let complaints = child.stderr.take().map(|pipe| {
        std::thread::spawn(move || {
            let mut kept: Vec<String> = Vec::new();
            for line in BufReader::new(pipe).lines().map_while(std::io::Result::ok) {
                if kept.len() == KEEP_COMPLAINTS {
                    kept.remove(0);
                }
                kept.push(line);
            }
            kept
        })
    });

    let mut stopped = false;
    if let Some(out) = child.stdout.take() {
        for line in BufReader::new(out).lines() {
            if stop.load(Ordering::SeqCst) {
                stopped = true;
                break;
            }
            match line {
                Ok(line) => on_line(&line),
                // A line that is not text is not worth ending a download for.
                Err(error) => tracing::debug!("a line could not be read: {error}"),
            }
        }
    }

    if stopped || stop.load(Ordering::SeqCst) {
        stopped = true;
        if let Some(group) = &group {
            group.stop();
        }
        let _ = child.kill();
    }

    let status = child.wait()?;
    let complaints = complaints
        .map(|handle| handle.join().unwrap_or_default())
        .unwrap_or_default();

    Ok(Finished {
        code: status.code(),
        ok: status.success() && !stopped,
        complaints,
        stopped,
    })
}

impl Finished {
    /// The program's own reason for failing, as one line.
    pub fn reason(&self) -> String {
        self.complaints
            .iter()
            .rev()
            .map(|line| line.trim())
            .find(|line| !line.is_empty())
            .unwrap_or("no reason given")
            .to_string()
    }

    /// Turns a program that failed into an error that says which one.
    pub fn ok(self, program: &str) -> Result<Self> {
        if self.ok {
            return Ok(self);
        }
        Err(Error::ProgramFailed {
            program: program.to_string(),
            code: self.code,
            said: self.reason(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    /// A program every machine of this kind has, made to print several
    /// lines slowly enough to be stopped part way.
    fn talker() -> (PathBuf, Vec<String>) {
        let (name, args): (&str, Vec<String>) = if cfg!(windows) {
            ("ping", vec!["-n".into(), "20".into(), "127.0.0.1".into()])
        } else {
            ("ping", vec!["-c".into(), "20".into(), "127.0.0.1".into()])
        };
        let path =
            crate::programs::tests_on_path(&platform::executable(name)).expect("a program to run");
        (path, args)
    }

    #[test]
    fn every_line_arrives_as_it_is_printed() {
        let (path, args) = talker();
        let stop = Arc::new(AtomicBool::new(false));
        let mut lines = Vec::new();
        let finished =
            stream(&path, &args, stop, |line| lines.push(line.to_string())).expect("it ran");
        assert!(finished.ok, "{finished:?}");
        assert!(lines.len() > 3, "{} lines", lines.len());
    }

    #[test]
    fn stopping_it_stops_it_now_rather_than_at_the_end() {
        let (path, args) = talker();
        let stop = Arc::new(AtomicBool::new(false));
        let asked = stop.clone();
        // Let a couple of lines through, then ask it to stop.
        let started = Instant::now();
        let mut seen = 0;
        let finished = stream(&path, &args, stop, |_| {
            seen += 1;
            if seen == 2 {
                asked.store(true, Ordering::SeqCst);
            }
        })
        .expect("it ran");
        let took = started.elapsed();
        assert!(finished.stopped, "{finished:?}");
        assert!(!finished.ok, "a stopped program did not succeed");
        assert!(
            took < Duration::from_secs(15),
            "it waited for the whole run: {took:?}"
        );
    }

    #[test]
    fn a_program_that_fails_says_why_in_one_line() {
        let finished = Finished {
            code: Some(1),
            ok: false,
            complaints: vec![
                "WARNING: something minor".into(),
                "".into(),
                "ERROR: unable to download webpage".into(),
            ],
            stopped: false,
        };
        assert_eq!(finished.reason(), "ERROR: unable to download webpage");
        let error = finished.ok("yt-dlp").expect_err("it failed");
        assert!(error.to_string().contains("unable to download webpage"));
    }
}
