//! Where does keyboard focus go when the focused control goes away?
//! (RFC-120, Unit 0: **measured**, not inferred.)
//!
//! RFC-116's long-term plan said "focus goes nowhere, and the next Tab starts
//! from the top of the window". That was read from the code and never measured,
//! and iced keeps a widget's state **by its position in the tree**, so the
//! control that moves into a removed one's place may inherit its focus.
//!
//! For each case this presses the removal control **from the keyboard** (Tab,
//! then Enter: the keyboard path RFC-121 does not change), rebuilds the view
//! without what the press removed, sends the `RedrawRequested` the runtime sends
//! before every draw, and reports:
//!
//! - **focus**: which controls the tree says are focused (`find_focused`);
//! - **ring**: where a ring is drawn, read from pixels (a frame compared with
//!   the same view never focused), and which control that is;
//! - **Enter**: what one more Enter publishes, which is the safest way to name
//!   the focused control.
//!
//! The rows are printed (`--nocapture`) as a table; the assertions are only the
//! premises (the removal control was focused, and Enter published its message),
//! so this file records today's behaviour and does not rule on it.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia), named by the
//! renderer itself. A wgpu run must use `--test-threads=1`.

#![cfg(all(feature = "widgets", feature = "design"))]

use iced::widget::{column, container, row, text};
use iced::{Element, Font, Pixels, Size, Theme, mouse};
use iced_runtime::core::clipboard;
use iced_runtime::core::renderer::Headless;
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Event, Rectangle, window};
use iced_runtime::user_interface::{self, UserInterface};

use snora::design::{Tokens, Tone, button, chip, notice::Notice};
use snora::{AppLayout, Crumb, LayoutDirection, Toast, ToastIntent, render};

#[derive(Debug, Clone, PartialEq)]
struct Msg(String);

fn msg(s: impl Into<String>) -> Msg {
    Msg(s.into())
}

const WINDOW: Size = Size::new(800.0, 600.0);

fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<Tokens> = OnceLock::new();
    T.get_or_init(Tokens::light)
}

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned())
}

// ---- the runtime's own interface ------------------------------------------------

struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    messages: Vec<Msg>,
}

struct Target {
    bounds: Rectangle,
    focused: bool,
}

impl Rig {
    fn new(root: Element<'static, Msg>) -> Self {
        let wanted = backend();
        let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::DEFAULT,
            Pixels(16.0),
            Some(&wanted),
        ))
        .expect("the requested renderer starts");
        assert_eq!(Headless::name(&renderer), wanted);
        let ui = UserInterface::build(
            root,
            WINDOW,
            user_interface::Cache::default(),
            &mut renderer,
        );
        let mut rig = Self {
            ui,
            renderer,
            messages: Vec::new(),
        };
        rig.redraw_requested();
        rig
    }

    /// What an application's `view` does after `update`, then the redraw request
    /// the runtime sends before every draw.
    fn rebuilt(self, root: Element<'static, Msg>) -> Self {
        let mut renderer = self.renderer;
        let ui = UserInterface::build(root, WINDOW, self.ui.into_cache(), &mut renderer);
        let mut rig = Self {
            ui,
            renderer,
            messages: self.messages,
        };
        rig.redraw_requested();
        rig
    }

    fn redraw_requested(&mut self) {
        let _ = self.ui.update(
            &[Event::Window(window::Event::RedrawRequested(
                iced_runtime::core::time::Instant::now(),
            ))],
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    fn enter(&mut self) {
        let _ = self.ui.update(
            &[iced_test::simulator::press_key(
                iced::keyboard::key::Named::Enter,
                None,
            )],
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    fn tab(&mut self) {
        let mut current: Box<dyn Operation<()>> = Box::new(focusable::focus_next::<()>());
        loop {
            self.ui
                .operate(&self.renderer, &mut operation::black_box(current.as_mut()));
            match current.finish() {
                Outcome::None | Outcome::Some(()) => return,
                Outcome::Chain(next) => current = next,
            }
        }
    }

    /// The ordered focus targets, each with the containers it sits in.
    fn scan(&mut self) -> Vec<Seen> {
        let mut scan = Scan::default();
        self.ui.operate(&self.renderer, &mut scan);
        scan.seen
    }

    fn unfocus_all(&mut self) {
        self.ui.operate(&self.renderer, &mut UnfocusAll);
    }

    fn focus_at(&mut self, at: usize) {
        self.ui
            .operate(&self.renderer, &mut FocusAt { at, seen: 0 });
    }

    fn targets(&mut self) -> Vec<Target> {
        #[derive(Default)]
        struct Collect(Vec<Target>);
        impl Operation for Collect {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn focusable(
                &mut self,
                _id: Option<&Id>,
                bounds: Rectangle,
                state: &mut dyn operation::Focusable,
            ) {
                self.0.push(Target {
                    bounds,
                    focused: state.is_focused(),
                });
            }
        }
        let mut c = Collect::default();
        self.ui.operate(&self.renderer, &mut c);
        c.0
    }

    /// The frame at scale 1.
    fn frame(&mut self) -> Vec<u8> {
        use iced::theme::Base;
        let theme = Theme::Light;
        let base = theme.base();
        self.ui.draw(
            &mut self.renderer,
            &theme,
            &iced_runtime::core::renderer::Style {
                text_color: base.text_color,
            },
            mouse::Cursor::Unavailable,
        );
        Headless::screenshot(
            &mut self.renderer,
            Size::new(WINDOW.width as u32, WINDOW.height as u32),
            1.0,
            base.background_color,
        )
    }
}

// ---- the prototype: what a tracker in `Inert` could do -------------------------------
//
// `Inert` is always present around the page, keeps its state across the
// application's rebuilds, and (RFC-119) already runs operations over its own
// content inside `update`. Everything below uses only what it can do there:
//
//   after each event   `observe`: survey the focus targets; if a control was just
//                      pressed, remember the survey and where focus was, and take
//                      focus off it (so no tree position can inherit it);
//   at RedrawRequested `resolve`: survey again; if anything is focused, the
//                      application did it, and wins (R-2); if the count is
//                      unchanged, give focus back; else apply R-1.
//
// The rig calls them where `Inert::update` would. The press is known because
// the control that was activated is the focused one, on the keyboard path.

/// One focus target and the containers it sits in, outermost first.
#[derive(Clone, Debug)]
struct Seen {
    focused: bool,
    scopes: Vec<usize>,
}

/// Lists the focus targets in Tab order. A container is numbered by the order
/// `traverse` meets it in, so a container that starts before a removal keeps
/// its number across it.
#[derive(Default)]
struct Scan {
    seen: Vec<Seen>,
    stack: Vec<usize>,
    next: usize,
}

impl Operation for Scan {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        self.stack.push(self.next);
        self.next += 1;
        operate(self);
        self.stack.pop();
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.seen.push(Seen {
            focused: state.is_focused(),
            scopes: self.stack.clone(),
        });
    }
}

struct UnfocusAll;

impl Operation for UnfocusAll {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        state.unfocus();
    }
}

struct FocusAt {
    at: usize,
    seen: usize,
}

impl Operation for FocusAt {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        if self.seen == self.at {
            state.focus();
        }
        self.seen += 1;
    }
}

/// Which candidate decides.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// 0.55.0: nothing is done.
    Today,
    /// (b) alone: a tracker that knows only that the focused control was
    /// activated. It cannot tell how many targets went, so it accepts only the
    /// case where exactly one did.
    Tracker,
    /// (a) + (b): the same, and the removing control also says how many
    /// targets go with it, before it and after it.
    Declared,
}

struct Pending {
    before: Vec<Seen>,
    at: usize,
    bef: usize,
    aft: usize,
}

#[derive(Default)]
struct Tracker {
    pending: Option<Pending>,
}

impl Tracker {
    /// After the event that pressed `declared`'s control.
    fn observe(&mut self, rig: &mut Rig, bef: usize, aft: usize) {
        let before = rig.scan();
        let Some(at) = before.iter().position(|s| s.focused) else {
            return;
        };
        rig.unfocus_all();
        self.pending = Some(Pending {
            before,
            at,
            bef,
            aft,
        });
    }

    /// At the `RedrawRequested` after the application's rebuild.
    fn resolve(&mut self, rig: &mut Rig, mode: Mode) {
        let Some(p) = self.pending.take() else {
            return;
        };
        let after = rig.scan();
        // R-2: if anything is focused, the application did it.
        if after.iter().any(|s| s.focused) {
            return;
        }
        // Nothing went: the control is still there, so focus returns to it.
        if after.len() == p.before.len() {
            rig.focus_at(p.at);
            return;
        }
        let span = match mode {
            Mode::Declared => p.at - p.bef..p.at + p.aft + 1,
            _ => p.at..p.at + 1,
        };
        if let Some(to) = group_first(&p.before, span, &after) {
            rig.focus_at(to);
        }
    }
}

/// R-1: the nearest survivor in the control's own group, the one after the
/// removed span if there is one, else the one before; else the next group out.
/// Gives up (focus nowhere) if the counts do not add up to exactly this removal.
fn group_first(before: &[Seen], span: std::ops::Range<usize>, after: &[Seen]) -> Option<usize> {
    let mut scopes = before[span.start.min(before.len() - 1)].scopes.clone();
    scopes.reverse();
    for g in scopes {
        let run_b: Vec<usize> = (0..before.len())
            .filter(|i| before[*i].scopes.contains(&g))
            .collect();
        // A group that sat wholly inside what went is not there to look in.
        if run_b.iter().all(|i| span.contains(i)) {
            continue;
        }
        let run_a: Vec<usize> = (0..after.len())
            .filter(|i| after[*i].scopes.contains(&g))
            .collect();
        let gone = run_b.iter().filter(|i| span.contains(i)).count();
        if run_a.len() + gone != run_b.len() {
            return None;
        }
        let first = run_b.iter().filter(|i| **i < span.start).count();
        if first < run_a.len() {
            return Some(run_a[first]);
        }
        if first > 0 {
            return Some(run_a[first - 1]);
        }
        // Nothing of this group is left: look further out.
    }
    None
}

// ---- the cases ---------------------------------------------------------------------

/// One case: a set of items, how to draw them, which control removes an item,
/// and what is left after it is pressed.
struct Case {
    name: &'static str,
    /// The items, in order (an id each).
    ids: &'static [u8],
    /// Which items' removal controls are pressed.
    pressed: &'static [u8],
    scene: fn(&[u8]) -> Element<'static, Msg>,
    /// What the removal control of `id` publishes.
    removal: fn(u8) -> Msg,
    /// The items left after `id`'s control is pressed.
    after: fn(&[u8], u8) -> Vec<u8>,
    /// What RFC-120 R-1 says should have focus, written by hand from the rule
    /// and the Tab order printed above the table, not from any mechanism.
    expected: fn(u8) -> &'static str,
    /// How many focus targets go with the pressed one, before it and after it
    /// in Tab order: what the removing widget itself would declare.
    declared: fn(u8) -> (usize, usize),
}

fn without(ids: &[u8], id: u8) -> Vec<u8> {
    ids.iter().copied().filter(|i| *i != id).collect()
}

fn chips(ids: &[u8]) -> Element<'static, Msg> {
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

fn notice_between(ids: &[u8]) -> Element<'static, Msg> {
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

fn toasts(ids: &[u8]) -> Element<'static, Msg> {
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
    render(AppLayout::new(body).toasts(toasts))
}

fn crumbs(ids: &[u8]) -> Element<'static, Msg> {
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
            snora::widget::app_breadcrumb(trail, &ON_CRUMB, LayoutDirection::Ltr),
            button::ghost(tokens(), "Footer", msg("footer")),
        ]
        .spacing(12),
    )
    .padding(24)
    .into()
}

fn list(ids: &[u8]) -> Element<'static, Msg> {
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

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "chip x (row of three)",
            ids: &[0, 1, 2],
            pressed: &[0, 1, 2],
            scene: chips,
            removal: |i| msg(format!("remove {i}")),
            after: without,
            expected: |i| ["toggle 1", "toggle 2", "remove 1"][usize::from(i)],
            declared: |_| (1, 0),
        },
        Case {
            name: "notice dismiss (between two controls)",
            ids: &[0],
            pressed: &[0],
            scene: notice_between,
            removal: |_| msg("notice dismiss"),
            after: without,
            expected: |_| "after",
            declared: |_| (1, 0),
        },
        Case {
            name: "toast x (stack of three)",
            ids: &[1, 2, 3],
            pressed: &[1, 2, 3],
            scene: toasts,
            removal: |i| msg(format!("toast {i}")),
            after: without,
            expected: |i| ["", "toast 2", "toast 1", "toast 2"][usize::from(i)],
            declared: |_| (0, 0),
        },
        Case {
            name: "ancestor crumb (trail of three)",
            ids: &[0, 1, 2],
            pressed: &[0, 1],
            scene: crumbs,
            removal: |i| msg(format!("crumb {i}")),
            // Pressing an ancestor makes it the current page: what is after it goes.
            after: |ids, id| {
                ids.iter()
                    .copied()
                    .take_while(|i| *i <= id)
                    .collect::<Vec<u8>>()
            },
            expected: |i| ["footer", "crumb 0"][usize::from(i)],
            declared: |i| (0, usize::from(i == 0)),
        },
        Case {
            name: "application list row's Delete (three rows)",
            ids: &[0, 1, 2],
            pressed: &[0, 1, 2],
            scene: list,
            removal: |i| msg(format!("delete {i}")),
            after: without,
            expected: |i| ["delete 1", "delete 2", "delete 1"][usize::from(i)],
            declared: |_| (0, 0),
        },
    ]
}

// ---- measuring --------------------------------------------------------------------

/// The message each focus target of `scene(ids)` publishes on Enter, by index.
fn names(scene: fn(&[u8]) -> Element<'static, Msg>, ids: &[u8]) -> Vec<String> {
    let n = Rig::new(scene(ids)).targets().len();
    (0..n)
        .map(|k| {
            let mut rig = Rig::new(scene(ids));
            for _ in 0..=k {
                rig.tab();
            }
            rig.enter();
            rig.messages
                .last()
                .map_or_else(|| "(nothing)".to_owned(), |m| m.0.clone())
        })
        .collect()
}

struct Row {
    case: &'static str,
    pressed: u8,
    focus: String,
    ring: String,
    enter: String,
    expected: &'static str,
}

fn nearest(targets: &[Target], at: iced::Point, names: &[String]) -> String {
    targets
        .iter()
        .zip(names)
        .min_by(|(a, _), (b, _)| {
            let da = a.bounds.center().distance(at);
            let db = b.bounds.center().distance(at);
            da.total_cmp(&db)
        })
        .map_or_else(|| "(no control)".to_owned(), |(_, n)| n.clone())
}

fn measure(case: &Case, pressed: u8, mode: Mode) -> Row {
    let removal = (case.removal)(pressed);
    let before_names = names(case.scene, case.ids);
    let index = before_names
        .iter()
        .position(|n| *n == removal.0)
        .unwrap_or_else(|| panic!("{}: no control publishes {removal:?}", case.name));

    // The press, from the keyboard.
    let mut rig = Rig::new((case.scene)(case.ids));
    for _ in 0..=index {
        rig.tab();
    }
    assert_eq!(
        rig.targets().iter().filter(|t| t.focused).count(),
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

    // After the event, as `Inert::update` would run it.
    let mut tracker = Tracker::default();
    let (bef, aft) = (case.declared)(pressed);
    if mode != Mode::Today {
        tracker.observe(&mut rig, bef, aft);
    }

    // The application's update removed what the press removed.
    let left = (case.after)(case.ids, pressed);
    let mut rig = rig.rebuilt((case.scene)(&left));
    if mode != Mode::Today {
        tracker.resolve(&mut rig, mode);
    }
    let targets = rig.targets();
    let after_names = names(case.scene, &left);
    let focused: Vec<&String> = targets
        .iter()
        .zip(&after_names)
        .filter(|(t, _)| t.focused)
        .map(|(_, n)| n)
        .collect();

    // Where a ring is drawn: this frame against the same view never focused.
    let ring_frame = rig.frame();
    let mut clean = Rig::new((case.scene)(&left));
    let clean_frame = clean.frame();
    let width = WINDOW.width as usize;
    let mut changed: Vec<(usize, usize)> = Vec::new();
    for (i, (a, b)) in ring_frame
        .as_chunks::<4>()
        .0
        .iter()
        .zip(clean_frame.as_chunks::<4>().0)
        .enumerate()
    {
        if a != b {
            changed.push((i % width, i / width));
        }
    }
    let ring = if changed.is_empty() {
        "no ring".to_owned()
    } else {
        let (sx, sy) = changed
            .iter()
            .fold((0usize, 0usize), |(x, y), (cx, cy)| (x + cx, y + cy));
        let at = iced::Point::new((sx / changed.len()) as f32, (sy / changed.len()) as f32);
        format!("ring at {}", nearest(&targets, at, &after_names))
    };

    // What one more Enter does.
    rig.enter();
    let enter = rig
        .messages
        .last()
        .map_or_else(|| "nothing".to_owned(), |m| m.0.clone());

    Row {
        case: case.name,
        pressed,
        focus: if focused.is_empty() {
            "nowhere".to_owned()
        } else {
            format!("{focused:?}")
        },
        ring,
        enter,
        expected: (case.expected)(pressed),
    }
}

/// Today's behaviour, for every case and every pressed item. Printed as a
/// table (`--nocapture`); asserts only the premises.
fn rows(mode: Mode) -> Vec<Row> {
    cases()
        .iter()
        .flat_map(|c| c.pressed.iter().map(move |p| measure(c, *p, mode)))
        .collect()
}

/// Today's behaviour, for every case and every pressed item. Printed as a
/// table (`--nocapture`); asserts only the premises.
#[test]
fn what_0_55_0_does_when_the_focused_control_goes_away() {
    println!("\n## focus after a removal ({})\n", backend());
    println!("| case | pressed | `find_focused` says focused | ring | one more Enter |");
    println!("|---|---|---|---|---|");
    for r in rows(Mode::Today) {
        println!(
            "| {} | {} | {} | {} | {} |",
            r.case, r.pressed, r.focus, r.ring, r.enter
        );
    }

    println!("\nTab order, before any removal:");
    for c in cases() {
        println!("- {}: {:?}", c.name, names(c.scene, c.ids));
    }
}

/// The removing control's declaration, checked against what really went: the
/// prototype may only be fed what a widget could know by construction.
#[test]
fn the_declared_spans_are_what_really_goes() {
    for c in cases() {
        let before = names(c.scene, c.ids);
        for p in c.pressed {
            let at = before
                .iter()
                .position(|n| *n == (c.removal)(*p).0)
                .expect("the removal control is in the order");
            let left = (c.after)(c.ids, *p);
            let after = names(c.scene, &left);
            let gone: Vec<usize> = (0..before.len())
                .filter(|i| !after.contains(&before[*i]))
                .collect();
            let (bef, aft) = (c.declared)(*p);
            let declared: Vec<usize> = (at - bef..=at + aft).collect();
            assert_eq!(
                declared, gone,
                "{} pressing {p}: {before:?} -> {after:?}",
                c.name
            );
        }
    }
}

/// The two prototypes against R-1, for every case. The tracker alone is
/// printed, because what it cannot do is the finding; the declared one is
/// asserted, because it is the claim.
#[test]
fn the_candidates_against_r_1() {
    for (title, mode) in [
        (
            "(b) a tracker alone: only 'the focused control went' is known",
            Mode::Tracker,
        ),
        (
            "(a)+(b) the tracker, and the removing control says how many go with it",
            Mode::Declared,
        ),
    ] {
        println!("\n## {title} ({})\n", backend());
        println!("| case | pressed | R-1 says | lands on | ring | match |");
        println!("|---|---|---|---|---|---|");
        let mut misses = Vec::new();
        for r in rows(mode) {
            let ok = r.enter == r.expected && r.ring == format!("ring at {}", r.expected);
            if !ok {
                misses.push(format!("{} pressing {}", r.case, r.pressed));
            }
            println!(
                "| {} | {} | {} | {} | {} | {} |",
                r.case,
                r.pressed,
                r.expected,
                r.enter,
                r.ring,
                if ok { "yes" } else { "**no**" }
            );
        }
        if mode == Mode::Declared {
            assert!(misses.is_empty(), "R-1 not met for: {misses:?}");
        }
    }
}

/// A removal that is not the focused control's: the application removes
/// something else (a click elsewhere, a toast expiring) while focus sits on a
/// control that stays. R-1 does not cover this, since nothing focused went
/// away, but the cause is the same positional state, so it is measured too.
#[test]
fn what_0_55_0_does_when_something_else_is_removed() {
    struct Elsewhere {
        name: &'static str,
        scene: fn(&[u8]) -> Element<'static, Msg>,
        ids: &'static [u8],
        focus: &'static str,
        remove: u8,
    }
    let elsewhere = [
        Elsewhere {
            name: "list: row 0 removed",
            scene: list,
            ids: &[0, 1, 2],
            focus: "delete 1",
            remove: 0,
        },
        Elsewhere {
            name: "list: row 0 removed",
            scene: list,
            ids: &[0, 1, 2],
            focus: "delete 2",
            remove: 0,
        },
        Elsewhere {
            name: "list: row 2 removed",
            scene: list,
            ids: &[0, 1, 2],
            focus: "delete 0",
            remove: 2,
        },
        Elsewhere {
            name: "chips: chip 0 removed",
            scene: chips,
            ids: &[0, 1, 2],
            focus: "remove 2",
            remove: 0,
        },
        Elsewhere {
            name: "toasts: toast 3 expires",
            scene: toasts,
            ids: &[1, 2, 3],
            focus: "toast 1",
            remove: 3,
        },
        Elsewhere {
            name: "toasts: toast 3 expires",
            scene: toasts,
            ids: &[1, 2, 3],
            focus: "toast 2",
            remove: 3,
        },
    ];
    println!(
        "\n## focus on a control that stays, something else removed ({})\n",
        backend()
    );
    println!("| removed | focus was on | still focused after |");
    println!("|---|---|---|");
    for e in elsewhere {
        let before = names(e.scene, e.ids);
        let at = before
            .iter()
            .position(|n| n == e.focus)
            .expect("a control of that name");
        let mut rig = Rig::new((e.scene)(e.ids));
        for _ in 0..=at {
            rig.tab();
        }
        let left = without(e.ids, e.remove);
        let mut rig = rig.rebuilt((e.scene)(&left));
        let landed = if rig.scan().iter().any(|s| s.focused) {
            rig.enter();
            rig.messages
                .last()
                .map_or("nothing".to_owned(), |m| m.0.clone())
        } else {
            "nowhere".to_owned()
        };
        println!("| {} | {} | {} |", e.name, e.focus, landed);
    }
}

/// R-2 for the prototype: if the application has focused something by the time
/// the tracker resolves, the tracker does nothing.
#[test]
fn the_applications_own_focus_wins() {
    let case = &cases()[4]; // the application's list
    let before = names(case.scene, case.ids);
    let at = before.iter().position(|n| n == "delete 1").unwrap();
    let mut rig = Rig::new((case.scene)(case.ids));
    for _ in 0..=at {
        rig.tab();
    }
    let mut tracker = Tracker::default();
    tracker.observe(&mut rig, 0, 0);
    let left = without(case.ids, 1);
    let mut rig = rig.rebuilt((case.scene)(&left));
    // The application's focus task ran first: the first control.
    rig.focus_at(0);
    tracker.resolve(&mut rig, Mode::Declared);
    // Not through Enter: with two targets focused the first captures the key
    // and the second never sees it, so an Enter check cannot tell the two apart.
    let focused: Vec<usize> = rig
        .scan()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.focused)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(focused, vec![0], "only the application's choice has focus");
}
