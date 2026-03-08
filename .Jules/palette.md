## 2024-03-08 - Added Bold Keyboard Shortcuts
**Learning:** UX Discoverability: Splitting keyboard shortcut hint strings into multiple Spans allows highlighting the specific letter/shortcut with a bold modifier and contrasting color, making shortcuts much more obvious and readable.
**Action:** Always wrap shortcut keys in a Span styled with `theme.text` and `Modifier::BOLD` when displaying shortcut hints, instead of leaving the entire line as `theme.muted_text`.
