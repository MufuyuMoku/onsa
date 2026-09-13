//! Playlists, manual and smart (SPEC §6.2, §6.3).
//!
//! A manual playlist keeps its own order and may hold the same track more
//! than once. A smart playlist keeps no items at all: it keeps rules, and
//! its contents are worked out from the library whenever they are asked
//! for, so they follow the library as it changes.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::Library;
use crate::error::{Error, Result};
use crate::search::{track_row, TrackRow, TRACK_COLUMNS};
use crate::smart::Rules;

/// What kind of playlist this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlaylistKind {
    /// Tracks the listener put there, in the order they put them.
    Manual,
    /// Tracks that match a set of rules (SPEC §6.3).
    Smart,
}

impl PlaylistKind {
    fn from_str(text: &str) -> Self {
        match text {
            "smart" => Self::Smart,
            _ => Self::Manual,
        }
    }
}

/// A playlist as the interface lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistRow {
    /// Playlist id.
    pub id: i64,
    /// Its name.
    pub name: String,
    /// Manual or smart.
    pub kind: PlaylistKind,
    /// The rules, for a smart playlist.
    pub rules: Option<Rules>,
    /// How many tracks it holds right now.
    pub track_count: usize,
    /// When it was made, in milliseconds since the Unix epoch.
    pub created_at: i64,
    /// When it last changed.
    pub updated_at: i64,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or_default()
}

/// Trims a name and refuses an empty one: a playlist with no name cannot be
/// told from another.
fn check_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::Invalid("a playlist needs a name".into()));
    }
    Ok(name.to_string())
}

impl Library {
    /// Makes a manual playlist and returns its id.
    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        let name = check_name(name)?;
        let now = now_ms();
        self.conn.execute(
            "INSERT INTO playlists (name, kind, rules, created_at, updated_at)
             VALUES (?1, 'manual', NULL, ?2, ?2)",
            params![name, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Makes a smart playlist from a set of rules.
    pub fn create_smart_playlist(&self, name: &str, rules: &Rules) -> Result<i64> {
        let name = check_name(name)?;
        let now = now_ms();
        self.conn.execute(
            "INSERT INTO playlists (name, kind, rules, created_at, updated_at)
             VALUES (?1, 'smart', ?2, ?3, ?3)",
            params![name, rules.to_json()?, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Renames a playlist.
    pub fn rename_playlist(&self, id: i64, name: &str) -> Result<()> {
        let name = check_name(name)?;
        self.conn.execute(
            "UPDATE playlists SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, name, now_ms()],
        )?;
        Ok(())
    }

    /// Replaces the rules of a smart playlist.
    pub fn set_playlist_rules(&self, id: i64, rules: &Rules) -> Result<()> {
        self.conn.execute(
            "UPDATE playlists SET rules = ?2, updated_at = ?3
             WHERE id = ?1 AND kind = 'smart'",
            params![id, rules.to_json()?, now_ms()],
        )?;
        Ok(())
    }

    /// Deletes a playlist and, with it, its items.
    pub fn delete_playlist(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Copies a playlist, items and all, under a new name.
    pub fn duplicate_playlist(&mut self, id: i64, name: &str) -> Result<i64> {
        let name = check_name(name)?;
        let now = now_ms();
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO playlists (name, kind, rules, created_at, updated_at)
             SELECT ?2, kind, rules, ?3, ?3 FROM playlists WHERE id = ?1",
            params![id, name, now],
        )?;
        let copy = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO playlist_items (playlist_id, position, track_id)
             SELECT ?2, position, track_id FROM playlist_items WHERE playlist_id = ?1",
            params![id, copy],
        )?;
        tx.commit()?;
        Ok(copy)
    }

    /// Every playlist, newest change first.
    pub fn playlists(&self) -> Result<Vec<PlaylistRow>> {
        let mut statement = self.conn.prepare(
            "SELECT id, name, kind, rules, created_at, updated_at
             FROM playlists ORDER BY updated_at DESC, name COLLATE NOCASE",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut playlists = Vec::with_capacity(rows.len());
        for (id, name, kind, rules, created_at, updated_at) in rows {
            let kind = PlaylistKind::from_str(&kind);
            let rules = match (kind, rules) {
                // A damaged rule set must not hide the playlist; it comes
                // back empty and the listener can edit it.
                (PlaylistKind::Smart, Some(json)) => Rules::from_json(&json).ok(),
                _ => None,
            };
            playlists.push(PlaylistRow {
                track_count: self.playlist_count(id, kind, rules.as_ref())?,
                id,
                name,
                kind,
                rules,
                created_at,
                updated_at,
            });
        }
        Ok(playlists)
    }

    /// One playlist, or `None` when there is no such playlist.
    pub fn playlist(&self, id: i64) -> Result<Option<PlaylistRow>> {
        let found = self
            .conn
            .query_row(
                "SELECT id, name, kind, rules, created_at, updated_at FROM playlists WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?;
        let Some((id, name, kind, rules, created_at, updated_at)) = found else {
            return Ok(None);
        };
        let kind = PlaylistKind::from_str(&kind);
        let rules = match (kind, rules) {
            (PlaylistKind::Smart, Some(json)) => Rules::from_json(&json).ok(),
            _ => None,
        };
        Ok(Some(PlaylistRow {
            track_count: self.playlist_count(id, kind, rules.as_ref())?,
            id,
            name,
            kind,
            rules,
            created_at,
            updated_at,
        }))
    }

    fn playlist_count(&self, id: i64, kind: PlaylistKind, rules: Option<&Rules>) -> Result<usize> {
        match (kind, rules) {
            (PlaylistKind::Smart, Some(rules)) => Ok(self.smart_tracks(rules)?.len()),
            (PlaylistKind::Smart, None) => Ok(0),
            (PlaylistKind::Manual, _) => Ok(self.conn.query_row(
                "SELECT COUNT(*) FROM playlist_items WHERE playlist_id = ?1",
                [id],
                |row| row.get::<_, i64>(0),
            )? as usize),
        }
    }

    /// What a playlist holds right now. A smart playlist is worked out from
    /// its rules every time, so it follows the library (SPEC §6.3).
    pub fn playlist_tracks(&self, id: i64) -> Result<Vec<TrackRow>> {
        let Some(playlist) = self.playlist(id)? else {
            return Ok(Vec::new());
        };
        match (playlist.kind, playlist.rules) {
            (PlaylistKind::Smart, Some(rules)) => self.smart_tracks(&rules),
            (PlaylistKind::Smart, None) => Ok(Vec::new()),
            (PlaylistKind::Manual, _) => {
                let mut statement = self.conn.prepare(&format!(
                    "SELECT {TRACK_COLUMNS} FROM track_view
                     JOIN playlist_items ON playlist_items.track_id = track_view.id
                     WHERE playlist_items.playlist_id = ?1
                     ORDER BY playlist_items.position"
                ))?;
                let rows = statement
                    .query_map([id], track_row)?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                Ok(rows)
            }
        }
    }

    /// Adds tracks to the end of a manual playlist, in the order given.
    pub fn add_to_playlist(&mut self, id: i64, tracks: &[i64]) -> Result<()> {
        if tracks.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        let mut next: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_items WHERE playlist_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        {
            let mut insert = tx.prepare(
                "INSERT INTO playlist_items (playlist_id, position, track_id) VALUES (?1, ?2, ?3)",
            )?;
            for track in tracks {
                insert.execute(params![id, next, track])?;
                next += 1;
            }
        }
        tx.execute(
            "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
            params![id, now_ms()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Removes the entry at `position`, closing the gap behind it.
    pub fn remove_from_playlist(&mut self, id: i64, position: usize) -> Result<()> {
        let tx = self.conn.transaction()?;
        let gone = tx.execute(
            "DELETE FROM playlist_items WHERE playlist_id = ?1 AND position = ?2",
            params![id, position as i64],
        )?;
        if gone > 0 {
            tx.execute(
                "UPDATE playlist_items SET position = position - 1
                 WHERE playlist_id = ?1 AND position > ?2",
                params![id, position as i64],
            )?;
            tx.execute(
                "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
                params![id, now_ms()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Moves the entry at `from` to `to`, as a drag in the playlist does.
    ///
    /// The whole run is renumbered rather than nudged: a position is part of
    /// the primary key, so shifting rows one at a time would collide with
    /// the neighbour it is moving onto.
    pub fn move_in_playlist(&mut self, id: i64, from: usize, to: usize) -> Result<()> {
        let mut order = self.playlist_track_ids(id)?;
        if order.is_empty() {
            return Ok(());
        }
        let last = order.len() - 1;
        let from = from.min(last);
        let to = to.min(last);
        if from == to {
            return Ok(());
        }
        let track = order.remove(from);
        order.insert(to, track);
        self.set_playlist_tracks(id, &order)
    }

    /// The track ids of a manual playlist, in their stored order.
    fn playlist_track_ids(&self, id: i64) -> Result<Vec<i64>> {
        let mut statement = self.conn.prepare(
            "SELECT track_id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position",
        )?;
        let ids = statement
            .query_map([id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<i64>, _>>()?;
        Ok(ids)
    }

    /// Replaces everything in a manual playlist with `tracks`.
    pub fn set_playlist_tracks(&mut self, id: i64, tracks: &[i64]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM playlist_items WHERE playlist_id = ?1", [id])?;
        {
            let mut insert = tx.prepare(
                "INSERT INTO playlist_items (playlist_id, position, track_id) VALUES (?1, ?2, ?3)",
            )?;
            for (position, track) in tracks.iter().enumerate() {
                insert.execute(params![id, position as i64, track])?;
            }
        }
        tx.execute(
            "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
            params![id, now_ms()],
        )?;
        tx.commit()?;
        Ok(())
    }
}
