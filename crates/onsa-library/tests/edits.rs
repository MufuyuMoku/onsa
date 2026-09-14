//! The safety rails around changing a lot of tracks at once (SPEC §8, §12).
//!
//! These are the tests that have to pass before anything is allowed near a
//! real library: a run stays inside the folder it was given, stops at the
//! cap it was given, and can be taken back — tags in the files included.
//!
//! Fixtures are made while the test runs. No music file is ever committed.

use std::path::{Path, PathBuf};

use lofty::config::WriteOptions;
use lofty::prelude::{Accessor, TagExt};
use lofty::tag::{Tag, TagType};
use onsa_library::edits::{Change, Scope, Skipped, DEFAULT_BATCH, MAX_BATCH};
use onsa_library::overrides::Field;
use onsa_library::{write, Library};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("onsa 編集 test {} {name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A very short mono WAV with nothing written on it.
fn song_bare(path: &Path) {
    let rate = 44_100u32;
    let frames = rate / 16;
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
}

/// The same, with a title and an artist on it.
fn song(path: &Path, title: &str, artist: &str) {
    song_bare(path);
    let mut tag = Tag::new(TagType::RiffInfo);
    tag.set_title(title.into());
    tag.set_artist(artist.into());
    tag.save_to_path(path, WriteOptions::default()).unwrap();
}

/// Two folders: one to try things in, one that must stay untouched.
fn library_with_two_folders(dir: &Path) -> (Library, PathBuf, PathBuf) {
    let trial = dir.join("salinan uji");
    let real = dir.join("音楽 asli");
    song(&trial.join("01 Satu.wav"), "Satu", "Rian");
    song(&trial.join("02 Dua.wav"), "Dua", "Rian");
    song(&trial.join("03 Tiga.wav"), "Tiga", "Mina");
    song(&real.join("01 Jangan Sentuh.wav"), "Jangan Sentuh", "Dewi");
    song(&real.join("02 Juga Jangan.wav"), "Juga Jangan", "Dewi");

    let mut library = Library::open(&dir.join("library.db"), &dir.join("cache")).unwrap();
    library.add_folder(&trial).unwrap();
    library.add_folder(&real).unwrap();
    library.scan(|_| {}).unwrap();
    (library, trial, real)
}

fn id_at(library: &Library, path: &Path) -> i64 {
    library
        .track_id(path)
        .unwrap()
        .unwrap_or_else(|| panic!("no track at {path:?}"))
}

fn title_of(library: &Library, id: i64) -> Option<String> {
    library.track(id).unwrap().unwrap().title
}

// ------------------------------------------------------------- the folder

#[test]
fn a_run_held_to_a_folder_touches_nothing_outside_it() {
    let dir = temp_dir("scope");
    let (mut library, trial, real) = library_with_two_folders(&dir);
    let inside = id_at(&library, &trial.join("01 Satu.wav"));
    let outside = id_at(&library, &real.join("01 Jangan Sentuh.wav"));

    let report = library
        .apply_edits(
            "uji",
            &Scope::folder(&trial),
            &[
                Change {
                    track_id: inside,
                    field: Field::Artist,
                    value: Some("Rian Baru".into()),
                },
                Change {
                    track_id: outside,
                    field: Field::Artist,
                    value: Some("TIDAK BOLEH".into()),
                },
            ],
        )
        .unwrap();

    assert_eq!(report.changed, 1);
    assert_eq!(report.out_of_scope, 1, "the other folder was refused");
    assert_eq!(
        library.track(inside).unwrap().unwrap().artist.as_deref(),
        Some("Rian Baru")
    );
    assert_eq!(
        library.track(outside).unwrap().unwrap().artist.as_deref(),
        Some("Dewi"),
        "the real library is exactly as it was"
    );
}

#[test]
fn the_folder_test_cannot_be_walked_around() {
    let scope = Scope::folder(Path::new("/music/salinan"));
    assert!(scope.allows(Path::new("/music/salinan/a.flac")));
    assert!(scope.allows(Path::new("/music/salinan/pop/a.flac")));

    // The folder itself is not a file inside it.
    assert!(!scope.allows(Path::new("/music/salinan")));
    // A neighbour whose name merely starts the same way.
    assert!(!scope.allows(Path::new("/music/salinan-lama/a.flac")));
    // Climbing out and back in again.
    assert!(!scope.allows(Path::new("/music/salinan/../asli/a.flac")));
    assert!(scope.allows(Path::new("/music/salinan/pop/../a.flac")));
    // Somewhere else entirely.
    assert!(!scope.allows(Path::new("/music/asli/a.flac")));

    // With no folder, everything is in scope, which is the whole point of
    // having to ask for it.
    assert!(Scope::default().allows(Path::new("/anywhere/at/all.flac")));
}

#[cfg(windows)]
#[test]
fn on_windows_the_folder_test_ignores_case() {
    let scope = Scope::folder(Path::new(r"C:\Music\Salinan"));
    assert!(scope.allows(Path::new(r"c:\music\salinan\a.flac")));
    assert!(scope.allows(Path::new(r"C:\MUSIC\SALINAN\pop\a.flac")));
}

// ---------------------------------------------------------------- the cap

#[test]
fn a_run_stops_at_the_cap_it_was_given() {
    let dir = temp_dir("cap");
    let (mut library, trial, _) = library_with_two_folders(&dir);
    let ids: Vec<i64> = ["01 Satu.wav", "02 Dua.wav", "03 Tiga.wav"]
        .iter()
        .map(|name| id_at(&library, &trial.join(name)))
        .collect();

    let changes: Vec<Change> = ids
        .iter()
        .map(|id| Change {
            track_id: *id,
            field: Field::Genre,
            value: Some("Uji".into()),
        })
        .collect();

    let report = library
        .apply_edits("uji", &Scope::folder(&trial).take(2), &changes)
        .unwrap();
    assert_eq!(report.changed, 2, "two of the three");
    assert_eq!(report.over_limit, 1);

    // A cap is never larger than the hard ceiling, and never zero.
    assert_eq!(Scope::default().take(100_000).limit, MAX_BATCH);
    assert_eq!(Scope::default().take(0).limit, 1);
    assert_eq!(Scope::default().limit, DEFAULT_BATCH);
}

// --------------------------------------------------------------- taking back

#[test]
fn a_run_can_be_taken_back_whole() {
    let dir = temp_dir("undo");
    let (mut library, trial, _) = library_with_two_folders(&dir);
    let one = id_at(&library, &trial.join("01 Satu.wav"));
    let two = id_at(&library, &trial.join("02 Dua.wav"));

    // One track already has an edit, so undo has something to put back
    // rather than merely something to remove.
    library
        .set_override(one, Field::Title, Some("Judul Lama"))
        .unwrap();

    let report = library
        .apply_edits(
            "pencocokan",
            &Scope::folder(&trial),
            &[
                Change {
                    track_id: one,
                    field: Field::Title,
                    value: Some("Judul Baru".into()),
                },
                Change {
                    track_id: two,
                    field: Field::Title,
                    value: Some("Dua Baru".into()),
                },
            ],
        )
        .unwrap();
    let batch = report.batch.expect("a run to take back");
    assert_eq!(title_of(&library, one).as_deref(), Some("Judul Baru"));
    assert_eq!(title_of(&library, two).as_deref(), Some("Dua Baru"));

    let undo = library.undo_batch(batch).unwrap();
    assert_eq!(undo.restored, 2);
    assert!(undo.failed.is_empty(), "{:?}", undo.failed);
    assert_eq!(
        title_of(&library, one).as_deref(),
        Some("Judul Lama"),
        "back to the edit that was there before"
    );
    assert_eq!(
        title_of(&library, two).as_deref(),
        Some("Dua"),
        "and back to what the file says"
    );

    // A run can only be taken back once.
    assert!(library.undo_batch(batch).is_err());
    let listed = library.batches(10).unwrap();
    assert_eq!(listed[0].id, batch);
    assert!(listed[0].undone_at.is_some());
    assert_eq!(listed[0].note, "pencocokan");
}

#[test]
fn a_run_that_changed_nothing_leaves_nothing_to_take_back() {
    let dir = temp_dir("nothing");
    let (mut library, trial, _) = library_with_two_folders(&dir);
    let one = id_at(&library, &trial.join("01 Satu.wav"));
    library
        .set_override(one, Field::Genre, Some("Jazz"))
        .unwrap();

    let report = library
        .apply_edits(
            "sama saja",
            &Scope::folder(&trial),
            &[Change {
                track_id: one,
                field: Field::Genre,
                value: Some("Jazz".into()),
            }],
        )
        .unwrap();
    assert_eq!(report.changed, 0);
    assert_eq!(report.unchanged, 1);
    assert_eq!(report.batch, None, "nothing worth remembering");
    assert!(library.batches(10).unwrap().is_empty());
}

// ------------------------------------------------------- into the files

#[test]
fn writing_to_the_files_can_be_taken_back_too() {
    let dir = temp_dir("write");
    let (mut library, trial, _) = library_with_two_folders(&dir);
    let file = trial.join("01 Satu.wav");
    let one = id_at(&library, &file);

    assert_eq!(
        write::read_field(&file, "artist").unwrap().as_deref(),
        Some("Rian"),
        "what the file says to begin with"
    );

    library
        .apply_edits(
            "pencocokan",
            &Scope::folder(&trial),
            &[
                Change {
                    track_id: one,
                    field: Field::Artist,
                    value: Some("Rian Sebenarnya".into()),
                },
                Change {
                    track_id: one,
                    field: Field::Album,
                    value: Some("Album Baru".into()),
                },
            ],
        )
        .unwrap();
    // The edit is in Onsa only; the file has not been touched.
    assert_eq!(
        write::read_field(&file, "artist").unwrap().as_deref(),
        Some("Rian"),
        "an override alone never reaches the disk"
    );

    let written = library
        .write_to_files("tulis ke file", &Scope::folder(&trial), &[one])
        .unwrap();
    assert!(written.failed.is_empty(), "{:?}", written.failed);
    assert_eq!(written.changed, 2, "both fields");
    let batch = written.batch.expect("a run to take back");

    assert_eq!(
        write::read_field(&file, "artist").unwrap().as_deref(),
        Some("Rian Sebenarnya")
    );
    assert_eq!(
        write::read_field(&file, "album").unwrap().as_deref(),
        Some("Album Baru")
    );

    let undo = library.undo_batch(batch).unwrap();
    assert!(undo.failed.is_empty(), "{:?}", undo.failed);
    assert_eq!(
        write::read_field(&file, "artist").unwrap().as_deref(),
        Some("Rian"),
        "the file says what it said before"
    );
    assert_eq!(
        write::read_field(&file, "album").unwrap(),
        None,
        "a field the file never had is gone again"
    );

    // The file still plays: the safe write left a whole file behind.
    let meta = onsa_library::tags::read(&file).expect("still readable");
    assert!(meta.duration_ms.unwrap_or(0) > 0);
}

#[test]
fn a_write_that_fails_leaves_the_original_alone() {
    let dir = temp_dir("failed write");
    let file = dir.join("rusak.wav");
    // Not audio at all: lofty will refuse it.
    std::fs::write(&file, b"this is not a sound file").unwrap();
    let before = std::fs::read(&file).unwrap();

    let outcome = write::write_fields(&file, &[("title".into(), Some("Apa pun".into()))]);
    assert!(outcome.is_err(), "a file lofty cannot read is refused");
    assert_eq!(
        std::fs::read(&file).unwrap(),
        before,
        "the original is byte for byte what it was"
    );

    // And nothing was left lying beside it.
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with(".onsa-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn a_field_written_twice_ends_up_with_the_second_value() {
    let dir = temp_dir("twice");
    let file = dir.join("satu.wav");
    song(&file, "Satu", "Rian");

    write::write_fields(&file, &[("artist".into(), Some("Kedua".into()))]).unwrap();
    assert_eq!(
        write::read_field(&file, "artist").unwrap().as_deref(),
        Some("Kedua")
    );

    write::write_fields(&file, &[("artist".into(), Some("Rian".into()))]).unwrap();
    assert_eq!(
        write::read_field(&file, "artist").unwrap().as_deref(),
        Some("Rian"),
        "writing it back is the same operation as writing it"
    );
}

#[test]
fn what_was_written_is_read_back_before_the_original_is_replaced() {
    let dir = temp_dir("verified");
    let file = dir.join("satu.wav");
    song(&file, "Satu", "Rian");

    // Numbers are the case where a write can look wrong without being
    // wrong: a tag given `05` hands back `5`, and the check knows that.
    write::write_fields(
        &file,
        &[
            ("track_number".into(), Some("05".into())),
            ("year".into(), Some("2011".into())),
        ],
    )
    .unwrap();
    assert_eq!(
        write::read_field(&file, "track_number").unwrap().as_deref(),
        Some("5")
    );
    assert_eq!(
        write::read_field(&file, "year").unwrap().as_deref(),
        Some("2011")
    );

    // Taking a field away is a change the check has to accept as well.
    write::write_fields(&file, &[("year".into(), None)]).unwrap();
    assert_eq!(write::read_field(&file, "year").unwrap(), None);
}

// ------------------------------------------------------- renaming and moving

#[test]
fn a_rename_is_listed_before_it_happens_and_can_be_taken_back() {
    let dir = temp_dir("rename");
    let (mut library, trial, real) = library_with_two_folders(&dir);
    let one = id_at(&library, &trial.join("01 Satu.wav"));
    let two = id_at(&library, &trial.join("02 Dua.wav"));
    let outside = id_at(&library, &real.join("01 Jangan Sentuh.wav"));

    let root = dir.join("tertata");
    let plan = library
        .rename_plan(
            &root,
            "{artist}/{album}/{track:02} {title}",
            &Scope::folder(&trial),
            &[one, two, outside],
        )
        .unwrap();

    assert_eq!(plan.moves.len(), 2, "the third is in another folder");
    assert_eq!(plan.out_of_scope, 1);
    assert_eq!(plan.moving(), 2);
    assert_eq!(plan.clashes(), 0);
    assert!(
        plan.moves[0].to.to_string_lossy().contains("Rian"),
        "{:?}",
        plan.moves[0].to
    );
    // Nothing has happened yet: a plan is only a plan.
    assert!(trial.join("01 Satu.wav").exists());

    let report = library
        .apply_rename("tata ulang", &Scope::folder(&trial), &plan)
        .unwrap();
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    assert_eq!(report.changed, 2);
    let batch = report.batch.expect("a run to take back");

    assert!(!trial.join("01 Satu.wav").exists(), "it moved");
    assert!(plan.moves[0].to.exists(), "and it is where the plan said");
    assert_eq!(
        library.track(one).unwrap().unwrap().path,
        plan.moves[0].to.to_string_lossy(),
        "the library followed it"
    );
    assert!(
        real.join("01 Jangan Sentuh.wav").exists(),
        "the other folder was never touched"
    );

    let undo = library.undo_batch(batch).unwrap();
    assert!(undo.failed.is_empty(), "{:?}", undo.failed);
    assert_eq!(undo.restored, 2);
    assert!(trial.join("01 Satu.wav").exists(), "the name came back");
    assert!(trial.join("02 Dua.wav").exists());
    assert!(!plan.moves[0].to.exists(), "and left where it had gone");
    assert_eq!(
        library.track(one).unwrap().unwrap().path,
        trial.join("01 Satu.wav").to_string_lossy(),
        "the library followed it back"
    );
}

#[test]
fn two_tracks_that_would_take_the_same_name_are_listed_not_moved() {
    let dir = temp_dir("clash");
    let (mut library, trial, _) = library_with_two_folders(&dir);
    let ids: Vec<i64> = ["01 Satu.wav", "02 Dua.wav", "03 Tiga.wav"]
        .iter()
        .map(|name| id_at(&library, &trial.join(name)))
        .collect();

    // A pattern that throws away everything that tells them apart.
    let root = dir.join("bentrok");
    let plan = library
        .rename_plan(&root, "{album}", &Scope::folder(&trial), &ids)
        .unwrap();
    assert_eq!(plan.moves.len(), 3);
    assert_eq!(plan.moving(), 1, "only the first can have the name");
    assert_eq!(plan.clashes(), 2);
    assert!(plan.moves[1].clash.is_some());

    let report = library
        .apply_rename("bentrok", &Scope::folder(&trial), &plan)
        .unwrap();
    assert_eq!(report.changed, 1);
    assert_eq!(report.unchanged, 2, "the two that clash were left alone");
    assert!(trial.join("02 Dua.wav").exists());
    assert!(trial.join("03 Tiga.wav").exists());
}

// ----------------------------------------------------- the summary first

#[test]
fn the_summary_says_what_would_happen_before_anything_does() {
    let dir = temp_dir("summary");
    let (mut library, trial, real) = library_with_two_folders(&dir);
    let one = id_at(&library, &trial.join("01 Satu.wav"));
    let two = id_at(&library, &trial.join("02 Dua.wav"));
    let three = id_at(&library, &trial.join("03 Tiga.wav"));
    let outside = id_at(&library, &real.join("01 Jangan Sentuh.wav"));

    library
        .set_override(two, Field::Genre, Some("Jazz"))
        .unwrap();

    let changes = vec![
        // A real change.
        Change {
            track_id: one,
            field: Field::Genre,
            value: Some("Jazz".into()),
        },
        // Already says that.
        Change {
            track_id: two,
            field: Field::Genre,
            value: Some("Jazz".into()),
        },
        // Past the cap of two tracks.
        Change {
            track_id: three,
            field: Field::Genre,
            value: Some("Jazz".into()),
        },
        // Another folder entirely.
        Change {
            track_id: outside,
            field: Field::Genre,
            value: Some("Jazz".into()),
        },
    ];
    let scope = Scope::folder(&trial).take(2);
    let summary = library.preview_edits(&scope, &changes).unwrap();

    assert_eq!(summary.fields, 1, "one field would really change");
    assert_eq!(summary.tracks, 2, "two tracks fit inside the cap");
    assert!(summary.would_do_anything());
    assert!(summary.skipped.contains(&(Skipped::NoChange, 1)));
    assert!(summary.skipped.contains(&(Skipped::OverLimit, 1)));
    assert!(summary.skipped.contains(&(Skipped::OutsideFolder, 1)));

    // And the preview really did not do any of it.
    assert_eq!(library.track(one).unwrap().unwrap().genre, None);
    assert!(library.batches(10).unwrap().is_empty());

    // What it said is what applying then does.
    let report = library.apply_edits("uji", &scope, &changes).unwrap();
    assert_eq!(report.changed, summary.fields);
    assert_eq!(report.out_of_scope, 1);
    assert_eq!(report.over_limit, 1);
    assert_eq!(report.unchanged, 1);
}

#[test]
fn the_summary_for_writing_counts_files_not_fields_alone() {
    let dir = temp_dir("summary write");
    let (mut library, trial, _) = library_with_two_folders(&dir);
    let one = id_at(&library, &trial.join("01 Satu.wav"));
    let two = id_at(&library, &trial.join("02 Dua.wav"));

    library
        .set_override(one, Field::Artist, Some("Baru"))
        .unwrap();
    library
        .set_override(one, Field::Album, Some("Baru"))
        .unwrap();

    let summary = library
        .preview_writes(&Scope::folder(&trial), &[one, two])
        .unwrap();
    assert_eq!(
        summary.files_written, 1,
        "only one file has anything waiting"
    );
    assert_eq!(summary.fields, 2, "two fields in it");
    assert!(summary.skipped.contains(&(Skipped::NoChange, 1)));
}

#[test]
fn the_rename_plan_reads_as_a_summary_too() {
    let dir = temp_dir("summary rename");
    let (library, trial, _) = library_with_two_folders(&dir);
    let ids: Vec<i64> = ["01 Satu.wav", "02 Dua.wav", "03 Tiga.wav"]
        .iter()
        .map(|name| id_at(&library, &trial.join(name)))
        .collect();
    let plan = library
        .rename_plan(
            &dir.join("tertata"),
            "{album}",
            &Scope::folder(&trial),
            &ids,
        )
        .unwrap();
    let summary = plan.summary();
    assert_eq!(summary.files_moved, 1);
    assert!(summary.skipped.contains(&(Skipped::NameTaken, 2)));
    assert!(summary.would_do_anything());
}
