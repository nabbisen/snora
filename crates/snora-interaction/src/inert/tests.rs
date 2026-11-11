//! Unit 0's measurements of `Inert` and `OnEscape` (RFC-119).
//!
//! Each test answers one of the handoff's questions, and says so. Pixels are
//! read from the renderer named by `ICED_TEST_BACKEND` (default tiny-skia),
//! checked by its own name, so the same file measures both. **A wgpu run must
//! use `--test-threads=1`.**

use super::*;
use crate::{Control, OnEscape, Ring, StyleFn};
use iced::widget::{column, container, mouse_area, row, scrollable, space, stack, text_input};
use iced::{Color, Font, Pixels, Point};
use iced_core::clipboard;
use iced_core::renderer::Headless;
use iced_core::widget::operation::{self, Outcome, focusable};
use iced_runtime::user_interface::{self, UserInterface};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Press(u8),
    CloseMenu,
    CloseModal,
    Typed(String),
}

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned())
}

fn headless() -> iced::Renderer {
    let wanted = backend();
    let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
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
    renderer
}

const RING: Color = Color::from_rgb(1.0, 0.0, 0.0);

fn ring() -> Ring<'static> {
    Ring {
        color: Box::new(|_| RING),
        width: 2.0,
        offset: 2.0,
    }
}

fn plain() -> StyleFn<'static> {
    Box::new(|_, _| iced::widget::button::Style::default())
}

/// A 40 x 30 control that publishes `Press(n)`.
fn ctl(n: u8) -> Control<'static, Msg> {
    Control::new(
        space()
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(30.0)),
        plain(),
        ring(),
    )
    .on_press(Msg::Press(n))
}

fn pad(content: impl Into<Element<'static, Msg>>) -> Element<'static, Msg> {
    container(content).padding(10).into()
}

const WINDOW: Size = Size::new(300.0, 200.0);

struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    messages: Vec<Msg>,
    cursor: mouse::Cursor,
}

impl Rig {
    fn new(root: Element<'static, Msg>) -> Self {
        let mut renderer = headless();
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

    /// What an application's `view` does after `update`: a new tree of
    /// widgets, built around the **same widget state**.
    fn rebuilt(self, root: Element<'static, Msg>) -> Self {
        let mut renderer = self.renderer;
        let ui = UserInterface::build(root, WINDOW, self.ui.into_cache(), &mut renderer);
        Self {
            ui,
            renderer,
            messages: self.messages,
            cursor: self.cursor,
        }
    }

    fn deliver(&mut self, events: &[Event]) -> Vec<iced_core::event::Status> {
        let (_, statuses) = self.ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        statuses
    }

    /// The event the runtime sends before every draw.
    fn redraw_requested(&mut self) {
        let _ = self.deliver(&[Event::Window(window::Event::RedrawRequested(
            iced_core::time::Instant::now(),
        ))]);
    }

    fn key(&mut self, key: iced::keyboard::key::Named) -> Vec<iced_core::event::Status> {
        self.deliver(&[iced_test::simulator::press_key(key, None)])
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

    fn tab(&mut self, times: usize) {
        for _ in 0..times {
            let _ = self.run(focusable::focus_next::<()>());
        }
    }

    /// `(id, is_focused)` of every focus target the tree **reports**.
    fn targets(&mut self) -> Vec<(Option<Id>, bool)> {
        #[derive(Default)]
        struct Collect(Vec<(Option<Id>, bool)>);
        impl Operation for Collect {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn focusable(&mut self, id: Option<&Id>, _b: Rectangle, s: &mut dyn Focusable) {
                self.0.push((id.cloned(), s.is_focused()));
            }
        }
        let mut c = Collect::default();
        self.ui.operate(&self.renderer, &mut c);
        c.0
    }

    fn flags(&mut self) -> Vec<bool> {
        self.targets().into_iter().map(|(_, f)| f).collect()
    }

    /// Red pixels in what the renderer draws now: the ring, and nothing else
    /// in these scenes is that red.
    fn ring_pixels(&mut self) -> usize {
        use iced::theme::Base;
        let theme = Theme::Light;
        let base = theme.base();
        self.ui.draw(
            &mut self.renderer,
            &theme,
            &renderer::Style {
                text_color: base.text_color,
            },
            self.cursor,
        );
        let physical = Size::new(WINDOW.width as u32 * 2, WINDOW.height as u32 * 2);
        let rgba = Headless::screenshot(&mut self.renderer, physical, 2.0, base.background_color);
        rgba.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > 240 && p[1] < 20 && p[2] < 20)
            .count()
    }

    /// A scrollable's offset, as an operation reports it.
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
                translation: Vector,
                _s: &mut dyn operation::Scrollable,
            ) {
                self.0.push(translation.y);
            }
        }
        let mut o = Offsets::default();
        self.ui.operate(&self.renderer, &mut o);
        o.0
    }
}

fn two_controls(inert: bool) -> Element<'static, Msg> {
    Inert::new(pad(row![ctl(1), ctl(2)].spacing(30)), inert).into()
}

// ---- Q-3: the opener's focus -----------------------------------------------

/// **Why the focus target is not remembered by its `Id`.** A control's id is
/// made when it is built, and every `view` builds again.
#[test]
fn a_controls_id_is_new_on_every_view() {
    let mut a = Rig::new(two_controls(false));
    let mut b = Rig::new(two_controls(false));
    let (a, b) = (a.targets(), b.targets());
    assert_eq!(a.len(), 2);
    assert!(
        a.iter().zip(&b).all(|(x, y)| x.0 != y.0),
        "the ids of two builds of the same view are equal, so an Id could have been used"
    );
}

/// **Parked on the way in, restored on the way out, across rebuilds.** Focus
/// the second control, cover the layer, uncover it: it is the second control,
/// in the rebuilt tree, that is focused again.
#[test]
fn focus_is_parked_when_covered_and_restored_by_position() {
    let mut rig = Rig::new(two_controls(false));
    rig.tab(2);
    assert_eq!(rig.flags(), vec![false, true], "the premise");

    // Covered: a new view with the flag set, and the runtime's redraw request.
    let mut rig = rig.rebuilt(two_controls(true));
    rig.redraw_requested();
    assert_eq!(
        rig.targets().len(),
        0,
        "a covered layer must report no focus targets"
    );
    assert_eq!(rig.ring_pixels(), 0, "a ring is drawn under the dim");

    // Uncovered again: a new view, new controls, new ids.
    let mut rig = rig.rebuilt(two_controls(false));
    rig.redraw_requested();
    assert_eq!(
        rig.flags(),
        vec![false, true],
        "the control that held focus does not hold it again"
    );
    assert!(rig.ring_pixels() > 0, "the restored control draws no ring");
}

/// **When it runs, measured.** A draw straight after the rebuild, with no
/// event in between, still shows the ring: the park happens on the
/// `RedrawRequested` that the runtime sends before every draw
/// (`iced_winit`: `update(RedrawRequested)`, then `draw`; `Simulator::snapshot`
/// the same). With that event, no frame has the ring.
#[test]
fn the_ring_is_gone_before_the_first_frame_the_runtime_draws() {
    let mut rig = Rig::new(two_controls(false));
    rig.tab(1);
    let mut rig = rig.rebuilt(two_controls(true));

    let without = rig.ring_pixels();
    rig.redraw_requested();
    let with = rig.ring_pixels();
    println!(
        "[{}] ring pixels: draw straight after the rebuild {without}; after RedrawRequested {with}",
        backend()
    );
    assert!(
        without > 0,
        "the premise: nothing has parked the focus before the event"
    );
    assert_eq!(with, 0, "a frame the runtime draws still has the ring");
}

/// A layer whose focus targets changed while it was covered restores nothing:
/// a position means nothing in a different layer.
#[test]
fn nothing_is_restored_into_a_different_layer() {
    let mut rig = Rig::new(two_controls(false));
    rig.tab(2);
    let mut rig = rig.rebuilt(two_controls(true));
    rig.redraw_requested();
    // Uncovered, but now with three controls.
    let mut rig = rig.rebuilt(Inert::new(pad(row![ctl(1), ctl(2), ctl(3)]), false).into());
    rig.redraw_requested();
    assert_eq!(rig.flags(), vec![false, false, false]);
}

// ---- the keyboard does not reach a covered layer ---------------------------

#[test]
fn a_covered_layer_takes_neither_focus_nor_keys() {
    let mut rig = Rig::new(two_controls(false));
    rig.tab(1);
    let mut rig = rig.rebuilt(two_controls(true));
    rig.redraw_requested();

    rig.tab(2);
    let _ = rig.key(iced::keyboard::key::Named::Enter);
    assert!(
        rig.messages.is_empty(),
        "a covered control was pressed from the keyboard: {:?}",
        rig.messages
    );

    // And uncovered, the same keys do press (the control).
    let mut rig = rig.rebuilt(two_controls(false));
    rig.redraw_requested();
    // No Tab: the control that had focus has it back, so Enter presses it.
    let _ = rig.key(iced::keyboard::key::Named::Enter);
    assert_eq!(rig.messages, vec![Msg::Press(1)]);
}

/// **Keys are dropped even before the focus is parked.** Between a rebuild that
/// sets the flag and the next redraw, a control may still hold focus: the key
/// must not press it. (The covered-layer test above cannot see this: once the
/// focus is parked, there is nothing for Enter to press whether or not keys are
/// dropped.)
#[test]
fn a_key_in_the_gap_before_the_redraw_presses_nothing() {
    let mut rig = Rig::new(two_controls(false));
    rig.tab(1);
    let mut rig = rig.rebuilt(two_controls(true));
    // No redraw request yet: the control still holds its focus.
    let _ = rig.key(iced::keyboard::key::Named::Enter);
    assert!(
        rig.messages.is_empty(),
        "a covered control was pressed from the keyboard before the park: {:?}",
        rig.messages
    );
}

// ---- R-5: state ------------------------------------------------------------

fn page(wrapped_when: impl Fn(bool) -> bool, open: bool) -> Element<'static, Msg> {
    let body: Element<'static, Msg> = container(
        column![
            scrollable(column(
                (0..40).map(|i| { iced::widget::text(format!("row {i}")).into() })
            ))
            .height(Length::Fixed(100.0)),
            text_input("", "hello").on_input(Msg::Typed),
        ]
        .spacing(10),
    )
    .padding(10)
    .into();
    if wrapped_when(open) {
        Inert::new(body, open).into()
    } else {
        body
    }
}

fn scrolled(rig: &mut Rig) {
    rig.cursor = mouse::Cursor::Available(Point::new(60.0, 50.0));
    let _ = rig.deliver(&[Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -5.0 },
    })]);
}

/// **The wrapper is always in the tree, so the page keeps its scroll offset
/// and its focused text input across a modal opening and closing.**
#[test]
fn state_survives_a_modal_opening_and_closing_when_the_wrapper_is_always_there() {
    let always = |_open: bool| true;
    let mut rig = Rig::new(page(always, false));
    scrolled(&mut rig);
    rig.tab(1); // the text input
    let before = (rig.scroll_offsets(), rig.flags());
    assert!(before.0[0] > 0.0, "the premise: the page scrolled");
    assert_eq!(before.1, vec![true], "the premise: the input has focus");

    let mut rig = rig.rebuilt(page(always, true));
    rig.redraw_requested();
    let mut rig = rig.rebuilt(page(always, false));
    rig.redraw_requested();
    assert_eq!((rig.scroll_offsets(), rig.flags()), before);
}

/// **The breakage row, and the proof the test above can fail:** the wrapper is
/// inserted only while the modal is open. The tree is reshaped on open and on
/// close, and the page loses its scroll offset.
#[test]
fn state_is_lost_when_the_wrapper_is_inserted_only_while_open() {
    let only_open = |open: bool| open;
    let mut rig = Rig::new(page(only_open, false));
    scrolled(&mut rig);
    assert!(
        rig.scroll_offsets()[0] > 0.0,
        "the premise: the page scrolled"
    );

    let rig = rig.rebuilt(page(only_open, true));
    let mut rig = rig.rebuilt(page(only_open, false));
    rig.redraw_requested();
    assert_eq!(
        rig.scroll_offsets(),
        vec![0.0],
        "the scroll offset survived a reshaped tree, so the always-wrapped test proves nothing"
    );
}

// ---- R-4: capture ----------------------------------------------------------

fn layers(menu: bool, modal: bool) -> Element<'static, Msg> {
    let mut s = stack![Inert::new(pad(ctl(1)), menu || modal)];
    if menu {
        s = s.push(OnEscape::new(
            mouse_area(space().width(Length::Fill).height(Length::Fill)).on_press(Msg::CloseMenu),
            Msg::CloseMenu,
        ));
    }
    if modal {
        s = s.push(OnEscape::new(
            mouse_area(space().width(Length::Fill).height(Length::Fill)).on_press(Msg::CloseModal),
            Msg::CloseModal,
        ));
    }
    s.into()
}

/// **An Escape the backdrop answers is `Captured`**, which is how
/// `iced::keyboard::listen()` (it passes only `Ignored`) never delivers it to
/// the application: no double message.
#[test]
fn escape_on_a_backdrop_is_published_once_and_captured() {
    let mut rig = Rig::new(layers(true, false));
    let statuses = rig.key(iced::keyboard::key::Named::Escape);
    assert_eq!(rig.messages, vec![Msg::CloseMenu]);
    assert_eq!(statuses, vec![iced_core::event::Status::Captured]);
}

/// **The trap, and the proof the test above can fail:** a plain `mouse_area`
/// backdrop takes no keys. Escape is ignored, nothing is published, and the
/// application's `listen()` would see it.
#[test]
fn a_plain_backdrop_ignores_escape_which_is_the_trap() {
    let plain: Element<'static, Msg> = stack![
        Inert::new(pad(ctl(1)), true),
        mouse_area(space().width(Length::Fill).height(Length::Fill)).on_press(Msg::CloseMenu),
    ]
    .into();
    let mut rig = Rig::new(plain);
    let statuses = rig.key(iced::keyboard::key::Named::Escape);
    assert!(rig.messages.is_empty());
    assert_eq!(statuses, vec![iced_core::event::Status::Ignored]);
}

/// **Law 1's order:** with a modal over a menu, the modal answers Escape and
/// the menu stays open.
#[test]
fn the_modal_answers_escape_before_the_menu_beneath_it() {
    let mut rig = Rig::new(layers(true, true));
    let statuses = rig.key(iced::keyboard::key::Named::Escape);
    assert_eq!(rig.messages, vec![Msg::CloseModal]);
    assert_eq!(statuses, vec![iced_core::event::Status::Captured]);
}

#[allow(dead_code)]
fn _use(_: Color) {}

// ---------------------------------------------------------------------------
// RFC-123 Unit 0b: which covered layers still answer the arrows' request for a dropdown
// ---------------------------------------------------------------------------

mod reach {
    use super::*;
    use crate::menu_anchor::OpenMenu;
    use crate::menu_keys::{Reach, Wants};
    use std::cell::Cell;
    use std::rc::Rc;

    /// What an open anchor does to an operation, and nothing else: offers it a `Reach`
    /// (after saying a menu is open, if it is) and records what the operation wanted.
    struct Anchor {
        open: bool,
        wanted: Rc<Cell<Wants>>,
    }

    impl Widget<Msg, Theme, iced::Renderer> for Anchor {
        fn tag(&self) -> tree::Tag {
            tree::Tag::stateless()
        }
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fixed(10.0), Length::Fixed(10.0))
        }
        fn layout(
            &mut self,
            _t: &mut Tree,
            _r: &iced::Renderer,
            _l: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(10.0, 10.0))
        }
        fn operate(
            &mut self,
            _t: &mut Tree,
            layout: Layout<'_>,
            _r: &iced::Renderer,
            operation: &mut dyn Operation,
        ) {
            if self.open {
                operation.custom(None, layout.bounds(), &mut OpenMenu);
            }
            let mut reach = Reach::default();
            operation.custom(None, layout.bounds(), &mut reach);
            self.wanted.set(reach.wants);
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

    /// The arrows' walk, as far as the request goes.
    struct Ask;

    impl Operation for Ask {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn custom(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn std::any::Any) {
            if let Some(reach) = state.downcast_mut::<Reach>() {
                reach.wants = Wants::Survey;
            }
        }
    }

    /// What the walk got, from an anchor in an `Inert` built like this.
    fn wanted_through(inert: bool, cover: bool, open: bool) -> Wants {
        let wanted = Rc::new(Cell::new(Wants::Nothing));
        let anchor = Anchor {
            open,
            wanted: wanted.clone(),
        };
        let layer = Inert::new(Element::new(anchor), inert)
            .beneath_menus(cover.then_some(Msg::CloseMenu), None);
        let mut rig = Rig::new(layer.into());
        rig.redraw_requested();
        let mut ask = Ask;
        rig.ui.operate(&rig.renderer, &mut ask);
        wanted.get()
    }

    /// Where the walk gets its dropdown, and where it must not (a dialog over a menu).
    #[test]
    fn a_menu_cover_lets_the_arrows_request_through_and_a_modal_drops_it() {
        // Not covered at all: it reaches the anchor.
        assert_eq!(
            wanted_through(false, true, false),
            Wants::Survey,
            "not covered"
        );
        // Covered by its own open menu (the cover is on, no modal): it still does.
        assert_eq!(
            wanted_through(false, true, true),
            Wants::Survey,
            "menu cover"
        );
        // The flag set too (a menu's backdrop slot) with the menu cover on: still the menu's.
        assert_eq!(
            wanted_through(true, true, true),
            Wants::Survey,
            "flag + menu cover"
        );
        // Under a modal the engine switches the menu cover off: the request is dropped.
        assert_eq!(
            wanted_through(true, false, true),
            Wants::Nothing,
            "under a modal"
        );
        // A layer that is merely inert: dropped.
        assert_eq!(wanted_through(true, false, false), Wants::Nothing, "inert");
    }
}
