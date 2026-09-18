//! Writing tags into the files themselves, and moving files about (SPEC §8).
//!
//! Both are the dangerous half of the metadata work: they touch what the
//! listener actually owns. The rules are the same for each.
//!
//! **Nothing is written in place.** A tag write copies the file beside
//! itself, changes the copy, flushes it to the disk, and only then renames
//! the copy over the original. A rename that fails leaves the original where
//! it was. Either way, a failure halfway through leaves a whole file rather
//! than a broken one.

use std::path::{Path, PathBuf};

use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::{Accessor, ItemKey};
use lofty::probe::Probe;
use lofty::tag::{ItemValue, Tag, TagItem};

use crate::error::{Error, Result};

/// The tag fields Onsa writes. The same names the `overrides` table uses.
pub const WRITABLE: &[&str] = &[
    "title",
    "artist",
    "album",
    "album_artist",
    "track_number",
    "disc_number",
    "year",
    "genre",
    "composer",
    "lyrics",
];

/// Reads one tag field from a file, as text.
///
/// This is what undo needs: what the file said before Onsa changed it.
/// A field the file does not carry comes back as `None`.
pub fn read_field(path: &Path, field: &str) -> Result<Option<String>> {
    let tagged = Probe::open(path)
        .map_err(|error| Error::Invalid(format!("{path:?} cannot be opened: {error}")))?
        .read()
        .map_err(|error| Error::Invalid(format!("{path:?} cannot be read: {error}")))?;
    let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) else {
        return Ok(None);
    };
    Ok(match field {
        "title" => tag.title().map(|value| value.to_string()),
        "artist" => tag.artist().map(|value| value.to_string()),
        "album" => tag.album().map(|value| value.to_string()),
        "album_artist" => tag.get_string(ItemKey::AlbumArtist).map(str::to_string),
        "track_number" => tag.track().map(|value| value.to_string()),
        "disc_number" => tag.disk().map(|value| value.to_string()),
        // A year can be stored as a date or on its own; both read back
        // as the year alone, which is what Onsa keeps.
        "year" => tag
            .date()
            .map(|date| date.year.to_string())
            .or_else(|| tag.get_string(ItemKey::Year).map(str::to_string)),
        "genre" => tag.genre().map(|value| value.to_string()),
        "composer" => tag.get_string(ItemKey::Composer).map(str::to_string),
        "lyrics" => read_lyrics(tag),
        _ => None,
    })
}

/// The words a tag carries, under whichever of the two names it uses.
fn read_lyrics(tag: &Tag) -> Option<String> {
    tag.get_string(ItemKey::Lyrics)
        .or_else(|| tag.get_string(ItemKey::UnsyncLyrics))
        .map(str::to_string)
}

/// Puts one field into a tag, or takes it out when the value is `None`.
fn set_field(tag: &mut Tag, field: &str, value: Option<&str>) -> Result<()> {
    let number = |value: Option<&str>| -> Result<Option<u32>> {
        match value {
            None => Ok(None),
            Some(text) => text
                .trim()
                .parse::<u32>()
                .map(Some)
                .map_err(|_| Error::Invalid(format!("{field} wants a number, not {text:?}"))),
        }
    };
    match field {
        "title" => match value {
            Some(value) => tag.set_title(value.to_string()),
            None => tag.remove_title(),
        },
        "artist" => match value {
            Some(value) => tag.set_artist(value.to_string()),
            None => tag.remove_artist(),
        },
        "album" => match value {
            Some(value) => tag.set_album(value.to_string()),
            None => tag.remove_album(),
        },
        "genre" => match value {
            Some(value) => tag.set_genre(value.to_string()),
            None => tag.remove_genre(),
        },
        "track_number" => match number(value)? {
            Some(value) => tag.set_track(value),
            None => tag.remove_track(),
        },
        "disc_number" => match number(value)? {
            Some(value) => tag.set_disk(value),
            None => tag.remove_disk(),
        },
        "year" => {
            tag.remove_key(ItemKey::Year);
            tag.remove_key(ItemKey::RecordingDate);
            if let Some(year) = number(value)? {
                tag.insert(TagItem::new(
                    ItemKey::RecordingDate,
                    ItemValue::Text(year.to_string()),
                ));
            }
        }
        "album_artist" | "composer" => {
            let key = if field == "composer" {
                ItemKey::Composer
            } else {
                ItemKey::AlbumArtist
            };
            tag.remove_key(key);
            if let Some(value) = value {
                tag.insert(TagItem::new(key, ItemValue::Text(value.to_string())));
            }
        }
        "lyrics" => {
            // Two keys, because two families of tag disagree about what the
            // field is called: ID3 keeps the words in `USLT`, which lofty
            // knows as the unsynchronised one, while a Vorbis comment and an
            // MP4 atom use the plain name. Both are cleared and whichever
            // the tag will take is written, so a file never ends up with the
            // words in one place and the old words in the other.
            tag.remove_key(ItemKey::Lyrics);
            tag.remove_key(ItemKey::UnsyncLyrics);
            if let Some(value) = value {
                let words = ItemValue::Text(value.to_string());
                if !tag.insert(TagItem::new(ItemKey::Lyrics, words.clone())) {
                    tag.insert(TagItem::new(ItemKey::UnsyncLyrics, words));
                }
            }
        }
        other => return Err(Error::Invalid(format!("{other} is not a tag Onsa writes"))),
    }
    Ok(())
}

/// Writes fields into a file, safely (SPEC §8).
///
/// The file is copied beside itself, the copy is changed and flushed, and
/// the copy is then renamed over the original. If anything fails before that
/// last step, the original has not been touched at all.
pub fn write_fields(path: &Path, fields: &[(String, Option<String>)]) -> Result<()> {
    if fields.is_empty() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| Error::Invalid(format!("{path:?} has nowhere to sit")))?;
    let temp = temp_beside(path);

    // The copy carries the original's tags; only the named fields change.
    std::fs::copy(path, &temp).map_err(|source| Error::io(&temp, source))?;
    let outcome = (|| -> Result<()> {
        let mut tagged = Probe::open(&temp)
            .map_err(|error| Error::Invalid(format!("{temp:?} cannot be opened: {error}")))?
            .read()
            .map_err(|error| Error::Invalid(format!("{temp:?} cannot be read: {error}")))?;
        if tagged.tags().is_empty() {
            let kind = tagged.primary_tag_type();
            tagged.insert_tag(Tag::new(kind));
        }
        // Every tag the file carries gets the same value. A file that holds
        // both an ID3v2 and a RIFF INFO tag would otherwise disagree with
        // itself, and which one a player believes is not Onsa's to decide.
        let kinds: Vec<_> = tagged.tags().iter().map(|tag| tag.tag_type()).collect();
        for kind in kinds {
            let Some(tag) = tagged.tag_mut(kind) else {
                continue;
            };
            for (field, value) in fields {
                set_field(tag, field, value.as_deref())?;
            }
        }
        // The whole file is saved, not one tag of it: saving a single tag
        // leaves the others as they were and the file contradicts itself.
        tagged
            .save_to_path(&temp, WriteOptions::default())
            .map_err(|error| Error::Invalid(format!("{temp:?} cannot be written: {error}")))?;
        // On the disk before it takes the original's place. The handle has
        // to be opened for writing: Windows refuses to flush a read-only one.
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&temp)
            .map_err(|source| Error::io(&temp, source))?;
        file.sync_all().map_err(|source| Error::io(&temp, source))?;
        Ok(())
    })();

    if let Err(error) = outcome {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }

    // Read the copy back before it takes the original's place.
    //
    // A tag a format cannot hold, or a tag writer that gets a format wrong,
    // would otherwise lose what was there without saying so — and that does
    // happen: saving an ID3v2 tag into a WAV twice in a row drops it, which
    // this copy-then-replace avoids but which shows the failure is real.
    // Refusing is the only honest answer, and the original is still whole.
    for (field, wanted) in fields {
        let got = read_field(&temp, field).unwrap_or(None);
        if !same_value(field, wanted.as_deref(), got.as_deref()) {
            let _ = std::fs::remove_file(&temp);
            return Err(Error::Invalid(format!(
                "{path:?} did not keep {field}: it reads back as {got:?}, not {wanted:?}.                  Nothing was changed."
            )));
        }
    }
    // The last step, and the only one that touches the original.
    std::fs::rename(&temp, path).map_err(|source| {
        let _ = std::fs::remove_file(&temp);
        Error::io(path, source)
    })?;
    let _ = parent;
    Ok(())
}

/// Whether a field reads back as what was asked for.
///
/// Numbers are compared as numbers: a tag that is given `05` hands back `5`,
/// and that is the same track number, not a failed write.
fn same_value(field: &str, wanted: Option<&str>, got: Option<&str>) -> bool {
    let number = matches!(field, "track_number" | "disc_number" | "year");
    match (wanted, got) {
        (None, None) => true,
        (Some(wanted), Some(got)) if number => {
            wanted.trim().parse::<u32>().ok() == got.trim().parse::<u32>().ok()
        }
        (Some(wanted), Some(got)) => wanted.trim() == got.trim(),
        // A value asked for and not there, or there and not asked for.
        _ => false,
    }
}

/// A name beside the file that nothing else will be using.
fn temp_beside(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or_default();
    path.with_file_name(format!(".onsa-{stamp}-{name}"))
}

/// Moves a file, making the folders it needs and refusing to land on
/// something that is already there.
pub fn move_file(from: &Path, to: &Path) -> Result<()> {
    if from == to {
        return Ok(());
    }
    if to.exists() {
        return Err(Error::Invalid(format!(
            "{to:?} is already there; nothing was moved"
        )));
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    match std::fs::rename(from, to) {
        Ok(()) => Ok(()),
        // Across two drives a rename is not possible; copy and then remove.
        Err(_) => {
            std::fs::copy(from, to).map_err(|source| Error::io(to, source))?;
            std::fs::remove_file(from).map_err(|source| Error::io(from, source))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_writable_field_is_one_the_writer_knows() {
        let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
        for field in WRITABLE {
            set_field(&mut tag, field, Some("1")).unwrap_or_else(|error| {
                panic!("{field} should be writable: {error}");
            });
            set_field(&mut tag, field, None).expect("and clearable");
        }
        assert!(set_field(&mut tag, "nonsense", Some("x")).is_err());
    }

    #[test]
    fn a_song_s_words_keep_their_lines_in_every_kind_of_tag() {
        // Lyrics are the one field with newlines in it, and the one whose
        // name the formats disagree about: ID3 will not take it under the
        // plain name at all.
        let words = "[00:01.00]Baris pertama
[00:05.00]Baris kedua";
        for kind in [
            lofty::tag::TagType::Id3v2,
            lofty::tag::TagType::VorbisComments,
            lofty::tag::TagType::Mp4Ilst,
        ] {
            let mut tag = Tag::new(kind);
            set_field(&mut tag, "lyrics", Some(words)).expect("it should write");
            assert_eq!(read_lyrics(&tag).as_deref(), Some(words), "{kind:?}");
            set_field(&mut tag, "lyrics", None).expect("it should clear");
            assert_eq!(read_lyrics(&tag), None, "{kind:?}");
        }
    }

    #[test]
    fn a_number_field_refuses_words() {
        let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
        assert!(set_field(&mut tag, "year", Some("dua ribu")).is_err());
        assert!(set_field(&mut tag, "track_number", Some("A1")).is_err());
    }

    #[test]
    fn a_number_reads_back_the_same_however_it_was_written() {
        assert!(same_value("track_number", Some("05"), Some("5")));
        assert!(same_value("year", Some("2011"), Some("2011")));
        assert!(!same_value("track_number", Some("5"), Some("6")));
        assert!(same_value("title", Some(" Satu "), Some("Satu")));
        assert!(!same_value("title", Some("Satu"), None));
        assert!(!same_value("title", None, Some("Satu")));
        assert!(same_value("title", None, None));
    }

    #[test]
    fn the_temporary_name_sits_beside_the_file_and_is_hidden() {
        let temp = temp_beside(Path::new("/music/pop/01 Lampu.wav"));
        assert_eq!(
            temp.parent(),
            Path::new("/music/pop")
                .parent()
                .map(|_| Path::new("/music/pop"))
        );
        let name = temp.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with(".onsa-"), "{name}");
        assert!(name.ends_with("01 Lampu.wav"), "{name}");
    }
}
