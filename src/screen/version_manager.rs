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
pub struct VersionManager {
    selected_loader: Option<ModLoader>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModLoader {
    Fabric,
    NeoForge,
}

#[derive(Debug, Clone)]
pub enum Message {
    Launch,
    LoaderSelected(ModLoader),
    VersionSelected(String),
}

impl VersionManager {
    pub fn view(&self) -> Element<'_, Message> {
        let minecraft_versions: [String; 0] = [];
        let fabric_versions: [String; 0] = [];
        let neoforge_versions: [String; 0] = [];
        let installed_versions: [String; 0] = [];

        let modloader_pick_list = match self.selected_loader {
            Some(ModLoader::Fabric) => Some(pick_list(
                "Fabric version:",
                fabric_versions,
                None::<String>,
                Message::VersionSelected,
            )),
            Some(ModLoader::NeoForge) => Some(pick_list(
                "NeoForge version:",
                neoforge_versions,
                None::<String>,
                Message::VersionSelected,
            )),
            None => None,
        };

        let mut download_content = column![
            text("Download version").size(22),
            pick_list(
                "Minecraft version:",
                minecraft_versions,
                None::<String>,
                Message::VersionSelected,
            ),
            row![
                button(
                    text("Install Vanilla").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/download.svg"
                    ))),
                    ButtonType::Primary,
                ),
                button(
                    text("Install Fabric").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/download.svg"
                    ))),
                    if self.selected_loader == Some(ModLoader::Fabric) {
                        ButtonType::Primary
                    } else {
                        ButtonType::Secondary
                    },
                )
                .on_press(Message::LoaderSelected(ModLoader::Fabric)),
                button(
                    text("Install NeoForge").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/download.svg"
                    ))),
                    if self.selected_loader == Some(ModLoader::NeoForge) {
                        ButtonType::Primary
                    } else {
                        ButtonType::Secondary
                    },
                )
                .on_press(Message::LoaderSelected(ModLoader::NeoForge)),
            ]
            .spacing(10),
        ];

        if let Some(modloader_pick_list) = modloader_pick_list {
            download_content = download_content.push(modloader_pick_list);
        }

        let download_panel =
            container(download_content.padding(15).spacing(14).width(Length::Fill));

        let installed_panel = container(
            column![
                text("Installed versions").size(22),
                pick_list(
                    "Version to remove:",
                    installed_versions,
                    None::<String>,
                    Message::VersionSelected,
                ),
                row![
                    text("No Minecraft versions installed yet.").width(Length::Fill),
                    button(
                        text("Remove").width(Length::Fill),
                        Some(svg::Handle::from_memory(include_bytes!(
                            "../../assets/icons/trash.svg"
                        ))),
                        ButtonType::Secondary,
                    ),
                ]
                .align_y(iced::Alignment::Center)
                .spacing(10),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        column![
            text("Minelander").size(50).height(50),
            text("Download and manage Minecraft versions.")
                .size(20)
                .height(50),
            download_panel,
            installed_panel,
        ]
        .width(Length::FillPortion(4))
        .spacing(14)
        .padding(6)
        .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        Task::none()
    }
}
