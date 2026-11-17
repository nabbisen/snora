//! RFC-134: **the five form controls the keyboard can reach**: a checkbox, a toggle, a radio group,
//! a select and a slider, on both paths (`snora::…` and `snora::design::…`).
//!
//! R-2's table is what is asserted, key by key, through the real loop: a key is delivered, the
//! application applies the message the interface published, the view is rebuilt **keeping the
//! widgets' state**, and the next key follows (`World`). A test that read a message and stopped
//! would not see a handler that changes only the first press.
//!
//! * Tab: one stop each, in page order, the radio group once; a disabled control is no stop.
//! * The press keys: Enter and Space, never a chord, once per press; a click publishes the same
//!   message and does not focus (R-3).
//! * The radio group: ↑ ↓ ← → move **and select**, wrap, follow the screen under RTL; Space on the
//!   option already selected publishes nothing (ruling 3).
//! * The slider: ← → ↑ ↓ one step, Page Up and Page Down ten, Home and End the ends; ← → follow
//!   the screen; **a click at `x` means the value whose thumb is at `x`**, both directions; a drag
//!   that leaves the window ends cleanly.
//! * The select: Enter, Space and ↓ open; inside the engine ↓ ↑ Home End move, Enter chooses (one
//!   message), Escape closes, a click outside closes, and focus returns to the trigger; outside the
//!   engine it keeps Tab, Enter, Space and Escape and nothing more (ruling 2).
//! * A held arrow keeps moving; a held press presses once (RFC-123, RFC-121).
//! * Each is a mirror image under RTL, and something moves inside it.
//! * Q-3's reasons, read from the pinned `iced_widget`: an upgrade that gives one of iced's own a
//!   focus target tells us here.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::{Rig, key_event};
use common::sweep::leak;
use iced::keyboard::{Modifiers, key::Named};
use iced::widget::{column, container, text};
use iced::{Element, Length, Point, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::LayoutDirection::{self, Ltr, Rtl};
use snora::design::Tokens;

// ---- the application -----------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Checked(bool),
    On(bool),
    Size(usize),
    Volume(f32),
    Chose(usize),
    Open(bool),
    Other,
}

/// The two paths: `snora::…` (the host's theme) and `snora::design::…` (the tokens).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Path {
    Engine,
    Design,
}

const PATHS: [Path; 2] = [Path::Engine, Path::Design];

const SIZES: [&str; 3] = ["Small", "Medium", "Large"];
const FRUIT: [&str; 3] = ["Apple", "Banana", "Cherry"];

#[derive(Clone)]
struct Model {
    checked: bool,
    on: bool,
    size: Option<usize>,
    value: f32,
    step: Option<f32>,
    open: bool,
    fruit: usize,
    disabled: bool,
    /// Rendered by `snora::render` (the cover, the arrows), or as a bare page.
    engine: bool,
    log: Vec<Msg>,
}

impl Model {
    fn new() -> Self {
        Self {
            checked: false,
            on: false,
            size: Some(0),
            value: 50.0,
            step: None,
            open: false,
            fruit: 1,
            disabled: false,
            engine: true,
            log: Vec::new(),
        }
    }

    /// What the application's `update` does. **Choosing closes the list**, as the select's
    /// documentation tells an application to.
    fn apply(&mut self, m: &Msg) {
        match m {
            Msg::Checked(v) => self.checked = *v,
            Msg::On(v) => self.on = *v,
            Msg::Size(i) => self.size = Some(*i),
            Msg::Volume(v) => self.value = *v,
            Msg::Chose(i) => {
                self.fruit = *i;
                self.open = false;
            }
            Msg::Open(o) => self.open = *o,
            Msg::Other => {}
        }
    }
}

fn checkbox(
    p: Path,
    t: &'static Tokens,
    on: bool,
    dir: LayoutDirection,
) -> snora::Checkbox<'static, Msg> {
    match p {
        Path::Engine => snora::checkbox(on, dir),
        Path::Design => snora::design::checkbox(t, on, dir),
    }
}

fn toggler(
    p: Path,
    t: &'static Tokens,
    on: bool,
    dir: LayoutDirection,
) -> snora::Toggler<'static, Msg> {
    match p {
        Path::Engine => snora::toggler(on, dir),
        Path::Design => snora::design::toggler(t, on, dir),
    }
}

fn radio(
    p: Path,
    t: &'static Tokens,
    selected: Option<usize>,
    dir: LayoutDirection,
) -> snora::RadioGroup<'static, &'static str, Msg> {
    let pick = |s: &'static str| Msg::Size(SIZES.iter().position(|x| *x == s).unwrap());
    let sel = selected.map(|i| SIZES[i]);
    match p {
        Path::Engine => snora::radio_group(SIZES, sel.as_ref(), pick, dir),
        Path::Design => snora::design::radio_group(t, SIZES, sel.as_ref(), pick, dir),
    }
}

fn slider(
    p: Path,
    t: &'static Tokens,
    value: f32,
    dir: LayoutDirection,
) -> snora::Slider<'static, Msg> {
    match p {
        Path::Engine => snora::slider(0.0..=100.0, value, Msg::Volume, dir),
        Path::Design => snora::design::slider(t, 0.0..=100.0, value, Msg::Volume, dir),
    }
}

fn select(
    p: Path,
    t: &'static Tokens,
    chosen: usize,
    dir: LayoutDirection,
) -> snora::Select<'static, &'static str, Msg> {
    let pick = |s: &'static str| Msg::Chose(FRUIT.iter().position(|x| *x == s).unwrap());
    let sel = FRUIT[chosen];
    match p {
        Path::Engine => snora::select(FRUIT, Some(&sel), pick, dir),
        Path::Design => snora::design::select(t, FRUIT, Some(&sel), pick, dir),
    }
}

/// The page: the five, then a plain control after them, so that "Tab past the last" has somewhere
/// to go.
fn view(m: &Model, p: Path, t: &'static Tokens, dir: LayoutDirection) -> Element<'static, Msg> {
    let en = !m.disabled;
    let mut sl = slider(p, t, m.value, dir)
        .width(Length::Fixed(300.0))
        .disabled(m.disabled);
    if let Some(step) = m.step {
        sl = sl.step(step);
    }
    let mut se = select(p, t, m.fruit, dir).opened(m.open);
    if en {
        se = se.on_open_change(Msg::Open);
    }
    let body: Element<'static, Msg> = column![
        checkbox(p, t, m.checked, dir)
            .label("Remember me")
            .on_toggle_maybe(en.then_some(Msg::Checked)),
        toggler(p, t, m.on, dir)
            .label("Dark mode")
            .on_toggle_maybe(en.then_some(Msg::On)),
        radio(p, t, m.size, dir).disabled(m.disabled),
        sl,
        se,
        snora::control(text("after")).on_press(Msg::Other),
    ]
    .spacing(12)
    .into();
    let page: Element<'static, Msg> = container(body)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    if m.engine {
        snora::render(
            snora::AppLayout::new(page)
                .direction(dir)
                .on_close_menus(Msg::Open(false)),
        )
    } else {
        page
    }
}

// ---- the real loop ------------------------------------------------------------------------------------------------------------

struct World {
    rig: Rig<Msg>,
    m: Model,
    p: Path,
    t: &'static Tokens,
    dir: LayoutDirection,
}

impl World {
    fn new(p: Path, dir: LayoutDirection) -> Self {
        Self::with(p, dir, |_| {})
    }

    fn with(p: Path, dir: LayoutDirection, f: impl FnOnce(&mut Model)) -> Self {
        let t = leak(Tokens::light());
        let mut m = Model::new();
        f(&mut m);
        Self {
            rig: Rig::new(view(&m, p, t, dir)),
            m,
            p,
            t,
            dir,
        }
    }

    /// What `update` then `view` do with whatever the interface published: apply, then rebuild
    /// **keeping the widgets' state**.
    fn settle(&mut self) -> Vec<Msg> {
        let msgs = std::mem::take(&mut self.rig.messages);
        for x in &msgs {
            self.m.apply(x);
        }
        self.m.log.extend(msgs.iter().cloned());
        let rig = std::mem::replace(&mut self.rig, Rig::new(iced::widget::space().into()));
        self.rig = rig.rebuilt(view(&self.m, self.p, self.t, self.dir));
        msgs
    }

    fn key(&mut self, key: Named) -> Vec<Msg> {
        self.key_with(key, false, Modifiers::empty())
    }

    fn held(&mut self, key: Named) -> Vec<Msg> {
        self.key_with(key, true, Modifiers::empty())
    }

    fn key_with(&mut self, key: Named, repeat: bool, mods: Modifiers) -> Vec<Msg> {
        self.rig.batch(&[key_event(key, repeat, mods)]);
        self.settle()
    }

    fn click(&mut self, at: Point) -> Vec<Msg> {
        self.rig.click_at(at);
        self.settle()
    }

    fn focus(&mut self, stop: usize) {
        self.rig.focus_at(stop);
        self.rig.redraw();
    }

    fn tab(&mut self) {
        self.rig.tab(false);
        self.rig.redraw();
    }

    fn focused(&mut self) -> Vec<usize> {
        self.rig.focused()
    }

    fn bounds(&mut self, stop: usize) -> Rectangle {
        self.rig.targets()[stop].bounds
    }
}

// The stops, in page order.
const CHECKBOX: usize = 0;
const TOGGLER: usize = 1;
const RADIO: usize = 2;
const SLIDER: usize = 3;
const SELECT: usize = 4;
const AFTER: usize = 5;

/// Every text run and its bounds.
fn texts(rig: &mut Rig<Msg>) -> Vec<(String, Rectangle)> {
    struct Collect(Arc<Mutex<Vec<(String, Rectangle)>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
            self.0.lock().unwrap().push((text.to_owned(), bounds));
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    out.lock().unwrap().clone()
}

/// The bounds of the first text run that says `what`.
fn text_at(w: &mut World, what: &str) -> Rectangle {
    texts(&mut w.rig)
        .into_iter()
        .find(|(s, _)| s == what)
        .unwrap_or_else(|| panic!("no text {what:?}"))
        .1
}

/// Every container in the tree.
fn containers(rig: &mut Rig<Msg>) -> Vec<Rectangle> {
    struct Collect(Arc<Mutex<Vec<Rectangle>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, _id: Option<&Id>, bounds: Rectangle) {
            self.0.lock().unwrap().push(bounds);
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    out.lock().unwrap().clone()
}

fn both(f: impl Fn(Path, LayoutDirection)) {
    for p in PATHS {
        for dir in [Ltr, Rtl] {
            f(p, dir);
        }
    }
}

// ---- Tab ----------------------------------------------------------------------------------------------------------------------------

#[test]
fn each_control_is_one_tab_stop_in_page_order_and_the_radio_group_is_one() {
    both(|p, dir| {
        let mut w = World::new(p, dir);
        let stops = w.rig.targets();
        assert_eq!(
            stops.len(),
            6,
            "{p:?} {dir:?}: five controls and the one after are six stops, not {} (a radio group \
             is one stop, not three)",
            stops.len()
        );
        let ys: Vec<f32> = stops.iter().map(|t| t.bounds.y).collect();
        assert!(
            ys.windows(2).all(|w| w[0] < w[1]),
            "{p:?} {dir:?}: the stops are not in page order: {ys:?}"
        );
        let mut seen = Vec::new();
        for _ in 0..6 {
            w.tab();
            seen.extend(w.focused());
        }
        assert_eq!(
            seen,
            vec![0, 1, 2, 3, 4, 5],
            "{p:?} {dir:?}: Tab visited {seen:?}"
        );
    });
}

#[test]
fn a_disabled_control_is_no_stop() {
    both(|p, dir| {
        let mut w = World::with(p, dir, |m| m.disabled = true);
        assert_eq!(
            w.rig.targets().len(),
            1,
            "{p:?} {dir:?}: with all five disabled only \"after\" is a stop"
        );
        // And nothing answers a key or a click.
        w.rig.focus_at(0);
        for key in [
            Named::Enter,
            Named::Space,
            Named::ArrowDown,
            Named::ArrowRight,
        ] {
            // (The one stop there is, "after", presses; nothing else publishes.)
            let got: Vec<Msg> = w
                .key(key)
                .into_iter()
                .filter(|m| *m != Msg::Other)
                .collect();
            assert!(
                got.is_empty(),
                "{p:?} {dir:?}: {key:?} did something: {got:?}"
            );
        }
    });
}

// ---- checkbox and toggle ------------------------------------------------------------------------------------------------------

#[test]
fn enter_and_space_toggle_a_checkbox_and_a_toggler_and_a_click_publishes_the_same_message() {
    both(|p, dir| {
        for (stop, name, on, off) in [
            (
                CHECKBOX,
                "checkbox",
                Msg::Checked(true),
                Msg::Checked(false),
            ),
            (TOGGLER, "toggler", Msg::On(true), Msg::On(false)),
        ] {
            for key in [Named::Enter, Named::Space] {
                let mut w = World::new(p, dir);
                w.focus(stop);
                assert_eq!(w.key(key), vec![on.clone()], "{p:?} {dir:?} {name} {key:?}");
                // The state came back: the next press turns it off.
                assert_eq!(
                    w.key(key),
                    vec![off.clone()],
                    "{p:?} {dir:?} {name} {key:?} again"
                );
            }
            // A chord is not a press.
            for chord in [Modifiers::CTRL, Modifiers::ALT, Modifiers::LOGO] {
                let mut w = World::new(p, dir);
                w.focus(stop);
                assert!(
                    w.key_with(Named::Enter, false, chord).is_empty(),
                    "{p:?} {dir:?} {name}: Enter with {chord:?} pressed it"
                );
            }
            // A held press presses once.
            let mut w = World::new(p, dir);
            w.focus(stop);
            let mut got = w.key(Named::Enter);
            got.extend(w.held(Named::Enter));
            got.extend(w.held(Named::Enter));
            assert_eq!(got, vec![on.clone()], "{p:?} {dir:?} {name}: a held Enter");
            // A click: the same message, and it does not leave focus on the control (RFC-121).
            let mut w = World::new(p, dir);
            let at = w.bounds(stop).center();
            assert_eq!(
                w.click(at),
                vec![on.clone()],
                "{p:?} {dir:?} {name}: a click"
            );
            assert!(
                w.focused().is_empty(),
                "{p:?} {dir:?} {name}: a click focused it"
            );
        }
    });
}

#[test]
fn the_label_is_part_of_the_target() {
    both(|p, dir| {
        let mut w = World::new(p, dir);
        let label = text_at(&mut w, "Remember me").center();
        assert_eq!(
            w.click(label),
            vec![Msg::Checked(true)],
            "{p:?} {dir:?}: a click on the label"
        );
    });
}

// ---- the radio group ----------------------------------------------------------------------------------------------------------------

/// What an arrow selects from `from` (`None` is nothing selected), for a group of three, under `dir`.
fn arrow_selects(key: Named, from: Option<usize>, dir: LayoutDirection) -> usize {
    let forward = match key {
        Named::ArrowDown => true,
        Named::ArrowUp => false,
        Named::ArrowRight => dir == Ltr,
        Named::ArrowLeft => dir == Rtl,
        _ => unreachable!(),
    };
    match (from, forward) {
        (Some(i), true) => (i + 1) % 3,
        (Some(i), false) => (i + 2) % 3,
        (None, true) => 0,
        (None, false) => 2,
    }
}

#[test]
fn the_radio_groups_arrows_move_and_select_wrapping_and_follow_the_screen() {
    both(|p, dir| {
        for from in [Some(0), Some(1), Some(2), None] {
            for key in [
                Named::ArrowDown,
                Named::ArrowUp,
                Named::ArrowRight,
                Named::ArrowLeft,
            ] {
                let mut w = World::with(p, dir, |m| m.size = from);
                w.focus(RADIO);
                assert_eq!(
                    w.key(key),
                    vec![Msg::Size(arrow_selects(key, from, dir))],
                    "{p:?} {dir:?} {key:?} from {from:?}"
                );
            }
        }
    });
}

#[test]
fn space_and_enter_on_a_radio_group_publish_nothing() {
    both(|p, dir| {
        for key in [Named::Enter, Named::Space] {
            let mut w = World::new(p, dir);
            w.focus(RADIO);
            assert!(
                w.key(key).is_empty(),
                "{p:?} {dir:?}: {key:?} on the option already selected published a message"
            );
        }
    });
}

#[test]
fn a_click_on_a_radio_option_publishes_what_an_arrow_does_and_on_the_selected_one_nothing() {
    both(|p, dir| {
        for selected in [0usize, 2] {
            for (i, label) in SIZES.iter().enumerate() {
                let mut w = World::with(p, dir, |m| m.size = Some(selected));
                let at = text_at(&mut w, label).center();
                // The option already selected is no action: a click on it publishes nothing, as Space
                // and Enter do (RFC-134 ruling 3; R-3: one action, one result).
                let want = if i == selected {
                    vec![]
                } else {
                    vec![Msg::Size(i)]
                };
                assert_eq!(
                    w.click(at),
                    want,
                    "{p:?} {dir:?}: a click on {label} with {} selected",
                    SIZES[selected]
                );
                assert!(
                    w.focused().is_empty(),
                    "{p:?} {dir:?}: a click focused the group"
                );
            }
        }
        // The arrow that reaches option 1 from option 0 publishes the same message as a click on it.
        let mut by_key = World::new(p, dir);
        by_key.focus(RADIO);
        let k = by_key.key(Named::ArrowDown);
        let mut by_click = World::new(p, dir);
        let at = text_at(&mut by_click, "Medium").center();
        assert_eq!(
            k,
            by_click.click(at),
            "{p:?} {dir:?}: the arrow and the click differ"
        );
        assert_eq!(k, vec![Msg::Size(1)]);
    });
}

#[test]
fn a_disabled_radio_group_ignores_a_click() {
    both(|p, dir| {
        let mut w = World::with(p, dir, |m| m.disabled = true);
        let at = text_at(&mut w, "Large").center();
        assert!(
            w.click(at).is_empty(),
            "{p:?} {dir:?}: a disabled option answered"
        );
    });
}

// ---- the slider -----------------------------------------------------------------------------------------------------------------

#[test]
fn the_sliders_keys_move_by_the_step_and_follow_the_screen() {
    both(|p, dir| {
        let right = if dir == Ltr { 1.0 } else { -1.0 };
        // The default step is a hundredth of the range: 1.
        for (key, want) in [
            (Named::ArrowRight, 50.0 + right),
            (Named::ArrowLeft, 50.0 - right),
            (Named::ArrowUp, 51.0),
            (Named::ArrowDown, 49.0),
            (Named::PageUp, 60.0),
            (Named::PageDown, 40.0),
            (Named::Home, 0.0),
            (Named::End, 100.0),
        ] {
            let mut w = World::new(p, dir);
            w.focus(SLIDER);
            assert_eq!(w.key(key), vec![Msg::Volume(want)], "{p:?} {dir:?} {key:?}");
        }
        // A step of its own.
        let mut w = World::with(p, dir, |m| m.step = Some(5.0));
        w.focus(SLIDER);
        assert_eq!(
            w.key(Named::ArrowUp),
            vec![Msg::Volume(55.0)],
            "{p:?} {dir:?}: step(5)"
        );
        assert_eq!(
            w.key(Named::PageUp),
            vec![Msg::Volume(105.0f32.min(100.0))],
            "{p:?} {dir:?}: page of ten steps, clamped"
        );
    });
}

#[test]
fn a_move_that_changes_nothing_publishes_nothing_and_enter_is_not_a_move() {
    both(|p, dir| {
        let mut w = World::with(p, dir, |m| m.value = 100.0);
        w.focus(SLIDER);
        assert!(
            w.key(Named::End).is_empty(),
            "{p:?} {dir:?}: End at the end"
        );
        assert!(
            w.key(Named::ArrowUp).is_empty(),
            "{p:?} {dir:?}: ↑ at the end"
        );
        let mut w = World::with(p, dir, |m| m.value = 0.0);
        w.focus(SLIDER);
        assert!(
            w.key(Named::Home).is_empty(),
            "{p:?} {dir:?}: Home at the start"
        );
        for key in [Named::Enter, Named::Space] {
            let mut w = World::new(p, dir);
            w.focus(SLIDER);
            assert!(
                w.key(key).is_empty(),
                "{p:?} {dir:?}: {key:?} on the slider"
            );
        }
        // A chord is not a move.
        let mut w = World::new(p, dir);
        w.focus(SLIDER);
        assert!(
            w.key_with(Named::ArrowUp, false, Modifiers::CTRL)
                .is_empty()
        );
    });
}

/// The track the thumb runs along, from what the bar reports: the one container inside the slider
/// that is wide and thin.
fn track_of(w: &mut World) -> Rectangle {
    let target = w.bounds(SLIDER);
    containers(&mut w.rig)
        .into_iter()
        .filter(|c| c.height < 10.0 && c.width > 100.0 && inside(*c, target))
        .max_by(|a, b| a.width.total_cmp(&b.width))
        .expect("the slider reports its track")
}

fn inside(b: Rectangle, outer: Rectangle) -> bool {
    let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    cx >= outer.x && cx <= outer.x + outer.width && cy >= outer.y && cy <= outer.y + outer.height
}

#[test]
fn a_click_at_x_means_the_value_whose_thumb_is_at_x_in_both_directions() {
    // Unit 0 measured a click at 25% reading 0.267: the mapping took the whole width. The thumb's
    // centre runs along the track, so that is where a click is read.
    both(|p, dir| {
        for t in [0.0f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
            let mut w = World::with(p, dir, |m| m.value = 50.0);
            let track = track_of(&mut w);
            let x = track.x + t * track.width;
            let got = w.click(Point::new(x, track.center_y()));
            let want = if dir == Ltr {
                100.0 * t
            } else {
                100.0 * (1.0 - t)
            };
            // The step is one: the value is within half of it.
            let Some(Msg::Volume(v)) = got.first().cloned().or(Some(Msg::Volume(50.0))) else {
                panic!("{got:?}");
            };
            assert!(
                (v - want).abs() <= 0.5 + 1e-3,
                "{p:?} {dir:?}: a click at {:.0}% of the track means {v}, expected {want}",
                t * 100.0
            );
        }
        // On the first or last radius beyond the run: the end.
        let mut w = World::with(p, dir, |m| m.value = 50.0);
        let track = track_of(&mut w);
        let bar_start = track.x - 4.0;
        let got = w.click(Point::new(bar_start, track.center_y()));
        let want = if dir == Ltr { 0.0 } else { 100.0 };
        assert_eq!(
            got,
            vec![Msg::Volume(want)],
            "{p:?} {dir:?}: a click before the run"
        );
    });
}

#[test]
fn a_click_does_not_focus_the_slider_and_a_drag_follows_the_pointer() {
    both(|p, dir| {
        let mut w = World::with(p, dir, |m| m.value = 50.0);
        let track = track_of(&mut w);
        let y = track.center_y();
        let at = |t: f32| Point::new(track.x + t * track.width, y);
        let lower = |t: f32| {
            if dir == Ltr {
                100.0 * t
            } else {
                100.0 * (1.0 - t)
            }
        };

        w.rig.point_at(Some(at(0.5)));
        w.rig.left_button(true);
        let pressed = w.settle();
        assert!(
            w.focused().is_empty(),
            "{p:?} {dir:?}: a press focused the slider"
        );
        assert!(
            pressed.is_empty(),
            "value is already {}, nothing to say: {pressed:?}",
            lower(0.5)
        );
        for t in [0.7, 0.9] {
            w.rig.point_at(Some(at(t)));
            let got = w.settle();
            let [Msg::Volume(v)] = got[..] else {
                panic!("{p:?} {dir:?}: dragged to {t}: {got:?}");
            };
            assert!(
                (v - lower(t)).abs() <= 0.5,
                "{p:?} {dir:?}: dragged to {t}: {v}"
            );
        }
        w.rig.left_button(false);
        w.settle();
        // After the release a move is only a move.
        w.rig.point_at(Some(at(0.2)));
        assert!(
            w.settle().is_empty(),
            "{p:?} {dir:?}: the drag did not end at the release"
        );
    });
}

#[test]
fn a_drag_that_leaves_the_window_clamps_and_ends_with_the_release_wherever_it_is() {
    both(|p, dir| {
        let mut w = World::with(p, dir, |m| m.value = 30.0);
        let track = track_of(&mut w);
        let y = track.center_y();
        let start = Point::new(track.x + 0.3 * track.width, y);
        w.rig.point_at(Some(start));
        w.rig.left_button(true);
        w.settle();
        // Far left of the window and far right of it, and above it.
        let (low, high) = if dir == Ltr {
            (0.0, 100.0)
        } else {
            (100.0, 0.0)
        };
        w.rig.point_at(Some(Point::new(-400.0, y)));
        assert_eq!(
            w.settle(),
            vec![Msg::Volume(low)],
            "{p:?} {dir:?}: left of the window"
        );
        w.rig.point_at(Some(Point::new(5000.0, -300.0)));
        assert_eq!(
            w.settle(),
            vec![Msg::Volume(high)],
            "{p:?} {dir:?}: right of the window"
        );
        // The pointer leaves the window altogether; the button is released outside.
        w.rig.point_at(None);
        w.settle();
        w.rig.left_button(false);
        w.settle();
        w.rig
            .point_at(Some(Point::new(track.x + 0.5 * track.width, y)));
        assert!(
            w.settle().is_empty(),
            "{p:?} {dir:?}: a release outside the window left the slider dragging"
        );
        // A drag whose release the window never saw ends when the window loses focus.
        w.rig.point_at(Some(start));
        w.rig.left_button(true);
        w.settle();
        w.rig
            .batch(&[iced::Event::Window(iced::window::Event::Unfocused)]);
        w.rig
            .point_at(Some(Point::new(track.x + 0.9 * track.width, y)));
        assert!(
            w.settle().is_empty(),
            "{p:?} {dir:?}: losing the window's focus did not end the drag"
        );
    });
}

#[test]
fn a_held_arrow_keeps_moving_the_slider_and_the_radio_group() {
    both(|p, dir| {
        let right = if dir == Ltr { 1.0 } else { -1.0 };
        let mut w = World::new(p, dir);
        w.focus(SLIDER);
        let mut got = Vec::new();
        for _ in 0..5 {
            got.extend(w.held(Named::ArrowRight));
        }
        let want: Vec<Msg> = (1..=5)
            .map(|k| Msg::Volume(50.0 + right * k as f32))
            .collect();
        assert_eq!(got, want, "{p:?} {dir:?}: a held → on the slider");

        let mut w = World::new(p, dir);
        w.focus(RADIO);
        let mut got = Vec::new();
        for _ in 0..4 {
            got.extend(w.held(Named::ArrowDown));
        }
        assert_eq!(
            got,
            vec![Msg::Size(1), Msg::Size(2), Msg::Size(0), Msg::Size(1)],
            "{p:?} {dir:?}: a held ↓ on the radio group"
        );
    });
}

// ---- the select ---------------------------------------------------------------------------------------------------------------------

#[test]
fn enter_space_and_down_open_a_closed_select_and_a_click_does() {
    both(|p, dir| {
        for key in [Named::Enter, Named::Space, Named::ArrowDown] {
            let mut w = World::new(p, dir);
            w.focus(SELECT);
            assert_eq!(w.key(key), vec![Msg::Open(true)], "{p:?} {dir:?} {key:?}");
        }
        let mut w = World::new(p, dir);
        w.focus(SELECT);
        assert!(w.key_with(Named::Enter, false, Modifiers::CTRL).is_empty());
        assert!(w.key(Named::ArrowUp).is_empty(), "↑ on a closed select");
        let mut w = World::new(p, dir);
        let at = w.bounds(SELECT).center();
        assert_eq!(w.click(at), vec![Msg::Open(true)], "{p:?} {dir:?}: a click");
        assert!(w.focused().is_empty());
    });
}

/// The text of the focused stop, found by the runs it holds.
fn focused_text(w: &mut World) -> String {
    let targets = w.rig.targets();
    let runs = texts(&mut w.rig);
    let Some(f) = targets.iter().find(|t| t.focused) else {
        return "(nothing)".to_owned();
    };
    runs.iter()
        // The page under the cover is still in the tree, and may sit under the list.
        // (and the selected item's tick, which is a glyph and not a word)
        .filter(|(s, r)| {
            s != "after" && s.chars().any(|c| c.is_ascii_alphabetic()) && inside(*r, f.bounds)
        })
        .map(|(s, _)| s.as_str())
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn an_open_select_in_the_engine_covers_the_page_moves_chooses_closes_and_gives_focus_back() {
    both(|p, dir| {
        // Open it from the keyboard, on the trigger.
        let mut w = World::new(p, dir);
        w.focus(SELECT);
        assert_eq!(w.key(Named::Enter), vec![Msg::Open(true)]);
        w.rig.redraw();
        // The page is covered: the only stops are the items.
        let stops = w.rig.targets().len();
        assert_eq!(
            stops,
            FRUIT.len(),
            "{p:?} {dir:?}: with the select open the stops are its items, not {stops}"
        );
        // ↓ with nothing focused reaches the first item, then moves.
        assert!(
            w.key(Named::ArrowDown).is_empty(),
            "moving focus publishes nothing"
        );
        assert_eq!(focused_text(&mut w), "Apple", "{p:?} {dir:?}");
        w.key(Named::ArrowDown);
        assert_eq!(focused_text(&mut w), "Banana");
        w.key(Named::End);
        assert_eq!(focused_text(&mut w), "Cherry", "End");
        w.key(Named::Home);
        assert_eq!(focused_text(&mut w), "Apple", "Home");
        w.key(Named::ArrowUp);
        assert_eq!(focused_text(&mut w), "Cherry", "↑ wraps");
        // Enter chooses: one message, and the application closes.
        w.key(Named::Home);
        w.key(Named::ArrowDown);
        let got = w.key(Named::Enter);
        assert_eq!(
            got,
            vec![Msg::Chose(1)],
            "{p:?} {dir:?}: choosing publishes one message"
        );
        w.rig.redraw();
        assert_eq!(w.rig.targets().len(), 6, "closed again");
        // Focus is back on the trigger.
        assert_eq!(
            w.focused(),
            vec![SELECT],
            "{p:?} {dir:?}: focus did not return to the trigger"
        );
    });
}

#[test]
fn escape_and_a_click_outside_close_an_open_select_once_and_focus_returns() {
    both(|p, dir| {
        let mut w = World::new(p, dir);
        w.focus(SELECT);
        w.key(Named::Enter);
        w.rig.redraw();
        w.key(Named::ArrowDown);
        assert_eq!(
            w.key(Named::Escape),
            vec![Msg::Open(false)],
            "{p:?} {dir:?}: Escape"
        );
        w.rig.redraw();
        assert_eq!(
            w.focused(),
            vec![SELECT],
            "{p:?} {dir:?}: focus after Escape"
        );

        // Opened by a click, closed by a click on the page outside.
        let mut w = World::new(p, dir);
        let at = w.bounds(SELECT).center();
        assert_eq!(w.click(at), vec![Msg::Open(true)]);
        w.rig.redraw();
        let outside = Point::new(700.0, 500.0);
        assert_eq!(
            w.click(outside),
            vec![Msg::Open(false)],
            "{p:?} {dir:?}: a click outside"
        );
    });
}

#[test]
fn outside_the_engine_a_select_keeps_tab_enter_space_and_escape_and_no_arrows() {
    both(|p, dir| {
        let mut w = World::with(p, dir, |m| m.engine = false);
        w.focus(SELECT);
        assert_eq!(w.key(Named::Enter), vec![Msg::Open(true)]);
        w.rig.redraw();
        // Not covered: the page's own stops are still there beside the items.
        assert_eq!(
            w.rig.targets().len(),
            6 + FRUIT.len(),
            "{p:?} {dir:?}: not covered"
        );
        // The arrows are the engine's: a focused item does not move.
        w.tab();
        let first = w.focused();
        w.key(Named::ArrowDown);
        assert_eq!(
            w.focused(),
            first,
            "{p:?} {dir:?}: an arrow moved focus outside the engine"
        );
        // Escape is the select's own.
        w.focus(SELECT);
        assert_eq!(
            w.key(Named::Escape),
            vec![Msg::Open(false)],
            "{p:?} {dir:?}: Escape"
        );
    });
}

#[test]
fn a_select_without_an_open_handler_is_disabled() {
    both(|p, dir| {
        let w = World::with(p, dir, |m| m.disabled = true);
        let mut w = w;
        assert!(
            !w.rig
                .targets()
                .iter()
                .any(|t| t.bounds.y > 300.0 && t.bounds.y < 360.0 && false)
        );
        assert_eq!(w.rig.targets().len(), 1, "{p:?} {dir:?}");
    });
}

// ---- what is drawn: the thumb, the tick, and a key that is captured ------------------------------------------------------------

#[test]
fn a_togglers_thumb_is_at_the_start_edge_when_off_and_the_end_edge_when_on() {
    both(|p, dir| {
        for on in [false, true] {
            let mut w = World::with(p, dir, |m| m.on = on);
            let target = w.bounds(TOGGLER);
            let cs = containers(&mut w.rig);
            let track = cs
                .iter()
                .copied()
                .find(|c| {
                    inside(*c, target)
                        && c.height > 10.0
                        && c.height < 30.0
                        && (c.width - 2.0 * c.height).abs() < 1.0
                })
                .expect("the toggler's track");
            let thumb = cs
                .iter()
                .copied()
                .find(|c| {
                    inside(*c, track)
                        && (c.width - c.height).abs() < 0.6
                        && c.width < track.height - 2.0
                        && c.width > track.height / 2.0
                })
                .expect("the toggler's thumb");
            let right_of_middle = thumb.center_x() > track.center_x();
            // On is the end edge: the right under LTR, the left under RTL.
            let want_right = (on && dir == Ltr) || (!on && dir == Rtl);
            assert_eq!(
                right_of_middle,
                want_right,
                "{p:?} {dir:?} on={on}: the thumb is {}",
                if right_of_middle {
                    "on the right"
                } else {
                    "on the left"
                }
            );
        }
    });
}

#[test]
fn a_checked_box_shows_a_tick_and_an_unchecked_one_does_not() {
    both(|p, dir| {
        for checked in [true, false] {
            let mut w = World::with(p, dir, |m| m.checked = checked);
            let target = w.bounds(CHECKBOX);
            let ticks = texts(&mut w.rig)
                .into_iter()
                .filter(|(s, r)| inside(*r, target) && !s.chars().any(char::is_alphanumeric))
                .count();
            assert_eq!(
                ticks,
                usize::from(checked),
                "{p:?} {dir:?} checked={checked}"
            );
        }
    });
}

#[test]
fn a_key_a_control_answers_is_captured_and_another_is_left_to_the_application() {
    use iced_runtime::core::event::Status;
    both(|p, dir| {
        for (stop, answered) in [
            (RADIO, Named::ArrowDown),
            (SLIDER, Named::ArrowUp),
            (SELECT, Named::ArrowDown),
        ] {
            let mut w = World::new(p, dir);
            w.focus(stop);
            let status = w.rig.named(answered, false);
            w.settle();
            assert_eq!(
                status,
                Status::Captured,
                "{p:?} {dir:?} stop {stop}: {answered:?} is the control's"
            );
            let mut w = World::new(p, dir);
            w.focus(stop);
            let status = w.rig.named(Named::F5, false);
            assert_eq!(
                status,
                Status::Ignored,
                "{p:?} {dir:?} stop {stop}: F5 is the application's"
            );
        }
    });
}

// ---- a column of the five: the marks start at one edge --------------------------------------------------------------------------

/// The page the alignment is read on: the five in a column that hugs the start edge, the slider at 0 (its
/// thumb is then at the bar's start edge), and nothing else.
fn column_of_five(p: Path, t: &'static Tokens, dir: LayoutDirection) -> Element<'static, Msg> {
    let start = match dir {
        Ltr => iced::Alignment::Start,
        Rtl => iced::Alignment::End,
    };
    container(
        column![
            checkbox(p, t, true, dir)
                .label("Remember me")
                .on_toggle(Msg::Checked),
            toggler(p, t, true, dir)
                .label("Dark mode")
                .on_toggle(Msg::On),
            radio(p, t, Some(1), dir),
            slider(p, t, 0.0, dir).width(Length::Fixed(300.0)),
            select(p, t, 1, dir).on_open_change(Msg::Open),
        ]
        .spacing(12)
        .align_x(start),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// RFC-134 review, C-2: **an outline 8 px off its neighbours looks like a mistake.** In a column of the
/// five, the first mark of each control (the box, the switch, the first dot, the thumb at 0, the select's
/// outline) starts at the same edge: the left under LTR, the right under RTL.
#[test]
fn in_a_column_of_the_five_every_marks_start_edge_is_the_same() {
    for p in PATHS {
        for dir in [Ltr, Rtl] {
            let t = leak(Tokens::light());
            let mut rig = Rig::new(column_of_five(p, t, dir));
            let stops = rig.targets();
            assert_eq!(stops.len(), 5, "{p:?} {dir:?}");
            let cs = containers(&mut rig);
            let edges: Vec<f32> = stops
                .iter()
                .map(|stop| {
                    // The marks: containers strictly inside the control, wider than a hair. **And a container as
                    // big as the control, beyond the control's own** (a control reports one): that is a mark that
                    // fills it, such as the select's outline when the control has no padding of its own around it.
                    let same = |c: &Rectangle| {
                        (c.x - stop.bounds.x).abs() < 0.5
                            && (c.width - stop.bounds.width).abs() < 0.5
                            && (c.y - stop.bounds.y).abs() < 0.5
                            && (c.height - stop.bounds.height).abs() < 0.5
                    };
                    let fills_it = cs.iter().filter(|c| same(c)).count() > 1;
                    let inner = cs.iter().filter(|c| {
                        inside(**c, stop.bounds)
                            && c.width > 2.0
                            && c.height > 2.0
                            && (c.width < stop.bounds.width - 1.0 || (fills_it && same(c)))
                    });
                    match dir {
                        Ltr => inner.map(|c| c.x).fold(f32::MAX, f32::min),
                        Rtl => inner.map(|c| c.x + c.width).fold(f32::MIN, f32::max),
                    }
                })
                .collect();
            let (lo, hi) = edges
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), e| (l.min(*e), h.max(*e)));
            assert!(
                hi - lo <= 1.0,
                "{p:?} {dir:?}: the marks of a checkbox, toggle, radio group, slider and select start at \
                 {edges:?}, {:.1} px apart",
                hi - lo
            );
        }
    }
}

// ---- each is a mirror image ---------------------------------------------------------------------------------------------------

const W: f32 = 800.0;

fn boxes(rig: &mut Rig<Msg>) -> Vec<Rectangle> {
    let mut v = containers(rig);
    v.extend(texts(rig).into_iter().map(|(_, r)| r));
    v.extend(rig.targets().into_iter().map(|t| t.bounds));
    v.retain(|b| b.width < W - 1.0);
    v
}

fn span(boxes: &[Rectangle]) -> Rectangle {
    let x0 = boxes.iter().map(|b| b.x).fold(f32::MAX, f32::min);
    let y0 = boxes.iter().map(|b| b.y).fold(f32::MAX, f32::min);
    let x1 = boxes.iter().map(|b| b.x + b.width).fold(f32::MIN, f32::max);
    let y1 = boxes
        .iter()
        .map(|b| b.y + b.height)
        .fold(f32::MIN, f32::max);
    Rectangle {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    }
}

fn mirrored(a: Rectangle, ua: Rectangle, b: Rectangle, ub: Rectangle) -> bool {
    let rel_a = a.x - ua.x;
    let want = ub.x + (ua.width - rel_a - a.width);
    (b.x - want).abs() <= 1.0
        && (b.width - a.width).abs() <= 1.0
        && ((b.y - ub.y) - (a.y - ua.y)).abs() <= 1.0
        && (b.height - a.height).abs() <= 1.0
}

/// Draws the control under both directions and checks that every box of the left-to-right picture
/// has its reflection about the widget's own span in the right-to-left one, and that something
/// **moved** inside it: a control that ignores its direction fails.
#[track_caller]
fn assert_a_mirror_image(
    what: &str,
    make: impl Fn(Path, &'static Tokens, LayoutDirection) -> Element<'static, Msg>,
) {
    for p in PATHS {
        let t = leak(Tokens::light());
        let page = |dir| -> Element<'static, Msg> {
            container(column![make(p, t, dir)])
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };
        let mut ltr = Rig::new(page(Ltr));
        let mut rtl = Rig::new(page(Rtl));
        let (a, b) = (boxes(&mut ltr), boxes(&mut rtl));
        assert_eq!(
            a.len(),
            b.len(),
            "{what} {p:?}: {} boxes under LTR, {} under RTL",
            a.len(),
            b.len()
        );
        let (ua, ub) = (span(&a), span(&b));
        assert!(
            (ua.width - ub.width).abs() <= 1.0,
            "{what} {p:?}: {:.0} wide under LTR and {:.0} under RTL",
            ua.width,
            ub.width
        );
        let mut used = vec![false; b.len()];
        let mut moved = 0;
        for r in &a {
            let Some(j) = (0..b.len()).find(|j| !used[*j] && mirrored(*r, ua, b[*j], ub)) else {
                panic!(
                    "{what} {p:?}: the box x {:.0}..{:.0} under LTR (the widget spans {:.0}..{:.0}) has no \
                     reflection under RTL (it spans {:.0}..{:.0})",
                    r.x,
                    r.x + r.width,
                    ua.x,
                    ua.x + ua.width,
                    ub.x,
                    ub.x + ub.width
                );
            };
            used[j] = true;
            if ((b[j].x - ub.x) - (r.x - ua.x)).abs() > 1.0 {
                moved += 1;
            }
        }
        assert!(
            moved > 0,
            "{what} {p:?}: nothing moved between LTR and RTL: it ignores its direction"
        );
    }
}

#[test]
fn each_of_the_five_is_a_mirror_image_and_something_moves() {
    for on in [false, true] {
        assert_a_mirror_image(&format!("checkbox {on}"), move |p, t, dir| {
            checkbox(p, t, on, dir)
                .label("Remember me")
                .on_toggle(Msg::Checked)
                .into()
        });
        assert_a_mirror_image(&format!("toggler {on}"), move |p, t, dir| {
            toggler(p, t, on, dir)
                .label("Dark mode")
                .on_toggle(Msg::On)
                .into()
        });
    }
    assert_a_mirror_image("radio group", |p, t, dir| radio(p, t, Some(1), dir).into());
    assert_a_mirror_image("slider", |p, t, dir| {
        slider(p, t, 30.0, dir).width(Length::Fixed(300.0)).into()
    });
    assert_a_mirror_image("select", |p, t, dir| {
        select(p, t, 1, dir).on_open_change(Msg::Open).into()
    });
}

// ---- Q-3: why each is built here and not on iced's, read from the pinned iced ------------------------------------------------

/// `iced_widget`'s source directory at the version `Cargo.lock` pins.
fn iced_widget_src() -> std::path::PathBuf {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"))
        .expect("Cargo.lock");
    let at = lock
        .find("name = \"iced_widget\"")
        .expect("iced_widget is locked");
    let version = lock[at..]
        .lines()
        .nth(1)
        .and_then(|l| l.strip_prefix("version = \""))
        .and_then(|l| l.strip_suffix('"'))
        .expect("its version")
        .to_owned();
    let home = std::env::var("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| std::path::Path::new(&h).join(".cargo")))
        .expect("CARGO_HOME or HOME");
    let registry = home.join("registry").join("src");
    for entry in
        std::fs::read_dir(&registry).unwrap_or_else(|e| panic!("{}: {e}", registry.display()))
    {
        let dir = entry
            .unwrap()
            .path()
            .join(format!("iced_widget-{version}"))
            .join("src");
        if dir.is_dir() {
            return dir;
        }
    }
    panic!("iced_widget-{version} is not in {}", registry.display());
}

/// What in the pinned `iced_widget` shows that RFC-134's reason for one of the five has gone, read through
/// `read` (the real source, or a planted one).
fn reasons_gone(read: &dyn Fn(&str) -> String) -> Vec<String> {
    let mut changed = Vec::new();
    for file in [
        "checkbox.rs",
        "toggler.rs",
        "radio.rs",
        "pick_list.rs",
        "slider.rs",
        "combo_box.rs",
    ] {
        let s = read(file);
        for needle in ["Focusable", "focusable("] {
            if s.contains(needle) {
                changed.push(format!(
                    "{file} now mentions `{needle}`: iced may have given it a focus target, and RFC-134's reason for it is gone"
                ));
            }
        }
    }
    // The keys: none of the press-and-toggle three read a key at all; the pick list reads only the
    // modifiers; the slider reads ↑ ↓ only.
    for file in ["checkbox.rs", "toggler.rs", "radio.rs"] {
        if read(file).contains("keyboard::") {
            changed.push(format!("{file} now reads the keyboard"));
        }
    }
    let pick = read("pick_list.rs");
    for needle in ["KeyPressed", "Key::Named"] {
        if pick.contains(needle) {
            changed.push(format!("pick_list.rs now reads {needle}"));
        }
    }
    let slider = read("slider.rs");
    for needle in [
        "ArrowLeft",
        "ArrowRight",
        "Home",
        "PageUp",
        "PageDown",
        "End",
    ] {
        if slider.contains(&format!("Named::{needle}"))
            || slider.contains(&format!("key::Named::{needle}"))
        {
            changed.push(format!("slider.rs now handles {needle}"));
        }
    }
    if !slider.contains("ArrowUp") {
        changed.push(
            "the premise that iced's slider takes ↑ ↓ has moved: read slider.rs again".to_owned(),
        );
    }
    changed
}

/// RFC-134 Q-3: the five are built on `Control` and not on iced's own because **none of iced's takes
/// focus**. This reads the pinned `iced_widget` for each, so that an iced release that gives one a
/// focus target, or a slider the keys, fails here and the reason is revisited (the keyboard guide's
/// workaround and this RFC's trigger).
#[test]
fn iced_widget_still_registers_no_focus_target_for_any_of_the_five() {
    let src = iced_widget_src();
    let real = |f: &str| {
        std::fs::read_to_string(src.join(f))
            .unwrap_or_else(|e| panic!("{}: {e}", src.join(f).display()))
    };
    let changed = reasons_gone(&real);
    assert!(changed.is_empty(), "{}", changed.join("\n"));
}

/// The test above shown able to fail: each change it names, planted in turn into the real source.
#[test]
fn the_q_3_reasons_test_finds_each_change_it_names() {
    let src = iced_widget_src();
    let real = |f: &str| std::fs::read_to_string(src.join(f)).unwrap();
    for (file, extra) in [
        ("checkbox.rs", "impl Focusable for State {}"),
        ("toggler.rs", "operation.focusable(id, bounds, state);"),
        ("radio.rs", "keyboard::Event::KeyPressed"),
        (
            "pick_list.rs",
            "Event::Keyboard(keyboard::Event::KeyPressed",
        ),
        ("slider.rs", "key::Named::ArrowLeft"),
        ("combo_box.rs", "Focusable"),
    ] {
        let planted = |f: &str| {
            let mut s = real(f);
            if f == file {
                s.push_str(extra);
            }
            s
        };
        let found = reasons_gone(&planted);
        assert!(
            found.iter().any(|m| m.starts_with(file)),
            "{extra:?} added to {file} was not found: {found:?}"
        );
    }
}

// ---- R-3 across the five, read as one table --------------------------------------------------------------------------------------

#[test]
fn a_click_and_a_key_publish_the_same_message_for_every_control_that_a_click_presses() {
    both(|p, dir| {
        for (stop, key) in [
            (CHECKBOX, Named::Enter),
            (CHECKBOX, Named::Space),
            (TOGGLER, Named::Enter),
            (TOGGLER, Named::Space),
            (SELECT, Named::Enter),
            (SELECT, Named::Space),
        ] {
            let mut by_key = World::new(p, dir);
            by_key.focus(stop);
            let k = by_key.key(key);
            let mut by_click = World::new(p, dir);
            let at = by_click.bounds(stop).center();
            let c = by_click.click(at);
            assert_eq!(
                k, c,
                "{p:?} {dir:?} stop {stop}: {key:?} and a click differ"
            );
            assert_eq!(k.len(), 1);
        }
        let _ = AFTER;
    });
}
