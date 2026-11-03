use iced::widget::toggler::Status;
use iced::{Background, Renderer, Theme};

pub fn toggler<'a, Message>(
    is_checked: bool,
) -> iced::widget::Toggler<'a, Message, Theme, Renderer> {
    iced::widget::toggler(is_checked).style(move |theme: &Theme, status: Status| {
        let border_color = match status {
            Status::Hovered { .. } => theme.extended_palette().primary.base.color,
            _ => theme.extended_palette().secondary.strong.color,
        };

        iced::widget::toggler::Style {
            background: match status {
                Status::Active { .. } | Status::Hovered { .. } => {
                    if is_checked {
                        Background::Color(theme.palette().primary)
                    } else {
                        Background::Color(theme.palette().background)
                    }
                }
                _ => Background::Color(theme.palette().background),
            },
            background_border_width: 1.5,
            background_border_color: border_color,
            foreground: Background::Color(theme.extended_palette().background.strongest.color),
            foreground_border_width: 1.5,
            foreground_border_color: border_color,
            text_color: Some(theme.palette().text),
            border_radius: None,
            padding_ratio: 0.1,
        }
    })
}
