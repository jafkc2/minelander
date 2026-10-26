use iced::{
    widget::{column, slider, text, text_input, toggler},
    Element, Length, Task,
};

use crate::widgets::{container::container, pick_list::pick_list};

#[derive(Default)]
pub struct Settings;

#[derive(Debug, Clone)]
pub enum Message {
    Launch,
    VersionSelected(String),
}

impl Settings {
    pub fn view(&self) -> Element<'_, Message> {
        let minecraft_section = container(
            column![
                text("Minecraft").size(22),
                column![
                    text("RAM allocated to the game: 8 GB"),
                    slider(1..=32, 8, |value| {
                        Message::VersionSelected(value.to_string())
                    })
                    .step(1)
                    .width(Length::Fill),
                ]
                .spacing(8),
                pick_list(
                    "Java version:",
                    [
                        "Java 8".to_string(),
                        "Java 17".to_string(),
                        "Java 21".to_string()
                    ],
                    Some("Java 17".to_string()),
                    Message::VersionSelected,
                ),
                pick_list(
                    "Game instance:",
                    ["Default".to_string()],
                    Some("Default".to_string()),
                    Message::VersionSelected,
                ),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let minelander_section = container(
            column![
                text("Minelander").size(22),
                pick_list(
                    "Theme:",
                    [
                        "Catppuccin Mocha".to_string(),
                        "Catppuccin Latte".to_string()
                    ],
                    Some("Catppuccin Mocha".to_string()),
                    Message::VersionSelected,
                ),
                toggler(false).label("Check for updates automatically"),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let system_section = container(
            column![
                text("System").size(22),
                column![
                    text("Environment variables"),
                    text_input("Variables used in the Minecraft command", "",).width(Length::Fill),
                ]
                .spacing(8),
                column![
                    text("Wrapper commands"),
                    text_input("Commands used to wrap the Minecraft command", "",)
                        .width(Length::Fill),
                ]
                .spacing(8),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        column![
            text("Minelander").size(50).height(50),
            text("Configure your game and launcher preferences.")
                .size(20)
                .height(50),
            minecraft_section,
            minelander_section,
            system_section,
        ]
        .width(Length::FillPortion(4))
        .spacing(14)
        .padding(6)
        .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Launch => Task::none(),
            Message::VersionSelected(_) => todo!(),
        }
    }
}
