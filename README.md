# atmos-tui

`atmos-tui` is a terminal weather dashboard with animated weather ambience, responsive panels, deterministic geocoding, and resilient freshness/offline semantics.

## Features
- Current weather hero panel with condition, temperature, H/L, and location
- Rich hero telemetry: feels-like, dew point, humidity, pressure trend, visibility, cloud layers, UV max
- Responsive hourly strip (12/8/6 columns based on terminal width)
- Hourly grid with labeled rows (time, weather, temp, precip, gust, visibility, cloud/pressure when space allows)
- 7-day forecast with normalized temperature range bars, optional precip/gust columns, and week insights
- Weather-aware themes with day/night adaptation, plus curated presets from iTerm2-Color-Schemes
- Landmark panel with known-city ASCII art, procedural skyline fallback, and optional web-sourced silhouette mode
- Web silhouette conversion powered by `rascii_art` using high-resolution RASCII block charset rendering
- Large-terminal scaling: landmark art and top layout expand to better use full-screen terminal space
- Particle animation engine (rain/snow/fog/thunder) with wind drift
- Fresh/stale/offline state handling with retry backoff and last-good data retention
- Deterministic geocode ranking with in-app disambiguation selector (keys `1..5`)
- Accessibility controls: `--no-animation`, `--reduced-motion`, `--no-flash`, `--ascii-icons`

## Prerequisites
- Rust stable toolchain (`rustup`, `cargo`, `rustc`)
- Terminal with UTF-8 support (TrueColor preferred)
- Network access for Open-Meteo API calls

## Installation
```bash
git clone <repo-url>
cd terminal-weather
rustup default stable
cargo build --release
```

## Usage
```bash
cargo run -- Stockholm
cargo run -- "São Paulo"
cargo run -- --units fahrenheit Tokyo
cargo run -- --ascii-icons --no-animation Reykjavik
cargo run -- --reduced-motion --no-flash London
cargo run -- --lat 59.3293 --lon 18.0686
cargo run -- --theme high-contrast Stockholm
cargo run -- --silhouette-source web Stockholm
```

### CLI flags
```bash
atmos-tui [CITY]

Options:
  --units <celsius|fahrenheit>   Default: celsius
  --fps <N>                      15..60 (default: 30)
  --no-animation                 Disable particle animation
  --reduced-motion               Lower motion mode
  --no-flash                     Disable thunder flash
  --ascii-icons                  Force ASCII icons
  --emoji-icons                  Force emoji icons
  --theme <auto|aurora|mono|high-contrast|dracula|gruvbox-material-dark|kanagawa-wave|ayu-mirage|ayu-light|poimandres-storm|selenized-dark|no-clown-fiesta>
                                 Visual theme override (default: auto)
  --silhouette-source <local|auto|web>
                                 Landmark art source strategy (default: auto)
  --country-code <ISO2>          Geocode bias (e.g., SE, US)
  --lat <FLOAT>                  Direct latitude (requires --lon)
  --lon <FLOAT>                  Direct longitude (requires --lat)
  --refresh-interval <secs>      Default 600
  --help
  --version
```

## Keybindings
- `q` or `Esc`: quit
- `r`: manual refresh
- `s`: open/close settings panel
- `l`: open/close city switcher
- `f`: switch to Fahrenheit
- `c`: switch to Celsius
- `←` / `→`: scroll hourly strip
- `1..5`: choose location during geocode disambiguation

### City switcher controls
- Type city name and press `Enter` to search
- `1..9`: quick-switch to recent cities
- `↑` / `↓`: select recent city, then `Enter` to switch
- `Backspace`: edit query
- Search/switch history is persisted in `~/.config/atmos-tui/settings.json`

### Settings panel controls
- `↑` / `↓`: select setting row
- `←` / `→`: cycle selected setting value
- `Enter`: run selected action (`Refresh now` / `Close`)
- Settings are persisted across sessions in `~/.config/atmos-tui/settings.json`
- Optional override for settings location: `ATMOS_TUI_CONFIG_DIR` (directory path)

## Terminal Compatibility and Color Fallback
Color rendering degrades in this order:
1. TrueColor (`COLORTERM=truecolor`/`24bit`)
2. xterm-256 quantized palette
3. 16-color semantic fallback (`NO_COLOR` forces this mode)

Icon rendering modes:
- Default: Unicode symbols
- `--ascii-icons`: fixed-width ASCII-safe labels
- `--emoji-icons`: emoji set

Theme modes:
- `auto`: weather/day-aware palette
- `aurora`: vivid cool palette
- `mono`: monochrome palette
- `high-contrast`: maximum text contrast
- `dracula`, `gruvbox-material-dark`, `kanagawa-wave`, `ayu-mirage`, `ayu-light`, `poimandres-storm`, `selenized-dark`, `no-clown-fiesta`: presets sourced from [iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes)

## Troubleshooting
- Network/API failure:
  - App keeps last known good weather visible.
  - Status moves to `⚠ stale` then `⚠ offline` based on age/failures.
  - Manual retry with `r`.
- Terminal too small:
  - Below `30x15`, app shows a resize warning only.
- Unicode/icon width issues:
  - Use `--ascii-icons` for stable alignment on limited terminals.
- Web silhouette issues:
  - `--silhouette-source web` fetches meaningful landmark/page images from Wikipedia and converts them with `rascii_art` block charset rendering.
  - Web silhouettes keep source image colors (not theme-tinted) for maximum visual detail.
  - If fetch fails, `auto` mode falls back to built-in/procedural silhouettes.
- Coordinate flags error:
  - `--lat` and `--lon` must be provided together.

## Development Commands
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets --all-features
cargo test --all --all-features
cargo build --release
```

## Weather Data Attribution
Weather and geocoding data are provided by [Open-Meteo](https://open-meteo.com/).
