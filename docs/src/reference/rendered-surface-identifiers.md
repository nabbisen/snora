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

## Driving a snora application under test

Identifiers exist so a harness can find a surface. Getting *input* to that
surface is a separate problem, and it is not one snora solves — we install no
input path and inject nothing at the OS level. Two routes exist, and the cheaper
one is not obvious.

### In-process, no window: `iced_test`

`iced_test::Simulator` drives the widget tree headlessly — no window, no
compositor, no injection path to fail. It exposes `tap_key`, `press_key` and
`typewrite`, and `snapshot(&Theme)` renders to pixels for comparison against a
stored PNG.

snora's own `render_semantics` suite uses this simulator (for clicks; we have
never used `snapshot`). We assert composition rather than pixels by choice —
RFC-011-D — so treat our experience as covering the input half only.

**Prefer the in-process route for input.** `tap_key` removes the entire class of
problem described below. **The `snapshot` half is a weaker recommendation** — see
the two defaults immediately after, and the reference-churn cost below them.

**Two defaults make a snapshot suite pass while testing nothing.** Both are
reasonable for a tool used interactively and become traps in CI. Reported by
**arama**, verified against `iced_test` 0.14:

- **A missing reference auto-passes.** `matches_image` returns `Ok(true)` *and
  writes the PNG* when the path does not exist. So does `matches_hash`. A first
  run always passes — and so does a run after someone deletes a reference,
  silently baselining whatever the code produced at that moment.
- **The reference filename embeds the renderer**, formatted `{stem}-{renderer}`.
  A `wgpu` reference and a `tiny-skia` reference are different files.

**They compound.** A CI runner without a GPU falls back to `tiny-skia`, looks
for a reference that does not exist under that name, creates it, and passes.
**The suite is green and inert, and nothing says so.**

If you adopt this route, assert your references exist before trusting a green
run, and treat a suite that has never failed as unproven rather than correct.
That is the same rule as the oracle trap below, one layer up: **a check that
cannot fail is not a check.**

**And reference images have to survive our releases, which they may not.** snora
shipped three rendered changes in the eleven releases 0.31.0–0.37.2 — the border
repair (0.34.0), `chip`'s dismiss control (0.36.0), and the modal dim (0.37.0).
Each invalidates every reference image that contains the affected surface.

One consumer (**aaai**, 2026-08-19) declined snapshot testing for exactly this
reason, and their reasoning is worth repeating rather than paraphrasing: they
are moving *"anything assertable becomes a test, and screenshots shrink to the
few things needing human judgement, kept out of CI."* Snapshot testing would
rebuild the problem they are removing.

We are not telling you not to use it. We are telling you the cost is recurring
and paid by you, on our schedule — and that we have never run it, so we cannot
tell you whether it needs a GPU in CI.

### External, real window: verified bounds

Where evidence must be about what a person sees rather than what a reducer
believes, an external driver is the only option. **apimokka**
([apimock-rs](https://github.com/nabbisen/apimock-rs)) verified the following
and asked that it be published with its limits, because an over-broad version
would be worse for a harness author than none:

**Verified by running it:**

- `wtype` — an ordinary Wayland client using `zwp_virtual_keyboard_manager_v1` —
  delivered synthesized keystrokes to a Wayland client, confirmed by reading the
  typed string back out of the target. **No root, no daemon, no `uinput`.**
- **niri did not act on its own keybinding** when that keybinding arrived from
  the virtual keyboard, while still forwarding the keys to clients. `Mod+Left`,
  bound to `focus-column-left`, produced no focus change; `niri msg
  focused-window` reported the same window before and after.

**Not verified — do not assume:**

- **Any compositor other than niri 26.04.** One compositor, once.
- **Pointer injection.** `zwlr_virtual_pointer_manager_v1` is advertised by that
  compositor; it was never exercised.

### The oracle trap, which is the general lesson

This is the part worth reading even if you never touch Wayland.

apimokka's first three probes injected `Ctrl+K` into their own application and
compared screenshots. All byte-identical, so they concluded delivery had failed.
It had not: their `view` short-circuits to a first-run picker while one piece of
state is unset, so **a successful injection and a failed one produced the same
pixels.** They then tried a compositor keybinding as the oracle, which reported
failure for the unrelated reason above — two independent false negatives,
agreeing with each other, both wrong.

> **Verify input delivery against a surface guaranteed to change, in a client you
> can read** — not the application under test, until you know it can show you the
> difference.

A terminal running `cat` settled it in two minutes.

**aaai** hit the same trap independently, from a different cause — three wrong
conclusions that keyboard synthesis was unavailable, when the keys they were
testing simply had no visible effect on the screen under test. Their settling
check is the cheaper one: **`xdotool type` into a text field.** If characters
appear, every later negative is an application finding, not a delivery finding.

### Four more traps, from driving a real window

Reported by **aaai** (2026-08-19), running an `iced` application under forced
XWayland. None is snora-specific; all cost them a diagnosis cycle.

- **Two clients of the same application, one at the wrong scale.** The same
  button measured 139 px alone and 81 px with a second instance running —
  identical commands, identical environment, no error, correct
  compositor-reported window size, correct screenshot dimensions. Only the
  content was wrong, and they misread it as a theme defect twice. **Launch
  capture instances sequentially.**
- **A compositor-reported window size does not validate a capture.** It reports
  logical size and says nothing about the scale the application laid out
  against. Re-taking a capture with the size re-verified immediately beforehand
  produced a pixel-identical wrong result. What works is measuring a stable
  element across the captures in a set.
- **A window off the visible workspace eats clicks silently.** Pointer events go
  by position and need no focus, but they do need the target displayed. A
  floated window on a workspace that was not on screen took every click with no
  error and a plausible-looking screenshot.
- **The invocation matters more than the path.** aaai delivered both pointer and
  keyboard events under forced XWayland using the *combined* form —
  `xdotool mousemove <x> <y> click 1`, after `xdotool windowactivate --sync` —
  and never separated `mousedown`/`mouseup`. **arama** reported pointer motion
  arriving but button press/release not, on what appears to be the same path.
  The two results are unreconciled; the combined-versus-separated call and
  whether the window was activated first are the likeliest difference. Neither
  team has reproduced the other, and snora has run neither.

The same shape applies to a snora application: if your `view` collapses to one
branch under some state, identifier-based assertions and screenshots will agree
with each other and both be uninformative. Establish that your harness can
observe a difference *before* you trust it to report the absence of one.

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
