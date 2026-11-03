use iced::{
    widget::{checkbox, column, row, svg, text},
    Element, Length, Task,
};

use crate::{
    downloader::version_downloader::AvailableVersions,
    widgets::{
        button::{button, ButtonType},
        container::container,
        pick_list::pick_list,
    },
};

pub struct VersionManager {
    pub show_only_releases: bool,
    pub selected_version: Option<String>,
    pub selected_loader: Option<ModLoader>,
}

impl Default for VersionManager {
    fn default() -> Self {
        Self {
            show_only_releases: true,
            selected_version: None,
            selected_loader: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModLoader {
    Fabric,
    NeoForge,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoaderSelected(ModLoader),
    VersionSelected(String),
    InstallVanilla,
    InstallFabric,
    InstallNeoForge,
    ShowOnlyReleases(bool),
}

impl VersionManager {
    pub fn view<'a>(
        &'a self,
        installed_versions: &[String],
        available: Option<&'a AvailableVersions>,
        is_downloading: bool,
        status: Option<&'a str>,
    ) -> Element<'a, Message> {
        let selected_version = self.selected_version.clone();
        let can_install = selected_version.is_some() && !is_downloading;
        let visible_versions: Vec<String> = available
            .map(|av| {
                av.minecraft
                    .iter()
                    .filter(|version| !self.show_only_releases || version.version_type == "release")
                    .map(|version| version.id.clone())
                    .collect()
            })
            .unwrap_or_default();

        let default_status = if is_downloading {
            "Download in progress..."
        } else if available.is_none() {
            "Loading available versions..."
        } else {
            "Select a Minecraft version to install."
        };
        let status_text = status.unwrap_or(default_status);

        let fabric_available = available.is_some();
        let neoforge_available = self
            .selected_version
            .as_ref()
            .and_then(|version| available?.neoforge_loaders.get(version))
            .is_some();

        let download_content = column![
            text("Download version").size(22),
            pick_list(
                "Minecraft version:",
                visible_versions,
                self.selected_version.clone(),
                Message::VersionSelected,
            ),
            checkbox(self.show_only_releases)
                .label("Show only releases")
                .on_toggle(Message::ShowOnlyReleases),
            row![
                button(
                    text("Install Vanilla").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/download.svg"
                    ))),
                    ButtonType::Primary,
                )
                .on_press_maybe(can_install.then_some(Message::InstallVanilla)),
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
                .on_press_maybe(
                    (can_install && fabric_available).then_some(Message::InstallFabric)
                ),
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
                .on_press_maybe(
                    (can_install && neoforge_available).then_some(Message::InstallNeoForge),
                ),
            ]
            .spacing(10),
            text(status_text),
        ]
        .padding(15)
        .spacing(14)
        .width(Length::Fill);

        let installed_content = if installed_versions.is_empty() {
            row![text("No Minecraft versions installed yet.")]
        } else {
            row![text(installed_versions.join(", "))]
        };

        let installed_panel = container(
            column![text("Installed versions").size(22), installed_content,]
                .spacing(14)
                .padding(15)
                .width(Length::Fill),
        );

        column![
            text("Version Manager").size(50).height(50),
            text("Download and manage Minecraft versions.")
                .size(20)
                .height(50),
            container(download_content).max_width(700),
            installed_panel.max_width(700),
        ]
        .width(Length::FillPortion(4))
        .spacing(14)
        .padding(6)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VersionSelected(version) => self.selected_version = Some(version),
            Message::ShowOnlyReleases(show_only_releases) => {
                self.show_only_releases = show_only_releases;
            }
            Message::LoaderSelected(loader) => self.selected_loader = Some(loader),
            Message::InstallVanilla | Message::InstallFabric | Message::InstallNeoForge => {}
        }
        Task::none()
    }
}
