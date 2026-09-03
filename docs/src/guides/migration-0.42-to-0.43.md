# Migration 0.42 → 0.43

> **Not breaking. Nothing in your application needs to change.** 0.43.0
> ships no behaviour change, no appearance change, and no public API
> change — it is a release of tests, gates, and one documentation
> correction. This guide exists because every minor ships one, *even to
> say nothing is required* (RFC-079), and because the correction below
> is worth five minutes of your attention if you rely on snora's prefab
> toasts or notices for accessibility conformance.

## Who is affected

**For code: nobody.** No public item changed, no rendered surface
changed, and no feature resolution changed. Upgrading is a version bump.

**For accessibility records: anyone relying on `snora::toast` or
`snora_widgets::design::notice` to distinguish semantic variants.** Read
the next section. It does not describe a change in 0.43.0 — it describes
what has always been true, stated plainly for the first time.

## The one thing worth reading: what our prefabs do and do not carry

**snora's prefab surfaces distinguish semantic variants by colour alone.**
A consumer relying on them for WCAG 2.1 SC 1.4.1 (Use of Colour) must
supply a non-colour channel themselves — typically per-variant text,
which most applications already do.

Three surfaces vary by semantic variant, and none of them varies anything
but colour:

| Surface | Varies by | Non-colour channel |
|---|---|---|
| `snora::toast` | `ToastIntent` (5 variants) | **none** |
| `snora_widgets::design::notice` | `Tone` (6 variants) | **none** |
| `snora_widgets::design::progress` | `Tone` | **none** |

**This is a division of labour, not a defect.** The framework supplies
the colour; your call site supplies the words. If each of your notices or
toasts already carries its own title and body — *"Folder could not be
added"* against *"Recent searches cleared"* — the outcome is conveyed by
wording whatever colour it arrives in, and you are already conforming.
That is how every adopting team that checked came back clear.

**What to check on your side.** If you raise a toast or notice whose
*only* per-variant difference is the tone you pass — the same title and
body for success and failure, distinguished by colour — then colour is
the only channel and 1.4.1 is not met at that call site. Adding a word is
usually the whole fix.

**Related, if you have not already acted on it:** 0.41.1 withdrew a claim
that our prefabs distinguish tone by more than colour. If you recorded
that, it came from us and was wrong. This guide is the positive statement
of what replaces it.

### Why we are telling you rather than fixing it

Adding an icon or a textual prefix per intent is a rendered appearance
change on the default path, for every consumer, and every team that
checked is already safe without one. **So it is deferred, not dismissed:
if you want the prefab to carry a non-colour channel itself, say so and
it is reconsidered.** Silence up to now was not evidence of no need —
nobody asks for a channel they have been told already exists.

## What changed, and why

Nothing you compile against. For completeness:

- **A channel register** (RFC-093) asserts, per surface and exhaustively
  over each variant enum, that nothing but colour differs between
  variants. Its job is to fail the day the code and the statement above
  diverge — in either direction.
- **A claim checker** (RFC-092). `scripts/check-docs-only.sh` turns
  *"documentation only"* from a judgement into a command, and a CI gate
  checks any commit carrying a `Docs-only: yes` trailer.
- **A workflow linter.** Workflow files are validated before they are
  pushed rather than only by GitHub afterwards. It found a real
  pre-existing defect in our own binary-size job on its first run.
- **A negative RTL assertion.** Overlay containment was only ever tested
  left-to-right; it is now tested under `LayoutDirection::Rtl` too, which
  closed the last positive-only dimension of our 1.0 render-semantics
  gate.

## What did not change

- No public API. No item added, removed, renamed, or re-signed.
- No rendered appearance. Unlike 0.41.0 and 0.42.0, **no visual-regression
  baseline is invalidated by this release.**
- No feature resolution, no dependency change, no MSRV change.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
including breaking ones in **0.41** (overlay pointer containment) and
**0.42** (toast colours, and the `iced` feature removal). The
[migration index](migrations.md) lists them.
