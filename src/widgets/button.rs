use iced::{
    border::Radius, widget::row, Background, Border, Color, Element, Renderer, Shadow, Theme,
};

pub enum ButtonType {
    Primary,
    Secondary,
}

pub fn button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    image_handle: Option<iced::widget::svg::Handle>,
    button_type: ButtonType,
) -> iced::widget::Button<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
{
    let mut r = row![].spacing(10);

    if let Some(img) = image_handle {
        let image = iced::widget::svg(img)
            .height(20)
            .width(20)
            .style(move |_: &Theme, _| iced::widget::svg::Style {
                color: Some(Color::WHITE),
            });
        r = r.push(image);
    }
    r = r.push(content);

    iced::widget::button(r).padding([10, 18]).style(
        move |theme: &Theme, status: iced::widget::button::Status| iced::widget::button::Style {
            background: Some(Background::Color(match button_type {
                ButtonType::Primary => theme.extended_palette().primary.base.color,
                ButtonType::Secondary => theme.extended_palette().secondary.strong.color,
            })),
            text_color: theme.extended_palette().background.strongest.text,
            border: Border {
                color: match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => {
                        theme.extended_palette().primary.base.color
                    }
                    _ => theme.extended_palette().secondary.strong.color,
                },
                width: 1.5,
                radius: Radius::new(8),
            },
            shadow: Shadow {
                color: theme.extended_palette().background.strongest.color,
                offset: [0., 3.].into(),
                blur_radius: 4.,
            },
            snap: true,
        },
    )
}
