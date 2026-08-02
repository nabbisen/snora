# Developer Handoff — RFC-038 token-derived `iced::Theme` emission

**Governing RFC.** [RFC-038](../../proposed/038-token-derived-theme-emission.md)
**Status.** Inherited from RFC-038 (Proposed; accepted by the owner).
**Release target.** v0.26 (minor — first additive public API of the milestone).
**Implementation units.** One.

---

## 1. Task title

Add `snora::design::theme(&Tokens) -> iced::Theme`, deriving a complete
iced theme from Snora Design tokens **without letting iced substitute its
own colors**.

## 2. Purpose

An application selecting `Tokens::high_contrast_dark()` today gets high
contrast on snora's design primitives and nowhere else — its `text_input`,
`pick_list`, `scrollable` and window background follow a separately
configured `iced::Theme`. This function removes that double configuration.

## 3. Background — read first

- `rfcs/proposed/038-token-derived-theme-emission.md` in full, especially
  §"Verified iced 0.14 facts" and §"The finding that shapes the design".
- `rfcs/done/037-coherent-defaults-positioning.md` — the positioning
  amendment that authorises this, and its **gating invariant**.
- `rfcs/done/036-design-surface-freeze-and-additive-covenant.md` — the
  covenant this must comply with.

**This is the first code change of the v0.26 milestone.** Everything in
0.25.3 was documentation and CI.

Conventions (the owner's rules document is not in the repository):

- **English only** for all prose and comments.
- Rust 2018+ module style, no `mod.rs`; tests in `foo/tests.rs`.
- `cargo fmt` **scoped to the crates you touch** — the workspace has ~152
  hunks of pre-existing drift; a workspace-wide `cargo fmt` would sweep
  them into your diff.

## 4. The load-bearing constraint

**Do not use `iced::theme::palette::Pair::new`.**

`Pair::new(color, text)` calls `readable(color, text)`, which lightens or
darkens the text colour in 0.1 steps until it clears a
`relative_contrast >= 6.0` bar — carrying an upstream
`TODO: Compute factor from relative contrast value`. Routing snora-design's
contrast-tested roles through it means iced silently replaces them with
heuristic approximations, and `snora-design`'s WCAG guarantees do not
transfer.

`Pair`'s fields are **public** (`pub color`, `pub text`), so construct
pairs with a struct literal instead:

```rust,ignore
Pair { color: to_iced_color(tokens.palette.surface), text: to_iced_color(tokens.palette.text) }
```

If you find yourself calling `Pair::new` anywhere in this work, stop —
that is the defect this RFC exists to avoid.

## 5. Verified iced 0.14 API shapes

Re-confirm these yourself against the pinned source before relying on them
(`iced_core-0.14.0/src/theme/palette.rs`):

```rust,ignore
Theme::custom_with_fn(name, palette: Palette, generate: impl FnOnce(Palette) -> Extended) -> Theme

struct Palette { background, text, primary, success, warning, danger }   // 6 Colors

struct Extended {
    background: Background, primary: Primary, secondary: Secondary,
    success: Success, warning: Warning, danger: Danger, is_dark: bool,
}

struct Background { base, weakest, weaker, weak, neutral, strong, stronger, strongest }  // 8 Pairs
struct Primary   { base, weak, strong }   // and Secondary / Success / Warning / Danger identically
struct Pair      { pub color: Color, pub text: Color }
```

Note `Background` has **eight** tiers while the semantic sets have three.
`Extended` is **not** `#[non_exhaustive]`, so constructing it exhaustively
means a future iced field addition is a **compile error, not a silent
default** — that is desirable; do not work around it.

## 6. Change scope

| File | Purpose |
|---|---|
| `crates/snora-widgets/src/design/theme.rs` | new — the emission function |
| `crates/snora-widgets/src/design/theme/tests.rs` | new — fidelity + contrast tests |
| `crates/snora-widgets/src/design.rs` | declare + re-export `theme` |
| `crates/snora/src/design.rs` | facade re-export as `snora::design::theme` |
| `docs/src/design/` | the 18→6 mapping, documented |
| `CHANGELOG.md` | `[Unreleased]` **Added** entry |

## 7. Explicit non-change scope

Do **not**:

- Touch `crates/snora-core/` or `crates/snora-design/` **at all**. This
  function is iced-typed and belongs in `snora-widgets` (NF-1).
- Change any existing signature. **RFC-036's covenant** forbids removing,
  renaming, retyping, or changing the meaning of anything in the frozen
  token or style-bridge surface. This work must be purely additive.
- Have snora call `theme()` on the application's behalf anywhere. Snora
  emits a value; the application owns it and passes it to iced.
- Restyle chrome, overlays, or toasts. That is RFC-039/040 territory and
  is explicitly **not** authorised by this RFC.
- Touch `WARNING_COLOR` in `crates/snora/src/toast.rs` — deferred, see §11.
- Run workspace-wide `cargo fmt`.

## 8. Required implementation

### Step 1 — The function

```rust,ignore
/// Derive a complete `iced::Theme` from a Snora Design token bundle.
pub fn theme(tokens: &Tokens) -> iced::Theme;
```

1. Build the six-slot base `iced::theme::Palette` from the token roles.
   This is what `Theme::palette()` reports; it is a lossy view.
2. Pass a **custom generator** to `Theme::custom_with_fn` that constructs
   `Extended` in full from the 18 roles, using `Pair { color, text }`
   struct literals throughout.
3. Set `is_dark` from the preset's own intent, **not** by inferring it
   from background luminance.
4. Name the theme from the preset so `Theme::to_string()` is meaningful.

Reuse `snora_widgets::design::style::color::to_iced_color` for conversion
rather than writing a second converter.

Where a token role has no obvious iced counterpart, or an iced tier has no
obvious token (`Background` has eight tiers, snora has fewer), **choose
deliberately and document the choice in the mapping table** — do not pick
silently. If a mapping is genuinely ambiguous, note it in the review
request rather than guessing.

### Step 2 — Tests (`design/theme/tests.rs`)

The contrast obligation is the blocking requirement, not a formality.

| Test | Assertion |
|---|---|
| **Fidelity** | For all four presets, every `Color` in the emitted `Extended` equals its source token role **exactly**. This is what proves iced's heuristic never ran. |
| **Contrast** | Every `Pair` in the emitted `Extended` meets WCAG AA (≥ 4.5) via `snora_design::contrast::contrast_ratio`. |
| **High-contrast strictness** | For the two HC presets, pairs meet AAA (≥ 7.0) wherever the underlying tokens already do. |
| **`is_dark`** | Correct for all four presets. |

If a contrast assertion fails, that is a **real finding about the token
mapping** — fix the mapping, or escalate. Do not relax the threshold.

### Step 3 — Facade and docs

Re-export via `crates/snora/src/design.rs` so it resolves as
`snora::design::theme`, matching how `style`, `button`, `card` etc. are
already exposed.

Document the 18→6 mapping under `docs/src/design/`, with an explicit note
that `Theme::palette()` is a **lossy view** and `extended_palette()` is
authoritative for rendering. Include a short usage example showing the
application passing the result to iced's `.theme()` hook.

## 9. Required tests

```bash
cargo fmt --check -p snora-widgets -p snora     # scoped
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora-widgets --features design
cargo test -p snora --lib --all-features
cargo check -p snora --no-default-features       # engine-only must still build
cargo check -p snora --no-default-features --features widgets
mdbook build docs && mdbook test docs
```

The two `--no-default-features` checks matter: they prove RFC-037's
**gating invariant** — with `design` inactive, nothing changed.

## 10. Acceptance criteria

RFC-038 §Acceptance criteria 1–7. Restated for checkability:

1. `snora::design::theme(&Tokens) -> iced::Theme` exists behind `design`
   and is documented.
2. `Extended` is built **without `Pair::new`**; fidelity tests prove exact
   equality with token roles.
3. Contrast tests pass over the emitted theme for all four presets.
4. The 18→6 mapping is documented, including the lossy-accessor note.
5. Engine-only and every feature-matrix combination still pass.
6. **RFC-036 covenant compliance is stated explicitly in the review
   request** — which frozen items you touched (expected: none).
7. No file under `crates/snora-core/` or `crates/snora-design/` changed.

## 11. Known risks and deferrals

- **`WARNING_COLOR`** (`crates/snora/src/toast.rs:46`) is now removable in
  principle — iced 0.14 does have a warning pair. **Not in this work.**
  Toasts render on the design-*inactive* path, so changing it would alter
  appearance for applications that never opted in, violating RFC-037's
  gating invariant. Owner decision, deferred (RFC-038 Q-2).
- **Mapping ambiguity** is the likeliest source of trouble. Surface it,
  don't resolve it silently.

## 12. Compatibility and security

**Compatibility.** Purely additive and feature-gated. With `design`
inactive the function does not exist and rendering is unchanged. Adopters
will see a visual change — that is a **Changed** entry in the v0.26
migration guide, not **Fixed**. State compliance with RFC-037's gating
invariant explicitly.

**Security.** No new data flow, dependency, or integration. The function
is pure. Confirm this in the review request.

## 13. Required evidence

- The `theme.rs` diff.
- Test output showing fidelity and contrast passing for all four presets.
- Confirmation that `Pair::new` appears nowhere in the new code
  (`grep -rn "Pair::new" crates/` should return nothing you added).
- Both `--no-default-features` check results.
- `git diff --stat -- crates/snora-core crates/snora-design` — must be empty.
- A statement of RFC-036 covenant compliance.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/038-token-derived-theme-emission/`. Report
paths relative to the project root.

**Requested review focus:** the 18→6 mapping choices, and any place you
had to decide what an iced tier means in snora's vocabulary.
