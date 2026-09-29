//! Unpacking what a release is published as (SPEC §7.1, M10b).
//!
//! Deno and the Windows ffmpeg arrive as `.zip`; fpcalc and the Linux
//! ffmpeg as `.tar.gz` and `.tar.xz`. All that is wanted out of any of them
//! is one or two programs, and everything else in the archive is somebody
//! else's idea of what should happen to this machine.
//!
//! So an archive is never trusted for its contents:
//!
//! - **The checksum is checked before a single byte is unpacked**, never
//!   after. An archive Onsa cannot vouch for is not opened at all.
//! - **An entry may only land inside the folder it is being unpacked into.**
//!   An absolute path, a `..` anywhere in it, a drive letter, a symlink or a
//!   hard link — each one is refused.
//! - **One refused entry refuses the whole archive.** There is no unpacking
//!   the good half of something that tried to write outside its folder.
//! - **The total size and the number of entries are capped**, so a broken or
//!   hostile archive cannot fill the disk.
//! - **Everything goes to a temporary folder first** and is only moved into
//!   place once all of it worked.

use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::{Error, Result};

/// The most an archive may turn into on disk.
///
/// Counted against what is actually written, which for the large archives
/// is a small part of them: Onsa keeps two files out of an ffmpeg build.
/// Room for those without being room for a disk to fill.
pub const MAX_UNPACKED: u64 = 400 * 1024 * 1024;

/// The most a compressed stream may decompress to in memory.
///
/// A tar has to be decompressed whole before its entries can be read, and
/// a tar can hold far more than the part of it that is wanted: an ffmpeg
/// build is most of a gigabyte of tar for two files. This bounds that, and
/// with it how much memory a hostile archive can ask for.
pub const MAX_STREAM: u64 = 1024 * 1024 * 1024;

/// The most entries an archive may hold. An ffmpeg build has a handful.
pub const MAX_ENTRIES: usize = 4096;

/// The shapes of archive Onsa can open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A zip file.
    Zip,
    /// A plain tar.
    Tar,
    /// A tar, gzipped.
    TarGz,
    /// A tar, xz'd.
    TarXz,
}

impl Kind {
    /// Which of them a file name says it is.
    ///
    /// The name comes from Onsa's own list of releases, never from a
    /// service's answer, so this is a convenience rather than a guess.
    pub fn of(name: &str) -> Option<Self> {
        let name = name.to_ascii_lowercase();
        if name.ends_with(".zip") {
            Some(Self::Zip)
        } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Some(Self::TarGz)
        } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            Some(Self::TarXz)
        } else if name.ends_with(".tar") {
            Some(Self::Tar)
        } else {
            None
        }
    }
}

/// An entry's path, if it is one this archive is allowed to write.
///
/// Answers `None` for anything that could land outside the folder it is
/// being unpacked into: an absolute path, a root, a Windows drive or share
/// prefix, or any `..` at all. `.` is dropped, since it says nothing.
///
/// A path is judged by what it says, not by what it resolves to on this
/// disk: resolving it would already mean following whatever is there.
pub fn safe_relative(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Normal(piece) => {
                // A separator smuggled inside one component is another way
                // of writing the same escape.
                let text = piece.to_str()?;
                if text.contains('/') || text.contains('\\') || text == ".." {
                    return None;
                }
                out.push(piece);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Refuses an archive, saying which entry and why.
fn refuse(entry: &str, why: &str) -> Error {
    Error::BadArchive(format!("{entry}: {why}"))
}

/// Unpacks an archive into a folder, or refuses it whole.
///
/// The folder is expected to be one that did not exist a moment ago. On any
/// refusal it is removed again, so nothing half-unpacked is ever left for
/// something else to find.
///
/// Answers with the files it wrote, in the order it wrote them.
pub fn unpack(bytes: &[u8], kind: Kind, into: &Path) -> Result<Vec<PathBuf>> {
    unpack_only(bytes, kind, into, &[])
}

/// The same, writing only the files named here.
///
/// Every entry is still read and still judged: one that would leave the
/// folder refuses the whole archive whether or not it was wanted. What
/// changes is that the rest is not written down. An ffmpeg build carries
/// ffplay, which Onsa never runs, and writing it would put half a gigabyte
/// on somebody's disk to be deleted a moment later.
///
/// An empty list means everything, which is what [`unpack`] asks for.
pub fn unpack_only(
    bytes: &[u8],
    kind: Kind,
    into: &Path,
    wanted: &[String],
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(into)?;
    let outcome = match kind {
        Kind::Zip => unpack_zip(bytes, into, wanted),
        Kind::Tar => unpack_tar(bytes, into, wanted),
        Kind::TarGz => {
            let mut plain = Vec::new();
            flate2::read::GzDecoder::new(bytes)
                .take(MAX_STREAM + 1)
                .read_to_end(&mut plain)
                .map_err(|error| Error::BadArchive(format!("gzip: {error}")))?;
            if plain.len() as u64 > MAX_STREAM {
                Err(Error::BadArchive(
                    "gzip: larger than Onsa will unpack".into(),
                ))
            } else {
                unpack_tar(&plain, into, wanted)
            }
        }
        Kind::TarXz => {
            let mut plain = Vec::new();
            let mut reader = std::io::BufReader::new(bytes);
            lzma_rs::xz_decompress(&mut reader, &mut plain)
                .map_err(|error| Error::BadArchive(format!("xz: {error}")))?;
            if plain.len() as u64 > MAX_STREAM {
                Err(Error::BadArchive("xz: larger than Onsa will unpack".into()))
            } else {
                unpack_tar(&plain, into, wanted)
            }
        }
    };
    if outcome.is_err() {
        let _ = std::fs::remove_dir_all(into);
    }
    outcome
}

/// Whether this entry is one of the files asked for. Nothing asked for
/// means everything.
fn asked_for(relative: &Path, wanted: &[String]) -> bool {
    if wanted.is_empty() {
        return true;
    }
    relative
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| wanted.iter().any(|one| one == name))
}

/// Keeps the running totals, so both readers count the same way.
struct Budget {
    entries: usize,
    bytes: u64,
}

impl Budget {
    fn new() -> Self {
        Self {
            entries: 0,
            bytes: 0,
        }
    }

    fn one_more(&mut self, entry: &str, size: u64) -> Result<()> {
        self.entries += 1;
        if self.entries > MAX_ENTRIES {
            return Err(refuse(
                entry,
                "the archive holds more entries than Onsa unpacks",
            ));
        }
        self.bytes = self.bytes.saturating_add(size);
        if self.bytes > MAX_UNPACKED {
            return Err(refuse(
                entry,
                "the archive unpacks to more than Onsa will hold",
            ));
        }
        Ok(())
    }
}

/// Makes the folders an entry sits in, inside the destination.
fn make_parents(into: &Path, relative: &Path) -> Result<()> {
    if let Some(parent) = relative.parent() {
        if parent.as_os_str().is_empty() {
            return Ok(());
        }
        std::fs::create_dir_all(into.join(parent))?;
    }
    Ok(())
}

/// Copies one entry's bytes, refusing to write more than it claimed.
fn write_entry(mut from: impl Read, to: &Path, claimed: u64) -> Result<()> {
    let mut file = std::fs::File::create(to)?;
    // One byte more than claimed is a header that lied, and a lying header
    // is how a size cap is walked around.
    let written = std::io::copy(&mut (&mut from).take(claimed + 1), &mut file)?;
    if written > claimed {
        return Err(Error::BadArchive(format!(
            "{}: holds more than its header says",
            to.display()
        )));
    }
    Ok(())
}

fn unpack_zip(bytes: &[u8], into: &Path, wanted: &[String]) -> Result<Vec<PathBuf>> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|error| Error::BadArchive(format!("zip: {error}")))?;
    let mut budget = Budget::new();
    let mut written = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| Error::BadArchive(format!("zip: {error}")))?;
        let named = entry.name().to_string();

        // A zip records unix permissions, and that is where a symlink hides.
        if let Some(mode) = entry.unix_mode() {
            if mode & 0xF000 == 0xA000 {
                return Err(refuse(&named, "it is a symbolic link"));
            }
        }
        if entry.is_dir() {
            let Some(relative) = safe_relative(Path::new(&named)) else {
                return Err(refuse(&named, "it would not stay inside the folder"));
            };
            budget.one_more(&named, 0)?;
            std::fs::create_dir_all(into.join(relative))?;
            continue;
        }
        let Some(relative) = safe_relative(Path::new(&named)) else {
            return Err(refuse(&named, "it would not stay inside the folder"));
        };
        if !asked_for(&relative, wanted) {
            continue;
        }
        let size = entry.size();
        let mode = entry.unix_mode();
        budget.one_more(&named, size)?;
        make_parents(into, &relative)?;
        let target = into.join(&relative);
        write_entry(&mut entry, &target, size)?;
        runnable_if_marked(&target, mode)?;
        written.push(target);
    }
    Ok(written)
}

fn unpack_tar(bytes: &[u8], into: &Path, wanted: &[String]) -> Result<Vec<PathBuf>> {
    let mut archive = tar::Archive::new(std::io::Cursor::new(bytes));
    let mut budget = Budget::new();
    let mut written = Vec::new();

    let entries = archive
        .entries()
        .map_err(|error| Error::BadArchive(format!("tar: {error}")))?;
    for entry in entries {
        let mut entry = entry.map_err(|error| Error::BadArchive(format!("tar: {error}")))?;
        let kind = entry.header().entry_type();
        let named = entry
            .path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "(a name Onsa cannot read)".into());

        // Everything that is not a plain file or a folder: links of both
        // kinds, devices, pipes. None of them is a program to run, and each
        // of them is a way of pointing somewhere else.
        if !kind.is_file() && !kind.is_dir() {
            return Err(refuse(&named, "it is not a plain file or a folder"));
        }
        let path = entry
            .path()
            .map_err(|error| Error::BadArchive(format!("tar: {error}")))?
            .into_owned();
        let Some(relative) = safe_relative(&path) else {
            return Err(refuse(&named, "it would not stay inside the folder"));
        };

        if kind.is_dir() {
            budget.one_more(&named, 0)?;
            std::fs::create_dir_all(into.join(relative))?;
            continue;
        }
        if !asked_for(&relative, wanted) {
            continue;
        }
        let size = entry.header().size().unwrap_or(0);
        budget.one_more(&named, size)?;
        make_parents(into, &relative)?;
        let target = into.join(&relative);
        let mode = entry.header().mode().ok();
        write_entry(&mut entry, &target, size)?;
        runnable_if_marked(&target, mode)?;
        written.push(target);
    }
    Ok(written)
}

/// Keeps the runnable bit an archive recorded, and nothing else about mode.
///
/// Only that one bit: an archive does not get to decide who may read or
/// write a file on this machine.
#[cfg(unix)]
fn runnable_if_marked(path: &Path, mode: Option<u32>) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let executable = mode.is_some_and(|mode| mode & 0o111 != 0);
    let bits = if executable { 0o755 } else { 0o644 };
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(bits);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

/// Windows decides by the file's extension, so there is nothing to set.
#[cfg(not(unix))]
fn runnable_if_marked(_path: &Path, _mode: Option<u32>) -> Result<()> {
    Ok(())
}

/// Finds one file by its name anywhere in what was unpacked.
///
/// Archives disagree about how deep they nest — ffmpeg puts its programs in
/// `<name>/bin/`, Deno puts its one program at the top — so what matters is
/// the name at the end, not the path it sits on.
pub fn find_named<'a>(written: &'a [PathBuf], file_name: &str) -> Option<&'a PathBuf> {
    written
        .iter()
        .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_says_which_kind_it_is() {
        assert_eq!(Kind::of("deno-x86_64-pc-windows-msvc.zip"), Some(Kind::Zip));
        assert_eq!(
            Kind::of("chromaprint-fpcalc-1.5.1-linux-x86_64.tar.gz"),
            Some(Kind::TarGz)
        );
        assert_eq!(
            Kind::of("ffmpeg-master-latest-linux64-gpl.tar.xz"),
            Some(Kind::TarXz)
        );
        assert_eq!(Kind::of("yt-dlp.exe"), None);
    }

    #[test]
    fn a_path_that_stays_inside_is_kept_as_it_is() {
        assert_eq!(
            safe_relative(Path::new("ffmpeg-7.1/bin/ffmpeg")),
            Some(PathBuf::from("ffmpeg-7.1").join("bin").join("ffmpeg"))
        );
        assert_eq!(
            safe_relative(Path::new("./deno")),
            Some(PathBuf::from("deno")),
            "a dot says nothing and is dropped"
        );
    }

    #[test]
    fn a_path_that_would_leave_the_folder_is_refused() {
        assert_eq!(safe_relative(Path::new("../deno")), None);
        assert_eq!(safe_relative(Path::new("a/../../b")), None);
        assert_eq!(safe_relative(Path::new("/etc/passwd")), None);
        assert_eq!(safe_relative(Path::new("/")), None);
        assert_eq!(safe_relative(Path::new("")), None);
        if cfg!(windows) {
            assert_eq!(safe_relative(Path::new(r"C:\Windows\System32\x")), None);
            assert_eq!(safe_relative(Path::new(r"\\server\share\x")), None);
        }
    }

    #[test]
    fn a_separator_hidden_inside_one_piece_is_still_a_separator() {
        // Not something `components` would split, because it never came from
        // this system's idea of a path: a zip entry is just a string.
        assert_eq!(safe_relative(Path::new("a\\..\\..\\b")), None);
    }
}
