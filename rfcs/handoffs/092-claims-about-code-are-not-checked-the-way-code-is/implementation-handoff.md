# Developer Handoff — RFC-092 checking claims

**Governing RFC.** **RFC-092** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships.)*
**Status.** Accepted (owner, 2026-09-02).
**Release target.** **0.43.0.** No crate code.
**Implementation units.** One is yours. The prose half is the architect's.

---

## Rulings on the open questions

**Q-2 — both parts adopted, Part 2 in its narrowest form.** The owner accepted
the RFC as written. Part 2 becomes **one sentence** in the existing review and
release documentation — not a new page, not a checklist. The RFC warned that a
half-hearted Part 2 is worse than none; the defence against that is brevity and
placement, not enthusiasm. *If the owner intended Part 1 only, this is the line
to correct.*

**Q-3 — yes, it binds the architect.** Four of the six recorded failures were
the architect's, and two were in review results and a release commit rather than
in implementation. A rule exempting the reviewer would miss most of the observed
instances.

**Q-1 — the script ships as a gate, not as a tool. Ruled against my own
suggestion in the RFC.**

The RFC suggested "script only, no gate," reasoning that CI cannot know which
claims a commit made. That reasoning is wrong, and it is the same reasoning that
kept three scripts manual for three releases until an outside auditor quoted our
own `ci.yaml` back at us (RFC-087, F-39).

CI *can* know, if the claim is written where a machine can read it. See Unit 1.

## Unit 1 — `scripts/check-docs-only.sh`, and the gate that uses it

**The script.** `scripts/check-docs-only.sh <rev>` prints every changed line
under `crates/` in that revision that is not a comment and not blank. Empty
output means the docs-only claim holds; any output refutes it.

This is the filter that found F-33 and F-34 by hand — **after 0.41.1 had already
shipped** claiming documentation-only.

**The gate.** A commit that claims docs-only says so in a trailer:

    Docs-only: yes

A CI job reads the trailer on each commit in the push. If present, it runs the
script on that commit and fails when the output is non-empty, printing the lines
that refute the claim. **No trailer, no check** — this gate has no opinion about
commits that make no claim.

If the trailer approach turns out unworkable in this repo's CI shape, **say so
and propose the alternative** — do not silently fall back to "manual script,"
which is the outcome Q-1's ruling exists to prevent.

## Required evidence

**Demonstrate the script on `2f83e72`** — RFC-089's own commit, whose message
says *"No behaviour change."* It must print the `tab.rs` `match` removal and the
`sheet.rs` `portions()` extraction. A script that reports that commit clean is
not implementing this RFC.

**Then demonstrate the gate failing**, per the standing rule: construct a commit
with a `Docs-only: yes` trailer and a real code change, confirm CI refuses and
names the offending lines; confirm a genuinely docs-only commit with the trailer
passes; confirm a commit with no trailer is unaffected. Three cases, transcripts
in the review package.

A gate that has only ever been seen to pass is the defect this project has now
hit four times.

## Not yours

`docs/src/contributing/` — the claim rules themselves, and where Part 2's
sentence lands. The architect's, same split as RFC-090's Unit 3.

## Acceptance criteria

1. `scripts/check-docs-only.sh` exists and is documented in `scripts/README.md`.
2. Demonstrated on `2f83e72`, printing both F-33 and F-34.
3. The gate is wired, and **all three cases demonstrated** — false claim refused,
   true claim passed, no-claim unaffected.
4. CHANGELOG entry, or one line stating why not. Say which, either way.
