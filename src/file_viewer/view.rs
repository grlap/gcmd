use iced::widget::{column, container, scrollable, text, Column};
use iced::{Element, Length};

use super::model::FileViewer;
use crate::app::Message;

pub fn view_file_viewer(viewer: &FileViewer) -> Element<'_, Message> {
    let title = format!(
        " {} - Line {}/{} ",
        viewer.file_name(),
        viewer.scroll_offset + 1,
        viewer.total_lines()
    );

    let title_bar = container(text(title).size(14))
        .width(Length::Fill)
        .padding([4, 8])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.0, 0.4, 0.6).into()),
            ..Default::default()
        });

    let help_bar = container(text("↑/↓:Scroll  PgUp/PgDn:Page  Home/End:Jump  Esc/F3:Close").size(12))
        .width(Length::Fill)
        .padding([4, 8])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.0, 0.3, 0.5).into()),
            ..Default::default()
        });

    let content_lines: Vec<Element<Message>> = viewer
        .visible_content()
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = viewer.scroll_offset + i + 1;
            let line_text = format!("{:5} │ {}", line_num, line);
            text(line_text)
                .size(14)
                .font(iced::Font::MONOSPACE)
                .into()
        })
        .collect();

    let content_column = Column::with_children(content_lines)
        .spacing(0)
        .width(Length::Fill);

    let content_area = container(scrollable(content_column))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.1, 0.1, 0.15).into()),
            ..Default::default()
        });

    let viewer_container = container(column![title_bar, content_area, help_bar])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.05, 0.05, 0.1).into()),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    viewer_container.into()
}
