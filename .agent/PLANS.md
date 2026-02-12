# atmos-tui ExecPlan

## Goal
Build atmos-tui v1.1 end-to-end and satisfy Definition of Done.

## Milestones
- [x] P0 Toolchain bootstrap
- [x] M-1 Architecture & contracts
- [x] M0 Project skeleton
- [x] M1 Weather client + resolver
- [x] M2 Static UI layout
- [x] M3 Theming + color tiers
- [x] M4 Particles + motion controls
- [x] M5 Freshness/offline semantics
- [x] M6 QA hardening + docs

## Acceptance Criteria by Milestone
### P0
- Rust toolchain and required components installed and verified

### M-1
- App state + event model defined
- Render pipeline contract documented

### M0
- Terminal init/teardown safe
- App starts and exits cleanly

### M1
- Fetch + parse + render current weather
- Deterministic location resolution

### M2
- 3-panel layout working across breakpoints
- Hourly/daily with placeholders for missing data

### M3
- Theme selection and color fallback tiers work

### M4
- Particle behaviors operational
- reduced-motion/no-flash work

### M5
- refresh/backoff/fresh-stale-offline semantics pass tests

### M6
- tests, lint, build pass
- README complete
- final evidence documented

## Architecture Contracts
- App state machine: `Loading`, `SelectingLocation`, `Ready`, `Error`, `Quit`.
- Event model: `Bootstrap`, `TickFrame`, `TickRefresh`, `Input`, `FetchStarted`, `GeocodeResolved`, `FetchSucceeded`, `FetchFailed`, `Quit`.
- DTO -> view-model boundary:
  - Data layer in `src/data/*` parses Open-Meteo DTOs to domain types in `src/domain/weather.rs`.
  - UI only consumes domain `ForecastBundle` + `AppState`.
- Render order contract:
  1. Background gradient
  2. Particle overlay
  3. Text/widgets/overlays (status, panels, selector)

## Risks
- Unicode width inconsistencies across terminals
- API partial/missing fields
- Animation CPU spikes on slower machines
- Ambiguous geocoding for common city names

## Decision Log
- 2026-02-12: Implement in current repo root as single binary crate -> user confirmed -> aligns with requested execution flow.
- 2026-02-12: Use automated+manual hybrid performance validation -> user confirmed -> keeps CI practical while still requiring evidence.
- 2026-02-12: Prompt in-app selector on ambiguous geocode at startup -> user confirmed -> avoids silent wrong-city render.
- 2026-02-12: Use rustup bootstrap gate -> user confirmed -> machine initially lacked rust toolchain.
- 2026-02-12: Rust stack: tokio + crossterm + ratatui + reqwest -> chosen for async TUI/event-stream integration.
- 2026-02-12: Snapshot tests stabilized by removing wall-clock dependency from expected render output.
- 2026-02-12: 7-day widget switched to adaptive width/height modes with explicit column labels -> improves readability and use of available terminal space.
- 2026-02-12: Theme system moved to contrast-first semantic tokens + explicit CLI overrides -> prevents light-on-light failures across terminal capabilities.
- 2026-02-12: Added async web-sourced silhouette pipeline via Wikipedia thumbnail fetch + ASCII conversion + local fallback -> enables dynamic landmark visuals without blocking UI.
- 2026-02-12: Switched web silhouette rendering to `image-to-ascii` with edge-aware conversion and meaningful Wikipedia page-image selection -> improves landmark quality and recognizability.
- 2026-02-12: Added in-app settings panel with persistent runtime preferences (theme, units, motion, flash, icons, silhouette source, refresh interval) -> enables live UX customization across sessions.
- 2026-02-12: Improved full-screen utilization by scaling web silhouettes to available hero space and adapting panel/hero split for large terminals -> reduces empty space on wide/tall layouts.
- 2026-02-12: Added contrast-enforced semantic colors with dedicated popup surface/text/border tokens -> keeps settings/selectors readable and visually separated in every theme/capability tier.
- 2026-02-12: Applied theme tinting to web landmark art and themed the small-terminal guard + thunder flash overlay -> ensures the selected theme impacts all visible UI regions.
- 2026-02-12: Added luxury polish pass with hero typography scaling, staged loading choreography, and wide-screen micro-spacing tuning (hourly density + daily range expansion) -> improves elegance and space usage on large terminals.
- 2026-02-12: Added clear-all flow for recent locations in the city picker (selectable row + Delete shortcut + persisted wipe) -> gives users explicit history reset control.

## Progress Log
- 2026-02-12 09:08: Completed P0 verification, ran tool checks (rustup/rustc/cargo/clippy/rustfmt), result: PASS.
- 2026-02-12 09:11: Completed M-1 contracts + module architecture, ran full gates, result: PASS.
- 2026-02-12 09:15: Completed M0 skeleton (CLI + terminal lifecycle + loading + quit), ran full gates, result: PASS.
- 2026-02-12 09:17: Completed M1 data clients + deterministic resolver + initial weather render, ran full gates, result: PASS.
- 2026-02-12 09:19: Completed M2 static 3-panel responsive layout + unit toggle + hourly scroll + small-terminal guard, ran full gates, result: PASS.
- 2026-02-12 09:20: Completed M3 day/night theme engine + color capability fallback tiers, ran full gates, result: PASS.
- 2026-02-12 09:21: Completed M4 particle engine + wind drift + reduced-motion/no-flash/no-animation controls, ran full gates, result: PASS.
- 2026-02-12 09:22: Completed M5 refresh jitter + backoff retries + fresh/stale/offline + last-good retention, ran full gates, result: PASS.
- 2026-02-12 09:24: Completed M6 tests/docs/evidence hardening and final gate run, result: PASS.
- 2026-02-12 09:30: Completed post-M6 UI refinement (hero contrast + 7-day responsiveness/clarity), reran full gates, result: PASS.
- 2026-02-12 09:38: Added animated location-aware landmark ASCII hero panel and richer color styling in hourly/daily widgets, reran full gates, result: PASS.
- 2026-02-12 09:45: Refined loading UX with staged animated loader, shimmer placeholders, and syncing status animation; reran full gates, result: PASS.
- 2026-02-12 09:52: Added dynamic procedural city-signature skyline fallback and expanded hero weather metrics (feels-like, humidity, wind, precip, clouds, UV), reran full gates, result: PASS.
- 2026-02-12 11:42: Completed contrast/theming hardening (semantic palette, basic16-safe backgrounds, theme override CLI), reran full gates, result: PASS.
- 2026-02-12 11:42: Added web-sourced silhouette mode (Wikipedia image search/summary, ASCII conversion cache, async fetch event path with fallback), reran full gates, result: PASS.
- 2026-02-12 12:18: Upgraded silhouette engine to `image-to-ascii`, added better Wikipedia landmark image selection/fallback filtering, reran full gates, result: PASS.
- 2026-02-12 12:34: Implemented settings overlay (`s`) with live toggles + persistent config file (`~/.config/atmos-tui/settings.json`), reran full gates, result: PASS.
- 2026-02-12 12:47: Enlarged/resampled web-sourced landmark art and tuned large-terminal layout proportions, reran full gates, result: PASS.
- 2026-02-12 13:00: Fixed global contrast regressions (panel/popup readability and border separation), reran full gates, result: PASS.
- 2026-02-12 13:06: Reviewed theme propagation and fixed remaining non-themed areas (landmark colorization, small-terminal guard, flash overlay), reran full gates, result: PASS.
- 2026-02-12 13:12: Completed luxury polish pass (hero scale system, animated loading choreography, 120+ column spacing refinements, snapshot refresh), reran full gates, result: PASS.
- 2026-02-12 13:16: Added location-history clear-all UX and persistence in city picker, reran full gates, result: PASS.

## Final Report Checklist
- [x] Local run commands added
- [x] Test evidence included
- [x] Open risks/follow-ups documented
