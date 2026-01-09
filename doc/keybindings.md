# gcmd Keybindings

## Global Shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `Escape` | Cancel drag / Exit terminal | Always |
| `Ctrl+O` | Toggle terminal focus | Always |
| `Ctrl+T` | New tab | Panel focused |
| `Ctrl+W` | Close current tab | Panel focused |
| `Ctrl+R` | Refresh all panels | Panel focused |

## Panel Navigation

| Key | Action |
|-----|--------|
| `Tab` | Switch between left/right panels |
| `Up` | Move cursor up |
| `Down` | Move cursor down |
| `Home` | Jump to first entry |
| `End` | Jump to last entry |
| `Left` | Jump to first entry |
| `Right` | Jump to last entry |

## File Operations

| Key | Action |
|-----|--------|
| `Enter` | Enter directory / Open file |
| `Backspace` | Go to parent directory |
| `Insert` | Toggle file selection |

## Tab Management

| Key | Action |
|-----|--------|
| `Ctrl+T` | New tab (same directory) |
| `Ctrl+W` | Close current tab |
| `Ctrl+PageDown` | Next tab |
| `Ctrl+PageUp` | Previous tab |

## Terminal Mode

When terminal is focused (`Ctrl+O` to toggle):

| Key | Action |
|-----|--------|
| `Escape` | Return to file panels |
| `Ctrl+C` | Send interrupt signal |
| `Ctrl+D` | Send EOF |
| `Ctrl+Z` | Suspend process |
| `Ctrl+L` | Clear terminal |
| Arrow keys | Terminal navigation |
| `Home/End` | Line start/end |
| `Delete` | Delete character |
| `Tab` | Tab completion (shell-dependent) |

## Mouse Actions

| Action | Result |
|--------|--------|
| Click on file entry | Select entry (move cursor) |
| Double-click on entry | Enter directory / Open file |
| Click on tab | Switch to tab |
| Click on `+` button | Add new tab |
| Drag from path header | Start tab drag |
| Release on target pane | Drop tab |
| Drag pane divider | Resize panes |

## Total Commander Compatibility

gcmd aims to be compatible with Total Commander keybindings:

| TC Key | gcmd Key | Action |
|--------|----------|--------|
| `Tab` | `Tab` | Switch panels |
| `Enter` | `Enter` | Enter directory |
| `Backspace` | `Backspace` | Parent directory |
| `Insert` | `Insert` | Toggle selection |
| `Home` | `Home` | First file |
| `End` | `End` | Last file |
| `Ctrl+R` | `Ctrl+R` | Refresh |
