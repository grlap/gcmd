# gcmd

A dual-pane file manager written in Rust, inspired by Far2l and Total Commander.

## Features

- Dual-pane navigation with keyboard controls
- Dark theme with syntax highlighting
- Total Commander-style keybindings
- Cross-platform (Windows, macOS, Linux)

## Keybindings

| Key | Action |
|-----|--------|
| Tab | Switch between panels |
| Up/Down | Navigate files |
| Enter | Enter directory |
| Backspace | Go to parent directory |
| Home | Jump to first file |
| End | Jump to last file |
| Insert | Toggle file selection |
| Ctrl+R | Refresh both panels |

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Project Structure

```
src/
├── main.rs      # Application entry point
├── app.rs       # Main app state and UI logic
├── panel.rs     # File panel component
├── terminal.rs  # Embedded terminal (planned)
├── file_ops.rs  # File operations (planned)
└── theme.rs     # Theming (planned)
```

## Dependencies

- [iced](https://iced.rs/) - Cross-platform GUI framework
- tokio - Async runtime
- chrono - Date/time formatting
- bytesize - Human-readable file sizes
- dirs - Home directory detection

## License

MIT
