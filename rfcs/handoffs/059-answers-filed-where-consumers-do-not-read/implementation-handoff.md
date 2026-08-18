# Developer Handoff — RFC-059 answers filed where consumers do not read

**Governing RFC.** [RFC-059](../../done/059-answers-filed-where-consumers-do-not-read.md)
**Status.** Inherited from RFC-059 — Accepted (owner, 2026-08-17).
**Release target.** 0.34.0, alongside RFC-058. **Documentation only** — doc
comments are the sole `crates/` change.
**Implementation units.** One. Independent of RFC-058; no ordering between them.

---

## 1. Task title

Re-scope the `BLOCKED` focus claim at all four sites, document `FocusTokens`'
present-day audience, publish a consumer-facing token-surface stability
statement, and widen the existing documentation-scope rule to cover standing
answers.

## 2. Purpose

Two findings from tekstide, neither about missing capability. Both are **answers
snora already has, filed where a consumer does not reach.** The second cost an
adoption decision: they named a `snora-design`-scoped stability statement as
*"changing our calculus more than any feature would"*, and declined partly for
want of one that **already exists** in `contributing/api-governance.md`.

## 3. Three open questions — answered here, do not re-litigate

**Q-3 (grepped, so you don't have to).** `BLOCKED` appears at exactly three
sites, and the over-scoped `FocusTokens` framing at a fourth:

| Site | Nature | Action |
|---|---|---|
| `contributing/semantic-accessibility.md:131` | the instruction, plus the audience list at 119–124 | **re-scope** |
| `design/iced-style-bridge.md:76–83` | **consumer-facing**, repeats the same claim *and* the "will be wired when iced exposes focus state" framing | **re-scope — see §4** |
| `contributing/accessibility-checklist.md:192` | generic definition of the `BLOCKED` severity label as a category | **leave alone** — correct and general |
| `contributing/accessibility-checklist.md:26` | "`FocusTokens` exist for future iced versions and custom widgets" | **re-scope** |

**No per-primitive checklist has recorded a `BLOCKED` row yet**, so there is no
inherited-claim cleanup. Report this as Q-3's answer; do not re-derive it, but
do re-run the grep once to confirm nothing landed since.

**Q-1 — do not write a fifth rule.** The rule exists:
`contributing/feature-gating-criteria.md` § *"Documentation scope when a
capability arrives or leaves"*, and RFC-056 already widened it once from
arrivals to removals. A fifth standalone rule would itself land in
`contributing/` and reproduce the exact defect it describes. See §6 for what to
do instead.

**Q-2 — the stability statement goes in a new `docs/src/design/stability.md`**,
registered under **Snora Design** in `SUMMARY.md`, linked from
`design/feature-flags.md` and `design/tokens.md`. `design/overview.md` has the
audience but is already long, and a linkable page is what a prospective adopter
can be pointed at. The owner may relocate it; do not invent a third option.

## 4. The site that matters most is the one the RFC's `Touches` line omits

RFC-059's header lists `semantic-accessibility.md` and `focus.rs`. It does
**not** list `design/iced-style-bridge.md` — which is under **Snora Design**,
i.e. the consumer-facing half of the book, and which carries both halves of the
problem:

> `FocusTokens` (`tokens.focus.*`) are valid vocabulary and **will be wired when
> iced exposes focus state.**
>
> A missing focus ring on a standard button is a known `BLOCKED` limitation…

That is the version a consumer actually reads, and it is the one that tells
tekstide's situation — an application that already owns focus state — that it
must wait for a future iced. **Treat this file as in scope and as the priority
instance.** An RFC that fixed only the contributor copy would be the same defect
one more time.

`accessibility-checklist.md:26` is the same framing again. Four sites, one
claim.

## 5. What the re-scoped claim must say

The accurate constraint, narrowed:

> **iced cannot tell a style closure that a widget iced owns is focused.**

Not "iced 0.14 cannot render a focus ring". Everything that follows changes:

- **`FocusTokens` has a present-day audience** — any application that owns focus
  as its own state (a focus-zone enum cycled by Tab, say) can capture the
  boolean in a `container` style closure and set border colour *and* width from
  it. A `container` style closure is an arbitrary `Fn(&iced::Theme) -> Style`, so
  anything the application knows is available to it. Stop describing the tokens
  as being for "future iced versions and custom widgets" only.
- **Stop instructing reviewers not to file.** Replace *"Do not file it as a
  bug"* with the condition under which filing **is** right: a missing ring on a
  widget snora hands to iced is the documented limitation; anything else is a
  bug. A blanket "do not file" is what closed the question — and it is the
  second such control in two releases, after RFC-057's line-height item.
- Keep it honest in the other direction too: **snora's own primitives let iced
  own focus, and that is unchanged.** Documenting that applications can style
  their own focus is not a new snora capability and must not read as one.

## 6. Q-1's actual deliverable — widen the rule, and make it fire

Two edits, both in `contributing/feature-gating-criteria.md`:

1. **Widen the rule a second time.** It covers capabilities *arriving* and
   *leaving*. RFC-059's instances are neither: they are **standing answers** — a
   governance guarantee, and the true scope of a constraint — that exist only in
   a contributor document. Add that third case in the same paragraph, in the
   same voice. Add both new rows to the instance table (tekstide, 2026-08-17),
   which currently has three.
2. **Make following the process reach the rule.** Add one line to
   `contributing/release-process.md`'s checklist pointing at it. The rule has
   now been missed five times while sitting in a page titled *feature-gating
   criteria* — a poor home for a general documentation-scope rule. Do not move
   the page (out of scope); do give the release checklist a pointer, so the
   mechanism fires without anyone having to remember.

## 7. What the stability statement must and must not claim

`design/stability.md` states, for a prospective adopter:

- **What is frozen** by RFC-036's additive-only covenant: `Tokens` and its four
  constructors, `Palette` and its 18 role fields, `Color`, `Spacing`,
  `Typography`/`TextRole`, `Radius`, `FocusTokens`, `Tone`/`Emphasis`/`Size`/
  `Density`, and — named individually — `relative_luminance`, `contrast_ratio`,
  `composite_over`.
- **What is not**: the design **primitives** are deliberately excluded and run
  on a more permissive lifecycle. Say so plainly; a guarantee whose boundary is
  vague is worth less than a narrow one that is exact.
- **What changing it would cost**: removal, rename or retype is forbidden
  without reopening gates D-3 and D-4 *in the same change*, with an explicit
  prohibition on proceeding and rationalising afterward.

**Link `api-governance.md`; do not duplicate it.** Two copies of a covenant
diverge, and the contributor page stays the normative one.

**The one thing it must not claim.** The covenant constrains *what may change*,
not cargo's version arithmetic. `0.33` → `0.34` remains incompatible to cargo
under pre-1.0 SemVer regardless, and nothing here makes upgrades mechanically
painless. The question being answered is whether the token surface **churns**,
and the honest answer is that it is contractually forbidden to. Write that
distinction explicitly — a reader who takes this page as an upgrade-compatibility
promise has been misled by us, and this page exists because an adopter took our
silence for an answer once already.

## 8. Change scope

| File | Purpose |
|---|---|
| `docs/src/design/iced-style-bridge.md` | re-scope — **the consumer-facing instance** (§4) |
| `docs/src/contributing/semantic-accessibility.md` | re-scope :131 and the audience list :119–124 |
| `docs/src/contributing/accessibility-checklist.md` | re-scope :26; **leave :192 alone** |
| `crates/snora-design/src/focus.rs` | doc comment — same re-scope, plus present-day audience |
| `docs/src/design/stability.md` | **new** (§7) |
| `docs/src/SUMMARY.md` | register the new page under **Snora Design** |
| `docs/src/design/feature-flags.md`, `docs/src/design/tokens.md` | link to `stability.md` |
| `docs/src/guides/accessibility.md` | one line pointing at the re-scoped focus answer, as RFC-057 did for readability |
| `docs/src/contributing/feature-gating-criteria.md` | widen the rule; two table rows (§6) |
| `docs/src/contributing/release-process.md` | the pointer line (§6) |
| `CHANGELOG.md` | Documentation entries |

Consistency check while you are in `focus.rs`: its doc comment currently defers
to *"the `snora-widgets` style bridge for the documented limitations."* The
bridge moved to **`snora-style`** in 0.32.0. Correct the reference.

## 9. Explicit non-change scope

Do **not**:

- **Amend the covenant.** This documents it; it does not change it. No gate
  rows move, no gate reopens.
- **Add any focus capability**, or apply `FocusTokens` in snora's own widgets.
  RFC-045's position stands: snora integrates an accessibility tree when iced
  exposes one, and will not build a parallel abstraction.
- **Change `accessibility-checklist.md:192`.** The generic severity definition
  is correct.
- **Move `feature-gating-criteria.md`** or restructure `contributing/`.
- **Answer tekstide's Q1, Q3 or Q4** (upstream signal, `composite_over`,
  pointer target size). Those go in the reply, not the codebase.
- Touch any preset value — that is RFC-058.
- Modify `render_semantics.rs`.

## 10. Required tests

```bash
mdbook build docs
mdbook test docs
cargo test -p snora-design            # doctest in focus.rs must still run
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
git diff --stat -- 'crates/**/*.rs'   # doc-comment lines only
```

Every new fence in `stability.md` needs a documentation-test-policy-conformant
annotation — no bare `rust` fences in `docs/src`.

## 11. Acceptance criteria

RFC-059 §Acceptance criteria 1–7, with these bindings:

- **1** covers the **three** over-scoped sites in §3 —
  `design/iced-style-bridge.md` included — and leaves
  `accessibility-checklist.md:192` alone.
- **3** — the stability statement **must not claim version-upgrade
  compatibility** (§7). This is the criterion most likely to be failed by
  writing something helpful-sounding.
- **4** — report that no per-primitive `BLOCKED` record exists, confirmed by a
  fresh grep.
- **5** — the widened rule, both instance rows, and the release-checklist
  pointer (§6).
- **6** — `git diff --stat -- 'crates/**/*.rs'` shows doc-comment lines only.

## 12. Required evidence

- Before/after for each of the four re-scoped sites.
- `stability.md` in full, and the `SUMMARY.md` diff.
- The `feature-gating-criteria.md` diff, including both new table rows.
- The `focus.rs` diff, including the `snora-widgets` → `snora-style` correction.
- `git diff --stat -- 'crates/**/*.rs'`, showing doc comments only.
- `mdbook build` / `mdbook test` output.

## 13. Flagged for the owner, not assumed

`release-process.md` says *"Inspects all four .crate archives"* and *"Do NOT
package the four crates individually"*. There have been **five** crates since
0.32.0. It is a one-word correction in a file you are already editing for §6,
and it is in the checklist used to cut every release. **Include it**, listed
separately in the review request so the owner can strike it if they would rather
keep this change purely RFC-059's.

## 14. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/059-answers-filed-where-consumers-do-not-read/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the stability statement's claims (§7) — specifically
whether any sentence could be read as promising upgrade compatibility — and
whether the four re-scoped sites now say the *same* narrow thing. Four
paraphrases of one constraint is how it drifted wide the first time.
