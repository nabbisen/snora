# Developer Handoff — RFC-081 assert the 12px text floor

**Governing RFC.** **RFC-081** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-081 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.2 — **patch.** One test and documentation. **No values.**
**Implementation units.** One.

---

## 1. Purpose

`readability.md:75` — *"**The floor is 12 logical pixels.** Nothing else."* —
and nothing asserts it. Its two neighbouring mandatory floors both are:
the 24px pointer target per role **and** padding step
(`snora-design/src/tests.rs:312`), and the contrast thresholds as a **compile
error** via `Palette::usages`.

`Typography` is a plain struct with public `f32` fields, so
`tokens.typography.body.size = 8.0;` compiles today.

## 2. Q-1 ruled — presets only. No validator.

Assert that every role in all four built-in presets is ≥ 12.0. **Do not add a
public `check_floor()`-shaped helper.** Nobody has asked for one, the owner's
stated prior is to keep snora simple, and a helper nobody calls is a third thing
to keep true.

## 3. Q-2 ruled — cite the guide, not a standard

**12px is snora's own rule, not a WCAG number.** SC 1.4.4 is about *resize*, not
a minimum size. The assertion's failure message must cite
`docs/src/guides/readability.md` and **must not name a WCAG criterion**. This
project has published a misattributed threshold before; do not add another.

## 4. State the limit, in the test and in the docs

**The assertion proves our built-in presets comply. It cannot constrain a
consumer's own `Tokens`** — those fields are public and RFC-036's covenant
freezes that surface, so `body.size = 8.0` in an application is unreachable by
any test we ship. Enforcing at construction would mean private fields, a
breaking change to a frozen surface, and is **not** in scope.

Write that limit into the test's doc comment **and** into `readability.md`
beside the floor. An assertion that proves less than a reader assumes is how
"verified" stops meaning anything.

## 5. Required tests

The assertion, plus **a perturbation demo**: drop one preset role below 12,
watch it fail naming **both the preset and the role**, restore. `git status`
clean afterwards.

Name the test so its subject is obvious next to its neighbour
(`pointer_target_height_meets_24px_for_every_role_and_padding_step`).

## 6. Explicit non-change scope

- **No preset value changes.** Every role is 14–32 today; the test passes on
  arrival. If one does not, stop and report — that is a finding.
- **No field made private, no `#[non_exhaustive]` added.**
- **No public validator** (§2).
- **No WCAG citation** (§3).

## 7. Required evidence

- The perturbation capture and the restore
- `cargo test -p snora-design` and `cargo test --workspace --all-features`
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py`
- `git diff --stat -- crates/snora-design/src/presets` — **expected empty**

## 8. Acceptance criteria

1. Every role in all four presets asserted ≥ 12.0.
2. Failure message names preset and role, cites `readability.md`, names no WCAG
   criterion.
3. The limit — built-in presets only — stated in the test doc and in
   `readability.md`.
4. Perturbation demo captured and restored.
5. No preset value changed; no validator added.
6. `CHANGELOG.md` `[Unreleased]` under **Fixed**, crediting tekstide.

## 9. Required review-request format

`.git-exclude/review-request/081-the-12px-text-floor-is-asserted-nowhere/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: §4's limit statement.** The test is four lines. Whether
a reader can tell what it does and does not cover is the whole value.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
