# RFC 053 — `responsive_render` excludes the design path

**Status.** Implemented (v0.31.0)
**Tracks.** Engine surface. Corrects a defect in
[RFC-046](../done/046-layout-width-exposure.md), which this author wrote.
**Touches.** `crates/snora/src/design/` (new entry point),
`crates/snora/src/responsive.rs` (documentation only),
`docs/src/guides/responsive.md`, `CHANGELOG.md`.
**Release target.** 0.31.0 (minor — new public API).

## Summary

`snora::responsive_render` hardcodes `crate::render::render`. A consumer on the
`design` path who adopts it **loses the design chrome** — the dialog card and
the token-derived modal dim.

So responsive layout and design chrome are **mutually exclusive as shipped**.
This RFC adds `snora::design::responsive_render`, the missing pair.

## Motivation

Reported by apimokka, 2026-08-15, and verified against source:

```rust,ignore
// crates/snora/src/responsive.rs:76
Responsive::new(move |size: Size| {
    let layout = build(size.width);
    crate::render::render(layout)      // ← engine renderer, unconditionally
})
```

There is no `design::responsive_render`; `grep responsive crates/snora/src/design/`
returns nothing. `render_with_style` — the shared composition path — is
`pub(crate)`, so `design::render` is the only design entry point and it takes a
whole `AppLayout`, not a width.

**The consequence for the reporting consumer is specific and severe.** Their
0.28 adoption existed to deliver an accessibility fix: the `high_contrast_dark`
modal dim, which before RFC-039 composited 40% black over pure black and was
invisible. Adopting `responsive_render` would regress exactly that.

### It blocks the evidence RFC-046 deferred behaviour on

RFC-046 shipped width exposure and **declined** to ship breakpoint behaviour,
on the stated grounds that snora does not yet know which thresholds real
applications converge on — and that shipping exposure is what makes that
evidence gatherable.

The consumer who asked for it reports their responsive spec cannot be
implemented today without regressing accessibility. **The deferral is blocked
on the gap**, so this is not only a missing convenience.

### And RFC-051's example demonstrates an unreachable configuration

`examples/responsive_body` (v0.30.0) is `default-features = false`, built to
match that consumer's architecture. They cannot run it: `design = ["widgets",
…]`, so any design-path consumer compiles `snora-widgets` regardless of call
sites. See Q-1.

## Design

```rust,ignore
// crates/snora/src/design/, gated as `design` alongside design::render
pub fn responsive_render<'a, Message, F>(
    build: F,
    tokens: &'a Tokens,
) -> Element<'a, Message>
where
    F: Fn(f32) -> AppLayout<Element<'a, Message>, Message> + 'a,
    Message: Clone + 'a,
{
    Responsive::new(move |size: Size| {
        crate::design::render::render(build(size.width), tokens)
    })
    .into()
}
```

**`&'a Tokens`, matching `design::render(layout, &tokens)`.** Consistency
between the two design entry points matters more than avoiding the borrow, and
the borrow is natural in the usual `fn view(&self) -> Element<'_, Message>`,
where the returned element already borrows `&self`. `Tokens` is `Clone` and
documented as cheap to clone, so a by-value variant remains available later
without breaking this one.

**It must not duplicate composition.** RFC-039 extracted the shared path for
exactly this reason, and RFC-046 already carried "do not duplicate the z-stack"
as a prohibited shortcut. This function is a wrapper: `Responsive` around the
existing `design::render`. If the implementation grows beyond that, something
is wrong.

### Also: document the engine one as engine-path-only

`snora::responsive_render`'s own documentation does not say it renders through
the engine path. That silence is how this was found — by reading the source,
not the docs. It gains one sentence and a pointer to the design pair.

## Non-goals

- **No `Breakpoint` type, thresholds, or adaptive behaviour.** Unchanged from
  RFC-046. This RFC removes a blocker to gathering that evidence; it does not
  pre-empt it.
- **No change to `snora::responsive_render`'s signature or behaviour.**
  Documentation only.
- **No change to `design::render`.**
- **No new composition path.** Wrapper only.
- **No making `render_with_style` public.** The two entry points are the
  surface; the shared internal stays internal.

## Open questions

**Q-1 — Does `design` genuinely require `widgets`?**
`design = ["widgets", "dep:snora-design", "snora-widgets?/design"]`, and the
manifest comment says enabling `design` without `widgets` "is a no-op except
for the snora-design token types". apimokka asks directly whether that edge is
incidental or load-bearing.

If severable, **engine-only-plus-design becomes a reachable configuration** —
which would make `examples/responsive_body` adoptable by the consumer it was
built for, and would let a design-path consumer with zero widget call sites
stop compiling `snora-widgets`.

This is a feature-graph change with binary-size consequences and is **not in
scope here**. **Now tracked as [RFC-054](../proposed/054-design-requires-widgets.md)**,
which establishes that the coupling is two style functions in
`design/render.rs`, one of them reused on RFC-039's explicit instruction.

**Q-2 — Should this ship with an example?**
`examples/responsive_body` cannot demonstrate it (Q-1). Adding a design-path
responsive example means a third responsive example, which RFC-051 explicitly
declined. Suggest documenting in the guide first and adding an example only if
a consumer reports the docs insufficient.

## Acceptance criteria

1. `snora::design::responsive_render` exists, `design`-gated, wrapping
   `design::render` without duplicating composition.
2. `snora::responsive_render` documentation states it renders through the
   engine path and points at the design pair. Signature and behaviour
   unchanged.
3. `docs/src/guides/responsive.md` states which entry point serves which path,
   and that mixing them was previously impossible.
4. `cargo test -p snora --test render_semantics` passes **unmodified**.
5. `cargo check -p snora --no-default-features` still passes — the new function
   must not leak out of the `design` gate.
6. A test asserts the design path's chrome survives through
   `design::responsive_render` — i.e. that it is not silently rendering through
   the engine path. Presence of the width is not sufficient; the defect being
   fixed is exactly "renders, but through the wrong path".

## Compatibility and security

**Compatibility.** Purely additive. New `design`-gated function; no existing
signature changes. Consumers not on `design` are unaffected.

**Security.** No new data flow, dependency, or integration.

## Release implications

**0.31.0, minor** — new public API. Migration guide not required (nothing
breaks), but the responsive guide change is the discoverability half and is an
acceptance criterion rather than a follow-up. `CHANGELOG.md` under **Added**,
naming the gap it closes and crediting apimokka.
