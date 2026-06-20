use super::*;
use snora_design::Tokens;

#[test]
fn chip_style_selected_all_statuses_all_presets() {
    let statuses = [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
        button::Status::Disabled,
    ];
    for t in [
        Tokens::light(), Tokens::dark(),
        Tokens::high_contrast_light(), Tokens::high_contrast_dark(),
    ] {
        for s in statuses {
            let style = chip_style_selected(&t, s);
            assert!(style.background.is_some(), "selected {s:?}: background must be set");
        }
    }
}

#[test]
fn chip_style_unselected_all_statuses_all_presets() {
    let statuses = [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
        button::Status::Disabled,
    ];
    for t in [
        Tokens::light(), Tokens::dark(),
        Tokens::high_contrast_light(), Tokens::high_contrast_dark(),
    ] {
        for s in statuses {
            let style = chip_style_unselected(&t, s);
            assert!(style.background.is_some(), "unselected {s:?}: background must be set");
        }
    }
}

#[test]
fn selected_has_accent_text_color() {
    let t = Tokens::light();
    let style = chip_style_selected(&t, button::Status::Active);
    let expected = style::color::to_iced_color(t.palette.accent);
    assert_eq!(style.text_color, expected);
}

#[test]
fn darken_clamps_to_zero() {
    let black = darken(iced::Color::BLACK, 1.0);
    assert!(black.r >= 0.0 && black.g >= 0.0 && black.b >= 0.0);
}
