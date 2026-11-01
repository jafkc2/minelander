use iced::{
    border::Radius,
    widget::{button, column, container, row, svg, text, Button},
    Background, Border, Color, Element, Length, Renderer, Shadow, Task, Theme,
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
    UpdateClicked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveScreen {
    Home,
    Settings,
    VersionManager,
    About,
    AccountManager,
    ModManager,
}

impl Sidebar {
    pub fn view(
        &self,
        active_screen: ActiveScreen,
        update_available: bool,
    ) -> Element<'_, Message> {
        //let bytes: &'static [u8] = include_bytes!("../../assets/icons/minelander.png");
        //let i = iced::widget::image(image::Handle::from_bytes(bytes)).height(40).width(40);
        container(
            column![
                sidebar_button(
                    "Home",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/home.svg")),
                    active_screen == ActiveScreen::Home,
                )
                .on_press(Message::HomeClicked),
                sidebar_button(
                    "Settings",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/settings.svg")),
                    active_screen == ActiveScreen::Settings,
                )
                .on_press(Message::SettingsClicked),
                sidebar_button(
                    "Versions",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/download.svg")),
                    active_screen == ActiveScreen::VersionManager,
                )
                .on_press(Message::VersionClicked),
                sidebar_button(
                    "Mods",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/folder.svg")),
                    active_screen == ActiveScreen::ModManager,
                )
                .on_press(Message::ModsClicked),
                sidebar_button(
                    "Accounts",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/account.svg")),
                    active_screen == ActiveScreen::AccountManager,
                )
                .on_press(Message::AccountsClicked),
                sidebar_button(
                    "About",
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/info.svg")),
                    active_screen == ActiveScreen::About,
                )
                .on_press(Message::AboutClicked),
                update_available.then(|| {
                    sidebar_button(
                        "Update",
                        svg::Handle::from_memory(include_bytes!("../../assets/icons/download.svg")),
                        false,
                    )
                    .on_press(Message::UpdateClicked)
                }),
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
    active: bool,
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
            move |theme: &Theme, status: button::Status| iced::widget::button::Style {
                background: Some(Background::Color(if active {
                    theme.extended_palette().primary.weak.color
                } else {
                    theme.extended_palette().secondary.base.color
                })),
                text_color: theme.extended_palette().background.strongest.text,
                border: Border {
                    color: match status {
                        button::Status::Hovered | button::Status::Pressed => Color::WHITE,
                        _ if active => theme.extended_palette().primary.strong.color,
                        _ => theme.extended_palette().secondary.strong.color,
                    },
                    width: 2.,
                    radius: Radius::new(5),
                },
                shadow: Shadow {
                    color: match status {
                        button::Status::Hovered | button::Status::Pressed => Color::WHITE,
                        _ if active => theme.extended_palette().primary.strong.color,
                        _ => theme.extended_palette().secondary.strong.color,
                    },
                    offset: [1., 2.].into(),
                    blur_radius: 0.,
                },
                snap: true,
            },
        )
        .width(Length::Fill)
}
