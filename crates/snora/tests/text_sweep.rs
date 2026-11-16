//! RFC-131: **every text a snora widget draws on the design path holds its palette's level** (4.5, or
//! 7), whatever its size, at rest and in every state it has, on every neutral surface it can sit on,
//! in every preset, **in both directions**. (A glyph that is not a word is held to 3:1 by
//! `nontext_sweep.rs`; here it only has to be listed as one.)
//!
//! `text_floor.rs` (RFC-128 R-2) draws the buttons, chips, links and a notice's action. This is the
//! sweep it promised and did not complete: tab labels, crumbs, sidebar names, menu items, tooltips, a
//! notice's title and body, toasts, the header's title, the footer and progress labels. Its inventory is made by drawing (`common/sweep.rs` says how), and it is checked for
//! completeness by three tripwires, each with a plant that must fail it:
//!
//! 1. **a text run in a frame that the scene's table does not name;**
//! 2. **a snora file that draws text with a renderer primitive and does not announce it** to
//!    `Operation::text`, the place the inventory is read from;
//! 3. **a design widget exported from `snora::design` with no scene in the sweep.**
//!
//! (A fourth, for non-text, is RFC-135's: a `fill_quad` with no finder.)
//!
//! **The unstyled path is out of scope**, as for RFC-129: its colours are the application's iced theme.
//!
//! # The cost
//!
//! The sweep draws about 4,000 frames at 2x. On tiny-skia it takes about one minute (Unit 0 measured
//! the 4x frame at 127 s and the 2x frame at 42 s for one direction). The rendered jobs run it on both
//! renderers; that is what they already do for every test in this directory.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use common::removal::Rig;
use common::sweep::*;
use iced::widget::{row, text};
use iced::{Color, Element, Rectangle};
use snora::LayoutDirection;
use snora::design::{Tokens, Tone, chip, notice::Notice, widget as dw};

/// The sweep, drawn once for the tests that read it.
fn shared() -> &'static Sweep {
    static SWEEP: std::sync::OnceLock<Sweep> = std::sync::OnceLock::new();
    SWEEP.get_or_init(sweep)
}

/// Cells under their floor, worded for a failure message. A word is held to the palette's level, a
/// glyph that is not a word to 3:1; the application's own content is not snora's to hold.
fn under(cells: &[Cell]) -> Vec<String> {
    cells
        .iter()
        .filter_map(|c| {
            let (floor, what) = match c.kind? {
                Kind::Word => (level(c.preset), "word".to_owned()),
                // A glyph that is not a word is held to 3:1 in `nontext_sweep.rs` (RFC-135); here it only has
                // to be *listed* as one, which is what keeps a label from being waved through as a glyph.
                Kind::Glyph(_) => return None,
                Kind::App => return None,
            };
            (c.ratio + TOLERANCE < floor).then(|| {
                format!(
                    "{} / {} / {} / {:?} / {}: {:?} {what} {:.2}:1 ({} on {}), floor {floor}",
                    c.preset,
                    c.scene,
                    c.surface,
                    c.direction,
                    c.state,
                    c.text,
                    c.ratio,
                    hex(c.ink),
                    hex(c.bg)
                )
            })
        })
        .collect()
}

/// **The classification is written, and it cannot be used to wave a label through.** A glyph that is
/// not a word is one character (an icon, a mark, a separator); a word holds a letter or a digit. So a
/// label cannot be listed as a glyph to be held to 3:1 instead of 4.5 or 7. (A one-letter label is the
/// case this cannot see; the table is reviewed.)
#[test]
fn a_glyph_is_one_character_and_a_word_is_not_a_bare_mark() {
    let mut bad = Vec::new();
    for sc in scenes() {
        for (s, kind) in sc.runs {
            match kind {
                Kind::Glyph(why) if s.chars().count() != 1 => bad.push(format!(
                    "{}: {s:?} is listed as a glyph ({why}) and is not one character",
                    sc.name
                )),
                Kind::Word if !s.chars().any(char::is_alphanumeric) => bad.push(format!(
                    "{}: {s:?} is listed as a word and holds no letter or digit",
                    sc.name
                )),
                _ => {}
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The table names exactly the presets (a seventh cannot go unlisted, a renamed one cannot linger).
#[test]
fn the_sweeps_level_table_names_every_preset_and_only_them() {
    let presets: Vec<&str> = Tokens::presets().into_iter().map(|(n, _)| n).collect();
    let table: Vec<&str> = LEVELS.iter().map(|(n, _)| *n).collect();
    assert_eq!(presets, table);
}

/// **The sweep.** Every word at its level, every glyph at 3:1, every run named by its scene's table.
/// Before RFC-131 an inactive tab on `light_aaa` read 6.60:1 on the page and `surface_raised` and 6.10:1
/// on `surface`, against 7: 24 cells.
#[test]
fn every_word_holds_its_presets_level_and_every_glyph_3_to_1_in_both_directions() {
    let sweep = shared();
    assert!(
        sweep.cells.len() > 3000,
        "premise: the sweep read {} cells",
        sweep.cells.len()
    );
    assert!(
        sweep.unlisted.is_empty(),
        "text runs no scene's table names (tripwire 1: add the row, with its kind):\n  {}",
        sweep.unlisted.join("\n  ")
    );
    let failures = under(&sweep.cells);
    // The lowest word in each scene, for the log (`--nocapture`).
    let mut lowest: std::collections::BTreeMap<&str, (f32, String)> = Default::default();
    for c in sweep.cells.iter().filter(|c| c.kind == Some(Kind::Word)) {
        let margin = c.ratio / level(c.preset);
        let e = lowest.entry(c.scene).or_insert((f32::MAX, String::new()));
        if margin < e.0 {
            *e = (
                margin,
                format!("{:?} {:.2}:1 on {}, {}", c.text, c.ratio, c.preset, c.state),
            );
        }
    }
    for (scene, (_, what)) in &lowest {
        println!("lowest word | {scene:<30} | {what}");
    }
    println!(
        "{} scenes, {} cells, {} under",
        scenes().len(),
        sweep.cells.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {} cells are under their floor:\n  {}",
        failures.len(),
        sweep.cells.len(),
        failures.join("\n  ")
    );
}

/// **Colour does not depend on direction** (RFC-131 asked for RTL frames; after RFC-132 the trees
/// differ). The same run, in the same preset, on the same surface, in the same state, reads the same
/// ratio under both.
#[test]
fn a_runs_colour_is_the_same_in_both_directions() {
    // Paired by (preset, scene, surface, state, control, string) and by occurrence: a scene can hold
    // the same string twice (the active tab's icon and an inactive tab's).
    type Key = (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        Option<usize>,
        String,
    );
    // The separator is the one run that differs by direction, on purpose: `›` flips to `‹` (RFC-132 R-4).
    let key = |c: &Cell| -> Key {
        let text = if c.text == "\u{2039}" {
            "\u{203a}".to_owned()
        } else {
            c.text.clone()
        };
        (c.preset, c.scene, c.surface, c.state, c.target, text)
    };
    let mut by_key: std::collections::HashMap<Key, (Vec<f32>, Vec<f32>)> = Default::default();
    for c in &shared().cells {
        let e = by_key.entry(key(c)).or_default();
        match c.direction {
            LayoutDirection::Ltr => e.0.push(c.ratio),
            LayoutDirection::Rtl => e.1.push(c.ratio),
        }
    }
    let mut compared = 0;
    let mut differing = Vec::new();
    for (k, (ltr, rtl)) in &by_key {
        if rtl.is_empty() {
            continue; // a widget with no direction is drawn once
        }
        assert_eq!(
            ltr.len(),
            rtl.len(),
            "{k:?}: the same runs in both directions"
        );
        for (a, b) in ltr.iter().zip(rtl) {
            compared += 1;
            if (a - b).abs() > 0.03 {
                differing.push(format!(
                    "{} / {} / {} / {} / {:?}: {:?} {a:.2} under LTR, {b:.2} under RTL",
                    k.0, k.1, k.2, k.3, k.4, k.5
                ));
            }
        }
    }
    assert!(compared > 1000, "premise: {compared} pairs compared");
    assert!(differing.is_empty(), "{}", differing.join("\n"));
}

// ---- a tooltip is an overlay `operate` does not report ------------------------------------------------------------------------

/// **A tooltip's text** (the rail's, the removable chip's and the notice's dismiss): found by what
/// changes when the control is hovered, read inside the body. Words, at the level.
#[test]
fn a_tooltips_text_holds_the_level() {
    type MakeT = Box<dyn Fn(&'static Tokens) -> Element<'static, Msg>>;
    let tips: Vec<(&str, usize, MakeT)> = vec![
        (
            "sidebar tooltip",
            0,
            Box::new(|t| {
                dw::app_side_bar_with(t, rail(3), LayoutDirection::Ltr, dw::SideBarLabels::Tooltip)
            }),
        ),
        (
            "removable chip tooltip",
            1,
            Box::new(|t| {
                chip::removable_with_tooltip(
                    t,
                    "Tag",
                    false,
                    Msg::Go,
                    Msg::Go,
                    "Remove tag",
                    snora::LayoutDirection::Ltr,
                )
            }),
        ),
        (
            "notice dismiss tooltip",
            0,
            Box::new(|t| {
                Notice::new(t, Tone::Info, "Body", snora::LayoutDirection::Ltr)
                    .dismiss(Msg::Go)
                    .dismiss_tooltip("Dismiss")
                    .render()
            }),
        ),
    ];
    let scale = SCALE as f32;
    let mut failures = Vec::new();
    let mut cells = 0;
    for (preset, tokens) in Tokens::presets() {
        let t = leak(tokens);
        let theme = design_theme(t);
        for (name, idx, make) in &tips {
            for surface in SURFACES {
                let fill = {
                    let c = surface_colour(t, surface);
                    Color {
                        r: c.r,
                        g: c.g,
                        b: c.b,
                        a: c.a,
                    }
                };
                let build = || -> Element<'static, Msg> {
                    iced::widget::container(make(t))
                        .padding(20)
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .style(move |_| iced::widget::container::Style {
                            background: Some(fill.into()),
                            ..Default::default()
                        })
                        .into()
                };
                let mut rig = Rig::new(build());
                let tb = rig.targets()[*idx].bounds;
                rig.redraw();
                let rest = rig.frame_scaled(&theme, scale);
                rig.point_at(Some(tb.center()));
                rig.redraw();
                let hot = rig.frame_scaled(&theme, scale);
                // What changed outside the control itself is the tooltip.
                let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
                let ex = |v: f32| (v * scale) as usize;
                for y in 0..600 * SCALE {
                    for x in 0..800 * SCALE {
                        if x + 8 >= ex(tb.x)
                            && x <= ex(tb.x + tb.width) + 8
                            && y + 8 >= ex(tb.y)
                            && y <= ex(tb.y + tb.height) + 8
                        {
                            continue;
                        }
                        let i = (y * 800 * SCALE + x) * 4;
                        if rest[i..i + 3] != hot[i..i + 3] {
                            x0 = x0.min(x);
                            y0 = y0.min(y);
                            x1 = x1.max(x);
                            y1 = y1.max(y);
                        }
                    }
                }
                assert!(
                    x1 > x0,
                    "{preset} / {name} / {surface}: hovering opened no tooltip"
                );
                // The body's inside: shrink by 6 logical px (border, shadow, padding), then read it like a run.
                let inset = 6.0;
                let b = Rectangle {
                    x: x0 as f32 / scale + inset,
                    y: y0 as f32 / scale + inset,
                    width: (x1 - x0) as f32 / scale - 2.0 * inset,
                    height: (y1 - y0) as f32 / scale - 2.0 * inset,
                };
                let (ink, bg, r) = read_run(&hot, b).unwrap_or_else(|| {
                    panic!("{preset} / {name} / {surface}: no ink in the tooltip")
                });
                cells += 1;
                if r + TOLERANCE < level(preset) {
                    failures.push(format!(
                        "{preset} / {name} / {surface}: {r:.2}:1 ({} on {}), level {}",
                        hex(ink),
                        hex(bg),
                        level(preset)
                    ));
                }
            }
        }
    }
    assert_eq!(
        cells, 54,
        "premise: three tooltips, six presets, three surfaces"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---- the completeness check: three tripwires, each with its plant -----------------------------------------------------------

/// A plant for tripwire 1 and for the sweep itself: the tab bar with a scratch label beside it, in a
/// colour of its own.
fn tab_bar_with_a_scratch_label(grey: u8) -> Scene {
    let mut scene = scenes().into_iter().find(|s| s.name == "tab bar").unwrap();
    let inner = scene.make;
    scene.make = Box::new(move |t, dir| {
        row![
            inner(t, dir),
            text("Extra label").color(Color::from_rgb8(grey, grey, grey))
        ]
        .into()
    });
    scene
}

/// **Tripwire 1.** A text run in a frame that the scene's table does not name fails, so a widget that
/// adds a label cannot be left out of the sweep by anyone's memory (the gap that let the tab through).
#[test]
fn tripwire_1_a_text_run_the_table_does_not_name_fails() {
    // The real tab bar is fully named.
    let real = sweep_of(
        vec![scenes().into_iter().find(|s| s.name == "tab bar").unwrap()],
        Some("light_aaa"),
    );
    assert!(real.unlisted.is_empty(), "{:?}", real.unlisted);
    // THE PLANT: a scratch label beside it.
    let planted = sweep_of(vec![tab_bar_with_a_scratch_label(200)], Some("light_aaa"));
    assert!(
        planted
            .unlisted
            .iter()
            .any(|m| m.ends_with("\"Extra label\"")),
        "the planted label should be reported as unnamed: {:?}",
        planted.unlisted
    );
}

/// The sweep itself catches a word under its level: the table is told the scratch label is a word (as
/// someone adding it properly would), and its grey is under 7:1 on `light_aaa`'s page.
#[test]
fn a_planted_word_under_its_level_is_found() {
    let mut scene = tab_bar_with_a_scratch_label(200);
    scene.runs = &[
        ("Tab0", Kind::Word),
        ("Tab1", Kind::Word),
        ("Tab2", Kind::Word),
        ("Tab3", Kind::Word),
        ("\u{2605}", Kind::Glyph("icon")),
        ("Extra label", Kind::Word),
    ];
    let planted = sweep_of(vec![scene], Some("light_aaa"));
    assert!(planted.unlisted.is_empty());
    let found = under(&planted.cells);
    assert!(
        found
            .iter()
            .any(|m| m.contains("Extra label") && m.contains("word")),
        "the grey label (200,200,200 on white, 1.7:1) should be under: {found:?}"
    );
}

/// A file that draws text with a renderer primitive must announce that text to `Operation::text`.
fn unannounced(files: &[(String, String)]) -> Vec<String> {
    const DRAWS: [&str; 3] = ["text::draw(", "fill_paragraph(", "fill_text("];
    files
        .iter()
        .filter(|(_, src)| DRAWS.iter().any(|d| src.contains(d)))
        .filter(|(_, src)| !src.contains("operation.text("))
        .map(|(p, _)| p.clone())
        .collect()
}

/// The non-test sources of the crates that draw (a path under `tests` and a `tests.rs` are skipped).
fn source_files() -> Vec<(String, String)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let name = p.to_string_lossy().to_string();
                if name.contains("/tests") || name.ends_with("tests.rs") {
                    continue;
                }
                out.push((name, std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for krate in [
        "snora",
        "snora-widgets",
        "snora-interaction",
        "snora-style",
        "snora-design",
        "snora-core",
    ] {
        walk(&root.join(krate).join("src"), &mut out);
    }
    out
}

/// **Tripwire 2.** The inventory is read from `Operation::text`, so a snora widget that draws text
/// without announcing it would be invisible to the sweep. The check reads source for **the primitive
/// the text is drawn with**, never for a colour (a colour search is the method that missed the tab).
/// Today two files draw text, `em.rs` (`Glyph`) and `underlined.rs`, and both announce.
#[test]
fn tripwire_2_a_widget_that_draws_text_without_announcing_it_fails() {
    let files = source_files();
    assert!(
        files.len() > 50,
        "premise: the sources were read ({} files)",
        files.len()
    );
    let real = unannounced(&files);
    assert!(
        real.is_empty(),
        "these files draw text and do not announce it: {real:?}"
    );
    // THE PLANT: a widget drawing a label with the renderer's primitive and no announcement.
    let mut planted = files.clone();
    planted.push((
        "crates/snora-widgets/src/scratch.rs".into(),
        "fn draw(r: &mut R) { r.fill_paragraph(p, pos, c, clip); }".into(),
    ));
    assert_eq!(
        unannounced(&planted),
        vec!["crates/snora-widgets/src/scratch.rs".to_owned()]
    );
}

/// The identifiers `snora::design` exports from `snora-widgets`: the builders a sweep must have a scene for.
fn exported(design_rs: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = design_rs;
    while let Some(i) = rest.find("pub use snora_widgets::") {
        rest = &rest[i + 6..];
        let end = rest.find(';').unwrap();
        let stmt = &rest[..end];
        if let (Some(a), Some(b)) = (stmt.find('{'), stmt.find('}')) {
            names.extend(
                stmt[a + 1..b]
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty()),
            );
        } else if let Some(last) = stmt.rsplit("::").next() {
            names.push(last.trim().to_owned());
        }
        rest = &rest[end..];
    }
    // A label-placement enum is an argument, not a widget.
    names.retain(|n| n != "SideBarLabels");
    names
}

/// **Tripwire 3.** A design widget exported from `snora::design` with no scene in the sweep fails.
#[test]
fn tripwire_3_an_exported_design_widget_with_no_scene_fails() {
    let covered: Vec<&str> = scenes()
        .iter()
        .flat_map(|s| s.covers.iter().copied())
        .collect::<Vec<_>>()
        .into_iter()
        .collect();
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/design.rs"),
    )
    .unwrap();
    let names = exported(&src);
    assert!(
        names.len() > 20,
        "premise: the exports were read: {names:?}"
    );
    let missing: Vec<_> = names
        .iter()
        .filter(|n| !covered.contains(&n.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "exported with no scene in the sweep: {missing:?}"
    );
    // THE PLANT: a widget added to the exports and not to the sweep.
    let planted = format!(
        "{src}\npub mod widget2 {{ pub use snora_widgets::design::widget::{{app_toolbar}}; }}\n"
    );
    let missing: Vec<_> = exported(&planted)
        .into_iter()
        .filter(|n| !covered.contains(&n.as_str()))
        .collect();
    assert_eq!(missing, vec!["app_toolbar".to_owned()]);
}
