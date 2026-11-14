//! A click presses a control and does not focus it; a held key presses once
//! (RFC-121).
//!
//! > **Clicking presses a control. It does not focus it. Focus moves only by
//! > the keyboard (Tab, through `traverse`) or by the application.**
//! >
//! > **A key press activates a control once. Holding the key does not repeat
//! > it.**
//!
//! 0.55.0 did neither, in every application, including those that never route
//! Tab: a focus ring appeared on every snora button a mouse or touch user
//! pressed, Space or Enter then pressed it again, and a held key repeated the
//! press. Nothing measured it: every keyboard test reached controls by Tab or by
//! an operation, and none clicked and then looked for a ring or pressed a key.
//!
//! # Every control, not one
//!
//! The finding was in `control`, and every snora control is one. The subjects
//! are the ones `every_control.rs` probes (`common::subjects`): every unstyled
//! control, and every design-path control in the light preset, each target of
//! each.
//!
//! # How it is driven
//!
//! Through the runtime's own `UserInterface`, as RFC-116 R-1 Unit 1 did: Tab is
//! `focus_next` run to completion; a click is a cursor move, a press and a
//! release; a held key is `KeyPressed { repeat: true }`. The ring is read from
//! **pixels** (a covered or unfocused control reports nothing to ask), and each
//! such test asserts its premise: that Tab to the same control **does** change
//! the frame, so it cannot pass because no ring is ever drawn.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia), named by the
//! renderer itself. **A wgpu run must use `--test-threads=1`.**

#![cfg(all(feature = "widgets", feature = "design"))]

use iced::keyboard::{self, Key, key::Named};
use iced::{Element, Font, Pixels, Point, Size, Theme, mouse};
use iced_runtime::core::clipboard;
use iced_runtime::core::renderer::Headless;
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Event, Rectangle, window};
use iced_runtime::user_interface::{self, UserInterface};

mod common;
use common::subjects::{Msg, Subject, WINDOW, designed, unstyled};

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned())
}

struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    messages: Vec<Msg>,
    cursor: mouse::Cursor,
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

    fn deliver(&mut self, events: &[Event]) {
        let _ = self.ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    fn redraw_requested(&mut self) {
        self.deliver(&[Event::Window(window::Event::RedrawRequested(
            iced_runtime::core::time::Instant::now(),
        ))]);
    }

    /// A key press, with its `repeat` flag.
    fn key(&mut self, named: Named, repeat: bool) {
        let key = Key::Named(named);
        self.deliver(&[Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            repeat,
            text: None,
        })]);
    }

    /// A key press with `modifiers`, and what the interface did with it: the event's `Status`, the
    /// testable stand-in for "an application's `keyboard::listen()` never receives it" (`listen()`
    /// passes only `Ignored`).
    fn press(
        &mut self,
        named: Named,
        modifiers: keyboard::Modifiers,
    ) -> iced_runtime::core::event::Status {
        let key = Key::Named(named);
        let (_, statuses) = self.ui.update(
            &[Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: keyboard::key::Physical::Unidentified(
                    keyboard::key::NativeCode::Unidentified,
                ),
                location: keyboard::Location::Standard,
                modifiers,
                repeat: false,
                text: None,
            })],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        statuses[0]
    }

    /// Cursor move, press, release: a click, left where it landed.
    fn click(&mut self, at: Point) {
        self.cursor = mouse::Cursor::Available(at);
        self.deliver(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
        self.deliver(&[Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        ))]);
        self.deliver(&[Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))]);
    }

    /// The pointer leaves the window, so no hover is in the frame.
    fn pointer_away(&mut self) {
        self.cursor = mouse::Cursor::Unavailable;
        self.deliver(&[Event::Mouse(mouse::Event::CursorLeft)]);
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

    fn tab(&mut self) {
        let _ = self.run(focusable::focus_next::<()>());
    }

    fn find_focused(&mut self) -> Option<Id> {
        self.run(focusable::find_focused())
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
        let mut found = Collect::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
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

/// Every subject, with the context it is probed in.
fn every_subject() -> Vec<(String, Subject)> {
    let tokens: &'static snora::design::Tokens =
        Box::leak(Box::new(snora::design::Tokens::light()));
    let mut all: Vec<(String, Subject)> = unstyled()
        .into_iter()
        .map(|s| ("unstyled".to_owned(), s))
        .collect();
    all.extend(
        designed(tokens)
            .into_iter()
            .map(|s| ("light".to_owned(), s)),
    );
    all
}

fn assert_clean(what: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} {what} failure(s) (backend {}):\n  {}",
        failures.len(),
        backend(),
        failures.join("\n  "),
    );
}

/// Runs `probe` once for every focus target of every subject, with a fresh
/// rig, and collects what it reports.
fn for_every_target(
    mut probe: impl FnMut(&str, &Subject, usize, &Msg) -> Option<String>,
) -> Vec<String> {
    let mut failures = Vec::new();
    let mut probed = 0;
    for (context, subject) in every_subject() {
        for (k, want) in subject.publishes.iter().enumerate() {
            probed += 1;
            if let Some(why) = probe(&context, &subject, k, want) {
                failures.push(format!("{context} / {} #{k}: {why}", subject.name));
            }
        }
    }
    assert!(
        probed >= 20,
        "only {probed} targets were probed: the subjects have gone missing"
    );
    failures
}

// ---- R-1: a click does not focus --------------------------------------------

/// **A click leaves nothing focused.**
#[test]
fn a_click_leaves_nothing_focused() {
    let failures = for_every_target(|_, subject, k, _| {
        let mut rig = Rig::new(subject.element());
        let at = rig.targets()[k].bounds.center();
        rig.click(at);
        rig.redraw_requested();
        let focused = rig.targets().iter().filter(|t| t.focused).count();
        (focused != 0 || rig.find_focused().is_some())
            .then(|| format!("{focused} control(s) focused after a click"))
    });
    assert_clean("click-focus", failures);
}

/// **A click draws no ring**, read from pixels: the frame after a click (the
/// pointer then away) is the frame of a control nobody touched. The premise is
/// asserted for the same control: Tab to it **does** change the frame.
#[test]
fn a_click_draws_no_ring() {
    let failures = for_every_target(|_, subject, k, _| {
        let mut untouched = Rig::new(subject.element());
        untouched.redraw_requested();
        let plain = untouched.frame();

        let mut tabbed = Rig::new(subject.element());
        for _ in 0..=k {
            tabbed.tab();
        }
        tabbed.redraw_requested();
        if tabbed.frame() == plain {
            return Some("the premise failed: Tab to the control changed no pixel".to_owned());
        }

        let mut clicked = Rig::new(subject.element());
        let at = clicked.targets()[k].bounds.center();
        clicked.click(at);
        clicked.pointer_away();
        clicked.redraw_requested();
        let differing = clicked
            .frame()
            .iter()
            .zip(&plain)
            .filter(|(a, b)| a != b)
            .count();
        (differing > 0).then(|| format!("{differing} bytes differ from an untouched frame: a ring"))
    });
    assert_clean("click-ring", failures);
}

/// **After a click, Space and Enter press nothing more**: one message in all,
/// the click's. In 0.55.0 a clicked control stayed focused, and each key
/// pressed it again.
#[test]
fn a_click_then_space_then_enter_publishes_only_the_click() {
    let failures = for_every_target(|_, subject, k, want| {
        let mut rig = Rig::new(subject.element());
        let at = rig.targets()[k].bounds.center();
        rig.click(at);
        rig.key(Named::Space, false);
        rig.key(Named::Enter, false);
        (rig.messages != vec![want.clone()]).then(|| {
            format!(
                "published {:?}, expected only the click's {want:?}",
                rig.messages
            )
        })
    });
    assert_clean("click-then-keys", failures);
}

// ---- R-3: a held key presses once ---------------------------------------------

/// **A held key presses once.** Tab to the control, press the key, then three
/// auto-repeats of it: one message, not four.
fn held_key_failures(key: Named) -> Vec<String> {
    for_every_target(|_, subject, k, want| {
        let mut rig = Rig::new(subject.element());
        for _ in 0..=k {
            rig.tab();
        }
        rig.key(key, false);
        for _ in 0..3 {
            rig.key(key, true);
        }
        (rig.messages != vec![want.clone()]).then(|| {
            format!(
                "{key:?} then three repeats published {:?}, expected one {want:?}",
                rig.messages
            )
        })
    })
}

#[test]
fn a_held_enter_presses_once() {
    assert_clean("held-enter", held_key_failures(Named::Enter));
}

#[test]
fn a_held_space_presses_once() {
    assert_clean("held-space", held_key_failures(Named::Space));
}

/// The control for the two above: an ordinary, non-repeat press still presses
/// every time it is pressed, so the tests are not passing because keys do
/// nothing.
#[test]
fn separate_presses_still_press_each_time() {
    let failures = for_every_target(|_, subject, k, want| {
        let mut rig = Rig::new(subject.element());
        for _ in 0..=k {
            rig.tab();
        }
        for _ in 0..3 {
            rig.key(Named::Enter, false);
        }
        (rig.messages != vec![want.clone(); 3])
            .then(|| format!("three presses published {:?}", rig.messages))
    });
    assert_clean("separate-presses", failures);
}

// ---- R-2: Tab after a click carries on from the clicked control ---------------

use iced::widget::{row, space};
use snora::design::button;

fn three() -> Element<'static, Msg> {
    let tokens: &'static snora::design::Tokens =
        Box::leak(Box::new(snora::design::Tokens::light()));
    iced::widget::container(
        iced::widget::column![
            row![
                button::ghost(tokens, "Alpha", Msg("a".to_owned())),
                button::ghost(tokens, "Bravo", Msg("b".to_owned())),
                button::ghost(tokens, "Charlie", Msg("c".to_owned())),
            ]
            .spacing(20),
            space().height(iced::Length::Fixed(200.0)),
        ]
        .spacing(10),
    )
    .padding(24)
    .into()
}

/// Tab, or Shift+Tab, **through `snora::keyboard::traverse`**: the task it
/// returns is taken apart, and the widget operation inside it is run on the
/// interface to completion, as the window loop does. (`focus_next` itself is
/// not what `traverse` runs any more, so a test that ran it would prove
/// nothing about R-2.)
fn traverse(rig: &mut Rig, shift: bool) {
    use iced::futures::StreamExt;
    let modifiers = if shift {
        keyboard::Modifiers::SHIFT
    } else {
        keyboard::Modifiers::empty()
    };
    let task = snora::keyboard::traverse::<Msg>(Key::Named(Named::Tab), modifiers)
        .expect("Tab is claimed");
    let mut stream = iced_runtime::task::into_stream(task).expect("a task with work in it");
    let Some(iced_runtime::Action::Widget(operation)) =
        iced::futures::executor::block_on(stream.next())
    else {
        panic!("traverse did not produce a widget operation");
    };
    let mut current: Box<dyn Operation<()>> = operation;
    loop {
        rig.ui
            .operate(&rig.renderer, &mut operation::black_box(current.as_mut()));
        match current.finish() {
            Outcome::None | Outcome::Some(()) => return,
            Outcome::Chain(next) => current = next,
        }
    }
}

fn focused_of(rig: &mut Rig) -> Vec<bool> {
    rig.targets().iter().map(|t| t.focused).collect()
}

fn centre(rig: &mut Rig, k: usize) -> Point {
    rig.targets()[k].bounds.center()
}

/// **After a click on B, Tab moves to C**; Shift+Tab, to A.
#[test]
fn tab_after_a_click_carries_on_from_the_clicked_control() {
    let mut rig = Rig::new(three());
    let b = centre(&mut rig, 1);
    rig.click(b);
    assert_eq!(
        focused_of(&mut rig),
        vec![false, false, false],
        "the premise: a click focuses nothing"
    );
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![false, false, true],
        "Tab after clicking B"
    );

    let mut rig = Rig::new(three());
    let b = centre(&mut rig, 1);
    rig.click(b);
    traverse(&mut rig, true);
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "Shift+Tab after clicking B"
    );
}

/// **After a press anywhere else, Tab starts from the top** (and Shift+Tab from
/// the bottom): the click's starting point is gone.
#[test]
fn tab_after_a_press_elsewhere_starts_from_the_top() {
    let mut rig = Rig::new(three());
    let b = centre(&mut rig, 1);
    rig.click(b);
    rig.click(Point::new(500.0, 500.0)); // empty space
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "Tab after clicking empty space"
    );

    let mut rig = Rig::new(three());
    let b = centre(&mut rig, 1);
    rig.click(b);
    rig.click(Point::new(500.0, 500.0));
    traverse(&mut rig, true);
    assert_eq!(
        focused_of(&mut rig),
        vec![false, false, true],
        "Shift+Tab from the bottom"
    );

    // And a plain Tab on an untouched window is the top, as before.
    let mut rig = Rig::new(three());
    traverse(&mut rig, false);
    assert_eq!(focused_of(&mut rig), vec![true, false, false]);
}

/// **After a click on the last control, Tab leaves the group** (no wrap, as
/// iced's `focus_next`), and the next Tab starts from the top.
#[test]
fn tab_after_a_click_on_the_last_control_leaves_the_group() {
    let mut rig = Rig::new(three());
    let c = centre(&mut rig, 2);
    rig.click(c);
    traverse(&mut rig, false);
    assert_eq!(focused_of(&mut rig), vec![false, false, false], "no wrap");
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "then from the top"
    );
}

/// **Once the keyboard has focus, the click is forgotten**: with something
/// focused, Tab is `focus_next`'s, whatever was clicked before.
#[test]
fn a_focused_control_beats_a_clicked_one() {
    let mut rig = Rig::new(three());
    let c = centre(&mut rig, 2);
    rig.click(c);
    // The application focuses a control itself, with the runtime's own
    // `focus_next`, which knows nothing of the click.
    rig.tab();
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "the application's focus"
    );
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![false, true, false],
        "Tab moved from the focused control, not from the clicked one"
    );
}

/// **A Tab run forgets the click**: click B, Tab to C, Tab past the end, and
/// the next Tab is the top, not C again. (The mark is cleared when the keyboard
/// takes over.)
#[test]
fn a_tab_run_forgets_the_click() {
    let mut rig = Rig::new(three());
    let b = centre(&mut rig, 1);
    rig.click(b);
    traverse(&mut rig, false); // C
    traverse(&mut rig, false); // past the end: nothing
    assert_eq!(focused_of(&mut rig), vec![false, false, false]);
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "the click's starting point outlived the Tab run"
    );
}

/// The premise of the ring test above, as a test of its own: Tab **does** draw
/// a ring on every control, so "a click draws none" is not true because no
/// control ever draws one.
#[test]
fn tab_to_a_control_draws_a_ring() {
    let failures = for_every_target(|_, subject, k, _| {
        let mut untouched = Rig::new(subject.element());
        untouched.redraw_requested();
        let plain = untouched.frame();
        let mut tabbed = Rig::new(subject.element());
        for _ in 0..=k {
            tabbed.tab();
        }
        tabbed.redraw_requested();
        (tabbed.frame() == plain).then(|| "Tab changed no pixel".to_owned())
    });
    assert_clean("tab-ring", failures);
}

// ---- the 0.62 audit, Unit 3 ---------------------------------------------------------------

/// **K-033: a focused snora control uses Enter and Space, and nothing else.** On every control (the
/// subjects of this file), focused by Tab: no arrow, Home, End, PageUp, PageDown, F6, Escape or
/// modified key is taken (each `Status::Ignored`) and nothing is published; Enter then presses it.
/// None of the shortcuts' keys (`Ctrl+Tab`, `Alt+↑`) is used by a control either.
#[test]
fn a_focused_snora_control_leaves_every_other_key_alone() {
    use keyboard::Modifiers as M;
    let others: [(Named, M); 14] = [
        (Named::ArrowDown, M::empty()),
        (Named::ArrowUp, M::empty()),
        (Named::ArrowLeft, M::empty()),
        (Named::ArrowRight, M::empty()),
        (Named::Home, M::empty()),
        (Named::End, M::empty()),
        (Named::PageDown, M::empty()),
        (Named::PageUp, M::empty()),
        (Named::F6, M::empty()),
        (Named::Escape, M::empty()),
        (Named::Tab, M::CTRL),
        (Named::PageDown, M::CTRL),
        (Named::ArrowUp, M::ALT),
        (Named::ArrowDown, M::SHIFT),
    ];
    let failures = for_every_target(|_, subject, k, want| {
        let mut rig = Rig::new(subject.element());
        for _ in 0..=k {
            rig.tab();
        }
        rig.redraw_requested();
        if !rig.targets()[k].focused {
            return Some("the premise failed: Tab did not focus it".to_owned());
        }
        for (key, mods) in others {
            let status = rig.press(key, mods);
            if status != iced_runtime::core::event::Status::Ignored || !rig.messages.is_empty() {
                return Some(format!(
                    "{key:?}+{mods:?} was {status:?} and published {:?}",
                    rig.messages
                ));
            }
        }
        rig.key(Named::Enter, false);
        (rig.messages != vec![want.clone()])
            .then(|| format!("Enter published {:?}, not {want:?}", rig.messages))
    });
    assert_clean("other-keys", failures);
}

/// **R-014, A-042: the engine binds neither Tab nor F6.** A rendered page, with no router and no
/// Escape field: Tab, Shift+Tab and F6 key events are `Ignored` and focus nothing. Tab moves focus
/// only where the application routes it (`snora::keyboard::traverse`); without the line, Tab does
/// nothing, as before RFC-121.
#[test]
fn the_engine_binds_neither_tab_nor_f6() {
    use keyboard::Modifiers as M;
    let mut rig = Rig::new(snora::render(snora::AppLayout::new(three())));
    for (key, mods) in [
        (Named::Tab, M::empty()),
        (Named::Tab, M::SHIFT),
        (Named::F6, M::empty()),
        (Named::F6, M::SHIFT),
    ] {
        let status = rig.press(key, mods);
        assert_eq!(
            status,
            iced_runtime::core::event::Status::Ignored,
            "{key:?}+{mods:?}"
        );
        assert_eq!(
            focused_of(&mut rig),
            vec![false, false, false],
            "{key:?}+{mods:?} moved focus"
        );
        assert!(rig.messages.is_empty(), "{:?}", rig.messages);
    }
    // The premise: the same page, with the application's routing, does move focus.
    let mut rig = Rig::new(three());
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "the premise"
    );
}

/// **R-043, A-017: Tab does not wrap, at either end, through `traverse`.** From a control focused
/// by Tab (not by a click): past the last one, nothing is focused for one press, and the next starts
/// at the top. Shift+Tab from the first: nothing, then the last.
#[test]
fn tab_past_either_end_leaves_the_group_through_traverse() {
    let mut rig = Rig::new(three());
    for _ in 0..3 {
        traverse(&mut rig, false);
    }
    assert_eq!(
        focused_of(&mut rig),
        vec![false, false, true],
        "the premise: Tab walked to the last"
    );
    traverse(&mut rig, false);
    assert_eq!(focused_of(&mut rig), vec![false, false, false], "no wrap");
    traverse(&mut rig, false);
    assert_eq!(
        focused_of(&mut rig),
        vec![true, false, false],
        "then from the top"
    );

    let mut rig = Rig::new(three());
    traverse(&mut rig, false);
    traverse(&mut rig, true);
    assert_eq!(
        focused_of(&mut rig),
        vec![false, false, false],
        "Shift+Tab before the first"
    );
    traverse(&mut rig, true);
    assert_eq!(
        focused_of(&mut rig),
        vec![false, false, true],
        "then the last"
    );
}

/// **R-086: no ring under the pointer.** The pointer rests on an unfocused control: the frame
/// differs from an untouched one only inside the control's bounds (the style's own hover look); a
/// ring lies outside them (offset 2 on both paths). The premise is asserted for the same control:
/// Tab to it **does** change pixels outside its bounds.
#[test]
fn a_pointer_resting_on_a_control_draws_no_ring() {
    let outside = |frame: &[u8], other: &[u8], b: Rectangle| -> usize {
        let w = WINDOW.width as usize;
        let (x0, x1) = (b.x as usize - 1, (b.x + b.width) as usize + 1);
        let (y0, y1) = (b.y as usize - 1, (b.y + b.height) as usize + 1);
        let mut n = 0;
        for y in 0..WINDOW.height as usize {
            for x in 0..w {
                if x >= x0 && x < x1 && y >= y0 && y < y1 {
                    continue;
                }
                let i = (y * w + x) * 4;
                if frame[i..i + 4] != other[i..i + 4] {
                    n += 1;
                }
            }
        }
        n
    };
    let failures = for_every_target(|_, subject, k, _| {
        let mut untouched = Rig::new(subject.element());
        untouched.redraw_requested();
        let plain = untouched.frame();
        let b = untouched.targets()[k].bounds;

        let mut tabbed = Rig::new(subject.element());
        for _ in 0..=k {
            tabbed.tab();
        }
        tabbed.redraw_requested();
        if outside(&tabbed.frame(), &plain, b) == 0 {
            return Some("the premise failed: Tab drew no ring outside the bounds".to_owned());
        }

        let mut hovered = Rig::new(subject.element());
        let at = hovered.targets()[k].bounds.center();
        hovered.cursor = mouse::Cursor::Available(at);
        hovered.deliver(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
        hovered.redraw_requested();
        let n = outside(&hovered.frame(), &plain, b);
        (n > 0).then(|| format!("{n} pixels outside the control differ under the pointer: a ring"))
    });
    assert_clean("hover-ring", failures);
}
