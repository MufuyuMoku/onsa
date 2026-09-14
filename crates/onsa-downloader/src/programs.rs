//! The one place Onsa runs an outside program (SPEC §7.3).
//!
//! Four programs are involved across the whole application: `fpcalc` reads a
//! fingerprint out of a track for AcoustID (SPEC §8), and `yt-dlp`, `ffmpeg`
//! and `deno` belong to the downloader (SPEC §7.1). They are found, checked
//! and run the same way, by this module, once — the downloader in M10 asks
//! the same questions of the same code rather than growing its own answer.
//!
//! The rules that make this safe are not optional and are not spread out:
//!
//! * a program is run with an **argument array and never through a shell**,
//!   so nothing in a file name, a URL or a tag can become a command;
//! * on Windows it runs with `CREATE_NO_WINDOW`, so no console flashes up;
//! * a run has a deadline, and a program that goes past it is killed rather
//!   than waited for forever;
//! * what it prints is read as it goes and capped, so a program that will
//!   not stop talking cannot fill memory;
//! * the caller gets a value back either way. Nothing here panics, and a
//!   missing program is an ordinary answer, not a failure of the
//!   application.
//!
//! Where a program comes from is the listener's choice (SPEC §7.1): the
//! folder Onsa manages, a path they picked themselves, or the system's own
//! `PATH`, which is how most Linux machines already have these.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::platform;
use crate::{Error, Result};

/// Most output one run may produce before Onsa stops reading it. The answers
/// these programs give are measured in kilobytes.
const MAX_OUTPUT: u64 = 4 * 1024 * 1024;

/// How often a running program is looked in on while waiting for it.
const POLL: Duration = Duration::from_millis(15);

/// How long asking a program its version may take.
const VERSION_TIMEOUT: Duration = Duration::from_secs(10);

/// An outside program Onsa knows how to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Program {
    /// Chromaprint's fingerprinter, which AcoustID needs (SPEC §8).
    Fpcalc,
    /// The downloader itself (SPEC §7.1, M10).
    YtDlp,
    /// Converting and muxing for the downloader (SPEC §7.1, M10).
    Ffmpeg,
    /// Reading what a downloaded file turned out to be (SPEC §7.1, M10).
    Ffprobe,
    /// The JavaScript runtime yt-dlp wants for YouTube (SPEC §7.1, M10).
    Deno,
}

impl Program {
    /// Every program, for listing what is and is not installed.
    pub const ALL: [Program; 5] = [
        Program::Fpcalc,
        Program::YtDlp,
        Program::Ffmpeg,
        Program::Ffprobe,
        Program::Deno,
    ];

    /// The name it is known by, in settings and in the interface.
    pub fn key(self) -> &'static str {
        match self {
            Self::Fpcalc => "fpcalc",
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
            Self::Ffprobe => "ffprobe",
            Self::Deno => "deno",
        }
    }

    /// The program a name stands for, or nothing if it stands for none.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|program| program.key() == key)
    }

    /// The file it is called on this system.
    pub fn file_name(self) -> String {
        platform::executable(self.key())
    }

    /// What to pass to make it say its version.
    fn version_args(self) -> &'static [&'static str] {
        match self {
            Self::Fpcalc => &["-version"],
            Self::YtDlp | Self::Deno => &["--version"],
            Self::Ffmpeg | Self::Ffprobe => &["-version"],
        }
    }
}

/// Where a program was found, which is what the interface reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// In the folder Onsa manages for them.
    Managed,
    /// At a path the listener chose.
    Chosen,
    /// On the system's own `PATH`.
    System,
}

/// A program that is actually there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Which one.
    pub program: Program,
    /// The file to run.
    pub path: PathBuf,
    /// Where it came from.
    pub from: Where,
}

/// What came of running a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// Its exit code, when it had one. `None` means a signal ended it.
    pub code: Option<i32>,
    /// Whether it said it succeeded.
    pub ok: bool,
    /// What it wrote to standard output, as text.
    pub out: String,
    /// What it wrote to standard error, as text.
    pub err: String,
}

impl Ran {
    /// The output, or the program's own complaint as an error.
    ///
    /// The complaint is trimmed to its first line: these programs answer a
    /// failure with a sentence and then a page of usage.
    pub fn ok(self, program: Program) -> Result<String> {
        if self.ok {
            return Ok(self.out);
        }
        let said = self
            .err
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("no reason given")
            .to_string();
        Err(Error::ProgramFailed {
            program: program.key().to_string(),
            code: self.code,
            said,
        })
    }
}

/// Where Onsa looks for the programs it runs, and how it runs them.
///
/// Cheap to clone and to hold: it keeps folders and choices, never a running
/// process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Programs {
    /// The folder Onsa keeps its own copies in: `<app_data>/bin` (SPEC §7.1).
    bin_dir: PathBuf,
    /// Whether a copy already on the system may be used (SPEC §7.1).
    use_system: bool,
    /// Paths the listener chose by hand, per program.
    chosen: BTreeMap<Program, PathBuf>,
}

impl Programs {
    /// A manager keeping its programs in `bin_dir`, allowed to fall back to
    /// whatever the system already has.
    pub fn new(bin_dir: impl Into<PathBuf>) -> Self {
        Self {
            bin_dir: bin_dir.into(),
            use_system: true,
            chosen: BTreeMap::new(),
        }
    }

    /// Whether a copy already on the system may be used.
    pub fn use_system(mut self, yes: bool) -> Self {
        self.use_system = yes;
        self
    }

    /// Points one program at a file the listener chose.
    ///
    /// A path that is not a file is not kept: a choice that cannot be run is
    /// worse than no choice, because it hides the copy that would have been
    /// found instead.
    pub fn choose(&mut self, program: Program, path: impl Into<PathBuf>) -> bool {
        let path = path.into();
        if !path.is_file() {
            return false;
        }
        self.chosen.insert(program, path);
        true
    }

    /// Forgets a chosen path, going back to looking for it.
    pub fn unchoose(&mut self, program: Program) {
        self.chosen.remove(&program);
    }

    /// The folder Onsa keeps its own copies in.
    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    /// Finds a program: the listener's choice first, then Onsa's own folder,
    /// then the system's `PATH` if that is allowed.
    ///
    /// Onsa's own copy is preferred over the system's because it is the one
    /// Onsa knows the version of; the listener's own choice beats both,
    /// because they said so.
    pub fn find(&self, program: Program) -> Option<Found> {
        if let Some(path) = self.chosen.get(&program) {
            if path.is_file() {
                return Some(Found {
                    program,
                    path: path.clone(),
                    from: Where::Chosen,
                });
            }
        }
        let managed = self.bin_dir.join(program.file_name());
        if managed.is_file() {
            return Some(Found {
                program,
                path: managed,
                from: Where::Managed,
            });
        }
        if !self.use_system {
            return None;
        }
        on_path(&program.file_name()).map(|path| Found {
            program,
            path,
            from: Where::System,
        })
    }

    /// Whether a program can be run at all.
    pub fn have(&self, program: Program) -> bool {
        self.find(program).is_some()
    }

    /// Runs a program and waits for it, for at most `limit`.
    ///
    /// The arguments are passed as they are: this never goes through a
    /// shell, so a file name with a quote or a semicolon in it is a file
    /// name and nothing else.
    pub fn run<S: AsRef<OsStr>>(
        &self,
        program: Program,
        args: &[S],
        limit: Duration,
    ) -> Result<Ran> {
        let found = self
            .find(program)
            .ok_or_else(|| Error::MissingProgram(program.key().to_string()))?;
        run_at(&found.path, args, limit).map_err(|error| match error {
            Error::Process(io) => Error::CannotRun {
                program: program.key().to_string(),
                said: io.to_string(),
            },
            other => other,
        })
    }

    /// What version a program says it is, for the interface to show.
    ///
    /// Only the first line is kept: these programs answer with a version and
    /// then a paragraph about their build.
    pub fn version(&self, program: Program) -> Option<String> {
        let ran = self
            .run(program, program.version_args(), VERSION_TIMEOUT)
            .ok()?;
        let text = if ran.out.trim().is_empty() {
            ran.err
        } else {
            ran.out
        };
        text.lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string)
    }
}

/// Runs one file with an argument array, waiting at most `limit`.
///
/// Kept apart from [`Programs`] so a caller with a path in hand — a test, or
/// the check that a chosen file really is the program it claims — can use
/// the same waiting, the same cap on output and the same deadline.
pub fn run_at<S: AsRef<OsStr>>(path: &Path, args: &[S], limit: Duration) -> Result<Ran> {
    let mut command = Command::new(path);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    platform::quietly(&mut command);
    let mut child = command.spawn()?;

    // Both pipes are drained on threads of their own. A program that fills
    // one of them while Onsa waits on the other would otherwise stop for
    // good, and neither side would ever say why.
    let out = child.stdout.take().map(drain);
    let err = child.stderr.take().map(drain);

    let deadline = Instant::now() + limit;
    let status = loop {
        match child.try_wait()? {
            Some(status) => break Some(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            None => std::thread::sleep(POLL),
        }
    };

    let out = out.map(join).unwrap_or_default();
    let err = err.map(join).unwrap_or_default();

    let Some(status) = status else {
        return Err(Error::TooSlow {
            program: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string()),
            seconds: limit.as_secs(),
        });
    };
    Ok(Ran {
        code: status.code(),
        ok: status.success(),
        out,
        err,
    })
}

/// Reads a pipe to its end on a thread, never past the cap.
fn drain<R: Read + Send + 'static>(mut pipe: R) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        // One byte past the cap would still be within what is read; the
        // point is only that it stops.
        let _ = (&mut pipe).take(MAX_OUTPUT).read_to_end(&mut buffer);
        buffer
    })
}

/// What a drained pipe said, as text. Output that is not UTF-8 is read as
/// best it can be: this is for showing and for parsing, not for storing.
fn join(handle: std::thread::JoinHandle<Vec<u8>>) -> String {
    handle
        .join()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// The first file of this name on the system's `PATH`.
fn on_path(file_name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|folder| folder.join(file_name))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A program every machine of this kind has, and what to pass it to make
    /// it print something and finish at once.
    fn quick() -> (&'static str, Vec<&'static str>) {
        if cfg!(windows) {
            ("ping", vec!["-n", "1", "127.0.0.1"])
        } else {
            ("echo", vec!["hello"])
        }
    }

    /// The same, but slow enough to run out of time.
    fn slow() -> (&'static str, Vec<&'static str>) {
        if cfg!(windows) {
            ("ping", vec!["-n", "30", "127.0.0.1"])
        } else {
            ("sleep", vec!["30"])
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("onsa programs {} {name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn every_program_has_a_name_that_finds_it_again() {
        for program in Program::ALL {
            assert_eq!(Program::from_key(program.key()), Some(program));
            assert!(program.file_name().starts_with(program.key()));
        }
        assert_eq!(Program::from_key("rm"), None);
    }

    #[test]
    fn a_missing_program_is_an_answer_not_a_failure() {
        let programs = Programs::new(temp_dir("empty")).use_system(false);
        assert!(!programs.have(Program::Fpcalc));
        assert_eq!(programs.find(Program::Fpcalc), None);
        let error = programs
            .run(Program::Fpcalc, &["-json"], Duration::from_secs(1))
            .expect_err("there is none");
        assert!(matches!(error, Error::MissingProgram(name) if name == "fpcalc"));
    }

    #[test]
    fn onsas_own_copy_is_preferred_to_the_systems() {
        let dir = temp_dir("managed");
        let mine = dir.join(Program::Ffmpeg.file_name());
        std::fs::write(&mine, b"not really ffmpeg").unwrap();
        let programs = Programs::new(&dir);
        let found = programs.find(Program::Ffmpeg).expect("the copy in bin");
        assert_eq!(found.path, mine);
        assert_eq!(found.from, Where::Managed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_chosen_path_beats_everything_else() {
        let dir = temp_dir("chosen");
        let managed = dir.join(Program::Fpcalc.file_name());
        std::fs::write(&managed, b"x").unwrap();
        let theirs = dir.join("somewhere else");
        std::fs::write(&theirs, b"x").unwrap();

        let mut programs = Programs::new(&dir);
        assert!(programs.choose(Program::Fpcalc, &theirs));
        let found = programs.find(Program::Fpcalc).expect("the chosen one");
        assert_eq!(found.path, theirs);
        assert_eq!(found.from, Where::Chosen);

        // A path that is not there is not a choice at all.
        assert!(!programs.choose(Program::Fpcalc, dir.join("nothing here")));
        programs.unchoose(Program::Fpcalc);
        assert_eq!(
            programs.find(Program::Fpcalc).map(|found| found.from),
            Some(Where::Managed)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_system_can_be_switched_off_entirely() {
        let (name, _) = quick();
        let dir = temp_dir("system");
        // Borrow a real program's name so there is certainly one on PATH.
        assert!(
            on_path(&platform::executable(name)).is_some(),
            "{name} should be on PATH"
        );
        let allowed = Programs::new(&dir);
        let refused = Programs::new(&dir).use_system(false);
        assert!(allowed.use_system);
        assert!(!refused.use_system);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_program_that_finishes_gives_back_what_it_said() {
        let (name, args) = quick();
        let path = on_path(&platform::executable(name)).expect("a program to run");
        let ran = run_at(&path, &args, Duration::from_secs(20)).expect("it ran");
        assert!(ran.ok, "{ran:?}");
        assert_eq!(ran.code, Some(0));
        assert!(!ran.out.trim().is_empty(), "it printed something");
    }

    #[test]
    fn a_program_that_will_not_stop_is_killed_rather_than_waited_for() {
        let (name, args) = slow();
        let path = on_path(&platform::executable(name)).expect("a program to run");
        let started = Instant::now();
        let error = run_at(&path, &args, Duration::from_millis(300)).expect_err("out of time");
        let taken = started.elapsed();
        assert!(matches!(error, Error::TooSlow { .. }), "{error:?}");
        assert!(taken < Duration::from_secs(10), "it waited {taken:?}");
    }

    #[test]
    fn an_argument_is_an_argument_and_never_a_command() {
        // Whatever a shell would have made of this, the program receives it
        // as one argument, because there is no shell. Both programs repeat
        // an argument they cannot use, so the proof is in the repeating:
        // the whole thing comes back as a single word.
        let (name, _) = quick();
        let path = on_path(&platform::executable(name)).expect("a program to run");
        let marker = "; echo owned & del *.* | \"quoted\"";
        let ran = run_at(&path, &[marker], Duration::from_secs(20)).expect("it ran");
        let said = format!("{}{}", ran.out, ran.err);
        assert!(
            said.contains(marker),
            "it arrived whole, not cut into commands: {said}"
        );
    }

    #[test]
    fn a_failure_carries_the_programs_own_first_line() {
        let ran = Ran {
            code: Some(2),
            ok: false,
            out: String::new(),
            err: "\nERROR: unable to read file\nusage: fpcalc [options]\n".to_string(),
        };
        let error = ran.ok(Program::Fpcalc).expect_err("it failed");
        let said = error.to_string();
        assert!(said.contains("unable to read file"), "{said}");
        assert!(!said.contains("usage"), "only the reason: {said}");
    }
}
