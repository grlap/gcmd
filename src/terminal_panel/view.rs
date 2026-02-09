use iced::widget::{Column, container, scrollable, span, text};
use iced::{Element, Length};

use super::model::TerminalPanel;
use crate::app::Message;

pub fn view_terminal(terminal: &TerminalPanel) -> Element<'_, Message> {
    let lines = terminal.screen_contents();
    let (cursor_row, cursor_col) = terminal.cursor_position();
    let cursor_row = cursor_row as usize;
    let cursor_col = cursor_col as usize;

    let content: Vec<Element<Message>> = lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if i == cursor_row {
                // Split line at cursor position and render with cursor highlight
                let chars: Vec<char> = line.chars().collect();
                let before: String = chars[..cursor_col.min(chars.len())].iter().collect();
                let cursor_char = if cursor_col < chars.len() {
                    chars[cursor_col].to_string()
                } else {
                    " ".to_string()
                };
                let after: String = if cursor_col + 1 < chars.len() {
                    chars[cursor_col + 1..].iter().collect()
                } else {
                    String::new()
                };

                let spans: Vec<iced::widget::text::Span<'_, (), iced::Font>> = vec![
                    span(before)
                        .color(iced::Color::WHITE)
                        .font(iced::Font::MONOSPACE),
                    span(cursor_char)
                        .color(iced::Color::BLACK)
                        .background(iced::Color::WHITE)
                        .font(iced::Font::MONOSPACE),
                    span(after)
                        .color(iced::Color::WHITE)
                        .font(iced::Font::MONOSPACE),
                ];
                iced::widget::text::Rich::with_spans(spans)
                    .size(14)
                    .into()
            } else {
                text(line)
                    .size(14)
                    .font(iced::Font::MONOSPACE)
                    .color(iced::Color::WHITE)
                    .into()
            }
        })
        .collect();

    let terminal_content = Column::with_children(content)
        .spacing(0)
        .width(Length::Fill);

    let scrollable_content = scrollable(terminal_content)
        .width(Length::Fill)
        .height(Length::Fill);

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(iced::Color::BLACK.into()),
            ..Default::default()
        })
        .into()
}
