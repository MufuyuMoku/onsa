//! Editing one song's tags (SPEC §8).
//!
//! The Tidy page changes many songs at once and asks for the folder, the
//! summary and the button before it does. This is the other half: one song,
//! its fields in front of the listener, typed by hand.
//!
//! It is the same machinery underneath. An edit here is an override like
//! any other: it is kept in the database first, it lands in the same
//! history and can be taken back the same way, and putting it into the file
//! itself stays a separate thing the listener asks for. The folder a run is
//! held to is the song's own folder, so an editor open on one song cannot
//! touch anything else even by accident.

use std::path::PathBuf;

use onsa_library::edits::{Change, Scope};
use onsa_library::Field;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::ErrorCode;
use crate::library::{LibraryService, CHANGED_EVENT};
use crate::tidy::{ReportDto, SummaryDto};

/// The fields this editor offers, in the order it shows them.
///
/// Track total, disc total and comment are missing because Onsa cannot hold
/// an override for them yet; they arrive with the rest of §8 (see
/// `docs/PROGRESS.md`).
pub const FIELDS: &[&str] = &[
    "title",
    "artist",
    "album",
    "album_artist",
    "track_number",
    "disc_number",
    "year",
    "genre",
    "composer",
    "lyrics",
];

/// One field, as the editor shows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDto {
    /// Its name, which is the name the rest of Onsa uses for it.
    pub name: String,
    /// What is shown: Onsa's value when there is one, else the file's.
    pub value: Option<String>,
    /// Whether that value is Onsa's rather than the file's.
    pub edited: bool,
    /// Whether the file itself still says something else.
    pub unwritten: bool,
}

/// What a file says about its own loudness. Read only (SPEC §8).
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GainDto {
    /// Track gain in dB.
    pub track_gain_db: Option<f64>,
    /// Track peak, linear.
    pub track_peak: Option<f64>,
    /// Album gain in dB.
    pub album_gain_db: Option<f64>,
    /// Album peak, linear.
    pub album_peak: Option<f64>,
}

/// One song, as the editor sees it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackFieldsDto {
    /// Which song.
    pub track_id: i64,
    /// Where it is, which is the one thing the listener cannot change here.
    pub path: String,
    /// The fields, in the order they are shown.
    pub fields: Vec<FieldDto>,
    /// What the file says about its loudness.
    pub gain: GainDto,
    /// Whether anything Onsa holds has yet to reach the file.
    pub any_unwritten: bool,
}

/// One value the listener typed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDto {
    /// The field's name.
    pub field: String,
    /// What was typed. Empty means the field is to be emptied.
    pub value: Option<String>,
}

/// The folder one song is held to: its own, and nothing else.
fn around(path: &str) -> Scope {
    let folder = PathBuf::from(path)
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(path));
    Scope::folder(folder).take(1)
}

/// A field name the editor knows, refused otherwise.
fn field_of(name: &str) -> Result<Field, ErrorCode> {
    Ok(match name {
        "title" => Field::Title,
        "artist" => Field::Artist,
        "album" => Field::Album,
        "album_artist" => Field::AlbumArtist,
        "track_number" => Field::TrackNumber,
        "disc_number" => Field::DiscNumber,
        "year" => Field::Year,
        "genre" => Field::Genre,
        "composer" => Field::Composer,
        "lyrics" => Field::Lyrics,
        _ => return Err(ErrorCode::Library),
    })
}

/// Everything the editor shows about one song.
#[tauri::command]
pub fn track_fields(
    library: State<'_, LibraryService>,
    track_id: i64,
) -> Result<TrackFieldsDto, ErrorCode> {
    let Some(track) = library.read(|library| library.track(track_id))? else {
        return Err(ErrorCode::Library);
    };
    let edits = library.read(|library| library.overrides_of(track_id))?;
    let gain = library
        .read(|library| library.replay_gain(track_id))?
        .unwrap_or_default();

    let mut fields = Vec::with_capacity(FIELDS.len());
    for name in FIELDS {
        let edit = edits.iter().find(|edit| edit.field == *name);
        // The library's view already answers with the override where there
        // is one — but only for the fields every list query needs. The
        // composer and the lyrics are not among them (a song's words in
        // every row of a list would be absurd), so for those the editor
        // reads the file itself, which is what it is editing anyway.
        let shown = match *name {
            "title" => track.title.clone(),
            "artist" => track.artist.clone(),
            "album" => track.album.clone(),
            "album_artist" => track.album_artist.clone(),
            "track_number" => track.track_number.map(|value| value.to_string()),
            "disc_number" => track.disc_number.map(|value| value.to_string()),
            "year" => track.year.map(|value| value.to_string()),
            "genre" => track.genre.clone(),
            "composer" | "lyrics" => match edit {
                Some(edit) => edit.value.clone(),
                None => onsa_library::write::read_field(std::path::Path::new(&track.path), name)
                    .unwrap_or_default(),
            },
            _ => None,
        };
        fields.push(FieldDto {
            name: (*name).to_string(),
            value: shown,
            edited: edit.is_some(),
            unwritten: edit.is_some_and(|edit| edit.unwritten),
        });
    }

    Ok(TrackFieldsDto {
        track_id,
        path: track.path.clone(),
        any_unwritten: fields.iter().any(|field| field.unwritten),
        fields,
        gain: GainDto {
            track_gain_db: gain.track_gain_db,
            track_peak: gain.track_peak,
            album_gain_db: gain.album_gain_db,
            album_peak: gain.album_peak,
        },
    })
}

/// Puts what was typed into the database, as a run that can be taken back.
///
/// Nothing reaches the file here. That is [`track_write`], which the
/// listener asks for separately (SPEC §8).
#[tauri::command]
pub fn track_apply(
    app: AppHandle,
    note: String,
    track_id: i64,
    fields: Vec<TypedDto>,
) -> Result<ReportDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    let Some(track) = library.read(|library| library.track(track_id))? else {
        return Err(ErrorCode::Library);
    };
    let scope = around(&track.path);
    let mut changes = Vec::with_capacity(fields.len());
    for typed in fields {
        changes.push(Change {
            track_id,
            field: field_of(&typed.field)?,
            value: typed.value.filter(|value| !value.trim().is_empty()),
        });
    }
    let report = library.write(|library| library.apply_edits(&note, &scope, &changes))?;
    // The lists showing this song are showing what it used to say.
    let _ = app.emit(CHANGED_EVENT, ());
    Ok(report.into())
}

/// What applying those values would do, before there is a button to press.
#[tauri::command]
pub fn track_preview(
    library: State<'_, LibraryService>,
    track_id: i64,
    fields: Vec<TypedDto>,
) -> Result<SummaryDto, ErrorCode> {
    let Some(track) = library.read(|library| library.track(track_id))? else {
        return Err(ErrorCode::Library);
    };
    let scope = around(&track.path);
    let mut changes = Vec::with_capacity(fields.len());
    for typed in fields {
        changes.push(Change {
            track_id,
            field: field_of(&typed.field)?,
            value: typed.value.filter(|value| !value.trim().is_empty()),
        });
    }
    Ok(library
        .read(|library| library.preview_edits(&scope, &changes))?
        .into())
}

/// Writes what Onsa holds into the file itself (SPEC §8).
#[tauri::command]
pub async fn track_write(
    app: AppHandle,
    note: String,
    track_id: i64,
) -> Result<ReportDto, ErrorCode> {
    let app_again = app.clone();
    // Rewriting a file is not work for the thread that answers the window.
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<LibraryService>();
        let Some(track) = library.read(|library| library.track(track_id))? else {
            return Err(ErrorCode::Library);
        };
        let scope = around(&track.path);
        library.write(|library| library.write_to_files(&note, &scope, &[track_id]))
    })
    .await
    .map_err(|_| ErrorCode::Library)??;
    let _ = app_again.emit(CHANGED_EVENT, ());
    Ok(outcome.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_song_is_held_to_its_own_folder_and_nothing_else() {
        let scope = around(if cfg!(windows) {
            r"C:\Music\Album\01 Satu.flac"
        } else {
            "/music/Album/01 Satu.flac"
        });
        assert_eq!(scope.limit, 1, "one song is one song");
        let inside = if cfg!(windows) {
            r"C:\Music\Album\02 Dua.flac"
        } else {
            "/music/Album/02 Dua.flac"
        };
        let outside = if cfg!(windows) {
            r"C:\Music\Lain\03 Tiga.flac"
        } else {
            "/music/Lain/03 Tiga.flac"
        };
        assert!(scope.allows(std::path::Path::new(inside)));
        assert!(!scope.allows(std::path::Path::new(outside)));
    }

    #[test]
    fn every_field_the_editor_offers_is_one_the_library_can_hold() {
        for name in FIELDS {
            field_of(name).unwrap_or_else(|_| panic!("{name} should be a field"));
        }
        assert!(field_of("nonsense").is_err());
        // The three §8 has yet to reach are refused rather than silently
        // doing nothing.
        for later in ["track_total", "disc_total", "comment"] {
            assert!(field_of(later).is_err(), "{later}");
        }
    }
}
