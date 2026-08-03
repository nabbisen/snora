//! Geometry regression and mapping tests for token-derived chrome
//! widgets (RFC-040).
//!
//! - **Unstyled geometry unchanged**: each unstyled builder's
//!   `::unstyled()` constructor still equals the literal inventory
//!   captured before this RFC — the regression test that protects
//!   RFC-037's gating invariant. Written against the geometry struct's
//!   fields, not rendered output, per the Handoff's explicit instruction.
//! - **Styled geometry is token-derived**: each styled variant's private
//!   `*_geometry` constructor equals the mapping table in this module's
//!   own documentation, for all four built-in presets.
//! - **Density plumbing**: since no built-in preset currently offers two
//!   distinct `Spacing`/`Radius` values (`Density::Compact` has no
//!   resolved scale — see the module documentation), this uses a pair of
//!   hand-mutated `Tokens` to prove the geometry constructors are a pure
//!   function of `tokens.spacing`/`tokens.radius`, with no widget-level
//!   branch that could silently ignore a future compact scale.

use super::*;
use snora_design::{Radius, Spacing, Tokens};

fn named_presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

// ---------------------------------------------------------------------------
// Unstyled geometry unchanged — the literal inventory, verified against
// source before this RFC (Handoff §4).
// ---------------------------------------------------------------------------

#[test]
fn header_unstyled_matches_literal_inventory() {
    let g = HeaderGeometry::unstyled();
    assert_eq!(g.gap, 12.0);
    assert_eq!(g.pad_x, 16.0);
    assert_eq!(g.pad_y, 8.0);
    assert_eq!(g.radius, 0.0);
    assert_eq!(g.menu_gap, 6.0);
}

#[test]
fn footer_unstyled_matches_literal_inventory() {
    let g = FooterGeometry::unstyled();
    assert_eq!(g.pad_x, 16.0);
    assert_eq!(g.pad_y, 6.0);
    assert_eq!(g.radius, 0.0);
}

#[test]
fn sidebar_unstyled_matches_literal_inventory() {
    let g = SideBarGeometry::unstyled();
    assert_eq!(g.gap, 16.0);
    assert_eq!(g.padding, 16.0);
    assert_eq!(g.button_radius, 6.0);
}

#[test]
fn tab_unstyled_matches_literal_inventory() {
    let g = TabGeometry::unstyled();
    assert_eq!(g.bar_gap, 2.0);
    assert_eq!(g.bar_pad_x, 12.0);
    assert_eq!(g.content_gap, 6.0);
    assert_eq!(g.tab_pad_x, 12.0);
    assert_eq!(g.tab_pad_y, 8.0);
    assert_eq!(g.bar_border_radius, 0.0);
}

#[test]
fn crumb_unstyled_matches_literal_inventory() {
    let g = CrumbGeometry::unstyled();
    assert_eq!(g.gap, 6.0);
    assert_eq!(g.row_pad_x, 12.0);
    assert_eq!(g.row_pad_y, 4.0);
    assert_eq!(g.btn_pad_x, 4.0);
    assert_eq!(g.btn_pad_y, 2.0);
    assert_eq!(g.btn_radius, 3.0);
}

#[test]
fn menu_unstyled_matches_literal_inventory() {
    let g = crate::menu::MenuGeometry::unstyled();
    assert_eq!(g.gap, 6.0);
}

// ---------------------------------------------------------------------------
// Styled geometry is token-derived — matches this module's mapping table,
// for all four built-in presets.
// ---------------------------------------------------------------------------

#[test]
fn header_geometry_matches_mapping_all_presets() {
    for (name, t) in named_presets() {
        let g = header_geometry(&t);
        assert_eq!(
            g.gap, t.spacing.md,
            "{name}: header gap should map to Spacing::md"
        );
        assert_eq!(
            g.pad_x, t.spacing.lg,
            "{name}: header pad_x should map to Spacing::lg"
        );
        assert_eq!(
            g.pad_y, t.spacing.sm,
            "{name}: header pad_y should map to Spacing::sm"
        );
        assert_eq!(
            g.radius, t.radius.sm,
            "{name}: header radius should map to Radius::sm"
        );
        assert_eq!(
            g.menu_gap, t.spacing.sm,
            "{name}: header menu_gap should map to Spacing::sm"
        );
    }
}

#[test]
fn footer_geometry_matches_mapping_all_presets() {
    for (name, t) in named_presets() {
        let g = footer_geometry(&t);
        assert_eq!(
            g.pad_x, t.spacing.lg,
            "{name}: footer pad_x should map to Spacing::lg"
        );
        assert_eq!(
            g.pad_y, t.spacing.sm,
            "{name}: footer pad_y should map to Spacing::sm"
        );
        assert_eq!(
            g.radius, t.radius.sm,
            "{name}: footer radius should map to Radius::sm"
        );
    }
}

#[test]
fn side_bar_geometry_matches_mapping_all_presets() {
    for (name, t) in named_presets() {
        let g = side_bar_geometry(&t);
        assert_eq!(
            g.gap, t.spacing.md,
            "{name}: sidebar gap should map to Spacing::md"
        );
        assert_eq!(
            g.padding, t.spacing.lg,
            "{name}: sidebar padding should map to Spacing::lg"
        );
        assert_eq!(
            g.button_radius, t.radius.md,
            "{name}: sidebar button_radius should map to Radius::md"
        );
    }
}

#[test]
fn tab_geometry_matches_mapping_all_presets() {
    for (name, t) in named_presets() {
        let g = tab_geometry(&t);
        assert_eq!(
            g.bar_gap, t.spacing.xs,
            "{name}: tab bar_gap should map to Spacing::xs"
        );
        assert_eq!(
            g.bar_pad_x, t.spacing.md,
            "{name}: tab bar_pad_x should map to Spacing::md"
        );
        assert_eq!(
            g.content_gap, t.spacing.sm,
            "{name}: tab content_gap should map to Spacing::sm"
        );
        assert_eq!(
            g.tab_pad_x, t.spacing.md,
            "{name}: tab tab_pad_x should map to Spacing::md"
        );
        assert_eq!(
            g.tab_pad_y, t.spacing.sm,
            "{name}: tab tab_pad_y should map to Spacing::sm"
        );
        assert_eq!(
            g.bar_border_radius, t.radius.sm,
            "{name}: tab bar_border_radius should map to Radius::sm"
        );
    }
}

#[test]
fn breadcrumb_geometry_matches_mapping_all_presets() {
    for (name, t) in named_presets() {
        let g = breadcrumb_geometry(&t);
        assert_eq!(
            g.gap, t.spacing.sm,
            "{name}: crumb gap should map to Spacing::sm"
        );
        assert_eq!(
            g.row_pad_x, t.spacing.md,
            "{name}: crumb row_pad_x should map to Spacing::md"
        );
        assert_eq!(
            g.row_pad_y, t.spacing.xs,
            "{name}: crumb row_pad_y should map to Spacing::xs"
        );
        assert_eq!(
            g.btn_pad_x, t.spacing.xs,
            "{name}: crumb btn_pad_x should map to Spacing::xs"
        );
        assert_eq!(
            g.btn_pad_y, t.spacing.xs,
            "{name}: crumb btn_pad_y should map to Spacing::xs"
        );
        assert_eq!(
            g.btn_radius, t.radius.sm,
            "{name}: crumb btn_radius should map to Radius::sm"
        );
    }
}

// ---------------------------------------------------------------------------
// Chrome-radius rhythm: header, footer, and the tab bar share one radius
// role (Radius::sm), the fix for RFC-040's stated "flat and dated" defect.
// ---------------------------------------------------------------------------

#[test]
fn chrome_radius_is_shared_across_header_footer_and_tab_bar() {
    for (name, t) in named_presets() {
        let header_radius = header_geometry(&t).radius;
        let footer_radius = footer_geometry(&t).radius;
        let tab_radius = tab_geometry(&t).bar_border_radius;
        assert_eq!(
            (header_radius, footer_radius, tab_radius),
            (t.radius.sm, t.radius.sm, t.radius.sm),
            "{name}: header/footer/tab-bar radius must share one role (Radius::sm)"
        );
        assert_ne!(
            header_radius, 0.0,
            "{name}: chrome radius must no longer be the flat 0.0 literal"
        );
    }
}

// ---------------------------------------------------------------------------
// Density plumbing (RFC-040 Q-1): geometry constructors are a pure
// function of tokens.spacing/tokens.radius, not a tokens.density branch.
// ---------------------------------------------------------------------------

#[test]
fn geometry_follows_hand_mutated_spacing_and_radius() {
    // No built-in preset offers two distinct Spacing/Radius values
    // (Density::Compact has no resolved scale), so this constructs two
    // hand-mutated Tokens differing only in spacing/radius, to prove the
    // geometry constructors track whatever the token bundle says rather
    // than a hardcoded assumption.
    let mut roomy = Tokens::light();
    roomy.spacing = Spacing::comfortable();
    roomy.radius = Radius::default_roles();

    let mut tight = Tokens::light();
    tight.spacing = Spacing {
        xs: 2.0,
        sm: 4.0,
        md: 6.0,
        lg: 8.0,
        xl: 12.0,
        xxl: 16.0,
    };
    tight.radius = Radius {
        sm: 2.0,
        md: 3.0,
        lg: 5.0,
        pill: 999.0,
    };

    let roomy_header = header_geometry(&roomy);
    let tight_header = header_geometry(&tight);
    assert_ne!(
        roomy_header, tight_header,
        "header geometry must differ when the underlying tokens' spacing/radius differ"
    );

    let roomy_tab = tab_geometry(&roomy);
    let tight_tab = tab_geometry(&tight);
    assert_ne!(
        roomy_tab, tight_tab,
        "tab geometry must differ when the underlying tokens' spacing/radius differ"
    );

    // And the exact values still follow the same mapping table as the
    // built-in-preset tests above — no hidden branch on tokens.density.
    assert_eq!(tight_header.gap, tight.spacing.md);
    assert_eq!(tight_header.radius, tight.radius.sm);
}
