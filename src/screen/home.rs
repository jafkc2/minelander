use std::path::PathBuf;

use iced::{
    widget::{column, row, scrollable, svg, text},
    Element, Length, Task,
};

use crate::{
    accounts::AccountInfo,
    config::Config,
    minecraft,
    widgets::{
        button::{button, ButtonType},
        container::container,
        pick_list::pick_list,
    },
};

pub struct Home {
    current_account: Option<String>,
    current_version: Option<String>,
    java_status: Option<String>,
    current_instance: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchClicked,
    StopClicked,
    LogsClicked,
    OpenMinecraftFolder,
    OpenInstanceFolder,
    AccountSelected(String),
    VersionSelected(String),
}

impl Home {
    pub fn from_config(config: &Config) -> Self {
        Self {
            current_account: non_empty_selection(&config.current_account),
            current_version: non_empty_selection(&config.current_version),
            java_status: None,
            current_instance: config.current_instance.clone(),
        }
    }

    pub fn view<'a>(
        &'a self,
        accounts: &'a [AccountInfo],
        versions: &'a [String],
        minecraft_running: bool,
        logs_visible: bool,
        logs: &'a [String],
    ) -> Element<'a, Message> {
        let hello_message = if let Some(account) = &self.current_account {
            format!("Hello, {}!", account)
        } else {
            "Hello!".to_string()
        };

        let account_names = accounts
            .iter()
            .map(|account| account.username.clone())
            .collect::<Vec<_>>();

        let util_panel = container(
            column![
                button(
                    text("Open .minecraft folder").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/folder.svg"
                    ))),
                    ButtonType::Secondary,
                )
                .on_press(Message::OpenMinecraftFolder),
                button(
                    text("Open instance folder").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/folder.svg"
                    ))),
                    ButtonType::Secondary,
                )
                .on_press(Message::OpenInstanceFolder),
                button(
                    text("Logs").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/settings.svg"
                    ))),
                    ButtonType::Secondary,
                )
                .on_press(Message::LogsClicked),
            ]
            .spacing(3)
            .padding(15)
            .width(300),
        );

        let launch_button = if minecraft_running {
            button(text("Launch").size(30), None, ButtonType::Primary)
        } else {
            button(text("Launch").size(30), None, ButtonType::Primary)
                .on_press(Message::LaunchClicked)
        };
        let launch_controls = if minecraft_running {
            row![
                launch_button,
                button(text("Stop").size(30), None, ButtonType::Secondary)
                    .on_press(Message::StopClicked)
            ]
            .spacing(10)
        } else {
            row![launch_button].spacing(10)
        };

        let col1 = column![
            text("Minelander").size(50).height(50),
            text(hello_message).size(20).height(50),
            row![
                column![
                    pick_list(
                        "Account:",
                        account_names,
                        self.current_account.clone(),
                        Message::AccountSelected,
                    ),
                    pick_list(
                        "Minecraft version:",
                        versions.to_vec(),
                        self.current_version.clone(),
                        Message::VersionSelected,
                    ),
                ]
                .spacing(14)
                .width(Length::Fixed(350.)),
                util_panel
            ]
            .spacing(20)
            .width(Length::Shrink),
            launch_controls
        ]
        .spacing(14)
        .padding(6);

        let mut content = column![col1];
        if let Some(status) = &self.java_status {
            content = content.push(text(status));
        }
        if logs_visible {
            let log_content = if logs.is_empty() {
                column![text("No Minecraft logs yet.")]
            } else {
                logs.iter().fold(column![], |column, line| {
                    column.push(text(line).width(Length::Fill))
                })
            };
            content = content.push(
                scrollable(log_content)
                    .height(Length::Fixed(300.))
                    .width(Length::Fill),
            );
        }

        content.width(Length::FillPortion(4)).spacing(5).into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LaunchClicked | Message::StopClicked | Message::LogsClicked => {}
            Message::OpenMinecraftFolder => {
                open::that(minecraft::get_minecraft_dir()).unwrap_or_else(|e| {
                    eprintln!("Failed to open Minecraft folder: {}", e);
                });
            }
            Message::OpenInstanceFolder => {
                let instance_path = match self.current_instance.is_empty() {
                    true => PathBuf::from(minecraft::get_minecraft_dir()),
                    false => match minecraft::get_instance_path(&self.current_instance) {
                        Ok(path) => path,
                        Err(e) => {
                            eprintln!("Failed to get instance path: {}", e);
                            return Task::none();
                        }
                    },
                };
                open::that(instance_path).unwrap_or_else(|e| {
                    eprintln!("Failed to open instance folder: {}", e);
                });
            }
            Message::AccountSelected(value) => self.current_account = Some(value),
            Message::VersionSelected(value) => self.current_version = Some(value),
        }
        Task::none()
    }

    pub fn set_java_status(&mut self, status: Option<String>) {
        self.java_status = status;
    }

    pub fn apply_to_config(&self, config: &mut Config) {
        config.current_account = self.current_account.clone().unwrap_or_default();
        config.current_version = self.current_version.clone().unwrap_or_default();
    }
}

fn non_empty_selection(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}
