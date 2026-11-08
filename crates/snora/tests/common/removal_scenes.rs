//! Scenes for the focus-after-removal tests (RFC-120): the five cases of the
//! finding, each a view that can be rebuilt without what a press removed.
//!
//! Used raw (iced's own positional state, Unit 0's characterisation) and wrapped
//! in the engine's page (`snora::render`, the behaviour RFC-120 rules).

#![allow(dead_code)]

use iced::Element;
use iced::widget::{column, container, row, text};

use snora::design::{Tokens, Tone, button, chip, notice::Notice};
use snora::{AppLayout, Crumb, LayoutDirection, Toast, ToastIntent, render};

use super::removal::Rig;

/// What every control in these scenes publishes: its own name.
#[derive(Debug, Clone, PartialEq)]
pub struct Msg(pub String);

pub fn msg(s: impl Into<String>) -> Msg {
    Msg(s.into())
}

pub fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<Tokens> = OnceLock::new();
    T.get_or_init(Tokens::light)
}

/// The page the engine builds around a view: the skeleton, in an `Inert`.
pub fn page(body: Element<'static, Msg>, direction: LayoutDirection) -> Element<'static, Msg> {
    render(AppLayout::new(body).direction(direction))
}

/// One case: a set of items, how to draw them, which control removes an item,
/// and what is left after it is pressed.
pub struct Case {
    pub name: &'static str,
    /// The items, in order (an id each).
    pub ids: &'static [u8],
    /// Which items' removal controls are pressed.
    pub pressed: &'static [u8],
    pub scene: fn(&[u8]) -> Element<'static, Msg>,
    /// Whether `scene` already goes through the engine (the toast stack can only
    /// be drawn by it).
    pub engine_only: bool,
    /// What the removal control of `id` publishes.
    pub removal: fn(u8) -> Msg,
    /// The items left after `id`'s control is pressed.
    pub after: fn(&[u8], u8) -> Vec<u8>,
    /// What RFC-120's rule says should have focus, written by hand from the
    /// rule and the Tab order, not from any mechanism. `None`: nothing.
    pub expected: fn(u8) -> Option<&'static str>,
    /// The layout direction of the page around the scene.
    pub direction: LayoutDirection,
}

impl Case {
    /// The view for `ids`, in the engine's page unless `raw`.
    pub fn view(&self, ids: &[u8], raw: bool) -> Element<'static, Msg> {
        let scene = (self.scene)(ids);
        if raw || self.engine_only {
            scene
        } else {
            page(scene, self.direction)
        }
    }
}

pub fn without(ids: &[u8], id: u8) -> Vec<u8> {
    ids.iter().copied().filter(|i| *i != id).collect()
}

pub fn chips(ids: &[u8]) -> Element<'static, Msg> {
    let t = tokens();
    container(
        row(ids.iter().map(|i| {
            chip::removable(
                t,
                format!("chip {i}"),
                false,
                msg(format!("toggle {i}")),
                msg(format!("remove {i}")),
            )
        }))
        .spacing(8),
    )
    .padding(24)
    .into()
}

pub fn notice_between(ids: &[u8]) -> Element<'static, Msg> {
    let t = tokens();
    let mut col = column![button::ghost(t, "Before", msg("before"))].spacing(12);
    if ids.contains(&0) {
        col = col.push(
            Notice::new(t, Tone::Info, "Index rebuilt.")
                .action("Undo", msg("notice undo"))
                .dismiss(msg("notice dismiss"))
                .render(),
        );
    }
    col = col.push(button::ghost(t, "After", msg("after")));
    container(col).padding(24).into()
}

pub fn toasts(ids: &[u8]) -> Element<'static, Msg> {
    toasts_in(ids, LayoutDirection::Ltr)
}

pub fn toasts_rtl(ids: &[u8]) -> Element<'static, Msg> {
    toasts_in(ids, LayoutDirection::Rtl)
}

fn toasts_in(ids: &[u8], direction: LayoutDirection) -> Element<'static, Msg> {
    let body = button::ghost(tokens(), "Page", msg("page"));
    let toasts = ids
        .iter()
        .map(|i| {
            Toast::new(
                u64::from(*i),
                ToastIntent::Info,
                "Saved",
                "All good.",
                msg(format!("toast {i}")),
            )
        })
        .collect();
    render(AppLayout::new(body).toasts(toasts).direction(direction))
}

pub fn crumbs(ids: &[u8]) -> Element<'static, Msg> {
    crumbs_in(ids, LayoutDirection::Ltr)
}

pub fn crumbs_rtl(ids: &[u8]) -> Element<'static, Msg> {
    crumbs_in(ids, LayoutDirection::Rtl)
}

fn crumbs_in(ids: &[u8], direction: LayoutDirection) -> Element<'static, Msg> {
    static ON_CRUMB: fn(snora::BreadcrumbAction<u8>) -> Msg = |a| {
        let snora::BreadcrumbAction::Pressed(i) = a;
        Msg(format!("crumb {i}"))
    };
    let last = ids.len().saturating_sub(1);
    let trail: Vec<Crumb<u8>> = ids
        .iter()
        .enumerate()
        .map(|(n, i)| {
            if n == last {
                Crumb::leaf(*i, format!("Level {i}"))
            } else {
                Crumb::ancestor(*i, format!("Level {i}"))
            }
        })
        .collect();
    container(
        column![
            snora::widget::app_breadcrumb(trail, &ON_CRUMB, direction),
            button::ghost(tokens(), "Footer", msg("footer")),
        ]
        .spacing(12),
    )
    .padding(24)
    .into()
}

pub fn list(ids: &[u8]) -> Element<'static, Msg> {
    container(
        column(ids.iter().map(|i| {
            row![
                text(format!("row {i}")),
                button::danger(tokens(), format!("Delete {i}"), msg(format!("delete {i}")))
            ]
            .spacing(12)
            .into()
        }))
        .spacing(8),
    )
    .padding(24)
    .into()
}

pub fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "chip x (row of three)",
            ids: &[0, 1, 2],
            pressed: &[0, 1, 2],
            scene: chips,
            engine_only: false,
            removal: |i| msg(format!("remove {i}")),
            after: without,
            expected: |i| Some(["toggle 1", "toggle 2", "remove 1"][usize::from(i)]),
            direction: LayoutDirection::Ltr,
        },
        Case {
            name: "notice dismiss (between two controls)",
            ids: &[0],
            pressed: &[0],
            scene: notice_between,
            engine_only: false,
            removal: |_| msg("notice dismiss"),
            after: without,
            expected: |_| Some("after"),
            direction: LayoutDirection::Ltr,
        },
        Case {
            name: "toast x (stack of three)",
            ids: &[1, 2, 3],
            pressed: &[1, 2, 3],
            scene: toasts,
            engine_only: true,
            removal: |i| msg(format!("toast {i}")),
            after: without,
            // Tab order is page, toast 3, toast 2, toast 1.
            expected: |i| Some(["", "toast 2", "toast 1", "toast 2"][usize::from(i)]),
            direction: LayoutDirection::Ltr,
        },
        Case {
            name: "ancestor crumb (trail of five)",
            ids: &[0, 1, 2, 3, 4],
            pressed: &[0, 1, 2, 3],
            scene: crumbs,
            engine_only: false,
            removal: |i| msg(format!("crumb {i}")),
            // Pressing an ancestor makes it the current page: what is after it goes.
            after: |ids, id| {
                ids.iter()
                    .copied()
                    .take_while(|i| *i <= id)
                    .collect::<Vec<u8>>()
            },
            // The crumb's new parent, which stays in the bar. The root crumb: its
            // group (the bar) has no control left, so nothing is focused, and not
            // the footer.
            expected: |i| [None, Some("crumb 0"), Some("crumb 1"), Some("crumb 2")][usize::from(i)],
            direction: LayoutDirection::Ltr,
        },
        Case {
            name: "application list row's Delete (three rows)",
            ids: &[0, 1, 2],
            pressed: &[0, 1, 2],
            scene: list,
            engine_only: false,
            removal: |i| msg(format!("delete {i}")),
            after: without,
            expected: |i| Some(["delete 1", "delete 2", "delete 1"][usize::from(i)]),
            direction: LayoutDirection::Ltr,
        },
    ]
}

/// The message each focus target of `view(ids)` publishes on Enter, in Tab
/// order.
pub fn names(view: impl Fn(&[u8]) -> Element<'static, Msg>, ids: &[u8]) -> Vec<String> {
    let n = Rig::new(view(ids)).targets().len();
    (0..n)
        .map(|k| {
            let mut rig = Rig::new(view(ids));
            for _ in 0..=k {
                rig.tab(false);
            }
            rig.enter();
            rig.messages
                .last()
                .map_or_else(|| "(nothing)".to_owned(), |m| m.0.clone())
        })
        .collect()
}

/// Where focus is after a removal, three ways, and they must agree: which
/// controls the tree says are focused, where a ring is drawn (pixels), and what
/// one more Enter publishes.
#[derive(Debug, PartialEq)]
pub struct Landing {
    /// The names of the focused controls (what each publishes on Enter).
    pub focused: Vec<String>,
    /// The control a ring is drawn around.
    pub ring: Option<String>,
    /// What one more Enter publishes, if anything.
    pub enter: Option<String>,
}

impl Landing {
    /// Nothing is focused: no focus, no ring, and Enter does nothing.
    pub fn nowhere() -> Self {
        Self {
            focused: vec![],
            ring: None,
            enter: None,
        }
    }

    /// Exactly `name` is focused, has the ring, and is what Enter presses.
    pub fn on(name: &str) -> Self {
        Self {
            focused: vec![name.to_owned()],
            ring: Some(name.to_owned()),
            enter: Some(name.to_owned()),
        }
    }
}

/// Presses `pressed`'s removal control **from the keyboard** (Tab to it,
/// Enter), rebuilds the view without what the press removed, and reads where
/// focus is. `raw`: the scene without the engine's page around it.
pub fn press_and_rebuild(case: &Case, pressed: u8, raw: bool) -> Landing {
    let view = |ids: &[u8]| case.view(ids, raw);
    let removal = (case.removal)(pressed);
    let before = names(view, case.ids);
    let index = before
        .iter()
        .position(|n| *n == removal.0)
        .unwrap_or_else(|| panic!("{}: no control publishes {removal:?}", case.name));

    let mut rig = Rig::new(view(case.ids));
    for _ in 0..=index {
        rig.tab(false);
    }
    assert_eq!(
        rig.focused().len(),
        1,
        "{}: the premise: the removal control has focus",
        case.name
    );
    rig.enter();
    assert_eq!(
        rig.messages,
        vec![removal],
        "{}: the premise: Enter pressed it",
        case.name
    );
    rig.messages.clear();

    let left = (case.after)(case.ids, pressed);
    let mut rig = rig.rebuilt(view(&left));
    read(&mut rig, &left, &view)
}

/// Where focus is now, in `rig`, drawn from `view(ids)`.
pub fn read(
    rig: &mut Rig<Msg>,
    ids: &[u8],
    view: &impl Fn(&[u8]) -> Element<'static, Msg>,
) -> Landing {
    let after = names(view, ids);
    let focused: Vec<String> = rig.focused().iter().map(|i| after[*i].clone()).collect();
    let ring = super::removal::ring_on(rig, &mut Rig::new(view(ids))).map(|i| after[i].clone());
    rig.messages.clear();
    rig.enter();
    let enter = rig.messages.last().map(|m| m.0.clone());
    Landing {
        focused,
        ring,
        enter,
    }
}

/// A list whose rows each carry **two** controls, Edit and Delete, and go
/// together: a removal of more than the pressed control that nothing declares.
pub fn list_of_rows_with_two_controls(ids: &[u8]) -> Element<'static, Msg> {
    container(
        column(ids.iter().map(|i| {
            row![
                text(format!("row {i}")),
                button::ghost(tokens(), format!("Edit {i}"), msg(format!("edit {i}"))),
                button::danger(tokens(), format!("Delete {i}"), msg(format!("delete {i}")))
            ]
            .spacing(12)
            .into()
        }))
        .spacing(8),
    )
    .padding(24)
    .into()
}

/// A page with an "Open" button under a dialog that holds the chips for `ids`.
pub fn dialog_with_chips(ids: &[u8]) -> Element<'static, Msg> {
    use snora::Dialog;
    let page = button::ghost(tokens(), "Open", msg("open"));
    render(AppLayout::new(page).dialog(Dialog::new(chips(ids))))
}

/// The same cases, in a right-to-left page: Tab goes the other way along a
/// breadcrumb trail, and "next" follows it.
pub fn rtl_cases() -> Vec<Case> {
    cases()
        .into_iter()
        .filter_map(|mut case| {
            case.scene = if case.name.starts_with("toast") {
                toasts_rtl
            } else if case.name.starts_with("ancestor") {
                crumbs_rtl
            } else {
                return None;
            };
            case.direction = LayoutDirection::Rtl;
            Some(case)
        })
        .collect()
}

/// The list's rows, and a text input after them: the application's own choice of
/// focus can come from the pointer, not only from a focus task.
pub fn list_and_a_text_input(ids: &[u8]) -> Element<'static, Msg> {
    column![
        list(ids),
        iced::widget::text_input("search", "").on_input(|_| msg("typed"))
    ]
    .into()
}
