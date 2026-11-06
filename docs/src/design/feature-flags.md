# Feature flags

## Snora feature graph

```text
snora
  ├── (always)          snora-core          — iced-free vocabulary
  ├── (feature=widgets) snora-widgets       — prefab iced elements
  │                       └── (feature=design, opt-in) snora-style — used internally
  └── (feature=design)  snora-design        — iced-free design tokens
                        snora-style         — iced style bridge (RFC-055)
```

`widgets` and `design` are **independent** (RFC-055) — neither requires
the other. `snora-widgets` also takes `snora-style` as an optional
dependency, activated by its own `design` feature, because its own
prefab widgets (`button`, `card`, `notice`, `chip`, `progress`) style
themselves with it internally — not to re-export it; `snora_widgets::
design::{style, theme}` were removed in RFC-056. That edge does not
make `snora`'s `design` feature depend on `widgets`.

## Flags

| Flag | Default | Activates |
|---|---|---|
| `widgets` | ✅ | `snora-widgets` crate; `snora::widget::*` re-exports |
| `iced-default` | ✅ | iced's own default features: renderers, windowing, `thread-pool`, `linux-theme-detection` (RFC-112). Also on `snora-widgets` and `snora-style`. |
| `design` | ❌ | `snora-design` token crate; `snora-style` iced style bridge; `snora::design::*` re-exports (`style`, `theme`, `render`, `responsive_render`, and — additionally, only when `widgets` is *also* on — `widget`, `button`, `card`, `notice`, `chip`, `progress`) |
| `lucide-icons` | ❌ | `Icon::Lucide`; `snora::lucide` constants. Independent of `design`. |
| `svg-icons` | ❌ | `Icon::Svg`; iced svg feature. Independent of `design`. |

## Four expressible configurations (RFC-055)

`widgets` and `design` are independent flags, so all four combinations
are reachable — three existed before RFC-055, `design` alone is new:

| `widgets` | `design` | What you get |
|---|---|---|
| ❌ | ❌ | Engine only — `AppLayout`, `render`, no prefab widgets, no design tokens |
| ✅ | ❌ | Today's default — engine + prefab widgets, no design tokens |
| ❌ | ✅ | **New** — engine + design tokens, `design::render`/`design::responsive_render`, `design::style::*`; no prefab widgets |
| ✅ | ✅ | Both — everything, including the prefab-widget re-exports under `snora::design` |

No existing configuration lost a path; `design` alone is purely
additive.

## Key invariants

- **`design` stays opt-in** until binary-size and build-cost are measured
  with and without it. `default = ["widgets"]` is unchanged. (RFC-054
  measured this: a `widgets`+`design` consumer with no widget call sites
  can recover ~95% of what `design` costs over engine-only by dropping
  `widgets`.)
- **`widgets` compiles without `design`** — existing applications are
  unaffected by adding `snora-design`/`snora-style` to the workspace.
- **`design` compiles without `widgets`** (RFC-055, new) — the
  configuration `examples/responsive_body`'s intended reader could not
  previously reach.
- **`snora-design` has no iced dependency** — the token crate is always
  iced-free (CI gate Q3). Adding one would be a CI failure.
- **`snora-style` depends on `iced` and `snora-design` only** — in
  particular not `snora-core` (RFC-054/RFC-055): the style layer is
  structurally below the widget layer, not beside it.
- **Icon features are independent of `design`** — `lucide-icons` and
  `svg-icons` work with and without the design feature.
- **Engine-only builds remain green** — `snora --no-default-features
  --features iced-default` compiles with no widgets and no design. Without
  `iced-default`, your own `iced` dependency must supply a renderer and a
  windowing backend, as every application's does.
- **The default configuration (`widgets`, no `design`) compiles no
  style code** — `snora-style` stays an optional dependency of
  `snora-widgets`, gated by the same `design` feature as before
  extraction; proven by measurement, not just by the feature
  declaration (`size_probe_widgets` is byte-for-byte unchanged across
  RFC-055's extraction — measured in the RFC-055 review package).

## Choosing iced's features yourself (`iced-default`, RFC-112)

**The rule, stated once:** an iced feature that decides *how snora draws* is
snora's, and always on. A feature that decides *what draws, or where it runs*,
is the application's. So `web-colors` and `crisp` are always on, because every
contrast figure snora publishes was measured with them. Everything else iced
enables by default reaches it through `iced-default`, which is on by default.
A plain `cargo add snora` is unchanged.

To choose them yourself, for example to drop `linux-theme-detection` and the
`zbus` tree behind it, turn snora's defaults off and name iced's features on
your own line:

```toml
snora = { version = "0.53", default-features = false, features = ["widgets", "design"] }
iced  = { version = "0.14", default-features = false, features = [
    "wgpu", "tiny-skia", "thread-pool", "x11", "wayland",
] }
```

snora never chooses a renderer: the renderers you list are the ones you get.
CI proves this recipe from a consumer's position (`examples/opt_out_probe`).

**The one case that stops compiling:** `default-features = false` on `snora`,
`snora-widgets` or `snora-style`, *together with* an `iced` line of your own that
also turns iced's defaults off and names no windowing backend. winit then
refuses the platform. Add `iced-default` back, or name the features you want.

## Cargo snippets

```toml
# Minimal (engine only)
snora = { version = "0.53", default-features = false }

# Default (layout + prefab widgets)
snora = { version = "0.53" }

# Design tokens + helpers, no prefab widgets (RFC-055 — new)
snora = { version = "0.53", default-features = false, features = ["design"] }

# Widgets + design tokens
snora = { version = "0.53", features = ["widgets", "design"] }

# Widgets + design + Lucide icons
snora = { version = "0.53", features = ["widgets", "design", "lucide-icons"] }
```

## Token-only use (no iced dependency in your crate)

Use `snora-design` directly for a pure-Rust token library with no iced
dependency:

```toml
snora-design = { version = "0.53" }
```

This is useful for testing token values, computing contrast in a CLI tool,
or building an alternative renderer against the token vocabulary.

## Does the token surface churn?

See [Stability](stability.md) for the contractual answer: `Tokens`,
`Palette`, and the rest of the token/style-bridge surface are under an
additive-only covenant, with one narrow accessibility-repair exception.
