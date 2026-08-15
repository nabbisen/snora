# Rendered surface identifiers

snora attaches a stable `iced::widget::Id` to every surface it renders
itself — the backdrops, the dialog card, the sheet panel, the toast
stack, the skeleton regions. An application can label its own content;
it cannot label these, because it never sees them.

**This provides labels on snora-rendered output — nothing more.** It is
not a test harness (see [`snora-test` — firm non-goal](../contributing/design-decisions.md)),
not a state-query API, and not accessibility semantics. An `Id` is not a
role; see [snora's position on assistive
technology](../contributing/semantic-accessibility.md#position-on-assistive-technology-rfc-045)
for what that would require.

## Compatibility

**From this release, the identifiers below are a compatibility
surface.** Renaming or removing one is a **minor** version bump, not a
patch — see the [versioning policy](../contributing/versioning-policy.md#rendered-surface-identifiers).
Once a downstream test asserts on `snora-modal-dim`, renaming it breaks
that assertion silently at runtime, not at compile time; treat these
names as carefully as any other public API.

## Convention

`snora-` prefix, kebab-case. The prefix keeps snora's identifiers
distinguishable from an application's own in a widget tree the
application also populates, which prevents collisions.

## Static identifiers

| Identifier | Surface | Source |
|---|---|---|
| `snora-menu-backdrop` | The menu backdrop — a transparent, full-window click sink shown while a header or context menu is open. | `render.rs` |
| `snora-modal-dim` | The modal dim — the full-window scrim shown while a dialog or sheet is open. Shared by both the click-capturing and non-capturing variants; see "Why the two dim variants share a name" below. | `render.rs` |
| `snora-dialog` | The dialog's centring container — the full-window layer that positions dialog content. **Always present**, on both `snora::render` and `snora::design::render`. | `overlay/dialog.rs` |
| `snora-dialog-card` | The dialog's styled card container (fill, border, radius; RFC-039). **`design` path only** — on the default `snora::render` path no card exists, so this identifier is never emitted there. Before v0.29.0, this name was attached to the centring container instead; see the [migration guide](../guides/migration-0.28-to-0.29.md). | `overlay/dialog.rs` |
| `snora-sheet-panel` | The sheet's own surface panel (the styled, opaque container — not the spacer cells around it). | `overlay/sheet.rs` |
| `snora-toast-stack` | The toast stack's outer container. | `toast.rs` |
| `snora-header` | The header **region** — the skeleton slot, not the application's header content. | `render.rs` |
| `snora-sidebar` | The sidebar region. | `render.rs` |
| `snora-body` | The body region. Always present — `body` is mandatory on every `AppLayout`, unlike the other three slots. | `render.rs` |
| `snora-footer` | The footer region. | `render.rs` |

## Per-toast identifiers

Each individual toast additionally carries `snora-toast-{id}`, where
`{id}` is the toast's own `u64` id (`Toast::id`) — for example,
`snora-toast-42`. This is deterministic: the same toast id always
derives the same identifier, so the same logical toast carries the same
identifier across every render, verified directly in
`crates/snora/src/identifiers/tests.rs`.

## What is *not* identified

Slot **contents** are the application's elements and the application's
to identify — snora does not, and will not, attach identifiers inside
content an application supplies. This includes: the application's
header/sidebar/footer/body content, dialog and sheet content, and
individual buttons or controls the application builds. Only the
surfaces snora itself composes (listed above) carry an identifier.

## Why the two dim variants share a name

`render.rs` has two functions that paint the modal dim: `dim_backdrop`
(click-capturing, used when the application supplied `on_close_modals`)
and `dim_without_capture` (used when it did not — the dim still paints,
to signal "this is modal", but does not capture clicks). Both carry
`snora-modal-dim`, deliberately the same name: an identifier names *the
surface*, not its interactive behavior. A test looking for "the dim"
wants either variant regardless of whether a close handler happens to be
wired; the two variants are the same visual surface with a different
click sink attached, not two different surfaces.

## Why the skeleton regions label the slot, not the content

`snora-header`/`snora-sidebar`/`snora-body`/`snora-footer` are attached
to a container `render.rs` wraps around whatever `Element` the
application supplied for that slot. The identifier marks the **slot** —
where snora composed it into the skeleton — not the application's
content inside it, which stays unlabeled and is the application's own to
identify if it wants to. A test looking for "the sidebar" almost always
means the region, which is what these identifiers give you.
