//! Edits kept in Onsa (SPEC §5.1, §8).
//!
//! The value shown is the override when there is one, otherwise what the
//! file says; `track_view` in the schema does the choosing. Writing overrides
//! into the files themselves is M7; here they are stored and searched.

use rusqlite::{params, OptionalExtension};

use crate::db::Library;
use crate::error::Result;
use crate::search::{track_row, TrackRow, TRACK_COLUMNS};

/// A field that can be overridden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// Title.
    Title,
    /// Artist.
    Artist,
    /// Album.
    Album,
    /// Album artist.
    AlbumArtist,
    /// Track number.
    TrackNumber,
    /// Disc number.
    DiscNumber,
    /// Year.
    Year,
    /// Genre.
    Genre,
    /// Composer.
    Composer,
    /// The song's words, kept in the file's own lyrics tag (SPEC §8, §10).
    Lyrics,
}

impl Field {
    /// The name stored in the `overrides` table; a fixed string per field.
    ///
    /// It is also the name the interface knows the field by, so a suggestion
    /// can say which field it is about without a second vocabulary.
    pub fn name(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Artist => "artist",
            Self::Album => "album",
            Self::AlbumArtist => "album_artist",
            Self::TrackNumber => "track_number",
            Self::DiscNumber => "disc_number",
            Self::Year => "year",
            Self::Genre => "genre",
            Self::Composer => "composer",
            Self::Lyrics => "lyrics",
        }
    }
}

impl Library {
    /// Sets (or, with `None`, clears) an override. Search follows at once.
    pub fn set_override(&mut self, track_id: i64, field: Field, value: Option<&str>) -> Result<()> {
        let tx = self.conn.transaction()?;
        match value {
            Some(value) => tx.execute(
                "INSERT INTO overrides (track_id, field, value, unwritten) VALUES (?1, ?2, ?3, 1)
                 ON CONFLICT(track_id, field) DO UPDATE SET value = excluded.value, unwritten = 1",
                params![track_id, field.name(), value],
            )?,
            None => tx.execute(
                "DELETE FROM overrides WHERE track_id = ?1 AND field = ?2",
                params![track_id, field.name()],
            )?,
        };
        tx.execute("DELETE FROM tracks_fts WHERE rowid = ?1", [track_id])?;
        tx.execute(
            "INSERT INTO tracks_fts (rowid, title, artist, album, album_artist, genre)
             SELECT id, title, artist, album, album_artist, genre FROM track_view WHERE id = ?1",
            [track_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// One track as displayed, overrides applied.
    pub fn track(&self, track_id: i64) -> Result<Option<TrackRow>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {TRACK_COLUMNS} FROM track_view WHERE id = ?1"),
                [track_id],
                track_row,
            )
            .optional()?)
    }

    /// The id of the track at `path`, if the library knows it.
    pub fn track_id(&self, path: &std::path::Path) -> Result<Option<i64>> {
        let Some(text) = path.to_str() else {
            return Ok(None);
        };
        Ok(self
            .conn
            .query_row("SELECT id FROM tracks WHERE path = ?1", [text], |row| {
                row.get(0)
            })
            .optional()?)
    }
}
