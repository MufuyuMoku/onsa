//! Lyrics that come with the song: a `.lrc` next to it, or the words kept
//! inside its own tags (SPEC §10).
//!
//! These are the two sources that need no network and no permission, so
//! they are looked at first and every time. Nothing here is cached: reading
//! a small text file beside a song costs less than deciding whether what was
//! remembered about it is still true.

use std::path::{Path, PathBuf};

use lofty::file::TaggedFileExt;
use lofty::prelude::ItemKey;

use crate::{Error, Result};

/// Most a `.lrc` file may be before Onsa stops believing it is one.
///
/// A long song's words are a few kilobytes. Something a hundred times that
/// is a file that happens to end in `.lrc`, and reading it whole into the
/// window would help nobody.
pub const MAX_LRC: u64 = 512 * 1024;

/// Where a song's `.lrc` would be: the same folder, the same stem.
pub fn lrc_path(song: &Path) -> PathBuf {
    song.with_extension("lrc")
}

/// Reads the `.lrc` beside a song, if there is one.
///
/// Both spellings of the extension are tried, because a file written on
/// Windows and then read on Linux may be shouting.
pub fn read_beside(song: &Path) -> Option<String> {
    for extension in ["lrc", "LRC"] {
        let path = song.with_extension(extension);
        let Ok(facts) = std::fs::metadata(&path) else {
            continue;
        };
        if !facts.is_file() || facts.len() > MAX_LRC {
            continue;
        }
        match std::fs::read(&path) {
            Ok(bytes) => return Some(text_of(bytes)),
            Err(error) => {
                tracing::debug!("{path:?} could not be read: {error}");
            }
        }
    }
    None
}

/// Reads the words kept inside the song's own tags.
///
/// Which tag that is depends on the format — `USLT` in ID3, `LYRICS` in a
/// Vorbis comment, `©lyr` in MP4 — and lofty knows them under two names:
/// ID3's frame is the unsynchronised one, everything else the plain one.
pub fn read_embedded(song: &Path) -> Option<String> {
    let tagged = match lofty::read_from_path(song) {
        Ok(tagged) => tagged,
        Err(error) => {
            tracing::debug!("{song:?} could not be read for its lyrics: {error}");
            return None;
        }
    };
    tagged
        .tags()
        .iter()
        .find_map(|tag| {
            tag.get_string(ItemKey::Lyrics)
                .or_else(|| tag.get_string(ItemKey::UnsyncLyrics))
        })
        .map(str::to_string)
        .filter(|words| !words.trim().is_empty())
}

/// Writes a `.lrc` beside the song (SPEC §10).
///
/// The same care the rest of Onsa takes with a listener's folder: the words
/// go to a temporary file in that folder, are flushed to the disk, and only
/// then take the name. A failure half way through leaves whatever was there
/// before untouched.
pub fn write_beside(song: &Path, text: &str) -> Result<PathBuf> {
    use std::io::Write;

    let path = lrc_path(song);
    let folder = path.parent().ok_or_else(|| {
        Error::Io(std::io::Error::other(format!(
            "{path:?} has no folder to write in"
        )))
    })?;
    let temporary = path.with_extension("lrc.part");
    {
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(Error::Io(error));
    }
    tracing::debug!("wrote lyrics beside the song in {folder:?}");
    Ok(path)
}

/// Bytes as text, whichever of the two usual encodings they are in.
///
/// Lyrics files are old and widely shared, and plenty of them were written
/// before UTF-8 won. One that is not UTF-8 is read as Latin-1 rather than
/// refused, which is wrong for some alphabets and right for more files than
/// giving up would be.
fn text_of(bytes: Vec<u8>) -> String {
    let without_bom = match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        Some(rest) => rest.to_vec(),
        None => bytes,
    };
    match String::from_utf8(without_bom) {
        Ok(text) => text,
        Err(error) => error
            .into_bytes()
            .into_iter()
            .map(|byte| byte as char)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A short silent WAV, which is a real file lofty will tag.
    fn song_at(path: &Path) {
        let frames = 400u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + frames * 2).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&44_100u32.to_le_bytes());
        bytes.extend_from_slice(&88_200u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(frames * 2).to_le_bytes());
        bytes.extend_from_slice(&vec![0u8; frames as usize * 2]);
        std::fs::write(path, bytes).expect("the song");
    }

    /// A folder of this test's own, named after the test.
    fn folder(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("onsa-lyrics-{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a folder for the test");
        path
    }

    #[test]
    fn the_lrc_belongs_to_the_song_by_name() {
        let song = Path::new("/music/Artis/Album/01 Judul.flac");
        assert_eq!(lrc_path(song), Path::new("/music/Artis/Album/01 Judul.lrc"));
        // A name with its own dots keeps all of them but the last.
        let dotted = Path::new("/music/a.b.mp3");
        assert_eq!(lrc_path(dotted), Path::new("/music/a.b.lrc"));
    }

    #[test]
    fn a_song_with_no_lrc_beside_it_has_no_lyrics() {
        let at = folder("none");
        let song = at.join("lagu.mp3");
        std::fs::write(&song, b"not really audio").expect("the song");
        assert_eq!(read_beside(&song), None);
    }

    #[test]
    fn the_lrc_beside_the_song_is_read() {
        let at = folder("beside");
        let song = at.join("lagu.mp3");
        std::fs::write(&song, b"not really audio").expect("the song");
        std::fs::write(at.join("lagu.lrc"), "[00:01.00]satu\n").expect("the lyrics");
        assert_eq!(read_beside(&song).as_deref(), Some("[00:01.00]satu\n"));
    }

    #[test]
    fn a_file_that_is_not_utf8_is_read_rather_than_refused() {
        let at = folder("latin1");
        let song = at.join("lagu.mp3");
        std::fs::write(&song, b"not really audio").expect("the song");
        // "café" in Latin-1, which is not valid UTF-8.
        std::fs::write(at.join("lagu.lrc"), [b'c', b'a', b'f', 0xE9]).expect("the lyrics");
        assert_eq!(read_beside(&song).as_deref(), Some("café"));
    }

    #[test]
    fn something_far_too_large_is_not_a_lyrics_file() {
        let at = folder("large");
        let song = at.join("lagu.mp3");
        std::fs::write(&song, b"not really audio").expect("the song");
        std::fs::write(at.join("lagu.lrc"), vec![b'x'; MAX_LRC as usize + 1]).expect("the lyrics");
        assert_eq!(read_beside(&song), None);
    }

    #[test]
    fn the_words_inside_the_song_s_own_tags_are_read() {
        use lofty::config::WriteOptions;
        use lofty::prelude::TagExt;
        use lofty::tag::{ItemValue, Tag, TagItem, TagType};

        let at = folder("embedded");
        let song = at.join("lagu.wav");
        song_at(&song);
        assert_eq!(read_embedded(&song), None, "nothing in it yet");

        let mut tag = Tag::new(TagType::Id3v2);
        // ID3 keeps the words in `USLT`, which lofty calls the
        // unsynchronised one; the plain name it will not take at all.
        tag.insert(TagItem::new(
            ItemKey::UnsyncLyrics,
            ItemValue::Text(
                "[00:01.00]satu
[00:02.00]dua"
                    .into(),
            ),
        ));
        tag.save_to_path(&song, WriteOptions::default())
            .expect("the tag");

        assert_eq!(
            read_embedded(&song).as_deref(),
            Some(
                "[00:01.00]satu
[00:02.00]dua"
            )
        );
    }

    #[test]
    fn something_that_is_not_audio_has_no_words_rather_than_failing() {
        let at = folder("notaudio");
        let song = at.join("lagu.mp3");
        std::fs::write(&song, b"not really audio").expect("the file");
        assert_eq!(read_embedded(&song), None);
    }

    #[test]
    fn what_is_written_beside_the_song_can_be_read_back() {
        let at = folder("write");
        let song = at.join("lagu.mp3");
        std::fs::write(&song, b"not really audio").expect("the song");
        let written = write_beside(&song, "[00:02.00]dua\n").expect("it should write");
        assert_eq!(written, at.join("lagu.lrc"));
        assert_eq!(read_beside(&song).as_deref(), Some("[00:02.00]dua\n"));
        // Writing again replaces, and leaves nothing half-written behind.
        write_beside(&song, "[00:03.00]tiga\n").expect("it should write again");
        assert_eq!(read_beside(&song).as_deref(), Some("[00:03.00]tiga\n"));
        let left: Vec<String> = std::fs::read_dir(&at)
            .expect("the folder")
            .filter_map(|entry| Some(entry.ok()?.file_name().to_string_lossy().into_owned()))
            .filter(|name| name.ends_with(".part"))
            .collect();
        assert!(left.is_empty(), "nothing half-written: {left:?}");
    }
}
