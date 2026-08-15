# Developer Handoff — RFC-055 extract `snora-style`

**Governing RFC.** [RFC-055](../../done/055-extract-the-style-bridge.md)
**Status.** Inherited from RFC-055 — Implemented (v0.32.0).
**Release target.** 0.32.0 (minor — new published crate, no public path
changes).
**Implementation units.** One. Independent of RFC-050.

---

## 1. Task title

Move `snora-widgets/src/design/style/` into a new peer crate `snora-style`,
make `design` independent of `widgets`, and keep every existing import path
working.

## 2. Purpose

`card_raised` has three callers: the card **widget**
(`snora-widgets/src/design/card.rs:81`), the **engine chrome**
(`snora/src/design/render.rs:173`), and **applications** directly, since
`snora::design::style::*` re-exports all five style modules.

One style vocabulary, three consumers, physically inside one of the three. The
engine therefore reaches sideways into a crate it otherwise would not need,
and `design` cannot be enabled without `widgets`.

This is a relocation. **No behaviour changes anywhere.**

## 3. Background — read first

- `rfcs/done/055-extract-the-style-bridge.md` in full.
- `rfcs/proposed/054-design-requires-widgets.md` — the investigation, including
  the measured figures and the feature-propagation bug (§5 below).
- `docs/src/reference/architecture.md` — opens *"Snora is four crates"*. You
  are changing that, and RFC-035 corrected this document once already for
  being wrong about the crate graph.

Conventions: English only. `cargo fmt --all --check` is CI-enforced.

## 4. Change scope

| File / area | Purpose |
|---|---|
| `crates/snora-style/` (new) | the five style modules |
| `crates/snora-widgets/Cargo.toml`, `src/design/style.rs` | optional dep; re-export |
| `crates/snora/Cargo.toml` | feature graph (§5) |
| `crates/snora/src/design.rs` | `#[cfg]` gating of widget-layer re-exports |
| `crates/snora/src/design/render.rs` | `card_raised` / `to_iced_color` now from `snora-style` |
| `Cargo.toml` (root) | `workspace.dependencies` entry |
| `examples/` | one new probe (§6) |
| `docs/src/reference/architecture.md` | four crates → five |
| `docs/src/design/feature-flags.md` | **loses** the "Requires `widgets`" caveat |
| `CHANGELOG.md` | `[Unreleased]` **Changed** |

No `workspace.members` edit — the root manifest globs `crates/*`.

## 5. The trap, found the hard way in RFC-054

```toml
design = ["dep:snora-design", "dep:snora-style", "snora-widgets?/design"]
```

`"widgets"` leaves the list. **`"snora-widgets?/design"` must stay, with the
`?`.** RFC-054's spike dropped it, and enabling `widgets` + `design` together
silently stopped activating `snora-widgets`'s own `design` submodule — failing
with `E0433`. The `?` means it applies only when `snora-widgets` is already
present, so it does not re-introduce the coupling being removed.

**Test both configurations, not just the new one.** The bug appears only in
the configuration you are *not* trying to create.

## 6. Required implementation

### Step 1 — The crate

`snora-style`. Dependencies: **`iced` and `snora-design` only.** Verified in
RFC-054: the style modules import nothing else — in particular not
`snora-core`. If you find you need a third dependency, stop and report; it
means the layering answer was wrong.

Move all five modules — `color`, `button`, `container`, `text`, `progress` —
not only the two the engine uses. Splitting the layer across two crates is
worse than either endpoint.

Workspace-inherited metadata (`version`, `edition`, `license`, …) as the other
four crates do.

### Step 2 — `snora-widgets` re-exports

`snora-widgets` takes `snora-style` as an **optional** dependency, activated by
its existing `design` feature:

```toml
design = ["dep:snora-design", "dep:snora-style"]
```

and re-exports so `snora_widgets::design::style::*` resolves exactly as today.
Its public API must not change (acceptance criterion 2).

**This is where criterion 5 is won or lost** — see §7.

### Step 3 — `snora::design` gating

Its style and token re-exports work without `widgets`. Its widget-layer
re-exports — `design::widget`, `button`, `card`, `notice`, `chip`,
`progress` — do not, and need `#[cfg(feature = "widgets")]`.

Keep the gating minimal and in one place. A reader should be able to see at a
glance which half of `snora::design` needs widgets.

### Step 4 — The new probe

Add a probe for the configuration this RFC creates —
`snora --no-default-features --features design` — matching the existing three
probes' baseline application exactly, so its number is comparable.

Note this probe needs the same explicit hand-pinned `snora` dependency as
`size_probe_engine` and `responsive_body`, because it disables default
features. **That makes it the third such example**, so update the root
`Cargo.toml` comment and the `release-process.md` checklist line, both of which
currently name two.

### Step 5 — Documentation

`architecture.md`: five crates, corrected diagram, dependency direction still
stated as strict.

`feature-flags.md`: remove the "Requires `widgets`" caveat from the `design`
row, and state the four now-expressible configurations.

## 7. What must not regress

**The default configuration compiles no style code today**, because
`snora-widgets` gates its whole `design` module behind its own opt-in feature.
Keeping `snora-style` optional (§6 step 2) preserves that.

Prove it by measuring, not by reasoning:

- `size_probe_widgets` — **must not grow**;
- `size_probe_design` — must not regress.

If either moves, the dependency is unconditional somewhere. Find it rather
than accepting the delta.

## 8. Explicit non-change scope

Do **not**:

- **Change any public path.** Every `snora::design::*` and
  `snora_widgets::design::*` path in use today must keep working. No
  deprecation markers in this release either — see §9.
- **Move widget-layer modules.** `widget`, `button`, `card`, `notice`, `chip`,
  `progress` stay in `snora-widgets`.
- **Touch `snora-design`.** It is iced-free by hard constraint; that is why
  `snora-style` exists.
- **Duplicate the card mapping.** `design::render` keeps calling `card_raised`,
  now from `snora-style`.
- **Change rendering.** `render_semantics` must pass unmodified.
- Add a feature to `snora-style` unless §7 forces it — and if it does, say why.

## 9. On deprecating the re-export — not now, and here is why

`snora_widgets::design::style::*` becomes a **compatibility re-export** the
moment `snora-style` exists. Shims accumulate, and the project's policy is to
mark superseded things deprecated.

**Do not mark it deprecated in this release.** Two reasons:

- `snora::design::style::*` — the path applications actually use — is
  re-exported *through* `snora-widgets` today. Deprecating the widgets path
  without first re-pointing `snora`'s own re-export at `snora-style` would emit
  warnings for consumers who did nothing wrong.
- Nothing is superseded yet from a consumer's perspective: `snora-style` is
  not a documented path for applications until the docs say so.

Instead, **record it**: note in `CHANGELOG.md` and in the RFC's follow-up that
the widgets re-export is now a compatibility shim, and that deprecating it is a
separate decision once `snora::design::style` points at `snora-style` directly.
That keeps the obligation visible rather than forgotten.

## 10. Required tests

```bash
cargo check -p snora --no-default-features                      # engine only
cargo check -p snora --no-default-features --features design    # NEW — the point
cargo check -p snora --features widgets                          # today's default
cargo check --workspace --all-features
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics    # MUST pass unmodified
cargo test -p snora-widgets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
scripts/measure-binary-size.sh 0.0.0-test      # §7 — compare, do not append
```

The size script's output is for comparison only. **Do not append a row to
`binary-size.csv`** — that happens on release tags.

## 11. Acceptance criteria

RFC-055 §Acceptance criteria 1–9. The two most likely to be skipped:

- **5** — `size_probe_widgets` must not grow, proven by measurement (§7).
- **9** — all four configurations expressible, and no existing configuration
  loses a path.

## 12. Prohibited shortcuts

- Do not drop `"snora-widgets?/design"` (§5).
- Do not make `snora-style` a non-optional dependency of `snora-widgets` to
  simplify the re-export — that is exactly what §7 forbids.
- Do not "temporarily" change a public path intending to restore it.
- Do not modify `render_semantics.rs`.
- Do not mark anything deprecated in this release (§9).

## 13. Required evidence

- The new crate's `Cargo.toml` and module tree.
- Diffs of both feature graphs (`snora`, `snora-widgets`).
- **All four `cargo check` configurations** from §10, passing.
- The size comparison (§7), before and after, same run.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it empty.
- `architecture.md` and `feature-flags.md` diffs.

## 14. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under `.git-exclude/review-request/055-extract-the-style-bridge/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** §7. The extraction itself is mechanical; whether
the default configuration is genuinely unchanged is the part that decides
whether this is a free relocation or a silent cost imposed on every consumer
who never asked for `design`.
