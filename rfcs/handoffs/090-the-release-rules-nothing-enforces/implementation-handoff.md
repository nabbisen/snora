# Developer Handoff — RFC-090 Publishing moves into CI

**Governing RFC.** **RFC-090** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does.)*
**Status.** Accepted (owner, 2026-09-02). High.
**Release target.** **Must exist before the 0.42.0 tag is pushed** — that cut is
the first one it governs. **No crate code.**
**Implementation units.** Two, plus one that is not yours.
**Q-2 ruled 2026-09-02 — unit 1 is unblocked.** See
`.git-exclude/reviewed/090-q2-trusted-publishing-verification/review-result.md`.
Unit 2 needs no credential at all and may start immediately.

---

## 0. What is already done, so you do not redo it

**RFC-087 D-1 is fixed** (`e0eeda5`) and CI is green — run `33565992772`, every
job. That was RFC-090's ordering constraint and it is discharged. You are not
inheriting a red main.

While you are here: the gate that D-1 fixed is the reason unit 2 exists. Read
what happened to it before you write a new gate of your own.

## 1. Unit 1 — `.github/workflows/release.yaml`

Publishing moves off a laptop and into a workflow triggered by the release tag.

**Q-1 ruled: tag push**, pattern-restricted. Not `workflow_dispatch` — that
splits cutting into two acts, and the tag is already the deliberate one.

    on:
      push:
        tags: ['[0-9]+.[0-9]+.[0-9]+']

That pattern is doing real work: `v0.41.1` was pushed by mistake on 2026-09-02
against 67 bare tags, and this trigger would not have fired for it.

The job checks out the tag and runs `cargo publish --workspace`. **One command,
five crates, as today** — RFC-090 does not change what is published.

Two refusals you write, before any upload — they map exactly onto Unit 2's
three test cases (the first covers two of them):

1. **The tagged commit has no green CI run.** **Q-5 ruled: require the existing
   run; do not re-run the suites.** Re-running is slow and, worse, can pass on a
   commit whose earlier run failed. **Fail closed** — if no run is found, that is
   a refusal, not a pass. This is the whole point of the RFC; get it right.
2. **The tag does not match `[workspace.package].version` in `Cargo.toml`.**
A missing credential needs no refusal of yours: with Trusted Publishing the
auth step fails on its own, before `cargo publish` runs. Do not write a check
for it.

A clean tree at the tag needs no check — the workflow *is* one. That is the
mechanism doing the work instead of a sentence asking someone to remember.

## 2. Unit 2 — prove all three refusals fire

**This unit is not optional and it is not paperwork.** RFC-087's gate was
accepted on evidence that it passed, shipped, and could not run at all. Its D-1
fix was accepted only after being shown failing on a deliberately broken input,
three ways. Same bar here.

Demonstrate, with evidence in the review package:

| # | Broken input | Expected |
|---|---|---|
| 1 | tag on a commit whose CI is red | refuses, names the run |
| 2 | tag on a commit with **no** CI run at all | refuses (fail closed — the case people forget) |
| 3 | tag `X.Y.Z` where `Cargo.toml` says something else | refuses, prints both |

Case 2 is the one to be careful about. "No run found" is the state a naive
implementation treats as "nothing said no", and that is exactly how the
migration-guide gate came to pass while verifying nothing.

Do this against a scratch tag you delete afterwards, not against a real release.

## 3. Not yours — `release-process.md`

The owner updates the prose. **Q-4 ruled:** the local `cargo publish` path stays
documented, but as break-glass with a named condition — *the workflow itself is
broken, and the owner says so for that specific release* — not as an equal
alternative. An exception with no condition becomes the default.

The three prose rules the mechanism replaces get **removed**, not left standing
beside it. Stale process text is how `release-process.md:53` came to be ignored
while sitting in the same file as the rule that has never failed.

## Q-2 — ruled: Trusted Publishing (OIDC). **No API token, ever.**

*Re-ruled 2026-09-02, same day, replacing an earlier ruling that chose a scoped
API token. That ruling would have had the owner create a credential we intended
to delete. It is corrected in RFC-090's Q-2 with the reasoning; read it there
rather than inferring it from this section.*

**Units 1 and 2 need no credential at all.** All three refusals fire before any
upload, so you can write `release.yaml` and produce every piece of Unit 2's
evidence with nothing configured. **Start now; nothing is waiting on the owner.**

Order of operations:

1. **You:** write `release.yaml`, prove the three refusals against a scratch tag.
2. **Owner:** configures Trusted Publishing on the five crates, bound to the
   workflow filename you settled on. One crates.io action, performed once. **Tell
   them the exact filename** when you get there — the configuration binds to it,
   and renaming the file later silently breaks publishing.
3. **First real cut:** publishes over OIDC. No secret is created, so there is
   nothing to revoke.

Use `rust-lang/crates-io-auth-action` for the exchange, then `cargo publish
--workspace`.

### The one unconfirmed thing, and what to do about it

It is **not confirmed** whether a single OIDC exchange authorizes all five
uploads of one `cargo publish --workspace` call, or whether each crate needs its
own authenticate-and-publish step. You researched this and could not close it
from documentation; that finding stands and was not overruled.

**The decision accepts that risk deliberately.** If it turns out to need a
per-crate loop, what changes is the **publish step** — not the workflow, not the
refusals, not Unit 2's evidence. Contained.

So: **do not attempt to resolve it before step 3, and do not design around
both possibilities.** Write the `--workspace` form. If step 3 disproves it,
reshape the publish step then, with the real error in hand. A workflow written
to hedge an unanswered question is harder to review than one written to a stated
assumption that turned out wrong.

Record the answer either way — RFC-090's Q-2 asks for it explicitly, and it is
the last open fact in this design.

## Q-6 — housekeeping, owner

`v0.41.1` is still on the remote alongside the correct `0.41.1`. Deleting it is
the owner's to run:

    git push origin :refs/tags/v0.41.1
    git tag -d v0.41.1

Harmless to the gates — `check-migration-guides.sh` matches bare `X.Y.Z` only,
confirmed against a 69-tag clone — but it is a wrong tag on a public repo.

## Acceptance criteria

1. `release.yaml` publishes on a bare `X.Y.Z` tag and only then.
2. Refuses when the tagged commit has no green CI run, **including when no run
   exists at all**.
3. Refuses when the tag and `[workspace.package].version` disagree.
4. **All three refusals demonstrated failing**, evidence in the review package.
5. `scripts/README.md` and any CI documentation reflect the new workflow.
6. **CHANGELOG entry** — and if you judge that this warrants none because it is
   CI-only, **say so in one line** rather than leaving the omission silent. That
   was the one thing missing from RFC-089's otherwise clean sweep.
