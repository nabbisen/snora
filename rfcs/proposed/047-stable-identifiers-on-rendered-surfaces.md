# RFC 047 — Stable identifiers on snora-rendered surfaces

**Status.** Proposed
**Tracks.** Testability and observability. Raised by the apimokka team
(2026-08-04) alongside the assistive-technology question.
**Touches.** `crates/snora/src/render.rs`, `overlay/dialog.rs`,
`overlay/sheet.rs`, `toast.rs`, `docs/src/guides/`.

## Summary

A snora application is externally unobservable. A downstream team building
scripted GUI verification found no widget identifiers, no semantic names,
and no state query — the only readable signal was the window title, which
they were using as an accessibility API because nothing else existed.

This RFC attaches **stable, documented `iced::widget::Id`s to the surfaces
snora renders itself** — the backdrops, the dialog card, the sheet panel,
the toast stack. It does not attempt a test harness, and it does not touch
application content.

## Motivation

**Verified:** `grep -rniE "widget::Id|semantic_id" crates/*/src/` returns
nothing.

The surfaces snora renders are precisely the ones an application *cannot*
label itself. An application can put identifiers on its own header, its own
dialog content, its own buttons — it owns those elements. It cannot label
the modal dim, the menu backdrop, or the card that snora wraps its dialog
content in, because it never sees them.

So the observable gap is exactly snora-shaped.

## The separation that matters

The downstream report suggested this and the assistive-technology request
"may be the same piece of work." **They are not, and treating them as one
would be a mistake:**

- An **accessibility tree** requires iced to expose one. It is blocked
  upstream and RFC-045 states the position rather than building it.
- **Identifiers** need only `iced_core::widget::Id`, which exists and is
  public today (`container(...).id(...)` is available in iced 0.14).

They share an intuition — "name the things snora renders" — but not a
blocker. Bundling them would hold the deliverable half hostage to the
blocked half. This RFC ships now; RFC-045 waits on iced.

If iced later exposes an accessibility API, these identifiers are plausibly
useful input to it. That is a reason to choose the names carefully, not a
reason to delay.

## Scope

**In:** identifiers on surfaces snora composes —

| Surface | Rendered by |
|---|---|
| menu backdrop (transparent click sink) | `render.rs` |
| modal dim (both click-capturing and not) | `render.rs` |
| dialog card / centred container | `overlay/dialog.rs` |
| sheet panel | `overlay/sheet.rs` |
| toast stack container, and each toast | `toast.rs` |
| skeleton regions (header/sidebar/body/footer slots) | `render.rs` |

**Out:** anything the application supplies. Slot *contents* are the
application's elements and the application's to identify.

## The real cost is not the code

Attaching an `Id` is a line per surface. **Publishing the names is a
stability commitment** — once a downstream test asserts on
`snora-modal-dim`, renaming it breaks that test silently at runtime rather
than at compile time.

So the identifiers must be treated as public API from the outset:

- documented in a reference page listing every identifier snora emits;
- covered by the versioning policy — renaming or removing one is a
  **minor**, not a patch;
- named deliberately, with a stated convention, because the names are the
  contract.

This is the part to get right; the wiring is trivial.

## Design

### Naming

Propose a convention and apply it uniformly — e.g. a `snora-` prefix and
kebab-case surface names (`snora-modal-dim`, `snora-dialog-card`,
`snora-toast-stack`). The prefix matters: it makes snora's identifiers
distinguishable from the application's own, which prevents collisions in a
tree the application also populates.

Per-toast identifiers need a discriminator. `Toast` already carries a
`u64` id; deriving from it (`snora-toast-{id}`) is the obvious choice and
means an application can find a specific toast rather than only the stack.
Confirm the derived form is stable across renders before relying on it.

### Opt-in or always-on?

**Recommended: always-on.** An `Id` on a container has no rendering effect
and no measurable cost, and a feature gate would mean tests pass or fail
depending on feature selection — which is exactly the class of
feature-dependent behaviour the project has avoided elsewhere.

Flag this if implementation shows a real cost. The binary-size probes
(RFC-043, now measuring correctly) can answer it if there is doubt.

## Goals

- G-1. Every surface snora renders itself carries a stable, documented
  identifier.
- G-2. The identifiers are treated as public API with a stated stability
  policy.
- G-3. No change to rendered appearance or behaviour.

## Non-goals

- **N-1. No test harness, no `snora-test` crate.** N-8 is a firm non-goal
  and this RFC does not approach it. Identifiers are not a harness; they
  are labels on output.
- **N-2. No state query API.** The downstream ask mentioned this; it is a
  much larger question about exposing engine internals and is not
  addressed here.
- **N-3. No accessibility semantics.** Roles, names and states belong to an
  accessibility tree — RFC-045, blocked on iced. An `Id` is not a role.
- **N-4. No identifiers on application content.**

## Compatibility

Purely additive; no appearance or behaviour change. `render_semantics` must
pass **unmodified**.

Note one genuine consequence: **from this release the identifiers are a
compatibility surface.** The versioning policy must say so before they
ship, not after — otherwise the first rename will be treated as a patch.

## Testing

| Test | Assertion |
|---|---|
| Identifiers present | Each documented surface emits its documented identifier |
| Names match the reference page | The documented list and the emitted set agree — a drift test, not a spot check |
| Appearance unchanged | `render_semantics` passes unmodified |
| Toast identifiers are stable | The same toast yields the same identifier across renders |

The second test is the one that earns its keep: a reference page listing
identifiers will drift from the code otherwise, and a stale identifier
reference is worse than none — a downstream test would assert on a name
that no longer exists.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Names become a compatibility burden | **Certain** | Medium | Accepted deliberately; stability policy stated before shipping |
| Documented list drifts from emitted names | High | Medium | Drift test above |
| Read as a promise of automated-testing support | Medium | Medium | The guide should say plainly what this does and does not provide: labels, not a harness |
| Encourages testing against internals | Low | Medium | Only composed surfaces are labelled, never internal structure |

## Open questions

- **Q-1.** Naming convention — prefix and case. Recommended `snora-` +
  kebab-case; the implementer should propose and flag rather than assume.
- **Q-2.** Always-on or feature-gated? Recommended always-on; see Design.
- **Q-3.** Should the skeleton region identifiers distinguish *slot* from
  *content wrapper*? A test looking for "the sidebar" probably means the
  region, but the distinction should be deliberate.

## Acceptance criteria

1. Every surface in §Scope emits a documented identifier.
2. A reference page lists them, and a test proves the list matches reality.
3. The versioning policy records identifiers as a compatibility surface.
4. `render_semantics` passes unmodified; no appearance change.
5. Naming convention stated and applied uniformly.

## Release implications

Ships in **0.28.0** with RFC-046. Additive. Its lasting cost is the
stability commitment, which is why the policy change is an acceptance
criterion rather than a follow-up.
