use std::ops::RangeInclusive;

use iced::widget::slider::{Handle, Rail, Status};
use iced::{Background, Theme};

use iced::widget::Slider;

pub fn slider<'a, T, Message>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
{
    iced::widget::slider(range, value, on_change).style(move |theme: &Theme, status: Status| {
        let border_color = match status {
            Status::Hovered | Status::Dragged => theme.extended_palette().primary.base.color,
            Status::Active => theme.extended_palette().secondary.strong.color,
        };

        iced::widget::slider::Style {
            rail: Rail {
                backgrounds: (
                    Background::Color(theme.palette().primary),
                    Background::Color(theme.palette().background),
                ),
                width: 4.0,
                border: iced::Border {
                    color: border_color,
                    width: 1.5,
                    radius: iced::border::Radius::new(2),
                },
            },
            handle: Handle {
                shape: iced::widget::slider::HandleShape::Circle { radius: 7.0 },
                background: Background::Color(theme.extended_palette().background.strongest.color),
                border_width: 1.5,
                border_color,
            },
        }
    })
}
