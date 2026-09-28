//! RFC-103's **state** register for `snora-widgets`' own surfaces.
//!
//! RFC-093's register (`design::channel_register`) covers **variants** —
//! that a `Tone` changes a style only through colour. This one covers
//! **states**: active, selected, open, disabled, leaf. The difference is
//! in what each asserts. A variant register asserts that *only colour*
//! varies. A state register asserts that *something other than colour*
//! does, because a state a user must identify cannot be carried by hue
//! alone (WCAG 1.4.1).
//!
//! **What this proves and what it does not.** Like RFC-093's register,
//! this inspects style functions and names the rendered tests that cover
//! the rest. It is not a 1.4.1 conformance check: what a user can
//! actually tell apart on a given screen is a property of the whole
//! rendered surface, and no test over style structs sees that. What is
//! checkable, and all this claims, is that for each state below snora
//! draws a cue that is not only a hue change — or, where it does not,
//! that the gap is recorded here rather than left to be discovered.
//!
//! # The register
//!
//! | State | Cue | Asserted by |
//! |---|---|---|
//! | Tab active | `Shape` — a 2px underline element | `crates/snora/tests/tab_bar_edges.rs` |
//! | Sidebar item active | `Luminance` ≥ 3.0 against the rail | `contrast_tests` (RFC-085) |
//! | Menu open | `Shape` — the dropdown is rendered | `crates/snora/tests/menu_dropdown.rs` |
//! | Breadcrumb leaf | `Position` — last, and plain text | documentation (RFC-103 Q-3) |
//! | Chip selected | `Luminance` ≥ 3.0 against unselected | this module |
//! | Chip selected, disabled | `Luminance` ≥ 2.6 against enabled | this module |
//! | Chip unselected, disabled | **none** | **nothing** — see "Gaps" |
//! | Design button disabled | `Luminance` ≥ 2.9 against enabled | this module |
//!
//! **The removable chip's remove control** is styled by the same two
//! functions as the chip body ([`chip_style_selected`] /
//! [`chip_style_unselected`]), so the two chip-disabled rows cover it
//! too. It is not a separate row because it is not a separate style; if
//! it ever gets one, it needs its own row here.
//!
//! **Hover and pressed are deliberately absent** (RFC-103 Q-2). They are
//! transient pointer feedback, not states a user must be able to
//! identify; registering them would dilute the register with rows no
//! conformance question turns on.
//!
//! # Gaps this register records
//!
//! A register that only listed the states with cues would be a list of
//! good news. This one is the reason it exists:
//!
//! - **A disabled *unselected* chip has no cue at all.** Measured below:
//!   its fill differs from the enabled one by **1.00–1.05:1**, because
//!   `surface` at half alpha over a page that is nearly `surface` is
//!   nearly `surface`. Its text colour and border do not change either.
//!   A disabled unselected chip is indistinguishable from an enabled one.
//!   Closing that is a change to `chip_style_unselected`, which is a
//!   design decision rather than a test one.
//!
//! [`states_without_a_cue`] pins it, so that closing it fails this
//! module and forces the register to be updated rather than silently
//! drifting back into good news.
//!
//! **The menu's cue was the second gap**, until RFC-103's review
//! required it closed: `crates/snora/tests/menu_dropdown.rs` now asserts
//! that an open menu draws its dropdown and a closed one does not.

use iced::widget::button::{self, Status};
use iced::{Background, Color};
use snora_design::{Tokens, contrast};
use snora_style::color::to_iced_color;

use crate::design::chip::{chip_style_selected, chip_style_unselected};

/// Every state `snora-widgets` draws for its own surfaces.
///
/// Found by grepping this crate for `is_active`, `selected`, `is_leaf`
/// and `Status::Disabled`, and then checking which controls can actually
/// reach `Disabled`: a button is disabled only when its press message is
/// `None`, and only the chip's `on_toggle` / `on_remove` and
/// `design::button`'s `*_maybe` constructors take an `Option`. The tab,
/// sidebar, breadcrumb and menu buttons all receive a message
/// unconditionally, so they cannot render disabled — checked at each
/// call site, not assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    TabActive,
    SidebarItemActive,
    MenuOpen,
    BreadcrumbLeaf,
    ChipSelected,
    ChipSelectedDisabled,
    ChipUnselectedDisabled,
    DesignButtonDisabled,
}

impl State {
    /// Hand-listed, because Rust has no reflection over enum variants.
    /// What *is* compiler-enforced is [`cue`]: it matches without a
    /// wildcard arm, so a new variant fails to compile until it is given
    /// a cue (RFC-063's pattern).
    const ALL: [State; 8] = [
        State::TabActive,
        State::SidebarItemActive,
        State::MenuOpen,
        State::BreadcrumbLeaf,
        State::ChipSelected,
        State::ChipSelectedDisabled,
        State::ChipUnselectedDisabled,
        State::DesignButtonDisabled,
    ];
}

/// What tells a user the state is on, beyond its hue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Cue {
    /// An element exists in the state and not out of it.
    ///
    /// `asserted_by` is the test's name and the source of the file it
    /// lives in, so that [`named_shape_tests_exist`] can check the name
    /// still resolves instead of trusting a comment. `None` would mean
    /// no test covers it; there is none such today.
    Shape {
        asserted_by: Option<(&'static str, &'static str)>,
    },
    /// The state's own surface differs in luminance from the surface it
    /// is read against, by at least `min`:1.
    Luminance { min: f32 },
    /// The state is carried by where the element sits, not by how it is
    /// painted.
    Position,
    /// **Snora draws nothing.** `measured` is the ratio that ought to be
    /// a cue and is not.
    Missing { measured_at_most: f32 },
}

/// The register itself. Exhaustive on purpose: a new [`State`] cannot be
/// added without deciding what tells a user it is on.
pub(crate) const fn cue(state: State) -> Cue {
    match state {
        State::TabActive => Cue::Shape {
            asserted_by: Some((
                "unstyled_active_tab_underline_spans_the_tab",
                TAB_BAR_EDGES_SRC,
            )),
        },
        State::MenuOpen => Cue::Shape {
            asserted_by: Some(("an_open_menu_draws_its_dropdown", MENU_DROPDOWN_SRC)),
        },
        State::SidebarItemActive => Cue::Luminance { min: 3.0 },
        State::ChipSelected => Cue::Luminance { min: 3.0 },
        // Measured 2.64 (dark) .. 3.59 (hc_light); floor is the minimum
        // rounded down to one decimal.
        State::ChipSelectedDisabled => Cue::Luminance { min: 2.6 },
        // Measured 2.92 (danger, light) .. 4.05; same rule.
        State::DesignButtonDisabled => Cue::Luminance { min: 2.9 },
        State::ChipUnselectedDisabled => Cue::Missing {
            measured_at_most: 1.1,
        },
        State::BreadcrumbLeaf => Cue::Position,
    }
}

/// The sources of the rendered tests this register names, so that
/// renaming or deleting one breaks this crate rather than leaving a
/// register entry pointing at nothing.
const TAB_BAR_EDGES_SRC: &str = include_str!("../../snora/tests/tab_bar_edges.rs");
const MENU_DROPDOWN_SRC: &str = include_str!("../../snora/tests/menu_dropdown.rs");

fn presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

fn sn(color: Color) -> snora_design::Color {
    snora_design::Color::rgba(color.r, color.g, color.b, color.a)
}

/// The style's solid fill, or fully transparent when it paints none.
fn fill(style: &button::Style) -> Color {
    match style.background {
        Some(Background::Color(color)) => color,
        _ => Color::TRANSPARENT,
    }
}

/// A colour as it reaches the eye: composited over the page, because a
/// half-alpha fill is not a colour until something is behind it.
fn over_page(color: Color, tokens: &Tokens) -> snora_design::Color {
    contrast::composite_over(sn(color), sn(to_iced_color(tokens.palette.background)))
}

fn ratio_over_page(a: Color, b: Color, tokens: &Tokens) -> f32 {
    contrast::contrast_ratio(over_page(a, tokens), over_page(b, tokens))
}

/// The floor a [`Cue::Luminance`] entry carries, for the states this
/// module asserts itself.
fn luminance_min(state: State) -> f32 {
    match cue(state) {
        Cue::Luminance { min } => min,
        other => panic!("{state:?} is registered as {other:?}, not a luminance cue"),
    }
}

// ---------------------------------------------------------------------
// The register's own shape
// ---------------------------------------------------------------------

/// Every state has a cue, and the two that do not are exactly the two
/// the module documents.
///
/// **This test fails when a gap is closed**, which is the point: the
/// register must be updated rather than quietly becoming out of date in
/// the flattering direction.
#[test]
fn states_without_a_cue() {
    let missing: Vec<State> = State::ALL
        .into_iter()
        .filter(|s| matches!(cue(*s), Cue::Missing { .. }))
        .collect();
    assert_eq!(
        missing,
        vec![State::ChipUnselectedDisabled],
        "the set of states with no cue changed; update the register and its \
         module documentation, and tell the architect either way",
    );

    let unasserted_shapes: Vec<State> = State::ALL
        .into_iter()
        .filter(|s| matches!(cue(*s), Cue::Shape { asserted_by: None }))
        .collect();
    assert!(
        unasserted_shapes.is_empty(),
        "these shape cues have no test: {unasserted_shapes:?}. Every shape cue names one \
         (RFC-103 R-1); add the test and point the entry at it, or say here why not",
    );
}

/// Every `Shape` entry that names a test names one that exists.
#[test]
fn named_shape_tests_exist() {
    for state in State::ALL {
        if let Cue::Shape {
            asserted_by: Some((name, source)),
        } = cue(state)
        {
            assert!(
                source.contains(&format!("fn {name}(")),
                "{state:?} names the test {name}, which is not in the file the register \
                 includes for it",
            );
        }
    }
}

// ---------------------------------------------------------------------
// The cues this module asserts
// ---------------------------------------------------------------------

/// A selected chip differs from an unselected one by luminance, not only
/// by hue.
///
/// Measured: 6.19 / 6.24 / 10.25 / 11.75:1 across the four presets, so
/// the 3.0 floor has room. Callers who need a second, non-visual cue can
/// add a mark to the label; RFC-103 Q-1 keeps that on record as the path
/// if one asks for it.
#[test]
fn chip_selected_differs_from_unselected_by_luminance() {
    let min = luminance_min(State::ChipSelected);
    for (name, tokens) in presets() {
        let selected = fill(&chip_style_selected(&tokens, Status::Active));
        let unselected = fill(&chip_style_unselected(&tokens, Status::Active));
        let ratio = contrast::contrast_ratio(sn(selected), sn(unselected));
        assert!(
            ratio >= min,
            "{name}: a selected chip's fill differs from an unselected one by {ratio:.2}:1, \
             under the {min}:1 floor — the only thing distinguishing a selected chip would be \
             its hue",
        );
    }
}

/// A disabled chip differs from an enabled one, when it is selected.
///
/// Measured against the enabled fill, both composited over the page:
/// 2.82 (light) / 2.64 (dark) / 3.59 (hc_light) / 3.48 (hc_dark). The
/// floor is the minimum rounded down, so it holds the measured margin
/// without pinning it to one preset's exact value.
#[test]
fn chip_selected_disabled_differs_from_enabled() {
    let min = luminance_min(State::ChipSelectedDisabled);
    for (name, tokens) in presets() {
        let enabled = fill(&chip_style_selected(&tokens, Status::Active));
        let disabled = fill(&chip_style_selected(&tokens, Status::Disabled));
        let ratio = ratio_over_page(enabled, disabled, &tokens);
        assert!(
            ratio >= min,
            "{name}: a disabled selected chip differs from an enabled one by {ratio:.2}:1, \
             under the {min}:1 floor",
        );
    }
}

/// The gap, pinned: a disabled **unselected** chip is indistinguishable
/// from an enabled one.
///
/// Measured 1.04 / 1.05 / 1.00 / 1.00 — its fill is `surface` at half
/// alpha over a page that is nearly `surface`, and neither its text
/// colour nor its border changes at all. This asserts the gap is still
/// there, so that closing it fails here and the register gets updated.
/// It is not an assertion that the gap is acceptable.
#[test]
fn chip_unselected_disabled_has_no_cue() {
    let Cue::Missing { measured_at_most } = cue(State::ChipUnselectedDisabled) else {
        panic!("ChipUnselectedDisabled is no longer registered as a missing cue");
    };
    for (name, tokens) in presets() {
        let enabled = chip_style_unselected(&tokens, Status::Active);
        let disabled = chip_style_unselected(&tokens, Status::Disabled);
        let ratio = ratio_over_page(fill(&enabled), fill(&disabled), &tokens);
        assert!(
            ratio <= measured_at_most,
            "{name}: a disabled unselected chip now differs from an enabled one by {ratio:.2}:1 \
             — the gap this register recorded has been closed. Move the state off \
             Cue::Missing, give it a measured floor, and update the module documentation",
        );
        assert_eq!(
            enabled.text_color, disabled.text_color,
            "{name}: the text colour now changes when an unselected chip is disabled; the \
             register says nothing does",
        );
        assert_eq!(
            enabled.border, disabled.border,
            "{name}: the border now changes when an unselected chip is disabled; the register \
             says nothing does",
        );
    }
}

/// A disabled prefab button differs from an enabled one — in the channel
/// that carries its cue.
///
/// The channel differs by family, and naming it is the point: `primary`
/// and `danger` fill their button, so their cue is the fill;
/// `secondary` and `ghost` are transparent in both states, so theirs is
/// the label. Asserting "whichever channel happens to differ" would pass
/// on a change that moved the cue somewhere a user does not look.
///
/// Measured on each family's own channel: 2.92 (danger, light) through
/// 4.05 (hc_light).
#[test]
fn design_button_disabled_differs_from_enabled() {
    /// Which channel carries each family's disabled cue.
    enum Channel {
        Fill,
        Label,
    }
    /// One of `snora_style::button`'s style functions.
    type StyleFn = fn(&Tokens, Status) -> button::Style;

    let families: [(&str, StyleFn, Channel); 4] = [
        ("primary", snora_style::button::primary, Channel::Fill),
        ("secondary", snora_style::button::secondary, Channel::Label),
        ("ghost", snora_style::button::ghost, Channel::Label),
        ("danger", snora_style::button::danger, Channel::Fill),
    ];

    let min = luminance_min(State::DesignButtonDisabled);
    for (preset, tokens) in presets() {
        for (family, style, channel) in &families {
            let enabled = style(&tokens, Status::Active);
            let disabled = style(&tokens, Status::Disabled);
            let (a, b, what) = match channel {
                Channel::Fill => (fill(&enabled), fill(&disabled), "fill"),
                Channel::Label => (enabled.text_color, disabled.text_color, "label"),
            };
            let ratio = ratio_over_page(a, b, &tokens);
            assert!(
                ratio >= min,
                "{preset}/{family}: a disabled button's {what} differs from an enabled one by \
                 {ratio:.2}:1, under the {min}:1 floor",
            );
        }
    }
}
