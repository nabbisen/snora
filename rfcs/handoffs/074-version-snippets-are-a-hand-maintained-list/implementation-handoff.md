# Developer Handoff — RFC-074 version snippets

**Governing RFC.** **RFC-074** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-074 — Accepted (owner, 2026-08-20).
**Release target.** 0.38.3 — **patch.** One doc comment in a shipped crate; the
rest is documentation and one new script. No API.
**Implementation units.** Two: correct the snippets, then replace the rule.
**Sequence:** with [RFC-075](../075-the-gate-register-and-the-frozen-surface-are-both-wrong/implementation-handoff.md);
either order.

---

## 1. Purpose

`release-process.md:126` names **`install.md` and `icons.md`** by hand. Those
two are current. Everything else has drifted since the line was written.

**The worst instance ships inside the crate:** `crates/snora/src/lib.rs:72`
tells a reader of snora 0.38.2's own docs.rs page to write
`snora = { version = "0.25" }` — a version predating `snora-style`, the border
repair, the modal dim and the line-height helpers.

## 2. Unit 1 — correct every stale snippet

| File:line | shows | write |
|---|---|---|
| `crates/snora/src/lib.rs:72` | `0.25` | `0.38` |
| `README.md:44` | `0.37` | `0.38` |
| `docs/src/design/feature-flags.md:78,81,84,87,90` | `0.28` ×4, `0.31` | `0.38` |
| `docs/src/design/feature-flags.md:99` | `snora-design = "0.25"` | `0.38` |
| `docs/src/reference/widgets.md:23` | `0.6` | `0.38` |

**Re-derive this table before trusting it.** Unit 2's check should produce it;
if the two disagree, the check is what you ship and this table is what is wrong.

Snippets carry the **minor**, not the patch — `snora = "0.38"` is a caret range
covering 0.38.2. Do not write `0.38.2` anywhere.

## 3. Unit 2 — replace the rule

**Q-1 ruled: a script in `scripts/`, run manually, not a CI gate.** Same shape
and same reasoning as `check-built-links.py` (RFC-073): committed, runnable,
inventoried in `scripts/README.md`, and **not wired into CI** — RFC-064's
precedent is that the audit and the written rule ship before a gate is pointed
at anything.

**Q-2 ruled: derive the expected minor from `[workspace.package].version` in
`Cargo.toml`.** A check carrying its own hard-coded expected version would be
this exact defect one level down, and would go stale the release after it is
written.

**Q-3 ruled: it must cover `crates/**/*.rs` doc comments.** That is where the
worst instance is and the only one that ships inside the artifact. A check that
scans `docs/` only would have missed the finding that prompted the RFC.

Then replace `release-process.md:126` with a line that **invokes the check
instead of naming files.**

## 4. The exclusion set is the actual design work

A sweep that rewrites history destroys findings. These must **never** be
touched, and the check must be silent on them:

| Excluded | Why, in one line |
|---|---|
| `docs/src/guides/migration-*.md` | they document what a version *was* |
| `CHANGELOG.md` | same |
| `rfcs/**` | RFC-051 quotes a consumer's then-current `0.25`; **RFC-056 quotes `snora-widgets = "0.6"` as the stale instruction it was reporting** — rewriting it deletes the finding |

**A check that cannot tell a live snippet from a quoted one is worse than no
check.** It will either cry wolf every release or be silenced, and a silenced
check is indistinguishable from one that never existed. Write the exclusions
into the script with those reasons as comments, not into a wrapper invocation
someone can forget.

## 5. Required tests — the demo runs both directions

1. **It fires:** stale one live snippet, run the check, see that file:line
   named, restore.
2. **It stays quiet:** confirm the check reports nothing for the excluded
   historical references — specifically `rfcs/done/056-remove-the-style-shims.md`
   and one migration guide, by name, in the evidence.

**Direction 2 is not optional.** A check that flags history is the failure mode
that gets checks disabled, and this project has no way to notice a check that
someone stopped running.

## 6. Explicit non-change scope

- **No CI gate.**
- **No code.** `crates/snora/src/lib.rs` is a doc comment; nothing else in
  `crates/` is touched.
- **Do not rewrite any migration guide, CHANGELOG entry, or RFC.**
- **Do not write patch versions** into snippets (§2).

## 7. Required evidence

- The check's output on the current tree, before and after Unit 1
- Both perturbation demos (§5), with restores
- `git diff --stat -- crates/` — doc comment only
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py` clean
- `cargo doc -p snora --no-deps` — the corrected `lib.rs` snippet renders

## 8. Acceptance criteria

1. Every location in §2 states the current minor; none states a patch.
2. `release-process.md` invokes the check and names no files.
3. The check derives its expected minor from `Cargo.toml` and scans crate doc
   comments as well as `docs/`.
4. Exclusions are in the script, each with its reason.
5. Both demo directions captured, including the silent-on-history one.
6. Script committed, inventoried in `scripts/README.md`, not in CI.
7. `CHANGELOG.md` `[Unreleased]` under **Fixed**.

## 9. Required review-request format

`.git-exclude/review-request/074-version-snippets-are-a-hand-maintained-list/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: the exclusion set and the silent-on-history demo.**
The replacements are mechanical; the boundary between a live snippet and a
quoted one is the whole design.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
