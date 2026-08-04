# RFC 046 — Layout width exposure

**Status.** Proposed
**Tracks.** Engine capability. First half of the responsive theme raised by
the apimokka team (2026-08-04); breakpoint *behaviour* is deliberately not
in scope.
**Touches.** `crates/snora/src/render.rs` or a sibling module,
`crates/snora/src/lib.rs`, `docs/src/guides/`, an example.

## Summary

snora has **no window-size awareness of any kind**. A consumer wanting
breakpoints must write window observation themselves, and the downstream
report is that a team may not notice it is missing until someone audits the
claim — their own responsive-design RFC had gone unimplemented, and the
investigation found neither their application nor snora observed size
anywhere.

This RFC exposes the layout's available width to the application. It does
**not** decide anything about what the application should do with it.

## Motivation

`AppLayout` is an application shell — header, sidebar, body, footer.
Adapting that composition to available width is close to the definition of
what a shell does, and "snora positions and stacks" is the project's own
statement of its job. Width is an input to positioning.

**Verified before writing:** `grep -rniE "responsive|breakpoint|
window::(resize|Event)|available_width" crates/*/src/` returns one doc
comment and no implementation. The gap is real.

## Why exposure and not behaviour

The downstream request had two halves: breakpoint-aware `AppLayout`
behaviour (sidebar auto-collapse), or failing that, width exposure.

**Only exposure is in scope here**, and the distinction is the project's
philosophy rather than caution:

- **Exposure prescribes nothing.** The application decides its own
  thresholds and what changes at them. This is "you supply the content";
  snora supplies the width.
- **Auto-collapse decides for the application** — which threshold, which
  element, what "collapsed" means. snora has consistently declined that
  kind of decision: no theming layer, no form widgets, no prescribed
  layout beyond the skeleton.

The project also has an established pattern for exactly this: anchored
popover and tooltip vocabulary are both **deferred pending a concrete
consuming case**. Here there is a concrete case for exposure and none yet
for prescribed breakpoints — one team asking for behaviour, with no
evidence yet about what thresholds are right.

Behaviour is expected to be revisited. It is not rejected.

## Design

### Verified implementation path

`iced::widget::Responsive` holds `Box<dyn Fn(Size) -> Element>` — a closure
receiving available size and returning an element.

It is reachable as `iced::widget::Responsive` **through the umbrella
crate's glob re-export**, with no additional feature and no additional
dependency. This was confirmed by compiling `use iced::widget::Responsive;`
against `iced = "0.14"` with snora's own feature set — not by reading, since
`iced` never names it and a source search suggests otherwise.

That matters for the implementer: **searching iced's source for
"responsive" finds nothing.** It arrives via `pub use iced_widget::*`.

### Shape

```rust,ignore
/// Render an `AppLayout` that may depend on the available width.
pub fn responsive_render<'a, Message, F>(build: F) -> Element<'a, Message>
where
    F: Fn(f32) -> AppLayout<Element<'a, Message>, Message> + 'a,
    Message: Clone + 'a;
```

The application supplies a closure that builds its layout given the
available width; snora renders the result through the existing z-stack.

`snora::render` is untouched. This is a sibling entry point, exactly as
`snora::design::render` is — a shape the project has now used twice, and
the reason is the same: `AppLayout` cannot carry the input without changing
its own contract.

**Open for the implementer to propose:** whether the closure receives `f32`
width or the full `Size`. Width is what was asked for and is the narrower
contract; height may be equally useful and costs nothing extra. Flag the
choice rather than deciding silently.

### What this deliberately does not add

No `Breakpoint` enum, no thresholds, no collapse behaviour. If snora later
adopts a breakpoint vocabulary it will be *because* consumers reported what
thresholds they converged on — which requires shipping this first.

## Goals

- G-1. An application can obtain the layout's available width without
  writing window observation itself.
- G-2. `snora::render`'s behaviour is unchanged.
- G-3. Nothing about thresholds or adaptive behaviour is prescribed.

## Non-goals

- **N-1. No breakpoint vocabulary or auto-collapse.** Deferred, with the
  trigger being downstream evidence about useful thresholds.
- **N-2. No `Tokens`-style width field on `AppLayout`.** Same reasoning as
  RFC-039: `AppLayout` lives in `snora-core`, and its contract should not
  gain a rendering-time input.
- **N-3. No new dependency.** `Responsive` is reachable through `iced`.
- **N-4. Not feature-gated.** This is engine capability, not design; it
  belongs in the default surface alongside `render`.

## ABDD

Any future breakpoint behaviour must be expressed in logical edges — a
collapsing sidebar collapses on `Edge::Start`, not "the left". This RFC
adds no such behaviour, but the constraint is recorded now so the
follow-up inherits it rather than rediscovering it.

## Compatibility

Purely additive. `snora::render` is untouched; applications that do not
call the new entry point are unaffected. The `render_semantics` suite must
pass **unmodified** — the same invariant RFC-039 established, for the same
reason: it encodes the z-stack contract and this work touches composition.

## Testing

| Test | Assertion |
|---|---|
| Width reaches the closure | A layout built through `responsive_render` receives a plausible width |
| Composition unchanged | The rendered z-stack matches `render`'s for an equivalent layout |
| `render_semantics` | Passes **unmodified** |
| Engine-only | `cargo check -p snora --no-default-features` passes — this is not design-gated |

`iced_test`'s simulator drives the existing render-semantics tests and is
the natural harness. If available width cannot be exercised there, say so
rather than asserting coverage that does not exist — an example that can be
run and observed is an acceptable substitute for that one property.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Two render entry points become three | **Certain** | Medium | Real cost, accepted. Documentation must make the choice obvious rather than presenting three peers; see Open questions |
| The closure shape forces layout rebuilds per frame | Medium | Medium | `Responsive` is iced's own mechanism for this and is used as intended; measure before optimising |
| Consumers expect breakpoints and find only width | Medium | Low | The guide should say plainly that thresholds are the application's, and why |

## Open questions

- **Q-1.** `snora::render`, `snora::design::render`, and now
  `responsive_render` — three entry points. Is a combined
  `responsive_render` + tokens variant needed, or is that a fourth we
  decline? Recommended: decline for now; add only if a consumer needs both
  together. Flagging because entry-point proliferation is a real cost to a
  framework whose value is a small readable surface.
- **Q-2.** `f32` width or full `Size`? See Design.

## Acceptance criteria

1. `snora::responsive_render` exists in the default surface (not
   design-gated) and is documented.
2. `snora::render` is unchanged; `render_semantics` passes unmodified.
3. No new dependency; no `AppLayout` field added.
4. No breakpoint vocabulary or adaptive behaviour is introduced.
5. A guide page and a runnable example show the intended use, and state
   that thresholds are the application's decision.

## Release implications

Ships in **0.28.0** with RFC-047. Additive; first new engine capability
since the appearance milestone, so it warrants the same care RFC-039 got.
