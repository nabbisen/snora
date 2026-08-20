# RFC 080 — The migration guide and the release letters say the same thing, and only one of them scales

**Status.** Accepted (owner, 2026-08-20). Handoff written — see
[`handoffs/080-…`](../handoffs/080-the-migration-guide-and-the-release-letters-are-the-same-document/implementation-handoff.md).
**Tracks.** Correspondence / release process.
**Found by** the owner, 2026-08-20, asking what RFC-079's rule costs per
release. The guide is not the expensive part.
**Touches.** `docs/src/contributing/release-process.md`, the correspondence
practice itself.
**Release target.** 0.39.1 — process and documentation only.

## Summary

A release is currently written up **four times**: the CHANGELOG entry, the
ROADMAP "Recently shipped" entry, the migration guide, and one letter per
downstream team. The last two say the same thing to the same audience — *here is
what this release means for you* — and only one of them scales.

**Each letter costs the maintainer a hand relay.** Six teams; two have said so
explicitly, and one (tekstide) closed their channel over it.

## The proposal

**The migration guide is the canonical statement of what a release means for a
consumer. A letter carries only what is specific to that team.**

For 0.39.0, that would have been:

| content | today | proposed |
|---|---|---|
| `snora::focus` re-export, what to do | in the guide **and** all three letters | guide |
| the dialog-card rationale withdrawal + re-check | in the guide **and** all three letters | guide |
| "your finding shipped, credited" (arama) | letter | letter |
| "our review's misquote charge was wrong" (knotra) | letter | letter |
| "your Phase 4 owes us nothing here" (orbok) | letter | letter |

Three letters shrink to a pointer plus a paragraph each. Nothing is lost,
because everything removed is in a document they are being pointed at.

## The real argument is not cost

**RFC-067's re-check currently depends on us choosing to write to a team.**

When we withdraw or narrow a claim, the rule says the note must name *what to
re-check*. Today that note lives in a letter — so it reaches the teams we decide
to write to, and nobody else. It does not reach a team we judged below the
correspondence bar, and it does not reach a **future** adopter jumping through
that version, who is precisely the person most likely to be carrying the
withdrawn claim without knowing it.

**Put it in the migration guide and it reaches everyone who reads the guide, on
their own schedule, including people who were not adopters when we withdrew it.**
That is a strictly wider reach than correspondence can achieve, and it is the
reason to do this even if letters were free.

The 0.38 → 0.39 guide already demonstrates the shape: its dialog-card section
carries the re-check line, so a team we never wrote to still gets it.

## Non-goals

- **The correspondence bar does not change.** Broken-now, a withdrawn claim
  a team acted on, or they asked. This RFC changes what a letter *contains*,
  not when one is sent.
- **The CHANGELOG does not change.** It is the record of what changed; the guide
  is the record of what to do. Different jobs, both earned.
- **No change to the ROADMAP's entry.** Whether that is also redundant is a
  separate question and is not asked here.

## Open questions

**Q-1 — does a withdrawn claim still earn a letter once the guide carries it?**
Two readings. Either the guide is sufficient and the bar drops to
broken-now/they-asked; or a withdrawal is exactly the case where we should not
rely on someone reading a guide they may skip.

**Suggest keeping it.** A withdrawal is the case with a known victim — we can
often name the team that acted on the claim — and a pointer costs one line.
But the letter says *"we withdrew X, the guide says what to re-check"*, not the
full argument.

**Q-2 — is a letter with nothing team-specific worth sending at all?** Under
this proposal, a team with no specific content receives only a pointer. **Suggest
not sending it.** A bare "0.39.0 is out, here is the guide" is exactly the note
tekstide asked us to stop sending, and the guide is on the published site
whether we write or not.

**Q-3 — does this belong in the release checklist?** The checklist has a line
for the migration guide and none for correspondence. **Suggest one line: after
the guide is written, ask which teams have something specific — and if none
does, send nothing.**

## Acceptance criteria

1. The practice is written down where release work is done, not only here.
2. The RFC-067 re-check obligation is stated as landing **in the migration
   guide**, with correspondence as an optional pointer.
3. Q-1, Q-2, Q-3 ruled.
4. **No change to the correspondence bar itself.**
5. No code.

## Compatibility and security

**Compatibility.** Process and documentation only. **Security.** None.
