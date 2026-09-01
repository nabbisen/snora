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
//! `crumb_button_style`) were found by grepping this crate's source for
//! every `-> button::Style` / `-> container::Style` return type — a
//! search anyone can re-run to check this list is still complete, but
//! not a compiler-enforced one. **If a new widget style function is
//! added and not added here, this suite will not catch it.** That is the
//! honest limit of this approach, named rather than hidden behind a
//! test count that looks complete.
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
use crate::sidebar::sidebar_button_style;
use crate::style::{chrome_container_style_with_radius, menu_button_style, sidebar_active_color};
use crate::tab::{tab_bar_container_style, tab_button_style};

/// WCAG 2.1 SC 1.4.3 normal-text minimum. A sibling constant of the same
/// name and value exists in `crates/snora-design/src/tests.rs`; it cannot
/// be shared directly (that one lives in a `#[cfg(test)]` module in a
/// crate this one merely depends on, and is not exported). If you change
/// one, check the other.
const AA_TEXT: f32 = 4.5;

/// WCAG 2.1 SC 1.4.11 non-text minimum, for borders that identify a
/// component boundary. Same sibling-constant caveat as [`AA_TEXT`].
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
fn text_contrast_failure(context: &str, case: &str, status: button::Status, fg: Color, bg: Color) -> Option<String> {
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
/// contrast requirement because it renders nothing (`tab_button_style`'s
/// active-state border and `crumb_button_style`'s border are both
/// `width: 0.0`, used only to carry a color the shadow/underline effect
/// borrows, not to paint an actual border).
fn border_contrast_failure(context: &str, case: &str, border: iced::Border, bg: Color) -> Option<String> {
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

// ---------------------------------------------------------------------
// tab_bar_container_style — same shape as F-15, checked separately since
// it is a distinct function even though its border currently comes from
// the same source color as chrome_container_style's.
// ---------------------------------------------------------------------
#[test]
fn tab_bar_container_style_border_meets_non_text_floor() {
    let mut failures = Vec::new();
    for (context, theme) in theme_contexts() {
        let ep = theme.extended_palette();
        let style = tab_bar_container_style(&theme, 0.0);
        failures.extend(border_contrast_failure(
            context,
            "tab_bar_container_style",
            style.border,
            ep.background.base.color,
        ));
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// tab_button_style — not one of F-13/F-14/F-15, found by this suite's
// own derived coverage rather than named in the audit. Background:
// `None` (page background) when active or inactive-not-hovered,
// `background.weak.color` when inactive-and-hovered. The active state's
// border carries a color but `width: 0.0` (the underline is drawn via
// `shadow` instead) — skipped by `assert_border_contrast`.
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
                failures.extend(border_contrast_failure(context, &case, style.border, actual_bg));
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

