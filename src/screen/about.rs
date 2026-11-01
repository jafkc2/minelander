use iced::{
    widget::{column, row, svg, text},
    Element, Length, Task,
};

use crate::widgets::{
    button::{button, ButtonType},
    container::container,
};

#[derive(Default)]
pub struct About;

#[derive(Debug, Clone)]
pub enum Message {
    GithubClicked,
    CheckForUpdates,
    InstallUpdate,
}

impl About {
    pub fn view(
        &self,
        automatic_updates: bool,
        checking_for_updates: bool,
        update_available: bool,
        update_status: Option<String>,
    ) -> Element<'_, Message> {
        let credits_section = container(
            column![
                text("Credits").size(22),
                text("Minelander by JafKC (jafkc2)."),
            ]
            .spacing(10)
            .padding(15)
            .width(Length::Fill),
        );

        let mut version_content = column![
            text("Version").size(22),
            text(format!("Minelander v{}", env!("CARGO_PKG_VERSION"))),
            text("Simple, fast and intuitive Minecraft launcher."),
        ]
        .spacing(10);
        if !automatic_updates {
            version_content = version_content.push(
                button(
                    text(if checking_for_updates {
                        "Checking for updates..."
                    } else {
                        "Check for updates"
                    }),
                    None,
                    ButtonType::Secondary,
                )
                .on_press_maybe((!checking_for_updates).then_some(Message::CheckForUpdates)),
            );
        }
        if update_available {
            version_content = version_content.push(
                button(text("Install update"), None, ButtonType::Primary)
                    .on_press(Message::InstallUpdate),
            );
        }
        if let Some(status) = update_status {
            version_content = version_content.push(text(status));
        }
        let version_section = container(version_content.padding(15).width(Length::Fill));

        let github_section = container(
            column![
                text("Project").size(22),
                text("Find the source code and updates on GitHub."),
                row![button(
                    text("jafkc2/minelander").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/github.svg"
                    ))),
                    ButtonType::Secondary,
                )
                .on_press(Message::GithubClicked),]
                .spacing(14)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(10)
            .padding(15)
            .width(Length::Fill),
        );

        column![
            text("About").size(50).height(50),
            text("Information about Minelander.").size(20).height(50),
            credits_section.max_width(700),
            version_section.max_width(700),
            github_section.max_width(700),
        ]
        .width(Length::FillPortion(4))
        .spacing(14)
        .padding(6)
        .into()
    }
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::GithubClicked => {
                open::that("https://github.com/jafkc2/minelander").unwrap_or_else(|e| {
                    eprintln!("Failed to open GitHub page: {}", e);
                });
            }
            Message::CheckForUpdates | Message::InstallUpdate => {}
        }
        Task::none()
    }
}
