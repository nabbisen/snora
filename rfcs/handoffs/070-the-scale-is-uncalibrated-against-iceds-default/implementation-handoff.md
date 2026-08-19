# Developer Handoff — RFC-070 the scale against iced's default

**Governing RFC.** [RFC-070](../../accepted/070-the-scale-is-uncalibrated-against-iceds-default.md)
**Status.** Inherited from RFC-070 — Accepted (owner, 2026-08-19).
**Release target.** 0.38.1 — **patch, documentation only**, *unless* Q-1
concludes a value must change, in which case **stop and report** (§4).
**Implementation units.** Two: the measurement (Q-1), then the documentation.
**Sequence:** do **RFC-071 first**.

---

## 1. Task title

State snora's typography scale against iced's actual default line-height, and
fix the guidance that assumes applying a role is always an improvement.

## 2. Purpose

**iced 0.14 renders text at `Relative(1.3)` when `.line_height()` is never
called.** `Format::default()` sets `line_height: LineHeight::default()`
(`iced_core-0.14.0/src/widget/text.rs:290`); that default is `Relative(1.3)`
(`iced_core-0.14.0/src/text.rs:215`). **Verify both line numbers against the
pinned source before writing anything** — they are the whole basis of this RFC.

Our six roles were calibrated against each other and never against that
baseline:

| role | ours | vs 1.3 | effect of applying the helper |
|---|---|---|---|
| `body` | 1.4 | +0.10 | more air — the intended benefit |
| `body_small` | 1.35 | +0.05 | slightly more air |
| `title` | 1.3 | **0** | **nothing at all** |
| `heading` | 1.25 | −0.05 | **less air than doing nothing** |
| `label` | 1.2 | −0.10 | less air |
| `display` | 1.2 | −0.10 | less air |

Found by **orbok**, who adopted only `body` and `body_small` for exactly this
reason and told us why.

## 3. The live defect

`docs/src/guides/readability.md`:

> The practical rule: **apply line-height to anything that might wrap**
> (`body`, `body_small`, and any `title`/`heading` text that isn't guaranteed
> short) …

Of the four roles that sentence names: `body` helps, `body_small` helps
slightly, **`title` does nothing**, and **`heading` gives wrapped prose less
air than ignoring the advice.** The rule is written as though applying a role's
line-height is uniformly an improvement. Against the real renderer default it is
not.

## 4. Q-1 is a measurement, not a judgement — and it can stop this work

**Does `heading` at 1.25 need to change?**

RFC-036 permits changing a preset value **only where a test proves an
accessibility defect**, recorded as **Fixed**. So this is not an opinion call.

**Produce the demonstration:** a wrapped multi-line heading at 24px, rendered at
`1.25` and at `1.3`, and state what actually differs — line-box height,
inter-line gap in pixels, and whether anything legibility-relevant changes at
that size.

**The expected answer is "no change needed."** Tightening leading as size grows
is ordinary typography and is *why* the ladder tightens; a 24px heading at 1.25
is 30px of line box, which is not a crowded line. If that is what you find, the
defect is **entirely in the guidance** and this stays documentation-only.

**If you find otherwise — stop.** A value change is a rendered change for every
application applying that role, which makes it a **minor (0.39.0)**, needs the
RFC-036 carve-out's required order (assertion added and failing *first*), a
migration note, and an RFC-067 re-check line. **Do not carry it in a patch
release.** Report and wait.

## 5. Q-2 and Q-3 are decided

**Q-2 — state the baseline once; let the table carry the rest.** Do not write
six per-role rules into the prose. Add iced's `Relative(1.3)` to
`typography.md`'s role table — a column, or a stated baseline immediately above
it — cited to the source file, so every value is readable against it. Then fix
`readability.md`'s practical rule so it no longer implies uniform improvement.

The honest short form, if you need one: **applying `body` or `body_small` adds
air; `title` matches the default; the rest remove air deliberately, because
larger text needs less.** That last clause matters — the tighter roles are not
a defect, and the docs must not read as an apology for them.

**Q-3 — yes, `TextRole`'s own doc gets it.** `crates/snora-design/src/typography.rs`
says *"Line-height multiplier (e.g. `1.4`), relative to `size`"* — true, and
silent on what a reader gets by not setting it at all. One clause fixes it.

## 6. `title_line_height` restates the default — say so where a caller reads it

RFC-068 shipped six helpers a week ago, one of which has **no observable effect
on any surface**. Put that in `title_line_height`'s own doc comment in
`crates/snora-style/src/text.rs`, not only in prose.

**Do not remove the helper.** Six helpers stay six: the two-axis contract in
`text.rs` enforces one helper per role, and symmetry is worth more than deleting
a redundant call. A caller who writes `title_line_height` should learn what it
does, not find it missing.

## 7. Change scope

| File | Change |
|---|---|
| `docs/src/design/typography.md` | The baseline, in or above the role table, cited to source |
| `docs/src/guides/readability.md` | The practical rule (§3) |
| `crates/snora-style/src/text.rs` | `title_line_height`'s doc comment (§6) |
| `crates/snora-design/src/typography.rs` | `TextRole::line_height`'s doc (§5) |
| `CHANGELOG.md` | `[Unreleased]` → **Fixed** |

## 8. Explicit non-change scope

- **No value changes without §4's demonstration and a ruling from the
  architect.** Not `heading`, not any other role.
- **No re-tuning of the ladder.** The roles' relationship to each other is
  deliberate and is not what this RFC questions.
- **No new role, no new field, no helper added or removed.** RFC-036 freezes the
  first two; §6 covers the third.
- **No line-height floor** — RFC-068 refused one and that refusal stands.
- **Do not touch `VISIBILITY_FLOOR` or `engine-surfaces.md`** — that is
  RFC-071, and it lands first.

## 9. Required tests

Documentation-only work, so the bar is different: **no test can catch this
class**, which is the point worth noticing. What is required:

1. `mdbook build docs && mdbook test docs`.
2. The six values in any table you write are **read from
   `Typography::default_roles()`**, not typed from this handoff.
3. If §4's demonstration produces figures, they ship as evidence whether or not
   they change anything.

## 10. Required evidence

- §4's demonstration, with its method and its conclusion stated plainly —
  **including "no change needed", which is a result, not an absence of one**
- The two iced source citations, verified against the pinned version
- `cargo test --workspace --all-features`
- `mdbook build docs && mdbook test docs`
- `git diff --stat -- 'crates/snora-design/src/presets'` — **expected empty**

## 11. Acceptance criteria

1. iced's `Relative(1.3)` default is stated in `typography.md`, cited to the
   source file, with every role's value readable against it.
2. `readability.md` no longer implies applying a role's line-height is
   uniformly an improvement, and does not read as an apology for the tighter
   roles.
3. `title_line_height`'s doc comment says it restates the renderer default.
4. `TextRole::line_height`'s doc says what a reader gets by not setting it.
5. Q-1 answered **with a demonstration**; if it concludes a value must change,
   nothing shipped and the finding reported instead.
6. No preset value changed. Six helpers still exist.
7. `CHANGELOG.md` records it under **Fixed**, crediting orbok.

## 12. Required review-request format

`.git-exclude/review-request/070-the-scale-is-uncalibrated-against-iceds-default/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus:** §4's demonstration and the conclusion drawn from
it. Everything else is transcription; that is the only judgement in the task,
and getting it wrong in the permissive direction means a rendered change in a
patch release.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
