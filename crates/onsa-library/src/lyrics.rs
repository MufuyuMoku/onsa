//! Where a song's words are kept between one playing and the next (SPEC §10).
//!
//! Two different things live in one row, because they belong to the same
//! song and neither is worth a table of its own.
//!
//! The first is what a source on the internet said, so it is not asked the
//! same question every time the song comes round. The second is the offset
//! the listener tuned by ear, which belongs to the song rather than to any
//! source: replacing the words leaves it alone, and clearing the words
//! leaves it alone too.
//!
//! What is beside the song on the disk is never cached. Reading a small
//! text file again costs less than working out whether what was remembered
//! about it is still true.

use rusqlite::{params, OptionalExtension};

use crate::db::Library;
use crate::error::Result;

/// The source said there are no words to be had.
///
/// Kept so a song without lyrics is not asked about every time it plays.
pub const NOTHING: &str = "none";

/// What is remembered about one song's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cached {
    /// Where the text came from, or [`NOTHING`] when the source had none.
    pub source: Option<String>,
    /// The words, as the source gave them: LRC, or plain lines.
    pub text: Option<String>,
    /// Whether those words carry timing.
    pub synced: bool,
    /// What the listener tuned, in milliseconds. Positive holds them back.
    pub offset_ms: i64,
    /// When the source was asked, in milliseconds since the Unix epoch.
    pub fetched_at: Option<i64>,
}

impl Cached {
    /// Whether a source has already been asked about this song.
    ///
    /// A remembered miss counts: the point of keeping it is not to ask
    /// again.
    pub fn was_asked(&self) -> bool {
        self.source.is_some()
    }

    /// The words, when there are any.
    pub fn words(&self) -> Option<&str> {
        self.text.as_deref().filter(|text| !text.trim().is_empty())
    }
}

impl Library {
    /// What is remembered about a song's words, if anything is.
    pub fn lyrics(&self, track_id: i64) -> Result<Option<Cached>> {
        Ok(self
            .conn
            .query_row(
                "SELECT source, text, synced, offset_ms, fetched_at FROM lyrics WHERE track_id = ?1",
                [track_id],
                |row| {
                    Ok(Cached {
                        source: row.get(0)?,
                        text: row.get(1)?,
                        synced: row.get::<_, i64>(2)? != 0,
                        offset_ms: row.get(3)?,
                        fetched_at: row.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    /// Remembers what a source answered, the offset left as it was.
    ///
    /// `text` of `None` with a source of [`NOTHING`] is how a miss is
    /// remembered.
    pub fn remember_lyrics(
        &mut self,
        track_id: i64,
        source: &str,
        text: Option<&str>,
        synced: bool,
    ) -> Result<()> {
        let now = now_ms();
        self.conn.execute(
            "INSERT INTO lyrics (track_id, source, text, synced, offset_ms, fetched_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5)
             ON CONFLICT(track_id) DO UPDATE SET
                 source = excluded.source,
                 text = excluded.text,
                 synced = excluded.synced,
                 fetched_at = excluded.fetched_at",
            params![track_id, source, text, i64::from(synced), now],
        )?;
        Ok(())
    }

    /// Sets the offset the listener tuned, the words left as they were.
    pub fn set_lyrics_offset(&mut self, track_id: i64, offset_ms: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO lyrics (track_id, offset_ms) VALUES (?1, ?2)
             ON CONFLICT(track_id) DO UPDATE SET offset_ms = excluded.offset_ms",
            params![track_id, offset_ms],
        )?;
        Ok(())
    }

    /// Forgets what a source said, so it can be asked again. The offset
    /// stays: it belongs to the song, not to the answer.
    pub fn forget_lyrics(&mut self, track_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE lyrics SET source = NULL, text = NULL, synced = 0, fetched_at = NULL
             WHERE track_id = ?1",
            [track_id],
        )?;
        Ok(())
    }
}

/// Now, in milliseconds since the Unix epoch.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A library in memory with one track in it, which is all these need.
    fn library_with_track() -> (Library, i64) {
        let cache = std::env::temp_dir().join(format!("onsa-lyrics-db-{}", std::process::id()));
        std::fs::create_dir_all(&cache).expect("a folder for the covers");
        let library = Library::open_in_memory(&cache).expect("a library");
        library
            .conn
            .execute(
                "INSERT INTO folders (id, path, added_at) VALUES (1, '/music', 0)",
                [],
            )
            .expect("a folder");
        library
            .conn
            .execute(
                "INSERT INTO tracks (id, path, folder_id, mtime, size, added_at)
                 VALUES (1, '/music/lagu.mp3', 1, 0, 0, 0)",
                [],
            )
            .expect("a track");
        (library, 1)
    }

    #[test]
    fn a_song_nothing_is_known_about_has_no_row() {
        let (library, track) = library_with_track();
        assert_eq!(library.lyrics(track).expect("it should ask"), None);
    }

    #[test]
    fn what_a_source_said_is_remembered() {
        let (mut library, track) = library_with_track();
        library
            .remember_lyrics(track, "lrclib", Some("[00:01.00]satu"), true)
            .expect("it should remember");
        let kept = library
            .lyrics(track)
            .expect("it should ask")
            .expect("a row by now");
        assert_eq!(kept.source.as_deref(), Some("lrclib"));
        assert_eq!(kept.words(), Some("[00:01.00]satu"));
        assert!(kept.synced);
        assert!(kept.was_asked());
        assert!(kept.fetched_at.is_some());
    }

    #[test]
    fn a_source_with_nothing_to_say_is_remembered_too() {
        let (mut library, track) = library_with_track();
        library
            .remember_lyrics(track, NOTHING, None, false)
            .expect("it should remember");
        let kept = library
            .lyrics(track)
            .expect("it should ask")
            .expect("a row by now");
        assert!(kept.was_asked(), "so it is not asked again");
        assert_eq!(kept.words(), None);
    }

    #[test]
    fn the_offset_outlives_the_words() {
        let (mut library, track) = library_with_track();
        library
            .set_lyrics_offset(track, -750)
            .expect("it should set");
        library
            .remember_lyrics(track, "lrclib", Some("[00:01.00]satu"), true)
            .expect("it should remember");
        let kept = library
            .lyrics(track)
            .expect("it should ask")
            .expect("a row by now");
        assert_eq!(
            kept.offset_ms, -750,
            "tuning by ear is not a source's to undo"
        );

        library.forget_lyrics(track).expect("it should forget");
        let after = library
            .lyrics(track)
            .expect("it should ask")
            .expect("a row still");
        assert_eq!(after.offset_ms, -750);
        assert!(!after.was_asked(), "and it can be asked again");
    }

    #[test]
    fn an_offset_can_be_set_before_there_are_any_words() {
        let (mut library, track) = library_with_track();
        library
            .set_lyrics_offset(track, 500)
            .expect("it should set");
        let kept = library
            .lyrics(track)
            .expect("it should ask")
            .expect("a row by now");
        assert_eq!(kept.offset_ms, 500);
        assert!(!kept.was_asked());
    }

    #[test]
    fn a_track_that_goes_takes_its_lyrics_with_it() {
        let (mut library, track) = library_with_track();
        library
            .remember_lyrics(track, "lrclib", Some("satu"), false)
            .expect("it should remember");
        library
            .conn
            .execute("DELETE FROM tracks WHERE id = ?1", [track])
            .expect("the track should go");
        assert_eq!(library.lyrics(track).expect("it should ask"), None);
    }
}
