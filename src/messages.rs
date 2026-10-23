use crate::{screen, widgets};

#[derive(Debug, Clone)]
pub enum Message{
    HomeScreen(screen::home::Message),
    SettingsScreen(screen::settings::Message),
    Sidebar(widgets::sidebar::Message)
}