# RFC 087 — CI runs a subset of the tests, and a conditional deferral has been renewed by habit

**Status.** Done — shipped in v0.41.1 (2026-09-02).
[Handoff](../handoffs/087-ci-runs-a-subset-of-the-tests/implementation-handoff.md).
**D-1 — closed 2026-09-02 (`e0eeda5`).** The migration-guide gate could not
run in CI: `actions/checkout` fetches no tags, the script derived its input
from `git tag --list`, and it died under `pipefail` before its first line of
output — silently red from `c651e93` to `f153a2b`. Fixed by pinning
`fetch-depth: 0` and by making an empty tag list a loud failure rather than a
"no gaps" pass. Proven failing three ways before being called fixed; CI run
`33565992772` green. **D-1 is where RFC-090 came from.**
**Tracks.** CI / measurement integrity. **Severity: High.**
**Found by** the external audit, 2026-09-01 (F-29, F-30, F-39).
**Touches.** `.github/workflows/ci.yaml`,
`docs/src/contributing/api-freeze-review.md`, `scripts/README.md`.
**Release target.** 0.41.1 — CI and documentation. **No crate code.**

## F-29 — two crates' suites have never run in CI

CI runs `cargo test` for `snora-core`, `snora-design` (**twice** — lines 37 and
120), and `snora` **at default features only**. It never runs `snora-widgets` or
`snora-style` at all.

**Counted today, not inherited:** `snora-widgets --all-features` **21**,
`snora-style --all-features` **26**, and the `snora` all-features delta **10** —
**57**, where the audit says 68. The finding holds; the arithmetic is ours to
re-derive, and this RFC uses ours.

`clippy --all-targets --all-features` compiles them, so they cannot rot into a
build error — but **an assertion can start failing silently.** Among the never-run:
RFC-068's two-axis contract, the whole reason that RFC exists.

## F-30 — gate 5 is marked satisfied and the negative assertions do not exist

Every containment test is positive. **RFC-084 exists because a negative one
would have caught a Critical on the day it was written.** Gate 5's status is
wrong and correcting it belongs here, with the gate register, rather than in the
overlay fix.

## F-39 — the deferral expired and I renewed it three times

`check-built-links.py`, `check-version-snippets.sh` and
`check-migration-guides.sh` are manual. I ruled that in RFC-073, RFC-074 and
RFC-079, each time citing RFC-064's precedent: **ship the audit and the written
rule first, so the number is known and stable before a gate is pointed at it.**

**That condition has been met.** All three pass on a clean tree and have for
several releases. The auditor quoted our own `ci.yaml` back at us — *"A gate
nothing runs is not a gate"* — and could not have known the rule. They were
right anyway, because the rule's own condition supplies the answer.

**A conditional deferral that is never re-checked is a permanent one.** That is
the finding, and it is larger than three scripts.

## Non-goals

- **No new tests.** This RFC runs what exists. RFC-084 writes the missing
  negative assertions; do not pre-empt them here.
- **No crate code.**
- **Do not fix the duplicate `snora-design` invocation by deleting one blindly**
  — check whether the two jobs differ by feature set first.

## Open questions

**Q-1 — `--all-features` everywhere, or per-crate feature matrices?** The
feature-matrix job already exists for combinations. **Suggest `--all-features`
for the two missing crates** and leave the matrix alone; adding a third
mechanism for the same question is how this got confusing.

**Q-2 — do the three scripts become gates in the `docs` job, or their own?**
**Suggest the `docs` job** — they are all documentation-consistency checks, they
are fast, and one job answerable for one property is the shape RFC-083 settled.

**Q-3 — what stops the next conditional deferral being renewed by habit?**
**Suggest: a deferral must name the condition that ends it**, and the release
checklist asks whether any recorded condition has been met. One line, and it is
the only part of this RFC that generalises.

## Acceptance criteria

1. CI runs `snora-widgets` and `snora-style`, and `snora` at `--all-features`.
2. The duplicate `snora-design` invocation is understood and either justified in
   a comment or removed.
3. The three scripts run in CI (Q-2), and `scripts/README.md` no longer calls
   them manual.
4. **Gate 5's status corrected** in `api-freeze-review.md`, citing RFC-084.
5. Q-3's mechanism exists, in one line.
6. Test counts re-derived and stated, not copied from this RFC.

## Compatibility and security

**Compatibility.** CI only. **Security.** None.
