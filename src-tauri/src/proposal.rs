//! What the internet suggests about a track, and how sure it is (SPEC §8).
//!
//! Nothing here is applied. A proposal is a list of fields with what the
//! track says now and what a service thinks it should say, and every one of
//! them carries a mark saying whether it is ticked. **A match Onsa is not
//! confident about arrives untitled**: the fields are there to read, but
//! none of them is ticked, so pressing apply without reading changes nothing.
//!
//! That matters most for exactly the library this was built for. A folder of
//! downloads with no tags gives a fingerprint matcher very little to go on,
//! and it will be wrong often; the ones it is unsure about must not ride
//! along with the ones it is sure of.

// These are the rails, and they were built before the thing they hold back:
// the owner asked for the safety before the first automatic write, not after
// it. Until the matching code lands, some of this has no caller yet.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// At or above this, a match is ticked by default.
///
/// AcoustID hands back a score between nothing and one. Below this, matches
/// are common enough to be worth reading one by one.
pub const TRUSTED: f64 = 0.85;
/// Below this a match is not worth showing at all.
pub const WORTH_SHOWING: f64 = 0.5;

/// Where a suggestion came from, so the listener can weigh it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    /// Recognised from the sound itself.
    AcoustId,
    /// Looked up by what the track already says.
    MusicBrainz,
    /// Worked out from the file name or the folder.
    FileName,
}

/// One field a service would change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldProposal {
    /// The field, by the name the overrides table uses.
    pub field: String,
    /// What the track says now, if anything.
    pub current: Option<String>,
    /// What the service suggests.
    pub suggested: Option<String>,
    /// Whether this one is ticked. Applying takes the ticked ones only.
    pub chosen: bool,
}

/// A picture a suggestion comes with, waiting to be looked at.
///
/// The picture itself is not in here. It is kept in the cover cache under
/// this name and served to the interface over Onsa's own protocol, because
/// a cover belongs in an `img` tag, not in an event (SPEC §2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverOffer {
    /// What it is called in the cache: the SHA-256 of the picture.
    pub key: String,
    /// How wide the picture is.
    pub width: u32,
    /// How tall it is.
    pub height: u32,
    /// Whether the listener wants it. Never ticked on its own: a cover is
    /// the one suggestion that is quicker to judge by looking than by
    /// reading, so it waits to be looked at.
    pub chosen: bool,
}

/// Everything suggested for one track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    /// The track it is about.
    pub track_id: i64,
    /// What the file is called now, for reading the list by.
    pub path: String,
    /// Where the suggestion came from.
    pub source: Source,
    /// How sure the service is, from nothing to one.
    pub confidence: f64,
    /// Whether Onsa is sure enough to tick it without being asked.
    pub trusted: bool,
    /// The fields themselves.
    pub fields: Vec<FieldProposal>,
    /// A cover that came with it, when the service had one.
    pub cover: Option<CoverOffer>,
    /// The release group it belongs to, which is what a cover is filed
    /// under. Not shown; it is how the picture is found.
    pub release_group: Option<String>,
}

impl Proposal {
    /// Builds a proposal, ticking its fields only when the match is one to
    /// be trusted **and** the field would actually change something.
    ///
    /// A field that suggests what the track already says is never ticked:
    /// applying it would be a step in the record with nothing behind it.
    pub fn new(
        track_id: i64,
        path: impl Into<String>,
        source: Source,
        confidence: f64,
        fields: Vec<FieldProposal>,
    ) -> Self {
        let confidence = confidence.clamp(0.0, 1.0);
        let trusted = confidence >= TRUSTED;
        let fields = fields
            .into_iter()
            .map(|field| {
                let changes = field.current.as_deref() != field.suggested.as_deref();
                FieldProposal {
                    chosen: trusted && changes,
                    ..field
                }
            })
            .collect();
        Self {
            track_id,
            path: path.into(),
            source,
            confidence,
            trusted,
            fields,
            cover: None,
            release_group: None,
        }
    }

    /// The same proposal, remembering which release it came from.
    pub fn about(mut self, release_group: Option<String>) -> Self {
        self.release_group = release_group;
        self
    }

    /// The same proposal with a picture attached, unticked.
    pub fn with_cover(mut self, cover: Option<CoverOffer>) -> Self {
        self.cover = cover.map(|cover| CoverOffer {
            chosen: false,
            ..cover
        });
        self
    }

    /// What this suggestion says a track is, for comparing two of them.
    fn says(&self, field: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|one| one.field == field)
            .and_then(|one| one.suggested.as_deref())
    }

    /// Takes every tick off, for when the listener has to choose first.
    fn untick(&mut self) {
        for field in &mut self.fields {
            field.chosen = false;
        }
        if let Some(cover) = &mut self.cover {
            cover.chosen = false;
        }
    }

    /// Whether anything would happen if this proposal were applied as it is.
    pub fn would_change(&self) -> bool {
        self.fields.iter().any(|field| field.chosen)
    }

    /// The fields that are ticked, as changes to apply.
    pub fn chosen(&self) -> Vec<(String, Option<String>)> {
        self.fields
            .iter()
            .filter(|field| field.chosen)
            .map(|field| (field.field.clone(), field.suggested.clone()))
            .collect()
    }
}

/// Everything that was found out about one track, and what came of it.
///
/// A track can be recognised two ways at once — by its sound and by the name
/// it was saved under — and the two do not always say the same thing. When
/// they disagree, **both are kept and neither is ticked**: a disagreement is
/// exactly the case a person has to look at, and picking one of them
/// silently would be the machine choosing while appearing not to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMatch {
    /// The track.
    pub track_id: i64,
    /// Where its file is, for reading the list by.
    pub path: String,
    /// What was found, the surest first.
    pub candidates: Vec<Proposal>,
    /// Which one is picked, when one is.
    pub picked: Option<usize>,
    /// Whether what was found disagrees with itself.
    pub disagree: bool,
    /// Why nothing was found, when nothing was: a fixed word to translate.
    pub failure: Option<String>,
}

impl TrackMatch {
    /// Gathers what was found for a track and decides what, if anything, is
    /// ticked without being asked.
    ///
    /// Only one thing ticks itself: a single answer Onsa is sure of, or two
    /// answers that are sure and agree. Anything else waits.
    pub fn new(track_id: i64, path: impl Into<String>, mut candidates: Vec<Proposal>) -> Self {
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let disagree = candidates.len() > 1 && !agree(&candidates[0], &candidates[1]);
        let picked = match candidates.first() {
            Some(best) if best.trusted && !disagree => Some(0),
            _ => None,
        };
        if picked.is_none() {
            for candidate in &mut candidates {
                candidate.untick();
            }
        }
        Self {
            track_id,
            path: path.into(),
            candidates,
            picked,
            disagree,
            failure: None,
        }
    }

    /// A track nothing could be found out about, and the reason.
    pub fn nothing(track_id: i64, path: impl Into<String>, why: impl Into<String>) -> Self {
        Self {
            track_id,
            path: path.into(),
            candidates: Vec::new(),
            picked: None,
            disagree: false,
            failure: Some(why.into()),
        }
    }

    /// The candidate the listener is going by, if any.
    pub fn chosen(&self) -> Option<&Proposal> {
        self.candidates.get(self.picked?)
    }

    /// Whether applying this as it stands would change anything.
    pub fn would_change(&self) -> bool {
        self.chosen().is_some_and(Proposal::would_change)
    }
}

/// Whether two suggestions say the same thing about what a track is.
///
/// Only the title and the artist are compared, and only by their letters and
/// numbers: one source writing "BANG BANG" and another "Bang Bang" is not a
/// disagreement, and neither is a full stop. A source that does not say at
/// all is not disagreeing either — it is only quieter.
fn agree(a: &Proposal, b: &Proposal) -> bool {
    ["title", "artist"]
        .iter()
        .all(|field| match (a.says(field), b.says(field)) {
            (Some(one), Some(other)) => squeeze(one) == squeeze(other),
            _ => true,
        })
}

/// A value with only its letters and numbers, in lower case.
fn squeeze(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, current: Option<&str>, suggested: Option<&str>) -> FieldProposal {
        FieldProposal {
            field: name.to_string(),
            current: current.map(str::to_string),
            suggested: suggested.map(str::to_string),
            // Whatever a caller puts here, the constructor decides.
            chosen: true,
        }
    }

    #[test]
    fn a_sure_match_arrives_ticked() {
        let proposal = Proposal::new(
            1,
            "/music/x.mp3",
            Source::AcoustId,
            0.97,
            vec![
                field("title", None, Some("Lampu Kota")),
                field("artist", None, Some("ミナ")),
            ],
        );
        assert!(proposal.trusted);
        assert!(proposal.would_change());
        assert_eq!(proposal.chosen().len(), 2);
    }

    #[test]
    fn an_unsure_match_arrives_with_nothing_ticked() {
        let proposal = Proposal::new(
            1,
            "/music/x.mp3",
            Source::AcoustId,
            0.62,
            vec![field("title", None, Some("Mungkin Ini"))],
        );
        assert!(!proposal.trusted);
        assert!(
            !proposal.would_change(),
            "applying without reading must do nothing"
        );
        assert!(proposal.chosen().is_empty());
        assert_eq!(
            proposal.fields.len(),
            1,
            "but it is still there to be read and ticked by hand"
        );
    }

    #[test]
    fn the_line_is_where_the_specification_puts_it() {
        let at = Proposal::new(1, "x", Source::AcoustId, TRUSTED, vec![]);
        assert!(at.trusted, "exactly at the mark counts as sure");
        let under = Proposal::new(1, "x", Source::AcoustId, TRUSTED - 0.001, vec![]);
        assert!(!under.trusted);
        // There has to be room between "worth reading" and "sure enough
        // to tick", or the untrusted band would be empty.
        const _: () = assert!(WORTH_SHOWING < TRUSTED);
    }

    #[test]
    fn a_field_that_changes_nothing_is_never_ticked() {
        let proposal = Proposal::new(
            1,
            "/music/x.mp3",
            Source::MusicBrainz,
            1.0,
            vec![
                field("title", Some("Lampu Kota"), Some("Lampu Kota")),
                field("album", Some("Lama"), Some("City Lights")),
            ],
        );
        assert!(proposal.trusted);
        assert_eq!(
            proposal.chosen(),
            vec![("album".to_string(), Some("City Lights".to_string()))],
            "only the one that would actually change"
        );
    }

    fn from(source: Source, confidence: f64, title: &str, artist: &str) -> Proposal {
        Proposal::new(
            7,
            "/music/x.mp3",
            source,
            confidence,
            vec![
                field("title", None, Some(title)),
                field("artist", None, Some(artist)),
            ],
        )
    }

    #[test]
    fn one_sure_answer_ticks_itself() {
        let found = TrackMatch::new(
            7,
            "/music/x.mp3",
            vec![from(Source::AcoustId, 0.96, "Lampu Kota", "Lilith")],
        );
        assert_eq!(found.picked, Some(0));
        assert!(!found.disagree);
        assert!(found.would_change());
    }

    #[test]
    fn two_answers_that_agree_leave_the_surer_one_ticked() {
        // The sound and the file name say the same thing in different
        // letters, which is not a disagreement.
        let found = TrackMatch::new(
            7,
            "/music/x.mp3",
            vec![
                from(Source::FileName, 0.6, "bang bang!", "lilith"),
                from(Source::AcoustId, 0.97, "BANG BANG", "Lilith"),
            ],
        );
        assert_eq!(
            found.candidates[0].source,
            Source::AcoustId,
            "the surer one"
        );
        assert!(!found.disagree);
        assert_eq!(found.picked, Some(0));
        assert!(found.candidates[0].would_change());
    }

    #[test]
    fn two_answers_that_differ_are_both_kept_and_neither_is_ticked() {
        let found = TrackMatch::new(
            7,
            "/music/x.mp3",
            vec![
                from(Source::AcoustId, 0.99, "Lampu Kota", "Lilith"),
                from(Source::FileName, 0.6, "Sore", "Mesin"),
            ],
        );
        assert!(found.disagree);
        assert_eq!(found.picked, None, "this one is the listener's to settle");
        assert_eq!(found.candidates.len(), 2, "both are there to read");
        assert!(
            found.candidates.iter().all(|one| !one.would_change()),
            "and applying without settling it does nothing"
        );
        assert!(!found.would_change());
    }

    #[test]
    fn a_quiet_source_is_not_a_disagreeing_one() {
        // A file name that gave up a title but no artist does not contradict
        // a fingerprint that named both.
        let quiet = Proposal::new(
            7,
            "/music/x.mp3",
            Source::FileName,
            0.35,
            vec![field("title", None, Some("Lampu Kota"))],
        );
        let found = TrackMatch::new(
            7,
            "/music/x.mp3",
            vec![from(Source::AcoustId, 0.95, "Lampu Kota", "Lilith"), quiet],
        );
        assert!(!found.disagree);
        assert_eq!(found.picked, Some(0));
    }

    #[test]
    fn an_unsure_answer_on_its_own_still_waits() {
        let found = TrackMatch::new(
            7,
            "/music/x.mp3",
            vec![from(Source::FileName, 0.6, "Lampu Kota", "Lilith")],
        );
        assert_eq!(found.picked, None);
        assert!(!found.disagree, "there is nothing to disagree with");
        assert!(!found.would_change());
    }

    #[test]
    fn a_track_nothing_was_found_for_says_why() {
        let none = TrackMatch::nothing(7, "/music/x.mp3", "noFingerprinter");
        assert_eq!(none.failure.as_deref(), Some("noFingerprinter"));
        assert!(none.candidates.is_empty());
        assert!(!none.would_change());
    }

    #[test]
    fn a_cover_is_never_ticked_before_it_has_been_looked_at() {
        let offered =
            from(Source::AcoustId, 0.99, "Lampu Kota", "Lilith").with_cover(Some(CoverOffer {
                key: "abc".into(),
                width: 500,
                height: 500,
                chosen: true,
            }));
        let cover = offered.cover.clone().expect("a cover");
        assert!(!cover.chosen, "a picture waits to be looked at");
        let found = TrackMatch::new(7, "/music/x.mp3", vec![offered]);
        assert_eq!(found.picked, Some(0), "the fields still tick themselves");
        assert!(!found.candidates[0].cover.as_ref().expect("a cover").chosen);
    }

    #[test]
    fn a_confidence_outside_its_range_is_brought_back_into_it() {
        assert_eq!(
            Proposal::new(1, "x", Source::FileName, 5.0, vec![]).confidence,
            1.0
        );
        assert_eq!(
            Proposal::new(1, "x", Source::FileName, -1.0, vec![]).confidence,
            0.0
        );
    }
}
