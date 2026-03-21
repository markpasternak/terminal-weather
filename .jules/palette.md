## 2024-03-10 - Discovering missing keyboard shortcuts
**Learning:** Found multiple UI components without proper keyboard navigation or hints. Empty states and popups lack sufficient hints for the user to understand what they can do next.
**Action:** Enhance UI widgets (e.g., settings, city_picker, selector) with clear keyboard shortcut hints to improve TUI accessibility and discoverability.

## 2024-05-15 - Highlighting Shortcut Keys for Better Discoverability
**Learning:** In terminal UI applications, displaying uniform muted text for keyboard shortcut instructions makes them hard to read and discover. By splitting the text into multiple `Span`s inside a `Line` to highlight specific shortcut keys using `Modifier::BOLD` and a primary text color (like `theme.text`), shortcut discoverability is significantly improved.
**Action:** Always format keyboard shortcut hints in UI overlays using contrasting styles (e.g. bolding keys) instead of uniform text to help users identify actionable shortcuts faster.

## 2024-05-23 - [Modal State Management]
**Learning:** In TUI apps (like ratatui), modal overlays should manage their visibility based on async operation success, not immediate submission. Closing immediately loses context and feedback (e.g. "Searching...").
**Action:** When implementing modals that trigger async actions, keep the modal open to show progress/status, and only close it on success or explicit cancel.

## 2025-02-13 - Capitalize standalone keyboard shortcut hints
**Learning:** In terminal UI text strings, standalone lowercase keyboard shortcuts (like `l`, `s`, `r`, `q`) can cause readability issues due to font ambiguity (e.g., lowercase `l` vs `1` or `I`) and reduce visual scannability.
**Action:** Always capitalize standalone keyboard shortcut hints in UI text (e.g., `L`, `S`) to improve immediate recognition and follow standard UX conventions for key bindings.

## 2025-02-15 - Actionable Shortcuts on Empty/Error States
**Learning:** When a TUI application fails to load core data (e.g., weather API error), users are often left stranded. Standard ratatui text displays don't have interactive buttons to guide users.
**Action:** Always append actionable, capitalized keyboard shortcut hints (e.g., "Tip: press L for cities, S for settings, R to retry, Q to quit") directly into the error/empty state lines so the user immediately knows how to recover or exit.

## 2025-05-15 - Highlighting Shortcut Keys for Better Discoverability
**Learning:** In terminal UI applications, displaying uniform muted text for keyboard shortcut instructions makes them hard to read and discover. By splitting the text into multiple `Span`s inside a `Line` to highlight specific shortcut keys using `Modifier::BOLD` and a primary text color (like `theme.text`), shortcut discoverability is significantly improved.
**Action:** Always format keyboard shortcut hints in UI overlays using contrasting styles (e.g. bolding keys) instead of uniform text to help users identify actionable shortcuts faster.

## 2025-10-21 - [TUI Input Cursor Positioning]
**Learning:** In `ratatui`, `Paragraph` widgets with `Borders::BOTTOM` do not offset the inner content area horizontally or vertically. To correctly position a cursor for a text input, use the base area's (x, y) plus the text width.
**Action:** When adding cursors to inputs, check the border configuration carefully to determine the correct offset.

## 2026-03-16 - Highlight Keyboard Shortcuts
**Learning:** In TUI interfaces, inline keyboard shortcuts (e.g. `Tab` for panel focus or `:` for command bar) can become invisible or hard to discover if they are merged into the muted hint text strings. Furthermore, using standardized unicode arrows (`←/→`) instead of ascii text (`<-/->`) makes the UI feel more polished and consistent.
**Action:** When displaying keyboard shortcut hints (like in the footer), separate the shortcut key itself into its own distinct span and apply bold styling and a primary text color (like `theme.text`) to make it immediately stand out. Always use unicode arrows for navigation hints instead of ascii equivalents.

## 2026-03-24 - Explicit Closing Shortcuts for Modals
**Learning:** UX Standard (Ratatui TUI): Modal overlays and popups (such as settings panels) must explicitly display all their closing/canceling keyboard shortcuts (e.g., 'Esc' alongside feature-specific toggle keys) within their UI hint controls to ensure users can clearly discover how to dismiss them. Users can get stuck if they don't know the shortcut and the standard expected shortcut (like `Esc`) isn't visibly indicated.
**Action:** When creating or modifying modal dialogs and overlays, ensure that the footer/hint area lists standard closing keys (like `Esc`) alongside any widget-specific toggles (like `S`).
