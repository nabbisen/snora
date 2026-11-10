//! Every snora control can be reached and pressed from the keyboard, shows
//! where it is, and is as big as RFC-115 says (RFC-116 R-1 Unit 3).
//!
//! The second half of this file (from "sizes") is the target and icon rules;
//! the first is the keyboard probe, described next.
//!
//! # What each control must do
//!
//! Unit 0's probe, applied to every control that used to be a `button`:
//!
//! 1. **Tab reaches it.** The runtime's own `focus_next`, run against the
//!    rendered tree, lands on it, and `find_focused` returns *its* id.
//! 2. **Enter and Space press it, and publish what a click publishes.**
//!    Two ways in must not mean two messages out. Nothing is published
//!    before the control has focus.
//! 3. **The ring draws at 3:1 or better against what it covers**, read
//!    from the pixels (WCAG 1.4.11), in every stock theme on the unstyled
//!    path and in every preset on the design path.
//! 4. **A disabled control is not a focus target** (design `*_maybe`
//!    with `None`).
//!
//! # How the controls are found
//!
//! Not by name. The tree reports every focus target and where it is, in
//! tree order, so each subject says how many targets it has and what each
//! one publishes. A control that stopped registering would make the count
//! come up short, and one that registered under the wrong id would fail
//! `find_focused`.
//!
//! # How the ring is measured
//!
//! The control is focused by Tab, the cursor is parked away so no
//! hover is in the frame, and two frames are compared: the pixels that
//! changed *are* the ring. On each of the control's four sides, the
//! changed pixels are compared with what was under them in the unfocused
//! frame, and the weakest is the figure. That reads "the ring against what
//! it covers" for any control on any surface, without the test having to
//! know what is behind it.
//!
//! # The renderer
//!
//! Focus and key tests use tiny-skia through the runtime's own
//! `UserInterface`, which needs no GPU, and do not follow
//! `ICED_TEST_BACKEND`. The ring tests do (they use the simulator).

#![cfg(feature = "widgets")]

use iced::widget::{column, row, space};
use iced::{Element, Font, Length, Pixels, Point, Rectangle, Size, Theme, mouse};
use iced_runtime::core::clipboard;
use iced_runtime::core::widget::operation::{self, Outcome, focusable};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Event, renderer::Headless, widget::operation::Focusable};
use iced_runtime::user_interface::{self, UserInterface};

use snora::{
    AppLayout, BreadcrumbAction, Crumb, LayoutDirection, Menu, MenuAction, MenuItem, Tab,
    TabAction, TabBar, Toast, ToastIntent, render,
};

mod common;
use common::tab_router::tab_router;
use common::{Frame, NON_TEXT_MIN, colour_of, contrast_ratio, frame_pixels, simulator};

const WINDOW: Size = Size::new(1024.0, 768.0);
const AWAY: Point = Point::new(1000.0, 760.0);

#[derive(Debug, Clone, PartialEq)]
struct Msg(String);

fn msg(text: &str) -> Msg {
    Msg(text.to_owned())
}

static ON_TAB: fn(TabAction<u8>) -> Msg = |a| Msg(format!("{a:?}"));
static ON_CRUMB: fn(BreadcrumbAction<u8>) -> Msg = |a| Msg(format!("{a:?}"));
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = |a| Msg(format!("{a:?}"));

/// One thing to probe: how to build it, and what each of its focus
/// targets, in tree order, must publish.
struct Subject {
    name: &'static str,
    build: Box<dyn Fn() -> Element<'static, Msg>>,
    publishes: Vec<Msg>,
    /// Some of its controls are in an **overlay** (RFC-124: a header menu's dropdown).
    /// A widget's `operate` does not reach an overlay, so the simulator and `tab_router`
    /// never see those controls: their ring is measured on the runtime's own interface.
    hosts_overlay: bool,
}

impl Subject {
    /// The subject, inset from the window's edge. A ring is drawn outside
    /// its control; one at the edge of the window would have its outer side
    /// off the frame, which is the window clipping it and nothing to do
    /// with the control.
    fn element(&self) -> Element<'static, Msg> {
        iced::widget::container((self.build)()).padding(24).into()
    }
}

fn subject(
    name: &'static str,
    publishes: &[&str],
    build: impl Fn() -> Element<'static, Msg> + 'static,
) -> Subject {
    Subject {
        name,
        build: Box::new(build),
        publishes: publishes.iter().map(|m| msg(m)).collect(),
        hosts_overlay: false,
    }
}

impl Subject {
    fn hosting_overlay(mut self) -> Self {
        self.hosts_overlay = true;
        self
    }
}

fn tab_bar() -> TabBar<u8> {
    TabBar {
        tabs: ["One", "Two"]
            .iter()
            .enumerate()
            .map(|(i, l)| Tab {
                id: i as u8,
                label: (*l).to_owned(),
                icon: None,
            })
            .collect(),
        active: 0,
    }
}

fn crumbs() -> Vec<Crumb<u8>> {
    vec![
        Crumb::ancestor(0, "Home"),
        Crumb::ancestor(1, "Library"),
        // The leaf is plain text: not a control, not a target.
        Crumb::leaf(2, "Books"),
    ]
}

fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".to_owned(),
        icon: None,
        items: [("Open", 7), ("Save", 8)]
            .iter()
            .map(|(l, id)| MenuItem {
                menu_id: 0,
                id: *id,
                label: (*l).to_owned(),
                icon: None,
            })
            .collect(),
    }
}

const MENU_MESSAGES: [&str; 3] = [
    "MenuPressed(0)",
    "MenuItemPressed { menu_id: 0, menu_item_id: 7 }",
    "MenuItemPressed { menu_id: 0, menu_item_id: 8 }",
];

fn toast_layout() -> AppLayout<Element<'static, Msg>, Msg> {
    let body: Element<'static, Msg> = space().width(Length::Fill).height(Length::Fill).into();
    AppLayout::new(body).toasts(vec![Toast::new(
        7,
        ToastIntent::Info,
        "Saved",
        "All good.",
        msg("dismiss"),
    )])
}

/// The unstyled controls.
fn unstyled() -> Vec<Subject> {
    vec![
        subject("tabs", &["Pressed(0)", "Pressed(1)"], || {
            snora::widget::app_tab_bar(tab_bar(), &ON_TAB, LayoutDirection::Ltr)
        }),
        subject(
            "breadcrumb ancestors",
            &["Pressed(0)", "Pressed(1)"],
            || snora::widget::app_breadcrumb(crumbs(), &ON_CRUMB, LayoutDirection::Ltr),
        ),
        subject("header menu (trigger and items)", &MENU_MESSAGES, || {
            snora::widget::app_header(
                "App",
                vec![menu()],
                &ON_MENU,
                Some(&0),
                None,
                LayoutDirection::Ltr,
            )
        })
        .hosting_overlay(),
        subject("render_menu (trigger and items)", &MENU_MESSAGES, || {
            snora::widget::render_menu(menu(), &ON_MENU, true)
        }),
        subject("toast close", &["dismiss"], || render(toast_layout())),
        // RFC-122: aaai's cases on the public `snora::control`.
        subject("control: glyph beside a label", &["open"], || {
            common::aaai_cases::glyph_label_engine(msg("open"))
        }),
        subject("control: file-tree row, Fill", &["tree"], || {
            common::aaai_cases::tree_row_engine(msg("tree"))
        }),
        // **Not reached** (RFC-119): a `snora::control` on the page, under an open
        // dialog. One focus target, the dialog's; a page control that were reachable
        // would make the count two and fail.
        subject(
            "a snora::control page control under an open dialog is not reached",
            &["dialog"],
            || {
                let body = common::aaai_cases::glyph_label_engine(msg("page"));
                // On a card of the theme's own `weak` tier, as an application sets a
                // dialog: bare on the engine's modal dim the ring is under 3:1 in 18
                // of the 22 themes (the finding in the RFC-122 Unit 1 request), and
                // this subject is about reach, not about that.
                let ok: iced::Element<'static, Msg> =
                    iced::widget::container(common::aaai_cases::glyph_label_engine(msg("dialog")))
                        .padding(16)
                        .style(|theme: &iced::Theme| iced::widget::container::Style {
                            background: Some(theme.extended_palette().background.weak.color.into()),
                            ..Default::default()
                        })
                        .into();
                render(snora::AppLayout::new(body).dialog(snora::Dialog::new(ok)))
            },
        ),
    ]
}

/// The design-path controls, for one preset.
#[cfg(feature = "design")]
fn designed(tokens: &'static snora::design::Tokens) -> Vec<Subject> {
    use snora::design::{Tone, button, chip, notice::Notice};

    vec![
        subject("design tabs", &["Pressed(0)", "Pressed(1)"], move || {
            snora::design::widget::app_tab_bar(tokens, tab_bar(), &ON_TAB, LayoutDirection::Ltr)
        }),
        subject(
            "design breadcrumb ancestors",
            &["Pressed(0)", "Pressed(1)"],
            move || {
                snora::design::widget::app_breadcrumb(
                    tokens,
                    crumbs(),
                    &ON_CRUMB,
                    LayoutDirection::Ltr,
                )
            },
        ),
        subject("design header menu", &MENU_MESSAGES, move || {
            snora::design::widget::app_header(
                tokens,
                "App",
                vec![menu()],
                &ON_MENU,
                Some(&0),
                None,
                LayoutDirection::Ltr,
            )
        })
        .hosting_overlay(),
        subject("button::primary", &["primary"], move || {
            button::primary(tokens, "Save", msg("primary"))
        }),
        subject("button::secondary", &["secondary"], move || {
            button::secondary(tokens, "Save", msg("secondary"))
        }),
        subject("button::ghost", &["ghost"], move || {
            button::ghost(tokens, "Save", msg("ghost"))
        }),
        subject("button::link", &["link"], move || {
            button::link(tokens, "Choose another folder", msg("link"))
        }),
        subject("button::danger", &["danger"], move || {
            button::danger(tokens, "Delete", msg("danger"))
        }),
        // A disabled button is not a focus target: Tab skips it.
        subject("button::primary_maybe(None)", &[], move || {
            button::primary_maybe(tokens, "Save", None)
        }),
        subject(
            "notice action and dismiss",
            &["act", "dismiss"],
            move || {
                Notice::new(tokens, Tone::Info, "Index rebuilt.")
                    .action("Undo", msg("act"))
                    .dismiss(msg("dismiss"))
                    .render()
            },
        ),
        subject("chip::filter", &["toggle"], move || {
            chip::filter(tokens, "Rust", false, msg("toggle"))
        }),
        subject("chip::removable", &["toggle", "remove"], move || {
            chip::removable(tokens, "Rust", false, msg("toggle"), msg("remove"))
        }),
        // **A control that is NOT reached** (RFC-119): the page's "page" control
        // is under an open dialog, so the tree reports one focus target, the
        // dialog's, and Tab, Enter and the ring are checked on that. A page
        // control that were reachable would make the count two and fail.
        subject(
            "a page control under an open dialog is not reached",
            &["dialog"],
            move || {
                let body = button::primary(tokens, "Delete", msg("page"));
                let ok = button::primary(tokens, "OK", msg("dialog"));
                snora::design::render(
                    snora::AppLayout::new(body).dialog(snora::Dialog::new(ok)),
                    tokens,
                )
            },
        ),
        subject("design toast close", &["dismiss"], move || {
            snora::design::render(toast_layout(), tokens)
        }),
        // RFC-122: aaai's cases on the public `snora::design::control`.
        subject(
            "design control: glyph beside a label, primary",
            &["open"],
            move || common::aaai_cases::glyph_label_design(tokens, msg("open")),
        ),
        subject(
            "design control: file-tree row, Fill",
            &["tree"],
            move || common::aaai_cases::tree_row_design(tokens, msg("tree")),
        ),
        subject(
            "design control: a suggestion, long data text",
            &["suggestion"],
            move || common::aaai_cases::suggestion_design(tokens, msg("suggestion")),
        ),
        // A disabled control is not a focus target: Tab skips it.
        subject("design control: on_press_maybe(None)", &[], move || {
            common::aaai_cases::disabled_design(tokens)
        }),
        subject(
            "a snora::design::control page control under an open dialog is not reached",
            &["dialog"],
            move || {
                let body = common::aaai_cases::glyph_label_design(tokens, msg("page"));
                let ok = common::aaai_cases::glyph_label_design(tokens, msg("dialog"));
                snora::design::render(
                    snora::AppLayout::new(body).dialog(snora::Dialog::new(ok)),
                    tokens,
                )
            },
        ),
    ]
}

// ---- the runtime's own interface ------------------------------------------

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

    /// Rebuilds around a different root, **keeping the widget tree's
    /// state** — what happens every time an application's `view` runs
    /// after its `update`.
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

    fn deliver(&mut self, events: &[Event]) {
        let _ = self.ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    /// A rig that draws on the renderer the run asked for (`ICED_TEST_BACKEND`, default
    /// tiny-skia), for the ring tests, which follow it.
    fn for_frames(root: Element<'static, Msg>) -> Self {
        let wanted = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned());
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
        // What the runtime sends before every draw: it is also what lets the overlay's
        // layout exist before the first frame.
        rig.deliver(&[Event::Window(
            iced_runtime::core::window::Event::RedrawRequested(
                iced_runtime::core::time::Instant::now(),
            ),
        )]);
        rig
    }

    /// The frame as drawn now, in the units [`Frame`] uses (scale 2).
    fn frame(&mut self, theme: &Theme) -> Frame {
        use iced::theme::Base;
        let base = theme.base();
        self.ui.draw(
            &mut self.renderer,
            theme,
            &iced_runtime::core::renderer::Style {
                text_color: base.text_color,
            },
            self.cursor,
        );
        let scale = Frame::SCALE;
        let (width, height) = (
            (WINDOW.width * scale) as u32,
            (WINDOW.height * scale) as u32,
        );
        let rgba = Headless::screenshot(
            &mut self.renderer,
            Size::new(width, height),
            scale,
            base.background_color,
        );
        Frame {
            renderer: Headless::name(&self.renderer),
            width,
            height,
            rgba,
        }
    }

    fn key(&mut self, key: iced::keyboard::key::Named) {
        self.deliver(&[iced_test::simulator::press_key(key, None)]);
    }

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

    /// Runs an operation to completion, as the window loop does:
    /// `focus_next` is two-pass, and one pass focuses nothing.
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

    fn targets(&mut self) -> Vec<Target> {
        let mut found = Collect::default();
        self.ui.operate(&self.renderer, &mut found);
        found.0
    }

    fn tab(&mut self, times: usize) {
        for _ in 0..times {
            let _ = self.run(focusable::focus_next::<()>());
        }
    }
}

struct Target {
    id: Option<Id>,
    bounds: Rectangle,
    focused: bool,
}

#[derive(Default)]
struct Collect(Vec<Target>);

impl Operation for Collect {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        self.0.push(Target {
            id: id.cloned(),
            bounds,
            focused: state.is_focused(),
        });
    }
}

/// Tab, Enter and Space, for every target of one subject. Returns what
/// went wrong, so one run reports everything.
fn keyboard_failures(context: &str, subject: &Subject) -> Vec<String> {
    use iced::keyboard::key::Named;

    let mut failures = Vec::new();
    let n = subject.publishes.len();
    let fresh = || Rig::new(subject.element());

    let mut rig = fresh();
    let targets = rig.targets();
    if targets.len() != n {
        failures.push(format!(
            "{context} / {}: {} focus target(s), expected {n} — Tab cannot reach what is \
             not registered, or reaches something that does nothing",
            subject.name,
            targets.len(),
        ));
        return failures;
    }
    if targets.iter().any(|t| t.id.is_none()) {
        failures.push(format!("{context} / {}: a target has no id", subject.name));
        return failures;
    }
    if rig.run(focusable::find_focused()).is_some() {
        failures.push(format!(
            "{context} / {}: something is focused before any Tab",
            subject.name
        ));
    }

    // Nothing focused: Enter publishes nothing.
    rig.key(Named::Enter);
    if !rig.messages.is_empty() {
        failures.push(format!(
            "{context} / {}: Enter published {:?} with nothing focused",
            subject.name, rig.messages
        ));
    }

    for k in 0..n {
        let want = &subject.publishes[k];

        // Tab lands on target k, and the runtime agrees which it is.
        let mut rig = fresh();
        rig.tab(k + 1);
        let now = rig.targets();
        let flags: Vec<bool> = now.iter().map(|t| t.focused).collect();
        let expected: Vec<bool> = (0..n).map(|i| i == k).collect();
        if flags != expected {
            failures.push(format!(
                "{context} / {} #{k}: after {} Tab(s) the focus flags are {flags:?}, expected {expected:?}",
                subject.name,
                k + 1,
            ));
            continue;
        }
        // The id comes from this same tree: a control's id is made when it
        // is built, so another build's would never match.
        if rig.run(focusable::find_focused()).as_ref() != now[k].id.as_ref() {
            failures.push(format!(
                "{context} / {} #{k}: Tab focused target {k}, but find_focused returns another id",
                subject.name,
            ));
        }

        // Enter and Space publish what a click publishes.
        for (name, key) in [("Enter", Named::Enter), ("Space", Named::Space)] {
            let mut rig = fresh();
            rig.tab(k + 1);
            rig.key(key);
            if rig.messages != vec![want.clone()] {
                failures.push(format!(
                    "{context} / {} #{k}: {name} published {:?}, expected {want:?}",
                    subject.name, rig.messages
                ));
            }
        }
        let mut rig = fresh();
        let at = rig.targets()[k].bounds.center();
        rig.click(at);
        if rig.messages != vec![want.clone()] {
            failures.push(format!(
                "{context} / {} #{k}: a click published {:?}, expected {want:?}",
                subject.name, rig.messages
            ));
        }
    }
    failures
}

fn assert_no_failures(what: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} {what} failure(s):\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}

#[test]
fn every_unstyled_control_is_reached_and_pressed_from_the_keyboard() {
    let mut failures = Vec::new();
    for s in unstyled() {
        failures.extend(keyboard_failures("unstyled", &s));
    }
    assert_no_failures("keyboard", failures);
}

#[cfg(feature = "design")]
fn presets() -> Vec<(&'static str, &'static snora::design::Tokens)> {
    snora::design::Tokens::presets()
        .into_iter()
        .map(|(name, tokens)| (name, &*Box::leak(Box::new(tokens))))
        .collect()
}

#[cfg(feature = "design")]
#[test]
fn every_design_control_is_reached_and_pressed_from_the_keyboard() {
    let mut failures = Vec::new();
    for (preset, tokens) in presets() {
        for s in designed(tokens) {
            failures.extend(keyboard_failures(preset, &s));
        }
    }
    assert_no_failures("keyboard", failures);
}

// ---- the ring ---------------------------------------------------------------

/// The weakest contrast, over a control's four sides, between the pixels
/// the ring changed and what was under them. `Err` names a side with no
/// ring at all.
fn ring_contrast(
    plain: &Frame,
    focused: &Frame,
    bounds: Rectangle,
    reach: f32,
) -> Result<f32, String> {
    let s = Frame::SCALE;
    let centre = bounds.center();
    // Each side is a short scan outward from the control's edge.
    let sides: [(&str, Vec<(u32, u32)>); 4] = {
        let scan = |points: Vec<(f32, f32)>| -> Vec<(u32, u32)> {
            points
                .into_iter()
                .filter(|(x, y)| *x >= 0.0 && *y >= 0.0)
                .map(|(x, y)| ((x * s) as u32, (y * s) as u32))
                .collect()
        };
        let steps = (reach * s) as usize;
        let at = |i: usize| i as f32 / s;
        [
            (
                "left",
                scan((1..=steps).map(|i| (bounds.x - at(i), centre.y)).collect()),
            ),
            (
                "right",
                scan(
                    (0..steps)
                        .map(|i| (bounds.x + bounds.width + at(i), centre.y))
                        .collect(),
                ),
            ),
            (
                "top",
                scan((1..=steps).map(|i| (centre.x, bounds.y - at(i))).collect()),
            ),
            (
                "bottom",
                scan(
                    (0..steps)
                        .map(|i| (centre.x, bounds.y + bounds.height + at(i)))
                        .collect(),
                ),
            ),
        ]
    };

    let mut weakest = f32::MAX;
    for (side, points) in sides {
        let mut best: Option<f32> = None;
        let mut changed_any = false;
        for (x, y) in points {
            let (Some(a), Some(b)) = (plain.pixel(x, y), focused.pixel(x, y)) else {
                continue;
            };
            if a[..3] != b[..3] {
                changed_any = true;
                let ratio = contrast_ratio(colour_of(b), colour_of(a));
                // The ring's own colour is the strongest pixel on this
                // side; blended edge pixels are weaker by construction.
                best = Some(best.map_or(ratio, |r| r.max(ratio)));
            }
        }
        if !changed_any {
            return Err(format!("no ring on the {side} side"));
        }
        weakest = weakest.min(best.unwrap_or(0.0));
    }
    Ok(weakest)
}

/// The ring of a subject whose controls are in an overlay, on the runtime's own
/// interface (the simulator and `tab_router` cannot reach an overlay). Same method as
/// [`ring_failures`]: Tab to the control, two frames, the changed pixels are the ring.
fn ring_failures_on_the_runtime(
    context: &str,
    theme: &Theme,
    subject: &Subject,
    out: &mut Vec<String>,
) {
    let mut plain_rig = Rig::for_frames(subject.element());
    let bounds: Vec<Rectangle> = plain_rig.targets().iter().map(|t| t.bounds).collect();
    assert_eq!(
        bounds.len(),
        subject.publishes.len(),
        "{context} / {}: the runtime rig finds {} focus targets, expected {}",
        subject.name,
        bounds.len(),
        subject.publishes.len()
    );
    let plain = plain_rig.frame(theme);
    for (k, target) in bounds.iter().enumerate() {
        let mut rig = Rig::for_frames(subject.element());
        rig.tab(k + 1);
        let focused = rig.frame(theme);
        match ring_contrast(&plain, &focused, *target, 10.0) {
            Err(why) => out.push(format!("{context} / {} #{k}: {why}", subject.name)),
            Ok(ratio) if ratio < NON_TEXT_MIN => out.push(format!(
                "{context} / {} #{k}: the ring is {ratio:.2}:1 against what it covers, \
                 under {NON_TEXT_MIN}",
                subject.name
            )),
            Ok(_) => {}
        }
    }
}

fn ring_failures(context: &str, theme: &Theme, subject: &Subject, out: &mut Vec<String>) {
    if subject.publishes.is_empty() {
        return;
    }
    if subject.hosts_overlay {
        return ring_failures_on_the_runtime(context, theme, subject, out);
    }
    // Where the controls are **in the simulator's own layout**. The runtime
    // rig lays out with its own headless renderer, and text widths differ
    // between the two by a few pixels, which is the difference between
    // clicking the middle of a control and measuring a ring around the
    // wrong box.
    let mut bounds = Vec::new();
    let _ = simulator(subject.element()).find(|candidate: iced_test::selector::Candidate<'_>| {
        if let iced_test::selector::Candidate::Focusable { bounds: b, .. } = candidate {
            bounds.push(b);
        }
        None::<()>
    });

    let mut ui = simulator(subject.element());
    ui.point_at(AWAY);
    let plain = frame_pixels(
        &mut ui,
        theme,
        &format!("ctl-plain-{context}-{}", subject.name),
    );

    for (k, target) in bounds.iter().enumerate() {
        // Focused by Tab (a click no longer focuses, RFC-121): target `k` is the
        // (k + 1)th, so that many Tabs reach it from nothing focused.
        let mut ui = simulator(tab_router(subject.element()));
        for _ in 0..=k {
            let _ = ui.tap_key(iced::keyboard::key::Named::Tab);
        }
        ui.point_at(AWAY);
        let focused = frame_pixels(
            &mut ui,
            theme,
            &format!("ctl-focused-{context}-{}-{k}", subject.name),
        );

        match ring_contrast(&plain, &focused, *target, 10.0) {
            Err(why) => out.push(format!("{context} / {} #{k}: {why}", subject.name)),
            Ok(ratio) if ratio < NON_TEXT_MIN => out.push(format!(
                "{context} / {} #{k}: the ring is {ratio:.2}:1 against what it covers, \
                 under {NON_TEXT_MIN}",
                subject.name
            )),
            Ok(_) => {}
        }
    }
}

#[test]
fn the_focus_ring_of_every_unstyled_control_clears_the_non_text_floor() {
    let mut failures = Vec::new();
    for theme in Theme::ALL {
        for s in unstyled() {
            ring_failures(&format!("stock {theme}"), theme, &s, &mut failures);
        }
    }
    assert_no_failures("focus-ring", failures);
}

#[cfg(feature = "design")]
#[test]
fn the_focus_ring_of_every_design_control_clears_the_non_text_floor() {
    let mut failures = Vec::new();
    for (preset, tokens) in presets() {
        let theme = snora::design::theme(tokens);
        for s in designed(tokens) {
            ring_failures(preset, &theme, &s, &mut failures);
        }
    }
    assert_no_failures("focus-ring", failures);
}

/// The instrument checks itself: a control that is not focused draws no
/// ring, so two unfocused frames are identical. If this failed, every
/// figure above would be measuring noise.
#[test]
fn two_unfocused_frames_are_identical() {
    let theme = Theme::Light;
    let build = || -> Element<'static, Msg> {
        column![
            row![snora::widget::app_tab_bar(
                tab_bar(),
                &ON_TAB,
                LayoutDirection::Ltr
            )],
            space().height(Length::Fill)
        ]
        .into()
    };
    let mut ui = simulator(build());
    ui.point_at(AWAY);
    let a = frame_pixels(&mut ui, &theme, "ctl-control-a");
    let mut ui = simulator(build());
    ui.point_at(AWAY);
    let b = frame_pixels(&mut ui, &theme, "ctl-control-b");
    assert!(
        (0..a.height).all(|y| (0..a.width).all(|x| a.pixel(x, y) == b.pixel(x, y))),
        "two frames of the same unfocused tab bar differ, so the ring measurement is reading noise"
    );
}

// ---- sizes (RFC-115's amendment) --------------------------------------------

fn settings_at(host: f32) -> iced::Settings {
    iced::Settings {
        default_text_size: host.into(),
        #[cfg(feature = "lucide-icons")]
        fonts: vec![lucide_icons::LUCIDE_FONT_BYTES.into()],
        ..iced::Settings::default()
    }
}

/// Every control's box in the simulator's own layout, at a host text size.
fn control_boxes(element: Element<'static, Msg>, host: f32) -> Vec<Rectangle> {
    let mut found = Vec::new();
    let mut ui = iced_test::Simulator::with_settings(settings_at(host), element);
    let _ = ui.find(|candidate: iced_test::selector::Candidate<'_>| {
        if let iced_test::selector::Candidate::Focusable { bounds, .. } = candidate {
            found.push(bounds);
        }
        None::<()>
    });
    found
}

/// **A target is at least `max(24, 1.5 x text)`**, in both dimensions, for
/// every unstyled control, at the default text size and at 24.
///
/// At 16 the rule is 24 and costs nothing; at 24 it is 36, and the controls
/// that were smaller (the toast's close mark was a fixed 24) now grow.
#[test]
fn every_unstyled_control_is_at_least_one_and_a_half_times_the_text() {
    let mut failures = Vec::new();
    for host in [16.0_f32, 24.0] {
        let floor = 24.0_f32.max(1.5 * host);
        for s in unstyled() {
            for (k, b) in control_boxes(s.element(), host).iter().enumerate() {
                if b.width < floor - 0.01 || b.height < floor - 0.01 {
                    failures.push(format!(
                        "host {host}px / {} #{k}: {:.1} x {:.1}, under the {floor}px rule",
                        s.name, b.width, b.height
                    ));
                }
            }
        }
    }
    assert_no_failures("target-size", failures);
}

/// **The minimum-target token is honoured**: at 44, every design control is
/// 44 x 44 or more. A token that was read and then ignored would leave the
/// small ones at 24-30.
#[cfg(feature = "design")]
#[test]
fn the_minimum_target_token_raises_every_design_control() {
    let mut failures = Vec::new();
    for (preset, tokens) in snora::design::Tokens::presets() {
        let mut tokens = tokens;
        tokens.targets.min = 44.0;
        let tokens: &'static snora::design::Tokens = Box::leak(Box::new(tokens));
        for s in designed(tokens) {
            for (k, b) in control_boxes(s.element(), 16.0).iter().enumerate() {
                if b.width < 43.99 || b.height < 43.99 {
                    failures.push(format!(
                        "{preset} / {} #{k}: {:.1} x {:.1}, under a 44px minimum-target token",
                        s.name, b.width, b.height
                    ));
                }
            }
        }
    }
    assert_no_failures("minimum-target token", failures);
}

/// And at the preset's own 24 nothing is under 24: the rule's floor holds
/// on the design path as well.
#[cfg(feature = "design")]
#[test]
fn every_design_control_is_at_least_24_in_every_preset() {
    let mut failures = Vec::new();
    for (preset, tokens) in presets() {
        for s in designed(tokens) {
            for (k, b) in control_boxes(s.element(), 16.0).iter().enumerate() {
                if b.width < 23.99 || b.height < 23.99 {
                    failures.push(format!(
                        "{preset} / {} #{k}: {:.1} x {:.1}",
                        s.name, b.width, b.height
                    ));
                }
            }
        }
    }
    assert_no_failures("24px floor", failures);
}

// ---- icons ------------------------------------------------------------------

fn bounds_of(element: Element<'static, Msg>, label: &str, host: f32) -> Rectangle {
    let mut ui = iced_test::Simulator::with_settings(settings_at(host), element);
    ui.find(label)
        .unwrap_or_else(|_| panic!("{label:?} is not rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{label:?} is not visible"))
}

fn same_size(a: Rectangle, b: Rectangle) -> bool {
    (a.width - b.width).abs() <= 0.1 && (a.height - b.height).abs() <= 0.1
}

#[cfg(feature = "lucide-icons")]
const CLOSE_GLYPH: &str = "\u{e1b2}";
#[cfg(not(feature = "lucide-icons"))]
const CLOSE_GLYPH: &str = "×";

fn reference_text(content: &str, size: Option<f32>) -> Element<'static, Msg> {
    let mut t = iced::widget::text(content.to_owned());
    if let Some(size) = size {
        t = t.size(size);
    }
    #[cfg(feature = "lucide-icons")]
    if content == CLOSE_GLYPH {
        t = t.font(Font::with_name("lucide"));
    }
    t.into()
}

/// **A pictogram is 1 em of the text it sits with**: the same box as a
/// `text` with no size of its own, at the default text size and at 24. It
/// was a literal 14, which is neither.
#[test]
fn an_unstyled_pictogram_is_one_em_of_the_host_text() {
    let icon = snora::Icon::Text("A".into());
    for host in [16.0_f32, 24.0] {
        let got = bounds_of(snora::widget::icon_element(&icon), "A", host);
        let want = bounds_of(reference_text("A", None), "A", host);
        assert!(
            same_size(got, want),
            "host {host}px: the icon is {:.1} x {:.1}, 1 em is {:.1} x {:.1}",
            got.width,
            got.height,
            want.width,
            want.height,
        );
    }
}

/// The three rail forms agree: Beneath, Beside and Tooltip each draw their
/// icons at 1 em of the host's text, so an icon does not change size when
/// the labels move.
#[test]
fn every_unstyled_rail_form_draws_its_icons_at_one_em() {
    use snora::widget::SideBarLabels;
    use snora::{SideBar, SideBarItem};
    for host in [16.0_f32, 24.0] {
        for (form, labels) in [
            ("Beneath", SideBarLabels::Beneath),
            ("Beside", SideBarLabels::Beside),
            ("Tooltip", SideBarLabels::Tooltip),
        ] {
            let bar = SideBar {
                items: vec![SideBarItem {
                    view_id: 0u8,
                    icon: snora::Icon::Text("Q".into()),
                    tooltip: "Home".to_owned(),
                    on_press: msg("go"),
                }],
                active: 0u8,
            };
            let rail = snora::widget::app_side_bar_with(bar, LayoutDirection::Ltr, labels);
            let got = bounds_of(rail, "Q", host);
            let want = bounds_of(reference_text("Q", None), "Q", host);
            assert!(
                same_size(got, want),
                "{form} at host {host}px: the icon is {:.1} x {:.1}, 1 em is {:.1} x {:.1}",
                got.width,
                got.height,
                want.width,
                want.height,
            );
        }
    }
}

/// **The × is 1.125 em**: 18 px at the default 16, and 27 at 24. A reference
/// built independently of the production helper, so a wrong ratio there
/// cannot also be the reference.
#[test]
fn the_toast_close_mark_is_one_and_an_eighth_em() {
    for host in [16.0_f32, 24.0] {
        let got = bounds_of(render(toast_layout()), CLOSE_GLYPH, host);
        let want = bounds_of(
            reference_text(CLOSE_GLYPH, Some(1.125 * host)),
            CLOSE_GLYPH,
            host,
        );
        assert!(
            same_size(got, want),
            "host {host}px: the close mark is {:.1} x {:.1}, 1.125 em is {:.1} x {:.1}",
            got.width,
            got.height,
            want.width,
            want.height,
        );
    }
}

/// Design-path icons are unchanged: they follow their label token, not the
/// host's text.
#[cfg(feature = "design")]
#[test]
fn a_design_path_icon_is_its_label_token_not_the_hosts_text() {
    use snora::design::style::text::label_size;
    let (_, tokens) = &snora::design::Tokens::presets()[0];
    let label = label_size(tokens).0;
    let icon = snora::Icon::Text("A".into());
    for host in [16.0_f32, 24.0] {
        let got = bounds_of(snora::widget::icon_element_sized(&icon, label), "A", host);
        let want = bounds_of(reference_text("A", Some(label)), "A", host);
        assert!(same_size(got, want), "host {host}px");
    }
}

// ---- focus survives the press -------------------------------------------------

/// **Pressing a tab from the keyboard must not lose the focus it was
/// pressed with.**
///
/// Found in a real window: Tab, then Space made the second tab active, and
/// its ring vanished. The active tab used to be wrapped in a `stack` that
/// the others were not, so when a tab became active its position in the
/// tree held a different widget, the runtime could not carry the state
/// over, and the focus went with it. A keyboard user who presses Enter on
/// a tab and finds nothing focused has to Tab from the start again.
///
/// The view is rebuilt with the second tab active, around the same tree
/// state, exactly as an application's `view` does after `update`.
#[test]
fn a_tab_keeps_its_focus_when_it_becomes_the_active_tab() {
    let bar = |active: u8| -> Element<'static, Msg> {
        let mut bar = tab_bar();
        bar.active = active;
        snora::widget::app_tab_bar(bar, &ON_TAB, LayoutDirection::Ltr)
    };

    let mut rig = Rig::new(bar(0));
    rig.tab(2);
    let before: Vec<bool> = rig.targets().iter().map(|t| t.focused).collect();
    assert_eq!(
        before,
        vec![false, true],
        "the premise: the second tab has focus"
    );

    let mut rig = rig.rebuilt(bar(1));
    let after: Vec<bool> = rig.targets().iter().map(|t| t.focused).collect();
    assert_eq!(
        after,
        vec![false, true],
        "the second tab became the active tab and lost its focus: a keyboard user pressing \
         Enter on a tab is left with nothing focused"
    );
}

/// The same for a menu trigger: pressing it opens the menu, and the trigger
/// is the control the user pressed, so it must still be the one with focus.
#[test]
fn a_menu_trigger_keeps_its_focus_when_its_menu_opens() {
    let header = |open: bool| -> Element<'static, Msg> {
        snora::widget::app_header(
            "App",
            vec![menu()],
            &ON_MENU,
            open.then_some(&0),
            None,
            LayoutDirection::Ltr,
        )
    };

    let mut rig = Rig::new(header(false));
    rig.tab(1);
    assert_eq!(
        rig.targets().iter().map(|t| t.focused).collect::<Vec<_>>(),
        vec![true],
        "the premise: the closed menu's trigger has focus",
    );

    let mut rig = rig.rebuilt(header(true));
    let flags: Vec<bool> = rig.targets().iter().map(|t| t.focused).collect();
    assert_eq!(
        flags,
        vec![true, false, false],
        "the menu opened and the trigger lost its focus (flags: trigger, item, item)"
    );
}

/// **An SVG icon is 1 em too**, which needs a number of pixels that only
/// layout can compute: `icon_element` puts it in a square of 1 em of the
/// host's text, filled. The file need not exist — the box is sized by the
/// host, not by the image.
#[cfg(feature = "svg-icons")]
#[test]
fn an_svg_icon_is_a_square_of_one_em() {
    let icon = snora::Icon::Svg("no-such-icon.svg".into());
    for host in [16.0_f32, 24.0] {
        let element: Element<'static, Msg> =
            iced::widget::container(snora::widget::icon_element(&icon)).into();
        let mut boxes = Vec::new();
        let mut ui = iced_test::Simulator::with_settings(settings_at(host), element);
        let _ = ui.find(|candidate: iced_test::selector::Candidate<'_>| {
            if let iced_test::selector::Candidate::Container { bounds, .. } = candidate {
                boxes.push(bounds);
            }
            None::<()>
        });
        let smallest = boxes
            .iter()
            .map(|b| (b.width, b.height))
            .fold((f32::MAX, f32::MAX), |a, b| (a.0.min(b.0), a.1.min(b.1)));
        // The outer container is the window; the inner shrinks to the icon.
        assert!(
            (smallest.0 - host).abs() <= 0.01 && (smallest.1 - host).abs() <= 0.01,
            "host {host}px: the SVG icon's box is {smallest:?}, 1 em is {host} x {host}",
        );
    }
}

// ---- focus survives the press, for every control that survives its press -------

/// The shape of `a_tab_keeps_its_focus_when_it_becomes_the_active_tab`, once,
/// for any control: focus it, **press it from the keyboard**, rebuild the view
/// with the state that press implies, and ask whether it still has focus.
///
/// `build(false)` is the view before the press and `build(true)` the view the
/// application would draw after handling its message. `which` is the control
/// pressed, among `count` focus targets, in tree order.
///
/// The class this guards: the runtime keeps a widget's state by its place in the
/// tree, so a press that reshapes the control's place drops its state, and its
/// focus with it. A keyboard user is left with nothing focused.
fn assert_keeps_focus_after_its_press(
    context: &str,
    build: &dyn Fn(bool) -> Element<'static, Msg>,
    which: usize,
    count: usize,
) {
    use iced::keyboard::key::Named;

    let mut rig = Rig::new(build(false));
    rig.tab(which + 1);
    rig.key(Named::Enter);
    assert!(
        !rig.messages.is_empty(),
        "{context}: Enter published nothing, so the press this test is about did not happen",
    );

    let mut rig = rig.rebuilt(build(true));
    let flags: Vec<bool> = rig.targets().iter().map(|t| t.focused).collect();
    let expected: Vec<bool> = (0..count).map(|i| i == which).collect();
    assert_eq!(
        flags, expected,
        "{context}: the control lost its focus when its own press changed how it looks"
    );
}

fn rail(active: u8, labels: snora::widget::SideBarLabels) -> snora::SideBar<Msg, u8> {
    let _ = labels;
    snora::SideBar {
        items: ["Home", "Search", "Settings"]
            .iter()
            .enumerate()
            .map(|(i, name)| snora::SideBarItem {
                view_id: i as u8,
                icon: snora::Icon::Text(i.to_string()),
                tooltip: (*name).to_owned(),
                on_press: msg(&format!("go {i}")),
            })
            .collect(),
        active,
    }
}

/// A sidebar item becomes the active one when it is pressed, in all three forms
/// of the rail and on both paths.
#[test]
fn a_sidebar_item_keeps_its_focus_when_it_becomes_active() {
    use snora::widget::SideBarLabels;
    for (form, labels) in [
        ("Beneath", SideBarLabels::Beneath),
        ("Beside", SideBarLabels::Beside),
        ("Tooltip", SideBarLabels::Tooltip),
    ] {
        assert_keeps_focus_after_its_press(
            &format!("unstyled rail, {form}"),
            &move |after| {
                let active = if after { 1 } else { 0 };
                snora::widget::app_side_bar_with(rail(active, labels), LayoutDirection::Ltr, labels)
            },
            1,
            3,
        );
    }
}

#[cfg(feature = "design")]
#[test]
fn a_design_sidebar_item_keeps_its_focus_when_it_becomes_active() {
    use snora::widget::SideBarLabels;
    for (preset, tokens) in presets() {
        for (form, labels) in [
            ("Beneath", SideBarLabels::Beneath),
            ("Beside", SideBarLabels::Beside),
            ("Tooltip", SideBarLabels::Tooltip),
        ] {
            assert_keeps_focus_after_its_press(
                &format!("{preset} rail, {form}"),
                &move |after| {
                    let active = if after { 1 } else { 0 };
                    snora::design::widget::app_side_bar_with(
                        tokens,
                        rail(active, labels),
                        LayoutDirection::Ltr,
                        labels,
                    )
                },
                1,
                3,
            );
        }
    }
}

/// A filter chip toggles to selected when it is pressed, and the selected chip
/// is drawn in a different style: the one place a control's *look* changes
/// with its own press on the design path.
#[cfg(feature = "design")]
#[test]
fn a_filter_chip_keeps_its_focus_when_it_becomes_selected() {
    for (preset, tokens) in presets() {
        assert_keeps_focus_after_its_press(
            &format!("{preset} chip::filter"),
            &move |selected| snora::design::chip::filter(tokens, "Rust", selected, msg("toggle")),
            0,
            1,
        );
    }
}

/// The same for a removable chip's label, which toggles; its remove button is
/// beside it and must not be disturbed.
#[cfg(feature = "design")]
#[test]
fn a_removable_chips_label_keeps_its_focus_when_it_becomes_selected() {
    for (preset, tokens) in presets() {
        assert_keeps_focus_after_its_press(
            &format!("{preset} chip::removable"),
            &move |selected| {
                snora::design::chip::removable(
                    tokens,
                    "Rust",
                    selected,
                    msg("toggle"),
                    msg("remove"),
                )
            },
            0,
            2,
        );
    }
}

// ---- the toast's close mark, on the design path ---------------------------------

/// **On the design path the × follows the toast's own text**, `Typography::body`,
/// at 1.125 em — and not the host's `default_text_size`, which the design
/// path's text does not use.
///
/// Two results nobody would predict if it did not: raising `body` would grow
/// the message and leave the ×, and raising the host with default tokens would
/// grow the × and leave the message.
#[cfg(feature = "design")]
mod design_toast_mark {
    use super::*;

    fn tokens_with_body(size: f32) -> &'static snora::design::Tokens {
        let mut tokens = snora::design::Tokens::light();
        tokens.typography.body.size = size;
        Box::leak(Box::new(tokens))
    }

    fn mark(tokens: &'static snora::design::Tokens, host: f32) -> Rectangle {
        bounds_of(
            snora::design::render(toast_layout(), tokens),
            CLOSE_GLYPH,
            host,
        )
    }

    fn reference(size: f32) -> Rectangle {
        bounds_of(reference_text(CLOSE_GLYPH, Some(size)), CLOSE_GLYPH, 16.0)
    }

    #[test]
    fn raising_the_body_text_grows_the_mark_with_it() {
        for body in [14.0_f32, 20.0, 28.0] {
            let got = mark(tokens_with_body(body), 16.0);
            let want = reference(1.125 * body);
            assert!(
                same_size(got, want),
                "body {body}px: the mark is {:.2} x {:.2}, 1.125 em of the body is {:.2} x {:.2}",
                got.width,
                got.height,
                want.width,
                want.height,
            );
        }
    }

    #[test]
    fn raising_the_host_text_with_default_tokens_leaves_the_mark_alone() {
        let tokens = tokens_with_body(14.0);
        let at_16 = mark(tokens, 16.0);
        let at_24 = mark(tokens, 24.0);
        assert!(
            same_size(at_16, at_24),
            "the host's text went 16 -> 24 and the design path's mark went from {:.2} x {:.2} \
             to {:.2} x {:.2}, following a size the toast's own text does not use",
            at_16.width,
            at_16.height,
            at_24.width,
            at_24.height,
        );
    }

    /// The control's target is measured against the same text: a body of 28
    /// makes `1.5 x 28 = 42`.
    #[test]
    fn the_close_target_follows_the_body_text_too() {
        let tokens = tokens_with_body(28.0);
        for b in control_boxes(snora::design::render(toast_layout(), tokens), 16.0) {
            assert!(
                b.width >= 41.99 && b.height >= 41.99,
                "the close target is {:.1} x {:.1}, under 1.5 x a 28px body",
                b.width,
                b.height,
            );
        }
    }
}
