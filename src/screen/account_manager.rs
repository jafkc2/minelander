use iced::{
    widget::{column, row, svg, text, text_input},
    Element, Length, Task,
};

use crate::widgets::{
    button::{button, ButtonType},
    container::container,
};

#[derive(Default)]
pub struct AccountManager {
    offline_account_name: String,
}

#[derive(Debug, Clone)]
pub enum MicrosoftLoginState {
    RequestingCode,
    WaitingForUser {
        verification_uri: String,
        user_code: String,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    OfflineAccountNameChanged(String),
    AddOfflineAccountClicked(String),
    AddMicrosoftAccountClicked,
    CancelMicrosoftLoginClicked,
    OpenUrl(String),
    CopyCode(String),
    RemoveAccount(String),
}

impl AccountManager {
    pub fn view<'a>(
        &'a self,
        accounts: &[crate::accounts::AccountInfo],
        microsoft_login: Option<&'a MicrosoftLoginState>,
    ) -> Element<'a, Message> {
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
                )
                .on_press(Message::AddMicrosoftAccountClicked),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill)
            .height(Length::Fixed(210.)),
        );

        let offline_account = container(
            column![
                text("Offline account").size(22),
                text("Choose a name for a local account."),
                text_input("Account name", &self.offline_account_name)
                    .width(Length::Fill)
                    .on_input(Message::OfflineAccountNameChanged),
                button(
                    text("Add offline account").width(Length::Fill),
                    Some(svg::Handle::from_memory(include_bytes!(
                        "../../assets/icons/account.svg"
                    ))),
                    ButtonType::Secondary,
                )
                .on_press(Message::AddOfflineAccountClicked(
                    self.offline_account_name.clone()
                )),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill)
            .height(Length::Fixed(210.)),
        );

        let login_container = microsoft_login.map(|state| {
            let content = match state {
                MicrosoftLoginState::RequestingCode => column![
                    text("Microsoft Login").size(22),
                    text("Requesting login code from Microsoft...").size(14),
                ]
                .spacing(10),
                MicrosoftLoginState::WaitingForUser {
                    verification_uri,
                    user_code,
                } => column![
                    text("Microsoft Login").size(22),
                    text("To complete sign-in, open the link below and enter this code:").size(14),
                    row![
                        text("Code:").size(18),
                        text(user_code).size(24),
                        button(text("Copy Code"), None, ButtonType::Secondary)
                            .on_press(Message::CopyCode(user_code.clone())),
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
                    row![
                        button(text("Open in Browser"), None, ButtonType::Primary)
                            .on_press(Message::OpenUrl(verification_uri.clone())),
                        button(text("Cancel"), None, ButtonType::Secondary)
                            .on_press(Message::CancelMicrosoftLoginClicked),
                    ]
                    .spacing(12),
                    text("Waiting for authorization in your browser...").size(14),
                ]
                .spacing(12),
                MicrosoftLoginState::Error(err) => column![
                    text("Microsoft Login Failed").size(22),
                    text(err).size(14),
                    button(text("Dismiss"), None, ButtonType::Secondary)
                        .on_press(Message::CancelMicrosoftLoginClicked),
                ]
                .spacing(10),
            };

            container(content.padding(15).width(Length::Fill)).max_width(700)
        });

        let account_rows = accounts.iter().fold(
            column![].spacing(10).width(Length::Fill),
            |accounts_column, account| {
                let username = account.username.clone();
                let account_type = if account.offline {
                    "Offline"
                } else {
                    "Microsoft"
                };

                accounts_column.push(
                    row![
                        column![text(account.username.clone()), text(account_type).size(14),]
                            .width(Length::Fill),
                        button(
                            text("Remove"),
                            Some(svg::Handle::from_memory(include_bytes!(
                                "../../assets/icons/trash.svg"
                            ))),
                            ButtonType::Secondary,
                        )
                        .on_press(Message::RemoveAccount(username)),
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center)
                    .width(Length::Fill),
                )
            },
        );

        let accounts_content = if accounts.is_empty() {
            column![text("No accounts added yet.").width(Length::Fill)]
        } else {
            account_rows
        };

        let accounts_section = container(
            column![text("Your accounts").size(22), accounts_content]
                .spacing(10)
                .padding(15)
                .width(Length::Fill),
        );

        let mut content = column![
            text("Accounts").size(50).height(50),
            text("Manage your Microsoft and offline accounts.")
                .size(20)
                .height(50),
            iced::widget::container(
                row![microsoft_account, offline_account]
                    .spacing(14)
                    .width(Length::Fill)
            )
            .max_width(700),
        ];

        if let Some(container_widget) = login_container {
            content = content.push(container_widget);
        }

        content = content.push(accounts_section.max_width(700));

        content
            .width(Length::FillPortion(4))
            .spacing(14)
            .padding(6)
            .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::OfflineAccountNameChanged(name) => {
                self.offline_account_name = name;
                Task::none()
            }
            Message::AddOfflineAccountClicked(_) => Task::none(),
            Message::AddMicrosoftAccountClicked => Task::none(),
            Message::CancelMicrosoftLoginClicked => Task::none(),
            Message::OpenUrl(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }
            Message::CopyCode(code) => iced::clipboard::write(code),
            Message::RemoveAccount(_) => Task::none(),
        }
    }
}
