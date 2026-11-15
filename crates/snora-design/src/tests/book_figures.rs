//! **The figures the book prints, asserted** (the 0.63 audit's ruling for request A: a number the book prints is
//! asserted, or it is not printed). Each test names the page it holds to. They state the page's own numbers as
//! literals, to the precision the page prints, so a palette edit that moves a printed figure fails here and points at
//! the page: before this, the suites held floors (7.0, 4.5, 3.0) and a regression from 17:1 to 7.5:1 passed.
//!
//! A failure means either the palette changed (then the page must say so) or the page was wrong (then it is a finding).

use super::*;
use crate::contrast::{composite_over, contrast_ratio};
use crate::surfaces::modal_dim;
use crate::{Color, Palette, Tokens};

fn preset(name: &str) -> Tokens {
    Tokens::presets()
        .into_iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("no preset `{name}`"))
        .1
}

fn hex(c: Color) -> String {
    let b = |v: f32| (v * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
}

fn near(what: &str, got: f32, want: f32, tol: f32) {
    assert!(
        (got - want).abs() <= tol,
        "{what}: {got:.4}, the page prints {want} (tolerance {tol})"
    );
}

/// The lowest contrast of `fg` over the three neutral surfaces.
fn lowest(fg: Color, p: &Palette) -> f32 {
    [p.background, p.surface, p.surface_raised]
        .into_iter()
        .map(|s| contrast_ratio(fg, s))
        .fold(f32::INFINITY, f32::min)
}

fn hue_shift(now: Color, before: Color) -> f32 {
    let (a, b) = (
        oklch_hue(now).expect("a chromatic role"),
        oklch_hue(before).expect("a chromatic role"),
    );
    let d = (a - b).abs();
    d.min(360.0 - d)
}

fn role(p: &Palette, name: &str) -> Color {
    match name {
        "text_primary" => p.text_primary,
        "text_secondary" => p.text_secondary,
        "text_muted" => p.text_muted,
        "accent" => p.accent,
        "info" => p.info,
        "success" => p.success,
        "warning" => p.warning,
        "danger" => p.danger,
        other => panic!("no role `{other}`"),
    }
}

/// The lowest declared text pair that involves `role` (what the page's last column says).
fn lowest_declared_text_pair(p: &Palette, role_name: &str) -> f32 {
    let labels: &[&str] = match role_name {
        "text_muted" => &["text_muted"],
        "accent" => &["accent (as text)", "accent_text"],
        "info" => &["info_text"],
        "success" => &["success_text"],
        "warning" => &["warning_text"],
        "danger" => &["danger_text"],
        other => panic!("no pair for `{other}`"),
    };
    let mut min = f32::INFINITY;
    for u in p.usages() {
        if u.class != Some(ThresholdClass::Text) || !labels.contains(&u.label) {
            continue;
        }
        for (_, surface) in u.surfaces {
            min = min.min(contrast_ratio(u.fg, surface));
        }
    }
    assert!(min.is_finite(), "no declared text pair for {role_name}");
    min
}

// ---------------------------------------------------------------------
// high-contrast.md
// ---------------------------------------------------------------------

/// `high-contrast.md`, "What the presets guarantee": the pair table, to the whole number the page prints.
#[test]
fn high_contrast_md_pair_table() {
    // (pair, light HC, dark HC)
    let rows: [(&str, f32, f32); 5] = [
        ("text_primary on background", 21.0, 21.0),
        ("text_secondary on background", 17.0, 17.0),
        ("accent_text on accent", 10.0, 12.0),
        ("danger_text on danger", 9.0, 10.0),
        ("focus on background", 10.0, 21.0),
    ];
    for (i, name) in ["high_contrast_light", "high_contrast_dark"]
        .into_iter()
        .enumerate()
    {
        let p = preset(name).palette;
        let got = [
            contrast_ratio(p.text_primary, p.background),
            contrast_ratio(p.text_secondary, p.background),
            contrast_ratio(p.accent_text, p.accent),
            contrast_ratio(p.danger_text, p.danger),
            contrast_ratio(p.focus, p.background),
        ];
        for ((pair, light, dark), got) in rows.iter().zip(got) {
            let want = if i == 0 { *light } else { *dark };
            // The page prints whole numbers: the value must round to it.
            near(&format!("{name}: {pair}"), got, want, 0.5);
        }
    }
}

/// `high-contrast.md` and `aaa-presets.md`: "the `border` role is `#000000` (light HC) or `#FFFFFF` (dark HC): 21:1
/// against the page and `surface`", and 19.80:1 against `high_contrast_dark`'s raised surface.
#[test]
fn high_contrast_md_border_role() {
    let l = preset("high_contrast_light").palette;
    let d = preset("high_contrast_dark").palette;
    assert_eq!(hex(l.border), "#000000");
    assert_eq!(hex(d.border), "#ffffff");
    for (name, p) in [("high_contrast_light", &l), ("high_contrast_dark", &d)] {
        near(
            &format!("{name}: border on background"),
            contrast_ratio(p.border, p.background),
            21.0,
            0.01,
        );
        near(
            &format!("{name}: border on surface"),
            contrast_ratio(p.border, p.surface),
            21.0,
            0.01,
        );
    }
    near(
        "high_contrast_light: border on surface_raised",
        contrast_ratio(l.border, l.surface_raised),
        21.0,
        0.01,
    );
    near(
        "high_contrast_dark: border on surface_raised",
        contrast_ratio(d.border, d.surface_raised),
        19.80,
        0.005,
    );
    // And the page's other sentence: only `high_contrast_dark` has a raised surface that is not the background.
    assert_eq!(hex(d.surface_raised), "#0a0a0a");
    assert_eq!(l.surface_raised, l.background);
    assert_eq!(d.surface, d.background);
}

// ---------------------------------------------------------------------
// aaa-presets.md
// ---------------------------------------------------------------------

/// `aaa-presets.md`, the `light_aaa` and `dark_aaa` tables: parent and AAA hex, hue shift to the printed 0.1 degree,
/// and the lowest declared text pair, 7.05 (the page prints 2 decimals). Then the two sentences under the tables.
#[test]
fn aaa_presets_md_tables() {
    // (role, parent hex, aaa hex, hue shift in degrees)
    #[rustfmt::skip]
    let light: [(&str, &str, &str, f32); 6] = [
        ("text_muted", "#6a717e", "#4d5461", 0.0),
        ("accent",     "#1d4ed8", "#1544ce", 0.0),
        ("info",       "#1d4ed8", "#1544ce", 0.0),
        ("success",    "#15803d", "#006725", 2.3),
        ("warning",    "#9a5b00", "#874900", 5.1),
        ("danger",     "#b3261e", "#ad1f18", 0.0),
    ];
    #[rustfmt::skip]
    let dark: [(&str, &str, &str, f32); 4] = [
        ("text_muted", "#828b97", "#a6afbc", 0.0),
        ("accent",     "#5b9bff", "#70b2ff", 5.4),
        ("info",       "#5b9bff", "#70b2ff", 5.4),
        ("danger",     "#ff6b6b", "#ff7372", 0.3),
    ];
    for (aaa, parent, rows) in [
        ("light_aaa", "light", light.as_slice()),
        ("dark_aaa", "dark", dark.as_slice()),
    ] {
        let (a, b) = (preset(aaa).palette, preset(parent).palette);
        for (name, parent_hex, aaa_hex, shift) in rows {
            assert_eq!(hex(role(&b, name)), *parent_hex, "{parent}: {name}");
            assert_eq!(hex(role(&a, name)), *aaa_hex, "{aaa}: {name}");
            near(
                &format!("{aaa}: {name} hue shift"),
                hue_shift(role(&a, name), role(&b, name)),
                *shift,
                0.06,
            );
            // The page's row is "accent, info": its 7.05 is the lowest of the two roles' pairs, and `info`'s own
            // pair is higher (`info_text` on `info`), so `info` is only held to the row's floor.
            let pair = lowest_declared_text_pair(&a, name);
            if *name == "info" {
                assert!(
                    pair >= 7.049,
                    "{aaa}: info lowest declared text pair {pair}"
                );
            } else {
                near(
                    &format!("{aaa}: {name} lowest declared text pair"),
                    pair,
                    7.05,
                    0.006,
                );
            }
        }
    }
    // "`text_primary` and `text_secondary` are unchanged, at 16.44 and 8.12" (light_aaa), and "`text_primary`,
    // `text_secondary`, `success` and `warning` are unchanged" (dark_aaa).
    let (la, l) = (preset("light_aaa").palette, preset("light").palette);
    assert_eq!(
        (la.text_primary, la.text_secondary),
        (l.text_primary, l.text_secondary)
    );
    near(
        "light_aaa: text_primary",
        lowest(la.text_primary, &la),
        16.44,
        0.006,
    );
    near(
        "light_aaa: text_secondary",
        lowest(la.text_secondary, &la),
        8.12,
        0.006,
    );
    let (da, d) = (preset("dark_aaa").palette, preset("dark").palette);
    assert_eq!(
        (da.text_primary, da.text_secondary, da.success, da.warning),
        (d.text_primary, d.text_secondary, d.success, d.warning)
    );
    // "7.05 against 8.12 in `light_aaa`, and 7.05 against 7.62 in `dark_aaa`."
    near("light_aaa: muted", lowest(la.text_muted, &la), 7.05, 0.006);
    near("dark_aaa: muted", lowest(da.text_muted, &da), 7.05, 0.006);
    near(
        "dark_aaa: secondary",
        lowest(da.text_secondary, &da),
        7.62,
        0.006,
    );
    // "Each keeps its parent's background, surfaces and border."
    for (aaa, parent) in [("light_aaa", "light"), ("dark_aaa", "dark")] {
        let (a, b) = (preset(aaa).palette, preset(parent).palette);
        assert_eq!(
            (a.background, a.surface, a.surface_raised, a.border),
            (b.background, b.surface, b.surface_raised, b.border),
            "{aaa} against {parent}"
        );
    }
}

// ---------------------------------------------------------------------
// engine-surfaces.md
// ---------------------------------------------------------------------

/// `engine-surfaces.md`, the dialog card: "Measured border-vs-background contrast" (3.38 / 3.81 / 21.0 / 21.0), the
/// border against the card's own fill (3.38 / 3.17 / 21.0 / 19.80), and "Measured contrast of the composited dim
/// against the plain background" (3.2424 / 4.3798 / 3.2424 / 4.2529). The AAA children keep their parents' values.
#[test]
fn engine_surfaces_md_border_and_dim_tables() {
    #[rustfmt::skip]
    let rows: [(&str, f32, f32, f32, f32); 4] = [
        // preset, border|background, border|own fill, tolerance of those two, dim|background
        ("light",               3.38, 3.38, 0.005, 3.2424),
        ("dark",                3.81, 3.17, 0.005, 4.3798),
        ("high_contrast_light", 21.0, 21.0, 0.05,  3.2424),
        ("high_contrast_dark",  21.0, 19.80, 0.05, 4.2529),
    ];
    for (name, border_bg, border_fill, tol, dim) in rows {
        let p = preset(name).palette;
        near(
            &format!("{name}: border on background"),
            contrast_ratio(p.border, p.background),
            border_bg,
            tol,
        );
        near(
            &format!("{name}: border on its own fill (surface_raised)"),
            contrast_ratio(p.border, p.surface_raised),
            border_fill,
            tol,
        );
        let t = preset(name);
        let over = composite_over(modal_dim(&t), p.background);
        near(
            &format!("{name}: dim on the plain background"),
            contrast_ratio(over, p.background),
            dim,
            0.0006,
        );
    }
    for (aaa, parent) in [("light_aaa", "light"), ("dark_aaa", "dark")] {
        let (a, b) = (preset(aaa), preset(parent));
        assert_eq!(modal_dim(&a), modal_dim(&b), "{aaa} dims as {parent}");
    }
}

/// `engine-surfaces.md`, "What actually separates the card from the dim": the sweep over every grey the content could
/// be (`border | dim` minimum, where it is, and maximum; `dim | fill` range). The table states them to two decimals.
#[test]
fn engine_surfaces_md_card_against_the_dim_sweep() {
    #[rustfmt::skip]
    let rows: [(&str, f32, f32, f32, f32, f32); 4] = [
        // preset, border|dim min, at content, border|dim max, dim|fill min, dim|fill max
        ("light",               1.00, 0.98, 6.21, 3.24, 21.00),
        ("dark",                1.00, 0.00, 4.93, 3.16, 15.61),
        ("high_contrast_light", 1.00, 0.00, 6.48, 3.24, 21.00),
        ("high_contrast_dark",  1.00, 1.00, 4.94, 4.01, 19.80),
    ];
    for (name, bd_min, at, bd_max, df_min, df_max) in rows {
        let t = preset(name);
        let (dim, border, fill) = (modal_dim(&t), t.palette.border, t.palette.surface_raised);
        let (mut mn, mut mn_at, mut mx) = (f32::INFINITY, 0.0, 0.0_f32);
        let (mut fmn, mut fmx) = (f32::INFINITY, 0.0_f32);
        for step in 0..=DIM_SWEEP_STEPS {
            let c = step as f32 / DIM_SWEEP_STEPS as f32;
            let over = composite_over(dim, Color::rgb(c, c, c));
            let b = contrast_ratio(border, over);
            let f = contrast_ratio(fill, over);
            if b < mn {
                mn = b;
                mn_at = c;
            }
            mx = mx.max(b);
            fmn = fmn.min(f);
            fmx = fmx.max(f);
        }
        near(&format!("{name}: border|dim min"), mn, bd_min, 0.01);
        near(
            &format!("{name}: where the border|dim minimum is"),
            mn_at,
            at,
            0.02,
        );
        near(&format!("{name}: border|dim max"), mx, bd_max, 0.01);
        near(&format!("{name}: dim|fill min"), fmn, df_min, 0.01);
        near(&format!("{name}: dim|fill max"), fmx, df_max, 0.01);
    }
}

// ---------------------------------------------------------------------
// typography.md, chrome-geometry.md
// ---------------------------------------------------------------------

/// `typography.md`, the role table: size and line-height multiplier of the six roles, in every preset (they share one
/// scale). Changing `heading` to 22/1.3 fails here and points at the page (the audit: no test pinned a typography
/// value).
#[test]
fn typography_md_role_table() {
    #[rustfmt::skip]
    let want: [(&str, f32, f32); 6] = [
        ("body",       16.0, 1.4),
        ("body_small", 14.0, 1.35),
        ("label",      14.0, 1.2),
        ("title",      18.0, 1.3),
        ("heading",    24.0, 1.25),
        ("display",    32.0, 1.2),
    ];
    for (name, t) in Tokens::presets() {
        let ty = t.typography;
        let got = [
            ty.body,
            ty.body_small,
            ty.label,
            ty.title,
            ty.heading,
            ty.display,
        ];
        for ((role, size, line_height), got) in want.iter().zip(got) {
            assert_eq!(
                (got.size, got.line_height),
                (*size, *line_height),
                "{name}: {role}"
            );
        }
    }
    // "`title` ... identical to iced's default: no effect": 1.3 is what the page compares the others against.
    assert_eq!(Tokens::light().typography.title.line_height, 1.3);
}

/// `chrome-geometry.md`, "The mapping": "`Spacing::comfortable()` is `{xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32}`;
/// `Radius::default_roles()` is `{sm: 4, md: 6, lg: 10, pill: 999}`". Every preset uses them.
#[test]
fn chrome_geometry_md_spacing_and_radius_scales() {
    for (name, t) in Tokens::presets() {
        let s = t.spacing;
        assert_eq!(
            [s.xs, s.sm, s.md, s.lg, s.xl, s.xxl],
            [4.0, 8.0, 12.0, 16.0, 24.0, 32.0],
            "{name}: Spacing"
        );
        let r = t.radius;
        assert_eq!(
            [r.sm, r.md, r.lg, r.pill],
            [4.0, 6.0, 10.0, 999.0],
            "{name}: Radius"
        );
    }
}

// ---------------------------------------------------------------------
// stability.md, tokens.md: the covenant, read from the source
// ---------------------------------------------------------------------

/// The attribute lines directly above `item` (a line starting with it, like `pub struct Tokens`), through any doc
/// comments between them.
fn attributes_of(source: &str, item: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim_start().starts_with(item))
        .unwrap_or_else(|| panic!("`{item}` is not in the source"));
    let mut out = Vec::new();
    for l in lines[..at].iter().rev() {
        let t = l.trim();
        if t.starts_with("#[") {
            out.push(t.to_owned());
        } else if t.starts_with("///") || t.starts_with("//") || t.is_empty() && out.is_empty() {
            continue;
        } else {
            break;
        }
    }
    out
}

/// **The covenant, as far as a source can show it** (`stability.md`, `tokens.md`; the 0.63 audit named it a gap):
/// - `Tokens`, `Palette` and `TargetTokens` carry `#[non_exhaustive]` ("new fields can be added in minor releases");
/// - `Tokens`, `Palette` and `TextContrast` have no `Default` ("a palette starts from a preset": no level nobody chose);
/// - `Tokens` is not `Copy` (future fields may not be).
///
/// Like the presets-list test, it reads this crate's own source. **What it does not show** is the page's "additive-only
/// across releases": nothing compares public surfaces between releases, and this test says so by not pretending to.
#[test]
fn stability_md_the_covenant_is_in_the_source() {
    let tokens = include_str!("../tokens.rs");
    let palette = include_str!("../palette.rs");
    let targets = include_str!("../targets.rs");
    let contrast = include_str!("../contrast.rs");

    for (what, source, item) in [
        ("Tokens", tokens, "pub struct Tokens"),
        ("Palette", palette, "pub struct Palette"),
        ("TargetTokens", targets, "pub struct TargetTokens"),
    ] {
        let attrs = attributes_of(source, item);
        assert!(
            attrs.iter().any(|a| a == "#[non_exhaustive]"),
            "`{what}` lost #[non_exhaustive]: {attrs:?}"
        );
    }
    for (what, source, item) in [
        ("Tokens", tokens, "pub struct Tokens"),
        ("Palette", palette, "pub struct Palette"),
        ("TextContrast", contrast, "pub enum TextContrast"),
    ] {
        let attrs = attributes_of(source, item).join(" ");
        assert!(
            !attrs.contains("Default"),
            "`{what}` derives Default: {attrs}"
        );
        assert!(
            !source.contains(&format!("impl Default for {what}")),
            "`{what}` implements Default"
        );
    }
    let attrs = attributes_of(tokens, "pub struct Tokens").join(" ");
    assert!(!attrs.contains("Copy"), "`Tokens` became Copy: {attrs}");
    // And the types the page says are plain: constructible by literal, not `#[non_exhaustive]`.
    for (what, source, item) in [
        (
            "Spacing",
            include_str!("../spacing.rs"),
            "pub struct Spacing",
        ),
        ("Radius", include_str!("../radius.rs"), "pub struct Radius"),
        (
            "Typography",
            include_str!("../typography.rs"),
            "pub struct Typography",
        ),
        (
            "FocusTokens",
            include_str!("../focus.rs"),
            "pub struct FocusTokens",
        ),
    ] {
        let attrs = attributes_of(source, item);
        assert!(
            !attrs.iter().any(|a| a == "#[non_exhaustive]"),
            "`{what}` became #[non_exhaustive], which `tokens.md` says it is not"
        );
    }
}
