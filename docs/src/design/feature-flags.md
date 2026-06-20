# Feature flags

## Snora feature graph

```text
snora
  ├── (always)          snora-core          — iced-free vocabulary
  ├── (feature=widgets) snora-widgets       — prefab iced elements
  └── (feature=design)  snora-design        — iced-free design tokens
                        snora-widgets/design — iced style bridge
```

## Flags

| Flag | Default | Activates |
|---|---|---|
| `widgets` | ✅ | `snora-widgets` crate; `snora::widget::*` re-exports |
| `design` | ❌ | `snora-design` token crate; `snora-widgets/design` style bridge; `snora::design::*` re-exports. Requires `widgets`. |
| `lucide-icons` | ❌ | `Icon::Lucide`; `snora::lucide` constants. Independent of `design`. |
| `svg-icons` | ❌ | `Icon::Svg`; iced svg feature. Independent of `design`. |

## Key invariants

- **`design` stays opt-in** until binary-size and build-cost are measured
  with and without it. `default = ["widgets"]` is unchanged in v0.20.
- **`widgets` compiles without `design`** — existing applications are
  unaffected by adding `snora-design` to the workspace.
- **`snora-design` has no iced dependency** — the token crate is always
  iced-free (CI gate Q3). Adding one would be a CI failure.
- **Icon features are independent of `design`** — `lucide-icons` and
  `svg-icons` work with and without the design feature.
- **Engine-only builds remain green** — `snora --no-default-features`
  compiles with no widgets and no design.

## Cargo snippets

```toml
# Minimal (engine only)
snora = { version = "0.24", default-features = false }

# Default (layout + prefab widgets)
snora = { version = "0.24" }

# Design tokens + helpers
snora = { version = "0.24", features = ["widgets", "design"] }

# Design + Lucide icons
snora = { version = "0.24", features = ["widgets", "design", "lucide-icons"] }
```

## Token-only use (no iced dependency in your crate)

Use `snora-design` directly for a pure-Rust token library with no iced
dependency:

```toml
snora-design = { version = "0.24" }
```

This is useful for testing token values, computing contrast in a CLI tool,
or building an alternative renderer against the token vocabulary.
