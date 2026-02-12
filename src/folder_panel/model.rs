use std::fs;
use std::path::PathBuf;

use bytesize::ByteSize;
use chrono::{DateTime, Local};
use iced::widget::Id;

use crate::panel::PanelEntry;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<DateTime<Local>>,
    pub selected: bool,
}

impl PanelEntry for FileEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn size_display(&self) -> String {
        if self.is_dir {
            "<DIR>".to_string()
        } else {
            ByteSize(self.size).to_string()
        }
    }

    fn date_display(&self) -> String {
        self.modified
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct FolderPanel {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub is_active: bool,
    pub scrollable_id: Id,
    pub visible_rows: usize,
    /// Track the visible range (first visible row index)
    pub scroll_offset: usize,
    /// When true, entries contain search results instead of directory listing
    pub search_results_mode: bool,
    /// The directory search was started from (for displaying relative paths)
    pub search_base_dir: Option<PathBuf>,
}

impl Default for FolderPanel {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self::new(home)
    }
}

impl FolderPanel {
    pub fn new(path: PathBuf) -> Self {
        let mut panel = Self {
            current_dir: path,
            entries: Vec::new(),
            cursor: 0,
            is_active: false,
            scrollable_id: Id::unique(),
            visible_rows: 20, // Default estimate, will be updated based on window size
            scroll_offset: 0,
            search_results_mode: false,
            search_base_dir: None,
        };
        panel.load_entries();
        panel
    }

    fn load_entries(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
                modified: None,
                selected: false,
            });
        }

        // Read directory contents
        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut entries: Vec<FileEntry> = read_dir
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let path = entry.path();
                    let metadata = entry.metadata().ok()?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    let modified = metadata.modified().ok().map(DateTime::<Local>::from);

                    Some(FileEntry {
                        name,
                        path,
                        is_dir: metadata.is_dir(),
                        size: metadata.len(),
                        modified,
                        selected: false,
                    })
                })
                .collect();

            // Sort: directories first, then by name
            entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.entries.extend(entries);
        }

        self.clamp_cursor();
    }

    fn clamp_cursor(&mut self) {
        if self.cursor >= self.entries.len() && !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
        }
    }

    pub fn set_cursor(&mut self, index: usize) {
        if index < self.entries.len() {
            self.cursor = index;
        }
    }

    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    /// Check if cursor is within the visible range
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor >= self.scroll_offset && self.cursor < self.scroll_offset + self.visible_rows
    }

    /// Update scroll offset to keep cursor visible (call after scrolling)
    pub fn update_scroll_offset_from_cursor(&mut self) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.cursor.saturating_sub(self.visible_rows - 1);
        }
    }

    /// Move cursor up by visible_rows (page up)
    pub fn page_up(&mut self) {
        if self.cursor > self.visible_rows {
            self.cursor -= self.visible_rows;
        } else {
            self.cursor = 0;
        }
        self.update_scroll_offset_from_cursor();
    }

    /// Move cursor down by visible_rows (page down)
    pub fn page_down(&mut self) {
        let new_cursor = self.cursor + self.visible_rows;
        if new_cursor < self.entries.len() {
            self.cursor = new_cursor;
        } else if !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
        }
        self.update_scroll_offset_from_cursor();
    }

    /// Jump to first entry matching the search string (case-insensitive prefix match)
    pub fn jump_to_match(&mut self, search: &str) -> bool {
        if search.is_empty() {
            return false;
        }
        let search_lower = search.to_lowercase();
        if let Some(idx) = self
            .entries
            .iter()
            .position(|e| e.name.to_lowercase().starts_with(&search_lower))
        {
            self.cursor = idx;
            return true;
        }
        false
    }

    /// Jump to first folder matching the search string (case-insensitive prefix match)
    pub fn jump_to_folder(&mut self, search: &str) -> bool {
        if search.is_empty() {
            return false;
        }
        let search_lower = search.to_lowercase();
        if let Some(idx) = self
            .entries
            .iter()
            .position(|e| e.is_dir && e.name.to_lowercase().starts_with(&search_lower))
        {
            self.cursor = idx;
            return true;
        }
        false
    }

    /// Navigate to an absolute path
    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_dir = path;
            self.cursor = 0;
            self.scroll_offset = 0;
            self.search_results_mode = false;
            self.load_entries();
        }
    }

    /// Replace entries with search results
    pub fn set_search_results(&mut self, results: Vec<FileEntry>, search_dir: PathBuf) {
        self.entries = results;
        self.cursor = 0;
        self.scroll_offset = 0;
        self.search_results_mode = true;
        self.search_base_dir = Some(search_dir);
    }

    /// Exit search results mode and return to normal directory listing
    pub fn exit_search_mode(&mut self) {
        self.search_results_mode = false;
        self.cursor = 0;
        self.scroll_offset = 0;
        self.load_entries();
    }
}
