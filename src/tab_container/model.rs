use std::path::PathBuf;

use crate::folder_panel::FolderPanel;
use crate::panel::Panel;

pub struct TabContainer {
    tabs: Vec<FolderPanel>,
    active_tab: usize,
    is_focused: bool,
}

impl Default for TabContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl TabContainer {
    pub fn new() -> Self {
        let mut panel = FolderPanel::default();
        panel.set_active(false);

        Self {
            tabs: vec![panel],
            active_tab: 0,
            is_focused: false,
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        let panel = FolderPanel::new(path);

        Self {
            tabs: vec![panel],
            active_tab: 0,
            is_focused: false,
        }
    }

    pub fn tabs(&self) -> &[FolderPanel] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut [FolderPanel] {
        &mut self.tabs
    }

    pub fn active_tab_index(&self) -> usize {
        self.active_tab
    }

    pub fn active_panel(&self) -> &FolderPanel {
        &self.tabs[self.active_tab]
    }

    pub fn active_panel_mut(&mut self) -> &mut FolderPanel {
        &mut self.tabs[self.active_tab]
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        self.update_panel_active_states();
    }

    pub fn add_tab(&mut self) {
        // New tab starts at same directory as current
        let current_dir = self.tabs[self.active_tab].current_dir.clone();
        let panel = FolderPanel::new(current_dir);
        self.tabs.push(panel);
        self.active_tab = self.tabs.len() - 1;
        self.update_panel_active_states();
    }

    pub fn add_tab_with_path(&mut self, path: PathBuf) {
        let panel = FolderPanel::new(path);
        self.tabs.push(panel);
        self.active_tab = self.tabs.len() - 1;
        self.update_panel_active_states();
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
            self.update_panel_active_states();
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.update_panel_active_states();
        }
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.update_panel_active_states();
        }
    }

    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
            self.update_panel_active_states();
        }
    }

    fn update_panel_active_states(&mut self) {
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            tab.set_active(self.is_focused && i == self.active_tab);
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Remove and return a tab at the given index (for drag-and-drop)
    /// Returns None if it's the last tab (must keep at least one)
    pub fn take_tab(&mut self, index: usize) -> Option<FolderPanel> {
        if self.tabs.len() <= 1 || index >= self.tabs.len() {
            return None;
        }

        let tab = self.tabs.remove(index);

        // Adjust active tab index if needed
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.active_tab > index {
            self.active_tab -= 1;
        }

        self.update_panel_active_states();
        Some(tab)
    }

    /// Add an existing FolderPanel as a new tab (for drag-and-drop)
    pub fn add_existing_tab(&mut self, tab: FolderPanel) {
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.update_panel_active_states();
    }
}
