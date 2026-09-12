# Migration 0.47 → 0.48

> **Not breaking, and this time there is no code in the release at all.**
> No public API change, no appearance change, no feature resolution change,
> no MSRV change, no dependency movement. This guide exists because every
> minor ships one, *even to say nothing is required* (RFC-079).

## Who is affected

**Nobody, for code.** Upgrading is a version bump, and there is nothing to
re-check afterwards. `cargo update -p snora` and carry on.

This release changes documentation and project governance only. It is
listed here because two of those changes answer questions a consumer might
otherwise have to ask us.

## What changed

### The non-colour cue decision is final, not pending

snora's prefab toasts and notices distinguish their semantic variants **by
colour alone**. A consumer relying on them for WCAG 1.4.1 supplies the
non-colour channel themselves — typically per-variant text, which most
applications already write.

That was already true and already documented. What changed is that
`accessibility.md` used to call it a *deferred decision*; it is now
**ruled**, with the reasoning recorded in RFC-093. If you were waiting to
see whether snora would add an icon or a text prefix before designing your
own channel: **stop waiting, and supply your own.** Adding a cue later
stays additive if a requirement ever arrives, so nothing is foreclosed —
but nothing is coming that you should plan around.

**If you expected the prefabs to carry a non-colour channel, tell us.**
That is the one report that reopens the decision, and it would mean our
documentation reached you too late.

### The 1.0 readiness register was swept

Relevant to you only if you track snora's 1.0 timeline. The design-track
gates moved from one of eight satisfied to four of eight, and the core
track stands at eight of ten.

**The more useful half is what `ROADMAP.md` now says at the top:** five of
the six remaining gates wait on things outside this project — an upstream
iced major release, and an adopter reaching production use. The snora-side
work is finished. If you have been reading the gate list as a queue we are
working through, it is not one.

### One documentation defect, found and fixed

`reference/vocabulary.md` opens *"Every public enum in snora-core"* and had
omitted `FocusZone` and `Cycle` since 0.39.0 — a page making a false
completeness claim about itself. It now has a **Zone navigation** section
covering `FocusZone`, `Cycle` and `ZonePresence`. These types were never
undocumented in rustdoc; what was missing was the reference enumeration.

## What did not change

- No public API, no rendered appearance, no feature resolution, no MSRV
  (still `1.88`), and **no third-party dependency moved.** `Cargo.lock`
  does change, but only by snora's own 26 crate versions — checked, not
  assumed: every line of its diff is a `0.47.0` → `0.48.0` bump on a
  workspace member.
- **No visual-regression baseline is invalidated** — the fifth release
  running.

## If you are jumping more than one minor

**0.47** enforced `#![forbid(unsafe_code)]` across all five crates and
adopted dependency advisory scanning, disclosing what its first run found.
**0.45** removed `snora_design::{Emphasis, Size}` — breaking, though all
six adopting teams confirmed beforehand that neither appeared in their
code. **0.41** and **0.42** carry real behavioural and dependency changes.
The [migration index](migrations.md) lists them all.
