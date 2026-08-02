# Versioning policy

Snora follows pre-1.0 SemVer. This page records the rules that govern
how public API changes are versioned, communicated, and bridged.

## Version levels

| Change type | Version level | Migration requirement |
|---|---|---|
| Bug fix, no API change | patch | Changelog note if behavior is visible |
| Additive API (new type, method, or variant) | minor or patch | Docs update |
| Rename public type or method | minor | Deprecation alias for ≥2 minors when practical; migration guide required |
| Remove public type or method | minor (pre-1.0 only) | Migration guide required |
| Feature flag rename | minor | Old feature name bridge if feasible; migration guide required |
| Behavior semantics change (fixes doc invariant) | patch or minor | Explicit changelog note; see rule below |
| Behavior semantics change (changes doc invariant) | minor | Changelog note under **Changed** |
| 1.0+ breaking change | major | Full migration guide |

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
