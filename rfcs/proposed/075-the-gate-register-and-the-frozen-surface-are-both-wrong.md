# RFC 075 — The gate register contradicts itself, and the frozen-surface list omits seven of the functions it freezes

**Status.** Proposed
**Tracks.** Governance records / documentation integrity.
**Found by** a commissioned dev-team audit of `docs/` and crate doc comments,
2026-08-20. Every load-bearing claim below re-verified against source by the
architect before acceptance.
**Source report.** `.git-exclude/review-request/docs-and-comments-staleness-audit/`
**Release target.** 0.38.3 — documentation and doc comments only. **No code.**

## Summary

An audit found 16 confirmed defects. **Category A (stale version pins) is
already [RFC-074](./074-version-snippets-are-a-hand-maintained-list.md)** — the
audit reached the same root cause independently, which is corroboration, not
duplication. This RFC takes the rest.

Two of them are not ordinary staleness.

## Defect 1 — the gate register disagrees with itself about gate 9b

`contributing/api-freeze-review.md` is **the** record of 1.0 gate status.
RFC-073 ruled, six days ago, that other pages must *link to it and never restate
its verdict*. Its own verdict is inconsistent four ways:

| line | says | correct? |
|---|---|---|
| `:8` | "**Gate 9 closed at v0.37.0** — 9a at v0.29.0, 9b on four…" | ✅ |
| `:107` | table row 9b: "✅ **v0.37.0 — closed**" | ✅ |
| `:110` | "Gates satisfied: 2, 4, 5, 6, 7, 8, 9a, 10 = **seven of ten**, plus the binary-size half of gate 9" | ❌ omits 9b — the count is **eight of ten** |
| `:159` | "**Remaining blockers:** iced upgrade (gate 1), third-party app (gate 3), **compile-time measurement noise (gate 9b)**" | ❌ 9b is not a blocker |
| `:168` | "satisfying 9b now on a 25%-noise series would be a quieter instance of the same mistake" | ❌ present tense, and 25% is the figure RFC-073 just deleted elsewhere as stale |

**A register that other pages are told to defer to must not be wrong about a
gate's status.** This is more serious than the `build-cost-budget.md` instance
RFC-073 fixed, because that page was deferring *to this one*.

## Defect 2 — the frozen-surface enumeration omits seven of the twenty-two functions it freezes

`api-governance.md:167` defines RFC-036's frozen style-bridge surface as **"all
public functions of `snora_style`"**, then enumerates them. `snora_style` has
**22** public functions. The list names **15**. Missing:

- the six `*_line_height` helpers (RFC-068, 0.38.0 — this session)
- **`theme::theme`** — missing since RFC-055 (0.32.0), six minors

**The rule is correct and complete; the enumeration is a redundant restatement
that drifted.** That is the interesting part: the covenant does not need the
list to be sound, and the list is the only thing that can be wrong.

## The rest

Verified, detailed in the source report, summarised:

- **B2** — `feature-gating-criteria.md` says "Ten" before its table and "Eleven"
  after it. The table has 11 rows.
- **B3** — `engine-surfaces.md:110` shows `a: 0.4` (the constant is `0.44`) and
  attributes the darkness check to *"iced's own public
  `iced::theme::palette::is_dark`"*. The real `is_dark` is **private** and
  computes locally — **`snora-design` has no iced dependency at all**, enforced
  by a CI gate (RFC-021/022 Q3). The page describes an architecture that would
  fail that gate.
- **C1–C5** — the RFC-055/056 style-bridge relocation was never fully swept.
  Sharpest: `snora-widgets/src/lib.rs:44` calls `pub mod design` "the iced style
  bridge", while `design.rs`'s own module doc one file down says it is prefab
  widgets and that "the iced style bridge… is `snora_style`, not this module."
  **Same module, two doc comments, opposite claims.**
- **D1** — `snora-style/src/text.rs` still says widget adoption is "gated on an
  adopter's deferred typography assessment landing its own evidence." **That
  evidence arrived and RFC-068 Q-2 was ruled on 2026-08-19.**
- **D2** — `overlay-interaction-semantics.md` describes RFC-014-A as future work
  three times while recommending its shipped deliverable as current API.
- **E1** — `snora-widgets/src/design/card.rs:8,93` carries a version-scoped
  qualifier on a still-true fact; the same qualifier was fixed once, in a
  sibling location, at 0.22.0 and left here.

## Three of these are mine, from this session

Stated because the pattern matters more than the instances:

- **B2** — RFC-072 changed "Ten" to "Eleven" *after* the table and not *before*.
  I accepted it, having written *"find every disagreeing statement, not just the
  one you were told about"* into a review of another RFC the same day.
- **B3** — RFC-071 corrected two tables on `engine-surfaces.md` and I did not
  look 27 lines up at the snippet.
- **D1** — I ruled RFC-068 Q-2 and recorded it in `rfcs/done/`, in the CHANGELOG
  and in a letter to orbok, but **not in the module doc that RFC-068's own
  acceptance criterion 5 required it be written into.** The place chosen to hold
  the reasoning is the place that went stale first.

## Open questions

**Q-1 — delete the frozen-surface enumeration, or derive it?** The covenant text
("all public functions of `snora_style`") is self-sufficient, so the list adds
no governance and can only be wrong. **Suggest deleting it** and keeping one
sentence pointing at the crate's rustdoc. If it is kept for readability, it must
be generated, not typed — a hand-listed definition of a frozen surface is a
governance defect waiting to recur, and this is its second occurrence.

**Q-2 — should the gate register carry a single-source count?** `:110`'s count
and `:159`'s blocker list are both derived from the gate table above them and
both drifted from it. **Suggest stating the count once, immediately under the
table**, and deleting the second restatement rather than repairing it.

**Q-3 — is a check warranted for "one page, two claims"?** Three instances this
session (B1, B2, B3), plus RFC-069's and RFC-073's. **Suggest not yet, and
record the count.** RFC-071 Q-4 already holds this question; adding a fifth data
point to it is more useful than a check nobody has specified.

## Acceptance criteria

1. Every statement in `api-freeze-review.md` about gate 9b agrees with its own
   table. The count and the blocker list are correct or gone.
2. The frozen-surface list is deleted or generated — **not extended by hand.**
3. B2, B3, C1–C5, D1, D2, E1 corrected, each verified against source rather than
   against this RFC.
4. `snora-widgets/src/lib.rs` and `design.rs` describe the same module the same
   way.
5. `text.rs`'s module doc states RFC-068 Q-2's **ruling**, not a pending gate.
6. No code change; `git diff -- crates/` shows doc comments only.

## Compatibility and security

**Compatibility.** Documentation and doc comments. `crates/*/src/*.rs` changes
are comment-only. **Security.** None.
