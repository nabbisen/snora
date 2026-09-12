# Developer Handoff — RFC-097 Unit 2: the supply-chain gate

**Governing RFC.** **RFC-097** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md).
**Status.** Accepted (owner, 2026-09-12).
**Release target.** **0.48.0.**
**Touches.** `.github/workflows/` (new scheduled job), `deny.toml` (new),
`docs/src/contributing/release-process.md` is **not** yours. No crate code.

---

## Unit 1 is done, and it is why this unit is scoped the way it is

`docs/src/reference/threat-model.md` shipped with the acceptance —
model first, mechanism second, per the RFC's own ordering. **Read its
supply-chain section before starting**: it currently says, in the
published documentation, that advisory scanning *does not exist* and
names this RFC as the reason. Your job closes the one paragraph in the
threat model that admits a gap.

## Rulings

**Q-1 — `cargo-deny`, not `cargo-audit`.** `cargo-audit` does advisories
only. `cargo-deny` does advisories **plus licences, duplicate versions and
source allow-lists**, and the audit separately found that **licence
compatibility across 310 packages is unverified** — the same missing
tooling. One mechanism, both answers.

**Start `deny.toml` permissive.** Advisories denied; licences, bans and
sources set to warn. Tightening is a later decision with evidence in hand,
and blocking this adoption on a licence-policy debate would be the
deferral shape this project keeps closing.

**Q-2 — scheduled, failing loudly. Not per-push, and not a `release.yaml`
refusal.** The reasoning is in the RFC and worth reading, because both
halves are counter-intuitive:

- **Not per-push**, because nothing reaches a consumer on a push. A
  per-push failure costs real blocked work for zero consumer protection.
  Same reasoning that put `unpinned-build` on a schedule.
- **Not a hard refusal at release either.** Advisories sometimes have no
  fixed version. A workflow step that cannot weigh severity, reachability
  or the existence of a fix would make snora unreleasable through no fault
  of ours, and would be routed around the first time it was wrong.

So: a scheduled job for standing visibility, and a release-checklist
decision point where a human holds the release with the advisory in front
of them. **The checklist line is the architect's to write** — do not add
it.

**Q-3, Q-4 — both done.** The threat model is in the book; `SECURITY.md`
links it and no longer asks for reproducers this project cannot have.

## The work

A scheduled workflow beside `unpinned-build.yaml`, same shape: `schedule`
plus `workflow_dispatch`, so it can be run on demand during a cut.

Install `cargo-deny` **pinned to an exact version**, the way
`check-workflows.sh` pins `actionlint` — and for the same reason. Run it
across the workspace.

**It must fail loudly on an advisory, and it must fail loudly if
`cargo-deny` cannot be obtained.** An unrunnable scanner reporting green
is the defect this project has now found five times; `check-workflows.sh`
already has the shape to copy — *"this tree is unvalidated, not clean."*

## Required evidence

**Demonstrate the gate failing on a real advisory.** Pin a dependency with
a known RustSec advisory in a scratch branch or worktree, confirm the job
refuses and names the advisory ID and the crate, restore.

**This is the acceptance criterion I care about most.** A security gate
that has only ever been seen to pass is exactly the thing RFC-094 found in
the gate register and RFC-087 found in D-1. Do not ship this one green and
untested.

If reaching a real CI run needs a branch pushed, **ask first** — that call
was right on RFC-090's scratch tag and on RFC-096, and it is right here.

Also demonstrate the unavailable-tool path, the cheaper of the two: make
the install fail and confirm the job refuses rather than passing.

## Acceptance criteria

1. A scheduled workflow runs `cargo-deny` at a pinned version, with
   `workflow_dispatch` so a cut can invoke it.
2. `deny.toml` exists, advisories denied, the rest warning, with the
   permissive start stated in a comment rather than left to look accidental.
3. **Demonstrated failing on a real advisory**, transcript in the review
   package.
4. Demonstrated failing when `cargo-deny` cannot be obtained.
5. Do **not** touch `release-process.md` or `threat-model.md` — both are
   the architect's, and the threat model's supply-chain paragraph gets
   rewritten when this lands.
6. CHANGELOG entry, or one line saying why not. This one is arguable: no
   consumer-observable change, but it is the first security mechanism the
   project has had. Say which and why.
