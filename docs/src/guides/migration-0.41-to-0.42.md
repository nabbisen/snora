# Migration 0.41 → 0.42

> **Rendered appearance change on the default path — no feature needed.**
> The engine's toast colours (`Warning` and `Info`) change — both were
> text that failed WCAG AA and now clears it. The dismiss `×` also no
> longer dims on rest / brightens on hover, on every intent — a
> deliberate margin simplification, not a contrast repair; see below.
> **If you hold reference images or visual-regression baselines that
> include a toast, they are invalidated** for `Warning` and `Info`
> intents, and for the dismiss `×` on every intent.

## Who is affected

Anyone who renders a `snora::Toast` with intent `Warning` or `Info`, or
who shows any toast at all (the dismiss `×`'s hover behaviour changed
for every intent). This is the **engine** path — no `widgets` or
`design` feature required, unlike 0.41.0's widget-layer contrast repair.

If your application never shows toasts, nothing changes for you.

## What changed, and why

**Two of the engine's five toast intents failed their own contrast
requirement, and this was never measured before RFC-086** — the
engine's toasts had no contrast assertion of any kind prior to this
release.

| Intent | Was (text vs. fill) | Measured | Now |
|---|---|---|---|
| `Warning` | white text on `WARNING_COLOR` (an engine-local literal, no `warning` role exists in iced's palette) | **3.18:1** — under AA, matching the audit (F-05) | Black text, same unchanged fill — **6.60:1** |
| `Info` | `primary.base.color`/`.text`, iced's own stock-theme derivation | **4.43:1** in both stock themes — under AA, and not one of the audit's named findings; found by measuring all five intents rather than trusting the two the audit reported | `primary.strong.color` with black text — **5.64:1** |
| `Debug` | `background.strong.color`/`.text` | 13.28:1 (light) / 6.02:1 (dark) — already correct | Unchanged |
| `Success` | `success.base.color`/`.text` | 6.61:1 (light) / 6.91:1 (dark) — already correct | Unchanged |
| `Error` | `danger.base.color`/`.text` | 4.83:1, both themes — already correct, thinner margin than `Success` but clears the floor | Unchanged |

**The dismiss `×` was a separate defect (F-06):** it was hard-coded white
regardless of intent, rather than sharing the toast's own text colour.
For `Debug` (a light-gray fill with black text) this measured **1.58:1**
— effectively invisible. The mark now shares the same colour the body
uses for each intent, so it cannot re-diverge from it.

**The hover/rest fade is gone — a margin choice, not a floor fix.** The
dismiss `×` previously dimmed to 75% opacity at rest and brightened to
100% on hover. With the corrected colours, a 0.75-alpha fade clears the
3.0 floor at every intent and both stock themes (worst case, `Error`:
**3.38:1**). It is removed because doing so raises that worst case to
**4.83:1** and makes the mark's contrast independent of interaction
state. The mark is now fully opaque at every status; hover no longer
changes its own appearance, and nothing else about hover/press
interaction changed.

**Why `Info`'s fill also changed, not only its text.** Neither of
iced's own paired tiers for `primary` (`.base.text` at 4.43:1, or
`.strong.text` at 4.58:1) clears AA with real margin — `primary.base
.color`'s luminance sits almost exactly where black and white text
contrast it equally, so no text colour choice against the original fill
clears comfortably. Widening the fill to `primary.strong.color` was
necessary to reach a defensible margin; both colours are recognizably
the same blue.

## What did not change

- No new `ToastIntent` variant, and no reordering of the existing five.
- No `snora-design` or `snora-style` dependency introduced — the engine
  remains token-free, checked with a standalone contrast
  implementation local to `crates/snora/src/toast.rs`. This is not
  0.41.0's widget-layer contrast repair (RFC-085) restated; different
  crate, different cause — see RFC-086 §"Why this is separate from
  RFC-085".
- `Debug`, `Success`, and `Error`'s colours are unchanged; all three
  already passed.
- No change to toast layout, position, TTL/lifecycle behaviour, or the
  close-sink/dismiss message contract.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
