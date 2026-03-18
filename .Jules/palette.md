## 2024-03-10 - Discovering missing keyboard shortcuts
**Learning:** Found multiple UI components without proper keyboard navigation or hints. Empty states and popups lack sufficient hints for the user to understand what they can do next.
**Action:** Enhance UI widgets (e.g., settings, city_picker, selector) with clear keyboard shortcut hints to improve TUI accessibility and discoverability.

## 2024-05-15 - Highlighting Shortcut Keys for Better Discoverability
**Learning:** In terminal UI applications, displaying uniform muted text for keyboard shortcut instructions makes them hard to read and discover. By splitting the text into multiple `Span`s inside a `Line` to highlight specific shortcut keys using `Modifier::BOLD` and a primary text color (like `theme.text`), shortcut discoverability is significantly improved.
**Action:** Always format keyboard shortcut hints in UI overlays using contrasting styles (e.g. bolding keys) instead of uniform text to help users identify actionable shortcuts faster.

## 2026-03-16 - Highlight Keyboard Shortcuts
**Learning:** In TUI interfaces, inline keyboard shortcuts (e.g. `Tab` for panel focus or `:` for command bar) can become invisible or hard to discover if they are merged into the muted hint text strings. Furthermore, using standardized unicode arrows (`←/→`) instead of ascii text (`<-/->`) makes the UI feel more polished and consistent.
**Action:** When displaying keyboard shortcut hints (like in the footer), separate the shortcut key itself into its own distinct span and apply bold styling and a primary text color (like `theme.text`) to make it immediately stand out. Always use unicode arrows for navigation hints instead of ascii equivalents.

## 2026-03-24 - Explicit Closing Shortcuts for Modals
**Learning:** UX Standard (Ratatui TUI): Modal overlays and popups (such as settings panels) must explicitly display all their closing/canceling keyboard shortcuts (e.g., 'Esc' alongside feature-specific toggle keys) within their UI hint controls to ensure users can clearly discover how to dismiss them. Users can get stuck if they don't know the shortcut and the standard expected shortcut (like `Esc`) isn't visibly indicated.
**Action:** When creating or modifying modal dialogs and overlays, ensure that the footer/hint area lists standard closing keys (like `Esc`) alongside any widget-specific toggles (like `S`).
