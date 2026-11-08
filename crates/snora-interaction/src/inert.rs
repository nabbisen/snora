//! [`Inert`]: a layer the keyboard cannot reach (RFC-119).
//!
//! # What it is for
//!
//! A modal stops the pointer (its dim captures the press) and not the
//! keyboard: `focus_next` walks every focusable in the window's tree, and a
//! key press reaches every layer of a `stack` that nobody captured. So with a
//! dialog open, Tab lands on a control the dim is covering and Enter presses
//! it. `Inert` wraps a layer the pointer cannot reach and, while it is inert:
//!
//! - reports **no focus target** from its content (`operate` hides
//!   `focusable`; `container`, `text`, `scrollable` and the rest still pass, so
//!   selectors and tests still find the content);
//! - delivers **no keyboard or input-method event** to its content;
//! - leaves drawing, layout and the pointer alone: pointer blocking is the
//!   backdrops' job, and an animation beneath must keep running.
//!
//! # Its public surface
//!
//! `Inert` and [`OnEscape`](crate::OnEscape) are public because the engine
//! (`snora`) builds them. They are not for applications: an application gets
//! their behaviour from `snora::render`.
//!
//! # Always in the tree
//!
//! The runtime keeps a widget's state by its place in the tree, so wrapping a
//! layer **only while a modal is open** would reshape the tree twice (on open
//! and on close) and reset the page each time: scroll offsets, a text input's
//! cursor, focus. `Inert` is therefore built around the layer **always**, and
//! switched by a flag.
//!
//! # The opener's focus
//!
//! When it becomes inert, the control that held focus in its content loses it
//! (so no ring is drawn under the dim) and `Inert` remembers which it was; when
//! it becomes reachable again, that control gets its focus back.
//!
//! **How the control is identified: by its position among the content's focus
//! targets, with their count.** Not by `widget::Id`: a `Control`'s id is made
//! when it is built, and every `view` builds again. The position survives a
//! rebuild because `Inert`'s own state does (it is always in the tree), and it
//! is checked against the count: if the layer's focus targets changed while
//! covered, nothing is restored, and focus is simply nowhere, which is what a
//! user can predict.
//!
//! **The application's own choice wins.** If anything in the layer has focus
//! when it becomes reachable again (the application closed the modal and moved
//! focus itself, say to a search field, with a focus task), nothing is
//! restored: two targets would otherwise hold focus, and the explicit choice
//! would be overridden.
//!
//! > **When a modal closes, focus returns to the control that had it, unless
//! > the page now has a different number of controls, or the application has
//! > focused something itself. Then focus is wherever the application put it,
//! > or nowhere.**
//!
//! # After a removal (RFC-120)
//!
//! `Inert` is also the **tracker** for where focus goes when the focused control
//! goes away, because it is always in the tree, keeps its state across the
//! application's rebuilds, and can run an operation over its content in
//! `update`. Nothing else can: a control's own `update` sees no sibling, and a
//! `Shell` cannot run an operation.
//!
//! > **When the focused control goes away, focus moves to its nearest neighbour
//! > in the same group: the next one, or else the previous one. Otherwise,
//! > nothing is focused.**
//!
//! - **After a press** of the focused control ([`Control`](crate::Control)
//!   marks itself when it publishes), `Inert` surveys its content's focus
//!   targets in Tab order, with the containers each sits in, and remembers the
//!   survey. (iced keeps state by position: 0.55.0 left focus on whichever
//!   control moved into the removed one's place, and one more Enter pressed it.)
//! - **At the next `RedrawRequested`**, before anything is drawn, or before an
//!   operation if one comes first: if focus is anywhere **but** the pressed
//!   control's position (where the position left it), the application put it
//!   there and it wins; otherwise that inherited focus is taken off, and if the
//!   count is unchanged, nothing went and focus returns to the same position;
//!   otherwise the targets
//!   that went are the span the control declared ([`Control::removes`]), and
//!   focus goes to the next target in the same group, or the previous.
//! - **The group** is the innermost container that held the control and is
//!   still there, and **never wider than this layer**: `Inert` surveys only its
//!   own content. With no target left in the group, or if the counts do not add
//!   up to exactly the declared removal, nothing is focused.
//!
//! If your container is left with no control in it, snora cannot tell that it is
//! still there, and treats it as gone: focus moves to the nearest control in the
//! container around it.
//!
//! **Not covered (a stated limit, RFC-116 L-7):** a removal *elsewhere* in an
//! application's list can still move focus, because nothing says which control
//! went.
//!
//! # Overlays
//!
//! iced draws overlays above every layer and delivers events to them first. A
//! tooltip or a `pick_list`'s dropdown that was open on the page when a modal
//! opened would therefore stay **above the dim** and keep taking the pointer and
//! the keys. While inert, `Inert` reports **no overlay**; the content's state is
//! untouched, so the overlay returns if it is still open when the layer is
//! uncovered.
//!
//! **When it runs: on the `RedrawRequested` event,** which the runtime sends
//! before every draw (`iced_winit` loops `update(RedrawRequested)` and then
//! `draw`; `Simulator::snapshot` does the same). `diff` has no layout and no
//! renderer, and `update` has both, so it can run the focus operation on its
//! own content directly. No frame is drawn with the ring under the dim.

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{
    Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer, touch, window,
};

/// What `Inert` remembers while its layer is covered.
#[derive(Default)]
struct State {
    /// Has the layer been made inert (and its focus parked) already?
    engaged: bool,
    /// The position of the focus target that held focus, among `count`.
    parked: Option<(usize, usize)>,
    /// A press of the focused control, waiting for the application's rebuild
    /// (RFC-120).
    pending: Option<Pending>,
}

/// What `observe` remembers across the application's rebuild.
struct Pending {
    before: Vec<Seen>,
    /// The pressed control's position in `before`.
    at: usize,
    /// How many targets go with it, before it and after it.
    removes: (usize, usize),
    /// The containers that were groups which stay ([`crate::Group`]).
    stays: Vec<usize>,
}

/// One focus target, the containers it sits in (outermost first), and what the
/// control said about its press.
#[derive(Clone)]
struct Seen {
    focused: bool,
    scopes: Vec<usize>,
    acted: bool,
    removes: (usize, usize),
}

/// Lists the focus targets in Tab order. A container is numbered by the order
/// `traverse` meets it, so a container that begins before a removal keeps its
/// number across it. Reads and **clears** each control's press mark.
#[derive(Default)]
struct Track {
    seen: Vec<Seen>,
    stack: Vec<usize>,
    next: usize,
    /// The next container opened is a [`crate::Group`], which stays.
    group_next: bool,
    /// The containers that are groups which stay (by their number).
    stays: Vec<usize>,
}

impl Operation for Track {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        if std::mem::take(&mut self.group_next) {
            self.stays.push(self.next);
        }
        self.stack.push(self.next);
        self.next += 1;
        operate(self);
        self.stack.pop();
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.seen.push(Seen {
            focused: state.is_focused(),
            scopes: self.stack.clone(),
            acted: false,
            removes: (0, 0),
        });
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if state.is::<crate::group::GroupMark>() {
            self.group_next = true;
            return;
        }
        // A control reports its mark right after its focus target.
        if let (Some(mark), Some(last)) = (
            state.downcast_mut::<crate::control::PressMark>(),
            self.seen.last_mut(),
        ) {
            last.acted = mark.acted;
            last.removes = mark.removes;
            mark.acted = false;
        }
    }
}

/// A layer that the keyboard cannot reach while `inert` is set.
pub struct Inert<'a, Message> {
    content: Element<'a, Message>,
    inert: bool,
}

impl<'a, Message> Inert<'a, Message> {
    /// `content`, unreachable by the keyboard while `inert` is true.
    pub fn new(content: impl Into<Element<'a, Message>>, inert: bool) -> Self {
        Self {
            content: content.into(),
            inert,
        }
    }
}

/// An operation that does not see focus targets.
struct Hide<'a>(&'a mut dyn Operation);

impl Operation for Hide<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        self.0.traverse(&mut |inner| {
            let mut hidden = Hide(inner);
            operate(&mut hidden);
        });
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.0.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        self.0
            .scrollable(id, bounds, content_bounds, translation, state);
    }

    // `focusable` is the one thing not forwarded.

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn iced_core::widget::operation::TextInput,
    ) {
        self.0.text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.0.text(id, bounds, text);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        self.0.custom(id, bounds, state);
    }
}

/// Counts the focus targets, and finds the focused one.
#[derive(Default)]
struct Survey {
    seen: usize,
    focused: Option<usize>,
}

impl Operation for Survey {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() && self.focused.is_none() {
            self.focused = Some(self.seen);
        }
        self.seen += 1;
    }
}

/// Takes focus from every target (so none is left focused under the dim).
pub(crate) struct Unfocus;

impl Operation for Unfocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        state.unfocus();
    }
}

/// Gives focus to the target at one position.
pub(crate) struct Refocus {
    pub(crate) at: usize,
    pub(crate) seen: usize,
}

impl Operation for Refocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.seen == self.at {
            state.focus();
        }
        self.seen += 1;
    }
}

/// The nearest neighbour **in the same group**: the target after the removed
/// span if there is one, else the one before. `None` is "nothing is focused".
///
/// The group is the innermost container of the pressed control that is not
/// wholly inside what went; it is never widened. If its targets do not add up
/// to exactly the declared removal, snora cannot tell what went, and nothing is
/// focused.
fn nearest_in_group(
    before: &[Seen],
    stays: &[usize],
    span: std::ops::Range<usize>,
    after: &[Seen],
) -> Option<usize> {
    let mut scopes = before.get(span.start)?.scopes.clone();
    scopes.reverse();
    for group in scopes {
        let held = |seen: &[Seen]| -> Vec<usize> {
            (0..seen.len())
                .filter(|i| seen[*i].scopes.contains(&group))
                .collect()
        };
        let was = held(before);
        // A container that sat wholly inside what went is not there to look in
        // **unless it is a group that stays** (the breadcrumb bar): that one is
        // still on the page, and with nothing left in it, nothing is focused.
        if was.iter().all(|i| span.contains(i)) && !stays.contains(&group) {
            continue;
        }
        let now = held(after);
        let gone = was.iter().filter(|i| span.contains(i)).count();
        if now.len() + gone != was.len() {
            return None;
        }
        let first = was.iter().filter(|i| **i < span.start).count();
        return if first < now.len() {
            Some(now[first])
        } else if first > 0 {
            Some(now[first - 1])
        } else {
            None
        };
    }
    None
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Inert<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        // Called by path: `scripts/check-literal-sizes.sh` reads a zero-argument
        // call of the accessor's name as a text size.
        Widget::size(self.content.as_widget())
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    /// The content's own node: `Inert` has no box of its own.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        // An operation (a Tab, a focus task) can arrive after the rebuild and
        // before the redraw that would resolve a pending removal, if the key
        // events came in one batch. It must see where focus has gone, not the
        // gap, so the pending removal is resolved first (RFC-120).
        self.resolve(tree, layout, renderer);

        if self.inert {
            let mut hidden = Hide(operation);
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut hidden,
            );
        } else {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.sync(tree, layout, renderer);
            self.resolve(tree, layout, renderer);
        }

        // The keyboard does not reach a covered layer, and neither does text
        // input. Everything else does: window events, redraws, the pointer.
        if self.inert && matches!(event, Event::Keyboard(_) | Event::InputMethod(_)) {
            return;
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        // Only a press can make a control go away, and only these events
        // publish one: Enter or Space (not a repeat), a release, a lift.
        let publishes = match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, repeat, .. }) => {
                !*repeat
                    && matches!(
                        key.as_ref(),
                        keyboard::Key::Named(
                            keyboard::key::Named::Enter | keyboard::key::Named::Space
                        )
                    )
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => true,
            _ => false,
        };
        if publishes && !self.inert {
            self.observe(tree, layout, renderer);
        }
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        // A covered layer has no overlay: it would float above the dim.
        if self.inert {
            return None;
        }

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<Message> Inert<'_, Message> {
    /// After a press: if the **focused** control was the one pressed, remember
    /// the survey (RFC-120). Focus is left alone until the rebuild, so a second
    /// press in the same batch of events still finds its control focused.
    ///
    /// A press of a control that is not focused (a click, which no longer
    /// focuses) is not the focused control going away, and is left alone: that
    /// is the stated limit.
    fn observe(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let mut track = Track::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut track);
        let Some(at) = track.seen.iter().position(|s| s.focused && s.acted) else {
            return;
        };
        let removes = track.seen[at].removes;
        tree.state.downcast_mut::<State>().pending = Some(Pending {
            before: track.seen,
            at,
            removes,
            stays: track.stays,
        });
    }

    /// At the redraw after the application's rebuild: where focus goes now
    /// (RFC-120).
    fn resolve(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let Some(pending) = tree.state.downcast_mut::<State>().pending.take() else {
            return;
        };
        if self.inert {
            return;
        }
        let mut track = Track::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut track);
        let after = track.seen;

        // Focus is either where iced's positional state left it (on the pressed
        // control's position, or nowhere), which this replaces, or somewhere
        // else, which only the application can have put there (a focus task,
        // a click into a text input): its choice wins.
        if after
            .iter()
            .enumerate()
            .any(|(i, s)| s.focused && i != pending.at)
        {
            return;
        }
        let to = if after.len() == pending.before.len() {
            // Nothing went: the control is still there.
            Some(pending.at)
        } else {
            let (bef, aft) = pending.removes;
            pending
                .at
                .checked_sub(bef)
                .and_then(|start| {
                    let end = pending.at + aft + 1;
                    (end <= pending.before.len()).then_some(start..end)
                })
                .and_then(|span| nearest_in_group(&pending.before, &pending.stays, span, &after))
        };
        // Whatever focus the position inherited goes, then the neighbour gets it.
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut Unfocus);
        if let Some(at) = to {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut Refocus { at, seen: 0 },
            );
        }
    }

    /// Brings the content's focus in line with the flag: park it on the way
    /// in, give it back on the way out.
    fn sync(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let state = tree.state.downcast_mut::<State>();

        if self.inert && !state.engaged {
            let mut survey = Survey::default();
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut survey,
            );
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut Unfocus,
            );
            let state = tree.state.downcast_mut::<State>();
            state.parked = survey.focused.map(|at| (at, survey.seen));
            state.engaged = true;
        } else if !self.inert && state.engaged {
            let parked = state.parked.take();
            state.engaged = false;
            if let Some((at, count)) = parked {
                let mut survey = Survey::default();
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut survey,
                );
                // Only if the layer still has the same focus targets (a
                // position means nothing in a different layer), and only if
                // the application has not focused something itself.
                if survey.seen == count && survey.focused.is_none() {
                    self.content.as_widget_mut().operate(
                        &mut tree.children[0],
                        layout,
                        renderer,
                        &mut Refocus { at, seen: 0 },
                    );
                }
            }
        }
    }
}

impl<'a, Message: 'a> From<Inert<'a, Message>> for Element<'a, Message> {
    fn from(inert: Inert<'a, Message>) -> Self {
        Element::new(inert)
    }
}

#[cfg(test)]
mod tests;
