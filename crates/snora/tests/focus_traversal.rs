//! Tab moves focus through the sidebar rail, end to end (RFC-116 R-1
//! Unit 1, item 3).
//!
//! # The gap this closes
//!
//! Unit 0 asserted "Tab reaches the item" as two halves — the rail
//! registers focus targets, and `keyboard::traverse` maps Tab to
//! `focus_next` — and named the **composition** as untested: that
//! running `focus_next` against the rail actually moves focus from one
//! item to the next. Its stated reason was that `iced_test::Simulator`
//! has no runtime and a `Selector` sees focus read-only.
//!
//! That reason was about the Simulator. It was not about iced. The
//! runtime's own `UserInterface` is public, and `operate` takes any
//! `Operation`, so a test can build the same interface the window loop
//! builds and run the same operations against it. This does, and the
//! only thing it does not cover is the key-to-operation mapping, which
//! `keyboard::traverse`'s own unit test already does.
//!
//! # Two-pass operations, run to completion
//!
//! `focus_next` and `focus_previous` are each `then(count(), ..)`: the
//! first `finish()` returns `Outcome::Chain(next)` and **nothing has
//! been focused yet**. The window loop keeps running until the chain
//! ends (`iced_winit-0.14.0/src/lib.rs`, `Action::Widget`), so
//! [`Rig::run`] does too. A helper that ran one pass would report "Tab
//! did nothing", which is a fault in the helper that looks exactly like
//! one in the widget.
//!
//! # The renderer
//!
//! Building a `UserInterface` needs a renderer. These tests measure
//! focus, not pixels, so they always use tiny-skia, which needs no GPU,
//! and do **not** follow `ICED_TEST_BACKEND`. **Any wgpu sweep added to
//! this file must run with `--test-threads=1`**: Unit 0 found that
//! parallel wgpu contexts segfault, and the wgpu CI job already passes
//! that flag.

#![cfg(feature = "widgets")]

use iced::widget::{row, space};
use iced::{Element, Font, Length, Pixels, Size, Theme};
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Rectangle, renderer::Headless, widget::operation::Focusable};
use iced_runtime::user_interface::{self, UserInterface};

use snora::widget::SideBarLabels;
use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

const NAMES: [&str; 3] = ["Home", "Search", "Settings"];

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go(u8),
}

fn rail(labels: SideBarLabels) -> Element<'static, Msg> {
    let bar = SideBar {
        items: NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| SideBarItem {
                view_id: i as u8,
                icon: Icon::Text(i.to_string()),
                tooltip: (*name).to_owned(),
                on_press: Msg::Go(i as u8),
            })
            .collect(),
        active: 0,
    };
    row![
        snora::widget::app_side_bar_with(bar, LayoutDirection::Ltr, labels),
        space().width(Length::Fill)
    ]
    .into()
}

fn forms() -> [(&'static str, SideBarLabels); 3] {
    [
        ("Beneath", SideBarLabels::Beneath),
        ("Beside", SideBarLabels::Beside),
        ("Tooltip", SideBarLabels::Tooltip),
    ]
}

/// The runtime's interface around the rail, and a renderer to operate
/// it with.
struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
}

impl Rig {
    fn new(labels: SideBarLabels) -> Self {
        let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::DEFAULT,
            Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("a tiny-skia renderer needs no GPU");

        let ui = UserInterface::build(
            rail(labels),
            Size::new(600.0, 400.0),
            user_interface::Cache::default(),
            &mut renderer,
        );
        Self { ui, renderer }
    }

    /// Runs an operation to completion, as the window loop does.
    fn run<T: 'static>(&mut self, operation: impl Operation<T> + 'static) -> Option<T> {
        let mut current: Box<dyn Operation<T>> = Box::new(operation);
        loop {
            self.ui
                .operate(&self.renderer, &mut operation::black_box(current.as_mut()));
            match current.finish() {
                Outcome::None => return None,
                Outcome::Some(value) => return Some(value),
                Outcome::Chain(next) => current = next,
            }
        }
    }

    /// Every focus target in tree order, as `(id, is_focused)`.
    fn focusables(&mut self) -> Vec<(Option<Id>, bool)> {
        let mut found = Collect::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
    }

    fn focused_flags(&mut self) -> Vec<bool> {
        self.focusables().into_iter().map(|(_, f)| f).collect()
    }

    /// The id of the focused target, as the runtime's own search finds it.
    fn find_focused(&mut self) -> Option<Id> {
        self.run(focusable::find_focused())
    }
}

#[derive(Default)]
struct Collect(Vec<(Option<Id>, bool)>);

impl Operation for Collect {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.0.push((id.cloned(), state.is_focused()));
    }
}

/// **Tab, Tab, Shift+Tab** through a three-item rail, in every form.
///
/// Each step is checked two ways, which is the point of having both:
/// the per-item flags say *which* item holds focus, and the runtime's
/// own `find_focused` returns its id — so a rail that focused the right
/// item but registered the wrong id would still fail.
#[test]
fn tab_moves_focus_down_the_rail_and_shift_tab_moves_it_back() {
    for (form, labels) in forms() {
        let mut rig = Rig::new(labels);

        let targets = rig.focusables();
        assert_eq!(
            targets.len(),
            NAMES.len(),
            "{form}: the rail offers {} focus target(s) for {} items — Tab has nothing to land on",
            targets.len(),
            NAMES.len(),
        );
        let ids: Vec<Id> = targets
            .iter()
            .map(|(id, _)| {
                id.clone()
                    .unwrap_or_else(|| panic!("{form}: a target has no id"))
            })
            .collect();
        assert_eq!(
            rig.find_focused(),
            None,
            "{form}: something is focused before any Tab"
        );

        // Tab: the first item.
        let _ = rig.run(focusable::focus_next::<()>());
        assert_eq!(
            rig.focused_flags(),
            vec![true, false, false],
            "{form}: first Tab"
        );
        assert_eq!(
            rig.find_focused().as_ref(),
            Some(&ids[0]),
            "{form}: first Tab focused something, but not the first item"
        );

        // Tab again: the second.
        let _ = rig.run(focusable::focus_next::<()>());
        assert_eq!(
            rig.focused_flags(),
            vec![false, true, false],
            "{form}: second Tab"
        );
        assert_eq!(
            rig.find_focused().as_ref(),
            Some(&ids[1]),
            "{form}: second Tab did not land on the second item"
        );

        // Shift+Tab: back to the first.
        let _ = rig.run(focusable::focus_previous::<()>());
        assert_eq!(
            rig.focused_flags(),
            vec![true, false, false],
            "{form}: Shift+Tab"
        );
        assert_eq!(
            rig.find_focused().as_ref(),
            Some(&ids[0]),
            "{form}: Shift+Tab did not return to the first item"
        );
    }
}

/// A rail whose items are not focus targets cannot be tabbed through.
///
/// This is the control for the test above: it fails there for exactly
/// the reason this one passes, and it checks the harness is not
/// reporting focus that is not there.
#[test]
fn an_empty_tree_has_nothing_to_focus() {
    let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        Font::DEFAULT,
        Pixels(16.0),
        Some("tiny-skia"),
    ))
    .expect("a tiny-skia renderer needs no GPU");
    let ui = UserInterface::build(
        space().width(Length::Fill).height(Length::Fill),
        Size::new(600.0, 400.0),
        user_interface::Cache::default(),
        &mut renderer,
    );
    let mut rig = Rig { ui, renderer };

    let _ = rig.run(focusable::focus_next::<()>());
    assert_eq!(rig.focusables().len(), 0);
    assert_eq!(
        rig.find_focused(),
        None,
        "focus_next found focus in a tree with no targets"
    );
}
