use iced::{
    widget::{column, row, scrollable, svg, text},
    Element, Length, Task,
};

use crate::{
    mods,
    widgets::{
        button::{button, ButtonType},
        container::container,
        pick_list::pick_list,
    },
};

pub struct ModManager {
    instance: String,
    mods: Vec<String>,
    status: Option<String>,
}

impl Default for ModManager {
    fn default() -> Self {
        Self {
            instance: String::new(),
            mods: Vec::new(),
            status: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddMod,
    ImportModpack,
    InstanceSelected(String),
    RemoveMod(String),
}

impl ModManager {
    pub fn from_instance(instance: &str) -> Self {
        let mut manager = Self {
            instance: instance.to_string(),
            ..Self::default()
        };
        manager.reload("");
        manager
    }

    pub fn view(&self, instances: &[String], current_instance: &str) -> Element<'_, Message> {
        let selected_instance = if current_instance.is_empty() {
            "Default"
        } else {
            current_instance
        };
        let mods_section = if self.mods.is_empty() {
            column![text("No mods installed in this game instance.")]
        } else {
            self.mods.iter().fold(
                column![text("Installed mods:")].spacing(8),
                |content, file_name| {
                    content.push(
                        row![
                            text(file_name).width(Length::Fill),
                            button(
                                text("Remove"),
                                Some(svg::Handle::from_memory(include_bytes!(
                                    "../../assets/icons/trash.svg"
                                ))),
                                ButtonType::Secondary,
                            )
                            .on_press(Message::RemoveMod(file_name.clone())),
                        ]
                        .align_y(iced::Alignment::Center)
                        .spacing(10),
                    )
                },
            )
        };

        let status = self
            .status
            .as_deref()
            .filter(|status| !status.is_empty())
            .map(text)
            .unwrap_or_else(|| text(""));
        let actions_section = container(
            column![
                text("Manage mods").size(22),
                row![
                    button(
                        text("Add mods").width(Length::Fill),
                        Some(svg::Handle::from_memory(include_bytes!(
                            "../../assets/icons/folder.svg"
                        ))),
                        ButtonType::Primary,
                    )
                    .on_press(Message::AddMod),
                    button(
                        text("Import modpack").width(Length::Fill),
                        Some(svg::Handle::from_memory(include_bytes!(
                            "../../assets/icons/download.svg"
                        ))),
                        ButtonType::Secondary,
                    ),
                ]
                .spacing(10),
                status,
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );
        scrollable(
            column![
                text("Mods").size(50).height(50),
                text("Manage the mods installed in your game.")
                    .size(20)
                    .height(50),
                pick_list(
                    "Game instance:",
                    std::iter::once("Default".to_string())
                        .chain(instances.iter().cloned())
                        .collect::<Vec<_>>(),
                    Some(selected_instance.to_string()),
                    Message::InstanceSelected,
                ),
                actions_section.max_width(700),
                container(mods_section.padding(15).spacing(10)).max_width(700),
            ]
            .width(Length::FillPortion(4))
            .spacing(14)
            .padding(6),
        )
        .height(Length::Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddMod => {
                let paths = rfd::FileDialog::new()
                    .add_filter("Mod files", &["jar", "zip"])
                    .pick_files();
                if let Some(paths) = paths {
                    let mut added = 0;
                    let mut errors = Vec::new();
                    for path in paths {
                        match mods::add(&self.instance, &path) {
                            Ok(()) => added += 1,
                            Err(error) => errors.push(format!("{}: {error}", path.display())),
                        }
                    }

                    if errors.is_empty() {
                        self.reload(&format!("{added} mod(s) added."));
                    } else {
                        self.reload(&format!(
                            "{added} mod(s) added. Failed to add: {}",
                            errors.join("; ")
                        ));
                    }
                }
            }
            Message::InstanceSelected(instance) => {
                self.instance = if instance == "Default" {
                    String::new()
                } else {
                    instance
                };
                self.reload("Instance selected.");
            }
            Message::RemoveMod(file_name) => match mods::remove(&self.instance, &file_name) {
                Ok(()) => self.reload("Mod removed."),
                Err(error) => self.status = Some(format!("Failed to remove mod: {error}")),
            },
            Message::ImportModpack => {}
        }
        Task::none()
    }

    fn reload(&mut self, status: &str) {
        match mods::list(&self.instance) {
            Ok(mods) => {
                self.mods = mods;
                self.status = Some(status.to_string());
            }
            Err(error) => self.status = Some(format!("Failed to list mods: {error}")),
        }
    }
}
