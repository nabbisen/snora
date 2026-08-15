# RFC 051 — The responsive example excludes the consumer who asked for it

**Status.** Implemented (v0.30.0)
**Tracks.** Examples and documentation. Follow-through from apimokka's
post-adoption report (2026-08-15).
**Touches.** `examples/` (one new crate), `examples/README.md`,
`docs/src/guides/responsive.md`, the root `Cargo.toml` **comment** and
`docs/src/contributing/release-process.md` (see the handoff §4 — this adds a
second hand-pinned example). **No library code**, and no `workspace.members`
edit: the root manifest globs `examples/*`.
**Release target.** 0.30.0 (minor — a new example crate is a workspace
member; no API change).

## Summary

`responsive_render` shipped in v0.28.0 because **apimokka asked for it**.
Every demonstration snora provides teaches it through `AppLayout::side_bar`,
built with `snora::widget::app_side_bar` — and apimokka uses **neither**.

The feature's own requester cannot copy the example that teaches it.

This RFC adds a second responsive example that varies **`body`'s own
composition**, the pattern both known consumers actually use, and scopes the
existing example rather than replacing it.

## Motivation

apimokka's post-adoption report gives their `AppLayout` usage precisely:

> **Used:** `new`, `header`, `direction`, `header_menu` + `on_close_menus`,
> `dialog` + `on_close_modals`, `sheet` + `on_close_modals`.
> **Ignored entirely:** `side_bar`, `footer`, `context_menu`, `toasts`,
> `toast_position`.

with the reason:

> we build our own chrome (a top bar, a horizontal tab bar, and per-screen
> sidebars) and compose it into the `body` node we pass to `AppLayout::new`
> … our tab bar sits *inside* body, stacked above the screen content, not in
> a slot `AppLayout` names

They also report **zero `snora::widget::*` call sites**, unchanged from
0.25.2 through 0.28.0. arama, the other known consumer, is the same shape.

Every responsive demonstration snora ships:

| Where | What it does |
|---|---|
| `docs/src/guides/responsive.md:17` | `layout.side_bar(sidebar(state))` |
| `docs/src/guides/migration-0.27-to-0.28.md:24` | `layout = layout.side_bar(sidebar)` |
| `examples/responsive/src/main.rs:105` | `layout.side_bar(sidebar)`, sidebar built with `app_side_bar` |

`examples/responsive` imports `AppLayout, LayoutDirection, SideBar,
SideBarItem, responsive_render` plus prefab widgets. An engine-only consumer
must translate every line before it teaches them anything.

## What this RFC is not claiming

**snora does have engine-only examples** — six of eighteen, verified by
reading each example's `use snora::{…}` imports rather than by grep:
`hello`, `dialog`, `sheet`, `toast`, `context_menu`, `size_probe_engine`.

Notably `dialog` and `sheet` are engine-only, and those are precisely the two
things apimokka reports it could not replicate itself. That part of the
example set already serves them.

The gap is **specific to the responsive feature**, not systemic. An earlier
draft of this analysis asserted the systemic version on a grep that matched
`iced::widget::` as well as `snora::widget::`; it was wrong and is recorded
here so the narrower, true claim is what gets implemented.

## Why this matters more than a normal docs gap

RFC-046 shipped width exposure and **deliberately declined** to ship
breakpoint behaviour, on the stated grounds that snora does not yet know which
thresholds real applications converge on — and that shipping exposure is what
makes that evidence gatherable.

apimokka is now sequencing exactly that evidence:

> Implementing `responsive_render`-based breakpoints is a behaviour change
> we're sequencing after our own UX validation sessions, specifically so we
> choose real thresholds from evidence of what users do at small window sizes
> rather than a guess.

**Their thresholds are the input that decides whether snora ever ships
breakpoint behaviour.** They will generate that evidence against whatever
example they work from. If the only worked example collapses a `side_bar`
they do not have, the evidence arrives shaped by a pattern neither consumer
uses — or arrives later, because they had to translate first.

That is the argument for doing this before their sessions rather than after.

## Design

### A second example: `examples/responsive_body`

Engine-only. No `snora::widget::*`, no `side_bar`, no `footer`.

Demonstrates the pattern both consumers have: application-owned chrome
composed **into** `body`, with the composition varying by available width.
The obvious analogue of apimokka's own layout is a horizontal tab bar that
stays horizontal at wide widths and becomes something compact below a
threshold the *example* picks for itself.

Constraints, inherited from RFC-046 and non-negotiable:

- **The example picks its own threshold and says so.** snora prescribes none.
  The number must read as the application's decision, not a snora default.
- **Must respect `LayoutDirection`.** Chrome composed into `body` is still
  direction-sensitive, and a demonstration that only works LTR would teach
  the wrong thing.
- Buildable with `snora` default features off, and **verified that way** —
  that is what makes "engine-only" a fact rather than a label.

### Scope the existing example, do not replace it

`examples/responsive` stays. Sidebar collapse is a fair archetype and the
prefab widgets are a legitimate way to build one.

`docs/src/guides/responsive.md` gains a short paragraph naming both, and
saying which reader each is for: slot-based chrome versus chrome composed
into `body`. Two sentences, not a section.

## Non-goals

- **No `Breakpoint` type, no thresholds, no adaptive behaviour in snora.**
  Unchanged from RFC-046. This RFC exists partly to protect that deferral by
  improving the evidence it waits on.
- **No change to `responsive_render`** or any library code.
- **No removal or rewrite of `examples/responsive`.**
- **No claim that prefab widgets are the wrong default.** They have no
  demonstrated downstream adoption, which is not the same as being wrong.
- **No third example.** Two patterns is the point; a matrix of examples is
  its own maintenance burden.

## Open questions

**Q-1 — Is a tab bar the right second pattern?**
It mirrors apimokka's actual layout, which is the argument for it. The
argument against is that it may read as "snora is about tab bars." An
alternative is a two-column body that becomes one column. **Pick one and say
why in the review request** — do not build both.

**Q-2 — Does the guide need the distinction, or just the example?**
A reader who lands on `responsive.md` and sees only the `side_bar` snippet may
never reach the second example. Suggest the guide names both up front; the
implementer should judge whether that costs more clarity than it buys.

## Acceptance criteria

1. A new engine-only responsive example exists, registered as a workspace
   member, using **no** `snora::widget::*` and **no** `side_bar`/`footer`.
2. It **builds with `snora` default features disabled**, demonstrated in the
   review request — not merely asserted.
3. It varies `body`'s composition by width, picks its own threshold, and says
   in a comment that the threshold is the application's choice.
4. It behaves correctly under both `LayoutDirection` values.
5. `docs/src/guides/responsive.md` names both examples and who each is for.
6. `examples/responsive` is unchanged.
7. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
   --all-features -- -D warnings`, and `mdbook test docs` pass;
   `render_semantics` passes unmodified.

## Compatibility and security

**Compatibility.** Additive. A new example crate and two sentences of prose.
No library API, no rendering change.

**Security.** No new dependency or data flow. The example crate uses the same
`snora` workspace dependency as every other example.

## Release implications

**0.30.0, minor** — a new workspace member. No migration guide; nothing to
migrate. Worth a CHANGELOG line under **Added**, naming who it is for, since
the point is discoverability by a particular kind of reader.
