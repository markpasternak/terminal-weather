# Changelog

## [0.9.0] - 2026-03-07

`terminal-weather` 0.9.0 is a release about atmosphere and clarity. It makes the app's palette more responsive to live weather, adds a few stronger manual theme choices, and gives the sky observatory a denser, more readable dashboard of its own.

### Highlights

- Auto theme is now weather-aware in a more literal way: it reacts to time of day, cloud cover, rain and snow pressure, fog, and short-horizon forecast signals instead of relying on the older coarse mapping.
- Three new preset themes have been added: Tokyo Night Storm, Rose Pine Moon, and Everforest Dark.
- The sky-observatory hero has been rebuilt with clearer celestial positioning, a structured observatory band, richer sunrise and daylight summaries, and stronger weather and precipitation lanes.

### UX and quality improvements

- Hourly and daily panels now resolve their theme through the live forecast path more consistently, so auto theming behaves the same way across the dashboard.
- Theme metadata has been reorganized so labels, mood notes, and palette specs stay aligned as the preset catalog grows.
- The CLI help and docs now reflect the expanded theme list, while `--one-shot` and the rest of the command surface remain unchanged.

### Maintenance

- Release tooling and local analysis scripts have been cleaned up so the quality gate runs without the recent warning noise.
- Internal `.jules` path casing has been normalized for consistency in local tooling and workspace references.

## [0.8.0] - 2026-03-06

`terminal-weather` 0.8.0 is a release about feel and trust. It makes the dashboard more alive on screen, more readable in the hourly pane, easier to recover when a fetch or search goes sideways, and stricter about the network data it accepts.

### Highlights

- Cinematic motion now drives the hero, loading states, landmark scenes, and panel transitions with weather-aware choreography instead of isolated effects.
- The hourly chart has been expanded and rebalanced so temperature trends, precipitation, and scale placement read more clearly at common terminal sizes.
- Loading and scene transitions are more deliberate across the dashboard, which makes the UI feel more cohesive during refreshes and startup.
- The demo recording workflow has been restored so release visuals can stay current with the shipped experience.

### UX and quality improvements

- Error states now show actionable keyboard shortcuts so it is clear how to retry, change settings, or exit.
- The city picker is easier to work with: searches keep the picker open, recent-location navigation wraps around, quick-switching is faster, and the input shows a character counter.
- Coordinate validation is stricter for `--lat` and `--lon`, which gives faster feedback on invalid input.
- Shortcut labels and settings navigation are more consistent with what the UI already shows on screen.
- `--one-shot` and other CLI paths continue to benefit from smaller parsing and allocation improvements without changing their output format.

### Hardening and reliability

- API clients now enforce tighter request timeouts and response size limits, which reduces the chance that a slow or oversized upstream response can hang or flood the app.
- HTTP client configuration is locked down more consistently, including versioned user-agent behavior and stronger failure handling during client setup.
- Update checks against the Homebrew formula now use tighter timeouts and payload limits, and stay quiet when they fail or find no update.
- GeoIP and geocoding text is sanitized before display, including control-character filtering and stricter field-length handling for remote data.
- Config-directory handling is more defensive, which reduces the risk of creating or using unsafe local paths.
