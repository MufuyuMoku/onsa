//! Onsa lyrics.
//!
//! Owns the LRC format, the lyrics that come from files (a `.lrc` beside the
//! song, or the words kept inside the song's own tags) and the shape of what
//! LRCLIB is asked and answers (SPEC §10).
//!
//! What this crate does not own is the network and the database. The one
//! HTTP client belongs to the application (SPEC §1) and the cache belongs to
//! the library, so what is here builds a question and reads an answer while
//! somebody else carries it. That keeps this crate free of the other Onsa
//! crates, as SPEC §2 asks.

#![warn(missing_docs)]

pub mod beside;
pub mod lrc;
pub mod lrclib;

use thiserror::Error;

/// Result alias for lyrics operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the lyrics module can fail at.
///
/// Reading lyrics is not one of them: a file Onsa cannot make sense of is
/// lyrics it does not have, which is an ordinary thing rather than a
/// failure. What can fail is writing, and only writing.
#[derive(Debug, Error)]
pub enum Error {
    /// A lyrics file could not be read or written.
    #[error("lyrics i/o failed: {0}")]
    Io(#[from] std::io::Error),
    /// A remote lyrics source answered with something unusable.
    #[error("lyrics source failed: {0}")]
    Source(String),
}

/// One word, and when it is sung.
///
/// Only files with per-word timing carry these (SPEC §10); most do not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// When it begins, in milliseconds from the start of the song.
    pub at_ms: i64,
    /// The word, with whatever spacing followed it in the file.
    pub text: String,
}

/// One line of lyrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// When it is sung, or `None` for a line the file gave no time.
    pub at_ms: Option<i64>,
    /// The words, as one string, markers taken out.
    pub text: String,
    /// The words on their own, when the file timed them one by one.
    pub words: Vec<Word>,
}

impl Line {
    /// A line with a time and nothing else about it.
    pub fn at(at_ms: i64, text: impl Into<String>) -> Self {
        Self {
            at_ms: Some(at_ms),
            text: text.into(),
            words: Vec::new(),
        }
    }

    /// A line the file gave no time.
    pub fn untimed(text: impl Into<String>) -> Self {
        Self {
            at_ms: None,
            text: text.into(),
            words: Vec::new(),
        }
    }
}

/// A song's words, in the order they are shown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lyrics {
    /// The lines. Timed ones are in time order; a line without a time keeps
    /// its place among the ones it was written between.
    pub lines: Vec<Line>,
    /// What the file's own `[offset:]` asked for, in milliseconds.
    ///
    /// It is already in [`Line::at_ms`]; this is kept so the value can be
    /// shown and tested, not so it can be applied again.
    pub offset_ms: i64,
}

impl Lyrics {
    /// Whether the words are timed, and can follow the song.
    pub fn synced(&self) -> bool {
        self.lines.iter().any(|line| line.at_ms.is_some())
    }

    /// Whether there is nothing to show.
    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|line| line.text.trim().is_empty())
    }

    /// Which line belongs to a moment, counting an offset the listener set.
    ///
    /// A positive offset moves the words later, which is what a listener who
    /// feels the lyrics arrive too early reaches for. Before the first line
    /// there is no answer.
    pub fn line_at(&self, at_ms: i64, offset_ms: i64) -> Option<usize> {
        let want = at_ms - offset_ms;
        let mut found = None;
        for (index, line) in self.lines.iter().enumerate() {
            match line.at_ms {
                Some(time) if time <= want => found = Some(index),
                Some(_) => break,
                None => {}
            }
        }
        found
    }

    /// The words as plain text, which is how untimed lyrics are shown.
    pub fn plain(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three() -> Lyrics {
        Lyrics {
            lines: vec![
                Line::at(1_000, "satu"),
                Line::at(2_000, "dua"),
                Line::at(3_000, "tiga"),
            ],
            offset_ms: 0,
        }
    }

    #[test]
    fn the_line_before_a_moment_is_the_one_being_sung() {
        let lyrics = three();
        assert_eq!(lyrics.line_at(0, 0), None);
        assert_eq!(lyrics.line_at(1_000, 0), Some(0));
        assert_eq!(lyrics.line_at(2_500, 0), Some(1));
        assert_eq!(lyrics.line_at(90_000, 0), Some(2));
    }

    #[test]
    fn a_positive_offset_holds_the_words_back() {
        let lyrics = three();
        // With the words half a second late, the second line has not
        // arrived yet at two seconds.
        assert_eq!(lyrics.line_at(2_000, 500), Some(0));
        assert_eq!(lyrics.line_at(2_500, 500), Some(1));
        // And a negative one brings them forward.
        assert_eq!(lyrics.line_at(1_800, -500), Some(1));
    }

    #[test]
    fn an_untimed_line_is_never_the_one_being_sung() {
        let lyrics = Lyrics {
            lines: vec![
                Line::at(1_000, "satu"),
                Line::untimed("(instrumental)"),
                Line::at(3_000, "tiga"),
            ],
            offset_ms: 0,
        };
        assert_eq!(lyrics.line_at(2_000, 0), Some(0));
        assert_eq!(lyrics.line_at(3_500, 0), Some(2));
        assert!(lyrics.synced());
    }

    #[test]
    fn lyrics_without_a_single_time_are_plain_text() {
        let lyrics = Lyrics {
            lines: vec![Line::untimed("satu"), Line::untimed("dua")],
            offset_ms: 0,
        };
        assert!(!lyrics.synced());
        assert_eq!(lyrics.plain(), "satu\ndua");
        assert_eq!(lyrics.line_at(5_000, 0), None);
    }
}
