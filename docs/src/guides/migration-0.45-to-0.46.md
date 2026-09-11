# Migration 0.45 → 0.46

> **Not breaking. Nothing in your application needs to change.** No public
> API change, no appearance change, no feature resolution change.
> This guide exists because every minor ships one, *even to say nothing
> is required* (RFC-079) — and because one small thing in it is
> consumer-visible.

## Who is affected

**For code: nobody.** Upgrading is a version bump.

**If you ever followed an overlay-semantics link out of our published
rustdoc and got nothing, that was us.** Four doc links in
`snora::render` and `snora::keyboard` pointed at `docs.snora.dev`, a
domain that does not exist; the book is published at
`nabbisen.github.io/snora`. Repointed. Nothing else about those items
changed.

## What changed

**The eight overlay laws now carry their own evidence (RFC-096).**
`docs/src/reference/overlay-interaction-semantics.md` states the
engine's behavioural contract — z-stack order, what a missing close sink
does, which layer dominates which. Each law now records whether a test
would actually fail if the engine stopped obeying it, and names that
test where one exists.

Of the eight: **six are guarded**, two state deliberate absences with
nothing to assert (focus trapping is staged, not shipped; keyboard
dismissal is application-owned), and **one gap found during the sweep is
now closed** — a menu with no close sink had never been constructed in
any test, though the modal-side version of that same claim was a shipped
defect in 0.41.0.

**No behaviour moved.** This is the contract document becoming checkable
rather than merely stated, which matters to you only in that the
guarantees you read there are now ones something fails on.

## What did not change

- No public API, no rendered appearance, no feature resolution, no MSRV.
- **No visual-regression baseline is invalidated** — the third release
  running.

## If you are jumping more than one minor

**0.45** removed `snora_design::{Emphasis, Size}` — breaking, though all
six adopting teams confirmed beforehand that neither appeared in their
code. **0.41** and **0.42** carry real behavioural and dependency
changes. The [migration index](migrations.md) lists them all.
