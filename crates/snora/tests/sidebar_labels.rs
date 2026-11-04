//! The sidebar names its items where a reader can see them (RFC-114).
//!
//! # What was wrong
//!
//! Every item on the rail was an icon and nothing else. Its name lived in
//! a tooltip, and iced opens a tooltip only while the cursor rests on the
//! item (`iced_widget-0.14.2/src/tooltip.rs:224`). So the name was there
//! for someone who already knew to hover, already knew which icon to
//! hover, and had a pointer at all — and absent for everyone else,
//! including every touch user.
//!
//! # What these tests measure
//!
//! Where things landed, read back from iced, rather than the arguments
//! handed to the builders. The rail is content-sized now, so its width is
//! a layout result and not a constant that can be asserted against
//! itself.
//!
//! # What they do not test, and cannot
//!
//! **Keyboard reachability.** iced 0.14's `button` takes no keyboard
//! input: its `update` handles mouse and touch only, and its `operate`
//! registers no focus target, so a button cannot be focused and no key
//! presses it. The rail is unreachable by keyboard in every form,
//! labelled or not, and nothing here should be read as claiming otherwise
//! (RFC-114's "Correction, 2026-11-04"; RFC-116 is the plan).

#![cfg(feature = "widgets")]

use iced::widget::{row, space};
use iced::{Element, Length, Point, Rectangle};

use snora::widget::SideBarLabels;
use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

mod common;
use common::{TOLERANCE, pointer_target, simulator};

/// Mirrors `snora_widgets::sidebar::LABELLED_RAIL_MAX_WIDTH` as an
/// **expectation**: the documented cap, confirmed by Unit 2's
/// measurement. It is not an input to anything under test.
const CAP: f32 = 240.0;

/// Mirrors `snora_widgets::sidebar::BUTTON_SIZE`: the icon's box, and so
/// the floor on an item's height in either labelled form.
const ICON_BOX: f32 = 48.0;

/// Item names. Distinct from the icon glyphs below so that `find` cannot
/// confuse one for the other.
const NAMES: [&str; 3] = ["Home", "Search", "Settings"];
/// Icon glyphs, one per item, each findable on its own.
const GLYPHS: [&str; 3] = ["1", "2", "3"];

/// The active item: the first, so its highlight is the one measured.
const ACTIVE: u8 = 0;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go(u8),
}

fn side_bar(names: &[&str]) -> SideBar<Msg, u8> {
    SideBar {
        items: names
            .iter()
            .enumerate()
            .map(|(i, name)| SideBarItem {
                view_id: i as u8,
                icon: Icon::Text(GLYPHS[i].to_owned()),
                tooltip: (*name).to_owned(),
                on_press: Msg::Go(i as u8),
            })
            .collect(),
        active: ACTIVE,
    }
}

/// A rail beside a page, which is how an application lays one out: the
/// rail has to take its own width from somewhere, and a rail alone in a
/// window would be handed the window's.
fn page(rail: Element<'static, Msg>) -> Element<'static, Msg> {
    row![rail, space().width(Length::Fill)].into()
}

/// The unstyled path.
fn unstyled(
    names: &'static [&'static str],
    direction: LayoutDirection,
    labels: SideBarLabels,
) -> impl Fn() -> Element<'static, Msg> {
    move || {
        page(snora::widget::app_side_bar_with(
            side_bar(names),
            direction,
            labels,
        ))
    }
}

/// The design path, light preset.
#[cfg(feature = "design")]
fn design(
    names: &'static [&'static str],
    direction: LayoutDirection,
    labels: SideBarLabels,
) -> impl Fn() -> Element<'static, Msg> {
    move || {
        let tokens = snora::design::Tokens::light();
        page(snora::design::widget::app_side_bar_with(
            &tokens,
            side_bar(names),
            direction,
            labels,
        ))
    }
}

/// Where a piece of text landed.
fn bounds_of(build: &impl Fn() -> Element<'static, Msg>, needle: &str) -> Rectangle {
    let mut ui = simulator(build());
    ui.find(needle)
        .unwrap_or_else(|_| panic!("{needle:?} is not drawn"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{needle:?} is drawn but not visible"))
}

/// How many times `needle` is drawn.
///
/// `Simulator::find` stops at the first match, so it cannot tell one
/// occurrence from two. The candidate sweep sees every text widget, which
/// is what distinguishes "named on the rail" from "named on the rail and
/// again in a tooltip".
fn occurrences(build: &impl Fn() -> Element<'static, Msg>, needle: &str) -> usize {
    let mut ui = simulator(build());
    let mut seen = 0usize;
    let _ = ui.find(|candidate: iced_test::selector::Candidate<'_>| {
        if matches!(candidate, iced_test::selector::Candidate::Text { content, .. } if content == needle)
        {
            seen += 1;
        }
        None::<()>
    });
    seen
}

// ---------------------------------------------------------------------
// Row 1 — the names are on the rail, with no pointer near it
// ---------------------------------------------------------------------

/// Checks every name is drawn at rest, which is RFC-114's whole point.
fn assert_names_at_rest(label: &str, build: impl Fn() -> Element<'static, Msg>) {
    let mut ui = simulator(build());
    for name in NAMES {
        assert!(
            ui.find(name).is_ok(),
            "{label}: {name:?} is not drawn on the resting rail — the only way to read it is to \
             hover the right icon, which a touch user cannot do at all",
        );
    }
}

#[test]
fn the_default_rail_names_its_items_at_rest_unstyled() {
    assert_names_at_rest(
        "unstyled default",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beneath),
    );
}

#[test]
fn the_beside_rail_names_its_items_at_rest_unstyled() {
    assert_names_at_rest(
        "unstyled Beside",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
    );
}

#[cfg(feature = "design")]
#[test]
fn the_default_rail_names_its_items_at_rest_design() {
    assert_names_at_rest(
        "design default",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beneath),
    );
}

#[cfg(feature = "design")]
#[test]
fn the_beside_rail_names_its_items_at_rest_design() {
    assert_names_at_rest(
        "design Beside",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
    );
}

/// The default is `Beneath`, not `Tooltip`: `app_side_bar` and
/// `app_side_bar_with(.., Beneath)` draw the same rail.
#[test]
fn app_side_bar_defaults_to_the_labelled_rail() {
    let mut plain = simulator(page(snora::widget::app_side_bar(
        side_bar(&NAMES),
        LayoutDirection::Ltr,
    )));
    for name in NAMES {
        assert!(
            plain.find(name).is_ok(),
            "app_side_bar does not name {name:?} at rest, so its default is not Beneath",
        );
    }
}

// ---------------------------------------------------------------------
// Row 2 — `Beneath` puts the name below the icon, `Beside` next to it
// ---------------------------------------------------------------------

/// `Beneath`: the name's top edge is at or below the icon's bottom edge,
/// and the two share a centre line.
fn assert_name_beneath_icon(label: &str, build: impl Fn() -> Element<'static, Msg>) {
    let icon = bounds_of(&build, GLYPHS[0]);
    let name = bounds_of(&build, NAMES[0]);

    assert!(
        name.y >= icon.y + icon.height - TOLERANCE,
        "{label}: the name's top ({}) is above the icon's bottom ({}) — this is Beside's row, not \
         Beneath's column (icon {icon:?}, name {name:?})",
        name.y,
        icon.y + icon.height,
    );
    assert!(
        (name.center().x - icon.center().x).abs() <= TOLERANCE,
        "{label}: the name's centre x ({}) is not under the icon's ({})",
        name.center().x,
        icon.center().x,
    );
}

#[test]
fn beneath_puts_the_name_below_the_icon_unstyled() {
    assert_name_beneath_icon(
        "unstyled",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beneath),
    );
}

#[cfg(feature = "design")]
#[test]
fn beneath_puts_the_name_below_the_icon_design() {
    assert_name_beneath_icon(
        "design",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beneath),
    );
}

/// `Beside`: the name is level with the icon, not under it, and on the
/// icon's end side.
fn assert_name_beside_icon(label: &str, build: impl Fn() -> Element<'static, Msg>) {
    let icon = bounds_of(&build, GLYPHS[0]);
    let name = bounds_of(&build, NAMES[0]);

    assert!(
        name.y < icon.y + icon.height - TOLERANCE,
        "{label}: the name's top ({}) is at or below the icon's bottom ({}) — this is Beneath's \
         column, not Beside's row",
        name.y,
        icon.y + icon.height,
    );
    assert!(
        (name.center().y - icon.center().y).abs() <= TOLERANCE,
        "{label}: the name's centre y ({}) is not level with the icon's ({})",
        name.center().y,
        icon.center().y,
    );
}

#[test]
fn beside_puts_the_name_level_with_the_icon_unstyled() {
    assert_name_beside_icon(
        "unstyled",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
    );
}

#[cfg(feature = "design")]
#[test]
fn beside_puts_the_name_level_with_the_icon_design() {
    assert_name_beside_icon(
        "design",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
    );
}

// ---------------------------------------------------------------------
// Row 5 — RTL mirrors `Beside`
// ---------------------------------------------------------------------

/// Under LTR the icon is at the start (left) of the name; under RTL it is
/// at the start (right). Asserted as an order, which is what mirroring
/// means, rather than as two coordinates that would also hold if the rail
/// happened to move.
fn assert_beside_mirrors(
    label: &str,
    ltr: impl Fn() -> Element<'static, Msg>,
    rtl: impl Fn() -> Element<'static, Msg>,
) {
    let ltr_icon = bounds_of(&ltr, GLYPHS[0]);
    let ltr_name = bounds_of(&ltr, NAMES[0]);
    assert!(
        ltr_icon.center().x < ltr_name.center().x,
        "{label} LTR: the icon ({:?}) is not before the name ({:?})",
        ltr_icon,
        ltr_name,
    );

    let rtl_icon = bounds_of(&rtl, GLYPHS[0]);
    let rtl_name = bounds_of(&rtl, NAMES[0]);
    assert!(
        rtl_name.center().x < rtl_icon.center().x,
        "{label} RTL: the name ({:?}) is not before the icon ({:?}) — the labelled row ignores \
         `direction`, so an RTL application gets an LTR rail",
        rtl_name,
        rtl_icon,
    );
}

#[test]
fn beside_mirrors_under_rtl_unstyled() {
    assert_beside_mirrors(
        "unstyled",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
        unstyled(&NAMES, LayoutDirection::Rtl, SideBarLabels::Beside),
    );
}

#[cfg(feature = "design")]
#[test]
fn beside_mirrors_under_rtl_design() {
    assert_beside_mirrors(
        "design",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
        design(&NAMES, LayoutDirection::Rtl, SideBarLabels::Beside),
    );
}

// ---------------------------------------------------------------------
// Each item is one target, at least 48px tall, and the highlight covers
// all of it
// ---------------------------------------------------------------------

/// The item's pointer target, found by clicking (RFC-100): the largest
/// region around the name whose four corners all press that item. Several
/// containers hold any given control, so picking one out of the tree
/// measures whichever happens to be first.
fn item_target(label: &str, build: &impl Fn() -> Element<'static, Msg>, index: usize) -> Rectangle {
    let name = bounds_of(build, NAMES[index]);
    let id = index as u8;
    pointer_target(label, build, name.center(), move |m: &Msg| {
        *m == Msg::Go(id)
    })
}

/// Both the height floor and the highlight's reach: the item is one
/// button, so what a click hits is also what the active background is
/// painted on.
fn assert_item_is_one_target(label: &str, build: impl Fn() -> Element<'static, Msg>) {
    for index in 0..NAMES.len() {
        let target = item_target(label, &build, index);
        let icon = bounds_of(&build, GLYPHS[index]);
        let name = bounds_of(&build, NAMES[index]);

        assert!(
            target.height >= ICON_BOX - TOLERANCE,
            "{label}: item {index} is {}px tall, below the {ICON_BOX}px floor",
            target.height,
        );
        for (what, part) in [("icon", icon), ("name", name)] {
            assert!(
                target.contains(part.center()) && target.contains(Point::new(part.x, part.y)),
                "{label}: item {index}'s {what} ({part:?}) is outside the item's own target \
                 ({target:?}) — the active highlight would stop short of it",
            );
        }
    }
}

#[test]
fn every_item_is_one_target_covering_icon_and_name_unstyled() {
    assert_item_is_one_target(
        "unstyled Beneath",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beneath),
    );
    assert_item_is_one_target(
        "unstyled Beside",
        unstyled(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
    );
}

#[cfg(feature = "design")]
#[test]
fn every_item_is_one_target_covering_icon_and_name_design() {
    assert_item_is_one_target(
        "design Beneath",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beneath),
    );
    assert_item_is_one_target(
        "design Beside",
        design(&NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
    );
}

// ---------------------------------------------------------------------
// Row 4 — the name wraps at the cap and is never clipped
// ---------------------------------------------------------------------

/// A name with no space in it, longer than any cap: `Wrapping::Word`
/// alone would push it past the cap, and `Wrapping::None` would cut it.
const UNBROKEN: &str = "Konfigurationsverwaltungsoberflaeche";
static UNBROKEN_NAMES: [&str; 1] = [UNBROKEN];

/// The rail stays inside the cap and the long name is still whole: its
/// drawn box is taller than one line, which is what wrapping looks like
/// from outside.
fn assert_long_name_wraps_inside_the_cap(
    label: &str,
    build: impl Fn() -> Element<'static, Msg>,
    one_line: f32,
) {
    let name = bounds_of(&build, UNBROKEN);
    let icon = bounds_of(&build, GLYPHS[0]);

    // The rail: the region that holds the icon and is no wider than the
    // cap, found from the frame rather than assumed.
    assert!(
        name.width <= CAP,
        "{label}: the name's own box is {}px wide, past the {CAP}px cap",
        name.width,
    );
    assert!(
        icon.x + icon.width <= CAP,
        "{label}: the icon's right edge ({}) is past the {CAP}px cap, so the rail is wider than \
         the cap",
        icon.x + icon.width,
    );
    assert!(
        name.height > one_line * 1.5,
        "{label}: a {}-character name with no break in it is {}px tall — about one line of \
         {one_line}px text, so it did not wrap. What is past the cap is cut off, and a reader \
         sees a name that stops mid-word",
        UNBROKEN.chars().count(),
        name.height,
    );
}

#[test]
fn a_long_name_wraps_inside_the_cap_unstyled() {
    // The unstyled path inherits the host's `default_text_size`, which
    // `iced::Settings::default()` puts at 16 (RFC-104).
    assert_long_name_wraps_inside_the_cap(
        "unstyled Beneath",
        unstyled(
            &UNBROKEN_NAMES,
            LayoutDirection::Ltr,
            SideBarLabels::Beneath,
        ),
        16.0,
    );
    assert_long_name_wraps_inside_the_cap(
        "unstyled Beside",
        unstyled(&UNBROKEN_NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
        16.0,
    );
}

#[cfg(feature = "design")]
#[test]
fn a_long_name_wraps_inside_the_cap_design() {
    // The design path uses `Typography::label`, 14px in all four shipped
    // presets.
    let one_line = snora::design::Tokens::light().typography.label.size;
    assert_long_name_wraps_inside_the_cap(
        "design Beneath",
        design(
            &UNBROKEN_NAMES,
            LayoutDirection::Ltr,
            SideBarLabels::Beneath,
        ),
        one_line,
    );
    assert_long_name_wraps_inside_the_cap(
        "design Beside",
        design(&UNBROKEN_NAMES, LayoutDirection::Ltr, SideBarLabels::Beside),
        one_line,
    );
}

// ---------------------------------------------------------------------
// The icon survives a long name in both directions
// ---------------------------------------------------------------------

/// Beside a long name, the icon is drawn the same size in both
/// directions.
///
/// **Asserted as a comparison between the directions, not as a floor.**
/// Under RTL the name is laid out before the icon, and iced's row hands
/// the first child the room and the next what is left. Measured with a
/// 36-character name and the name unbounded: the glyph is 7.63px wide
/// under LTR and **2.26px** under RTL. A floor would have to be set
/// above 2.26 to catch that, which is picking a number to make a test
/// fail; the two directions differing at all is the defect, and nothing
/// about a name's length may change the icon's size.
fn assert_icon_is_the_same_in_both_directions(label: &str, labels: SideBarLabels) {
    let ltr = bounds_of(
        &unstyled(&UNBROKEN_NAMES, LayoutDirection::Ltr, labels),
        GLYPHS[0],
    );
    let rtl = bounds_of(
        &unstyled(&UNBROKEN_NAMES, LayoutDirection::Rtl, labels),
        GLYPHS[0],
    );
    assert!(
        (ltr.width - rtl.width).abs() <= TOLERANCE && (ltr.height - rtl.height).abs() <= TOLERANCE,
        "{label}: beside a {}-character name the icon is drawn {:.2}x{:.2} under LTR and \
         {:.2}x{:.2} under RTL — the name is taking the icon's room in one direction only",
        UNBROKEN.chars().count(),
        ltr.width,
        ltr.height,
        rtl.width,
        rtl.height,
    );
}

#[test]
fn a_long_name_does_not_squeeze_the_icon_in_either_direction() {
    assert_icon_is_the_same_in_both_directions("Beside", SideBarLabels::Beside);
    assert_icon_is_the_same_in_both_directions("Beneath", SideBarLabels::Beneath);
}

// ---------------------------------------------------------------------
// A labelled rail has no tooltip
// ---------------------------------------------------------------------

/// The name appears once — on the rail. In the tooltip form it appears
/// only in the overlay, and a labelled form that kept the tooltip would
/// draw it twice.
#[test]
fn a_labelled_rail_draws_each_name_once() {
    for labels in [SideBarLabels::Beneath, SideBarLabels::Beside] {
        let build = unstyled(&NAMES, LayoutDirection::Ltr, labels);
        for name in NAMES {
            let n = occurrences(&build, name);
            assert_eq!(
                n, 1,
                "{labels:?}: {name:?} is drawn {n} times; a labelled rail names its item on the \
                 rail and nowhere else",
            );
        }
    }
}
