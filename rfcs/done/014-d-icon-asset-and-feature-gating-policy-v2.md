# RFC-014-D — Icon, Asset, and Feature-Gating Policy v2

**Status.** Implemented (v0.14.0)
**Tracks.** Feature policy / dependency control.
**Touches.** `docs/src/guides/icons.md` (why-gated section),
`docs/src/contributing/feature-gating-criteria.md` (icon-policy note).

## 1. [Decision] Documentation only; no code or feature name changes

The existing feature set (`widgets`, `lucide-icons`, `svg-icons`) is
correct. The CI matrix already covers all meaningful combinations
(RFC-011-A). No feature names change — renaming without strong reason
is an anti-pattern per the planning draft.

Open question resolved: `lucide-icons` and `svg-icons` are subordinate
to `widgets`. The CI matrix deliberately omits `--features lucide-icons`
without `widgets`; this is already documented in `feature-gating-criteria.md`
(RFC-011-A §7.2). The icons guide should state this explicitly.

## 2. icons.md additions

Add two sections:

### "Why icons are feature-gated"

Lucide ships ~1500 icon constants; the `lucide-icons` crate is not
trivial in compile time. SVG icons require iced's `svg` feature and
a file-loading path. Keeping both optional means:

- engine-only builds stay small;
- projects that use only `Icon::Text` pay nothing for the icon packs;
- the CI matrix can verify each combination independently.

### "Feature combinations"

```toml
snora = { version = "..." }                                        # widgets (default)
snora = { version = "...", default-features = false }              # engine only
snora = { version = "...", features = ["widgets", "lucide-icons"] }
snora = { version = "...", features = ["widgets", "svg-icons"] }
snora = { version = "...", features = ["widgets", "lucide-icons", "svg-icons"] }
```

Note: `lucide-icons` and `svg-icons` require `widgets`. The CI matrix
verifies these combinations automatically on every PR.

## 3. feature-gating-criteria.md addition

Add a short icon-policy cross-reference section pointing to `icons.md`.

## 4. ABDD check

Not direction-sensitive. ABDD does not apply.

## 5. Acceptance criteria

- `icons.md` has "Why icons are feature-gated" section.
- `icons.md` shows all four supported `Cargo.toml` snippets.
- `feature-gating-criteria.md` references icon policy.
- No feature name changes.
- CI matrix unchanged (already correct).
