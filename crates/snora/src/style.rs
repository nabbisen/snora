use iced::{
    widget::{button, container},
    // Background,
    Border,
    Color,
    Shadow,
    Theme,
};

pub fn menu_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered => Color::from_rgb8(0x22, 0x66, 0xCC),
        button::Status::Pressed => Color::from_rgb8(0x11, 0x44, 0xAA),
        _ => Color::from_rgb8(0x33, 0x55, 0x99),
    };

    button::Style {
        background: None,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn container_box_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        background: None, // Some(Background::Color(Color::from_rgb8(0xF7, 0xF7, 0xF7))),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb8(0xDD, 0xDD, 0xDD),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
