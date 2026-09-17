# Migration 0.49 → 0.50

> **No API change, no feature-resolution change, no MSRV change — but this
> release changes rendered appearance on the default path.** If you keep
> visual baselines that include a sidebar, they are invalidated. Nothing in
> your code needs to change.

## Who is affected

**For code: nobody.** Upgrading is a version bump.

**For appearance: every application that renders `app_side_bar`**, in either
the unstyled or the styled (`design`) variant, in every preset. If you compile
without `snora-widgets` you do not render a sidebar, and this section does not
apply to you.

## 1. The sidebar now renders as it was always meant to

The icon rail is 64 px wide and its buttons are meant to be **48 × 48**. The rail
padded all four sides by 16 px, which left a 32 px content box, so every button
rendered **32 × 48**: a tall, narrow pill instead of a square. Separately, the
icon had no centring container and was drawn at the top-left of the button's
content area, about 10 px left of and 10 px above centre.

**This had been true since at least 0.10.0**, in both variants and every
preset. It is fixed here (RFC-099):

| | Before | After |
|---|---|---|
| Button | 32 × 48 | **48 × 48** |
| Icon offset from button centre | ≈ (−10, −10) px | **(0, 0)** |
| Rail width | 64 px | 64 px — **unchanged** |
| Body content position | — | **unchanged; nothing reflows** |
| Pointer target (WCAG 2.5.8) | 32 × 48 | **48 × 48** |

**What to re-check:** screenshot or pixel-diff baselines that include the
sidebar, and any layout measurements you took of the rail's buttons. **Nothing
outside the rail moves**, because the rail width did not change. That was the
deciding reason for this fix's shape.

**Accessibility records:** both the old and new pointer targets clear WCAG
2.5.8's 24 × 24 minimum. **No conformance status moves.** If your record cites
the sidebar's target size, the figure is now 48 × 48.

**If you worked around it** by building your own rail, you can return to
`app_side_bar`.

**Found by orbok**, whose owner noticed the icons looked off-centre. They traced
it to source and deliberately did not work around it, so that this fix is what
they now see.

## 2. `iced_test` is not CPU-only — relevant if you write simulator tests

Our testing guide used to describe `iced_test` as *"a CPU-only headless
renderer."* **That was wrong.** It tries wgpu first. When several simulators
start at once, as a parallel `cargo test` does, wgpu's start-up was observed to
crash the whole test binary with **SIGSEGV** inside the system Vulkan loader.
Cargo reports that crash with the same exit code as a failing assertion.

If you use `iced_test`, add this to your workspace's `.cargo/config.toml`, as
snora now does:

```toml
[env]
ICED_TEST_BACKEND = "tiny-skia"
```

Two caveats, both in [the testing guide](testing.md): a value already exported
in your shell overrides this line silently, and it depends on `iced_test` 0.14
behaviour.

## 3. Our advisory disclosures describe our graph, not yours

This is now stated in [the threat model](../reference/threat-model.md), so it
does not depend on a letter reaching you. **Your dependency graph is a superset
of snora's.** Enabling more of iced's features, or declaring directly a crate
that reaches snora only through a path we never compile, puts advisories in
your graph that no list of ours can contain. Run a scanner against your own
graph. The reachability judgement that makes an advisory urgent is a property
of that graph.

`anyhow` is the worked case. It is unreachable in snora's graph, and we took
1.0.102 → 1.0.104 in our lockfile anyway. **That does not reach yours.** Check
with `cargo tree -i anyhow --target all`.

## What did not change

- No public API, no feature resolution, no MSRV (still `1.88`).
- **`Spacing` is unchanged.** The fix derives the rail's horizontal padding from
  the rail and button sizes. It does not alter any design token, so the frozen
  design surface (RFC-036) is untouched.
- **`Cargo.lock`:** one third-party package moved, `anyhow` 1.0.102 → 1.0.104,
  plus snora's own crate versions. No package was added or removed.
- **Visual baselines without a sidebar are not invalidated.**

## If you are jumping more than one minor

**0.49** repaired an advisory gate that could not see `unsound` advisories, and
asks you to check `unsound` in your own `cargo-deny` configuration. **0.48**
ruled the non-colour cue question closed. **0.47** enforced
`#![forbid(unsafe_code)]`. **0.45** removed `snora_design::{Emphasis, Size}`.
**0.41** and **0.42** carry real behavioural and dependency changes. The
[migration index](migrations.md) lists them all.
