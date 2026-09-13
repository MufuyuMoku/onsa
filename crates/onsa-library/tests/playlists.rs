//! Playlist tests (SPEC §12): manual playlists, smart playlists compiled to
//! SQL, and M3U8 round trips.
//!
//! Fixtures are made while the test runs, in folders whose names hold
//! Japanese characters and spaces. No music file is ever committed.

use std::path::{Path, PathBuf};

use lofty::config::WriteOptions;
use lofty::prelude::{Accessor, ItemKey, TagExt};
use lofty::tag::{Tag, TagType};
use onsa_library::m3u::{read_m3u8, write_m3u8};
use onsa_library::smart::{Field, Op, Rule};
use onsa_library::{Library, Match, PathStyle, PlaylistKind, Rules, Sort, SortField, TrackSort};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "onsa プレイリスト test {} {name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A very short mono WAV, tagged.
fn song(path: &Path, title: &str, artist: &str, album: &str, genre: &str, year: &str) {
    let rate = 44_100u32;
    let frames = rate / 8;
    let data_len = frames * 2;
    let mut bytes = Vec::with_capacity(44 + data_len as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for n in 0..frames {
        let phase = n as f32 * 440.0 * std::f32::consts::TAU / rate as f32;
        bytes.extend_from_slice(&((phase.sin() * 3000.0) as i16).to_le_bytes());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();

    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title(title.into());
    tag.set_artist(artist.into());
    tag.set_album(album.into());
    tag.set_genre(genre.into());
    tag.insert_text(ItemKey::RecordingDate, year.into());
    tag.save_to_path(path, WriteOptions::default()).unwrap();
}

/// Five tracks across two folders, enough to tell rules apart.
fn library_with_songs(dir: &Path) -> (Library, PathBuf) {
    let music = dir.join("音楽 folder");
    song(
        &music.join("jazz").join("01 Blue Note.wav"),
        "Blue Note",
        "Rian",
        "Night Jazz",
        "Jazz",
        "1998",
    );
    song(
        &music.join("jazz").join("02 Red Note.wav"),
        "Red Note",
        "Rian",
        "Night Jazz",
        "Jazz",
        "2004",
    );
    song(
        &music.join("pop").join("01 夜明け.wav"),
        "夜明け",
        "ミナ",
        "City Lights",
        "City Pop",
        "2011",
    );
    song(
        &music.join("pop").join("02 Lampu Kota.wav"),
        "Lampu Kota",
        "ミナ",
        "City Lights",
        "City Pop",
        "2015",
    );
    song(
        &music.join("pop").join("03 Café Crème.wav"),
        "Café Crème",
        "Dewi",
        "City Lights",
        "City Pop",
        "2020",
    );

    let mut library = Library::open(&dir.join("library.db"), &dir.join("cache")).unwrap();
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    (library, music)
}

fn titles(tracks: &[onsa_library::TrackRow]) -> Vec<String> {
    tracks
        .iter()
        .map(|track| track.title.clone().unwrap_or_default())
        .collect()
}

fn id_of(library: &Library, title: &str) -> i64 {
    library
        .tracks_page(TrackSort::Title, false, 0, 100)
        .unwrap()
        .into_iter()
        .find(|track| track.title.as_deref() == Some(title))
        .unwrap_or_else(|| panic!("no track called {title}"))
        .id
}

// ------------------------------------------------------------ manual lists

#[test]
fn a_manual_playlist_keeps_its_own_order() {
    let dir = temp_dir("manual");
    let (mut library, _) = library_with_songs(&dir);
    let blue = id_of(&library, "Blue Note");
    let red = id_of(&library, "Red Note");
    let dawn = id_of(&library, "夜明け");

    let id = library.create_playlist("  Malam  ").unwrap();
    assert_eq!(library.playlist(id).unwrap().unwrap().name, "Malam");

    // The same track twice is allowed (SPEC §6.2).
    library
        .add_to_playlist(id, &[blue, dawn, red, blue])
        .unwrap();
    assert_eq!(
        titles(&library.playlist_tracks(id).unwrap()),
        ["Blue Note", "夜明け", "Red Note", "Blue Note"]
    );
    assert_eq!(library.playlist(id).unwrap().unwrap().track_count, 4);

    library.move_in_playlist(id, 0, 2).unwrap();
    assert_eq!(
        titles(&library.playlist_tracks(id).unwrap()),
        ["夜明け", "Red Note", "Blue Note", "Blue Note"]
    );

    library.move_in_playlist(id, 3, 0).unwrap();
    assert_eq!(
        titles(&library.playlist_tracks(id).unwrap()),
        ["Blue Note", "夜明け", "Red Note", "Blue Note"]
    );

    library.remove_from_playlist(id, 1).unwrap();
    assert_eq!(
        titles(&library.playlist_tracks(id).unwrap()),
        ["Blue Note", "Red Note", "Blue Note"]
    );

    let copy = library.duplicate_playlist(id, "Malam (salinan)").unwrap();
    assert_eq!(
        titles(&library.playlist_tracks(copy).unwrap()),
        titles(&library.playlist_tracks(id).unwrap())
    );

    library.rename_playlist(id, "Malam panjang").unwrap();
    assert_eq!(
        library.playlist(id).unwrap().unwrap().name,
        "Malam panjang",
        "renaming one must not touch the other"
    );
    assert_eq!(
        library.playlist(copy).unwrap().unwrap().name,
        "Malam (salinan)"
    );

    assert_eq!(library.playlists().unwrap().len(), 2);
    library.delete_playlist(copy).unwrap();
    assert_eq!(library.playlists().unwrap().len(), 1);
    assert!(library.create_playlist("   ").is_err(), "a name is needed");
}

#[test]
fn a_track_that_leaves_the_library_leaves_the_playlist() {
    let dir = temp_dir("manual removal");
    let (mut library, music) = library_with_songs(&dir);
    let id = library.create_playlist("Semua").unwrap();
    let all: Vec<i64> = library
        .tracks_page(TrackSort::Title, false, 0, 100)
        .unwrap()
        .iter()
        .map(|track| track.id)
        .collect();
    library.add_to_playlist(id, &all).unwrap();
    assert_eq!(library.playlist_tracks(id).unwrap().len(), 5);

    // A file that is gone is only marked missing, so the playlist keeps it
    // (SPEC §5.2); it disappears when the listener clears it out.
    std::fs::remove_file(music.join("jazz").join("01 Blue Note.wav")).unwrap();
    library.scan(|_| {}).unwrap();
    assert_eq!(library.playlist_tracks(id).unwrap().len(), 5);

    assert_eq!(library.purge_missing().unwrap(), 1, "the missing one goes");
    assert_eq!(library.playlist_tracks(id).unwrap().len(), 4);
}

// ------------------------------------------------------------- smart lists

#[test]
fn smart_rules_compile_to_parameterised_sql() {
    let rules = Rules {
        match_all: Match::All,
        rules: vec![
            Rule {
                field: Field::Artist,
                op: Op::Is,
                value: serde_json::json!("Rian"),
            },
            Rule {
                field: Field::Rating,
                op: Op::Ge,
                value: serde_json::json!(4),
            },
            Rule {
                field: Field::LastPlayed,
                op: Op::NotInLast,
                value: serde_json::json!({ "days": 30 }),
            },
        ],
        sort: Sort {
            field: SortField::Random,
            descending: false,
        },
        limit: Some(100),
    };
    let compiled = rules.compile(1_000_000_000_000).unwrap();

    // Nothing the listener typed appears in the statement itself.
    assert!(!compiled.sql.contains("Rian"), "{}", compiled.sql);
    assert_eq!(compiled.sql.matches('?').count(), compiled.params.len());
    assert!(compiled.sql.contains("ORDER BY RANDOM()"));
    assert_eq!(compiled.params.len(), 4, "three rules and the limit");

    // A value that is not a number is refused rather than guessed at.
    let broken = Rules {
        rules: vec![Rule {
            field: Field::Year,
            op: Op::Gt,
            value: serde_json::json!("dua ribu"),
        }],
        ..Rules::default()
    };
    assert!(broken.compile(0).is_err());

    // So is an operator that does not belong to the field.
    let mismatched = Rules {
        rules: vec![Rule {
            field: Field::Title,
            op: Op::Between,
            value: serde_json::json!([1, 2]),
        }],
        ..Rules::default()
    };
    assert!(mismatched.compile(0).is_err());
}

#[test]
fn a_quote_in_a_rule_is_a_value_and_never_sql() {
    let dir = temp_dir("smart injection");
    let (library, _) = library_with_songs(&dir);
    let rules = Rules {
        rules: vec![Rule {
            field: Field::Title,
            op: Op::Contains,
            // If this were ever pasted into the statement, the table would
            // be gone; as a parameter it simply matches nothing.
            value: serde_json::json!("'; DROP TABLE tracks; --"),
        }],
        ..Rules::default()
    };
    assert!(library.smart_tracks(&rules).unwrap().is_empty());
    assert_eq!(library.track_count().unwrap(), 5, "the library is intact");

    // A percent sign is a character, not a wildcard.
    let percent = Rules {
        rules: vec![Rule {
            field: Field::Title,
            op: Op::Contains,
            value: serde_json::json!("%"),
        }],
        ..Rules::default()
    };
    assert!(library.smart_tracks(&percent).unwrap().is_empty());
}

#[test]
fn smart_playlists_follow_the_rules_they_are_given() {
    let dir = temp_dir("smart rules");
    let (mut library, _) = library_with_songs(&dir);

    let jazz = Rules {
        rules: vec![Rule {
            field: Field::Genre,
            op: Op::Is,
            value: serde_json::json!("Jazz"),
        }],
        sort: Sort {
            field: SortField::Title,
            descending: false,
        },
        ..Rules::default()
    };
    assert_eq!(
        titles(&library.smart_tracks(&jazz).unwrap()),
        ["Blue Note", "Red Note"]
    );

    // Any of the rules, rather than all of them.
    let either = Rules {
        match_all: Match::Any,
        rules: vec![
            Rule {
                field: Field::Artist,
                op: Op::Is,
                value: serde_json::json!("Dewi"),
            },
            Rule {
                field: Field::Year,
                op: Op::Between,
                value: serde_json::json!([1990, 2000]),
            },
        ],
        sort: Sort {
            field: SortField::Title,
            descending: false,
        },
        ..Rules::default()
    };
    assert_eq!(
        titles(&library.smart_tracks(&either).unwrap()),
        ["Blue Note", "Café Crème"]
    );

    // Text rules that say "not" also match tracks with nothing there.
    let not_pop = Rules {
        rules: vec![Rule {
            field: Field::Genre,
            op: Op::NotContains,
            value: serde_json::json!("pop"),
        }],
        sort: Sort {
            field: SortField::Title,
            descending: false,
        },
        ..Rules::default()
    };
    assert_eq!(
        titles(&library.smart_tracks(&not_pop).unwrap()),
        ["Blue Note", "Red Note"]
    );

    // The folder a file sits in is a field of its own.
    let in_pop = Rules {
        rules: vec![Rule {
            field: Field::Folder,
            op: Op::Contains,
            value: serde_json::json!("pop"),
        }],
        ..Rules::default()
    };
    assert_eq!(library.smart_tracks(&in_pop).unwrap().len(), 3);

    // Statistics count too, and a track never played is "not in the last".
    let dawn = id_of(&library, "夜明け");
    library.set_rating(dawn, 5).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    library.record_play(dawn, now - 1000, 30_000).unwrap();

    let loved = Rules {
        rules: vec![Rule {
            field: Field::Rating,
            op: Op::Ge,
            value: serde_json::json!(4),
        }],
        ..Rules::default()
    };
    assert_eq!(titles(&library.smart_tracks(&loved).unwrap()), ["夜明け"]);

    let unheard = Rules {
        rules: vec![Rule {
            field: Field::LastPlayed,
            op: Op::NotInLast,
            value: serde_json::json!({ "days": 7 }),
        }],
        ..Rules::default()
    };
    let unheard = titles(&library.smart_tracks(&unheard).unwrap());
    assert_eq!(unheard.len(), 4);
    assert!(!unheard.contains(&"夜明け".to_string()));

    // A limit is a limit.
    let limited = Rules {
        limit: Some(2),
        ..Rules::default()
    };
    assert_eq!(library.smart_tracks(&limited).unwrap().len(), 2);
}

#[test]
fn a_smart_playlist_changes_with_the_library() {
    let dir = temp_dir("smart follows");
    let (mut library, music) = library_with_songs(&dir);
    let rules = Rules {
        rules: vec![Rule {
            field: Field::Genre,
            op: Op::Is,
            value: serde_json::json!("City Pop"),
        }],
        sort: Sort {
            field: SortField::Title,
            descending: false,
        },
        ..Rules::default()
    };
    let id = library.create_smart_playlist("Kota", &rules).unwrap();
    assert_eq!(library.playlist_tracks(id).unwrap().len(), 3);
    assert_eq!(library.playlist(id).unwrap().unwrap().track_count, 3);
    assert_eq!(
        library.playlist(id).unwrap().unwrap().kind,
        PlaylistKind::Smart
    );

    // A new track that matches joins it, without anyone touching the
    // playlist (SPEC §6.3).
    song(
        &music.join("pop").join("04 Senja.wav"),
        "Senja",
        "ミナ",
        "City Lights",
        "City Pop",
        "2024",
    );
    library.scan(|_| {}).unwrap();
    assert_eq!(
        titles(&library.playlist_tracks(id).unwrap()),
        ["Café Crème", "Lampu Kota", "Senja", "夜明け"]
    );

    // And the rules can be changed without making a new playlist.
    let jazz = Rules {
        rules: vec![Rule {
            field: Field::Genre,
            op: Op::Is,
            value: serde_json::json!("Jazz"),
        }],
        ..Rules::default()
    };
    library.set_playlist_rules(id, &jazz).unwrap();
    assert_eq!(library.playlist_tracks(id).unwrap().len(), 2);

    // Rules survive the trip through JSON exactly.
    let stored = library.playlist(id).unwrap().unwrap().rules.unwrap();
    assert_eq!(stored, jazz);
}

// -------------------------------------------------------------------- m3u8

#[test]
fn m3u8_survives_a_round_trip() {
    let dir = temp_dir("m3u8");
    let (mut library, music) = library_with_songs(&dir);
    let id = library.create_playlist("Ekspor").unwrap();
    let chosen: Vec<i64> = ["Blue Note", "夜明け", "Café Crème"]
        .iter()
        .map(|title| id_of(&library, title))
        .collect();
    library.add_to_playlist(id, &chosen).unwrap();
    let tracks = library.playlist_tracks(id).unwrap();

    // Relative paths, next to the music itself.
    let relative_file = music.join("Ekspor.m3u8");
    write_m3u8(&relative_file, &tracks, PathStyle::Relative).unwrap();
    let text = std::fs::read_to_string(&relative_file).unwrap();
    assert!(text.starts_with("#EXTM3U\n"), "{text}");
    assert!(text.contains("#EXTINF:0,Rian - Blue Note"), "{text}");
    assert!(text.contains("jazz/01 Blue Note.wav"), "{text}");
    assert!(!text.contains(&music.to_string_lossy().replace('\\', "/").to_string()));

    let back = read_m3u8(&relative_file, &library).unwrap();
    assert!(back.missing.is_empty(), "{:?}", back.missing);
    assert_eq!(back.tracks, chosen, "the same tracks, in the same order");

    // Absolute paths, from a folder of their own.
    let elsewhere = dir.join("playlists");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let absolute_file = elsewhere.join("Ekspor absolut.m3u8");
    write_m3u8(&absolute_file, &tracks, PathStyle::Absolute).unwrap();
    let absolute = read_m3u8(&absolute_file, &library).unwrap();
    assert_eq!(absolute.tracks, chosen);

    // Relative paths that climb out of their own folder are followed.
    write_m3u8(&absolute_file, &tracks, PathStyle::Relative).unwrap();
    let climbing = std::fs::read_to_string(&absolute_file).unwrap();
    assert!(climbing.contains("../"), "{climbing}");
    assert_eq!(read_m3u8(&absolute_file, &library).unwrap().tracks, chosen);

    // What the library does not know is reported, not dropped, while
    // everything around it still comes through.
    let mixed_file = music.join("Campur.m3u8");
    std::fs::write(
        &mixed_file,
        "#EXTM3U
#EXTINF:123,Someone - Missing
belum ada.wav

# a comment
jazz/01 Blue Note.wav
",
    )
    .unwrap();
    let report = read_m3u8(&mixed_file, &library).unwrap();
    assert_eq!(report.missing, vec!["belum ada.wav".to_string()]);
    assert_eq!(report.tracks, vec![chosen[0]], "the rest still arrives");
}
