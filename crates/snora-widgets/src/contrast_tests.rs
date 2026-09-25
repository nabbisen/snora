//! Widget-layer contrast suite (RFC-085).
//!
//! Every contrast assertion before this suite lived in `snora-design` and
//! tested **tokens against roles** (`Palette::usages`, RFC-063) — whether
//! `border` clears its floor against `background`, whether `text_primary`
//! clears AA against `surface_raised`, and so on. That suite is correct
//! and heavily hardened (RFC-058, RFC-063, RFC-065, RFC-066, RFC-071,
//! RFC-081). It cannot see this crate: `menu_button_style` invents a
//! pairing — `primary.weak.color` used as a foreground — that is not a
//! token-to-role assignment at all, just a render-time decision made in
//! `snora-widgets`. `Palette::usages` has no way to know it exists.
//!
//! # What is derived and what is not
//!
//! **Derived, not hand-listed:**
//! - Every [`button::Status`] variant ([`ALL_STATUSES`]) — a closed,
//!   externally-defined enum; iterated exhaustively via an array literal
//!   because Rust has no reflection over enum variants, but adding a
//!   variant to `button::Status` upstream would be caught by iced's own
//!   compile break, not silently skipped here.
//! - All four built-in token presets, via [`snora_design::Tokens`]'s own
//!   four constructors — the same set `snora-design`'s own suite uses.
//! - Both theme paths (Q-3): two stock `iced::Theme` variants and the
//!   four `design`-derived ones, in [`theme_contexts`].
//!
//! **Not derivable, and stated rather than hidden (per the Handoff's
//! explicit instruction to say so rather than quietly hand-list):**
//! Rust has no way to enumerate "every function in this crate that
//! returns a `button::Style` or `container::Style`" — there is no
//! reflection over functions or impls. The six functions tested below
//! (`menu_button_style`, `chrome_container_style_with_radius`,
//! `sidebar_button_style`, `tab_bar_container_style`, `tab_button_style`,
//! `crumb_button_style`), plus the three RFC-102 added
//! (`tab_indicator_style`, `tab_bar_rule_style`,
//! `unstyled_tooltip_body_style`), were found by grepping this crate's source for
//! every `-> button::Style` / `-> container::Style` return type — a
//! search anyone can re-run to check this list is still complete, but
//! not a compiler-enforced one. **If a new widget style function is
//! added and not added here, this suite will not catch it.** That is the
//! honest limit of this approach, named rather than hidden behind a
//! test count that looks complete.
//!
//! `tab_bar_container_style` is the one name on that list with nothing
//! to assert here since RFC-102: it paints no border and no background
//! at all, so it has no colour pairing. That it stays that way — an
//! all-sided border is what outlined the whole bar — is asserted by
//! `crate::tab::tests::tab_bar_container_style_paints_no_border`.
//!
//! For each function, the background it is *actually* painted over is
//! also not derivable from its own signature — several of them return
//! `background: None`, meaning "whatever is beneath in the render tree,"
//! which is a fact about the calling widget's composition, not something
//! the style function itself states. Resolved by reading each call site
//! (`menu.rs`, `header.rs`/`footer.rs`, `sidebar.rs`, `tab.rs`,
//! `crumb.rs`) and recorded per case below.
//!
//! # The `Disabled` exemption
//!
//! WCAG 2.1 SC 1.4.3 explicitly exempts text that is part of an
//! **inactive** user interface component from any contrast requirement.
//! None of the six functions here currently give `Disabled` a distinct,
//! dimmed treatment (it falls through the same match arm as `Active`),
//! so testing it would only assert that the *active* pairing holds,
//! under a different name. Excluded from the contrast-checked status
//! set; still constructed (to confirm the function does not panic) via
//! [`ALL_STATUSES`], just not asserted on.
//!
//! # Cost accepted (Q-3)
//!
//! Six of these contexts go through `iced::Theme::extended_palette()`'s
//! own tier-derivation algorithm (`Pair::new`, `Primary::generate`, and
//! friends) for the two stock themes, and through `snora_style::theme`'s
//! derivation for the four `design` ones — both are **iced's own**
//! palette machinery, not snora's. An iced upgrade that changes how
//! `extended_palette()` derives `weak`/`strong` tiers, or how the two
//! built-in `Theme::Light`/`Theme::Dark` palettes are defined, can shift
//! these numbers without any change on snora's side. That is a real
//! maintenance liability accepted deliberately (RFC-085 Q-3) — this
//! suite is at least as exposed to an iced upgrade as any other contrast
//! assertion in the project, and more exposed than `snora-design`'s own
//! suite, which never touches iced at all.

use iced::widget::button;
use iced::{Color, Theme};

use snora_design::Tokens;
use snora_design::contrast::contrast_ratio;

use crate::crumb::crumb_button_style;
use crate::sidebar::{sidebar_button_style, unstyled_tooltip_body_style};
use crate::style::{chrome_container_style_with_radius, menu_button_style, sidebar_active_color};
use crate::tab::{tab_bar_rule_style, tab_button_style, tab_indicator_style};

/// WCAG 2.1 SC 1.4.3 normal-text minimum. Three copies of this name
/// exist (this one, `snora-design/src/tests.rs`, and
/// `snora/src/toast/contrast_tests.rs`); none can be shared directly.
/// **Enforced by `scripts/check-wcag-floors.sh`**, not by hand — see
/// that script's own header for why a comment was not enough (audit
/// 2026-09-12, C-1).
const AA_TEXT: f32 = 4.5;

/// WCAG 2.1 SC 1.4.11 non-text minimum, for borders that identify a
/// component boundary. **Four** copies of this name exist — this one,
/// `snora-design/src/tests.rs`, `snora/src/toast/contrast_tests.rs`,
/// and `snora/src/design/render/tests.rs` (not the same sibling set as
/// [`AA_TEXT`] above, which has three). Enforced by
/// `scripts/check-wcag-floors.sh`.
const NON_TEXT_MIN: f32 = 3.0;

/// Every [`button::Status`] variant, derived from the enum rather than
/// re-declared piecemeal per test. See the module doc's "Disabled
/// exemption" section for why not all four are contrast-checked.
const ALL_STATUSES: [button::Status; 4] = [
    button::Status::Active,
    button::Status::Hovered,
    button::Status::Pressed,
    button::Status::Disabled,
];

/// [`ALL_STATUSES`] minus `Disabled` (WCAG 1.4.3 exemption, see module
/// doc) — the statuses actually asserted on.
fn contrast_checked_statuses() -> impl Iterator<Item = button::Status> {
    ALL_STATUSES
        .into_iter()
        .filter(|s| !matches!(s, button::Status::Disabled))
}

/// Both theme paths (Q-3): two stock `iced::Theme` variants most
/// consumers start on, and the four `design`-derived ones snora ships.
fn theme_contexts() -> Vec<(&'static str, Theme)> {
    vec![
        ("stock Light", Theme::Light),
        ("stock Dark", Theme::Dark),
        ("design light", snora_style::theme::theme(&Tokens::light())),
        ("design dark", snora_style::theme::theme(&Tokens::dark())),
        (
            "design high_contrast_light",
            snora_style::theme::theme(&Tokens::high_contrast_light()),
        ),
        (
            "design high_contrast_dark",
            snora_style::theme::theme(&Tokens::high_contrast_dark()),
        ),
    ]
}

fn to_sn(c: Color) -> snora_design::Color {
    snora_design::Color::rgba(c.r, c.g, c.b, c.a)
}

/// Returns a failure description if below [`AA_TEXT`], or `None` if it
/// clears the floor. Callers collect every failure across the full
/// (theme × status × ...) sweep and report them together — a single
/// `assert!` per combination would stop at the first failure and hide
/// how many others exist, which is exactly the "state every figure"
/// requirement this suite exists to satisfy.
fn text_contrast_failure(
    context: &str,
    case: &str,
    status: button::Status,
    fg: Color,
    bg: Color,
) -> Option<String> {
    let r = contrast_ratio(to_sn(fg), to_sn(bg));
    (r < AA_TEXT).then(|| {
        format!(
            "{context}: {case}/{status:?} text contrast {r:.2} < {AA_TEXT} \
             (fg {fg:?} vs actual background {bg:?})",
        )
    })
}

/// Same shape as [`text_contrast_failure`], against [`NON_TEXT_MIN`].
/// Skips borders with `width == 0.0` — an invisible border has no
/// contrast requirement because it renders nothing (`crumb_button_style`'s
/// border is `width: 0.0`; so is `tab_button_style`'s, which since
/// RFC-102 carries no colour either, the underline being its own element
/// with its own asserted colour — see
/// [`tab_indicator_meets_non_text_floor`]).
fn border_contrast_failure(
    context: &str,
    case: &str,
    border: iced::Border,
    bg: Color,
) -> Option<String> {
    if border.width <= 0.0 {
        return None;
    }
    let r = contrast_ratio(to_sn(border.color), to_sn(bg));
    (r < NON_TEXT_MIN).then(|| {
        format!(
            "{context}: {case} border contrast {r:.2} < {NON_TEXT_MIN} \
             (border {:?} vs actual background {bg:?})",
            border.color,
        )
    })
}

/// Panics with every collected failure, not just the first, if any exist.
fn assert_no_failures(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} failing combination(s):\n{}",
        failures.len(),
        failures.join("\n"),
    );
}

// ---------------------------------------------------------------------
// menu_button_style — F-13. `background: None` always; painted over the
// page background (menu.rs wraps its buttons in a bare, unstyled
// `container`, and nothing else in the render chain up to the header
// sets a background either) — `ep.background.base.color`.
// ---------------------------------------------------------------------
#[test]
fn menu_button_style_text_meets_aa() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        for status in contrast_checked_statuses() {
            let style = menu_button_style(&theme, status);
            failures.extend(text_contrast_failure(
                context,
                "menu_button_style",
                status,
                style.text_color,
                ep.background.base.color,
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// chrome_container_style_with_radius — F-15. Border painted over the
// page background, same reasoning as above (header.rs/footer.rs give it
// no background of their own either).
// ---------------------------------------------------------------------
#[test]
fn chrome_container_style_border_meets_non_text_floor() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        let style = chrome_container_style_with_radius(&theme, 0.0);
        failures.extend(border_contrast_failure(
            context,
            "chrome_container_style",
            style.border,
            ep.background.base.color,
        ));
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// sidebar_active_color — F-14, the *other* half. The RFC's stock-path
// figure (1.89:1) is not text-on-highlight — it is the highlight itself
// against the rail it sits on ("active highlight … against the rail"),
// a non-text distinctness check: is the active state visible *at all*
// against its own surroundings, independent of whatever icon sits on
// top of it. `sidebar.rs`'s outer container has no explicit style
// either, so the rail's own background is the same page background as
// everywhere else in this suite.
// ---------------------------------------------------------------------
#[test]
fn sidebar_active_highlight_meets_non_text_floor_against_rail() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        let highlight = sidebar_active_color(&theme);
        let rail_bg = ep.background.base.color;
        let r = contrast_ratio(to_sn(highlight), to_sn(rail_bg));
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: sidebar_active_color non-text contrast {r:.2} < {NON_TEXT_MIN} \
                 (highlight {highlight:?} vs rail background {rail_bg:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// sidebar_button_style — F-14. Background varies: explicit
// `sidebar_active_color` when active, `background.weak.color` when
// hovered-and-inactive, otherwise `None` (page background).
// ---------------------------------------------------------------------
#[test]
fn sidebar_button_style_text_meets_aa() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        for is_active in [true, false] {
            for status in contrast_checked_statuses() {
                let style = sidebar_button_style(&theme, status, is_active, 6.0);
                let actual_bg = if is_active {
                    sidebar_active_color(&theme)
                } else if status == button::Status::Hovered {
                    ep.background.weak.color
                } else {
                    ep.background.base.color
                };
                failures.extend(text_contrast_failure(
                    context,
                    &format!("sidebar_button_style(is_active={is_active})"),
                    status,
                    style.text_color,
                    actual_bg,
                ));
            }
        }
    }
    assert_no_failures(failures);
}

/// The solid colour a container style paints, or a panic naming the
/// style that stopped painting one.
fn background_color(style: &iced::widget::container::Style, what: &str) -> Color {
    match style.background {
        Some(iced::Background::Color(color)) => color,
        other => panic!("{what}: expected a solid background colour, found {other:?}"),
    }
}

// ---------------------------------------------------------------------
// The tab bar's bottom rule — RFC-085 F-15's guarantee, moved.
//
// This was `tab_bar_container_style_border_meets_non_text_floor`: the
// same assertion against the bar container's 1 px border. RFC-102
// replaced that border with a rule element, because iced 0.14 borders
// are all-sided and the bar was never meant to be outlined. The colour
// is unchanged (`background.base.text`), so F-15's guarantee moves here
// rather than disappearing — and the container is asserted to paint no
// border at all in `crate::tab::tests`.
// ---------------------------------------------------------------------
#[test]
fn tab_bar_bottom_rule_meets_non_text_floor() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        let rule = background_color(&tab_bar_rule_style(&theme), "tab_bar_rule_style");
        let r = contrast_ratio(to_sn(rule), to_sn(ep.background.base.color));
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: tab bar bottom rule non-text contrast {r:.2} < {NON_TEXT_MIN} \
                 (rule {rule:?} vs page background {:?})",
                ep.background.base.color,
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// Sidebar tooltip body (RFC-102 R-4) — the tooltip used to be bare text
// over the page, so its contrast was whatever the application rendered
// underneath. Now it has a body, and the pairing that matters is the
// text against **that body**, not against the page.
// ---------------------------------------------------------------------
#[test]
fn unstyled_tooltip_body_text_meets_aa_and_its_border_meets_non_text_floor() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        let style = unstyled_tooltip_body_style(&theme);
        let body = background_color(&style, "unstyled_tooltip_body_style");
        let text = style
            .text_color
            .expect("the tooltip body sets its own text colour");

        let r = contrast_ratio(to_sn(text), to_sn(body));
        if r < AA_TEXT {
            failures.push(format!(
                "{context}: unstyled tooltip text contrast {r:.2} < {AA_TEXT} \
                 (text {text:?} vs its own body {body:?})",
            ));
        }
        // The body must also be visible as an object against the page it
        // floats over, or it cannot separate its text from that page.
        failures.extend(border_contrast_failure(
            context,
            "unstyled tooltip body",
            style.border,
            ep.background.base.color,
        ));
    }
    assert_no_failures(failures);
}

/// The styled body is `card_raised(tokens)` (RFC-102 Q-2 (a)), which
/// takes a `Tokens` bundle rather than a `Theme`, so it has no
/// counterpart in the two stock contexts: a styled sidebar cannot be
/// built without tokens. The four bundles below are exactly the ones the
/// four `design` entries of [`theme_contexts`] are derived from, so
/// between the two tests every context a tooltip can render in is
/// covered.
#[test]
fn styled_tooltip_body_text_meets_aa_and_its_border_meets_non_text_floor() {
    let mut failures = Vec::new();
    for (name, tokens) in [
        ("design light", Tokens::light()),
        ("design dark", Tokens::dark()),
        ("design high_contrast_light", Tokens::high_contrast_light()),
        ("design high_contrast_dark", Tokens::high_contrast_dark()),
    ] {
        let style = snora_style::container::card_raised(&tokens);
        let body = background_color(&style, "card_raised");
        let text = style
            .text_color
            .expect("card_raised sets its own text colour");
        let page = snora_style::color::to_iced_color(tokens.palette.background);

        let r = contrast_ratio(to_sn(text), to_sn(body));
        if r < AA_TEXT {
            failures.push(format!(
                "{name}: styled tooltip text contrast {r:.2} < {AA_TEXT} \
                 (text {text:?} vs its own body {body:?})",
            ));
        }
        failures.extend(border_contrast_failure(
            name,
            "styled tooltip body",
            style.border,
            page,
        ));
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// tab indicator (RFC-102 R-3) — the active tab's underline is the state
// indicator (WCAG 1.4.11, "visual information required to identify
// states"), so it carries the non-text floor against the page it is
// drawn on. Before RFC-102 the underline was a `Shadow` on the tab
// button, which this suite skipped as decorative; it is now its own
// element with its own style function, and asserted here.
// ---------------------------------------------------------------------

/// The colour the active tab's indicator is drawn in.
///
/// RFC-102 moved this from `tab_button_style`'s shadow to
/// [`tab_indicator_style`]'s background. Asserted against today's
/// shadow colour before that change, it failed on stock Dark at 2.99 —
/// which is the defect the RFC was raised for.
fn indicator_color(theme: &Theme) -> Color {
    background_color(&tab_indicator_style(theme), "tab_indicator_style")
}

#[test]
fn tab_indicator_meets_non_text_floor() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        let indicator = indicator_color(&theme);
        let r = contrast_ratio(to_sn(indicator), to_sn(ep.background.base.color));
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: tab indicator non-text contrast {r:.2} < {NON_TEXT_MIN} \
                 (indicator {indicator:?} vs page background {:?}) — the active tab's \
                 underline is its state indicator (WCAG 1.4.11)",
                ep.background.base.color,
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// tab_button_style — not one of F-13/F-14/F-15, found by this suite's
// own derived coverage rather than named in the audit. Background:
// `None` (page background) when active or inactive-not-hovered,
// `background.weak.color` when inactive-and-hovered.
//
// This is the label's test. The active tab's *indicator* is no longer
// part of this style at all: RFC-102 made it an element, and
// `tab_indicator_meets_non_text_floor` above asserts its colour. The
// comment that used to stand here said the underline was "drawn via
// `shadow` instead — skipped by `assert_border_contrast`", which is how
// an indicator sat 0.01 under the non-text floor on stock Dark through
// every release since 0.10.0 with a full contrast suite in place.
// ---------------------------------------------------------------------
#[test]
fn tab_button_style_text_meets_aa() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        for is_active in [true, false] {
            for status in contrast_checked_statuses() {
                let style = tab_button_style(&theme, status, is_active);
                let actual_bg = if !is_active && status == button::Status::Hovered {
                    ep.background.weak.color
                } else {
                    ep.background.base.color
                };
                let case = format!("tab_button_style(is_active={is_active})");
                failures.extend(text_contrast_failure(
                    context,
                    &case,
                    status,
                    style.text_color,
                    actual_bg,
                ));
                failures.extend(border_contrast_failure(
                    context,
                    &case,
                    style.border,
                    actual_bg,
                ));
            }
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// crumb_button_style — not one of F-13/F-14/F-15 either, same as
// tab_button_style above. Background: `background.weak.color` when
// hovered or pressed, otherwise `None` (page background). Border is
// always `TRANSPARENT`/`width: 0.0` — skipped.
// ---------------------------------------------------------------------
#[test]
fn crumb_button_style_text_meets_aa() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        for status in contrast_checked_statuses() {
            let style = crumb_button_style(&theme, status, 4.0);
            let actual_bg = match status {
                button::Status::Hovered | button::Status::Pressed => ep.background.weak.color,
                _ => ep.background.base.color,
            };
            failures.extend(text_contrast_failure(
                context,
                "crumb_button_style",
                status,
                style.text_color,
                actual_bg,
            ));
        }
    }
    assert_no_failures(failures);
}
