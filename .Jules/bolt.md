## 2025-02-23 - TUI Update Loop Optimization
**Learning:** Even seemingly cheap operations like `rand::rng()` (TLS access) and `sin()` (trig) consume measurable CPU in tight update loops (e.g., 60Hz).
**Action:** Guard update logic with early returns or conditionals based on state (e.g., `if weather != Clear`) to skip processing for invisible effects.
