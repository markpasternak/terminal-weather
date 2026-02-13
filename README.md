# terminal weather

`terminal weather` is an animated, terminal-first weather dashboard with resilient live data, rich theming, and dense but readable forecasting views.

![terminal weather demo](assets/screenshots/demo.gif)

## Features
- Responsive 3-panel layout:
  - `Current` hero panel with live metrics + visual scene
  - `Hourly` strip with adaptive rows and horizontal scrolling
  - `7-Day` forecast with range bars and weekly summaries
- Hero visuals (settings + CLI selectable):
  - `Atmos Canvas`: weather-driven terrain/sky scene
  - `Gauge Cluster`: instrument-style telemetry panel
  - `Sky Observatory`: sun/moon arc, weather strip, precipitation lane
- Rich weather data:
  - current: temperature, feels-like, dew point, humidity, pressure + trend, visibility, wind + gust, cloud cover/layers, UV, sunrise/sunset
  - hourly: time, weather, temp, precipitation, gust, visibility, cloud, pressure, RH (rows scale with available space)
  - daily: min/max spans, precip totals, gust maxima, daylight/sunshine and weekly rollups
- Deterministic location resolution and ambiguity handling
- Live freshness semantics (`fresh`, `stale`, `offline`) with retry backoff and manual refresh
- Particle ambience (rain/snow/fog/thunder) with motion and flash accessibility controls
- Contrast-hardened themes across TrueColor, xterm-256, and 16-color terminals
- Persistent settings + recent city history (including clear-all from the city picker)
- Built-in demo mode for deterministic showcases

## Prerequisites
- Rust stable (`rustup`, `cargo`, `rustc`)
- UTF-8 capable terminal (TrueColor recommended)
- Network access to Open-Meteo APIs

## Install (Homebrew)
```bash
brew tap markpasternak/tap
brew install markpasternak/tap/terminal-weather
```

## Build
```bash
git clone <repo-url>
cd terminal-weather
rustup default stable
cargo build --release
```

## Run
```bash
cargo run -- Stockholm
cargo run -- --units fahrenheit Tokyo
cargo run -- --theme midnight-cyan --hero-visual gauge-cluster "San Diego"
cargo run -- --hero-visual sky-observatory --reduced-motion London
cargo run -- --ascii-icons --no-animation Reykjavik
cargo run -- --lat 59.3293 --lon 18.0686
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

## CLI
```bash
terminal-weather [CITY]

Arguments:
  [CITY]                              City name (default: Stockholm)

Options:
  --units <celsius|fahrenheit>        Default: celsius
  --fps <N>                           15..60 (default: 30)
  --no-animation                      Disable particle animation
  --reduced-motion                    Lower motion mode
  --no-flash                          Disable thunder flash
  --ascii-icons                       Force ASCII icons
  --emoji-icons                       Force emoji icons
  --theme <auto|aurora|midnight-cyan|aubergine|hoth|monument|nord|catppuccin-mocha|mono|high-contrast|dracula|gruvbox-material-dark|kanagawa-wave|ayu-mirage|ayu-light|poimandres-storm|selenized-dark|no-clown-fiesta>
  --hero-visual <atmos-canvas|gauge-cluster|sky-observatory>
  --country-code <ISO2>               Geocode bias (e.g. SE, US)
  --lat <FLOAT>                       Direct latitude (requires --lon)
  --lon <FLOAT>                       Direct longitude (requires --lat)
  --refresh-interval <secs>           Default: 600
  --demo                              Run automated showcase and exit
  --help
  --version
```

## Keybindings
Global:
- `q` or `Esc`: quit
- `Ctrl+C`: immediate quit
- `r`: manual refresh
- `s`: open/close settings
- `l`: open/close city switcher
- `f`: switch to Fahrenheit
- `c`: switch to Celsius
- `←` / `→`: scroll hourly columns
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
- hero visual mode
- refresh interval
- recent locations

## Terminal and Color Behavior
Color capability fallback order:
1. TrueColor (`COLORTERM=truecolor` / `24bit`)
2. xterm-256 quantized
3. Basic 16-color semantic fallback (also used when `NO_COLOR` is set)

Icon modes:
- Unicode (default)
- ASCII (`--ascii-icons`)
- Emoji (`--emoji-icons`)

## Screenshots
The repository no longer includes a screenshot script.

To add/update a README screenshot manually:
1. Run the app in your terminal at the desired size/theme
2. Capture your screen/window with your OS tooling
3. Save it to `assets/screenshots/app-preview.png`
4. Add a Markdown image link in this README if you want it rendered on GitHub

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

## Development
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets --all-features
cargo test --all --all-features
cargo build --release
```

## Release Automation (Maintainers)
This repository uses `cargo-dist` to build release artifacts and update Homebrew formulae.

One-time setup:
1. Create the tap repo `markpasternak/homebrew-tap` with a `Formula/` directory.
2. Add a repository secret named `HOMEBREW_TAP_TOKEN` in this repo:
   - token scope: write access to `markpasternak/homebrew-tap` contents

Release:
```bash
git tag v0.1.0
git push origin v0.1.0
```

The GitHub workflow at `.github/workflows/release.yml` will publish release artifacts and update the Homebrew formula in the tap.

## Attribution
Weather + geocoding data: [Open-Meteo](https://open-meteo.com/)
