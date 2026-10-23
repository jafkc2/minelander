use iced::{
    widget::{button, column, pick_list, row, text},
    Alignment, Element, Length, Task,
};

#[derive(Default)]
pub struct Home {
    selected_version: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Launch,
    VersionSelected(String),
}

impl Home {
    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            text("Minelander").size(50),
            text("Account:"),
            pick_list(
                [String::from("a")],
                Some(String::from("b")),
                Message::VersionSelected
            ),
            text("Minecraft version:"),
            pick_list(
                [String::from("a")],
                Some(String::from("b")),
                Message::VersionSelected
            ),
            button("Launch").on_press(Message::Launch)
        ]
        .width(Length::FillPortion(4))
        .padding(6);

        content.into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Launch => Task::none(),
            Message::VersionSelected(_) => todo!(),
        }
    }
}
