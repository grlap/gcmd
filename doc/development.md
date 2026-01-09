# gcmd Development Guide

## Prerequisites

- Rust 1.75+ (2024 edition)
- cargo

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `iced` | 0.14 | GUI framework |
| `tokio` | 1.x | Async runtime |
| `chrono` | 0.4 | Date/time formatting |
| `bytesize` | 1.3 | Human-readable file sizes |
| `dirs` | 5 | Home directory detection |
| `portable-pty` | 0.8 | PTY management |
| `vt100` | 0.15 | Terminal emulation |

## Project Structure

```
gcmd/
├── Cargo.toml           # Project manifest
├── README.md            # Project overview
├── doc/                 # Documentation
│   ├── architecture.md  # System design
│   ├── features.md      # Feature list
│   ├── keybindings.md   # Keyboard shortcuts
│   └── development.md   # This file
├── fonts/               # Embedded fonts
└── src/
    ├── main.rs          # Entry point
    ├── app.rs           # Main application
    ├── panel.rs         # Panel traits
    ├── folder_panel/    # File browser
    ├── tab_container/   # Tab management
    └── terminal_panel/  # Embedded terminal
```

## Adding a New Panel Type

1. Create a new module under `src/`:
   ```
   src/my_panel/
   ├── mod.rs
   ├── model.rs
   └── view.rs
   ```

2. Define your entry type implementing `PanelEntry`:
   ```rust
   impl PanelEntry for MyEntry {
       fn name(&self) -> &str { ... }
       fn is_dir(&self) -> bool { ... }
       fn is_selected(&self) -> bool { ... }
       fn size_display(&self) -> String { ... }
       fn date_display(&self) -> String { ... }
   }
   ```

3. Define your panel implementing `Panel`:
   ```rust
   impl Panel for MyPanel {
       type Entry = MyEntry;
       // Implement required methods...
   }
   ```

4. Add to `TabContainer` if it should support tabs.

## Adding New Keybindings

1. Add a new `Message` variant in `src/app.rs`:
   ```rust
   pub enum Message {
       // ...
       MyNewAction { param: Type },
   }
   ```

2. Handle the key in `App::update()`:
   ```rust
   Message::EventOccurred(Event::Keyboard(...)) => {
       // Check for your key combination
   }
   ```

3. Process the message:
   ```rust
   Message::MyNewAction { param } => {
       // Handle the action
   }
   ```

## Iced Framework Notes

### Elm Architecture

Iced uses the Elm architecture:
- **Model**: Application state (`App` struct)
- **Message**: User actions (`Message` enum)
- **Update**: State transitions (`App::update`)
- **View**: UI rendering (`App::view`)

### Widget System

Common widgets used:
- `container`: Styling wrapper
- `column`, `row`: Layout
- `text`: Text display
- `scrollable`: Scrolling content
- `pane_grid`: Resizable panes
- `mouse_area`: Mouse event capture
- `stack`: Overlay layers

### Subscriptions

Event handling via subscriptions:
```rust
pub fn subscription(&self) -> Subscription<Message> {
    event::listen().map(Message::EventOccurred)
}
```

## Testing

```bash
# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run
```

## Common Issues

### PTY on macOS

The terminal requires a valid PTY. If you see errors:
- Ensure `portable-pty` is properly linked
- Check `$SHELL` environment variable

### Font Rendering

If fonts appear incorrectly:
- Check the `fonts/` directory for required fonts
- Verify font loading in Iced settings

### Performance

For large directories:
- Consider lazy loading
- Implement virtual scrolling for many entries
