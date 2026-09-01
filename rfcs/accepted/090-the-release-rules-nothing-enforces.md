# RFC 090 — The release rules nothing enforces

**Status.** Accepted (owner, 2026-09-02). Handoff written — see
[`handoffs/090-…`](../handoffs/090-the-release-rules-nothing-enforces/implementation-handoff.md).
**Tracks.** Release integrity / process. **Severity: High.**
**Found by** the 0.41.1 cut, 2026-09-02 — three of this document's own rules
broken in one release by the person who wrote them.
**Touches.** `.github/workflows/` (new `release.yaml`),
`docs/src/contributing/release-process.md`.
**Release target.** Must exist **before the 0.42.0 tag is pushed** — that cut
is the first one it governs. No crate code.

## The finding

snora's rules fall into two kinds, and only one kind has ever held.

| Rule | Enforced by | Held? |
|---|---|---|
| Palette gains a field → `usages()` must too | `E0027` (RFC-063) | **Always** |
| Never publish a dirty tree | cargo refuses | **Always** — caught 0.27.1 |
| Version snippets match the workspace | `check-version-snippets.sh` | **Always** |
| Contrast floors | `snora-design` tests | **Always** |
| *"No release merges while this is red"* (`release-process.md:53`) | prose | **No** — main sat red four commits; 0.41.1 published on red |
| `git tag -s X.Y.Z` (`release-process.md:294`) | prose | **No** — 0.41.1 was first tagged `v0.41.1`, against 67 bare tags |
| Never `git add -A` (RFC "Who commits what", controls 1 and 2) | prose | **No** — twice, the second time by the commit that wrote the rule |
| *"No behaviour change"* in a docs-only RFC | prose | **No** — RFC-089 shipped F-33 and F-34 |

The pattern is not carelessness and cannot be fixed by more care. **A rule a
machine refuses has never been broken here. A rule a person must remember has
been broken every time it mattered.** RFC-063 is the proof in the other
direction: nobody has ever forgotten to update `usages()`, because forgetting
does not compile.

The response to a failed prose control has so far been **more prose**. "Who
commits what" is two hand-remembered controls written after two incidents, and
the commit introducing it violated it. That is the strongest available evidence
that the category, not the wording, is the problem.

## The specific failure this RFC closes

`release-process.md` states the CI rule at line 53, inside a workflow
**reference table**, 320 lines above the checklist anyone actually follows
during a cut. Its Publishing section (line 375) is thorough about the dirty
tree — *"Cargo refuses a dirty tree by default, **which is the guard**"* — and
says **nothing about CI state at all.**

So the one release rule with a named enforcement mechanism is the one that has
never failed, in the same document where the unenforced one failed twice.

Three separate prose rules govern the moment of publishing:

1. CI must be green.
2. Publish from a clean tree at the tag.
3. Tag as `X.Y.Z`, signed.

All three describe **the state a laptop must be in.** None of them can be
checked by the thing that does the publishing, because that thing is a person
typing `cargo publish` in a terminal.

## Proposal

**Publishing moves into CI, triggered by pushing the release tag.**

A new `.github/workflows/release.yaml` checks out the tag, verifies it, and runs
`cargo publish --workspace`. The three rules stop being instructions and become
properties of the mechanism:

| Rule today | After |
|---|---|
| Remember to check CI is green | The workflow refuses to publish a commit whose CI is not green |
| Publish from a clean tree at the tag | The workflow *is* a clean checkout of the tag — a dirty tree cannot exist |
| Tag `X.Y.Z`, not `vX.Y.Z` | The trigger pattern refuses the wrong shape |
| *(unwritten)* tag must match the workspace version | Asserted before upload |

The human decision to cut a release does not move. Pushing a signed tag is still
a deliberate act, performed by the same person, at the same point. What moves is
the **execution**, out of an environment whose state nobody can verify and into
one where the preconditions are structural.

This also makes the standing "publish when cutting" authorization *safer* rather
than broader: the authorized action becomes "push a signed tag", which is
reversible for about a minute, instead of "upload five crates from whatever this
working directory currently contains", which is not reversible at all.

## Non-goals

- **Not changing what is published.** Still `cargo publish --workspace`, still
  one command, still the five crates together.
- **Not automating the decision to release.** No auto-bump, no release-on-merge.
  A person still writes the CHANGELOG, bumps the version, and signs the tag.
- **Not removing the local path as break-glass** — but see Q-4; if it stays it
  must be documented as an exception with a named condition, not as an equal
  alternative, or it becomes the default again.
- **Not fixing RFC-087 D-1.** See the ordering constraint below.

## Ordering constraint — this RFC cannot ship first

~~CI is **red right now**~~ — **discharged 2026-09-02.** D-1 is fixed in
`e0eeda5` and CI run `33565992772` is green on every job. The gate now prints
its own result in CI (*"18 total gap(s) found; 0 at or after 0.39"*), where
before it printed nothing at all.

The constraint was real: a green-CI precondition introduced before `e0eeda5`
would have blocked **every** release. It is recorded here rather than deleted,
because the sequencing was the point — it was written down before the next cut
rather than discovered during it.

## Open questions

**Q-1 — trigger: tag push, or `workflow_dispatch` with a tag input?**
Tag push is the smaller change and matches how cutting already works. Dispatch
adds a confirmation step but makes the tag no longer sufficient, which splits
the act of cutting in two. **Suggest tag push**, pattern-restricted to
`[0-9]+.[0-9]+.[0-9]+`.

**Q-2 — credentials. Ruled 2026-09-02: crates.io Trusted Publishing (OIDC).
No long-lived token is created at any point.**

*First ruled the other way, and corrected the same day.* The first ruling chose
a scoped API token to keep an unverified mechanism out of Unit 1, and deferred
Trusted Publishing to RFC-091. It was wrong in a way worth recording: it bought
certainty for the implementer by making the owner **create a credential we
already intended to delete**, and it read "do the setup once, later" as cheaper
than "do the setup once, now" when both are the same single action on
crates.io. The owner asked why they were paying twice. They were not paying
twice, but they were paying for something disposable, which is worse.

**The sequencing objection that produced the first ruling dissolves on
inspection.** A Trusted Publisher Configuration binds to a workflow *filename*,
and `release.yaml` does not exist yet — so it looked as though a credential was
needed before the file could be written. It is not: **Unit 1 and Unit 2 need no
credential at all.** The three refusals all fire before any upload. Only the
final publish step authenticates, and by the time it runs, the filename is
settled and the configuration can point at something real.

Order of operations:

1. Implementer writes `release.yaml`; proves the three refusals (Unit 2). No
   credential exists.
2. Filename now fixed. **Owner** configures Trusted Publishing on the five
   crates against that workflow — the one crates.io action, performed once.
3. First cut publishes over OIDC. Nothing to revoke afterwards.

**The residual risk is accepted, named, and bounded.** It remains unconfirmed
whether one OIDC exchange authorizes all five uploads of a `cargo publish
--workspace` call, or whether each crate needs its own authenticate-and-publish
step. If it is the latter, the **publish step** is reshaped into a per-crate
loop — not the workflow, not the refusals, not Unit 2's evidence. That is a
contained blast radius, and it is hit once, deliberately, at the moment the
question can actually be answered instead of researched.

Answer it empirically at step 3 and record the result.

**Q-3 — partial publish.** `cargo publish --workspace` uploads five crates in
dependency order and can fail after two. Today a person sees that and reacts.
What does the workflow do — fail loudly and leave the rest to a human, or
retry? **Suggest fail loudly**; a half-published release needs judgement, and
crates.io versions cannot be un-published, only yanked.

**Q-4 — does the local `cargo publish` path stay documented?** If yes, under
what named condition may it be used? An undocumented exception becomes the
habit; a documented one with no condition is the same thing more slowly.

**Q-5 — does the workflow re-run the suites, or require the commit's existing
run to be green?** Re-running is self-contained but slow and could pass on a
commit whose earlier run failed. Requiring the existing run is fast but depends
on that run existing and being findable. **Suggest requiring the existing green
run**, and failing closed when none is found.

**Q-6 — what about the tag already on the remote?** `v0.41.1` is still pushed,
alongside the correct `0.41.1`. Deleting it is out of scope here but should not
be forgotten; a future pattern-restricted trigger would not have created it.

## Acceptance criteria

1. `.github/workflows/release.yaml` publishes the workspace on a tag matching
   the bare `X.Y.Z` pattern, and only then.
2. It refuses when the tagged commit has no green CI run (Q-5's ruling).
3. It refuses when the tag does not match `[workspace.package].version`.
4. **Proven to fail**, per RFC-090's companion principle and the owner's
   standing rule for defect tests: demonstrate each of the three refusals
   firing on a deliberately broken input before the workflow is accepted.
   Evidence in the review package. A release gate that has only ever been seen
   to pass is exactly the defect RFC-087 D-1 documents.
5. `release-process.md`'s Publishing section describes the mechanism, and the
   three prose rules it replaces are removed rather than left beside it —
   stale process text is how line 53 came to be ignored.
6. CHANGELOG entry.

## What this does not fix

The other half of the finding: **claims about code are not checked the way code
is.** "No behaviour change", "documentation only", "all thirteen items landed",
"confirmed passing on a clean tree" — every one of those was accepted this cycle
on the claimant's word, and two were wrong. RFC-090 mechanizes the release. It
does not mechanize the review, and a separate RFC should.
