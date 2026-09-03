# Migration 0.43 → 0.44

> **Not breaking. Nothing in your application needs to change.** Like
> 0.43.0 before it, this release ships no behaviour change, no appearance
> change, and no public API change. It exists because every minor ships a
> guide *even to say nothing is required* (RFC-079) — and because one
> paragraph below is worth your time if you rely on snora's prefab
> toasts or notices for accessibility conformance.

## Who is affected

**For code: nobody.** No public item changed, no rendered surface
changed, no feature resolution changed. Upgrading is a version bump.

## What changed

Tests, one CI entry, and corrections to our own 1.0 readiness register.

We went through the register asking, of each row that claims something is
tested, *what actually tests it* — and found three of the first seven
claims were wrong or overstated:

- A supported feature combination (`--features design` without
  `widgets`) was documented but covered by no CI job. It works; it is
  now covered.
- `toast::subscription` was listed as tested and had no test.
- The z-stack row claimed the layer *order* was tested when only its
  consequences were. The dialog↔sheet boundary is now directly asserted.

None of this changes what snora does. It changes what we can honestly
say about it, which is the point.

## Still worth reading, if you have not: what our prefabs distinguish

Unchanged from 0.43.0 and repeated because it is the one thing in these
two releases that can affect your own conformance record:

**snora's prefab surfaces distinguish semantic variants by colour
alone.** `snora::toast`, `snora_widgets::design::notice` and
`snora_widgets::design::progress` vary colour and nothing else across
their intents and tones. A consumer relying on them for WCAG 2.1 SC
1.4.1 (Use of Colour) must supply a non-colour channel — typically
per-variant text, which most applications already do.

If a toast or notice of yours differs between variants *only* by the
tone you pass, colour is the only channel at that call site. Adding a
word is usually the whole fix.

**We would rather add the channel ourselves than have you work around
its absence.** It is deferred, not dismissed: say so and it is
reconsidered. Nobody has asked, and until 0.43.0 nobody could have —
the documentation claimed the channel already existed.

## What did not change

- No public API, no rendered appearance, no feature resolution, no MSRV.
- **No visual-regression baseline is invalidated**, for the second
  release running.

## If you are jumping more than one minor

Read the guides in between — **0.41** (overlay pointer containment) and
**0.42** (toast colours, and the `iced` feature removal) both carry real
breaking changes. The [migration index](migrations.md) lists them.
