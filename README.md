# terminal weather

[![CI](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml/badge.svg)](https://github.com/markpasternak/terminal-weather/actions/workflows/ci.yml)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/b1cd0d1c4dbb4fc2aaed5852536d50b0)](https://app.codacy.com/gh/markpasternak/terminal-weather/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![Release](https://img.shields.io/github/v/release/markpasternak/terminal-weather?sort=semver)](https://github.com/markpasternak/terminal-weather/releases)
[![Homebrew](https://img.shields.io/badge/homebrew-tap-FBB040?logo=homebrew&logoColor=white)](https://github.com/markpasternak/homebrew-tap)
[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)](LICENSE)

Weather, in your terminal. Beautiful by default, scriptable by design.

![terminal weather demo](assets/screenshots/demo.gif)

- **3 panels** — Current hero, Hourly (Table / Hybrid / Chart), 7-Day forecast
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
terminal-weather --hero-visual sky-observatory --reduced-motion London
terminal-weather --ascii-icons --no-animation Reykjavik
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
  --no-animation                        Disable particle animation
  --reduced-motion                      Lower motion mode
  --no-flash                            Disable thunder flash
  --ascii-icons                         Force ASCII icons
  --emoji-icons                         Force emoji icons
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
| `1..5` | Select ambiguous location |

**Settings panel:** `↑`/`↓` navigate · `←`/`→` or `Enter` change value · `s`/`Esc` close

**City switcher:** type to search · `Enter` confirm · `↑`/`↓` browse recents · `1..9` quick-switch · `Delete` clear all · `Esc` close

---

## Configuration

Settings persist to `~/.config/terminal-weather/settings.json`. Override the directory with `TERMINAL_WEATHER_CONFIG_DIR` (legacy `ATMOS_TUI_CONFIG_DIR` is also supported).

Persisted values: units, theme, motion (`full`/`reduced`/`off`), thunder flash, icon mode (`unicode`/`ascii`/`emoji`), hourly view, hero visual, refresh interval, recent locations.

Color detection falls back TrueColor → xterm-256 → 16-color based on `COLORTERM` and `TERM`. `NO_COLOR` is honored in `auto` mode.

API endpoint overrides:

- `TERMINAL_WEATHER_FORECAST_URL` sets the forecast endpoint
- `TERMINAL_WEATHER_AIR_QUALITY_URL` sets the air-quality endpoint
- `--forecast-url` / `--air-quality-url` override env vars for the current run

Precedence is: CLI flag → environment variable → built-in default URL.

Custom endpoints must remain Open-Meteo compatible (same query parameters and response shape).

---

## Privacy

When launched without a city argument, `terminal-weather` sends your IP address to [ipapi.co](https://ipapi.co/) to determine your location. Pass a city name or `--lat`/`--lon` to skip this lookup entirely.

Weather and city search data come from [Open-Meteo](https://open-meteo.com/) (your coordinates and search string are sent). The IP-based auto-location lookup uses [ipapi.co](https://ipapi.co/). All services are free and open; no account or API key required, and no data is stored by this app.

---

## What's New in v0.5.1

- **Faster runtime path** — improved refresh/render efficiency with tighter terminal sizing logic, concurrent fetches, and forecast caching.
- **Hardening pass** — strengthened settings-file safety (Unix `0600` permissions and defensive size checks) to reduce local risk.
- **Better operator controls** — forecast and air-quality API endpoints are now overrideable via CLI flags and environment variables.
- **Quality and maintainability uplift** — substantial deduplication/refactoring plus expanded tests and stricter complexity/coverage gates.

---

## Contributing

Bug reports and pull requests are welcome. [Open an issue](https://github.com/markpasternak/terminal-weather/issues) to discuss a bug or feature before sending a PR.

To contribute code:

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Run `./scripts/ci-local.sh` — CI must pass before a PR can merge
4. Open a pull request against `main`

The maintainer reviews and merges all PRs. Direct pushes to `main` are restricted.

### Local Quality Gate

Install local tooling once:

```bash
cargo install --locked rust-code-analysis-cli --version 0.0.25
cargo install --locked cargo-dupes --version 0.2.1
# install jq via your package manager
```

Primary pre-PR check:

```bash
./scripts/ci-local.sh
```

If you want to run checks step-by-step:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::if_same_then_else -D clippy::match_same_arms -D clippy::branches_sharing_code
./scripts/static-analysis-gate.sh
./scripts/duplication-gate.sh
cargo check --all-targets --all-features
cargo test --all --all-features
cargo build --release
```

Optional local parity gate:

```bash
./scripts/codacy-complexity-gate.sh
```

---

## Attribution & License

Weather and geocoding data: [Open-Meteo](https://open-meteo.com/)

Licensed under [GPL-3.0-only](LICENSE).
