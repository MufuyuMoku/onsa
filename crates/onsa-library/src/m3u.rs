//! M3U8 playlists, in and out (SPEC §6.4).
//!
//! Export writes `#EXTM3U` with an `#EXTINF` line per track, and paths
//! either relative to the playlist file or absolute. Import matches each
//! path back to the library and **reports what it could not find** rather
//! than dropping it quietly.

use std::path::{Component, Path, PathBuf};

use crate::db::Library;
use crate::error::{Error, Result};
use crate::search::TrackRow;

/// Whether exported paths are relative to the playlist file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathStyle {
    /// Relative where possible, so the playlist travels with the music.
    #[default]
    Relative,
    /// Always the full path.
    Absolute,
}

/// What came of reading a playlist file (SPEC §6.4).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportReport {
    /// Track ids, in the order the file listed them.
    pub tracks: Vec<i64>,
    /// Entries the library has no track for, as the file wrote them.
    pub missing: Vec<String>,
}

/// Writes tracks as an M3U8 playlist.
///
/// The file is UTF-8 without a byte order mark, and its lines end with a
/// single newline: that is what other players read, on either system.
pub fn write_m3u8(path: &Path, tracks: &[TrackRow], style: PathStyle) -> Result<()> {
    let base = path.parent().unwrap_or(Path::new(""));
    let mut text = String::from("#EXTM3U\n");
    for track in tracks {
        let seconds = track
            .duration_ms
            .map(|ms| (ms as f64 / 1000.0).round() as i64)
            .unwrap_or(-1);
        let artist = track.artist.clone().unwrap_or_default();
        let title = track
            .title
            .clone()
            .unwrap_or_else(|| file_name(&track.path).to_string());
        let name = if artist.is_empty() {
            title
        } else {
            format!("{artist} - {title}")
        };
        text.push_str(&format!("#EXTINF:{seconds},{name}\n"));
        text.push_str(&written_path(&track.path, base, style));
        text.push('\n');
    }
    std::fs::write(path, text).map_err(|source| Error::io(path, source))
}

/// How one track's path is written in the file.
fn written_path(path: &str, base: &Path, style: PathStyle) -> String {
    let full = Path::new(path);
    if style == PathStyle::Absolute {
        return slashes(full);
    }
    match relative_to(full, base) {
        Some(relative) => slashes(&relative),
        None => slashes(full),
    }
}

/// `path` as seen from `base`, when both are on the same root.
fn relative_to(path: &Path, base: &Path) -> Option<PathBuf> {
    if base.as_os_str().is_empty() {
        return None;
    }
    let mut from = base.components().peekable();
    let mut to = path.components().peekable();
    // Drop the part they share.
    while let (Some(a), Some(b)) = (from.peek(), to.peek()) {
        if same_component(a, b) {
            from.next();
            to.next();
        } else {
            break;
        }
    }
    if base.components().count() == from.clone().count() {
        // Nothing in common at all: a different drive or root.
        return None;
    }
    let mut relative = PathBuf::new();
    for _ in from {
        relative.push("..");
    }
    for part in to {
        relative.push(part.as_os_str());
    }
    (!relative.as_os_str().is_empty()).then_some(relative)
}

/// Windows paths are compared without regard to case, as the system does.
fn same_component(a: &Component<'_>, b: &Component<'_>) -> bool {
    if cfg!(windows) {
        a.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
    } else {
        a == b
    }
}

/// Forward slashes, which every player reads on either system.
fn slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Reads an M3U8 playlist and matches its entries to the library.
///
/// `#EXTINF` lines are read for nothing but their place: the library's own
/// tags are better than a playlist file's. Comments and blank lines are
/// skipped; everything else is a path.
pub fn read_m3u8(path: &Path, library: &Library) -> Result<ImportReport> {
    let text = std::fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    let base = path.parent().unwrap_or(Path::new(""));
    let mut report = ImportReport::default();
    for line in text.lines() {
        let line = line.trim_start_matches('\u{feff}').trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match library.track_at_path(&resolve(line, base))? {
            Some(id) => report.tracks.push(id),
            None => report.missing.push(line.to_string()),
        }
    }
    Ok(report)
}

/// The full path an entry points at, relative entries included.
fn resolve(entry: &str, base: &Path) -> PathBuf {
    let written = PathBuf::from(entry.replace('\\', "/"));
    if written.is_absolute() || entry.len() > 1 && entry.as_bytes()[1] == b':' {
        return PathBuf::from(entry);
    }
    tidy(&base.join(written))
}

/// Resolves `.` and `..` without touching the disk, so a playlist that
/// points outside its own folder still matches.
fn tidy(path: &Path) -> PathBuf {
    let mut parts: Vec<Component<'_>> = Vec::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(parts.last(), Some(Component::Normal(_))) {
                    parts.pop();
                } else {
                    parts.push(part);
                }
            }
            other => parts.push(other),
        }
    }
    parts.iter().collect()
}

impl Library {
    /// The track stored at a path, if the library knows it. Paths are
    /// compared the way the system compares them.
    pub fn track_at_path(&self, path: &Path) -> Result<Option<i64>> {
        let wanted = path.to_string_lossy().replace('/', "\\");
        let alternative = path.to_string_lossy().replace('\\', "/");
        let mut statement = if cfg!(windows) {
            self.conn.prepare(
                "SELECT id FROM tracks WHERE path = ?1 COLLATE NOCASE
                 OR path = ?2 COLLATE NOCASE LIMIT 1",
            )?
        } else {
            self.conn
                .prepare("SELECT id FROM tracks WHERE path = ?1 OR path = ?2 LIMIT 1")?
        };
        let mut rows = statement.query(rusqlite::params![wanted, alternative])?;
        Ok(match rows.next()? {
            Some(row) => Some(row.get(0)?),
            None => None,
        })
    }
}
