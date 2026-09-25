//! Finding out what a track is, without holding anything up (SPEC §8).
//!
//! This is the only part of Onsa that asks the internet about somebody's
//! music, and it is built around three promises.
//!
//! **It cannot start by itself.** Looking things up is off until it is
//! turned on (SPEC §14), and turning it off again stops a run that is under
//! way. A run is asked for by name, over the tracks the folder allows, and
//! never over anything else.
//!
//! **It cannot hold the application up.** The whole run happens on a thread
//! of its own; the interface is told what has been found as it is found, and
//! can ask at any moment without waiting for the thread. A service that is
//! slow costs that thread its time and nothing else — the player does not
//! stutter and the window does not stop drawing.
//!
//! **What has been found is not lost.** Every result is put where the
//! interface can read it the moment it exists. Stopping the run, the service
//! going away in the middle, a track that cannot be fingerprinted: none of
//! them throw away the tracks already done, and all of them leave a run that
//! can simply be asked for again.
//!
//! Nothing here applies anything. What comes out is a list of suggestions
//! with a confidence on each, and applying them goes through the same door
//! as everything else on the Tidy page: the folder, the summary, the button
//! under it, and the run that can be taken back.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use onsa_downloader::{Program, Programs};
use onsa_library::clean;
use onsa_library::TrackRow;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::acoustid::{self, Failure};
use crate::musicbrainz;
use crate::proposal::{CoverOffer, FieldProposal, Proposal, Source, TrackMatch};

/// What the interface listens for while a run is going.
pub const MATCH_EVENT: &str = "match://progress";

/// The folder inside the cover cache where a suggested picture waits.
const PROPOSED: &str = "proposed";

/// How a run stands, as the interface sees it.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// Whether a run is going.
    pub running: bool,
    /// Tracks looked at so far.
    pub done: usize,
    /// Tracks in the run.
    pub total: usize,
    /// What stopped it, when something did: a fixed word to translate.
    pub trouble: Option<String>,
}

/// A run, as it is held while it happens.
#[derive(Debug, Default)]
struct Run {
    found: Vec<TrackMatch>,
    done: usize,
    total: usize,
    trouble: Option<String>,
}

/// Everything one run of matching holds.
///
/// Held by the application, not by the thread doing the work, so that
/// stopping the thread, or its dying, never takes the results with it.
#[derive(Debug, Default)]
pub struct Matching {
    run: Mutex<Run>,
    running: AtomicBool,
    cancel: AtomicBool,
}

impl Matching {
    /// How the run stands and everything found so far.
    pub fn state(&self) -> (Progress, Vec<TrackMatch>) {
        let run = self.run.lock().unwrap_or_else(|e| e.into_inner());
        (
            Progress {
                running: self.running.load(Ordering::Relaxed),
                done: run.done,
                total: run.total,
                trouble: run.trouble.clone(),
            },
            run.found.clone(),
        )
    }

    /// How the run stands, without carrying the whole list back.
    pub fn progress(&self) -> Progress {
        let run = self.run.lock().unwrap_or_else(|e| e.into_inner());
        Progress {
            running: self.running.load(Ordering::Relaxed),
            done: run.done,
            total: run.total,
            trouble: run.trouble.clone(),
        }
    }

    /// Asks a run to stop. What it has found stays where it is.
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    /// Whether a run is going.
    pub fn busy(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// The picture waiting under this name, if it is still there.
    pub fn proposed_cover(&self, covers_dir: &Path, key: &str) -> Option<Vec<u8>> {
        if !is_a_hash(key) {
            return None;
        }
        std::fs::read(covers_dir.join(PROPOSED).join(format!("{key}.bin"))).ok()
    }
}

/// Whether a name is one Onsa wrote: a SHA-256 in lower-case hex and nothing
/// else. It becomes part of a path, so nothing else is ever accepted.
pub fn is_a_hash(key: &str) -> bool {
    key.len() == 64
        && key
            .chars()
            .all(|ch| ch.is_ascii_digit() || ('a'..='f').contains(&ch))
}

/// What one run needs to know, gathered before the thread starts so that the
/// thread never reaches back into the database for settings.
pub struct Orders {
    /// The AcoustID key, when there is one.
    pub key: Option<String>,
    /// Where the external programs are.
    pub programs: Programs,
    /// Where covers are kept.
    pub covers_dir: PathBuf,
    /// The tracks to look at, already held to the folder.
    pub tracks: Vec<TrackRow>,
}

/// Starts a run on its own thread and comes straight back.
pub fn start(app: &AppHandle, orders: Orders) {
    let state = app.state::<Arc<Matching>>().inner().clone();
    if state
        .running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        tracing::info!("a matching run is already going");
        return;
    }
    state.cancel.store(false, Ordering::SeqCst);
    {
        let mut run = state.run.lock().unwrap_or_else(|e| e.into_inner());
        *run = Run {
            found: Vec::new(),
            done: 0,
            total: orders.tracks.len(),
            trouble: None,
        };
    }
    // A run's pictures are its own; last time's are no longer offered.
    let proposed = orders.covers_dir.join(PROPOSED);
    let _ = std::fs::remove_dir_all(&proposed);
    if let Err(error) = std::fs::create_dir_all(&proposed) {
        tracing::warn!("the folder for suggested covers cannot be made: {error}");
    }

    let handle = app.clone();
    let theirs = state.clone();
    // A thread rather than the async runtime: every step of this blocks on
    // purpose, and none of it belongs on a thread that answers the
    // interface.
    match std::thread::Builder::new()
        .name("onsa-matching".into())
        .spawn(move || {
            work(&handle, &theirs, orders);
            theirs.running.store(false, Ordering::SeqCst);
            let _ = handle.emit(MATCH_EVENT, Found::end(theirs.progress()));
        }) {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("the matching run could not be started: {error}");
            state.running.store(false, Ordering::SeqCst);
        }
    }
}

/// One thing to tell the interface: how the run stands, and a track if one
/// was just finished.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Found {
    #[serde(flatten)]
    progress: Progress,
    track: Option<TrackMatch>,
}

impl Found {
    fn end(progress: Progress) -> Self {
        Self {
            progress,
            track: None,
        }
    }
}

/// The run itself, on its own thread.
fn work(app: &AppHandle, state: &Matching, orders: Orders) {
    let Orders {
        key,
        programs,
        covers_dir,
        tracks,
    } = orders;
    // One picture and one year per release group, however many tracks
    // share it: a folder of one album asks once, not forty times.
    let mut covers: HashMap<String, Option<CoverOffer>> = HashMap::new();
    let mut years: HashMap<String, Option<i32>> = HashMap::new();
    // Whether the fingerprinter is usable at all, asked once and then
    // remembered: a folder of fifty untagged tracks asks fpcalc what it is
    // once, not fifty times.
    let mut fingerprinter: Option<Result<(), Failure>> = None;

    for track in tracks {
        if state.cancel.load(Ordering::SeqCst) {
            note_trouble(state, "stopped");
            return;
        }
        let found = look_at(
            &track,
            key.as_deref(),
            &programs,
            &covers_dir,
            &mut covers,
            &mut years,
            &mut fingerprinter,
        );
        let mut run = state.run.lock().unwrap_or_else(|e| e.into_inner());
        run.done += 1;
        run.found.push(found.clone());
        let progress = Progress {
            running: true,
            done: run.done,
            total: run.total,
            trouble: run.trouble.clone(),
        };
        drop(run);
        let _ = app.emit(
            MATCH_EVENT,
            Found {
                progress,
                track: Some(found),
            },
        );
    }
}

/// Writes down why a run ended early. What it found stays.
fn note_trouble(state: &Matching, why: &str) {
    let mut run = state.run.lock().unwrap_or_else(|e| e.into_inner());
    run.trouble = Some(why.to_string());
}

/// Everything Onsa can find out about one track.
#[allow(clippy::too_many_arguments)]
fn look_at(
    track: &TrackRow,
    key: Option<&str>,
    programs: &Programs,
    covers_dir: &Path,
    covers: &mut HashMap<String, Option<CoverOffer>>,
    years: &mut HashMap<String, Option<i32>>,
    fingerprinter: &mut Option<Result<(), Failure>>,
) -> TrackMatch {
    let path = Path::new(&track.path);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut candidates: Vec<Proposal> = Vec::new();
    let mut trouble: Option<Failure> = None;

    let tagged = has_tags(track);
    if tagged {
        // It says something about itself, so that is what is asked about.
        candidates.extend(from_musicbrainz(
            track,
            track.artist.as_deref().unwrap_or_default(),
            track.title.as_deref().unwrap_or_default(),
            track.album.as_deref(),
            1.0,
        ));
    }

    if candidates.is_empty() {
        // The main way for a track that says nothing: the sound itself.
        match fingerprint_match(track, path, key, programs, years, fingerprinter) {
            Ok(found) => candidates.extend(found),
            Err(failure) => trouble = Some(failure),
        }
    }

    // The way back. A name that gave up both halves is an answer in its own
    // right and is always offered; a name that gave up only a title is worth
    // showing only when there is nothing else, because a file called
    // "03 kosong" is a file name and not a claim about a recording.
    let guess = clean::guess_from_name(&stem);
    if !guess.is_empty() && !tagged {
        if candidates.is_empty() {
            // A name with both halves is worth asking MusicBrainz about,
            // but the answer can only ever be as sure as the name was.
            if let (Some(artist), Some(title)) = (&guess.artist, &guess.title) {
                candidates.extend(from_musicbrainz(
                    track,
                    artist,
                    title,
                    None,
                    guess.confidence,
                ));
            }
        }
        if guess.confidence >= crate::proposal::WORTH_SHOWING || candidates.is_empty() {
            candidates.push(from_the_name(track, &guess));
        }
    }

    if candidates.is_empty() {
        let why = trouble
            .as_ref()
            .map(Failure::name)
            .unwrap_or("nothingFound")
            .to_string();
        return TrackMatch::nothing(track.id, &track.path, why);
    }

    // The picture goes with whichever answer is surest and names a release.
    let mut best: Option<usize> = None;
    let mut top = 0.0f64;
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.confidence > top && candidate.release_group.is_some() {
            top = candidate.confidence;
            best = Some(index);
        }
    }
    if let Some(index) = best {
        if let Some(group) = candidates[index].release_group.clone() {
            let offer = covers
                .entry(group.clone())
                .or_insert_with(|| fetch_cover(&group, covers_dir))
                .clone();
            candidates[index] = candidates[index].clone().with_cover(offer);
        }
    }

    // A row can carry a suggestion and a complaint at once: the sound could
    // not be read, and here is what the name says anyway.
    TrackMatch::new(track.id, &track.path, candidates)
        .despite(trouble.as_ref().map(|failure| failure.name().to_string()))
}

/// Whether a track already says enough about itself to be looked up by it.
fn has_tags(track: &TrackRow) -> bool {
    let said = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .is_some_and(|text| !text.is_empty())
    };
    said(&track.title) && said(&track.artist)
}

/// Whether the fingerprinter is fpcalc, asked before any track is.
///
/// A file that is not fpcalc, asked about a song, complains — and its
/// complaint reads exactly like the song's fault. So the question is put to
/// the program itself first, with nobody's music in it: if what is there
/// does not answer as fpcalc, every track in the run is told that the
/// fingerprinter is the trouble, and none of them is blamed.
fn fingerprinter(programs: &Programs) -> Result<(), Failure> {
    let Some(found) = programs.find(Program::Fpcalc) else {
        return Err(Failure::NoFingerprinter);
    };
    onsa_downloader::identify(Program::Fpcalc, &found.path)
        .map(|_| ())
        .map_err(|why| Failure::BadFingerprinter(why.to_string()))
}

/// Recognising a track by its sound.
fn fingerprint_match(
    track: &TrackRow,
    path: &Path,
    key: Option<&str>,
    programs: &Programs,
    years: &mut HashMap<String, Option<i32>>,
    checked: &mut Option<Result<(), Failure>>,
) -> Result<Vec<Proposal>, Failure> {
    let key = key.ok_or(Failure::NoKey)?;
    if let Err(failure) = checked.get_or_insert_with(|| fingerprinter(programs)) {
        return Err(failure.clone());
    }
    let print = acoustid::fingerprint(programs, path)?;
    let found = acoustid::lookup(key, &print)?;
    let mut proposals = Vec::new();
    for one in found {
        if one.score < crate::proposal::WORTH_SHOWING {
            continue;
        }
        // AcoustID names the release but not when it came out, so the year
        // is asked for once per release and then remembered.
        let year = match &one.release_group_id {
            Some(group) => *years
                .entry(group.clone())
                .or_insert_with(|| musicbrainz::release_group(group).unwrap_or(None)),
            None => None,
        };
        let fields = fields_for(
            track,
            one.title.clone(),
            one.artist.clone(),
            one.album.clone(),
            one.album_artist.clone(),
            year,
            None,
        );
        proposals.push(
            Proposal::new(track.id, &track.path, Source::AcoustId, one.score, fields)
                .about(one.release_group_id),
        );
    }
    Ok(proposals)
}

/// Asking MusicBrainz by what something says.
///
/// `strength` is how much the question itself was worth: asking by a track's
/// own tags is worth the whole answer, and asking by a guess at a file name
/// is worth only as much as the guess was.
fn from_musicbrainz(
    track: &TrackRow,
    artist: &str,
    title: &str,
    album: Option<&str>,
    strength: f64,
) -> Vec<Proposal> {
    let found = match musicbrainz::search(artist, title, album) {
        Ok(found) => found,
        Err(error) => {
            tracing::debug!("MusicBrainz found nothing for a track: {error}");
            return Vec::new();
        }
    };
    found
        .into_iter()
        .filter(|one| one.confidence * strength >= crate::proposal::WORTH_SHOWING)
        .map(|one| {
            let fields = fields_for(
                track,
                one.title.clone(),
                one.artist.clone(),
                one.album.clone(),
                one.album_artist.clone(),
                one.year,
                one.track_number,
            );
            Proposal::new(
                track.id,
                &track.path,
                Source::MusicBrainz,
                one.confidence * strength,
                fields,
            )
            .about(one.release_group_id)
        })
        .collect()
}

/// What the file name says, as an answer in its own right.
fn from_the_name(track: &TrackRow, guess: &clean::Guess) -> Proposal {
    let fields = fields_for(
        track,
        guess.title.clone(),
        guess.artist.clone(),
        None,
        None,
        None,
        guess.track_number,
    );
    Proposal::new(
        track.id,
        &track.path,
        Source::FileName,
        guess.confidence,
        fields,
    )
}

/// The fields a suggestion would change, leaving out everything it has
/// nothing to say about.
fn fields_for(
    track: &TrackRow,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    album_artist: Option<String>,
    year: Option<i32>,
    track_number: Option<u32>,
) -> Vec<FieldProposal> {
    let mut fields = Vec::new();
    let mut add = |name: &str, current: Option<String>, suggested: Option<String>| {
        if let Some(suggested) = suggested {
            fields.push(FieldProposal {
                field: name.to_string(),
                current,
                suggested: Some(suggested),
                chosen: false,
            });
        }
    };
    add("title", track.title.clone(), title);
    add("artist", track.artist.clone(), artist);
    add("album", track.album.clone(), album);
    add("album_artist", track.album_artist.clone(), album_artist);
    add(
        "year",
        track.year.map(|year| year.to_string()),
        year.map(|year| year.to_string()),
    );
    add(
        "track_number",
        track.track_number.map(|number| number.to_string()),
        track_number.map(|number| number.to_string()),
    );
    fields
}

/// Fetches a cover and leaves it where the interface can look at it.
///
/// Both the picture as it came and a pair of thumbnails are written under
/// the hash of the bytes. The thumbnails are what the list shows; the
/// picture itself is what gets stored if the listener says yes, so saying
/// yes costs no second download.
fn fetch_cover(release_group_id: &str, covers_dir: &Path) -> Option<CoverOffer> {
    let bytes = match musicbrainz::cover(release_group_id) {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::debug!("no cover for a release group: {error}");
            return None;
        }
    };
    let folder = covers_dir.join(PROPOSED);
    if let Err(error) = std::fs::create_dir_all(&folder) {
        tracing::warn!("the folder for suggested covers cannot be made: {error}");
        return None;
    }
    let thumbs = onsa_library::cover::make_thumbnails(
        &bytes,
        &folder,
        Path::new("a cover from the Cover Art Archive"),
    )?;
    if let Err(error) = std::fs::write(folder.join(format!("{}.bin", thumbs.hash)), &bytes) {
        tracing::warn!("a suggested cover cannot be kept: {error}");
        return None;
    }
    Some(CoverOffer {
        key: thumbs.hash,
        width: thumbs.width,
        height: thumbs.height,
        chosen: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_track(title: Option<&str>, artist: Option<&str>) -> TrackRow {
        TrackRow {
            id: 1,
            path: "C:/music/downloads/Lilith - BANG BANG [dQw4w9WgXcQ].mp3".into(),
            title: title.map(str::to_string),
            artist: artist.map(str::to_string),
            ..TrackRow::default()
        }
    }

    /// A folder with nothing runnable in it, and no system copy allowed.
    fn nothing_installed(name: &str) -> (PathBuf, Programs) {
        let dir = std::env::temp_dir().join(format!("onsa matching {} {name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a folder to work in");
        let programs = Programs::new(&dir).use_system(false);
        (dir, programs)
    }

    #[test]
    fn a_fingerprinter_that_is_not_there_is_not_a_fingerprinter_that_is_wrong() {
        let (dir, programs) = nothing_installed("none");
        assert_eq!(fingerprinter(&programs), Err(Failure::NoFingerprinter));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_is_not_fpcalc_is_the_fingerprinters_fault_and_not_a_tracks() {
        // The whole point of asking first: what is sitting where fpcalc
        // should be is not fpcalc, and the run says so about fpcalc rather
        // than about the first song it was pointed at.
        let (dir, programs) = nothing_installed("wrong");
        std::fs::write(
            dir.join(Program::Fpcalc.file_name()),
            b"not a program at all",
        )
        .expect("a file to find");
        let verdict = fingerprinter(&programs);
        assert!(
            matches!(verdict, Err(Failure::BadFingerprinter(_))),
            "{verdict:?}"
        );
        assert_eq!(
            verdict.unwrap_err().name(),
            "badFingerprinter",
            "a word of its own, so the interface can say something else"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_fingerprinter_is_asked_what_it_is_once_for_the_whole_run() {
        // Remembered rather than asked again: fifty untagged tracks must
        // not mean fifty runs of a program that has already answered.
        let (dir, programs) = nothing_installed("once");
        std::fs::write(
            dir.join(Program::Fpcalc.file_name()),
            b"not a program at all",
        )
        .expect("a file to find");
        let mut checked: Option<Result<(), Failure>> = None;
        let mut years = HashMap::new();
        for _ in 0..3 {
            let outcome = fingerprint_match(
                &a_track(None, None),
                Path::new("C:/music/nothing.flac"),
                Some("a-key"),
                &programs,
                &mut years,
                &mut checked,
            );
            assert!(matches!(outcome, Err(Failure::BadFingerprinter(_))));
        }
        assert!(checked.is_some(), "the answer was kept");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_track_that_says_who_it_is_counts_as_tagged() {
        assert!(has_tags(&a_track(Some("BANG BANG"), Some("Lilith"))));
        assert!(!has_tags(&a_track(Some("BANG BANG"), None)));
        assert!(!has_tags(&a_track(None, None)));
        assert!(
            !has_tags(&a_track(Some("  "), Some("Lilith"))),
            "a tag of spaces says nothing"
        );
    }

    #[test]
    fn a_suggestion_only_carries_what_it_has_something_to_say_about() {
        let track = a_track(Some("track01"), None);
        let fields = fields_for(
            &track,
            Some("BANG BANG".into()),
            None,
            None,
            None,
            Some(2019),
            None,
        );
        let names: Vec<&str> = fields.iter().map(|field| field.field.as_str()).collect();
        assert_eq!(names, vec!["title", "year"]);
        assert_eq!(fields[0].current.as_deref(), Some("track01"));
        assert_eq!(fields[1].suggested.as_deref(), Some("2019"));
    }

    #[test]
    fn a_file_name_is_read_as_its_own_answer() {
        let track = a_track(None, None);
        let guess = clean::guess_from_name("Lilith - BANG BANG [dQw4w9WgXcQ]");
        let proposal = from_the_name(&track, &guess);
        assert_eq!(proposal.source, Source::FileName);
        assert!(!proposal.trusted, "a name is never enough on its own");
        assert_eq!(
            proposal
                .fields
                .iter()
                .find(|field| field.field == "title")
                .and_then(|field| field.suggested.as_deref()),
            Some("BANG BANG"),
            "and the video id is not part of the title"
        );
    }

    #[test]
    fn only_a_name_onsa_wrote_can_name_a_picture() {
        assert!(is_a_hash(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        ));
        for wrong in [
            "../../../etc/passwd",
            "ba7816bf",
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
            "",
        ] {
            assert!(!is_a_hash(wrong), "{wrong}");
        }
    }

    #[test]
    fn a_run_that_has_not_started_is_not_going() {
        let matching = Matching::default();
        let (progress, found) = matching.state();
        assert!(!progress.running);
        assert_eq!(progress.done, 0);
        assert_eq!(progress.total, 0);
        assert!(found.is_empty());
        // Stopping one that is not going is not an error.
        matching.stop();
        assert!(!matching.busy());
    }
}
