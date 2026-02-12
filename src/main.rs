mod app;
mod file_operation;
mod file_viewer;
mod folder_panel;
mod panel;
mod search;
mod tab_container;
mod terminal_panel;
mod text_utils;

use app::{App, Message};
use iced::{Element, Font, Subscription, Task, Theme};

// Fira Code font (embedded)
const FIRA_CODE: &[u8] = include_bytes!("../fonts/FiraCode-Regular.ttf");

use image::GenericImageView;

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("gcmd - File Manager")
        .window(iced::window::Settings {
            icon: Some(load_icon()),
            ..Default::default()
        })
        .subscription(subscription)
        .theme(theme)
        .default_font(Font::with_name("Fira Code"))
        .font(FIRA_CODE)
        .window_size((1200.0, 800.0))
        .run()
}

fn load_icon() -> iced::window::Icon {
    let bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(bytes)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    iced::window::icon::from_rgba(rgba, width, height)
        .expect("Failed to create icon")
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
