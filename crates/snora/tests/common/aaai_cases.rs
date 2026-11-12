//! aaai's three cases (RFC-122), built on `snora::control` and `snora::design::control`.
//!
//! - **a glyph beside a label**, in a button that is the call to action
//!   (design path, `style::button::primary`);
//! - **a file-tree row**, pressable as a whole: an indent, an icon and a name, with
//!   `width(Fill)` (engine and design paths);
//! - **a suggestion whose text is data**: `text(caller_string)`, long enough that it
//!   must wrap (RFC-105), on the design path with the default style (`ghost`).

#![allow(dead_code)]

#[cfg(feature = "design")]
use iced::widget::text::Wrapping;
use iced::widget::{container, row, space, text};
use iced::{Element, Length};
use snora::control;
#[cfg(feature = "design")]
use snora::design::Tokens;
#[cfg(feature = "design")]
use snora::design::control as design_control;

#[cfg(feature = "design")]
/// Long, with a run that has no space in it, so only `WordOrGlyph` can wrap it.
pub const SUGGESTION: &str = "Replace the placeholder with the project's actual title: a_very_long_unbroken_identifier_that_will_not_fit_on_one_line_at_all";

fn glyph_and_label<'a, M: 'a>() -> iced::widget::Row<'a, M> {
    row![text("★"), text("Open")]
        .spacing(6)
        .align_y(iced::Alignment::Center)
}

fn tree_row_content<'a, M: 'a>() -> iced::widget::Row<'a, M> {
    row![
        space().width(Length::Fixed(24.0)),
        text("▸"),
        text("src/main.rs"),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
}

pub fn glyph_label_engine<M: Clone + 'static>(m: M) -> Element<'static, M> {
    control(glyph_and_label()).on_press(m).into()
}

#[cfg(feature = "design")]
pub fn glyph_label_design<M: Clone + 'static>(
    tokens: &'static Tokens,
    m: M,
) -> Element<'static, M> {
    design_control(tokens, glyph_and_label())
        .on_press(m)
        .style(move |_theme, status| snora::design::style::button::primary(tokens, status))
        .into()
}

pub fn tree_row_engine<M: Clone + 'static>(m: M) -> Element<'static, M> {
    container(
        control(tree_row_content())
            .on_press(m)
            .width(Length::Fill)
            .padding([6, 12]),
    )
    .width(Length::Fixed(320.0))
    .into()
}

#[cfg(feature = "design")]
pub fn tree_row_design<M: Clone + 'static>(tokens: &'static Tokens, m: M) -> Element<'static, M> {
    container(
        design_control(tokens, tree_row_content())
            .on_press(m)
            .width(Length::Fill)
            .padding([6, 12]),
    )
    .width(Length::Fixed(320.0))
    .into()
}

#[cfg(feature = "design")]
pub fn suggestion_design<M: Clone + 'static>(tokens: &'static Tokens, m: M) -> Element<'static, M> {
    container(
        design_control(tokens, text(SUGGESTION).wrapping(Wrapping::WordOrGlyph))
            .on_press(m)
            .width(Length::Fill),
    )
    .width(Length::Fixed(260.0))
    .into()
}

#[cfg(feature = "design")]
pub fn disabled_design<M: Clone + 'static>(tokens: &'static Tokens) -> Element<'static, M> {
    design_control(tokens, glyph_and_label())
        .on_press_maybe(None)
        .into()
}

// ---- the same controls given an id (RFC-127) ---------------------------------

pub fn glyph_label_engine_id<M: Clone + 'static>(m: M, id: &'static str) -> Element<'static, M> {
    control(glyph_and_label()).on_press(m).id(id).into()
}

#[cfg(feature = "design")]
pub fn glyph_label_design_id<M: Clone + 'static>(
    tokens: &'static Tokens,
    m: M,
    id: &'static str,
) -> Element<'static, M> {
    design_control(tokens, glyph_and_label())
        .on_press(m)
        .id(id)
        .style(move |_theme, status| snora::design::style::button::primary(tokens, status))
        .into()
}

pub fn tree_row_engine_id<M: Clone + 'static>(m: M, id: &'static str) -> Element<'static, M> {
    container(
        control(tree_row_content())
            .on_press(m)
            .id(id)
            .width(Length::Fill)
            .padding([6, 12]),
    )
    .width(Length::Fixed(320.0))
    .into()
}

#[cfg(feature = "design")]
pub fn tree_row_design_id<M: Clone + 'static>(
    tokens: &'static Tokens,
    m: M,
    id: &'static str,
) -> Element<'static, M> {
    container(
        design_control(tokens, tree_row_content())
            .on_press(m)
            .id(id)
            .width(Length::Fill)
            .padding([6, 12]),
    )
    .width(Length::Fixed(320.0))
    .into()
}
