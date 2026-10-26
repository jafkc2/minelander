use iced::{
    widget::{column, row, svg, text, text_input},
    Element, Length, Task,
};

use crate::widgets::{
    button::{button, ButtonType},
    container::container,
};

#[derive(Default)]
pub struct AccountManager;

#[derive(Debug, Clone)]
pub enum Message {}

impl AccountManager {
    pub fn view(&self) -> Element<'_, Message> {
        let microsoft_account = container(
            column![
                text("Microsoft account").size(22),
                text("Sign in with a Microsoft account."),
                button(
                    text("Add Microsoft account").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/account.svg"
                    ))),
                    ButtonType::Primary,
                ),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let offline_account = container(
            column![
                text("Offline account").size(22),
                text("Choose a name for a local account."),
                text_input("Account name", "").width(Length::Fill),
                button(
                    text("Add offline account").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/account.svg"
                    ))),
                    ButtonType::Secondary,
                ),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let accounts_section = container(
            column![
                text("Your accounts").size(22),
                text("No accounts added yet.").width(Length::Fill),
            ]
            .spacing(10)
            .padding(15)
            .width(Length::Fill),
        );

        column![
            text("Accounts").size(50).height(50),
            text("Manage your Microsoft and offline accounts.")
                .size(20)
                .height(50),
            row![microsoft_account, offline_account]
                .spacing(14)
                .width(Length::Fill),
            accounts_section,
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
