//! The keyboard and an open modal (RFC-119).
//!
//! # Unit 0: the finding, reproduced
//!
//! From RFC-116 R-1 every control snora draws is a focus target, and an
//! application that routes Tab with `snora::keyboard::traverse` can reach
//! **controls beneath an open dialog or sheet**, and press them with Enter. The
//! modal's dim stops the pointer and not the keyboard: `focus_next` walks every
//! focusable in the window's tree, and a key press reaches every layer of a
//! `stack` that nobody captured.
//!
//! These tests state what must be true and **fail on `main` today**, which is
//! the reproduction: the failure message names the control that was reached and
//! the message that was published. They are `#[ignore]`d so the branch's CI stays
//! green until Unit 1 lands; run them with `-- --ignored`. They are the file RFC-119's Unit 1 grows
//! (and `render_semantics.rs`, gate 5, stays untouched).
//!
//! Driven through the runtime's own `UserInterface` as RFC-116 R-1 Unit 1 did:
//! Tab is the runtime's `focus_next` run to completion, Enter a real key event.
//!
//! The renderer is tiny-skia always: these measure focus, not pixels.

#![cfg(all(feature = "widgets", feature = "design"))]

use iced::{Element, Font, Pixels, Size, Theme, mouse};
use iced_runtime::core::clipboard;
use iced_runtime::core::renderer::Headless;
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::user_interface::{self, UserInterface};

use snora::design::{Tokens, button};
use snora::{AppLayout, Dialog, render};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    /// The page's control: "Delete", the action a confirmation dialog gates.
    Delete,
    DialogOk,
    CloseModals,
}

const WINDOW: Size = Size::new(800.0, 600.0);

fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<Tokens> = OnceLock::new();
    T.get_or_init(Tokens::light)
}

/// A skeleton with one control, and a dialog over it with one control of its
/// own, when `open`.
fn window(open: bool) -> Element<'static, Msg> {
    let body = button::primary(tokens(), "Delete", Msg::Delete);
    let mut layout = AppLayout::new(body);
    if open {
        let ok: Element<'static, Msg> = button::primary(tokens(), "OK", Msg::DialogOk);
        layout = layout
            .dialog(Dialog::new(ok))
            .on_close_modals(Msg::CloseModals);
    }
    render(layout)
}

struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    messages: Vec<Msg>,
    cursor: mouse::Cursor,
}

impl Rig {
    fn new(root: Element<'static, Msg>) -> Self {
        let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::DEFAULT,
            Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("a tiny-skia renderer needs no GPU");
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

    /// What an application's `view` does after `update`.
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

    fn enter(&mut self) {
        let _ = self.ui.update(
            &[iced_test::simulator::press_key(
                iced::keyboard::key::Named::Enter,
                None,
            )],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    /// `focus_next`, run to completion as the window loop does (it is
    /// two-pass: one pass focuses nothing).
    fn tab(&mut self) {
        let mut current: Box<dyn Operation<()>> = Box::new(focusable::focus_next::<()>());
        loop {
            self.ui
                .operate(&self.renderer, &mut operation::black_box(current.as_mut()));
            match current.finish() {
                Outcome::None | Outcome::Some(_) => return,
                Outcome::Chain(next) => current = next,
            }
        }
    }

    /// The focus targets the tree reports, in tree order.
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
                _bounds: iced::Rectangle,
                state: &mut dyn operation::Focusable,
            ) {
                self.0.push((id.cloned(), state.is_focused()));
            }
        }
        let mut found = Collect::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
    }
}

/// **With a dialog open, Tab never lands on the page beneath it.**
///
/// Tab, then Enter, as many times as there are focus targets and one more: if
/// any press publishes the page's `Delete`, the keyboard reached a control the
/// dim was covering.
#[test]
#[ignore = "RFC-119 Unit 0: reproduces the finding, fails until Unit 1 (run with --ignored)"]
fn tab_never_reaches_a_control_beneath_an_open_dialog() {
    let mut rig = Rig::new(window(true));
    let reported = rig.targets().len();
    let mut trail = Vec::new();
    for step in 0..=reported + 1 {
        rig.tab();
        rig.enter();
        trail.push(format!("Tab {}: {:?}", step + 1, rig.messages.last()));
    }
    assert!(
        !rig.messages.contains(&Msg::Delete),
        "with a dialog open, Tab reached the page's \"Delete\" control and Enter pressed it. \
         The tree reports {reported} focus targets (a dialog has one); the presses went:\n  {}",
        trail.join("\n  "),
    );
}

/// **A control that had focus before the dialog opened is not pressed by
/// Enter once it is open.** The page's control is focused, a dialog opens, and
/// Enter is pressed: the dialog was meant to gate exactly that action.
#[test]
#[ignore = "RFC-119 Unit 0: reproduces the finding, fails until Unit 1 (run with --ignored)"]
fn enter_does_not_press_a_page_control_focused_before_the_dialog_opened() {
    let mut rig = Rig::new(window(false));
    rig.tab();
    assert_eq!(
        rig.targets().iter().map(|t| t.1).collect::<Vec<_>>(),
        vec![true],
        "the premise: the page's control has focus"
    );

    let mut rig = rig.rebuilt(window(true));
    rig.enter();
    assert!(
        rig.messages.is_empty(),
        "a dialog is open, and Enter published {:?} from the page beneath it",
        rig.messages
    );
}
