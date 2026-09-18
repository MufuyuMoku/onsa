//! LRCLIB: what to ask it, and how to read what it says (SPEC §10).
//!
//! The asking itself is not here. This builds a question and reads an
//! answer; the one HTTP client in the application carries it, keeps to the
//! timeouts and the rate limit, and decides whether the listener has allowed
//! the network at all.
//!
//! LRCLIB needs no key and no account. What Onsa sends is the artist, the
//! title, the album and the length of the song — nothing about the listener,
//! and nothing of the audio.

use serde_json::Value;

/// Where a single answer is asked for.
pub const GET: &str = "/get";
/// Where several are, when the exact one is not there.
pub const SEARCH: &str = "/search";

/// How far a search result's length may be from the song's, in seconds,
/// before it is a different recording rather than the same one.
const CLOSE_ENOUGH: i64 = 3;

/// What Onsa knows about the song it wants the words for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ask {
    /// The artist as the song's tags give it.
    pub artist: String,
    /// The title.
    pub title: String,
    /// The album, when the song says.
    pub album: Option<String>,
    /// How long the song is, in seconds.
    pub duration_s: Option<i64>,
}

impl Ask {
    /// Whether there is enough here to ask about at all.
    ///
    /// A song with no artist or no title would have LRCLIB guessing, and a
    /// guess is not worth a request.
    pub fn is_enough(&self) -> bool {
        !self.artist.trim().is_empty() && !self.title.trim().is_empty()
    }

    /// The query for an exact lookup.
    pub fn get_query(&self) -> Vec<(String, String)> {
        let mut query = vec![
            ("artist_name".to_string(), self.artist.trim().to_string()),
            ("track_name".to_string(), self.title.trim().to_string()),
        ];
        if let Some(album) = self
            .album
            .as_deref()
            .map(str::trim)
            .filter(|a| !a.is_empty())
        {
            query.push(("album_name".to_string(), album.to_string()));
        }
        if let Some(duration) = self.duration_s.filter(|seconds| *seconds > 0) {
            query.push(("duration".to_string(), duration.to_string()));
        }
        query
    }

    /// The query for a search, which is the fallback when the exact lookup
    /// finds nothing: no album, no length, so more can come back.
    pub fn search_query(&self) -> Vec<(String, String)> {
        vec![
            ("artist_name".to_string(), self.artist.trim().to_string()),
            ("track_name".to_string(), self.title.trim().to_string()),
        ]
    }
}

/// One set of words LRCLIB knows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    /// LRCLIB's own number for it, kept for the log and nothing else.
    pub id: Option<i64>,
    /// The title it answered with, which may be spelled differently.
    pub title: String,
    /// The artist it answered with.
    pub artist: String,
    /// How long it thinks the recording is.
    pub duration_s: Option<i64>,
    /// The timed words, when it has them.
    pub synced: Option<String>,
    /// The words without timing.
    pub plain: Option<String>,
    /// Whether it says the recording has no words at all.
    pub instrumental: bool,
}

impl Found {
    /// The best text it carries: timed words when there are any.
    pub fn best(&self) -> Option<&str> {
        self.synced
            .as_deref()
            .or(self.plain.as_deref())
            .filter(|text| !text.trim().is_empty())
    }

    /// Whether it is worth keeping: words, or a clear "there are none".
    pub fn is_useful(&self) -> bool {
        self.instrumental || self.best().is_some()
    }
}

/// Reads the answer to an exact lookup.
///
/// A refusal — LRCLIB answers a miss with a small JSON object saying so —
/// reads as nothing found, which is not an error.
pub fn read_get(answer: &Value) -> Option<Found> {
    let found = one(answer)?;
    found.is_useful().then_some(found)
}

/// Reads the answer to a search, and picks the one that is the same song.
///
/// LRCLIB will happily answer with covers, live versions and other people's
/// recordings of the same title. The length is what tells them apart, so a
/// result whose length is not the song's is not used unless nothing else
/// fits and the song never said how long it was.
pub fn read_search(answer: &Value, ask: &Ask) -> Option<Found> {
    let list = answer.as_array()?;
    let mut best: Option<(i64, Found)> = None;
    for value in list {
        let Some(found) = one(value) else { continue };
        if !found.is_useful() {
            continue;
        }
        let Some(score) = fit(&found, ask) else {
            continue;
        };
        if best.as_ref().is_none_or(|(theirs, _)| score > *theirs) {
            best = Some((score, found));
        }
    }
    best.map(|(_, found)| found)
}

/// How well a result answers what was asked, or `None` when it does not.
fn fit(found: &Found, ask: &Ask) -> Option<i64> {
    let mut score = 0;
    match (ask.duration_s, found.duration_s) {
        (Some(wanted), Some(theirs)) => {
            if (wanted - theirs).abs() > CLOSE_ENOUGH {
                return None;
            }
            score += 4;
        }
        // A song that never said how long it is cannot rule anything out.
        _ => score += 1,
    }
    if same(&found.title, &ask.title) {
        score += 2;
    }
    if same(&found.artist, &ask.artist) {
        score += 2;
    }
    if found.synced.is_some() {
        score += 3;
    }
    Some(score)
}

/// Whether two names are the same name, allowing for how they are written.
fn same(one: &str, other: &str) -> bool {
    one.trim().to_lowercase() == other.trim().to_lowercase()
}

/// Reads one LRCLIB record.
fn one(value: &Value) -> Option<Found> {
    let object = value.as_object()?;
    // A miss comes back as an object with a status code and no words.
    if object.contains_key("statusCode") && !object.contains_key("id") {
        return None;
    }
    let text = |key: &str| -> Option<String> {
        object
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|value| !value.trim().is_empty())
    };
    Some(Found {
        id: object.get("id").and_then(Value::as_i64),
        title: text("trackName").unwrap_or_default(),
        artist: text("artistName").unwrap_or_default(),
        duration_s: object
            .get("duration")
            .and_then(Value::as_f64)
            .map(|seconds| seconds.round() as i64),
        synced: text("syncedLyrics"),
        plain: text("plainLyrics"),
        instrumental: object
            .get("instrumental")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ask() -> Ask {
        Ask {
            artist: "Lilith".to_string(),
            title: "Kota Sunyi".to_string(),
            album: Some("Lampu Kota".to_string()),
            duration_s: Some(200),
        }
    }

    #[test]
    fn a_song_without_artist_or_title_is_not_worth_asking_about() {
        assert!(ask().is_enough());
        assert!(!Ask {
            artist: "  ".to_string(),
            title: "Kota Sunyi".to_string(),
            ..Ask::default()
        }
        .is_enough());
    }

    #[test]
    fn the_question_carries_the_song_and_nothing_else() {
        let query = ask().get_query();
        assert_eq!(
            query,
            vec![
                ("artist_name".to_string(), "Lilith".to_string()),
                ("track_name".to_string(), "Kota Sunyi".to_string()),
                ("album_name".to_string(), "Lampu Kota".to_string()),
                ("duration".to_string(), "200".to_string()),
            ]
        );
        // The search asks for less, so more can come back.
        assert_eq!(ask().search_query().len(), 2);
    }

    #[test]
    fn a_miss_is_nothing_found_rather_than_a_failure() {
        let answer = json!({ "statusCode": 404, "name": "TrackNotFound" });
        assert_eq!(read_get(&answer), None);
    }

    #[test]
    fn timed_words_are_preferred_to_untimed_ones() {
        let answer = json!({
            "id": 7,
            "trackName": "Kota Sunyi",
            "artistName": "Lilith",
            "duration": 200.0,
            "plainLyrics": "satu\ndua",
            "syncedLyrics": "[00:01.00]satu"
        });
        let found = read_get(&answer).expect("it should read");
        assert_eq!(found.best(), Some("[00:01.00]satu"));
        assert_eq!(found.id, Some(7));
    }

    #[test]
    fn a_recording_with_no_words_says_so_rather_than_being_nothing() {
        let answer = json!({
            "id": 8,
            "trackName": "Interlude",
            "artistName": "Lilith",
            "duration": 60.0,
            "instrumental": true,
            "plainLyrics": null,
            "syncedLyrics": null
        });
        let found = read_get(&answer).expect("it should read");
        assert!(found.instrumental);
        assert_eq!(found.best(), None);
    }

    #[test]
    fn a_search_result_of_the_wrong_length_is_a_different_recording() {
        let answer = json!([{
            "id": 9,
            "trackName": "Kota Sunyi",
            "artistName": "Lilith",
            "duration": 320.0,
            "plainLyrics": "lain"
        }]);
        assert_eq!(read_search(&answer, &ask()), None);
    }

    #[test]
    fn among_results_of_the_right_length_the_timed_one_wins() {
        let answer = json!([
            {
                "id": 1,
                "trackName": "Kota Sunyi",
                "artistName": "Lilith",
                "duration": 201.0,
                "plainLyrics": "tanpa waktu"
            },
            {
                "id": 2,
                "trackName": "Kota Sunyi",
                "artistName": "Lilith",
                "duration": 199.0,
                "syncedLyrics": "[00:01.00]dengan waktu"
            }
        ]);
        let found = read_search(&answer, &ask()).expect("one of them");
        assert_eq!(found.id, Some(2));
    }

    #[test]
    fn a_song_that_never_said_its_length_still_gets_an_answer() {
        let without = Ask {
            duration_s: None,
            ..ask()
        };
        let answer = json!([{
            "id": 3,
            "trackName": "Kota Sunyi",
            "artistName": "Lilith",
            "duration": 320.0,
            "plainLyrics": "kata-kata"
        }]);
        assert_eq!(
            read_search(&answer, &without).map(|found| found.id),
            Some(Some(3))
        );
    }

    #[test]
    fn an_answer_that_is_not_a_list_is_nothing_found() {
        assert_eq!(read_search(&json!({ "statusCode": 404 }), &ask()), None);
        assert_eq!(read_search(&json!([]), &ask()), None);
    }
}
