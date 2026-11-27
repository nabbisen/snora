# Implementation handoff — RFC-140: the release path's actions pinned, and the tag's signature verified

**RFC.** `rfcs/accepted/140-the-release-path-runs-actions-from-refs-that-can-move.md`. Read the finding, R-1 to R-6 and
the Rulings (every question as suggested).
**Release target.** 0.67.0. CI, a script and the book; no crate changes.
**Starts** now (owner, 2026-11-28). **Order:** before RFC-141. The audit's request D, which is read-only, may go on
alongside.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.
**Order of units.** Send Unit 0 for review first. Unit 1 starts after that review.

## Rules

1. **Every `uses:` is a full 40-hex commit SHA, GitHub's own `actions/*` included,** with the release or tag in a
   trailing comment (`# v6.0.1`). One rule, with no list of trusted publishers.
2. **The toolchain does not change.** `dtolnay/rust-toolchain@stable` chooses the toolchain by its ref, so once
   pinned it takes `with: toolchain: stable`. The two `@master` uses (`ci.yaml`'s MSRV job and `unpinned-build.yaml`)
   already pass `toolchain:`. Show each job's `rustc --version` before and after.
3. **Resolve each SHA from the action's own repository,** for the tag we use today (`git ls-remote`, or the GitHub API
   on the tag). **Never copy a SHA from a third-party page.** Record the command and its output for each one.
4. **`scripts/check-action-pins.sh`** fails on any `uses:` in `.github/workflows/*.y*ml` that is not `owner/repo@` +
   40 hex characters (local `./` actions excepted, if any). It runs in Workflow lint, and locally. It can only be shown
   failing by giving it bad input: a tag ref, a branch ref, a short SHA, and a SHA with no comment if you make the
   comment required. **Say whether you make it required, and why.**
5. **Every workflow declares `permissions:` at the top,** `contents: read` unless a job needs more. The four with none
   today: `ci.yaml`, `supply-chain.yaml`, `unpinned-build.yaml` and `workflow-lint.yaml`. Leave the jobs that state
   more (`publish`, `github-release`, the two commit-back jobs and `docs.yaml`) as they are.
6. **`.github/dependabot.yml`:** the `github-actions` ecosystem only, weekly. Nothing for `cargo`, because the crate
   graph has its own gates (`unpinned-build`, `supply-chain`).
7. **The signature check: `scripts/check-tag-signature.sh <tag>`.**
   - It imports `.github/release-signing-key.asc` into a throwaway `GNUPGHOME` and refuses unless
     `git verify-tag <tag>` passes with that key.
   - It runs in `publish`, after the two existing refusals and before the toolchain.
   - **The key file is the architect's to add.** It is the public half of the key that signs the tags (the owner's
     identity), exported in the same commit as Unit 1 or just before it. Do not export or commit a key yourself.
   - **Check in Unit 0 that the publish job's checkout has the annotated tag object,** not only the commit. If it does
     not, the fetch the job needs is part of Unit 1.
8. **Never push a release tag to test anything.** Show the script refusing locally:
   - an unsigned local tag;
   - a tag signed by a scratch key;
   - and passing on `0.66.0`.

   Delete the local scratch tags afterwards, and never push them.

## Unit 0 — inventory and prototypes (nothing committed)

- **Every `uses:` line:** file, job, current ref, the SHA it resolves to today, how you resolved it, and that SHA's
  release.
- **Every `dtolnay/rust-toolchain` use:** the toolchain each installs today.
- **Dependabot's handling of a SHA pin with a version comment,** from its documentation, with the source named.
- **Whether `actions/checkout` in `publish` fetches the annotated tag object,** read from its documentation or
  measured in a scratch repository. Name which.
- **The two scripts,** prototyped and shown refusing and passing as rules 4 and 8 say.

## Unit 1

- Rules 1 to 7 in every workflow. Each workflow's first run after the push is green, and every toolchain is the same
  as before.
- **Breakage rows, each shown failing:** an unpinned `uses:` (the pin gate); a workflow with its `permissions:` removed
  (the gate, if you make it check that too; **say whether**); a tag that is not signed, and one signed by another key
  (the signature script).
- No CHANGELOG entry for adopters, since no crate changes. The 0.67 migration guide gets one line from me.

## Acceptance criteria

1. Each guard shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout` or `git restore`.
2. `scripts/check-workflows.sh` and the new scripts pass. If `shellcheck` or `actionlint` is still missing here, say
   so, and CI's Workflow lint is the check.
3. Every workflow green after the push, read by name.
4. **Not yours:** the key file and the threat model's §4 (the architect), and the repository setting (the owner, after
   this lands).
