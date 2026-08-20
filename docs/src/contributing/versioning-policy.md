# Versioning policy

Snora follows pre-1.0 SemVer. This page records the rules that govern
how public API changes are versioned, communicated, and bridged.

## Version levels

**Every minor release ships a migration guide. No condition** (RFC-079)
— including a release that only adds API, where the guide says plainly
that nothing is required. This is not one requirement among several in
the table below; it is unconditional at the minor level, and the table's
per-row requirements are *additional* to it (a deprecation alias, a
feature bridge), not a gate on whether a guide is written at all.

| Change type | Version level | Migration requirement |
|---|---|---|
| Bug fix, no API change | patch | Changelog note if behavior is visible |
| Additive API (new type, method, or variant) | minor or patch | Migration guide required when minor (RFC-079 — every minor ships one, even to say nothing is required); changelog note if patch |
| Rename public type or method | minor | Migration guide required; deprecation alias for ≥2 minors when practical |
| Remove public type or method | minor (pre-1.0 only) | Migration guide required |
| Feature flag rename | minor | Migration guide required; old feature name bridge if feasible |
| Behavior semantics change (fixes doc invariant) | patch or minor | Migration guide required when minor; explicit changelog note always — see rule below |
| Behavior semantics change (changes doc invariant) | minor | Migration guide required; changelog note under **Changed** |
| 1.0+ breaking change | major | Full migration guide |

## Rendered surface identifiers

The `iced::widget::Id`s snora attaches to the surfaces it renders itself
(RFC-047; see the [reference page](../reference/rendered-surface-identifiers.md)
for the full list) are a compatibility surface, effective from the
release that ships them, even though they are not `pub` Rust API in the
usual sense — nothing about them appears in a type signature or shows up
in `cargo doc`.

**Renaming or removing a rendered-surface identifier is a minor bump,
not a patch.** The reasoning is the same as any other public API rename:
once a downstream test asserts on `snora-modal-dim`, changing that string
breaks the assertion — the difference is that the break is **silent at
runtime** rather than caught at compile time the way a Rust API rename
would be, which if anything argues for treating it *more* carefully, not
less. Adding a new identifier is additive (patch-or-minor, per the table
above); renaming or removing an existing one requires the same migration-
guide discipline as renaming a public type.

**Worked example: RFC-049, v0.29.0** — the first rename exercised under
this rule. `snora-dialog-card` had been attached to the wrong element
since v0.27.0 (the dialog's full-window centring container, not the
actual card). The fix split it: the centring container is now
`snora-dialog`, and `snora-dialog-card` was **re-pointed**, not retired,
to the element it should have named from the start — see the
[reference page](../reference/rendered-surface-identifiers.md#static-identifiers)
and the [migration guide](../guides/migration-0.28-to-0.29.md).

This is a **minor**, per the table above, but it does not fit the
[deprecation bridge](#deprecation-bridges) mechanism cleanly: these
identifiers are plain strings, not Rust symbols, so there is no
`#[deprecated]` to attach — a downstream test asserting on the old name
does not get a compiler warning, and does not fail at all after the
upgrade. It silently starts resolving the actual card instead of the
window. That silent-repurposing risk was accepted for this one release
only because no known consumer had adopted 0.28.0 identifiers yet
(checked against the two known integrations at the time). A future
rename of a rendered-surface identifier with known adopters on the old
name should retire the string rather than repurpose it, precisely to
avoid this risk.

**The premise was subsequently confirmed, and has since expired.** It was
an inference when v0.29.0 shipped. Days later a downstream team reported
independently that the v0.28.0 identifiers were inert for them — no
`iced_test`, no `widget::Id` usage, nothing walking or snapshotting a
widget tree — which is the strongest form this check can take: a
statement from the adopter rather than an assessment of them.

Their report also said they had scheduled work to build scripted
verification **against** these identifiers — which would have ended the
premise. **They then corrected that**: their verification drives the
application from a separate process over compositor IPC, and an
`iced::widget::Id` lives inside iced's widget tree, never surfaced to the
compositor, to X11, or to any accessibility API. It is invisible to them.
The task is not scheduled, and would only return if they adopted an
in-process harness such as `iced_test`.

So the premise holds longer than either side first thought. Two lessons
worth more than the conclusion:

- **`Id`s serve in-process harnesses only.** Any consumer driving snora
  externally gets nothing from them. That bounds who the compatibility
  surface is *for*, and it is a narrower group than "anyone writing GUI
  tests" — see [`guides/testing.md`](../guides/testing.md), which is what
  led them to the correction.
- **Ask the adopter; do not infer from their last reported version.** Both
  the original premise and its apparent expiry were inferences, and the
  expiry was wrong. Before any further rename, ask the known integrations
  directly what they assert on.

## MSRV bump policy

Snora declares `rust-version` in `[workspace.package]` (adopted in RFC-041,
after verification showed the documented floor was wrong by three point
releases — see `contributing/architecture.md`'s `Cargo.lock` rationale for
how that happened). Raising it is versioned by *why* it rises, not just
that it does:

| Case | Level | Reasoning |
|---|---|---|
| **Inherited** rise — a dependency (`iced`, `wgpu`, …) raises its own `rust-version`, forcing snora's floor up with it | **patch** | Snora controls neither the timing nor the value; it is reporting a floor already imposed on it, not making a design decision. With `rust-version` declared, cargo's MSRV-aware resolver (resolver `"3"`) keeps users on older toolchains at the last compatible snora version rather than breaking them outright. |
| **Chosen** rise — snora adopts a newer language edition or toolchain feature by choice | **minor** | This is a decision snora makes, with alternatives (stay on the older feature set). It narrows who can currently build snora by snora's own choice, which is a minor-level compatibility decision like any other. |

Either way, the release checklist requires re-verifying the declared value
against the committed lockfile before tagging (see `release-process.md`) —
an MSRV that isn't re-checked at release time is a claim, not a fact.

## "Fixed" vs "Changed" rule

A fix that makes behavior match *already-published* documentation is
**Fixed**, not **Changed**. The canonical example: the v0.11 toast
ordering fix restored the documented `ToastPosition` invariant. It was
recorded as **Fixed** because the contract did not change — the code was
brought back into line.

A change that updates the documented contract is **Changed**, even if it
is an improvement. This distinction matters because "Fixed" tells
downstream users "code that followed the docs was already correct."

## Deprecation bridges

When a public name is renamed:

1. Add a `#[deprecated]` alias in the same PR.
2. Keep the alias for **at least two** consecutive minor releases.
3. Remove the alias in a minor release, citing the minor where it was
   added and the current minor.

"At least two" gives downstream projects that move slowly one full
release cycle to migrate before the alias disappears.

## Questions any PR must answer when touching public API

1. Is this a change to public API (`snora-core/src/lib.rs`,
   `snora/src/lib.rs`, or any `pub` item in those trees)?
2. Is this additive, a rename, a removal, or a semantic change?
3. Does it need a deprecation bridge?
4. Does it need a migration guide?

Add the answers in the PR description. Leave them blank only for
documentation-only or internal changes.

## Public API diff (deferred)

`cargo-semver-checks` is a reasonable candidate for automated API diff
before 1.0. Deferred until a downstream app provides a baseline worth
checking. When adopted, add it to the CI workflow.

## Changelog labels

Use these headers, consistent with Keep a Changelog:

- **Added** — new public API, new examples, new docs pages.
- **Changed** — changes to existing behavior or docs (not bug fixes).
- **Deprecated** — items that will be removed in a future release.
- **Removed** — items removed after their deprecation window.
- **Fixed** — bug fixes; behavior brought back into line with docs.

Do not put breaking changes under **Fixed**. Breaking changes go under
**Changed** even if they are improvements.
