# gcmd Architecture

## Overview

gcmd is a dual-pane file manager written in Rust using the Iced 0.14 GUI framework. It follows the Elm architecture (Model-View-Update) pattern that Iced implements.

## Core Components

### Application State (`src/app.rs`)

The main `App` struct holds the entire application state:

```rust
pub struct App {
    panes: pane_grid::State<TabContainer>,  // Dual pane layout
    focus_pane: pane_grid::Pane,            // Currently focused pane
    terminal: TerminalPanel,                 // Embedded terminal
    focus: Focus,                            // Panel or Terminal focus
    dragging_tab: Option<DraggingTab>,       // Tab drag state
    window_size: Size,                       // Window dimensions
}
```

### Message Enum

All user interactions are represented as messages:

- **Tab actions**: `SelectTab`, `AddTab`, `CloseTab`
- **Tab dragging**: `TabDragStart`, `TabDragFromHeader`, `TabDropOnPane`, `TabDragCancel`
- **File panel actions**: `SelectEntry`, `ActivateEntry`
- **Pane grid events**: `PaneClicked`, `PaneDragged`, `PaneResized`
- **Keyboard/Mouse**: `EventOccurred(Event)`

## Module Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # Main App state, update, view
├── panel.rs             # Panel and PanelEntry traits
├── folder_panel/
│   ├── mod.rs           # Module exports
│   ├── model.rs         # FolderPanel, FileEntry structs
│   └── view.rs          # Panel rendering + Panel trait impl
├── tab_container/
│   ├── mod.rs           # Module exports
│   ├── model.rs         # TabContainer struct
│   └── view.rs          # Tab bar rendering
└── terminal_panel/
    ├── mod.rs           # Module exports
    ├── model.rs         # TerminalPanel, PTY management
    └── view.rs          # Terminal rendering
```

## Key Design Patterns

### Trait-Based Panel Abstraction

The `Panel` trait (`src/panel.rs`) defines a common interface for any panel type:

```rust
pub trait Panel: Default {
    type Entry: PanelEntry;

    fn entries(&self) -> &[Self::Entry];
    fn cursor(&self) -> usize;
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn enter_selected(&mut self) -> bool;
    // ... etc
}
```

This allows for future panel types (e.g., network, archive) to be added easily.

### Pane Grid Layout

The dual-pane layout uses Iced's `pane_grid` widget:

- Panes can be resized by dragging the divider
- Each pane contains a `TabContainer`
- Focus is tracked to highlight the active pane

### Tab Container

Each pane has a `TabContainer` that manages multiple tabs:

- Tabs can be added (`Ctrl+T`) and closed (`Ctrl+W`)
- Tab switching via `Ctrl+PageUp/Down`
- Tabs can be dragged between panes

## Data Flow

```
User Input → Event → Message → update() → State Change → view() → UI
```

1. **Events**: Keyboard/mouse events captured via `event::listen()`
2. **Messages**: Events converted to semantic `Message` variants
3. **Update**: `App::update()` handles messages, modifies state
4. **View**: `App::view()` renders UI based on current state

## File System Integration

`FolderPanel` reads directory contents using `std::fs`:

- Entries sorted: directories first, then alphabetically
- Parent directory (`..`) always shown when not at root
- Metadata includes size, modification date
- Selection state tracked per-entry

## Terminal Integration

The embedded terminal uses:

- `portable-pty`: PTY creation and management
- `vt100`: Terminal escape sequence parsing

Terminal is toggled with `Ctrl+O` and runs the user's shell.
