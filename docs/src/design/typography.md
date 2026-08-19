# Typography

Snora Design carries a six-role typography scale: each role pairs a font
**size** with a **line-height** multiplier, and both are usable today through
public API — no code change to snora required.

## The six roles

```rust,ignore
pub struct Typography {
    pub body: TextRole,        // ordinary explanatory text
    pub body_small: TextRole,  // secondary metadata, compact help
    pub label: TextRole,       // button, field and chip labels
    pub title: TextRole,       // card / dialog / notice title
    pub heading: TextRole,     // page or section heading
    pub display: TextRole,     // rare major page title
}

pub struct TextRole {
    pub size: f32,        // logical pixels
    pub line_height: f32, // multiplier, e.g. 1.4 = 140% of size
}
```

| Role | Size | Line height | Purpose |
|---|---|---|---|
| `body` | 16.0 | 1.4 | ordinary explanatory text |
| `body_small` | 14.0 | 1.35 | secondary metadata, compact help |
| `label` | 14.0 | 1.2 | button, field and chip labels |
| `title` | 18.0 | 1.3 | card / dialog / notice title |
| `heading` | 24.0 | 1.25 | page or section heading |
| `display` | 32.0 | 1.2 | rare major page title |

**These are defaults a preset supplies, not constants.** All four built-in
presets (`light`, `dark`, `high_contrast_light`, `high_contrast_dark`) share
`Typography::default_roles()`, but `Typography` is a plain, non-`#[non_exhaustive]`
struct — an application supplying its own `Tokens` can set every field to
whatever its design calls for. See [Design tokens](tokens.md#customizing).

## Applying a role to your own text

```rust,ignore
use iced::widget::text::LineHeight;

iced::widget::text("wrapping prose")
    .size(snora::design::style::text::body_size(&tokens))
    .line_height(LineHeight::Relative(tokens.typography.body.line_height))
```

`snora::design::style::text::*` exposes one size helper per role
(`body_size`, `body_small_size`, `label_size`, `title_size`, `heading_size`,
`display_size`), each returning `iced::Pixels`. Line-height is not wrapped in
a helper — read the multiplier straight off `tokens.typography.<role>.line_height`
and pass it to iced's own `LineHeight::Relative`, as above. `tokens.typography`
is `pub`.

See [Readability](../guides/readability.md) for how to pick a role and why
line-height matters for prose.

## What snora's own widgets use today

**snora uses two of these six roles: `label` and `body`.** `body_small`,
`title`, `heading` and `display` are applied by **nothing in any snora crate**
— not `snora-widgets`, not the engine, not the style bridge. They exist for the
application's own text, not for snora's chrome.

That is a **commitment, not just a description of the current source**: if a
snora widget ever starts applying one of the other four roles, that is a
rendered change and will be announced as one. It is stated as a commitment
because consumers act on it — an application can safely redefine
`body_small`, `title`, `heading` or `display` on its own `Tokens` knowing snora's
chrome will not shift underneath it. Redefining `label` or `body` *will* reach
snora's widgets.

The previous wording scoped this to `snora-widgets` alone, which did not answer
the question for a consumer reaching those widgets through `snora::design::*`
(knotra, 2026-08-19, who audited all four crates rather than ask).

One exception worth naming plainly: the [notice](notices.md) widget renders
its *title* at `label_size`, not `title_size`. That is a known gap, named
here rather than left implicit. Widening role coverage in the prefab widgets
is a deliberate, deferred appearance change (tracked separately), not part
of this page's scope. This page teaches the vocabulary; it does not claim
snora renders a full type hierarchy on your behalf.

## Accessibility floor

Text in notices, labels and help content should use at least `body` or
`body_small` — never a custom size below 12 logical pixels. See
[Readability](../guides/readability.md) for the full guidance.
