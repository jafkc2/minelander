#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod messages;
mod screen;
mod widgets;

use iced::widget::{button, column, row};
use iced::{widget::text, Element, Task, Theme};

use crate::messages::Message;
use crate::screen::Screen;
use crate::widgets::sidebar::Sidebar;


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
    sidebar: widgets::sidebar::Sidebar
}

impl Minelander {
    fn new() -> (Self, Task<Message>) {
        (Self {theme: Theme::Dracula, screen: Screen::Home(screen::home::Home::default()), sidebar: Sidebar::default() }, Task::none())
    }

    fn title(&self) -> String {
        format!("Minelander 1.0")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.sidebar.view().map(Message::Sidebar);

        let view = match &self.screen{
            Screen::Home(home) => home.view().map(Message::HomeScreen),
            Screen::Settings(settings) => settings.view().map(Message::SettingsScreen),
            Screen::VersionManager => todo!(),
            Screen::About => todo!(),
        };


        row![sidebar, view].spacing(20.).into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
