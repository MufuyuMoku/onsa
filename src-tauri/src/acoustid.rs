//! Recognising a track by its sound (SPEC §8).
//!
//! Two steps, and neither of them happens unless the listener has turned
//! the internet on. `fpcalc` reads a fingerprint out of the file — through
//! the shared program manager, with an argument array, a deadline and no
//! console window — and AcoustID is asked what that fingerprint belongs to.
//!
//! No audio leaves the machine. A Chromaprint fingerprint is a summary of
//! how the sound changes over time; it cannot be turned back into the
//! recording. What is sent is that summary, the track's length, and the
//! application key.
//!
//! Everything here answers with a value. A missing `fpcalc`, a file that
//! cannot be read, a service that is down or slow: all of them come back as
//! errors the caller counts and carries on from, because the run they belong
//! to may be fifty tracks long and the other forty-nine still have to work.

use std::path::Path;
use std::time::Duration;

use onsa_downloader::{Program, Programs};

use crate::net::{self, NetError, Service};

/// How long one file may be fingerprinted for.
///
/// Reading a fingerprint means decoding the first two minutes of audio; on
/// anything but a failing disk it takes under a second. A file that is still
/// going after this is one `fpcalc` cannot make sense of.
const FINGERPRINT_TIMEOUT: Duration = Duration::from_secs(30);

/// Matches worth carrying back from one lookup. AcoustID sorts them, and
/// past the third they are guesses about guesses.
const MOST_MATCHES: usize = 3;

/// What a fingerprint is, as far as Onsa is concerned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    /// The track's length in whole seconds, as `fpcalc` measured it.
    pub seconds: u32,
    /// The fingerprint itself, in the compressed form AcoustID expects.
    pub value: String,
}

/// What went wrong, in terms a list of tracks can show.
///
/// The first two are deliberately not the same failure. One of them is
/// about `fpcalc` and one is about this track, and saying the wrong one
/// sends somebody looking at a song file that was never the problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// `fpcalc` is not installed, so nothing can be fingerprinted.
    NoFingerprinter,
    /// There is a file where `fpcalc` should be, but it is not usable:
    /// it will not start, or it is some other program entirely.
    BadFingerprinter(String),
    /// The file could not be fingerprinted.
    Unreadable(String),
    /// There is no AcoustID key.
    NoKey,
    /// AcoustID could not be reached, or would not answer.
    Offline,
    /// AcoustID would not accept the key.
    Refused,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFingerprinter => write!(f, "fpcalc is not installed"),
            Self::BadFingerprinter(why) => write!(f, "fpcalc cannot be used: {why}"),
            Self::Unreadable(why) => write!(f, "the file could not be fingerprinted: {why}"),
            Self::NoKey => write!(f, "there is no AcoustID key"),
            Self::Offline => write!(f, "AcoustID could not be reached"),
            Self::Refused => write!(f, "AcoustID would not accept the key"),
        }
    }
}

/// A fixed word per failure, for the interface to translate.
impl Failure {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NoFingerprinter => "noFingerprinter",
            Self::BadFingerprinter(_) => "badFingerprinter",
            Self::Unreadable(_) => "unreadable",
            Self::NoKey => "noKey",
            Self::Offline => "offline",
            Self::Refused => "refused",
        }
    }
}

/// One recording AcoustID thinks a fingerprint might be.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Match {
    /// How sure AcoustID is, from nothing to one.
    pub score: f64,
    /// The MusicBrainz recording, when it named one.
    pub recording_id: Option<String>,
    /// What it is called.
    pub title: Option<String>,
    /// Who it is by, with the credited artists joined as they are credited.
    pub artist: Option<String>,
    /// The release it belongs to, when there is an obvious one.
    pub album: Option<String>,
    /// Whose release that is.
    pub album_artist: Option<String>,
    /// The release group, which is what the Cover Art Archive files a
    /// picture under.
    pub release_group_id: Option<String>,
}

/// Reads a fingerprint out of a file.
pub fn fingerprint(programs: &Programs, path: &Path) -> Result<Fingerprint, Failure> {
    if !programs.have(Program::Fpcalc) {
        return Err(Failure::NoFingerprinter);
    }
    let args: Vec<std::ffi::OsString> = vec![
        "-json".into(),
        // Two minutes is what AcoustID matches on; reading further is time
        // spent on audio nobody will compare.
        "-length".into(),
        "120".into(),
        path.as_os_str().to_os_string(),
    ];
    let ran = programs
        .run(Program::Fpcalc, &args, FINGERPRINT_TIMEOUT)
        .map_err(blame)?
        .ok(Program::Fpcalc)
        .map_err(blame)?;
    read_fingerprint(&ran)
}

/// Whose fault a failed run was: the fingerprinter's, or this track's.
///
/// A program that will not start is not a file that cannot be read, and the
/// two are never worth confusing. `fpcalc` complaining about a track, and
/// `fpcalc` taking longer than it is given, are about the track; everything
/// else is about the program that was supposed to be `fpcalc`.
fn blame(error: onsa_downloader::Error) -> Failure {
    use onsa_downloader::Error;
    match error {
        Error::MissingProgram(_) => Failure::NoFingerprinter,
        Error::ProgramFailed { said, .. } => Failure::Unreadable(said),
        Error::TooSlow { seconds, .. } => {
            Failure::Unreadable(format!("fpcalc was still going after {seconds}s"))
        }
        other => Failure::BadFingerprinter(other.to_string()),
    }
}

/// Reads what `fpcalc -json` printed.
fn read_fingerprint(printed: &str) -> Result<Fingerprint, Failure> {
    // Nothing that is not fpcalc's own answer says anything about the
    // track: it says the program that was run is not fpcalc.
    let answer: serde_json::Value = serde_json::from_str(printed).map_err(|_| {
        Failure::BadFingerprinter("what it printed is not what fpcalc prints".into())
    })?;
    let value = answer
        .get("fingerprint")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Failure::Unreadable("fpcalc found no fingerprint".into()))?;
    // fpcalc prints the duration as a number of seconds, sometimes with a
    // fraction. AcoustID wants whole seconds.
    let seconds = answer
        .get("duration")
        .and_then(|duration| duration.as_f64())
        .filter(|seconds| *seconds > 0.0)
        .ok_or_else(|| Failure::Unreadable("fpcalc gave no duration".into()))?;
    Ok(Fingerprint {
        seconds: seconds.round() as u32,
        value: value.to_string(),
    })
}

/// Asks AcoustID what a fingerprint belongs to.
pub fn lookup(key: &str, print: &Fingerprint) -> Result<Vec<Match>, Failure> {
    let seconds = print.seconds.to_string();
    let answer = net::ask_json(
        Service::AcoustId,
        "/lookup",
        &[
            ("client", key),
            ("duration", &seconds),
            ("fingerprint", &print.value),
            // Everything Onsa can fill a tag with, in one request: asking
            // per track is already three a second at best.
            ("meta", "recordings+releasegroups+compress"),
            ("format", "json"),
        ],
    )
    .map_err(|error| match error {
        NetError::TooLarge | NetError::Unreachable | NetError::Stopped => Failure::Offline,
    })?;
    read_matches(&answer.body)
}

/// Reads an AcoustID answer.
///
/// Every field is read as optional. AcoustID leaves out what it does not
/// know, and a recording with no release group or no artist is common; a
/// missing field is a field Onsa does not suggest, never a failed lookup.
fn read_matches(answer: &serde_json::Value) -> Result<Vec<Match>, Failure> {
    if answer.get("status").and_then(|status| status.as_str()) != Some("ok") {
        let said = answer
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(|message| message.as_str())
            .unwrap_or_default()
            .to_lowercase();
        if said.contains("api key") || said.contains("apikey") {
            return Err(Failure::Refused);
        }
        tracing::debug!("AcoustID answered with an error");
        return Err(Failure::Offline);
    }
    let mut found = Vec::new();
    let results = answer
        .get("results")
        .and_then(|results| results.as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    for result in results {
        let score = result
            .get("score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let recordings = result
            .get("recordings")
            .and_then(|list| list.as_array())
            .map(Vec::as_slice)
            .unwrap_or_default();
        if recordings.is_empty() {
            // A fingerprint AcoustID knows but has no recording for says
            // only that somebody else has heard this file too.
            continue;
        }
        for recording in recordings {
            let group = recording
                .get("releasegroups")
                .and_then(|list| list.as_array())
                .and_then(|list| list.first());
            found.push(Match {
                score,
                recording_id: text(recording.get("id")),
                title: text(recording.get("title")),
                artist: credited(recording.get("artists")),
                album: group.and_then(|group| text(group.get("title"))),
                album_artist: group.and_then(|group| credited(group.get("artists"))),
                release_group_id: group.and_then(|group| text(group.get("id"))),
            });
            if found.len() >= MOST_MATCHES {
                return Ok(found);
            }
        }
    }
    Ok(found)
}

/// A string field that is actually there and actually says something.
fn text(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// The artists as they are credited, joined the way the credit says.
///
/// MusicBrainz keeps the words between names — "feat.", " & ", " x " — as
/// part of the credit, and a name put together any other way is not the one
/// on the record.
fn credited(value: Option<&serde_json::Value>) -> Option<String> {
    // Each credit carries the name and the words that follow it, so they
    // are gathered first and only then joined.
    let credits: Vec<(String, Option<String>)> = value?
        .as_array()?
        .iter()
        .filter_map(|artist| {
            let name = text(artist.get("name"))?;
            // A join phrase is whitespace and punctuation, so it is taken
            // as it is written rather than trimmed.
            let join = artist
                .get("joinphrase")
                .and_then(|join| join.as_str())
                .filter(|join| !join.is_empty())
                .map(str::to_string);
            Some((name, join))
        })
        .collect();
    let mut out = String::new();
    for (index, (name, join)) in credits.iter().enumerate() {
        out.push_str(name);
        if index + 1 < credits.len() {
            out.push_str(join.as_deref().unwrap_or(", "));
        }
    }
    let out = out.trim();
    (!out.is_empty()).then(|| out.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What `fpcalc -json` prints, cut to the two fields that matter.
    const FPCALC: &str = r#"{"duration":215.53,"fingerprint":"AQADtEmUaEkSRZEGP8cP9AiPHz_wI_2BH-EP_Eh-4Ef6Az_SH_iR_kCP9Ad-pD_wI_2BH"}"#;

    /// A recorded AcoustID answer (SPEC §12: a test that would need the
    /// network uses an answer that was written down instead).
    const ANSWERED: &str = r#"{
      "status": "ok",
      "results": [
        {
          "id": "9ff43b6a-4f16-427c-93c2-92307ca505e0",
          "score": 0.934783,
          "recordings": [
            {
              "id": "1e4d6b56-9e1e-4b1a-9f3e-000000000001",
              "title": "Lampu Kota",
              "duration": 215,
              "artists": [
                { "id": "a1", "name": "Lilith", "joinphrase": " feat. " },
                { "id": "a2", "name": "ミナ" }
              ],
              "releasegroups": [
                {
                  "id": "rg-0001",
                  "title": "Kota Sunyi",
                  "type": "Album",
                  "artists": [{ "id": "a1", "name": "Lilith" }]
                }
              ]
            }
          ]
        }
      ]
    }"#;

    #[test]
    fn a_fingerprint_is_read_with_its_length_in_whole_seconds() {
        let print = read_fingerprint(FPCALC).expect("fpcalc said enough");
        assert_eq!(print.seconds, 216, "215.53 rounds up");
        assert!(print.value.starts_with("AQAD"));
    }

    #[test]
    fn half_an_answer_from_fpcalc_is_a_failure_that_says_so() {
        assert!(matches!(
            read_fingerprint("{}"),
            Err(Failure::Unreadable(_))
        ));
        assert!(matches!(
            read_fingerprint(r#"{"duration":0,"fingerprint":"AQAD"}"#),
            Err(Failure::Unreadable(_))
        ));
    }

    #[test]
    fn something_that_is_not_fpcalc_is_not_the_tracks_fault() {
        // A program that is not fpcalc prints something that is not
        // fpcalc's answer. Reading that as "this song could not be read"
        // is how somebody ends up looking at a song file that was fine.
        assert!(matches!(
            read_fingerprint("not json at all"),
            Err(Failure::BadFingerprinter(_))
        ));
        assert!(matches!(
            read_fingerprint("Usage: ffmpeg [options]"),
            Err(Failure::BadFingerprinter(_))
        ));
    }

    #[test]
    fn a_failed_run_is_blamed_on_whichever_of_the_two_it_was() {
        use onsa_downloader::Error;
        assert_eq!(
            blame(Error::MissingProgram("fpcalc".into())),
            Failure::NoFingerprinter
        );
        // fpcalc ran and complained about the file: that one is the file.
        assert!(matches!(
            blame(Error::ProgramFailed {
                program: "fpcalc".into(),
                code: Some(2),
                said: "ERROR: unable to read file".into()
            }),
            Failure::Unreadable(said) if said.contains("unable to read")
        ));
        assert!(matches!(
            blame(Error::TooSlow {
                program: "fpcalc".into(),
                seconds: 30
            }),
            Failure::Unreadable(_)
        ));
        // It never started, so it never saw the file.
        assert!(matches!(
            blame(Error::CannotRun {
                program: "fpcalc".into(),
                said: "%1 is not a valid Win32 application".into()
            }),
            Failure::BadFingerprinter(_)
        ));
    }

    #[test]
    fn a_recorded_answer_reads_as_one_match() {
        let answer: serde_json::Value = serde_json::from_str(ANSWERED).expect("valid JSON");
        let found = read_matches(&answer).expect("an answer");
        assert_eq!(found.len(), 1);
        let one = &found[0];
        assert!((one.score - 0.934783).abs() < 1e-6);
        assert_eq!(one.title.as_deref(), Some("Lampu Kota"));
        assert_eq!(
            one.artist.as_deref(),
            Some("Lilith feat. ミナ"),
            "credited the way the record credits them"
        );
        assert_eq!(one.album.as_deref(), Some("Kota Sunyi"));
        assert_eq!(one.album_artist.as_deref(), Some("Lilith"));
        assert_eq!(one.release_group_id.as_deref(), Some("rg-0001"));
    }

    #[test]
    fn an_answer_with_nothing_in_it_is_an_answer() {
        let empty: serde_json::Value =
            serde_json::from_str(r#"{"status":"ok","results":[]}"#).expect("valid");
        assert_eq!(read_matches(&empty), Ok(Vec::new()));

        // A fingerprint that is known but belongs to no recording says
        // nothing useful, and is not offered.
        let bare: serde_json::Value =
            serde_json::from_str(r#"{"status":"ok","results":[{"id":"x","score":0.9}]}"#)
                .expect("valid");
        assert_eq!(read_matches(&bare), Ok(Vec::new()));
    }

    #[test]
    fn a_refused_key_is_told_apart_from_a_service_that_is_down() {
        let refused: serde_json::Value = serde_json::from_str(
            r#"{"status":"error","error":{"code":4,"message":"invalid API key"}}"#,
        )
        .expect("valid");
        assert_eq!(read_matches(&refused), Err(Failure::Refused));

        let broken: serde_json::Value = serde_json::from_str(
            r#"{"status":"error","error":{"code":2,"message":"missing parameter"}}"#,
        )
        .expect("valid");
        assert_eq!(read_matches(&broken), Err(Failure::Offline));
    }

    #[test]
    fn a_missing_field_is_a_field_onsa_will_not_suggest() {
        let thin: serde_json::Value = serde_json::from_str(
            r#"{"status":"ok","results":[{"score":0.5,"recordings":[{"id":"r1"}]}]}"#,
        )
        .expect("valid");
        let found = read_matches(&thin).expect("an answer");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, None);
        assert_eq!(found[0].artist, None);
        assert_eq!(found[0].release_group_id, None);
    }

    #[test]
    fn every_failure_has_a_word_of_its_own_and_something_to_say() {
        let all = [
            Failure::NoFingerprinter,
            Failure::BadFingerprinter("x".into()),
            Failure::Unreadable("x".into()),
            Failure::NoKey,
            Failure::Offline,
            Failure::Refused,
        ];
        let mut names: Vec<&str> = all.iter().map(Failure::name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), all.len(), "one word each");
        for failure in &all {
            assert!(!failure.to_string().is_empty());
        }
    }
}
