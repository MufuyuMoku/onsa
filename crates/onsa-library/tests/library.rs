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
    assert_eq!(
        artists[0].track_count, 2,
        "the album track and the loose one"
    );
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
    assert!(tracks
        .iter()
        .all(|track| track.album.as_deref() == Some("Second")));
    let root = library.directory_tracks(music.to_str().unwrap()).unwrap();
    assert_eq!(root.len(), 2, "the loose track and the file that failed");

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Receives watcher batches and applies them until `done` holds.
///
/// What is waited for is the whole of the expected state, not one part of
/// it: a watcher may split what happened across two batches, and a test
/// that checks the rest the moment the first part lands is a test that
/// passes or fails by how busy the machine is.
fn apply_until(
    library: &mut Library,
    changes: &mpsc::Receiver<Vec<Change>>,
    mut done: impl FnMut(&Library) -> bool,
) -> bool {
    // Long enough for a loaded machine, and it is only ever waited out when
    // something is actually wrong.
    let deadline = Instant::now() + Duration::from_secs(30);
    while !done(library) {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else {
            return false;
        };
        match changes.recv_timeout(left) {
            Ok(batch) => {
                eprintln!("watcher batch: {batch:?}");
                library.apply_changes(&batch).unwrap();
            }
            Err(_) => return done(library),
        }
    }
    true
}

/// Waits until the watcher has shown it is awake, and says so.
///
/// Registering a folder and getting events out of it are not the same
/// moment: on Windows the first change after `watch()` can fall into the
/// gap while the asynchronous read is still being queued, and on a loaded
/// machine that gap is wider. A test that starts its real work inside that
/// gap waits for an event that was never going to come — which is how this
/// one failed, once, on a busy machine.
///
/// So a probe file is touched until a batch mentioning it arrives. After
/// that the watcher is known to be delivering, and everything the test does
/// next can be waited for on its own terms.
fn watcher_is_awake(root: &Path, changes: &mpsc::Receiver<Vec<Change>>) -> bool {
    let probe = root.join(".onsa-watch-probe");
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        let _ = std::fs::write(&probe, b"awake?");
        let waited = Duration::from_millis(1500);
        let until = Instant::now() + waited;
        while let Some(left) = until.checked_duration_since(Instant::now()) {
            let Ok(batch) = changes.recv_timeout(left) else {
                break;
            };
            if batch.iter().any(|change| match change {
                Change::Changed(path) | Change::Removed(path) => {
                    path.ends_with(".onsa-watch-probe")
                }
                Change::Renamed { from, to } => {
                    from.ends_with(".onsa-watch-probe") || to.ends_with(".onsa-watch-probe")
                }
            }) {
                let _ = std::fs::remove_file(&probe);
                // Whatever the removal stirs up is not the test's business.
                while changes.recv_timeout(Duration::from_millis(1500)).is_ok() {}
                return true;
            }
        }
    }
    false
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
    assert!(
        watcher_is_awake(root, &changes),
        "the watcher never reported anything at all, so there is nothing to test"
    );

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

    // Moved within the library: same track, statistics kept. Both halves
    // of that are waited for together, because a move can arrive as a
    // removal in one batch and an addition in the next.
    let moved = root.join("pindah.wav");
    std::fs::rename(&added, &moved).unwrap();
    assert!(
        apply_until(&mut library, &changes, |l| {
            status(l, &moved).as_deref() == Some("ok") && l.track_id(&moved).unwrap() == Some(id)
        }),
        "the move was not picked up as the same track"
    );
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

/// A move the watcher reports in two goes is still a move.
///
/// This is the same thing `the_watcher_follows_added_moved_and_removed_files`
/// checks, with the watcher taken out of it: the removal is applied, and
/// then the addition, as two separate batches. No timing decides the
/// outcome, so nothing about the machine can make it pass one day and fail
/// the next — which is exactly how that test once failed.
#[test]
fn a_move_split_across_two_batches_is_still_a_move() {
    let dir = temp_dir("split move");
    let music = dir.join("音楽 folder");
    std::fs::create_dir_all(&music).unwrap();
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();

    let from = music.join("satu.wav");
    song_file(
        &from,
        &Song {
            title: "Pindah",
            artist: "Dewi",
            album: "Watch",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    library.scan(|_| {}).unwrap();
    let id = library.track_id(&from).unwrap().unwrap();
    library.record_play(id, 1, 1_000).unwrap();

    let to = music.join("サブ folder").join("dua.wav");
    std::fs::create_dir_all(to.parent().unwrap()).unwrap();
    std::fs::rename(&from, &to).unwrap();

    // One batch says it went, and only later does another say it arrived.
    library
        .apply_changes(&[Change::Removed(from.clone())])
        .unwrap();
    assert_eq!(
        status(&library, &from).as_deref(),
        Some("missing"),
        "the removal stands on its own"
    );
    library
        .apply_changes(&[Change::Changed(to.clone())])
        .unwrap();

    assert_eq!(
        library.track_id(&to).unwrap(),
        Some(id),
        "the file that turned up is the one that left"
    );
    assert_eq!(status(&library, &to).as_deref(), Some("ok"));
    assert_eq!(library.stats(id).unwrap().play_count, 1, "its history too");
    assert_eq!(
        library.track_count().unwrap(),
        1,
        "and it is not there twice"
    );

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A watcher is allowed to report only half of a move. Windows coalesces
/// the two sides of a rename into one notification often enough that a
/// loaded machine can see the arrival and never the departure — which is
/// exactly how the watcher test failed once in fifty runs under load. The
/// database being told nothing about the old path must not cost the
/// listener their play count.
#[test]
fn an_arrival_with_no_departure_is_still_a_move() {
    let dir = temp_dir("arrival only");
    let music = dir.join("音楽 folder");
    std::fs::create_dir_all(&music).unwrap();
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();

    let from = music.join("satu.wav");
    song_file(
        &from,
        &Song {
            title: "Pindah",
            artist: "Dewi",
            album: "Watch",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    library.scan(|_| {}).unwrap();
    let id = library.track_id(&from).unwrap().unwrap();
    library.record_play(id, 1, 1_000).unwrap();

    let to = music.join("サブ folder").join("dua.wav");
    std::fs::create_dir_all(to.parent().unwrap()).unwrap();
    std::fs::rename(&from, &to).unwrap();

    // Nothing at all is said about the old path: the row still claims the
    // file is there and well.
    assert_eq!(status(&library, &from).as_deref(), Some("ok"));
    library
        .apply_changes(&[Change::Changed(to.clone())])
        .unwrap();

    assert_eq!(
        library.track_id(&to).unwrap(),
        Some(id),
        "the file that turned up is the one that left"
    );
    assert_eq!(status(&library, &to).as_deref(), Some("ok"));
    assert_eq!(library.stats(id).unwrap().play_count, 1, "its history too");
    assert_eq!(
        library.track_count().unwrap(),
        1,
        "and it is not there twice"
    );

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A file deleted from inside Onsa is in the Recycle Bin or the Trash, and
/// comes back from there with the song it belongs to (SPEC §15).
///
/// The test does not take Onsa's word for any of it: it looks into the
/// system's own trash through the same protocol the Recycle Bin window uses,
/// finds the file there, and puts it back the way the Restore menu does.
#[test]
fn a_deleted_file_waits_in_the_recycle_bin_and_comes_back() {
    let dir = temp_dir("deleted");
    let music = dir.join("音楽 folder");
    std::fs::create_dir_all(&music).unwrap();
    let song = music.join("dibuang.wav");
    song_file(
        &song,
        &Song {
            title: "Dibuang",
            artist: "Dewi",
            album: "Tempat sampah",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    let id = library.track_id(&song).unwrap().unwrap();
    library.record_play(id, 1, 1_000).unwrap();

    let report = library.delete_files(&[id]).unwrap();
    if let Some((path, why)) = report.kept.first() {
        // A machine whose temp folder has no trash cannot prove this, and
        // saying so is better than pretending otherwise. What matters is
        // that the file is still there: nothing was deleted for good.
        assert!(song.is_file(), "{} was deleted anyway", path.display());
        // The Linux runner on CI has a freedesktop trash, so there a step
        // aside is not a note but a failure: a green run has to mean the
        // trash really was proven, not that the test found a reason not to
        // look. The Windows runner is the other case, and an interesting
        // one — its service account has no Recycle Bin for that folder, so
        // this is a real place where Windows would have deleted the file
        // outright and Onsa refused to. That refusal is what is checked.
        if std::env::var_os("CI").is_some() {
            assert!(
                cfg!(windows),
                "this machine cannot prove the trash here: {why}"
            );
            assert_eq!(
                why,
                &onsa_library::Refusal::NoTrashHere,
                "a place with no Recycle Bin says so plainly"
            );
        }
        eprintln!("this machine cannot prove the trash here: {why}");
        drop(library);
        let _ = std::fs::remove_dir_all(&dir);
        return;
    }
    assert_eq!(report.count(), 1);
    assert!(!song.exists(), "the file left its folder");
    assert_eq!(
        status(&library, &song).as_deref(),
        Some("missing"),
        "the song is still a song, it is just not there"
    );

    // Now look where the operating system says it put it.
    let mine: Vec<trash::TrashItem> = trash::os_limited::list()
        .unwrap()
        .into_iter()
        .filter(|item| item.original_path() == song)
        .collect();
    assert_eq!(mine.len(), 1, "exactly one of them is in the trash");

    // And put it back, which is what the system's own Restore does.
    trash::os_limited::restore_all(mine).unwrap();
    assert!(song.is_file(), "the file is back where it was");

    library.scan(|_| {}).unwrap();
    assert_eq!(
        library.track_id(&song).unwrap(),
        Some(id),
        "and it is the same song, not a new one"
    );
    assert_eq!(status(&library, &song).as_deref(), Some("ok"));
    assert_eq!(library.stats(id).unwrap().play_count, 1, "with its history");

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Onsa never deletes for good, so a place with no trash is refused rather
/// than deleted from (SPEC §15).
#[test]
fn a_place_with_no_trash_is_refused_and_says_so() {
    // There is no way to make a folder on this machine lose its Recycle Bin,
    // so what is checked here is the promise itself: the question is asked
    // before anything is deleted, and its answer decides.
    let dir = temp_dir("trash question");
    match onsa_library::delete::trash_is_real(&dir) {
        Ok(()) => {
            assert!(
                !dir.join(".onsa-trash-check").exists(),
                "the probe left nothing behind"
            );
        }
        Err(why) => {
            eprintln!("this machine has no trash for {}: {why}", dir.display());
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// What is not there is not deleted, and the report says which is which.
#[test]
fn a_file_that_is_already_gone_is_reported_not_deleted() {
    let dir = temp_dir("already gone");
    let music = dir.join("音楽 folder");
    std::fs::create_dir_all(&music).unwrap();
    let song = music.join("hilang.wav");
    song_file(
        &song,
        &Song {
            title: "Hilang",
            artist: "Dewi",
            album: "Tempat sampah",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    let id = library.track_id(&song).unwrap().unwrap();

    // Somebody else got there first.
    std::fs::remove_file(&song).unwrap();
    let report = library.delete_files(&[id]).unwrap();
    assert_eq!(report.count(), 0);
    assert!(!report.all_done());
    assert_eq!(report.kept, vec![(song, onsa_library::Refusal::NotThere)]);

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Gives a file the modification time of another, without leaving std.
fn same_time(path: &Path, when: std::time::SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(when)
        .unwrap();
}

/// Two files that are alike in every measurable way are not guessed between.
#[test]
fn two_files_that_match_each_other_are_left_where_they_are() {
    let dir = temp_dir("twins");
    let music = dir.join("音楽 folder");
    std::fs::create_dir_all(&music).unwrap();
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();

    let song = Song {
        title: "Kembar",
        artist: "Dewi",
        album: "Watch",
        album_artist: None,
        track: 1,
        genre: "Pop",
        picture: None,
    };
    let one = music.join("satu.wav");
    let two = music.join("dua.wav");
    song_file(&one, &song);
    std::fs::copy(&one, &two).unwrap();
    // The same bytes and the same time: nothing tells them apart.
    let when = std::fs::metadata(&one).unwrap().modified().unwrap();
    same_time(&two, when);
    library.scan(|_| {}).unwrap();

    std::fs::remove_file(&one).unwrap();
    std::fs::remove_file(&two).unwrap();
    library
        .apply_changes(&[Change::Removed(one.clone()), Change::Removed(two.clone())])
        .unwrap();

    let three = music.join("tiga.wav");
    song_file(&three, &song);
    same_time(&three, when);
    library
        .apply_changes(&[Change::Changed(three.clone())])
        .unwrap();

    // Neither of the two is claimed: a new row is the honest answer.
    assert_eq!(status(&library, &one).as_deref(), Some("missing"));
    assert_eq!(status(&library, &two).as_deref(), Some("missing"));
    assert_eq!(status(&library, &three).as_deref(), Some("ok"));

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Scrolling to the track that is playing means asking where it is, and the
/// answer has to agree with the page the list would draw at that row —
/// otherwise the list scrolls to the wrong song, which is worse than not
/// scrolling at all.
#[test]
fn where_a_track_sits_agrees_with_the_page_at_that_row() {
    let dir = temp_dir("place");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();

    for sort in [TrackSort::Title, TrackSort::Artist, TrackSort::Duration] {
        for descending in [false, true] {
            let all = library.tracks_page(sort, descending, 0, 100).unwrap();
            assert!(all.len() > 2, "something to look through");
            for (row, track) in all.iter().enumerate() {
                let place = library
                    .track_place(sort, descending, track.id)
                    .unwrap()
                    .expect("a track in the library is somewhere in the list");
                assert_eq!(
                    place as usize, row,
                    "{sort:?} descending={descending}: {:?}",
                    track.title
                );
                // And the page starting there begins with that track, which
                // is what the interface actually asks for when it scrolls.
                let page = library.tracks_page(sort, descending, row, 1).unwrap();
                assert_eq!(page.first().map(|one| one.id), Some(track.id));
            }
        }
    }

    // A track the library does not have is nowhere, not row zero.
    assert_eq!(
        library
            .track_place(TrackSort::Title, false, 999_999)
            .unwrap(),
        None
    );
    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------- folders (M13)

/// Letting a folder go takes its songs out of the library and leaves every
/// file exactly where it is.
#[test]
fn a_folder_that_is_let_go_leaves_its_files_alone() {
    let dir = temp_dir("let go");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    assert!(library.track_count().unwrap() > 0);
    let files: Vec<PathBuf> = std::fs::read_dir(&music)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();

    assert!(library.remove_folder(&music).unwrap());

    assert_eq!(library.track_count().unwrap(), 0, "the songs are gone");
    assert!(library.folder_paths().unwrap().is_empty());
    for file in files {
        assert!(file.exists(), "{} was touched", file.display());
    }
    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A folder that moved is the same folder: the songs in it keep their
/// history, their playlists and the edits not yet written to them.
#[test]
fn a_folder_that_moved_keeps_everything_hanging_from_its_songs() {
    let dir = temp_dir("moved folder");
    let before = dir.join("dulu");
    let music = before.join("音楽 folder");
    fixture(&music);
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();

    // A song that reads properly: the fixture also holds a file that does
    // not, and what happens to that one is another test's business.
    let one = library
        .tracks_page(TrackSort::Title, false, 0, 50)
        .unwrap()
        .into_iter()
        .find(|track| track.status == "ok")
        .unwrap();
    let relative = Path::new(&one.path)
        .strip_prefix(&music)
        .unwrap()
        .to_path_buf();
    library.record_play(one.id, 1, 60_000).unwrap();
    library.set_rating(one.id, 4).unwrap();
    let playlist = library.create_playlist("Bawa pindah").unwrap();
    library.add_to_playlist(playlist, &[one.id]).unwrap();
    library
        .set_override(one.id, Field::Title, Some("Judul baru"))
        .unwrap();

    // The whole thing moves, as a drive letter changing would move it.
    let after = dir.join("sekarang");
    std::fs::create_dir_all(&after).unwrap();
    let moved = after.join("音楽 folder");
    std::fs::rename(&music, &moved).unwrap();
    let carried = library.move_folder(&music, &moved).unwrap();
    assert!(carried > 0, "the tracks came along");

    assert_eq!(library.folder_paths().unwrap(), vec![moved.clone()]);
    let now = library.track(one.id).unwrap().unwrap();
    assert_eq!(
        Path::new(&now.path),
        moved.join(&relative),
        "the same song, at its new address"
    );
    assert_eq!(now.status, "ok", "and it is here again");
    assert_eq!(
        library.track_id(&moved.join(&relative)).unwrap(),
        Some(one.id)
    );
    assert_eq!(library.stats(one.id).unwrap().play_count, 1, "its history");
    assert_eq!(library.stats(one.id).unwrap().rating, 4);
    assert_eq!(
        library.playlist_tracks(playlist).unwrap()[0].id,
        one.id,
        "still on the playlist"
    );
    assert_eq!(
        library.track(one.id).unwrap().unwrap().title.as_deref(),
        Some("Judul baru"),
        "and the edit that was never written to the file"
    );

    // A rescan at the new place finds nothing new to do.
    let report = library.scan(|_| {}).unwrap();
    assert_eq!(report.missing, 0, "{report:?}");
    assert_eq!(report.read, 0, "nothing had to be read again: {report:?}");

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A folder left out of scanning stays out, and its songs leave the
/// library without leaving the disk.
#[test]
fn an_excluded_subfolder_stays_out_until_it_is_asked_back() {
    let dir = temp_dir("excluded");
    let music = dir.join("音楽 folder");
    fixture(&music);
    let samples = music.join("サンプル");
    std::fs::create_dir_all(&samples).unwrap();
    let sample = samples.join("loop.wav");
    song_file(
        &sample,
        &Song {
            title: "Sampel",
            artist: "Kerja",
            album: "Bahan",
            album_artist: None,
            track: 1,
            genre: "Pop",
            picture: None,
        },
    );
    let mut library = open(&dir);
    library.add_folder(&music).unwrap();
    library.scan(|_| {}).unwrap();
    assert!(
        library.track_id(&sample).unwrap().is_some(),
        "the sample is in the library to begin with"
    );
    let before = library.track_count().unwrap();

    let gone = library.exclude(&samples).unwrap();
    assert_eq!(gone, 1, "the sample left the library");
    assert!(sample.is_file(), "and stayed on the disk");
    assert_eq!(library.exclusions().unwrap(), vec![samples.clone()]);
    assert_eq!(library.track_count().unwrap(), before - 1);

    // A scan does not bring it back, and does not mark anything missing.
    let report = library.scan(|_| {}).unwrap();
    assert_eq!(report.missing, 0, "{report:?}");
    assert!(library.track_id(&sample).unwrap().is_none(), "still out");

    // Nor does the watcher notice anything there.
    library
        .apply_changes(&[Change::Changed(sample.clone())])
        .unwrap();
    assert!(library.track_id(&sample).unwrap().is_none(), "still out");

    // Asked back, it returns with the next scan.
    assert!(library.include(&samples).unwrap());
    library.scan(|_| {}).unwrap();
    assert!(library.track_id(&sample).unwrap().is_some(), "back again");

    drop(library);
    let _ = std::fs::remove_dir_all(&dir);
}
