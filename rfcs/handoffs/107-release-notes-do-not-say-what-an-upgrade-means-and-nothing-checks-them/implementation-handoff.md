# Implementation handoff — RFC-107: release notes do not say what an upgrade means, and nothing checks them

**RFC.** `rfcs/accepted/107-release-notes-do-not-say-what-an-upgrade-means-and-nothing-checks-them.md`
**Release target.** 0.53.0. Land it **before that cut**, so 0.53.0's entry is the
first one checked and the first published as a GitHub Release.
**Rulings.** Q-1: the four-label lead. Q-2: no length cap. Q-3: grandfather the
past rewrites, with correction lines. Q-4: GitHub Releases from 0.53.0 only, no
backfill. Q-5: split by series of ten minors (done). Q-6: 0.53.0.
**Touch list.**
- new: `scripts/check-changelog.sh`, `scripts/extract-release-notes.sh`;
- `.github/workflows/ci.yaml` (`docs` job), `.github/workflows/release.yaml`;
- `scripts/README.md`;
- **not** `CHANGELOG.md` or `changelog/*.md`, except scratch breakage for the
  failing-first demos, restored from named backups. **Not** `docs/`.

## Already established (done by the architect, committed)

- **Layout:** `CHANGELOG.md` holds `[Unreleased]` and 0.50–0.52, with a series
  table in its header. `changelog/0.0x.md` (0.7–0.9), `0.1x.md`, `0.2x.md`,
  `0.3x.md` and `0.4x.md` hold the closed series. Each file ends with its own
  compare links, newest first.
- **Links:** moved entries had `](docs/` rebased to `](../docs/`. That is the
  only form of relative link in the history (35 links).
- **Navigation line**, exact form, on every minor from 0.39:
  `**Upgrading from 0.X:** read the [0.X → 0.Y migration guide](<path>).`
- **Correction lines** begin `*Corrected YYYY-MM-DD:*`. Three exist, at the end
  of 0.19.1, 0.20.0 and 0.25.2.
- **Normalisation already verified.** All 67 entries, compared with their own tag
  as below, are identical except the five listed under rule 5.
- `check-version-snippets.sh` already skips `changelog/*.md`.
- The CI `docs` job checks out with `fetch-depth: 0`.

## Unit 1 — `scripts/check-changelog.sh`

Use bash, in the `check-*.sh` family, and print `OK` or a `FAIL <rule>: <file>:<line> <why>`
line for each failure. **It fails, never skips**, if `git tag -l '0.*'` is empty.
Its seven rules (rule 7 only warns):

1. **Compare links.** Every `## [x.y.z]` header in a file has a `[x.y.z]: `
   reference in **the same file**. Every compare endpoint is an existing tag.
   `[Unreleased]` is in `CHANGELOG.md` and compares `<newest tag>...HEAD`.
2. **Relative links resolve.** Every `](…)` link that is not `http(s):` and not
   `#…` resolves from its file's own directory, in `CHANGELOG.md` and
   `changelog/*.md`.
3. **Guide links.** Every `0.Y.0` from 0.39.0 has the navigation line as the first
   non-blank line after its header, and its target is
   `docs/src/guides/migration-0.(Y-1)-to-0.Y.md` (rebased in `changelog/`).
4. **Entries from 0.53.0 on** use only these headings: Added, Changed,
   Deprecated, Removed, Fixed, Security. Each carries the four lead labels
   `**In short:**`, `**Code:**`, `**Appearance:**` and `**Security:**`, before its
   first `###`. `[Unreleased]` is exempt, because the lead is written at the cut.
5. **Frozen entries.** For each tag `T`, take `T`'s entry from the file that holds
   it now, and from `git show T:<path>`, where the path is `CHANGELOG.md` before
   the split and the series file after. Normalise both:
   - drop blank lines;
   - drop `^\[[^]]+\]: ` lines;
   - drop the navigation line;
   - drop lines beginning `*Corrected `;
   - replace `](../docs/` with `](docs/`.

   They must be equal. **Exceptions**, as a list in the script with these
   reasons:

   | Version | Commit(s) | Reason |
   |---|---|---|
   | 0.7.0 | `0a26eb8`, `1c52464` | link paths repaired |
   | 0.8.0 | `1c52464` | file layout repaired: 0.7.0 text had sat inside 0.8.0's section |
   | 0.19.1 | `bd215f8` | revised after release; corrected line present |
   | 0.20.0 | `d8d3f9d` | revised after release; corrected line present |
   | 0.25.2 | `6642b94` | revised after release; corrected line present |

   For the three "corrected line present" rows, **also assert the line exists**,
   so an exception cannot outlive its disclosure.
6. **Series placement.** `changelog/0.Nx.md` holds only 0.N0–0.N9 (`0.0x`: 0.7–0.9).
   `CHANGELOG.md` holds only the newest series and `[Unreleased]`. No version
   appears in two files. Every tag `0.y.z` with y ≥ 7 has exactly one entry.
7. **Warning only:** a file larger than 256 KB prints `WARN`; the exit status
   stays 0.

**Performance:** rule 5 runs `git show` once per tag (66 today), not per line.

## Unit 2 — `scripts/extract-release-notes.sh <version>`

It prints that version's entry from whichever file holds it, from the header
through the line before the next `## [`. Then:

- link references are dropped;
- every relative link is rewritten to
  `https://github.com/nabbisen/snora/blob/<version>/<repo path>`, resolved from the
  file's directory, because a release body does not resolve relative links;
- the header line itself is dropped, since the Release title carries it.

It exits non-zero if the version has no entry.

## Unit 3 — wiring

- **`ci.yaml`, `docs` job:** add the step `Changelog is well-formed (RFC-107)`,
  `run: scripts/check-changelog.sh`.
  - **While you are in `ci.yaml`:** the bounded-work step should run with
    `-- --nocapture`, so its measured ratio appears in the log (an RFC-105
    follow-up).
- **`release.yaml`:** add a second job, `github-release`.
  - It runs `needs: publish`, so it **runs only after crates.io succeeds**.
  - It has job-level `permissions: contents: write`, and nothing else: no
    `id-token`, no `actions`.
  - Give `publish` **job-level** permissions equal to today's workflow-level
    block (`contents: read`, `actions: read`, `id-token: write`). Then change
    the workflow-level block to `contents: read` alone, so that no job inherits
    `id-token`.
  - Steps: check out the tag, run `scripts/extract-release-notes.sh "$TAG" > notes.md`,
    then `gh release create "$TAG" --verify-tag --title "$TAG" --notes-file notes.md`.
    If the release already exists (a re-run), fall back to
    `gh release edit "$TAG" --notes-file notes.md`.
  - **Add no new trigger.** Re-running is "Re-run failed jobs" on the run. Do not
    add `workflow_dispatch`, which would let `publish` run untagged.
  - A failure here must not, and cannot, affect the already-published crates.
    Say so in a comment.
- **`scripts/README.md`:** a row for each new script.

## Check each assertion can fail

**Show each one failing**, on scratch breakage. Back up with `cp` to named
files, demo, and restore by copying back, then `sha256sum -c`. **Never
`git checkout`/`restore`.**

| Rule | Breakage |
|---|---|
| 1 | delete `[0.51.0]: …` from `CHANGELOG.md` |
| 1 | point `[Unreleased]` at `0.51.0...HEAD` |
| 2 | rename one `../docs/` target in `changelog/0.4x.md` to a missing path |
| 3 | delete 0.45.0's navigation line |
| 4 | add a fake `## [0.53.0]` entry with a `### Notes` heading, and another missing `**Code:**` |
| 5 | change one word inside 0.47.0's entry |
| 5 | delete 0.20.0's `*Corrected` line, which must fail as "exception without disclosure" |
| 6 | move 0.41.1's entry into `changelog/0.3x.md` |
| — | run in a clone with no tags (`git clone --no-tags`), which must fail, not pass |

Then show every rule passing on the restored tree. For Unit 2, run
`extract-release-notes.sh 0.52.0` and `0.45.0`, and show that every link in the
output is absolute and resolves: `curl -sI`, or `gh api` for `blob` paths.

## Acceptance criteria

1. Both scripts exist; `check-changelog.sh` passes on `main` and is in CI.
2. Every row of the table above is shown failing, then the tree is shown
   restored with `sha256sum -c`.
3. The extract output for 0.52.0 and 0.45.0 is attached, with links checked.
4. The `release.yaml` diff is quoted:
   - `github-release` has `needs: publish` and only `contents: write`;
   - `publish` has its three permissions at job level;
   - the workflow-level block is `contents: read`;
   - there is no new trigger.
5. `check-workflows.sh` passes on the new workflow.
6. `CHANGELOG.md` and `changelog/*.md` are unchanged at the end
   (`git diff --stat` empty for them).
7. CHANGELOG: nothing. The architect writes 0.53.0's entry at the cut, using the
   new lead.
