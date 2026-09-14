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

/// Where AcoustID answers.
const ACOUSTID_LOOKUP: &str = "https://api.acoustid.org/v2/lookup";
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
    let sent = client
        .get(ACOUSTID_LOOKUP)
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
