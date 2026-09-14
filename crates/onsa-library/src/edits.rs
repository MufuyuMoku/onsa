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
