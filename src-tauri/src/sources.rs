//! The sites yt-dlp knows about (SPEC §15, M15).
//!
//! The list is asked of the yt-dlp that is installed, never written down
//! here: a list in Onsa's code would be a list about the yt-dlp Onsa was
//! built against, and would quietly drift from the one doing the work. Swap
//! yt-dlp for another version and this page changes with it.
//!
//! What the list means is another matter, and the page says so: yt-dlp
//! knowing a site's name is not a promise that a download from it will
//! work. Some need an account, some are broken today and fixed next week —
//! and yt-dlp marks the ones it knows are broken, which is passed through
//! rather than hidden.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, State};

use onsa_downloader::Program;

use crate::error::ErrorCode;
use crate::online;

/// How long yt-dlp is given to print its list. It is a long list, and on a
/// cold disk the first run of a program is the slow one.
const PATIENCE: Duration = Duration::from_secs(60);

/// What yt-dlp marks a broken extractor with.
const BROKEN: &str = "(CURRENTLY BROKEN)";

/// The music sites worth putting first, in this order.
///
/// Matched against the beginning of an extractor's name, so `soundcloud`
/// brings its playlist and user pages with it. Everything else keeps the
/// order yt-dlp printed, which is alphabetical.
const MUSIC_FIRST: &[&str] = &[
    "youtube",
    "soundcloud",
    "bandcamp",
    "mixcloud",
    "audiomack",
    "jamendo",
    "archive.org",
    "vimeo",
];

/// One site yt-dlp knows about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDto {
    /// The name yt-dlp calls it.
    pub name: String,
    /// Whether yt-dlp itself says this one is broken at the moment.
    pub broken: bool,
    /// Whether it is one of the music sites shown first.
    pub music: bool,
}

/// The list, and whether there was a yt-dlp to ask.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcesDto {
    /// Whether yt-dlp is installed at all. When it is not, the list is
    /// empty and the page says where to get one rather than showing
    /// nothing.
    pub available: bool,
    /// The version that answered, so the page can say which yt-dlp this is
    /// a list about.
    pub version: Option<String>,
    /// The sites, music first and the rest as yt-dlp printed them.
    pub sources: Vec<SourceDto>,
}

/// Reads what `yt-dlp --list-extractors` printed.
///
/// One name per line, with `(CURRENTLY BROKEN)` after the ones yt-dlp knows
/// are not working. Anything that is not shaped like a name — a blank line,
/// a warning that found its way onto the same stream — is left out.
pub fn read_list(printed: &str) -> Vec<SourceDto> {
    let mut found: Vec<SourceDto> = Vec::new();
    for line in printed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (name, broken) = match line.split_once(BROKEN) {
            Some((before, _)) => (before.trim(), true),
            None => (line, false),
        };
        // A name has no spaces in it. Whatever else came down the pipe is
        // not one of these.
        if name.is_empty() || name.contains(char::is_whitespace) {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        found.push(SourceDto {
            music: MUSIC_FIRST.iter().any(|one| lower.starts_with(one)),
            name: name.to_string(),
            broken,
        });
    }
    // The music sites first, in the order they are listed above; everything
    // else keeps the order it arrived in.
    let rank = |one: &SourceDto| -> usize {
        let lower = one.name.to_ascii_lowercase();
        MUSIC_FIRST
            .iter()
            .position(|first| lower.starts_with(first))
            .unwrap_or(MUSIC_FIRST.len())
    };
    found.sort_by_key(rank);
    found
}

/// What is kept between one asking and the next.
#[derive(Default)]
pub struct Sources {
    /// The answer, and the version it was an answer about.
    known: Mutex<Option<(String, Vec<SourceDto>)>>,
}

/// The sites the installed yt-dlp knows about.
///
/// Asked once per version per run: the list is seventeen hundred names and
/// running a program to print them is not work to repeat on every keystroke
/// in the search box.
#[tauri::command]
pub async fn download_sources(
    app: AppHandle,
    kept: State<'_, std::sync::Arc<Sources>>,
) -> Result<SourcesDto, ErrorCode> {
    let kept = kept.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let programs = online::programs(&app)?;
        let Some(version) = programs.version(Program::YtDlp) else {
            return Ok(SourcesDto::default());
        };

        if let Ok(known) = kept.known.lock() {
            if let Some((was, sources)) = known.as_ref() {
                if was == &version {
                    return Ok(SourcesDto {
                        available: true,
                        version: Some(version),
                        sources: sources.clone(),
                    });
                }
            }
        }

        let ran = programs
            .run(Program::YtDlp, &["--list-extractors"], PATIENCE)
            .map_err(|error| {
                tracing::warn!("yt-dlp would not list what it knows: {error}");
                ErrorCode::Download
            })?;
        let sources = read_list(&ran.out);
        tracing::info!(
            version = %version,
            sites = sources.len(),
            "the list of sites came from the installed yt-dlp"
        );
        if let Ok(mut known) = kept.known.lock() {
            *known = Some((version.clone(), sources.clone()));
        }
        Ok(SourcesDto {
            available: true,
            version: Some(version),
            sources,
        })
    })
    .await
    .map_err(|_| ErrorCode::Library)?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real lines from `yt-dlp --list-extractors`, kept here so the test
    /// needs neither yt-dlp nor the network (SPEC §12).
    const PRINTED: &str = "\
10play
10play:season
20min (CURRENTLY BROKEN)
Bandcamp
Bandcamp:album
mixcloud
soundcloud
soundcloud:playlist
youtube
youtube:tab
zingmp3

WARNING: something on the same stream
";

    #[test]
    fn a_name_is_read_and_a_broken_one_is_marked() {
        let found = read_list(PRINTED);
        let names: Vec<&str> = found.iter().map(|one| one.name.as_str()).collect();
        assert!(names.contains(&"10play"));
        assert!(names.contains(&"zingmp3"));
        assert!(
            !names.iter().any(|name| name.contains("WARNING")),
            "a warning is not a site: {names:?}"
        );
        let broken = found
            .iter()
            .find(|one| one.name == "20min")
            .expect("it is listed");
        assert!(broken.broken, "yt-dlp said this one is broken");
        assert!(!found.iter().any(|one| one.name.contains(BROKEN)));
    }

    #[test]
    fn the_music_sites_come_first_in_the_order_they_are_named() {
        let found = read_list(PRINTED);
        let music: Vec<&str> = found
            .iter()
            .take_while(|one| one.music)
            .map(|one| one.name.as_str())
            .collect();
        assert_eq!(
            music,
            vec![
                "youtube",
                "youtube:tab",
                "soundcloud",
                "soundcloud:playlist",
                "Bandcamp",
                "Bandcamp:album",
                "mixcloud"
            ],
            "the order of MUSIC_FIRST, and each site keeps its own pages"
        );
        assert!(
            found.iter().skip(music.len()).all(|one| !one.music),
            "and nothing musical is left further down"
        );
    }

    #[test]
    fn nothing_printed_is_no_sites_rather_than_a_guess() {
        assert!(read_list("").is_empty());
        assert!(read_list("\n\n  \n").is_empty());
    }
}
