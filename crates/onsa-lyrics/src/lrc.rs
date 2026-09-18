//! The LRC format: reading it, and writing it back out (SPEC §10).
//!
//! The parser is forgiving on purpose. Lyrics files are written by hand and
//! by a hundred different programs, and half of them get something slightly
//! wrong: a colon where a dot belongs, a stray `[Chorus]`, three digits of
//! fraction where there should be two, lines out of order. None of that is
//! worth refusing a song's words over, so anything unreadable is left as
//! text and anything readable is kept.

use crate::{Line, Lyrics, Word};

/// The tags an LRC file may carry that are about the file, not the song.
///
/// Anything else in square brackets that is not a time is left alone, so a
/// line that really begins "[Chorus]" stays a line of lyrics.
const TAGS: &[&str] = &[
    "ti", "ar", "al", "au", "by", "re", "ve", "id", "offset", "length", "tool", "#",
];

/// Reads lyrics from LRC text, or from plain words with no timing at all.
///
/// The file's own `[offset:]` is applied to the times here, so what comes
/// back needs no further arithmetic: a value of `+500` means the file is
/// saying its times run half a second late, and the lines move to suit.
pub fn parse(text: &str) -> Lyrics {
    let mut offset_ms = 0i64;
    // The line's sort key is its own time, or the time of the last line
    // above it that had one: an untimed line keeps the place it was
    // written in rather than falling to the top or the bottom.
    let mut collected: Vec<(i64, usize, Line)> = Vec::new();
    let mut last_time = i64::MIN;
    let mut any_time = false;

    for raw in text.lines() {
        let (times, rest) = head(raw, &mut offset_ms);
        let (line_text, words) = words_of(rest);
        if times.is_empty() {
            // A line with nothing on it and no time is spacing, and spacing
            // before the first time is the file's own header.
            if line_text.trim().is_empty() {
                continue;
            }
            let key = if last_time == i64::MIN { 0 } else { last_time };
            collected.push((
                key,
                collected.len(),
                Line {
                    at_ms: None,
                    text: line_text,
                    words,
                },
            ));
            continue;
        }
        any_time = true;
        for time in times {
            last_time = time;
            collected.push((
                time,
                collected.len(),
                Line {
                    at_ms: Some(time),
                    text: line_text.clone(),
                    words: words.clone(),
                },
            ));
        }
    }

    // Stable, so two lines given the same time keep the order they were
    // written in, and an untimed line stays under the line it followed.
    collected.sort_by_key(|(time, order, _)| (*time, *order));

    let mut lines: Vec<Line> = collected.into_iter().map(|(_, _, line)| line).collect();
    if any_time && offset_ms != 0 {
        for line in &mut lines {
            if let Some(at) = line.at_ms.as_mut() {
                *at = (*at - offset_ms).max(0);
            }
            for word in &mut line.words {
                word.at_ms = (word.at_ms - offset_ms).max(0);
            }
        }
    }
    // Trailing blank lines are the file breathing out, not part of the song.
    while lines
        .last()
        .is_some_and(|line| line.at_ms.is_none() && line.text.trim().is_empty())
    {
        lines.pop();
    }

    Lyrics { lines, offset_ms }
}

/// Reads the brackets at the front of a line: times, and tags about the file.
///
/// Returns the times found and whatever text was left.
fn head<'a>(line: &'a str, offset_ms: &mut i64) -> (Vec<i64>, &'a str) {
    let mut times = Vec::new();
    let mut rest = line.trim_start_matches('\u{feff}').trim_start();
    let mut took_any = false;
    loop {
        if !rest.starts_with('[') {
            break;
        }
        let Some(close) = rest.find(']') else { break };
        let inside = &rest[1..close];
        if let Some(time) = stamp(inside) {
            times.push(time);
            took_any = true;
            rest = &rest[close + 1..];
            continue;
        }
        let Some((key, value)) = inside.split_once(':') else {
            break;
        };
        let key = key.trim().to_ascii_lowercase();
        if !TAGS.contains(&key.as_str()) {
            break;
        }
        if key == "offset" {
            if let Ok(value) = value.trim().trim_start_matches('+').parse::<i64>() {
                *offset_ms = value;
            }
        }
        took_any = true;
        rest = &rest[close + 1..];
    }
    // A line nothing was taken off the front of is handed back whole: it
    // may be indented on purpose, and nothing here has earned the right to
    // change it.
    if took_any {
        (times, rest.trim_start())
    } else {
        (times, line)
    }
}

/// Splits a line into its text and, when it carries them, its timed words.
fn words_of(text: &str) -> (String, Vec<Word>) {
    if !text.contains('<') {
        return (text.trim_end().to_string(), Vec::new());
    }
    let mut whole = String::new();
    let mut words: Vec<Word> = Vec::new();
    let mut rest = text;
    let mut pending: Option<i64> = None;
    while let Some(open) = rest.find('<') {
        let before = &rest[..open];
        push_word(&mut whole, &mut words, pending.take(), before);
        let Some(close) = rest[open..].find('>').map(|at| open + at) else {
            // An opening bracket with no closing one is just text.
            whole.push_str(&rest[open..]);
            rest = "";
            break;
        };
        match stamp(&rest[open + 1..close]) {
            Some(time) => pending = Some(time),
            // Something else in angle brackets belongs to the words.
            None => whole.push_str(&rest[open..=close]),
        }
        rest = &rest[close + 1..];
    }
    push_word(&mut whole, &mut words, pending, rest);
    (whole.trim_end().to_string(), words)
}

/// Adds a piece of a line, as text and, when it was timed, as a word.
fn push_word(whole: &mut String, words: &mut Vec<Word>, at: Option<i64>, piece: &str) {
    if piece.is_empty() {
        return;
    }
    whole.push_str(piece);
    if let Some(at_ms) = at {
        words.push(Word {
            at_ms,
            text: piece.to_string(),
        });
    }
}

/// Reads `mm:ss.xx`, and the several other ways files write the same thing.
///
/// A fraction may be one, two or three digits, and a file may part it from
/// the seconds with a dot or another colon. Hours are not in the format and
/// are not read.
fn stamp(text: &str) -> Option<i64> {
    let text = text.trim();
    if !text.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let (minutes, rest) = text.split_once(':')?;
    let minutes: i64 = minutes.trim().parse().ok()?;
    let (seconds, fraction) = match rest.split_once(['.', ':']) {
        Some((seconds, fraction)) => (seconds, Some(fraction)),
        None => (rest, None),
    };
    let seconds: i64 = seconds.trim().parse().ok()?;
    let millis = match fraction {
        None => 0,
        Some(fraction) => {
            let digits: String = fraction
                .trim()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if digits.is_empty() {
                return None;
            }
            let value: i64 = digits.get(..3).unwrap_or(&digits).parse().ok()?;
            match digits.len() {
                1 => value * 100,
                2 => value * 10,
                _ => value,
            }
        }
    };
    Some(minutes * 60_000 + seconds * 1_000 + millis)
}

/// Writes lyrics back out as LRC text.
///
/// Used when a listener asks for a `.lrc` beside the song (SPEC §10). The
/// times written are the ones Onsa holds, so the file needs no `[offset:]`
/// of its own.
pub fn write(lyrics: &Lyrics) -> String {
    let mut out = String::new();
    for line in &lyrics.lines {
        match line.at_ms {
            Some(at) => out.push_str(&format!("[{}]{}\n", clock(at), line.text)),
            None => out.push_str(&format!("{}\n", line.text)),
        }
    }
    out
}

/// A time as LRC writes it: `mm:ss.xx`.
fn clock(at_ms: i64) -> String {
    let at_ms = at_ms.max(0);
    let minutes = at_ms / 60_000;
    let seconds = (at_ms % 60_000) / 1_000;
    let hundredths = (at_ms % 1_000) / 10;
    format!("{minutes:02}:{seconds:02}.{hundredths:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_lrc_file_is_read_in_time_order() {
        let lyrics = parse("[00:12.34]Baris pertama\n[01:05.00]Baris kedua\n");
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].at_ms, Some(12_340));
        assert_eq!(lyrics.lines[0].text, "Baris pertama");
        assert_eq!(lyrics.lines[1].at_ms, Some(65_000));
        assert!(lyrics.synced());
    }

    #[test]
    fn several_times_on_one_line_become_several_lines() {
        let lyrics = parse("[00:10.00][00:30.00][01:00.00]Reff\n");
        assert_eq!(lyrics.lines.len(), 3);
        assert_eq!(
            lyrics
                .lines
                .iter()
                .map(|line| line.at_ms)
                .collect::<Vec<_>>(),
            vec![Some(10_000), Some(30_000), Some(60_000)]
        );
        assert!(lyrics.lines.iter().all(|line| line.text == "Reff"));
    }

    #[test]
    fn lines_out_of_order_are_put_in_order() {
        let lyrics = parse("[00:30.00]nanti\n[00:10.00]dulu\n");
        assert_eq!(lyrics.lines[0].text, "dulu");
        assert_eq!(lyrics.lines[1].text, "nanti");
    }

    #[test]
    fn the_file_s_own_offset_moves_the_times() {
        // The file says its times run half a second late.
        let lyrics = parse("[offset:+500]\n[00:10.00]satu\n");
        assert_eq!(lyrics.offset_ms, 500);
        assert_eq!(lyrics.lines[0].at_ms, Some(9_500));
        // And the other way.
        let back = parse("[offset:-500]\n[00:10.00]satu\n");
        assert_eq!(back.lines[0].at_ms, Some(10_500));
    }

    #[test]
    fn tags_about_the_file_are_not_lyrics() {
        let lyrics = parse("[ti:Judul]\n[ar:Artis]\n[al:Album]\n[00:01.00]satu\n");
        assert_eq!(lyrics.lines.len(), 1);
        assert_eq!(lyrics.lines[0].text, "satu");
    }

    #[test]
    fn something_in_brackets_that_is_not_a_tag_stays_a_line() {
        let lyrics = parse("[00:01.00]satu\n[Chorus]\n[00:02.00]dua\n");
        let texts: Vec<&str> = lyrics.lines.iter().map(|line| line.text.as_str()).collect();
        assert!(texts.contains(&"[Chorus]"), "{texts:?}");
    }

    #[test]
    fn an_untimed_line_keeps_the_place_it_was_written_in() {
        let lyrics = parse("[00:01.00]satu\n(diam)\n[00:02.00]dua\n");
        let texts: Vec<&str> = lyrics.lines.iter().map(|line| line.text.as_str()).collect();
        assert_eq!(texts, vec!["satu", "(diam)", "dua"]);
        assert_eq!(lyrics.lines[1].at_ms, None);
    }

    #[test]
    fn fractions_of_one_two_or_three_digits_all_read() {
        assert_eq!(parse("[00:01.5]x").lines[0].at_ms, Some(1_500));
        assert_eq!(parse("[00:01.05]x").lines[0].at_ms, Some(1_050));
        assert_eq!(parse("[00:01.005]x").lines[0].at_ms, Some(1_005));
        // Some files part the fraction with a colon.
        assert_eq!(parse("[00:01:50]x").lines[0].at_ms, Some(1_500));
    }

    #[test]
    fn words_can_carry_their_own_times() {
        let lyrics = parse("[00:01.00]<00:01.00>Aku <00:01.50>pergi\n");
        assert_eq!(lyrics.lines[0].text, "Aku pergi");
        assert_eq!(lyrics.lines[0].words.len(), 2);
        assert_eq!(lyrics.lines[0].words[0].at_ms, 1_000);
        assert_eq!(lyrics.lines[0].words[1].at_ms, 1_500);
        assert_eq!(lyrics.lines[0].words[1].text, "pergi");
    }

    #[test]
    fn words_move_with_the_file_s_offset_too() {
        let lyrics = parse("[offset:+200]\n[00:01.00]<00:01.00>Aku\n");
        assert_eq!(lyrics.lines[0].at_ms, Some(800));
        assert_eq!(lyrics.lines[0].words[0].at_ms, 800);
    }

    #[test]
    fn plain_words_with_no_timing_are_lyrics_as_well() {
        let lyrics = parse("Baris satu\nBaris dua\n");
        assert!(!lyrics.synced());
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.plain(), "Baris satu\nBaris dua");
    }

    #[test]
    fn nothing_at_all_is_no_lyrics_rather_than_a_failure() {
        let lyrics = parse("");
        assert!(lyrics.is_empty());
        assert!(lyrics.lines.is_empty());
        let spaces = parse("\n\n   \n");
        assert!(spaces.is_empty());
    }

    #[test]
    fn a_time_that_makes_no_sense_leaves_the_line_as_text() {
        let lyrics = parse("[99:aa.bb]satu\n");
        assert_eq!(lyrics.lines[0].at_ms, None);
        assert_eq!(lyrics.lines[0].text, "[99:aa.bb]satu");
    }

    #[test]
    fn what_is_written_reads_back_the_same() {
        let first = parse("[00:12.34]Baris pertama\n[01:05.00]Baris kedua\n");
        let again = parse(&write(&first));
        assert_eq!(first.lines, again.lines);
    }

    #[test]
    fn a_line_beginning_with_a_bracket_survives_being_written() {
        let first = parse("[00:01.00][Chorus] mulai\n");
        let again = parse(&write(&first));
        assert_eq!(again.lines[0].text, "[Chorus] mulai");
        assert_eq!(again.lines[0].at_ms, Some(1_000));
    }
}
