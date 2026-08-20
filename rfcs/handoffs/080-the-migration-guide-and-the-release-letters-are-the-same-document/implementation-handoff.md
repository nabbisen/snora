# Developer Handoff — RFC-080 the guide is canonical, the letter is specific

**Governing RFC.** [RFC-080](../../accepted/080-the-migration-guide-and-the-release-letters-are-the-same-document.md)
**Status.** Inherited from RFC-080 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.1 — process and documentation only. **No code.**
**Implementation units.** One, small. **Ships with RFC-079**, which it depends
on: this practice only works if a guide exists for every minor.

---

## 1. The practice, ruled

**The migration guide is the canonical statement of what a release means for a
consumer. A letter carries only what is specific to that team.**

The reason is reach, not cost. **RFC-067's re-check obligation currently depends
on us choosing to write to a team** — so it reaches the teams we decide to write
to and nobody else, including no future adopter jumping through that version,
who is exactly the person most likely to be carrying a withdrawn claim
unknowingly. In the guide it reaches everyone who reads the guide.

**Write that reasoning down, not just the practice.** A practice without its
reason gets optimised away by the next person who finds it tedious.

## 2. The three questions, ruled

**Q-1 — a withdrawn claim still earns a letter. Keep it.** It is the case with a
known victim: we can usually name the team that acted on the claim, and a
pointer costs one line. **But the letter points; it does not restate.** *"We
withdrew X in 0.39.0 — the migration guide says what to re-check."* The argument
lives in the guide.

**Q-2 — do not send a letter with nothing team-specific in it.** A bare "0.39.0
is out, here is the guide" is precisely the note tekstide asked us to stop
sending, and the guide is on the published site whether we write or not. **No
letter is the correct output** when a team has nothing addressed to them.

**Q-3 — one line in the release checklist**, sited after the migration-guide
line so the two are read in order: *after the guide is written, ask which teams
have something specific — and if none does, send nothing.*

## 3. Where it is written down

- **`contributing/release-process.md`** — Q-3's checklist line, plus a short
  statement of the practice near the migration-guide step.
- **Wherever the RFC-067 re-check obligation is currently stated** — it must now
  say the re-check lands **in the migration guide**, with correspondence as an
  optional pointer. Find every place that states the obligation and make them
  agree; **do not fix only the first one you find.**

## 4. Explicit non-change scope

- **The correspondence bar does not change.** Broken-now, a withdrawn claim a
  team acted on, or they asked. This RFC changes what a letter *contains*, never
  when one is sent.
- **The CHANGELOG does not change.** It records what changed; the guide records
  what to do.
- **The ROADMAP entry is not touched.** Whether it is also redundant is a
  separate question and RFC-080 explicitly does not ask it. **Do not answer it
  here.**
- **Do not write, rewrite, or send any letter.** Correspondence is the
  architect's. The three drafted 0.39.0 letters will be re-cut against this
  practice by the architect, not by this work.

## 5. Required evidence

- The checklist line, in place, quoted
- **Every location stating the RFC-067 re-check obligation, listed**, with each
  one's wording after the change — this is the part that fails by being done
  partially
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py` clean
- `git diff --stat -- crates/` — **expected empty**

## 6. Acceptance criteria

1. The practice **and its reason** are stated where release work is done.
2. The RFC-067 re-check obligation says the guide is where it lands; every
   statement of it agrees.
3. Q-1, Q-2, Q-3 reflected in the checklist wording.
4. The correspondence bar is unchanged — quote it before and after to show it.
5. No letter written or altered. No code.
6. `CHANGELOG.md` `[Unreleased]`.

## 7. Required review-request format

`.git-exclude/review-request/080-the-migration-guide-and-the-release-letters-are-the-same-document/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: the completeness of §3's second bullet.** A re-check
obligation that is stated in two places and updated in one is worse than before,
because the two will be read as offering a choice.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
