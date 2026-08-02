# RFC 042 — Commit `Cargo.lock`

**Status.** Proposed
**Tracks.** Build reproducibility and measurement integrity. Reverses a
prior deliberate decision (`b7af344`), so it requires explicit owner
approval.
**Touches.** `Cargo.lock` (newly tracked), `.gitignore`,
`docs/src/contributing/architecture.md`, a new scheduled CI workflow,
`docs/src/contributing/release-process.md`.

## Summary

Commit `Cargo.lock`, and add a scheduled CI job that builds with
dependencies deliberately unpinned so the early-warning value of an
unpinned tree is preserved rather than lost.

This reverses `b7af344` ("remove Cargo.lock from vcs"). RFC-035 F-6
confirmed the current untracked state is intentional and correctly
documented, and the owner confirmed it stands *absent a good reason*. This
RFC argues the reason now exists.

## Motivation

Three concrete harms, two of which materialised during the RFC-035/041
review cycle rather than being hypothetical:

**M-1 — Measurement comparability (immediate).** RFC-041 is rebuilding
1.0 gate 9 from zero usable data points. Every budget measurement is taken
in CI, and with no lockfile CI resolves dependencies fresh on every run.
A binary-size or build-cost delta between two releases therefore mixes
snora's own change with whatever upstream published in between. The
budgets exist to track *drift between releases*; unpinned resolution makes
the thing they measure partly noise. The first rebuilt data point lands in
0.25.3 — pinning is worth strictly more before that row than after.

**M-2 — MSRV floor drift (observed).** RFC-041 assumed the floor was 1.85;
verification showed 1.88, forced by `iced 0.14.0` and `wgpu 27.0.1`.
Because `iced = "0.14"` is a caret range and resolution is unpinned, a
patch release of iced can raise snora's effective floor with no change on
snora's side, silently invalidating the `rust-version` declaration RFC-041
is about to add. A lockfile makes that movement an explicit, reviewable
diff instead of a surprise.

**M-3 — Reproducibility of the workspace's own builds.** The workspace
carries 17 example and size-probe binaries. `cargo run -p
snora-example-workbench` producing different dependency versions on two
machines is a debugging cost with no compensating benefit.

## Goals

- G-1. Make this repository's own builds and measurements deterministic.
- G-2. Preserve early warning of upstream breakage.
- G-3. Make dependency movement a reviewable event.

## Non-goals

- **N-1. No effect on downstream consumers is claimed or intended** — see
  Compatibility.
- **N-2. Not a version-pinning policy change.** Dependency *ranges* in
  `Cargo.toml` are unchanged; only resolution within them is recorded.
- **N-3. Not a vendoring proposal.**

## Proposed changes

### C-1 — Track `Cargo.lock`

`git add Cargo.lock`; remove the `Cargo.lock` line from `.gitignore`.

### C-2 — Replace the architecture rationale

`docs/src/contributing/architecture.md` currently has a section titled
"Why no `Cargo.lock` in version control", which is accurate today and
becomes false on adoption. Retitle and rewrite it to record why the
lockfile *is* committed: the workspace is also a measurement harness, and
the budgets require deterministic resolution.

The section must also state the **cost being accepted** (C-3), so the
trade-off is preserved rather than only the conclusion.

### C-3 — Add a scheduled unpinned-build job

This is the part that makes C-1 safe. Committing a lockfile means CI stops
noticing that a fresh resolution has broken. Add a workflow that runs on a
schedule (weekly is sufficient) and on `workflow_dispatch`:

```bash
cargo update
cargo check --workspace --all-features
cargo test -p snora-core -p snora-design
# and verify the declared MSRV still holds against the fresh resolution:
cargo +<declared MSRV> check --workspace --all-features
```

It must **not** commit the updated lockfile — it exists to fail loudly and
tell a human that upstream moved. A failure means "investigate", not
"auto-merge".

The MSRV line is what makes an **inherited** floor rise visible weekly
instead of at release time, which is what the RFC-041 bump policy
(inherited rise → patch) needs in order to be actionable.

This converts an unpinned tree's implicit, continuous early warning into
an explicit, scheduled one — which is strictly better, because today that
warning arrives interleaved with unrelated CI runs and nobody attributes
it.

### C-4 — Release-checklist step

Add: confirm `Cargo.lock` is current and its diff is intentional before
tagging. A lockfile that drifts unreviewed is worse than none, because it
carries an implied assertion that someone looked.

## Compatibility

**Downstream consumers are unaffected, in either direction.** A library's
committed `Cargo.lock` is ignored by Cargo when the library is used as a
dependency; consumers resolve against their own lockfile. This change is
purely about this repository's own builds. Nothing about the published
crates changes.

Version level: **patch**. No crate content changes.

## Security

Mildly positive: a committed lockfile makes dependency changes visible in
review, so an unexpected transitive addition appears as a diff rather than
arriving silently. It does not by itself protect against a compromised
version already in the graph, and no such claim is made. `cargo audit` is
not proposed here.

## Alternatives considered

- **Status quo (untracked).** Preserves continuous early warning and one
  fewer file to review. Rejected: it makes gate 9's data partly noise at
  the exact moment that data is being rebuilt, and it lets the MSRV floor
  move unobserved.
- **Track the lockfile, no scheduled job.** Rejected: trades early warning
  for determinism instead of keeping both. The scheduled job is cheap.
- **Pin exact dependency versions in `Cargo.toml` instead.** Rejected:
  that *does* affect downstream consumers, by over-constraining their
  graphs. The lockfile achieves the same determinism here with none of
  that cost.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Lockfile diffs add review noise | Certain | Low | Usually zero-line; only moves when someone runs `cargo update` |
| Upstream breakage discovered later than today | Medium | Medium | C-3 scheduled job; weekly is well inside a pre-1.0 release cadence |
| Lockfile drifts unreviewed and implies false assurance | Medium | Medium | C-4 checklist step |
| Reverses a deliberate prior decision | Certain | Low | That is why this is an RFC and not an edit |

## Open questions

Both answered; the Developer Handoff carries no open decisions.

- **Q-1 — ANSWERED.** Adopted, targeting **0.25.3**, so the first rebuilt
  budget data point (RFC-041) is measured against pinned dependencies.
- **Q-2 — ANSWERED.** Weekly cadence for the scheduled unpinned-build job.

## Sequencing with RFC-041

These two land in the same release and interact. **The lockfile must be
committed before the MSRV declaration is finalised**, because the declared
floor is only meaningful relative to a specific resolution: if `Cargo.lock`
were committed after the fact, an intervening `cargo update` could move the
floor and re-invalidate the declaration — the exact failure RFC-041 exists
to fix.

Required final state, whatever the order of the intermediate steps:
`Cargo.lock` is tracked, and `cargo +1.88 check --workspace --all-features`
passes **against that exact lockfile**.

The scheduled job (C-3) must therefore also verify the MSRV, so an
*inherited* rise is caught weekly rather than at release time. This is what
operationalises the inherited-rise-is-a-patch policy adopted in RFC-041:
the job tells us the floor moved, and a patch follows.

## Acceptance criteria

1. `Cargo.lock` is tracked; `.gitignore` no longer lists it.
2. `contributing/architecture.md` records why it is committed **and** the
   cost accepted.
3. A scheduled unpinned-build workflow exists, does not commit its
   lockfile, and has been proven to run via `workflow_dispatch`.
4. The release checklist requires confirming the lockfile diff.
5. No dependency range in any `Cargo.toml` changes.

## Release implications

Patch-level; no crate content change. Should land **before** the first
rebuilt budget data point if the measurement rationale (M-1) is to apply
to it, which argues for 0.25.3 alongside RFC-041.
