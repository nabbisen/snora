//! RFC-115's rule, living in `control`'s layout (RFC-116 Unit 0).
//!
//! The floor is `max(24, 1.5 x text)`, and the text size comes from
//! the renderer — `iced::Settings::default_text_size`, the one
//! setting RFC-104 made the chrome follow. So the floor follows the
//! text with no new API and no new token, which is the feasibility
//! question Unit 0 was asked.
//!
//! **These measure a control with no content of its own**, because
//! that is the only way to see the floor bind. The sidebar's own
//! items are already larger than it at both text sizes — measured
//! in `the_sidebar_item_is_already_above_the_floor` — so a test on
//! the rail would pass whether the floor existed or not.

use super::*;
use iced::widget::space;
use iced::{Pixels, Settings, Size};
use iced_test::Simulator;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Press,
}

fn bare_control() -> Element<'static, Msg> {
    Control::new(
        space().width(Length::Fixed(1.0)).height(Length::Fixed(1.0)),
        Box::new(|_theme, _status| button::Style::default()),
        Ring {
            color: Box::new(|_| Color::BLACK),
            width: 0.0,
            offset: 0.0,
        },
    )
    .on_press(Msg::Press)
    .into()
}

/// The control's drawn size at a given host text size.
fn measured(text_size: f32) -> Size {
    let settings = || Settings {
        default_text_size: Pixels(text_size),
        ..Settings::default()
    };
    let mut ui: Simulator<'_, Msg> =
        Simulator::with_size(settings(), Size::new(400.0, 400.0), bare_control());

    // Found by clicking, not by tree position: what is measured is
    // the region that answers the control's own message.
    ui.point_at(iced::Point::new(12.0, 12.0));
    let _ = ui.simulate(iced_test::simulator::click());
    assert!(
        ui.into_messages().any(|m| m == Msg::Press),
        "the probe point is not inside the control, so this measures the wrong thing"
    );

    let mut ui: Simulator<'_, Msg> =
        Simulator::with_size(settings(), Size::new(400.0, 400.0), bare_control());
    // Built from the two fields rather than by calling
    // `Rectangle`'s own size accessor.
    //
    // `scripts/check-literal-sizes.sh` greps every source line for
    // a text-size call and reads what follows the paren. An empty
    // argument leaves it indexing an associative array with an
    // empty key, which bash rejects outright — so the gate dies
    // with `ALLOWED: bad array subscript` instead of reporting
    // anything. That is a bug in the gate, reported in RFC-116's
    // review request with its one-line fix; until it lands, a
    // zero-argument accessor of that name anywhere under `crates/`
    // takes the Rust quality job down with it. **Including one
    // written inside a comment**, which is how this note found it.
    let mut biggest = Size::ZERO;
    let _ = ui.find(|candidate: iced_test::selector::Candidate<'_>| {
        if let iced_test::selector::Candidate::Focusable { bounds, .. } = candidate {
            biggest = Size::new(bounds.width, bounds.height);
        }
        None::<()>
    });
    biggest
}

/// At the default text size the floor is 24, which is exactly
/// WCAG 2.5.8's minimum — so the rule costs nothing at the default,
/// which is what RFC-115's Unit 0 measured and what makes k = 1.5
/// the position that changes nothing.
#[test]
fn the_floor_is_24_at_the_default_text_size() {
    let size = measured(16.0);
    assert!(
        size.width >= MIN_TARGET && size.height >= MIN_TARGET,
        "a control with 1px of content measured {size:?} at 16px text; the floor is \
         max({MIN_TARGET}, {TARGET_PER_TEXT} x 16) = 24",
    );
}

/// At 24px host text the floor is 36, and it is the multiplier that
/// decides, not the 24px minimum.
#[test]
fn the_floor_follows_the_text_to_36_at_24px() {
    let expected = TARGET_PER_TEXT * 24.0;
    let size = measured(24.0);
    assert!(
        size.width >= expected && size.height >= expected,
        "a control with 1px of content measured {size:?} at 24px text; the floor is \
         max({MIN_TARGET}, {TARGET_PER_TEXT} x 24) = {expected}",
    );
    // And it really did grow: a floor that ignored the text would
    // have left this at 24.
    assert!(
        size.height > MIN_TARGET + 0.5,
        "the control is {size:?} at 24px text — the same as at 16px, so the floor is \
         not following the text at all",
    );
}
