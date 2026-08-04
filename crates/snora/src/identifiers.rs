//! Stable identifiers snora attaches to the surfaces it renders itself
//! (RFC-047).
//!
//! An application can label its own header content, its own dialog
//! content, its own buttons — it owns those elements. It cannot label
//! the modal dim, the menu backdrop, or the card snora wraps its dialog
//! content in, because it never sees them. This module is the single
//! source of truth for every name snora emits for those surfaces.
//!
//! # Convention
//!
//! `snora-` prefix, kebab-case (RFC-047 Q-1). The prefix is not
//! decoration: it keeps snora's identifiers distinguishable from an
//! application's own in a widget tree the application also populates,
//! which prevents collisions.
//!
//! # These are public API from the first commit
//!
//! Attaching an `Id` is a line per surface; publishing the name is a
//! stability commitment. Once a downstream test asserts on
//! `snora-modal-dim`, renaming it breaks that test **silently at
//! runtime**, not at compile time. `docs/src/reference/rendered-surface-identifiers.md`
//! documents every name below for consumers; `docs/src/contributing/
//! versioning-policy.md` records that renaming or removing one is a
//! **minor**, not a patch. `identifiers/tests.rs`'s drift test keeps the
//! two in sync — see [`ALL_STATIC`].
//!
//! # Every call site uses these constants, never a re-typed literal
//!
//! This is what makes documentation drift structurally harder, not just
//! discouraged: `render.rs`, `overlay/dialog.rs`, `overlay/sheet.rs`, and
//! `toast.rs` all reference the symbols below rather than writing out
//! `"snora-..."` a second time.

/// The menu backdrop — a transparent, full-window click sink shown while
/// a header/context menu is open, used to detect an outside click.
/// Attached in `render.rs`'s `transparent_backdrop`.
pub(crate) const MENU_BACKDROP: &str = "snora-menu-backdrop";

/// The modal dim — the full-window scrim shown while a dialog or sheet
/// is open. **Shared** by both variants in `render.rs`
/// (`dim_backdrop`, the click-capturing one, and `dim_without_capture`,
/// used when the application supplied no `on_close_modals`) —
/// deliberately the same name. An `Id` identifies *the surface*, not its
/// interactive behavior: a test looking for "the dim" wants either
/// variant regardless of whether a close handler was wired, and the two
/// variants are the same visual surface with a different click sink, not
/// two different surfaces.
pub(crate) const MODAL_DIM: &str = "snora-modal-dim";

/// The dialog's centered container — attached whether or not the
/// `design`-gated card styling (RFC-039) is active, so the identifier is
/// stable across both `snora::render` and `snora::design::render`.
/// Attached in `overlay/dialog.rs`.
pub(crate) const DIALOG_CARD: &str = "snora-dialog-card";

/// The sheet's own surface container (the styled, opaque panel — not the
/// spacer cells around it). Attached in `overlay/sheet.rs`.
pub(crate) const SHEET_PANEL: &str = "snora-sheet-panel";

/// The toast stack's outer container. Individual toasts additionally get
/// [`toast_id`]. Attached in `toast.rs`.
pub(crate) const TOAST_STACK: &str = "snora-toast-stack";

/// The header **region** — the skeleton slot, not the application's
/// header content. snora wraps whatever `Element` the application
/// supplied in a container carrying this id; the content inside remains
/// unlabeled and is the application's to identify (RFC-047 N-4).
/// Attached in `render.rs`'s `build_skeleton`.
pub(crate) const HEADER_REGION: &str = "snora-header";

/// The sidebar region. See [`HEADER_REGION`] for the slot-vs-content
/// distinction (RFC-047 Q-3): this labels the slot snora composes, never
/// the application's sidebar content.
pub(crate) const SIDEBAR_REGION: &str = "snora-sidebar";

/// The body region. Unlike the other three skeleton slots, `body` is
/// mandatory on every `AppLayout` (not `Option`), so this identifier is
/// always present.
pub(crate) const BODY_REGION: &str = "snora-body";

/// The footer region. See [`HEADER_REGION`].
pub(crate) const FOOTER_REGION: &str = "snora-footer";

/// Derives a stable identifier for an individual toast from its
/// application-supplied `u64` id (`Toast::id`). Deterministic: the same
/// `id` always produces the same string, so the same logical toast
/// carries the same identifier across every render — verified directly
/// in `identifiers/tests.rs` rather than assumed, since per-toast
/// stability was flagged as something to confirm, not assume (RFC-047
/// §"Naming").
///
/// Dynamic, so it is not part of [`ALL_STATIC`] — the reference page
/// documents its *pattern*, `snora-toast-{id}`, not a specific instance.
pub(crate) fn toast_id(id: u64) -> String {
    format!("snora-toast-{id}")
}

/// Every static identifier name snora emits. Used by the documentation-
/// drift test to confirm `docs/src/reference/
/// rendered-surface-identifiers.md` lists exactly this set — no more, no
/// fewer. [`toast_id`] is intentionally excluded; see its own docs.
///
/// `#[cfg(test)]`: this constant's only purpose is the drift test; it has
/// no non-test consumer, so it is scoped out of non-test builds entirely
/// rather than left to trip `dead_code` there.
#[cfg(test)]
pub(crate) const ALL_STATIC: &[&str] = &[
    MENU_BACKDROP,
    MODAL_DIM,
    DIALOG_CARD,
    SHEET_PANEL,
    TOAST_STACK,
    HEADER_REGION,
    SIDEBAR_REGION,
    BODY_REGION,
    FOOTER_REGION,
];

#[cfg(test)]
mod tests;
