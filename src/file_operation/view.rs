use iced::widget::{column, container, text, Column};
use iced::{Element, Length};

use super::model::{FileOpDialog, FileOpKind, FileOpState};
use crate::app::Message;

pub fn view_file_op_dialog(dialog: &FileOpDialog) -> Element<'_, Message> {
    let kind_label = dialog.kind.label();
    let is_delete = matches!(dialog.kind, FileOpKind::Delete);

    let (title, help_text) = match &dialog.state {
        FileOpState::Confirming => (
            format!(" {} Files ", kind_label),
            "Enter=Confirm  Esc=Cancel".to_string(),
        ),
        FileOpState::Completed { count } => (
            format!(" {} Complete ", kind_label),
            format!(
                "Successfully {} {} item(s)  |  Enter/Esc=Close",
                match dialog.kind {
                    FileOpKind::Move => "moved",
                    FileOpKind::Delete => "deleted",
                    _ => "copied",
                },
                count
            ),
        ),
        FileOpState::Error(msg) => (
            format!(" {} Error ", kind_label),
            format!("Error: {}  |  Enter/Esc=Close", msg),
        ),
    };

    // Title bar — red for errors and delete, teal otherwise
    let is_error = matches!(dialog.state, FileOpState::Error(_));
    let title_bg = if is_error || (is_delete && matches!(dialog.state, FileOpState::Confirming)) {
        iced::Color::from_rgb(0.7, 0.1, 0.1)
    } else {
        iced::Color::from_rgb(0.0, 0.4, 0.6)
    };
    let title_bar = container(text(title).size(14))
        .width(Length::Fill)
        .padding([4, 8])
        .style(move |_theme| container::Style {
            background: Some(title_bg.into()),
            ..Default::default()
        });

    // Content area
    let content_text = match &dialog.state {
        FileOpState::Confirming => {
            if is_delete {
                format!("Delete {}?", dialog.summary())
            } else {
                format!(
                    "{} {} to:\n{}",
                    kind_label,
                    dialog.summary(),
                    dialog.destination.to_string_lossy()
                )
            }
        }
        FileOpState::Completed { count } => format!(
            "Successfully {} {} item(s).",
            match dialog.kind {
                FileOpKind::Move => "moved",
                FileOpKind::Delete => "deleted",
                _ => "copied",
            },
            count
        ),
        FileOpState::Error(msg) => msg.clone(),
    };

    let content_area = container(text(content_text).size(14))
        .width(Length::Fill)
        .padding([12, 16])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.1, 0.1, 0.15).into()),
            ..Default::default()
        });

    // Item list (show up to 10 items when confirming)
    let items_section: Element<'_, Message> =
        if matches!(dialog.state, FileOpState::Confirming) && !dialog.items.is_empty() {
            let show_count = dialog.items.len().min(10);
            let lines: Vec<Element<Message>> = dialog.items[..show_count]
                .iter()
                .map(|item| {
                    let prefix = if item.is_dir { "/" } else { " " };
                    text(format!("  {}{}", prefix, item.name))
                        .size(13)
                        .font(iced::Font::MONOSPACE)
                        .color(iced::Color::from_rgb(0.8, 0.8, 0.8))
                        .into()
                })
                .collect();

            let mut col = Column::with_children(lines).spacing(0);
            if dialog.items.len() > 10 {
                col = col.push(
                    text(format!("  ... and {} more", dialog.items.len() - 10))
                        .size(13)
                        .font(iced::Font::MONOSPACE)
                        .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                );
            }

            container(col)
                .width(Length::Fill)
                .padding([4, 16])
                .style(|_theme| container::Style {
                    background: Some(iced::Color::from_rgb(0.08, 0.08, 0.12).into()),
                    ..Default::default()
                })
                .into()
        } else {
            container(text("")).height(Length::Fixed(0.0)).into()
        };

    // Help bar
    let help_bar = container(text(help_text).size(12))
        .width(Length::Fill)
        .padding([4, 8])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.0, 0.3, 0.5).into()),
            ..Default::default()
        });

    // Dialog box
    let dialog_box = container(column![title_bar, content_area, items_section, help_bar])
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

    // Center on screen with semi-transparent backdrop
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
