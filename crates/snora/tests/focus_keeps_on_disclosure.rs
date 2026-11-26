//! A control that **keeps focus** (RFC-137, ruled 2026-11-26): `keeps_focus(id)`.
//!
//! > **After a press from the keyboard, focus stays on a control that declared it, whatever else
//! > the press adds or removes. It is found by its id. If no target, or more than one, has the id,
//! > nothing is focused.**
//!
//! A *disclosure* is a control that stays where it is when pressed and shows or hides others after
//! it. Before 0.66 a press that changed how many controls the page had left **nothing** focused
//! (RFC-120: position and count cannot tell a disclosure from a replacement), so the next Tab
//! started at the top of the window.
//!
//! Each case presses **from the keyboard** (Tab to it, Enter) through the engine's page, lets the
//! application rebuild, and reads focus three ways that must agree: the tree's flags, a ring read
//! from pixels, and what one more Enter publishes. Both public paths (`snora::control`,
//! `snora::design::control`) and both directions are covered.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). A wgpu run must use
//! `--test-threads=1`.

#![cfg(all(feature = "widgets", feature = "design"))]

mod common;
use common::disclosure_scenes::{
    ABOVE, DIRECTIONS, Decl, GONE, INPUT, PATHS, TOOL2, TWIN, clicked, disclosure, file_tree,
    press, pressed, toolbar_disclosure,
};
use common::removal::Rig;
use common::removal_scenes::{Landing, Msg, names, read};

/// Runs `check` for every path and direction, collecting what is off so one run names every row.
fn everywhere(
    mut check: impl FnMut(common::disclosure_scenes::Path, snora::LayoutDirection) -> Vec<String>,
) {
    let mut off = Vec::new();
    for path in PATHS {
        for direction in DIRECTIONS {
            off.extend(
                check(path, direction)
                    .into_iter()
                    .map(|f| format!("{path:?} / {direction:?}: {f}")),
            );
        }
    }
    assert!(
        off.is_empty(),
        "{} row(s) off:\n  {}",
        off.len(),
        off.join("\n  ")
    );
}

fn expect(rows: &mut Vec<String>, what: &str, got: Landing, want: Landing) {
    if got != want {
        rows.push(format!("{what}: expected {want:?}, got {got:?}"));
    }
}

/// "Advanced settings" opens, two options appear after it: focus stays on the header.
#[test]
fn a_disclosure_that_opens_keeps_focus() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        expect(
            &mut rows,
            "open",
            press(&view, &[], "header", &[0, 1]),
            Landing::on("header"),
        );
        rows
    });
}

/// The same, closing: the two options go, focus stays on the header.
#[test]
fn a_disclosure_that_closes_keeps_focus() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        expect(
            &mut rows,
            "close",
            press(&view, &[0, 1], "header", &[]),
            Landing::on("header"),
        );
        rows
    });
}

/// Open, close and open again, from the keyboard each time: focus stays through all three
/// presses, and the one more Enter that reads it is the header's.
#[test]
fn a_disclosure_keeps_focus_through_repeated_presses() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        let mut rig = pressed(&view, &[], "header");
        for next in [&[0u8, 1][..], &[][..], &[0, 1][..]] {
            rig.messages.clear();
            rig = rig.rebuilt(view(next));
            let got = read(&mut rig, next, &view);
            expect(
                &mut rows,
                &format!("after rebuilding to {next:?}"),
                got,
                Landing::on("header"),
            );
            // `read` pressed Enter once on the header (it published "header"); the press is the
            // next one of the sequence, so the rebuild that follows reads its own focus.
        }
        rows
    });
}

/// aaai's file-tree row (`width(Fill)`, an indent, an icon and a name) expands a folder, and
/// collapses it: focus stays on the row.
#[test]
fn a_file_tree_row_keeps_focus_when_a_folder_opens_and_closes() {
    everywhere(|path, direction| {
        let view = file_tree(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        expect(
            &mut rows,
            "expand",
            press(&view, &[], "src", &[1, 2]),
            Landing::on("src"),
        );
        expect(
            &mut rows,
            "collapse",
            press(&view, &[1, 2], "src", &[]),
            Landing::on("src"),
        );
        // A folder below the open one: its row moves down by two, and it is found by its id.
        expect(
            &mut rows,
            "expand the second folder while the first is open",
            press(&view, &[1, 2], "tests", &[1, 2]),
            Landing::on("tests"),
        );
        rows
    });
}

/// A control is **inserted before the header** in the same press: the header moves in Tab order, and
/// it is found by its id, not by where it was.
#[test]
fn a_control_inserted_before_the_header_does_not_move_focus_off_it() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        expect(
            &mut rows,
            "above and options appear",
            press(&view, &[], "header", &[ABOVE, 0, 1]),
            Landing::on("header"),
        );
        // and one that goes before it
        expect(
            &mut rows,
            "above goes and options appear",
            press(&view, &[ABOVE], "header", &[0, 1]),
            Landing::on("header"),
        );
        rows
    });
}

/// **A change earlier in the page** (RFC-137 review, B-1). iced keeps a control's state by its place in
/// the **tree**, which is not its place in Tab order: when the number of controls *before* the pressed one
/// changes, the focus its state carries can end up on a **stranger**, at a different Tab index. The
/// header is still found by its id, whatever moved, and `above` / `option 1` never get the focus.
///
/// The first two rows are the ones that held before the fix (the second only because the state stayed with
/// the header itself, so the check passed for the wrong reason); the other three are the ones that did not.
#[test]
fn a_change_earlier_in_the_page_does_not_hand_focus_to_a_stranger() {
    everywhere(|path, direction| {
        let view = toolbar_disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        for (what, before, after) in [
            ("same column: above appears", &[][..], &[ABOVE, 1, 2][..]),
            (
                "earlier container only: tool 2 appears",
                &[][..],
                &[TOOL2, 1, 2][..],
            ),
            (
                "tool 2 appears and above appears",
                &[][..],
                &[ABOVE, TOOL2, 1, 2][..],
            ),
            (
                "tool 2 appears and above goes",
                &[ABOVE][..],
                &[TOOL2, 1, 2][..],
            ),
            (
                "tool 2 goes and above appears",
                &[TOOL2][..],
                &[ABOVE, 1, 2][..],
            ),
            (
                "tool 2 goes and the options go",
                &[TOOL2, 1, 2][..],
                &[][..],
            ),
        ] {
            expect(
                &mut rows,
                what,
                press(&view, before, "header", after),
                Landing::on("header"),
            );
        }
        rows
    });
}

/// A control that **declared `keeps_focus` and goes away** (the application removed it): nothing
/// is focused, never the neighbour that moved into its place.
#[test]
fn a_misdeclared_control_that_goes_away_focuses_nothing() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        // With one control left (`after`), which moves into the header's position: by position it
        // would be focused.
        expect(
            &mut rows,
            "the header goes, its neighbour moves into its place",
            press(&view, &[], "header", &[GONE]),
            Landing::nowhere(),
        );
        expect(
            &mut rows,
            "the header goes, and a neighbour is added too",
            press(&view, &[0], "header", &[GONE, 0, 1]),
            Landing::nowhere(),
        );
        rows
    });
}

/// **Two targets with the id**: snora cannot resolve it, and nothing is focused.
#[test]
fn two_targets_with_one_id_focus_nothing() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        expect(
            &mut rows,
            "a twin appears",
            press(&view, &[], "header", &[0, TWIN]),
            Landing::nowhere(),
        );
        rows
    });
}

/// An **undeclared** disclosure is as before: nothing is focused after the count changes.
#[test]
fn an_undeclared_disclosure_focuses_nothing_as_before() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Undeclared, direction);
        let mut rows = Vec::new();
        expect(
            &mut rows,
            "open",
            press(&view, &[], "header", &[0, 1]),
            Landing::nowhere(),
        );
        expect(
            &mut rows,
            "close",
            press(&view, &[0, 1], "header", &[]),
            Landing::nowhere(),
        );
        // Nothing changed: still where it was, as RFC-120 says.
        expect(
            &mut rows,
            "no change",
            press(&view, &[0], "header", &[0]),
            Landing::on("header"),
        );
        rows
    });
}

/// A **click** focuses nothing, declared or not, and Tab carries on from the clicked control
/// (RFC-121): the next Tab goes to the target after it.
#[test]
fn a_click_on_a_disclosure_focuses_nothing_and_tab_carries_on_from_it() {
    everywhere(|path, direction| {
        let mut rows = Vec::new();
        for decl in [Decl::Keeps, Decl::Undeclared] {
            let view = disclosure(path, decl, direction);
            for (what, before, after) in [
                ("open", &[][..], &[0u8, 1][..]),
                ("close", &[0, 1][..], &[][..]),
            ] {
                let mut rig = clicked(&view, before, "header", after);
                let got = read(&mut rig, after, &view);
                expect(
                    &mut rows,
                    &format!("{decl:?}: click and {what}"),
                    got,
                    Landing::nowhere(),
                );

                let mut rig = clicked(&view, before, "header", after);
                rig.tab(false);
                let order = names(&view, after);
                let header = order
                    .iter()
                    .position(|n| n == "header")
                    .expect("the header");
                let focused: Vec<String> =
                    rig.focused().iter().map(|i| order[*i].clone()).collect();
                if focused != [order[header + 1].clone()] {
                    rows.push(format!(
                        "{decl:?}: click and {what}, then Tab: expected focus on {:?}, got {focused:?}",
                        order[header + 1]
                    ));
                }
            }
        }
        rows
    });
}

/// **The application's own focus wins**: a focus task in the same update, in either order
/// against the redraw, and the header does not take it.
#[test]
fn the_applications_own_focus_wins_over_keeps_focus() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        for task_first in [true, false] {
            let mut rig = pressed(&view, &[], "header");
            rig.messages.clear();
            let mut rig: Rig<Msg> = rig.rebuilt_quiet(view(&[0, 1]));
            // Target 1 is "option 0".
            if task_first {
                rig.focus_at(1);
                rig.redraw();
            } else {
                rig.redraw();
                rig.focus_at(1);
            }
            if rig.focused() != vec![1] {
                rows.push(format!(
                    "task first = {task_first}: expected only the application's choice, got {:?}",
                    rig.focused()
                ));
            }
        }
        rows
    });
}

/// **The application's focus wins even when nothing runs an operation**: a click into a text input
/// between the press and the redraw focuses it by event, and `keeps_focus` must not take focus from it.
/// (A focus task always finds the press already settled, since `Inert` resolves a pending press before
/// any operation, so the test above passes either way; this is the path that needs the check itself.)
#[test]
fn a_click_into_a_text_input_wins_over_keeps_focus_too() {
    everywhere(|path, direction| {
        let view = disclosure(path, Decl::Keeps, direction);
        let mut rows = Vec::new();
        let after = [0u8, 1, INPUT];
        let mut rig = pressed(&view, &[], "header");
        rig.messages.clear();
        let mut rig: Rig<Msg> = rig.rebuilt_quiet(view(&after));
        // Where the input is is read from **another** interface: asking this one would run an operation,
        // and an operation settles the pending press first.
        let input = Rig::new(view(&after))
            .targets()
            .last()
            .expect("the input")
            .bounds
            .center();
        rig.click_at(input);
        rig.redraw();
        let last = rig.targets().len() - 1;
        if rig.focused() != vec![last] {
            rows.push(format!(
                "expected only the text input to have focus, got {:?}",
                rig.focused()
            ));
        }
        rows
    });
}

/// A **disabled** control registers no focus target, so a declaration on it has no effect, and no
/// focus target is invented for it.
#[test]
fn a_disabled_control_with_the_declaration_is_not_a_focus_target() {
    use iced::widget::{column, text};
    let build = || -> iced::Element<'static, Msg> {
        snora::render(snora::AppLayout::new(
            column![
                snora::control(text("off"))
                    .on_press_maybe(None)
                    .keeps_focus("off"),
                snora::control(text("on"))
                    .on_press(common::removal_scenes::msg("on"))
                    .keeps_focus("on"),
            ]
            .into(),
        ))
    };
    let mut rig = Rig::new(build());
    assert_eq!(
        rig.targets().len(),
        1,
        "only the enabled control is a focus target"
    );
}

/// `id` called after `keeps_focus` is the id that is looked for (the doc comment says so): the
/// later id is the control's. A control that has the **first** id is placed before the header after
/// the press, so a lookup by the replaced id would land on it.
#[test]
fn an_id_set_after_keeps_focus_is_the_one_looked_for() {
    use common::removal_scenes::msg;
    use iced::widget::{column, text};
    let build = |open: bool| -> iced::Element<'static, Msg> {
        let mut col = column![];
        if open {
            col = col.push(
                snora::control(text("decoy"))
                    .on_press(msg("decoy"))
                    .id("first"),
            );
        }
        col = col.push(
            snora::control(text("header"))
                .on_press(msg("header"))
                .keeps_focus("first")
                .id("second"),
        );
        snora::render(snora::AppLayout::new(col.into()))
    };
    let mut rig = Rig::new(build(false));
    rig.tab(false);
    rig.enter();
    let mut rig = rig.rebuilt(build(true));
    let order = names(|ids: &[u8]| build(!ids.is_empty()), &[1]);
    assert_eq!(
        order,
        vec!["decoy", "header"],
        "the premise: the decoy is first"
    );
    assert_eq!(
        rig.focused(),
        vec![1],
        "focus stays on the header, found by its own id, not the decoy's"
    );
}
