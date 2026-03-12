## 2023-10-25 - [Optimize Date/Time Parsing for Open-Meteo Payloads]
**Learning:** `chrono::NaiveDateTime::parse_from_str` with format strings like `"%Y-%m-%dT%H:%M"` is very slow because it handles generic formatting dynamically. In systems where APIs return massive arrays of timestamps in exact formats, this quickly becomes a bottleneck. Replacing it with manual byte indexing and parsing integer substrings yields a ~300x performance increase.
**Action:** When repeatedly parsing known, fixed-length datetime formats from high-frequency APIs, prefer manual byte-slicing and integer calculation over generic `chrono` string parsing logic (with `parse_from_str` as a fallback).

## 2025-03-10 - [Avoid string formatting in accumulation loops]
**Learning:** `chrono::NaiveDate::format("%a").to_string()` creates dynamic formatting overhead and a new heap allocation every time it's called. Calling this repeatedly inside an aggregation loop (e.g. tracking max weather values) is unnecessarily expensive.
**Action:** When tracking time-based items inside a loop for UI representation, store lightweight structures like `chrono::NaiveDate` directly. Only perform the `.format().to_string()` allocation at the very end when generating the final UI text.

## 2025-03-10 - [Optimize Weekday Formatting in Rendering Loops]
**Learning:** `chrono::NaiveDate::format("%a").to_string()` requires string parsing and heap allocation on every call. In a TUI rendering loop (e.g., drawing `Cell`s for a daily forecast table every tick), this creates unnecessary allocator pressure.
**Action:** Replace `date.format("%a").to_string()` with a fast helper function that matches on `chrono::Datelike::weekday()` and returns a static `&'static str` (e.g., `"Mon"`, `"Tue"`). This guarantees zero-allocation weekday extraction.
