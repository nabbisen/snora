# Developer Handoff — RFC-057 typography documentation

**Governing RFC.** [RFC-057](../../done/057-typography-is-undiscoverable.md)
**Status.** Inherited from RFC-057 — Implemented (v0.33.1).
**Release target.** 0.33.1 (patch). Ships alone.
**Implementation units.** One. **Documentation only — no executable code.**

---

## 1. Task title

Make snora's typography vocabulary discoverable: two new pages, a corrected
README sentence, and the removal of a review-checklist item that instructs
people not to use line-height.

## 2. Purpose

snora has a six-role typography scale carrying size **and** line-height. It is
tested, demonstrated in the design workbench, and **fully usable today with no
change to snora**.

Nothing tells a developer it exists. There is no typography page, typography
is absent from `SUMMARY.md`, the consumer accessibility guide says nothing
about text, and the README's only mention is a disclaimer. Applications built
on snora therefore have flat, uniform text — a documentation outcome, not a
capability gap.

## 3. Background — read first

- `rfcs/done/057-typography-is-undiscoverable.md` in full, especially
  §"The honesty constraint".
- `crates/snora-design/src/typography.rs` — the six roles and their doc
  comments, which already describe each role's purpose well. **Reuse that
  wording rather than inventing new descriptions.**
- `docs/src/design/tokens.md` — the existing page typography is currently
  buried inside.
- `docs/src/contributing/documentation-test-policy.md` — **no bare `rust`
  fences in `docs/src`**; `mdbook test` enforces it and it has bitten several
  releases.

## 4. The trap — the teaching snippet is not verified by anything

RFC-057 contains this, and it is the centrepiece of both pages:

```rust,ignore
use iced::widget::text::LineHeight;

iced::widget::text("wrapping prose")
    .size(snora::design::style::text::body_size(&tokens))
    .line_height(LineHeight::Relative(tokens.typography.body.line_height))
```

**`rust,ignore` fences are not compiled by `mdbook test`.** If you transcribe
this and it is wrong, we ship a documentation page teaching code that does not
build — which is precisely the defect class this RFC exists to fix.

**Compile-probe it yourself.** Add it temporarily to an example that already
has `snora::design` in scope (`examples/design_workbench` works), `cargo
check`, then revert. Include the probe and its output in your evidence, and
confirm the tree is clean afterwards.

It did compile when RFC-057 was written. Verify it still does rather than
trusting that.

## 5. What to write

### `docs/src/design/typography.md` — the vocabulary

Registered in `SUMMARY.md` under **Snora Design**, beside Tokens / Buttons /
Cards.

The six roles, their purposes, and their default values:

| Role | Size | Line height | Purpose |
|---|---|---|---|
| `body` | 16.0 | 1.4 | ordinary explanatory text |
| `body_small` | 14.0 | 1.35 | secondary metadata, compact help |
| `label` | 14.0 | 1.2 | button, field and chip labels |
| `title` | 18.0 | 1.3 | card / dialog / notice title |
| `heading` | 24.0 | 1.25 | page or section heading |
| `display` | 32.0 | 1.2 | rare major page title |

Values are `Typography::default_roles()`, shared by all built-in presets. State
them as **defaults a preset supplies**, not as constants — an application
supplying its own `Tokens` changes them, and the page should not imply
otherwise.

**The honesty constraint applies here.** State plainly that snora's own prefab
widgets currently use **two** of these roles — `label` and `body` — and that
the rest exist for the application's own text. Do **not** write a page that
implies snora renders a full type hierarchy. It does not, and a page that says
otherwise is the next RFC-048.

### `docs/src/guides/readability.md` — the task

Registered in `SUMMARY.md` under **Guides**, beside Accessibility / Direction.

Task-oriented, not a vocabulary restatement:

- how to pick a role for a given piece of text;
- **why line-height matters for prose and not for labels** — this is the point
  a reader most needs and the least obvious;
- the floor, promoted from the contributor checklist: *text in notices, labels
  and help content uses at least `body` or `body_small`, never a custom size
  below 12 logical pixels*;
- the compile-verified snippet (§4).

**Link `design/typography.md`; do not restate the table.** Duplication drifts —
RFC-045 declined to duplicate for exactly this reason.

`docs/src/guides/accessibility.md` gains **one line** pointing here. Not a
section.

### `README.md`

Its typography sentence currently reads as a disclaimer, and the paragraph
advertising Snora Design scopes what design supplies to "chrome colours as of
v0.26". Correct the scoping so typography is not presented as outside what
Snora Design offers.

**One sentence.** This is not a rewrite of the README's design paragraph.

### `docs/src/contributing/accessibility-checklist.md`

Replace:

> Line-height multipliers (stored in `TextRole.line_height`) are used where the
> rendering path supports them. **In iced 0.14, line-height is
> vocabulary-only; the limitation is documented.**

It is not vocabulary-only — see §4. The replacement should say line-height **is
available** and, honestly, that snora's own widgets do not yet apply it, with
the follow-up named. Do not replace one false statement with an aspirational
one.

### `crates/snora-design/src/typography.rs`

Line 7 says line-height configuration "happens in `snora-widgets`". Doubly
stale: it never did, and since v0.33.0 `snora-widgets` does not hold the style
layer at all. **Doc comment only** — no code changes in this task.

## 6. Explicit non-change scope

Do **not**:

- **Change any executable code.** Doc comments only, and only the one above.
- **Change which roles the prefab widgets use.** That is an appearance change,
  deliberately deferred pending orbok's Phase 4 evidence (RFC-057
  §"Deliberately deferred").
- **Fix the notice's title rendering at `label_size`.** Same reason. Mention it
  in the typography page's honesty statement if useful; do not change it.
- **Add `*_line_height()` style helpers.** Line-height is reachable today; a
  convenience wrapper is a later decision.
- **Add font weight or family.** Separate design question.
- **Restructure `SUMMARY.md`** beyond registering the two new pages. Grouping
  Direction / Accessibility / Readability under one heading is recorded as
  optional and separable.

## 7. Required tests

```bash
mdbook build docs && mdbook test docs
cargo test -p snora --test render_semantics   # MUST pass unmodified
cargo doc --workspace --no-deps
cargo fmt --all --check
```

Plus the §4 compile probe, which nothing above covers.

## 8. Acceptance criteria

RFC-057 §Acceptance criteria 1–7. The two most likely to be skipped:

- **2** — the snippet is **compile-checked**, not transcribed (§4).
- **3** — the typography page states which roles snora's widgets actually use.
  A page that omits this reads as a promise.

## 9. Prohibited shortcuts

- Do not transcribe the snippet without compiling it (§4).
- Do not duplicate the role table into the readability guide.
- Do not replace the false checklist item with an aspirational one — say what
  is true today, including what snora does not yet do.
- Do not modify `render_semantics.rs`.

## 10. Required evidence

- Both new pages in full.
- The **compile probe and its output**, plus `git status --porcelain` showing
  the tree clean afterwards.
- `git diff --stat -- 'crates/**/*.rs'` proving doc comments only.
- Diffs of `README.md`, `accessibility-checklist.md`,
  `guides/accessibility.md`, `SUMMARY.md`.
- `mdbook build` / `mdbook test` output.

## 11. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/057-typography-is-undiscoverable/`. **State the
single entry-point path** in the completion summary.

**Requested review focus:** the honesty statement in the typography page.
Everything else is mechanical; whether that page promises more than snora
delivers is the judgement, and getting it wrong reproduces the exact defect
this RFC was raised to correct.
