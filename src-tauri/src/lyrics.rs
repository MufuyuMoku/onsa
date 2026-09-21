//! A song's words, from wherever they can be had (SPEC §10).
//!
//! Four places are looked at, in this order, and the first one that has
//! anything wins:
//!
//! 1. what the listener typed themselves, which is a tag edit like any other;
//! 2. the `.lrc` beside the song;
//! 3. the words in the song's own tags;
//! 4. LRCLIB, and only when the listener has allowed it.
//!
//! The first three cost a small file read and happen every time the song
//! comes round. The fourth is the only one that leaves this machine, so it
//! is off until it is turned on, is remembered once it has answered — a
//! "there are none" included, so a song is not asked about forever — and
//! never happens on the thread the interface is waiting on. A service that
//! is slow costs a thread of its own and nothing else: the words that are
//! already here are shown at once, and the ones from away arrive when they
//! arrive.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use onsa_library::lyrics::NOTHING;
use onsa_library::Field;
use onsa_lyrics::{beside, lrc, lrclib};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::ErrorCode;
use crate::library::LibraryService;
use crate::net::{self, Service};
use crate::settings::{self, LyricsPrefs};

/// What the interface listens for when words arrive from away.
pub const LYRICS_EVENT: &str = "lyrics://arrived";

/// Where a song's words came from.
///
/// A fixed word rather than a sentence: the interface says it in the
/// listener's own language (SPEC §9.6).
const EDITED: &str = "edited";
const FILE: &str = "file";
const TAG: &str = "tag";
const LRCLIB: &str = "lrclib";

/// One line, as the interface draws it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineDto {
    /// When it is sung, or `None` for a line with no time of its own.
    pub at_ms: Option<i64>,
    /// The words.
    pub text: String,
}

/// A song's words, and everything the interface needs to show them.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsDto {
    /// Which track these belong to, so a late answer can be matched up.
    pub track_id: i64,
    /// Where they came from, or `none` when there are none.
    pub source: String,
    /// Whether they follow the song.
    pub synced: bool,
    /// The lines.
    pub lines: Vec<LineDto>,
    /// What the listener tuned, in milliseconds. Positive holds them back.
    pub offset_ms: i64,
    /// Whether a service is being asked about this song right now.
    pub looking: bool,
    /// Whether asking a service is allowed at all.
    pub online: bool,
    /// Whether a service could still be asked about this song.
    pub can_ask: bool,
    /// Whether there is already a `.lrc` beside the song.
    pub beside: bool,
    /// Whether the last look for this song could not reach the service.
    ///
    /// Kept apart from "there are none": a song with no words is a fact
    /// worth remembering, while a service that was down is worth nothing
    /// except saying so, so that nobody concludes the song has no words.
    pub unreachable: bool,
}

impl LyricsDto {
    /// Nothing at all, for a track the library does not know.
    fn nothing(track_id: i64) -> Self {
        Self {
            track_id,
            source: NOTHING.to_string(),
            synced: false,
            lines: Vec::new(),
            offset_ms: 0,
            looking: false,
            online: false,
            can_ask: false,
            beside: false,
            unreachable: false,
        }
    }
}

/// The lookups going on, and the right to abandon them.
///
/// Turning the internet off stops the lookups that are already under way,
/// in the only sense that matters: whatever they come back with is dropped
/// rather than stored or shown. The thread itself ends when its request
/// times out, and holds nothing up in the meantime.
#[derive(Default)]
pub struct Lookups {
    going: Mutex<HashSet<i64>>,
    /// Songs whose last look could not reach the service. Nothing is
    /// written to the library about them; this lives only as long as Onsa
    /// is running, and clears the moment an answer arrives.
    unreachable: Mutex<HashSet<i64>>,
    era: AtomicU64,
}

impl Lookups {
    /// Whether this track is already being asked about, and takes it on if
    /// it is not.
    fn take(&self, track_id: i64) -> bool {
        let mut going = lock(&self.going);
        going.insert(track_id)
    }

    /// Gives the track back, whatever came of it.
    fn give_back(&self, track_id: i64) {
        lock(&self.going).remove(&track_id);
    }

    /// Whether this track is being asked about.
    fn busy_with(&self, track_id: i64) -> bool {
        lock(&self.going).contains(&track_id)
    }

    /// Remembers that the service could not be reached about this song.
    fn note_unreachable(&self, track_id: i64) {
        lock(&self.unreachable).insert(track_id);
    }

    /// Forgets that, because something was heard back.
    fn heard_back(&self, track_id: i64) {
        lock(&self.unreachable).remove(&track_id);
    }

    /// Whether the last look for this song ended without an answer.
    fn was_unreachable(&self, track_id: i64) -> bool {
        lock(&self.unreachable).contains(&track_id)
    }

    /// The era a lookup belongs to.
    fn era(&self) -> u64 {
        self.era.load(Ordering::SeqCst)
    }

    /// Ends the era: everything going on now is abandoned.
    pub fn abandon(&self) {
        self.era.fetch_add(1, Ordering::SeqCst);
        lock(&self.going).clear();
        lock(&self.unreachable).clear();
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the library knows about a song, as far as lyrics care.
struct Song {
    path: PathBuf,
    ask: lrclib::Ask,
}

/// The words themselves, and where they were found.
struct Found {
    source: String,
    text: Option<String>,
}

/// The lyrics settings as they stand.
pub fn prefs(library: &LibraryService) -> Result<LyricsPrefs, ErrorCode> {
    library.read(|library| Ok(settings::load::<LyricsPrefs>(library, settings::LYRICS_KEY)))
}

/// Everything about a song that the four sources need.
fn song_of(library: &LibraryService, track_id: i64) -> Result<Option<Song>, ErrorCode> {
    let Some(track) = library.read(|library| library.track(track_id))? else {
        return Ok(None);
    };
    Ok(Some(Song {
        path: PathBuf::from(&track.path),
        ask: lrclib::Ask {
            artist: track.artist.clone().unwrap_or_default(),
            title: track.title.clone().unwrap_or_default(),
            album: track.album.clone(),
            duration_s: track.duration_ms.map(|ms| ms / 1_000),
        },
    }))
}

/// Looks in every place that costs nothing, nearest first.
fn look_nearby(library: &LibraryService, track_id: i64, path: &Path) -> Result<Found, ErrorCode> {
    // What the listener typed themselves comes before anything a file or a
    // service says: it is their correction, and the point of a correction.
    let edited = library.read(|library| library.override_value(track_id, Field::Lyrics))?;
    if let Some(words) = edited.filter(|words| !words.trim().is_empty()) {
        return Ok(Found {
            source: EDITED.to_string(),
            text: Some(words),
        });
    }
    if let Some(words) = beside::read_beside(path) {
        if !words.trim().is_empty() {
            return Ok(Found {
                source: FILE.to_string(),
                text: Some(words),
            });
        }
    }
    if let Some(words) = beside::read_embedded(path) {
        return Ok(Found {
            source: TAG.to_string(),
            text: Some(words),
        });
    }
    Ok(Found {
        source: NOTHING.to_string(),
        text: None,
    })
}

/// Builds the answer for one track, without asking anything of the network.
fn assemble(app: &AppHandle, track_id: i64, start_looking: bool) -> Result<LyricsDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    let Some(song) = song_of(&library, track_id)? else {
        return Ok(LyricsDto::nothing(track_id));
    };
    let prefs = prefs(&library)?;
    let lookups = app.state::<Arc<Lookups>>();
    let kept = library.read(|library| library.lyrics(track_id))?;
    let offset_ms = kept.as_ref().map(|kept| kept.offset_ms).unwrap_or(0);
    let asked = kept.as_ref().is_some_and(|kept| kept.was_asked());

    let mut found = look_nearby(&library, track_id, &song.path)?;
    if found.text.is_none() {
        if let Some(words) = kept.as_ref().and_then(|kept| kept.words()) {
            found = Found {
                source: kept
                    .as_ref()
                    .and_then(|kept| kept.source.clone())
                    .unwrap_or_else(|| LRCLIB.to_string()),
                text: Some(words.to_string()),
            };
        }
    }

    // A service is worth asking only when there is nothing here, the
    // listener has allowed it, it has not answered before, and the song
    // says enough about itself to ask a sensible question.
    let worth_asking = found.text.is_none() && prefs.online && !asked && song.ask.is_enough();
    let looking = app.state::<Arc<Lookups>>().busy_with(track_id);
    if worth_asking && start_looking && !looking {
        ask_away(app.clone(), track_id, song.ask.clone());
    }

    let parsed = found.text.as_deref().map(lrc::parse).unwrap_or_default();
    Ok(LyricsDto {
        track_id,
        source: found.source,
        synced: parsed.synced(),
        lines: parsed
            .lines
            .iter()
            .map(|line| LineDto {
                at_ms: line.at_ms,
                text: line.text.clone(),
            })
            .collect(),
        offset_ms,
        looking: looking || (worth_asking && start_looking),
        online: prefs.online,
        can_ask: found.text.is_none() && song.ask.is_enough(),
        beside: beside::beside_exists(&song.path),
        unreachable: found.text.is_none() && lookups.was_unreachable(track_id),
    })
}

/// What came of asking a service.
///
/// The difference that matters is between "it knows of no words for this
/// song" and "it could not be asked". The first is worth remembering, so a
/// song is not asked about every time it plays. The second is worth
/// nothing: an outage is not an answer, and a song must not be marked as
/// wordless because the network was down for a minute.
enum Answer {
    /// It answered: these words, or none it knows of.
    Said(Option<lrclib::Found>),
    /// It could not be asked at all.
    Unreachable,
}

/// Asks LRCLIB, on a thread of its own.
fn ask_away(app: AppHandle, track_id: i64, ask: lrclib::Ask) {
    let lookups = app.state::<Arc<Lookups>>().inner().clone();
    if !lookups.take(track_id) {
        return;
    }
    let era = lookups.era();
    let giving_back = lookups.clone();
    let unstarted = lookups.clone();
    let spawned = std::thread::Builder::new()
        .name("onsa-lyrics".into())
        .spawn(move || {
            let answer = fetch(&ask);
            lookups.give_back(track_id);
            // The listener may have turned the internet off while this was
            // in the air. Then it never happened: nothing is stored and
            // nothing is shown.
            if lookups.era() != era {
                tracing::debug!(track_id, "a lyrics lookup was abandoned");
                return;
            }
            let library = app.state::<LibraryService>();
            let still_allowed = prefs(&library).map(|prefs| prefs.online).unwrap_or(false);
            if !still_allowed {
                return;
            }
            let said = match answer {
                Answer::Said(said) => said,
                Answer::Unreachable => {
                    // Nothing is written down about the song itself, so it
                    // can be asked about again once the service is back —
                    // but the window is told, so it does not say the song
                    // has no words.
                    tracing::info!(track_id, "a lyrics service could not be reached");
                    giving_back.note_unreachable(track_id);
                    let _ = app.emit(LYRICS_EVENT, track_id);
                    return;
                }
            };
            giving_back.heard_back(track_id);
            let (text, synced) = match &said {
                Some(found) => (found.best().map(str::to_string), found.synced.is_some()),
                None => (None, false),
            };
            let source = if text.is_some() { LRCLIB } else { NOTHING };
            if let Err(error) = library
                .write(|library| library.remember_lyrics(track_id, source, text.as_deref(), synced))
            {
                tracing::warn!(track_id, "the words could not be kept: {error:?}");
            }
            if let Some(words) = text.as_deref() {
                keep_beside(&library, track_id, words);
            }
            tracing::info!(track_id, found = text.is_some(), "a lyrics lookup finished");
            let _ = app.emit(LYRICS_EVENT, track_id);
        });
    if let Err(error) = spawned {
        tracing::warn!("a lyrics lookup could not be started: {error}");
        unstarted.give_back(track_id);
    }
}

/// Asks the service, the exact question first and the broad one after.
fn fetch(ask: &lrclib::Ask) -> Answer {
    let exact: Vec<(String, String)> = ask.get_query();
    let query: Vec<(&str, &str)> = exact
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    match net::ask_json(Service::LrcLib, lrclib::GET, &query) {
        // A service that refuses — too busy, fallen over, asked too often —
        // has said nothing about the song. Remembering that as "this one
        // has no words" would be remembering the weather.
        Ok(answer) if !answer.is_an_answer() => {
            tracing::debug!(status = answer.status, "lrclib refused to answer");
            return Answer::Unreachable;
        }
        Ok(answer) => {
            if let Some(found) = lrclib::read_get(&answer.body) {
                return Answer::Said(Some(found));
            }
        }
        Err(error) => {
            tracing::debug!("lrclib could not be asked: {error}");
            return Answer::Unreachable;
        }
    }
    let broad: Vec<(String, String)> = ask.search_query();
    let query: Vec<(&str, &str)> = broad
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    match net::ask_json(Service::LrcLib, lrclib::SEARCH, &query) {
        Ok(answer) if !answer.is_an_answer() => {
            tracing::debug!(status = answer.status, "lrclib refused to search");
            Answer::Unreachable
        }
        Ok(answer) => Answer::Said(lrclib::read_search(&answer.body, ask)),
        Err(error) => {
            tracing::debug!("lrclib could not be searched: {error}");
            Answer::Unreachable
        }
    }
}

/// Writes the words beside the song, when the listener asked for that.
fn keep_beside(library: &LibraryService, track_id: i64, words: &str) {
    let wanted = prefs(library)
        .map(|prefs| prefs.write_beside)
        .unwrap_or(false);
    if !wanted {
        return;
    }
    let Ok(Some(song)) = song_of(library, track_id) else {
        return;
    };
    match beside::write_beside(&song.path, words) {
        Ok(path) => tracing::info!("the words were written beside the song at {path:?}"),
        Err(error) => tracing::warn!("the words could not be written beside the song: {error}"),
    }
}

/// A song's words, and a lookup started if one is worth starting.
#[tauri::command]
pub fn lyrics_for(app: AppHandle, track_id: i64) -> Result<LyricsDto, ErrorCode> {
    assemble(&app, track_id, true)
}

/// The same, without starting anything: what the interface asks for after
/// being told a lookup finished.
#[tauri::command]
pub fn lyrics_state(app: AppHandle, track_id: i64) -> Result<LyricsDto, ErrorCode> {
    assemble(&app, track_id, false)
}

/// Asks the service again, forgetting what it said last time.
///
/// The listener's own doing, so it is allowed even for a song that has been
/// asked about before — but not when the internet is off.
#[tauri::command]
pub fn lyrics_look_again(app: AppHandle, track_id: i64) -> Result<LyricsDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    if !prefs(&library)?.online {
        return Err(ErrorCode::Offline);
    }
    library.write(|library| library.forget_lyrics(track_id))?;
    assemble(&app, track_id, true)
}

/// Moves the words against the song, and remembers it.
#[tauri::command]
pub fn lyrics_offset(
    app: AppHandle,
    track_id: i64,
    offset_ms: i64,
) -> Result<LyricsDto, ErrorCode> {
    // Half a minute either way is more than any lyrics file is out by, and
    // stops a stuck button from writing nonsense.
    let offset_ms = offset_ms.clamp(-30_000, 30_000);
    app.state::<LibraryService>()
        .write(|library| library.set_lyrics_offset(track_id, offset_ms))?;
    assemble(&app, track_id, false)
}

/// Writes the words beside the song as a `.lrc` (SPEC §10).
#[tauri::command]
pub fn lyrics_write_beside(app: AppHandle, track_id: i64) -> Result<LyricsDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    let Some(song) = song_of(&library, track_id)? else {
        return Err(ErrorCode::Library);
    };
    let found = look_nearby(&library, track_id, &song.path)?;
    let words = match found.text {
        Some(words) => words,
        None => library
            .read(|library| library.lyrics(track_id))?
            .and_then(|kept| kept.words().map(str::to_string))
            .ok_or(ErrorCode::Io)?,
    };
    beside::write_beside(&song.path, &words).map_err(|error| {
        tracing::warn!("the words could not be written beside the song: {error}");
        ErrorCode::Io
    })?;
    assemble(&app, track_id, false)
}

/// The lyrics settings, as the interface shows them.
#[tauri::command]
pub fn lyrics_settings(library: State<'_, LibraryService>) -> Result<LyricsPrefs, ErrorCode> {
    prefs(&library)
}

/// Turns the lyrics service on or off, and says whether to keep a `.lrc`.
#[tauri::command]
pub fn lyrics_set_settings(
    app: AppHandle,
    online: bool,
    write_beside: bool,
) -> Result<LyricsPrefs, ErrorCode> {
    let library = app.state::<LibraryService>();
    if !online {
        // Off means off, including the lookups in the air at this moment.
        app.state::<Arc<Lookups>>().abandon();
    }
    let prefs = LyricsPrefs {
        online,
        write_beside,
    };
    library.read(|library| settings::save(library, settings::LYRICS_KEY, &prefs))?;
    tracing::info!(online, write_beside, "the lyrics settings changed");
    Ok(prefs)
}
