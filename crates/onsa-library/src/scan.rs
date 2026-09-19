//! Scanning folders into the library (SPEC §5.2).
//!
//! The first scan reads every file; later scans skip files whose size and
//! modification time did not change, so an unchanged library rescans in a
//! fraction of the time. Writes go in batched transactions. A file that is
//! gone is marked missing rather than deleted, so its statistics and
//! playlist entries survive a drive that is only unplugged; the user clears
//! missing files explicitly.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::{params, OptionalExtension, Transaction};

use crate::cover;
use crate::db::Library;
use crate::error::{Error, Result};
use crate::tags::{self, TrackMeta};

/// File extensions the scanner picks up: the formats the engine plays
/// (SPEC §3.2). Opus joins with its decoder in M11.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "flac", "mp3", "m4a", "m4b", "mp4", "aac", "ogg", "oga", "wav", "wave", "aif", "aiff", "aifc",
];

/// Files written per transaction.
const BATCH_FILES: usize = 200;

/// Where a scan stands, reported after every batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScanProgress {
    /// Audio files found so far.
    pub seen: u64,
    /// Files whose tags were read.
    pub read: u64,
    /// Files skipped because they did not change.
    pub unchanged: u64,
    /// Files that could not be read.
    pub failed: u64,
}

/// The outcome of a scan.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScanReport {
    /// Audio files found.
    pub seen: u64,
    /// Files whose tags were read.
    pub read: u64,
    /// Files skipped because they did not change.
    pub unchanged: u64,
    /// Files that could not be read, recorded as failed.
    pub failed: u64,
    /// Tracks newly marked missing.
    pub missing: u64,
    /// How long the scan took.
    pub elapsed: Duration,
}

/// Whether the scanner handles this file, by extension.
pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        })
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as i64)
}

fn mtime_ms(metadata: &std::fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_millis() as i64)
}

pub(crate) fn path_text(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::NonUnicodePath(path.to_path_buf()))
}

/// A LIKE pattern matching everything below `dir`.
fn below(dir: &str) -> String {
    let mut pattern = String::with_capacity(dir.len() + 2);
    for c in dir.trim_end_matches(['/', '\\']).chars() {
        if matches!(c, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    if std::path::MAIN_SEPARATOR == '\\' {
        pattern.push_str("\\\\");
    } else {
        pattern.push(std::path::MAIN_SEPARATOR);
    }
    pattern.push('%');
    pattern
}

/// Supported files under `dir`, sorted. Hidden and symlinked directories
/// are skipped (a symlink loop must not hang a scan).
fn audio_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(dir = %dir.display(), "cannot read folder: {error}");
                continue;
            }
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            let hidden = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'));
            if kind.is_dir() {
                if !hidden {
                    pending.push(path);
                }
            } else if (kind.is_file() || kind.is_symlink()) && is_supported(&path) {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Writes tracks, albums and covers within one transaction.
pub(crate) struct Indexer {
    covers_dir: PathBuf,
    /// Cover of each folder already looked at in this run.
    folder_covers: HashMap<PathBuf, Option<i64>>,
    /// Hashes of covers that could not be decoded, so each is tried once.
    broken_covers: HashSet<String>,
}

impl Indexer {
    pub(crate) fn new(covers_dir: &Path) -> Self {
        Self {
            covers_dir: covers_dir.to_path_buf(),
            folder_covers: HashMap::new(),
            broken_covers: HashSet::new(),
        }
    }

    /// Reads a file and stores it. Returns whether it could be read.
    pub(crate) fn index(
        &mut self,
        tx: &Transaction<'_>,
        folder_id: i64,
        path: &Path,
        text: &str,
        mtime: i64,
        size: i64,
    ) -> Result<bool> {
        match tags::read(path) {
            Ok(meta) => {
                let cover_id = self.cover_for(tx, path, meta.picture.as_deref())?;
                let album_id = album_for(tx, &meta, cover_id)?;
                upsert(
                    tx, folder_id, text, mtime, size, &meta, album_id, cover_id, "ok",
                )?;
                Ok(true)
            }
            Err(reason) => {
                tracing::warn!(path = %path.display(), "cannot read the file, recorded as failed: {reason}");
                upsert(
                    tx,
                    folder_id,
                    text,
                    mtime,
                    size,
                    &TrackMeta::default(),
                    None,
                    None,
                    "failed",
                )?;
                Ok(false)
            }
        }
    }

    fn cover_for(
        &mut self,
        tx: &Transaction<'_>,
        track: &Path,
        embedded: Option<&[u8]>,
    ) -> Result<Option<i64>> {
        if let Some(bytes) = embedded {
            if let Some(id) = self.cover_from_bytes(tx, bytes, track)? {
                return Ok(Some(id));
            }
        }
        let Some(dir) = track.parent() else {
            return Ok(None);
        };
        if let Some(known) = self.folder_covers.get(dir) {
            return Ok(*known);
        }
        let id = match cover::folder_image(dir) {
            Some(image) => match std::fs::read(&image) {
                Ok(bytes) => self.cover_from_bytes(tx, &bytes, &image)?,
                Err(error) => {
                    tracing::warn!(image = %image.display(), "cannot read the cover: {error}");
                    None
                }
            },
            None => None,
        };
        self.folder_covers.insert(dir.to_path_buf(), id);
        Ok(id)
    }

    fn cover_from_bytes(
        &mut self,
        tx: &Transaction<'_>,
        bytes: &[u8],
        origin: &Path,
    ) -> Result<Option<i64>> {
        let hash = cover::sha256_hex(bytes);
        if self.broken_covers.contains(&hash) {
            return Ok(None);
        }
        let known = tx
            .query_row("SELECT id FROM covers WHERE hash = ?1", [&hash], |row| {
                row.get(0)
            })
            .optional()?;
        if known.is_some() {
            return Ok(known);
        }
        let Some(thumbnails) = cover::make_thumbnails(bytes, &self.covers_dir, origin) else {
            self.broken_covers.insert(hash);
            return Ok(None);
        };
        tx.execute(
            "INSERT INTO covers (hash, width, height, thumb_128, thumb_512)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                thumbnails.hash,
                thumbnails.width,
                thumbnails.height,
                thumbnails.thumb_128,
                thumbnails.thumb_512
            ],
        )?;
        Ok(Some(tx.last_insert_rowid()))
    }
}

/// The album a track belongs to, created on first sight. An album is its
/// title plus its album artist (or, lacking one, the track artist).
fn album_for(tx: &Transaction<'_>, meta: &TrackMeta, cover_id: Option<i64>) -> Result<Option<i64>> {
    let Some(title) = meta.album.as_deref() else {
        return Ok(None);
    };
    let artist = meta
        .album_artist
        .as_deref()
        .or(meta.artist.as_deref())
        .unwrap_or("");
    tx.prepare_cached(
        "INSERT INTO albums (title, album_artist, year, cover_id) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(title, album_artist) DO UPDATE SET
             year = COALESCE(albums.year, excluded.year),
             cover_id = COALESCE(albums.cover_id, excluded.cover_id)",
    )?
    .execute(params![title, artist, meta.year, cover_id])?;
    Ok(Some(tx.query_row(
        "SELECT id FROM albums WHERE title = ?1 AND album_artist = ?2",
        params![title, artist],
        |row| row.get(0),
    )?))
}

#[allow(clippy::too_many_arguments)]
fn upsert(
    tx: &Transaction<'_>,
    folder_id: i64,
    path: &str,
    mtime: i64,
    size: i64,
    meta: &TrackMeta,
    album_id: Option<i64>,
    cover_id: Option<i64>,
    status: &str,
) -> Result<()> {
    tx.prepare_cached(
        "INSERT INTO tracks (
             path, folder_id, mtime, size, duration_ms, codec, sample_rate, bit_depth,
             channels, bitrate, title, artist, album, album_artist, track_number,
             track_total, disc_number, disc_total, year, genre, composer,
             rg_track_gain, rg_track_peak, rg_album_gain, rg_album_peak,
             album_id, cover_id, status, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                 ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)
         ON CONFLICT(path) DO UPDATE SET
             folder_id = excluded.folder_id, mtime = excluded.mtime, size = excluded.size,
             duration_ms = excluded.duration_ms, codec = excluded.codec,
             sample_rate = excluded.sample_rate, bit_depth = excluded.bit_depth,
             channels = excluded.channels, bitrate = excluded.bitrate,
             title = excluded.title, artist = excluded.artist, album = excluded.album,
             album_artist = excluded.album_artist, track_number = excluded.track_number,
             track_total = excluded.track_total, disc_number = excluded.disc_number,
             disc_total = excluded.disc_total, year = excluded.year, genre = excluded.genre,
             composer = excluded.composer, rg_track_gain = excluded.rg_track_gain,
             rg_track_peak = excluded.rg_track_peak, rg_album_gain = excluded.rg_album_gain,
             rg_album_peak = excluded.rg_album_peak, album_id = excluded.album_id,
             cover_id = excluded.cover_id, status = excluded.status",
    )?
    .execute(params![
        path,
        folder_id,
        mtime,
        size,
        meta.duration_ms,
        meta.codec,
        meta.sample_rate,
        meta.bit_depth,
        meta.channels,
        meta.bitrate,
        meta.title,
        meta.artist,
        meta.album,
        meta.album_artist,
        meta.track_number,
        meta.track_total,
        meta.disc_number,
        meta.disc_total,
        meta.year,
        meta.genre,
        meta.composer,
        meta.rg_track_gain,
        meta.rg_track_peak,
        meta.rg_album_gain,
        meta.rg_album_peak,
        album_id,
        cover_id,
        status,
        now_ms()
    ])?;
    Ok(())
}

/// What the watcher's changes did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChangeReport {
    /// Files added or re-read.
    pub updated: u64,
    /// Tracks marked missing.
    pub missing: u64,
    /// Tracks that moved to a new path, keeping their identity.
    pub moved: u64,
}

impl Library {
    /// Adds a folder to watch and scan. Returns its id; adding it twice is
    /// harmless.
    pub fn add_folder(&mut self, path: &Path) -> Result<i64> {
        let absolute = std::path::absolute(path).map_err(|source| Error::io(path, source))?;
        let text = path_text(&absolute)?;
        self.conn.execute(
            "INSERT INTO folders (path, added_at) VALUES (?1, ?2) ON CONFLICT(path) DO NOTHING",
            params![text, now_ms()],
        )?;
        Ok(self
            .conn
            .query_row("SELECT id FROM folders WHERE path = ?1", [&text], |row| {
                row.get(0)
            })?)
    }

    /// Forgets a folder and every track in it, with their statistics.
    pub fn remove_folder(&mut self, path: &Path) -> Result<bool> {
        let absolute = std::path::absolute(path).map_err(|source| Error::io(path, source))?;
        let removed = self.conn.execute(
            "DELETE FROM folders WHERE path = ?1",
            [path_text(&absolute)?],
        )?;
        self.remove_orphans()?;
        Ok(removed > 0)
    }

    /// The watched folders.
    pub fn folder_paths(&self) -> Result<Vec<PathBuf>> {
        let mut statement = self
            .conn
            .prepare_cached("SELECT path FROM folders ORDER BY path")?;
        let paths = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(paths.into_iter().map(PathBuf::from).collect())
    }

    /// Deletes tracks marked missing, which the user asked for explicitly
    /// (SPEC §5.2). Returns how many went.
    pub fn purge_missing(&mut self) -> Result<u64> {
        let removed = self
            .conn
            .execute("DELETE FROM tracks WHERE status = 'missing'", [])?;
        self.remove_orphans()?;
        Ok(removed as u64)
    }

    fn remove_orphans(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM albums WHERE id NOT IN (SELECT album_id FROM tracks WHERE album_id IS NOT NULL)",
            [],
        )?;
        Ok(())
    }

    /// Scans every folder. `progress` hears about every batch.
    pub fn scan(&mut self, mut progress: impl FnMut(&ScanProgress)) -> Result<ScanReport> {
        let started = Instant::now();
        let folders: Vec<(i64, String)> = {
            let mut statement = self.conn.prepare_cached("SELECT id, path FROM folders")?;
            let rows = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            rows
        };
        let mut report = ScanReport::default();
        let mut indexer = Indexer::new(&self.covers_dir);
        for (folder_id, root) in folders {
            self.scan_folder(
                folder_id,
                Path::new(&root),
                &mut indexer,
                &mut report,
                &mut progress,
            )?;
        }
        report.elapsed = started.elapsed();
        tracing::info!(
            seen = report.seen,
            read = report.read,
            unchanged = report.unchanged,
            failed = report.failed,
            missing = report.missing,
            ms = report.elapsed.as_millis() as u64,
            "scan finished"
        );
        Ok(report)
    }

    fn scan_folder(
        &mut self,
        folder_id: i64,
        root: &Path,
        indexer: &mut Indexer,
        report: &mut ScanReport,
        progress: &mut dyn FnMut(&ScanProgress),
    ) -> Result<()> {
        let known: HashMap<String, (i64, i64, String)> = {
            let mut statement = self.conn.prepare_cached(
                "SELECT path, mtime, size, status FROM tracks WHERE folder_id = ?1",
            )?;
            let rows = statement
                .query_map([folder_id], |row| {
                    Ok((row.get(0)?, (row.get(1)?, row.get(2)?, row.get(3)?)))
                })?
                .collect::<std::result::Result<HashMap<_, _>, _>>()?;
            rows
        };

        // An unreachable folder (an unplugged drive) leaves everything in it
        // missing, never deleted.
        let files = if root.is_dir() {
            audio_files(root)
        } else {
            tracing::warn!(folder = %root.display(), "library folder is not reachable");
            Vec::new()
        };

        let mut seen: HashSet<String> = HashSet::with_capacity(files.len());
        for batch in files.chunks(BATCH_FILES) {
            let tx = self.conn.transaction()?;
            for path in batch {
                let Ok(text) = path_text(path) else {
                    tracing::warn!(path = %path.display(), "path is not valid Unicode, skipped");
                    continue;
                };
                let Ok(metadata) = std::fs::metadata(path) else {
                    continue;
                };
                let (mtime, size) = (mtime_ms(&metadata), metadata.len() as i64);
                report.seen += 1;
                let unchanged =
                    known
                        .get(&text)
                        .is_some_and(|(known_mtime, known_size, status)| {
                            *known_mtime == mtime && *known_size == size && status != "missing"
                        });
                if unchanged {
                    report.unchanged += 1;
                } else if indexer.index(&tx, folder_id, path, &text, mtime, size)? {
                    report.read += 1;
                } else {
                    report.failed += 1;
                }
                seen.insert(text);
            }
            tx.commit()?;
            progress(&ScanProgress {
                seen: report.seen,
                read: report.read,
                unchanged: report.unchanged,
                failed: report.failed,
            });
        }

        let tx = self.conn.transaction()?;
        {
            let mut mark = tx.prepare_cached(
                "UPDATE tracks SET status = 'missing' WHERE path = ?1 AND status != 'missing'",
            )?;
            for path in known.keys().filter(|path| !seen.contains(*path)) {
                report.missing += mark.execute([path])? as u64;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// The watched folder that holds `path`, as id and path.
    fn folder_for(&self, path: &Path) -> Result<Option<i64>> {
        let mut best: Option<(usize, i64)> = None;
        let mut statement = self.conn.prepare_cached("SELECT id, path FROM folders")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, folder) = row?;
            let folder = Path::new(&folder);
            if path.starts_with(folder) {
                let depth = folder.components().count();
                if best.is_none_or(|(best_depth, _)| depth > best_depth) {
                    best = Some((depth, id));
                }
            }
        }
        Ok(best.map(|(_, id)| id))
    }

    /// Whether `to`, new to the library, is what used to be at `from`: the
    /// same size and modification time (a move keeps both). For a folder,
    /// one of its known files is compared at its new place.
    pub(crate) fn same_file_moved(&self, from: &Path, to: &Path) -> Result<bool> {
        let Ok(from_text) = path_text(from) else {
            return Ok(false);
        };
        if self.folder_for(to)?.is_none() {
            return Ok(false);
        }
        let (old, new) = if to.is_file() {
            let old: Option<(String, i64, i64)> = self
                .conn
                .query_row(
                    "SELECT path, mtime, size FROM tracks WHERE path = ?1 AND status != 'missing'",
                    [&from_text],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?;
            (old, to.to_path_buf())
        } else if to.is_dir() {
            let old: Option<(String, i64, i64)> = self
                .conn
                .query_row(
                    "SELECT path, mtime, size FROM tracks
                     WHERE path LIKE ?1 ESCAPE '\\' AND status != 'missing' LIMIT 1",
                    [below(&from_text)],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?;
            let new = old
                .as_ref()
                .and_then(|(path, _, _)| Path::new(path).strip_prefix(from).ok())
                .map(|relative| to.join(relative));
            match new {
                Some(new) => (old, new),
                None => return Ok(false),
            }
        } else {
            return Ok(false);
        };
        let Some((_, mtime, size)) = old else {
            return Ok(false);
        };
        if self.track_id(&new)?.is_some() {
            return Ok(false);
        }
        Ok(std::fs::metadata(&new)
            .is_ok_and(|metadata| mtime_ms(&metadata) == mtime && metadata.len() as i64 == size))
    }

    /// A file (or folder) appeared or changed.
    pub(crate) fn file_changed(&mut self, path: &Path, report: &mut ChangeReport) -> Result<()> {
        let Some(folder_id) = self.folder_for(path)? else {
            return Ok(());
        };
        let files = if path.is_dir() {
            audio_files(path)
        } else if path.is_file() && is_supported(path) {
            vec![path.to_path_buf()]
        } else {
            return Ok(());
        };
        let mut indexer = Indexer::new(&self.covers_dir);
        let tx = self.conn.transaction()?;
        for file in files {
            let Ok(text) = path_text(&file) else {
                continue;
            };
            let Ok(metadata) = std::fs::metadata(&file) else {
                continue;
            };
            let (mtime, size) = (mtime_ms(&metadata), metadata.len() as i64);
            let same: bool = tx
                .query_row(
                    "SELECT 1 FROM tracks WHERE path = ?1 AND mtime = ?2 AND size = ?3 AND status != 'missing'",
                    params![text, mtime, size],
                    |_| Ok(true),
                )
                .optional()?
                .unwrap_or(false);
            if !same {
                indexer.index(&tx, folder_id, &file, &text, mtime, size)?;
                report.updated += 1;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Marks one track missing, by id.
    ///
    /// The scanner marks tracks missing when it walks the folders, but
    /// playing one is the other moment the truth shows up: the file the
    /// engine went to open was not there. Saying so at once means the list
    /// stops offering it as if nothing had happened.
    pub fn mark_missing(&mut self, track_id: i64) -> Result<bool> {
        let changed = self.conn.execute(
            "UPDATE tracks SET status = 'missing' WHERE id = ?1 AND status != 'missing'",
            [track_id],
        )?;
        Ok(changed > 0)
    }

    /// A file (or folder) went away: its tracks are marked missing.
    pub(crate) fn file_removed(&mut self, path: &Path, report: &mut ChangeReport) -> Result<()> {
        let Ok(text) = path_text(path) else {
            return Ok(());
        };
        let exact = self.conn.execute(
            "UPDATE tracks SET status = 'missing' WHERE path = ?1 AND status != 'missing'",
            [&text],
        )?;
        let below = if exact == 0 {
            self.conn.execute(
                "UPDATE tracks SET status = 'missing' WHERE path LIKE ?1 ESCAPE '\\' AND status != 'missing'",
                [below(&text)],
            )?
        } else {
            0
        };
        report.missing += (exact + below) as u64;
        Ok(())
    }

    /// A file (or folder) moved. Its tracks keep their identity, and so
    /// their statistics and playlist entries; only the path changes.
    pub(crate) fn file_renamed(
        &mut self,
        from: &Path,
        to: &Path,
        report: &mut ChangeReport,
    ) -> Result<()> {
        let (Ok(from_text), Ok(to_text)) = (path_text(from), path_text(to)) else {
            return Ok(());
        };
        let Some(folder_id) = self.folder_for(to)? else {
            // Moved out of the library.
            return self.file_removed(from, report);
        };
        let tx = self.conn.transaction()?;
        // A file replaced by the move loses its old row.
        tx.execute("DELETE FROM tracks WHERE path = ?1", [&to_text])?;
        let mut moved = tx.execute(
            "UPDATE tracks SET path = ?2, folder_id = ?3, status = 'ok' WHERE path = ?1",
            params![from_text, to_text, folder_id],
        )?;
        if moved == 0 {
            let offset = from_text.trim_end_matches(['/', '\\']).chars().count() as i64 + 1;
            moved = tx.execute(
                "UPDATE tracks SET path = ?2 || substr(path, ?3), folder_id = ?4
                 WHERE path LIKE ?1 ESCAPE '\\'",
                params![
                    below(&from_text),
                    to_text.trim_end_matches(['/', '\\']),
                    offset,
                    folder_id
                ],
            )?;
        }
        tx.commit()?;
        report.moved += moved as u64;
        if moved == 0 {
            // Nothing known under the old name: it is simply new here.
            self.file_changed(to, report)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_the_supported_formats() {
        assert!(is_supported(Path::new("a/b.FLAC")));
        assert!(is_supported(Path::new("夜明け.m4a")));
        assert!(!is_supported(Path::new("cover.jpg")));
        assert!(!is_supported(Path::new("song.opus")), "Opus joins in M11");
        assert!(!is_supported(Path::new("noextension")));
    }

    #[test]
    fn like_patterns_escape_wildcards() {
        let pattern = below("C:\\Music\\100%_mix");
        assert!(pattern.starts_with("C:\\\\Music\\\\100\\%\\_mix"));
        assert!(pattern.ends_with('%'));
    }
}
