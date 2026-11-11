//! RFC-125 Unit 0: what a covered layer does with the pointer, measured on `main` before
//! anything changes. Prints the tables (`--nocapture`); asserts only each row's premise, so a
//! reading of "nothing happened" cannot come from a scene in which nothing could.
//!
//! Hover and tooltips are read from **pixels**, after the `RedrawRequested` the runtime sends
//! before a draw (iced 0.14 widgets take their status there, not on the pointer move).
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use common::removal::Rig;
use iced::widget::{button, column, scrollable, slider, text, tooltip};
use iced::{Element, Point};
use iced_runtime::core::event::Status;
use snora::{AppLayout, Dialog, Menu, MenuAction, MenuItem, Sheet};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    CloseModals,
    CloseMenus,
    Page,
    Slide(i32),
    Menu(MenuAction<u8, u8>),
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

/// On the dim, clear of the dialog's card (centred), of the sheet (along the bottom) and of
/// the context menu (top left).
const ON_DIM: Point = Point::new(700.0, 100.0);
/// On the page's control, placed clear of every cover's own content.
const ON_BUTTON: Point = Point::new(110.0, 155.0);
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
/// Away from everything.
const AWAY: Point = Point::new(780.0, 30.0);

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
}

impl Cover {
    const ALL: [Self; 6] = [
        Self::None,
        Self::Dialog,
        Self::DialogNoSink,
        Self::Sheet,
        Self::ContextMenu,
        Self::HeaderMenu,
    ];
}

fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".into(),
        icon: None,
        items: vec![MenuItem {
            menu_id: 0,
            id: 0,
            label: "Open".into(),
            icon: None,
        }],
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
            .context_menu(snora::widget::render_menu(menu(), &ON_MENU, true))
            .on_close_menus(Msg::CloseMenus),
        Cover::HeaderMenu => layout
            .header_menu(button(text("own menu content")).into())
            .on_close_menus(Msg::CloseMenus),
    };
    snora::render(layout)
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
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

fn page_slider(value: i32) -> Element<'static, Msg> {
    placed(slider(0..=100, value, Msg::Slide).width(400).into())
}

fn status_name(s: Status) -> &'static str {
    match s {
        Status::Ignored => "Ignored",
        Status::Captured => "Captured",
    }
}

fn rig(page: Element<'static, Msg>, cover: Cover) -> Rig<Msg> {
    Rig::new(scene(page, cover))
}

// ---------------------------------------------------------------------------

#[test]
fn the_wheel_on_the_dim() {
    println!("\n## a wheel on the dim ({ON_DIM:?}), the pointer resting there\n");
    println!("| cover | events | messages published | statuses | page scrolled? |");
    println!("|---|---|---|---|---|");
    for cover in [
        Cover::Dialog,
        Cover::DialogNoSink,
        Cover::Sheet,
        Cover::ContextMenu,
        Cover::HeaderMenu,
    ] {
        for times in [1usize, 10] {
            let mut r = rig(page_scroller(), cover);
            r.redraw();
            let before = r.frame();
            let statuses = r.wheel(ON_DIM, times);
            r.redraw();
            let scrolled = px_diff(&before, &r.frame()) > 0;
            let closes = r
                .messages
                .iter()
                .filter(|m| matches!(m, Msg::CloseModals | Msg::CloseMenus))
                .count();
            let names: Vec<_> = statuses.iter().map(|s| status_name(*s)).collect();
            let summary = if names.windows(2).all(|w| w[0] == w[1]) {
                format!("{} x {}", names.len(), names[0])
            } else {
                names.join(",")
            };
            println!(
                "| {cover:?} | {times} | {closes} close message(s) ({:?}) | {summary} | {scrolled} |",
                r.messages.first()
            );
        }
    }
    // The premise: with no cover, the same wheel scrolls the same page.
    let mut r = rig(page_scroller(), Cover::None);
    r.redraw();
    let before = r.frame();
    r.wheel(ON_DIM, 3);
    r.redraw();
    assert!(
        px_diff(&before, &r.frame()) > 0,
        "premise: a wheel scrolls the uncovered page"
    );
}

#[test]
fn hover_shape_and_tooltip_under_each_cover() {
    println!("\n## the page under each cover: hover (px), pointer shape, tooltip (px)\n");
    println!(
        "| cover | hover on a page button (px differ) | pointer shape over it | tooltip bubble drawn (px) |"
    );
    println!("|---|---|---|---|");
    for cover in Cover::ALL {
        // Hover: pointer on the button vs away, after a redraw.
        let frame = |at: Point| {
            let mut r = rig(page_button(), cover);
            r.point_at(Some(at));
            r.redraw();
            r.frame()
        };
        let hover = px_diff(&frame(AWAY), &frame(ON_BUTTON));
        // Shape.
        let mut r = rig(page_button(), cover);
        r.redraw();
        let shape = r.interaction_at(ON_BUTTON);
        // Tooltip: the page with a tooltip against the same page without, pointer on the target.
        let tip = |page: Element<'static, Msg>| {
            let mut r = rig(page, cover);
            r.point_at(Some(Point::new(100.0, 148.0)));
            r.redraw();
            r.frame()
        };
        let bubble = px_diff(&tip(page_tooltip()), &tip(page_without_tooltip()));
        println!("| {cover:?} | {hover} | {shape:?} | {bubble} |");
        if cover == Cover::None {
            assert!(hover > 0, "premise: an uncovered page button shows hover");
            assert!(bubble > 0, "premise: an uncovered page tooltip is drawn");
        }
    }
}

/// An iced `slider` dragged on the page, a cover opened by the application mid-drag, the
/// pointer moved and released; then whether the drag is over.
#[test]
fn a_drag_that_began_before_the_cover() {
    println!("\n## a slider dragged, a cover appears mid-drag, the pointer moves, then releases\n");
    println!(
        "| cover | slides before the cover | slides while covered | slides after the release, still covered | slides after the cover is gone (pointer moves, button up) |"
    );
    println!("|---|---|---|---|---|");
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
        let before = slides(&r);
        assert!(before > 0, "premise: dragging the slider publishes Slide");
        // The application opens the cover, keeping the widget state.
        let mut r = r.rebuilt(scene(page_slider(10), cover));
        r.messages.clear();
        r.point_at(Some(Point::new(300.0, 152.0)));
        r.point_at(Some(Point::new(380.0, 152.0)));
        let covered = slides(&r);
        r.messages.clear();
        r.left_button(false);
        r.point_at(Some(Point::new(330.0, 152.0)));
        let after_release = slides(&r);
        // The application closes the cover; the button is up and the pointer moves.
        let mut r = r.rebuilt(scene(page_slider(10), Cover::None));
        r.messages.clear();
        r.point_at(Some(Point::new(250.0, 152.0)));
        r.point_at(Some(Point::new(260.0, 152.0)));
        let gone = slides(&r);
        println!("| {cover:?} | {before} | {covered} | {after_release} | {gone} |");
    }
}

/// R-3's baseline: a press on the dim or the backdrop, and Escape.
#[test]
fn a_press_on_the_dim_today() {
    println!("\n## a click on the dim / backdrop ({ON_DIM:?}), and Escape\n");
    println!("| cover | click publishes | Escape publishes (status) |");
    println!("|---|---|---|");
    for cover in [
        Cover::Dialog,
        Cover::DialogNoSink,
        Cover::Sheet,
        Cover::ContextMenu,
        Cover::HeaderMenu,
    ] {
        let mut r = rig(page_scroller(), cover);
        r.redraw();
        r.click_at(ON_DIM);
        let click = r.messages.clone();
        let mut r = rig(page_scroller(), cover);
        r.redraw();
        let status = r.press(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        );
        println!(
            "| {cover:?} | {click:?} | {:?} ({}) |",
            r.messages,
            status_name(status)
        );
    }
}
