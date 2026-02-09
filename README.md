# gcmd

A dual-pane file manager written in Rust, inspired by Far2l and Total Commander.

## Features

- Dual-pane navigation with keyboard controls
- Tabbed browsing (Ctrl+T/W to open/close, Ctrl+Tab to switch)
- Embedded terminal with PTY support (Ctrl+O to toggle)
- Bidirectional directory sync between panels and terminal
- F3 fullscreen file viewer with line numbers
- Incremental folder search via `cd` command
- Clipboard support (copy file name/path)
- External terminal launch (`cmd` command)
- Auto-respawn terminal on exit
- Dark theme
- Cross-platform (Windows, macOS, Linux)

## Keybindings

| Key | Action |
|-----|--------|
| Tab | Switch between panels |
| Up/Down | Navigate files |
| Enter | Enter directory / execute command |
| Backspace | Delete last character from command line |
| Home/End | Jump to first/last file |
| PageUp/PageDown | Page through file list |
| Space | Toggle file selection |
| F3 | Open/close file viewer |
| Ctrl+O | Toggle terminal overlay |
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+Tab | Next tab |
| Ctrl+PageUp/Down | Switch tabs |
| Ctrl+R | Refresh both panels |
| Ctrl+Enter | Copy file/folder name to clipboard |
| Shift+Ctrl+Enter | Copy full path to clipboard |

### Terminal Mode (Ctrl+O)

| Key | Action |
|-----|--------|
| All keys | Sent directly to shell |
| Ctrl+O | Return to panels (syncs directory) |
| Ctrl+C/D/Z/L | Terminal control sequences |

### Command Line

| Input | Action |
|-------|--------|
| `cd <path>` | Navigate panel to directory |
| `cmd` | Open new external terminal window |
| Any other text | Execute in embedded terminal |

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
├── main.rs                  # Application entry point
├── app.rs                   # Main app state, UI layout, keyboard handling
├── panel.rs                 # Panel trait definition
├── text_utils.rs            # Shared text utilities
├── folder_panel/
│   ├── model.rs             # File panel state and navigation
│   └── view.rs              # File panel rendering
├── tab_container/
│   ├── model.rs             # Tab management
│   └── view.rs              # Tab bar rendering
├── file_viewer/
│   ├── model.rs             # File viewer state and scrolling
│   └── view.rs              # File viewer rendering
└── terminal_panel/
    ├── model.rs             # PTY management, shell I/O, directory sync
    └── view.rs              # Terminal rendering with cursor
```

## Dependencies

- [iced](https://iced.rs/) - Cross-platform GUI framework
- [portable-pty](https://crates.io/crates/portable-pty) - Portable pseudo-terminal
- [vt100](https://crates.io/crates/vt100) - Terminal emulation
- tokio - Async runtime
- chrono - Date/time formatting
- bytesize - Human-readable file sizes
- dirs - Home directory detection

## License

MIT
