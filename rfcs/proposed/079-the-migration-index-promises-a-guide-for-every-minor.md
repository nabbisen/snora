# RFC 079 — The migration index promises a guide for every minor, and six are missing

**Status.** Proposed
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

**Suggest making the promise true.** It is the only option that removes the
ambiguity rather than documenting it, and the cost is a short guide per additive
minor — which is the case where writing it is cheapest. Whichever wins, **the
other two documents must be amended to agree**, in the same change.

**Q-2 — how far back to backfill?**

**Correction, 2026-08-20:** this RFC's first draft said the 0.29→0.32 gaps
"predate every current adopter's floor." **That is false.** Three adopters have
been on `snora = "0.25"` in recent months, and **knotra went 0.25 → 0.39 this
month** — a jump that crosses all three of those gaps. They are not historical;
they were traversed by a live migration days ago.

**Suggest backfilling all six.** The recent three are unarguable. The
0.29→0.32 span was crossed by a real consumer this month, which is the only
test of relevance that matters here — and a jump from 0.25 is the *worst* case
for the index, because the reader has the most ground to cover and the least
context for inferring what changed.

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
