# Implementation handoff — 0.67 follow-up: two small changes, nine guards, two measurements

**Ruled.** The owner, 2026-11-28, on the architect's scope (`.git-exclude/tasks/architect/scope-0-67.md`, §3 to §5),
from the four audit reviews (`.git-exclude/reviewed/audit-0-67-{a,b,c,d}/review-result.md`).
**Release target.** 0.67.0.
**Order.** After RFC-140's Unit 1 and RFC-141. RFC-142 and RFC-143 have handoffs of their own once accepted.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.
**Shape.** Four review requests (Units 1 to 4), each reviewed before the next is pushed.

## Rules for every guard

1. **Shown failing** on input that makes its claim false, with named `cp` backups and `sha256sum -c`; never
   `git checkout` or `git restore`. A guard that cannot be made to fail is reported, not shipped.
2. **A guard must not read what it guards.** It compares the claim with an independent source: the code, a CSV, or a
   literal in the test.
3. **It runs in CI,** in the job its suite belongs to, and `rendered-targets.sh check` and `check-workflows.sh` (or
   CI's Workflow lint) stay green.
4. **Enumerate, never search** (the audit's rule). A guard of "every X" lists the X from the source of truth.

## Unit 1 — the two small changes

- **`ShortcutRoute` and `TargetRole` re-exported from `snora::keyboard`** (`crates/snora/src/keyboard.rs`, beside
  `Digits`, `Escape`, `Platform`, `ShortcutAction` and `Shortcuts`). Additive. A doc-test that names both through
  `snora::keyboard`. CHANGELOG **Added**.
- **`build-cost.yaml`'s 30 000 ms warning removed** (the `wid_ms > 30000` block). Remove only that. The CSV append
  and the summary stay. Its comment says why: RFC-062 retired the proxy, and the warning fired on 55 of 56 rows.

## Unit 2 — guards on the public API (source-reading, in `crates/snora/tests/` beside `direction_guard.rs`)

1. **Every `pub enum` of `snora-core` appears on `docs/src/reference/vocabulary.md`.**
   - Enumerate the enums by parsing `crates/snora-core/src/*.rs`. There are 17 on `main` today, and the test asserts
     that count as a premise, so a parser that finds fewer fails.
   - Breakage: remove one name from the page.
2. **Every type in a public signature of `snora` (and `snora::design`) is nameable through `snora`.**
   - **Propose the mechanism in this unit's request before relying on it.** A source parse, or `rustdoc --output-format
     json` on nightly (say whether CI can run it).
   - Include a measured false-negative check: plant a public function returning a type that is not re-exported, and
     show the guard catches it.
   - It must catch the two names this audit found (`ShortcutRoute` and `TargetRole`) when Unit 1 is reverted.
3. **`#![forbid(unsafe_code)]` is present in every crate's `lib.rs`:** the six crates listed from the workspace
   members, not from a hard-coded list. Breakage: remove it from one.
4. **The rendered-surface identifiers are pinned by literal strings in the test,** independent of the constants and of
   the page.
   - Today `documented_identifiers_match_emitted_set` compares the page with the code, so a rename that edits both
     passes (request C, C3-I-17b).
   - Breakage: rename an identifier in the constant *and* the page.

## Unit 3 — guards on the book

5. **`check-migration-guides.sh` reads `migrations.md`'s "Jumping several minors" table:** every row's version has
   its guide file and its link. Breakage: a row whose guide does not exist.
6. **The book's `default = [...]` lines match `crates/snora/Cargo.toml`'s `default`:** a script, in the Docs job.
   Enumerate the lines with a regex over `docs/src` and `README.md`, not a fixed list of pages. Breakage: one page
   says `["widgets"]`.
7. **The budget pages' prose numbers match the last row of each CSV:** `binary-size-budget.md`, `build-cost-budget.md`
   and indicator 2's cell in `feature-gating-criteria.md`. A script that reads the last row and finds the stated
   numbers.
   - **Name in the request which numbers it checks**, since not every number on those pages is a last-row number.
   - Breakage: a stale `widgets_diff_bytes`.
8. **`consumer-requirements.md`: each `REQ-` entry has exactly one current-status line, and it comes first.**
   - The architect re-lays-out the register in the same 0.67 (current status first, labelled with its release, history
     beneath). **Agree the exact form of the line with the architect's commit before writing the check.**
   - Breakage: a second current-status line.
9. **The built-links check reads fragments:** a link to `page.html#anchor` fails if the built page has no element
   with that id. Extend `scripts/check-built-links.py`. Report how many existing links it finds broken on today's
   tree (request D found one). The architect fixes the book's, then the check goes in.

## Unit 4 — two measurements (no change promised)

- **Do `responsive_body`'s unit tests run in CI?** Read `ci.yaml` and the targets script, then run the jobs' own
  commands with `--list` and look for them. Say yes or no, with the command.
- **A form control whose single label is wider than the room:** what each of the five draws at a window narrower than
  that label, on both paths and both renderers (wraps, clips, overflows or squeezes), from pixels. RFC-138's Unit 0
  probe is the starting point. **Report only.** I rule on what, if anything, changes.

## Acceptance criteria

1. Each guard shown able to fail, and each on CI.
2. Every `ci.yaml` command run locally before each push, after the review approves it.
3. **Not yours:** the register's re-layout, the book's broken anchors, and the CHANGELOG lead (the architect).
