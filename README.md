# Onsa

Onsa (音叉, tuning fork) is a desktop music player for Windows and Linux, built with Tauri 2, Rust and SvelteKit. It looks like an instrument panel: calm, tidy, and honest about what is happening to the audio signal.

The project is in its first milestone. The full specification lives in [`docs/SPEC.md`](docs/SPEC.md), progress in [`docs/PROGRESS.md`](docs/PROGRESS.md), and decisions taken along the way in [`docs/DECISIONS.md`](docs/DECISIONS.md).

## Layout

```
crates/onsa-audio        audio engine (no dependency on other Onsa crates)
crates/onsa-library      database, scanner, tags, playlists, search
crates/onsa-downloader   external program management and download runner
crates/onsa-lyrics       LRC parser, lyrics sources, cache
crates/onsa-scrobble     Last.fm scrobbling
crates/onsa-cli          command line front end for the audio engine
src-tauri/               application shell: state, commands, OS integration
ui/                      SvelteKit interface
themes/                  built-in themes
```

## Requirements

All platforms:

- Rust stable with `rustfmt` and `clippy` (`rustup component add rustfmt clippy`)
- Node.js 20 or newer, with npm

Windows 10/11:

- Microsoft C++ Build Tools (the "Desktop development with C++" workload)
- WebView2 Runtime (preinstalled on Windows 11)

Linux (Debian/Ubuntu package names):

```sh
sudo apt install build-essential curl wget file pkg-config libssl-dev \
  libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev \
  libasound2-dev
```

`libasound2-dev` is the ALSA development package the audio output needs. On PipeWire or PulseAudio systems, audio goes through their ALSA compatibility layer.

## Running

```sh
cd ui
npm ci
cd ../src-tauri
npx --prefix ../ui tauri dev
```

`tauri dev` starts the interface dev server and opens the window. If the Tauri CLI is installed through Cargo (`cargo install tauri-cli --version "^2"`), `cargo tauri dev` from `src-tauri/` works as well.

Set `ONSA_LOG` (for example `ONSA_LOG=debug`) to change the log level.

## Checks

The same checks CI runs, on your own machine:

```sh
scripts/check.sh                                          # Linux
powershell -ExecutionPolicy Bypass -File scripts\check.ps1  # Windows
```

They run `svelte-check`, build the interface, then `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.

## Last.fm keys

The Last.fm API key and secret are read at build time from `ONSA_LASTFM_API_KEY` and `ONSA_LASTFM_API_SECRET` and are never committed. In an open source application the secret cannot truly be kept secret, so users can enter their own key in the settings.

## License

Not decided yet.
