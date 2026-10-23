use std::io::Read;

use iced::{
    Background, Border, Color, Element, Length, Renderer, Shadow, Task, Theme, border::Radius, color, widget::{Button, button, column, container, image, row, svg, text},
};

#[derive(Default)]
pub struct Sidebar {}

#[derive(Debug, Clone)]
pub enum Message {
    HomeClicked,
    SettingsClicked,
    VersionClicked,
    ModsClicked,
    AccountsClicked,
    AboutClicked,
}

impl Sidebar {
    pub fn view(&self) -> Element<'_, Message> {
        //let bytes: &'static [u8] = include_bytes!("../../assets/icons/minelander.png");
        //let i = iced::widget::image(image::Handle::from_bytes(bytes)).height(40).width(40);
        container(
            column![
                text("Minelander").height(Length::from(25)).size(25),
                sidebar_button(
                    "Home",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/home.svg"))
                ).on_press(Message::HomeClicked),
                sidebar_button(
                    "Settings",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/settings.svg"))
                ).on_press(Message::SettingsClicked),
                sidebar_button(
                    "Versions",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/download.svg"))
                ).on_press(Message::VersionClicked),
                sidebar_button(
                    "Mods",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/download.svg"))
                ).on_press(Message::ModsClicked),
                sidebar_button(
                    "Accounts",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/account.svg"))
                ).on_press(Message::AccountsClicked),
                sidebar_button(
                    "About",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/info.svg"))
                ).on_press(Message::AboutClicked),
                text(format!("v{}", env!("CARGO_PKG_VERSION"),))
            ]
            .padding(15.)
            .spacing(15.)
            .width(Length::FillPortion(1)),
        )
        .style(move |theme: &Theme| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(
                theme.extended_palette().background.strong.color,
            )),
            border: Border {
                color: theme.extended_palette().background.strong.color,
                width: 2.,
                radius: Radius::new(4),
            },
            shadow: Shadow {
                color: theme.extended_palette().background.strongest.color,
                offset: [5., 0.].into(),
                blur_radius: 2.,
            },
            snap: true,
        })
        .height(Length::Fill)
        .padding(10)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }
}

pub fn sidebar_button<'a>(
    text_content: &'a str,
    image: svg::Handle,
) -> Button<'a, Message, Theme, Renderer> {
    let image =
        svg(image)
            .height(20)
            .width(20)
            .style(move |_: &Theme, _| iced::widget::svg::Style {
                color: Some(Color::WHITE),
            });

    let r = row![image, text(text_content)].spacing(10);

    iced::widget::button(r)
        .style(
            move |theme: &Theme, status: button::Status|     
            iced::widget::button::Style {
                background: Some(Background::Color(
                    theme.extended_palette().secondary.base.color,
                )),
                text_color: theme.palette().text,
                border: Border {
                    color: match status{
                        button::Status::Hovered | button::Status::Pressed => Color::WHITE,
                        _ => theme.extended_palette().secondary.strong.color
                    },
                    width: 2.,
                    radius: Radius::new(5),
                },
                shadow: Shadow {
                    color: match status{
                        button::Status::Hovered | button::Status::Pressed => Color::WHITE,
                        _ => theme.extended_palette().secondary.strong.color
                    },
                    offset: [1., 2.].into(),
                    blur_radius: 0.,
                },
                snap: true,
            },
        )
        .width(Length::Fill)
}
