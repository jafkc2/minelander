#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod messages;
mod screen;
mod updater;
mod widgets;

use iced::widget::row;
use iced::{Element, Task, Theme};

use crate::messages::Message;
use crate::screen::Screen;
use crate::widgets::sidebar::{ActiveScreen, Sidebar};

fn main() {
    iced::application(Minelander::new, Minelander::update, Minelander::view)
        .title(Minelander::title)
        .theme(Minelander::theme)
        .run()
        .unwrap();
}

struct Minelander {
    theme: Theme,
    screen: Screen,
    sidebar: widgets::sidebar::Sidebar,
}

impl Minelander {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::CatppuccinMocha,
                screen: Screen::Home(screen::home::Home::default()),
                sidebar: Sidebar::default(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        format!("Minelander 1.0")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(sidebar_message) => {
                match sidebar_message {
                    widgets::sidebar::Message::HomeClicked => {
                        self.screen = Screen::Home(screen::home::Home::default());
                    }
                    widgets::sidebar::Message::SettingsClicked => {
                        self.screen = Screen::Settings(screen::settings::Settings::default());
                    }
                    widgets::sidebar::Message::VersionClicked => {
                        self.screen = Screen::VersionManager(
                            screen::version_manager::VersionManager::default(),
                        );
                    }
                    widgets::sidebar::Message::AboutClicked => {
                        self.screen = Screen::About(screen::about::About::default());
                    },
                    widgets::sidebar::Message::ModsClicked => {
                        self.screen = Screen::ModManager(screen::mod_manager::ModManager::default());
                    }
                    widgets::sidebar::Message::AccountsClicked => {
                        self.screen = Screen::AccountManager(screen::account_manager::AccountManager::default());
                    }
                }

                Task::none()
            }
            Message::HomeScreen(screen_message) => {
                if let Screen::Home(home) = &mut self.screen {
                    home.update(screen_message).map(Message::HomeScreen)
                } else {
                    Task::none()
                }
            }
            Message::SettingsScreen(screen_message) => {
                if let Screen::Settings(settings) = &mut self.screen {
                    settings.update(screen_message).map(Message::SettingsScreen)
                } else {
                    Task::none()
                }
            }
            Message::VersionManagerScreen(message) => {
                if let Screen::VersionManager(version_manager) = &mut self.screen {
                    version_manager
                        .update(message)
                        .map(Message::VersionManagerScreen)
                } else {
                    Task::none()
                }
            }
            Message::AboutScreen(message) => {
                if let Screen::About(about) = &mut self.screen {
                    about.update(message).map(Message::AboutScreen)
                } else {
                    Task::none()
                }
            }
            Message::AccountManagerScreen(message) => {
                if let Screen::AccountManager(account_manager) = &mut self.screen {
                    account_manager.update(message).map(Message::AccountManagerScreen)
                } else {
                    Task::none()
                }
            }
            Message::ModManagerScreen(message) => {
                if let Screen::ModManager(mod_manager) = &mut self.screen {
                    mod_manager.update(message).map(Message::ModManagerScreen)
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let active_screen = match &self.screen {
            Screen::Home(_) => ActiveScreen::Home,
            Screen::Settings(_) => ActiveScreen::Settings,
            Screen::VersionManager(_) => ActiveScreen::VersionManager,
            Screen::About(_) => ActiveScreen::About,
            Screen::AccountManager(_) => ActiveScreen::AccountManager,
            Screen::ModManager(_) => ActiveScreen::ModManager,
        };
        let sidebar = self.sidebar.view(active_screen).map(Message::Sidebar);

        let view = match &self.screen {
            Screen::Home(home) => home.view().map(Message::HomeScreen),
            Screen::Settings(settings) => settings.view().map(Message::SettingsScreen),
            Screen::VersionManager(version_manager) => {
                version_manager.view().map(Message::VersionManagerScreen)
            }
            Screen::About(about) => about.view().map(Message::AboutScreen),
            Screen::AccountManager(account_manager) => {
                account_manager.view().map(Message::AccountManagerScreen)
            }
            Screen::ModManager(mod_manager) => {
                mod_manager.view().map(Message::ModManagerScreen)
            }
        };

        row![sidebar, view].spacing(20.).into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
