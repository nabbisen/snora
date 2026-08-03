# Developer Handoff — RFC-040 chrome geometry

**Governing RFC.** [RFC-040](../../proposed/040-chrome-geometry.md)
**Status.** Inherited from RFC-040 (Proposed; accepted by the owner).
**Release target.** v0.27.0, alongside RFC-039.
**Implementation units.** One. Independent of RFC-039 — parallel is fine.

---

## 1. Task title

Add token-derived styled variants of the prefab chrome widgets under
`snora::design::widget::*`, sharing one implementation with the existing
unstyled set.

## 2. Purpose

RFC-038 made chrome *colours* follow the emitted theme. Geometry cannot
follow — an `iced::Theme` carries no spacing or radius — so the prefab
widgets still hardcode unrelated magic numbers, and `radius: 0.0` is a
large part of why stock snora chrome reads as flat and dated.

## 3. Background — read first

- `rfcs/proposed/040-chrome-geometry.md` in full.
- `rfcs/done/037-coherent-defaults-positioning.md` — the gating invariant.
- `rfcs/done/036-design-surface-freeze-and-additive-covenant.md` — this
  work *consumes* the frozen surface and must not modify it.

Conventions: English only; Rust 2018+ modules, tests in `foo/tests.rs`;
`cargo fmt` **scoped to `snora-widgets` and `snora`** (~152 hunks of
pre-existing workspace drift).

## 4. The current literals — preserve these exactly

The unstyled path must render **identically** after this change. Here is
the full inventory, captured before work began:

| File | Line | Value |
|---|---|---|
| `header.rs` | 67, 81 | `.spacing(12)` |
| `header.rs` | 90 | `.padding([8.0, 16.0])` |
| `footer.rs` | 20 | `.padding([6.0, 16.0])` |
| `sidebar.rs` | 38 | `.spacing(16)` |
| `sidebar.rs` | 57 | `.padding(16.0)` |
| `sidebar.rs` | 82, 83 | `radius: 6.0`, `width: 0.0` |
| `tab.rs` | 56 | `.spacing(2)` |
| `tab.rs` | 81 | `.padding([0.0, 12.0])` |
| `tab.rs` | 97 | `.spacing(6)` |
| `tab.rs` | 106 | `.padding([8.0, 12.0])` |
| `tab.rs` | 123, 124 | `width: 1.0`, `radius: 0.0` |
| `crumb.rs` | 58 | `.spacing(6)` |
| `crumb.rs` | 76 | `.padding([4.0, 12.0])` |
| `crumb.rs` | 96 | `.padding([2.0, 4.0])` |
| `crumb.rs` | 119, 120 | `width: 0.0`, `radius: 3.0` |
| `menu.rs` | 39, 60 | `.spacing(6)` |
| `style.rs` | 41, 42 | `radius: 0.0`, `width: 1.0` |

**Verify these against the source yourself** — line numbers drift.

## 5. Change scope

| File | Purpose |
|---|---|
| `crates/snora-widgets/src/{header,footer,sidebar,tab,crumb,menu}.rs` | extract shared builders |
| `crates/snora-widgets/src/design/widget.rs` + `widget/` (new) | styled variants |
| `crates/snora-widgets/src/design/widget/tests.rs` (new) | geometry tests |
| `crates/snora-widgets/src/design.rs` | declare + re-export |
| `crates/snora/src/design.rs` | facade re-export |
| `examples/design_workbench/src/main.rs` | styled-chrome view |
| `docs/src/design/` | the geometry mapping |
| `CHANGELOG.md` | `[Unreleased]` **Added** |

## 6. Explicit non-change scope

Do **not**:

- Change the signature of any existing `snora::widget::*` function.
- Change what the unstyled path renders. §4 is the contract.
- Add or modify any token role or scale. **There is no elevation or shadow
  token and this RFC does not add one** — see RFC-040 §"Scope correction".
- Touch typography. `Typography` exists and chrome could use it, but font
  changes reflow layout; deferred with its own trigger.
- Add new widgets. The permanent non-goals (no form, data-display or
  decorative widgets) stand — this restyles what exists.
- Touch engine surfaces. Dialog card and modal dim are RFC-039.

## 7. Required implementation

### Step 1 — Extract one implementation per widget

This is the load-bearing requirement. **Do not write a second copy of any
widget body.**

For each widget, extract construction into a private builder parameterised
by geometry, e.g.:

```rust,ignore
struct ChromeGeometry { pad_x: f32, pad_y: f32, gap: f32, radius: f32 }
```

- the **unstyled** entry point passes today's literals from §4, unchanged;
- the **styled** entry point passes token-derived values.

Today's literals then live in exactly one place, explicitly labelled as the
pre-design defaults. Drift between the two paths becomes structurally
impossible rather than merely discouraged.

The struct shape above is a sketch — widgets differ (some have no radius,
`crumb.rs` has two padding sites). Use whatever shape fits; the requirement
is *one body, two geometry sources*, not this exact struct.

### Step 2 — Styled variants

```rust,ignore
// snora::design::widget::*
pub fn app_header(tokens: &Tokens, /* existing params */) -> Element<'_, Message>;
pub fn app_side_bar(tokens: &Tokens, /* … */) -> …;
pub fn app_footer(tokens: &Tokens, /* … */) -> …;
pub fn app_tab_bar(tokens: &Tokens, /* … */) -> …;
pub fn app_breadcrumb(tokens: &Tokens, /* … */) -> …;
```

`&Tokens` first, matching every existing style-bridge signature.

### Step 3 — The mapping, justified

Map each geometry value to a `Spacing` (xs/sm/md/lg/xl/xxl) or `Radius`
(sm/md/lg/pill) token, and **document why**.

Do **not** reverse-engineer the mapping to reproduce §4's numbers. The
point is a shared rhythm; §4 is an inconsistent set (padding of 8/16, 6/16,
16, 0/12, 8/12, 4/12, 2/4 — seven different shapes). Where a current
literal has no clean token equivalent, that is evidence the literal was
arbitrary — **say so in the mapping table** rather than inventing a token
value to match it.

**`Density` participates** (owner-accepted): chrome padding is exactly what
a density setting should affect. If it turns out awkward in practice, flag
it rather than silently dropping it.

### Step 4 — Workbench

The design workbench gains a styled-chrome view. Without it the result
cannot be judged — nothing in the test suite can tell you whether the
chrome now looks coherent.

## 8. Required tests

```bash
cargo fmt --check -p snora-widgets -p snora
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora-widgets --features design
cargo test -p snora --lib --all-features
cargo check -p snora --no-default-features
cargo check -p snora --no-default-features --features widgets
mdbook build docs && mdbook test docs
```

New tests:

| Test | Assertion |
|---|---|
| **Unstyled geometry unchanged** | Each unstyled builder receives exactly §4's literals. This is the regression test that protects the gating invariant — write it against the geometry values, not rendered output |
| Styled geometry is token-derived | Each styled variant's geometry equals its mapped token value, all four presets |
| Density is respected | If `Density` participates, compact and comfortable yield different values |

Note what is **not** claimed: none of this tests that the result looks
good. That is the workbench's job, and it is a human judgement.

## 9. Acceptance criteria

RFC-040 §Acceptance criteria 1–7:

1. Styled variants exist for header, sidebar, footer, tab bar, breadcrumb.
2. **One implementation per widget**, parameterised by geometry.
3. Unstyled variants unchanged — signatures and geometry both.
4. The mapping is documented, including any literal found to be arbitrary.
5. Gating-invariant checks pass.
6. The workbench can display styled chrome.
7. RFC-036 covenant compliance stated in the review request.

## 10. Prohibited shortcuts

- Do not duplicate widget bodies. Two implementations will drift; that is
  the single most likely way this change causes harm six months from now.
- Do not change unstyled geometry "while you're in there" — even to make it
  consistent. That is a v0.28 conversation with its own evidence.
- Do not pick token mappings by matching today's numbers.
- Do not add an elevation token.

## 11. Compatibility and security

**Compatibility.** Purely additive. Applications not calling the styled
variants cannot be affected — they call different functions. Adopters see a
visual change: **Changed** in the v0.27 migration guide.

**Security.** No new data flow, dependency or integration.

## 12. Required evidence

- The shared-builder extraction for at least one widget, in full, so the
  one-body-two-sources shape can be reviewed.
- The mapping table.
- Unstyled-geometry regression test output.
- Both `--no-default-features` check results.
- `git diff --stat -- crates/snora-core crates/snora-design` — empty.
- A screenshot or description of the workbench's styled-chrome view.

## 13. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/040-chrome-geometry/`. **State the single
entry-point path to hand to the reviewer** in the completion summary.

**Requested review focus:** the geometry mapping — specifically whether any
value was chosen to match today's literal rather than to fit the scale.
