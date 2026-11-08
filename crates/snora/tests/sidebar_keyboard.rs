//! The sidebar item is a focus target and answers Enter and Space
//! (RFC-116 Unit 0).
//!
//! # What was wrong
//!
//! iced 0.14's `button` handles mouse and touch events only, and its
//! `operate` registers no focus target. Only `text_input` and
//! `text_editor` implement `Focusable`. So **no** snora control could
//! be focused and no key pressed one — established by experiment in
//! `.git-exclude/reviewed/iced-buttons-take-no-keyboard-input/`, not by
//! reading.
//!
//! # One row of the handoff cannot be tested end to end, and this says so
//!
//! The handoff asks for "Tab reaches the item". It cannot be shown in
//! one headless test, and the reason is not snora's:
//!
//! * iced binds no Tab. Focus moves only when something runs
//!   `focus_next()` / `focus_previous()`
//!   (`iced_runtime/src/widget/operation.rs:50,55`), which return a
//!   **`Task`**.
//! * `iced_test::Simulator` has no runtime. It can deliver events and
//!   run a `Selector`, and a `Selector` sees a focusable's state only
//!   as `&dyn Focusable` — read-only. There is no public way to run an
//!   arbitrary `Operation` against it.
//!
//! So "Tab reaches the item" is asserted as its two halves, each of
//! which *can* be measured:
//!
//! 1. **the rail offers focus targets at all** — [`the_rail_registers_a_focus_target_per_item`],
//!    which is the half that was missing, and which fails on `main`;
//! 2. **Tab maps to `focus_next`** — `snora::keyboard`'s own
//!    `only_tab_is_claimed`.
//!
//! What is *not* asserted anywhere is the composition: that running
//! `focus_next`'s task actually moves focus from one `control` to the
//! next. That needs a runtime. It is called out in RFC-116's review
//! request as the one gap in Unit 0's evidence, rather than papered
//! over with a test that looks like it covers it.

#![cfg(feature = "widgets")]

use iced::keyboard::key::Named;
use iced::widget::{row, space};
use iced::{Element, Length};
use iced_test::selector::Candidate;

use snora::widget::SideBarLabels;
use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

mod common;
use common::simulator;
use common::tab_router::tab_router;

const NAMES: [&str; 3] = ["Home", "Search", "Settings"];
const GLYPHS: [&str; 3] = ["1", "2", "3"];

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go(u8),
}

fn side_bar() -> SideBar<Msg, u8> {
    SideBar {
        items: NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| SideBarItem {
                view_id: i as u8,
                icon: Icon::Text(GLYPHS[i].to_owned()),
                tooltip: (*name).to_owned(),
                on_press: Msg::Go(i as u8),
            })
            .collect(),
        active: 0,
    }
}

fn rail(labels: SideBarLabels) -> Element<'static, Msg> {
    row![
        snora::widget::app_side_bar_with(side_bar(), LayoutDirection::Ltr, labels),
        space().width(Length::Fill)
    ]
    .into()
}

fn forms() -> [(&'static str, SideBarLabels); 3] {
    [
        ("Beneath", SideBarLabels::Beneath),
        ("Beside", SideBarLabels::Beside),
        ("Tooltip", SideBarLabels::Tooltip),
    ]
}

/// Every focusable in the tree, as `(is_focused, bounds)`.
fn focusables(labels: SideBarLabels) -> Vec<bool> {
    let mut ui = simulator(rail(labels));
    let mut out = Vec::new();
    let _ = ui.find(|candidate: Candidate<'_>| {
        if let Candidate::Focusable { state, .. } = candidate {
            out.push(state.is_focused());
        }
        None::<()>
    });
    out
}

// ---------------------------------------------------------------------
// Row 1a — the rail offers focus targets
// ---------------------------------------------------------------------

/// One focus target per item, in every form.
///
/// **This is the half of "Tab reaches it" that was missing.** On
/// `main`'s sidebar the count is zero in all three forms, because
/// `button` registers nothing — so there is nothing for `focus_next` to
/// land on however the key is routed.
#[test]
fn the_rail_registers_a_focus_target_per_item() {
    for (form, labels) in forms() {
        let found = focusables(labels);
        assert_eq!(
            found.len(),
            NAMES.len(),
            "{form}: the rail offers {} focus target(s) for {} items — \
             Tab has nothing to land on",
            found.len(),
            NAMES.len(),
        );
        assert!(
            found.iter().all(|focused| !focused),
            "{form}: an item is focused before anything focused it",
        );
    }
}

// ---------------------------------------------------------------------
// Row 1b — a click focuses it
// ---------------------------------------------------------------------

/// A click presses and does **not** focus (RFC-121), and Tab then carries on
/// from the item that was clicked.
///
/// This replaces the test that asserted the opposite, "clicking an item focuses
/// it, so a pointer user who then reaches for the keyboard carries on from where
/// they were". The promise is kept, by a different route: the click sets where
/// Tab starts from, not where focus is.
#[test]
fn a_click_focuses_nothing_and_tab_carries_on_after_the_item() {
    for (form, labels) in forms() {
        let mut ui = simulator(tab_router(rail(labels)));
        // The first item's icon is inside its target in every form.
        let glyph = ui
            .find(GLYPHS[0])
            .expect("the rail draws the first item's icon")
            .visible_bounds()
            .expect("the icon is visible");
        ui.point_at(glyph.center());
        let _ = ui.simulate(iced_test::simulator::click());

        let focused_now = |ui: &mut iced_test::Simulator<'_, Msg>| {
            let mut focused = Vec::new();
            let _ = ui.find(|candidate: Candidate<'_>| {
                if let Candidate::Focusable { state, .. } = candidate {
                    focused.push(state.is_focused());
                }
                None::<()>
            });
            focused
        };

        assert_eq!(
            focused_now(&mut ui),
            vec![false; NAMES.len()],
            "{form}: a click focused an item",
        );

        let _ = ui.tap_key(Named::Tab);
        assert_eq!(
            focused_now(&mut ui),
            vec![false, true, false],
            "{form}: Tab after a click on the first item did not land on the second",
        );
    }
}

// ---------------------------------------------------------------------
// Row 2 and 3 — Enter and Space press it, with the same message
// ---------------------------------------------------------------------

/// Focuses the first item by Tab (a click no longer focuses, RFC-121), then
/// taps `key`, and returns the messages that came out **after** the Tab.
fn press_with_key(labels: SideBarLabels, key: Named) -> Vec<Msg> {
    // The cursor is parked away, so that anything the key produces cannot be
    // attributed to a pointer resting on the item.
    let mut ui = simulator(tab_router(rail(labels)));
    ui.point_at(iced::Point::new(900.0, 900.0));
    let _ = ui.tap_key(Named::Tab);
    let before = ui.into_messages().count();

    let mut ui = simulator(tab_router(rail(labels)));
    ui.point_at(iced::Point::new(900.0, 900.0));
    let _ = ui.tap_key(Named::Tab);
    let _ = ui.tap_key(key);

    let all: Vec<Msg> = ui.into_messages().collect();
    all.into_iter().skip(before).collect()
}

/// Enter, once focused, emits the item's message — **the same message a
/// click emits**, not a second one.
#[test]
fn enter_presses_the_focused_item() {
    for (form, labels) in forms() {
        let after = press_with_key(labels, Named::Enter);
        assert_eq!(
            after,
            vec![Msg::Go(0)],
            "{form}: Enter on the focused item produced {after:?}",
        );
    }
}

/// Space does the same.
#[test]
fn space_presses_the_focused_item() {
    for (form, labels) in forms() {
        let after = press_with_key(labels, Named::Space);
        assert_eq!(
            after,
            vec![Msg::Go(0)],
            "{form}: Space on the focused item produced {after:?}",
        );
    }
}

/// A key press on an **unfocused** rail produces nothing.
///
/// Without this, `enter_presses_the_focused_item` would pass just as
/// well if `control` answered Enter whether focused or not — which
/// would make every rail item fire on one key press.
#[test]
fn a_key_on_an_unfocused_rail_presses_nothing() {
    for (form, labels) in forms() {
        for key in [Named::Enter, Named::Space] {
            let mut ui = simulator(rail(labels));
            let _ = ui.tap_key(key);
            let messages: Vec<Msg> = ui.into_messages().collect();
            assert!(
                messages.is_empty(),
                "{form}: {key:?} on a rail nobody focused produced {messages:?}",
            );
        }
    }
}

// ---------------------------------------------------------------------
// Row 4 — a click still presses it
// ---------------------------------------------------------------------

/// The pointer route is unchanged. This one passes on `main` too; it is
/// here so that gaining the keyboard cannot cost the pointer.
#[test]
fn a_click_still_presses_the_item() {
    for (form, labels) in forms() {
        let mut ui = simulator(rail(labels));
        let glyph = ui
            .find(GLYPHS[0])
            .expect("the rail draws the first item's icon")
            .visible_bounds()
            .expect("the icon is visible");
        ui.point_at(glyph.center());
        let _ = ui.simulate(iced_test::simulator::click());
        let messages: Vec<Msg> = ui.into_messages().collect();
        assert!(
            messages.contains(&Msg::Go(0)),
            "{form}: a click on the first item produced {messages:?}",
        );
    }
}

// ---------------------------------------------------------------------
// RFC-115's floor, on the real rail
// ---------------------------------------------------------------------

/// The sidebar's items are already above `max(24, 1.5 x text)` at both
/// text sizes, so the floor `control` applies costs the rail nothing.
///
/// **This is the companion to `control`'s own sizing tests**, which use
/// a control with 1 px of content because that is the only way to watch
/// the floor bind. Here the point is the opposite one: on the rail, it
/// does not bind, which is why RFC-115's k = 1.5 changes nothing for an
/// application that changes nothing.
#[test]
fn the_sidebar_item_is_already_above_the_floor() {
    /// Mirrors `control`'s own constants as expectations.
    const MIN_TARGET: f32 = 24.0;
    const TARGET_PER_TEXT: f32 = 1.5;

    for (form, labels) in forms() {
        for text in [16.0_f32, 24.0] {
            let floor = MIN_TARGET.max(TARGET_PER_TEXT * text);
            let settings = iced::Settings {
                default_text_size: iced::Pixels(text),
                fonts: vec![],
                ..iced::Settings::default()
            };
            let mut ui: iced_test::Simulator<'_, Msg> = iced_test::Simulator::with_size(
                settings,
                iced::Size::new(1200.0, 900.0),
                rail(labels),
            );

            let mut sizes = Vec::new();
            let _ = ui.find(|candidate: Candidate<'_>| {
                if let Candidate::Focusable { bounds, .. } = candidate {
                    // Built from the two fields; see `control.rs`'s
                    // note on `check-literal-sizes.sh`.
                    sizes.push(iced::Size::new(bounds.width, bounds.height));
                }
                None::<()>
            });

            assert_eq!(
                sizes.len(),
                NAMES.len(),
                "{form} at {text}px: {} focus targets for {} items",
                sizes.len(),
                NAMES.len(),
            );
            for (i, size) in sizes.iter().enumerate() {
                assert!(
                    size.width >= floor && size.height >= floor,
                    "{form} at {text}px: item {i} is {size:?}, under the \
                     max({MIN_TARGET}, {TARGET_PER_TEXT} x {text}) = {floor} floor",
                );
            }
        }
    }
}
