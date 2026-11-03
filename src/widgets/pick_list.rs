use std::borrow::Borrow;

use iced::{
    border::Radius,
    widget::{column, text},
    Background, Border, Length, Shadow, Theme,
};

use crate::widgets::container::container;

pub fn pick_list<'a, T, L, V, Message>(
    label: &'a str,
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> iced::widget::Container<'a, Message, Theme, iced::Renderer>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
{
    let label = label.to_owned();

    let pick_list = iced::widget::pick_list(options, selected, on_selected)
        .width(Length::Fill)
        .style(
            move |theme: &Theme, status: iced::widget::pick_list::Status| {
                iced::widget::pick_list::Style {
                    text_color: theme.palette().text,
                    placeholder_color: theme.palette().text,
                    handle_color: theme.palette().primary,
                    background: Background::Color(theme.extended_palette().background.strong.color),
                    border: Border {
                        color: match status {
                            iced::widget::pick_list::Status::Hovered
                            | iced::widget::pick_list::Status::Opened { .. } => {
                                theme.extended_palette().primary.base.color
                            }
                            _ => theme.extended_palette().secondary.strong.color,
                        },
                        width: 1.5,
                        radius: Radius::new(6),
                    },
                }
            },
        )
        .menu_style(|theme: &Theme| iced::overlay::menu::Style {
            background: Background::Color(theme.extended_palette().background.stronger.color),
            border: Border {
                color: theme.extended_palette().background.strongest.color,
                width: 2.,
                radius: Radius::new(5),
            },
            text_color: theme.palette().text,
            selected_text_color: theme.palette().text,
            selected_background: Background::Color(
                theme.extended_palette().background.strong.color,
            ),
            shadow: Shadow {
                color: theme.extended_palette().background.strongest.color,
                offset: [5., 0.].into(),
                blur_radius: 2.,
            },
        });

    container(column![text(label).size(14), pick_list])
        .padding([12, 14])
        .max_width(350)
}
