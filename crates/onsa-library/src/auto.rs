//! Tidying up what the tags already say (SPEC §8).
//!
//! Nothing here asks the internet anything. It reads what the library holds
//! and suggests what it would look like written consistently: the leftovers
//! of a download taken off a title, a title that shouts or whispers set in
//! ordinary case, a credit written one way rather than four, and a name that
//! appears three ways in one library settled on the spelling that appears
//! most.
//!
//! Every one of these is a **suggestion**, and it carries the reason it was
//! made so the listener can see why. Applying them goes through
//! [`Library::apply_edits`] like any other change: the same folder, the same
//! summary, the same run that can be taken back.
//!
//! The rules are deliberately shy. Text somebody clearly wrote on purpose —
//! mixed case, a name in capitals among names that are not, anything that is
//! not written in Latin letters — is left exactly as it is. A tidier that
//! overwrites intention is worse than no tidier at all.

use std::collections::HashMap;

use crate::clean;
use crate::db::Library;
use crate::edits::Scope;
use crate::error::Result;
use crate::overrides::Field;

/// Why a suggestion was made. A fixed word per reason, for the interface to
/// translate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// What a download left behind: `(Official Video)`, a video id.
    Residue,
    /// Text written all in capitals or all in lower case.
    Capitals,
    /// A featuring credit written some other way.
    Credit,
    /// A name that appears more than one way in this library.
    Spelling,
    /// An album with no album artist, filed under whoever it is mostly by.
    AlbumArtist,
}

impl Reason {
    /// The word the interface translates.
    pub fn name(self) -> &'static str {
        match self {
            Self::Residue => "residue",
            Self::Capitals => "capitals",
            Self::Credit => "credit",
            Self::Spelling => "spelling",
            Self::AlbumArtist => "albumArtist",
        }
    }
}

/// One thing Onsa would write differently.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// The track.
    pub track_id: i64,
    /// Where its file is, for reading the list by.
    pub path: String,
    /// The field it is about.
    pub field: Field,
    /// What it says now.
    pub current: Option<String>,
    /// What it would say.
    pub value: String,
    /// Why, in the order the rules applied.
    pub reasons: Vec<Reason>,
}

/// How a name is spelled across the whole library: the key every spelling of
/// one name shares, and the spelling that appears most often under it.
type Spellings = HashMap<String, String>;

impl Library {
    /// What Onsa would write differently about the tracks in scope.
    ///
    /// The spellings are counted across the **whole** library, because that
    /// is where the answer lives: a folder of ten tracks cannot say which of
    /// three spellings of a name is the usual one. Only tracks inside the
    /// folder are suggested changes.
    pub fn tidy_suggestions(&self, scope: &Scope, limit: usize) -> Result<Vec<Suggestion>> {
        let spellings = self.usual_spellings()?;
        let mut found = Vec::new();
        for track in self.tracks_in_scope(scope, limit)? {
            let mut about = |field: Field, current: Option<&String>| {
                if let Some(one) = suggest(track.id, &track.path, field, current, &spellings) {
                    found.push(one);
                }
            };
            about(Field::Title, track.title.as_ref());
            about(Field::Album, track.album.as_ref());
            about(Field::Artist, track.artist.as_ref());
            about(Field::AlbumArtist, track.album_artist.as_ref());

            // An album with nobody to file it under is filed under whoever
            // the track is mostly by, so one album does not become four.
            let missing = track
                .album_artist
                .as_deref()
                .map(str::trim)
                .is_none_or(str::is_empty);
            if missing && track.album.is_some() {
                if let Some(artist) = track.artist.as_deref().map(str::trim) {
                    let main = clean::main_artist(artist);
                    if !main.is_empty() {
                        found.push(Suggestion {
                            track_id: track.id,
                            path: track.path.clone(),
                            field: Field::AlbumArtist,
                            current: track.album_artist.clone(),
                            value: main,
                            reasons: vec![Reason::AlbumArtist],
                        });
                    }
                }
            }
        }
        Ok(found)
    }

    /// The spelling each name usually has in this library.
    ///
    /// Names are grouped by what they have in common once case, spaces and
    /// punctuation are set aside, and the spelling used by the most tracks
    /// wins. Nothing is invented: the winner is always a spelling that
    /// already appears. A tie leaves the name out entirely — if the library
    /// itself cannot say which is usual, Onsa certainly cannot.
    fn usual_spellings(&self) -> Result<Spellings> {
        let mut counts: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut statement = self.conn.prepare(
            "SELECT artist, COUNT(*) FROM track_view
             WHERE artist IS NOT NULL AND TRIM(artist) <> '' GROUP BY artist
             UNION ALL
             SELECT album_artist, COUNT(*) FROM track_view
             WHERE album_artist IS NOT NULL AND TRIM(album_artist) <> '' GROUP BY album_artist",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;
        for row in rows {
            let (name, count) = row?;
            let key = clean::same_name_key(&name);
            if key.is_empty() {
                continue;
            }
            *counts.entry(key).or_default().entry(name).or_default() += count;
        }

        let mut usual = Spellings::new();
        for (key, spellings) in counts {
            if spellings.len() < 2 {
                continue;
            }
            let most = spellings.values().copied().max().unwrap_or_default();
            let mut winners: Vec<&String> = spellings
                .iter()
                .filter(|(_, count)| **count == most)
                .map(|(name, _)| name)
                .collect();
            if winners.len() != 1 {
                // Two spellings used equally often: the library has not
                // decided, so neither does Onsa.
                continue;
            }
            if let Some(winner) = winners.pop() {
                usual.insert(key, winner.clone());
            }
        }
        Ok(usual)
    }
}

/// What one field would say once tidied, or nothing if it already does.
fn suggest(
    track_id: i64,
    path: &str,
    field: Field,
    current: Option<&String>,
    spellings: &Spellings,
) -> Option<Suggestion> {
    let current = current.map(String::as_str).map(str::trim)?;
    if current.is_empty() {
        return None;
    }
    let mut value = current.to_string();
    let mut reasons = Vec::new();

    let names = matches!(field, Field::Artist | Field::AlbumArtist);

    // A name the library spells one way most of the time is spelled that
    // way. This runs first, and on its own: the spelling that won is one
    // somebody actually uses, and correcting its case afterwards would be
    // Onsa arguing with the library.
    if names {
        if let Some(usual) = spellings.get(&clean::same_name_key(&value)) {
            if usual != &value {
                return Some(Suggestion {
                    track_id,
                    path: path.to_string(),
                    field,
                    current: Some(current.to_string()),
                    value: usual.clone(),
                    reasons: vec![Reason::Spelling],
                });
            }
        }
    }

    if !names {
        let cleaned = clean::strip_residue(&value);
        if cleaned != value && !cleaned.is_empty() {
            value = cleaned;
            reasons.push(Reason::Residue);
        }
    } else {
        let credited = clean::tidy_credit(&value);
        if credited != value {
            value = credited;
            reasons.push(Reason::Credit);
        }
    }

    if clean::one_case(&value) {
        let cased = clean::title_case(&value);
        if cased != value {
            value = cased;
            reasons.push(Reason::Capitals);
        }
    }

    if reasons.is_empty() || value == current {
        return None;
    }
    Some(Suggestion {
        track_id,
        path: path.to_string(),
        field,
        current: Some(current.to_string()),
        value,
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reason_has_a_word_of_its_own() {
        let all = [
            Reason::Residue,
            Reason::Capitals,
            Reason::Credit,
            Reason::Spelling,
            Reason::AlbumArtist,
        ];
        let mut names: Vec<&str> = all.iter().map(|reason| reason.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), all.len());
    }

    #[test]
    fn a_title_loses_what_the_download_left_and_stops_shouting() {
        let one = suggest(
            1,
            "/music/x.mp3",
            Field::Title,
            Some(&"BANG BANG (Official Video)".to_string()),
            &Spellings::new(),
        )
        .expect("something to tidy");
        assert_eq!(one.value, "Bang Bang");
        assert_eq!(one.reasons, vec![Reason::Residue, Reason::Capitals]);
    }

    #[test]
    fn a_title_somebody_wrote_carefully_is_left_alone() {
        // A name in capitals, a script with no case, a deliberate mixture,
        // and a single word are all somebody's decision.
        for careful in [
            "Lampu Kota",
            "夜明けのうた",
            "iPhone Era",
            "AC/DC",
            "LILITH",
        ] {
            assert_eq!(
                suggest(
                    1,
                    "/music/x.mp3",
                    Field::Title,
                    Some(&careful.to_string()),
                    &Spellings::new()
                ),
                None,
                "{careful}"
            );
        }
        // A whole phrase in capitals is offered a correction — offered, not
        // applied: the interface leaves this one unticked.
        assert_eq!(
            suggest(
                1,
                "/music/x.mp3",
                Field::Title,
                Some(&"BANG BANG".to_string()),
                &Spellings::new()
            )
            .expect("case is offered")
            .value,
            "Bang Bang"
        );
    }

    #[test]
    fn a_credit_is_settled_on_one_way_of_writing_it() {
        let one = suggest(
            1,
            "/music/x.mp3",
            Field::Artist,
            Some(&"Lilith ft ミナ".to_string()),
            &Spellings::new(),
        )
        .expect("something to tidy");
        assert_eq!(one.value, "Lilith feat. ミナ");
        assert_eq!(one.reasons, vec![Reason::Credit]);
    }

    #[test]
    fn a_name_the_library_usually_spells_one_way_is_spelled_that_way() {
        let mut spellings = Spellings::new();
        spellings.insert(clean::same_name_key("LILITH"), "Lilith".to_string());
        let one = suggest(
            1,
            "/music/x.mp3",
            Field::Artist,
            Some(&"LILITH".to_string()),
            &spellings,
        )
        .expect("something to tidy");
        assert_eq!(one.value, "Lilith");
        assert_eq!(
            one.reasons,
            vec![Reason::Spelling],
            "and its case is not argued with afterwards"
        );

        // The spelling that won is left exactly as it is.
        assert_eq!(
            suggest(
                1,
                "/music/x.mp3",
                Field::Artist,
                Some(&"Lilith".to_string()),
                &spellings
            ),
            None
        );
    }

    #[test]
    fn an_empty_field_is_not_something_to_tidy() {
        for nothing in [None, Some(&String::new()), Some(&"   ".to_string())] {
            assert_eq!(
                suggest(1, "/music/x.mp3", Field::Title, nothing, &Spellings::new()),
                None
            );
        }
    }
}
