## 2023-10-25 - [Optimize Date/Time Parsing for Open-Meteo Payloads]
**Learning:** `chrono::NaiveDateTime::parse_from_str` with format strings like `"%Y-%m-%dT%H:%M"` is very slow because it handles generic formatting dynamically. In systems where APIs return massive arrays of timestamps in exact formats, this quickly becomes a bottleneck. Replacing it with manual byte indexing and parsing integer substrings yields a ~300x performance increase.
**Action:** When repeatedly parsing known, fixed-length datetime formats from high-frequency APIs, prefer manual byte-slicing and integer calculation over generic `chrono` string parsing logic (with `parse_from_str` as a fallback).

## 2025-03-10 - [Avoid string formatting in accumulation loops]
**Learning:** `chrono::NaiveDate::format("%a").to_string()` creates dynamic formatting overhead and a new heap allocation every time it's called. Calling this repeatedly inside an aggregation loop (e.g. tracking max weather values) is unnecessarily expensive.
**Action:** When tracking time-based items inside a loop for UI representation, store lightweight structures like `chrono::NaiveDate` directly. Only perform the `.format().to_string()` allocation at the very end when generating the final UI text.

## 2025-03-10 - [Optimize Ratatui background filling loops]
**Learning:** `ratatui::buffer::Cell::set_symbol` and related builder functions can be surprisingly slow in tight background rendering loops (e.g., iterating over thousands of cells). Passing string slices requires the inner struct to drop existing strings, parse graphemes, and potentially reallocate. Constructing a single "blank" `Cell` entirely beforehand and using `*cell = blank_cell.clone()` is about ~3x faster.
**Action:** When filling or repainting large background areas with identical properties, configure a dummy `Cell` once and assign clones of it via `buf.cell_mut` instead of using `cell.set_symbol(" ")` over and over.

## 2025-03-10 - [Eliminate Redundant Enum Allocations by Passing Value Ownership]
**Learning:** In code dealing with types that contain heap-allocated elements like `String`s (e.g. `Location`), wrapping them in structures that get cloned creates multiple string allocations behind the scenes. Passing arguments by reference (`&T`) only to `clone()` them immediately to wrap inside an enum forces unnecessary allocations.
**Action:** In resolution or transformation functions that ultimately produce values (like `GeocodeResolution::NeedsDisambiguation`), change the signature to take ownership of its arguments (`T` instead of `&T`). Move the owned data into the resulting enum variants to avoid deep copies and reduce allocation overhead.
