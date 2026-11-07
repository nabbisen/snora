//! `control`'s behaviour, driven through the runtime's own
//! `UserInterface` (RFC-116 R-1 Unit 1).
//!
//! # Why `UserInterface` and not a hand-built `Shell`
//!
//! The window loop does not look at a widget's `Shell`. It calls
//! `UserInterface::update`, which folds every widget's redraw
//! request into one `State::Updated { redraw_request, .. }`, and it
//! acts on that (`iced_winit-0.14.0/src/lib.rs`: `window.request_redraw(
//! _redraw_request)`). So that is the thing asserted here: not "the
//! widget called a method" but "the runtime was told".
//!
//! # The renderer is tiny-skia, always
//!
//! Building a `UserInterface` needs a renderer, and these tests
//! measure behaviour, not pixels, so the one that needs no GPU is
//! used regardless of `ICED_TEST_BACKEND`. **Any wgpu sweep added
//! here must run with `--test-threads=1`**: Unit 0 found that
//! parallel wgpu contexts segfault.

use super::*;
use iced::widget::{row, space, text};
use iced::{Font, Point, Settings, window};
use iced_core::clipboard;
use iced_core::widget::operation::Outcome;
use iced_core::window::RedrawRequest;
use iced_runtime::user_interface::{self, State, UserInterface};
use iced_test::Simulator;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    A,
    B,
}

/// A headless renderer that needs no GPU.
fn headless() -> iced::Renderer {
    use iced_core::renderer::Headless;
    iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        Font::DEFAULT,
        Pixels(16.0),
        Some("tiny-skia"),
    ))
    .expect("a tiny-skia renderer needs no GPU")
}

fn plain_ring() -> Ring<'static> {
    Ring {
        color: Box::new(|_| Color::BLACK),
        width: 2.0,
        offset: 0.0,
    }
}

fn plain_style() -> StyleFn<'static> {
    Box::new(|_theme, _status| button::Style::default())
}

/// A 40 x 30 control with fixed-size content, so its position in a
/// row is exact.
fn control(message: Msg) -> Control<'static, Msg> {
    Control::new(
        space()
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(30.0)),
        plain_style(),
        plain_ring(),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(30.0))
    .on_press(message)
}

/// A control with no press handler: disabled, as `button` is.
fn disabled() -> Control<'static, Msg> {
    Control::new(
        space()
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(30.0)),
        plain_style(),
        plain_ring(),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(30.0))
    .on_press_maybe(None)
}

/// The runtime's user interface around some root, and the state the
/// window loop would keep beside it.
struct Rig {
    ui: UserInterface<'static, Msg, Theme, iced::Renderer>,
    renderer: iced::Renderer,
    messages: Vec<Msg>,
    cursor: mouse::Cursor,
}

impl Rig {
    fn new(root: impl Into<Element<'static, Msg>>) -> Self {
        let mut renderer = headless();
        let ui = UserInterface::build(
            root.into(),
            Size::new(400.0, 100.0),
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

    /// Rebuilds around a different root, **keeping the widget
    /// tree's state** — what happens every time an application's
    /// `view` runs.
    fn rebuilt(self, root: impl Into<Element<'static, Msg>>) -> Self {
        let mut renderer = self.renderer;
        let ui = UserInterface::build(
            root.into(),
            Size::new(400.0, 100.0),
            self.ui.into_cache(),
            &mut renderer,
        );
        Self {
            ui,
            renderer,
            messages: self.messages,
            cursor: self.cursor,
        }
    }

    /// Delivers events at the current cursor, returning what the
    /// window loop would be told about redrawing.
    fn update(&mut self, events: &[Event]) -> RedrawRequest {
        let (state, _) = self.ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        match state {
            State::Updated { redraw_request, .. } => redraw_request,
            State::Outdated => {
                panic!("the events invalidated the interface; this test measures redraws")
            }
        }
    }

    fn move_to(&mut self, x: f32, y: f32) -> RedrawRequest {
        let position = Point::new(x, y);
        self.cursor = mouse::Cursor::Available(position);
        self.update(&[Event::Mouse(mouse::Event::CursorMoved { position })])
    }

    /// A frame is drawn: the runtime sends this before every draw, and
    /// it is when a widget records what it is about to look like.
    fn frame(&mut self) -> RedrawRequest {
        self.update(&[Event::Window(window::Event::RedrawRequested(
            iced_core::time::Instant::now(),
        ))])
    }

    fn press(&mut self) -> RedrawRequest {
        self.update(&[Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        ))])
    }

    fn release(&mut self) -> Option<RedrawRequest> {
        let (state, _) = self.ui.update(
            &[Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        match state {
            State::Updated { redraw_request, .. } => Some(redraw_request),
            State::Outdated => None,
        }
    }

    fn key(&mut self, key: iced::keyboard::key::Named) -> Option<RedrawRequest> {
        let (state, _) = self.ui.update(
            &[iced_test::simulator::press_key(key, None)],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        match state {
            State::Updated { redraw_request, .. } => Some(redraw_request),
            State::Outdated => None,
        }
    }

    /// Every focus target in the tree, in tree order, as
    /// `(id, is_focused)`.
    fn focusables(&mut self) -> Vec<(Option<Id>, bool)> {
        let mut found = Focusables::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
    }

    /// Runs an operation **to completion**, as the window loop does.
    ///
    /// `focus_next` and `focus_previous` are two-pass: each is
    /// `then(count(), ..)`, so the first `finish()` returns
    /// `Outcome::Chain(next)` and nothing has been focused yet. The
    /// runtime loops until the chain ends
    /// (`iced_winit-0.14.0/src/lib.rs`, `Action::Widget`); a helper
    /// that ran one pass would report "Tab did nothing", which is a
    /// fault in the helper and looks exactly like one in the widget.
    fn run<T: 'static>(&mut self, operation: impl Operation<T> + 'static) -> Option<T> {
        let mut current: Box<dyn Operation<T>> = Box::new(operation);
        loop {
            self.ui.operate(
                &self.renderer,
                &mut iced_core::widget::operation::black_box(current.as_mut()),
            );
            match current.finish() {
                Outcome::None => return None,
                Outcome::Some(value) => return Some(value),
                Outcome::Chain(next) => current = next,
            }
        }
    }
}

/// Collects every focus target the tree registers.
#[derive(Default)]
struct Focusables(Vec<(Option<Id>, bool)>);

impl Operation for Focusables {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.0.push((id.cloned(), state.is_focused()));
    }
}

const A: (f32, f32) = (20.0, 15.0);
const B: (f32, f32) = (60.0, 15.0);
const NOWHERE: (f32, f32) = (300.0, 80.0);

fn two_controls() -> Rig {
    Rig::new(row![control(Msg::A), control(Msg::B)])
}

fn focused(rig: &mut Rig) -> Vec<bool> {
    rig.focusables().into_iter().map(|(_, f)| f).collect()
}

// -----------------------------------------------------------------
// 1 — the runtime is told to redraw when the control's look changes
// -----------------------------------------------------------------

/// Hovering changes how the control draws, so the runtime must be
/// told. `button` caches its status and asks for a redraw when it
/// differs; without that a hover colour does not appear until
/// something else happens to redraw the window.
#[test]
fn hovering_asks_the_runtime_for_a_redraw() {
    let mut rig = Rig::new(control(Msg::A));

    // A frame is drawn with the cursor away: the control records
    // that it looks idle.
    rig.frame();

    // The cursor enters. The status is now different from what was
    // last drawn, so the runtime has to be told.
    assert_eq!(
        rig.move_to(A.0, A.1),
        RedrawRequest::NextFrame,
        "the cursor entered the control and the runtime was not asked to redraw, so a \\
         hover colour would not appear until something else redrew the window",
    );
}

/// **The other direction, which is what stops "always ask" passing.**
/// A redraw requested on every event would burn a frame per mouse
/// move for nothing, and would satisfy the test above.
#[test]
fn an_unchanged_look_asks_for_nothing() {
    let mut rig = Rig::new(control(Msg::A));
    rig.frame();
    rig.move_to(A.0, A.1);
    // The hover is drawn: the control now records that it looks hovered.
    rig.frame();

    assert_eq!(
        rig.move_to(A.0 + 1.0, A.1),
        RedrawRequest::Wait,
        "moving within the control changed nothing it draws, yet the runtime was asked \\
         to redraw — that is a frame per mouse move",
    );
}

/// Leaving changes the look back, so it asks too.
#[test]
fn leaving_asks_the_runtime_for_a_redraw() {
    let mut rig = Rig::new(control(Msg::A));
    rig.move_to(A.0, A.1);
    rig.frame();
    assert_eq!(
        rig.move_to(NOWHERE.0, NOWHERE.1),
        RedrawRequest::NextFrame,
        "the cursor left a hovered control and the runtime was not asked to redraw",
    );
}

// -----------------------------------------------------------------
// 1b — focus changes made inside `update` also ask
// -----------------------------------------------------------------

/// Clicking elsewhere takes focus away from a control, and the
/// ring has to disappear.
///
/// **Isolated from the status path on purpose:** the cursor is
/// parked away from the control before the click, so its status
/// (idle) does not change. Only the focus logic can ask for this
/// redraw, which is what makes the assertion about it.
#[test]
fn losing_focus_to_a_click_elsewhere_asks_for_a_redraw() {
    let mut rig = Rig::new(control(Msg::A));
    rig.move_to(A.0, A.1);
    rig.press();
    rig.release();
    // `release` published, so rebuild as the application would.
    let mut rig = rig.rebuilt(control(Msg::A));
    assert_eq!(
        focused(&mut rig),
        vec![true],
        "the click focused the control"
    );

    rig.move_to(NOWHERE.0, NOWHERE.1);
    rig.frame();

    assert_eq!(
        rig.press(),
        RedrawRequest::NextFrame,
        "a click elsewhere took focus from the control, but the runtime was not asked to \\
         redraw, so the ring would stay painted",
    );
    assert_eq!(
        focused(&mut rig),
        vec![false],
        "the click elsewhere did not unfocus it"
    );
}

// -----------------------------------------------------------------
// 1c — at most one control is focused
// -----------------------------------------------------------------

/// **A defect Unit 0 had and its tests could not see.** Clicking A
/// then B left *both* focused: a press focused the control it hit
/// and nothing unfocused the one it had left. `Focusable` assumes
/// one focus at a time, and the focus operations enforce it — but a
/// click is not an operation.
#[test]
fn clicking_one_control_then_another_leaves_only_the_second_focused() {
    let mut rig = two_controls();

    rig.move_to(A.0, A.1);
    rig.press();
    rig.release();
    let mut rig = rig.rebuilt(row![control(Msg::A), control(Msg::B)]);
    assert_eq!(focused(&mut rig), vec![true, false]);

    rig.move_to(B.0, B.1);
    rig.press();
    rig.release();
    let mut rig = rig.rebuilt(row![control(Msg::A), control(Msg::B)]);

    assert_eq!(
        focused(&mut rig),
        vec![false, true],
        "after clicking A and then B, the focus state of (A, B) is not (false, true)",
    );
}

// -----------------------------------------------------------------
// 2 — disabled: on_press_maybe(None)
// -----------------------------------------------------------------

/// A control with no handler ignores the pointer.
#[test]
fn a_disabled_control_ignores_a_click() {
    let mut rig = Rig::new(disabled());
    rig.move_to(A.0, A.1);
    rig.press();
    rig.release();
    assert!(
        rig.messages.is_empty(),
        "a disabled control published {:?}",
        rig.messages
    );
}

/// **Not a focus target.** Reaching a control that does nothing is
/// the confusion the owner's rule forbids: Tab would land on it and
/// show a ring around something that cannot be pressed.
#[test]
fn a_disabled_control_is_not_a_focus_target() {
    let mut rig = Rig::new(row![disabled(), control(Msg::B)]);
    assert_eq!(
        rig.focusables().len(),
        1,
        "a disabled control registered as a focus target, so Tab would stop on it",
    );
}

/// Tab skips it: the one enabled control is what `focus_next` finds.
#[test]
fn tab_skips_a_disabled_control() {
    let mut rig = Rig::new(row![disabled(), control(Msg::B), disabled()]);
    let _ = rig.run(iced_core::widget::operation::focusable::focus_next::<()>());
    let state = focused(&mut rig);
    assert_eq!(
        state,
        vec![true],
        "focus_next did not land on the one enabled control"
    );
}

/// A disabled control is not left holding focus, and Enter does not
/// press it — even if it had focus a moment ago.
#[test]
fn a_control_that_becomes_disabled_drops_focus_and_ignores_keys() {
    let mut rig = Rig::new(control(Msg::A));
    rig.move_to(A.0, A.1);
    rig.press();
    rig.release();
    let mut rig = rig.rebuilt(control(Msg::A));
    assert_eq!(focused(&mut rig), vec![true]);
    rig.messages.clear();

    // The application rebuilds it with no handler.
    let mut rig = rig.rebuilt(disabled());
    rig.frame();
    let _ = rig.key(iced::keyboard::key::Named::Enter);
    assert!(
        rig.messages.is_empty(),
        "a control that became disabled while focused still answered Enter: {:?}",
        rig.messages,
    );

    // And re-enabling does not resurrect the old focus.
    let mut rig = rig.rebuilt(control(Msg::A));
    assert_eq!(
        focused(&mut rig),
        vec![false],
        "the control kept a stale focus through being disabled",
    );
}

// -----------------------------------------------------------------
// 2 — on_press_with builds the message only on press
// -----------------------------------------------------------------

fn counting(count: &Rc<Cell<usize>>) -> Control<'static, Msg> {
    let count = Rc::clone(count);
    Control::new(
        space()
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(30.0)),
        plain_style(),
        plain_ring(),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(30.0))
    .on_press_with(move || {
        count.set(count.get() + 1);
        Msg::A
    })
}

/// The closure is not called while the control is drawn, hovered or
/// focused — only when it is pressed.
#[test]
fn on_press_with_builds_the_message_only_on_press() {
    let count = Rc::new(Cell::new(0));
    let mut rig = Rig::new(counting(&count));

    rig.frame();
    rig.move_to(A.0, A.1);
    rig.frame();
    rig.press();
    assert_eq!(
        count.get(),
        0,
        "the message was built before anything was pressed"
    );

    rig.release();
    assert_eq!(
        count.get(),
        1,
        "a click built the message {} times",
        count.get()
    );
    assert_eq!(
        rig.messages,
        vec![Msg::A],
        "and the message it built was not published"
    );

    // The keyboard route builds it too, once per press.
    rig.messages.clear();
    let mut rig = rig.rebuilt(counting(&count));
    let _ = rig.key(iced::keyboard::key::Named::Enter);
    assert_eq!(
        count.get(),
        2,
        "Enter did not build the message exactly once"
    );
    assert_eq!(rig.messages, vec![Msg::A]);
}

/// A control built with `on_press_with` is enabled: it is a focus
/// target, unlike one built with `on_press_maybe(None)`.
#[test]
fn on_press_with_is_an_enabled_control() {
    let count = Rc::new(Cell::new(0));
    let mut rig = Rig::new(counting(&count));
    assert_eq!(rig.focusables().len(), 1);
}

// -----------------------------------------------------------------
// 2 — clip
// -----------------------------------------------------------------

/// What a control's content is told about the area it may draw in.
struct ViewportProbe {
    seen: Rc<Cell<Option<Rectangle>>>,
}

impl Widget<Msg, Theme, iced::Renderer> for ViewportProbe {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(200.0), Length::Fixed(30.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Deliberately wider than the control it sits in.
        layout::Node::new(limits.resolve(Length::Fixed(200.0), Length::Fixed(30.0), Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.seen.set(Some(*viewport));
    }
}

fn probe_in_control(clip: bool, seen: &Rc<Cell<Option<Rectangle>>>) -> Control<'static, Msg> {
    let probe: Element<'static, Msg> = Element::new(ViewportProbe {
        seen: Rc::clone(seen),
    });
    Control::new(probe, plain_style(), plain_ring())
        .width(Length::Fixed(40.0))
        .height(Length::Fixed(30.0))
        .on_press(Msg::A)
        .clip(clip)
}

fn viewport_seen(clip: bool) -> Rectangle {
    let seen = Rc::new(Cell::new(None));
    let mut rig = Rig::new(probe_in_control(clip, &seen));
    rig.frame();
    rig.ui.draw(
        &mut rig.renderer,
        &Theme::Light,
        &renderer::Style {
            text_color: Color::BLACK,
        },
        rig.cursor,
    );
    seen.get().expect("the content was drawn")
}

/// With `clip`, the content is drawn inside the control's own
/// bounds; without it, inside the whole window. **Content wider than
/// the control is what makes the two differ**: the probe is 200 wide
/// in a 40-wide control.
#[test]
fn clip_narrows_what_the_content_may_draw_in() {
    let clipped = viewport_seen(true);
    let free = viewport_seen(false);

    assert!(
        clipped.width <= 40.5 && clipped.height <= 30.5,
        "with clip, overflowing content was offered {clipped:?}, wider than the 40 x 30 \\
         control",
    );
    assert!(
        free.width > 40.5,
        "without clip, the content was offered {free:?}, so the two cases do not differ \\
         and this test measures nothing",
    );
}

/// The effect, in pixels: overflowing text is cut off at the
/// control's edge when `clip` is set and runs on past it when not.
#[test]
fn clip_cuts_overflowing_text_at_the_controls_edge() {
    fn ink_beyond_the_control(clip: bool) -> (usize, usize) {
        let long = text("WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW")
            .wrapping(iced::widget::text::Wrapping::None)
            .size(14);
        let control = Control::new(
            long,
            Box::new(|_theme, _status| button::Style {
                text_color: Color::BLACK,
                ..button::Style::default()
            }),
            plain_ring(),
        )
        .width(Length::Fixed(60.0))
        .height(Length::Fixed(30.0))
        .on_press(Msg::A)
        .clip(clip);

        // A white page, so any dark pixel is ink.
        let page: Element<'static, Msg> = iced::widget::container(row![control])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(Color::WHITE.into()),
                ..Default::default()
            })
            .into();

        let mut ui: Simulator<'_, Msg> =
            Simulator::with_size(Settings::default(), Size::new(300.0, 60.0), page);
        let dir =
            std::env::temp_dir().join(format!("snora-control-clip-{}-{clip}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create a scratch directory");
        let snapshot = ui.snapshot(&Theme::Light).expect("render a frame");
        assert!(
            snapshot
                .matches_image(dir.join("frame"))
                .expect("write the frame")
        );

        let written = std::fs::read_dir(&dir)
            .expect("read the scratch directory")
            .next()
            .expect("a frame was written")
            .expect("directory entry")
            .path();
        let decoder = png::Decoder::new(std::io::BufReader::new(
            std::fs::File::open(&written).expect("open the frame"),
        ));
        let mut reader = decoder.read_info().expect("PNG header");
        let mut rgba = vec![0; reader.output_buffer_size().expect("frame fits in memory")];
        let info = reader.next_frame(&mut rgba).expect("decode the frame");
        let _ = std::fs::remove_dir_all(&dir);

        // Scale 2: the control spans physical x 0..120.
        let dark = |x: u32, y: u32| {
            let i = ((y * info.width + x) * 4) as usize;
            rgba[i] < 128 && rgba[i + 1] < 128 && rgba[i + 2] < 128
        };
        let (mut inside, mut beyond) = (0, 0);
        for y in 0..info.height {
            for x in 0..info.width {
                if dark(x, y) {
                    if x < 120 {
                        inside += 1;
                    } else {
                        beyond += 1;
                    }
                }
            }
        }
        (inside, beyond)
    }

    let (inside_clipped, beyond_clipped) = ink_beyond_the_control(true);
    let (_, beyond_free) = ink_beyond_the_control(false);

    assert!(
        inside_clipped > 50,
        "no text was drawn inside the clipped control ({inside_clipped} dark pixels), so \\
         'nothing beyond it' proves nothing",
    );
    assert!(
        beyond_free > 50,
        "the unclipped text did not overflow ({beyond_free} dark pixels past the edge), \\
         so this test cannot tell clipping from its absence",
    );
    assert_eq!(
        beyond_clipped, 0,
        "with clip set, {beyond_clipped} dark pixels were drawn past the control's edge",
    );
}
