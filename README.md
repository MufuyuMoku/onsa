# Onsa

Onsa (音叉, tuning fork) is a desktop music player for Windows and Linux, built with Tauri 2, Rust and SvelteKit. It looks like an instrument panel: calm, tidy, and honest about what is happening to the audio signal.

The project is in early development. The audio engine, the DSP chain, the library and the interface are in place: choosing a folder, scanning, browsing, playing, setting the EQ and changing themes all work without the command line (see `docs/UJI-M4.md`). The full specification lives in [`docs/SPEC.md`](docs/SPEC.md), progress in [`docs/PROGRESS.md`](docs/PROGRESS.md), and decisions taken along the way in [`docs/DECISIONS.md`](docs/DECISIONS.md).

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
ui/static/fonts/         the themes' fonts, bundled with their licences
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

## Audio engine from the command line

`onsa-cli` drives the audio engine without the interface:

```sh
cargo run -p onsa-cli -- devices
cargo run -p onsa-cli -- play song.flac --start 30
cargo run -p onsa-cli -- queue a.flac b.flac c.flac
cargo run -p onsa-cli -- queue --list album.txt --crossfade 4
cargo run -p onsa-cli -- render-wav a.flac b.flac -o out.wav --rate 48000
```

The DSP chain is set with options on `play`, `queue` and `render-wav`:

```sh
cargo run -p onsa-cli -- play song.flac --eq 3,2,0,0,0,0,0,1,2,3 --auto-preamp
cargo run -p onsa-cli -- play song.flac --eq-file ParametricEQ.txt --meter
cargo run -p onsa-cli -- queue *.flac --replaygain auto --rg-preamp 3
cargo run -p onsa-cli -- eq-check ParametricEQ.txt
```

`--eq` takes ten gains in dB for 31 Hz to 16 kHz; `--eq-file` reads an
Equalizer APO / AutoEQ preset (its own `Preamp:` line is used unless
`--preamp` is given). Other options: `--no-limiter`, `--limiter-release`,
`--volume`, `--rg-fallback`, `--no-dither`. `eq-check` shows how a preset
is read, which lines are skipped, and the auto preamp.

While `play` or `queue` runs, type a line and press Enter:

| Line | Effect |
|---|---|
| empty or `p` | pause / resume |
| `n`, `b` | next, previous |
| `s 90`, `+10`, `-10` | seek to 1:30, skip ten seconds |
| `eq 6 +9` | set graphic band 6 (1 kHz) to +9 dB |
| `eq on`, `eq off`, `eq flat` | EQ on, off, or flat graphic |
| `pre -3`, `auto` | manual preamp, or toggle auto preamp |
| `vol -6` | volume in dB |
| `lim on`, `lim off` | limiter |
| `rg off`, `rg track`, `rg album`, `rg auto` | ReplayGain mode |
| `m` | peak and RMS meters, limiter state, spectral centroid |
| `q` | quit |

Run with `--help` for every option (device, fixed sample rate, crossfade
curve, resampler quality, buffer size).

## Library scan from the command line

The library crate has an example that scans a folder twice, into a fresh
database in the temp folder, and compares the first scan with the unchanged
rescan. The music folder is only read.

```sh
cargo run -p onsa-library --release --example scan -- ~/Music
```

## Portable build

`tauri build --no-bundle` produces one executable that runs without being installed:

```sh
cd src-tauri
npx --prefix ../ui tauri build --no-bundle
```

It lands in `target/release/` (`onsa.exe` on Windows). Test builds handed to the project owner live in `dist-test/`, which is never committed; `docs/UJI-M4.md` is the guide that goes with them. Installers arrive in M12.

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
