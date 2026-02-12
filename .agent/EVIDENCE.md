# atmos-tui Evidence (v1.1)

## Quality Gate Transcript Summary
All required gate commands executed successfully:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets --all-features
cargo test --all --all-features
cargo build --release
```

Result: PASS

## Test Inventory and Results
- Unit tests: 10 passing
- Flow/integration tests: 5 passing (`flows`, `freshness_integration`, `geocode_ambiguity`)
- Property tests: 1 passing (`property_range_bar`)
- Snapshot tests: 5 passing (`120x40`, `80x24`, `60x20`, `40x15` with clear/rain/snow/fog/thunder fixtures)

Total tests passing: 21

## User Flow Evidence
1. Startup + render: PASS (manual TTY run + rendered loading/forecast output observed)
2. Unit toggle (`f`, `c`): PASS (`tests/flows.rs`)
3. Manual refresh (`r`): PASS (event handling path implemented and exercised in runtime)
4. Hourly scroll (`←`, `→`): PASS (`tests/flows.rs` + clamped offset logic)
5. Resize behavior: PASS (responsive density rules + snapshot coverage)
6. Small terminal guard: PASS (renderer hard fail below `30x15`)
7. Accessibility flags (`--reduced-motion`, `--no-flash`): PASS (engine flags wired and verified in code path)
8. ASCII fallback (`--ascii-icons`): PASS (manual TTY run with `--ascii-icons`)
9. Ambiguous geocode selector: PASS (`tests/geocode_ambiguity.rs`)

## Build Evidence
- Release build: PASS
- Release binary: `target/release/atmos-tui`
- Binary size: `3.3M` (under 15MB target)

## Performance/NFR Evidence
- Binary size target (<15MB): PASS (`3.3M`)
- First-loading/frame/CPU/RSS precise p95 targets: PARTIAL (not fully benchmarked in automated harness in this iteration)

## Updated Artifacts
- `.gitignore`
- `assets/screenshots/app-preview.svg`
- `scripts/capture_fullscreen_screenshot.sh`
- `src/cli.rs`
- `src/app/events.rs`
- `src/app/settings.rs`
- `src/app/state.rs`
- `src/data/mod.rs`
- `src/ui/widgets/landmark.rs`
- `tests/flows.rs`
- `src/data/forecast.rs`
- `src/domain/weather.rs`
- `src/ui/mod.rs`
- `src/ui/widgets/daily.rs`
- `src/ui/widgets/hero.rs`
- `src/ui/widgets/hourly.rs`
- `tests/render_snapshots.rs`
- `tests/snapshots/render_snapshots__120x40_clear.snap`
- `tests/snapshots/render_snapshots__80x24_rain.snap`
- `tests/snapshots/render_snapshots__80x24_thunder.snap`
- `tests/snapshots/render_snapshots__60x20_snow.snap`
- `tests/snapshots/render_snapshots__40x15_fog.snap`
- `.agent/PLANS.md`
- `.agent/EVIDENCE.md`
- `README.md`
## Open Risks / Follow-ups
1. CPU and frame-time p95 targets need explicit benchmark harness in CI/release profiles.
2. RSS measurement should be captured with scripted runtime sampling on representative terminals.
3. Unicode width variance remains terminal-dependent; `--ascii-icons` is mitigation.
4. Auto-refresh interval changes from settings are persisted, but active background refresh schedule updates on next launch.
