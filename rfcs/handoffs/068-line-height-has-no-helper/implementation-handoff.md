# Developer Handoff — RFC-068 line-height helpers

**Governing RFC.** [RFC-068](../../accepted/068-line-height-has-no-helper.md)
**Status.** Inherited from RFC-068 — Accepted (owner, 2026-08-19).
**Release target.** 0.38.0 — **minor**, because `snora-style` gains public
functions. **No rendering changes.**
**Implementation units.** One.

---

## 1. Task title

Add six line-height helpers to `snora-style::text`, put a two-axis compile
contract behind the helper set, and correct the three documentation passages
that say line-height has no helper.

## 2. Purpose

`TextRole` carries `size` and `line_height`. `snora-style::text` provides six
helpers for the first and none for the second.

The asymmetry is **visible in a single line of our own published example**,
which appears in both `typography.md` and `readability.md`:

```rust
    .size(snora::design::style::text::body_size(&tokens))
    .line_height(LineHeight::Relative(tokens.typography.body.line_height))
```

One half is a helper call; the other reaches through two struct fields and
constructs an iced enum by hand. We wrote that line ourselves and shipped it as
the recommended form.

`text.rs`'s module doc frames the omission as deliberate — "applying it is the
application's own call." The size is equally the application's own call and has
six helpers. The reasoning justifies providing neither or both.

**Evidence that it matters:** knotra measured **zero line-height occurrences
across three crates**, alongside 38 call sites below the 12px floor. They found
the size half because it has a floor *and* a helper; the line-height half only
as an absence. orbok appears to be in the same position.

## 3. All three questions are decided — do not reopen them

### Q-1 — the return type is `LineHeight::Relative`, not `f32`

The size helpers already return an iced type (`Pixels`, not `f32`), and the
multiplier in `TextRole` is *defined* as relative. `f32` would make the caller
re-state a fact the token already fixed, and would leave every call site free to
pass a relative multiplier where an **absolute pixel height** is expected —
`1.4` is a plausible-looking absolute line height and a catastrophic one.

Verified against the pinned iced 0.14, so you do not need to re-check:

- `LineHeight` is `iced_core::text::LineHeight`, re-exported at
  `iced::widget::text::LineHeight`. Variants `Relative(f32)` and
  `Absolute(Pixels)`. It derives `Debug` and `PartialEq`, so `assert_eq!`
  works on it directly.
- **`snora-style` already depends on `iced`.** No new dependency; no change to
  the workspace dependency graph.
- **`snora-design` is not touched.** The `f32` stays in the token and only the
  bridge converts. The iced-free constraint on `snora-design` never comes into
  play — if your change reaches that crate, you have gone wrong.

### Q-2 — this does **not** oblige snora's own widgets to apply line-height

It does not, and **you must write that down**, because "why don't we use our own
helpers?" is the next question a reader has and the answer must not read as
"nobody noticed." The answer: applying them changes the rendering of shipped
primitives, and is gated on orbok's Phase 4 evidence.

### Q-3 — a contract for our half, documentation for theirs

The question is *who gets noticed*, and it splits:

| Audience | What can fire | Mechanism |
|---|---|---|
| A snora dev adding a role or a `TextRole` field | **A compile error** | §4 |
| A snora dev wiring a helper to the wrong role | **A test failure** | §4 |
| An application developer who never sets line-height | **Nothing we ship** | §6 |

## 4. The contract — two axes, both compile-enforced

**The enabling fact:** `Typography` and `TextRole` are **plain structs; neither
is `#[non_exhaustive]`**, unlike `Tokens` and `Palette`. Exhaustive
destructuring therefore binds from *any* crate, so this test lives in
`snora-style` beside the helpers it constrains. `Palette::usages()` had to be
pushed down into `snora-design` for exactly the opposite reason (RFC-063).
**Do not add `#[non_exhaustive]` to either struct** — it would break this test
from outside `snora-design` (`E0638`) and defeat the whole mechanism.

One test in `crates/snora-style/src/text.rs`:

```rust
// Exhaustive on the ROLE axis, deliberately. A seventh role fails to compile
// (E0027) until it is listed here with both helpers. Do NOT add `..` — the
// compiler will suggest it on a missing-field error, and that suggestion
// silently defeats the entire point of this test.
let Typography { body, body_small, label, title, heading, display } = t.typography;

for (name, role, size, line_height) in [
    ("body", body, body_size(&t), body_line_height(&t)),
    // ... one row per role, all six
] {
    // Exhaustive on the FIELD axis, deliberately. A third TextRole field fails
    // to compile until it is either given a helper or explicitly declined
    // here. Do NOT add `..`.
    let TextRole { size: want_size, line_height: want_line_height } = role;
    assert_eq!(size.0, want_size, "{name}_size returns the wrong role");
    assert_eq!(
        line_height,
        LineHeight::Relative(want_line_height),
        "{name}_line_height returns the wrong role",
    );
}
```

The role axis catches a new role arriving untooled. **The field axis catches
this RFC's own defect class** — a token field growing while the bridge silently
does not follow. Be clear-eyed about what it does not do: it could not have
caught the present instance, since both fields have existed since v0.20 and the
test is written after the fact. It fires on the next one.

**The equality assertions are not ceremony.** Six near-identical one-liners is a
copy-paste bug waiting to happen. `title_line_height` returning `body`'s
multiplier would pass any "the helper exists" check and is not visible by
reading. Name the role in the failure message so the failure locates itself.

### It replaces a live instance of the same defect

`text.rs`'s existing `sizes_are_positive_and_monotonic` (lines 64–83) builds a
**hand-written array of six**. A seventh role would not break it. That is the
RFC-063 hand-maintained-list shape sitting in the file you are editing.

Fold it in: keep the monotonicity assertion (it is a real check, and it is
ordering, not coverage), but drive it from the exhaustively-destructured
bindings rather than a literal array.

## 5. The value bounds — one goes in, one is refused

**In: a definitional sanity bound.** Assert `line_height > 1.0` for all six
roles, below which lines overlap. `typography.rs`'s doctest already asserts it
for `body` alone. Comment it as a **sanity bound, not an accessibility
threshold** — those words, because the next reader will assume otherwise.

**Refused: a leading floor.** WCAG 2.1 SC 1.4.12 (Text Spacing) is the number
that would be reached for, and it does not say what it appears to: it requires
that content **survive a user setting** line height to 1.5, not that a design
system ship 1.5. Our `body` is 1.4 and that is not a violation. Deriving a
shipped floor from it would fabricate a threshold from a misread standard —
RFC-058's defect inverted, and this project has made that exact error once
already. **Do not introduce a leading floor anywhere**, in code or in prose.

## 6. The documentation half, which is not a fallback

Nothing snora ships can fire in an application's source — no lint, no macro, no
build script reaches it. **knotra's zero-occurrences finding was not catchable
by any contract available to us.** Two levers exist:

**Lever 1 — API shape, which is the main one.** `body_line_height` sitting
beside `body_size`, same module, same naming, is discoverable by autocomplete at
the point of use. That is the closest thing to "noticed while implementing" an
application developer gets from us, and it is the main reason the helpers are
worth adding at all. Mirror the size helpers exactly: same order, same
`#[must_use]`, same doc-comment shape.

**Lever 2 — three specific passages**, all of which currently assert the
absence:

1. **`crates/snora-style/src/text.rs`, module doc (lines 1–11).** It says
   "these helpers cover size only" and explains why line-height is not wrapped.
   That explanation is now false. **Keep the sentence that is still true** —
   snora's own prefab widgets do not apply it — and attach Q-2's reasoning to
   it.
2. **`docs/src/design/typography.md`, lines 52–56.** "Line-height is not
   wrapped in a helper — read the multiplier straight off
   `tokens.typography.<role>.line_height`" is a direct contradiction after this
   change.
3. **`docs/src/guides/readability.md`.** Note: it **already has** a leading
   section — *"Why line-height matters for prose, and not for labels"* (line
   29). Do not add a second one. What it lacks is a pointer to the helpers,
   because there were none. Its `## Applying a role` example (line 74) is the
   other half of the asymmetry quoted in §2.

   **Delete line 72 while you are there** — *"Compile-checked against the
   pinned iced 0.14:"*. It sits directly above the `rust,ignore` fence you are
   rewriting, and it is false: nothing compiles that block. This is the one
   line RFC-069 hands to you rather than keeping, because leaving an untrue
   sentence on top of a block you rewrote is worse than the scope discipline
   that would preserve it. Replace it with nothing, or with a plain lead-in
   that claims no verification.

Both book examples become:

```rust
iced::widget::text("wrapping prose")
    .size(snora::design::style::text::body_size(&tokens))
    .line_height(snora::design::style::text::body_line_height(&tokens))
```

The `use iced::widget::text::LineHeight;` line in both examples is then unused —
remove it.

## 7. Change scope

| File | Change |
|---|---|
| `crates/snora-style/src/text.rs` | Six helpers; module doc; the §4 test replacing the hand-written array |
| `docs/src/design/typography.md` | Lines 52–56 and the example |
| `docs/src/guides/readability.md` | The `## Applying a role` example; a pointer from the existing leading section |
| `CHANGELOG.md` | `[Unreleased]` → **Added** |

**No facade change.** `crates/snora/src/design.rs:39` re-exports the whole
`snora_style::text` module, so the new functions arrive at
`snora::design::style::text::*` with no edit. Do not add per-item re-exports.

## 8. Explicit non-change scope

- **Do not make snora's own widgets apply line-height.** Q-2. It is a rendered
  change to shipped primitives, it collides with orbok's deferred typography
  assessment, and it is gated on their Phase 4 evidence.
- **No change to any `line_height` or `size` value.**
- **No new `TextRole` field, no change to `Typography`.** RFC-036's covenant
  freezes both.
- **No `#[non_exhaustive]` added to either struct** — see §4.
- **No leading floor** — §5.
- **Out of scope — [RFC-069](../../proposed/069-book-examples-cannot-be-compiled.md)
  owns it.** The book's 110 `rust,ignore` fences cannot be compiled at all,
  because the book has no library path. **Do not add reasons to these fences,
  do not change any fence tag, and do not touch
  `documentation-test-policy.md`.** One single-line exception is carried into
  scope below, because it sits inside a block you are already rewriting.

## 9. Required tests

1. The §4 exhaustive test, both axes, no `..` in either pattern.
2. The §5 sanity bound over all six roles.
3. Monotonicity preserved, driven from the destructured bindings.

**A perturbation demo, not a green run — this is the requested review focus.**
A guard that has never fired is unproven. Demonstrate **both** axes:

- Add a seventh role to `Typography`; capture the `E0027`; restore.
- Add a third field to `TextRole`; capture the `E0027`; restore.
- Point `title_line_height` at `body`; capture the assertion failure naming
  `title`; restore.

Three captured failures, three restores. `git status` clean afterwards.

## 10. Required evidence

Under `evidence/`:

- `cargo test -p snora-style` — full pass
- `cargo test --workspace` — full pass
- the three perturbation captures, one file each, each showing the error **and**
  the restored green run
- `mdbook build docs && mdbook test docs`
- `git diff --stat -- 'crates/snora/src' 'crates/snora-widgets' 'crates/snora-design'`
  — **expected empty**; this is the "no rendered output changes" check
- `cargo doc -p snora-style --no-deps` — no new warnings

## 11. Acceptance criteria

1. Six `<role>_line_height` helpers exist, one per role, returning
   `iced::widget::text::LineHeight::Relative`, mirroring the size helpers in
   name, order, `#[must_use]`, and doc-comment shape.
2. The §4 test exists with both patterns exhaustive and neither carrying `..`.
3. All three perturbation demos captured, with restores.
4. `sizes_are_positive_and_monotonic`'s hand-written array is gone.
5. All six roles assert `line_height > 1.0`, commented as a sanity bound and not
   an accessibility threshold. No leading floor anywhere.
6. The three passages in §6 no longer assert the absence of a helper; the true
   statement about snora's own widgets survives, with Q-2's reasoning attached.
   `readability.md`'s false "Compile-checked" line is gone, and **no fence tag
   anywhere has changed**.
7. **No rendered output changes** — the `git diff --stat` above is empty.
8. `render_semantics` passes unmodified.
9. `CHANGELOG.md` `[Unreleased]` records the addition under **Added**.

## 12. Required review-request format

Package under `.git-exclude/review-request/068-line-height-has-no-helper/`, with
`README.md` as the entry point and evidence under `evidence/`. Report relative
paths. **State the single entry-point path** in the completion summary.

**Requested review focus:** the perturbation demos (§9). Everything else here is
mechanical; the demos are the only thing that distinguishes a contract from a
test that happens to pass.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.** If §4's test cannot be written as specified, stop and say so rather than
substituting a weaker check.
