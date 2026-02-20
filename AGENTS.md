# AGENTS.md

Agent instructions for `terminal-weather` — an animated terminal weather dashboard written in Rust.

---

## Project Overview

Single-binary Rust TUI app (Rust 2024 edition) using:

- **`ratatui` + `crossterm`** — terminal UI and raw-mode event loop
- **`tokio`** — async runtime (multi-thread)
- **`reqwest` + `serde`** — HTTP client for Open-Meteo and geocoding APIs
- **`clap`** — CLI argument parsing
- **`anyhow` / `thiserror`** — error handling
- **Dev:** `insta` (snapshot tests), `proptest` (property tests), `wiremock` (HTTP mocking)
- **Static tooling:** `jq`, `rust-code-analysis-cli` `0.0.25`

---

## Commands

Run these before marking anything done. Put them in this order:

```bash
cargo fmt --all                                  # format
cargo fmt --all -- --check                       # verify formatting (CI gate)
cargo clippy --all-targets --all-features -- -D warnings   # lint, zero warnings
cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic  # pedantic lint gate
./scripts/static-analysis-gate.sh                # complexity/MI static gate (CI gate)
cargo check --all-targets --all-features         # type-check
cargo test --all --all-features                  # full test suite
cargo build --release                            # release build
```

Additional local parity gate:
```bash
./scripts/codacy-complexity-gate.sh              # Codacy-style complexity parity (critical fail by default)
```

Install static tooling once:
```bash
cargo install --locked rust-code-analysis-cli --version 0.0.25
# jq via package manager (brew/apt/etc.)
```

Static gate thresholds (enforced by CI and local script):
- cyclomatic complexity: `< 20`
- cognitive complexity: `< 30`
- maintainability index (MI): `>= 30`
- scope: all functions in `src/` + `tests/`

Codacy complexity parity thresholds (`scripts/codacy-complexity-gate.sh`):
- file NLOC: medium `> 500`, critical `> 1000`
- function NLOC: medium `> 50`, critical `> 100`
- cyclomatic complexity: medium `> 8`, critical `> 12`
- function parameter count: medium `> 8`, critical `> 12`
- default fail policy: fail critical (`TW_FAIL_ON_CRITICAL=1`), report medium (`TW_FAIL_ON_MEDIUM=0`)

For fast iteration during development:
```bash
cargo check                       # fastest type-check
cargo test <test_name>            # single test
cargo run -- Stockholm            # run with a city
cargo run -- --demo               # run scripted demo mode
cargo run -- --one-shot Stockholm # non-interactive snapshot to stdout
```

Update snapshots when UI changes are intentional:
```bash
INSTA_UPDATE=always cargo test --all --all-features
```

---

## Project Structure

```
src/
  main.rs           # entrypoint, wires Cli + App
  lib.rs            # crate root, re-exports
  cli.rs            # clap CLI definitions, flags, enums
  events.rs         # top-level event dispatch
  app/
    mod.rs          # App struct, run loop
    events.rs       # AppEvent enum + handlers
    settings.rs     # persistent settings (JSON at ~/.config/terminal-weather/)
    state.rs        # AppState machine: Loading → SelectingLocation → Ready → Error → Quit
    state/
      methods_async.rs
      methods_fetch.rs
      methods_ui.rs
  data/
    mod.rs
    forecast.rs     # Open-Meteo API client + DTO parsing
    geocode.rs      # Nominatim geocoding
    geoip.rs        # IP-based location fallback
  domain/
    weather.rs      # domain types (ForecastBundle, etc.)
    weather/
      tests.rs
  resilience/       # retry/backoff logic, freshness semantics
  ui/
    mod.rs          # render entrypoint
    layout.rs       # responsive breakpoint helpers
    particles.rs    # particle animation engine
    theme.rs        # semantic color tokens, theme variants, capability tiers
    theme/
      capability.rs
      data.rs
      extended.rs
      tests.rs
    widgets/
      hero/         # current conditions hero panel (readouts, sparklines, background)
      hourly.rs     # per-hour table/chart panel
      hourly/
        daypart.rs
        table.rs
        timeline.rs
        tests.rs
      daily.rs      # 7-day forecast panel
      daily/
        layout.rs
        summary.rs
        summary/utils.rs
        tests.rs
      landmark/     # ASCII landmark art scenes (AtmosCanvas / GaugeCluster / SkyObservatory)
        atmos/
          effects.rs
          tests.rs
      city_picker.rs
      settings.rs
      alerts.rs
      help.rs
      selector.rs

tests/
  flows.rs                    # integration: unit toggle, hourly scroll, key events
  freshness_integration.rs    # stale/offline semantics
  geocode_ambiguity.rs        # multi-result city picker flow
  property_range_bar.rs       # proptest: range bar widget
  render_snapshots.rs         # insta snapshots at 40x15, 60x20, 80x24, 120x40
  snapshots/                  # committed snapshot files — update intentionally only
```

---

## Architecture Contracts

**State machine:** `Loading` → `SelectingLocation` → `Ready` → `Error` → `Quit`

**Event model:** `Bootstrap`, `TickFrame`, `TickRefresh`, `Input`, `FetchStarted`, `GeocodeResolved`, `FetchSucceeded`, `FetchFailed`, `Quit`

**DTO boundary:** data layer (`src/data/`) parses Open-Meteo DTOs into domain types (`src/domain/weather.rs`). UI only consumes `ForecastBundle` + `AppState`.

**Render order:**
1. Background gradient
2. Particle overlay
3. Widgets / text / overlays (status bar, panels, dialogs)

---

## Code Style

Rust 2024 edition idioms. Match the surrounding code's style exactly before adding anything new.

**Prefer:**
- `thiserror` for domain errors, `anyhow` for application-level propagation
- Strongly typed enums over stringly-typed flags
- `impl Display` and `From` impls over manual error messages
- Functional iterator chains over imperative loops where clarity improves
- Descriptive variable names — no single-letter vars outside tight loops

**Avoid:**
- `unwrap()` / `expect()` outside tests
- Dead code — delete unused functions and fields
- Unnecessary `clone()` — prefer borrowing
- Adding dependencies for trivial things the stdlib or existing deps already handle
- Touching snapshot files manually — let `insta` manage them

**Naming:**
- Types: `PascalCase`, files: `snake_case`, modules: `snake_case`
- Match existing module naming — look at the file before naming a new one

---

## Testing

- Every non-trivial behavioral change needs a test or a snapshot update
- Snapshot tests live in `tests/render_snapshots.rs` — update intentionally with `INSTA_UPDATE=always`
- Unit tests go in `#[cfg(test)]` blocks in the same file as the code
- Integration tests go in `tests/`
- Don't delete tests to make the suite green — fix the underlying issue

---

## Git Workflow

- Work on `main` branch (single-developer project)
- Commit messages: `type: short description` (types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`)
- Keep commits atomic — one logical change per commit
- Never force-push without explicit instruction
- Never commit: secrets, `.env` files, `target/`, `.claude/`, `.agent/`
- Run the full quality gate before committing

---

## Boundaries — Never Touch

- `Cargo.lock` version pins without explicit instruction (dependency updates are deliberate)
- Snapshot files in `tests/snapshots/` without running the test suite and confirming intent
- `~/.config/terminal-weather/` — user's live settings (only tests or the app itself touch this)
- Release profile settings in `Cargo.toml` (`lto`, `codegen-units`, `strip`) — tuned deliberately

---

## Workflow Principles

### Plan Before Acting
Enter plan mode for any task with 3+ steps or architectural decisions. Write the plan out, check in before implementing. If something goes sideways mid-task, stop and re-plan — don't push forward blindly.

### Subagent Strategy
Use subagents to keep the main context clean. Offload research, codebase exploration, and parallel analysis. One focused tack per subagent.

### Verification Before Done
Never call a task complete without proving it works. Run the full quality gate. Ask: *"Would a staff engineer approve this?"* Diff behavior between before and after when relevant.

### Autonomous Bug Fixing
Given a bug report: fix it. Don't ask for hand-holding. Point at logs, errors, failing tests — resolve them. Fix failing CI without being told how.

### Demand Elegance (Balanced)
For non-trivial changes: pause and ask "is there a more elegant solution?" If a fix feels hacky: "Knowing everything I know now, implement the elegant solution." Skip this for obvious one-liners — don't over-engineer.

### Self-Improvement Loop
After any correction: capture the pattern so it doesn't repeat. Update your understanding of what this project values. Ruthlessly reduce mistake rate.

---

## Task Management

1. **Plan first** — write a plan with checkable items before touching code
2. **Verify the plan** — check in before starting implementation on anything non-trivial
3. **Track progress** — mark items complete as you go
4. **Explain changes** — high-level summary at each significant step
5. **Document results** — summarize what was done and what was verified

---

## Core Principles

- **Simplicity first** — make every change as simple as possible, touching minimal code
- **No laziness** — find root causes, no temporary workarounds, senior-developer standards
- **Minimal impact** — changes should only affect what's necessary; avoid collateral modifications
- **No over-engineering** — three similar lines beat a premature abstraction; YAGNI
- **No security holes** — no `unwrap()` on untrusted input, no command injection, validate at boundaries

---

## Interview Me Relentlessly

Before starting any non-trivial task, ask questions until you understand:

1. **Intent** — what outcome does the user actually want? Not just what they typed, but why.
2. **Taste** — what does "good" look like to them? Ask for examples or comparisons if unclear.
3. **Constraints** — what must not change? What are the hard limits?
4. **Priority** — if tradeoffs arise (speed vs. correctness, simplicity vs. features), what wins?
5. **Definition of done** — how will we know when it's finished?

Don't start implementing until you can answer all five. If the answer to any of them is "I'm not sure," ask. A single clarifying question now saves hours of rework. Keep asking until you have a crisp mental model of what success looks like — then build exactly that, nothing more.
