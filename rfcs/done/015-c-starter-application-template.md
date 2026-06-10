# RFC-015-C — Starter Application Template

**Status.** Implemented (v0.15.0)
**Tracks.** Adoption aid / example.
**Touches.** `examples/starter/` (new workspace member),
`docs/src/getting-started/07-starter-application.md` (new),
`docs/src/SUMMARY.md`, `examples/README.md`.

> Option A (copyable `examples/starter`) + Option C (docs chapter).
> Option B (separate `cargo generate` repo) deferred until first
> downstream adoption.

## 1. Design decisions

### Use prefab widgets (not custom iced elements)

A starter using raw `iced::widget::*` everywhere would be harder to
read than a Snora-idiomatic app. Using `app_header`, `app_side_bar`,
`app_footer` shows the recommended path. The workbench already does
this; the starter is a *minimal* version of it.

### Single `src/main.rs`, ≤200 ELOC

The starter must be immediately readable. Split-file structure
(`app.rs`, `message.rs`, etc.) from the planning draft is rejected
for the first iteration — it implies a project structure Snora does
not own. Keep all code in one file with section comments.

### Wire Escape via `snora::keyboard::dismiss_on_escape`

v0.14 ships the helper. The starter should use it — that's why it was
added. This is one of the key "correct pattern" demonstrations.

### No async background tasks

Scope creep risk. The starter demonstrates `AppLayout` + overlays +
toasts + direction. Background work belongs to application documentation,
not the starter.

## 2. Starter surfaces

- Header with a single menu button + RTL toggle.
- Sidebar with two navigation items.
- Tab bar (two tabs: Overview, Settings).
- Dialog opened from a button.
- Toast added from a button (transient).
- Direction toggle (LTR↔RTL) demonstrating ABDD.
- Escape wiring via `dismiss_on_escape`.
- Footer showing active tab and direction.

Intentionally omitted from starter (to keep it ≤200 ELOC):
- Sheet overlay, context menu, breadcrumb, all six toast intents/positions.
Those are in the workbench.

## 3. Acceptance criteria

- `examples/starter` builds in CI.
- ≤200 ELOC in `src/main.rs`.
- Uses `snora::keyboard::dismiss_on_escape`.
- Has LTR/RTL toggle.
- Has comments distinguishing Snora-owned and app-owned code.
- Getting-started page `07-starter-application.md` exists and is in SUMMARY.
- `examples/README.md` mentions the starter and its purpose.
