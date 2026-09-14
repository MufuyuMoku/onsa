//! Changing a lot of tracks at once, and being able to take it back
//! (SPEC §8).
//!
//! Nothing here happens to a track without three things being true: it is
//! inside the folder the run was held to, the run is no longer than the cap
//! it was given, and every step is written down before the next one starts.
//! That last part is what makes undo possible, and undo is why the rest is
//! worth doing at all.
//!
//! The order of the layers matters. An edit lands in `overrides` first, where
//! nothing about the file has changed yet and taking it back costs nothing.
//! Writing those values into the files is a second, separate action, and
//! moving the files is a third. Undo walks back up the same stairs.

use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension};

use crate::db::Library;
use crate::error::{Error, Result};
use crate::overrides::Field;
use crate::search::{track_columns, track_row, TrackRow};
use crate::write;

/// The most tracks one run may touch, whatever it was asked for.
///
/// A run that goes wrong has to be read through and taken back by a person,
/// and there is a size past which nobody does that carefully.
pub const MAX_BATCH: usize = 500;
/// What a run is held to when nobody says otherwise.
pub const DEFAULT_BATCH: usize = 50;

/// The folder a run is allowed to touch, and how many tracks it may take.
///
/// The folder is the point of the whole thing: it is how a listener tries
/// Onsa out on a copy of ten albums before letting it near the real library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// Only tracks under this folder may be touched. `None` means the whole
    /// library, which is what the listener has to ask for on purpose.
    pub folder: Option<PathBuf>,
    /// Most tracks this run may change.
    pub limit: usize,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            folder: None,
            limit: DEFAULT_BATCH,
        }
    }
}

impl Scope {
    /// A run held to one folder, for trying things out.
    pub fn folder(path: impl Into<PathBuf>) -> Self {
        Self {
            folder: Some(path.into()),
            limit: DEFAULT_BATCH,
        }
    }

    /// The same scope with a different cap, never above [`MAX_BATCH`].
    pub fn take(self, limit: usize) -> Self {
        Self {
            limit: limit.clamp(1, MAX_BATCH),
            ..self
        }
    }

    /// Whether a file is inside the folder this run is held to.
    ///
    /// Both paths are tidied of `.` and `..` first, so a path that climbs
    /// out of the folder and back in cannot sneak past, and on Windows the
    /// comparison ignores case the way the system does.
    pub fn allows(&self, path: &Path) -> bool {
        let Some(folder) = &self.folder else {
            return true;
        };
        let folder = tidy(folder);
        let path = tidy(path);
        let mut inside = path.components();
        for part in folder.components() {
            match inside.next() {
                Some(mine) if same(&mine, &part) => {}
                _ => return false,
            }
        }
        // The folder itself is not a file in it; there has to be more path.
        inside.next().is_some()
    }
}

/// Resolves `.` and `..` without touching the disk.
fn tidy(path: &Path) -> PathBuf {
    let mut parts: Vec<std::path::Component<'_>> = Vec::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if matches!(parts.last(), Some(std::path::Component::Normal(_))) {
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

/// Windows compares paths without regard to case; so does this.
fn same(a: &std::path::Component<'_>, b: &std::path::Component<'_>) -> bool {
    if cfg!(windows) {
        a.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
    } else {
        a == b
    }
}

/// One change to one field of one track, as proposed.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// The track it applies to.
    pub track_id: i64,
    /// The field to set.
    pub field: Field,
    /// The value to set it to; `None` clears the override.
    pub value: Option<String>,
}

/// What a run did, and what it left alone.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BatchReport {
    /// The batch, for taking it back. `None` when nothing changed.
    pub batch: Option<i64>,
    /// Fields actually changed.
    pub changed: usize,
    /// Changes that asked for what was already there.
    pub unchanged: usize,
    /// Tracks left alone because they sit outside the folder.
    pub out_of_scope: usize,
    /// Tracks left over because the run hit its cap.
    pub over_limit: usize,
    /// What could not be done, with the reason. The files are untouched.
    pub failed: Vec<String>,
}

/// One run of changes as it is listed for the listener.
#[derive(Debug, Clone, PartialEq)]
pub struct Batch {
    /// Its id.
    pub id: i64,
    /// What the run was.
    pub note: String,
    /// The folder it was held to.
    pub scope: Option<String>,
    /// When it ran, in milliseconds since the Unix epoch.
    pub created_at: i64,
    /// When it was taken back, if it was.
    pub undone_at: Option<i64>,
    /// How many steps it holds.
    pub steps: usize,
}

/// What came of taking a run back.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UndoReport {
    /// Steps put back.
    pub restored: usize,
    /// Steps that could not be put back, with the reason.
    pub failed: Vec<String>,
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or_default()
}

/// One step of a run, as it comes back out of the record: what kind it was,
/// which track, which field, and the values either side of it.
type StepRow = (String, i64, Option<String>, Option<String>, Option<String>);

/// What to write into one track's file: the field and its new value.
type FieldWrite = (String, Option<String>);

impl Library {
    /// Applies changes as one batch that can be taken back as one batch.
    ///
    /// Changes to tracks outside the folder are not applied and not written
    /// down; they are counted and reported. A change that asks for what is
    /// already there is not a change and leaves no step behind, so undoing
    /// the run does not put back something nobody altered.
    pub fn apply_edits(
        &mut self,
        note: &str,
        scope: &Scope,
        changes: &[Change],
    ) -> Result<BatchReport> {
        let note = note.trim();
        if note.is_empty() {
            return Err(Error::Invalid("a run needs a name".into()));
        }
        let mut report = BatchReport::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);

        // Which tracks are allowed, and which of them fit inside the cap.
        let mut allowed: Vec<&Change> = Vec::new();
        let mut taken: Vec<i64> = Vec::new();
        for change in changes {
            let Some(path) = self.track_path(change.track_id)? else {
                report.out_of_scope += 1;
                continue;
            };
            if !scope.allows(Path::new(&path)) {
                report.out_of_scope += 1;
                continue;
            }
            if !taken.contains(&change.track_id) {
                if taken.len() >= limit {
                    report.over_limit += 1;
                    continue;
                }
                taken.push(change.track_id);
            }
            allowed.push(change);
        }

        let stamp = now_ms();
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO edit_batches (note, scope, created_at) VALUES (?1, ?2, ?3)",
            params![
                note,
                scope.folder.as_ref().map(|p| p.display().to_string()),
                stamp
            ],
        )?;
        let batch = tx.last_insert_rowid();
        let mut position = 0i64;
        for change in allowed {
            let before: Option<String> = tx
                .query_row(
                    "SELECT value FROM overrides WHERE track_id = ?1 AND field = ?2",
                    params![change.track_id, change.field.name()],
                    |row| row.get(0),
                )
                .optional()?;
            if before.as_deref() == change.value.as_deref() {
                report.unchanged += 1;
                continue;
            }
            match &change.value {
                Some(value) => tx.execute(
                    "INSERT INTO overrides (track_id, field, value, unwritten) VALUES (?1, ?2, ?3, 1)
                     ON CONFLICT(track_id, field) DO UPDATE SET value = excluded.value, unwritten = 1",
                    params![change.track_id, change.field.name(), value],
                )?,
                None => tx.execute(
                    "DELETE FROM overrides WHERE track_id = ?1 AND field = ?2",
                    params![change.track_id, change.field.name()],
                )?,
            };
            tx.execute(
                "INSERT INTO edit_steps (batch_id, position, track_id, kind, field, before, after)
                 VALUES (?1, ?2, ?3, 'override', ?4, ?5, ?6)",
                params![
                    batch,
                    position,
                    change.track_id,
                    change.field.name(),
                    before,
                    change.value
                ],
            )?;
            position += 1;
            report.changed += 1;
        }

        if report.changed == 0 {
            // A run that changed nothing is not worth remembering.
            tx.execute("DELETE FROM edit_batches WHERE id = ?1", [batch])?;
            tx.commit()?;
            return Ok(report);
        }
        for track in &taken {
            reindex(&tx, *track)?;
        }
        tx.commit()?;
        report.batch = Some(batch);
        Ok(report)
    }

    /// Writes the edits that are still only in Onsa into the files (SPEC §8).
    ///
    /// This is the separate action the specification asks for: an override
    /// changes what Onsa shows, and only this changes what is on the disk.
    /// What the file said before is written down first, so undo can put it
    /// back even though the override no longer knows it.
    pub fn write_to_files(
        &mut self,
        note: &str,
        scope: &Scope,
        tracks: &[i64],
    ) -> Result<BatchReport> {
        let note = note.trim();
        if note.is_empty() {
            return Err(Error::Invalid("a run needs a name".into()));
        }
        let mut report = BatchReport::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);

        // What to write, per track, before anything is opened.
        let mut work: Vec<(i64, String, Vec<FieldWrite>)> = Vec::new();
        for track_id in tracks {
            let Some(path) = self.track_path(*track_id)? else {
                report.out_of_scope += 1;
                continue;
            };
            if !scope.allows(Path::new(&path)) {
                report.out_of_scope += 1;
                continue;
            }
            if work.len() >= limit {
                report.over_limit += 1;
                continue;
            }
            let mut fields = Vec::new();
            {
                let mut statement = self.conn.prepare(
                    "SELECT field, value FROM overrides
                     WHERE track_id = ?1 AND unwritten = 1 ORDER BY field",
                )?;
                let rows: Vec<FieldWrite> = statement
                    .query_map([track_id], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                for (field, value) in rows {
                    if write::WRITABLE.contains(&field.as_str()) {
                        fields.push((field, value));
                    }
                }
            }
            if fields.is_empty() {
                report.unchanged += 1;
                continue;
            }
            work.push((*track_id, path, fields));
        }

        if work.is_empty() {
            return Ok(report);
        }

        let stamp = now_ms();
        self.conn.execute(
            "INSERT INTO edit_batches (note, scope, created_at) VALUES (?1, ?2, ?3)",
            params![
                note,
                scope.folder.as_ref().map(|p| p.display().to_string()),
                stamp
            ],
        )?;
        let batch = self.conn.last_insert_rowid();
        let mut position = 0i64;

        for (track_id, path, fields) in work {
            let file = Path::new(&path);
            // What the file says now is what undo has to put back.
            let mut before: Vec<FieldWrite> = Vec::new();
            for (field, _) in &fields {
                before.push((
                    field.clone(),
                    write::read_field(file, field).unwrap_or(None),
                ));
            }
            match write::write_fields(file, &fields) {
                Ok(()) => {
                    for ((field, after), (_, was)) in fields.iter().zip(before.iter()) {
                        self.conn.execute(
                            "INSERT INTO edit_steps
                               (batch_id, position, track_id, kind, field, before, after)
                             VALUES (?1, ?2, ?3, 'tag', ?4, ?5, ?6)",
                            params![batch, position, track_id, field, was, after],
                        )?;
                        position += 1;
                        report.changed += 1;
                    }
                    self.conn.execute(
                        "UPDATE overrides SET unwritten = 0 WHERE track_id = ?1",
                        [track_id],
                    )?;
                }
                Err(error) => {
                    // The original is untouched; say so and carry on.
                    tracing::warn!("{path} was not written: {error}");
                    report.failed.push(format!("{path}: {error}"));
                }
            }
        }

        if report.changed == 0 {
            self.conn
                .execute("DELETE FROM edit_batches WHERE id = ?1", [batch])?;
            return Ok(report);
        }
        report.batch = Some(batch);
        Ok(report)
    }

    /// Puts a new cover on tracks, as one run that can be taken back.
    ///
    /// The picture comes from outside — the caller fetched it; this crate
    /// never speaks to the network (SPEC §2) — and is stored the way every
    /// other cover is: thumbnails in the cache, one row per picture, and the
    /// track pointed at it. Nothing is written into the file, so taking it
    /// back is a matter of pointing the track at what it had before.
    ///
    /// A picture that cannot be decoded is counted as failed and changes
    /// nothing, which is what a broken download looks like from here.
    pub fn apply_covers(
        &mut self,
        note: &str,
        scope: &Scope,
        picks: &[(i64, Vec<u8>)],
    ) -> Result<BatchReport> {
        let note = note.trim();
        if note.is_empty() {
            return Err(Error::Invalid("a run needs a name".into()));
        }
        let mut report = BatchReport::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);

        let mut allowed: Vec<(i64, &Vec<u8>)> = Vec::new();
        for (track_id, bytes) in picks {
            let Some(path) = self.track_path(*track_id)? else {
                report.out_of_scope += 1;
                continue;
            };
            if !scope.allows(Path::new(&path)) {
                report.out_of_scope += 1;
                continue;
            }
            if allowed.len() >= limit {
                report.over_limit += 1;
                continue;
            }
            allowed.push((*track_id, bytes));
        }
        if allowed.is_empty() {
            return Ok(report);
        }

        let covers_dir = self.covers_dir.clone();
        let stamp = now_ms();
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO edit_batches (note, scope, created_at) VALUES (?1, ?2, ?3)",
            params![
                note,
                scope.folder.as_ref().map(|p| p.display().to_string()),
                stamp
            ],
        )?;
        let batch = tx.last_insert_rowid();
        let mut position = 0i64;
        for (track_id, bytes) in allowed {
            let before: Option<i64> = tx
                .query_row(
                    "SELECT cover_id FROM tracks WHERE id = ?1",
                    [track_id],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();
            let Some(after) = store_cover(&tx, &covers_dir, bytes)? else {
                report
                    .failed
                    .push(format!("track {track_id}: the picture could not be read"));
                continue;
            };
            if before == Some(after) {
                report.unchanged += 1;
                continue;
            }
            tx.execute(
                "UPDATE tracks SET cover_id = ?2 WHERE id = ?1",
                params![track_id, after],
            )?;
            tx.execute(
                "INSERT INTO edit_steps (batch_id, position, track_id, kind, field, before, after)
                 VALUES (?1, ?2, ?3, 'cover', NULL, ?4, ?5)",
                params![
                    batch,
                    position,
                    track_id,
                    before.map(|id| id.to_string()),
                    after.to_string()
                ],
            )?;
            position += 1;
            report.changed += 1;
        }

        if report.changed == 0 {
            tx.execute("DELETE FROM edit_batches WHERE id = ?1", [batch])?;
            tx.commit()?;
            return Ok(report);
        }
        tx.commit()?;
        report.batch = Some(batch);
        Ok(report)
    }

    /// What [`Library::apply_covers`] would do, without storing anything.
    pub fn preview_covers(&self, scope: &Scope, tracks: &[i64]) -> Result<Summary> {
        let mut summary = Summary::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);
        for track_id in tracks {
            let Some(path) = self.track_path(*track_id)? else {
                summary.count_skip(Skipped::OutsideFolder);
                continue;
            };
            if !scope.allows(Path::new(&path)) {
                summary.count_skip(Skipped::OutsideFolder);
                continue;
            }
            if summary.covers >= limit {
                summary.count_skip(Skipped::OverLimit);
                continue;
            }
            summary.covers += 1;
        }
        summary.tracks = summary.covers;
        Ok(summary)
    }

    /// The tracks a run would be allowed to look at, in path order.
    ///
    /// The folder is matched in SQL first so a large library does not come
    /// back whole, and then every row is put through the same test the run
    /// itself uses: SQL `LIKE` knows nothing about `..`.
    pub fn tracks_in_scope(&self, scope: &Scope, limit: usize) -> Result<Vec<TrackRow>> {
        let prefix = scope.folder.as_ref().map(|folder| {
            let text = folder.to_string_lossy().to_string();
            format!("{}%", text.trim_end_matches(['/', '\\']))
        });
        let sql = format!(
            "SELECT {} FROM track_view v
             WHERE (?1 IS NULL OR v.path LIKE ?1)
             ORDER BY v.path LIMIT ?2",
            track_columns("v.")
        );
        let mut statement = self.conn.prepare(&sql)?;
        let rows: Vec<TrackRow> = statement
            .query_map(params![prefix, (limit.max(1) * 4) as i64], track_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows
            .into_iter()
            .filter(|row| scope.allows(Path::new(&row.path)))
            .take(limit)
            .collect())
    }

    /// The runs Onsa has made, newest first.
    pub fn batches(&self, limit: usize) -> Result<Vec<Batch>> {
        let mut statement = self.conn.prepare(
            "SELECT b.id, b.note, b.scope, b.created_at, b.undone_at,
                    (SELECT COUNT(*) FROM edit_steps s WHERE s.batch_id = b.id)
             FROM edit_batches b
             ORDER BY b.created_at DESC, b.id DESC
             LIMIT ?1",
        )?;
        let rows: Vec<Batch> = statement
            .query_map([limit as i64], |row| {
                Ok(Batch {
                    id: row.get(0)?,
                    note: row.get(1)?,
                    scope: row.get(2)?,
                    created_at: row.get(3)?,
                    undone_at: row.get(4)?,
                    steps: row.get::<_, i64>(5)? as usize,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// The most recent run that has not been taken back.
    pub fn last_batch(&self) -> Result<Option<Batch>> {
        Ok(self
            .batches(50)?
            .into_iter()
            .find(|batch| batch.undone_at.is_none()))
    }

    /// Takes a whole run back, newest step first.
    ///
    /// Steps are undone in reverse so that a file which was both retagged
    /// and renamed is put back in the order it was changed: the tags go back
    /// while the file is still under its new name, and only then does the
    /// name go back.
    pub fn undo_batch(&mut self, batch: i64) -> Result<UndoReport> {
        let already: Option<i64> = self
            .conn
            .query_row(
                "SELECT undone_at FROM edit_batches WHERE id = ?1",
                [batch],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| Error::Invalid("there is no such run".into()))?;
        if already.is_some() {
            return Err(Error::Invalid("that run was already taken back".into()));
        }

        let steps = {
            let mut statement = self.conn.prepare(
                "SELECT kind, track_id, field, before, after FROM edit_steps
                 WHERE batch_id = ?1 ORDER BY position DESC",
            )?;
            let rows: Vec<StepRow> = statement
                .query_map([batch], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            rows
        };

        let mut report = UndoReport::default();
        let mut touched: Vec<i64> = Vec::new();
        for (kind, track_id, field, before, after) in steps {
            if !touched.contains(&track_id) {
                touched.push(track_id);
            }
            let outcome = match kind.as_str() {
                "override" => self.undo_override(track_id, field.as_deref(), before.as_deref()),
                "tag" => self.undo_tag(track_id, field.as_deref(), before.as_deref()),
                "move" => self.undo_move(track_id, before.as_deref(), after.as_deref()),
                "cover" => self.undo_cover(track_id, before.as_deref()),
                other => Err(Error::Invalid(format!("a step of kind {other}"))),
            };
            match outcome {
                Ok(()) => report.restored += 1,
                Err(error) => report.failed.push(error.to_string()),
            }
        }

        self.conn.execute(
            "UPDATE edit_batches SET undone_at = ?2 WHERE id = ?1",
            params![batch, now_ms()],
        )?;
        for track in touched {
            reindex(&self.conn, track)?;
        }
        Ok(report)
    }

    fn undo_override(
        &self,
        track_id: i64,
        field: Option<&str>,
        before: Option<&str>,
    ) -> Result<()> {
        let field = field.ok_or_else(|| Error::Invalid("a step with no field".into()))?;
        match before {
            Some(value) => self.conn.execute(
                "INSERT INTO overrides (track_id, field, value, unwritten) VALUES (?1, ?2, ?3, 1)
                 ON CONFLICT(track_id, field) DO UPDATE SET value = excluded.value, unwritten = 1",
                params![track_id, field, value],
            )?,
            None => self.conn.execute(
                "DELETE FROM overrides WHERE track_id = ?1 AND field = ?2",
                params![track_id, field],
            )?,
        };
        Ok(())
    }

    /// Puts a tag value back into the file it came from.
    fn undo_tag(&self, track_id: i64, field: Option<&str>, before: Option<&str>) -> Result<()> {
        let field = field.ok_or_else(|| Error::Invalid("a step with no field".into()))?;
        let path = self
            .track_path(track_id)?
            .ok_or_else(|| Error::Invalid(format!("track {track_id} is no longer here")))?;
        write::write_fields(
            Path::new(&path),
            &[(field.to_string(), before.map(str::to_string))],
        )
    }

    /// Points a track back at the cover it had.
    ///
    /// The picture it was given is left in the cache and in `covers`: other
    /// tracks may be pointing at it, and a picture costs a few kilobytes
    /// while getting it back would cost another download.
    fn undo_cover(&self, track_id: i64, before: Option<&str>) -> Result<()> {
        let before: Option<i64> = match before {
            Some(text) => Some(
                text.parse()
                    .map_err(|_| Error::Invalid("a cover step with no cover".into()))?,
            ),
            None => None,
        };
        self.conn.execute(
            "UPDATE tracks SET cover_id = ?2 WHERE id = ?1",
            params![track_id, before],
        )?;
        Ok(())
    }

    /// Puts a file back where it was, and tells the library about it.
    fn undo_move(&self, track_id: i64, before: Option<&str>, after: Option<&str>) -> Result<()> {
        let (before, after) = match (before, after) {
            (Some(before), Some(after)) => (before, after),
            _ => return Err(Error::Invalid("a move with nowhere to go".into())),
        };
        write::move_file(Path::new(after), Path::new(before))?;
        self.conn.execute(
            "UPDATE tracks SET path = ?2 WHERE id = ?1",
            params![track_id, before],
        )?;
        Ok(())
    }

    /// The path the library has for a track right now.
    pub(crate) fn track_path(&self, track_id: i64) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT path FROM tracks WHERE id = ?1", [track_id], |row| {
                row.get(0)
            })
            .optional()?)
    }
}

/// Why a track was left out of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skipped {
    /// It sits outside the folder the run is held to.
    OutsideFolder,
    /// The run was already as long as it may be.
    OverLimit,
    /// What was asked for is what it already says.
    NoChange,
    /// Another track would take the same name.
    NameTaken,
}

impl Skipped {
    /// A fixed word per reason, for the interface to translate.
    pub fn name(self) -> &'static str {
        match self {
            Self::OutsideFolder => "outsideFolder",
            Self::OverLimit => "overLimit",
            Self::NoChange => "noChange",
            Self::NameTaken => "nameTaken",
        }
    }
}

/// What a run would do, worked out without doing any of it (SPEC §8).
///
/// This is what the listener reads before there is a button to press. It
/// counts what would happen and, just as importantly, what would not and
/// why.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Summary {
    /// Tracks the run would touch.
    pub tracks: usize,
    /// Field values it would change.
    pub fields: usize,
    /// Files whose tags it would rewrite.
    pub files_written: usize,
    /// Files it would move or rename.
    pub files_moved: usize,
    /// Tracks it would put a new cover on.
    pub covers: usize,
    /// What it would leave alone, and why.
    pub skipped: Vec<(Skipped, usize)>,
}

impl Summary {
    pub(crate) fn count_skip(&mut self, reason: Skipped) {
        match self.skipped.iter_mut().find(|(kind, _)| *kind == reason) {
            Some((_, count)) => *count += 1,
            None => self.skipped.push((reason, 1)),
        }
    }

    /// Whether pressing apply would do anything at all.
    pub fn would_do_anything(&self) -> bool {
        self.fields > 0 || self.files_written > 0 || self.files_moved > 0 || self.covers > 0
    }
}

impl Library {
    /// What [`Library::apply_edits`] would do, without doing it.
    pub fn preview_edits(&self, scope: &Scope, changes: &[Change]) -> Result<Summary> {
        let mut summary = Summary::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);
        let mut taken: Vec<i64> = Vec::new();
        for change in changes {
            let Some(path) = self.track_path(change.track_id)? else {
                summary.count_skip(Skipped::OutsideFolder);
                continue;
            };
            if !scope.allows(Path::new(&path)) {
                summary.count_skip(Skipped::OutsideFolder);
                continue;
            }
            if !taken.contains(&change.track_id) {
                if taken.len() >= limit {
                    summary.count_skip(Skipped::OverLimit);
                    continue;
                }
                taken.push(change.track_id);
            }
            let before: Option<String> = self
                .conn
                .query_row(
                    "SELECT value FROM overrides WHERE track_id = ?1 AND field = ?2",
                    params![change.track_id, change.field.name()],
                    |row| row.get(0),
                )
                .optional()?;
            if before.as_deref() == change.value.as_deref() {
                summary.count_skip(Skipped::NoChange);
                continue;
            }
            summary.fields += 1;
        }
        summary.tracks = taken.len();
        Ok(summary)
    }

    /// What [`Library::write_to_files`] would do, without touching a file.
    pub fn preview_writes(&self, scope: &Scope, tracks: &[i64]) -> Result<Summary> {
        let mut summary = Summary::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);
        for track_id in tracks {
            let Some(path) = self.track_path(*track_id)? else {
                summary.count_skip(Skipped::OutsideFolder);
                continue;
            };
            if !scope.allows(Path::new(&path)) {
                summary.count_skip(Skipped::OutsideFolder);
                continue;
            }
            if summary.files_written >= limit {
                summary.count_skip(Skipped::OverLimit);
                continue;
            }
            let waiting: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM overrides WHERE track_id = ?1 AND unwritten = 1",
                [track_id],
                |row| row.get(0),
            )?;
            if waiting == 0 {
                summary.count_skip(Skipped::NoChange);
                continue;
            }
            summary.files_written += 1;
            summary.fields += waiting as usize;
        }
        summary.tracks = summary.files_written;
        Ok(summary)
    }
}

/// Stores a picture as a cover and gives back its id.
///
/// A picture Onsa already has is not stored twice: covers are kept by the
/// hash of their bytes, so the same album art on forty tracks is one row and
/// one pair of thumbnails. A picture that cannot be decoded gives `None`.
fn store_cover(
    conn: &rusqlite::Connection,
    covers_dir: &Path,
    bytes: &[u8],
) -> Result<Option<i64>> {
    let hash = crate::cover::sha256_hex(bytes);
    let known: Option<i64> = conn
        .query_row("SELECT id FROM covers WHERE hash = ?1", [&hash], |row| {
            row.get(0)
        })
        .optional()?;
    if known.is_some() {
        return Ok(known);
    }
    let Some(thumbnails) =
        crate::cover::make_thumbnails(bytes, covers_dir, Path::new("a suggested cover"))
    else {
        return Ok(None);
    };
    conn.execute(
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
    Ok(Some(conn.last_insert_rowid()))
}

/// Brings one track's search entry back in line with what it now shows.
fn reindex(conn: &rusqlite::Connection, track_id: i64) -> Result<()> {
    conn.execute("DELETE FROM tracks_fts WHERE rowid = ?1", [track_id])?;
    conn.execute(
        "INSERT INTO tracks_fts (rowid, title, artist, album, album_artist, genre)
         SELECT id, title, artist, album, album_artist, genre FROM track_view WHERE id = ?1",
        [track_id],
    )?;
    Ok(())
}
