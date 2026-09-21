//! Where the keys for the outside services come from (SPEC §14).
//!
//! A key is never written into the code, never committed, and never logged.
//! It reaches Onsa one of three ways, and the first one that has it wins:
//!
//! 1. what the listener typed into Settings, stored in their own database;
//! 2. the environment of the running application;
//! 3. the environment of the machine that built it.
//!
//! Nothing outside this module ever sees a key it did not ask for by name,
//! and the interface is only ever told **where** the key came from — never
//! what it is.

use serde::Serialize;

/// The environment variable holding an AcoustID application key.
pub const ACOUSTID_ENV: &str = "ONSA_ACOUSTID_API_KEY";

/// Whether a build-time variable holds nothing worth calling a key.
///
/// An unset variable and an empty one mean the same thing here, so a build
/// that clears a variable rather than removing it counts as keyless.
// Its callers are the guard below and the tests: both are read at compile
// time, which dead-code analysis does not count as use.
#[allow(dead_code)]
const fn absent(value: Option<&str>) -> bool {
    match value {
        None => true,
        Some(value) => value.is_empty(),
    }
}

/// A build meant for somebody else carries nobody's key.
///
/// Setting `ONSA_KEYLESS` at build time makes this refuse to compile if any
/// key would be baked in. The evidence is then the binary's own existence:
/// it could not have been built with a key in it, rather than merely having
/// been built on a machine where the variable happened to be unset.
///
/// Builds for the author's own use set nothing and keep working as before.
const _KEYLESS_BUILD_CARRIES_NO_KEY: () = {
    if !absent(option_env!("ONSA_KEYLESS")) {
        assert!(
            absent(option_env!("ONSA_ACOUSTID_API_KEY")),
            "this build was told to carry no keys, but an AcoustID key was in the build environment"
        );
        assert!(
            absent(option_env!("ONSA_LASTFM_API_KEY")),
            "this build was told to carry no keys, but a Last.fm key was in the build environment"
        );
        assert!(
            absent(option_env!("ONSA_LASTFM_API_SECRET")),
            "this build was told to carry no keys, but a Last.fm secret was in the build environment"
        );
    }
};

/// Where a key came from. This, and not the key, is what gets reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KeySource {
    /// Typed into Settings by the listener.
    Settings,
    /// Found in the environment of the running application.
    Environment,
    /// Built into this copy of Onsa from the build environment.
    Build,
    /// There is no key.
    None,
}

/// A key and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// The key itself. Never logged, never sent to the interface.
    pub value: String,
    /// Where it was found.
    pub source: KeySource,
}

/// What the interface is allowed to know about a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyStatus {
    /// Whether there is a key at all.
    pub present: bool,
    /// Where it came from, so the listener can tell which one is in use.
    pub source: KeySource,
}

impl From<Option<&Key>> for KeyStatus {
    fn from(key: Option<&Key>) -> Self {
        match key {
            Some(key) => KeyStatus {
                present: true,
                source: key.source,
            },
            None => KeyStatus {
                present: false,
                source: KeySource::None,
            },
        }
    }
}

/// A key from the setting, the running environment, or the build, in that
/// order. Blank values count as absent, so clearing the field falls back to
/// whatever was there before it.
pub fn resolve(
    stored: Option<&str>,
    env_name: &str,
    built_in: Option<&'static str>,
) -> Option<Key> {
    let tidy = |value: &str| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    };
    if let Some(value) = stored.and_then(tidy) {
        return Some(Key {
            value,
            source: KeySource::Settings,
        });
    }
    if let Some(value) = std::env::var(env_name).ok().as_deref().and_then(tidy) {
        return Some(Key {
            value,
            source: KeySource::Environment,
        });
    }
    built_in.and_then(tidy).map(|value| Key {
        value,
        source: KeySource::Build,
    })
}

/// The AcoustID application key, if Onsa has one.
///
/// This is the **application** key from <https://acoustid.org/new-application>,
/// the `client` parameter of the web service — not the personal key shown on
/// the account page, which only signs fingerprint submissions.
pub fn acoustid(stored: Option<&str>) -> Option<Key> {
    resolve(stored, ACOUSTID_ENV, option_env!("ONSA_ACOUSTID_API_KEY"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_and_an_empty_string_both_count_as_no_key() {
        assert!(absent(None));
        assert!(absent(Some("")));
        assert!(!absent(Some("a-key")));
    }

    /// The same thing the build-time guard says, said by something that
    /// runs: on a build told to carry no keys, none of them is there.
    #[test]
    fn a_keyless_build_has_no_key_built_in() {
        if absent(option_env!("ONSA_KEYLESS")) {
            // An ordinary build, which may carry the author's own key.
            return;
        }
        assert!(absent(option_env!("ONSA_ACOUSTID_API_KEY")));
        assert!(absent(option_env!("ONSA_LASTFM_API_KEY")));
        assert!(absent(option_env!("ONSA_LASTFM_API_SECRET")));
    }

    /// A name no real environment would have, so the tests do not depend on
    /// what happens to be set on the machine running them.
    const NOWHERE: &str = "ONSA_TEST_KEY_THAT_IS_NOT_SET";

    #[test]
    fn the_setting_comes_first() {
        let key =
            resolve(Some(" from-settings "), NOWHERE, Some("from-build")).expect("a key was given");
        assert_eq!(key.value, "from-settings", "and it is trimmed");
        assert_eq!(key.source, KeySource::Settings);
    }

    #[test]
    fn a_blank_setting_is_no_setting() {
        let key = resolve(Some("   "), NOWHERE, Some("from-build")).expect("the build has one");
        assert_eq!(key.source, KeySource::Build);
        assert_eq!(key.value, "from-build");
    }

    #[test]
    fn without_a_key_anywhere_there_is_none() {
        assert_eq!(resolve(None, NOWHERE, None), None);
        assert_eq!(resolve(Some(""), NOWHERE, Some("  ")), None);
    }

    #[test]
    fn the_status_says_where_but_never_what() {
        let key = resolve(Some("secret"), NOWHERE, None);
        let status = KeyStatus::from(key.as_ref());
        assert!(status.present);
        assert_eq!(status.source, KeySource::Settings);
        let json = serde_json::to_string(&status).expect("serialisable");
        assert!(!json.contains("secret"), "{json}");
        assert_eq!(json, r#"{"present":true,"source":"settings"}"#);

        let empty = KeyStatus::from(None);
        assert!(!empty.present);
        assert_eq!(empty.source, KeySource::None);
    }
}
