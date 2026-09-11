//! The cover cache (SPEC §5.2).
//!
//! A cover comes from the picture embedded in the file or, failing that,
//! from `cover.*`, `folder.*` or `front.*` next to it. Covers are
//! deduplicated by the SHA-256 of their bytes, and only small JPEG
//! thumbnails (128 and 512 px) are kept, so a list never loads a
//! full-resolution image. A cover that cannot be decoded (a broken file, or a
//! format such as AVIF) is skipped with a warning; it never fails a scan.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use sha2::{Digest, Sha256};

/// Thumbnail edge lengths, in pixels.
pub const THUMB_SIZES: [u32; 2] = [128, 512];

/// JPEG quality of the thumbnails.
const JPEG_QUALITY: u8 = 85;

/// Names looked for next to a track, in order of preference.
const FOLDER_IMAGE_STEMS: [&str; 3] = ["cover", "folder", "front"];

/// Image extensions a folder cover may have.
const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

/// A decoded cover and its cached thumbnails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thumbnails {
    /// SHA-256 of the original image bytes, lowercase hex.
    pub hash: String,
    /// Width of the original image.
    pub width: u32,
    /// Height of the original image.
    pub height: u32,
    /// File name of the 128 px thumbnail in the cover cache.
    pub thumb_128: String,
    /// File name of the 512 px thumbnail in the cover cache.
    pub thumb_512: String,
}

/// SHA-256 of `bytes`, as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The cover image in `dir`, if any: `cover.*`, then `folder.*`, then
/// `front.*`, in any letter case.
pub fn folder_image(dir: &Path) -> Option<PathBuf> {
    let entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .collect();
    FOLDER_IMAGE_STEMS.iter().find_map(|stem| {
        entries
            .iter()
            .find(|path| {
                let stem_matches = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case(stem));
                let extension_matches =
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| {
                            IMAGE_EXTENSIONS
                                .iter()
                                .any(|known| ext.eq_ignore_ascii_case(known))
                        });
                stem_matches && extension_matches
            })
            .cloned()
    })
}

/// Decodes `bytes` and writes both thumbnails into `cache_dir`, named by
/// the image's hash. `origin` only names the source in warnings. Returns
/// `None` when the image cannot be decoded or the thumbnails cannot be
/// written; the caller then goes on without a cover.
pub fn make_thumbnails(bytes: &[u8], cache_dir: &Path, origin: &Path) -> Option<Thumbnails> {
    let hash = sha256_hex(bytes);
    let image = match image::load_from_memory(bytes) {
        Ok(image) => image,
        Err(error) => {
            tracing::warn!(source = %origin.display(), "cover cannot be decoded, skipped: {error}");
            return None;
        }
    };
    let (width, height) = (image.width(), image.height());
    let mut names = Vec::with_capacity(THUMB_SIZES.len());
    for size in THUMB_SIZES {
        let name = format!("{hash}_{size}.jpg");
        let target = cache_dir.join(&name);
        if !target.is_file() {
            // Never enlarge a small cover; only shrink a large one.
            let edge = size.min(width.max(height));
            let thumbnail = image.thumbnail(edge, edge).to_rgb8();
            if let Err(error) = write_jpeg_atomically(&thumbnail, &target) {
                tracing::warn!(source = %origin.display(), "cover thumbnail cannot be written: {error}");
                return None;
            }
        }
        names.push(name);
    }
    let thumb_512 = names.pop()?;
    let thumb_128 = names.pop()?;
    Some(Thumbnails {
        hash,
        width,
        height,
        thumb_128,
        thumb_512,
    })
}

/// Writes next to the target and renames, so a crash never leaves half a
/// thumbnail behind.
fn write_jpeg_atomically(image: &image::RgbImage, target: &Path) -> std::io::Result<()> {
    let partial = target.with_extension("jpg.part");
    let result = (|| {
        let mut out = BufWriter::new(File::create(&partial)?);
        image
            .write_with_encoder(JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY))
            .map_err(std::io::Error::other)?;
        out.into_inner()
            .map_err(|error| error.into_error())?
            .sync_all()?;
        std::fs::rename(&partial, target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "onsa ライブラリ cover {} {name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        let mut bytes = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn hashes_like_sha256() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn makes_both_thumbnails_keeping_the_shape() {
        let dir = temp_dir("thumbs");
        let thumbs = make_thumbnails(&png(1200, 800), &dir, Path::new("test.png")).unwrap();
        assert_eq!((thumbs.width, thumbs.height), (1200, 800));
        let small = image::open(dir.join(&thumbs.thumb_128)).unwrap();
        let large = image::open(dir.join(&thumbs.thumb_512)).unwrap();
        assert_eq!((small.width(), small.height()), (128, 85));
        assert_eq!((large.width(), large.height()), (512, 341));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn never_enlarges_a_small_cover() {
        let dir = temp_dir("small");
        let thumbs = make_thumbnails(&png(100, 100), &dir, Path::new("small.png")).unwrap();
        let large = image::open(dir.join(&thumbs.thumb_512)).unwrap();
        assert_eq!((large.width(), large.height()), (100, 100));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_broken_image_is_skipped_without_failing() {
        let dir = temp_dir("broken");
        assert!(make_thumbnails(b"not an image at all", &dir, Path::new("x.jpg")).is_none());
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            0,
            "nothing half-written"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn folder_images_follow_the_order_of_preference() {
        let dir = temp_dir("folder");
        assert_eq!(folder_image(&dir), None);
        std::fs::write(dir.join("Front.PNG"), b"x").unwrap();
        std::fs::write(dir.join("notes.txt"), b"x").unwrap();
        assert_eq!(folder_image(&dir), Some(dir.join("Front.PNG")));
        std::fs::write(dir.join("folder.jpg"), b"x").unwrap();
        assert_eq!(folder_image(&dir), Some(dir.join("folder.jpg")));
        std::fs::write(dir.join("Cover.JPG"), b"x").unwrap();
        assert_eq!(folder_image(&dir), Some(dir.join("Cover.JPG")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
