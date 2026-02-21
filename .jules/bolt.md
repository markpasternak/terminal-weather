## 2025-02-12 - Redundant ioctl in event loop
**Learning:** `ratatui` TUIs often call `terminal.size()` inside the render loop, which is an ioctl. Avoid adding *another* redundant call to `terminal.size()` in the main loop if you can rely on `Event::Resize`.
**Action:** When optimizing TUI loops, check for redundant size queries and rely on event-driven updates.
