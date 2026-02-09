use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileOpKind {
    Copy,
    Move,
    Delete,
}

impl FileOpKind {
    pub fn label(&self) -> &str {
        match self {
            FileOpKind::Copy => "Copy",
            FileOpKind::Move => "Move",
            FileOpKind::Delete => "Delete",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileOpItem {
    pub source: PathBuf,
    pub is_dir: bool,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum FileOpState {
    Confirming,
    Completed { count: usize },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct FileOpDialog {
    pub kind: FileOpKind,
    pub items: Vec<FileOpItem>,
    pub destination: PathBuf,
    pub state: FileOpState,
}

impl FileOpDialog {
    pub fn new(kind: FileOpKind, items: Vec<FileOpItem>, destination: PathBuf) -> Self {
        Self {
            kind,
            items,
            destination,
            state: FileOpState::Confirming,
        }
    }

    pub fn summary(&self) -> String {
        let file_count = self.items.iter().filter(|i| !i.is_dir).count();
        let dir_count = self.items.iter().filter(|i| i.is_dir).count();
        let mut parts = Vec::new();
        if file_count > 0 {
            parts.push(format!(
                "{} file{}",
                file_count,
                if file_count != 1 { "s" } else { "" }
            ));
        }
        if dir_count > 0 {
            parts.push(format!(
                "{} folder{}",
                dir_count,
                if dir_count != 1 { "s" } else { "" }
            ));
        }
        parts.join(" and ")
    }
}

pub fn copy_items(items: Vec<FileOpItem>, destination: PathBuf) -> Result<usize, String> {
    let mut count = 0;
    for item in &items {
        let dest_path = destination.join(&item.name);
        if item.is_dir {
            copy_dir_recursive(&item.source, &dest_path)
                .map_err(|e| format!("Failed to copy '{}': {}", item.name, e))?;
        } else {
            fs::copy(&item.source, &dest_path)
                .map_err(|e| format!("Failed to copy '{}': {}", item.name, e))?;
        }
        count += 1;
    }
    Ok(count)
}

pub fn move_items(items: Vec<FileOpItem>, destination: PathBuf) -> Result<usize, String> {
    let mut count = 0;
    for item in &items {
        let dest_path = destination.join(&item.name);
        // Try rename first (fast, same filesystem)
        if fs::rename(&item.source, &dest_path).is_ok() {
            count += 1;
            continue;
        }
        // Fallback: copy then delete
        if item.is_dir {
            copy_dir_recursive(&item.source, &dest_path)
                .map_err(|e| format!("Failed to copy '{}': {}", item.name, e))?;
            fs::remove_dir_all(&item.source)
                .map_err(|e| format!("Failed to remove source '{}': {}", item.name, e))?;
        } else {
            fs::copy(&item.source, &dest_path)
                .map_err(|e| format!("Failed to copy '{}': {}", item.name, e))?;
            fs::remove_file(&item.source)
                .map_err(|e| format!("Failed to remove source '{}': {}", item.name, e))?;
        }
        count += 1;
    }
    Ok(count)
}

pub fn delete_items(items: Vec<FileOpItem>) -> Result<usize, String> {
    let mut count = 0;
    for item in &items {
        if item.is_dir {
            fs::remove_dir_all(&item.source)
                .map_err(|e| format!("Failed to delete '{}': {}", item.name, e))?;
        } else {
            fs::remove_file(&item.source)
                .map_err(|e| format!("Failed to delete '{}': {}", item.name, e))?;
        }
        count += 1;
    }
    Ok(count)
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }
    Ok(())
}
