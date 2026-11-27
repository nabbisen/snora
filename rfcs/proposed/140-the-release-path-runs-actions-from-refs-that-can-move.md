# RFC-140 — The release path runs third-party actions from refs that can move

**Status.** Proposed, 2026-11-28.
**Raised.** 2026-11-28, the 0.67 audit's request C (row C3-T-45, reported first and as its class only), verified by
the architect (`.git-exclude/reviewed/audit-0-67-c/review-result.md`). The owner asked for this RFC for 0.67 on
2026-11-28.
**Release target.** 0.67.0. It changes CI and the book, not a crate.
**Priority.** A supply-chain integrity gap in how snora is published. It is not a vulnerability in the library an
adopter builds.

---

## The finding

Read on `66dbd23`, across the eight files in `.github/workflows/`:

- **No `uses:` is pinned to a commit.** All 60 refer to a tag or a branch. 29 of them are third-party:
  - `dtolnay/rust-toolchain@stable` 14 times, and `@master` twice;
  - `Swatinem/rust-cache@v2` 10 times;
  - `stefanzweifel/git-auto-commit-action@v5` twice;
  - `rust-lang/crates-io-auth-action@v1` once.

  A tag or a branch can be moved to other code by whoever controls that repository, or by anyone who compromises it.
- **The jobs that hold authority run them:**

  | Workflow, job | Authority | Third-party actions it runs |
  |---|---|---|
  | `release.yaml`, `publish` | `id-token: write`, crates.io Trusted Publishing | `dtolnay/rust-toolchain@stable`, and **`rust-lang/crates-io-auth-action@v1`, the step that obtains the publishing credential** |
  | `binary-size.yaml`, `build-cost.yaml` | `contents: write` (commit-back on tags) | `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` (binary-size), `stefanzweifel/git-auto-commit-action@v5` |
  | `docs.yaml` | `pages: write`, `id-token: write` | GitHub's own Pages actions only |

- **Nothing keeps a pin current, and nothing checks for one.** There is no `.github/dependabot.yml`, and `actionlint`
  does not check pinning. The repository's own setting that requires full-length SHAs is available and is off
  (`gh api repos/nabbisen/snora/actions/permissions`: `"sha_pinning_required": false`).
- **The tag's signature is not verified.** The checklist signs tags (`git tag -s`), and the threat model says
  publishing happens "on a signed tag push". But `release.yaml` triggers on a tag *name*
  (`[0-9]+.[0-9]+.[0-9]+`) and never checks a signature.
- **Four workflows declare no `permissions:`** (`ci.yaml`, `supply-chain.yaml`, `unpinned-build.yaml`,
  `workflow-lint.yaml`), so they take the repository's default. That default is `read` today
  (`default_workflow_permissions`), but a change to the setting would widen them silently.
- **The threat model's §4 ("Supply chain and release") is silent on all of this.** Its guards cover crate
  dependencies (`cargo-deny`, `unpinned-build`), not the workflows' own.

**What limits it today:**
- Trusted Publishing is bound to this repository and this workflow file.
- The tag-version and green-CI refusals run before the upload.

Neither stops a moved action ref from running inside the `publish` job, with its token.

## Proposal

**R-1. Every `uses:` is pinned to a full 40-hex commit SHA, with its version in a comment.** "Every" includes
GitHub's own `actions/*`.

```yaml
- uses: actions/checkout@<40-hex sha> # v6.x.y
- uses: dtolnay/rust-toolchain@<40-hex sha> # stable branch, YYYY-MM-DD
  with:
    toolchain: stable
```

- **One rule a reader can predict,** rather than a judgement about which publishers to trust. GitHub's own actions
  carry the same kind of risk, and the cost of pinning them is the same.
- `dtolnay/rust-toolchain@stable` and `@master` choose the toolchain *by the ref*. Pinned, the toolchain is named in
  `with: toolchain:` instead, so the Rust version each job uses does not change. Unit 0 lists every such use and its
  current toolchain.

**R-2. A gate fails on any `uses:` that is not a 40-hex SHA,** in the Workflow lint workflow, as a script
(`scripts/check-action-pins.sh`) that can be run locally. It is shown failing on a tag ref, a branch ref and a short
SHA.

**R-3. Every workflow declares `permissions:`** at the top, with `contents: read` unless a job states more. That
makes four files explicit, so that a change to the repository's default cannot widen them.

**R-4. The pins are kept current** by a mechanism the owner rules on (Q-1). A pin left alone forever is its own risk:
it misses a security fix.

**R-5. The publish job verifies the tag's signature** before any upload, against a public key kept in the repository
(Q-2).

**R-6. The threat model's §4 says what the release path's own supply chain is,** and what guards it:
- now, as a book correction, the gap in plain words, until R-1 to R-5 ship;
- then, the pins, the gate, the permissions and the signature check, each naming what checks it.

## Open questions, with suggested rulings

**Q-1. How are the pins kept current?** **Suggest: Dependabot, for the `github-actions` ecosystem only, weekly.**
- It understands SHA pins with version comments, and it proposes the update as a pull request.
- That pull request is reviewed and merged by the owner. The one-`main` route governs the dev team's and the
  architect's work, not a bot's proposals, which need a human's eye anyway.
- **The alternative,** a scheduled workflow that reports newer releases, keeps everything on `main`, but someone must
  then look up each SHA by hand. That is more work, and a hand is how a wrong SHA gets in.

**Q-2. Should the publish job verify the tag's signature?** **Suggest: yes.**
- Import a public key committed to the repository, and refuse unless `git verify-tag "$TAG"` passes.
- **Stated limit:** someone who can push to `main` could replace the key and push a tag signed with their own. So this
  adds a second thing an attacker must control, the signing key, rather than closing the door. It also makes the
  threat model's "signed tag push" true, which it is not today.

**Q-3. Should the repository's "require SHA pinning" setting be turned on too?** **Suggest: yes, by the owner, after
R-1 lands.** It is the platform refusing an unpinned action, independently of our gate. It is a repository setting,
so it is the owner's to switch on, and our gate remains the check a contributor can run locally.

**Q-4. When.** **Suggest: 0.67.0.** It is CI and the book only, so it needs no crate release. Under the 2026-11-18
ruling it could land before 0.67.0 is cut, and the cut would carry it.

## Review against snora's principles

- **Safe and secure.** Code that runs with the publishing credential is the code that was reviewed, by its hash.
- **Nobody is confused.** "Every `uses:` is a SHA" is predictable, and a script says so. The threat model stops
  implying a signature check it does not make.
- **Robust.** Three independent layers, each of which can be shown failing: our gate, the platform's setting and the
  signature check.
- **Clean.** One rule for every workflow, not a list of trusted publishers.
- **Correspondence cost.** None. Adopters build the crates, not our workflows. A line in 0.67's migration guide, in
  "What did not change", says so.

## Units

**Unit 0, measured (nothing committed):**
- Every `uses:` line: its current ref, the SHA it resolves to today, and that SHA's release or tag.
- Every `dtolnay/rust-toolchain` use, and the toolchain it installs today.
- What Dependabot's `github-actions` updates look like for a SHA pin with a version comment, read from its
  documentation, with the source named.
- R-2's script, shown failing on the three bad refs and passing on a pinned file.

**Unit 1:**
- R-1, R-2 and R-3 in every workflow. Each pinned workflow's run is green, and its toolchain is unchanged.
- R-4 as ruled: `.github/dependabot.yml`, or the scheduled check.
- R-5 as ruled: the key file, the import and `git verify-tag` in `publish`, shown refusing an unsigned tag and a tag
  signed by another key. Do it on a scratch repository or by a dry run, **never by pushing a real release tag.**
- **The book is the architect's** (R-6), and **the repository setting is the owner's** (Q-3).

## Acceptance criteria

1. Unit 0's inventory, and R-2's gate shown failing.
2. Unit 1: every workflow green, pinned, and declaring `permissions:`. The signature check is shown refusing, without
   a real tag.
3. The threat model's §4 rewritten, with each guard named.
