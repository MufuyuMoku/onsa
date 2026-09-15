//! Reading a file name for what it says (SPEC §8).
//!
//! This is the second way Onsa can work out what a track is. The first is
//! the sound itself, through a fingerprint; when that finds nothing — and on
//! a folder of downloads with no tags it often will not — what is left is
//! the name somebody saved the file under.
//!
//! A downloaded name carries more than a title. `yt-dlp` leaves the video id
//! in brackets, and uploaders add "(Official Video)" and the like to a name
//! that was never meant to be a tag. Those are taken off first, and what is
//! left is read for an artist, a title and a track number.
//!
//! Nothing here is ever sure of itself. A guess comes back with a
//! confidence well under the mark at which Onsa ticks a suggestion on its
//! own, because a file name is what somebody typed once, not what the
//! recording is. Everything this module produces is a suggestion for a
//! person to read.
//!
//! There is no regular expression in here on purpose: the patterns are read
//! character by character, which is longer to write and much easier to be
//! sure of.

/// How sure a guess is when the name gave up both an artist and a title.
pub const NAME_WITH_ARTIST: f64 = 0.6;
/// How sure a guess is when the name gave up a title alone.
pub const NAME_ALONE: f64 = 0.35;

/// Bracketed notes that say something about the upload rather than the
/// recording. Compared without case, spaces or punctuation, so
/// "Official Music Video" and "official-music-video" are the same thing.
const NOISE: &[&str] = &[
    "officialvideo",
    "officialmusicvideo",
    "officialaudio",
    "officialvideoclip",
    "officiallyricvideo",
    "officiallyricsvideo",
    "officialvisualizer",
    "officialmv",
    "official",
    "musicvideo",
    "lyricvideo",
    "lyricsvideo",
    "withlyrics",
    "lyrics",
    "lyric",
    "audio",
    "video",
    "visualizer",
    "videoclip",
    "mv",
    "pv",
    "hd",
    "hq",
    "sd",
    "4k",
    "8k",
    "1080p",
    "720p",
    "480p",
    "fullhd",
    "freedownload",
    "downloadlink",
];

/// What a file name looked like it was saying.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Guess {
    /// The artist, when the name had one.
    pub artist: Option<String>,
    /// The title, when anything was left to be one.
    pub title: Option<String>,
    /// A leading number, read as a track number.
    pub track_number: Option<u32>,
    /// How much of this is worth believing, from nothing to one.
    pub confidence: f64,
}

impl Guess {
    /// Whether the name said anything at all.
    pub fn is_empty(&self) -> bool {
        self.artist.is_none() && self.title.is_none()
    }
}

/// Takes the upload's own noise off a name.
///
/// Only whole bracketed groups are removed, and only ones that say nothing
/// about the recording: `(Official Video)` goes, `(Live)`, `(Remix)` and
/// `(feat. ...)` stay. The video id `yt-dlp` leaves behind goes too, but
/// only when it is exactly the eleven characters a YouTube id has — a
/// bracketed word of any other length is somebody's note.
pub fn strip_residue(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, inside, closer, after)) = next_group(rest) {
        kept.push_str(before);
        if !drop_group(inside, closer) {
            kept.push(opener_of(closer));
            kept.push_str(inside);
            kept.push(closer);
        } else {
            // What surrounded the group must not run together.
            kept.push(' ');
        }
        rest = after;
    }
    kept.push_str(rest);

    let mut out = tidy_spaces(&kept);
    // yt-dlp writes an uploader's channel as "Someone - Topic".
    for tail in [" - Topic", " – Topic", " - topic"] {
        if let Some(stripped) = out.strip_suffix(tail) {
            out = stripped.trim_end().to_string();
        }
    }
    trim_joiners(&out)
}

/// Reads a file's name (without its extension) for an artist and a title.
///
/// The shapes it knows are the ones that actually turn up: `Artist - Title`,
/// `07 - Artist - Title`, `07 Title`, `07. Title`, and a bare title. A name
/// written with underscores instead of spaces is read as spaces, but only
/// when it has no spaces at all — an underscore inside a spaced name is part
/// of what somebody wrote.
pub fn guess_from_name(file_stem: &str) -> Guess {
    let spaced = if file_stem.contains(' ') {
        file_stem.to_string()
    } else {
        file_stem.replace('_', " ")
    };
    let cleaned = strip_residue(&spaced);
    let (track_number, rest) = leading_number(&cleaned);
    let rest = trim_joiners(rest);
    if rest.is_empty() {
        return Guess {
            track_number,
            ..Guess::default()
        };
    }

    let parts = split_on_dash(&rest);
    let (artist, title) = match parts.len() {
        0 => (None, None),
        1 => (None, Some(parts[0].clone())),
        // More than two parts is usually "Artist - Album - Title" or a
        // title with a dash in it. The ends are the two that are worth
        // anything, and the middle is left out of the guess entirely.
        _ => (Some(parts[0].clone()), Some(parts[parts.len() - 1].clone())),
    };

    let confidence = match (&artist, &title) {
        (Some(_), Some(_)) => NAME_WITH_ARTIST,
        (_, Some(_)) => NAME_ALONE,
        _ => 0.0,
    };
    Guess {
        artist,
        title,
        track_number,
        confidence,
    }
}

/// The next bracketed group: what came before it, what is inside it, which
/// bracket closed it, and what follows.
fn next_group(text: &str) -> Option<(&str, &str, char, &str)> {
    let (start, closer) = text.char_indices().find_map(|(at, ch)| match ch {
        '(' => Some((at, ')')),
        '[' => Some((at, ']')),
        '{' => Some((at, '}')),
        '【' => Some((at, '】')),
        '（' => Some((at, '）')),
        _ => None,
    })?;
    let opener_len = text[start..].chars().next()?.len_utf8();
    let body = &text[start + opener_len..];
    let end = body.find(closer)?;
    Some((
        &text[..start],
        &body[..end],
        closer,
        &body[end + closer.len_utf8()..],
    ))
}

/// The bracket that opens what this one closes.
fn opener_of(closer: char) -> char {
    match closer {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '】' => '【',
        '）' => '（',
        other => other,
    }
}

/// Whether a bracketed group says nothing about the recording.
fn drop_group(inside: &str, closer: char) -> bool {
    let squeezed: String = inside
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    if squeezed.is_empty() {
        return true;
    }
    if NOISE.contains(&squeezed.as_str()) {
        return true;
    }
    // "Official Video 4K", "Official HD Video": anything that begins with
    // "official" is about the upload.
    if squeezed.starts_with("official") {
        return true;
    }
    // The id yt-dlp appends, which is always in square brackets and always
    // eleven of these characters. An eleven-letter English word would fit
    // the shape too, so there has to be a digit or an underscore or a dash
    // in it as well — which leaves a plain word alone at the cost of
    // leaving the occasional id behind.
    let id_shaped = |ch: char| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-';
    let not_a_word = |ch: char| ch.is_ascii_digit() || ch == '_' || ch == '-';
    closer == ']'
        && inside.chars().count() == 11
        && inside.chars().all(id_shaped)
        && inside.chars().any(not_a_word)
}

/// Collapses runs of whitespace and trims the ends.
fn tidy_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() {
            out.push(' ');
        }
        space = false;
        out.push(ch);
    }
    out
}

/// Trims the characters a name is joined with, from both ends.
fn trim_joiners(text: &str) -> String {
    text.trim_matches(|ch: char| ch.is_whitespace() || "-–—_.·|".contains(ch))
        .to_string()
}

/// A track number at the front of a name, and what follows it.
///
/// A number only counts as a track number when something follows it: a file
/// called `07` has a title of "07", not a track number and no title. Numbers
/// past 999 are years or ids, not track numbers.
fn leading_number(text: &str) -> (Option<u32>, &str) {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() || digits.len() > 3 {
        return (None, text);
    }
    let rest = &text[digits.len()..];
    // There has to be a separator, or "1979" would read as track 197.
    let separated = rest
        .chars()
        .next()
        .is_some_and(|ch| ch.is_whitespace() || ".-_)".contains(ch));
    if !separated {
        return (None, text);
    }
    let number = digits.parse::<u32>().ok().filter(|n| *n > 0);
    match number {
        Some(number) => (Some(number), rest),
        None => (None, text),
    }
}

/// Splits a name on a dash that has space around it.
///
/// The space is what makes it a separator: `Re-Entry` and `Jean-Luc` are one
/// word, and `Artist - Title` is two.
fn split_on_dash(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        let is_dash = matches!(ch, '-' | '–' | '—');
        let spaced_before = index > 0 && chars[index - 1].is_whitespace();
        let spaced_after = index + 1 < chars.len() && chars[index + 1].is_whitespace();
        if is_dash && spaced_before && spaced_after {
            parts.push(current.trim().to_string());
            current.clear();
            index += 1;
            continue;
        }
        current.push(ch);
        index += 1;
    }
    parts.push(current.trim().to_string());
    parts.retain(|part| !part.is_empty());
    parts
}

// ------------------------------------------------- tidying what a tag says

/// Words that stay in lower case inside a title, unless they begin or end it.
///
/// English only, and short on purpose: these are the words whose capitals
/// look wrong to everybody. Anything longer is somebody's choice.
const SMALL_WORDS: &[&str] = &[
    "a",
    "an",
    "and",
    "as",
    "at",
    "but",
    "by",
    "for",
    "from",
    "in",
    "into",
    "nor",
    "of",
    "on",
    "or",
    "the",
    "to",
    "vs",
    "via",
    "with",
    // A credit is written in lower case wherever it appears in a title.
    "feat",
    "feat.",
    "ft",
    "ft.",
    "featuring",
];

/// The ways people write a featuring credit, and the one Onsa settles on.
const CREDITS: &[&str] = &["feat.", "feat", "ft.", "ft", "featuring"];
/// What a featuring credit is written as once tidied.
pub const CREDIT: &str = "feat.";

/// Whether a word has no case to correct at all: a word with a digit in it,
/// or one not written in Latin letters. `MP3`, `1985`, `夜明けのうた`.
fn never_touch(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|ch| ch.is_alphanumeric()).collect();
    if letters.is_empty() {
        return true;
    }
    if letters.iter().any(char::is_ascii_digit) {
        return true;
    }
    // Anything outside the Latin alphabet has no case to correct.
    !letters.iter().all(|ch| ch.is_ascii_alphabetic())
}

/// Whether a word in capitals is standing out on purpose: `DJ`, `EP`,
/// `MINA`, `BANG`.
///
/// This only means anything when the words around it are **not** in capitals.
/// In a title that shouts every word, `THE` and `OF` are in capitals too, and
/// protecting them would leave half a title shouting.
fn stands_out(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|ch| ch.is_alphanumeric()).collect();
    !letters.is_empty() && letters.iter().all(|ch| ch.is_uppercase())
}

/// Whether a piece of text is written all in one case, and so was probably
/// not written that way on purpose.
///
/// Text that already mixes cases is left alone: somebody wrote `iPhone`,
/// `dArkSide`, or `BANG BANG` next to `Sore`, and Onsa is not the judge of
/// that.
///
/// Neither is a **single word**. `AC/DC`, `LILITH` and `deadmau5` are one
/// word each, and a single word in one case is how names are written at
/// least as often as it is a mistake. Shouting is something a phrase does,
/// so that is where a correction is offered.
pub fn one_case(text: &str) -> bool {
    if text.split_whitespace().count() < 2 {
        return false;
    }
    let letters: Vec<char> = text.chars().filter(|ch| ch.is_alphabetic()).collect();
    if letters.len() < 4 {
        return false;
    }
    if !letters.iter().any(char::is_ascii_alphabetic) {
        return false;
    }
    letters.iter().all(|ch| ch.is_uppercase()) || letters.iter().all(|ch| ch.is_lowercase())
}

/// A title with its words capitalised the way a title usually is.
///
/// Only the words that need it are touched, and a word that looks deliberate
/// is not one of them (see [`leave_alone`]). The first and last words are
/// always capitalised, whatever they are.
pub fn title_case(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let last = words.len().saturating_sub(1);
    // When the whole thing is in capitals, nothing in it is standing out.
    let shouting = text
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .all(|ch| ch.is_uppercase());
    let mut out = Vec::with_capacity(words.len());
    for (index, word) in words.iter().enumerate() {
        if never_touch(word) || (!shouting && stands_out(word)) {
            out.push((*word).to_string());
            continue;
        }
        let lower = word.to_lowercase();
        let small = SMALL_WORDS.contains(&lower.trim_matches(|ch: char| !ch.is_alphanumeric()));
        if small && index != 0 && index != last {
            out.push(lower);
        } else {
            out.push(capitalise(&lower));
        }
    }
    out.join(" ")
}

/// One word with its first letter in upper case, keeping what surrounds it.
///
/// A word inside brackets or quotes still starts at its first letter, and a
/// hyphenated name has both halves capitalised: `jean-luc` is `Jean-Luc`.
fn capitalise(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut fresh = true;
    for ch in word.chars() {
        if fresh && ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
            fresh = false;
        } else {
            out.push(ch);
            // After a hyphen or a slash the next letter starts a new word.
            if ch == '-' || ch == '/' || ch == '(' || ch == '"' || ch == '\'' {
                fresh = true;
            }
        }
    }
    out
}

/// A credit written the one way, whichever way it came.
///
/// `ft`, `FEAT`, `Featuring` all become `feat.`, and the spacing around it is
/// made even. Nothing else about the name is changed: who is credited and in
/// what order is what the record says.
pub fn tidy_credit(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    for word in words {
        let bare = word.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '.');
        if CREDITS.iter().any(|known| bare.eq_ignore_ascii_case(known)) {
            out.push(CREDIT.to_string());
        } else {
            out.push(word.to_string());
        }
    }
    out.join(" ")
}

/// The artist a track is mostly by: what comes before the credit.
///
/// `Lilith feat. ミナ` is mostly by `Lilith`. This is what an album should be
/// filed under when nothing says otherwise — not so the other name is
/// forgotten, but so one album does not become four.
pub fn main_artist(text: &str) -> String {
    let tidied = tidy_credit(text);
    match tidied.split(&format!(" {CREDIT} ")).next() {
        Some(first) if !first.trim().is_empty() => first.trim().to_string(),
        _ => text.trim().to_string(),
    }
}

/// The key two spellings of the same name share.
///
/// Case, spaces and punctuation are dropped, so `Lilith`, `LILITH` and
/// `lilith.` all answer to the same key. It is only ever used to **group**
/// spellings; what gets suggested is always one of the spellings that
/// actually appear.
pub fn same_name_key(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_video_id_yt_dlp_leaves_behind_comes_off() {
        assert_eq!(
            strip_residue("Lampu Kota [j9RGt9Z_UeE]"),
            "Lampu Kota",
            "the id goes"
        );
        assert_eq!(
            strip_residue("Lampu Kota [Remaster]"),
            "Lampu Kota [Remaster]",
            "a note of any other length stays"
        );
    }

    #[test]
    fn what_the_upload_says_about_itself_comes_off() {
        for name in [
            "Sore (Official Video)",
            "Sore (OFFICIAL MUSIC VIDEO)",
            "Sore [Official Audio]",
            "Sore (Lyrics)",
            "Sore 【MV】",
            "Sore (4K)",
            "Sore (Official Video 4K)",
        ] {
            assert_eq!(strip_residue(name), "Sore", "{name}");
        }
    }

    #[test]
    fn what_the_recording_is_stays_on() {
        for name in [
            "Sore (Live at Bandung)",
            "Sore (Remix)",
            "Sore (feat. ミナ)",
            "Sore (Acoustic Version)",
        ] {
            assert_eq!(strip_residue(name), name, "{name}");
        }
    }

    #[test]
    fn a_japanese_name_survives_the_cleaning() {
        assert_eq!(
            strip_residue("夜明けのうた 【Official Music Video】"),
            "夜明けのうた"
        );
        assert_eq!(strip_residue("夜明けのうた（MV）"), "夜明けのうた");
    }

    #[test]
    fn a_channel_name_loses_its_topic() {
        assert_eq!(strip_residue("ミナ - Topic"), "ミナ");
    }

    #[test]
    fn a_name_with_an_artist_and_a_title_gives_up_both() {
        let guess = guess_from_name("Lilith - BANG BANG");
        assert_eq!(guess.artist.as_deref(), Some("Lilith"));
        assert_eq!(guess.title.as_deref(), Some("BANG BANG"));
        assert_eq!(guess.track_number, None);
        assert_eq!(guess.confidence, NAME_WITH_ARTIST);
    }

    #[test]
    fn a_number_at_the_front_is_a_track_number() {
        let guess = guess_from_name("07 - Lilith - BANG BANG");
        assert_eq!(guess.track_number, Some(7));
        assert_eq!(guess.artist.as_deref(), Some("Lilith"));
        assert_eq!(guess.title.as_deref(), Some("BANG BANG"));

        let plain = guess_from_name("02. Lampu Kota");
        assert_eq!(plain.track_number, Some(2));
        assert_eq!(plain.artist, None);
        assert_eq!(plain.title.as_deref(), Some("Lampu Kota"));
        assert_eq!(plain.confidence, NAME_ALONE);
    }

    #[test]
    fn a_year_is_not_a_track_number() {
        let guess = guess_from_name("1979 - Smashing Pumpkins");
        assert_eq!(guess.track_number, None, "four digits is not a track");
        assert_eq!(guess.artist.as_deref(), Some("1979"));
    }

    #[test]
    fn a_dash_inside_a_word_does_not_split_it() {
        let guess = guess_from_name("Jean-Luc Ponty");
        assert_eq!(guess.artist, None);
        assert_eq!(guess.title.as_deref(), Some("Jean-Luc Ponty"));
    }

    #[test]
    fn underscores_stand_in_for_spaces_only_when_there_are_no_spaces() {
        let guess = guess_from_name("Lilith_-_BANG_BANG");
        assert_eq!(guess.artist.as_deref(), Some("Lilith"));
        assert_eq!(guess.title.as_deref(), Some("BANG BANG"));

        let mixed = guess_from_name("Lilith - BANG_BANG");
        assert_eq!(
            mixed.title.as_deref(),
            Some("BANG_BANG"),
            "somebody wrote that on purpose"
        );
    }

    #[test]
    fn a_download_is_read_the_way_it_was_saved() {
        let guess = guess_from_name("ミナ - 夜明けのうた (Official Video) [dQw4w9WgXcQ]");
        assert_eq!(guess.artist.as_deref(), Some("ミナ"));
        assert_eq!(guess.title.as_deref(), Some("夜明けのうた"));
        assert_eq!(guess.confidence, NAME_WITH_ARTIST);
    }

    #[test]
    fn a_name_with_nothing_in_it_guesses_nothing() {
        let guess = guess_from_name("   ");
        assert!(guess.is_empty());
        assert_eq!(guess.confidence, 0.0);
        let bracket = guess_from_name("(Official Video)");
        assert!(bracket.is_empty(), "nothing was left of it");
    }

    #[test]
    fn a_guess_is_never_sure_enough_to_tick_itself() {
        // The mark at which src-tauri ticks a suggestion is 0.85. Nothing a
        // file name can say reaches it, and that is the point: a name is
        // what somebody typed once.
        for name in ["Lilith - BANG BANG", "02 Lampu Kota", "夜明け"] {
            assert!(guess_from_name(name).confidence < 0.85, "{name}");
        }
    }

    // ------------------------------------------------- tidying what a tag says

    #[test]
    fn text_written_all_in_one_case_is_the_only_text_offered_a_correction() {
        assert!(one_case("BANG BANG"));
        assert!(one_case("lampu kota"));
        // Somebody wrote these this way on purpose.
        assert!(!one_case("Lampu Kota"));
        assert!(!one_case("iPhone"));
        assert!(!one_case("dArkSide"));
        // One word is a name as often as it is shouting: AC/DC, LILITH,
        // deadmau5. A phrase is what shouts.
        assert!(!one_case("AC/DC"));
        assert!(!one_case("LILITH"));
        assert!(!one_case("deadmau5"));
        // Too short to tell, or nothing to correct.
        assert!(!one_case("EP"));
        assert!(!one_case("夜明けのうた"));
        assert!(!one_case(""));
    }

    #[test]
    fn a_title_is_capitalised_the_way_a_title_is() {
        assert_eq!(
            title_case("lampu kota di malam hari"),
            "Lampu Kota Di Malam Hari"
        );
        assert_eq!(title_case("THE END OF THE LINE"), "The End of the Line");
        assert_eq!(title_case("a day in the life"), "A Day in the Life");
    }

    #[test]
    fn the_first_and_last_words_are_capitalised_whatever_they_are() {
        assert_eq!(title_case("the who"), "The Who");
        assert_eq!(title_case("all i ask of you"), "All I Ask of You");
        assert_eq!(
            title_case("what are you waiting for"),
            "What Are You Waiting For"
        );
    }

    #[test]
    fn what_looks_deliberate_is_left_exactly_as_it_is() {
        // An abbreviation, a number, and a script with no case at all.
        assert_eq!(
            title_case("DJ SHADOW feat. MINA"),
            "DJ SHADOW feat. MINA",
            "capitals among words that are not keep them"
        );
        assert_eq!(title_case("live at wembley 1985"), "Live at Wembley 1985");
        assert_eq!(
            title_case("mp3 era"),
            "mp3 Era",
            "a word with a digit is untouched"
        );
        assert_eq!(title_case("夜明けのうた"), "夜明けのうた");
        assert_eq!(title_case("ミナ feat. lilith"), "ミナ feat. Lilith");
    }

    #[test]
    fn a_hyphenated_name_is_capitalised_on_both_sides() {
        assert_eq!(title_case("jean-luc ponty"), "Jean-Luc Ponty");
        assert_eq!(title_case("(live at bandung)"), "(Live at Bandung)");
    }

    #[test]
    fn a_credit_is_written_one_way_however_it_arrived() {
        for written in [
            "Lilith ft ミナ",
            "Lilith ft. ミナ",
            "Lilith FEAT ミナ",
            "Lilith Featuring ミナ",
            "Lilith feat. ミナ",
        ] {
            assert_eq!(tidy_credit(written), "Lilith feat. ミナ", "{written}");
        }
    }

    #[test]
    fn nothing_but_the_credit_word_is_changed() {
        // Who is credited, and in what order, is what the record says.
        assert_eq!(tidy_credit("ミナ & Lilith"), "ミナ & Lilith");
        assert_eq!(tidy_credit("AC/DC"), "AC/DC");
        assert_eq!(tidy_credit(""), "");
        // A word that merely contains "ft" is not a credit.
        assert_eq!(tidy_credit("Soft Machine"), "Soft Machine");
    }

    #[test]
    fn an_album_is_filed_under_the_artist_it_is_mostly_by() {
        assert_eq!(main_artist("Lilith feat. ミナ"), "Lilith");
        assert_eq!(main_artist("Lilith ft ミナ"), "Lilith");
        assert_eq!(main_artist("Lilith"), "Lilith");
        assert_eq!(
            main_artist("ミナ & Lilith"),
            "ミナ & Lilith",
            "not a credit"
        );
        assert_eq!(main_artist("  "), "");
    }

    #[test]
    fn two_spellings_of_one_name_answer_to_the_same_key() {
        let key = same_name_key("Lilith");
        assert_eq!(same_name_key("LILITH"), key);
        assert_eq!(same_name_key("lilith."), key);
        assert_eq!(same_name_key(" Li li th "), key);
        assert_ne!(same_name_key("Lilit"), key);
        // It never invents a spelling: it is only a key.
        assert_eq!(same_name_key("ミナ"), "ミナ");
    }
}
