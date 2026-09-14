//! What MusicBrainz knows, and the picture the Cover Art Archive keeps
//! (SPEC §8).
//!
//! Two jobs. For a track that already says something about itself,
//! MusicBrainz is searched for a recording by what it says; this is how a
//! half-tagged file gets its album and its year. For a track a fingerprint
//! recognised, MusicBrainz is asked about the release group AcoustID named,
//! which is what a cover is filed under.
//!
//! Both are held to one request a second, as MusicBrainz asks (SPEC §8), and
//! both carry the User-Agent that says who is calling. The rate limit is
//! shared with the Cover Art Archive, because it is the same house.
//!
//! A search is not a fingerprint and is not treated like one. MusicBrainz
//! scores a search by how closely the text matched, which says how well two
//! strings line up and nothing about whether this is the right recording;
//! a match that does not agree with the file on both the title and the
//! artist is held below the mark at which Onsa would tick it on its own.

use crate::net::{self, NetError, Service};

/// Most releases to read out of one search.
const MOST_RESULTS: usize = 3;
/// Most a cover may weigh before Onsa stops reading it. The archive's 500 px
/// thumbnail is tens of kilobytes; this is room for a large one, not for a
/// scan of a gatefold.
pub const MAX_COVER: u64 = 4 * 1024 * 1024;

/// The most a text search is ever believed when it does not agree with the
/// file on both the title and the artist.
const UNSURE_CEILING: f64 = 0.8;

/// A recording MusicBrainz suggests.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Found {
    /// How sure this is, from nothing to one.
    pub confidence: f64,
    /// The recording.
    pub recording_id: Option<String>,
    /// What it is called.
    pub title: Option<String>,
    /// Who it is by.
    pub artist: Option<String>,
    /// The release it was found on.
    pub album: Option<String>,
    /// Whose release that is.
    pub album_artist: Option<String>,
    /// When that release came out.
    pub year: Option<i32>,
    /// Which track it is on that release.
    pub track_number: Option<u32>,
    /// The release group, which is what a cover is filed under.
    pub release_group_id: Option<String>,
}

/// Searches for a recording by what the file already says.
///
/// `album` only narrows the search; a file that does not know its album
/// still gets an answer.
pub fn search(artist: &str, title: &str, album: Option<&str>) -> Result<Vec<Found>, NetError> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = format!("recording:{}", quoted(title));
    if !artist.trim().is_empty() {
        query.push_str(&format!(" AND artist:{}", quoted(artist)));
    }
    if let Some(album) = album.map(str::trim).filter(|album| !album.is_empty()) {
        query.push_str(&format!(" AND release:{}", quoted(album)));
    }
    let answer = net::ask_json(
        Service::MusicBrainz,
        "/recording",
        &[("query", query.as_str()), ("limit", "5"), ("fmt", "json")],
    )?;
    Ok(read_search(&answer, artist, title))
}

/// A value for a Lucene query, with the characters that mean something to
/// Lucene taken out.
///
/// Onsa quotes the whole thing and then removes the two characters a quoted
/// phrase can still be broken by. Nothing a tag contains can become part of
/// the query itself.
fn quoted(value: &str) -> String {
    let inside: String = value
        .chars()
        .filter(|ch| *ch != '"' && *ch != '\\')
        .collect();
    format!("\"{}\"", inside.trim())
}

/// Reads a recording search.
fn read_search(answer: &serde_json::Value, asked_artist: &str, asked_title: &str) -> Vec<Found> {
    let recordings = answer
        .get("recordings")
        .and_then(|list| list.as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut found = Vec::new();
    for recording in recordings.iter().take(MOST_RESULTS) {
        let title = text(recording.get("title"));
        let artist = credited(recording.get("artist-credit"));
        let score = recording
            .get("score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0)
            / 100.0;
        let release = recording
            .get("releases")
            .and_then(|list| list.as_array())
            .and_then(|list| list.first());
        let agrees = same(title.as_deref(), asked_title) && same(artist.as_deref(), asked_artist);
        found.push(Found {
            confidence: confidence(score, agrees),
            recording_id: text(recording.get("id")),
            title,
            artist,
            album: release.and_then(|release| text(release.get("title"))),
            album_artist: release.and_then(|release| credited(release.get("artist-credit"))),
            year: release.and_then(|release| year_of(text(release.get("date")).as_deref())),
            track_number: release.and_then(track_number_of),
            release_group_id: release
                .and_then(|release| release.get("release-group"))
                .and_then(|group| text(group.get("id"))),
        });
    }
    found
}

/// How much a text search is worth believing.
///
/// A search that agrees with the file on both the title and the artist is
/// worth what MusicBrainz scored it. One that does not is held below the
/// mark at which a suggestion ticks itself, however confident the scoring
/// was: the score says the words are close, not that this is the recording.
pub fn confidence(score: f64, agrees: bool) -> f64 {
    let score = score.clamp(0.0, 1.0);
    if agrees {
        score
    } else {
        score.min(UNSURE_CEILING)
    }
}

/// Whether what came back is what was asked for, allowing for case and
/// surrounding space.
fn same(found: Option<&str>, asked: &str) -> bool {
    match found {
        Some(found) => found.trim().eq_ignore_ascii_case(asked.trim()),
        None => false,
    }
}

/// The year out of a MusicBrainz date, which may be a year, a year and a
/// month, or a whole date.
fn year_of(date: Option<&str>) -> Option<i32> {
    let date = date?;
    let digits: String = date.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() != 4 {
        return None;
    }
    digits.parse().ok()
}

/// Which track this recording is on the release it came back with.
fn track_number_of(release: &serde_json::Value) -> Option<u32> {
    release
        .get("media")
        .and_then(|media| media.as_array())
        .and_then(|media| media.first())
        .and_then(|medium| medium.get("track"))
        .and_then(|tracks| tracks.as_array())
        .and_then(|tracks| tracks.first())
        .and_then(|track| track.get("number"))
        .and_then(|number| number.as_str())
        .and_then(|number| number.parse().ok())
}

/// A string field that is there and says something.
fn text(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// The artists as MusicBrainz credits them, with its own joining words.
fn credited(value: Option<&serde_json::Value>) -> Option<String> {
    let credits: Vec<(String, Option<String>)> = value?
        .as_array()?
        .iter()
        .filter_map(|credit| {
            let name = text(credit.get("name"))
                .or_else(|| text(credit.get("artist").and_then(|artist| artist.get("name"))))?;
            let join = credit
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

/// When a release group came out, for a match that only named the group.
pub fn release_group(id: &str) -> Result<Option<i32>, NetError> {
    if !looks_like_an_id(id) {
        return Ok(None);
    }
    let answer = net::ask_json(
        Service::MusicBrainz,
        &format!("/release-group/{id}"),
        &[("fmt", "json")],
    )?;
    Ok(year_of(text(answer.get("first-release-date")).as_deref()))
}

/// The front cover of a release group, at the size a list can show.
///
/// The archive answers a redirect to wherever the picture actually is, which
/// the shared client follows, and a release group with no picture answers
/// that it has none. Both are ordinary answers.
pub fn cover(release_group_id: &str) -> Result<Vec<u8>, NetError> {
    if !looks_like_an_id(release_group_id) {
        return Err(NetError::Unreachable);
    }
    net::ask_bytes(
        Service::CoverArt,
        &format!("/release-group/{release_group_id}/front-500"),
        MAX_COVER,
    )
}

/// Whether something is shaped like a MusicBrainz id.
///
/// Ids go into the path of a request, so nothing that is not one of these
/// characters is ever put there.
fn looks_like_an_id(id: &str) -> bool {
    id.len() == 36 && id.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposal::TRUSTED;

    /// A recorded MusicBrainz answer (SPEC §12).
    const ANSWERED: &str = r#"{
      "created": "2026-09-14T10:00:00.000Z",
      "count": 1,
      "recordings": [
        {
          "id": "11111111-2222-3333-4444-555555555555",
          "score": 100,
          "title": "Lampu Kota",
          "length": 215000,
          "artist-credit": [
            { "name": "Lilith", "joinphrase": " feat. ", "artist": { "id": "a1", "name": "Lilith" } },
            { "name": "ミナ", "artist": { "id": "a2", "name": "ミナ" } }
          ],
          "releases": [
            {
              "id": "66666666-7777-8888-9999-000000000000",
              "title": "Kota Sunyi",
              "date": "2019-04-21",
              "artist-credit": [{ "name": "Lilith", "artist": { "id": "a1", "name": "Lilith" } }],
              "release-group": { "id": "rg-1", "title": "Kota Sunyi" },
              "media": [{ "format": "Digital Media", "track": [{ "number": "4", "title": "Lampu Kota" }] }]
            }
          ]
        }
      ]
    }"#;

    #[test]
    fn a_recorded_search_reads_as_one_recording() {
        let answer: serde_json::Value = serde_json::from_str(ANSWERED).expect("valid JSON");
        let found = read_search(&answer, "Lilith feat. ミナ", "Lampu Kota");
        assert_eq!(found.len(), 1);
        let one = &found[0];
        assert_eq!(one.title.as_deref(), Some("Lampu Kota"));
        assert_eq!(one.artist.as_deref(), Some("Lilith feat. ミナ"));
        assert_eq!(one.album.as_deref(), Some("Kota Sunyi"));
        assert_eq!(one.album_artist.as_deref(), Some("Lilith"));
        assert_eq!(one.year, Some(2019));
        assert_eq!(one.track_number, Some(4));
        assert_eq!(one.release_group_id.as_deref(), Some("rg-1"));
        assert_eq!(one.confidence, 1.0, "it agreed on both, and scored full");
    }

    #[test]
    fn a_search_that_does_not_agree_is_never_sure_enough_to_tick_itself() {
        let answer: serde_json::Value = serde_json::from_str(ANSWERED).expect("valid JSON");
        // The same answer, but the file said something else.
        let found = read_search(&answer, "Somebody Else", "Lampu Kota");
        assert!(found[0].confidence < TRUSTED, "{}", found[0].confidence);
        assert_eq!(found[0].confidence, UNSURE_CEILING);
    }

    #[test]
    fn the_ceiling_leaves_room_for_a_match_that_does_agree() {
        // There has to be room between the two, or a search could never
        // tick anything and the ceiling would mean nothing.
        const _: () = assert!(UNSURE_CEILING < TRUSTED);
        assert_eq!(confidence(0.5, true), 0.5);
        assert_eq!(confidence(0.5, false), 0.5, "a low score is its own limit");
        assert_eq!(confidence(1.0, true), 1.0);
        assert_eq!(confidence(5.0, true), 1.0, "brought into range");
    }

    #[test]
    fn a_date_gives_up_its_year_and_nothing_else_does() {
        assert_eq!(year_of(Some("2019-04-21")), Some(2019));
        assert_eq!(year_of(Some("2019")), Some(2019));
        assert_eq!(year_of(Some("")), None);
        assert_eq!(year_of(Some("unknown")), None);
        assert_eq!(year_of(None), None);
    }

    #[test]
    fn a_tag_cannot_become_part_of_the_query() {
        // A tag with quotes and a Lucene operator in it goes in as one
        // quoted phrase and cannot end it early.
        let nasty = r#"AC/DC" OR artist:* AND "x"#;
        let quoted = quoted(nasty);
        assert!(quoted.starts_with('"') && quoted.ends_with('"'));
        assert_eq!(
            quoted.matches('"').count(),
            2,
            "only the two that Onsa put there: {quoted}"
        );
        assert!(!quoted.contains('\\'));
        assert!(quoted.contains("AC/DC"), "the name itself is kept");
    }

    #[test]
    fn only_something_shaped_like_an_id_is_put_in_a_path() {
        assert!(looks_like_an_id("11111111-2222-3333-4444-555555555555"));
        for wrong in [
            "../../etc/passwd",
            "11111111-2222-3333-4444-55555555555",
            "11111111/2222/3333/4444/555555555555x",
            "",
        ] {
            assert!(!looks_like_an_id(wrong), "{wrong}");
        }
        assert_eq!(cover("../secret"), Err(NetError::Unreachable));
        assert_eq!(release_group("../secret"), Ok(None));
    }

    #[test]
    fn a_search_with_nothing_to_search_for_asks_nothing() {
        assert_eq!(search("Lilith", "   ", None), Ok(Vec::new()));
    }

    #[test]
    fn an_answer_with_no_releases_still_gives_what_it_has() {
        let thin: serde_json::Value = serde_json::from_str(
            r#"{"recordings":[{"id":"x","score":88,"title":"Sore","artist-credit":[{"name":"Lilith"}]}]}"#,
        )
        .expect("valid");
        let found = read_search(&thin, "Lilith", "Sore");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].album, None);
        assert_eq!(found[0].year, None);
        assert!((found[0].confidence - 0.88).abs() < 1e-9);
    }
}
