use iced::widget::{container, scrollable, text, Column};
use iced::{Element, Length};

use super::model::TerminalPanel;
use crate::app::Message;

pub fn view_terminal(terminal: &TerminalPanel) -> Element<'_, Message> {
    let lines = terminal.screen_contents();

    let content: Vec<Element<Message>> = lines
        .into_iter()
        .map(|line| {
            text(line)
                .size(14)
                .font(iced::Font::MONOSPACE)
                .into()
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
        .height(Length::Fixed(200.0))
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(iced::Color::BLACK.into()),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
