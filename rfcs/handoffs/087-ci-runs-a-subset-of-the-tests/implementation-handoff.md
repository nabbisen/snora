# Developer Handoff — RFC-087 CI coverage

**Governing RFC.** **RFC-087** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Accepted (owner, 2026-09-01). High.
**Release target.** **0.41.1** — CI and documentation. **No crate code.**
*(Re-targeted 2026-09-01: this said 0.40.2, which is now in the past — 0.41.0
shipped first. **The ordering was wrong and the consequence is on the record:**
this RFC was scheduled to ship before the two Criticals precisely because it is
what should have caught them, and it did not ship at all. It is now the oldest
unstarted item from the audit.)*
**Implementation units.** Three. **Ships first, before the Criticals** — it costs
nothing and it is what should have caught them.

---

## 1. Unit 1 — run the suites that have never run

CI runs `cargo test` for `snora-core`, `snora-design` (**twice** — lines 37 and
120), and `snora` at **default features**. It never runs `snora-widgets` or
`snora-style`.

**Q-1 ruled: `--all-features` for the two missing crates, and `snora` at
`--all-features` too.** Leave the feature-matrix job alone; it answers a
different question and a third mechanism for the same one is how this became
confusing.

**Count them yourself.** Ours today: `snora-widgets` **21**, `snora-style`
**26**, `snora` all-features delta **10** — **57**. The audit says 68. **Use
your number and state it**; the discrepancy is not important, inheriting one is.

**The duplicate `snora-design` invocation:** check whether lines 37 and 120
differ by feature set before touching either. If they are genuinely identical,
remove one and say so; if not, comment why both exist.

## 2. Unit 2 — the three scripts become gates

**Q-2 ruled: the `docs` job.** They are all documentation-consistency checks,
they are fast, and one job answerable for one property is the shape RFC-083
settled.

`check-built-links.py`, `check-version-snippets.sh`,
`check-migration-guides.sh`. **All three pass on a clean tree today — confirm
that before wiring them**, because a gate that ships red is a gate people learn
to ignore, which is the trap RFC-079 was designed around.

Update `scripts/README.md`: they are no longer manual.

## 3. Unit 3 — gate 5, and the thing that generalises

**Gate 5 is marked satisfied in `api-freeze-review.md` and should not be.**
RFC-084 exists because a negative containment assertion would have caught a
Critical on the day the code was written. Correct the row, cite RFC-084, **and
do not re-tick it** — that judgement is the owner's, after 084 lands.

**Q-3 ruled — one line, and it is the only part of this RFC that generalises:**
a deferral must **name the condition that ends it**, and the release checklist
asks whether any recorded condition has been met.

This RFC exists partly because the architect renewed a conditional deferral
three times (RFC-073, 074, 079) without re-checking its condition. **One line in
the checklist; no framework.**

## 4. Explicit non-change scope

- **No new tests.** RFC-084 writes the missing negative assertions. Running what
  exists is this RFC's whole job.
- **No crate code.**
- **Do not re-tick gate 5.**
- **No changes to the feature-matrix job.**

## 5. Required evidence

- CI config diff, and a green run showing the new invocations
- Your own test counts per crate
- The three scripts' output on a clean tree, pre-wiring
- The duplicate-invocation finding, either way

## 6. Acceptance criteria

1. `snora-widgets`, `snora-style` and `snora --all-features` run in CI.
2. Duplicate `snora-design` invocation explained or removed.
3. Three scripts gated in the `docs` job; `scripts/README.md` updated.
4. Gate 5 corrected, not re-ticked, citing RFC-084.
5. Q-3's checklist line exists, one line.
6. Counts re-derived and stated.

## 7. Required review-request format

`.git-exclude/review-request/087-ci-runs-a-subset-of-the-tests/`, `README.md`
entry point, evidence under `evidence/`.

**Requested review focus: the green CI run.** Everything here is configuration;
the only question is whether it actually runs and actually passes.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
