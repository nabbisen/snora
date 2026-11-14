//! RFC-125: a covered layer gets no pointer.
//!
//! > Whatever the keyboard cannot reach (RFC-119), the pointer does not reach either: no
//! > hover, no pointer shape, no tooltip, no wheel. What covers it is a surface. A press on
//! > it and Escape close what is open (Law 7). Nothing else it receives does anything.
//!
//! These rows replace Unit 0's measurement file (`covered_pointer_unit0.rs`, which asserted
//! only its premises) and `menu_overlay.rs`'s `does_a_page_under_a_modal_dim_show_hover`, which
//! measured the hover without asserting it: `a_page_under_a_cover_shows_no_hover` is what took
//! its place.
//!
//! Hover and tooltips are read from **pixels**, after the `RedrawRequested` the runtime sends
//! before a draw (iced 0.14 widgets take their status there, not on the pointer move). Every
//! row asserts its **premise** first: the same scene with no cover does what the covered one is
//! said not to, so a reading of "nothing happened" cannot come from a scene in which nothing
//! could.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::removal::Rig;
use iced::widget::{button, column, pick_list, scrollable, slider, text, tooltip};
use iced::{Element, Length, Point, Rectangle, Size, Theme};
use iced_runtime::core::event::Status;
use iced_runtime::core::layout::{self, Layout};
use iced_runtime::core::widget::{Operation, Tree, tree};
use iced_runtime::core::{Clipboard, Event, Shell, Widget, mouse, renderer, window};
use snora::{
    AppLayout, Dialog, Menu, MenuAction, MenuItem, Sheet, Toast, ToastAttention, ToastIntent,
};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    CloseModals,
    CloseMenus,
    Page,
    Slide(i32),
    Menu(MenuAction<u8, u8>),
    Attention(u64, ToastAttention),
    Dismiss(u64),
    Picked(&'static str),
    SheetOk,
    Own,
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

/// On the dim, clear of the dialog's card (centred), of the sheet (along the bottom) and of
/// the context menu (top left).
const ON_DIM: Point = Point::new(700.0, 100.0);
/// On the page's control, placed clear of every cover's own content.
const ON_BUTTON: Point = Point::new(110.0, 155.0);
/// Away from everything.
const AWAY: Point = Point::new(780.0, 30.0);

/// Where the page's controls are: left and low, so no cover's content is over them.
fn placed(content: Element<'static, Msg>) -> Element<'static, Msg> {
    iced::widget::container(content)
        .padding(iced::Padding {
            top: 140.0,
            left: 80.0,
            ..iced::Padding::ZERO
        })
        .into()
}

fn px_diff(a: &[u8], b: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

fn dialog() -> Dialog<Element<'static, Msg>, Msg> {
    Dialog::new(text("dialog").into())
}

/// What covers the page.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cover {
    None,
    /// A dialog, with `on_close_modals`.
    Dialog,
    /// A dialog with no close message.
    DialogNoSink,
    Sheet,
    /// A menu from the `context_menu` slot, with `on_close_menus`.
    ContextMenu,
    /// The application's own content in `header_menu`, with `on_close_menus`.
    HeaderMenu,
    /// `app_header`'s dropdown, open, with `on_close_menus` (RFC-124's cover).
    HeaderDropdown,
}

impl Cover {
    /// Every cover. `None` first: it is the premise.
    const ALL: [Self; 7] = [
        Self::None,
        Self::Dialog,
        Self::DialogNoSink,
        Self::Sheet,
        Self::ContextMenu,
        Self::HeaderMenu,
        Self::HeaderDropdown,
    ];

    /// Every cover except `None`.
    fn covers() -> impl Iterator<Item = Self> {
        Self::ALL.into_iter().filter(|c| *c != Self::None)
    }
}

fn menu(items: u8) -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".into(),
        icon: None,
        items: (0..items)
            .map(|i| MenuItem {
                menu_id: 0,
                id: i,
                label: "Open".into(),
                icon: None,
            })
            .collect(),
    }
}

/// `page` with `cover` over it.
fn scene(page: Element<'static, Msg>, cover: Cover) -> Element<'static, Msg> {
    let layout = AppLayout::new(page);
    let layout = match cover {
        Cover::None => layout,
        Cover::Dialog => layout.dialog(dialog()).on_close_modals(Msg::CloseModals),
        Cover::DialogNoSink => layout.dialog(dialog()),
        Cover::Sheet => layout
            .sheet(Sheet::new(text("sheet").into()))
            .on_close_modals(Msg::CloseModals),
        Cover::ContextMenu => layout
            .context_menu(snora::widget::render_menu(menu(1), &ON_MENU, true))
            .on_close_menus(Msg::CloseMenus),
        Cover::HeaderMenu => layout
            .header_menu(button(text("own menu content")).into())
            .on_close_menus(Msg::CloseMenus),
        Cover::HeaderDropdown => layout
            .header(snora::widget::app_header(
                "App",
                vec![menu(2)],
                &ON_MENU,
                Some(&0u8),
                None,
                snora::LayoutDirection::Ltr,
            ))
            .on_close_menus(Msg::CloseMenus),
    };
    snora::render(layout)
}

fn rig(page: Element<'static, Msg>, cover: Cover) -> Rig<Msg> {
    Rig::new(scene(page, cover))
}

fn page_button() -> Element<'static, Msg> {
    placed(button(text("page button")).on_press(Msg::Page).into())
}

fn page_tooltip() -> Element<'static, Msg> {
    placed(
        tooltip(
            text("hover me"),
            text("TOOLTIP BUBBLE"),
            tooltip::Position::Bottom,
        )
        .into(),
    )
}

fn page_without_tooltip() -> Element<'static, Msg> {
    placed(text("hover me").into())
}

fn page_scroller() -> Element<'static, Msg> {
    scrollable(column((0..80).map(|i| text(format!("row {i}")).into())))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn page_slider(value: i32) -> Element<'static, Msg> {
    placed(slider(0..=100, value, Msg::Slide).width(400).into())
}

// ---------------------------------------------------------------------------
// R-2: a wheel over a dim or a backdrop does nothing
// ---------------------------------------------------------------------------

/// Whether a page scroller scrolled when `times` wheel events were sent at `at`, the
/// messages published and the status of each.
fn wheel_over(cover: Cover, at: Point, times: usize) -> (bool, Vec<Msg>, Vec<Status>) {
    let mut r = rig(page_scroller(), cover);
    r.redraw();
    let before = r.frame();
    let statuses = r.wheel(at, times);
    r.redraw();
    (
        px_diff(&before, &r.frame()) > 0,
        r.messages.clone(),
        statuses,
    )
}

#[test]
fn a_wheel_over_the_dim_or_a_backdrop_does_nothing() {
    // The premise: with no cover, the same wheel scrolls the same page.
    let (scrolled, _, _) = wheel_over(Cover::None, ON_DIM, 3);
    assert!(scrolled, "premise: a wheel scrolls the uncovered page");

    for cover in Cover::covers() {
        for times in [1usize, 10] {
            let (scrolled, messages, statuses) = wheel_over(cover, ON_DIM, times);
            assert!(
                messages.is_empty(),
                "[{cover:?}, {times} wheel events] a wheel published {messages:?}: it must not close \
                 what is open, once per event or at all"
            );
            assert!(
                statuses.iter().all(|s| *s == Status::Captured),
                "[{cover:?}, {times}] a wheel over the dim or backdrop must be captured, got {statuses:?}"
            );
            assert!(
                !scrolled,
                "[{cover:?}, {times}] the page beneath scrolled under the cover"
            );
        }
    }
}

/// A dialog whose content is a tall scroller, and a page scroller beneath it.
fn dialog_with_a_card_that_scrolls() -> Element<'static, Msg> {
    let card = scrollable(column(
        (0..60).map(|i| text(format!("card row {i}")).into()),
    ))
    .width(300)
    .height(200);
    snora::render(
        AppLayout::new(page_scroller())
            .dialog(Dialog::new(card.into()))
            .on_close_modals(Msg::CloseModals),
    )
}

/// The constraint on R-2: the layer above the cover is offered the wheel first, so a wheel
/// over the dialog's own card scrolls the card; and when the card is at the end of its scroll
/// the wheel goes no further: it does not scroll the page, and it closes nothing.
#[test]
fn a_wheel_over_the_dialogs_card_scrolls_the_card_and_goes_no_further() {
    let card_centre = Point::new(400.0, 300.0);
    let mut r = Rig::new(dialog_with_a_card_that_scrolls());
    r.redraw();
    let before = r.frame();
    let statuses = r.wheel(card_centre, 1);
    r.redraw();
    assert!(
        px_diff(&before, &r.frame()) > 0,
        "premise: a wheel over the card scrolls the card"
    );
    assert_eq!(statuses, vec![Status::Captured]);
    assert!(
        r.messages.is_empty(),
        "scrolling the card published {:?}",
        r.messages
    );

    // To the end of the card's scroll, then more.
    r.wheel(card_centre, 80);
    r.redraw();
    let at_the_end = r.frame();
    let statuses = r.wheel(card_centre, 10);
    r.redraw();
    assert_eq!(
        px_diff(&at_the_end, &r.frame()),
        0,
        "past the end of the card's scroll, something moved: the wheel chained to the page"
    );
    assert!(
        statuses.iter().all(|s| *s == Status::Captured),
        "past the end of the card's scroll the wheel must still be captured, got {statuses:?}"
    );
    assert!(
        r.messages.is_empty(),
        "the wheel past the card's end published {:?}",
        r.messages
    );
}

// ---------------------------------------------------------------------------
// R-1: no hover, no pointer shape, no tooltip
// ---------------------------------------------------------------------------

#[test]
fn a_page_under_a_cover_shows_no_hover() {
    let hover = |cover: Cover| {
        let frame = |at: Point| {
            let mut r = rig(page_button(), cover);
            r.point_at(Some(at));
            r.redraw();
            r.frame()
        };
        px_diff(&frame(AWAY), &frame(ON_BUTTON))
    };
    assert!(
        hover(Cover::None) > 0,
        "premise: an uncovered page button shows hover"
    );
    for cover in Cover::covers() {
        assert_eq!(
            hover(cover),
            0,
            "[{cover:?}] the page shows hover under a cover: a tint that promises a click the cover will not let through"
        );
    }
}

#[test]
fn the_pointer_shape_over_a_covered_page_control_is_not_the_hand() {
    let shape = |cover: Cover| {
        let mut r = rig(page_button(), cover);
        r.redraw();
        r.interaction_at(ON_BUTTON)
    };
    assert_eq!(
        shape(Cover::None),
        mouse::Interaction::Pointer,
        "premise: the uncovered page control shows the hand"
    );
    for cover in Cover::covers() {
        assert_ne!(
            shape(cover),
            mouse::Interaction::Pointer,
            "[{cover:?}] the hand cursor shows over a control the cover will not let be pressed"
        );
    }
}

#[test]
fn a_page_tooltip_under_a_cover_is_not_drawn() {
    let bubble = |cover: Cover| {
        let frame = |page: Element<'static, Msg>| {
            let mut r = rig(page, cover);
            r.point_at(Some(Point::new(100.0, 148.0)));
            r.redraw();
            r.frame()
        };
        px_diff(&frame(page_tooltip()), &frame(page_without_tooltip()))
    };
    assert!(
        bubble(Cover::None) > 0,
        "premise: an uncovered page tooltip is drawn"
    );
    for cover in Cover::covers() {
        assert_eq!(
            bubble(cover),
            0,
            "[{cover:?}] a page tooltip is drawn under a cover"
        );
    }
}

/// An overlay that is **not** driven by the pointer: a `pick_list`'s open list. The cursor
/// rule cannot hide it, so this is what pins that a covered layer's overlays are hidden (a
/// tooltip is gone by the cursor rule alone).
#[test]
fn a_page_pick_list_left_open_is_not_drawn_above_a_dialogs_dim() {
    fn page() -> Element<'static, Msg> {
        placed(pick_list(vec!["alpha", "beta"], None::<&'static str>, Msg::Picked).into())
    }
    // Open the list on the uncovered page: a click on the pick_list.
    let mut r = Rig::new(scene(page(), Cover::None));
    r.redraw();
    let closed = r.frame();
    r.click_at(Point::new(110.0, 155.0));
    r.redraw();
    assert!(
        px_diff(&closed[180 * 800 * 4..], &r.frame()[180 * 800 * 4..]) > 0,
        "premise: clicking the pick_list opens its list, below the field"
    );
    // The application opens a dialog; the list is still open in the widget's state.
    let mut covered = r.rebuilt(scene(page(), Cover::Dialog));
    covered.point_at(Some(AWAY));
    covered.redraw();
    let mut never_opened = Rig::new(scene(page(), Cover::Dialog));
    never_opened.point_at(Some(AWAY));
    never_opened.redraw();
    // The field itself may be drawn in its "opened" style; the list is below it. Compare the
    // rows below the field (it ends well above y = 180), where only the list can differ.
    let below = |frame: &[u8]| frame[180 * 800 * 4..].to_vec();
    assert_eq!(
        px_diff(&below(&covered.frame()), &below(&never_opened.frame())),
        0,
        "the page's open pick_list is drawn above the dim"
    );
}

// ---------------------------------------------------------------------------
// R-4: nothing beneath acts on the pointer once it is covered
// ---------------------------------------------------------------------------

#[test]
fn a_pointer_resting_on_a_page_control_when_a_dialog_opens_shows_no_hover_on_the_first_frame() {
    let mut r = Rig::new(scene(page_button(), Cover::None));
    r.point_at(Some(ON_BUTTON));
    r.redraw();
    let hovered = r.frame();
    let mut plain = Rig::new(scene(page_button(), Cover::None));
    plain.point_at(Some(AWAY));
    plain.redraw();
    assert!(
        px_diff(&hovered, &plain.frame()) > 0,
        "premise: the resting pointer shows hover before the dialog"
    );
    // The application opens the dialog with the pointer still on the control.
    let mut under = r.rebuilt(scene(page_button(), Cover::Dialog));
    let first_frame = under.frame();
    let mut never_hovered = Rig::new(scene(page_button(), Cover::Dialog));
    never_hovered.point_at(Some(AWAY));
    never_hovered.redraw();
    assert_eq!(
        px_diff(&first_frame, &never_hovered.frame()),
        0,
        "the first frame under the dim still shows the hover the control had"
    );
}

#[test]
fn a_drag_that_began_before_the_cover_does_not_move_anything_while_covered() {
    let slides = |r: &Rig<Msg>| {
        r.messages
            .iter()
            .filter(|m| matches!(m, Msg::Slide(_)))
            .count()
    };
    for cover in [Cover::Dialog, Cover::ContextMenu] {
        let mut r = rig(page_slider(0), Cover::None);
        r.redraw();
        // Press on the slider, then drag a little: the drag is under way.
        r.point_at(Some(Point::new(100.0, 152.0)));
        r.left_button(true);
        r.point_at(Some(Point::new(190.0, 152.0)));
        assert!(
            slides(&r) > 0,
            "premise: dragging the uncovered slider publishes Slide"
        );
        // The application opens the cover, keeping the widget state; the pointer moves on.
        let mut r = r.rebuilt(scene(page_slider(10), cover));
        r.messages.clear();
        r.point_at(Some(Point::new(300.0, 152.0)));
        r.point_at(Some(Point::new(380.0, 152.0)));
        assert_eq!(
            slides(&r),
            0,
            "[{cover:?}] the slider moved while covered: {:?}",
            r.messages
        );
        // The release ends the drag: no message follows it, and none once the cover is gone.
        r.left_button(false);
        r.point_at(Some(Point::new(330.0, 152.0)));
        assert_eq!(slides(&r), 0, "[{cover:?}] a message followed the release");
        let mut r = r.rebuilt(scene(page_slider(10), Cover::None));
        r.messages.clear();
        r.point_at(Some(Point::new(250.0, 152.0)));
        r.point_at(Some(Point::new(260.0, 152.0)));
        assert_eq!(
            slides(&r),
            0,
            "[{cover:?}] the drag was still under way once the cover was gone"
        );
    }
}

// ---------------------------------------------------------------------------
// R-3: what is unchanged
// ---------------------------------------------------------------------------

#[test]
fn a_press_on_the_dim_or_a_backdrop_publishes_its_close_message_once() {
    for (cover, expected) in [
        (Cover::Dialog, Msg::CloseModals),
        (Cover::Sheet, Msg::CloseModals),
        (Cover::ContextMenu, Msg::CloseMenus),
        (Cover::HeaderMenu, Msg::CloseMenus),
        // An open header dropdown: the cover itself publishes (RFC-124).
        (Cover::HeaderDropdown, Msg::CloseMenus),
    ] {
        let mut r = rig(page_scroller(), cover);
        r.redraw();
        r.click_at(ON_DIM);
        assert_eq!(
            r.messages,
            vec![expected.clone()],
            "[{cover:?}] a press must publish {expected:?} once"
        );
    }
    // A dim with no close message blocks the pointer and publishes nothing.
    let mut r = rig(page_scroller(), Cover::DialogNoSink);
    r.redraw();
    r.click_at(ON_DIM);
    assert!(r.messages.is_empty());
}

/// Toasts are never covered (Law 6): under a dialog the pointer reaches them and their ×
/// can be pressed.
#[test]
fn a_toast_under_no_cover_gets_the_pointer_and_its_close_control_is_pressable() {
    fn view(cover: Cover) -> Element<'static, Msg> {
        let toast = Toast::new(7, ToastIntent::Info, "Saved", "All good.", Msg::Dismiss(7));
        let base = AppLayout::new(page_scroller())
            .toasts(vec![toast])
            .on_toast_attention(Msg::Attention);
        snora::render(match cover {
            Cover::Dialog => base.dialog(dialog()).on_close_modals(Msg::CloseModals),
            _ => base,
        })
    }
    for cover in [Cover::None, Cover::Dialog] {
        let mut r = Rig::new(view(cover));
        r.redraw();
        let close = r
            .targets()
            .last()
            .expect("the toast's × is a focus target")
            .bounds
            .center();
        r.point_at(Some(close));
        r.redraw();
        assert!(
            r.messages
                .contains(&Msg::Attention(7, ToastAttention::PointerEntered)),
            "[{cover:?}] the pointer did not reach the toast: {:?}",
            r.messages
        );
        r.messages.clear();
        r.click_at(close);
        assert_eq!(
            r.messages,
            vec![Msg::Dismiss(7)],
            "[{cover:?}] the toast's × did not dismiss it"
        );
    }
}

/// A widget that counts the redraw requests it is sent.
struct Heartbeat(Arc<AtomicUsize>);

impl Widget<Msg, Theme, iced::Renderer> for Heartbeat {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(10.0), Length::Fixed(10.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(10.0, 10.0))
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &iced::Renderer,
        _operation: &mut dyn Operation,
    ) {
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Msg>,
        _viewport: &Rectangle,
    ) {
        if matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}

/// Window events and redraws reach a covered layer, so an animation beneath keeps running.
#[test]
fn an_animation_beneath_a_cover_keeps_running() {
    for cover in Cover::ALL {
        let beats = Arc::new(AtomicUsize::new(0));
        let page: Element<'static, Msg> = Element::new(Heartbeat(beats.clone()));
        let mut r = Rig::new(scene(page, cover));
        let before = beats.load(Ordering::SeqCst);
        r.redraw();
        r.redraw();
        assert!(
            beats.load(Ordering::SeqCst) >= before + 2,
            "[{cover:?}] a redraw did not reach the layer beneath the cover"
        );
    }
}

// ---------------------------------------------------------------------------
// The 0.62 audit, Unit 3: touch, two menus together, and a sheet with no sink
// ---------------------------------------------------------------------------

/// A finger on the screen at `at`: it goes down and comes up. The cursor is set as well; **whether
/// a real window's touch moves the cursor is `iced_winit`'s**, and this cannot say (the audit's
/// caveat, L-024).
fn tap(r: &mut Rig<Msg>, at: Point) {
    use iced_runtime::core::touch;
    r.set_cursor(at);
    let id = touch::Finger(0);
    r.batch(&[Event::Touch(touch::Event::FingerPressed {
        id,
        position: at,
    })]);
    r.batch(&[Event::Touch(touch::Event::FingerLifted {
        id,
        position: at,
    })]);
}

/// **L-024, O-009: a tap closes what a click closes**, once: on a dialog's dim, a sheet's, a context
/// menu's backdrop, a header menu's, and an open dropdown's cover. A dim with no close message
/// publishes nothing.
#[test]
fn a_tap_on_the_dim_or_a_backdrop_publishes_its_close_message_once() {
    for (cover, expected) in [
        (Cover::Dialog, Msg::CloseModals),
        (Cover::Sheet, Msg::CloseModals),
        (Cover::ContextMenu, Msg::CloseMenus),
        (Cover::HeaderMenu, Msg::CloseMenus),
        (Cover::HeaderDropdown, Msg::CloseMenus),
    ] {
        let mut r = rig(page_scroller(), cover);
        r.redraw();
        tap(&mut r, ON_DIM);
        assert_eq!(
            r.messages,
            vec![expected.clone()],
            "[{cover:?}] a tap must publish {expected:?} once"
        );
    }
    let mut r = rig(page_scroller(), Cover::DialogNoSink);
    r.redraw();
    tap(&mut r, ON_DIM);
    assert!(r.messages.is_empty(), "{:?}", r.messages);
}

/// **L-066: a `header_menu` and a `context_menu` together.** Both menus' controls are in the tree
/// (the page's is not: they cover it), each press publishes its own message, and a press outside
/// both closes once.
#[test]
fn a_header_menu_and_a_context_menu_together_are_both_live_and_close_once() {
    let view = || {
        snora::render(
            AppLayout::new(page_button())
                .header_menu(snora::control(text("own")).on_press(Msg::Own).into())
                // The menus draw where the application puts them; unpositioned, both sit at the
                // top-left corner and one hides the other, so this one is placed.
                .context_menu(placed(snora::widget::render_menu(menu(2), &ON_MENU, true)))
                .on_close_menus(Msg::CloseMenus),
        )
    };
    let mut r = Rig::new(view());
    let n = r.targets().len();
    // The header's control, and the context menu's trigger and two items; the page's button is
    // under the backdrop.
    assert_eq!(n, 4, "both menus' controls are in the tree");
    let mut published = Vec::new();
    for k in 0..n {
        let mut r = Rig::new(view());
        let at = r.targets()[k].bounds.center();
        r.click_at(at);
        published.push(r.messages.clone());
    }
    assert!(
        published.contains(&vec![Msg::Own]),
        "the header menu's own content answers a press: {published:?}"
    );
    assert!(
        published
            .iter()
            .filter(|m| matches!(m.as_slice(), [Msg::Menu(_)]))
            .count()
            >= 2,
        "the context menu's items answer a press: {published:?}"
    );
    let mut r = Rig::new(view());
    r.click_at(AWAY);
    assert_eq!(r.messages, vec![Msg::CloseMenus], "outside both: once");
}

/// **L-013: a sheet with no close message is content that is never dropped.** Its one control is the
/// only target (the page is covered, Tab included), a corner click and a click at the page's own
/// control publish nothing, and a wheel is captured.
#[test]
fn a_sheet_with_no_close_message_blocks_the_pointer_the_wheel_and_tab() {
    let view = || {
        snora::render(AppLayout::new(page_button()).sheet(Sheet::new(
            snora::control(text("sheet")).on_press(Msg::SheetOk).into(),
        )))
    };
    let mut r = Rig::new(view());
    assert_eq!(
        r.targets().len(),
        1,
        "only the sheet's control is reachable"
    );
    r.click_at(AWAY);
    r.click_at(ON_BUTTON);
    assert!(
        r.messages.is_empty(),
        "a click through the sheet's dim published {:?}",
        r.messages
    );
    let statuses = r.wheel(Point::new(400.0, 300.0), 2);
    assert!(
        statuses.iter().all(|s| *s == Status::Captured),
        "a wheel over the sheet went through: {statuses:?}"
    );
    // Tab, through the router's operation, stays on the sheet's one control.
    for _ in 0..4 {
        r.tab(false);
        r.enter();
    }
    assert!(!r.messages.contains(&Msg::Page), "{:?}", r.messages);
    assert!(
        r.messages.contains(&Msg::SheetOk),
        "the premise: Tab and Enter reach the sheet"
    );
}
