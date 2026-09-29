//! Putting an outside program where Onsa can run it (SPEC §7.1).
//!
//! This module does not speak to the network: it says **what** should be
//! fetched and from where, checks what came back against the checksum the
//! release published, and puts it in place. `src-tauri` does the fetching
//! with the one HTTP client the whole project shares (SPEC §1), the same way
//! it fetches a cover and hands the bytes to the library.
//!
//! Nothing is installed that did not match its checksum. A file that does
//! not match is not written anywhere, not even to be looked at: a binary
//! Onsa cannot vouch for is one it has no business keeping.
//!
//! Some releases are a single file (yt-dlp) and some are archives (Deno,
//! ffmpeg, fpcalc). Both go through the same door: the bytes are checked
//! against something Onsa did not download, and only then is anything
//! written. Unpacking itself, and the rules that make it safe, are in
//! [`crate::archive`].

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::archive::{self, Kind};
use crate::programs::Program;
use crate::{Error, Result};

/// How Onsa satisfies itself that what arrived is what it asked for.
///
/// There is no fourth arm, and there is deliberately no arm for "do not
/// check": a file that was downloaded is never installed without being
/// matched against something already inside Onsa or published by the
/// release itself (SPEC §7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verify {
    /// The release publishes a list that names its files; ours is in it.
    Listed {
        /// Where the list is.
        sums_url: &'static str,
        /// The name the list calls this file.
        listed_as: &'static str,
    },
    /// The release publishes a file holding the checksum of this one file
    /// and nothing else. Deno does this, in two different shapes depending
    /// on which machine built it.
    Alone {
        /// Where that file is.
        sums_url: &'static str,
    },
    /// The release publishes nothing to check against, so Onsa carries the
    /// fingerprint of a version somebody looked at. That pins the version:
    /// the URL names it, and a different file will not match.
    Pinned {
        /// The SHA-256 of the file at that exact URL.
        sha256: &'static str,
    },
}

/// Where one program comes from, and roughly what it weighs.
///
/// Every field is fixed in the code. Nothing about a release is ever taken
/// from something Onsa downloaded: the URL, the file it expects, and the
/// way it is checked are all decided here (SPEC §7.1, §14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// Which program this release is fetched for.
    pub program: Program,
    /// Where the file itself is.
    pub url: &'static str,
    /// What the release calls the file. Its ending also says whether this
    /// is an archive, and which kind.
    pub file_name: &'static str,
    /// Roughly how large it is, for the list shown before anything starts.
    pub about_bytes: u64,
    /// How the bytes are checked before anything is written.
    pub verify: Verify,
    /// The programs to take out of it. One release can carry two: ffmpeg
    /// and ffprobe arrive together and are never useful apart.
    pub holds: &'static [Program],
}

impl Release {
    /// Which kind of archive this is, or nothing when it is one plain file.
    pub fn archive(&self) -> Option<Kind> {
        Kind::of(self.file_name)
    }
}

/// Most a fetched file may weigh before Onsa stops reading it.
///
/// The ffmpeg builds are the large ones, around two hundred megabytes, and
/// they grow. This is room for that without being room for a mistake.
pub const MAX_BINARY: u64 = 400 * 1024 * 1024;

/// The releases Onsa can install on this system.
///
/// Per platform, because the file differs; everything else about it does
/// not. The order is the order the list is shown in.
pub fn catalogue() -> Vec<Release> {
    vec![yt_dlp(), ffmpeg(), deno(), fpcalc()]
}

/// yt-dlp's official release, which is one file and a published list of
/// checksums beside it (SPEC §7.1).
#[cfg(windows)]
fn yt_dlp() -> Release {
    Release {
        program: Program::YtDlp,
        url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        file_name: "yt-dlp.exe",
        about_bytes: 18 * 1024 * 1024,
        verify: Verify::Listed {
            sums_url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS",
            listed_as: "yt-dlp.exe",
        },
        holds: &[Program::YtDlp],
    }
}

#[cfg(not(windows))]
fn yt_dlp() -> Release {
    Release {
        program: Program::YtDlp,
        url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux",
        file_name: "yt-dlp_linux",
        about_bytes: 30 * 1024 * 1024,
        verify: Verify::Listed {
            sums_url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS",
            listed_as: "yt-dlp_linux",
        },
        holds: &[Program::YtDlp],
    }
}

/// Deno's official release: a zip, with a file beside it holding that zip's
/// checksum and nothing else (SPEC §7.1).
///
/// The two machines that build it write that file differently — Linux with
/// `sha256sum`, Windows with PowerShell's `Get-FileHash` — so what is read
/// out of it is the one checksum in it, whatever shape it came in.
#[cfg(windows)]
fn deno() -> Release {
    Release {
        program: Program::Deno,
        url: "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip",
        file_name: "deno-x86_64-pc-windows-msvc.zip",
        about_bytes: 43 * 1024 * 1024,
        verify: Verify::Alone {
            sums_url: "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip.sha256sum",
        },
        holds: &[Program::Deno],
    }
}

#[cfg(not(windows))]
fn deno() -> Release {
    Release {
        program: Program::Deno,
        url: "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-unknown-linux-gnu.zip",
        file_name: "deno-x86_64-unknown-linux-gnu.zip",
        about_bytes: 42 * 1024 * 1024,
        verify: Verify::Alone {
            sums_url: "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-unknown-linux-gnu.zip.sha256sum",
        },
        holds: &[Program::Deno],
    }
}

/// The build of ffmpeg named in SPEC §7.1, pinned to one dated release.
///
/// BtbN publishes a moving `latest` with no checksums beside it, and a
/// dated release every day that does publish them. The dated one is what
/// Onsa fetches: a moving file cannot be checked against anything, and a
/// checksum written into Onsa for a file that changes daily would be wrong
/// by tomorrow. The cost is that ffmpeg moves when Onsa moves, which is the
/// right way round for something Onsa vouches for.
#[cfg(windows)]
fn ffmpeg() -> Release {
    Release {
        program: Program::Ffmpeg,
        url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/ffmpeg-N-126947-g45f3fecca9-win64-gpl.zip",
        file_name: "ffmpeg-N-126947-g45f3fecca9-win64-gpl.zip",
        about_bytes: 196 * 1024 * 1024,
        verify: Verify::Listed {
            sums_url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/checksums.sha256",
            listed_as: "ffmpeg-N-126947-g45f3fecca9-win64-gpl.zip",
        },
        holds: &[Program::Ffmpeg, Program::Ffprobe],
    }
}

#[cfg(not(windows))]
fn ffmpeg() -> Release {
    Release {
        program: Program::Ffmpeg,
        url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/ffmpeg-N-126947-g45f3fecca9-linux64-gpl.tar.xz",
        file_name: "ffmpeg-N-126947-g45f3fecca9-linux64-gpl.tar.xz",
        about_bytes: 153 * 1024 * 1024,
        verify: Verify::Listed {
            sums_url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/checksums.sha256",
            listed_as: "ffmpeg-N-126947-g45f3fecca9-linux64-gpl.tar.xz",
        },
        holds: &[Program::Ffmpeg, Program::Ffprobe],
    }
}

/// fpcalc from Chromaprint's official release (SPEC §7.1, §8).
///
/// Chromaprint publishes no checksums at all, so this is the one release
/// Onsa carries the fingerprint for itself. Both were downloaded, hashed,
/// and their sizes checked against what the release lists.
#[cfg(windows)]
fn fpcalc() -> Release {
    Release {
        program: Program::Fpcalc,
        url: "https://github.com/acoustid/chromaprint/releases/download/v1.6.1/chromaprint-fpcalc-1.6.1-windows-x86_64.zip",
        file_name: "chromaprint-fpcalc-1.6.1-windows-x86_64.zip",
        about_bytes: 1_816_911,
        verify: Verify::Pinned {
            sha256: "735d6182b38e9f364b84ce6f4ccd682c75e2851de89735711d6b762d12b92a4e",
        },
        holds: &[Program::Fpcalc],
    }
}

#[cfg(not(windows))]
fn fpcalc() -> Release {
    Release {
        program: Program::Fpcalc,
        url: "https://github.com/acoustid/chromaprint/releases/download/v1.6.1/chromaprint-fpcalc-1.6.1-linux-x86_64.tar.gz",
        file_name: "chromaprint-fpcalc-1.6.1-linux-x86_64.tar.gz",
        about_bytes: 2_396_444,
        verify: Verify::Pinned {
            sha256: "fc16cd37a70168040bc9ceb45f1d4d1216f5a75bc4c9cf8564bea70ac6a45733",
        },
        holds: &[Program::Fpcalc],
    }
}

/// The release that brings one program, if Onsa can install it here.
///
/// ffprobe has no release of its own: it comes out of ffmpeg's.
pub fn release_for(program: Program) -> Option<Release> {
    catalogue()
        .into_iter()
        .find(|one| one.holds.contains(&program))
}

/// The page a person is sent to when they fetch a program themselves.
///
/// The same sources as the catalogue above (SPEC §7.1), as pages rather
/// than as files — including the ones Onsa cannot install for itself yet,
/// which is exactly when somebody has to go and get it.
///
/// Fixed here, like every other URL in this module. Nothing about where a
/// program comes from is ever taken from what somebody typed or from what a
/// service answered (SPEC §14).
pub fn release_page(program: Program) -> &'static str {
    match program {
        Program::Fpcalc => "https://github.com/acoustid/chromaprint/releases",
        Program::YtDlp => "https://github.com/yt-dlp/yt-dlp/releases",
        Program::Ffmpeg | Program::Ffprobe => "https://github.com/BtbN/FFmpeg-Builds/releases",
        Program::Deno => "https://github.com/denoland/deno/releases",
    }
}

/// What the program is called in Debian and Ubuntu's own packages.
///
/// Only useful where a package manager is how software arrives, so it is
/// only answered there. Onsa already looks along `PATH`, which means the
/// shortest way to give it the program on those systems is to install the
/// distribution's package and nothing else.
#[cfg(windows)]
pub fn system_package(_program: Program) -> Option<&'static str> {
    None
}

/// What the program is called in Debian and Ubuntu's own packages.
///
/// Only useful where a package manager is how software arrives, so it is
/// only answered there. Onsa already looks along `PATH`, which means the
/// shortest way to give it the program on those systems is to install the
/// distribution's package and nothing else.
#[cfg(not(windows))]
pub fn system_package(program: Program) -> Option<&'static str> {
    match program {
        Program::Fpcalc => Some("libchromaprint-tools"),
        Program::Ffmpeg | Program::Ffprobe => Some("ffmpeg"),
        Program::YtDlp => Some("yt-dlp"),
        // Deno is not in Debian or Ubuntu; saying a package name that is
        // not there would be worse than saying nothing.
        Program::Deno => None,
    }
}

/// The SHA-256 of some bytes, as lower-case hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The checksum a published list gives for one file name.
///
/// The format is the one `sha256sum` writes and every release uses: the
/// checksum, some spaces, the file name. Lines about other files are
/// skipped, and so is anything that is not shaped like a line at all.
pub fn checksum_for(list: &str, file_name: &str) -> Option<String> {
    for line in list.lines() {
        let mut parts = line.split_whitespace();
        let (Some(sum), Some(name)) = (parts.next(), parts.next()) else {
            continue;
        };
        // Some lists mark binary files with a star before the name.
        let name = name.trim_start_matches('*');
        if name != file_name {
            continue;
        }
        let sum = sum.trim().to_lowercase();
        if sum.len() == 64 && sum.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Some(sum);
        }
    }
    None
}

/// The one checksum in a file that holds a checksum for one file.
///
/// Deno publishes such a file beside each release, and the two machines
/// that build it write it differently: Linux with `sha256sum`, so the file
/// is `<hex>  <name>`; Windows with PowerShell's `Get-FileHash`, so it is
/// three labelled lines and the name is a path on the build machine. What
/// both have is exactly one thing that is shaped like a SHA-256, so that is
/// what is looked for — and "exactly one" is the point. A file with two
/// would be a file about two things, and this is not the reader for it.
pub fn only_checksum(text: &str) -> Option<String> {
    let mut found: Option<String> = None;
    for word in text.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if word.len() != 64 || !word.chars().all(|ch| ch.is_ascii_hexdigit()) {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(word.to_lowercase());
    }
    found
}

/// The checksum for a release, out of whatever the release publishes.
///
/// `published` is the text fetched from [`Verify::Listed`] or
/// [`Verify::Alone`], and is not read at all for [`Verify::Pinned`].
pub fn expected_checksum(release: &Release, published: &str) -> Option<String> {
    match &release.verify {
        Verify::Listed { listed_as, .. } => checksum_for(published, listed_as),
        Verify::Alone { .. } => only_checksum(published),
        Verify::Pinned { sha256 } => Some(sha256.to_lowercase()),
    }
}

/// Where the checksum is fetched from, when it has to be fetched at all.
pub fn checksum_url(release: &Release) -> Option<&'static str> {
    match &release.verify {
        Verify::Listed { sums_url, .. } | Verify::Alone { sums_url } => Some(sums_url),
        Verify::Pinned { .. } => None,
    }
}

/// Installs whatever a release is: one file, or the programs inside an
/// archive.
///
/// The checksum is checked first in both cases, before a byte is written or
/// an archive is opened. Answers with what was put in place.
pub fn install_release(
    bin_dir: &Path,
    release: &Release,
    bytes: &[u8],
    expected: &str,
) -> Result<Vec<PathBuf>> {
    match release.archive() {
        None => Ok(vec![install(bin_dir, release.program, bytes, expected)?]),
        Some(kind) => install_archive(bin_dir, release, bytes, expected, kind),
    }
}

/// Takes the programs a release carries out of its archive.
///
/// In order: the bytes are checked, then unpacked into a folder of their
/// own that did not exist a moment ago, then the programs that were asked
/// for are moved into place, then the folder goes. Nothing is moved until
/// every one of them has been found, so a release missing half of what it
/// should hold leaves the previous versions alone.
fn install_archive(
    bin_dir: &Path,
    release: &Release,
    bytes: &[u8],
    expected: &str,
    kind: Kind,
) -> Result<Vec<PathBuf>> {
    let found = sha256_hex(bytes);
    if !found.eq_ignore_ascii_case(expected.trim()) {
        return Err(Error::ChecksumMismatch(release.program.key().to_string()));
    }

    std::fs::create_dir_all(bin_dir)?;
    let unpacked_into = bin_dir.join(format!(".unpacking-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&unpacked_into);
    // Only what this release is fetched for. An ffmpeg build also carries
    // ffplay, which Onsa never runs.
    let wanted: Vec<String> = release
        .holds
        .iter()
        .map(|program| program.file_name())
        .collect();
    let written = archive::unpack_only(bytes, kind, &unpacked_into, &wanted)?;

    let tidy = |outcome: Result<Vec<PathBuf>>| {
        let _ = std::fs::remove_dir_all(&unpacked_into);
        outcome
    };

    // Every one of them is found before any of them is moved.
    let mut taking = Vec::new();
    for program in release.holds {
        let wanted = program.file_name();
        match archive::find_named(&written, &wanted) {
            Some(path) => taking.push((*program, path.clone())),
            None => {
                return tidy(Err(Error::BadArchive(format!(
                    "{wanted} is not in {}",
                    release.file_name
                ))))
            }
        }
    }

    let mut installed = Vec::new();
    for (program, path) in taking {
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => return tidy(Err(Error::Process(error))),
        };
        let sum = sha256_hex(&bytes);
        match install(bin_dir, program, &bytes, &sum) {
            Ok(target) => installed.push(target),
            Err(error) => return tidy(Err(error)),
        }
    }
    tidy(Ok(installed))
}

/// Puts a fetched program in place, but only if it is what it should be.
///
/// The checksum is compared first. Then the bytes are written beside where
/// they are going, flushed, made runnable on systems that have such a
/// notion, and renamed into place — so a program that is half-written is
/// never a program Onsa can find.
pub fn install(bin_dir: &Path, program: Program, bytes: &[u8], expected: &str) -> Result<PathBuf> {
    let found = sha256_hex(bytes);
    if !found.eq_ignore_ascii_case(expected.trim()) {
        return Err(Error::ChecksumMismatch(program.key().to_string()));
    }

    std::fs::create_dir_all(bin_dir)?;
    let target = bin_dir.join(program.file_name());
    let partial = bin_dir.join(format!("{}.part", program.file_name()));

    let write = || -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(&partial)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        runnable(&partial)?;
        // Windows refuses to rename onto a file that is running; the old one
        // is moved aside first so an update never lands half done.
        if target.exists() {
            let old = bin_dir.join(format!("{}.old", program.file_name()));
            let _ = std::fs::remove_file(&old);
            std::fs::rename(&target, &old)?;
            let _ = std::fs::remove_file(&old);
        }
        std::fs::rename(&partial, &target)
    };

    if let Err(error) = write() {
        let _ = std::fs::remove_file(&partial);
        return Err(Error::Process(error));
    }
    Ok(target)
}

/// Marks a file as something the system may run.
#[cfg(unix)]
fn runnable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut mode = std::fs::metadata(path)?.permissions();
    mode.set_mode(0o755);
    std::fs::set_permissions(path, mode)
}

/// Windows decides by the file's extension, so there is nothing to set.
#[cfg(not(unix))]
fn runnable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("onsa install {} {name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// What a published list of checksums actually looks like.
    const SUMS: &str = "\
0000000000000000000000000000000000000000000000000000000000000000  yt-dlp
1111111111111111111111111111111111111111111111111111111111111111  yt-dlp.exe
2222222222222222222222222222222222222222222222222222222222222222 *yt-dlp_linux
not a checksum line at all
";

    #[test]
    fn a_checksum_is_read_for_the_file_it_belongs_to() {
        assert_eq!(
            checksum_for(SUMS, "yt-dlp.exe").as_deref(),
            Some("1111111111111111111111111111111111111111111111111111111111111111")
        );
        assert_eq!(
            checksum_for(SUMS, "yt-dlp_linux").as_deref(),
            Some("2222222222222222222222222222222222222222222222222222222222222222"),
            "a star before the name is not part of the name"
        );
        assert_eq!(checksum_for(SUMS, "ffmpeg"), None);
        assert_eq!(checksum_for("", "yt-dlp.exe"), None);
    }

    #[test]
    fn a_line_that_is_not_a_checksum_is_not_read_as_one() {
        let nonsense = "deadbeef  yt-dlp.exe\nGGGG...GGGG  yt-dlp.exe\n";
        assert_eq!(
            checksum_for(nonsense, "yt-dlp.exe"),
            None,
            "too short, and not hex"
        );
    }

    #[test]
    fn what_does_not_match_its_checksum_is_not_written_anywhere() {
        let dir = temp_dir("mismatch");
        let outcome = install(
            &dir,
            Program::YtDlp,
            b"not what was published",
            &"0".repeat(64),
        );
        assert!(
            matches!(outcome, Err(Error::ChecksumMismatch(_))),
            "{outcome:?}"
        );
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            0,
            "not even beside where it was going"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn what_matches_is_put_where_onsa_looks_for_it() {
        let dir = temp_dir("install");
        let bytes = b"a program, more or less";
        let target = install(&dir, Program::YtDlp, bytes, &sha256_hex(bytes)).expect("installed");
        assert_eq!(target, dir.join(Program::YtDlp.file_name()));
        assert_eq!(std::fs::read(&target).unwrap(), bytes);

        // And the manager finds it there without being told where.
        let programs = crate::Programs::new(&dir).use_system(false);
        let found = programs.find(Program::YtDlp).expect("it is installed");
        assert_eq!(found.path, target);
        assert_eq!(found.from, crate::Where::Managed);

        // Nothing half-written is left behind.
        let leftovers: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".part") || name.ends_with(".old"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn installing_again_replaces_what_was_there() {
        let dir = temp_dir("again");
        let first = b"version one";
        install(&dir, Program::YtDlp, first, &sha256_hex(first)).expect("installed");
        let second = b"version two, which is longer";
        let target = install(&dir, Program::YtDlp, second, &sha256_hex(second)).expect("replaced");
        assert_eq!(std::fs::read(&target).unwrap(), second);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn what_is_installed_is_something_the_system_may_run() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("runnable");
        let bytes = b"a program";
        let target = install(&dir, Program::YtDlp, bytes, &sha256_hex(bytes)).expect("installed");
        let mode = std::fs::metadata(&target).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "{mode:o}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_release_says_where_it_comes_from_and_how_it_is_checked() {
        for release in catalogue() {
            assert!(
                release.url.starts_with("https://github.com/"),
                "{}: releases come from GitHub and nowhere else (SPEC §7.1)",
                release.program.key()
            );
            assert!(
                release.url.ends_with(release.file_name),
                "{}: the URL ends in the file it says it fetches",
                release.program.key()
            );
            assert!(release.about_bytes > 0);
            assert!(
                !release.holds.is_empty(),
                "{}: a release brings at least one program",
                release.program.key()
            );
            match &release.verify {
                Verify::Listed {
                    sums_url,
                    listed_as,
                } => {
                    assert!(sums_url.starts_with("https://github.com/"));
                    assert!(!listed_as.is_empty());
                }
                Verify::Alone { sums_url } => {
                    assert!(sums_url.starts_with("https://github.com/"));
                }
                Verify::Pinned { sha256 } => {
                    assert_eq!(sha256.len(), 64, "{}", release.program.key());
                    assert!(sha256.chars().all(|ch| ch.is_ascii_hexdigit()));
                    assert!(
                        !release.url.contains("/latest/"),
                        "{}: a fingerprint in the code pins the version, so the URL must too",
                        release.program.key()
                    );
                }
            }
        }
    }

    #[test]
    fn every_program_onsa_runs_can_now_be_fetched() {
        // Including ffprobe, which has no release of its own and comes out
        // of ffmpeg's.
        for program in Program::ALL {
            let release = release_for(program)
                .unwrap_or_else(|| panic!("{} has nowhere to come from", program.key()));
            assert!(release.holds.contains(&program));
        }
        assert_eq!(
            release_for(Program::Ffprobe).map(|one| one.program),
            Some(Program::Ffmpeg)
        );
    }

    #[test]
    fn an_archive_is_known_by_what_the_release_calls_it() {
        for release in catalogue() {
            let is_archive = release.archive().is_some();
            let one_file = release.holds.len() == 1 && release.program == Program::YtDlp;
            assert!(
                is_archive || one_file,
                "{}: everything but yt-dlp arrives as an archive",
                release.program.key()
            );
        }
    }

    /// The two shapes the one-file checksum arrives in, both real.
    #[test]
    fn the_one_checksum_in_a_file_is_read_whichever_way_it_was_written() {
        let unix = "c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490  deno.zip\n";
        assert_eq!(
            only_checksum(unix).as_deref(),
            Some("c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490")
        );
        // What PowerShell's Get-FileHash writes, including the build
        // machine's own path, which is not a name Onsa could match on.
        let windows = "\nAlgorithm : SHA256\nHash      : A0C3101B4158D1DFB7D6A78A7BF0F3DE80C96BB423C152BEEC8BEB22786F2238\nPath      : C:\\a\\deno\\deno\\target\\release\\deno.zip\n";
        assert_eq!(
            only_checksum(windows).as_deref(),
            Some("a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238")
        );
    }

    #[test]
    fn a_file_about_two_things_is_not_read_as_being_about_one() {
        let two = "0000000000000000000000000000000000000000000000000000000000000000  a\n1111111111111111111111111111111111111111111111111111111111111111  b\n";
        assert_eq!(only_checksum(two), None);
        assert_eq!(only_checksum(""), None);
        assert_eq!(only_checksum("nothing here"), None);
    }

    #[test]
    fn every_program_has_a_page_a_person_can_be_sent_to() {
        // Including the ones Onsa cannot install for itself, which is
        // exactly when somebody has to go and fetch one by hand.
        for program in Program::ALL {
            let page = release_page(program);
            assert!(
                page.starts_with("https://github.com/"),
                "{}: {page}",
                program.key()
            );
            assert!(!page.contains("/download/"), "a page, not a file: {page}");
        }
        // The page and the file Onsa fetches are the same project.
        for release in catalogue() {
            let page = release_page(release.program);
            let owner_and_repo =
                |url: &str| url.split('/').skip(3).take(2).collect::<Vec<_>>().join("/");
            assert_eq!(owner_and_repo(page), owner_and_repo(release.url));
        }
    }

    #[test]
    fn a_package_name_is_only_given_where_packages_are_how_software_arrives() {
        // fpcalc is the one worth saying out loud: Onsa looks along PATH,
        // so installing the distribution's package is the whole job there.
        if cfg!(windows) {
            assert_eq!(system_package(Program::Fpcalc), None);
        } else {
            assert_eq!(
                system_package(Program::Fpcalc),
                Some("libchromaprint-tools")
            );
            assert_eq!(system_package(Program::Deno), None, "not in Debian");
        }
    }
}
