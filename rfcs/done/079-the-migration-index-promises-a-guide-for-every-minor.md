# RFC 079 — The migration index promises a guide for every minor, and six are missing

**Status.** Done — shipped in v0.39.1 (2026-08-20).
[Handoff](../handoffs/079-the-migration-index-promises-a-guide-for-every-minor/implementation-handoff.md).
**Tracks.** Documentation integrity / release process.
**Found by** the owner, 2026-08-20 — *"the docs lack migration about 0.38.0 to
0.39.0."* The audit that followed found five more.
**Touches.** `docs/src/guides/migrations.md`,
`docs/src/contributing/release-process.md`,
`docs/src/contributing/versioning-policy.md`, `docs/src/guides/`.
**Release target.** 0.39.1 — documentation only.

## Summary

`docs/src/guides/migrations.md` — the page a consumer lands on when planning a
bump — states:

> **Each minor release ships a focused migration guide** describing exactly what
> to change and why.

**Six minors have none:**

| jump | guide |
|---|---|
| 0.29 → 0.30 | **missing** |
| 0.30 → 0.31 | **missing** |
| 0.31 → 0.32 | **missing** |
| 0.34 → 0.35 | **missing** |
| 0.37 → 0.38 | **missing** |
| 0.38 → 0.39 | **missing** |

The last two are mine, from this month, and I made them the same way twice: I
applied `versioning-policy.md`'s rule and never checked what `migrations.md`
promised.

## Three documents, three different rules

| source | says |
|---|---|
| `guides/migrations.md:5` | **every** minor ships a guide — unconditional, and consumer-facing |
| `contributing/release-process.md:82` | `[ ] Update docs/guides/migration-X.Y-to-X.Z.md (minor only)` — reads as unconditional for minors |
| `contributing/versioning-policy.md` | additive API → *"Docs update"*; a guide is **required** only for rename, removal, or feature-flag rename |

I resolved that conflict silently, in favour of the most permissive of the
three, on 0.38.0 and again on 0.39.0. **A conflict resolved by whoever is
cutting the release is not a policy.**

## Why the gaps matter more than they look

**Absence is currently ambiguous.** A missing guide means either "nothing was
needed" or "nobody wrote it", and a consumer cannot tell which. That is the
worst property a documentation index can have, and it is exactly why the
promise being false costs more than the guides would have.

**Multi-version jumps are the normal case here, not the exception.** knotra went
**0.25 → 0.39** this month; knotra and aaai went 0.25 → 0.37 before that. A jump
of that length crosses several of these gaps, and the index is the artefact
those teams use to plan.

**A no-op guide is not a no-op.** *"0.38 → 0.39: nothing required. `snora::focus`
is new and optional."* answers the question a consumer actually has — **is this
bump safe?** — which today they can only answer by reading the CHANGELOG and
inferring.

## Open questions

**Q-1 — which rule wins?** Three options: make the promise true (a guide per
minor, short where nothing broke); weaken `migrations.md` to match
`versioning-policy.md`; or keep both and have the index say explicitly which
jumps need no guide.

**Recommendation, third and final form — every minor gets a guide. No
condition.**

Two earlier answers are recorded here because both were wrong and the way they
were wrong is the point.

**First answer: a guide per minor.** Right rule, weak argument.

**Second answer: a guide only where a consumer "has something to do or know",
an index line otherwise.** The owner rejected this, and correctly: *"I don't
like complicated conditions attached to the rules."*

It reads as one rule and behaves as a judgement call. **"Something to do or
know" is vaguer than "something broke", not sharper** — it has to be decided per
release, by whoever is cutting, which is precisely how the original defect
happened: three documents disagreed, and I resolved the conflict twice by
judgement, silently. A rule whose application needs a decision reproduces that
failure in a new coat.

**And my argument against the simple rule was based on a false analogy.** I
objected that nineteen no-op guides are nineteen artifacts that can go stale, in
a project that had just spent a week deleting stale artifacts. **Migration
guides do not rot.** They describe a transition between two fixed versions and
are written once — verified: `migration-0.32-to-0.33`, `-0.33-to-0.34` and
`-0.35-to-0.36` have **exactly one commit each**, and `-0.36-to-0.37` has two.
Unlike a reference page, there is nothing in them for reality to drift away
from. The cost is a short file, once, and then nothing.

So: **every minor release ships a guide. A guide for a minor that broke nothing
says so in a sentence.** No conditions, no per-release judgement, and Q-3's
check becomes trivially exact — *for every minor tag, a file exists* — with
nothing for anyone to interpret.

`migrations.md`'s existing promise is already this rule stated correctly. **It
does not change; the other two documents are amended to match it**, and the
practice is brought up to what the page has said all along.

**Q-2 — how far back to backfill?**

**Correction, 2026-08-20:** this RFC's first draft said the 0.29→0.32 gaps
"predate every current adopter's floor." **That is false.** Three adopters have
been on `snora = "0.25"` in recent months, and **knotra went 0.25 → 0.39 this
month** — a jump that crosses all three of those gaps. They are not historical;
they were traversed by a live migration days ago.

**Ruled by the owner, 2026-08-20: 0.38 → 0.39 only, now.** It is needed to
announce 0.39.0 to the app teams, and the rest does not block that.

**Written and shipped ahead of this RFC** —
`docs/src/guides/migration-0.38-to-0.39.md`, indexed in `migrations.md` and
`SUMMARY.md`, built and link-checked. It is the worked example of the Q-1
recommendation: nothing is required, and it still has two things worth a
consumer's attention.

The remaining five gaps stay open under this RFC. The 0.29→0.32 span was
crossed by a real consumer this month (knotra, 0.25 → 0.39), so it is not
historical — but it is also not urgent, and nobody is blocked.

**Q-3 — what stops the seventh?** The checklist line has been there throughout
and did not fire, because ticking a checkbox produces no artefact. A check —
*for every released minor tag, a guide exists* — is trivially derivable from
`git tag` and the filesystem, and would have caught all six.
**Suggest the same shape as `check-version-snippets.sh` and
`check-built-links.py`: committed, runnable, inventoried, not a CI gate.**

## Acceptance criteria

1. Q-1 ruled; **all three documents agree afterwards.**
2. Guides exist for the jumps Q-2 selects; any remaining gap is **named in the
   index**, not left silent.
3. A guide for an additive minor says plainly that nothing is required — it does
   not pad to look substantial.
4. Q-3's check exists, is runnable, and its output on the current tree is stated.
5. **A perturbation demo:** delete a guide, run the check, see it named; restore.
6. No code change.

## Compatibility and security

**Compatibility.** Documentation only. **Security.** None.
