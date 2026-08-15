# RFC 056 — Remove the `snora-widgets` style shims

**Status.** Implemented (v0.33.0)
**Tracks.** Crate boundaries. Discharges
[RFC-055](./055-extract-the-style-bridge.md) Q-4.
**Touches.** `crates/snora-widgets/src/design.rs` and its four style-consuming
modules; `CHANGELOG.md`; `docs/src/guides/migration-0.32-to-0.33.md` (new).
**Release target.** 0.33.0 (minor — a public path is removed from
`snora-widgets`).

## Summary

RFC-055 relocated the style layer to `snora-style` and kept
`snora_widgets::design::{style, theme}` as compatibility re-exports so nothing
broke during the move. Its precondition for retiring them is now met:
`snora::design::style` and `::theme` point at `snora_style` directly.

**Remove the shims rather than deprecating them.**

## Why removal, not deprecation

Deprecation was the original plan (RFC-055 Q-4). Removal is better here, for
three reasons.

**1. The deprecation would not work the obvious way.** Tested against this
workspace: `#[deprecated]` on a `pub use` re-export emits **no warning at all**
— forced rebuild, four internal modules using the path, zero output. It only
works when attached to a module *definition*, which means `theme` — a bare
re-export — would need wrapping in a local module purely to carry an attribute.
Machinery in service of a warning.

**2. The audience is the same either way, and a compile error serves it
better.** No current documentation directs anyone to depend on `snora-widgets`
directly; `architecture.md` describes it as an optional crate consumed through
`snora`, and both known design-path consumers use `snora::design::*`. So the
audience for a warning is hypothetical — and a hypothetical user is exactly
who benefits from an unmissable error over a warning they may never read.

**3. It leaves nothing behind.** No shim, no caveat in the docs, no
deprecation cycle to track and eventually close. RFC-055 already cost the
project a permanent explanation once; this avoids a second.

The relocate-then-remove sequence is also shorter and clearer than
relocate-then-deprecate-then-remove: 0.32.0 moved the layer without breaking
anything, 0.33.0 removes the vestige.

## What is removed

| Path | Replacement |
|---|---|
| `snora_widgets::design::style` (and its five submodules) | `snora_style::{button, color, container, progress, text}` |
| `snora_widgets::design::theme` | `snora_style::theme` |

**`snora::design::style::*` and `snora::design::theme` are unaffected** — they
point at `snora-style` directly and are the documented consumer route. Neither
known design-path consumer sees any change.

## The blocker inside our own crate

`snora-widgets` consumes its own shim:

```text
crates/snora-widgets/src/design/chip.rs:54     use super::style;
crates/snora-widgets/src/design/notice.rs:41   use super::style;
crates/snora-widgets/src/design/card.rs:41     use super::style;
crates/snora-widgets/src/design/button.rs:48   use super::style;
```

Removing the module breaks all four. **Re-point them at `snora_style::` first.**
This was found by deprecating the module experimentally, which produced five
warnings from our own build — the same four sites plus one.

## The risk, stated

`snora_widgets::design::style` is not a new path. It predates RFC-055 and was
the *real* location of the style layer, so removing it removes something
long-standing rather than a one-release-old shim.

The mitigation is that nothing currently documents `snora-widgets` as a direct
dependency. The one historical exception is
`docs/src/guides/migration-0.5-to-0.6.md:142`, which shows
`snora-widgets = "0.6"` — a nine-minor-old instruction. If a consumer followed
it and reached the style layer directly, this breaks them at compile time with
a clear replacement path, which is the intended behaviour rather than a
surprise.

## Non-goals

- **No change to `snora::design::*`.** The consumer route is untouched.
- **No removal from `snora-style`.** Nothing there is retired.
- **No deprecation cycle.** Removal is the alternative to one, not a step
  after it.
- **No change to `snora-widgets`' widget surface** — `widget`, `button`,
  `card`, `notice`, `chip`, `progress` are unaffected.

## Open questions

**Q-1 — is a migration guide warranted?** snora's convention attaches guides to
releases that break or rename something. This one does, so: yes, and it should
be short — a two-row table of old path to new. It is the first guide since
0.29.0.

**Q-2 — does `snora-widgets`' own `design` feature still make sense?** After
removal it gates the prefab *design widgets* only, not the style layer. Its
manifest comment still says it "exposes the Snora Design style bridge", which
will be false. Correct the comment; do not change the feature.

## Acceptance criteria

1. `snora_widgets::design::{style, theme}` no longer exist.
2. The four internal imports reference `snora_style::` directly, and
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   passes.
3. `snora::design::style::*` and `snora::design::theme` resolve unchanged, and
   a design-path consumer sees no difference — demonstrated, not asserted.
4. A migration guide exists with the old→new path table (Q-1).
5. `snora-widgets`' `design` feature comment no longer claims to expose the
   style bridge (Q-2).
6. `render_semantics` passes unmodified.

## Compatibility and security

**Compatibility.** **Breaking for direct `snora-widgets` consumers only**, of
which none are known. Consumers of `snora` — the documented route — are
unaffected. Pre-1.0, and consistent with the versioning policy's treatment of
public-item removal.

**Security.** None.
