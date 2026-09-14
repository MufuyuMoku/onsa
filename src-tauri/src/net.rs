//! The one way Onsa talks to the internet (SPEC §1, §14).
//!
//! Every request in the application goes through the client built here, so
//! the rules hold everywhere at once: a User-Agent that says who is calling,
//! timeouts on both the connection and the whole exchange, a ceiling on how
//! much will be read, and failures that come back as values rather than
//! taking anything down with them.
//!
//! The client blocks. Onsa has no request it wants to wait for on a thread
//! that matters: the library has its own thread, commands hand long work to
//! `spawn_blocking`, and the audio engine never speaks to the network at
//! all. Calling one of these from an async task would block its runtime
//! thread, so `spawn_blocking` is not optional there.

use std::io::Read;
use std::time::Duration;

/// How long to wait for the far end to answer at all.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How long one whole request may take, connection included.
const TOTAL_TIMEOUT: Duration = Duration::from_secs(30);
/// Most an answer may be before Onsa stops reading it. Metadata answers are
/// measured in kilobytes; anything near this is not what was asked for.
pub const MAX_ANSWER: u64 = 8 * 1024 * 1024;
/// What went wrong, in terms the interface can translate (SPEC §9.6).
///
/// It says what Onsa can do about it, not what the protocol called it: the
/// difference that matters is between "try again later" and "this will not
/// work".
#[derive(Debug, PartialEq, Eq)]
pub enum NetError {
    /// The service could not be reached, or took too long.
    Unreachable,
    /// The answer was larger than Onsa is willing to read.
    TooLarge,
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreachable => write!(f, "the service could not be reached"),
            Self::TooLarge => write!(f, "the answer was too large"),
        }
    }
}

impl std::error::Error for NetError {}

/// The User-Agent every request carries.
///
/// MusicBrainz asks for the application, its version and a way to get in
/// touch (SPEC §8); the others are happy with the same thing.
pub fn user_agent() -> String {
    format!(
        "Onsa/{} ( https://github.com/MufuyuMoku/onsa )",
        env!("CARGO_PKG_VERSION")
    )
}

/// Builds the client the whole application shares.
///
/// A client that cannot be built is not worth bringing the application down
/// for: the caller gets `None` and the feature that wanted the network says
/// it is unavailable.
pub fn client() -> Option<reqwest::blocking::Client> {
    match reqwest::blocking::Client::builder()
        .user_agent(user_agent())
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TOTAL_TIMEOUT)
        // Onsa asks services for data; it never follows a service somewhere
        // else, which is one fewer way for a redirect to lead astray.
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    {
        Ok(client) => Some(client),
        Err(error) => {
            tracing::error!("the HTTP client could not be built: {error}");
            None
        }
    }
}

/// Reads a response body, refusing anything past `limit`.
///
/// The length the server claims is checked first, and then the reading is
/// capped as well: a server that lies about the length, or does not say,
/// still cannot make Onsa read forever.
pub fn read_capped(response: reqwest::blocking::Response, limit: u64) -> Result<Vec<u8>, NetError> {
    if let Some(length) = response.content_length() {
        if length > limit {
            return Err(NetError::TooLarge);
        }
    }
    let mut body = Vec::new();
    // One byte past the limit is enough to know it was over it.
    let mut reader = response.take(limit + 1);
    reader.read_to_end(&mut body).map_err(|error| {
        tracing::debug!("the answer could not be read: {error}");
        NetError::Unreachable
    })?;
    if body.len() as u64 > limit {
        return Err(NetError::TooLarge);
    }
    Ok(body)
}

/// An error without its URL: a URL can carry an API key in its query.
fn tidy(error: &reqwest::Error) -> String {
    let mut without = error.to_string();
    if let Some(url) = error.url() {
        without = without.replace(url.as_str(), "<url>");
    }
    without
}

/// How often Onsa may ask one service, and when it last did.
///
/// MusicBrainz asks for no more than one request a second (SPEC §8) and
/// AcoustID for no more than three; both are house rules worth keeping, not
/// limits to be crept up to. The wait happens on the thread that is asking,
/// which is always a worker thread, so a queue of lookups slows itself down
/// rather than anything the listener is looking at.
pub struct RateLimit {
    every: Duration,
    last: std::sync::Mutex<Option<std::time::Instant>>,
}

impl RateLimit {
    /// A limit of one request every `every`.
    pub const fn new(every: Duration) -> Self {
        Self {
            every,
            last: std::sync::Mutex::new(None),
        }
    }

    /// Waits until the next request is allowed, then marks it as made.
    pub fn wait(&self) {
        let mut last = self
            .last
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = std::time::Instant::now();
        if let Some(previous) = *last {
            let since = now.duration_since(previous);
            if since < self.every {
                std::thread::sleep(self.every - since);
            }
        }
        *last = Some(std::time::Instant::now());
    }
}

/// MusicBrainz: one request a second, as its own rules ask (SPEC §8).
// Waiting for the lookups it holds back, which land next.
#[allow(dead_code)]
pub static MUSICBRAINZ: RateLimit = RateLimit::new(Duration::from_millis(1000));
/// AcoustID: three requests a second, as its documentation asks.
pub static ACOUSTID: RateLimit = RateLimit::new(Duration::from_millis(334));

/// A service Onsa asks about metadata (SPEC §8).
///
/// Each one carries its own address and its own rate limit, so a caller
/// cannot ask one of them faster than it agreed to by going through a
/// different door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    /// AcoustID, which recognises a fingerprint.
    AcoustId,
    /// MusicBrainz, which knows what a recording belongs to.
    MusicBrainz,
    /// The Cover Art Archive, which has the picture.
    CoverArt,
}

impl Service {
    /// Where it really lives.
    const fn home(self) -> &'static str {
        match self {
            Self::AcoustId => "https://api.acoustid.org/v2",
            Self::MusicBrainz => "https://musicbrainz.org/ws/2",
            Self::CoverArt => "https://coverartarchive.org",
        }
    }

    /// The variable that can move it, for testing only. See [`Service::base`].
    const fn moved_by(self) -> &'static str {
        match self {
            Self::AcoustId => "ONSA_ACOUSTID_URL",
            Self::MusicBrainz => "ONSA_MUSICBRAINZ_URL",
            Self::CoverArt => "ONSA_COVERART_URL",
        }
    }

    /// How often it may be asked.
    pub fn limit(self) -> &'static RateLimit {
        match self {
            Self::AcoustId => &ACOUSTID,
            // The Cover Art Archive is fronted by MusicBrainz and counted
            // against the same allowance.
            Self::MusicBrainz | Self::CoverArt => &MUSICBRAINZ,
        }
    }

    /// The address to ask, which is the real one unless a test has stood a
    /// service up on this machine.
    ///
    /// This is the seam the whole flow is tested through: a fake AcoustID on
    /// loopback answers the release build exactly as the real one would, so
    /// the matching, the confidence, the cover and the undo can all be tried
    /// end to end without a key, without the network, and without touching
    /// anybody's music.
    ///
    /// **It can only ever point at this machine.** An address that is not
    /// loopback is ignored and said so in the log, so a variable set by
    /// something other than a test cannot send a key, a fingerprint or
    /// anything else somewhere new.
    pub fn base(self) -> String {
        let Some(value) = std::env::var(self.moved_by()).ok() else {
            return self.home().to_string();
        };
        let value = value.trim().trim_end_matches('/').to_string();
        if value.is_empty() {
            return self.home().to_string();
        }
        if loopback(&value) {
            tracing::warn!(
                service = ?self,
                "answering from this machine instead of the real service"
            );
            return value;
        }
        tracing::warn!(
            service = ?self,
            variable = self.moved_by(),
            "ignored: a service can only be moved to this machine"
        );
        self.home().to_string()
    }
}

/// Whether an address is this machine and nowhere else.
fn loopback(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }
    matches!(
        parsed.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("[::1]") | Some("::1")
    )
}

/// Asks a service for JSON, keeping to its rate limit.
///
/// Everything about it comes back as a value: a service that is down, slow,
/// too talkative or speaking nonsense all end as [`NetError`], and the
/// caller carries on with whatever it already has.
pub fn ask_json(
    service: Service,
    path: &str,
    query: &[(&str, &str)],
) -> Result<serde_json::Value, NetError> {
    let client = client().ok_or(NetError::Unreachable)?;
    service.limit().wait();
    let url = format!("{}{path}", service.base());
    let sent = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/json")
        .query(query)
        .send();
    let response = match sent {
        Ok(response) => response,
        Err(error) => {
            tracing::debug!(service = ?service, "could not be asked: {}", tidy(&error));
            return Err(NetError::Unreachable);
        }
    };
    let status = response.status();
    let body = read_capped(response, MAX_ANSWER)?;
    if !status.is_success() {
        tracing::debug!(service = ?service, status = status.as_u16(), "answered with a refusal");
        // A refusal still carries JSON often enough to be worth reading; the
        // caller decides what an answer without results means.
    }
    serde_json::from_slice(&body).map_err(|error| {
        tracing::debug!(service = ?service, "answered something that is not JSON: {error}");
        NetError::Unreachable
    })
}

/// Asks for a file, refusing anything past `limit`.
pub fn ask_bytes(service: Service, path: &str, limit: u64) -> Result<Vec<u8>, NetError> {
    let client = client().ok_or(NetError::Unreachable)?;
    service.limit().wait();
    let url = format!("{}{path}", service.base());
    let response = client.get(&url).send().map_err(|error| {
        tracing::debug!(service = ?service, "could not be asked: {}", tidy(&error));
        NetError::Unreachable
    })?;
    if !response.status().is_success() {
        return Err(NetError::Unreachable);
    }
    read_capped(response, limit)
}

/// A well-formed track id that stands for nothing in particular. AcoustID
/// answers a key it knows with an empty result, which is all this asks.
const NOWHERE_TRACK: &str = "00000000-0000-4000-8000-000000000000";

/// Whether AcoustID accepts this key, without reading any audio.
///
/// A lookup by track id needs no fingerprint, so the only thing the request
/// really asks about is the key. AcoustID does not document its error codes,
/// so the answer is read the way it reads: `ok` means the key went through,
/// and an error that names the key means it did not. An error about anything
/// else still means the key was accepted — it got far enough to complain
/// about the rest of the request.
pub fn acoustid_accepts(key: &str) -> crate::commands::KeyTest {
    use crate::commands::KeyTest;
    let Some(client) = client() else {
        return KeyTest::Unreachable;
    };
    Service::AcoustId.limit().wait();
    let sent = client
        .get(format!("{}/lookup", Service::AcoustId.base()))
        .query(&[
            ("client", key),
            ("trackid", NOWHERE_TRACK),
            ("meta", "recordingids"),
            ("format", "json"),
        ])
        .send();
    let response = match sent {
        Ok(response) => response,
        Err(error) => {
            tracing::debug!("AcoustID could not be asked: {}", tidy(&error));
            return KeyTest::Unreachable;
        }
    };
    // A refusal carries the reason in its body, so it is read either way.
    let Ok(body) = read_capped(response, MAX_ANSWER) else {
        return KeyTest::Unreachable;
    };
    let Ok(answer) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return KeyTest::Unreachable;
    };
    if answer.get("status").and_then(|status| status.as_str()) == Some("ok") {
        return KeyTest::Works;
    }
    let message = answer
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(|message| message.as_str())
        .unwrap_or_default()
        .to_lowercase();
    if message.contains("api key") || message.contains("apikey") {
        KeyTest::Refused
    } else {
        // It complained about something other than the key, which means the
        // key itself got through.
        tracing::debug!("AcoustID answered about the request, not the key");
        KeyTest::Works
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_agent_says_who_is_calling() {
        let agent = user_agent();
        assert!(agent.starts_with("Onsa/"), "{agent}");
        assert!(agent.contains(env!("CARGO_PKG_VERSION")));
        assert!(agent.contains("github.com"), "a way to get in touch");
    }

    #[test]
    fn a_client_can_be_built() {
        assert!(client().is_some(), "with rustls there is nothing to find");
    }

    #[test]
    fn every_failure_has_something_to_say() {
        for error in [NetError::Unreachable, NetError::TooLarge] {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn a_rate_limit_holds_requests_apart() {
        let limit = RateLimit::new(Duration::from_millis(120));
        let started = std::time::Instant::now();
        for _ in 0..3 {
            limit.wait();
        }
        // The first goes at once; the other two wait their turn.
        let taken = started.elapsed();
        assert!(
            taken >= Duration::from_millis(240),
            "three requests took only {taken:?}"
        );
        assert!(taken < Duration::from_millis(900), "and not much longer");
    }

    #[test]
    fn the_services_are_held_to_what_they_ask_for() {
        // MusicBrainz asks for one a second; AcoustID for three.
        assert_eq!(MUSICBRAINZ.every, Duration::from_millis(1000));
        assert!(
            ACOUSTID.every >= Duration::from_millis(334),
            "three a second"
        );
    }

    #[test]
    fn a_service_can_only_be_moved_to_this_machine() {
        for here in [
            "http://127.0.0.1:9330",
            "http://localhost:9330/v2",
            "https://127.0.0.1:9330",
        ] {
            assert!(loopback(here), "{here}");
        }
        for elsewhere in [
            "https://api.acoustid.example.com",
            "http://127.0.0.1.example.com",
            "http://evil.test/127.0.0.1",
            "file:///etc/passwd",
            "not a url",
            "",
        ] {
            assert!(!loopback(elsewhere), "{elsewhere}");
        }
    }

    #[test]
    fn a_service_that_was_not_moved_is_where_it_always_was() {
        // The variables are not set in a plain test run, which is the case
        // that matters: the real addresses are the default.
        for service in [Service::AcoustId, Service::MusicBrainz, Service::CoverArt] {
            if std::env::var(service.moved_by()).is_ok() {
                continue;
            }
            assert_eq!(service.base(), service.home());
            assert!(service.base().starts_with("https://"));
        }
    }

    #[test]
    fn the_cover_archive_is_counted_against_musicbrainz() {
        // It is the same house, and it asks for the same one a second.
        assert!(std::ptr::eq(
            Service::CoverArt.limit(),
            Service::MusicBrainz.limit()
        ));
    }

    #[test]
    fn an_error_is_logged_without_the_url_it_came_from() {
        // A URL carries the API key in its query, so it never reaches the
        // log. The client refuses a scheme it does not speak, which is the
        // cheapest way to get a real error with a URL attached.
        let client = client().expect("a client");
        let error = client
            .get("ftp://example.invalid/?client=SECRETKEY")
            .send()
            .expect_err("reqwest speaks no ftp");
        let said = tidy(&error);
        assert!(!said.contains("SECRETKEY"), "{said}");
        assert!(said.contains("<url>"), "{said}");
    }
}
