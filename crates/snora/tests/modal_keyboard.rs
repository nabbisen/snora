//! The keyboard does what the pointer can (RFC-119).
//!
//! > **The keyboard reaches only what the pointer can press, and leaves an
//! > overlay the way the pointer can.**
//!
//! | Open | The keyboard |
//! |---|---|
//! | nothing | unchanged |
//! | a menu **with** its backdrop | cannot reach the page; **Escape closes the menu** |
//! | a menu **without** a backdrop | reaches the page (unchanged) |
//! | a dialog or sheet **with** `on_close_modals` | cannot reach beneath; **Escape closes it** |
//! | a dialog or sheet **without** it | cannot reach beneath; Escape is the application's |
//! | toasts, in any case | reachable; Tab goes dialog, sheet, toasts |
//!
//! "Beneath" is everything under the topmost backdrop, backdrops included.
//!
//! # Unit 0, and what it left
//!
//! The first two tests are Unit 0's reproduction of the finding: with a dialog
//! open, Tab reached the page and Enter pressed it. They failed on `main` and
//! are the first rows of the table now.
//!
//! # How it is driven
//!
//! Through the runtime's own `UserInterface`, as RFC-116 R-1 Unit 1 did: Tab is
//! the runtime's `focus_next` run to completion, Enter and Escape are real key
//! events, and `UserInterface::update` returns each event's `Status`, which is
//! the testable stand-in for "an application's `keyboard::listen()` never
//! receives it" (`listen()` passes only `Status::Ignored`).
//!
//! **A test that asks `find_focused` whether a covered control has focus cannot
//! tell:** a covered layer reports no focus targets. That is why the Q-3 ring is
//! read from **pixels**.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia), named by the
//! renderer itself. **A wgpu run must use `--test-threads=1`.**
//!
//! `render_semantics.rs` (gate 5) is untouched, and its unchanged pass is the
//! evidence that the composition did not move.

#![cfg(all(feature = "widgets", feature = "design"))]

use iced::widget::{column, scrollable, text, text_input, tooltip};
use iced::{Element, Font, Length, Pixels, Point, Size, Theme, mouse};
use iced_runtime::core::clipboard;
use iced_runtime::core::event::Status;
use iced_runtime::core::renderer::Headless;
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Event, Rectangle};
use iced_runtime::user_interface::{self, UserInterface};

use snora::design::{Tokens, button};
use snora::{AppLayout, Dialog, Sheet, Toast, ToastIntent, render};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    /// The page's controls: "Delete" is the action a confirmation dialog gates.
    Delete,
    Search,
    MenuItem,
    DialogOk,
    SheetOk,
    ToastDismiss,
    CloseMenus,
    CloseModals,
    Typed(String),
}

const WINDOW: Size = Size::new(800.0, 600.0);

fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<Tokens> = OnceLock::new();
    T.get_or_init(Tokens::light)
}

fn btn(label: &str, msg: Msg) -> Element<'static, Msg> {
    button::primary(tokens(), label.to_owned(), msg)
}

/// What the page beneath is made of.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    /// Two controls: "Delete" and "Search".
    Buttons,
    /// A scrollable and a text input: the state R-5 says a modal must not reset.
    Rich,
    /// A `tooltip`, whose bubble is an overlay.
    Overlay,
    /// The same label with no tooltip: the reference the overlay is compared with.
    NoOverlay,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Menu {
    None,
    WithBackdrop,
    WithoutBackdrop,
}

/// What is open.
#[derive(Clone, Copy, Debug)]
struct Scene {
    page: Page,
    menu: Menu,
    dialog: bool,
    sheet: bool,
    toast: bool,
    close_modals: bool,
    /// A `text_input` inside the dialog, for Escape's two-press case.
    dialog_input: bool,
}

impl Scene {
    const fn page(page: Page) -> Self {
        Self {
            page,
            menu: Menu::None,
            dialog: false,
            sheet: false,
            toast: false,
            close_modals: true,
            dialog_input: false,
        }
    }

    const fn buttons() -> Self {
        Self::page(Page::Buttons)
    }

    const fn with_dialog(mut self) -> Self {
        self.dialog = true;
        self
    }

    const fn with_sheet(mut self) -> Self {
        self.sheet = true;
        self
    }

    const fn with_toast(mut self) -> Self {
        self.toast = true;
        self
    }

    const fn with_menu(mut self, menu: Menu) -> Self {
        self.menu = menu;
        self
    }

    const fn without_close_modals(mut self) -> Self {
        self.close_modals = false;
        self
    }

    const fn with_dialog_input(mut self) -> Self {
        self.dialog_input = true;
        self
    }
}

fn body(page: Page) -> Element<'static, Msg> {
    match page {
        Page::Buttons => column![btn("Delete", Msg::Delete), btn("Search", Msg::Search)]
            .spacing(8)
            .into(),
        Page::Rich => column![
            scrollable(column((0..40).map(|i| text(format!("row {i}")).into())))
                .height(Length::Fixed(100.0)),
            text_input("", "hello").on_input(Msg::Typed),
        ]
        .spacing(10)
        .into(),
        Page::Overlay => tooltip(
            text("hover me"),
            text("TOOLTIP BUBBLE"),
            tooltip::Position::Bottom,
        )
        .into(),
        Page::NoOverlay => text("hover me").into(),
    }
}

fn window(scene: Scene) -> Element<'static, Msg> {
    let mut layout = AppLayout::new(body(scene.page));

    match scene.menu {
        Menu::None => {}
        Menu::WithBackdrop => {
            layout = layout
                .header_menu(btn("Menu item", Msg::MenuItem))
                .on_close_menus(Msg::CloseMenus);
        }
        Menu::WithoutBackdrop => layout = layout.header_menu(btn("Menu item", Msg::MenuItem)),
    }
    if scene.dialog {
        let content: Element<'static, Msg> = if scene.dialog_input {
            column![
                text_input("", "").on_input(Msg::Typed),
                btn("OK", Msg::DialogOk)
            ]
            .into()
        } else {
            btn("OK", Msg::DialogOk)
        };
        layout = layout.dialog(Dialog::new(content));
    }
    if scene.sheet {
        layout = layout.sheet(Sheet::new(btn("Sheet OK", Msg::SheetOk)));
    }
    if (scene.dialog || scene.sheet) && scene.close_modals {
        layout = layout.on_close_modals(Msg::CloseModals);
    }
    if scene.toast {
        layout = layout.toasts(vec![Toast::new(
            1,
            ToastIntent::Info,
            "Saved",
            "All good.",
            Msg::ToastDismiss,
        )]);
    }
    render(layout)
}

// ---- the runtime's own interface --------------------------------------------

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned())
}

struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    messages: Vec<Msg>,
    cursor: mouse::Cursor,
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
        assert_eq!(
            Headless::name(&renderer),
            wanted,
            "another renderer drew: not the pixels under test"
        );
        let ui = UserInterface::build(
            root,
            WINDOW,
            user_interface::Cache::default(),
            &mut renderer,
        );
        Self {
            ui,
            renderer,
            messages: Vec::new(),
            cursor: mouse::Cursor::Unavailable,
        }
    }

    /// What an application's `view` does after `update`: a new tree of widgets
    /// around the same widget state. Followed, as in the runtime, by the
    /// `RedrawRequested` that precedes every draw, unless `redraw` is false.
    fn rebuilt_with(self, root: Element<'static, Msg>, redraw: bool) -> Self {
        let mut renderer = self.renderer;
        let ui = UserInterface::build(root, WINDOW, self.ui.into_cache(), &mut renderer);
        let mut rig = Self {
            ui,
            renderer,
            messages: self.messages,
            cursor: self.cursor,
        };
        if redraw {
            rig.redraw_requested();
        }
        rig
    }

    fn rebuilt(self, root: Element<'static, Msg>) -> Self {
        self.rebuilt_with(root, true)
    }

    fn deliver(&mut self, events: &[Event]) -> Vec<Status> {
        self.ui
            .update(
                events,
                self.cursor,
                &mut self.renderer,
                &mut clipboard::Null,
                &mut self.messages,
            )
            .1
    }

    fn redraw_requested(&mut self) {
        let _ = self.deliver(&[Event::Window(
            iced_runtime::core::window::Event::RedrawRequested(
                iced_runtime::core::time::Instant::now(),
            ),
        )]);
    }

    fn key(&mut self, key: iced::keyboard::key::Named) -> Vec<Status> {
        self.deliver(&[iced_test::simulator::press_key(key, None)])
    }

    fn enter(&mut self) {
        let _ = self.key(iced::keyboard::key::Named::Enter);
    }

    fn escape(&mut self) -> Vec<Status> {
        self.key(iced::keyboard::key::Named::Escape)
    }

    fn run<T: 'static>(&mut self, op: impl Operation<T> + 'static) -> Option<T> {
        let mut current: Box<dyn Operation<T>> = Box::new(op);
        loop {
            self.ui
                .operate(&self.renderer, &mut operation::black_box(current.as_mut()));
            match current.finish() {
                Outcome::None => return None,
                Outcome::Some(v) => return Some(v),
                Outcome::Chain(next) => current = next,
            }
        }
    }

    /// `focus_next`, run to completion as the window loop does.
    fn tab(&mut self) {
        let _ = self.run(focusable::focus_next::<()>());
    }

    /// The focus targets the tree **reports**, in tree order.
    fn targets(&mut self) -> Vec<(Option<Id>, bool)> {
        #[derive(Default)]
        struct Collect(Vec<(Option<Id>, bool)>);
        impl Operation for Collect {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn focusable(
                &mut self,
                id: Option<&Id>,
                _bounds: Rectangle,
                state: &mut dyn operation::Focusable,
            ) {
                self.0.push((id.cloned(), state.is_focused()));
            }
        }
        let mut found = Collect::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
    }

    fn flags(&mut self) -> Vec<bool> {
        self.targets().into_iter().map(|t| t.1).collect()
    }

    fn scroll_offsets(&mut self) -> Vec<f32> {
        #[derive(Default)]
        struct Offsets(Vec<f32>);
        impl Operation for Offsets {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn scrollable(
                &mut self,
                _id: Option<&Id>,
                _b: Rectangle,
                _c: Rectangle,
                translation: iced::Vector,
                _s: &mut dyn operation::Scrollable,
            ) {
                self.0.push(translation.y);
            }
        }
        let mut o = Offsets::default();
        self.ui.operate(&self.renderer, &mut o);
        o.0
    }

    /// The frame as the renderer draws it now, at scale 1.
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
            self.cursor,
        );
        let physical = Size::new(WINDOW.width as u32, WINDOW.height as u32);
        Headless::screenshot(&mut self.renderer, physical, 1.0, base.background_color)
    }
}

/// Presses Tab then Enter `rounds` times and returns what was published, in
/// order.
fn tab_enter(rig: &mut Rig, rounds: usize) -> Vec<Msg> {
    let before = rig.messages.len();
    for _ in 0..rounds {
        rig.tab();
        rig.enter();
    }
    rig.messages[before..].to_vec()
}

fn page_messages(m: &[Msg]) -> Vec<&Msg> {
    m.iter()
        .filter(|m| matches!(m, Msg::Delete | Msg::Search))
        .collect()
}

// ---- R-1: the keyboard reaches only what the pointer can --------------------

/// **With a dialog open, Tab never lands on the page beneath it.** The tree
/// reports exactly the dialog's control and the toast's, and pressing Tab and
/// Enter all the way round publishes nothing from the page.
#[test]
fn tab_never_reaches_a_control_beneath_an_open_dialog() {
    let mut rig = Rig::new(window(Scene::buttons().with_dialog().with_toast()));
    assert_eq!(
        rig.targets().len(),
        2,
        "the tree should report the dialog's one control and the toast's one: the page is covered"
    );
    let published = tab_enter(&mut rig, 6);
    assert!(
        page_messages(&published).is_empty(),
        "Tab reached the page beneath an open dialog and Enter pressed it: {published:?}"
    );
    assert!(
        published.contains(&Msg::DialogOk),
        "the dialog is reachable: {published:?}"
    );
}

/// **A control that had focus before the dialog opened is not pressed by Enter
/// once it is open** — in the gap before the next redraw, and after it.
#[test]
fn enter_does_not_press_a_page_control_focused_before_the_dialog_opened() {
    for redraw in [false, true] {
        let mut rig = Rig::new(window(Scene::buttons()));
        rig.tab();
        assert_eq!(
            rig.flags(),
            vec![true, false],
            "the premise: \"Delete\" has focus"
        );

        let mut rig = rig.rebuilt_with(window(Scene::buttons().with_dialog()), redraw);
        rig.enter();
        assert!(
            rig.messages.is_empty(),
            "(redraw requested: {redraw}) a dialog is open, and Enter published {:?} from the \
             page beneath it",
            rig.messages
        );
    }
}

/// **A menu with its backdrop covers the page**, as a dialog does: a click
/// outside closes the menu and presses nothing, so the keyboard must not reach
/// the page either.
#[test]
fn a_menu_with_a_backdrop_covers_the_page_from_the_keyboard() {
    let scene = Scene::buttons().with_menu(Menu::WithBackdrop);
    let mut rig = Rig::new(window(scene));
    assert_eq!(
        rig.targets().len(),
        1,
        "the tree should report only the menu's one control"
    );
    let published = tab_enter(&mut rig, 4);
    assert!(
        page_messages(&published).is_empty(),
        "Tab reached the page beneath an open menu with a backdrop: {published:?}"
    );

    // Focused before the menu opened.
    let mut rig = Rig::new(window(Scene::buttons()));
    rig.tab();
    for redraw in [false, true] {
        let mut covered = Rig::new(window(Scene::buttons()));
        covered.tab();
        let mut covered = covered.rebuilt_with(window(scene), redraw);
        covered.enter();
        assert!(
            covered.messages.is_empty(),
            "(redraw requested: {redraw}) Enter pressed the page beneath an open menu: {:?}",
            covered.messages
        );
    }
    drop(rig);
}

/// **A menu without a backdrop leaves the page pressable, and so reachable.**
#[test]
fn a_menu_without_a_backdrop_leaves_the_page_reachable() {
    let mut rig = Rig::new(window(Scene::buttons().with_menu(Menu::WithoutBackdrop)));
    assert_eq!(
        rig.targets().len(),
        3,
        "the page's two controls and the menu's one"
    );
    let published = tab_enter(&mut rig, 3);
    assert!(
        published.contains(&Msg::Delete) && published.contains(&Msg::Search),
        "the pointer can press the page, so the keyboard must reach it: {published:?}"
    );
}

/// **A sheet covers the page exactly as a dialog does.**
#[test]
fn a_sheet_covers_the_page_from_the_keyboard() {
    let mut rig = Rig::new(window(Scene::buttons().with_sheet()));
    assert_eq!(
        rig.targets().len(),
        1,
        "only the sheet's control is reachable"
    );
    let published = tab_enter(&mut rig, 4);
    assert!(
        page_messages(&published).is_empty(),
        "Tab reached the page beneath an open sheet: {published:?}"
    );
    assert!(published.contains(&Msg::SheetOk), "{published:?}");

    for redraw in [false, true] {
        let mut rig = Rig::new(window(Scene::buttons()));
        rig.tab();
        let mut rig = rig.rebuilt_with(window(Scene::buttons().with_sheet()), redraw);
        rig.enter();
        assert!(
            rig.messages.is_empty(),
            "(redraw {redraw}) {:?}",
            rig.messages
        );
    }
}

/// **Q-4: Tab goes dialog, sheet, then toasts**: the order of the layers.
#[test]
fn tab_order_is_dialog_then_sheet_then_toasts() {
    let mut rig = Rig::new(window(
        Scene::buttons().with_dialog().with_sheet().with_toast(),
    ));
    let published = tab_enter(&mut rig, 3);
    assert_eq!(
        published,
        vec![Msg::DialogOk, Msg::SheetOk, Msg::ToastDismiss],
        "Tab order"
    );
}

// ---- R-4: leaving the way the pointer can ------------------------------------

/// **Escape closes a menu that has a backdrop**, once, and the event is
/// `Captured`, which is what keeps an application's `keyboard::listen()` (it
/// passes only `Ignored`) from also seeing it.
#[test]
fn escape_closes_a_menu_with_a_backdrop_once_and_is_captured() {
    let mut rig = Rig::new(window(Scene::buttons().with_menu(Menu::WithBackdrop)));
    let statuses = rig.escape();
    assert_eq!(rig.messages, vec![Msg::CloseMenus]);
    assert_eq!(statuses, vec![Status::Captured]);
}

/// **Escape closes a modal that closes on a click outside**, once, and a menu
/// open beneath it stays open (its message is not published).
#[test]
fn escape_closes_the_modal_and_the_menu_beneath_it_stays_open() {
    let scene = Scene::buttons().with_menu(Menu::WithBackdrop).with_dialog();
    let mut rig = Rig::new(window(scene));
    let statuses = rig.escape();
    assert_eq!(rig.messages, vec![Msg::CloseModals]);
    assert_eq!(statuses, vec![Status::Captured]);
}

/// **A modal that cannot be closed by a click outside is not closed by
/// Escape either**, and the menu beneath it must not answer in its place: the
/// message is nothing, and the status is `Ignored`, so the application's own
/// Escape handling still sees it.
#[test]
fn escape_without_on_close_modals_publishes_nothing_and_is_ignored() {
    let scene = Scene::buttons()
        .with_menu(Menu::WithBackdrop)
        .with_dialog()
        .without_close_modals();
    let mut rig = Rig::new(window(scene));
    let statuses = rig.escape();
    assert!(
        rig.messages.is_empty(),
        "the sinkless dim did not answer, but the menu beneath did: {:?}",
        rig.messages
    );
    assert_eq!(statuses, vec![Status::Ignored]);
}

/// **A focused `text_input` in a dialog takes Escape first**, as iced does; the
/// second press closes the dialog. Never more than two.
#[test]
fn escape_in_a_dialogs_text_input_takes_at_most_two_presses() {
    let mut rig = Rig::new(window(Scene::buttons().with_dialog().with_dialog_input()));
    rig.tab(); // the dialog's text input is its first control
    assert_eq!(
        rig.flags(),
        vec![true, false],
        "the premise: the input has focus"
    );

    let mut presses = 0;
    while !rig.messages.contains(&Msg::CloseModals) {
        presses += 1;
        assert!(
            presses <= 2,
            "Escape took {presses} presses to close the dialog"
        );
        let _ = rig.escape();
    }
    assert_eq!(presses, 2, "the first press belongs to the text input");
    assert_eq!(
        rig.messages
            .iter()
            .filter(|m| **m == Msg::CloseModals)
            .count(),
        1,
        "published more than once"
    );
}

// ---- R-5, Q-3: the page is as it was ------------------------------------------

/// **Opening and closing a modal does not reset the page**: a scroll offset
/// and a focused text input are as they were.
#[test]
fn a_modal_opening_and_closing_does_not_reset_the_page() {
    let page = Scene::page(Page::Rich);
    let mut rig = Rig::new(window(page));
    rig.cursor = mouse::Cursor::Available(Point::new(20.0, 50.0));
    let _ = rig.deliver(&[Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -5.0 },
    })]);
    rig.tab(); // the text input
    let before = (rig.scroll_offsets(), rig.flags());
    assert!(before.0[0] > 0.0, "the premise: the page scrolled");
    assert_eq!(before.1, vec![true], "the premise: the input has focus");

    let rig = rig.rebuilt(window(page.with_dialog()));
    let mut rig = rig.rebuilt(window(page));
    assert_eq!(
        (rig.scroll_offsets(), rig.flags()),
        before,
        "opening and closing a modal changed the page"
    );
}

/// **Q-3: the control that had focus has it again when the modal closes.**
#[test]
fn focus_returns_to_the_control_that_had_it_when_the_modal_closes() {
    let mut rig = Rig::new(window(Scene::buttons()));
    rig.tab();
    rig.tab(); // "Search"
    assert_eq!(rig.flags(), vec![false, true], "the premise");

    let rig = rig.rebuilt(window(Scene::buttons().with_dialog()));
    let mut rig = rig.rebuilt(window(Scene::buttons()));
    assert_eq!(
        rig.flags(),
        vec![false, true],
        "focus did not return to \"Search\""
    );
}

/// **Q-3: no ring under the dim**, read from pixels (a covered layer reports no
/// focus targets, so asking `find_focused` could not tell). The frame with a
/// dialog open over a page whose control had focus is pixel-for-pixel the frame
/// over a page whose control never did.
#[test]
fn no_focus_ring_is_drawn_under_the_dim() {
    let dialog = Scene::buttons().with_dialog();

    let mut focused_before = Rig::new(window(Scene::buttons()));
    focused_before.tab();
    let mut focused_before = focused_before.rebuilt(window(dialog));

    let mut never = Rig::new(window(Scene::buttons()));
    never.redraw_requested();
    let mut never = never.rebuilt(window(dialog));

    let (a, b) = (focused_before.frame(), never.frame());
    let differing = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert_eq!(
        differing,
        0,
        "[{}] the frame differs from the unfocused one in {differing} bytes: a ring is drawn \
         under the dim",
        backend()
    );

    // The premise: with no dialog, the focus ring is visible and does differ.
    let mut plain = Rig::new(window(Scene::buttons()));
    plain.tab();
    plain.redraw_requested();
    let ring = plain.frame();
    let mut bare = Rig::new(window(Scene::buttons()));
    bare.redraw_requested();
    let bare = bare.frame();
    assert!(
        ring.iter().zip(&bare).any(|(x, y)| x != y),
        "the premise failed: a focused control draws no ring at all, so the test above proves nothing"
    );
}

// ---- Unit 0's review: A and B --------------------------------------------------

/// **A. The application's own focus wins over the restore.** The dialog closes,
/// and the application's `update` returns a task that focuses a page control of
/// its own; the restore must not also focus the opener.
#[test]
fn the_applications_own_focus_wins_over_the_restore() {
    let mut rig = Rig::new(window(Scene::buttons()));
    rig.tab();
    rig.tab(); // "Search" holds focus when the dialog opens
    let rig = rig.rebuilt(window(Scene::buttons().with_dialog()));

    // The dialog closes; the view is rebuilt, and before the next redraw the
    // application's task runs `focus_next`, which lands on the first control.
    let mut rig = rig.rebuilt_with(window(Scene::buttons()), false);
    rig.tab();
    rig.redraw_requested();
    assert_eq!(
        rig.flags(),
        vec![true, false],
        "the application focused \"Delete\", and the restore focused \"Search\" as well"
    );
}

/// **B. Nothing from a covered layer floats above what covers it.** A page
/// `tooltip` showing when a dialog opens: iced draws overlays above every layer,
/// so its bubble would stay above the dim, and a dropdown would keep taking the
/// pointer and the keys. The frame is the one over a page whose label has no
/// tooltip at all.
#[test]
fn a_page_overlay_does_not_float_above_the_dim() {
    let hover = |rig: &mut Rig| {
        let at = Point::new(10.0, 10.0);
        rig.cursor = mouse::Cursor::Available(at);
        let _ = rig.deliver(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
        rig.redraw_requested();
    };

    let mut with = Rig::new(window(Scene::page(Page::Overlay)));
    hover(&mut with);
    let mut without = Rig::new(window(Scene::page(Page::NoOverlay)));
    hover(&mut without);
    assert!(
        with.frame() != without.frame(),
        "the premise failed: hovering the label showed no tooltip, so the test proves nothing"
    );

    let mut with = with.rebuilt(window(Scene::page(Page::Overlay).with_dialog()));
    let mut without = without.rebuilt(window(Scene::page(Page::NoOverlay).with_dialog()));
    let (a, b) = (with.frame(), without.frame());
    let differing = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert_eq!(
        differing,
        0,
        "[{}] the page's tooltip is still drawn above the dim ({differing} bytes differ)",
        backend()
    );
}
