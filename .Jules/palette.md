## 2025-10-21 - [TUI Input Cursor Positioning]
**Learning:** In `ratatui`, `Paragraph` widgets with `Borders::BOTTOM` do not offset the inner content area horizontally or vertically. To correctly position a cursor for a text input, use the base area's (x, y) plus the text width.
**Action:** When adding cursors to inputs, check the border configuration carefully to determine the correct offset.
