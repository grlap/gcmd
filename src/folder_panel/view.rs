use iced::widget::{Column, column, container, mouse_area, pane_grid, row, scrollable, text};
use iced::{Element, Length};

use super::model::{FileEntry, FolderPanel};
use crate::app::Message;
use crate::panel::{Panel, PanelEntry};
use crate::text_utils::{max_chars_for_width, truncate};

const ENTRY_FONT_SIZE: f32 = 15.0;

/// Calculate max filename characters based on available panel width
/// Name column gets 5/10 of the width
fn calc_max_name_chars(panel_width: f32) -> usize {
    let name_column_width = panel_width * 0.5; // FillPortion(5) out of 10
    let padding = 20.0; // Account for icon, padding, spacing
    let available = (name_column_width - padding).max(50.0);
    max_chars_for_width(available, ENTRY_FONT_SIZE)
}

/// View panel with pane information for mouse events
pub fn view_panel_with_pane(
    panel: &FolderPanel,
    pane: pane_grid::Pane,
    panel_width: f32,
) -> Element<'_, Message> {
    // Header is now in the pane_grid title bar, so just show entries
    let entries = view_entries_with_pane(panel, pane, panel_width);

    let panel_content = column![entries]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    let border_color = if panel.is_active {
        iced::Color::from_rgb(0.3, 0.5, 0.8)
    } else {
        iced::Color::from_rgb(0.3, 0.3, 0.3)
    };

    container(panel_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: border_color,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

impl Panel for FolderPanel {
    type Entry = FileEntry;

    fn entries(&self) -> &[Self::Entry] {
        &self.entries
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn title(&self) -> String {
        self.current_dir.to_string_lossy().to_string()
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
        }
    }

    fn move_to_top(&mut self) {
        self.cursor = 0;
    }

    fn move_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
        }
    }

    fn set_cursor(&mut self, index: usize) {
        if index < self.entries.len() {
            self.cursor = index;
        }
    }

    fn enter_selected(&mut self) -> bool {
        if let Some(entry) = self.entries.get(self.cursor).cloned() {
            // In search results mode, Enter navigates to the result's location
            if self.search_results_mode {
                let target_dir = if entry.is_dir {
                    entry.path.clone()
                } else {
                    entry.path.parent().unwrap_or(&entry.path).to_path_buf()
                };
                let target_name = entry.name.clone();
                self.search_results_mode = false;
                self.navigate_to(target_dir);
                // Select the file/folder in the directory listing
                if !entry.is_dir {
                    if let Some(idx) = self.entries.iter().position(|e| e.name == target_name) {
                        self.cursor = idx;
                    }
                }
                return true;
            }

            if entry.is_dir {
                // Check if navigating to parent (..)
                let is_parent = entry.name == "..";
                let current_dir_name = if is_parent {
                    self.current_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                } else {
                    None
                };

                self.current_dir = entry.path.clone();
                self.refresh();

                // New directory: start at top (refresh preserves old cursor/scroll)
                self.cursor = 0;
                self.scroll_offset = 0;

                // If we went to parent, select the directory we came from
                if let Some(name) = current_dir_name {
                    if let Some(idx) = self.entries.iter().position(|e| e.name == name) {
                        self.cursor = idx;
                    }
                }
                return true;
            }
        }
        false
    }

    fn go_parent(&mut self) -> bool {
        // In search results mode, Backspace exits search and returns to normal dir
        if self.search_results_mode {
            self.exit_search_mode();
            return true;
        }

        // Remember the current directory name before navigating up
        let current_dir_name = self
            .current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string());

        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh();

            // New directory: start at top (refresh preserves old cursor/scroll)
            self.cursor = 0;
            self.scroll_offset = 0;

            // Find and select the directory we just left
            if let Some(name) = current_dir_name {
                if let Some(idx) = self.entries.iter().position(|e| e.name == name) {
                    self.cursor = idx;
                }
            }
            return true;
        }
        false
    }

    fn toggle_selection(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.cursor) {
            if entry.name != ".." {
                entry.selected = !entry.selected;
            }
        }
        self.move_down();
    }

    fn refresh(&mut self) {
        if self.search_results_mode {
            return; // Don't reload directory when showing search results
        }
        let was_active = self.is_active;
        let old_cursor = self.cursor;
        let old_scroll = self.scroll_offset;
        let scrollable_id = self.scrollable_id.clone();
        *self = Self::new(self.current_dir.clone());
        self.is_active = was_active;
        self.cursor = old_cursor.min(self.entries.len().saturating_sub(1));
        self.scroll_offset = old_scroll;
        self.scrollable_id = scrollable_id;
    }

    fn view(&self) -> Element<'_, Message> {
        view_panel(self)
    }
}

pub fn view_panel(panel: &FolderPanel) -> Element<'_, Message> {
    let header = view_header(panel);
    let entries = view_entries(panel);

    let panel_content = column![header, entries]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    let border_color = if panel.is_active {
        iced::Color::from_rgb(0.3, 0.5, 0.8)
    } else {
        iced::Color::from_rgb(0.3, 0.3, 0.3)
    };

    container(panel_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: border_color,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_header(panel: &FolderPanel) -> Element<'_, Message> {
    container(text(panel.title()).size(16))
        .width(Length::Fill)
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.15, 0.15, 0.2).into()),
            ..Default::default()
        })
        .into()
}

fn view_entries(panel: &FolderPanel) -> Element<'_, Message> {
    let entries: Vec<Element<Message>> = panel
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| view_entry(entry, idx == panel.cursor, panel.is_active))
        .collect();

    let content = Column::with_children(entries)
        .spacing(0)
        .width(Length::Fill);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_entry(
    entry: &FileEntry,
    is_cursor: bool,
    is_active_panel: bool,
) -> Element<'static, Message> {
    let name_color = if entry.is_dir() {
        iced::Color::from_rgb(0.4, 0.7, 1.0)
    } else {
        iced::Color::from_rgb(0.9, 0.9, 0.9)
    };

    let bg_color = if is_cursor && is_active_panel {
        iced::Color::from_rgb(0.2, 0.3, 0.5)
    } else if is_cursor {
        iced::Color::from_rgb(0.2, 0.2, 0.3)
    } else if entry.is_selected() {
        iced::Color::from_rgb(0.3, 0.2, 0.2)
    } else {
        iced::Color::TRANSPARENT
    };

    let icon = if entry.is_dir() { "/" } else { " " };
    // Truncate long filenames to prevent overflow into other columns
    // 28 chars fits comfortably in dual-pane layout at 1200px window
    let truncated_name = truncate(entry.name(), 28);
    let name_display = format!("{}{}", icon, truncated_name);

    let entry_row = row![
        text(name_display)
            .size(15)
            .width(Length::FillPortion(5))
            .color(name_color),
        text(entry.size_display())
            .size(15)
            .width(Length::FillPortion(2)),
        text(entry.date_display())
            .size(15)
            .width(Length::FillPortion(3)),
    ]
    .spacing(8)
    .padding(2);

    container(entry_row)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        })
        .into()
}

fn view_entries_with_pane(
    panel: &FolderPanel,
    pane: pane_grid::Pane,
    panel_width: f32,
) -> Element<'_, Message> {
    let max_chars = calc_max_name_chars(panel_width);
    let search_base = panel.search_base_dir.clone();
    let entries: Vec<Element<Message>> = panel
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            view_entry_with_pane(entry, idx, idx == panel.cursor, panel.is_active, pane, max_chars, &search_base)
        })
        .collect();

    let content = Column::with_children(entries)
        .spacing(0)
        .width(Length::Fill);

    scrollable(content)
        .id(panel.scrollable_id.clone())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_entry_with_pane(
    entry: &FileEntry,
    entry_index: usize,
    is_cursor: bool,
    is_active_panel: bool,
    pane: pane_grid::Pane,
    max_name_chars: usize,
    search_base: &Option<std::path::PathBuf>,
) -> Element<'static, Message> {
    let name_color = if entry.is_dir() {
        iced::Color::from_rgb(0.4, 0.7, 1.0)
    } else {
        iced::Color::from_rgb(0.9, 0.9, 0.9)
    };

    let bg_color = if is_cursor && is_active_panel {
        iced::Color::from_rgb(0.2, 0.3, 0.5)
    } else if is_cursor {
        iced::Color::from_rgb(0.2, 0.2, 0.3)
    } else if entry.is_selected() {
        iced::Color::from_rgb(0.3, 0.2, 0.2)
    } else {
        iced::Color::TRANSPARENT
    };

    let icon = if entry.is_dir() { "/" } else { " " };
    // In search mode, show path relative to search directory
    let display_name = if let Some(base) = search_base {
        entry
            .path
            .strip_prefix(base)
            .unwrap_or(&entry.path)
            .to_string_lossy()
            .to_string()
    } else {
        entry.name().to_string()
    };
    let truncated_name = truncate(&display_name, max_name_chars);
    let name_display = format!("{}{}", icon, truncated_name);

    let entry_row = row![
        text(name_display)
            .size(15)
            .width(Length::FillPortion(5))
            .color(name_color),
        text(entry.size_display())
            .size(15)
            .width(Length::FillPortion(2)),
        text(entry.date_display())
            .size(15)
            .width(Length::FillPortion(3)),
    ]
    .spacing(8)
    .padding(2);

    let entry_container = container(entry_row)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        });

    // Single click selects, double click activates (enters directory)
    mouse_area(entry_container)
        .on_press(Message::SelectEntry { pane, entry_index })
        .on_double_click(Message::ActivateEntry { pane, entry_index })
        .into()
}
