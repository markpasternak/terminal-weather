# terminal weather

[![CI](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml/badge.svg)](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/b1cd0d1c4dbb4fc2aaed5852536d50b0)](https://app.codacy.com/gh/markpasternak/terminal-weather/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![Release](https://img.shields.io/github/v/release/markpasternak/terminal-weather?sort=semver)](https://github.com/markpasternak/terminal-weather/releases)
[![Homebrew](https://img.shields.io/badge/homebrew-tap-FBB040?logo=homebrew&logoColor=white)](https://github.com/markpasternak/homebrew-tap)
[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)](LICENSE)

Weather, in your terminal. Beautiful by default, scriptable by design.

![terminal weather demo](assets/screenshots/demo.gif)

- **3 panels** — Current hero, Hourly (Table / Hybrid / Chart), 7-Day forecast
- **Weather-native motion** — cinematic, standard, reduced, and off modes with condition-aware scenes
- **Auto-location** — detects your city from IP when you launch without arguments
- **18 themes** — auto-selected by terminal capability; TrueColor to 16-color fallback
- **`--one-shot`** — pipe a weather snapshot to stdout; works in scripts, cron, and shell prompts

---

## Install

**Homebrew (recommended):**
```bash
brew tap markpasternak/tap
brew install markpasternak/tap/terminal-weather
```

**Build from source** (requires Rust stable):
```bash
git clone https://github.com/markpasternak/terminal-weather.git
cd terminal-weather
cargo build --release
# binary: target/release/terminal-weather
```

Requires a UTF-8 capable terminal. TrueColor recommended for full theme fidelity.

---

## Usage

```bash
terminal-weather                                         # auto-detect location via IP
terminal-weather Stockholm
terminal-weather --units fahrenheit Tokyo
terminal-weather --theme midnight-cyan --hero-visual gauge-cluster "San Diego"
terminal-weather --hero-visual sky-observatory --motion cinematic London
terminal-weather --motion reduced London
terminal-weather --ascii-icons --no-animation Reykjavik
terminal-weather --nerd-font Stockholm                   # use Nerd Font weather icons
terminal-weather --lat 59.3293 --lon 18.0686
terminal-weather --one-shot Tokyo                        # snapshot to stdout and exit
terminal-weather --one-shot | head -10                   # pipe-friendly
terminal-weather --demo                                  # scripted showcase
```

### CLI options

```
terminal-weather [OPTIONS] [CITY]

Arguments:
  [CITY]  City name (default: auto-detect via IP, fallback: Stockholm)

Options:
  --units <celsius|fahrenheit>
  --fps <N>                             15..60 (default: 30)
  --motion <cinematic|standard|reduced|off>
  --no-animation                        Alias for --motion off
  --reduced-motion                      Alias for --motion reduced
  --no-flash                            Disable thunder flash
  --ascii-icons                         Force ASCII icons
  --emoji-icons                         Force emoji icons
  --nerd-font                           Use Nerd Font weather icons
  --color <auto|always|never>           Color policy (default: auto)
  --no-color                            Alias for --color never
  --hourly-view <table|hybrid|chart>    Hourly panel mode
  --theme <THEME>                       Theme (default: auto)
  --hero-visual <atmos-canvas|gauge-cluster|sky-observatory>
  --country-code <ISO2>                 Geocode bias (e.g. SE, US)
  --lat <FLOAT>                         Direct latitude (requires --lon)
  --lon <FLOAT>                         Direct longitude (requires --lat)
  --forecast-url <URL>                  Override forecast API base URL
  --air-quality-url <URL>               Override air-quality API base URL
  --refresh-interval <secs>             Default: 600
  --one-shot                            Print snapshot to stdout and exit
  --demo                                Run scripted showcase and exit
  --help
  --version
```

Available themes: `auto` `aurora` `midnight-cyan` `aubergine` `hoth` `monument` `nord` `catppuccin-mocha` `mono` `high-contrast` `dracula` `gruvbox-material-dark` `kanagawa-wave` `ayu-mirage` `ayu-light` `poimandres-storm` `selenized-dark` `no-clown-fiesta`

### Keybindings

**Global**

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Ctrl+C` | Immediate quit |
| `?` / `F1` | Help overlay |
| `Ctrl+L` | Force full redraw |
| `r` | Manual refresh |
| `v` | Cycle hourly view (Table → Hybrid → Chart) |
| `s` | Settings panel |
| `l` | City switcher |
| `f` / `c` | Switch to Fahrenheit / Celsius |
| `←` / `→` | Move hourly cursor |
| `Tab` / `Shift+Tab` | Cycle panel focus (Current / Hourly / 7-Day) |
| `:` | Open command bar (when enabled in Settings) |
| `1..5` | Select ambiguous location |

**Settings panel:** `↑`/`↓` navigate in visual order · `←`/`→` or `Enter` change value · `s`/`Esc` close

**City switcher:** type to search · `Enter` confirm · `↑`/`↓` browse recents · `1..9` quick-switch · `Delete` clear all · `Esc` close

**Command bar:** `:refresh` · `:quit` · `:units c|f` · `:view table|hybrid|chart` · `:theme <name>` · `:city <name>`

---

## Configuration

Settings persist to `~/.config/terminal-weather/settings.json`. Override the directory with `TERMINAL_WEATHER_CONFIG_DIR` (legacy `ATMOS_TUI_CONFIG_DIR` is also supported).

Persisted values: units, theme, motion (`cinematic`/`standard`/`reduced`/`off`), thunder flash, icon mode (`unicode`/`ascii`/`emoji`/`nerd-font`), inline hints, command bar enabled/disabled, hourly view, hero visual, refresh interval, recent locations, and update-check metadata (`last_update_check_unix`, `last_seen_latest_version`).

Color detection falls back TrueColor → xterm-256 → 16-color based on `COLORTERM` and `TERM`. `NO_COLOR` is honored in `auto` mode.

API endpoint overrides:

- `TERMINAL_WEATHER_FORECAST_URL` sets the forecast endpoint
- `TERMINAL_WEATHER_AIR_QUALITY_URL` sets the air-quality endpoint
- `--forecast-url` / `--air-quality-url` override env vars for the current run

Update-check controls:

- `TERMINAL_WEATHER_DISABLE_UPDATE_CHECK=1` disables background Homebrew update checks
- `TERMINAL_WEATHER_UPDATE_FORMULA_URL=<url>` overrides the Homebrew formula source (advanced/testing)

Precedence is: CLI flag → environment variable → built-in default URL.

Custom endpoints must remain Open-Meteo compatible (same query parameters and response shape).

---

## Privacy

When launched without a city argument, `terminal-weather` sends your IP address to [ipapi.co](https://ipapi.co/) to determine your location. Pass a city name or `--lat`/`--lon` to skip this lookup entirely.

Weather and city search data come from [Open-Meteo](https://open-meteo.com/) (your coordinates and search string are sent). Reverse geocoding for coordinate-based locations (for example `--lat/--lon` and coordinate-only history entries) uses [Nominatim](https://nominatim.openstreetmap.org/) (your coordinates are sent). The IP-based auto-location lookup uses [ipapi.co](https://ipapi.co/). Startup may also request the Homebrew formula file from `raw.githubusercontent.com` to detect a newer release; this check is throttled to once per 24 hours, has a short timeout, and can be disabled with `TERMINAL_WEATHER_DISABLE_UPDATE_CHECK=1`. No account or API key is required, and this app does not persist data outside local settings/history on your machine.

---

## What's New in v0.7.0

- **Cinematic weather motion** — the app now uses a dedicated motion system with condition-aware choreography across the hero, landmark scenes, loading states, and panel transitions.
- **Motion tiers** — `--motion cinematic|standard|reduced|off` is now the primary control, while `--reduced-motion` and `--no-animation` remain as compatibility aliases.
- **Expanded hourly chart** — chart mode now uses the pane height more effectively with a real temperature plot, a compact precipitation lane, and clearer scale placement.
- **Settings navigation alignment** — keyboard navigation in Settings now follows the same top-to-bottom order shown on screen.
- **Silent Homebrew update checks** — startup now performs a background, timeout-bounded check against the Homebrew tap formula to detect newer releases.
- **Quiet-by-default UX** — no messages are shown for failures or no-update states; a subtle footer hint appears only when a newer version is available.
- **24-hour check cadence** — update-check metadata is persisted in settings and throttled to once per day.
- **Legacy settings compatibility** — new update metadata fields are backward-compatible with existing `settings.json` files.
- **Code quality hardening** — complexity and file-length audit warnings were eliminated while keeping all required and recommended local gates green.

---

## Contributing

Bug reports and pull requests are welcome. [Open an issue](https://github.com/markpasternak/terminal-weather/issues) to discuss a bug or feature before sending a PR.

To contribute code:

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Run `./scripts/check.sh` — CI must pass before a PR can merge
4. Open a pull request against `main`

The maintainer reviews and merges all PRs. Direct pushes to `main` are restricted.

### Local Quality Gate

```bash
./scripts/check.sh            # structured summary (default)
./scripts/check.sh --verbose   # full tool output
```

The script checks formatting, linting (including pedantic), complexity, duplication, tests, release build, and coverage — then prints a final report classifying each check as **required** or **recommended**.

Checks requiring optional tooling are auto-skipped when the tool is missing. Install everything for full coverage:

```bash
cargo install --locked rust-code-analysis-cli --version 0.0.25
cargo install --locked cargo-dupes --version 0.2.1
cargo install --locked cargo-llvm-cov
# install jq via your package manager (brew install jq / apt install jq)
```

---

## Attribution & License

Weather and forward geocoding data: [Open-Meteo](https://open-meteo.com/)

Reverse geocoding: [Nominatim](https://nominatim.openstreetmap.org/) powered by OpenStreetMap data.  
OpenStreetMap data attribution: [Data © OpenStreetMap contributors](https://www.openstreetmap.org/copyright) (ODbL 1.0).

Licensed under [GPL-3.0-only](LICENSE).
