use std::fs;
use std::path::PathBuf;

use bytesize::ByteSize;
use chrono::{DateTime, Local};

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
}
