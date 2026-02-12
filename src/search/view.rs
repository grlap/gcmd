use iced::widget::{column, container, text};
use iced::{Element, Length};

use super::model::{SearchDialog, SearchField, SearchState};
use crate::app::Message;

pub fn view_search_dialog(dialog: &SearchDialog) -> Element<'_, Message> {
    let title_bar = container(text(" Find Files ").size(14))
        .width(Length::Fill)
        .padding([4, 8])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.0, 0.4, 0.6).into()),
            ..Default::default()
        });

    let cursor = "▌";

    // "Search for:" field
    let name_active = dialog.active_field == SearchField::NamePattern;
    let name_cursor = if name_active { cursor } else { "" };
    let name_display = format!("{}{}", dialog.name_pattern, name_cursor);
    let name_input = input_field("Search for:".to_string(), name_display, name_active);

    // "Find text:" field
    let text_active = dialog.active_field == SearchField::FindText;
    let text_cursor = if text_active { cursor } else { "" };
    let text_display = format!("{}{}", dialog.find_text, text_cursor);
    let find_text_input = input_field("Find text:".to_string(), text_display, text_active);

    // "Search in:" (read-only)
    let search_in = container(
        column![
            text("Search in:").size(13).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text(dialog.search_dir.to_string_lossy().to_string())
                .size(14)
                .font(iced::Font::MONOSPACE)
                .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .padding([8, 0]);

    let content_area = container(
        column![name_input, find_text_input, search_in].spacing(4),
    )
    .width(Length::Fill)
    .padding([12, 16])
    .style(|_theme| container::Style {
        background: Some(iced::Color::from_rgb(0.1, 0.1, 0.15).into()),
        ..Default::default()
    });

    let help_text = match dialog.state {
        SearchState::Input => "Tab=Switch Field  Enter=Search  Esc=Cancel".to_string(),
        SearchState::Searching => {
            if dialog.current_search_dir.is_empty() {
                "Searching...".to_string()
            } else {
                format!("Searching: {}", dialog.current_search_dir)
            }
        }
    };

    let help_bar = container(
        text(help_text)
            .size(12)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .width(Length::Fill)
    .padding([4, 8])
    .clip(true)
    .style(|_theme| container::Style {
        background: Some(iced::Color::from_rgb(0.0, 0.3, 0.5).into()),
        ..Default::default()
    });

    let dialog_box = container(column![title_bar, content_area, help_bar])
        .width(Length::Fixed(500.0))
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.05, 0.05, 0.1).into()),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    container(dialog_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
            ..Default::default()
        })
        .into()
}

fn input_field(label: String, display_value: String, is_active: bool) -> Element<'static, Message> {
    let border_color = if is_active {
        iced::Color::from_rgb(0.3, 0.5, 0.7)
    } else {
        iced::Color::from_rgb(0.2, 0.2, 0.3)
    };

    container(
        column![
            text(label).size(13).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            container(
                text(display_value)
                    .size(14)
                    .font(iced::Font::MONOSPACE)
                    .color(iced::Color::from_rgb(0.9, 0.9, 0.9)),
            )
            .width(Length::Fill)
            .padding([6, 10])
            .style(move |_theme| container::Style {
                background: Some(iced::Color::from_rgb(0.05, 0.05, 0.1).into()),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            }),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .into()
}
