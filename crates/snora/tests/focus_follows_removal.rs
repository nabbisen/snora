//! Where focus goes when the focused control goes away (RFC-120, ruled
//! 2026-11-08).
//!
//! > **When the focused control goes away, focus moves to its nearest neighbour
//! > in the same group: the next one, or else the previous one. Otherwise,
//! > nothing is focused.**
//!
//! Every case here presses the removal control **from the keyboard** (Tab to
//! it, Enter), through the engine's page (`snora::render`, so with its
//! `Inert` tracker and its keyed toast stack), lets the application rebuild
//! without what the press removed, and reads where focus is three ways that
//! must agree: the tree's focus flags, a ring **read from pixels**, and what one
//! more Enter publishes. (A test that only asks `find_focused` can see only what
//! `operate` reports; RFC-119 found the trap.)
//!
//! The raw behaviour these replace, and the stated limit (L-7), are in
//! `focus_after_removal.rs`.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). A wgpu run must
//! use `--test-threads=1`.

#![cfg(all(feature = "widgets", feature = "design"))]

mod common;
use common::removal_scenes::{Landing, cases, press_and_rebuild};

/// One test per row of Unit 0's table, against the ruled target.
#[test]
fn every_removal_hands_focus_to_the_neighbour_in_its_group() {
    let mut failures = Vec::new();
    for case in cases() {
        for pressed in case.pressed {
            let want = match (case.expected)(*pressed) {
                Some(name) => Landing::on(name),
                None => Landing::nowhere(),
            };
            let got = press_and_rebuild(&case, *pressed, false);
            if got != want {
                failures.push(format!(
                    "{} pressing {pressed}: expected {want:?}, got {got:?}",
                    case.name
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} row(s) off:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

use common::removal::Rig;
use common::removal_scenes::{
    Case, dialog_with_chips, list_of_rows_with_two_controls, msg, names, read, rtl_cases, toasts,
    without,
};

fn assert_landings(cases: Vec<Case>) {
    let mut failures = Vec::new();
    for case in cases {
        for pressed in case.pressed {
            let want = match (case.expected)(*pressed) {
                Some(name) => Landing::on(name),
                None => Landing::nowhere(),
            };
            let got = press_and_rebuild(&case, *pressed, false);
            if got != want {
                failures.push(format!(
                    "{} pressing {pressed}: expected {want:?}, got {got:?}",
                    case.name
                ));
            }
        }
    }
    assert!(failures.is_empty(), "off:\n  {}", failures.join("\n  "));
}

/// **Under RTL as well: the mirrored order.** Tab goes the other way along a
/// breadcrumb trail, so "the next one" is the other crumb, and the ruled
/// target is the same crumb: the bar's one that is still there.
#[test]
fn right_to_left_follows_the_mirrored_order() {
    assert_landings(rtl_cases());
}

fn the_list() -> Case {
    common::removal_scenes::cases()
        .into_iter()
        .find(|c| c.name.starts_with("application list"))
        .unwrap()
}

/// **The application's own focus wins** (RFC-119's rule A, as `Inert`'s restore
/// has it): a focus task in the same update, and R-1 does nothing.
///
/// Both orders are covered. In a real runtime (measured under niri, native
/// Wayland: `evidence/probe-app-B.log` in the Unit 1 package) the task's
/// operation runs **after `view` and before the first `RedrawRequested`**, so
/// the first order is the real one, and no frame is drawn between the
/// tracker's choice and the application's. The second is kept because nothing
/// in iced promises it.
#[test]
fn the_applications_own_focus_wins_in_either_order() {
    for task_first in [true, false] {
        let case = the_list();
        let view = |ids: &[u8]| case.view(ids, false);
        let mut rig = Rig::new(view(case.ids));
        for _ in 0..2 {
            rig.tab(false); // delete 1
        }
        rig.enter();
        let left = without(case.ids, 1);
        let mut rig = rig.rebuilt_quiet(view(&left));
        if task_first {
            rig.focus_at(0);
            rig.redraw();
        } else {
            rig.redraw();
            rig.focus_at(0);
        }
        assert_eq!(
            rig.focused(),
            vec![0],
            "task first = {task_first}: only the application's choice has focus"
        );
    }
}

/// **The application's focus wins even when nothing runs an operation**: a click
/// into a text input between the press and the redraw focuses it by event, and
/// the tracker must not take focus from it.
///
/// (An operation, a focus task, always finds the removal already settled, since
/// the tracker resolves a pending removal before any operation runs, so the
/// application's operation is the last word and the test above passes either
/// way. This is the path that needs the check itself.)
#[test]
fn a_click_into_a_text_input_wins_too() {
    let view = |ids: &[u8]| {
        common::removal_scenes::page(
            common::removal_scenes::list_and_a_text_input(ids),
            snora::LayoutDirection::Ltr,
        )
    };
    let ids: &[u8] = &[0, 1, 2];
    let mut rig = Rig::new(view(ids));
    for _ in 0..2 {
        rig.tab(false); // delete 1
    }
    rig.enter();
    let left = without(ids, 1);
    let mut rig = rig.rebuilt_quiet(view(&left));
    // The text input is the last focus target, below the two rows. Where it is
    // is read from **another** interface: asking this one would run an
    // operation, and an operation settles the pending removal first.
    let input = Rig::new(view(&left))
        .targets()
        .last()
        .unwrap()
        .bounds
        .center();
    rig.click_at(input);
    rig.redraw();
    let targets = rig.targets();
    let last = targets.len() - 1;
    assert_eq!(
        rig.focused(),
        vec![last],
        "only the text input the application's user clicked has focus"
    );
}

/// A press that removes nothing: the control is still there, so focus stays on
/// it. The tracker took focus off everything to make room for the application's
/// choice, and gives it back.
#[test]
fn a_press_that_removes_nothing_keeps_focus() {
    let case = the_list();
    let view = |ids: &[u8]| case.view(ids, false);
    let mut rig = Rig::new(view(case.ids));
    for _ in 0..2 {
        rig.tab(false);
    }
    rig.enter();
    // The application rebuilt with the same rows: nothing was removed.
    let mut rig = rig.rebuilt(view(case.ids));
    let landing = read(&mut rig, case.ids, &view);
    assert_eq!(landing, Landing::on("delete 1"));
}

/// **The group boundary, the root crumb.** Its group is the bar, which stays
/// and holds no control now, so nothing is focused: never the footer beyond it.
#[test]
fn the_root_crumb_leaves_nothing_focused_and_not_the_footer() {
    let case = common::removal_scenes::cases()
        .into_iter()
        .find(|c| c.name.starts_with("ancestor"))
        .unwrap();
    let got = press_and_rebuild(&case, 0, false);
    assert_eq!(got, Landing::nowhere(), "focus went to {got:?}");
}

/// **The group boundary, the last toast.** The stack is its own layer and group:
/// with no toast left, nothing is focused, and never the page beneath.
#[test]
fn the_last_toast_leaves_nothing_focused_and_not_the_page() {
    let view = |ids: &[u8]| toasts(ids);
    let mut rig = Rig::new(view(&[1]));
    rig.tab(false); // the page's button
    rig.tab(false); // the toast's ×
    rig.enter();
    assert_eq!(rig.messages, vec![msg("toast 1")], "premise: it was the ×");
    let mut rig = rig.rebuilt(view(&[]));
    assert_eq!(rig.focused(), Vec::<usize>::new(), "the page took focus");
}

/// **A removal of more than the pressed control, undeclared:** a row with Edit
/// and Delete goes whole when Delete is pressed. snora cannot tell what went,
/// so nothing is focused, and never a control inherited by position.
#[test]
fn an_undeclared_removal_of_two_controls_leaves_nothing_focused() {
    let view = |ids: &[u8]| {
        common::removal_scenes::page(
            list_of_rows_with_two_controls(ids),
            snora::LayoutDirection::Ltr,
        )
    };
    let ids: &[u8] = &[0, 1, 2];
    let tab_order = names(view, ids);
    let at = tab_order.iter().position(|n| n == "delete 0").unwrap();
    let mut rig = Rig::new(view(ids));
    for _ in 0..=at {
        rig.tab(false);
    }
    rig.enter();
    let left = without(ids, 0);
    let mut rig = rig.rebuilt(view(&left));
    assert_eq!(read(&mut rig, &left, &view), Landing::nowhere());
}

/// **A dialog is a layer of its own.** Removing a chip in a dialog hands focus to
/// a control in the dialog, and with none left, nothing: never the page.
#[test]
fn a_removal_in_a_dialog_stays_in_the_dialog() {
    let view = |ids: &[u8]| dialog_with_chips(ids);
    // Two chips: pressing the last one's × lands on the previous one's ×.
    let ids: &[u8] = &[0, 1];
    let order = names(view, ids);
    assert_eq!(
        order,
        ["toggle 0", "remove 0", "toggle 1", "remove 1"],
        "premise: the dialog's controls are what Tab reaches"
    );
    let mut rig = Rig::new(view(ids));
    for _ in 0..4 {
        rig.tab(false);
    }
    rig.enter();
    let left = without(ids, 1);
    let mut rig = rig.rebuilt(view(&left));
    assert_eq!(read(&mut rig, &left, &view), Landing::on("remove 0"));

    // One chip: its × is the last control. Nothing is left in the dialog.
    let ids: &[u8] = &[0];
    let mut rig = Rig::new(view(ids));
    for _ in 0..2 {
        rig.tab(false);
    }
    rig.enter();
    let mut rig = rig.rebuilt(view(&[]));
    assert_eq!(read(&mut rig, &[], &view), Landing::nowhere());
}

/// **Toasts are known by id** (RFC-120 D-2). A toast that expires or is closed
/// somewhere else leaves focus on the toast's × where it was, whatever moves
/// into the vacated place. (Unit 0, case 1b: toast 3 expiring moved focus from
/// toast 2's × to toast 1's, and one more Enter closed a toast nobody was
/// reading.)
#[test]
fn a_toast_that_goes_elsewhere_leaves_focus_on_its_toast() {
    for focus in [1u8, 2, 3] {
        for gone in [1u8, 2, 3] {
            if focus == gone {
                continue;
            }
            let view = |ids: &[u8]| toasts(ids);
            let all: &[u8] = &[1, 2, 3];
            let at = names(view, all)
                .iter()
                .position(|n| *n == format!("toast {focus}"))
                .unwrap();
            let mut rig = Rig::new(view(all));
            for _ in 0..=at {
                rig.tab(false);
            }
            // The next redraw is what the runtime sends before showing the ring.
            rig.redraw();
            let left = without(all, gone);
            let mut rig = rig.rebuilt(view(&left));
            assert_eq!(
                read(&mut rig, &left, &view),
                Landing::on(&format!("toast {focus}")),
                "focus was on toast {focus} and toast {gone} went"
            );
        }
    }
}

// ---- the sources of the hold, as the engine wires them (RFC-120 R-3) --------------

use iced::widget::container;
use snora::design::button;
use snora::{AppLayout, Toast, ToastAttention, ToastIntent};

#[derive(Debug, Clone, PartialEq)]
enum Wired {
    Attention(u64, ToastAttention),
    Hover(u64),
    Closed(u64),
}

#[expect(deprecated, reason = "wires the deprecated `on_toast_hover` as well")]
fn wired_view(both: bool, hover_only: bool) -> iced::Element<'static, Wired> {
    let tokens: &'static snora::design::Tokens =
        Box::leak(Box::new(snora::design::Tokens::light()));
    let body: iced::Element<'static, Wired> =
        container(button::ghost(tokens, "Page", Wired::Closed(0))).into();
    let toast = Toast::new(7, ToastIntent::Info, "Saved", "All good.", Wired::Closed(7));
    let mut layout = AppLayout::new(body).toasts(vec![toast]);
    if both || !hover_only {
        layout = layout.on_toast_attention(Wired::Attention);
    }
    if both || hover_only {
        layout = layout.on_toast_hover(|id, _| Wired::Hover(id));
    }
    snora::render(layout)
}

/// Keyboard focus on a toast's × is reported: gained on Tab, lost on Tab away.
#[test]
fn focus_on_a_toasts_close_control_is_reported() {
    let mut rig = Rig::new(wired_view(false, false));
    rig.tab(false); // the page's button
    rig.redraw();
    assert!(
        !rig.messages
            .contains(&Wired::Attention(7, ToastAttention::FocusGained)),
        "premise: focus is on the page, not the toast"
    );
    rig.tab(false); // the toast's ×
    rig.redraw();
    assert!(
        rig.messages
            .contains(&Wired::Attention(7, ToastAttention::FocusGained)),
        "focus arrived on the toast's ×: {:?}",
        rig.messages
    );
    rig.messages.clear();
    rig.tab(true); // back to the page's button
    rig.redraw();
    assert_eq!(
        rig.messages,
        vec![Wired::Attention(7, ToastAttention::FocusLost)],
        "focus left the toast's ×"
    );
}

/// **Both callbacks set: one hold per crossing.** The pointer's crossings go to
/// `on_toast_attention` only, so a toast is never held twice for one crossing.
#[test]
fn with_both_callbacks_set_a_crossing_reports_once() {
    let mut rig = Rig::new(wired_view(true, false));
    // Onto the toast's × (it is the last focus target; its box is on the toast).
    let on = rig.targets().last().unwrap().bounds.center();
    rig.point_at(Some(on));
    assert_eq!(
        rig.messages,
        vec![Wired::Attention(7, ToastAttention::PointerEntered)],
        "one report per crossing, to the new callback only"
    );
    rig.messages.clear();
    rig.point_at(None);
    assert_eq!(
        rig.messages,
        vec![Wired::Attention(7, ToastAttention::PointerLeft)]
    );
}

/// **A deprecated path still works:** `on_toast_hover` alone still reports.
#[test]
fn on_toast_hover_alone_still_reports_the_pointer() {
    let mut rig = Rig::new(wired_view(false, true));
    let on = rig.targets().last().unwrap().bounds.center();
    rig.point_at(Some(on));
    assert_eq!(rig.messages, vec![Wired::Hover(7)]);
}
