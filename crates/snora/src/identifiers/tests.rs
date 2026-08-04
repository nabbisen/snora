//! Tests for RFC-047's stable identifiers.
//!
//! - **Identifiers present**: every surface in scope actually carries its
//!   documented identifier in a genuinely rendered widget tree, verified
//!   with `iced_test::Simulator::find` — the same harness
//!   `render_semantics.rs` uses — not merely asserted from reading the
//!   source.
//! - **Documentation drift**: `docs/src/reference/
//!   rendered-surface-identifiers.md`'s identifier table lists exactly
//!   [`ALL_STATIC`] — no more, no fewer. This is the test that earns its
//!   keep, per the Handoff: a hand-maintained list drifts, and a stale
//!   reference is worse than none.
//! - **Toast identifier stability**: the same toast id always derives the
//!   same identifier.

use super::*;
use iced::Element;
use iced::widget::{Id, button, text};
use iced_test::simulator;
use snora_core::{AppLayout, Dialog, Sheet, SheetEdge, Toast, ToastIntent};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Noop,
}

fn el<'a>() -> Element<'a, Msg> {
    button(text("x")).on_press(Msg::Noop).into()
}

// ---------------------------------------------------------------------------
// Identifiers present in rendered output.
// ---------------------------------------------------------------------------

/// Every documented surface, rendered simultaneously (menu, dialog+sheet
/// coexisting as `render_semantics.rs` already establishes is supported,
/// toasts, and all four skeleton regions), then checked one by one with
/// [`Simulator::find`]. This proves the identifier is actually attached
/// to a real widget in the tree, not just referenced in source.
#[test]
fn identifiers_present_in_rendered_output() {
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(el());
    let sheet: Sheet<Element<Msg>, Msg> = Sheet::new(el()).at(SheetEdge::Bottom);
    let toast = Toast::new(7, ToastIntent::Info, "t", "m", Msg::Noop);

    let layout = AppLayout::new(el())
        .header(el())
        .side_bar(el())
        .footer(el())
        .header_menu(el())
        .dialog(dialog)
        .sheet(sheet)
        .toasts(vec![toast])
        .on_close_menus(Msg::Noop)
        .on_close_modals(Msg::Noop);

    let element = crate::render::render(layout);
    let mut sim = simulator(element);

    for name in ALL_STATIC {
        sim.find(Id::new(name))
            .unwrap_or_else(|_| panic!("expected to find a widget with id {name:?}"));
    }

    // The per-toast identifier, derived from the toast's own id (7 above).
    sim.find(Id::from(toast_id(7)))
        .expect("expected to find the individual toast's identifier");
}

/// `dim_without_capture` (no `on_close_modals` wired) is a distinct code
/// path from `dim_backdrop` — both must carry [`MODAL_DIM`]. The test
/// above only exercises the click-capturing variant (since it sets
/// `on_close_modals`); this one exercises the other, per the Handoff's
/// explicit instruction not to skip either dim variant "because they look
/// alike."
#[test]
fn modal_dim_present_without_close_handler() {
    let dialog: Dialog<Element<Msg>, Msg> = Dialog::new(el());
    let layout = AppLayout::new(el()).dialog(dialog);
    // on_close_modals intentionally absent — triggers dim_without_capture.

    let element = crate::render::render(layout);
    let mut sim = simulator(element);

    sim.find(Id::new(MODAL_DIM))
        .expect("dim_without_capture must also carry MODAL_DIM");
}

// ---------------------------------------------------------------------------
// Documentation drift.
// ---------------------------------------------------------------------------

/// Extracts every identifier listed in the reference page's table —
/// lines of the form `` | `snora-...` | ... `` — rather than scanning the
/// whole document loosely, so a prose mention of the dynamic
/// `snora-toast-{id}` pattern elsewhere on the page cannot accidentally
/// match.
fn documented_identifiers() -> Vec<String> {
    let doc = include_str!("../../../../docs/src/reference/rendered-surface-identifiers.md");
    doc.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("| `")?;
            let end = rest.find('`')?;
            let name = &rest[..end];
            name.starts_with("snora-").then(|| name.to_string())
        })
        .collect()
}

#[test]
fn documented_identifiers_match_emitted_set() {
    let mut documented = documented_identifiers();
    documented.sort();
    documented.dedup();

    let mut emitted: Vec<String> = ALL_STATIC.iter().map(|s| (*s).to_string()).collect();
    emitted.sort();

    assert_eq!(
        documented, emitted,
        "docs/src/reference/rendered-surface-identifiers.md's identifier table must list \
         exactly ALL_STATIC — no more, no fewer. A stale reference is worse than none."
    );
}

// ---------------------------------------------------------------------------
// Toast identifier stability.
// ---------------------------------------------------------------------------

#[test]
fn toast_id_is_stable_and_derived_deterministically() {
    assert_eq!(
        toast_id(42),
        toast_id(42),
        "the same toast id must always derive the same identifier"
    );
    assert_eq!(toast_id(42), "snora-toast-42");
    assert_ne!(
        toast_id(1),
        toast_id(2),
        "different toast ids must derive different identifiers"
    );
}
