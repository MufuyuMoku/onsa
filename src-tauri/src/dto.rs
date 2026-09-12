//! The shapes the interface receives and sends (SPEC §2).
//!
//! The feature crates keep their own types free of serialisation; these
//! mirror them for the command boundary, in camelCase.

use onsa_audio::DeviceInfo;
use onsa_library::{AlbumRow, NameCount, SearchResults, TrackRow, TrackSort};
use serde::{Deserialize, Serialize};

use crate::settings::BandPrefs;

/// A track as listed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackDto {
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
}

impl From<&TrackRow> for TrackDto {
    fn from(row: &TrackRow) -> Self {
        Self {
            id: row.id,
            path: row.path.clone(),
            title: row.title.clone(),
            artist: row.artist.clone(),
            album: row.album.clone(),
            album_artist: row.album_artist.clone(),
            track_number: row.track_number,
            disc_number: row.disc_number,
            year: row.year,
            genre: row.genre.clone(),
            duration_ms: row.duration_ms,
            album_id: row.album_id,
            cover_id: row.cover_id,
            status: row.status.clone(),
        }
    }
}

/// One entry of the play queue: the track, and what tells this entry apart
/// from another entry of the same track.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueEntryDto {
    /// Identity of the entry, stable while it sits in the queue.
    pub entry_id: u64,
    /// The track it plays.
    pub track: TrackDto,
}

/// An album as listed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDto {
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
    /// Tracks that are not missing.
    pub track_count: u32,
}

impl From<&AlbumRow> for AlbumDto {
    fn from(row: &AlbumRow) -> Self {
        Self {
            id: row.id,
            title: row.title.clone(),
            album_artist: row.album_artist.clone(),
            year: row.year,
            cover_id: row.cover_id,
            track_count: row.track_count,
        }
    }
}

/// A name with its track count (artists, folders).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameCountDto {
    /// The name.
    pub name: String,
    /// Tracks that are not missing.
    pub track_count: u32,
}

impl From<&NameCount> for NameCountDto {
    fn from(row: &NameCount) -> Self {
        Self {
            name: row.name.clone(),
            track_count: row.track_count,
        }
    }
}

/// Search results, grouped.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDto {
    /// Matching tracks.
    pub tracks: Vec<TrackDto>,
    /// Albums with a matching track.
    pub albums: Vec<AlbumDto>,
    /// Artists with a matching track.
    pub artists: Vec<NameCountDto>,
}

impl From<&SearchResults> for SearchDto {
    fn from(results: &SearchResults) -> Self {
        Self {
            tracks: results.tracks.iter().map(TrackDto::from).collect(),
            albums: results.albums.iter().map(AlbumDto::from).collect(),
            artists: results.artists.iter().map(NameCountDto::from).collect(),
        }
    }
}

/// Column the track list is sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortKey {
    /// Title.
    Title,
    /// Artist.
    Artist,
    /// Album.
    Album,
    /// Year.
    Year,
    /// Length.
    Duration,
    /// Date added.
    DateAdded,
}

impl From<SortKey> for TrackSort {
    fn from(key: SortKey) -> Self {
        match key {
            SortKey::Title => Self::Title,
            SortKey::Artist => Self::Artist,
            SortKey::Album => Self::Album,
            SortKey::Year => Self::Year,
            SortKey::Duration => Self::Duration,
            SortKey::DateAdded => Self::DateAdded,
        }
    }
}

/// What to play, and from where in it.
#[derive(Debug, Clone, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlayContext {
    /// The whole track list in its current order.
    Library {
        /// Sort column.
        sort: SortKey,
        /// Descending order.
        descending: bool,
        /// Row to start at.
        index: usize,
    },
    /// One album.
    Album {
        /// Album id.
        album_id: i64,
        /// Track to start at.
        index: usize,
    },
    /// Everything by one artist.
    Artist {
        /// The artist as listed.
        name: String,
        /// Track to start at.
        index: usize,
    },
    /// Everything in one genre.
    Genre {
        /// The genre as listed.
        name: String,
        /// Track to start at.
        index: usize,
    },
    /// Everything in one folder.
    Folder {
        /// The folder.
        path: String,
        /// Track to start at.
        index: usize,
    },
    /// A list of tracks, such as search results.
    Tracks {
        /// Track ids in order.
        ids: Vec<i64>,
        /// Track to start at.
        index: usize,
    },
}

/// Where new entries join the queue (SPEC §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueuePlace {
    /// Right after the playing track.
    Next,
    /// At the end of the queue.
    End,
}

/// An output device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    /// Identifier.
    pub id: String,
    /// Name.
    pub name: String,
    /// Whether it is the system default.
    pub is_default: bool,
    /// Its sample rate.
    pub sample_rate: u32,
    /// Its channel count.
    pub channels: u16,
}

impl From<&DeviceInfo> for DeviceDto {
    fn from(device: &DeviceInfo) -> Self {
        Self {
            id: device.id.clone(),
            name: device.name.clone(),
            is_default: device.is_default,
            sample_rate: device.sample_rate,
            channels: device.channels,
        }
    }
}

/// An EQ response curve.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurveDto {
    /// Frequency (Hz) and gain (dB) pairs, on a logarithmic grid.
    pub points: Vec<(f64, f64)>,
    /// The preamp the auto preamp would set, in dB.
    pub auto_preamp_db: f64,
}

/// An imported EQ preset.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoEqDto {
    /// The filters.
    pub bands: Vec<BandPrefs>,
    /// The preset's own preamp, if it has one.
    pub preamp_db: Option<f64>,
    /// Lines that were skipped (details in the log).
    pub skipped: usize,
}
