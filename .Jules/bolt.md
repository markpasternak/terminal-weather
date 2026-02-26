## 2025-02-23 - TUI Update Loop Optimization
**Learning:** Even seemingly cheap operations like `rand::rng()` (TLS access) and `sin()` (trig) consume measurable CPU in tight update loops (e.g., 60Hz).
**Action:** Guard update logic with early returns or conditionals based on state (e.g., `if weather != Clear`) to skip processing for invisible effects.

## 2026-02-24 - Vec::retain vs swap_remove for Front Removal
**Learning:** `Vec::retain` (linear `memmove`) outperforms manual `swap_remove` (multiple small copies) when removing contiguous blocks from the front of small/medium vectors (<1000 items).
**Action:** Prefer `retain` over `swap_remove` loops for FIFO-like patterns in vectors unless order explicitly doesn't matter AND removal is truly random/scattered.
