//! Statistics and play history (SPEC §5.1: `stats`, `plays`).
//!
//! The library only records what the caller reports. Deciding when a play
//! counts (and when it is a skip) belongs to the caller, which knows how long
//! the track was actually heard.

use rusqlite::{params, OptionalExtension};

use crate::db::Library;
use crate::error::Result;

/// Counters of one track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrackStats {
    /// Times the track was played.
    pub play_count: u32,
    /// Times the track was skipped.
    pub skip_count: u32,
    /// When it was last played, in milliseconds since the Unix epoch.
    pub last_played: Option<i64>,
    /// Rating from 0 (none) to 5.
    pub rating: u8,
}

/// One entry of the play history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Play {
    /// When playback started, in milliseconds since the Unix epoch.
    pub started_at: i64,
    /// How long was actually heard, in milliseconds.
    pub listened_ms: i64,
}

impl Library {
    /// Records a play: a history entry, one more play and the time.
    pub fn record_play(&mut self, track_id: i64, started_at: i64, listened_ms: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO plays (track_id, started_at, listened_ms) VALUES (?1, ?2, ?3)",
            params![track_id, started_at, listened_ms.max(0)],
        )?;
        tx.execute(
            "INSERT INTO stats (track_id, play_count, last_played) VALUES (?1, 1, ?2)
             ON CONFLICT(track_id) DO UPDATE SET
                 play_count = play_count + 1,
                 last_played = MAX(COALESCE(last_played, 0), excluded.last_played)",
            params![track_id, started_at],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Records a skip.
    pub fn record_skip(&self, track_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO stats (track_id, skip_count) VALUES (?1, 1)
             ON CONFLICT(track_id) DO UPDATE SET skip_count = skip_count + 1",
            [track_id],
        )?;
        Ok(())
    }

    /// Sets the rating, 0 (none) to 5; higher values are capped at 5.
    pub fn set_rating(&self, track_id: i64, rating: u8) -> Result<()> {
        self.conn.execute(
            "INSERT INTO stats (track_id, rating) VALUES (?1, ?2)
             ON CONFLICT(track_id) DO UPDATE SET rating = excluded.rating",
            params![track_id, rating.min(5)],
        )?;
        Ok(())
    }

    /// Counters of a track (all zero when it was never played or rated).
    pub fn stats(&self, track_id: i64) -> Result<TrackStats> {
        Ok(self
            .conn
            .query_row(
                "SELECT play_count, skip_count, last_played, rating FROM stats WHERE track_id = ?1",
                [track_id],
                |row| {
                    Ok(TrackStats {
                        play_count: row.get(0)?,
                        skip_count: row.get(1)?,
                        last_played: row.get(2)?,
                        rating: row.get(3)?,
                    })
                },
            )
            .optional()?
            .unwrap_or_default())
    }

    /// The most recent plays of a track, newest first.
    pub fn play_history(&self, track_id: i64, limit: usize) -> Result<Vec<Play>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT started_at, listened_ms FROM plays WHERE track_id = ?1
             ORDER BY started_at DESC LIMIT ?2",
        )?;
        let plays = statement
            .query_map(params![track_id, limit as i64], |row| {
                Ok(Play {
                    started_at: row.get(0)?,
                    listened_ms: row.get(1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(plays)
    }
}
