use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};

use crate::folder_panel::FileEntry;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchField {
    NamePattern,
    FindText,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Input,
    Searching,
}

#[derive(Debug, Clone)]
pub struct SearchDialog {
    pub name_pattern: String,
    pub find_text: String,
    pub search_dir: PathBuf,
    pub active_field: SearchField,
    pub state: SearchState,
    /// Shared progress: current directory being searched (updated by search thread)
    pub progress: Arc<Mutex<String>>,
    /// Display copy of current search directory (updated by tick)
    pub current_search_dir: String,
}

impl SearchDialog {
    pub fn new(search_dir: PathBuf) -> Self {
        Self {
            name_pattern: String::new(),
            find_text: String::new(),
            search_dir,
            active_field: SearchField::NamePattern,
            state: SearchState::Input,
            progress: Arc::new(Mutex::new(String::new())),
            current_search_dir: String::new(),
        }
    }

    pub fn toggle_field(&mut self) {
        self.active_field = match self.active_field {
            SearchField::NamePattern => SearchField::FindText,
            SearchField::FindText => SearchField::NamePattern,
        };
    }

    pub fn active_input_mut(&mut self) -> &mut String {
        match self.active_field {
            SearchField::NamePattern => &mut self.name_pattern,
            SearchField::FindText => &mut self.find_text,
        }
    }
}

/// Match a filename against a wildcard pattern (supports * and ?)
/// Case-insensitive.
pub fn wildcard_match(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();
    wildcard_match_impl(pattern.as_bytes(), name.as_bytes())
}

fn wildcard_match_impl(pattern: &[u8], name: &[u8]) -> bool {
    let mut pi = 0;
    let mut ni = 0;
    let mut star_pi = usize::MAX;
    let mut star_ni = 0;

    while ni < name.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == name[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            star_pi = pi;
            star_ni = ni;
            pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_ni += 1;
            ni = star_ni;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }

    pi == pattern.len()
}

/// Recursively search for files matching criteria.
/// Runs in spawn_blocking — must not reference any iced types.
pub fn search_files(
    search_dir: PathBuf,
    name_pattern: String,
    find_text: String,
    progress: Arc<Mutex<String>>,
) -> Vec<FileEntry> {
    let mut results = Vec::new();
    let effective_pattern = if name_pattern.is_empty() {
        "*".to_string()
    } else {
        name_pattern
    };
    let find_text_lower = if find_text.is_empty() {
        None
    } else {
        Some(find_text.to_lowercase())
    };

    // Skip the root search directory itself — only search in subdirectories
    if let Ok(entries) = fs::read_dir(&search_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                search_recursive(&entry.path(), &effective_pattern, find_text_lower.as_deref(), &mut results, &progress);
            }
        }
    }
    results
}

fn search_recursive(
    dir: &PathBuf,
    pattern: &str,
    find_text: Option<&str>,
    results: &mut Vec<FileEntry>,
    progress: &Arc<Mutex<String>>,
) {
    if let Ok(mut p) = progress.lock() {
        *p = dir.to_string_lossy().to_string();
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = metadata.is_dir();

        let name_matches = wildcard_match(pattern, &name);

        if name_matches {
            if let Some(text) = find_text {
                // Text search: only match files whose content contains the text
                if !is_dir {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.to_lowercase().contains(text) {
                            let modified = metadata.modified().ok().map(DateTime::<Local>::from);
                            results.push(FileEntry {
                                name,
                                path: path.clone(),
                                is_dir,
                                size: metadata.len(),
                                modified,
                                selected: false,
                            });
                        }
                    }
                }
            } else {
                // No text search — name match is sufficient
                let modified = metadata.modified().ok().map(DateTime::<Local>::from);
                results.push(FileEntry {
                    name,
                    path: path.clone(),
                    is_dir,
                    size: metadata.len(),
                    modified,
                    selected: false,
                });
            }
        }

        // Always recurse into subdirectories
        if is_dir {
            search_recursive(&path, pattern, find_text, results, progress);
        }
    }
}
