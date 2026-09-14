//! Renaming and moving files by a pattern (SPEC §8).
//!
//! Nothing is moved without being listed first. [`Library::rename_plan`] is
//! the dry run: it says where every file would go and which of them would
//! land on top of something. Only a plan that the listener has read gets
//! handed to [`Library::apply_rename`], and every move it makes is written
//! down so the whole run can be taken back.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::params;

use crate::db::Library;
use crate::edits::{now_ms, BatchReport, Scope, Skipped, Summary, MAX_BATCH};
use crate::error::{Error, Result};
use crate::search::TrackRow;
use crate::write;

/// Characters no file name may hold on Windows. Replaced on both systems so
/// a library keeps working when it is carried between them.
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '|', '?', '*'];
/// What they are replaced with.
const INSTEAD: char = '_';

/// Where one file would go.
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    /// The track.
    pub track_id: i64,
    /// Where the file is now.
    pub from: PathBuf,
    /// Where the pattern puts it.
    pub to: PathBuf,
    /// Why this one cannot be done, if it cannot.
    pub clash: Option<String>,
}

impl Move {
    /// Whether this move is worth doing: it goes somewhere, and it can.
    pub fn worth_doing(&self) -> bool {
        self.clash.is_none() && self.from != self.to
    }
}

/// A whole dry run.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenamePlan {
    /// Every track the plan looked at, in order.
    pub moves: Vec<Move>,
    /// Tracks left out because they sit outside the folder.
    pub out_of_scope: usize,
    /// Tracks left over because the run hit its cap.
    pub over_limit: usize,
}

impl RenamePlan {
    /// How many files would actually move.
    pub fn moving(&self) -> usize {
        self.moves.iter().filter(|one| one.worth_doing()).count()
    }

    /// How many cannot be moved, and why they are listed anyway.
    pub fn clashes(&self) -> usize {
        self.moves.iter().filter(|one| one.clash.is_some()).count()
    }

    /// The plan as the summary the listener reads before applying it.
    pub fn summary(&self) -> Summary {
        let mut summary = Summary {
            files_moved: self.moving(),
            tracks: self.moving(),
            ..Summary::default()
        };
        for _ in 0..self.out_of_scope {
            summary.count_skip(Skipped::OutsideFolder);
        }
        for _ in 0..self.over_limit {
            summary.count_skip(Skipped::OverLimit);
        }
        for one in &self.moves {
            if one.clash.is_some() {
                summary.count_skip(Skipped::NameTaken);
            } else if one.from == one.to {
                summary.count_skip(Skipped::NoChange);
            }
        }
        summary
    }
}

/// Fills a pattern in for one track.
///
/// A field the track has nothing for becomes `Unknown <field>` rather than an
/// empty stretch of path, because a folder with no name is worse than one
/// that admits it does not know.
fn fill(pattern: &str, track: &TrackRow) -> Result<String> {
    let mut out = String::with_capacity(pattern.len() + 32);
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Err(Error::Invalid("a { with no } after it".into()));
        };
        let token = &after[..close];
        rest = &after[close + 1..];

        let (name, width) = match token.split_once(':') {
            Some((name, pad)) => {
                let width = pad
                    .trim_start_matches('0')
                    .parse::<usize>()
                    .or_else(|_| pad.parse::<usize>())
                    .map_err(|_| Error::Invalid(format!("{token} is not a width")))?;
                (name, Some(width))
            }
            None => (token, None),
        };
        let value = match name {
            "title" => track
                .title
                .clone()
                .unwrap_or_else(|| "Unknown title".into()),
            "artist" => track
                .artist
                .clone()
                .unwrap_or_else(|| "Unknown artist".into()),
            "album" => track
                .album
                .clone()
                .unwrap_or_else(|| "Unknown album".into()),
            "album_artist" => track
                .album_artist
                .clone()
                .or_else(|| track.artist.clone())
                .unwrap_or_else(|| "Unknown artist".into()),
            "genre" => track
                .genre
                .clone()
                .unwrap_or_else(|| "Unknown genre".into()),
            "year" => track
                .year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "0000".into()),
            "track" => number(track.track_number, width),
            "disc" => number(track.disc_number, width),
            other => return Err(Error::Invalid(format!("{other} is not a field"))),
        };
        out.push_str(&tidy_part(&value));
    }
    out.push_str(rest);
    Ok(out)
}

fn number(value: Option<u32>, width: Option<usize>) -> String {
    let value = value.unwrap_or(0);
    match width {
        Some(width) => format!("{value:0width$}"),
        None => value.to_string(),
    }
}

/// One piece of a path, made safe to be one.
///
/// Separators inside a value would turn one folder into two, so they go as
/// well: the pattern decides the shape of the path, not the tags.
fn tidy_part(value: &str) -> String {
    let mut out: String = value
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c == '/' || c == '\\' || (c as u32) < 0x20 {
                INSTEAD
            } else {
                c
            }
        })
        .collect();
    // Windows will not keep a trailing dot or space on a name.
    while out.ends_with('.') || out.ends_with(' ') {
        out.pop();
    }
    let trimmed = out.trim_start().to_string();
    if trimmed.is_empty() {
        INSTEAD.to_string()
    } else {
        trimmed
    }
}

impl Library {
    /// Works out where a pattern would put each track, without moving
    /// anything (SPEC §8).
    ///
    /// `root` is the folder the new tree grows under. Every track keeps the
    /// extension it already has.
    pub fn rename_plan(
        &self,
        root: &Path,
        pattern: &str,
        scope: &Scope,
        tracks: &[i64],
    ) -> Result<RenamePlan> {
        if pattern.trim().is_empty() {
            return Err(Error::Invalid("a pattern is needed".into()));
        }
        let mut plan = RenamePlan::default();
        let limit = scope.limit.clamp(1, MAX_BATCH);
        // Where each destination is already spoken for, so two tracks that
        // would land on the same name are both told about it.
        let mut claimed: HashMap<String, i64> = HashMap::new();

        for track_id in tracks {
            let Some(track) = self.track(*track_id)? else {
                plan.out_of_scope += 1;
                continue;
            };
            let from = PathBuf::from(&track.path);
            if !scope.allows(&from) {
                plan.out_of_scope += 1;
                continue;
            }
            if plan.moves.len() >= limit {
                plan.over_limit += 1;
                continue;
            }

            let extension = from
                .extension()
                .map(|ext| format!(".{}", ext.to_string_lossy()))
                .unwrap_or_default();
            let relative = fill(pattern, &track)?;
            let to = root.join(format!("{relative}{extension}"));

            let key = key_of(&to);
            let clash = if let Some(other) = claimed.get(&key) {
                Some(format!("two tracks would both become this name ({other})"))
            } else if to != from && to.exists() {
                Some("a file is already there".to_string())
            } else {
                None
            };
            if clash.is_none() {
                claimed.insert(key, *track_id);
            }
            plan.moves.push(Move {
                track_id: *track_id,
                from,
                to,
                clash,
            });
        }
        Ok(plan)
    }

    /// Carries out a plan, writing down every move so it can be taken back.
    ///
    /// Only the moves that are worth doing are done; the rest are counted.
    /// A move that fails stops that file and nothing else: the ones already
    /// made stay in the run and can still be taken back together.
    pub fn apply_rename(
        &mut self,
        note: &str,
        scope: &Scope,
        plan: &RenamePlan,
    ) -> Result<BatchReport> {
        let note = note.trim();
        if note.is_empty() {
            return Err(Error::Invalid("a run needs a name".into()));
        }
        let doing: Vec<&Move> = plan.moves.iter().filter(|one| one.worth_doing()).collect();
        let mut report = BatchReport {
            unchanged: plan.moves.len() - doing.len(),
            out_of_scope: plan.out_of_scope,
            over_limit: plan.over_limit,
            ..BatchReport::default()
        };
        if doing.is_empty() {
            return Ok(report);
        }

        self.conn.execute(
            "INSERT INTO edit_batches (note, scope, created_at) VALUES (?1, ?2, ?3)",
            params![
                note,
                scope.folder.as_ref().map(|p| p.display().to_string()),
                now_ms()
            ],
        )?;
        let batch = self.conn.last_insert_rowid();
        let mut position = 0i64;

        for one in doing {
            // The folder rule is checked again here, not only in the plan:
            // a plan can be made, kept, and handed over later.
            if !scope.allows(&one.from) {
                report.out_of_scope += 1;
                continue;
            }
            match write::move_file(&one.from, &one.to) {
                Ok(()) => {
                    self.conn.execute(
                        "UPDATE tracks SET path = ?2 WHERE id = ?1",
                        params![one.track_id, one.to.to_string_lossy()],
                    )?;
                    self.conn.execute(
                        "INSERT INTO edit_steps
                           (batch_id, position, track_id, kind, field, before, after)
                         VALUES (?1, ?2, ?3, 'move', NULL, ?4, ?5)",
                        params![
                            batch,
                            position,
                            one.track_id,
                            one.from.to_string_lossy(),
                            one.to.to_string_lossy()
                        ],
                    )?;
                    position += 1;
                    report.changed += 1;
                }
                Err(error) => {
                    tracing::warn!("{:?} was not moved: {error}", one.from);
                    report
                        .failed
                        .push(format!("{}: {error}", one.from.display()));
                }
            }
        }

        if report.changed == 0 {
            self.conn
                .execute("DELETE FROM edit_batches WHERE id = ?1", [batch])?;
            return Ok(report);
        }
        report.batch = Some(batch);
        Ok(report)
    }
}

/// How two destinations are compared for being the same one.
fn key_of(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> TrackRow {
        TrackRow {
            id: 1,
            path: "/music/x.flac".into(),
            title: Some("Lampu Kota".into()),
            artist: Some("ミナ".into()),
            album: Some("City Lights".into()),
            album_artist: Some("V/A".into()),
            track_number: Some(5),
            disc_number: Some(1),
            year: Some(2015),
            genre: Some("City Pop".into()),
            duration_ms: Some(1000),
            album_id: None,
            cover_id: None,
            status: "ok".into(),
            codec: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
        }
    }

    #[test]
    fn a_pattern_fills_in_from_the_track() {
        let filled = fill("{album_artist}/{album}/{disc}-{track:02} {title}", &track()).unwrap();
        // The slash inside "V/A" is not allowed to become a folder.
        assert_eq!(filled, "V_A/City Lights/1-05 Lampu Kota");
    }

    #[test]
    fn a_field_the_track_has_nothing_for_says_so() {
        let mut bare = track();
        bare.album = None;
        bare.track_number = None;
        let filled = fill("{album}/{track:02} {title}", &bare).unwrap();
        assert_eq!(filled, "Unknown album/00 Lampu Kota");
    }

    #[test]
    fn a_pattern_that_makes_no_sense_is_refused() {
        assert!(fill("{nonsense}", &track()).is_err());
        assert!(fill("{title", &track()).is_err());
        assert!(fill("{track:xx}", &track()).is_err());
    }

    #[test]
    fn a_name_is_made_safe_for_both_systems() {
        assert_eq!(tidy_part("AC/DC"), "AC_DC");
        assert_eq!(tidy_part(r"a\b"), "a_b");
        assert_eq!(tidy_part("what? *really*"), "what_ _really_");
        assert_eq!(tidy_part("trailing dot."), "trailing dot");
        assert_eq!(tidy_part("  spaced  "), "spaced");
        assert_eq!(tidy_part(""), "_");
        assert_eq!(tidy_part("。。"), "。。", "other alphabets are left alone");
    }
}
