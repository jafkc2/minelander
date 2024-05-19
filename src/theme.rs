use iced::overlay::menu;
use iced::widget::{
    button, container, pick_list, scrollable, slider, svg, text, text_input, toggler,
};
use iced::{application, color, Background, Border, Color};

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: Color::from_rgb8(30, 30, 46),
            text_color: color!(205, 214, 244),
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub enum Text {
    #[default]
    Default,
    Peach,
    Green,
    Red,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Default => text::Appearance {
                color: color!(205, 214, 244).into(),
            },
            Text::Peach => text::Appearance {
                color: color!(250, 179, 135).into(),
            },
            Text::Green => text::Appearance {
                color: color!(166, 218, 149).into(),
            },
            Text::Red => text::Appearance {
                color: color!(150, 0, 0).into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Default,
    BlackContainer,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default => container::Appearance::default(),
            Container::BlackContainer => container::Appearance {
                background: Some(Background::Color(Color::from_rgb8(49, 50, 68))),
                border: Border {
                    radius: 25.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Red,
    Transparent,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(30, 102, 245))),
                border: Border {
                    radius: 15.0.into(),
                    width: 1.0,
                    color: color!(30, 102, 245),
                },

                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(5, 194, 112))),
                border: Border {
                    radius: 15.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Red => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(210, 15, 57))),
                border: Border {
                    radius: 15.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Transparent => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(30, 102, 245))),
                border: Border {
                    radius: 15.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(205, 214, 244),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(5, 194, 112))),
                border: Border {
                    radius: 15.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(205, 214, 244),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Transparent => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border {
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Red => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(210, 15, 57))),
                border: Border {
                    radius: 15.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(46, 59, 98))),
                border: Border {
                    radius: 15.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(46, 59, 98),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(5, 194, 112))),
                border: Border {
                    radius: 15.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Transparent => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..Default::default()
            },
            Button::Red => button::Appearance {
                background: Some(Background::Color(Color::from_rgb8(210, 15, 57))),
                border: Border {
                    radius: 15.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

#[derive(Default)]
pub enum TextInput {
    #[default]
    Default,
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            border: Border {
                radius: 15.0.into(),
                width: 1.0,
                color: Color::from_rgb8(49, 50, 68),
                ..Default::default()
            },
            icon_color: Color::from_rgb8(205, 214, 244),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            border: Border {
                radius: 15.0.into(),
                width: 1.0,
                color: Color::from_rgb8(205, 214, 244),
                ..Default::default()
            },
            icon_color: Color::from_rgb8(205, 214, 244),
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            border: Border {
                radius: 15.0.into(),
                width: 1.0,
                color: Color::from_rgb8(205, 214, 244),
                ..Default::default()
            },
            icon_color: Color::from_rgb8(205, 214, 244),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb8(88, 91, 112)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb8(205, 214, 244)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb8(205, 214, 244)
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb(
                0x20 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x25 as f32 / 255.0,
            )),
            border: Border {
                radius: 15.0.into(),
                width: 1.0,
                color: Color::from_rgb(
                    0x20 as f32 / 255.0,
                    0x22 as f32 / 255.0,
                    0x25 as f32 / 255.0,
                ),
                ..Default::default()
            },
            icon_color: Color::from_rgb8(205, 214, 244),
        }
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.placeholder_color(style)
    }
}

#[derive(Clone, Default)]
pub enum PickList {
    #[default]
    Default,
}

impl pick_list::StyleSheet for Theme {
    type Style = PickList;

    fn active(&self, style: &Self::Style) -> pick_list::Appearance {
        match style {
            PickList::Default => pick_list::Appearance {
                text_color: Color::from_rgb8(205, 214, 244),
                background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
                placeholder_color: Color::from_rgb(
                    0x20 as f32 / 255.0,
                    0x22 as f32 / 255.0,
                    0x25 as f32 / 255.0,
                ),
                handle_color: Color::from_rgb8(205, 214, 244),

                border: Border {
                    radius: 15.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(49, 50, 68),
                },
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: Color::from_rgb8(205, 214, 244),
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            placeholder_color: Color::from_rgb(
                0x20 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x25 as f32 / 255.0,
            ),
            handle_color: Color::from_rgb8(205, 214, 244),

            border: Border {
                radius: 15.0.into(),
                width: 1.0,
                color: Color::from_rgb8(205, 214, 244),
            },
        }
    }
}

#[derive(Default)]
pub enum Svg {
    #[default]
    Default,
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        match style {
            Svg::Default => svg::Appearance {
                color: Some(Color::from_rgb8(255, 255, 255)),
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> svg::Appearance {
        match style {
            Svg::Default => svg::Appearance {
                color: Some(Color::from_rgb8(220, 220, 255)),
            },
        }
    }
}

#[derive(Default)]
pub enum Slider {
    #[default]
    Default,
}

impl slider::StyleSheet for Theme {
    type Style = Slider;

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let handle = slider::Handle {
                    shape: slider::HandleShape::Rectangle {
                        width: 8,
                        border_radius: 4.0.into(),
                    },
                    color: Color::from_rgb8(30, 102, 245),
                    border_color: Color::from_rgb8(30, 102, 245),
                    border_width: 1.0,
                };

                slider::Appearance {
                    rail: slider::Rail {
                        colors: (
                            Color::from_rgb8(205, 214, 244),
                            Color::from_rgb8(205, 214, 244),
                        ),
                        width: 2.0,
                        border_radius: 4.0.into(),
                    },
                    handle: slider::Handle {
                        color: Color::from_rgb(
                            0x20 as f32 / 255.0,
                            0x22 as f32 / 255.0,
                            0x25 as f32 / 255.0,
                        ),
                        border_color: Color::from_rgb8(205, 214, 244),
                        ..handle
                    },
                }
            }
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let active = self.active(style);

                slider::Appearance {
                    handle: slider::Handle {
                        color: Color::from_rgb8(30, 102, 245),
                        ..active.handle
                    },
                    ..active
                }
            }
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let active = self.active(style);

                slider::Appearance {
                    handle: slider::Handle {
                        color: Color::from_rgb8(30, 102, 245),
                        ..active.handle
                    },
                    ..active
                }
            }
        }
    }
}

#[derive(Clone, Default)]
pub enum Menu {
    #[default]
    Default,
}

impl menu::StyleSheet for Theme {
    type Style = Menu;

    fn appearance(&self, style: &Self::Style) -> menu::Appearance {
        match style {
            Menu::Default => menu::Appearance {
                text_color: Color::from_rgb8(205, 214, 244),
                background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),

                border: Border {
                    radius: 15.0.into(),
                    width: 0.5,
                    color: Color::from_rgb8(30, 102, 245),
                },
                selected_text_color: Color::from_rgb8(205, 214, 244),
                selected_background: iced::Background::Color(Color::from_rgb8(30, 102, 245)),
            },
        }
    }
}

impl From<PickList> for Menu {
    fn from(_pick_list: PickList) -> Self {
        Self::Default
    }
}

#[derive(Default)]
pub enum Scrollable {
    #[default]
    Default,
}

impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, style: &Self::Style) -> scrollable::Appearance {
        match style {
            Scrollable::Default => scrollable::Appearance {
                scrollbar: scrollable::Scrollbar {
                    background: Some(iced::Background::Color(Color::from_rgb(
                        0x20 as f32 / 255.0,
                        0x22 as f32 / 255.0,
                        0x25 as f32 / 255.0,
                    ))),

                    border: Border {
                        radius: 15.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    scroller: scrollable::Scroller {
                        color: Color::from_rgb8(205, 214, 244),

                        border: Border {
                            radius: 15.0.into(),
                            width: 1.0,
                            color: Color::TRANSPARENT,
                        },
                    },
                },
                gap: None,
                container: container::Appearance::default(),
            },
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
        match style {
            Scrollable::Default => {
                if is_mouse_over_scrollbar {
                    scrollable::Appearance {
                        scrollbar: scrollable::Scrollbar {
                            background: Some(iced::Background::Color(Color::from_rgb(
                                0x20 as f32 / 255.0,
                                0x22 as f32 / 255.0,
                                0x25 as f32 / 255.0,
                            ))),

                            border: Border {
                                radius: 15.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            scroller: scrollable::Scroller {
                                color: Color::from_rgb8(205, 214, 244),

                                border: Border {
                                    radius: 15.0.into(),
                                    width: 1.0,
                                    color: Color::TRANSPARENT,
                                },
                            },
                        },
                        gap: None,
                        container: container::Appearance::default(),
                    }
                } else {
                    self.active(style)
                }
            }
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Appearance {
        match style {
            Scrollable::Default => self.hovered(style, true),
        }
    }
}

#[derive(Default)]
pub enum Toggler {
    #[default]
    Default,
}

impl toggler::StyleSheet for Theme {
    type Style = Toggler;

    fn active(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        match style {
            Toggler::Default => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(30, 102, 245)
                } else {
                    Color::from_rgb8(205, 214, 244)
                },
                foreground: if is_active {
                    Color::from_rgb8(205, 214, 244)
                } else {
                    Color::from_rgb8(88, 91, 112)
                },
                background_border_width: 0.0,
                background_border_color: Color::WHITE,
                foreground_border_width: 0.0,
                foreground_border_color: Color::WHITE,
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        match style {
            Toggler::Default => toggler::Appearance {
                foreground: if is_active {
                    Color::from_rgb8(205, 214, 244)
                } else {
                    Color::from_rgb8(88, 91, 112)
                },
                ..self.active(style, is_active)
            },
        }
    }
}
