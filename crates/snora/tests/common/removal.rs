//! The runtime's own interface, for tests of where focus goes when something
//! is removed (RFC-120).
//!
//! A `UserInterface` built and updated the way the window loop does it, with
//! a headless renderer named by `ICED_TEST_BACKEND` (default tiny-skia, and
//! the renderer that drew is asserted). What it adds to the simulator is what
//! these tests need and the simulator has not got: **running an operation**
//! (Tab, a focus task), **rebuilding around a different view while keeping the
//! widget state** (what `view` does after `update`), and **the redraw request
//! the runtime sends before every draw**.

#![allow(dead_code)]

use iced::keyboard::key::Named;
use iced::{Element, Font, Pixels, Point, Rectangle, Size, Theme, mouse};
use iced_runtime::core::clipboard;
use iced_runtime::core::renderer::Headless;
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Event, window};
use iced_runtime::user_interface::{self, UserInterface};

/// The window the scenes are laid out in.
pub const WINDOW: Size = Size::new(800.0, 600.0);

/// The renderer the run asked for.
pub fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned())
}

/// One focus target, in Tab order.
#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub bounds: Rectangle,
    pub focused: bool,
}

pub struct Rig<M: 'static> {
    ui: UserInterface<'static, M, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    /// What the interface published, as the runtime would collect it.
    pub messages: Vec<M>,
    cursor: mouse::Cursor,
}

impl<M: Clone + 'static> Rig<M> {
    /// A first build, and the redraw request the runtime sends before the
    /// first draw.
    pub fn new(root: Element<'static, M>) -> Self {
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
        let mut rig = Self {
            ui,
            renderer,
            messages: Vec::new(),
            cursor: mouse::Cursor::Unavailable,
        };
        rig.redraw();
        rig
    }

    /// What an application's `view` does after `update`, **keeping the widget
    /// state**, and then the redraw request the runtime sends before a draw.
    pub fn rebuilt(self, root: Element<'static, M>) -> Self {
        let mut rig = self.rebuilt_quiet(root);
        rig.redraw();
        rig
    }

    /// The same, **without** the redraw request: the gap between the rebuild and
    /// the redraw, in which a task (a focus operation) may run.
    pub fn rebuilt_quiet(self, root: Element<'static, M>) -> Self {
        let mut renderer = self.renderer;
        let ui = UserInterface::build(root, WINDOW, self.ui.into_cache(), &mut renderer);
        Self {
            ui,
            renderer,
            messages: self.messages,
            cursor: self.cursor,
        }
    }

    /// The redraw request the runtime sends before every draw.
    pub fn redraw(&mut self) {
        self.deliver(&[Event::Window(window::Event::RedrawRequested(
            iced_runtime::core::time::Instant::now(),
        ))]);
    }

    fn deliver(&mut self, events: &[Event]) {
        let _ = self.ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    /// The pointer moves onto `at`, or off the window if `None`.
    pub fn point_at(&mut self, at: Option<Point>) {
        self.cursor = at.map_or(mouse::Cursor::Unavailable, mouse::Cursor::Available);
        if let Some(position) = at {
            self.deliver(&[Event::Mouse(mouse::Event::CursorMoved { position })]);
        } else {
            self.deliver(&[Event::Mouse(mouse::Event::CursorLeft)]);
        }
    }

    /// `times` wheel events with the pointer at `at` (one line each, scrolling down):
    /// what `UserInterface::update` returned for each.
    pub fn wheel(&mut self, at: Point, times: usize) -> Vec<iced_runtime::core::event::Status> {
        self.point_at(Some(at));
        (0..times)
            .map(|_| {
                let (_, statuses) = self.ui.update(
                    &[Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
                    })],
                    self.cursor,
                    &mut self.renderer,
                    &mut clipboard::Null,
                    &mut self.messages,
                );
                statuses[0]
            })
            .collect()
    }

    /// The pointer shape the interface reports with the pointer at `at`.
    pub fn interaction_at(&mut self, at: Point) -> mouse::Interaction {
        self.cursor = mouse::Cursor::Available(at);
        let (state, _) = self.ui.update(
            &[Event::Mouse(mouse::Event::CursorMoved { position: at })],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        match state {
            user_interface::State::Updated {
                mouse_interaction, ..
            } => mouse_interaction,
            user_interface::State::Outdated => panic!("the interface is outdated"),
        }
    }

    /// The left button goes down, or up, where the pointer is.
    pub fn left_button(&mut self, down: bool) {
        let button = mouse::Button::Left;
        self.deliver(&[Event::Mouse(if down {
            mouse::Event::ButtonPressed(button)
        } else {
            mouse::Event::ButtonReleased(button)
        })]);
    }

    /// A left click at `at`: the pointer moves there, presses and releases.
    pub fn click_at(&mut self, at: Point) {
        self.point_at(Some(at));
        self.deliver(&[Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        ))]);
        self.deliver(&[Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))]);
    }

    /// A key press with `modifiers`, and what the interface did with it: the
    /// event's `Status`, the testable stand-in for "an application's
    /// `keyboard::listen()` never receives it" (`listen()` passes only `Ignored`).
    pub fn press(
        &mut self,
        key: iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> iced_runtime::core::event::Status {
        self.press_on(
            key,
            iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            modifiers,
        )
    }

    /// The same, with the **physical** key a keyboard layout reports beside the
    /// logical one (on AZERTY the key marked `1` has the logical key `&`).
    pub fn press_on(
        &mut self,
        key: iced::keyboard::Key,
        physical: iced::keyboard::key::Physical,
        modifiers: iced::keyboard::Modifiers,
    ) -> iced_runtime::core::event::Status {
        let event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: physical,
            location: iced::keyboard::Location::Standard,
            modifiers,
            repeat: false,
            text: None,
        });
        let (_, statuses) = self.ui.update(
            &[event],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        statuses[0]
    }

    /// A plain named key, with `repeat` (a held key), and the `Status` the interface gave it.
    pub fn named(&mut self, key: Named, repeat: bool) -> iced_runtime::core::event::Status {
        self.batch(&[key_event(key, repeat, iced::keyboard::Modifiers::empty())])[0]
    }

    /// Several events in **one** `UserInterface::update`, as the window loop hands them
    /// over between two redraws: the statuses, one per event.
    pub fn batch(&mut self, events: &[Event]) -> Vec<iced_runtime::core::event::Status> {
        let (_, statuses) = self.ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        statuses
    }

    /// The pointer position the rig holds.
    pub fn set_cursor(&mut self, at: Point) {
        self.cursor = mouse::Cursor::Available(at);
    }

    pub fn key(&mut self, key: Named) {
        self.deliver(&[iced_test::simulator::press_key(key, None)]);
    }

    pub fn enter(&mut self) {
        self.key(Named::Enter);
    }

    /// Runs an operation to completion, as the window loop does.
    pub fn run(&mut self, operation: Box<dyn Operation<()>>) {
        let mut current = operation;
        loop {
            self.ui
                .operate(&self.renderer, &mut operation::black_box(current.as_mut()));
            match current.finish() {
                Outcome::None | Outcome::Some(()) => return,
                Outcome::Chain(next) => current = next,
            }
        }
    }

    /// Tab, or Shift+Tab, **through `snora::keyboard::traverse`**: the task it
    /// returns is taken apart and the operation inside run on this interface.
    pub fn tab(&mut self, shift: bool) {
        use iced::futures::StreamExt;
        let modifiers = if shift {
            iced::keyboard::Modifiers::SHIFT
        } else {
            iced::keyboard::Modifiers::empty()
        };
        let task =
            snora::keyboard::traverse::<M>(iced::keyboard::Key::Named(Named::Tab), modifiers)
                .expect("Tab is claimed");
        let mut stream = iced_runtime::task::into_stream(task).expect("a task with work in it");
        let Some(iced_runtime::Action::Widget(op)) =
            iced::futures::executor::block_on(stream.next())
        else {
            panic!("traverse did not produce a widget operation");
        };
        self.run(op);
    }

    /// The application focusing a control itself, as a focus task does: every
    /// other target is unfocused and target `at` is focused.
    pub fn focus_at(&mut self, at: usize) {
        self.run(Box::new(FocusAt { at, seen: 0 }));
    }

    /// Every focus target, in Tab order.
    pub fn targets(&mut self) -> Vec<Target> {
        let mut found = Collect::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
    }

    pub fn focused(&mut self) -> Vec<usize> {
        self.targets()
            .iter()
            .enumerate()
            .filter(|(_, t)| t.focused)
            .map(|(i, _)| i)
            .collect()
    }

    /// The frame as the renderer draws it now, at scale 1.
    pub fn frame(&mut self) -> Vec<u8> {
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
        Headless::screenshot(
            &mut self.renderer,
            Size::new(WINDOW.width as u32, WINDOW.height as u32),
            1.0,
            base.background_color,
        )
    }
}

/// Which focus target a ring is drawn around, **read from pixels**: this frame
/// against `clean`, the same view never focused, with the changed region's
/// centre matched to the nearest target. `None` if nothing changed (no ring).
///
/// Where the question is "is a ring drawn there", only pixels can answer: a
/// test that asks `find_focused` sees only what `operate` reports (RFC-119).
pub fn ring_on<M: Clone + 'static>(rig: &mut Rig<M>, clean: &mut Rig<M>) -> Option<usize> {
    let targets = rig.targets();
    let (shown, plain) = (rig.frame(), clean.frame());
    let width = WINDOW.width as usize;
    let (mut n, mut sx, mut sy) = (0usize, 0usize, 0usize);
    for (i, (a, b)) in shown
        .as_chunks::<4>()
        .0
        .iter()
        .zip(plain.as_chunks::<4>().0)
        .enumerate()
    {
        if a != b {
            n += 1;
            sx += i % width;
            sy += i / width;
        }
    }
    if n == 0 {
        return None;
    }
    let at = Point::new((sx / n) as f32, (sy / n) as f32);
    (0..targets.len()).min_by(|a, b| {
        let (da, db) = (
            targets[*a].bounds.center().distance(at),
            targets[*b].bounds.center().distance(at),
        );
        da.total_cmp(&db)
    })
}

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
        // Focusing one takes focus from the others, as a focus-by-id does.
        if self.seen == self.at {
            state.focus();
        } else {
            state.unfocus();
        }
        self.seen += 1;
    }
}

/// A key press event.
pub fn key_event(key: Named, repeat: bool, modifiers: iced::keyboard::Modifiers) -> Event {
    let key = iced::keyboard::Key::Named(key);
    Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced::keyboard::Location::Standard,
        modifiers,
        repeat,
        text: None,
    })
}

/// `focus_next`, for tests that want the runtime's own operation.
pub fn focus_next() -> Box<dyn Operation<()>> {
    Box::new(focusable::focus_next::<()>())
}
