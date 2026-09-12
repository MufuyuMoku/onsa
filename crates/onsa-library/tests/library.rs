//! Library tests (SPEC §12): scanning a fixture folder, incremental rescans,
//! the folder watcher (add, remove, move), search, overrides and statistics.
//!
//! Fixtures are made while the test runs: short sine WAV files tagged with
//! lofty, in folders whose names hold Japanese characters and spaces.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::{Accessor, ItemKey, TagExt};
use lofty::tag::{Tag, TagType};
use onsa_library::{Change, Field, FolderWatcher, Library, TrackSort};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "onsa ライブラリ test {} {name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A mono 16-bit WAV file with a quiet sine.
fn write_wav(path: &Path, seconds: f32) {
    let rate = 44_100u32;
    let frames = (rate as f32 * seconds) as u32;
    let data_len = frames * 2;
    let mut bytes = Vec::with_capacity(44 + data_len as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for n in 0..frames {
        let phase = n as f32 * 440.0 * std::f32::consts::TAU / rate as f32;
        let sample = (phase.sin() * 3000.0) as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

struct Song<'a> {
    title: &'a str,
    artist: &'a str,
    album: &'a str,
    album_artist: Option<&'a str>,
    track: u32,
    genre: &'a str,
    picture: Option<Vec<u8>>,
}

fn tag(path: &Path, song: &Song<'_>) {
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title(song.title.into());
    tag.set_artist(song.artist.into());
    tag.set_album(song.album.into());
    tag.set_genre(song.genre.into());
    tag.set_track(song.track);
    tag.insert_text(ItemKey::RecordingDate, "2011-04-03".into());
    tag.insert_text(ItemKey::ReplayGainTrackGain, "-6.50 dB".into());
    if let Some(album_artist) = song.album_artist {
        tag.insert_text(ItemKey::AlbumArtist, album_artist.into());
    }
    if let Some(picture) = &song.picture {
        tag.push_picture(
            Picture::unchecked(picture.clone())
                .pic_type(PictureType::CoverFront)
                .mime_type(MimeType::Png)
                .build(),
        );
    }
    tag.save_to_path(path, WriteOptions::default()).unwrap();
}

fn song_file(path: &Path, song: &Song<'_>) {
    write_wav(path, 0.25);
    tag(path, song);
}

fn png(size: u32, shade: u8) -> Vec<u8> {
    let image = image::RgbImage::from_fn(size, size, |x, y| {
        image::Rgb([(x % 256) as u8, (y % 256) as u8, shade])
    });
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
}

fn open(dir: &Path) -> Library {
    Library::open(&dir.join("library.db"), &dir.join("cache")).unwrap()
}

/// The fixture: two albums, a loose track, a broken cover, a folder cover
/// and a file that is not audio at all.
fn fixture(music: &Path) {
    let album = music.join("夜明け の 街");
    for (track, title) in [(1, "夜明け"), (2, "Lampu Kota"), (3, "Café Crème")] {
        song_file(
            &album.join(format!("{track:02} {title}.wav")),
            &Song {
                title,
                artist: "ミナ",
                album: "夜明けの街",
                album_artist: Some("ミナ"),
                track,
                genre: "City Pop",
                picture: Some(png(700, 40)),
            },
        );
    }
    let second = music.join("Second Album");
    std::fs::create_dir_all(&second).unwrap();
    std::fs::write(second.join("cover.png"), png(300, 200)).unwrap();
    for (track, artist) in [(1, "Rian"), (2, "Rian feat. Dewi")] {
        song_file(
            &second.join(format!("{track} song.wav")),
            &Song {
                title: &format!("Second {track}"),
                artist,
                album: "Second",
                album_artist: None,
                track,
                genre: "Jazz",
                picture: None,
            },
        );
    }
    // A cover that cannot be decoded: the track still enters the library.
    song_file(
        &music.join("loose track.wav"),
        &Song {
            title: "Loose",
            artist: "Rian",
            album: "Singles",
            album_artist: None,
            track: 1,
            genre: "Jazz",
            picture: Some(b"this is not an image".to_vec()),
        },
    );
    std::fs::write(music.join("broken.flac"), b"not a flac file").unwrap();
    std::fs::write(music.join("notes.txt"), b"ignored").unwrap();
}

#[test]
fn scans_a_fixture_folder() {
    let dir = temp_dir("fixture");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    let mut batches = 0;
    let report = library.scan(|_| batches += 1).unwrap();

    assert_eq!(report.seen, 7, "{report:?}");
    assert_eq!(report.read, 6);
    assert_eq!(report.failed, 1);
    assert_eq!(report.missing, 0);
    assert!(batches >= 1);
    assert_eq!(library.track_count().unwrap(), 7);

    let tracks = library
        .tracks_page(TrackSort::Album, false, 0, 100)
        .unwrap();
    let first = tracks
        .iter()
        .find(|track| track.title.as_deref() == Some("夜明け"))
        .unwrap();
    assert_eq!(first.artist.as_deref(), Some("ミナ"));
    assert_eq!(first.year, Some(2011));
    assert_eq!(first.track_number, Some(1));
    assert!(first
        .duration_ms
        .is_some_and(|ms| (240..=260).contains(&ms)));
    assert!(first.cover_id.is_some(), "embedded cover");
    let failed = tracks
        .iter()
        .find(|t| t.path.ends_with("broken.flac"))
        .unwrap();
    assert_eq!(failed.status, "failed");

    // Albums: the album artist falls back to the track artist.
    let albums = library.albums_page(0, 100).unwrap();
    let names: Vec<(&str, &str, u32)> = albums
        .iter()
        .map(|a| (a.title.as_str(), a.album_artist.as_str(), a.track_count))
        .collect();
    assert!(names.contains(&("夜明けの街", "ミナ", 3)), "{names:?}");
    assert!(names.contains(&("Second", "Rian", 1)), "{names:?}");
    assert!(
        names.contains(&("Second", "Rian feat. Dewi", 1)),
        "{names:?}"
    );
    let city = albums.iter().find(|a| a.title == "夜明けの街").unwrap();
    assert_eq!(city.year, Some(2011));
    assert!(city.cover_id.is_some());
    let songs = library.album_tracks(city.id).unwrap();
    let order: Vec<_> = songs.iter().map(|s| s.track_number).collect();
    assert_eq!(order, [Some(1), Some(2), Some(3)]);

    // Covers: one per distinct image, a folder image used when nothing is
    // embedded, and a broken one simply absent.
    let second: Vec<_> = tracks
        .iter()
        .filter(|t| t.album.as_deref() == Some("Second"))
        .collect();
    assert!(second.iter().all(|t| t.cover_id.is_some()), "folder cover");
    assert_eq!(second[0].cover_id, second[1].cover_id);
    assert_ne!(second[0].cover_id, first.cover_id);
    let loose = tracks
        .iter()
        .find(|t| t.title.as_deref() == Some("Loose"))
        .unwrap();
    assert_eq!(loose.status, "ok");
    assert_eq!(loose.cover_id, None, "broken cover skipped");
    let thumbnails = std::fs::read_dir(library.covers_dir()).unwrap().count();
    assert_eq!(thumbnails, 4, "two covers, two sizes each");

    // Search: prefixes, Japanese, diacritics ignored, grouped.
    let results = library.search("lamp", 10).unwrap();
    assert_eq!(results.tracks.len(), 1);
    assert_eq!(results.albums[0].title, "夜明けの街");
    assert_eq!(results.artists[0].name, "ミナ");
    assert_eq!(library.search("夜明け", 10).unwrap().tracks.len(), 3);
    assert_eq!(library.search("cafe creme", 10).unwrap().tracks.len(), 1);
    assert_eq!(library.search("rian second", 10).unwrap().tracks.len(), 2);
    assert!(library
        .search("\"unbalanced OR", 10)
        .unwrap()
        .tracks
        .is_empty());

    let genres = library.genres_page(0, 10).unwrap();
    assert_eq!(genres.len(), 2);
    assert_eq!(genres[1].name, "Jazz");
    assert_eq!(genres[1].track_count, 3);
    assert_eq!(library.folders().unwrap()[0].track_count, 7);

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rescans_only_what_changed() {
    let dir = temp_dir("rescan");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();

    let again = library.scan(|_| {}).unwrap();
    // The failed file counts as unchanged: it is not retried until it changes.
    assert_eq!(
        (again.read, again.unchanged, again.missing),
        (0, 7, 0),
        "{again:?}"
    );
    assert_eq!(again.failed, 0);

    // Change one file, remove another, add a third.
    let changed = music.join("Second Album").join("1 song.wav");
    tag(
        &changed,
        &Song {
            title: "Second 1 (remaster with a longer title)",
            artist: "Rian",
            album: "Second",
            album_artist: None,
            track: 1,
            genre: "Jazz",
            picture: None,
        },
    );
    let id_before = library.track_id(&changed).unwrap().unwrap();
    std::fs::remove_file(music.join("loose track.wav")).unwrap();
    song_file(
        &music.join("new").join("new.wav"),
        &Song {
            title: "Baru",
            artist: "Dewi",
            album: "Baru",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    let third = library.scan(|_| {}).unwrap();
    assert_eq!(third.read, 2, "{third:?}");
    assert_eq!(third.missing, 1);
    assert_eq!(
        library.track_id(&changed).unwrap(),
        Some(id_before),
        "same identity"
    );
    assert_eq!(
        library.track(id_before).unwrap().unwrap().title.as_deref(),
        Some("Second 1 (remaster with a longer title)")
    );
    assert!(
        library.search("loose", 10).unwrap().tracks.is_empty(),
        "missing is hidden"
    );

    // A file that comes back is found again, with its old identity.
    let loose_id = library
        .track_id(&music.join("loose track.wav"))
        .unwrap()
        .unwrap();
    song_file(
        &music.join("loose track.wav"),
        &Song {
            title: "Loose",
            artist: "Rian",
            album: "Singles",
            album_artist: None,
            track: 1,
            genre: "Jazz",
            picture: None,
        },
    );
    library.scan(|_| {}).unwrap();
    let back = library.track(loose_id).unwrap().unwrap();
    assert_eq!(back.status, "ok");

    // An unreachable folder leaves its tracks missing, never deleted.
    std::fs::rename(&music, dir.join("unplugged")).unwrap();
    let gone = library.scan(|_| {}).unwrap();
    assert_eq!(gone.seen, 0);
    assert_eq!(gone.missing, 8);
    assert_eq!(library.track_count().unwrap(), 8);
    assert_eq!(library.purge_missing().unwrap(), 8);
    assert_eq!(library.track_count().unwrap(), 0);
    assert!(library.albums_page(0, 10).unwrap().is_empty());

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unchanged_rescan_is_much_faster_than_the_first_scan() {
    let dir = temp_dir("speed");
    let music = dir.join("音楽 folder");
    for album in 0..8u8 {
        let cover = png(600, album * 30);
        for track in 1..=20 {
            song_file(
                &music
                    .join(format!("Album {album}"))
                    .join(format!("{track:02}.wav")),
                &Song {
                    title: &format!("Track {track}"),
                    artist: "Artist",
                    album: &format!("Album {album}"),
                    album_artist: None,
                    track,
                    genre: "Test",
                    picture: Some(cover.clone()),
                },
            );
        }
    }
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    let first = library.scan(|_| {}).unwrap();
    let second = library.scan(|_| {}).unwrap();
    println!(
        "first scan {:?} ({} read), unchanged rescan {:?} ({} skipped)",
        first.elapsed, first.read, second.elapsed, second.unchanged
    );
    assert_eq!(first.read, 160);
    assert_eq!(second.unchanged, 160);
    assert!(
        second.elapsed * 5 < first.elapsed,
        "rescan {:?} vs first {:?}",
        second.elapsed,
        first.elapsed
    );
    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn overrides_are_shown_and_searched() {
    let dir = temp_dir("overrides");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    let id = library.search("lampu", 1).unwrap().tracks[0].id;

    library
        .set_override(id, Field::Title, Some("Lampu Kota (Live)"))
        .unwrap();
    library.set_override(id, Field::Year, Some("1999")).unwrap();
    let track = library.track(id).unwrap().unwrap();
    assert_eq!(track.title.as_deref(), Some("Lampu Kota (Live)"));
    assert_eq!(track.year, Some(1999));
    assert_eq!(library.search("live", 5).unwrap().tracks[0].id, id);

    // A rescan of a changed file keeps the override on top.
    library.set_override(id, Field::Title, None).unwrap();
    assert!(library.search("live", 5).unwrap().tracks.is_empty());
    assert_eq!(library.track(id).unwrap().unwrap().year, Some(1999));

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn statistics_and_history_follow_a_moved_file() {
    let dir = temp_dir("stats");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    let old = music.join("夜明け の 街").join("01 夜明け.wav");
    let id = library.track_id(&old).unwrap().unwrap();

    library.record_play(id, 1_000, 200_000).unwrap();
    library.record_play(id, 5_000, 180_000).unwrap();
    library.record_skip(id).unwrap();
    library.set_rating(id, 9).unwrap();
    let stats = library.stats(id).unwrap();
    assert_eq!(
        (stats.play_count, stats.skip_count, stats.rating),
        (2, 1, 5)
    );
    assert_eq!(stats.last_played, Some(5_000));
    let history = library.play_history(id, 10).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].started_at, 5_000, "newest first");

    // Rename the file, then its folder, as the watcher reports them.
    let renamed = music.join("夜明け の 街").join("01 夜明け (2024).wav");
    std::fs::rename(&old, &renamed).unwrap();
    let report = library
        .apply_changes(&[Change::Renamed {
            from: old.clone(),
            to: renamed.clone(),
        }])
        .unwrap();
    assert_eq!(report.moved, 1);
    let folder = music.join("夜明け の 街");
    let moved_folder = music.join("夜明けの街 (Deluxe)");
    std::fs::rename(&folder, &moved_folder).unwrap();
    let report = library
        .apply_changes(&[Change::Renamed {
            from: folder,
            to: moved_folder.clone(),
        }])
        .unwrap();
    assert_eq!(report.moved, 3);
    let now = moved_folder.join("01 夜明け (2024).wav");
    assert_eq!(library.track_id(&now).unwrap(), Some(id));
    assert_eq!(library.stats(id).unwrap().play_count, 2);
    assert_eq!(library.play_history(id, 10).unwrap().len(), 2);

    // Nothing else was touched and the next scan agrees.
    let rescan = library.scan(|_| {}).unwrap();
    assert_eq!((rescan.read, rescan.missing), (0, 0), "{rescan:?}");

    // Removed, then added under a name the library never saw.
    let report = library
        .apply_changes(&[Change::Removed(now.clone())])
        .unwrap();
    assert_eq!(report.missing, 1);
    assert_eq!(library.track(id).unwrap().unwrap().status, "missing");
    let fresh = moved_folder.join("04 bonus.wav");
    song_file(
        &fresh,
        &Song {
            title: "Bonus",
            artist: "ミナ",
            album: "夜明けの街",
            album_artist: Some("ミナ"),
            track: 4,
            genre: "City Pop",
            picture: None,
        },
    );
    let report = library
        .apply_changes(&[Change::Changed(fresh.clone())])
        .unwrap();
    assert_eq!(report.updated, 1);
    assert!(library.track_id(&fresh).unwrap().is_some());

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_removal_and_an_addition_of_the_same_file_is_a_move() {
    // How Windows reports a move between folders.
    let dir = temp_dir("pairing");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();

    let file = music.join("loose track.wav");
    let id = library.track_id(&file).unwrap().unwrap();
    let moved = music.join("Second Album").join("loose track.wav");
    std::fs::rename(&file, &moved).unwrap();
    let report = library
        .apply_changes(&[Change::Changed(moved.clone()), Change::Removed(file)])
        .unwrap();
    assert_eq!((report.moved, report.missing, report.updated), (1, 0, 0));
    assert_eq!(library.track_id(&moved).unwrap(), Some(id));

    let folder = music.join("夜明け の 街");
    let ids: Vec<i64> = library
        .search("夜明けの街", 10)
        .unwrap()
        .tracks
        .iter()
        .map(|t| t.id)
        .collect();
    assert_eq!(ids.len(), 3);
    let nested = music.join("Archive").join("夜明け の 街");
    std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
    std::fs::rename(&folder, &nested).unwrap();
    let report = library
        .apply_changes(&[
            Change::Removed(folder),
            Change::Changed(music.join("Archive")),
            Change::Changed(nested.clone()),
        ])
        .unwrap();
    assert_eq!(report.moved, 3, "{report:?}");
    for id in ids {
        let track = library.track(id).unwrap().unwrap();
        assert!(
            Path::new(&track.path).starts_with(&nested),
            "{}",
            track.path
        );
        assert_eq!(track.status, "ok");
    }

    // A different file under the old name is not a move.
    let other = music.join("Second Album").join("other.wav");
    song_file(
        &other,
        &Song {
            title: "Other",
            artist: "Rian",
            album: "Second",
            album_artist: None,
            track: 9,
            genre: "Jazz",
            picture: None,
        },
    );
    let report = library
        .apply_changes(&[Change::Removed(moved.clone()), Change::Changed(other)])
        .unwrap();
    assert_eq!((report.moved, report.missing, report.updated), (0, 1, 1));

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn browses_by_artist_genre_and_folder() {
    let dir = temp_dir("browse");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();

    assert_eq!(library.artist_count().unwrap(), 3);
    assert_eq!(library.genre_count().unwrap(), 2);
    let artists = library.artists_page(0, 10).unwrap();
    assert_eq!(artists[0].name, "Rian");
    assert_eq!(artists[0].track_count, 2, "the album track and the loose one");
    assert_eq!(library.artist_tracks("Rian").unwrap().len(), 2);
    // The name is matched the way it is listed, whatever the case.
    assert_eq!(library.artist_tracks("rian").unwrap().len(), 2);
    assert_eq!(library.artist_tracks("ミナ").unwrap().len(), 3);
    assert_eq!(library.genre_tracks("Jazz").unwrap().len(), 3);

    // Folders: the two album folders and the root the loose files sit in.
    assert_eq!(library.directory_count().unwrap(), 3);
    let folders = library.directories_page(0, 10).unwrap();
    let names: Vec<&str> = folders.iter().map(|folder| folder.name.as_str()).collect();
    assert!(
        names.iter().any(|name| name.ends_with("Second Album")),
        "{names:?}"
    );
    let second = music.join("Second Album");
    let tracks = library.directory_tracks(second.to_str().unwrap()).unwrap();
    assert_eq!(tracks.len(), 2);
    assert!(tracks.iter().all(|track| track.album.as_deref() == Some("Second")));
    let root = library.directory_tracks(music.to_str().unwrap()).unwrap();
    assert_eq!(root.len(), 2, "the loose track and the file that failed");

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Receives watcher batches and applies them until `done` holds.
fn apply_until(
    library: &mut Library,
    changes: &mpsc::Receiver<Vec<Change>>,
    mut done: impl FnMut(&Library) -> bool,
) -> bool {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !done(library) {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else {
            return false;
        };
        if let Ok(batch) = changes.recv_timeout(left) {
            eprintln!("watcher batch: {batch:?}");
            library.apply_changes(&batch).unwrap();
        }
    }
    true
}

fn status(library: &Library, path: &Path) -> Option<String> {
    let id = library.track_id(path).unwrap()?;
    Some(library.track(id).unwrap().unwrap().status)
}

#[test]
fn the_watcher_follows_added_moved_and_removed_files() {
    let dir = temp_dir("watch");
    let music = dir.join("音楽 folder");
    std::fs::create_dir_all(&music).unwrap();
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    let folders = library.folder_paths().unwrap();

    let (sender, changes) = mpsc::channel();
    let watcher = FolderWatcher::start(&folders, move |batch| {
        let _ = sender.send(batch);
    })
    .unwrap();
    let root = &folders[0];

    // Added: written elsewhere, then moved in whole.
    let staging = dir.join("staging.wav");
    song_file(
        &staging,
        &Song {
            title: "Datang",
            artist: "Dewi",
            album: "Watch",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    let added = root.join("サブ folder").join("datang.wav");
    std::fs::create_dir_all(added.parent().unwrap()).unwrap();
    std::fs::rename(&staging, &added).unwrap();
    assert!(
        apply_until(&mut library, &changes, |l| status(l, &added).as_deref()
            == Some("ok")),
        "the added file was not picked up"
    );
    let id = library.track_id(&added).unwrap().unwrap();
    library.record_play(id, 1, 1_000).unwrap();

    // Moved within the library: same track, statistics kept.
    let moved = root.join("pindah.wav");
    std::fs::rename(&added, &moved).unwrap();
    assert!(
        apply_until(&mut library, &changes, |l| status(l, &moved).as_deref()
            == Some("ok")),
        "the move was not picked up"
    );
    assert_eq!(library.track_id(&moved).unwrap(), Some(id), "identity kept");
    assert_eq!(library.stats(id).unwrap().play_count, 1);

    // Removed: marked missing.
    std::fs::remove_file(&moved).unwrap();
    assert!(
        apply_until(&mut library, &changes, |l| {
            status(l, &moved).as_deref() == Some("missing")
        }),
        "the removal was not picked up"
    );

    drop(watcher);
    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}
