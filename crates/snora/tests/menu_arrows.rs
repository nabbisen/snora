//! RFC-123: ↓ ↑ Home End in snora's menus, on the header's dropdown (an overlay) and the context
//! menu (a layer of the stack), through the real engine and the runtime's own interface.
//! Prints the table (`--nocapture`) and asserts each row. The rules are in
//! `snora-interaction/src/menu_keys.rs`.
//!
//! **After a key the root captured, redraw before reading pixels** (`shot()`): see `Rig::frame`.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use common::removal::{Rig, key_event};
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named::{ArrowDown, ArrowLeft, ArrowUp, End, Home};
use iced::widget::{column, text, text_input};
use iced::{Element, Length, Point, Rectangle, Size, Theme};
use iced_runtime::core::event::Status;
use iced_runtime::core::layout::{self, Layout};
use iced_runtime::core::widget::{Id, Operation, Tree, tree};
use iced_runtime::core::{Clipboard, Event, Shell, Widget, keyboard, mouse, renderer};
use snora::{AppLayout, LayoutDirection, Menu, MenuAction, MenuItem, Toast, ToastIntent};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Menu(MenuAction<u8, u8>),
    Close,
    Page,
    Typed(String),
    Dismiss(u64),
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;
const LTR: LayoutDirection = LayoutDirection::Ltr;

fn menu(items: u8) -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".into(),
        icon: None,
        items: (0..items)
            .map(|i| MenuItem {
                menu_id: 0,
                id: i,
                label: format!("Item {i}"),
                icon: None,
            })
            .collect(),
    }
}

fn item_pressed(id: u8) -> Msg {
    Msg::Menu(MenuAction::MenuItemPressed {
        menu_id: 0,
        menu_item_id: id,
    })
}

/// A widget that counts the key presses it is offered, and the `operate` calls it gets.
struct Tap {
    keys: Arc<AtomicUsize>,
    operates: Arc<AtomicUsize>,
}

impl Widget<Msg, Theme, iced::Renderer> for Tap {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(10.0), Length::Fixed(10.0))
    }
    fn layout(&mut self, _t: &mut Tree, _r: &iced::Renderer, _l: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(10.0, 10.0))
    }
    fn operate(
        &mut self,
        _t: &mut Tree,
        _l: Layout<'_>,
        _r: &iced::Renderer,
        _o: &mut dyn Operation,
    ) {
        self.operates.fetch_add(1, Ordering::SeqCst);
    }
    fn update(
        &mut self,
        _t: &mut Tree,
        event: &Event,
        _l: Layout<'_>,
        _c: mouse::Cursor,
        _r: &iced::Renderer,
        _cb: &mut dyn Clipboard,
        _s: &mut Shell<'_, Msg>,
        _v: &Rectangle,
    ) {
        if matches!(event, Event::Keyboard(keyboard::Event::KeyPressed { .. })) {
            self.keys.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn draw(
        &self,
        _t: &Tree,
        _r: &mut iced::Renderer,
        _th: &Theme,
        _s: &renderer::Style,
        _l: Layout<'_>,
        _c: mouse::Cursor,
        _v: &Rectangle,
    ) {
    }
}

#[derive(Clone, Default)]
struct Counts {
    keys: Arc<AtomicUsize>,
    operates: Arc<AtomicUsize>,
}

fn page(c: &Counts) -> Element<'static, Msg> {
    column![
        snora::control(text("page")).on_press(Msg::Page),
        text_input("field", "").on_input(Msg::Typed),
        Element::new(Tap {
            keys: c.keys.clone(),
            operates: c.operates.clone(),
        }),
    ]
    .into()
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Host {
    /// `app_header`'s dropdown, an overlay (RFC-124).
    Dropdown,
    /// `render_menu` as `AppLayout::context_menu`: inline, a layer of the stack.
    Context,
}

struct Scene {
    host: Host,
    open: bool,
    /// `on_close_menus` set (the backdrop; the page is covered).
    close: bool,
    items: u8,
    dir: LayoutDirection,
    toast: bool,
    dialog: bool,
    /// A block above the dropdown's trigger whose height an `F5` key press changes, so that
    /// an event moves the trigger (the dropdown host only).
    grow: Option<Arc<AtomicBool>>,
}

/// A block of no height until it sees `F5`, then 300 px: it asks for a new layout when it
/// does, as a widget that changed its size would.
struct Grow(Arc<AtomicBool>);

impl Widget<Msg, Theme, iced::Renderer> for Grow {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(10.0), Length::Shrink)
    }
    fn layout(&mut self, _t: &mut Tree, _r: &iced::Renderer, _l: &layout::Limits) -> layout::Node {
        let height = if self.0.load(Ordering::SeqCst) {
            300.0
        } else {
            0.0
        };
        layout::Node::new(Size::new(10.0, height))
    }
    fn update(
        &mut self,
        _t: &mut Tree,
        event: &Event,
        _l: Layout<'_>,
        _c: mouse::Cursor,
        _r: &iced::Renderer,
        _cb: &mut dyn Clipboard,
        shell: &mut Shell<'_, Msg>,
        _v: &Rectangle,
    ) {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::F5),
            ..
        }) = event
        {
            self.0.store(true, Ordering::SeqCst);
            shell.invalidate_layout();
        }
    }
    fn draw(
        &self,
        _t: &Tree,
        _r: &mut iced::Renderer,
        _th: &Theme,
        _s: &renderer::Style,
        _l: Layout<'_>,
        _c: mouse::Cursor,
        _v: &Rectangle,
    ) {
    }
}

impl Scene {
    fn new(host: Host, close: bool) -> Self {
        Self {
            host,
            open: true,
            close,
            items: 3,
            dir: LTR,
            toast: false,
            dialog: false,
            grow: None,
        }
    }
    fn build(&self, c: &Counts) -> Element<'static, Msg> {
        let mut l = AppLayout::new(page(c)).direction(self.dir);
        match self.host {
            Host::Dropdown => {
                let active = self.open.then_some(&0u8);
                let header = snora::widget::app_header(
                    "App",
                    vec![menu(self.items)],
                    &ON_MENU,
                    active,
                    None,
                    self.dir,
                );
                l = l.header(match &self.grow {
                    Some(flag) => column![Element::new(Grow(flag.clone())), header].into(),
                    None => header,
                });
            }
            Host::Context => {
                l = l.context_menu(snora::widget::render_menu(
                    menu(self.items),
                    &ON_MENU,
                    self.open,
                ));
            }
        }
        if self.close {
            l = l.on_close_menus(Msg::Close);
        }
        if self.toast {
            l = l.toasts(vec![Toast::new(
                7,
                ToastIntent::Info,
                "T",
                "b",
                Msg::Dismiss(7),
            )]);
        }
        if self.dialog {
            l = l
                .dialog(snora::Dialog::new(
                    snora::control(text("OK")).on_press(Msg::Page).into(),
                ))
                .on_close_modals(Msg::Close);
        }
        snora::render(l)
    }
    fn rig(&self) -> (Rig<Msg>, Counts) {
        let c = Counts::default();
        (Rig::new(self.build(&c)), c)
    }
}

/// For each focus target in order: is it a text field.
fn kinds(rig: &mut Rig<Msg>) -> Vec<bool> {
    struct K(Arc<Mutex<Vec<bool>>>, bool);
    impl Operation for K {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text_input(
            &mut self,
            _i: Option<&Id>,
            _b: Rectangle,
            _s: &mut dyn iced_runtime::core::widget::operation::TextInput,
        ) {
            self.1 = true;
        }
        fn focusable(
            &mut self,
            _i: Option<&Id>,
            _b: Rectangle,
            _s: &mut dyn iced_runtime::core::widget::operation::Focusable,
        ) {
            self.0.lock().unwrap().push(self.1);
            self.1 = false;
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(K(out.clone(), false)));
    std::mem::take(&mut *out.lock().unwrap())
}

fn focused(rig: &mut Rig<Msg>) -> Vec<usize> {
    rig.focused()
}

/// The focused item among the last `n` targets (the menu's items come last for a
/// dropdown, an overlay, which `UserInterface::operate` visits after the tree).
fn item_of(rig: &mut Rig<Msg>, items: usize) -> Option<usize> {
    let n = rig.targets().len();
    focused(rig).first().and_then(|f| f.checked_sub(n - items))
}

// ---------------------------------------------------------------------------
// The premise: what each scene holds
// ---------------------------------------------------------------------------

#[test]
fn the_premise_what_is_reachable_in_each_scene() {
    println!("\n## focus targets, in the order the runtime's Tab visits them\n");
    println!("| scene | targets | text field at | focused |");
    println!("|---|---|---|---|");
    for (name, host, close) in [
        ("header dropdown, close message", Host::Dropdown, true),
        (
            "header dropdown, **no** close message",
            Host::Dropdown,
            false,
        ),
        ("context menu, close message", Host::Context, true),
        ("context menu, **no** close message", Host::Context, false),
    ] {
        let (mut rig, _) = Scene::new(host, close).rig();
        let k = kinds(&mut rig);
        println!(
            "| {name} | {} | {:?} | {:?} |",
            k.len(),
            k.iter().position(|t| *t),
            rig.focused()
        );
    }
}

// ---------------------------------------------------------------------------
// Rules 1 and 2 (and 7): the dropdown with a close message, then the context menu
// ---------------------------------------------------------------------------

fn walk(host: Host, close: bool) {
    let (mut rig, _) = Scene::new(host, close).rig();
    let n = rig.targets().len();
    let trig = usize::from(host == Host::Context && close); // the trigger leads, when it is reachable
    assert_eq!(
        focused(&mut rig),
        Vec::<usize>::new(),
        "opening moved focus"
    );
    let mut seen = vec![];
    for (label, key) in [
        ("↓ (from nothing: first)", ArrowDown),
        ("↓", ArrowDown),
        ("↓", ArrowDown),
        ("↓ (wraps to first)", ArrowDown),
        ("↑ (wraps to last)", ArrowUp),
        ("Home", Home),
        ("End", End),
    ] {
        let status = rig.named(key, false);
        seen.push((label, item_of(&mut rig, 3), status));
    }
    println!("\n### {host:?}, close message {close}: {n} targets\n");
    for (label, i, s) in &seen {
        println!("- {label}: item {i:?} {s:?}");
    }
    let got: Vec<_> = seen.iter().map(|s| s.1).collect();
    assert_eq!(
        got,
        vec![
            Some(0),
            Some(1),
            Some(2),
            Some(0),
            Some(2),
            Some(0),
            Some(2)
        ]
    );
    assert!(
        seen.iter().all(|s| s.2 == Status::Captured),
        "an arrow was not captured"
    );
    let _ = trig;
}

#[test]
fn rules_1_and_2_the_dropdown_with_a_close_message() {
    walk(Host::Dropdown, true);
}

#[test]
fn rules_1_and_2_the_context_menu_with_a_close_message() {
    walk(Host::Context, true);
}

#[test]
fn rules_1_and_2_the_dropdown_without_a_close_message() {
    // From nothing focused anywhere: the page is live, but nothing has focus.
    walk(Host::Dropdown, false);
}

#[test]
fn rules_1_and_2_the_context_menu_without_a_close_message() {
    walk(Host::Context, false);
}

/// ↑ from nothing is the last, Home the first, End the last.
#[test]
fn rule_3_from_nothing_each_key_lands_where_it_says() {
    for host in [Host::Dropdown, Host::Context] {
        for (name, key, want) in [
            ("↓", ArrowDown, 0),
            ("↑", ArrowUp, 2),
            ("Home", Home, 0),
            ("End", End, 2),
        ] {
            let (mut rig, _) = Scene::new(host, true).rig();
            assert_eq!(focused(&mut rig), Vec::<usize>::new());
            rig.named(key, false);
            assert_eq!(item_of(&mut rig, 3), Some(want), "{host:?}: {name}");
        }
    }
}

// ---------------------------------------------------------------------------
// Rule 3: opening by the pointer and by Enter, nothing focused before the key
// ---------------------------------------------------------------------------

#[test]
fn rule_3_opened_by_the_pointer_or_by_enter_nothing_is_focused_before_the_key() {
    let closed = Scene {
        open: false,
        ..Scene::new(Host::Dropdown, true)
    };
    let open = Scene::new(Host::Dropdown, true);

    // By the pointer.
    let c = Counts::default();
    let mut rig = Rig::new(closed.build(&c));
    let trigger = rig.targets()[0].bounds.center();
    rig.click_at(trigger);
    assert!(
        rig.messages
            .contains(&Msg::Menu(MenuAction::MenuPressed(0)))
    );
    let mut rig = rig.rebuilt(open.build(&c));
    println!("\nopened by the pointer: focused {:?}", focused(&mut rig));
    assert_eq!(focused(&mut rig), Vec::<usize>::new());
    rig.named(ArrowDown, false);
    assert_eq!(item_of(&mut rig, 3), Some(0), "↓ after the pointer");

    // By Enter on the trigger.
    let mut rig = Rig::new(closed.build(&c));
    rig.focus_at(0);
    rig.enter();
    assert!(
        rig.messages
            .contains(&Msg::Menu(MenuAction::MenuPressed(0)))
    );
    let mut rig = rig.rebuilt(open.build(&c));
    println!("opened by Enter: focused {:?}", focused(&mut rig));
    assert_eq!(focused(&mut rig), Vec::<usize>::new());
    rig.named(ArrowUp, false);
    assert_eq!(item_of(&mut rig, 3), Some(2), "↑ after Enter");
}

// ---------------------------------------------------------------------------
// Rule 4
// ---------------------------------------------------------------------------

/// A toast's × focused: it keeps the focus, the menu does not move.
#[test]
fn rule_4_a_toasts_close_control_keeps_its_focus() {
    let (mut rig, c) = Scene {
        toast: true,
        ..Scene::new(Host::Dropdown, true)
    }
    .rig();
    let n = rig.targets().len();
    let toast = n - 4; // the toast's ×, then the 3 items
    rig.focus_at(toast);
    assert_eq!(focused(&mut rig), vec![toast]);
    for key in [ArrowDown, ArrowUp, Home, End] {
        let status = rig.named(key, false);
        println!(
            "toast × focused, {key:?}: focused {:?} {status:?}",
            focused(&mut rig)
        );
        assert_eq!(focused(&mut rig), vec![toast], "{key:?} moved focus");
    }
    let _ = c;
}

/// 3.2: without a close message, from the focused trigger, ↓ goes to the first item and the
/// trigger loses focus (one focus, not two).
#[test]
fn rule_4_and_3_2_from_the_trigger_the_arrow_enters_the_menu() {
    for (key, want) in [(ArrowDown, 0), (ArrowUp, 2), (Home, 0), (End, 2)] {
        let (mut rig, c) = Scene::new(Host::Dropdown, false).rig();
        rig.focus_at(0); // the trigger
        assert_eq!(focused(&mut rig), vec![0]);
        let status = rig.named(key, false);
        println!(
            "dropdown, no close message, trigger focused, {key:?}: focused {:?} {status:?} (key reached the page {} times)",
            focused(&mut rig),
            c.keys.load(Ordering::SeqCst)
        );
        let n = rig.targets().len();
        assert_eq!(focused(&mut rig), vec![n - 3 + want], "{key:?}");
        assert_eq!(status, Status::Captured);
        assert_eq!(c.keys.load(Ordering::SeqCst), 0, "the page saw the key");
    }
    // The context menu, trigger focused (inline, root stage only).
    let (mut rig, _) = Scene::new(Host::Context, false).rig();
    let trigger = kinds(&mut rig).len() - 4; // page control, field, trigger, 3 items
    rig.focus_at(trigger);
    let status = rig.named(ArrowDown, false);
    println!(
        "context menu, no close message, trigger focused, ↓: {:?} {status:?}",
        focused(&mut rig)
    );
    assert_eq!(focused(&mut rig), vec![trigger + 1]);
}

/// A focused page text field keeps it, a focused page control keeps it.
#[test]
fn rule_4_a_focused_field_or_page_control_keeps_its_arrows() {
    for host in [Host::Dropdown, Host::Context] {
        for (what, which) in [("page control", 0usize), ("text field", 1)] {
            let (mut rig, c) = Scene::new(host, false).rig();
            let k = kinds(&mut rig);
            let field = k.iter().position(|t| *t).expect("a text field");
            let at = if which == 0 { field - 1 } else { field };
            rig.focus_at(at);
            assert_eq!(focused(&mut rig), vec![at]);
            for key in [ArrowDown, ArrowUp, Home, End] {
                let status = rig.named(key, false);
                println!(
                    "{host:?}, {what} focused, {key:?}: focused {:?} {status:?}, page offered the key {} time(s)",
                    focused(&mut rig),
                    c.keys.load(Ordering::SeqCst)
                );
                assert_eq!(
                    focused(&mut rig),
                    vec![at],
                    "{host:?} {what}: {key:?} moved"
                );
            }
        }
    }
}

/// No stale view: the application focuses a page text field **with a task** between two
/// events, and the next ↓ does not take it.
#[test]
fn rule_4_a_task_that_focuses_the_field_is_seen_by_the_next_key() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene::new(host, false).rig();
        // A key first, so an earlier frame has had nothing focused...
        rig.redraw();
        assert_eq!(focused(&mut rig), Vec::<usize>::new());
        // ...then the task, with no redraw and no rebuild between it and the key.
        let field = kinds(&mut rig).iter().position(|t| *t).unwrap();
        rig.focus_at(field);
        rig.named(ArrowDown, false);
        println!(
            "{host:?}: task focused the field, then ↓: focused {:?}",
            focused(&mut rig)
        );
        assert_eq!(focused(&mut rig), vec![field], "{host:?}: a stale view");
    }
}

/// A click on the field and a ↓ **in one batch**. iced offers a batch to the overlay before the
/// tree, but the decision is the root's alone: the click has focused the field when the key
/// reaches it. **One focus straight after the batch**, with no redraw and no repair.
#[test]
fn a_click_on_the_field_and_a_down_in_one_batch_leave_one_focus() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene::new(host, false).rig();
        let k = kinds(&mut rig);
        let field = k.iter().position(|t| *t).unwrap();
        let at = rig.targets()[field].bounds.center();
        rig.set_cursor(at);
        let events = [
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            key_event(ArrowDown, false, Modifiers::empty()),
        ];
        let statuses = rig.batch(&events);
        let at_once = focused(&mut rig);
        rig.redraw();
        let after_redraw = focused(&mut rig);
        println!(
            "{host:?}: [click on the field, ↓] in one batch: focused {at_once:?} straight after, {after_redraw:?} after the redraw (field is {field}), statuses {statuses:?}"
        );
        assert_eq!(
            at_once,
            vec![field],
            "{host:?}: not one focus straight after"
        );
        assert_eq!(after_redraw, vec![field], "{host:?}");
        assert_eq!(
            statuses[2],
            Status::Ignored,
            "{host:?}: the field's key was taken"
        );
    }
}

/// Two arrows in one batch move twice.
#[test]
fn two_arrows_in_one_batch_move_twice() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene::new(host, true).rig();
        let statuses = rig.batch(&[
            key_event(ArrowDown, false, Modifiers::empty()),
            key_event(ArrowDown, false, Modifiers::empty()),
        ]);
        assert_eq!(item_of(&mut rig, 3), Some(1), "{host:?}");
        assert_eq!(
            statuses,
            vec![Status::Captured, Status::Captured],
            "{host:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Scrolling into view, held keys, Enter, modifiers, dialog, RTL
// ---------------------------------------------------------------------------

fn changed_box(shown: &[u8], plain: &[u8]) -> Option<(Point, usize)> {
    let (mut n, mut sx, mut sy) = (0usize, 0usize, 0usize);
    let (mut top, mut bottom) = (usize::MAX, 0usize);
    for (i, (a, b)) in shown
        .as_chunks::<4>()
        .0
        .iter()
        .zip(plain.as_chunks::<4>().0)
        .enumerate()
    {
        if a != b {
            let (x, y) = (i % 800, i / 800);
            n += 1;
            sx += x;
            sy += y;
            top = top.min(y);
            bottom = bottom.max(y);
        }
    }
    (n > 0).then(|| {
        (
            Point::new((sx / n) as f32, (sy / n) as f32),
            bottom - top + 1,
        )
    })
}

/// A frame as the window loop draws one: the redraw request first. (A captured key
/// leaves no overlay for `draw` to draw until the next `update` sets it: the runtime
/// always sends that redraw before a draw.)
fn shot(rig: &mut Rig<Msg>) -> Vec<u8> {
    rig.redraw();
    rig.frame()
}

/// 40 items in 600 px: End, then ↓ (wraps to the first), each item focused is in view,
/// the whole ring, by pixels.
#[test]
fn forty_items_end_then_down_wraps_and_each_focused_item_is_in_view() {
    const ITEMS: u8 = 40;
    let (mut rig, _) = Scene {
        items: ITEMS,
        ..Scene::new(Host::Dropdown, true)
    }
    .rig();
    let ring_height = {
        // Item 0's ring, not near an edge.
        rig.named(Home, false);
        let shown = shot(&mut rig);
        rig.focus_at(usize::MAX);
        let plain = shot(&mut rig);
        rig.focus_at(0);
        let b = changed_box(&shown, &plain).expect("item 0 has a ring");
        b.1
    };
    println!("\n## 40 items: End, ↓ (wrap), then ↑ ↑ and a run of ↓\n");
    let check = |rig: &mut Rig<Msg>, label: &str, want: usize| {
        // The frame as the key left it, before any operation (an operation on the overlay
        // brings the focused item into view again, which would hide a walk that did not).
        let as_left = shot(rig);
        let at = focused(rig)[0];
        assert_eq!(at, want, "{label}: item {at}, wanted {want}");
        let shown = shot(rig);
        assert!(
            as_left == shown,
            "{label}: the key left the focused item where an operation then had to move it"
        );
        rig.focus_at(usize::MAX);
        let plain = shot(rig);
        rig.focus_at(at);
        let (centre, height) = changed_box(&shown, &plain)
            .unwrap_or_else(|| panic!("{label}: item {at} is focused and no ring is drawn"));
        assert!(
            height.abs_diff(ring_height) <= 1,
            "{label}: ring {height} px tall, item 0's is {ring_height}"
        );
        assert!(
            (40.0..600.0).contains(&centre.y),
            "{label}: ring at y = {:.0}",
            centre.y
        );
        println!("- {label}: item {at}, ring at y = {:.0}", centre.y);
    };
    let last = usize::from(ITEMS) - 1;
    rig.named(End, false);
    check(&mut rig, "End", last);
    rig.named(ArrowDown, false);
    check(&mut rig, "↓ (wraps to the first)", 0);
    rig.named(ArrowUp, false);
    check(&mut rig, "↑ (wraps to the last)", last);
    for k in 0..usize::from(ITEMS) {
        rig.named(ArrowDown, true);
        if k % 13 == 0 {
            let want = k % usize::from(ITEMS);
            check(&mut rig, &format!("held ↓ #{k}"), want);
        }
    }
    // Enter presses the focused one.
    rig.messages.clear();
    rig.enter();
    let pressed = focused(&mut rig)[0] as u8;
    assert_eq!(rig.messages, vec![item_pressed(pressed)]);
}

#[test]
fn a_held_arrow_moves_each_time_and_enter_presses_once() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene::new(host, true).rig();
        rig.named(ArrowDown, false);
        for want in [1, 2, 0, 1] {
            let status = rig.named(ArrowDown, true);
            assert_eq!(item_of(&mut rig, 3), Some(want), "{host:?}: held ↓");
            assert_eq!(status, Status::Captured);
        }
        rig.messages.clear();
        rig.enter();
        assert_eq!(rig.messages, vec![item_pressed(1)], "{host:?}: Enter");
    }
}

/// Status `Captured` (what `keyboard::listen()` would not receive), and whether the
/// base tree is offered a key the overlay captured.
#[test]
fn a_captured_arrow_is_not_offered_to_the_base_tree() {
    for (host, close) in [
        (Host::Dropdown, false),
        (Host::Dropdown, true),
        (Host::Context, false),
        (Host::Context, true),
    ] {
        let (mut rig, c) = Scene::new(host, close).rig();
        let before = c.keys.load(Ordering::SeqCst);
        let status = rig.named(ArrowDown, false);
        let after = c.keys.load(Ordering::SeqCst);
        // A key that is not ours goes through to the tree.
        let other = rig.named(ArrowLeft, false);
        let after_left = c.keys.load(Ordering::SeqCst);
        println!(
            "{host:?}, close {close}: ↓ {status:?}, the page was offered it {} time(s); ← {other:?}, the page was offered it {} time(s)",
            after - before,
            after_left - after
        );
        assert_eq!(status, Status::Captured);
        assert_eq!(after - before, 0, "the page was offered a captured key");
        assert_eq!(other, Status::Ignored, "← was captured");
        if !close {
            // (With a close message the page is covered and offered nothing at all.)
            assert_eq!(after_left - after, 1, "← did not reach the tree");
        }
    }
}

#[test]
fn shift_and_ctrl_are_not_answered() {
    for host in [Host::Dropdown, Host::Context] {
        for (name, key, mods) in [
            ("Shift+↓", ArrowDown, Modifiers::SHIFT),
            ("Ctrl+Home", Home, Modifiers::CTRL),
            ("Alt+↓", ArrowDown, Modifiers::ALT),
        ] {
            let (mut rig, _) = Scene::new(host, true).rig();
            let status = rig.batch(&[key_event(key, false, mods)])[0];
            assert_eq!(status, Status::Ignored, "{host:?} {name}");
            assert_eq!(focused(&mut rig), Vec::<usize>::new(), "{host:?} {name}");
        }
    }
}

/// A dialog over an open menu: silent, as measured in the first Unit 0.
#[test]
fn a_dialog_over_an_open_menu_is_silent() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene {
            dialog: true,
            ..Scene::new(host, true)
        }
        .rig();
        let before = focused(&mut rig);
        let n = rig.targets().len();
        let status = rig.named(ArrowDown, false);
        println!(
            "{host:?} under a dialog: ↓ {status:?}, {n} targets, focused {before:?} -> {:?}",
            focused(&mut rig)
        );
        assert_eq!(focused(&mut rig), before);
    }
}

#[test]
fn rtl_does_not_mirror_the_arrows() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene {
            dir: LayoutDirection::Rtl,
            ..Scene::new(host, true)
        }
        .rig();
        rig.named(ArrowDown, false);
        assert_eq!(item_of(&mut rig, 3), Some(0));
        rig.named(ArrowDown, false);
        assert_eq!(item_of(&mut rig, 3), Some(1));
    }
}

/// A closed menu answers nothing, and an arrow with no menu is not captured.
#[test]
fn a_closed_menu_answers_nothing() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene {
            open: false,
            ..Scene::new(host, false)
        }
        .rig();
        let status = rig.named(ArrowDown, false);
        assert_eq!(
            status,
            Status::Ignored,
            "{host:?}: a closed menu captured ↓"
        );
        assert_eq!(focused(&mut rig), Vec::<usize>::new());
    }
}

// ---------------------------------------------------------------------------
// The cost
// ---------------------------------------------------------------------------

/// How many times the base tree is walked (`operate`) for the work of one `update`.
#[test]
fn measured_how_often_the_base_tree_is_surveyed() {
    println!("\n## `operate` calls reaching a widget in the page, per call\n");
    for (name, host, open, close) in [
        ("dropdown open, close message", Host::Dropdown, true, true),
        (
            "dropdown open, no close message",
            Host::Dropdown,
            true,
            false,
        ),
        ("dropdown closed", Host::Dropdown, false, false),
        ("context menu open", Host::Context, true, false),
    ] {
        let (mut rig, c) = Scene {
            open,
            ..Scene::new(host, close)
        }
        .rig();
        let base = c.operates.load(Ordering::SeqCst);
        rig.redraw();
        let redraw = c.operates.load(Ordering::SeqCst) - base;
        let before = c.operates.load(Ordering::SeqCst);
        rig.named(ArrowLeft, false);
        let other_key = c.operates.load(Ordering::SeqCst) - before;
        let before = c.operates.load(Ordering::SeqCst);
        rig.named(ArrowDown, false);
        let arrow = c.operates.load(Ordering::SeqCst) - before;
        let before = c.operates.load(Ordering::SeqCst);
        let _ = rig.frame();
        let draw = c.operates.load(Ordering::SeqCst) - before;
        println!(
            "- {name}: building the interface {base}; one redraw update {redraw}; a key that is not ours {other_key}; ↓ {arrow}; a draw {draw}"
        );
        assert_eq!(redraw, 0, "{name}: a redraw walks the page");
        assert_eq!(
            other_key, 0,
            "{name}: a key that is not ours walks the page"
        );
        assert_eq!(draw, 0, "{name}: a draw walks the page");
        // One walk to decide; a second to move, only if the key is answered.
        let answered = open;
        assert_eq!(arrow, if answered { 2 } else { 1 }, "{name}: ↓");
    }
    // An arrow that is not answered (a page field has focus): the deciding walk alone.
    let (mut rig, c) = Scene::new(Host::Dropdown, false).rig();
    let field = kinds(&mut rig).iter().position(|t| *t).unwrap();
    rig.focus_at(field);
    let before = c.operates.load(Ordering::SeqCst);
    rig.named(ArrowDown, false);
    let walks = c.operates.load(Ordering::SeqCst) - before;
    println!("- a page field focused, ↓ (not answered): {walks}");
    assert_eq!(walks, 1);
}

/// The design path draws the same dropdown, so it carries the same marks.
#[cfg(feature = "design")]
#[test]
fn the_design_headers_dropdown_answers_the_arrows_too() {
    let tokens: &'static snora::design::Tokens =
        Box::leak(Box::new(snora::design::Tokens::light()));
    let c = Counts::default();
    let header = snora::design::widget::app_header(
        tokens,
        "App",
        vec![menu(3)],
        &ON_MENU,
        Some(&0),
        None,
        LTR,
    );
    let l = AppLayout::new(page(&c))
        .header(header)
        .on_close_menus(Msg::Close);
    let mut rig = Rig::new(snora::design::render(l, tokens));
    let status = rig.named(ArrowUp, false);
    assert_eq!(status, Status::Captured);
    assert_eq!(item_of(&mut rig, 3), Some(2));
}

/// Keep `Id` and `keyboard` imports honest on every feature set.
#[allow(dead_code)]
fn _imports(_: Option<Id>) {}

// ---------------------------------------------------------------------------
// Unit 0b: the other operations see each item once; the stored layout is current
// ---------------------------------------------------------------------------

/// The danger in this design is reporting the dropdown twice: once through the anchor's
/// `operate` and once through the overlay. Every operation but the arrows' must see each item
/// **once**: the targets are distinct, a Tab visits each once and wraps, Shift+Tab the same.
#[test]
fn every_other_operation_sees_each_dropdown_item_once() {
    println!("\n## targets, one Tab pass and one Shift+Tab pass\n");
    for (host, close) in [
        (Host::Dropdown, true),
        (Host::Dropdown, false),
        (Host::Context, true),
        (Host::Context, false),
    ] {
        let (mut rig, _) = Scene::new(host, close).rig();
        let targets = rig.targets();
        let n = targets.len();
        // Distinct places: an item reported twice would repeat a bounds.
        for (i, a) in targets.iter().enumerate() {
            for b in &targets[i + 1..] {
                assert_ne!(
                    (a.bounds.x, a.bounds.y, a.bounds.width, a.bounds.height),
                    (b.bounds.x, b.bounds.y, b.bounds.width, b.bounds.height),
                    "{host:?} close {close}: a target is reported twice"
                );
            }
        }
        // The count, as the first Unit 0 measured it (the premise table, unchanged).
        let expected = match (host, close) {
            (Host::Dropdown, true) => 3,
            (Host::Dropdown, false) => 6,
            (Host::Context, true) => 4,
            (Host::Context, false) => 6,
        };
        assert_eq!(n, expected, "{host:?} close {close}");

        // Tab visits every target once, in order (iced then leaves focus nowhere before it
        // starts again: its own behaviour, not changed here).
        let mut visited = vec![];
        for _ in 0..n {
            rig.tab(false);
            visited.push(rig.focused());
        }
        let want: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        assert_eq!(visited, want, "{host:?} close {close}: Tab");
        // And Shift+Tab walks back over the same targets, once each.
        let mut back = vec![];
        for _ in 1..n {
            rig.tab(true);
            back.push(rig.focused());
        }
        let want: Vec<Vec<usize>> = (0..n - 1).rev().map(|i| vec![i]).collect();
        assert_eq!(back, want, "{host:?} close {close}: Shift+Tab");
        println!("- {host:?}, close {close}: {n} targets, distinct, Tab and Shift+Tab once each");
    }
}

/// An arrow, then Tab: the arrow's focus is the one Tab continues from, and there is one.
#[test]
fn tab_continues_from_an_arrows_focus_and_there_is_one() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene::new(host, true).rig();
        rig.named(ArrowDown, false);
        let at = focused(&mut rig);
        assert_eq!(at.len(), 1, "{host:?}");
        rig.tab(false);
        let next = focused(&mut rig);
        assert_eq!(next.len(), 1, "{host:?}");
        assert_ne!(next, at, "{host:?}: Tab did not move");
    }
}

/// A dialog over an open menu: the menu is silent, it is not a Tab target, and the dialog's one
/// control is the only target (strengthened: the key's status and a Tab lap).
#[test]
fn under_a_dialog_the_menu_is_silent_and_not_a_tab_target() {
    for host in [Host::Dropdown, Host::Context] {
        let (mut rig, _) = Scene {
            dialog: true,
            ..Scene::new(host, true)
        }
        .rig();
        assert_eq!(
            rig.targets().len(),
            1,
            "{host:?}: only the dialog's control"
        );
        for key in [ArrowDown, ArrowUp, Home, End] {
            let status = rig.named(key, false);
            assert_eq!(status, Status::Ignored, "{host:?} {key:?}");
            assert_eq!(focused(&mut rig), Vec::<usize>::new(), "{host:?} {key:?}");
        }
        rig.tab(false);
        assert_eq!(focused(&mut rig), vec![0], "{host:?}: Tab");
        // Another Tab leaves the dialog's control or stays on it: never anywhere else.
        rig.tab(false);
        assert!(focused(&mut rig).iter().all(|f| *f == 0), "{host:?}: Tab");
    }
}

/// The stored layout belongs to its build: the menu closes, then opens again with a different
/// number of items, and the key that follows the rebuild (no redraw between) uses the new one.
#[test]
fn a_reopened_menu_with_other_items_is_asked_not_the_old_one() {
    let c = Counts::default();
    let three = Scene::new(Host::Dropdown, true);
    let five = Scene {
        items: 5,
        ..Scene::new(Host::Dropdown, true)
    };
    let closed = Scene {
        open: false,
        ..Scene::new(Host::Dropdown, true)
    };
    let mut rig = Rig::new(three.build(&c));
    rig.named(End, false);
    assert_eq!(item_of(&mut rig, 3), Some(2));
    // Closed, then open with five, rebuilt each time with no redraw: the key is the first update.
    let rig = rig.rebuilt_quiet(closed.build(&c));
    let mut rig = rig.rebuilt_quiet(five.build(&c));
    let status = rig.named(End, false);
    assert_eq!(status, Status::Captured);
    assert_eq!(item_of(&mut rig, 5), Some(4), "the old layout was used");
    // And straight to a menu of two.
    let two = Scene {
        items: 2,
        ..Scene::new(Host::Dropdown, true)
    };
    let mut rig = rig.rebuilt_quiet(two.build(&c));
    rig.named(End, false);
    assert_eq!(item_of(&mut rig, 2), Some(1));
}

/// Scrolling between the overlay's layout and the key: a wheel over the dropdown and a ↓ in
/// **one batch**. The scroll offset is state read when the walk operates, not part of the
/// layout, so the focused item is brought into view from where the wheel left it.
#[test]
fn a_wheel_and_an_arrow_in_one_batch_keep_the_item_in_view() {
    const ITEMS: u8 = 40;
    let (mut rig, _) = Scene {
        items: ITEMS,
        ..Scene::new(Host::Dropdown, true)
    }
    .rig();
    rig.named(Home, false);
    // Over the dropdown (it is at the window's top-left, below the header).
    rig.set_cursor(Point::new(60.0, 200.0));
    let events = [
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -30.0 },
        }),
        key_event(ArrowDown, false, Modifiers::empty()),
    ];
    let statuses = rig.batch(&events);
    println!(
        "[wheel, ↓] in one batch: {statuses:?}, focused {:?}",
        focused(&mut rig)
    );
    assert_eq!(statuses[1], Status::Captured);
    assert_eq!(item_of(&mut rig, usize::from(ITEMS)), Some(1));
    // Item 1 is in view, its whole ring, by pixels.
    let shown = shot(&mut rig);
    let at = focused(&mut rig)[0];
    rig.focus_at(usize::MAX);
    let plain = shot(&mut rig);
    rig.focus_at(at);
    let (centre, _) = changed_box(&shown, &plain).expect("item 1 is focused and no ring is drawn");
    assert!(
        (40.0..600.0).contains(&centre.y),
        "item 1's ring is at y = {:.0}: out of view",
        centre.y
    );
}

/// A dialog that opens **over a menu whose dropdown has been laid out**: the view still says the
/// menu is open, the overlay is gone (a covered layer has none), and the arrows are not
/// answered. (The scene with a dialog built from the start never lays the dropdown out, so it
/// cannot tell a silent menu from one that was never asked.)
#[test]
fn a_dialog_opened_over_a_laid_out_dropdown_silences_it() {
    for close in [true, false] {
        let c = Counts::default();
        let mut rig = Rig::new(Scene::new(Host::Dropdown, close).build(&c));
        assert_eq!(
            rig.named(ArrowDown, false),
            Status::Captured,
            "close {close}"
        );
        let mut rig = rig.rebuilt(
            Scene {
                dialog: true,
                ..Scene::new(Host::Dropdown, close)
            }
            .build(&c),
        );
        for key in [ArrowDown, ArrowUp, Home, End] {
            let status = rig.named(key, false);
            assert_eq!(
                status,
                Status::Ignored,
                "close {close}: {key:?} under a dialog"
            );
        }
    }
}

/// RFC-123 ruling 3: an event **earlier in the same batch moves the trigger**, then End. The
/// dropdown's node is computed at the walk from the trigger as it then is, so its room is the
/// real one and the focused item is in view **as the walk left it**, by pixels.
///
/// Every operation on the overlay (this rig's `targets()` is one) brings the focused item into
/// view again with the layout it is then given, so a stale answer from the walk is mended by
/// the next operation. The test therefore reads the frame **before** any operation, and then
/// the frame after one: they must be the same pixels. (A node stored at the overlay's last
/// layout gives the walk the old room, 300 px more: the first frame has the item out of view,
/// and the second moves it in.) Without a close message: with one, the page is covered and the
/// key never reaches the block that moves the trigger.
#[test]
fn an_earlier_event_that_moves_the_trigger_leaves_the_focused_item_in_view() {
    const ITEMS: u8 = 40;
    let flag = Arc::new(AtomicBool::new(false));
    let (mut rig, _) = Scene {
        items: ITEMS,
        grow: Some(flag.clone()),
        ..Scene::new(Host::Dropdown, false)
    }
    .rig();
    rig.named(Home, false);
    let statuses = rig.batch(&[
        key_event(keyboard::key::Named::F5, false, Modifiers::empty()),
        key_event(End, false, Modifiers::empty()),
    ]);
    println!("[F5 (moves the trigger by 300 px), End] in one batch: {statuses:?}");
    assert!(flag.load(Ordering::SeqCst), "the trigger did not move");
    assert_eq!(statuses, vec![Status::Ignored, Status::Captured]);

    // The frame as the walk left it (a redraw first, as the window loop sends; no operation).
    let as_left = shot(&mut rig);
    // Any operation mends the scroll with the layout it is given; then the frame again.
    assert_eq!(
        item_of(&mut rig, usize::from(ITEMS)),
        Some(usize::from(ITEMS) - 1)
    );
    let mended = shot(&mut rig);
    assert!(
        as_left == mended,
        "the walk left the focused item where an operation then had to move it"
    );

    // And it is in view, the whole ring, inside the room below the moved trigger.
    let at = focused(&mut rig)[0];
    rig.focus_at(usize::MAX);
    let plain = shot(&mut rig);
    rig.focus_at(at);
    let (centre, _) = changed_box(&mended, &plain)
        .expect("the last item is focused and no ring is drawn: it is out of view");
    println!("the last item's ring is at y = {:.0}", centre.y);
    assert!(
        (300.0..600.0).contains(&centre.y),
        "the ring is at y = {:.0}: not inside the room below the moved trigger (300..600)",
        centre.y
    );
}
