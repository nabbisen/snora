//! Render-semantics tests for the Snora engine.
//!
//! These tests verify the runtime behavioral contract of [`snora::render`]:
//! z-stack ordering, backdrop dismissal, modal pointer-blocking, and toast
//! visibility. They use [`iced_test::Simulator`] — a CPU-only headless
//! renderer that runs without a display — so they execute in CI the same way
//! they do locally.
//!
//! # Scope
//!
//! These tests cover the *engine's* behavior. Applications should test their
//! own `update` state-machine logic separately (see the testing guide). Snora
//! deliberately does not ship a public `snora-test` crate; all helpers here
//! are private to this integration-test target.
//!
//! # Harness
//!
//! Every test builds an `AppLayout`, passes it through `snora::render`, feeds
//! the resulting `iced::Element` into a fresh `Simulator`, performs
//! interactions (clicks, point-and-press), and asserts on the `Message`
//! values produced.
//!
//! `Simulator::click(selector)` selects a widget by the text it contains and
//! fires a pointer-press + release at its center.
//!
//! `point_at(Point) + simulate(click())` fires at an arbitrary coordinate,
//! allowing tests to hit the backdrop at a corner where no overlay content
//! sits.

use iced::widget::{button, center, text};
use iced::{Element, Point};
use iced_test::simulator;

use snora::{AppLayout, Dialog, Sheet, SheetEdge, Toast, ToastIntent, render};

// ---------------------------------------------------------------------------
// Shared message type for all render-semantics tests.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // some variants are reserved for v0.12 expansion
enum Msg {
    BodyPressed,
    CloseMenus,
    CloseModals,
    DialogOk,
    SheetAction,
    DismissToast(u64),
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn btn<'a>(label: &'static str, msg: Msg) -> Element<'a, Msg> {
    button(text(label)).on_press(msg).into()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Body button is reachable when no overlays are present.
///
/// Verifies: skeleton (layer 0) receives clicks.
#[test]
fn body_button_reachable_without_overlays() {
    let layout = AppLayout::new(btn("body", Msg::BodyPressed));
    let element = render(layout);

    let mut ui = simulator(element);
    ui.click("body").expect("body button should be findable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert_eq!(msgs, vec![Msg::BodyPressed]);
}

/// Outside click on a modal dim backdrop emits `on_close_modals`.
///
/// Verifies: layer 4 (modal backdrop with click sink) is installed when a
/// dialog is present and `on_close_modals` is wired.
#[test]
fn outside_click_on_modal_emits_close_modals() {
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .dialog(dialog)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    // Click a corner far from the centered dialog content.
    ui.point_at(Point::new(4.0, 4.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::CloseModals),
        "corner click should produce CloseModals; got {msgs:?}",
    );
}

/// Dialog content button is reachable while a modal is open.
///
/// Verifies: layer 5 (dialog) is rendered above the dim; interactive
/// content inside the dialog fires its own message.
#[test]
fn dialog_content_button_reachable() {
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(center(btn("OK", Msg::DialogOk)).into());
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .dialog(dialog)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.click("OK").expect("dialog OK button should be findable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert_eq!(
        msgs,
        vec![Msg::DialogOk],
        "clicking dialog content should produce DialogOk only",
    );
}

/// When `on_close_modals` is `None`, outside click produces no dismiss message.
///
/// Verifies Law 5 (RFC-011-E): missing close sink omits the backdrop click
/// capture but still renders the content. The dialog remains visible (its
/// button is still findable).
#[test]
fn no_close_sink_means_no_dismiss_but_content_renders() {
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .dialog(dialog);
    // on_close_modals intentionally absent.
    let element = render(layout);

    let mut ui = simulator(element);

    // Corner click should not produce CloseModals.
    ui.point_at(Point::new(4.0, 4.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let msgs_after_corner: Vec<Msg> = ui.into_messages().collect();
    assert!(
        !msgs_after_corner.contains(&Msg::CloseModals),
        "no close sink → corner click must not produce CloseModals; got {msgs_after_corner:?}",
    );

    // Dialog content still renders.
    let rebuild: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout2 = AppLayout::new(btn("body", Msg::BodyPressed)).dialog(rebuild);
    let element2 = render(layout2);
    let mut ui2 = simulator(element2);
    ui2.find("OK").expect("dialog content should still be renderable with no close sink");
}

/// Toast dismiss button fires its message even while a modal is open.
///
/// Verifies Law 6 (RFC-011-E): toasts (layer 7) render above modal state
/// (layers 4–6) and remain interactive during a modal workflow.
#[test]
fn toast_dismiss_reachable_above_modal() {
    let toast = Toast::new(
        7,
        ToastIntent::Info,
        "Saved",
        "All good.",
        Msg::DismissToast(7),
    );
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .dialog(dialog)
        .on_close_modals(Msg::CloseModals)
        .toasts(vec![toast]);
    let element = render(layout);

    let mut ui = simulator(element);
    // The toast close button renders the glyph "×".
    ui.click("×").expect("toast close button (×) should be findable above the modal");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::DismissToast(7)),
        "toast dismiss should fire even while a modal is present; got {msgs:?}",
    );
}

/// Sheet content is reachable (sheet body uses `opaque`, preventing click
/// fall-through to the modal dim).
///
/// This also implicitly verifies z-order: sheet (layer 6) renders above
/// the modal dim (layer 4), and the `opaque` wrapper captures clicks.
#[test]
fn sheet_content_button_reachable() {
    let sheet: Sheet<Element<Msg>, Msg> =
        Sheet::new(btn("Sheet action", Msg::SheetAction)).at(SheetEdge::Bottom);
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .sheet(sheet)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.click("Sheet action").expect("sheet action button should be findable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert_eq!(
        msgs,
        vec![Msg::SheetAction],
        "clicking sheet content should produce SheetAction only",
    );
}
