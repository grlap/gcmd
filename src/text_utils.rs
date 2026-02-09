/// Text utility functions for truncation and width calculation

/// Approximate character width in pixels for common font sizes
/// Based on Fira Code monospace font
pub fn char_width(font_size: f32) -> f32 {
    // Fira Code is roughly 0.6 * font_size wide per character
    font_size * 0.6
}

/// Calculate maximum characters that fit in a given pixel width
pub fn max_chars_for_width(available_width: f32, font_size: f32) -> usize {
    let cw = char_width(font_size);
    (available_width / cw).max(1.0) as usize
}

/// Truncate a string to fit within max_chars, adding ellipsis if needed
pub fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}

/// Truncate a string to fit within a pixel width at given font size
pub fn truncate_to_width(text: &str, available_width: f32, font_size: f32) -> String {
    let max_chars = max_chars_for_width(available_width, font_size);
    truncate(text, max_chars)
}
