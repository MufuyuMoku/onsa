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
