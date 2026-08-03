# Developer Handoff — RFC-037 coherent defaults positioning

**Governing RFC.** [RFC-037](../../proposed/037-coherent-defaults-positioning.md)
**Status.** Inherited from RFC-037 (Proposed; accepted by the owner).
**Release target.** v0.26.0, alongside RFC-038 and RFC-043.
**Implementation units.** One. Documentation and governance only — no code.

---

## 1. Task title

Extend RFC-020's design-system boundary to the surfaces snora itself
renders, and amend DEC-02 to permit snora to *emit* an `iced::Theme`
without owning theme state.

## 2. Purpose — and why it must ship with RFC-038

RFC-038 adds `snora::design::theme(&Tokens) -> iced::Theme`. The decision
register currently records:

> Theme-aware, not theme-owning | **Firm boundary** | iced adds an
> insufficient theming layer

Shipping the emitter while that stands unchanged would leave the code and
the governing decision in direct contradiction — the exact defect RFC-035
was raised to eliminate. **This handoff must land in the same release as
RFC-038.** It carries no code, so it can be implemented and reviewed in
parallel.

## 3. Background — read first

- `rfcs/proposed/037-coherent-defaults-positioning.md` in full.
- `rfcs/done/020-design-system-boundary-and-philosophy.md` — the boundary
  this extends. Note RFC-020's acceptance criterion "Boundary statement is
  added to docs" was **never satisfied**; this work discharges it.
- `docs/src/contributing/design-decisions.md` — the register being amended.

Conventions: English only. No `cargo fmt` needed (no code), and do **not**
run it workspace-wide regardless — ~152 hunks of pre-existing drift.

## 4. Change scope

| File | Purpose |
|---|---|
| `docs/src/design/overview.md` | the amended boundary statement (discharges RFC-020's open criterion) |
| `docs/src/contributing/design-decisions.md` | DEC-02 index row + section; theme-owning vs theme-producing |
| `docs/src/contributing/feedback-and-scope.md` | scope-table row: owning vs emitting |
| `README.md` | design-notes bullet |
| `CHANGELOG.md` | `[Unreleased]` **Changed** entry |

## 5. Explicit non-change scope

Do **not**:

- Change any source file. This handoff carries no code.
- Weaken any declined-scope item: N-3 (form widgets), N-4 (data display),
  N-5 (decorative widgets), `snora-test`, i18n, game-loop rendering. They
  stand exactly as they are.
- Change `DEC-11` or the `design` feature default. Whether `design` becomes
  default-on is a separate owner decision, informed by RFC-043's corrected
  measurements — explicitly out of scope.
- Touch any 1.0 gate or D-gate row.
- Change RFC-020's own text. It is in `done/` and is historical.

## 6. Required implementation

### Step 1 — The amended boundary statement

Add to `docs/src/design/overview.md`, replacing the existing paraphrase,
the statement from RFC-037 §"Amended boundary statement" verbatim.

**Add one scoping sentence RFC-037 does not yet contain**, because the
claim outruns what v0.26.0 delivers:

> Surface coverage arrives incrementally. As of v0.26.0, chrome colours
> follow the emitted theme because snora's prefab widgets already read
> iced's palette; overlay styling and layout geometry (dialog card, modal
> dim, spacing rhythm, elevation) are not yet token-derived.

Without that, the page claims a coherent visual default across snora's
rendered surfaces while the dialog is still an unstyled `center()`. Say
what is true now.

### Step 2 — DEC-02

In `docs/src/contributing/design-decisions.md`:

- **Index row**: status changes from *Firm boundary* to **Accepted**, with
  a reconsideration trigger. The theme-*owning* half stays firm — make
  that visible in the row, not only in the section body.
- **Section** ("Why Snora is theme-aware but not theme-owning (v0.14)"):
  add the distinction from RFC-037 §"Amendment: theme-producing, not
  theme-owning":
  - **Theme-owning** — snora defines a parallel theming abstraction, holds
    theme state, requires apps to configure appearance through snora
    rather than iced. **Still declined, permanently.**
  - **Theme-producing** — a pure function from tokens to an `iced::Theme`.
    The app decides whether to call it, owns the result, hands it to iced
    itself. Snora holds no state and intercepts nothing. **Permitted under
    `design`.**

  Include the reasoning that emission *removes* a double configuration
  which exists today (apps maintaining tokens and an `iced::Theme`
  separately), rather than creating one.

  Retitle the section so it no longer reads as a flat prohibition.

### Step 3 — Scope table

In `docs/src/contributing/feedback-and-scope.md`, split the row:

```
| Snora-owned theming layer | Firm non-goal — use iced theming |
```

into owning (firm non-goal) and emitting (available under `design`, see
`snora::design::theme`).

### Step 4 — README

The design-notes bullet should reflect that `design` supplies defaults for
snora's own surfaces, not only for app-built primitives — with the same
incremental-coverage honesty as Step 1. Keep it to the bullet's existing
length; the README is deliberately concise.

### Step 5 — CHANGELOG

`[Unreleased]` under **Changed**: the boundary extension and the DEC-02
amendment, noting the theme-owning half remains declined.

## 7. Required tests

No code, so no test suite. Run:

```bash
mdbook build docs
mdbook test docs
```

Both must exit 0.

## 8. Acceptance criteria

RFC-037 §Acceptance criteria 1–7, plus the scoping sentence from Step 1:

1. `design/overview.md` carries the amended boundary statement, satisfying
   RFC-020's long-open criterion.
2. It also states that surface coverage is incremental, and what is *not*
   yet token-derived as of v0.26.0.
3. `design-decisions.md` distinguishes theme-owning (declined) from
   theme-producing (permitted); the index row's status and trigger are
   updated.
4. `feedback-and-scope.md` splits owning from emitting.
5. `README.md` reflects the extended scope.
6. No declined-scope item is weakened; no gate row changed; no source file
   changed.
7. `mdbook build docs` and `mdbook test docs` pass.

## 9. Prohibited shortcuts

- Do not soften the theme-*owning* prohibition while amending the
  theme-*producing* half. The whole point of the split is that one stays
  firm.
- Do not claim v0.26.0 delivers more surface coverage than it does. The
  dialog card, modal dim, and spacing rhythm are RFC-039/040.
- Do not resolve the DEC-11 / default-on question in passing.

## 10. Compatibility and security

Neither is affected — documentation only, no API, no dependency, no data
flow. State this explicitly.

## 11. Required evidence

- Diffs of all five files.
- `mdbook build docs` / `mdbook test docs` output.
- Explicit confirmation that no source file and no gate row changed.
- A quoted before/after of the DEC-02 index row, so the status change is
  reviewable at a glance.

## 12. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/037-coherent-defaults-positioning/`. Report
paths relative to the project root, and **state the single entry-point
path to hand to the reviewer** in the completion summary.

**Requested review focus:** whether the boundary statement claims more
than v0.26.0 actually delivers.
