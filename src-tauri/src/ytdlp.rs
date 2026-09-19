//! Asking yt-dlp what is at a URL, and fetching it (SPEC §7.2).
//!
//! yt-dlp is a program Onsa drives, not a library it links. Everything goes
//! through an argument array — the URL included — so nothing anybody pastes
//! can become a command (SPEC §7.3).
//!
//! Two things happen here. A **probe** asks what is at a URL without
//! fetching anything, so the listener can see what they are about to get and
//! untick what they do not want. A **download** runs the real thing and
//! reports as it goes, and can be stopped together with everything yt-dlp
//! started.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use onsa_downloader::{Program, Programs};
use serde::{Deserialize, Serialize};

/// How long asking what is at a URL may take before Onsa gives up.
const PROBE_SECONDS: u64 = 60;

/// What is at a URL, as far as yt-dlp can say without fetching it.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    /// What the page calls itself.
    pub title: String,
    /// Whether it is a playlist rather than one recording.
    pub playlist: bool,
    /// What it holds. One entry for a single video.
    pub entries: Vec<Entry>,
}

/// One thing at a URL.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// The URL of this one item, which is what gets downloaded.
    pub url: String,
    /// What it is called.
    pub title: String,
    /// How long it is, in seconds, when that is known.
    pub seconds: Option<f64>,
    /// Who uploaded it.
    pub uploader: Option<String>,
}

/// What to ask yt-dlp for (SPEC §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    /// The audio as it already is, with no conversion. What Onsa suggests.
    Original,
    /// Converted to MP3.
    Mp3,
    /// Converted to FLAC.
    Flac,
}

impl Format {
    /// The name a theme-free interface uses for it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
        }
    }

    /// Reads the name back.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "original" => Some(Self::Original),
            "mp3" => Some(Self::Mp3),
            "flac" => Some(Self::Flac),
            _ => None,
        }
    }
}

/// How a download is going.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// How far along the current file is, from nothing to one.
    pub fraction: Option<f64>,
    /// Bytes a second, as yt-dlp reckons it.
    pub speed: Option<f64>,
    /// Seconds left, as yt-dlp reckons it.
    pub eta: Option<f64>,
    /// The file it is working on.
    pub name: Option<String>,
}

/// Whether a URL is one Onsa will hand to yt-dlp at all.
///
/// Only http and https. A local path, a `file:` URL, or something with a
/// scheme nobody has heard of is refused here rather than handed over to
/// see what happens.
pub fn is_a_url(text: &str) -> bool {
    let text = text.trim();
    (text.starts_with("http://") || text.starts_with("https://"))
        && text.len() > 10
        && !text.contains(char::is_whitespace)
}

/// Asks what is at a URL, without fetching any of it.
pub fn probe(programs: &Programs, url: &str) -> Result<Probe, String> {
    if !is_a_url(url) {
        return Err("notAUrl".to_string());
    }
    let args = vec!["-J", "--flat-playlist", "--no-warnings", url];
    let ran = programs
        .run(
            Program::YtDlp,
            &args,
            std::time::Duration::from_secs(PROBE_SECONDS),
        )
        .map_err(|error| {
            tracing::warn!("yt-dlp could not be asked about a URL: {error}");
            "cannotRun".to_string()
        })?;
    if !ran.ok {
        let said = ran
            .err
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or_default()
            .to_string();
        tracing::info!("yt-dlp refused a URL: {said}");
        return Err("refused".to_string());
    }
    read_probe(&ran.out, url).ok_or_else(|| "unreadable".to_string())
}

/// Reads what `yt-dlp -J --flat-playlist` printed.
fn read_probe(printed: &str, asked: &str) -> Option<Probe> {
    let answer: serde_json::Value = serde_json::from_str(printed).ok()?;
    let title = text(answer.get("title")).unwrap_or_else(|| asked.to_string());

    // A playlist has entries; a single video does not.
    let entries = answer.get("entries").and_then(|list| list.as_array());
    let Some(entries) = entries else {
        return Some(Probe {
            title: title.clone(),
            playlist: false,
            entries: vec![Entry {
                url: text(answer.get("webpage_url")).unwrap_or_else(|| asked.to_string()),
                title,
                seconds: answer.get("duration").and_then(serde_json::Value::as_f64),
                uploader: text(answer.get("uploader")),
            }],
        });
    };

    let mut items = Vec::with_capacity(entries.len());
    for entry in entries {
        // A flat playlist gives a url or an id; an entry with neither is one
        // nobody can fetch, so it is left out rather than shown as a row
        // that will fail.
        let Some(url) = text(entry.get("url")).or_else(|| text(entry.get("webpage_url"))) else {
            continue;
        };
        items.push(Entry {
            url,
            title: text(entry.get("title")).unwrap_or_else(|| "?".to_string()),
            seconds: entry.get("duration").and_then(serde_json::Value::as_f64),
            uploader: text(entry.get("uploader")),
        });
    }
    Some(Probe {
        title,
        playlist: true,
        entries: items,
    })
}

/// The formats Onsa asks for, in order of preference.
///
/// Only formats Onsa can actually play are named, and there is deliberately
/// no `/bestaudio` at the end: without one, a site that has nothing but
/// Opus is refused before anything is fetched, and the listener is told.
/// With one, a file Onsa cannot decode would land on the disk and then
/// quietly fail to appear in the library, which is worse than a refusal.
///
/// Opus is what changes in v1.1 (M11), here and in the decoder.
pub const WANTED: &str = "bestaudio[ext=m4a]/bestaudio[ext=mp3]/bestaudio[ext=aac]";

/// The arguments one download is run with (SPEC §7.2).
///
/// Built here, in one place, so what Onsa asks of yt-dlp can be read at a
/// glance — and so the format priority that has to change in M11 is a single
/// line rather than a search.
pub fn arguments(
    url: &str,
    format: Format,
    into: &Path,
    ffmpeg: Option<&Path>,
) -> Vec<std::ffi::OsString> {
    let mut args: Vec<std::ffi::OsString> = vec![
        "--newline".into(),
        "--no-warnings".into(),
        "--progress-template".into(),
        "download:%(progress)j".into(),
        "--print".into(),
        "after_move:filepath".into(),
        "--no-playlist".into(),
    ];

    // Everything that happens to a file after it lands needs ffmpeg —
    // extracting the audio, putting the tags in, putting the cover in. On a
    // machine without one, asking for any of it fails the whole download,
    // and the listener is left with a file they never got and a word that
    // does not say why. So Onsa asks for what it can have: the audio as the
    // site already keeps it, and nothing written into it.
    args.push("-f".into());
    args.push(WANTED.into());
    if let Some(found) = ffmpeg {
        args.push("--embed-metadata".into());
        args.push("--embed-thumbnail".into());
        args.push("-x".into());
        args.push("--ffmpeg-location".into());
        args.push(found.as_os_str().to_os_string());
        match format {
            Format::Original => {}
            Format::Mp3 => {
                args.push("--audio-format".into());
                args.push("mp3".into());
            }
            Format::Flac => {
                args.push("--audio-format".into());
                args.push("flac".into());
            }
        }
    }

    args.push("-o".into());
    let mut pattern = into.as_os_str().to_os_string();
    pattern.push(std::path::MAIN_SEPARATOR.to_string());
    pattern.push("%(artist,uploader)s/%(title)s [%(id)s].%(ext)s");
    args.push(pattern);

    args.push(url.into());
    args
}

/// Fetches one URL, reporting as it goes.
///
/// Every line yt-dlp prints is read: the progress lines become [`Step`]s,
/// and the one line that is a path is the file that ended up on disk. A line
/// that is neither goes to the log and stops nothing, as SPEC §7.2 asks.
pub fn download(
    programs: &Programs,
    url: &str,
    format: Format,
    into: &Path,
    stop: Arc<AtomicBool>,
    mut watching: impl FnMut(Step),
) -> Result<Vec<PathBuf>, String> {
    if !is_a_url(url) {
        return Err("notAUrl".to_string());
    }
    let found = programs
        .find(Program::YtDlp)
        .ok_or_else(|| "noYtDlp".to_string())?;
    std::fs::create_dir_all(into).map_err(|error| {
        tracing::warn!("the download folder cannot be made: {error}");
        "cannotWrite".to_string()
    })?;

    // What yt-dlp will find for itself, not what Onsa would choose: it
    // searches PATH on its own account (see `Programs::reachable`).
    let ffmpeg = programs.reachable(Program::Ffmpeg);
    let args = arguments(
        url,
        format,
        into,
        ffmpeg.as_ref().map(|one| one.path.as_path()),
    );
    let mut files: Vec<PathBuf> = Vec::new();
    let finished =
        onsa_downloader::runner::stream(&found.path, &args, stop, |line| match read_line(line) {
            Line::Progress(step) => watching(step),
            Line::File(path) => files.push(path),
            Line::Other => tracing::debug!(target: "onsa::ytdlp", "{line}"),
        })
        .map_err(|error| {
            tracing::warn!("yt-dlp could not be run: {error}");
            "cannotRun".to_string()
        })?;

    if finished.stopped {
        return Err("stopped".to_string());
    }
    if !finished.ok {
        let said = finished.reason();
        tracing::warn!("yt-dlp failed: {said}");
        return Err(why_of(&said));
    }
    Ok(files)
}

/// What a complaint from yt-dlp means, in a word the interface can say in
/// the listener's own language.
///
/// Two of these are worth telling apart, because the listener can do
/// something about each: a machine with no ffmpeg, and a site that has the
/// song only in a format Onsa cannot play yet. Anything else keeps the
/// program's own sentence, which is more use than "it failed".
fn why_of(said: &str) -> String {
    let lowered = said.to_lowercase();
    if lowered.contains("ffmpeg not found") || lowered.contains("ffprobe and ffmpeg not found") {
        return "noFfmpeg".to_string();
    }
    if lowered.contains("requested format is not available") {
        return "noPlayableFormat".to_string();
    }
    said.to_string()
}

/// What one line of yt-dlp's output turned out to be.
enum Line {
    Progress(Step),
    File(PathBuf),
    Other,
}

/// Reads one line, without ever letting an unfamiliar one matter.
fn read_line(line: &str) -> Line {
    let line = line.trim();
    if line.is_empty() {
        return Line::Other;
    }
    if let Some(json) = line.strip_prefix("download:") {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
            return Line::Progress(read_step(&value));
        }
        return Line::Other;
    }
    // The only other line Onsa asked for is the path of a finished file.
    let looks_like_a_path = Path::new(line).is_absolute();
    if looks_like_a_path {
        return Line::File(PathBuf::from(line));
    }
    Line::Other
}

/// Reads one progress report.
fn read_step(value: &serde_json::Value) -> Step {
    let number = |key: &str| value.get(key).and_then(serde_json::Value::as_f64);
    let done = number("downloaded_bytes");
    let total = number("total_bytes").or_else(|| number("total_bytes_estimate"));
    Step {
        fraction: match (done, total) {
            (Some(done), Some(total)) if total > 0.0 => Some((done / total).clamp(0.0, 1.0)),
            _ => None,
        },
        speed: number("speed"),
        eta: number("eta"),
        name: value
            .get("info_dict")
            .and_then(|info| info.get("title"))
            .and_then(|title| title.as_str())
            .map(str::to_string)
            .or_else(|| text(value.get("filename")).map(|path| file_name(&path))),
    }
}

/// The last part of a path, whichever separator it uses.
fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

/// A string field that is there and says something.
fn text(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_web_address_is_handed_over() {
        assert!(is_a_url("https://example.com/watch?v=abc"));
        assert!(is_a_url("http://example.com/x"));
        for wrong in [
            "file:///C:/Windows/System32",
            "C:\\music\\song.mp3",
            "/etc/passwd",
            "ftp://example.com/x",
            "https://a b.com",
            "https://",
            "",
            "   ",
        ] {
            assert!(!is_a_url(wrong), "{wrong}");
        }
    }

    #[test]
    fn a_single_video_reads_as_one_entry() {
        let printed = r#"{
            "title": "Lampu Kota",
            "webpage_url": "https://example.com/watch?v=abc",
            "duration": 215.4,
            "uploader": "Lilith"
        }"#;
        let probe = read_probe(printed, "https://example.com/x").expect("read");
        assert!(!probe.playlist);
        assert_eq!(probe.entries.len(), 1);
        assert_eq!(probe.entries[0].title, "Lampu Kota");
        assert_eq!(probe.entries[0].seconds, Some(215.4));
        assert_eq!(probe.entries[0].uploader.as_deref(), Some("Lilith"));
    }

    #[test]
    fn a_playlist_reads_as_its_items() {
        let printed = r#"{
            "title": "Kota Sunyi",
            "entries": [
                { "url": "https://example.com/1", "title": "Satu", "duration": 100 },
                { "url": "https://example.com/2", "title": "Dua" },
                { "title": "no url at all" }
            ]
        }"#;
        let probe = read_probe(printed, "https://example.com/list").expect("read");
        assert!(probe.playlist);
        assert_eq!(probe.title, "Kota Sunyi");
        assert_eq!(
            probe.entries.len(),
            2,
            "the one nobody can fetch is left out"
        );
        assert_eq!(probe.entries[0].seconds, Some(100.0));
    }

    #[test]
    fn a_progress_line_is_read_and_anything_else_is_not() {
        let line = r#"download:{"downloaded_bytes": 500, "total_bytes": 2000, "speed": 125000.0, "eta": 12, "filename": "C:\\music\\Lampu Kota [abc].m4a"}"#;
        match read_line(line) {
            Line::Progress(step) => {
                assert_eq!(step.fraction, Some(0.25));
                assert_eq!(step.speed, Some(125_000.0));
                assert_eq!(step.eta, Some(12.0));
                assert_eq!(step.name.as_deref(), Some("Lampu Kota [abc].m4a"));
            }
            _ => panic!("that was a progress line"),
        }

        // Nothing else is allowed to matter.
        for other in [
            "[youtube] abc: Downloading webpage",
            "download:{not json at all",
            "WARNING: something",
            "",
        ] {
            assert!(matches!(read_line(other), Line::Other), "{other}");
        }
    }

    #[test]
    fn the_line_that_is_a_path_is_the_file_that_arrived() {
        let path = if cfg!(windows) {
            r"C:\music\Lilith\Lampu Kota [abc].m4a"
        } else {
            "/home/rian/music/Lilith/Lampu Kota [abc].m4a"
        };
        match read_line(path) {
            Line::File(found) => assert_eq!(found, PathBuf::from(path)),
            _ => panic!("that was the finished file"),
        }
    }

    #[test]
    fn a_progress_line_without_a_total_says_only_what_it_knows() {
        let line = r#"download:{"downloaded_bytes": 500}"#;
        match read_line(line) {
            Line::Progress(step) => {
                assert_eq!(step.fraction, None, "a bar with no length is not drawn");
                assert_eq!(step.speed, None);
            }
            _ => panic!("that was a progress line"),
        }
    }

    /// The arguments as words, which is how these tests read them.
    fn words_of(format: Format, ffmpeg: Option<&Path>) -> Vec<String> {
        arguments(
            "https://example.com/watch?v=abc",
            format,
            Path::new("/music"),
            ffmpeg,
        )
        .iter()
        .map(|one| one.to_string_lossy().to_string())
        .collect()
    }

    #[test]
    fn what_onsa_asks_yt_dlp_for_is_readable_at_a_glance() {
        let ffmpeg = PathBuf::from("/usr/bin/ffmpeg");
        let words = words_of(Format::Original, Some(&ffmpeg));

        assert!(words.contains(&"-x".to_string()), "audio only");
        assert!(words.contains(&"--embed-metadata".to_string()));
        assert!(
            words.contains(&"--no-playlist".to_string()),
            "one at a time"
        );
        // Only formats Onsa can play, and no `bestaudio` to fall back to
        // one it cannot. That is what changes in v1.1 (SPEC §7.2).
        assert!(words.contains(&WANTED.to_string()));
        assert!(!WANTED.contains("/bestaudio\""), "nothing catches all");
        assert_eq!(
            words.last().map(String::as_str),
            Some("https://example.com/watch?v=abc"),
            "the URL is an argument, and the last one"
        );
    }

    #[test]
    fn without_ffmpeg_nothing_is_asked_that_needs_it() {
        // A machine with no ffmpeg cannot extract, cannot write tags in and
        // cannot put a cover in. Asking anyway fails the whole download, so
        // Onsa asks for the audio as it is and says nothing about the rest.
        let words = words_of(Format::Original, None);
        for needs_it in [
            "-x",
            "--embed-metadata",
            "--embed-thumbnail",
            "--ffmpeg-location",
            "--audio-format",
        ] {
            assert!(!words.contains(&needs_it.to_string()), "{needs_it}");
        }
        assert!(
            words.contains(&WANTED.to_string()),
            "the audio is still asked for"
        );
    }

    #[test]
    fn a_conversion_says_which_one() {
        let ffmpeg = PathBuf::from("/usr/bin/ffmpeg");
        for (format, name) in [(Format::Mp3, "mp3"), (Format::Flac, "flac")] {
            let words = words_of(format, Some(&ffmpeg));
            assert!(words.contains(&"--audio-format".to_string()));
            assert!(words.contains(&name.to_string()));
        }
        // And a conversion asked for without one is simply not asked for:
        // the interface does not offer it, and the arguments do not carry it.
        let words = words_of(Format::Mp3, None);
        assert!(!words.contains(&"mp3".to_string()));
    }

    #[test]
    fn a_complaint_becomes_something_the_window_can_say() {
        assert_eq!(
            why_of("ERROR: Postprocessing: ffmpeg not found. Please install"),
            "noFfmpeg"
        );
        assert_eq!(
            why_of("ERROR: ffprobe and ffmpeg not found. Please install"),
            "noFfmpeg"
        );
        assert_eq!(
            why_of("ERROR: [youtube] abc: Requested format is not available"),
            "noPlayableFormat"
        );
        // Anything else keeps its own words, which say more than "failed".
        assert_eq!(
            why_of("ERROR: unable to download webpage"),
            "ERROR: unable to download webpage"
        );
    }

    #[test]
    fn every_format_survives_the_round_trip_to_its_name() {
        for format in [Format::Original, Format::Mp3, Format::Flac] {
            assert_eq!(Format::from_name(format.name()), Some(format));
        }
        assert_eq!(Format::from_name("wav"), None);
    }
}
