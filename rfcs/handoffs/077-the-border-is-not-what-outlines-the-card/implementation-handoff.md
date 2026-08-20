# Developer Handoff — RFC-077 what actually outlines the dialog card

**Governing RFC.** [RFC-077](../../accepted/077-the-border-is-not-what-outlines-the-card.md)
**Status.** Inherited from RFC-077 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.0 — documentation and rationale only. **No values, no
assertions, no rendered output.**
**Implementation units.** One.

---

## 1. Purpose

**arama measured the dialog card over real photographic content and found the
border invisible against the dim** — 1.02:1 in `light`, 1.23:1 in `dark`. What
separates the card from the dimmed page is the dim-to-fill step, at 3.46 and
3.89.

Their figures hold at 0.38.3: `git diff 0.38.0 0.38.3` on `presets/` and
`surfaces.rs` is empty.

## 2. Q-1 answered — and it refuted the guess in the RFC

RFC-077's Q-1 speculated that in the high-contrast presets the border would be
*highly* visible against the dim, "the opposite of `light`." **That is wrong.**

Swept over greyscale content in 1000 steps, dim composited at `DIM_ALPHA = 0.44`:

| preset | `border ǀ dim` min | at content | max | `dim ǀ fill` range |
|---|---|---|---|---|
| `light` | **1.00** | 0.98 | 6.21 | 3.24 – 21.00 |
| `dark` | **1.00** | 0.00 | 4.93 | 3.16 – 15.61 |
| `high_contrast_light` | **1.00** | 0.00 | 6.48 | 3.24 – 21.00 |
| `high_contrast_dark` | **1.00** | 1.00 | 4.94 | 4.01 – 19.80 |

**Every preset has a content luminance at which the border vanishes against the
dim.** The presets differ only in *which* content does it — near-white in
`light`, black in `dark` and `high_contrast_light`, white in
`high_contrast_dark`. There is no preset in which the border reliably outlines
the card.

**And `dim ǀ fill` never drops below 3.16 in any preset.** That is the mechanism
that always works, across the whole content range, in all four — a stronger
result than arama's two-preset photo sample, and it confirms their conclusion
rather than qualifying it.

**Re-derive this table.** It is a cross-check, not a source. If yours disagrees,
yours wins and this handoff is wrong — say so.

## 3. What to write

In `docs/src/design/engine-surfaces.md`, where the border and the dim are
explained:

1. **The border's contrast job is against the card's own fill**, at the inner
   edge — 3.38:1 `light` / 3.17:1 `dark`. That is real and required.
2. **The card's separation from the dimmed page is carried by the dim-to-fill
   step**, never by the border. State that the border can reach 1.00:1 against
   the dim in **every** preset at some content.
3. **RFC-066's `max(border ǀ dim, fill ǀ dim)` assertion is load-bearing on one
   branch.** It is correct and stays exactly as it is — but the prose must not
   imply two working mechanisms where there is one.

Then correct the RFC-058 rationale where it is restated — `CHANGELOG.md`'s
0.34.0 entry says the card "was deliberately chosen as border-defined rather
than shadow-defined", and `contributing/accessibility-checklist.md` carries the
same framing.

**`CHANGELOG.md` is a historical record — do not rewrite the 0.34.0 entry.**
Correct the *live* restatements only, and if the checklist's wording is the
thing that is wrong, fix it there.

## 4. Q-2 ruled — measure the sheet, and say either way

The sheet panel has a border and sits over the same dim. Nobody has measured it.
**Measure it and state the result**, even if it is identical to the dialog's.
"Same as the dialog" is a finding; silence is not.

## 5. Explicit non-change scope

- **No palette value, no `DIM_ALPHA`, no assertion changed.** Everything
  measured clears its threshold. Nothing is failing.
- **Do not weaken the border requirement.** It does necessary work at the inner
  edge. That it does not *also* outline the card is not an argument for relaxing
  it, and a reader must not be able to draw that conclusion from what you write.
- **Do not rewrite the 0.34.0 CHANGELOG entry** (§3).
- **No new test.** RFC-066 already asserts the correct quantity.

## 6. Required evidence

- Your own derivation of the §2 table, method stated, all four presets
- The sheet-panel measurement (§4)
- `cargo test --workspace --all-features` — unchanged, nothing touched
- `git diff --stat -- crates/` — **expected empty**
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py` clean

## 7. Acceptance criteria

1. All four presets derived independently; the table in §2 confirmed or
   corrected.
2. `engine-surfaces.md` states which mechanism separates the card from the dim
   and which does not, without implying the border requirement is optional.
3. Live restatements of the RFC-058 rationale corrected; the 0.34.0 CHANGELOG
   entry untouched.
4. Sheet panel measured and reported.
5. No value, no assertion, no rendered output changed.
6. arama credited in `CHANGELOG.md` `[Unreleased]` under **Fixed**.

## 8. Required review-request format

`.git-exclude/review-request/077-the-border-is-not-what-outlines-the-card/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: the wording of §3.2 and §3.3 together.** The failure
mode is a page that now says the border does not matter. It does; it matters
somewhere other than where we said.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
