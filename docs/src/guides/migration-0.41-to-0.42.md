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

---

# Also in 0.42.0 — the workspace no longer forces three iced features nobody asked for (RFC-088)

> **Possibly breaking — a narrow, specific case.** If your application (or
> a dependency of it) relied on `iced::widget::canvas` or `iced/tokio`
> arriving *transitively*, through depending on `snora`, that stops
> working. This is the same shape RFC-083 treated as breaking one
> release ago, for the same reason: undocumented and unlikely reliance
> is not reliance that cannot exist.

## Who is affected

Anyone whose own code (or whose other dependencies) uses
`iced::widget::canvas` without separately declaring `iced`'s `canvas`
feature themselves, counting on `snora` to have turned it on. The same
applies to any direct use of `tokio` APIs relying on `snora` having
enabled `iced`'s `tokio` feature as a side effect.

**`svg-icons` users are unaffected.** `snora`'s own `svg-icons` feature
already declares `iced/svg` independently (`crates/snora/Cargo.toml`,
`crates/snora-widgets/Cargo.toml`) — confirmed by building with and
without `svg-icons` and inspecting the resolved dependency tree both
ways.

If you don't use `canvas` directly and don't rely on a `tokio`-specific
API arriving unannounced, nothing changes for you.

## What changed, and why

The workspace's own `[workspace.dependencies]` declaration —

```toml
iced = { version = "0.14", features = ["canvas", "svg", "tokio"] }
```

— forced all three features into every consumer's build, regardless of
whether `snora` (or the consumer) used them:

| Feature | Verified usage | Resolution |
|---|---|---|
| `canvas` | **Zero occurrences** anywhere in `crates/` | Removed |
| `svg` | Only under `snora`'s own `svg-icons` feature, which already declares `iced/svg` independently | Removed from this line — `svg-icons` consumers unaffected |
| `tokio` | **Structurally required**: `snora::toast::subscription` (unconditional, no feature gate) calls `iced::time::every`, which does not exist at all without an executor feature — confirmed by removing it and reproducing `error[E0425]: cannot find function 'every' in module 'iced::time'` | **Kept** |

**Re-derived, not quoted, per this project's own rule that a figure you
did not measure is one you cannot defend:**

- Dependency count (`cargo tree --workspace --all-features`, unique
  packages): **399 → 392** (masked by `--all-features` re-enabling
  `svg` through `svg-icons`). With **default** features — closer to what
  a typical consumer's own build resolves — **397 → 354**, a difference
  of **43 packages**. `Cargo.lock` itself drops exactly the `lyon`
  tessellation family (6 packages) that backed `canvas`.
- Binary size (`snora-size-probe-engine`, stripped, `release-baseline`
  profile): **15,695,200 → 13,777,632 bytes**, a reduction of
  **1,917,568 bytes (1.83 MiB)**. `snora-size-probe-widgets` and
  `snora-size-probe-design` both show the same ~1.83 MiB reduction.
  This matches the external audit's own **−1.83 MiB** figure, measured
  independently rather than inherited from it.

The audit's crate-count figure was **−34**; ours (default features) is
**−43**. Both are real reductions measured by different means at
different times; the discrepancy is not the point — inheriting either
number without re-deriving it would have been.

## Why `tokio` is not also removed

Unlike `canvas` and `svg`, `tokio` is not an unused capability — it
supplies iced's async executor, and `snora`'s own baseline toast
lifecycle support (`subscription()`, used by any application with
transient toasts) requires one to exist. Gating that function behind a
new opt-in `snora` feature would remove the forced choice entirely, but
is a materially larger change than "remove three dead words" and was
not part of this release.

## The general gate this release adds

`scripts/check-workspace-iced-features.sh`, wired into CI
(`design-isolation` job): for every feature the workspace's `iced` line
declares, confirms it is actually used (via
`iced::widget::<feature>`/`widget::<feature>`), except `tokio` — named
and commented as a structural exemption, not silently skipped. This is
the property RFC-083's own gate did not check: not "does a crate depend
on iced" but "does the workspace ask iced for something nothing uses" —
exactly the shape that let this survive RFC-083 by one line. Proven by
perturbation: re-adding `canvas` to the workspace line makes the gate
fail, naming it; restoring the fix makes it pass again.

## If your lockfile shrinks, re-resolve before you trust it

**Reported by a consumer, 2026-09-05, and verified here.** Removing `canvas` and
`svg` shrank one adopter's lockfile by 26 packages — and the smaller graph let
cargo re-resolve a transitive dependency *downward*:

- `gpu-allocator 0.27.0` declares `windows = ">=0.53,<=0.58"` — a loose range
  with an upper bound (confirmed in the published manifest).
- With the other requirements that had been holding it at `0.58` gone, it
  resolved to `0.56` while `wgpu-hal` stayed on `0.58`.
- Two incompatible `ID3D12Heap` types, **ten compile errors, Windows only.**

One command fixes it:

```bash
cargo update -p gpu-allocator
```

**Nothing here is snora's bug** — it is a loose version range in someone else's
crate meeting a smaller graph. But our change is what made the graph smaller, so
any consumer whose lockfile shrinks across this release could hit the same shape
with a different crate.

**And we could not have caught it.** Every snora CI job runs `ubuntu-latest`, and
this failure lives inside a `cfg(windows)` dependency. The reporting team's
Windows job went red while every other gate — theirs and ours — stayed green.
**If you ship for Windows or macOS, your own CI is the only thing that will see
this class of problem.**

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
