use iced::{
    Background, Border, Color, Element, Length, Renderer, Shadow, Task, Theme, border::Radius, widget::{Button, column, row, svg, text},
};

use crate::widgets::{button::ButtonType, pick_list::pick_list, sidebar::sidebar_button};
use crate::widgets::{button::button, container::container};

#[derive(Default)]
pub struct Home {
    selected_version: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Launch,
    VersionSelected(String),
}

impl Home {
    pub fn view(&self) -> Element<'_, Message> {
        let util_panel = container(
            column![
                button(text("Open .minecraft folder").width(Length::Fill), Some(svg::Handle::from_memory(include_bytes!("../../assets/icons/folder.svg"))), ButtonType::Secondary).on_press(Message::Launch),
                button(text("Open instance folder").width(Length::Fill), Some(svg::Handle::from_memory(include_bytes!("../../assets/icons/folder.svg"))), ButtonType::Secondary).on_press(Message::Launch),
                button(text("Logs").width(Length::Fill), Some(svg::Handle::from_memory(include_bytes!("../../assets/icons/settings.svg"))), ButtonType::Secondary).on_press(Message::Launch),
            ]
            .spacing(3)
            .padding(15).width(300),
        );

        let col1 = column![
            text("Minelander").size(50).height(50),
            text("Hello, User!").size(20).height(50),
            row![
                column![
                    pick_list(
                        "Account:",
                        [String::from("a")],
                        Some(String::from("b")),
                        Message::VersionSelected
                    ),
                    pick_list(
                        "Minecraft version:",
                        [String::from("a")],
                        Some(String::from("b")),
                        Message::VersionSelected
                    ),
                ]
                .spacing(14)
                .width(Length::Fixed(350.)),
                util_panel
            ]
            .spacing(20)
            .width(Length::Shrink),
            button(text("Launch").size(30), None, ButtonType::Primary).on_press(Message::Launch)
        ]
        .spacing(14)
        .padding(6);

        col1.width(Length::FillPortion(4)).spacing(5).into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Launch => Task::none(),
            Message::VersionSelected(_) => todo!(),
        }
    }
}