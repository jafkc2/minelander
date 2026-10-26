use iced::{
    widget::{column, row, svg, text},
    Element, Length, Task,
};

use crate::widgets::{
    button::{button, ButtonType},
    container::container,
    pick_list::pick_list,
};

#[derive(Default)]
pub struct ModManager;

#[derive(Debug, Clone)]
pub enum Message {
    AddMod,
    ImportModpack,
    InstanceSelected(String),
    RemoveMod(String),
}

impl ModManager {
    pub fn view(&self) -> Element<'_, Message> {
        let actions_section = container(
            column![
                text("Manage mods").size(22),
                row![
                    button(
                        text("Add mod").width(Length::Fill),
                        Some(svg::Handle::from_memory(include_bytes!(
                            "../../assets/icons/folder.svg"
                        ))),
                        ButtonType::Primary,
                    ),
                    button(
                        text("Import modpack").width(Length::Fill),
                        Some(svg::Handle::from_memory(include_bytes!(
                            "../../assets/icons/download.svg"
                        ))),
                        ButtonType::Secondary,
                    ),
                ]
                .spacing(10),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let mods_section = container(
            column![
                text("Mods").size(22),
                text("No mods installed in this game instance.").width(Length::Fill),
                row![
                    text("Installed mods will appear here.").width(Length::Fill),
                    button(
                        text("Remove").width(Length::Fill),
                        Some(svg::Handle::from_memory(include_bytes!(
                            "../../assets/icons/trash.svg"
                        ))),
                        ButtonType::Secondary,
                    ),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(10)
            .padding(15)
            .width(Length::Fill),
        );

        column![
            text("Mods").size(50).height(50),
            text("Manage the mods installed in your game.")
                .size(20)
                .height(50),
            pick_list(
                "Game instance:",
                ["Default".to_string()],
                Some("Default".to_string()),
                Message::InstanceSelected,
            ),
            actions_section,
            mods_section,
        ]
        .width(Length::FillPortion(4))
        .spacing(14)
        .padding(6)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddMod
            | Message::ImportModpack
            | Message::InstanceSelected(_)
            | Message::RemoveMod(_) => Task::none(),
        }
    }
}
