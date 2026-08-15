# RFC 056 — Document `snora-style`, then deprecate the `snora-widgets` shims

**Status.** Proposed
**Tracks.** Crate boundaries. Discharges
[RFC-055](../done/055-extract-the-style-bridge.md) Q-4.
**Touches.** `crates/snora-widgets/src/design.rs` and its four style-consuming
modules; `docs/src/design/` (a consumer page for `snora-style`);
`CHANGELOG.md`.
**Release target.** 0.33.0 (minor — new deprecation warnings; no removals).

## Summary

RFC-055 left `snora_widgets::design::style` and `::theme` as compatibility
re-exports and deliberately did **not** deprecate them, because
`snora::design::*` still reached the style layer *through* `snora-widgets` —
warning then would have punished consumers who did nothing wrong.

**That precondition is now met.** `snora::design::style` and
`snora::design::theme` point at `snora_style` directly, done as part of
RFC-055's implementation without being called out.

So the obligation is live: document `snora-style` as a path consumers may use,
then deprecate the shims.

## The finding that shapes this RFC

**`#[deprecated]` on a `pub use` re-export emits no warning.** Tested against
this workspace:

```rust,ignore
#[deprecated(since = "0.33.0", note = "moved to `snora_style::container`")]
pub use snora_style::container;
```

`cargo check -p snora-widgets --features design`, with a forced rebuild and
four internal modules using the path: **zero warnings.** The attribute attaches
to the re-export item; using the re-exported path does not trigger it.

The module form does work:

```rust,ignore
#[deprecated(since = "0.33.0", note = "moved to the `snora-style` crate")]
pub mod style;
```

→ `warning: use of deprecated module 'design::style'`, at every use site.

**So the deprecation must attach to a module *definition*, not to a
re-export.** Marking the individual `pub use` lines — the obvious approach —
would ship a deprecation that warns nobody and would be indistinguishable from
having done nothing.

`snora_widgets::design::theme` is currently `pub use snora_style::theme;`, a
re-export, so it needs wrapping in a local module declaration to carry the
attribute at all.

## The blocker inside our own crate

`snora-widgets`' own modules reach the style layer through the shim:

```text
crates/snora-widgets/src/design/chip.rs:54     use super::style;
crates/snora-widgets/src/design/notice.rs:41   use super::style;
crates/snora-widgets/src/design/card.rs:41     use super::style;
crates/snora-widgets/src/design/button.rs:48   use super::style;
```

Deprecating the module produced **five warnings from our own build**, and CI
runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

**Those four imports must be re-pointed at `snora_style::` before the
deprecation lands**, or the release fails its own gate. This is the step most
likely to be discovered late.

## Design

Three steps, in order. The order is the point.

### 1. Document `snora-style` as a consumer-facing path

Today it appears only in `architecture.md`, `feature-flags.md` and
`design-decisions.md` — all "how snora is built", none "how you use it".

A consumer following a deprecation note to `snora_style::container` needs a
page telling them what the crate is, that depending on it directly is
supported, and that `snora::design::style::*` remains the ordinary route.
**Deprecating before this exists sends people somewhere undocumented.**

### 2. Re-point the four internal imports

Per the blocker above. `snora-widgets` stops consuming its own shim.

### 3. Deprecate, at module granularity

- `#[deprecated]` on `pub mod style;` in `snora-widgets/src/design.rs`.
- Wrap `theme` in a module declaration so the attribute applies.

`since` = the release this ships in. `note` names the replacement path
explicitly — `snora_style::container`, not "the snora-style crate", because a
warning that does not say where to go costs the reader a search.

## Non-goals

- **No removal.** The shims keep working; deprecation is a signal, not a
  break. Removal is a later, major decision.
- **No change to `snora::design::*`.** Consumers on the ordinary path see
  nothing and get no warnings.
- **No deprecation of anything in `snora-style` or `snora`.**
- **No new capability.**

## Open questions

**Q-1 — is a deprecation warranted at all, given nobody is known to use the
shim path?** Both known design-path consumers reach the style layer through
`snora::design::*`, which is not deprecated. The shims exist for a
hypothetical direct user of `snora-widgets`.

The argument for doing it anyway: the standing project policy is to mark
superseded things at the next release, and a compatibility shim nobody marks
is how permanent debt forms. The argument against: warnings with no known
audience are noise, and the same effect could be had with a documentation note
alone.

**Recommend proceeding**, on the grounds that the cost is three lines and the
alternative is a shim with no expiry. But it is a judgement call and the
owner's to make.

**Q-2 — should `since` be the shipping version or the version the move
happened (0.32.0)?** The move was 0.32.0; the warning starts in 0.33.0.
Convention favours the version the deprecation *ships*, so a reader can match
it to a changelog entry that mentions it.

## Acceptance criteria

1. A consumer-facing page for `snora-style` exists before any deprecation
   attribute is added.
2. The four `snora-widgets` internal imports reference `snora_style::`
   directly; `cargo clippy --workspace --all-targets --all-features -- -D
   warnings` passes.
3. Deprecation attributes are on module **definitions**, and a use of
   `snora_widgets::design::style` demonstrably warns — shown in the review
   request, not asserted.
4. `snora::design::style::*` and `snora::design::theme` produce **no**
   warnings; a consumer on the ordinary path is unaffected.
5. Nothing is removed; every existing path still resolves.
6. `render_semantics` passes unmodified.

## Compatibility and security

**Compatibility.** Additive warnings only. No path is removed or moved. A
consumer on `snora::design::*` — which is both known design-path consumers —
sees no change at all.

**Security.** None.
