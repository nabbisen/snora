# Migration 0.44 → 0.45

> **Breaking, but narrow: two unused types removed.**
> `snora_design::{Emphasis, Size}` (and their re-exports through
> `snora::design`) no longer exist. Neither was read by any widget or
> style function in this workspace, and all six teams that adopted
> snora checked their own code and confirmed neither appears there
> either. `cargo build` will name the type directly if we were wrong
> about a seventh consumer.

## Who is affected

**Almost certainly nobody.** If your code does not mention
`snora::design::Emphasis` or `snora::design::Size` (or the
`snora_design` crate's own re-exports of the same names), nothing here
applies to you — upgrading is a version bump.

If you do reference either: `cargo build` fails immediately with a
"cannot find type" or "unresolved import" error naming the type and
the line. There is no deprecation alias bridging this one, for the
reason below.

## What changed, and why

`snora_design::variants` defined four enums: `Tone`, `Density`,
`Emphasis`, `Size`. Checked against the source on 2026-09-02: **`Tone`
and `Density` are read** (`Tone` by `snora_widgets::design::notice` and
`progress`; `Density` as a `Tokens` field). **`Emphasis` and `Size` are
read by nothing** — no widget, no style function, anywhere in the
workspace, for the 24 minors since they shipped in v0.19.

That alone would not be reason enough to remove them mid-covenant — a
type nobody in *our* tree reads could still be load-bearing for a
consumer. So we asked, in a 2026-09-04 letter to all six teams that have
adopted snora, rather than assuming their silence meant agreement (the
same reasoning error RFC-078 made once already, treating a team's
unrelated decline as evidence against an unrelated feature). **All six
replied, and all six confirmed neither type appears anywhere in their
code.** Three went further and enumerated their entire `snora::design`
import surface rather than grepping the two names and reporting a null
— the stronger method, and not one we asked for.

**`Size` had a second problem independent of its disuse: it shadows
`iced::Size`.** The engine uses `iced::Size` heavily
(`responsive.rs`, `design/render.rs`). A consumer writing
`use snora::design::Size` expecting a sizing type would get an inert
four-variant enum instead, with no compiler error to flag the mistake.
Two of the six teams confirmed they would not have noticed the
collision, precisely because they never reach for either name — which
argues for removing it, not renaming it: a collision nobody hits today
is still a collision the next reader hits.

## Why this is a minor bump, not a wait for 1.0

`variants.rs` is part of RFC-036's frozen public surface — removing an
item from it is a **forbidden change** under the additive-only
covenant that otherwise governs `snora-design`. The covenant permits
forbidden changes only through its own reopening condition, paid
explicitly rather than smuggled in: this release resets gates **D-3**
(token model stable for ≥2 consecutive minors) and **D-4** (style
bridge stable for ≥2 consecutive minors) to open in
`docs/src/contributing/api-freeze-review.md`, in the same change that
removes the types. Both are re-earned by holding stable across two
consecutive minors — **0.47.0 at the earliest**.

The reasoning for paying that price now rather than waiting for a 1.0
break: snora is pre-1.0 and has shipped breaking changes in two of the
last four minors already, so this does not add a new kind of
disruption; the design track's D-1/D-2 gates are already open
(coupled to the still-pending iced major upgrade), so this reopening
blocks nothing that was not already blocked; and a public name that
shadows `iced::Size` is cheap to remove now and expensive to remove
after 1.0, where it would need a 2.0 instead of a minor.

## What did not change

- `Tone` and `Density` — both still exist, unchanged, still read by the
  same surfaces as before.
- No rendered appearance changed anywhere. Nothing that reads `Tone` or
  `Density` was touched.
- No deprecation alias for `Emphasis`/`Size` bridges this release —
  unlike some breaking changes, there is nothing meaningful to alias a
  type nobody uses *to*; a compile error naming the type directly is
  more informative than a silently-deprecated no-op would be.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
