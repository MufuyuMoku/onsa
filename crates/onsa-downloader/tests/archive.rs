//! What an archive is not allowed to do (SPEC §7.1, M10b).
//!
//! The archives here are built by hand rather than by a well-behaved writer,
//! on purpose: an archive meant to escape its folder would not be written by
//! one either. A tar header is 512 bytes of fixed layout, and a zip is a
//! local header, the bytes, a central directory and an end record — both are
//! short enough to write out plainly, and writing them out is the only way
//! to be sure the test really holds what it says it holds.

use std::path::{Path, PathBuf};

use onsa_downloader::archive::{find_named, unpack, Kind};
use onsa_downloader::Error;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("onsa archive {} {name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

// --- tar ------------------------------------------------------------------

/// One 512-byte tar header, with the checksum worked out the way tar does.
fn tar_header(name: &str, size: u64, kind: u8, link: &str) -> [u8; 512] {
    let mut header = [0u8; 512];
    let put = |header: &mut [u8; 512], at: usize, text: &[u8]| {
        header[at..at + text.len()].copy_from_slice(text);
    };
    put(&mut header, 0, name.as_bytes());
    put(&mut header, 100, b"000755 \0"); // mode
    put(&mut header, 108, b"000000 \0"); // uid
    put(&mut header, 116, b"000000 \0"); // gid
    put(&mut header, 124, format!("{size:011o} ").as_bytes());
    put(&mut header, 136, b"00000000000 "); // mtime
    header[148..156].copy_from_slice(b"        "); // checksum, blank while summing
    header[156] = kind;
    put(&mut header, 157, link.as_bytes());
    put(&mut header, 257, b"ustar\x0000");

    let sum: u32 = header.iter().map(|byte| u32::from(*byte)).sum();
    let checksum = format!("{sum:06o}\0 ");
    header[148..148 + checksum.len()].copy_from_slice(checksum.as_bytes());
    header
}

/// A tar holding exactly these entries: (name, contents, type flag, link).
fn tar(entries: &[(&str, &[u8], u8, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, body, kind, link) in entries {
        out.extend_from_slice(&tar_header(name, body.len() as u64, *kind, link));
        out.extend_from_slice(body);
        let padding = (512 - body.len() % 512) % 512;
        out.extend(std::iter::repeat_n(0u8, padding));
    }
    // Two empty blocks end a tar.
    out.extend(std::iter::repeat_n(0u8, 1024));
    out
}

const FILE: u8 = b'0';
const SYMLINK: u8 = b'2';
const HARDLINK: u8 = b'1';
const DIR: u8 = b'5';

// --- zip ------------------------------------------------------------------

/// A zip holding exactly these entries, stored rather than deflated, with
/// whatever external attributes are asked for.
fn zip(entries: &[(&str, &[u8], u32)]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut directory: Vec<u8> = Vec::new();
    let mut count = 0u16;

    for (name, body, unix_mode) in entries {
        let offset = out.len() as u32;
        let crc = crc32(body);
        let size = body.len() as u32;

        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // local header
        out.extend_from_slice(&10u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0u16.to_le_bytes()); // flags
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&0u16.to_le_bytes()); // time
        out.extend_from_slice(&0u16.to_le_bytes()); // date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(body);

        directory.extend_from_slice(&0x0201_4b50u32.to_le_bytes()); // central header
        directory.extend_from_slice(&0x031Eu16.to_le_bytes()); // made by unix
        directory.extend_from_slice(&10u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes()); // extra
        directory.extend_from_slice(&0u16.to_le_bytes()); // comment
        directory.extend_from_slice(&0u16.to_le_bytes()); // disk
        directory.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        directory.extend_from_slice(&(unix_mode << 16).to_le_bytes()); // external attrs
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(name.as_bytes());
        count += 1;
    }

    let directory_at = out.len() as u32;
    let directory_len = directory.len() as u32;
    out.extend_from_slice(&directory);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // end of directory
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&directory_len.to_le_bytes());
    out.extend_from_slice(&directory_at.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment
    out
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

// --- the rules ------------------------------------------------------------

fn refused(outcome: Result<Vec<PathBuf>, Error>, why: &str) {
    match outcome {
        Err(Error::BadArchive(said)) => {
            assert!(!said.is_empty(), "a refusal says which entry and why");
        }
        Err(other) => panic!("{why}: refused, but as {other:?}"),
        Ok(written) => panic!("{why}: it was unpacked anyway ({written:?})"),
    }
}

#[test]
fn an_ordinary_archive_is_unpacked() {
    let dir = temp_dir("ordinary tar");
    let bytes = tar(&[
        ("ffmpeg-7.1/", b"", DIR, ""),
        ("ffmpeg-7.1/bin/ffmpeg", b"not really ffmpeg", FILE, ""),
    ]);
    let written = unpack(&bytes, Kind::Tar, &dir).expect("it unpacks");
    let found = find_named(&written, "ffmpeg").expect("the program is in there");
    assert_eq!(std::fs::read(found).unwrap(), b"not really ffmpeg");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_ordinary_zip_is_unpacked() {
    let dir = temp_dir("ordinary zip");
    let bytes = zip(&[("deno", b"not really deno", 0o100755)]);
    let written = unpack(&bytes, Kind::Zip, &dir).expect("it unpacks");
    let found = find_named(&written, "deno").expect("the program is in there");
    assert_eq!(std::fs::read(found).unwrap(), b"not really deno");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_tar_that_climbs_out_of_its_folder_is_refused() {
    let dir = temp_dir("tar dotdot");
    let bytes = tar(&[("../escaped", b"somewhere else", FILE, "")]);
    refused(unpack(&bytes, Kind::Tar, &dir), "a tar with ..");
    assert!(!dir.exists(), "and nothing is left behind");
    assert!(
        !dir.parent().unwrap().join("escaped").exists(),
        "least of all outside the folder"
    );
}

#[test]
fn a_tar_with_an_absolute_path_is_refused() {
    let dir = temp_dir("tar absolute");
    let bytes = tar(&[("/etc/cron.d/onsa", b"no", FILE, "")]);
    refused(unpack(&bytes, Kind::Tar, &dir), "a tar with an absolute path");
    assert!(!dir.exists());
}

#[test]
fn a_tar_with_a_link_is_refused() {
    let dir = temp_dir("tar symlink");
    let bytes = tar(&[("innocent", b"", SYMLINK, "/etc/passwd")]);
    refused(unpack(&bytes, Kind::Tar, &dir), "a tar with a symbolic link");
    assert!(!dir.exists());

    let dir = temp_dir("tar hardlink");
    let bytes = tar(&[("innocent", b"", HARDLINK, "/etc/passwd")]);
    refused(unpack(&bytes, Kind::Tar, &dir), "a tar with a hard link");
    assert!(!dir.exists());
}

#[test]
fn a_zip_that_climbs_out_of_its_folder_is_refused() {
    let dir = temp_dir("zip dotdot");
    let bytes = zip(&[("../escaped", b"somewhere else", 0o100644)]);
    refused(unpack(&bytes, Kind::Zip, &dir), "a zip with ..");
    assert!(!dir.exists());
    assert!(!dir.parent().unwrap().join("escaped").exists());
}

#[test]
fn a_zip_with_an_absolute_path_is_refused() {
    let dir = temp_dir("zip absolute");
    let bytes = zip(&[("/etc/cron.d/onsa", b"no", 0o100644)]);
    refused(unpack(&bytes, Kind::Zip, &dir), "a zip with an absolute path");
    assert!(!dir.exists());
}

#[test]
fn a_zip_with_a_symbolic_link_is_refused() {
    let dir = temp_dir("zip symlink");
    // 0o120000 is what a zip records for a link, and the body is where it
    // points — which is how this trick is played.
    let bytes = zip(&[("innocent", b"/etc/passwd", 0o120777)]);
    refused(unpack(&bytes, Kind::Zip, &dir), "a zip with a symbolic link");
    assert!(!dir.exists());
}

#[test]
fn one_bad_entry_refuses_the_whole_archive() {
    let dir = temp_dir("tar mixed");
    let bytes = tar(&[
        ("good/thing", b"fine", FILE, ""),
        ("../escaped", b"not fine", FILE, ""),
    ]);
    refused(unpack(&bytes, Kind::Tar, &dir), "one entry out of two");
    assert!(
        !dir.exists(),
        "the half that was fine does not survive the half that was not"
    );
}

#[test]
fn an_archive_larger_than_onsa_will_hold_is_refused() {
    let dir = temp_dir("tar huge");
    // The header claims half a gigabyte; the bytes are not there, and that
    // is the point — the claim alone is what the cap is checked against.
    let mut bytes = tar(&[("small", b"x", FILE, "")]);
    bytes.truncate(bytes.len() - 1024);
    bytes.extend_from_slice(&{
        let mut header = [0u8; 512];
        header.copy_from_slice(&tar_header("enormous", 500 * 1024 * 1024, FILE, ""));
        header
    });
    bytes.extend(std::iter::repeat_n(0u8, 1024));
    refused(unpack(&bytes, Kind::Tar, &dir), "an archive that is too large");
    assert!(!dir.exists());
}

#[test]
fn a_gzipped_tar_is_unpacked_the_same_way() {
    // Written with gzip's simplest stored form so the test needs no
    // compressor of its own: the archive module is what is being tested.
    use std::io::Write;
    let plain = tar(&[("fpcalc", b"not really fpcalc", FILE, "")]);
    let mut encoder =
        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&plain).unwrap();
    let bytes = encoder.finish().unwrap();

    let dir = temp_dir("targz");
    let written = unpack(&bytes, Kind::TarGz, &dir).expect("it unpacks");
    assert!(find_named(&written, "fpcalc").is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_gzipped_tar_that_climbs_out_is_still_refused() {
    use std::io::Write;
    let plain = tar(&[("../escaped", b"no", FILE, "")]);
    let mut encoder =
        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&plain).unwrap();
    let bytes = encoder.finish().unwrap();

    let dir = temp_dir("targz dotdot");
    refused(unpack(&bytes, Kind::TarGz, &dir), "a gzipped tar with ..");
    assert!(!dir.exists());
    let _ = std::fs::remove_dir_all(Path::new(&dir));
}

// --- installing out of an archive ----------------------------------------

/// A release made up for the test, so the rules can be checked without
/// fetching anything.
fn pretend(file_name: &'static str, sha256: &'static str) -> onsa_downloader::Release {
    onsa_downloader::Release {
        program: onsa_downloader::Program::Fpcalc,
        url: "https://github.com/acoustid/chromaprint/releases/download/v1.6.1/made-up",
        file_name,
        about_bytes: 1,
        verify: onsa_downloader::install::Verify::Pinned { sha256 },
        holds: &[onsa_downloader::Program::Fpcalc],
    }
}

#[test]
fn a_program_is_taken_out_of_its_archive_and_put_where_onsa_looks() {
    let dir = temp_dir("install from tar");
    std::fs::create_dir_all(&dir).unwrap();
    let inside = onsa_downloader::Program::Fpcalc.file_name();
    let named = format!("chromaprint-fpcalc-1.6.1/{inside}");
    let bytes = tar(&[(&named, b"not really fpcalc", FILE, "")]);
    let sum = onsa_downloader::install::sha256_hex(&bytes);
    let release = pretend("made-up.tar", Box::leak(sum.clone().into_boxed_str()));

    let installed = onsa_downloader::install::install_release(&dir, &release, &bytes, &sum)
        .expect("it installs");
    assert_eq!(installed, vec![dir.join(&inside)]);
    assert_eq!(std::fs::read(&installed[0]).unwrap(), b"not really fpcalc");

    // And the folder it was unpacked into is gone again.
    let leftovers: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with(".unpacking"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_archive_that_does_not_match_its_fingerprint_is_never_opened() {
    let dir = temp_dir("install mismatch");
    std::fs::create_dir_all(&dir).unwrap();
    let inside = onsa_downloader::Program::Fpcalc.file_name();
    let bytes = tar(&[(&inside, b"not really fpcalc", FILE, "")]);
    let release = pretend("made-up.tar", "0000000000000000000000000000000000000000000000000000000000000000");

    let outcome = onsa_downloader::install::install_release(
        &dir,
        &release,
        &bytes,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(
        matches!(outcome, Err(Error::ChecksumMismatch(_))),
        "{outcome:?}"
    );
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        0,
        "nothing was unpacked, not even to be looked at"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_archive_missing_what_it_should_hold_leaves_what_was_there_alone() {
    let dir = temp_dir("install missing");
    std::fs::create_dir_all(&dir).unwrap();
    let bytes = tar(&[("something-else", b"not the program", FILE, "")]);
    let sum = onsa_downloader::install::sha256_hex(&bytes);
    let release = pretend("made-up.tar", Box::leak(sum.clone().into_boxed_str()));

    let outcome = onsa_downloader::install::install_release(&dir, &release, &bytes, &sum);
    assert!(matches!(outcome, Err(Error::BadArchive(_))), "{outcome:?}");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}
