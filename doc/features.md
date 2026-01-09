# gcmd Features

## Dual-Pane Navigation

gcmd provides a classic dual-pane file manager interface:

- **Left and Right Panels**: Two independent file browsers side by side
- **Resizable Divider**: Drag the center divider to resize panes
- **Active Panel Highlight**: Blue border indicates the focused panel
- **Quick Switching**: Press `Tab` to switch between panels

## Tab System

Each pane supports multiple tabs:

- **Add Tab**: Click `+` button or press `Ctrl+T`
- **Close Tab**: Press `Ctrl+W`
- **Switch Tabs**: Click tab or use `Ctrl+PageUp/PageDown`
- **Tab Labels**: Show truncated directory name (max 12 chars)
- **New Tab Location**: Opens in same directory as current tab

## Tab Drag & Drop

Tabs can be moved between panes:

1. **Start Drag**: Click and hold on the path header (shows `≡` icon)
2. **Visual Feedback**: Full-size panel replica follows the cursor
3. **Drop**: Release mouse on the target pane
4. **Cancel**: Press `Escape` to cancel drag

The drag indicator:
- Matches the exact size of the source panel
- Shows complete file listing
- Highlights with blue border during drag

## File Navigation

### Keyboard Controls

| Key | Action |
|-----|--------|
| `Up/Down` | Move cursor |
| `Enter` | Enter directory / Open file |
| `Backspace` | Go to parent directory |
| `Home` | Jump to first entry |
| `End` | Jump to last entry |
| `Left` | Jump to first entry |
| `Right` | Jump to last entry |
| `Insert` | Toggle file selection |
| `Ctrl+R` | Refresh all panels |

### Mouse Controls

- **Single Click**: Select entry (move cursor)
- **Double Click**: Enter directory / Open file

## File Selection

- **Toggle Selection**: Press `Insert` on any file/directory
- **Selection Color**: Red background for selected items
- **Selection Persistence**: Selections preserved when navigating
- **Parent Directory**: Cannot be selected (`..` entry)

## File Display

Each entry shows:
- **Icon**: `/` prefix for directories
- **Name**: File or directory name
- **Size**: Human-readable size (e.g., "1.5 KB") or `<DIR>`
- **Date**: Modification date in `YYYY-MM-DD HH:MM` format

### Sorting

Entries are sorted:
1. Directories first
2. Alphabetically (case-insensitive)

## Embedded Terminal

A built-in terminal panel at the bottom:

- **Toggle**: Press `Ctrl+O` to focus/unfocus terminal
- **Shell**: Uses user's `$SHELL` environment variable
- **Exit**: Press `Escape` to return to file panels

### Terminal Keys

| Key | Action |
|-----|--------|
| `Ctrl+C` | Interrupt |
| `Ctrl+D` | EOF |
| `Ctrl+Z` | Suspend |
| `Ctrl+L` | Clear screen |
| Arrow keys | Navigation |
| `Home/End` | Line navigation |

## Visual Theme

- **Dark Theme**: Dark background with contrasting text
- **Directory Color**: Blue text for directories
- **File Color**: Light gray text for files
- **Cursor**: Highlighted background (blue when active, gray when inactive)
- **Selection**: Red background for selected items

## Status Bar

Bottom status bar shows:
- **Left**: Current file info (name, size, or `<DIR>`)
- **Right**: Quick keyboard shortcut reference
