# RFC 060 — Frame-level keyboard navigation

**Status.** Proposed
**Tracks.** Accessibility / vocabulary. Closes the 14-minor-stale deferral
recorded as **RFC-014-B**.
**Touches.** `crates/snora-core/src/focus.rs` (new),
`crates/snora-core/src/lib.rs`, `crates/snora/src/keyboard.rs`,
`docs/src/contributing/semantic-accessibility.md`,
`docs/src/reference/overlay-interaction-semantics.md`,
`docs/src/guides/accessibility.md`, `CHANGELOG.md`.
**Release target.** 0.35.0 (minor — `snora-core` gains public vocabulary).

## Summary

Snora owns the frame; applications own what is inside a pane. Keyboard
navigation splits on exactly that seam, and snora currently supplies **none** of
its half.

This RFC adds **frame-level zone navigation** as pure, iced-free decision
vocabulary in `snora-core`: given the current zone, a direction, and which
skeleton slots are populated, which zone is next — and what happens to that
answer while an overlay is open.

It deliberately does **not** take the Tab key, does not capture keyboard
events, and does not require a new iced feature.

## Motivation

Three things converged on this.

1. **The deferral is 14 minors stale.** `semantic-accessibility.md`'s keyboard
   ownership table still reads *"Focus trapping in modals | Out of v0.20 scope
   (deferred, RFC-014-B)"*, and `overlay-interaction-semantics.md` Law 7 says
   Escape is application-owned *"(for now)"*. v0.20 was fourteen minor releases
   ago and nothing has revisited it.
2. **A rigorous evaluator hit it.** tekstide implemented zone-based focus
   themselves — a `FocusZone` enum cycled by Tab/Shift+Tab — and called modal
   focus trapping *"security machinery here, proven with a positive control."*
   They built it because snora did not offer it.
3. **`FocusTokens` still has no consumer inside snora.** RFC-059 documented its
   present-day audience; the frame is that audience.

Every application with more than one region reimplements this. The decision
logic is identical across all of them, depends only on structure snora already
owns, and is pure.

## What snora owns, precisely

`AppLayout`'s skeleton has **four** slots, and that is the zone set:

| Zone | Slot | Presence |
|---|---|---|
| `Body` | `body` | always — required |
| `Header` | `header` | `Option` |
| `SideBar` | `side_bar` | `Option` |
| `Footer` | `footer` | `Option` |

Tabs and breadcrumbs are **not** zones. `crumb` and `tab` are content an
application places *inside* a slot, so they belong to whichever zone contains
them. Treating them as zones would put snora in charge of intra-pane structure
it does not own.

## The Tab key is the wrong key, and this RFC does not take it

The obvious design — Tab/Shift+Tab cycles zones — is wrong, and the reason is
worth recording so it is not proposed again.

**Tab already means "next control."** iced implements it, users expect it, and
every application with a form or a text input depends on it. If snora claims
Tab for zone cycling, in-pane Tab breaks everywhere, and the applications most
likely to want zone navigation — the ones with enough regions to get lost in —
are exactly the ones with the most in-pane controls to break.

The desktop convention for region cycling is **F6 / Shift+F6**, with Ctrl+Tab
as a secondary. This RFC therefore:

- supplies the **decision**, not the binding;
- **recommends F6 / Shift+F6** in the documentation;
- leaves the application free to bind anything, because the helper takes a
  direction, not a key.

**The one place Tab is legitimately snora's concern is inside a modal**, where
it must be *bounded* rather than reassigned. That is containment, and it is
staged — see Q-1.

## Design

### Pure decisions in `snora-core`, no event capture

Snora already has the right pattern, shipped: `snora::keyboard::dismiss_on_escape`
is a pure function taking the key and the overlay state, returning
`Option<Message>`. The application wires `iced::keyboard::listen()` and calls
it. Snora captures nothing.

This RFC follows that shape exactly. The new module is **iced-free** and lives
in `snora-core`, so it is unit-testable without a renderer — the same property
that makes the contrast suite trustworthy.

```rust,ignore
pub enum FocusZone { Header, SideBar, Body, Footer }

pub enum Cycle { Forward, Backward }

/// Which zones the current layout actually has.
pub struct ZonePresence { /* header, side_bar, footer — body is implied */ }

/// The next zone in logical order, skipping absent slots and wrapping.
pub fn next_zone(current: FocusZone, cycle: Cycle, present: ZonePresence)
    -> Option<FocusZone>;
```

Exact signatures are the implementer's, subject to §Constraints.

### Logical order, and why no mirroring

The cycle order is **Header → SideBar → Body → Footer**, wrapping.

This is *logical* order, not visual, so it is **direction-independent**: under
RTL the sidebar renders on the other edge, but it is still the start-edge rail
and still follows the header. Unlike `ToastPosition`, this needs no mirroring —
and that is a deliberate ABDD decision to record, not an omission. A future
reader should not "fix" it by adding a `LayoutDirection` parameter.

### Containment while an overlay is open

Snora already owns the z-stack and the modal-before-menu dismissal priority.
Containment is the same ownership expressed for focus, and it is a *decision*,
not an interception:

- **A modal is open (`dialog` or `sheet`)** — zone cycling is **suspended**.
  Focus belongs inside the modal, and its contents are an application-supplied
  `Node` that snora cannot enumerate. `next_zone` reports this rather than
  silently returning `Body`.
- **Only a menu is open (`header_menu` / `context_menu`)** — cycling is
  unaffected. Menus are light-weight and dismissible on outside click; they do
  not own focus.

This mirrors `dismiss_on_escape`'s existing priority exactly, which is the
point: one overlay-precedence model, expressed twice.

### A thin engine-side helper

`snora::keyboard` gains a companion to `dismiss_on_escape` that takes a key and
returns the cycle direction, so applications are not re-deriving
"F6 means forward, Shift+F6 means backward." Same shape, same non-capture
policy.

## What this RFC does not do

- **Does not take Tab or Shift+Tab.** See above.
- **Does not capture keyboard events.** No subscription is installed by snora.
- **Does not require the iced `advanced` feature.** Verified — see Constraints.
- **Does not render the focus ring.** Staged; see Q-2.
- **Does not implement full modal focus trapping.** Staged; see Q-1.
- **Does not add zones for tabs or breadcrumbs.**
- **Does not change `AppLayout`'s fields**, the z-stack, or dismissal.
- **Does not touch `render_semantics`.**

## Constraints, verified rather than assumed

Probed against iced 0.14 with **snora's exact feature set** (`canvas`, `svg`,
`tokio` — no `advanced`), compile-checked:

| Capability | Path | Reachable today |
|---|---|---|
| Move focus forward | `iced::widget::operation::focus_next()` → `Task` | **yes** |
| Move focus backward | `iced::widget::operation::focus_previous()` → `Task` | **yes** |
| Query which widget is focused | `focusable::find_focused()` | **no** — needs `advanced` |

The asymmetry is the whole scoping argument: **moving** focus is available now,
**knowing where it is** is not. Zone navigation only needs to move. Trapping —
wrapping at the modal boundary — needs to know, so it needs `advanced`.

Stating this narrowly matters. Per RFC-059, the failure mode here is recording
"iced 0.14 cannot do focus" and closing the question; the accurate constraint is
that iced does not *report* focus without an opt-in feature.

## Open questions

**Q-1 — Does full modal focus trapping justify enabling iced's `advanced`
feature?** Trapping needs `find_focused`, which needs `advanced`. That widens
snora's iced surface, has a compile-cost and binary-size consequence measurable
under our own budgets, and touches API stability. **Do not enable it as an
implementation detail of this RFC.** Measure it and decide separately; this RFC
ships the useful half without betting on the answer.

**Q-2 — Should the frame render a focus ring on the active zone?** Snora draws
the skeleton containers, so it *can* — and `FocusTokens` exists precisely for
it, with no consumer inside snora today. But it is `design`-gated, it is an
appearance change on the default path, and it is a second unit of work.
Suggest: not in this RFC, opened immediately after, with this RFC's zone
vocabulary as its input.

**Q-3 — Does the application or snora hold the current zone?** The application,
consistent with every other piece of snora state (`toasts`, overlay flags).
`next_zone` is a pure function *of* that state. Confirm rather than assume, and
record the reasoning where a reader will find it.

## Acceptance criteria

1. `snora-core` gains an iced-free focus module with the zone vocabulary and
   `next_zone`, unit-tested including absent slots, wrap-around in both
   directions, and the body-only degenerate case.
2. Cycling is suspended while a modal is open and unaffected while only a menu
   is open, tested for both.
3. `snora::keyboard` gains the direction helper, following
   `dismiss_on_escape`'s shape; snora installs no subscription.
4. The keyboard ownership table is **rewritten**, not amended — its RFC-014-B
   row and the v0.20 scope statement are both retired, and it states what is
   now snora's, what remains the application's, and what is staged behind Q-1.
5. `overlay-interaction-semantics.md` Law 7's *"(for now)"* is resolved, and
   the focus-trapping row states the narrow constraint rather than a blanket
   deferral.
6. `guides/accessibility.md` documents zone navigation for consumers, recommends
   **F6 / Shift+F6**, and states plainly that snora does not take Tab and why.
7. `render_semantics` passes unmodified; no new iced feature is enabled.

## Compatibility and security

**Compatibility.** Purely additive: new vocabulary in `snora-core`, one new
helper in `snora::keyboard`. No existing type, field, or signature changes; no
rendering changes; nothing is deprecated. Applications that navigate zones
today keep working untouched.

**Security.** No new data flow or dependency. Worth noting the inverse,
however: tekstide treats modal focus containment as security machinery, because
focus escaping a modal can let a keyboard user reach controls the modal is
meant to be gating. This RFC does not deliver containment — Q-1 does — and
must not be read as having done so.
