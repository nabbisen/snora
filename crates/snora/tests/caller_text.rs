//! Caller text is shown whole, and handed to iced unchanged (RFC-105).
//!
//! # Why `visible_bounds()` is not the detector
//!
//! A clipped string's `visible_bounds()` equals its `bounds()` — the
//! glyphs overflow *inside* the text widget's own box, so the box looks
//! healthy. Measured before the fix: 400 unbroken characters in a toast
//! rendered 288.0 x 20.8, exactly the message column, while the string's
//! natural size is 3481.6 x 20.8. About 48 characters were visible and
//! nothing said so.
//!
//! So each assertion here compares the **natural** (unconstrained) size
//! of the string with what the surface actually rendered, the technique
//! RFC-099 and RFC-100 used for glyphs.
//!
//! # What "shown whole" means here
//!
//! The rendered box must have room for every glyph: its area must be at
//! least the natural area, and its width must stay inside the column.
//! Wrapping trades width for height, so area is the invariant, and the
//! width bound is what says the text did not simply overflow again.
//!
//! **What this does not assert:** which glyphs iced draws. A bidi
//! override or a control character is handed to cosmic-text and drawn by
//! it; the harness exposes frame hashes, not pixel reads, so this file
//! can show that snora passes the string through unchanged and that it
//! has room to be drawn — not what it looks like.

#![cfg(feature = "widgets")]

mod common;

use iced::widget::{container, text};
use iced::{Element, Length, Size};
use iced_test::Simulator;

/// Wide enough that nothing wraps for the natural measurement, tall
/// enough for the wrapped result.
const MEASURING_WINDOW: Size = Size::new(4000.0, 4000.0);
/// The window the surfaces themselves are rendered in.
const WINDOW: Size = Size::new(600.0, 900.0);

/// A notice takes its width from its parent; a toast is fixed at 340.
#[cfg(feature = "design")]
const NOTICE_PARENT_WIDTH: f32 = 340.0;

/// Long enough to overflow every surface in scope several times over,
/// and with no space in it, which is the case iced's default `Word`
/// wrapping cannot help with.
fn unbroken() -> String {
    "a".repeat(400)
}

/// Ordinary prose, to show the change leaves it alone.
const PROSE: &str = "Index rebuilt, and everything is fine.";

/// tekstide's verbatim criterion: a bidi override, a newline and a
/// control character, in one string.
fn verbatim() -> String {
    "before\u{202e}after\nsecond line\u{0007}end".to_owned()
}

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Dismiss,
}

/// The string's size with nothing constraining it.
fn natural_size(label: &str, size: Option<f32>) -> Size {
    let mut widget = text(label.to_owned());
    if let Some(size) = size {
        widget = widget.size(size);
    }
    let element: Element<'_, Msg> = widget.into();
    let mut ui = Simulator::with_size(iced::Settings::default(), MEASURING_WINDOW, element);
    ui.find(label)
        .expect("the string renders on its own")
        .visible_bounds()
        .expect("the string is visible")
        .size()
}

/// What the surface rendered for `label`.
fn rendered_size(element: Element<'_, Msg>, label: &str) -> Size {
    let mut ui = Simulator::with_size(iced::Settings::default(), WINDOW, element);
    ui.find(label)
        .unwrap_or_else(|_| panic!("the surface does not render the string"))
        .visible_bounds()
        .expect("the string is visible")
        .size()
}

// ---------------------------------------------------------------------------
// The surfaces in scope
// ---------------------------------------------------------------------------

/// A toast carrying `title` and `message`, through the public engine path.
fn toast(title: &str, message: &str) -> Element<'static, Msg> {
    let toast = snora::Toast::new(
        7,
        snora::ToastIntent::Info,
        title.to_owned(),
        message.to_owned(),
        Msg::Dismiss,
    );
    let body: Element<'static, Msg> = text("body").into();
    snora::render(snora::AppLayout::new(body).toasts(vec![toast]))
}

/// Every caller-text surface in scope, as (what it is, builder, the size
/// its text is drawn at on this path).
type Surface = (&'static str, fn(&str) -> Element<'static, Msg>, Option<f32>);

fn toast_title(s: &str) -> Element<'static, Msg> {
    toast(s, "short")
}

fn toast_message(s: &str) -> Element<'static, Msg> {
    toast("short", s)
}

fn surfaces() -> Vec<Surface> {
    vec![
        ("toast title", toast_title, None),
        ("toast message", toast_message, None),
    ]
}

/// The rendered box has room for every glyph, and stays in its column.
fn assert_shown_whole(what: &str, rendered: Size, natural: Size, column: f32) {
    assert!(
        rendered.width <= column + 0.5,
        "{what}: rendered {:.1}px wide, past its {column}px column — the text overflows rather \
         than wrapping",
        rendered.width,
    );
    let rendered_area = rendered.width * rendered.height;
    let natural_area = natural.width * natural.height;
    assert!(
        rendered_area >= natural_area * 0.9,
        "{what}: the rendered box is {:.0}px² ({:.1}x{:.1}) for a string whose natural size is \
         {:.0}px² ({:.1}x{:.1}) — there is no room for most of it, so it is being cut off",
        rendered_area,
        rendered.width,
        rendered.height,
        natural_area,
        natural.width,
        natural.height,
    );
}

// ---------------------------------------------------------------------------
// Unit 2 — long unbroken text wraps instead of being cut off
// ---------------------------------------------------------------------------

#[test]
fn unbroken_text_is_shown_whole_in_toasts() {
    let long = unbroken();
    for (what, build, size) in surfaces() {
        let natural = natural_size(&long, size);
        let rendered = rendered_size(build(&long), &long);
        // The toast's message column, measured: a fixed 340 surface less
        // its 12px padding on each side, the 4px row gap and the 24px
        // close button.
        assert_shown_whole(what, rendered, natural, 288.0);
    }
}

/// Ordinary prose is untouched: same size before and after, because
/// `WordOrGlyph` only differs from `Word` for a word that cannot fit.
#[test]
fn ordinary_prose_is_unchanged() {
    for (what, build, size) in surfaces() {
        let natural = natural_size(PROSE, size);
        let rendered = rendered_size(build(PROSE), PROSE);
        assert!(
            (rendered.width - natural.width).abs() <= 0.5
                && (rendered.height - natural.height).abs() <= 0.5,
            "{what}: prose rendered {:.1}x{:.1} against its natural {:.1}x{:.1} — this string \
             fits its column, so wrapping must not have touched it",
            rendered.width,
            rendered.height,
            natural.width,
            natural.height,
        );
    }
}

// ---------------------------------------------------------------------------
// Unit 3 — the string reaches iced unchanged
// ---------------------------------------------------------------------------

/// snora hands caller text to iced verbatim: a bidi override, a newline
/// and a control character all survive, and the result is shown whole.
///
/// Found by **exact content**, which is what makes it a verbatim check:
/// `Simulator::find` matches a text widget's content string, so a
/// component that trimmed, escaped or normalised anything would not be
/// found.
#[test]
fn verbatim_text_reaches_iced_unchanged() {
    let fixture = verbatim();
    for (what, build, size) in surfaces() {
        let natural = natural_size(&fixture, size);
        let rendered = rendered_size(build(&fixture), &fixture);
        assert_shown_whole(
            &format!("{what} (verbatim fixture)"),
            rendered,
            natural,
            288.0,
        );
    }
}

// ---------------------------------------------------------------------------
// The same three, for the notice (design path)
// ---------------------------------------------------------------------------

#[cfg(feature = "design")]
mod notice {
    use super::*;
    use snora::design::{Tokens, Tone, notice::Notice};

    /// The notice's text column: its parent's width, less the 4px accent
    /// bar and the container's own `spacing.md` padding on each side.
    fn column(tokens: &Tokens) -> f32 {
        NOTICE_PARENT_WIDTH - 4.0 - 2.0 * tokens.spacing.md
    }

    fn notice<'a>(tokens: &'a Tokens, title: Option<&str>, body: &str) -> Element<'a, Msg> {
        let mut notice = Notice::new(tokens, Tone::Info, body.to_owned());
        if let Some(title) = title {
            notice = notice.title(title.to_owned());
        }
        container(notice.render())
            .width(Length::Fixed(NOTICE_PARENT_WIDTH))
            .into()
    }

    #[test]
    fn unbroken_text_is_shown_whole_in_notices() {
        let tokens = Tokens::light();
        let long = unbroken();
        let label = snora::design::style::text::label_size(&tokens).0;
        let body = snora::design::style::text::body_size(&tokens).0;

        let natural = natural_size(&long, Some(label));
        let rendered = rendered_size(notice(&tokens, Some(&long), "short"), &long);
        assert_shown_whole("notice title", rendered, natural, column(&tokens));

        let natural = natural_size(&long, Some(body));
        let rendered = rendered_size(notice(&tokens, None, &long), &long);
        assert_shown_whole("notice body", rendered, natural, column(&tokens));
    }

    #[test]
    fn notice_prose_is_unchanged() {
        let tokens = Tokens::light();
        let body = snora::design::style::text::body_size(&tokens).0;
        let natural = natural_size(PROSE, Some(body));
        let rendered = rendered_size(notice(&tokens, None, PROSE), PROSE);
        assert!(
            (rendered.width - natural.width).abs() <= 0.5
                && (rendered.height - natural.height).abs() <= 0.5,
            "notice body: prose rendered {:.1}x{:.1} against its natural {:.1}x{:.1}",
            rendered.width,
            rendered.height,
            natural.width,
            natural.height,
        );
    }

    #[test]
    fn notice_text_reaches_iced_unchanged() {
        let tokens = Tokens::light();
        let fixture = verbatim();
        let body = snora::design::style::text::body_size(&tokens).0;
        let natural = natural_size(&fixture, Some(body));
        let rendered = rendered_size(notice(&tokens, None, &fixture), &fixture);
        assert_shown_whole(
            "notice body (verbatim fixture)",
            rendered,
            natural,
            column(&tokens),
        );
    }
}

// ---------------------------------------------------------------------------
// Tooltips (RFC-105 R-1)
// ---------------------------------------------------------------------------

/// Tooltip text cannot be measured the way every other surface here is:
/// iced 0.14's tooltip overlay takes no part in widget operations, so
/// `Simulator::find` cannot see it at all — not its text, not its body
/// (RFC-100 §4).
///
/// What can be seen is the frame. These compare the rendered frame of a
/// tooltip against the same tooltip with **one extra visible character**
/// at the end. If that character reaches the screen, the frames differ.
/// At 400 characters they used not to, which is what "clipped" looks
/// like from outside: measured before the fix, appending `Z` to 400
/// characters changed nothing.
///
/// Each test keeps the 20-character control, so it cannot pass by the
/// tooltip not being drawn at all.
mod tooltips {
    use super::*;
    use iced::{Event, Point, Theme, mouse};

    use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

    /// Big enough for a 320px-wide tooltip and the rail beside it.
    const TOOLTIP_WINDOW: Size = Size::new(1024.0, 600.0);

    /// The first rail button's centre.
    const HOVER: Point = Point::new(32.0, 40.0);

    fn rail_frame(tip: &str, name: &str) -> String {
        let bar = SideBar {
            items: vec![SideBarItem {
                view_id: 0u8,
                icon: Icon::Text("S".into()),
                tooltip: tip.to_owned(),
                on_press: Msg::Dismiss,
            }],
            active: 0,
        };
        let element: Element<'_, Msg> = iced::widget::row![
            snora::widget::app_side_bar(bar, LayoutDirection::Ltr),
            container(text("page")).width(Length::Fill)
        ]
        .into();

        let mut ui = Simulator::with_size(iced::Settings::default(), TOOLTIP_WINDOW, element);
        ui.point_at(HOVER);
        let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved { position: HOVER })]);
        common::frame_hash(&mut ui, &Theme::Light, name)
    }

    /// The control: at a length that fits, the probe sees one character.
    #[test]
    fn the_probe_can_see_one_character() {
        let short = "a".repeat(20);
        assert_ne!(
            rail_frame(&short, "rfc105-ctl-a"),
            rail_frame(&format!("{short}Z"), "rfc105-ctl-b"),
            "a one-character difference in a short tooltip did not change the frame — the probe \
             below would pass whether or not long text is shown",
        );
    }

    /// The property: at 400 characters, the last character is still drawn.
    #[test]
    fn long_tooltip_text_is_shown_whole() {
        let long = unbroken();
        assert_ne!(
            rail_frame(&long, "rfc105-long-a"),
            rail_frame(&format!("{long}Z"), "rfc105-long-b"),
            "appending a visible character to a {}-character tooltip changed nothing on screen — \
             the text is cut off inside the overlay, which is what a user sees as a tooltip that \
             stops mid-word",
            long.len(),
        );
    }
}
