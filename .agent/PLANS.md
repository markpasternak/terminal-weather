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

## Final Report Checklist
- [x] Local run commands added
- [x] Test evidence included
- [x] Open risks/follow-ups documented
