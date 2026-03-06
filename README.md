# terminal weather

[![CI](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml/badge.svg)](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/b1cd0d1c4dbb4fc2aaed5852536d50b0)](https://app.codacy.com/gh/markpasternak/terminal-weather/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![Release](https://img.shields.io/github/v/release/markpasternak/terminal-weather?sort=semver)](https://github.com/markpasternak/terminal-weather/releases)
[![Homebrew](https://img.shields.io/badge/homebrew-tap-FBB040?logo=homebrew&logoColor=white)](https://github.com/markpasternak/homebrew-tap)
[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)](LICENSE)

Weather, in your terminal. Animated by default, scriptable when you need text, and packaged for a quick Homebrew install.

![terminal weather demo](assets/screenshots/demo.gif)

- **Weather-native motion**: cinematic, standard, reduced, and off presets with condition-aware hero, loading, and landmark scenes
- **Three forecast panels**: current conditions, hourly detail in table or chart form, and a seven-day outlook
- **Multiple hero visuals**: `atmos-canvas`, `gauge-cluster`, and `sky-observatory`
- **Script-friendly mode**: `--one-shot` prints a clean forecast snapshot to stdout and exits
- **Location UX that fits the terminal**: auto-detect on interactive launch, city picker, recent locations, and command bar support
- **Terminal-aware themes**: 18 themes with TrueColor, 256-color, and 16-color fallback
- **Quiet update awareness**: subtle in-app Homebrew upgrade hint when a newer release is available

---

## Install

**Homebrew (recommended):**
```bash
brew tap markpasternak/tap
brew install markpasternak/tap/terminal-weather
```

Upgrade later with:
```bash
brew upgrade markpasternak/tap/terminal-weather
```

**Build from source** (requires Rust stable):
```bash
git clone https://github.com/markpasternak/terminal-weather.git
cd terminal-weather
cargo build --release
# binary: target/release/terminal-weather
```

Requires a UTF-8 terminal. TrueColor gives the best result, but the app falls back cleanly on more limited terminals.

---

## Usage

### Interactive

```bash
terminal-weather
terminal-weather Stockholm
terminal-weather --units fahrenheit Tokyo
terminal-weather --lat 59.3293 --lon 18.0686
```

If you launch the full TUI without a city, the app first tries GeoIP auto-location and falls back to Stockholm if that lookup fails.

### Customization

```bash
terminal-weather --theme midnight-cyan --hero-visual gauge-cluster "San Diego"
terminal-weather --hero-visual sky-observatory --motion cinematic London
terminal-weather --motion reduced London
terminal-weather --ascii-icons --no-animation Reykjavik
terminal-weather --nerd-font Stockholm
terminal-weather --demo
```

### Scripts And Automation

```bash
terminal-weather --one-shot Tokyo
terminal-weather --one-shot "San Francisco"
terminal-weather --one-shot Tokyo | head -10
```

`--one-shot` is non-interactive. If you omit the city there, it resolves Stockholm rather than doing GeoIP auto-location.

### CLI options

```text
terminal-weather [OPTIONS] [CITY]

Arguments:
  [CITY]  City name. Interactive mode auto-detects via IP if omitted, then falls back to Stockholm. --one-shot falls back to Stockholm directly.

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
| `v` | Cycle hourly view (Table -> Hybrid -> Chart) |
| `s` | Settings panel |
| `l` | City switcher |
| `f` / `c` | Switch to Fahrenheit / Celsius |
| `←` / `→` | Move hourly cursor |
| `Tab` / `Shift+Tab` | Cycle panel focus (Current / Hourly / 7-Day) |
| `:` | Open command bar (when enabled in Settings) |
| `1..5` | Select ambiguous location |

**Settings panel:** `↑`/`↓` navigate in visual order, `←`/`→` or `Enter` change a value, `s` or `Esc` close

**City switcher:** type to search, `Enter` confirm, `↑`/`↓` browse recents, `1..9` quick-switch, `Delete` clear all, `Esc` close

Recent-location navigation wraps around, and searches keep the picker open so you can refine input without reopening it.

**Command bar:** `:refresh`, `:quit`, `:units c|f`, `:view table|hybrid|chart`, `:theme <name>`, `:city <name>`

If a fetch fails, the error state now shows direct keyboard actions so recovery does not require guesswork.

---

## Configuration

Settings persist to `~/.config/terminal-weather/settings.json`. Override the directory with `TERMINAL_WEATHER_CONFIG_DIR`. The legacy `ATMOS_TUI_CONFIG_DIR` name still works for compatibility.

Persisted values include units, theme, motion mode, thunder flash, icon mode, inline hints, command bar enabled/disabled, hourly view, hero visual, refresh interval, recent locations, and update-check metadata (`last_update_check_unix`, `last_seen_latest_version`).

Color detection falls back from TrueColor to xterm-256 to 16-color based on `COLORTERM` and `TERM`. `NO_COLOR` is honored when color mode is `auto`.

API endpoint overrides:

- `TERMINAL_WEATHER_FORECAST_URL` sets the forecast endpoint
- `TERMINAL_WEATHER_AIR_QUALITY_URL` sets the air-quality endpoint
- `--forecast-url` and `--air-quality-url` override those environment variables for the current run

Update-check controls:

- `TERMINAL_WEATHER_DISABLE_UPDATE_CHECK=1` disables the background Homebrew update check
- `TERMINAL_WEATHER_UPDATE_FORMULA_URL=<url>` overrides the Homebrew formula source for testing

Precedence is: CLI flag -> environment variable -> built-in default URL.

Custom endpoints must stay Open-Meteo compatible with the same query parameters and response shape.

---

## Privacy

When you launch the interactive app without a city, `terminal-weather` may send your IP address to [ipapi.co](https://ipapi.co/) to estimate your location. Pass a city name or `--lat` and `--lon` to skip that lookup entirely.

Forecast and forward-geocoding requests go to [Open-Meteo](https://open-meteo.com/). Reverse geocoding for coordinate-based locations goes to [Nominatim](https://nominatim.openstreetmap.org/). The app also may fetch the Homebrew formula from `raw.githubusercontent.com` to check whether a newer release exists. That update check is throttled to once every 24 hours, has a short timeout, ignores quiet failure cases, and can be disabled with `TERMINAL_WEATHER_DISABLE_UPDATE_CHECK=1`.

Remote text from GeoIP and geocoding responses is sanitized before it reaches the UI, and network requests use bounded timeouts and payload limits. No account or API key is required. Outside local settings and recent-location history on your machine, the app does not persist your data.

---

## What's New in v0.8.0

- **Cinematic motion across the app**: hero visuals, loading states, landmark scenes, and panel transitions now move with weather-aware choreography.
- **A stronger hourly chart**: chart mode uses space better, reads more clearly, and carries temperature and precipitation detail more cleanly.
- **Faster recovery and navigation**: the city picker is more forgiving, recent-location shortcuts are easier to use, and error states show actionable keys.
- **Safer network behavior**: update checks and API calls now fail quietly, time out aggressively, and reject oversized or unsafe remote payloads.
- **Better Homebrew release awareness**: the app can quietly detect a newer tap release and surface a subtle upgrade hint in the UI.

For the full release narrative, see [CHANGELOG.md](CHANGELOG.md).

---

## Contributing

Bug reports and pull requests are welcome. [Open an issue](https://github.com/markpasternak/terminal-weather/issues) before sending a PR for a larger bug or feature change.

To contribute code:

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Run `./scripts/check.sh`
4. Open a pull request against `main`

The maintainer reviews and merges all PRs. Direct pushes to `main` are restricted.

### Local Quality Gate

```bash
./scripts/check.sh             # structured summary (default)
./scripts/check.sh --verbose   # full tool output
```

The script checks formatting, linting, pedantic Clippy, complexity, duplication, tests, release build, and coverage, then prints a final report that labels each check as required or recommended.

Checks that depend on optional tooling are auto-skipped when the tool is missing. Install everything for full coverage:

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
