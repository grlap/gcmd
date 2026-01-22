mod app;
mod folder_panel;
mod panel;
mod tab_container;
mod terminal_panel;
mod text_utils;

use app::{App, Message};
use iced::{Element, Font, Subscription, Task, Theme};

// Fira Code font (embedded)
const FIRA_CODE: &[u8] = include_bytes!("../fonts/FiraCode-Regular.ttf");

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("gcmd - File Manager")
        .subscription(subscription)
        .theme(theme)
        .default_font(Font::with_name("Fira Code"))
        .font(FIRA_CODE)
        .window_size((1200.0, 800.0))
        .run()
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    app.update(message)
}

fn view(app: &App) -> Element<'_, Message> {
    app.view()
}

fn subscription(app: &App) -> Subscription<Message> {
    app.subscription()
}

fn theme(app: &App) -> Theme {
    app.theme()
}
