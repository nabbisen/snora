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
//!
//! # Negative pointer-containment assertions (RFC-084)
//!
//! Every containment test before RFC-084 was **positive**: a button inside
//! an overlay is reachable, a corner click dismisses. None asked whether
//! pointer input that should be *blocked* actually is — which is exactly
//! the shape of bug this crate had four instances of at once.
//!
//! The negative assertions added here are derived from [Law
//! 8](https://docs.snora.dev/reference/overlay-interaction-semantics.html#law-8--modal-focus-trapping-is-staged-not-shipped)'s
//! own table, specifically its unconditional *"Pointer blocking (backdrop
//! capture) — yes"* row, rather than invented independently — so the suite
//! and the law describe the same claim and cannot drift apart. At minimum,
//! that row implies:
//!
//! * a click inside the dialog does not dismiss it
//!   (`dialog_click_does_not_dismiss_modal`);
//! * a click on a toast's own body does not reach content beneath it
//!   (`toast_body_click_does_not_reach_content_beneath`);
//! * a click on the modal dim, with no close sink, does not reach content
//!   beneath it (`modal_with_no_close_sink_still_blocks_pointer_at_dim`) —
//!   Law 5 says a missing sink omits the *dismiss message*, not the
//!   *containment*, and before RFC-084 it wrongly omitted both.
//!
//! Two more were added while measuring, not because the minimum list named
//! them: wheel-scroll is a second pointer-input kind the same "yes" row
//! covers, tested separately for both the with-sink and no-sink dims
//! (`modal_dim_with_close_sink_blocks_wheel_scroll`,
//! `modal_with_no_close_sink_also_blocks_wheel_scroll`) since scroll and
//! click are captured by different mechanisms in this codebase (see their
//! doc comments) and neither should be assumed from the other.
//!
//! **What Law 8 does not imply, and is deliberately not tested here:**
//! its other two rows — keyboard dismissal and focus trapping/zone
//! navigation — are a different axis (keyboard focus, not pointer
//! containment) that RFC-084 explicitly does not touch; they are Law 7's
//! and RFC-060's concerns respectively, already covered where those are
//! tested.

use iced::widget::{button, center, container, mouse_area, space, text};
use iced::{Element, Length, Point};
use iced_test::simulator;

use snora::{AppLayout, Dialog, Sheet, SheetEdge, Toast, ToastIntent, render};

// ---------------------------------------------------------------------------
// Shared message type for all render-semantics tests.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
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

/// A click inside the dialog — on its own padding or plain text, not on any
/// interactive content — must not dismiss the modal (RFC-084 F-01).
///
/// Before the fix: `render_dialog` wraps content in `center(...)` with no
/// `opaque`, so a click that the content itself does not capture falls
/// through the dialog layer to the modal backdrop beneath it and fires
/// `on_close_modals` — a click on the dialog's own body dismisses it.
///
/// Verifies Law 8 ("pointer blocking — yes"): the dialog is part of what
/// pointer input must not fall through, not just the backdrop.
#[test]
fn dialog_click_does_not_dismiss_modal() {
    let content: Element<Msg> = container(text("Are you sure…")).padding(80).into();
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(content);
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .dialog(dialog)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    // Dead centre of the default 1024x768 window — inside the dialog's
    // padded content, not on any interactive widget.
    ui.point_at(Point::new(512.0, 384.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        !msgs.contains(&Msg::CloseModals),
        "a click inside the dialog's own content must not dismiss it; got {msgs:?}",
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
    let layout = AppLayout::new(btn("body", Msg::BodyPressed)).dialog(dialog);
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
    ui2.find("OK")
        .expect("dialog content should still be renderable with no close sink");
}

/// A modal with no `on_close_modals` sink must still block pointer input
/// from reaching content beneath it (RFC-084 F-02 / Law 8).
///
/// Before the fix: `dim_without_capture` is a plain `container(space())` —
/// it neither captures clicks nor reports a `mouse_interaction`, so a click
/// over the dim reaches whatever is beneath it. This contradicts Law 8's
/// unconditional "Pointer blocking — yes": blocking must not depend on
/// whether a dismiss message was provided.
///
/// The body is a full-window `mouse_area` (rather than a small `button`) so
/// a click at any point within the window tests capture, independent of
/// where a button widget would happen to lay itself out.
#[test]
fn modal_with_no_close_sink_still_blocks_pointer_at_dim() {
    let body: Element<Msg> = mouse_area(space().width(Length::Fill).height(Length::Fill))
        .on_press(Msg::BodyPressed)
        .into();
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout = AppLayout::new(body).dialog(dialog);
    // on_close_modals intentionally absent.
    let element = render(layout);

    let mut ui = simulator(element);
    // Corner, away from the centered dialog content — over the dim only.
    ui.point_at(Point::new(4.0, 4.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        !msgs.contains(&Msg::BodyPressed),
        "a modal dim with no close sink must still block pointer input \
         from reaching content beneath it; got {msgs:?}",
    );
}

/// Measures whether the modal dim blocks wheel-scroll input the same way
/// it blocks clicks (RFC-084 F-03 / Q-2). Same cause as F-02, different
/// input: measured, not assumed, and the first reading of the source was
/// wrong. `opaque`'s own `update` only captures
/// `mouse::Event::ButtonPressed` (checked via `is_mouse_press`) — reading
/// only that method suggests it would not stop `WheelScrolled`. But
/// `iced_widget::Stack::update` dispatches top-down and, after each layer,
/// checks that layer's `mouse_interaction()`: if it is not `None` while the
/// cursor is over it, the cursor **levitates** for every layer beneath —
/// which makes `cursor.is_over(...)` false for them, for *any* event, not
/// only the one that triggered the check. `opaque`'s `mouse_interaction`
/// unconditionally returns `Idle` (never `None`) whenever hovered, so it
/// triggers this regardless of event type. The empirical test below (run
/// against the F-01/F-02 fix, before any scroll-specific change) confirmed
/// this: it failed as "known gap" on first write and had to be corrected —
/// see its own doc comment.
///
/// **With a close sink**, `dim_backdrop`'s `mouse_area` is additionally
/// given an explicit `on_scroll` handler using the same message it already
/// has for `on_press` (`Message: Clone` is already required there) —
/// `mouse_area` does *not* get the same unconditional `mouse_interaction`
/// treatment `opaque` does (its own `mouse_interaction` only overrides when
/// `.interaction(...)` was explicitly set, which this call site does not
/// do), so it needs the explicit handler rather than inheriting capture
/// from Stack's dispatch.
#[test]
fn modal_dim_with_close_sink_blocks_wheel_scroll() {
    let body: Element<Msg> = mouse_area(space().width(Length::Fill).height(Length::Fill))
        .on_scroll(|_delta| Msg::BodyPressed)
        .into();
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout = AppLayout::new(body)
        .dialog(dialog)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.point_at(Point::new(4.0, 4.0));
    let _ = ui.simulate([iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
        delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
    })]);
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        !msgs.contains(&Msg::BodyPressed),
        "modal dim with a close sink must also block wheel scroll from \
         reaching content beneath it; got {msgs:?}",
    );
}

/// A modal with **no** close sink also blocks wheel-scroll input, the same
/// as it blocks clicks (RFC-084 F-03 / Q-2).
///
/// **This is the corrected version of a wrong first assumption**, kept
/// visible rather than quietly rewritten: reading only `opaque`'s own
/// `update` method (it checks for `ButtonPressed` and nothing else)
/// suggested scroll would fall through here uncaught, the same as the
/// `mouse_area`-without-`on_scroll` case. Written first as a "known,
/// deferred gap" test asserting exactly that. **It failed on the very
/// first run** — the actual result was `[]`, not `[BodyPressed]` — which is
/// what measuring instead of assuming is for. The real mechanism is
/// `iced_widget::Stack`'s own dispatch: it consults each layer's
/// `mouse_interaction()` after every event, not just clicks, and `opaque`
/// reports non-`None` interaction unconditionally whenever hovered — see
/// the sibling test's doc comment for the full mechanism. `opaque`'s
/// click-only-looking `update` method was never the whole story.
#[test]
fn modal_with_no_close_sink_also_blocks_wheel_scroll() {
    let body: Element<Msg> = mouse_area(space().width(Length::Fill).height(Length::Fill))
        .on_scroll(|_delta| Msg::BodyPressed)
        .into();
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("OK", Msg::DialogOk));
    let layout = AppLayout::new(body).dialog(dialog);
    // on_close_modals intentionally absent.
    let element = render(layout);

    let mut ui = simulator(element);
    ui.point_at(Point::new(4.0, 4.0));
    let _ = ui.simulate([iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
        delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
    })]);
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        !msgs.contains(&Msg::BodyPressed),
        "a modal dim with no close sink must also block wheel scroll from \
         reaching content beneath it; got {msgs:?}",
    );
}

/// A click on a toast's own body — its title or message text, not the `×`
/// button — must not reach content beneath it (RFC-084 F-04 / Law 8).
///
/// Before the fix: `render_single_toast`'s outer `container(body)` has no
/// capture; only the `×` button captures, because it is a button. A click
/// anywhere else on the toast surface falls through to whatever renders
/// beneath it. The body here is a full-window `mouse_area` so the test does
/// not depend on exactly where the toast anchors on screen.
#[test]
fn toast_body_click_does_not_reach_content_beneath() {
    let body: Element<Msg> = mouse_area(space().width(Length::Fill).height(Length::Fill))
        .on_press(Msg::BodyPressed)
        .into();
    let toast = Toast::new(
        7,
        ToastIntent::Info,
        "Saved",
        "All good.",
        Msg::DismissToast(7),
    );
    let layout = AppLayout::new(body).toasts(vec![toast]);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.click("Saved")
        .expect("toast title text should be findable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        !msgs.contains(&Msg::BodyPressed),
        "a click on the toast's own body must not reach content beneath it; got {msgs:?}",
    );
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
    ui.click("×")
        .expect("toast close button (×) should be findable above the modal");
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
    ui.click("Sheet action")
        .expect("sheet action button should be findable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert_eq!(
        msgs,
        vec![Msg::SheetAction],
        "clicking sheet content should produce SheetAction only",
    );
}

// ---------------------------------------------------------------------------
// v0.17 expansion — RTL / LayoutDirection integration coverage
// ---------------------------------------------------------------------------

/// Sheet content is reachable when `LayoutDirection::Rtl` is active.
///
/// Under RTL the `End` edge resolves to the left physical side. The
/// engine must still render the sheet content and make it interactive.
/// This test verifies that direction mirroring does not break the
/// interactive surface of a sheet overlay.
///
/// Verifies: ABDD render path — `SheetEdge::End` under RTL.
#[test]
fn sheet_end_edge_reachable_under_rtl() {
    use snora::LayoutDirection;
    let sheet: Sheet<Element<Msg>, Msg> =
        Sheet::new(btn("RTL Sheet", Msg::SheetAction)).at(SheetEdge::End);
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .sheet(sheet)
        .direction(LayoutDirection::Rtl)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.find("RTL Sheet")
        .expect("sheet content must be findable under RTL direction");
    ui.click("RTL Sheet")
        .expect("sheet action button should be clickable under RTL");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::SheetAction),
        "sheet button must fire SheetAction under RTL layout; got {msgs:?}",
    );
}

/// Toast dismiss button is reachable when `LayoutDirection::Rtl` is active.
///
/// Under RTL `ToastPosition::TopEnd` anchors toasts to the top-left corner.
/// The horizontal mirroring must not interfere with the dismiss button's
/// interactivity.
///
/// Verifies: ABDD render path — toast dismiss under RTL.
#[test]
fn toast_dismiss_reachable_under_rtl() {
    use snora::{LayoutDirection, ToastPosition};
    let toast = Toast::new(
        42,
        ToastIntent::Success,
        "Done",
        "Task complete.",
        Msg::DismissToast(42),
    );
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .toasts(vec![toast])
        .toast_position(ToastPosition::TopEnd)
        .direction(LayoutDirection::Rtl);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.click("×")
        .expect("toast close button (×) should be findable under RTL");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::DismissToast(42)),
        "toast dismiss must fire under RTL direction; got {msgs:?}",
    );
}

/// Context menu content (layer 3) is findable and interactive.
///
/// Verifies: the `context_menu` field uses a separate code path from
/// `header_menu` in render.rs (both are pushed after the menu backdrop
/// but as distinct stack entries). This test confirms layer 3 renders
/// correctly and does not regress if the push order changes.
#[test]
fn context_menu_content_reachable() {
    let context_el: Element<Msg> = btn("Context action", Msg::DialogOk);
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .context_menu(context_el)
        .on_close_menus(Msg::CloseMenus);
    let element = render(layout);

    let mut ui = simulator(element);
    ui.find("Context action")
        .expect("context_menu content must be findable (layer 3)");
    ui.click("Context action")
        .expect("context_menu content must be clickable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::DialogOk),
        "context_menu action must fire; got {msgs:?}",
    );
}

/// Outside click emits `on_close_menus` when a menu is open and no modal
/// is present.
///
/// Verifies: layer 1 (menu backdrop) dispatches `on_close_menus` on any
/// click outside the menu area.
#[test]
fn outside_click_on_menu_emits_close_menus() {
    // A menu element rendered at a fixed position (top-left corner area).
    // The backdrop covers the whole window; clicking the opposite corner
    // should hit the backdrop.
    let menu_el: Element<Msg> = btn("File item", Msg::BodyPressed);
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .header_menu(menu_el)
        .on_close_menus(Msg::CloseMenus);
    let element = render(layout);

    let mut ui = simulator(element);
    // Click a point far from where a menu item would typically render.
    ui.point_at(Point::new(4.0, 500.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::CloseMenus),
        "outside click with menu open should produce CloseMenus; got {msgs:?}",
    );
}

/// When both dialog and sheet are present, both render and both contain
/// interactive content.
///
/// Verifies Law 3 (RFC-011-E): dialog+sheet coexistence is supported.
/// The sheet (layer 6) is above the dialog (layer 5); both wrap their
/// content in `opaque` (RFC-084), so each captures clicks within its own
/// area regardless of which is on top. We verify the sheet content is
/// findable and clickable in this coexistence layout specifically.
#[test]
fn dialog_and_sheet_coexist_sheet_content_reachable() {
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(btn("Dialog btn", Msg::DialogOk));
    let sheet: Sheet<Element<Msg>, Msg> =
        Sheet::new(btn("Sheet action", Msg::SheetAction)).at(SheetEdge::Bottom);
    let layout = AppLayout::new(btn("body", Msg::BodyPressed))
        .dialog(dialog)
        .sheet(sheet)
        .on_close_modals(Msg::CloseModals);
    let element = render(layout);

    let mut ui = simulator(element);
    // Sheet content must be findable (sheet is topmost modal surface).
    ui.find("Sheet action")
        .expect("sheet content must be findable when both dialog and sheet are present");
    ui.click("Sheet action")
        .expect("sheet action button should be clickable");
    let msgs: Vec<Msg> = ui.into_messages().collect();

    assert!(
        msgs.contains(&Msg::SheetAction),
        "sheet button click must produce SheetAction in coexistence layout; got {msgs:?}",
    );
}

