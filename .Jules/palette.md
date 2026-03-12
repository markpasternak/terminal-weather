## 2024-03-10 - Discovering missing keyboard shortcuts
**Learning:** Found multiple UI components without proper keyboard navigation or hints. Empty states and popups lack sufficient hints for the user to understand what they can do next.
**Action:** Enhance UI widgets (e.g., settings, city_picker, selector) with clear keyboard shortcut hints to improve TUI accessibility and discoverability.

## 2024-05-15 - Highlighting Shortcut Keys for Better Discoverability
**Learning:** In terminal UI applications, displaying uniform muted text for keyboard shortcut instructions makes them hard to read and discover. By splitting the text into multiple `Span`s inside a `Line` to highlight specific shortcut keys using `Modifier::BOLD` and a primary text color (like `theme.text`), shortcut discoverability is significantly improved.
**Action:** Always format keyboard shortcut hints in UI overlays using contrasting styles (e.g. bolding keys) instead of uniform text to help users identify actionable shortcuts faster.
