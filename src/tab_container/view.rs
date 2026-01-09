use iced::widget::{column, container, mouse_area, pane_grid, text, Row};
use iced::{Element, Length};

use super::model::TabContainer;
use crate::app::Message;
use crate::folder_panel::view::view_panel_with_pane;

pub fn view_tab_container(container_widget: &TabContainer, pane: pane_grid::Pane) -> Element<'_, Message> {
    let tab_bar = view_tab_bar(container_widget, pane);
    let panel_content = view_panel_with_pane(container_widget.active_panel(), pane);

    // Wrap panel content in mouse_area to receive tab drops
    let panel_drop_zone = mouse_area(panel_content)
        .on_release(Message::TabDropOnPane { target_pane: pane });

    column![tab_bar, panel_drop_zone]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_tab_bar(container_widget: &TabContainer, pane: pane_grid::Pane) -> Element<'_, Message> {
    let tab_count = container_widget.tab_count();

    let tabs: Vec<Element<Message>> = container_widget
        .tabs()
        .iter()
        .enumerate()
        .map(|(i, panel)| {
            let is_active = i == container_widget.active_tab_index();
            let dir_name = panel
                .current_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());

            let label = if dir_name.len() > 12 {
                format!("{}...", &dir_name[..10])
            } else {
                dir_name
            };

            let bg_color = if is_active && container_widget.is_focused() {
                iced::Color::from_rgb(0.2, 0.3, 0.5)
            } else if is_active {
                iced::Color::from_rgb(0.25, 0.25, 0.3)
            } else {
                iced::Color::from_rgb(0.15, 0.15, 0.2)
            };

            let tab_content = container(text(label).size(13))
                .padding([2, 8])
                .style(move |_theme| container::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                });

            // Only allow dragging if there's more than one tab
            let mut tab_mouse = mouse_area(tab_content)
                .on_press(Message::SelectTab { pane, tab_index: i });

            if tab_count > 1 {
                // Right-click to start drag
                tab_mouse = tab_mouse
                    .on_right_press(Message::TabDragStart { pane, tab_index: i });
            }

            tab_mouse.into()
        })
        .collect();

    let mut tab_row = Row::with_children(tabs).spacing(2);

    // Add "+" button for new tab
    let add_btn = container(text("+").size(13))
        .padding([2, 6])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.2, 0.25).into()),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let add_btn_clickable = mouse_area(add_btn)
        .on_press(Message::AddTab { pane });

    tab_row = tab_row.push(add_btn_clickable);

    container(tab_row)
        .width(Length::Fill)
        .padding([2, 4])
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgb(0.1, 0.1, 0.12).into()),
            ..Default::default()
        })
        .into()
}
