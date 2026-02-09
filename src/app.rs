#[cfg(windows)]
use std::os::windows::process::CommandExt;

use iced::keyboard::key::Named;
use iced::widget::{Space, column, container, mouse_area, pane_grid, row, scrollable, stack, text};
use iced::window;
use iced::{
    Element, Event, Length, Point, Size, Subscription, Task, Theme, event, keyboard, mouse,
};

use crate::file_viewer::FileViewer;
use crate::file_viewer::view_file_viewer;
use crate::folder_panel::FolderPanel;
use crate::folder_panel::view::view_panel_with_pane;
use crate::panel::{Panel, PanelEntry};
use crate::tab_container::TabContainer;
use crate::tab_container::view_tab_container;
use crate::terminal_panel::view::view_terminal;
use crate::terminal_panel::{TerminalKey, TerminalPanel};
use crate::text_utils::{max_chars_for_width, truncate};

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    // Tab actions
    SelectTab {
        pane: pane_grid::Pane,
        tab_index: usize,
    },
    AddTab {
        pane: pane_grid::Pane,
    },
    CloseTab {
        pane: pane_grid::Pane,
        tab_index: usize,
    },
    // Tab drag between panes (drag active tab from path header)
    TabDragStart {
        pane: pane_grid::Pane,
        tab_index: usize,
    },
    TabDragFromHeader {
        pane: pane_grid::Pane,
    },
    TabDropOnPane {
        target_pane: pane_grid::Pane,
    },
    TabDragCancel,
    // File panel actions
    SelectEntry {
        pane: pane_grid::Pane,
        entry_index: usize,
    },
    ActivateEntry {
        pane: pane_grid::Pane,
        entry_index: usize,
    },
    // Pane grid events
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    // Terminal
    TerminalTick,
}

/// Tracks a tab being dragged between panes
#[derive(Debug, Clone)]
pub struct DraggingTab {
    pub source_pane: pane_grid::Pane,
    pub tab_index: usize,
    pub panel: FolderPanel, // Clone of the panel being dragged
    pub mouse_pos: Point,
}

pub struct App {
    panes: pane_grid::State<TabContainer>,
    focus_pane: pane_grid::Pane,
    terminal: TerminalPanel,
    focus: Focus,
    dragging_tab: Option<DraggingTab>,
    window_size: Size,
    command_line: String,
    file_viewer: Option<FileViewer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Focus {
    #[default]
    Panel,
    Terminal,
    FileViewer,
}

impl Default for App {
    fn default() -> Self {
        // Create initial pane with left panel
        let (mut panes, first_pane) = pane_grid::State::new(TabContainer::new());

        // Split to create right panel
        let _second_pane = panes
            .split(pane_grid::Axis::Vertical, first_pane, TabContainer::new())
            .expect("Failed to split pane");

        // Set left panel as focused
        if let Some(left_container) = panes.get_mut(first_pane) {
            left_container.set_focused(true);
        }

        Self {
            panes,
            focus_pane: first_pane,
            terminal: TerminalPanel::default(),
            focus: Focus::Panel,
            dragging_tab: None,
            window_size: Size::new(1200.0, 800.0), // Default, will be updated on resize
            command_line: String::new(),
            file_viewer: None,
        }
    }
}

impl App {
    fn active_tab_container_mut(&mut self) -> Option<&mut TabContainer> {
        self.panes.get_mut(self.focus_pane)
    }

    fn active_tab_container_ref(&self) -> Option<&TabContainer> {
        self.panes.get(self.focus_pane)
    }

    fn set_focus_to_pane(&mut self, pane: pane_grid::Pane) {
        // Unfocus all panes
        for (_, container) in self.panes.iter_mut() {
            container.set_focused(false);
        }
        // Focus the selected pane
        if let Some(container) = self.panes.get_mut(pane) {
            container.set_focused(true);
        }
        self.focus_pane = pane;
        self.focus = Focus::Panel;
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Only scroll if cursor is outside the visible range
    fn scroll_if_needed(&mut self) -> Task<Message> {
        if let Some(container) = self.active_tab_container_mut() {
            let panel = container.active_panel_mut();
            if !panel.is_cursor_visible() {
                panel.update_scroll_offset_from_cursor();
                let cursor = panel.cursor;
                let total = panel.entries.len();
                if total == 0 {
                    return Task::none();
                }
                let ratio = cursor as f32 / total.saturating_sub(1).max(1) as f32;
                return iced::widget::operation::snap_to(
                    panel.scrollable_id.clone(),
                    scrollable::RelativeOffset { x: 0.0, y: ratio },
                );
            }
        }
        Task::none()
    }

    /// Calculate visible rows based on window height
    fn visible_rows(&self) -> usize {
        // Entry row: text size 15 (~19px with Fira Code) + row padding 2*2 = ~23px
        let row_height = 23.0;
        // Chrome: path header(33) + tab bar(27) + panel border(4) + command line(30) + status bar(26)
        let chrome_height = 120.0;
        let available_height = (self.window_size.height - chrome_height).max(row_height);
        // Floor division already discards the partial row at the bottom
        (available_height / row_height) as usize
    }

    /// Snap scrollable to match the stored scroll_offset (restore scroll position)
    fn restore_scroll_position(&self) -> Task<Message> {
        if let Some(container) = self.active_tab_container_ref() {
            let panel = container.active_panel();
            let total = panel.entries.len();
            let visible_rows = self.visible_rows();
            let max_scroll_row = total.saturating_sub(visible_rows);

            let ratio = if max_scroll_row > 0 {
                panel.scroll_offset as f32 / max_scroll_row as f32
            } else {
                0.0
            };

            return iced::widget::operation::snap_to(
                panel.scrollable_id.clone(),
                scrollable::RelativeOffset { x: 0.0, y: ratio },
            );
        }
        Task::none()
    }

    /// Calculate visible lines for file viewer based on full window height
    fn viewer_visible_lines(&self) -> usize {
        // Line height: text size 14 monospace ≈ 18px
        let line_height = 18.0;
        // Chrome: title bar(~26) + help bar(~24) + content padding(8) + border(4)
        let chrome_height = 62.0;
        let available = (self.window_size.height - chrome_height).max(line_height);
        (available / line_height) as usize
    }

    /// Scroll only if cursor is outside the visible range.
    /// Scrolls by the minimum amount to keep cursor visible (no centering).
    fn scroll_if_cursor_not_visible(&mut self) -> Task<Message> {
        let visible_rows = self.visible_rows();

        if let Some(container) = self.active_tab_container_mut() {
            let panel = container.active_panel_mut();
            let cursor = panel.cursor;
            let total = panel.entries.len();

            if total == 0 {
                return Task::none();
            }

            let old_offset = panel.scroll_offset;

            // If cursor moved above visible area, adjust scroll_offset
            if cursor < panel.scroll_offset {
                panel.scroll_offset = cursor;
            }
            // If cursor moved below visible area, adjust scroll_offset
            else if cursor >= panel.scroll_offset + visible_rows {
                panel.scroll_offset = cursor.saturating_sub(visible_rows - 1);
            }

            // Only send a scroll command if offset actually changed
            if panel.scroll_offset == old_offset {
                return Task::none();
            }

            let max_scroll_row = total.saturating_sub(visible_rows);
            let ratio = if max_scroll_row > 0 {
                panel.scroll_offset as f32 / max_scroll_row as f32
            } else {
                0.0
            };

            let scrollable_id = panel.scrollable_id.clone();
            return iced::widget::operation::snap_to(
                scrollable_id,
                scrollable::RelativeOffset { x: 0.0, y: ratio },
            );
        }
        Task::none()
    }

    /// Always scroll to center cursor (used for page up/down, tab switch)
    fn scroll_to_cursor(&mut self) -> Task<Message> {
        let visible_rows = self.visible_rows();

        if let Some(container) = self.active_tab_container_mut() {
            let panel = container.active_panel_mut();
            let cursor = panel.cursor;
            let total = panel.entries.len();
            if total == 0 {
                return Task::none();
            }

            let half_visible = visible_rows / 2;
            let scroll_to_row = cursor.saturating_sub(half_visible);
            let max_scroll_row = total.saturating_sub(visible_rows);
            let target_row = scroll_to_row.min(max_scroll_row);

            panel.scroll_offset = target_row;

            let ratio = if max_scroll_row > 0 {
                target_row as f32 / max_scroll_row as f32
            } else {
                0.0
            };

            return iced::widget::operation::snap_to(
                panel.scrollable_id.clone(),
                scrollable::RelativeOffset { x: 0.0, y: ratio },
            );
        }
        Task::none()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Pane grid events
            Message::PaneClicked(pane) => {
                self.set_focus_to_pane(pane);
            }
            Message::PaneDragged(_) => {
                // Pane dragging disabled - use path header to drag tabs instead
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }

            // Mouse click messages
            Message::SelectTab { pane, tab_index } => {
                self.set_focus_to_pane(pane);
                if let Some(container) = self.panes.get_mut(pane) {
                    container.select_tab(tab_index);
                }
            }
            Message::AddTab { pane } => {
                self.set_focus_to_pane(pane);
                if let Some(container) = self.panes.get_mut(pane) {
                    container.add_tab();
                }
            }
            Message::CloseTab { pane, tab_index } => {
                if let Some(container) = self.panes.get_mut(pane) {
                    container.select_tab(tab_index);
                    container.close_tab();
                }
            }

            // Tab drag between panes
            Message::TabDragStart { pane, tab_index } => {
                if let Some(container) = self.panes.get(pane) {
                    if let Some(panel) = container.tabs().get(tab_index) {
                        self.dragging_tab = Some(DraggingTab {
                            source_pane: pane,
                            tab_index,
                            panel: panel.clone(),
                            mouse_pos: Point::ORIGIN,
                        });
                    }
                }
            }
            Message::TabDragFromHeader { pane } => {
                // Drag the active tab from path header
                if let Some(container) = self.panes.get(pane) {
                    let tab_index = container.active_tab_index();
                    self.dragging_tab = Some(DraggingTab {
                        source_pane: pane,
                        tab_index,
                        panel: container.active_panel().clone(),
                        mouse_pos: Point::ORIGIN,
                    });
                }
            }
            Message::TabDropOnPane { target_pane } => {
                if let Some(dragging) = self.dragging_tab.take() {
                    // Don't do anything if dropping on same pane
                    if dragging.source_pane != target_pane {
                        // Get the tab from source pane
                        if let Some(source_container) = self.panes.get_mut(dragging.source_pane) {
                            if let Some(tab) = source_container.take_tab(dragging.tab_index) {
                                // Add to target pane
                                if let Some(target_container) = self.panes.get_mut(target_pane) {
                                    target_container.add_existing_tab(tab);
                                    self.set_focus_to_pane(target_pane);
                                }
                            }
                        }
                    }
                }
            }
            Message::TabDragCancel => {
                self.dragging_tab = None;
            }

            Message::SelectEntry { pane, entry_index } => {
                self.set_focus_to_pane(pane);
                if let Some(container) = self.panes.get_mut(pane) {
                    container.active_panel_mut().set_cursor(entry_index);
                }
            }
            Message::ActivateEntry { pane, entry_index } => {
                self.set_focus_to_pane(pane);
                if let Some(container) = self.panes.get_mut(pane) {
                    let panel = container.active_panel_mut();
                    panel.set_cursor(entry_index);
                    panel.enter_selected();
                }
            }

            // Keyboard events
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            })) => {
                // Escape cancels drag, closes viewer, clears command line, or returns focus to panels
                if key == keyboard::Key::Named(Named::Escape) {
                    if self.file_viewer.is_some() {
                        self.file_viewer = None;
                        self.focus = Focus::Panel;
                        self.command_line.clear();
                        return self.restore_scroll_position();
                    }
                    if self.dragging_tab.is_some() {
                        self.dragging_tab = None;
                        return Task::none();
                    }
                    if !self.command_line.is_empty() {
                        self.command_line.clear();
                        return Task::none();
                    }
                    if self.focus == Focus::Terminal {
                        self.set_focus_to_pane(self.focus_pane);
                    }
                    return Task::none();
                }

                // F3 opens/closes file viewer
                if key == keyboard::Key::Named(Named::F3) {
                    if self.file_viewer.is_some() {
                        // Close viewer
                        self.file_viewer = None;
                        self.focus = Focus::Panel;
                        self.command_line.clear();
                    } else if self.focus == Focus::Panel {
                        // Open viewer for selected file
                        if let Some(container) = self.active_tab_container_ref() {
                            if let Some(entry) = container.active_panel().current_entry() {
                                if !entry.is_dir() {
                                    let visible_lines = self.viewer_visible_lines();
                                    self.file_viewer =
                                        Some(FileViewer::new(entry.path.clone(), visible_lines));
                                    self.focus = Focus::FileViewer;
                                }
                            }
                        }
                    }
                    return self.restore_scroll_position();
                }

                // Global shortcuts
                if let keyboard::Key::Character(ref c) = key {
                    // Ctrl+O toggles terminal focus
                    if c.as_str() == "o" && modifiers.control() {
                        self.toggle_terminal_focus();
                        return Task::none();
                    }
                    // Ctrl+T new tab
                    if c.as_str() == "t" && modifiers.control() && self.focus != Focus::Terminal {
                        if let Some(container) = self.active_tab_container_mut() {
                            container.add_tab();
                        }
                        return Task::none();
                    }
                    // Ctrl+W close tab
                    if c.as_str() == "w" && modifiers.control() && self.focus != Focus::Terminal {
                        if let Some(container) = self.active_tab_container_mut() {
                            container.close_tab();
                        }
                        return Task::none();
                    }
                    // Ctrl+Tab next tab
                    if c.as_str() == "\t" && modifiers.control() && self.focus != Focus::Terminal {
                        if let Some(container) = self.active_tab_container_mut() {
                            container.next_tab();
                        }
                        return Task::none();
                    }
                }

                // Ctrl+PageDown / Ctrl+PageUp for tab switching
                if modifiers.control() && self.focus != Focus::Terminal {
                    match key {
                        keyboard::Key::Named(Named::PageDown) => {
                            if let Some(container) = self.active_tab_container_mut() {
                                container.next_tab();
                            }
                            return Task::none();
                        }
                        keyboard::Key::Named(Named::PageUp) => {
                            if let Some(container) = self.active_tab_container_mut() {
                                container.prev_tab();
                            }
                            return Task::none();
                        }
                        _ => {}
                    }
                }

                // Route to terminal if focused
                if self.focus == Focus::Terminal {
                    self.handle_terminal_key(&key, &modifiers);
                    return Task::none();
                }

                // Route to file viewer if focused
                if self.focus == Focus::FileViewer {
                    if let Some(ref mut viewer) = self.file_viewer {
                        match key {
                            keyboard::Key::Named(Named::ArrowUp) => viewer.scroll_up(1),
                            keyboard::Key::Named(Named::ArrowDown) => viewer.scroll_down(1),
                            keyboard::Key::Named(Named::PageUp) => viewer.page_up(),
                            keyboard::Key::Named(Named::PageDown) => viewer.page_down(),
                            keyboard::Key::Named(Named::Home) => viewer.scroll_to_top(),
                            keyboard::Key::Named(Named::End) => viewer.scroll_to_bottom(),
                            _ => {}
                        }
                    }
                    return Task::none();
                }

                // Panel navigation - these need scroll updates
                match key {
                    keyboard::Key::Named(Named::Tab) => {
                        self.switch_panel();
                        return self.scroll_to_cursor();
                    }
                    keyboard::Key::Named(Named::ArrowUp) => {
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().move_up();
                        }
                        return self.scroll_if_cursor_not_visible();
                    }
                    keyboard::Key::Named(Named::ArrowDown) => {
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().move_down();
                        }
                        return self.scroll_if_cursor_not_visible();
                    }
                    keyboard::Key::Named(Named::ArrowLeft | Named::PageUp) => {
                        // Page up - move by visible rows
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().page_up();
                        }
                        return self.scroll_to_cursor();
                    }
                    keyboard::Key::Named(Named::ArrowRight | Named::PageDown) => {
                        // Page down - move by visible rows
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().page_down();
                        }
                        return self.scroll_to_cursor();
                    }
                    keyboard::Key::Named(Named::Enter) => {
                        if self.command_line.starts_with("cd ") {
                            let path = self.command_line[3..].trim().to_string();
                            if path == "~" || path.starts_with("~/") || path.starts_with("~\\") {
                                // Home directory path
                                if let Some(home) = dirs::home_dir() {
                                    let resolved = if path == "~" {
                                        home
                                    } else {
                                        home.join(&path[2..])
                                    };
                                    if resolved.is_dir() {
                                        if let Some(c) = self.active_tab_container_mut() {
                                            c.active_panel_mut().navigate_to(resolved);
                                        }
                                    }
                                }
                            } else if self.is_absolute_path(&path) {
                                // Absolute path - navigate directly
                                let abs_path = self.resolve_absolute_path(&path);
                                if abs_path.is_dir() {
                                    if let Some(c) = self.active_tab_container_mut() {
                                        c.active_panel_mut().navigate_to(abs_path);
                                    }
                                }
                                // If path doesn't exist, just clear command line (ignore)
                            } else if !path.is_empty() {
                                // Relative path - only enter if selected entry matches
                                if let Some(c) = self.active_tab_container_mut() {
                                    let panel = c.active_panel_mut();
                                    // Check if current entry name starts with the typed path
                                    let should_enter = panel
                                        .current_entry()
                                        .map(|e| {
                                            e.is_dir
                                                && e.name
                                                    .to_lowercase()
                                                    .starts_with(&path.to_lowercase())
                                        })
                                        .unwrap_or(false);
                                    if should_enter {
                                        panel.enter_selected();
                                    }
                                    // If no match, just clear command line (ignore)
                                }
                            }
                            self.command_line.clear();
                            return self.scroll_to_cursor();
                        }
                        if !self.command_line.is_empty() {
                            // Execute command in terminal
                            self.execute_command_line();
                            return Task::none();
                        }
                        let navigated = self
                            .active_tab_container_mut()
                            .map(|c| c.active_panel_mut().enter_selected())
                            .unwrap_or(false);
                        if navigated {
                            return self.scroll_to_cursor();
                        }
                    }
                    keyboard::Key::Named(Named::Home) => {
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().move_to_top();
                        }
                        return self.scroll_to_cursor();
                    }
                    keyboard::Key::Named(Named::End) => {
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().move_to_bottom();
                        }
                        return self.scroll_to_cursor();
                    }
                    keyboard::Key::Named(Named::Insert) => {
                        if let Some(c) = self.active_tab_container_mut() {
                            c.active_panel_mut().toggle_selection();
                        }
                        return self.scroll_if_cursor_not_visible();
                    }
                    keyboard::Key::Character(ref c) if c.as_str() == "r" && modifiers.control() => {
                        // Refresh all tabs in all panes
                        for (_, container) in self.panes.iter_mut() {
                            for panel in container.tabs_mut() {
                                panel.refresh();
                            }
                        }
                    }
                    keyboard::Key::Named(Named::Backspace) => {
                        // Delete last character from command line
                        self.command_line.pop();
                        self.search_from_command_line();
                        return self.scroll_if_cursor_not_visible();
                    }
                    keyboard::Key::Named(Named::Space) => {
                        if self.command_line.is_empty() {
                            // Toggle selection when command line is empty
                            if let Some(c) = self.active_tab_container_mut() {
                                c.active_panel_mut().toggle_selection();
                            }
                            return self.scroll_if_cursor_not_visible();
                        } else {
                            // Add space to command line
                            self.command_line.push(' ');
                            self.search_from_command_line();
                            return self.scroll_if_cursor_not_visible();
                        }
                    }
                    keyboard::Key::Character(ref c) if !modifiers.control() && !modifiers.alt() => {
                        // Append character to command line
                        // iced sends lowercase even with shift - convert if shift is pressed
                        let char_to_add = if modifiers.shift() {
                            // Handle shift+key for special characters
                            match c.as_str() {
                                ";" => ":".to_string(),
                                "`" => "~".to_string(),
                                "1" => "!".to_string(),
                                "2" => "@".to_string(),
                                "3" => "#".to_string(),
                                "4" => "$".to_string(),
                                "5" => "%".to_string(),
                                "6" => "^".to_string(),
                                "7" => "&".to_string(),
                                "8" => "*".to_string(),
                                "9" => "(".to_string(),
                                "0" => ")".to_string(),
                                "-" => "_".to_string(),
                                "=" => "+".to_string(),
                                "[" => "{".to_string(),
                                "]" => "}".to_string(),
                                "\\" => "|".to_string(),
                                "'" => "\"".to_string(),
                                "," => "<".to_string(),
                                "." => ">".to_string(),
                                "/" => "?".to_string(),
                                _ => c.as_str().to_uppercase(),
                            }
                        } else {
                            c.as_str().to_string()
                        };
                        self.command_line.push_str(&char_to_add);
                        self.search_from_command_line();
                        return self.scroll_if_cursor_not_visible();
                    }
                    _ => {}
                }
            }
            // Track mouse position during tab drag
            Message::EventOccurred(Event::Mouse(mouse::Event::CursorMoved { position })) => {
                if let Some(ref mut dragging) = self.dragging_tab {
                    dragging.mouse_pos = position;
                }
            }
            // Cancel drag if mouse released outside drop zones
            Message::EventOccurred(Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))) => {
                // If still dragging when release happens here, it means we're outside drop zones
                // The drop zones handle their own releases via on_release
                // This is a fallback - cancel the drag
                if self.dragging_tab.is_some() {
                    self.dragging_tab = None;
                }
            }
            // Track window size for drag indicator sizing
            Message::EventOccurred(Event::Window(window::Event::Resized(size))) => {
                self.window_size = size;
            }
            Message::TerminalTick => {
                self.terminal.poll_output();
            }
            _ => {}
        }
        Task::none()
    }

    /// Check if a path is absolute (works on both Unix and Windows)
    fn is_absolute_path(&self, path: &str) -> bool {
        // Unix absolute path
        if path.starts_with('/') {
            return true;
        }
        // Windows absolute paths: C:\, D:\, etc. or just \ for current drive root
        if path.starts_with('\\') {
            return true;
        }
        // Windows drive letter path: C:\, D:/, etc.
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            return true;
        }
        false
    }

    /// Resolve an absolute path, handling Windows drive-relative paths like \
    fn resolve_absolute_path(&self, path: &str) -> std::path::PathBuf {
        // Windows: \ means root of current drive
        if path == "\\" || path == "/" {
            if let Some(container) = self.active_tab_container_ref() {
                let current = &container.active_panel().current_dir;
                // Get drive root (e.g., C:\)
                if let Some(prefix) = current.components().next() {
                    return std::path::PathBuf::from(prefix.as_os_str()).join("\\");
                }
            }
        }
        // Windows: \foo means current_drive:\foo
        if path.starts_with('\\') && !path.starts_with("\\\\") {
            if let Some(container) = self.active_tab_container_ref() {
                let current = &container.active_panel().current_dir;
                if let Some(prefix) = current.components().next() {
                    let drive = prefix.as_os_str().to_string_lossy();
                    return std::path::PathBuf::from(format!("{}{}", drive, path));
                }
            }
        }
        std::path::PathBuf::from(path)
    }

    /// Search for entries matching command line input
    /// Only triggers after user types "cd " - jumps cursor to matching folders
    /// Does NOT navigate - that happens on Enter
    fn search_from_command_line(&mut self) {
        if !self.command_line.starts_with("cd ") {
            return;
        }

        let path_str = self.command_line[3..].trim().to_string();

        if path_str.is_empty() {
            return;
        }

        // For incremental search, we only move the cursor to matching entries
        // in the current directory. Actual navigation happens on Enter.
        // Don't do incremental search for absolute paths - just wait for Enter.
        if !self.is_absolute_path(&path_str) {
            // Relative path: search in current directory
            if let Some(container) = self.active_tab_container_mut() {
                container.active_panel_mut().jump_to_folder(&path_str);
            }
        }
    }

    fn execute_command_line(&mut self) {
        let cmd = self.command_line.clone();
        self.command_line.clear();

        let cwd = self
            .active_tab_container_ref()
            .map(|c| c.active_panel().current_dir.clone());

        // "cmd" opens a new external terminal window
        if cmd.trim().eq_ignore_ascii_case("cmd") {
            let mut process = std::process::Command::new("cmd.exe");
            process.args(["/C", "start", "cmd.exe"]);
            process.creation_flags(0x08000000); // CREATE_NO_WINDOW - hide the intermediate cmd
            if let Some(dir) = cwd {
                process.current_dir(dir);
            }
            let _ = process.spawn();
            return;
        }

        // Everything else goes to the embedded PTY terminal
        if !self.terminal.is_running() {
            let _ = self.terminal.spawn_shell();
        }
        if let Some(dir) = cwd {
            self.terminal.set_working_dir(dir);
        }
        let input = format!("{}\n", cmd);
        self.terminal.send_input(&input);
    }

    fn toggle_terminal_focus(&mut self) {
        if self.focus == Focus::Terminal {
            // Return to last panel
            self.set_focus_to_pane(self.focus_pane);
        } else {
            // Enter terminal mode
            self.focus = Focus::Terminal;
            for (_, container) in self.panes.iter_mut() {
                container.set_focused(false);
            }

            // Start shell if not running
            if !self.terminal.is_running() {
                let _ = self.terminal.spawn_shell();
            }
        }
    }

    fn handle_terminal_key(&mut self, key: &keyboard::Key, modifiers: &keyboard::Modifiers) {
        let terminal_key = match key {
            keyboard::Key::Named(Named::Enter) => Some(TerminalKey::Enter),
            keyboard::Key::Named(Named::Backspace) => Some(TerminalKey::Backspace),
            keyboard::Key::Named(Named::Tab) => Some(TerminalKey::Tab),
            keyboard::Key::Named(Named::ArrowUp) => Some(TerminalKey::Up),
            keyboard::Key::Named(Named::ArrowDown) => Some(TerminalKey::Down),
            keyboard::Key::Named(Named::ArrowLeft) => Some(TerminalKey::Left),
            keyboard::Key::Named(Named::ArrowRight) => Some(TerminalKey::Right),
            keyboard::Key::Named(Named::Home) => Some(TerminalKey::Home),
            keyboard::Key::Named(Named::End) => Some(TerminalKey::End),
            keyboard::Key::Named(Named::Delete) => Some(TerminalKey::Delete),
            keyboard::Key::Character(c) => {
                if modifiers.control() {
                    match c.as_str() {
                        "c" => Some(TerminalKey::CtrlC),
                        "d" => Some(TerminalKey::CtrlD),
                        "z" => Some(TerminalKey::CtrlZ),
                        "l" => Some(TerminalKey::CtrlL),
                        _ => None,
                    }
                } else {
                    c.chars().next().map(TerminalKey::Char)
                }
            }
            _ => None,
        };

        if let Some(tk) = terminal_key {
            self.terminal.send_key(tk);
        }
    }

    fn switch_panel(&mut self) {
        if self.focus == Focus::Terminal {
            return;
        }

        // Get all pane IDs
        let pane_ids: Vec<pane_grid::Pane> = self.panes.iter().map(|(pane, _)| *pane).collect();

        if pane_ids.len() < 2 {
            return;
        }

        // Find current pane index and switch to next
        if let Some(current_idx) = pane_ids.iter().position(|&p| p == self.focus_pane) {
            let next_idx = (current_idx + 1) % pane_ids.len();
            self.set_focus_to_pane(pane_ids[next_idx]);
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let events = event::listen().map(Message::EventOccurred);
        if self.terminal.is_running() {
            Subscription::batch([
                events,
                iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::TerminalTick),
            ])
        } else {
            events
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Calculate panel width for filename truncation
        // Each pane gets roughly half the window width minus spacing
        let pane_count = self.panes.iter().count() as f32;
        let panel_width = (self.window_size.width - 10.0) / pane_count;

        let pane_grid_view =
            pane_grid::PaneGrid::new(&self.panes, |pane, tab_container, _is_maximized| {
                let is_focused = pane == self.focus_pane;
                let current_path = tab_container
                    .active_panel()
                    .current_dir
                    .to_string_lossy()
                    .to_string();

                // Path header - drag from here to move active tab to other pane
                let path_header_content =
                    container(row![text("≡ ").size(16), text(current_path).size(15),])
                        .padding([6, 8])
                        .width(Length::Fill)
                        .style(move |_theme| {
                            let bg = if is_focused {
                                iced::Color::from_rgb(0.2, 0.25, 0.4)
                            } else {
                                iced::Color::from_rgb(0.15, 0.15, 0.2)
                            };
                            container::Style {
                                background: Some(bg.into()),
                                ..Default::default()
                            }
                        });

                // Wrap path header in mouse_area to start tab drag and receive drops
                let path_header = mouse_area(path_header_content)
                    .on_press(Message::TabDragFromHeader { pane })
                    .on_release(Message::TabDropOnPane { target_pane: pane });

                // Full content: path header + tabs + file list
                let full_content =
                    column![path_header, view_tab_container(tab_container, pane, panel_width),]
                        .width(Length::Fill)
                        .height(Length::Fill);

                pane_grid::Content::new(full_content)
            })
            .on_click(Message::PaneClicked)
            .on_resize(10, Message::PaneResized)
            .spacing(2)
            .width(Length::Fill)
            .height(Length::Fill);

        let command_line = self.view_command_line();
        let status = self.view_status_bar();

        // Stack panels and terminal so panels widget tree is always stable (prevents scroll reset)
        // Terminal overlays panels when focused; panels are always Fill so scrollable state is preserved
        let terminal_height = if self.focus == Focus::Terminal {
            Length::Fill
        } else {
            Length::Fixed(0.0)
        };

        let terminal_overlay: Element<'_, Message> = container(view_terminal(&self.terminal))
            .width(Length::Fill)
            .height(terminal_height)
            .clip(true)
            .into();

        let main_area: Element<'_, Message> =
            stack![pane_grid_view, terminal_overlay].into();

        let content = column![main_area, command_line, status]
            .spacing(0)
            .height(Length::Fill);

        let main_content: Element<'_, Message> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // If dragging a tab, show floating panel (exact copy of the real panel)
        if let Some(ref dragging) = self.dragging_tab {
            // Path header (same as real panel)
            let current_path = dragging.panel.current_dir.to_string_lossy().to_string();
            let header = container(row![text("≡ ").size(16), text(current_path).size(15),])
                .padding([6, 8])
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.25, 0.4).into()),
                    ..Default::default()
                });

            // Combined panel - size calculated from actual window size
            // Panel width = (window_width - spacing) / 2
            // Panel height = window_height - terminal_height(~100) - status_bar(~30) - path_header(~30) - tab_bar(~30)
            let panel_width = (self.window_size.width - 4.0) / 2.0;
            let panel_height = self.window_size.height - 160.0;

            // Render the actual panel view (same as the real panel)
            let panel_view =
                view_panel_with_pane(&dragging.panel, dragging.source_pane, panel_width);

            let drag_indicator = container(
                column![header, panel_view]
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fixed(panel_width))
            .height(Length::Fixed(panel_height))
            .style(|_theme| container::Style {
                background: Some(iced::Color::from_rgba(0.1, 0.1, 0.12, 0.98).into()),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.4, 0.6, 0.9),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

            // Position near cursor, clamped to stay within window bounds
            let max_x = (self.window_size.width - panel_width).max(0.0);
            let max_y = (self.window_size.height - panel_height).max(0.0);
            let x_pos = (dragging.mouse_pos.x + 20.0).clamp(0.0, max_x);
            let y_pos = (dragging.mouse_pos.y - 20.0).clamp(0.0, max_y);

            let positioned = column![
                Space::new().height(Length::Fixed(y_pos)),
                row![Space::new().width(Length::Fixed(x_pos)), drag_indicator,]
            ];

            stack![main_content, positioned].into()
        } else if let Some(ref viewer) = self.file_viewer {
            // Show file viewer overlay (full screen)
            let viewer_content = view_file_viewer(viewer);

            let viewer_container = container(viewer_content)
                .width(Length::Fill)
                .height(Length::Fill);

            stack![main_content, viewer_container].into()
        } else {
            stack![main_content].into()
        }
    }

    fn view_command_line(&self) -> Element<'_, Message> {
        let prompt = "> ";
        let cursor = if self.focus == Focus::Panel {
            "▌"
        } else {
            ""
        };
        let display_text = format!("{}{}{}", prompt, self.command_line, cursor);

        container(text(display_text).size(15).font(iced::Font::MONOSPACE))
            .width(Length::Fill)
            .padding([4, 8])
            .style(|_theme| container::Style {
                background: Some(iced::Color::from_rgb(0.08, 0.08, 0.1).into()),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.25, 0.25, 0.3),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        const STATUS_FONT_SIZE: f32 = 14.0;
        let help = "Tab:Switch  Ctrl+T:NewTab  Ctrl+W:CloseTab  Ctrl+O:Terminal";

        // Calculate max filename chars based on window width
        // Help text is ~60 chars, leave room for size info (~15 chars), spacing
        let help_width = 60.0 * 8.0; // ~8px per char at size 13
        let size_width = 15.0 * 8.0;
        let available = (self.window_size.width - help_width - size_width - 40.0).max(100.0);
        let max_name_chars = max_chars_for_width(available, STATUS_FONT_SIZE);

        let focus_info = match self.focus {
            Focus::Panel => self
                .active_tab_container_ref()
                .and_then(|c| c.active_panel().current_entry())
                .map(|e| {
                    let truncated = truncate(e.name(), max_name_chars);
                    if e.is_dir() {
                        format!("{} <DIR>", truncated)
                    } else {
                        format!("{} ({})", truncated, e.size_display())
                    }
                })
                .unwrap_or_default(),
            Focus::Terminal => "[TERMINAL]".to_string(),
            Focus::FileViewer => self
                .file_viewer
                .as_ref()
                .map(|v| format!("[VIEWER] {}", v.file_name()))
                .unwrap_or_else(|| "[VIEWER]".to_string()),
        };

        container(
            row![
                text(focus_info).size(14).width(Length::Fill),
                text(help).size(13),
            ]
            .spacing(20),
        )
        .width(Length::Fill)
        .height(Length::Fixed(26.0))
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.1, 0.1, 0.15).into()),
            ..Default::default()
        })
        .into()
    }
}
