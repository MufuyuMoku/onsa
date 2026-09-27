//! The folders the library is made of: letting one go, following one that
//! has moved, and leaving part of one out (SPEC §15, M13).
//!
//! None of this touches a single file on disk. A folder that leaves takes
//! its tracks out of the library and nothing else; a folder that moved is
//! the same folder with a new address, so the tracks in it stay the tracks
//! they were — with the play counts, the playlists they are on and the
//! edits not yet written to them, all of which hang from a track's id and
//! not from where its file happens to sit.

use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension};

use crate::db::Library;
use crate::error::{Error, Result};
use crate::scan::{now_ms, path_text};

impl Library {
    /// Follows a library folder to where it is now.
    ///
    /// A drive letter that changed, a folder somebody moved, the same disk
    /// mounted somewhere else: the library folder and every track under it
    /// take the new address, keeping their ids. Nothing on disk is touched.
    ///
    /// Answers with how many tracks moved with it.
    pub fn move_folder(&mut self, from: &Path, to: &Path) -> Result<u64> {
        let from = std::path::absolute(from).map_err(|source| Error::io(from, source))?;
        let to = std::path::absolute(to).map_err(|source| Error::io(to, source))?;
        let (from_text, to_text) = (path_text(&from)?, path_text(&to)?);
        if from_text == to_text {
            return Ok(0);
        }
        if !to.is_dir() {
            return Err(Error::Invalid(format!(
                "there is no folder at {}",
                to.display()
            )));
        }

        let folder_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM folders WHERE path = ?1",
                [&from_text],
                |row| row.get(0),
            )
            .optional()?;
        let Some(folder_id) = folder_id else {
            return Err(Error::Invalid(format!(
                "{} is not one of the library folders",
                from.display()
            )));
        };
        let taken: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM folders WHERE path = ?1",
                [&to_text],
                |row| row.get(0),
            )
            .optional()?;
        if taken.is_some() {
            return Err(Error::Invalid(format!(
                "{} is already a library folder",
                to.display()
            )));
        }

        // The paths of the tracks are rewritten by their beginning, so a
        // folder's own separator is kept: what changes is the address, not
        // the shape of it.
        let cut = from_text.chars().count() as i64 + 1;
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE folders SET path = ?2 WHERE id = ?1",
            params![folder_id, to_text],
        )?;
        let moved = tx.execute(
            "UPDATE tracks SET path = ?2 || substr(path, ?3) WHERE folder_id = ?1",
            params![folder_id, to_text, cut],
        )?;
        tx.execute(
            "UPDATE folder_exclusions SET path = ?2 || substr(path, ?3) WHERE folder_id = ?1",
            params![folder_id, to_text, cut],
        )?;
        // A track whose file is where it should now be is here again. One
        // that is not stays missing, and the next scan will say so.
        {
            let mut statement = tx.prepare(
                "SELECT id, path FROM tracks WHERE folder_id = ?1 AND status = 'missing'",
            )?;
            let rows: Vec<(i64, String)> = statement
                .query_map([folder_id], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            drop(statement);
            for (id, path) in rows {
                if Path::new(&path).is_file() {
                    tx.execute("UPDATE tracks SET status = 'ok' WHERE id = ?1", [id])?;
                }
            }
        }
        tx.commit()?;
        tracing::info!(
            from = %from.display(),
            to = %to.display(),
            tracks = moved,
            "a library folder moved"
        );
        Ok(moved as u64)
    }

    /// Leaves a subfolder out of scanning, and takes what is in it out of
    /// the library.
    ///
    /// The files stay where they are; Onsa simply stops looking there.
    /// Answers with how many tracks left the library.
    pub fn exclude(&mut self, path: &Path) -> Result<u64> {
        let path = std::path::absolute(path).map_err(|source| Error::io(path, source))?;
        let text = path_text(&path)?;
        let Some(folder_id) = self.folder_holding(&path)? else {
            return Err(Error::Invalid(format!(
                "{} is not inside a library folder",
                path.display()
            )));
        };
        if self
            .conn
            .query_row("SELECT id FROM folders WHERE path = ?1", [&text], |row| {
                row.get::<_, i64>(0)
            })
            .optional()?
            .is_some()
        {
            return Err(Error::Invalid(
                "a library folder is left with \"stop using this folder\", not with this".into(),
            ));
        }

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO folder_exclusions (folder_id, path, added_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO NOTHING",
            params![folder_id, text, now_ms()],
        )?;
        let gone = tx.execute(
            "DELETE FROM tracks WHERE path LIKE ?1 ESCAPE '\\'",
            [crate::scan::below(&text)],
        )?;
        tx.commit()?;
        self.forget_orphans()?;
        tracing::info!(folder = %path.display(), tracks = gone, "a subfolder is left out of scanning");
        Ok(gone as u64)
    }

    /// Looks in a subfolder again. The next scan brings back what is in it.
    pub fn include(&mut self, path: &Path) -> Result<bool> {
        let path = std::path::absolute(path).map_err(|source| Error::io(path, source))?;
        let removed = self.conn.execute(
            "DELETE FROM folder_exclusions WHERE path = ?1",
            [path_text(&path)?],
        )?;
        Ok(removed > 0)
    }

    /// The subfolders left out of scanning, in order.
    pub fn exclusions(&self) -> Result<Vec<PathBuf>> {
        let mut statement = self
            .conn
            .prepare_cached("SELECT path FROM folder_exclusions ORDER BY path")?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows.into_iter().map(PathBuf::from).collect())
    }

    /// Whether this path is inside a subfolder that is left out.
    pub(crate) fn is_excluded(&self, path: &Path) -> Result<bool> {
        let Ok(text) = path_text(path) else {
            return Ok(false);
        };
        let mut statement = self
            .conn
            .prepare_cached("SELECT path FROM folder_exclusions")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows {
            let excluded = row?;
            if text == excluded || path.starts_with(Path::new(&excluded)) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Which library folder a path sits inside, if any.
    fn folder_holding(&self, path: &Path) -> Result<Option<i64>> {
        let mut statement = self.conn.prepare_cached("SELECT id, path FROM folders")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut best: Option<(usize, i64)> = None;
        for row in rows {
            let (id, folder) = row?;
            let folder = Path::new(&folder);
            if path.starts_with(folder) {
                let depth = folder.components().count();
                if best.is_none_or(|(deepest, _)| depth > deepest) {
                    best = Some((depth, id));
                }
            }
        }
        Ok(best.map(|(_, id)| id))
    }

    /// Albums nothing points at any more.
    fn forget_orphans(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM albums WHERE id NOT IN (SELECT album_id FROM tracks WHERE album_id IS NOT NULL)",
            [],
        )?;
        Ok(())
    }
}
