use iced::Element;

use crate::app::Message;

/// A panel entry that can be displayed in a list
pub trait PanelEntry: Clone {
    fn name(&self) -> &str;
    fn is_dir(&self) -> bool;
    fn is_selected(&self) -> bool;
    fn size_display(&self) -> String;
    fn date_display(&self) -> String;
}

/// A panel that displays a list of entries with navigation
pub trait Panel: Default {
    type Entry: PanelEntry;

    fn entries(&self) -> &[Self::Entry];
    fn cursor(&self) -> usize;
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    fn title(&self) -> String;

    // Navigation
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn move_to_top(&mut self);
    fn move_to_bottom(&mut self);
    fn set_cursor(&mut self, index: usize);

    // Actions
    fn enter_selected(&mut self) -> bool;
    fn go_parent(&mut self) -> bool;
    fn toggle_selection(&mut self);
    fn refresh(&mut self);

    fn current_entry(&self) -> Option<&Self::Entry> {
        self.entries().get(self.cursor())
    }

    fn view(&self) -> Element<'_, Message>;
}
