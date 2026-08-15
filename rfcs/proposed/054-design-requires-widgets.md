# RFC 054 — `design` requires `widgets`, and the engine surface pays for it

**Status.** Accepted (owner, 2026-08-15) — as an **investigation**: the
deliverable is a measured recommendation among A/B/C, not a chosen option.
**Tracks.** Feature graph and crate boundaries. Answers
[RFC-053](../done/053-responsive-render-excludes-the-design-path.md) Q-1, asked
directly by apimokka.
**Touches.** `crates/snora/Cargo.toml` (features),
`crates/snora/src/design/render.rs`, possibly
`crates/snora-widgets/src/design/style/`, `docs/src/design/feature-flags.md`.
**Release target.** Undecided — see §"Why this may be declined".

## Summary

`design = ["widgets", …]`. A consumer on the design path compiles
`snora-widgets` whether or not they call it — **46 KB** of prefab widget code
that apimokka, with zero `snora::widget::*` call sites, ships and never
executes.

The coupling is not arbitrary. The engine surface (`design::render`,
`design::responsive_render`) genuinely calls into `snora-widgets`, and it does
so **because RFC-039 told it to**. Severing the feature edge therefore either
moves those primitives or reintroduces the duplication RFC-039 refused.

This RFC establishes what the coupling actually is, and asks whether it should
be paid.

## Motivation

apimokka, 2026-08-15:

> **`design` requires `widgets`.** Any consumer on the design path pulls
> `snora-widgets` in whether or not they call it. We have **zero**
> `snora::widget::*` call sites and still compile the crate.
>
> So "engine-only adoption", the thing you have described us as, is true of our
> *call sites* and false of our *build*. Worth correcting in your notes, and
> worth asking whether `design` genuinely needs `widgets` or whether that edge
> is incidental.

They are right on both counts, and the second consequence is worse than the
first: `examples/responsive_body`, shipped in v0.30.0 and built specifically to
match apimokka's architecture, is `default-features = false` and therefore
**cannot be run by them**. The example demonstrates a configuration its
intended reader cannot reach.

## What the coupling actually is

Verified by reading, and narrower than the feature line suggests.

**Most of `snora::design` genuinely is `snora-widgets` re-exported** —
`style`, `theme`, `widget`, `button`, `card`, `notice`, `chip`, `progress`.
That surface cannot be freed and there is no proposal to try.

**The engine surface is different.** `crates/snora/src/design/render.rs`
touches `snora-widgets` in exactly two places:

| Call | What it is |
|---|---|
| `snora_widgets::design::style::color::to_iced_color` | a three-line field-wise `snora_design::Color → iced::Color` conversion |
| `snora_widgets::design::style::container::card_raised` | the RFC-029 card style: fill, border, radius, shadow, all derived from tokens |

Everything else it needs comes from `snora-core`, `snora-design`, `iced`, and
`snora` itself.

**Note the second one carefully.** An earlier reading of this problem counted
only the `use` imports and found one helper; `card_raised` is called
fully-qualified at `design/render.rs:173` and does not appear in the import
list. Do not re-derive this from imports alone.

## Why it is not simply a misplaced helper

`to_iced_color` could move anywhere that depends on `iced`. `card_raised`
cannot be treated the same way, because **reusing it was a deliberate
decision**. RFC-039:

> Reuses `snora::design::style::container::card_raised` (RFC-029) directly —
> the exact same fill/border/radius mapping the card primitive already uses.

`design/render.rs` then takes that style and clears the shadow, with a comment
explaining why the dialog card is border-defined rather than shadow-defined.
The intent is that the dialog card and the card *primitive* cannot drift apart.

So severing the feature edge means choosing one of:

- **move the style primitives** out of `snora-widgets` into somewhere both can
  reach — which is a crate-boundary change, not a feature-flag change;
- **duplicate the mapping** in the engine surface — which is precisely what
  RFC-039 rejected, and would let the dialog card and the card primitive drift;
- **leave it** and document the coupling honestly.

`snora-design` is **not** a candidate home: it is iced-free by hard constraint,
and both functions produce `iced` types.

## What is at stake, measured

`binary-size.csv` at v0.30.0: `widgets_diff_bytes` = **46,336**, against
`design_diff_bytes` = 3,840. A design-path consumer with no widget call sites
pays roughly **46 KB for code they never call** — about twelve times what the
design tokens themselves cost.

Whether 46 KB matters is a judgement this RFC does not pre-empt. It is stated
because the feature-gating criteria (indicator 2) ask exactly this question,
and because "no measurable cost" is not available as an answer.

## Options

**A — A narrower feature.** Something like `design-engine`: tokens plus
`design::render` and `design::responsive_render`, without the prefab widget
surface. Requires the two style functions to live somewhere reachable without
`snora-widgets`, i.e. a crate-boundary change.

**B — Move the iced style bridge to `snora`.** Relocate `to_iced_color` and
the container style functions, with `snora-widgets` re-exporting them so its
own surface is unchanged. Concentrates the bridge in one crate; grows `snora`.

**C — Decline, and document.** State plainly in `feature-flags.md` that the
design path implies the widget crate, why, and what it costs. Cheapest, and
honest — but it leaves `examples/responsive_body` unreachable for its intended
reader and leaves apimokka shipping 46 KB they do not use.

**No option was recommended here.** The measurement and the RFC-039 constraint
were the inputs; the decision was the owner's.

**Decided 2026-08-15: option B′** — extract the style layer into a peer crate.
Tracked as [RFC-055](./055-extract-the-style-bridge.md). The deciding criteria
were **future technical debt** and **documentation cleanliness**, not size:
this RFC's size framing was wrong, and the investigation it commissioned
answered a question that decides nothing. Option B was found to be
structurally unavailable — `snora` depends on `snora-widgets` and not the
reverse, so relocating the bridge into `snora` would be a dependency cycle.

## Why this may be declined

Worth stating up front so the RFC is not read as advocacy.

- 46 KB against a ~15.7 MB binary is **0.3%**.
- Option B moves code across a crate boundary that RFC-034's governance
  treats as stable surface.
- The consumer asking is not blocked: they can and do use the design path
  today. What they cannot do is *stop paying* for widgets, and separately
  cannot run one example.
- The third consumer, orbok, uses prefab widgets **and** `design`. For them
  the coupling costs nothing and any split adds a configuration to support.

The strongest argument for acting is not size. It is that snora currently
cannot express "engine plus design", a configuration two of its three known
consumers arguably want, and that its own v0.30.0 example demonstrates that
configuration while being unable to compile in it.

## Open questions

**Q-1 — Does `card_raised` belong to the widget layer at all?**
It is a container *style*, not a widget. If the style layer is conceptually
below the widget layer, options A and B are both smaller than they look, and
the current placement is the accident. Answer this before pricing anything.

**Q-2 — What would the split cost in maintenance?**
Two feature paths through `snora::design` means two configurations in the CI
feature matrix and two ways for the engine surface to be wrong. RFC-034's
promotion policy applies.

**Q-3 — Is 46 KB the right number?**
It is `widgets_diff_bytes` at v0.30.0, measuring the whole widgets crate. A
design-path consumer might retain part of it through `design::render`'s two
calls even after a split. **Do not assume the saving equals the current diff.**

## Non-goals

- **No change to `snora::design`'s public surface.** Whatever is decided, the
  paths consumers already import keep working.
- **No duplication of the card mapping.** RFC-039's reuse decision stands
  regardless of which option is chosen.
- **No move of anything into `snora-design`.** It is iced-free; that
  constraint is not negotiable here.
- **No change to `snora-widgets`'s own public API.**

## Acceptance criteria

Deliberately thin, because this RFC's output may be a decision not to act.

1. Q-1 answered with reasoning about layering, not only mechanics.
2. Q-3 answered with a **measured** figure for what a design-path consumer
   would actually save, not the current `widgets_diff_bytes`.
3. A decision recorded in `docs/src/contributing/design-decisions.md` with a
   reconsideration trigger, whichever way it goes.
4. `docs/src/design/feature-flags.md` states the coupling and its cost
   explicitly — **required even if the decision is to decline**, because the
   consumer who asked discovered it by reading `Cargo.toml`.

## Compatibility and security

**Compatibility.** Options A and B are additive if done carefully; option C
changes nothing. No existing import path may break under any option.

**Security.** No new data flow, dependency, or integration.
