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
//! Only programs that are published as a **single file** are handled here.
//! yt-dlp is one. Deno and ffmpeg come as archives, and the unpacking they
//! need — with the dependencies it would add — waits for the rest of M10.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::programs::Program;
use crate::{Error, Result};

/// Where one program comes from, and roughly what it weighs.
///
/// Every field is fixed in the code. Nothing about a release is ever taken
/// from something Onsa downloaded: the URL, the file it expects, and the
/// list it checks against are all decided here (SPEC §7.1, §14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// Which program this installs.
    pub program: Program,
    /// Where the file itself is.
    pub url: &'static str,
    /// Where the published list of checksums is.
    pub sums_url: &'static str,
    /// The name the checksum list calls this file.
    pub listed_as: &'static str,
    /// Roughly how large it is, for the list shown before anything starts.
    pub about_bytes: u64,
}

/// Most a fetched binary may weigh before Onsa stops reading it.
///
/// yt-dlp is tens of megabytes; this is room for the ones that come later
/// without being room for a mistake.
pub const MAX_BINARY: u64 = 200 * 1024 * 1024;

/// The releases Onsa can install on this system today.
///
/// Only single-file releases (see the module note). The list is per platform
/// because the file differs; everything else about it does not.
pub fn catalogue() -> Vec<Release> {
    vec![yt_dlp()]
}

/// yt-dlp's official release, which is one file and a published list of
/// checksums beside it (SPEC §7.1).
#[cfg(windows)]
fn yt_dlp() -> Release {
    Release {
        program: Program::YtDlp,
        url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        sums_url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS",
        listed_as: "yt-dlp.exe",
        about_bytes: 18 * 1024 * 1024,
    }
}

#[cfg(not(windows))]
fn yt_dlp() -> Release {
    Release {
        program: Program::YtDlp,
        url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux",
        sums_url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS",
        listed_as: "yt-dlp_linux",
        about_bytes: 30 * 1024 * 1024,
    }
}

/// The release for one program, if Onsa can install it on this system.
pub fn release_for(program: Program) -> Option<Release> {
    catalogue().into_iter().find(|one| one.program == program)
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
    fn every_release_says_where_it_comes_from_and_where_to_check_it() {
        for release in catalogue() {
            assert!(
                release.url.starts_with("https://github.com/"),
                "{}: releases come from GitHub and nowhere else (SPEC §7.1)",
                release.program.key()
            );
            assert!(release.sums_url.starts_with("https://github.com/"));
            assert!(!release.listed_as.is_empty());
            assert!(release.about_bytes > 0);
        }
        assert_eq!(
            release_for(Program::YtDlp).map(|one| one.program),
            Some(Program::YtDlp)
        );
        // The ones that come as archives are not offered yet.
        assert_eq!(release_for(Program::Ffmpeg), None);
        assert_eq!(release_for(Program::Deno), None);
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
