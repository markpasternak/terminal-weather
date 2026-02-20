# terminal weather

[![CI](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml/badge.svg)](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/b1cd0d1c4dbb4fc2aaed5852536d50b0)](https://app.codacy.com/gh/markpasternak/terminal-weather/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![Release](https://img.shields.io/github/v/release/markpasternak/terminal-weather?sort=semver)](https://github.com/markpasternak/terminal-weather/releases)
[![Homebrew](https://img.shields.io/badge/homebrew-tap-FBB040?logo=homebrew&logoColor=white)](https://github.com/markpasternak/homebrew-tap)
[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)](LICENSE)
[![Rust Edition](https://img.shields.io/badge/rust-2024-edition?logo=rust)](https://www.rust-lang.org/)

`terminal weather` is an animated, terminal-first weather dashboard with resilient live data, rich theming, and dense but readable forecasting views.

![terminal weather demo](assets/screenshots/demo.gif)

## Features
- Responsive 3-panel layout:
  - `Current` hero panel with live metrics + visual scene
  - `Hourly` panel with adaptive rows, horizontal scrolling, and view modes (`table`, `hybrid`, `chart`)
  - `7-Day` forecast with range bars and weekly summaries
- Hero visuals (settings + CLI selectable):
  - `Atmos Canvas`: weather-driven terrain/sky scene
  - `Gauge Cluster`: instrument-style telemetry panel
  - `Sky Observatory`: sun/moon arc, weather strip, precipitation lane
  - compact/degraded canvas mode with clearer weather glyphs + labels
- Rich weather data:
  - current: temperature, feels-like, dew point, humidity, pressure + trend, visibility, wind + gust, cloud cover/layers, UV, sunrise/sunset
  - hourly: time, weather, temp, precipitation, gust, visibility, cloud, pressure, RH (rows scale with available space)
  - daily: min/max spans, precip totals, gust maxima, daylight/sunshine and weekly rollups
- Day/night-aware clear-sky rendering (sun by day, moon by night in current/hourly views)
- IP-based geolocation: auto-detects your city when launched without arguments
- Weather alerts banner: highlights notable conditions (high wind, UV, freezing rain, heavy precip, low visibility, extreme temps, thunderstorms)
- Hourly cursor: `←`/`→` moves a visible cursor across hours with date boundary labels
- Non-interactive `--one-shot` mode for scripting and shell prompts
- Deterministic location resolution and ambiguity handling
- Live freshness semantics (`fresh`, `stale`, `offline`) with retry backoff and manual refresh
- Particle ambience (rain/snow/fog/thunder) with motion and flash accessibility controls
- Contrast-hardened themes across TrueColor, xterm-256, and 16-color terminals
- Persistent settings + recent city history (including clear-all from the city picker)
- Built-in demo mode for deterministic showcases

## What's New in v0.4.0
- IP geolocation: no city argument → auto-detects your location via IP
- Weather alerts banner between Current and Hourly panels when notable conditions are detected
- `--one-shot` mode: print a weather snapshot to stdout and exit (works in pipes, scripts, cron)
- Hourly cursor with `←`/`→` navigation, visible highlight, and auto-scroll
- Date labels in the hourly panel title and a Date row at day boundaries when scrolling
- Hourly panel title now shows the date range of visible hours

## What's New in v0.3.0
- Hourly view modes you can switch live with `v`: `table`, `hybrid`, `chart`
- New `Hybrid` hourly mode combining:
  - temp/precip timeline strip
  - daypart cards (`Morning`, `Noon`, `Evening`, `Night`) with date chips
- New `Chart` hourly mode with expanded trend strip + compact metric line
- Better discoverability and recovery:
  - `?` / `F1` help overlay
  - persistent key legend footer
  - `Ctrl+L` full redraw recovery
- Explicit and standards-aligned color behavior:
  - `--color auto|always|never`
  - `--no-color` alias
  - `NO_COLOR` handling in auto mode
- Non-interactive runtime guard: dashboard mode fails fast outside an interactive TTY

## Prerequisites
- Rust stable (`rustup`, `cargo`, `rustc`)
- UTF-8 capable terminal (TrueColor recommended)
- Network access to Open-Meteo APIs
- Network access to IP lookup (`https://ipapi.co/json/`) when using auto-detect location

For development/static analysis:
- `jq` (JSON processing for metric gates)
- `rust-code-analysis-cli` `0.0.25`

## Install (Homebrew)
```bash
brew tap markpasternak/tap
brew install markpasternak/tap/terminal-weather
```

## Build
```bash
git clone https://github.com/markpasternak/terminal-weather.git
cd terminal-weather
rustup default stable
cargo build --release
```

Release tags follow `vMAJOR.MINOR.PATCH` (e.g. `v0.4.0`).

## Run
```bash
cargo run                                          # auto-detect location via IP
cargo run -- Stockholm
cargo run -- --units fahrenheit Tokyo
cargo run -- --theme midnight-cyan --hero-visual gauge-cluster "San Diego"
cargo run -- --hero-visual sky-observatory --reduced-motion London
cargo run -- --ascii-icons --no-animation Reykjavik
cargo run -- --lat 59.3293 --lon 18.0686
cargo run -- --hourly-view hybrid "San Francisco"
cargo run -- --color never --hourly-view chart Tokyo
cargo run -- --one-shot Tokyo                      # print snapshot and exit
cargo run -- --one-shot | head -10                 # pipe-friendly
```

## Demo Mode
```bash
cargo run -- --demo
```

`--demo` clears persisted settings for that run, then automatically:
1. Opens the city picker, shows a search query, and selects cities (`New York` → `Miami` → `Sydney` → `Peking`, 5s each)
2. Opens settings, selects hero visuals, then closes settings so each preview is clearly visible (`Gauge Cluster`, `Sky Observatory`, 5s each)
3. Cycles through themes
4. Exits the app

## Create the Animated GIF
Prerequisites:
- `asciinema`
- `agg` (asciinema gif generator)
- `gifsicle`

Generate/update the README demo GIF:
```bash
./record-demo.sh
```

The script builds the release binary, records a `--demo` run, converts the cast to GIF, optimizes it, and writes `assets/screenshots/demo.gif`.

## CLI
```bash
terminal-weather [CITY]

Arguments:
  [CITY]                              City name (default: auto-detect via IP, fallback: Stockholm)

Options:
  --units <celsius|fahrenheit>        Default: celsius
  --fps <N>                           15..60 (default: 30)
  --no-animation                      Disable particle animation
  --reduced-motion                    Lower motion mode
  --no-flash                          Disable thunder flash
  --ascii-icons                       Force ASCII icons
  --emoji-icons                       Force emoji icons
  --color <auto|always|never>         Color policy (default: auto)
  --no-color                          Alias for --color never
  --hourly-view <table|hybrid|chart>  Hourly panel mode override
  --theme <auto|aurora|midnight-cyan|aubergine|hoth|monument|nord|catppuccin-mocha|mono|high-contrast|dracula|gruvbox-material-dark|kanagawa-wave|ayu-mirage|ayu-light|poimandres-storm|selenized-dark|no-clown-fiesta>
  --hero-visual <atmos-canvas|gauge-cluster|sky-observatory>
  --country-code <ISO2>               Geocode bias (e.g. SE, US)
  --lat <FLOAT>                       Direct latitude (requires --lon)
  --lon <FLOAT>                       Direct longitude (requires --lat)
  --refresh-interval <secs>           Default: 600
  --one-shot                          Print weather snapshot to stdout and exit
  --demo                              Run automated showcase and exit
  --help
  --version
```

## Keybindings
Global:
- `q` or `Esc`: quit
- `Ctrl+C`: immediate quit
- `?` or `F1`: open/close help overlay
- `Ctrl+L`: force full redraw (screen recovery)
- `r`: manual refresh
- `v`: cycle hourly view (`Table` → `Hybrid` → `Chart`)
- `s`: open/close settings
- `l`: open/close city switcher
- `f`: switch to Fahrenheit
- `c`: switch to Celsius
- `←` / `→`: move hourly cursor (auto-scrolls at edges)
- `1..5`: choose location when ambiguity selector is shown

Settings panel:
- `↑` / `↓`: move selection
- `←` / `→` or `Enter`: change selected editable setting
- `Enter` on action rows: run action (`Refresh now`, `Close`)
- `s` or `Esc`: close settings

City switcher:
- Type to search (Unicode letters supported, e.g. `Åre`)
- `Enter`: search or switch to highlighted recent city
- `↑` / `↓`: move through recent cities / clear-all row
- `1..9`: quick-switch to recent city
- `Delete`: clear all recent locations
- `Backspace`: edit query
- `Esc`: close

## Terminal Best-Practice Patterns Used
- Discoverable in-app help: `?` / `F1`
- Persistent shortcut legend footer in the main layout
- Recovery shortcut: `Ctrl+L` clears and redraws the screen
- Explicit color policy: `--color auto|always|never`, `--no-color`, plus `NO_COLOR` in auto mode

Examples from established TUIs:
- htop help conventions: [man7 htop(1)](https://man7.org/linux/man-pages/man1/htop.1.html)
- lazygit bottom-line key hints: [lazygit config docs](https://github.com/jesseduffield/lazygit/blob/master/docs/Config.md)
- k9s `?` keyboard help mnemonic: [k9s README](https://github.com/derailed/k9s)

## Persisted Settings
Saved to:
- `~/.config/terminal-weather/settings.json`
- Override config directory with `TERMINAL_WEATHER_CONFIG_DIR`

Persisted values:
- units
- theme
- motion (`full`, `reduced`, `off`)
- thunder flash on/off
- icon mode (`unicode`, `ascii`, `emoji`)
- hourly view (`table`, `hybrid`, `chart`)
- hero visual mode
- refresh interval
- recent locations

## Terminal and Color Behavior
Color capability fallback order:
1. TrueColor (`COLORTERM=truecolor` / `24bit`)
2. xterm-256 quantized
3. Basic 16-color semantic fallback

Color precedence:
1. `--no-color` or `--color never` forces Basic16
2. `--color always` ignores `NO_COLOR` and uses terminal capability detection
3. `--color auto`:
   - honors `NO_COLOR` only when set to a non-empty value
   - treats `TERM=dumb` as Basic16
   - otherwise detects via `COLORTERM` / `TERM`

Icon modes:
- Unicode (default)
- ASCII (`--ascii-icons`)
- Emoji (`--emoji-icons`)

## Troubleshooting
- API/network failures:
  - app keeps last known good weather visible
  - status transitions to `stale` / `offline` with retry backoff
  - press `r` to trigger manual refresh
- Tiny terminal:
  - below `20x10`, only resize guidance is rendered
- Icon alignment issues:
  - use `--ascii-icons`
- Coordinate mode errors:
  - `--lat` and `--lon` must be provided together
- Non-interactive shell / redirected stdout:
  - dashboard mode requires an interactive TTY
  - use `--one-shot` for non-interactive output (scripts, pipes, cron)
  - use `--help` for CLI reference

## Development
Install static-analysis tools used by local gates:
```bash
cargo install --locked rust-code-analysis-cli --version 0.0.25
# jq via package manager, e.g.:
# macOS: brew install jq
# Ubuntu: sudo apt-get install -y jq
```
Tool versions are tracked in `Cargo.toml` under `[package.metadata.dev-tools]`.

Quality gate commands (CI-enforced steps):
```bash
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic
./scripts/static-analysis-gate.sh              # cyclomatic/cognitive/MI thresholds (run in CI)
cargo check --all-targets --all-features
cargo test --all --all-features
cargo build --release
```

Additional local parity gate:
```bash
./scripts/codacy-complexity-gate.sh            # Codacy-style complexity thresholds
```

Static-analysis gate policy (`./scripts/static-analysis-gate.sh`):
- analyzes `src/` + `tests/` with `rust-code-analysis-cli`
- fails on function metrics at/over thresholds:
  - cyclomatic complexity `>= 20`
  - cognitive complexity `>= 30`
  - maintainability index (MI) `< 30`
- override thresholds locally with:
  - `TW_CYCLOMATIC_MAX`
  - `TW_COGNITIVE_MAX`
  - `TW_MI_MIN`

Codacy complexity parity (`./scripts/codacy-complexity-gate.sh`):
- analyzes `src/` + `tests/` with `rust-code-analysis-cli`
- tracks these thresholds:
  - file NLOC: medium `> 500`, critical `> 1000`
  - function NLOC: medium `> 50`, critical `> 100`
  - cyclomatic complexity: medium `> 8`, critical `> 12`
  - function parameter count: medium `> 8`, critical `> 12`
- default fail policy:
  - fail on critical violations (`TW_FAIL_ON_CRITICAL=1`)
  - report-only for medium violations (`TW_FAIL_ON_MEDIUM=0`)
- override thresholds locally with:
  - `TW_FILE_NLOC_MEDIUM_MAX`, `TW_FILE_NLOC_CRITICAL_MAX`
  - `TW_FUNCTION_NLOC_MEDIUM_MAX`, `TW_FUNCTION_NLOC_CRITICAL_MAX`
  - `TW_CYCLOMATIC_MEDIUM_MAX`, `TW_CYCLOMATIC_CRITICAL_MAX`
  - `TW_PARAM_MEDIUM_MAX`, `TW_PARAM_CRITICAL_MAX`

## Release Automation (Maintainers)
This repository uses `cargo-dist` to build release artifacts and update Homebrew formulae.

One-time setup:
1. Create the tap repo `markpasternak/homebrew-tap` with a `Formula/` directory.
2. Add a repository secret named `HOMEBREW_TAP_TOKEN` in this repo:
   - token scope: write access to `markpasternak/homebrew-tap` contents

Release:
```bash
git tag v0.4.0
git push origin v0.4.0
```

The GitHub workflow at `.github/workflows/release.yml` publishes release artifacts and updates the Homebrew formula in the tap.
There is no local release script in this repository; release publishing is tag-triggered workflow automation.

## Attribution
Weather + geocoding data: [Open-Meteo](https://open-meteo.com/)
