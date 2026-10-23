use iced::{
    widget::{button, column, container, text},
    Element, Length, Task,
};

#[derive(Default)]
pub struct Sidebar {}

#[derive(Debug, Clone)]
pub enum Message {
    HomeClicked,
    SettingsClicked,
    VersionClicked,
    AccountsClicked,
    AboutClicked,
}

impl Sidebar {
    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                text("Minelander").height(Length::from(20)),
                button(text("Home"))
                    .on_press(Message::HomeClicked)
                    .width(Length::Fill),
                button(text("Settings"))
                    .on_press(Message::SettingsClicked)
                    .width(Length::Fill),
                button(text("Version Manager"))
                    .on_press(Message::VersionClicked)
                    .width(Length::Fill),
                button(text("Accounts"))
                    .on_press(Message::AccountsClicked)
                    .width(Length::Fill),
                button(text("About"))
                    .on_press(Message::AboutClicked)
                    .width(Length::Fill)
            ]
            .padding(10.)
            .spacing(10.)
            .width(Length::FillPortion(1)),
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }
}
