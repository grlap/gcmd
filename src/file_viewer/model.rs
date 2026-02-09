use std::fs;
use std::path::PathBuf;

pub struct FileViewer {
    pub file_path: PathBuf,
    pub content: String,
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub visible_lines: usize,
}

impl FileViewer {
    pub fn new(file_path: PathBuf, visible_lines: usize) -> Self {
        let content = fs::read_to_string(&file_path).unwrap_or_else(|e| {
            format!("Error reading file: {}", e)
        });

        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Self {
            file_path,
            content,
            lines,
            scroll_offset: 0,
            visible_lines,
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.lines.len().saturating_sub(self.visible_lines);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.lines.len().saturating_sub(self.visible_lines);
    }

    pub fn page_up(&mut self) {
        self.scroll_up(self.visible_lines.saturating_sub(1));
    }

    pub fn page_down(&mut self) {
        self.scroll_down(self.visible_lines.saturating_sub(1));
    }

    pub fn visible_content(&self) -> Vec<&str> {
        self.lines
            .iter()
            .skip(self.scroll_offset)
            .take(self.visible_lines)
            .map(|s| s.as_str())
            .collect()
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }
}
