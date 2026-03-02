## 2025-10-21 - [TUI Input Cursor Positioning]
**Learning:** In `ratatui`, `Paragraph` widgets with `Borders::BOTTOM` do not offset the inner content area horizontally or vertically. To correctly position a cursor for a text input, use the base area's (x, y) plus the text width.
**Action:** When adding cursors to inputs, check the border configuration carefully to determine the correct offset.

## 2024-05-23 - [Modal State Management]
**Learning:** In TUI apps (like ratatui), modal overlays should manage their visibility based on async operation success, not immediate submission. Closing immediately loses context and feedback (e.g. "Searching...").
**Action:** When implementing modals that trigger async actions, keep the modal open to show progress/status, and only close it on success or explicit cancel.
## 2025-02-13 - Capitalize standalone keyboard shortcut hints
**Learning:** In terminal UI text strings, standalone lowercase keyboard shortcuts (like `l`, `s`, `r`, `q`) can cause readability issues due to font ambiguity (e.g., lowercase `l` vs `1` or `I`) and reduce visual scannability.
**Action:** Always capitalize standalone keyboard shortcut hints in UI text (e.g., `L`, `S`) to improve immediate recognition and follow standard UX conventions for key bindings.
