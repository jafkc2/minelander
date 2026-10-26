use crate::{screen, widgets};

#[derive(Debug, Clone)]
pub enum Message{
    HomeScreen(screen::home::Message),
    SettingsScreen(screen::settings::Message),
    Sidebar(widgets::sidebar::Message),
    VersionManagerScreen(screen::version_manager::Message),
    AboutScreen(screen::about::Message),
    AccountManagerScreen(screen::account_manager::Message),
    ModManagerScreen(screen::mod_manager::Message),
}