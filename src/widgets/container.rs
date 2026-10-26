use iced::{border::Radius, Background, Border, Renderer, Shadow, Theme};

pub fn container<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
) -> iced::widget::Container<'a, Message, Theme, Renderer> {
    iced::widget::container(content).style(move |theme: &Theme| iced::widget::container::Style {
        text_color: Some(theme.palette().text),
        background: Some(Background::Color(
            theme.extended_palette().background.strong.color,
        )),
        border: Border {
            color: theme.extended_palette().background.stronger.color,
            width: 1.,
            radius: Radius::new(8),
        },
        shadow: Shadow {
            color: theme.extended_palette().background.strongest.color,
            offset: [1., 2.].into(),
            blur_radius: 0.,
        },
        snap: true,
    })
}
