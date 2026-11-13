//! What iced does with focus when a control goes away, **without** the
//! engine's help (RFC-120 Unit 0, kept as the characterisation).
//!
//! RFC-116's long-term plan said "focus goes nowhere, and the next Tab starts
//! from the top of the window". That was read from the code and never
//! measured, and iced keeps a widget's state **by its position in the tree**,
//! so the control that moves into a removed one's place may inherit its focus.
//! Unit 0 measured it: in 6 of 12 removals focus went nowhere, and in the other 6
//! it landed on a control the user never reached, and one more Enter pressed it.
//!
//! These scenes are **raw**: no `snora::render` around them, so no `Inert`
//! tracker and no keyed toast stack. They show what the engine corrects, and
//! they are not what an application gets (that is `focus_follows_removal.rs`).
//! The table is printed (`--nocapture`); the assertions are only the premises,
//! so this file records and does not rule.
//!
//! **Not covered, a stated limit (RFC-116 L-7):** a removal *elsewhere* in an
//! application's list moves focus too (`a_removal_elsewhere_…`). Nothing says
//! which control went, so snora cannot put focus back. Those rows are a
//! characterisation of the limit, not a claim that it is fixed. (The toast stack
//! *is* fixed, by id, and is tested in `focus_follows_removal.rs`.)
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). A wgpu run must
//! use `--test-threads=1`.

#![cfg(all(feature = "widgets", feature = "design"))]

mod common;
use common::removal::{Rig, backend};
use common::removal_scenes::{Case, cases, chips, list, names, press_and_rebuild, read, without};

fn raw_cases() -> Vec<Case> {
    // The toast stack can only be drawn by the engine, so it has no raw scene.
    cases().into_iter().filter(|c| !c.engine_only).collect()
}

/// Iced's own behaviour, for every raw case and every pressed item. Printed as a
/// table; asserts only the premises (inside `press_and_rebuild`).
#[test]
fn what_iced_does_when_the_focused_control_goes_away() {
    println!("\n## focus after a removal, no engine ({})\n", backend());
    println!("| case | pressed | focused | ring | one more Enter |");
    println!("|---|---|---|---|---|");
    for case in raw_cases() {
        for pressed in case.pressed {
            let l = press_and_rebuild(&case, *pressed, true);
            println!(
                "| {} | {} | {:?} | {:?} | {:?} |",
                case.name, pressed, l.focused, l.ring, l.enter
            );
        }
    }
    println!("\nTab order, before any removal:");
    for case in raw_cases() {
        let view = |ids: &[u8]| case.view(ids, true);
        println!("- {}: {:?}", case.name, names(view, case.ids));
    }
}

/// **L-7, a known limit.** A removal that is not the focused control's: the
/// application removes something else while focus sits on a control that
/// stays. Focus is lost or moved onto another control. snora cannot fix this:
/// nothing in an application's list says which control went.
///
/// **This test records a stated limit; it does not guard anything, and it asserts nothing.** It
/// prints where focus is after each removal (run it with `--nocapture`), so that the table in the
/// book can be checked against what iced does today. It cannot fail, by design: "can" in the
/// book's sentence is true by construction of iced (an application's list has no identity snora
/// can read), and a limit has no behaviour to hold. The 0.62 audit (A-090) counts it as a
/// measurement, not as a guard.
#[test]
fn a_removal_elsewhere_in_an_applications_list_moves_focus_l7() {
    type Scene = fn(&[u8]) -> iced::Element<'static, common::removal_scenes::Msg>;
    struct Elsewhere {
        name: &'static str,
        scene: Scene,
        focus: &'static str,
        remove: u8,
    }
    let rows = [
        Elsewhere {
            name: "list: row 0 removed",
            scene: list,
            focus: "delete 1",
            remove: 0,
        },
        Elsewhere {
            name: "list: row 0 removed",
            scene: list,
            focus: "delete 2",
            remove: 0,
        },
        Elsewhere {
            name: "list: row 2 removed",
            scene: list,
            focus: "delete 0",
            remove: 2,
        },
        Elsewhere {
            name: "chips: chip 0 removed",
            scene: chips,
            focus: "remove 2",
            remove: 0,
        },
    ];
    println!(
        "\n## L-7: focus on a control that stays, something else removed ({})\n",
        backend()
    );
    println!("| removed | focus was on | still focused after |");
    println!("|---|---|---|");
    for e in rows {
        let ids: &[u8] = &[0, 1, 2];
        let view = |ids: &[u8]| (e.scene)(ids);
        let at = names(view, ids)
            .iter()
            .position(|n| n == e.focus)
            .expect("a control of that name");
        let mut rig = Rig::new(view(ids));
        for _ in 0..=at {
            rig.tab(false);
        }
        let left = without(ids, e.remove);
        let mut rig = rig.rebuilt(view(&left));
        let landing = read(&mut rig, &left, &view);
        let after = landing
            .focused
            .first()
            .cloned()
            .unwrap_or_else(|| "nowhere".to_owned());
        println!("| {} | {} | {} |", e.name, e.focus, after);
    }
}
