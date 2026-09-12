//! Browsing and searching (SPEC §5.3).
//!
//! Lists come a page at a time, so the interface never holds the whole
//! library (SPEC §13.1). Sorting is chosen from fixed enums, never from text,
//! and every value from the user is a bound parameter: nothing typed by the
//! user ever becomes part of the SQL.

use rusqlite::{params, OptionalExtension, Row};

use crate::db::Library;
use crate::error::Result;

/// A track as the interface lists it (displayed values, overrides applied).
#[derive(Debug, Clone, PartialEq)]
pub struct TrackRow {
    /// Track id.
    pub id: i64,
    /// Path of the file.
    pub path: String,
    /// Title.
    pub title: Option<String>,
    /// Artist.
    pub artist: Option<String>,
    /// Album.
    pub album: Option<String>,
    /// Album artist.
    pub album_artist: Option<String>,
    /// Track number.
    pub track_number: Option<u32>,
    /// Disc number.
    pub disc_number: Option<u32>,
    /// Year.
    pub year: Option<i32>,
    /// Genre.
    pub genre: Option<String>,
    /// Length in milliseconds.
    pub duration_ms: Option<i64>,
    /// Album id.
    pub album_id: Option<i64>,
    /// Cover id.
    pub cover_id: Option<i64>,
    /// `ok`, `missing` or `failed`.
    pub status: String,
    /// Container or codec name, for the signal path.
    pub codec: Option<String>,
    /// Sample rate of the file in Hz.
    pub sample_rate: Option<u32>,
    /// Bits per sample, for lossless formats.
    pub bit_depth: Option<u8>,
    /// Channel count of the file.
    pub channels: Option<u8>,
}

/// An album as the interface lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlbumRow {
    /// Album id.
    pub id: i64,
    /// Title.
    pub title: String,
    /// Album artist.
    pub album_artist: String,
    /// Year.
    pub year: Option<i32>,
    /// Cover id.
    pub cover_id: Option<i64>,
    /// Tracks on the album that are not missing.
    pub track_count: u32,
}

/// An artist, genre or folder with its number of tracks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameCount {
    /// The name.
    pub name: String,
    /// Tracks that are not missing.
    pub track_count: u32,
}

/// Search results, grouped (SPEC §5.3).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchResults {
    /// Matching tracks, best first.
    pub tracks: Vec<TrackRow>,
    /// Albums with a matching track, best first.
    pub albums: Vec<AlbumRow>,
    /// Artists with a matching track, best first.
    pub artists: Vec<NameCount>,
}

/// Column to sort the track list by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackSort {
    /// Title.
    #[default]
    Title,
    /// Artist, then album, disc and track.
    Artist,
    /// Album, then disc and track.
    Album,
    /// Album artist, then album, disc and track.
    AlbumArtist,
    /// Year.
    Year,
    /// Genre.
    Genre,
    /// Length.
    Duration,
    /// When the track entered the library.
    DateAdded,
}

impl TrackSort {
    /// The ORDER BY expression; a fixed string per variant.
    fn order_by(self, descending: bool) -> String {
        let direction = if descending { "DESC" } else { "ASC" };
        let then = "album COLLATE NOCASE, disc_number, track_number, title COLLATE NOCASE";
        match self {
            Self::Title => format!("title COLLATE NOCASE {direction}, artist COLLATE NOCASE"),
            Self::Artist => format!("artist COLLATE NOCASE {direction}, {then}"),
            Self::Album => format!("album COLLATE NOCASE {direction}, disc_number, track_number"),
            Self::AlbumArtist => format!("album_artist COLLATE NOCASE {direction}, {then}"),
            Self::Year => format!("year {direction}, {then}"),
            Self::Genre => format!("genre COLLATE NOCASE {direction}, {then}"),
            Self::Duration => format!("duration_ms {direction}, title COLLATE NOCASE"),
            Self::DateAdded => format!("added_at {direction}, {then}"),
        }
    }
}

pub(crate) const TRACK_COLUMNS: &str =
    "id, path, title, artist, album, album_artist, track_number, \
     disc_number, year, genre, duration_ms, album_id, cover_id, status,      codec, sample_rate, bit_depth, channels";

pub(crate) fn track_row(row: &Row<'_>) -> rusqlite::Result<TrackRow> {
    Ok(TrackRow {
        id: row.get(0)?,
        path: row.get(1)?,
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        album_artist: row.get(5)?,
        track_number: row.get(6)?,
        disc_number: row.get(7)?,
        year: row.get(8)?,
        genre: row.get(9)?,
        duration_ms: row.get(10)?,
        album_id: row.get(11)?,
        cover_id: row.get(12)?,
        status: row.get(13)?,
        codec: row.get(14)?,
        sample_rate: row.get(15)?,
        bit_depth: row.get(16)?,
        channels: row.get(17)?,
    })
}

fn album_row(row: &Row<'_>) -> rusqlite::Result<AlbumRow> {
    Ok(AlbumRow {
        id: row.get(0)?,
        title: row.get(1)?,
        album_artist: row.get(2)?,
        year: row.get(3)?,
        cover_id: row.get(4)?,
        track_count: row.get(5)?,
    })
}

fn name_count(row: &Row<'_>) -> rusqlite::Result<NameCount> {
    Ok(NameCount {
        name: row.get(0)?,
        track_count: row.get(1)?,
    })
}

/// Turns what the user typed into an FTS5 query: every word must appear,
/// each as a prefix. Words are quoted, so FTS operators typed by the user
/// are just text. `None` when nothing searchable remains.
pub fn fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split_whitespace()
        .map(|word| word.replace('"', "\"\""))
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{word}\"*"))
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

impl Library {
    /// Searches titles, artists, albums, album artists and genres.
    pub fn search(&self, input: &str, limit: usize) -> Result<SearchResults> {
        let Some(query) = fts_query(input) else {
            return Ok(SearchResults::default());
        };
        let limit = limit as i64;

        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {TRACK_COLUMNS} FROM (
                 SELECT v.*, tracks_fts.rank AS match_rank FROM tracks_fts
                 JOIN track_view v ON v.id = tracks_fts.rowid
                 WHERE tracks_fts MATCH ?1 AND v.status != 'missing')
             ORDER BY match_rank LIMIT ?2"
        ))?;
        let tracks = statement
            .query_map(params![query, limit], track_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut statement = self.conn.prepare_cached(
            "SELECT a.id, a.title, a.album_artist, a.year, a.cover_id, COUNT(*)
             FROM tracks_fts
             JOIN tracks t ON t.id = tracks_fts.rowid
             JOIN albums a ON a.id = t.album_id
             WHERE tracks_fts MATCH ?1 AND t.status != 'missing'
             GROUP BY a.id ORDER BY MIN(tracks_fts.rank) LIMIT ?2",
        )?;
        let albums = statement
            .query_map(params![query, limit], album_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut statement = self.conn.prepare_cached(
            "SELECT v.artist, COUNT(*) FROM tracks_fts
             JOIN track_view v ON v.id = tracks_fts.rowid
             WHERE tracks_fts MATCH ?1 AND v.status != 'missing' AND v.artist IS NOT NULL
             GROUP BY v.artist COLLATE NOCASE ORDER BY MIN(tracks_fts.rank) LIMIT ?2",
        )?;
        let artists = statement
            .query_map(params![query, limit], name_count)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(SearchResults {
            tracks,
            albums,
            artists,
        })
    }

    /// Number of tracks in the library, missing ones included.
    pub fn track_count(&self) -> Result<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// One page of the track list.
    pub fn tracks_page(
        &self,
        sort: TrackSort,
        descending: bool,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<TrackRow>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {TRACK_COLUMNS} FROM track_view ORDER BY {} LIMIT ?1 OFFSET ?2",
            sort.order_by(descending)
        ))?;
        let rows = statement
            .query_map(params![limit as i64, offset as i64], track_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Number of albums with at least one track that is not missing.
    pub fn album_count(&self) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT album_id) FROM tracks
             WHERE album_id IS NOT NULL AND status != 'missing'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// The thumbnail file of a cover, `size` being 128 or 512 (anything
    /// else gets the larger one). `None` when the cover is unknown.
    pub fn cover_file(&self, cover_id: i64, size: u32) -> Result<Option<std::path::PathBuf>> {
        let column = if size <= 128 {
            "thumb_128"
        } else {
            "thumb_512"
        };
        let name: Option<String> = self
            .conn
            .query_row(
                &format!("SELECT {column} FROM covers WHERE id = ?1"),
                [cover_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(name.map(|name| self.covers_dir.join(name)))
    }

    /// One page of albums, by album artist then title.
    pub fn albums_page(&self, offset: usize, limit: usize) -> Result<Vec<AlbumRow>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT a.id, a.title, a.album_artist, a.year, a.cover_id, COUNT(t.id)
             FROM albums a JOIN tracks t ON t.album_id = a.id AND t.status != 'missing'
             GROUP BY a.id
             ORDER BY a.album_artist COLLATE NOCASE, a.year, a.title COLLATE NOCASE
             LIMIT ?1 OFFSET ?2",
        )?;
        let rows = statement
            .query_map(params![limit as i64, offset as i64], album_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// The tracks of an album, in disc and track order.
    pub fn album_tracks(&self, album_id: i64) -> Result<Vec<TrackRow>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {TRACK_COLUMNS} FROM track_view WHERE album_id = ?1
             ORDER BY disc_number, track_number, title COLLATE NOCASE"
        ))?;
        let rows = statement
            .query_map([album_id], track_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// One page of artists with their track counts.
    pub fn artists_page(&self, offset: usize, limit: usize) -> Result<Vec<NameCount>> {
        self.grouped_page("artist", offset, limit)
    }

    /// One page of genres with their track counts.
    pub fn genres_page(&self, offset: usize, limit: usize) -> Result<Vec<NameCount>> {
        self.grouped_page("genre", offset, limit)
    }

    /// `column` is one of two fixed names above, never user input.
    fn grouped_page(&self, column: &str, offset: usize, limit: usize) -> Result<Vec<NameCount>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {column}, COUNT(*) FROM track_view
             WHERE {column} IS NOT NULL AND status != 'missing'
             GROUP BY {column} COLLATE NOCASE ORDER BY {column} COLLATE NOCASE
             LIMIT ?1 OFFSET ?2"
        ))?;
        let rows = statement
            .query_map(params![limit as i64, offset as i64], name_count)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Number of distinct artists.
    pub fn artist_count(&self) -> Result<u64> {
        self.grouped_count("artist")
    }

    /// Number of distinct genres.
    pub fn genre_count(&self) -> Result<u64> {
        self.grouped_count("genre")
    }

    /// `column` is one of the two fixed names above, never user input.
    fn grouped_count(&self, column: &str) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(DISTINCT {column} COLLATE NOCASE) FROM track_view
                 WHERE {column} IS NOT NULL AND status != 'missing'"
            ),
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// The tracks of one artist, in album order.
    pub fn artist_tracks(&self, artist: &str) -> Result<Vec<TrackRow>> {
        self.grouped_tracks("artist", artist)
    }

    /// The tracks of one genre, in album order.
    pub fn genre_tracks(&self, genre: &str) -> Result<Vec<TrackRow>> {
        self.grouped_tracks("genre", genre)
    }

    fn grouped_tracks(&self, column: &str, value: &str) -> Result<Vec<TrackRow>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {TRACK_COLUMNS} FROM track_view
             WHERE {column} = ?1 COLLATE NOCASE AND status != 'missing'
             ORDER BY album COLLATE NOCASE, disc_number, track_number, title COLLATE NOCASE"
        ))?;
        let rows = statement
            .query_map([value], track_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Number of folders that hold at least one track.
    pub fn directory_count(&self) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT onsa_parent(path)) FROM tracks WHERE status != 'missing'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// One page of the folders tracks sit in, with their track counts.
    pub fn directories_page(&self, offset: usize, limit: usize) -> Result<Vec<NameCount>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT onsa_parent(path) AS dir, COUNT(*) FROM tracks
             WHERE status != 'missing'
             GROUP BY dir ORDER BY dir LIMIT ?1 OFFSET ?2",
        )?;
        let rows = statement
            .query_map(params![limit as i64, offset as i64], name_count)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// The tracks directly inside one folder, in track order.
    pub fn directory_tracks(&self, dir: &str) -> Result<Vec<TrackRow>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {TRACK_COLUMNS} FROM track_view
             WHERE onsa_parent(path) = ?1 AND status != 'missing'
             ORDER BY disc_number, track_number, title COLLATE NOCASE"
        ))?;
        let rows = statement
            .query_map([dir], track_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// The watched folders with their track counts.
    pub fn folders(&self) -> Result<Vec<NameCount>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT f.path, COUNT(t.id) FROM folders f
             LEFT JOIN tracks t ON t.folder_id = f.id AND t.status != 'missing'
             GROUP BY f.id ORDER BY f.path",
        )?;
        let rows = statement
            .query_map([], name_count)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_input_becomes_quoted_prefix_terms() {
        assert_eq!(
            fts_query("  lampu  kota "),
            Some("\"lampu\"* \"kota\"*".into())
        );
        assert_eq!(fts_query("夜明け"), Some("\"夜明け\"*".into()));
        // FTS syntax typed by the user is just text.
        assert_eq!(
            fts_query("a\" OR b"),
            Some("\"a\"\"\"* \"OR\"* \"b\"*".into())
        );
        assert_eq!(fts_query("   "), None);
    }
}
