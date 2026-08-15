# Developer Handoff — RFC-048 dialog card documentation

**Governing RFC.** [RFC-048](../../proposed/048-dialog-card-documentation-contradiction.md)
**Status.** Inherited from RFC-048 — Proposed.
**Release target.** 0.28.1 (patch). Ships alone.
**Implementation units.** One. **Documentation and doc-comments only — no
executable code changes.**

---

## 1. Task title

Correct six sites that promise a dialog card the default path does not draw,
make the `design`-path card discoverable from the dialogs guide, and record
the documentation rule whose absence caused this.

## 2. Purpose

A downstream team (arama) read
`crates/snora/src/overlay/dialog.rs`, whose first line says *"Dialog — the
centered modal card"*, found a function that centres content and does nothing
else, and filed a report. The behaviour is correct; the card they wanted
shipped in v0.27.0 via RFC-039. **The documentation is what is wrong.**

Their root-cause line is the task:

> The doc comment and the behaviour disagreeing is the actual defect; either
> half can move.

This handoff moves the documentation half.

## 3. Background — read first

- `rfcs/proposed/048-dialog-card-documentation-contradiction.md` in full.
- `.git-exclude/reviewed/arama-dialog-overlay-card/review-result.md` — the
  verification behind every claim below.
- `docs/src/design/engine-surfaces.md` §"The dialog card" — what actually
  ships on the `design` path, and the page you will be linking to.

Conventions: English only. **No bare `rust` fences in `docs/src`** — see
`docs/src/contributing/documentation-test-policy.md`; `mdbook test` enforces
it and it has bitten two recent releases.

## 4. The framing that must survive into the text

arama wrote that the docs promised a card "so consumers reasonably do not
check." At v0.25.0 `docs/src/guides/overlays.md:43` **already denied it** —
eleven lines below the line 32 gloss that promises it. Verified with
`git show 0.25.0:docs/src/guides/overlays.md`.

**Do not write this up as a consumer misreading the documentation.** The
accurate finding is that snora's own page contradicted itself, in one file,
eleven lines apart, for four releases. A document that disagrees with the
code can be resolved by reading the code; a document that disagrees with
*itself* cannot. Their behaviour was correct.

This matters for tone in `CHANGELOG.md` (§8 step 5), which is consumer-facing.

## 5. Change scope

| File | What |
|---|---|
| `crates/snora-core/src/layout.rs` | site 1 — **do this first**, see §6 |
| `crates/snora/src/overlay/dialog.rs` | site 2 — the module doc arama read |
| `crates/snora/src/render.rs` | site 3 + the dim line in the same table |
| `docs/src/guides/overlays.md` | sites 4 and 5, the dim line, **and §7** |
| `docs/src/reference/overlay-interaction-semantics.md` | site 6 + dim line |
| `docs/src/reference/architecture.md` | **dim line only** — see §6 warning |
| `docs/src/contributing/feature-gating-criteria.md` | the rule (§8 step 4) |
| `CHANGELOG.md` | `[Unreleased]` **Changed** |

## 6. The six card sites, verified at v0.28.0

| # | Site | Current text |
|---|---|---|
| 1 | `crates/snora-core/src/layout.rs:114` | `/// A centered modal card.` |
| 2 | `crates/snora/src/overlay/dialog.rs:1` | `//! Dialog — the centered modal card.` |
| 3 | `crates/snora/src/render.rs:16` | `//! 5. dialog            — centered card` |
| 4 | `docs/src/guides/overlays.md:32` | `A centered modal card. Snora paints the dim backdrop…` |
| 5 | `docs/src/guides/overlays.md:133` | `5. dialog             centered card` |
| 6 | `docs/src/reference/overlay-interaction-semantics.md:20` | `5. dialog         — centered card` |

Line numbers verified at v0.28.0 but **confirm each yourself** — they drift.

**Site 1 first, and independently.** It is the `AppLayout::dialog` field's
rustdoc on a **published crate**, so it reaches docs.rs unqualified with no
correcting text nearby. It is the highest-value line in this handoff and does
not depend on any other change.

**Do not invent wording.** Two sites already say it correctly; copy their
shape:

- `crates/snora-core/src/overlay.rs:38` — *"The dialog body content. The
  engine centers this in the window and paints the dim backdrop around it."*
- `crates/snora/src/render.rs:63` — *"Today's literal, unmodified: opaque
  black at 40% alpha, **no card**."*

**⚠ `docs/src/reference/architecture.md` has no card claim.** Its layer 5 is
a bare `dialog` (line ~135). It carries only the dim claim. Do not add a
"correction" to a claim that is not there.

## 7. The z-stack tables — the trap

Four tables document layer order. Since v0.27.0 they describe **neither** path
accurately, and the two claims fail in **opposite** directions:

- **"centered card"** — wrong on the default path, right on the `design` path.
- **"40 % black"** — right on the default path, wrong on the `design` path.

That symmetry is why neither has ever looked obviously wrong.

**Be precise about the dim.** RFC-039 left the **alpha unchanged at 40%** and
derived only the **colour** (`Color::WHITE` on dark presets, `Color::BLACK` on
light — `crates/snora/src/design/render.rs:39–48`). So `40%` is correct on
both paths and must not be "corrected"; `black` is what is path-specific.

Sites: `guides/overlays.md:132`, `reference/overlay-interaction-semantics.md:19`,
`reference/architecture.md:135`, `crates/snora/src/render.rs:15`.

## 8. Required implementation

### Step 1 — Site 1, alone

Correct `crates/snora-core/src/layout.rs:114`. One line. Commit-able on its
own; nothing else depends on it.

### Step 2 — The remaining five card sites

Per §6. Each states what the default path renders and that a token-styled
card is available via `snora::design::render`.

### Step 3 — The four z-stack tables

Per §7. Distinguish default from `design` for **both** claims. Keep the tables
readable — a layer-order table that grows a paragraph per row has failed.

### Step 4 — `docs/src/guides/overlays.md`, the discoverability fix

This is the finding, not a side-effect. At v0.28.0 the page says
**"snora is a positioner, not a styler"** and never mentions `design::render`,
RFC-039, or the card. A consumer with arama's exact problem is told the card
will never exist.

Required:

- State that a token-styled card **is** available via `snora::design::render`
  as of v0.27.0, with a link to `docs/src/design/engine-surfaces.md`.
- **Scope, do not delete, "positioner, not a styler."** It is correct for the
  default path and stays. It is being qualified, not withdrawn.

**Outcome test, not an edit test:** could a reader who arrives at this page
with an unreadable dialog over a photo grid find out that a card exists,
without already knowing to look in `docs/src/design/`? If not, the edit has
not done its job.

### Step 5 — The rule, in `feature-gating-criteria.md`

Two downstream reports three releases apart are the same omission. Add:

> **When a `design`-gated capability lands, every default-path page that
> states the capability is absent is part of the change scope.** Documenting
> the new behaviour in `docs/src/design/` is necessary and not sufficient: a
> consumer who never reads `design/` is left with the old denial, which now
> reads as a statement that the capability will not exist.

Match the file's existing section style.

### Step 6 — `CHANGELOG.md`

`[Unreleased]` → **Changed**. Name the contradiction; do not describe it as a
wording tidy-up. **Credit arama** — it is their finding. Observe §4's framing.

## 9. Explicit non-change scope

Do **not**:

- **Change any executable code.** Doc comments only. Both paths are correct.
- Add a card to the default path. That breaks the v0.25 rendering guarantee
  and RFC-037's gating invariant.
- Rename `snora-dialog-card`. It is wrong (RFC-048 F-5) and is **RFC-049**'s
  job — a minor bump, not this patch.
- Delete "positioner, not a styler" (§8 step 4).
- Add a caveat to every mention of `Dialog`. Six sites, four tables, one guide
  section, one rule. **A seventh caveat means the intent has been overshot.**
- Re-open RFC-039's design decisions.

## 10. Required tests

```bash
mdbook build docs && mdbook test docs
cargo test -p snora --test render_semantics    # MUST pass unmodified
cargo doc --workspace --no-deps
cargo test -p snora --lib --all-features
```

### On `cargo fmt` — read this, the standing gate is broken

**`cargo fmt --check` does not pass on a clean tree at v0.28.0**, and has not
for several releases. Verified: **82 hunks** workspace-wide, and **7 hunks**
under `-p snora`, **all of them in `crates/snora/tests/render_semantics.rs`** —
the file every recent handoff simultaneously requires to be left *unmodified*.
Those two instructions cannot both be satisfied. No CI workflow runs `cargo
fmt` at all, so nothing has caught it.

**Do not run `cargo fmt --all` as part of this task.** Reformatting is a
separate concern from a documentation patch, and it would touch the invariant
test file.

Use a **delta check** instead:

```bash
cargo fmt --check --all 2>/dev/null | grep '^Diff in' | sort > /tmp/fmt-before
# … make your changes …
cargo fmt --check --all 2>/dev/null | grep '^Diff in' | sort > /tmp/fmt-after
diff /tmp/fmt-before /tmp/fmt-after      # MUST be empty
```

Report the before/after counts in your review request. This is flagged to the
architect separately; it is not yours to fix here.

## 11. Acceptance criteria

RFC-048 §Acceptance criteria 1–6:

1. All six §6 sites describe what each path renders; site 1 corrected first.
2. `guides/overlays.md` names `snora::design::render`, links
   `design/engine-surfaces.md`, and scopes "positioner, not a styler".
3. All four §7 tables distinguish both paths for **both** claims, with the
   dim's alpha left correctly at 40% on both.
4. `feature-gating-criteria.md` carries the §8 step 5 rule.
5. `mdbook build docs` and `mdbook test docs` pass.
6. `git diff --stat -- 'crates/**/*.rs'` shows **doc-comment lines only**, and
   `render_semantics` passes unmodified.

## 12. Prohibited shortcuts

- Do not report an empty grep as the acceptance signal. **Several "40%" hits
  must survive** — the alpha is accurate on both paths. The criterion is that
  every surviving hit reads correctly for the path it describes.
- Do not fix site 1 by deleting the sentence. `AppLayout::dialog` needs a
  description; it needs a *true* one.
- Do not "correct" a card claim in `architecture.md` (§6).
- Do not modify `render_semantics.rs` for any reason, including formatting.
- Do not resolve the guide contradiction by deleting line 32 and keeping line
  43. Both need to describe both paths.

## 13. Compatibility and security

**Compatibility.** Documentation only. No API, no rendering, no gate rows. The
corrected text describes behaviour stable since v0.27.0.

**Security.** No new data flow, dependency, or integration.

## 14. Required evidence

- Diffs of every changed file.
- `git diff --stat -- 'crates/**/*.rs'` proving doc comments only.
- `render_semantics` output, plus `git diff --stat -- crates/snora/tests/`
  showing it is **empty**.
- `mdbook build` / `mdbook test` output.
- The re-run greps from RFC-048 §Testing, with a sentence per surviving hit
  explaining why it is correct.
- The fmt delta from §10.
- Your answer to §8 step 4's outcome test.

## 15. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/048-dialog-card-documentation-contradiction/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** §8 step 4 — whether `guides/overlays.md` now
answers arama's question for a reader who does not already know the answer.
The six site corrections are mechanical; that one is a judgement call.
