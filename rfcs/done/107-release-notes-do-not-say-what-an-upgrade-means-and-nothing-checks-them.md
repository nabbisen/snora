# RFC-107 — Release notes do not say what an upgrade means, and nothing checks them

**Status.** Done — shipped v0.53.0. Q-1 to Q-4 and Q-6 ruled as suggested; Q-5 revised to a split by series (see Rulings).
**Raised.** 2026-10-18, architect, from an owner-requested audit of `CHANGELOG.md`
for readability (long-time and new users) and for long-term management.
**Release target.** 0.53.0 (Q-6). **Priority.** Medium: no defect reaches code,
but every release adds to the cost.

---

## The finding

Measured on `main` before `1b58f13`, the navigation fix below.

**Readers are not told what an upgrade means for them.**

- **No entry opens with its impact.** 0.52.0 is 147 lines and 0.51.0 is 150, the
  two longest in the file. A long-time user has to read all of 0.52.0 to learn
  that no code change is needed. That answer is in the migration guide, not the
  changelog.
- **Since 0.39, only 4 of 14 minors linked their migration guide**, though the
  guide is the readable document for most users. *Fixed in `1b58f13`* (see
  "Already done").
- **Entries are written for reviewers.** 0.52.0's first bullet describes an
  `#[ignore]`d test and what frame hashes can and cannot show. That is true and
  useful, but its home is the RFC.
- **It is hard to reach.** It is not in the book. There are no GitHub Releases
  (`gh api repos/nabbisen/snora/releases` returns 0). It is not in any crate
  package, since all five crates are packaged from `crates/*`. A crates.io
  reader reaches it only through the repository.

**Nothing checks it.**

- **The compare links stopped at 0.46.0.** 0.47–0.52 had no compare links, and
  `[Unreleased]` still compared `0.46.0...HEAD`. That lasted six releases, and no
  gate saw it. *Fixed in `1b58f13`.*
- **Headings drift from our own versioning policy.** The policy lists Added,
  Changed, Deprecated, Removed and Fixed. The file also uses Tests (3), Notes (2),
  Breaking changes, Design API changes and Deferred (1 each). There is no
  Security heading, though RUSTSEC advisories have shipped in 0.47.0, 0.49.0 and
  0.50.0, under other headings.
- **Three released entries were rewritten after their tags, unmarked.** Each
  release's section at its own tag was compared with `main`: 61 of 66 are
  identical. 0.7.0 and 0.8.0 differ only in link paths and link layout, which is
  harmless. **0.19.1 and 0.20.0 were reworded, and 0.25.2 gained a whole Changed
  section** (`6642b94`, 2026-08-02). None carries a mark. Our requirements
  register keeps history and adds corrections beneath; the changelog does not.

**Size is not the problem, growth is.** The file is 197 KB, 3,652 lines, and 67
versions, with recent minors adding about 150 lines each. No plan says what
happens to 0.x history at 1.0.

## Already done (`1b58f13`, docs-only, owner-authorized)

- Every minor from 0.39 to 0.52 opens with one navigation line:
  `**Upgrading from 0.X:** read the [0.X → 0.Y migration guide](…).`
- Compare links added for 0.47–0.52, and `[Unreleased]` now compares `0.52.0...HEAD`.
  Every header has a link, and every link endpoint is an existing tag. Entry text
  is unchanged; only these lines and blank lines differ.

**Note:** `check-repo-links.py` matches only links beginning with `.`, so it does
not check the changelog's `docs/src/…` links. The 14 targets were checked by hand.
R-4 closes that.

## Proposal

**R-1 — every entry opens with a fixed lead, from 0.53.0.** It comes right after
the navigation line:

```markdown
## [0.53.0] — 2026-10-xx

**Upgrading from 0.52:** read the [0.52 → 0.53 migration guide](docs/src/guides/migration-0.52-to-0.53.md).

> **In short:** one to three sentences on what changed for users.
> **Code:** no change needed. *(or: what changes, with the guide section)*
> **Appearance:** unchanged. *(or: what moves, and which baselines it invalidates)*
> **Security:** none. *(or: the advisory, and whether a consumer must act)*
```

Patches carry the same lead without the navigation line. Below it, bullets
describe **user impact**. Measurement and mechanism go to the RFC, which the
bullet names. The four labels are mandatory and "none" or "unchanged" is a valid
value, so their presence can be checked (Q-1).

**R-2 — headings are exactly Keep a Changelog 1.1.0's six:** Added, Changed,
Deprecated, Removed, Fixed and Security. `versioning-policy.md` gains Security:
"advisories that affect snora or reach consumers through it, and what a consumer
must do". Old non-standard headings stay, because released text is not
rewritten (R-3).

**R-3 — released entries are frozen at their tag.** After a tag, three kinds of
edit are allowed:

1. **the navigation line**, in its exact form;
2. **a correction appended under the entry**, as a line beginning
   `*Corrected YYYY-MM-DD:*` that says what was wrong. The original text stays;
3. **blank lines**.

Anything else fails R-4. The three past rewrites are handled per Q-3.

**R-4 — `scripts/check-changelog.sh`, in CI's `docs` job.** That job already
checks out with `fetch-depth: 0`, so tags are present. The script **fails, never
skips**, if no tags are visible. It checks:

1. every `## [x.y.z]` header has a link reference, every link endpoint is an
   existing tag, and `[Unreleased]` compares the newest tag `...HEAD`;
2. every repo-relative link in `CHANGELOG.md` and `changelog/*.md` resolves
   (`check-repo-links.py` checks only links beginning with `.`);
3. every minor from 0.39 has the navigation line, pointing at its guide;
4. entries from 0.53.0 use only R-2's headings and carry R-1's four labels;
5. each released entry, **found in whichever file holds it now**, equals its
   text at its own tag, after dropping blank lines, link-reference lines, the
   navigation line and `*Corrected …*` lines, and after undoing the `../`
   rebase of links in `changelog/*.md`. The exception list is keyed by version
   and the rewriting commit (see Rulings);
6. each `changelog/0.Nx.md` holds only versions 0.N0–0.N9 (0.0x: 0.7–0.9),
   `CHANGELOG.md` holds only the current series and `[Unreleased]`, and no
   version appears twice;
7. **warning only:** any one file larger than 256 KB.

The script is in the `check-*.sh` family. **Each rule is shown failing first**
on a deliberate breakage, then restored.

**R-5 — each tag gets a GitHub Release, generated from its entry.** It is a
separate job in `release.yaml` that runs **only after the crates.io publish
succeeds**. That job alone has `contents: write`; the publish job keeps
`contents: read`. The body is the entry, extracted by
`scripts/extract-release-notes.sh <version>`, **with repo-relative links
rewritten to absolute `blob/<tag>/` URLs**, because a release body does not
resolve relative links. If this job fails, a release the registry already has is
not undone or blocked. It is re-runnable by hand.

**R-6 — the release checklist gains the lead, the link references and Security.**
`release-process.md`'s "Move the [Unreleased] section…" step becomes:

- write the lead;
- add the navigation line;
- add the version's compare link and repoint `[Unreleased]`;
- run `check-changelog.sh`.

## Open questions

**Q-1 — the lead's form.** **Suggest the four-label blockquote above.** Fixed
labels make it checkable, and make it scannable for a reader skimming ten
releases. Free prose would read better once and worse across many.

**Q-2 — a length cap on entries?** **Suggest no hard cap.** A cap would push
needed facts out, or into vaguer words. R-1's lead gives the short answer, and
moving measurement to the RFC shortens the rest. Review it at 0.55.

**Q-3 — the three past rewrites (0.19.1, 0.20.0, 0.25.2).** **Suggest
grandfathering them** in the script's exception list, with the commit and a
one-line reason, and adding a `*Corrected 2026-10-xx:*` line to each saying the
entry was revised after release in `<commit>`. Restoring the tagged text would
itself be a second rewrite. It would also drop 0.25.2's Changed section, which
records a real change the tagged entry omitted: 0.25.2's own `Cargo.toml` has
`resolver = "3"`, and 0.25.1's has `"2"`.

**Q-4 — GitHub Releases: backfill?** **Suggest no: 0.53.0 onward.** Backfilled
releases would carry today's date, not the release date. The changelog and the
tags already cover the past.

**Q-5 — the 0.x archive.** *(Revised; see Rulings.)* **Suggest deferring, with a recorded trigger: the
1.0.0 cut.** At that point, move 0.x history to `CHANGELOG-0.x.md` and link it
from the top. Nothing needs to move before then.

**Q-6 — release.** **Suggest 0.53.0.** The script, the policy text and the
release job land before that cut, so 0.53.0's entry is the first one checked and
the first one published as a GitHub Release.

## What this is not

- **Not a rewrite of history.** No released entry's text changes. The only
  additions are R-3's permitted kinds.
- **No change to code or the public API.** It touches docs, a script and two
  workflows.
- **No replacement for migration guides.** The lead points to the guide, and the
  guide stays the place for how to upgrade.

## Acceptance criteria

1. `check-changelog.sh` is in CI and passes on `main`. **Each of its five rules
   is shown failing** on a scratch breakage, restored from a named backup and
   verified with `sha256sum -c`:
   - a missing link reference;
   - a broken guide link;
   - a non-standard heading in a post-0.53 entry;
   - a missing lead label;
   - an unmarked edit to a released entry.
2. With tags hidden (for example, a shallow clone), the script **fails**.
3. `extract-release-notes.sh 0.52.0` emits the 0.52.0 entry with every link
   absolute and resolving.
4. `release.yaml`'s release job has `contents: write`, the publish job keeps
   `contents: read`, and the release job `needs:` the publish job. Quote the diff.
5. `versioning-policy.md` lists the six headings, and `release-process.md` has
   R-6's step.
6. The three grandfathered entries carry their `*Corrected …*` line (if Q-3 is
   ruled as suggested).

---

## Rulings, 2026-10-18

**Q-1, Q-2, Q-3, Q-4, Q-6:** as suggested.

**Q-5, revised twice.** The owner rejected "at the 1.0.0 cut": no 1.0 is
planned, so that trigger would never fire. A size trigger (archive past 256 KB)
was then put forward and also rejected, because it answers a question about
the file, not the reader's question: where is the entry for my version? **Ruled:
split mechanically by series of ten minors.**

**R-7 — history is filed by series.** `CHANGELOG.md` holds `[Unreleased]` and
the current series (0.50 onward). Each closed series lives in
`changelog/0.Nx.md` (0.0x is 0.7–0.9), with its own compare links, and does not
change. At each 0.X0.0 cut, the previous series moves, as a release-checklist
step. Measured, the largest closed series is 0.3x at 64 KB.

## Done by the architect, 2026-10-18

These are docs and need no handoff:

- **The split.** It produced `changelog/0.0x.md` … `0.4x.md` (3 / 14 / 15 / 20 / 11
  entries), and `CHANGELOG.md` fell from 197 KB to 21 KB, with a series table in
  its header. Verified entry by entry: all 67 entries are identical after
  normalisation, and all 67 compare links are preserved. In the moved files the
  35 `](docs/` links were rebased to `](../docs/`, and `check-repo-links.py` now
  checks them (420 links, 0 broken).
- **Q-3's correction lines**, under 0.19.1 (`bd215f8`), 0.20.0 (`d8d3f9d`) and
  0.25.2 (`6642b94`).
- **`versioning-policy.md`:** the six headings, the lead, the freeze rule and the
  series rule. **`release-process.md`:** R-6's checklist step and the series-move
  step.
- **`scripts/check-version-snippets.sh`:** skips `changelog/*.md`, as it already
  skips `CHANGELOG.md`. This was needed for the split to leave CI green.

**The exception list for R-4 rule 5** is five entries, found by comparing each
tagged entry with `main` under rule 5's normalisation:

| Version | Commit(s) | What changed | Corrected line |
|---|---|---|---|
| 0.7.0 | `0a26eb8`, `1c52464` | link paths repaired (`docs/…` → `docs/src/…`) | no, link repair |
| 0.8.0 | `1c52464` | file layout: 0.7.0 text that sat inside 0.8.0's section moved out | no, layout repair |
| 0.19.1 | `bd215f8` | binary-size bullet replaced | yes |
| 0.20.0 | `d8d3f9d` | an overclaim about `design` measurements corrected | yes |
| 0.25.2 | `6642b94` | the whole entry added: at the tag there was none *(corrected 2026-10-18 from "an entry that was empty", found by the dev team)* | yes |

The handoff carries the dev team's part: R-4, R-5 and rule 7.
