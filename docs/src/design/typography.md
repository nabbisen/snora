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

**iced 0.14's own default line-height is `Relative(1.3)`** — text that
never calls `.line_height()` already renders at 1.3
(`iced_core::widget::text::Format::default()` sets
`line_height: LineHeight::default()`,
`iced_core-0.14.0/src/widget/text.rs:290`; `impl Default for LineHeight`
returns `Self::Relative(1.3)`, `iced_core-0.14.0/src/text.rs:215-219`).
Every role below is stated against that baseline, not only against each
other, so what applying a role's helper actually buys you is legible
from the table alone (RFC-070):

| Role | Size | Line height | vs iced's 1.3 default | Purpose |
|---|---|---|---|---|
| `body` | 16.0 | 1.4 | **+0.10, looser** | ordinary explanatory text |
| `body_small` | 14.0 | 1.35 | **+0.05, looser** | secondary metadata, compact help |
| `label` | 14.0 | 1.2 | −0.10, tighter | button, field and chip labels |
| `title` | 18.0 | 1.3 | **identical — no effect** | card / dialog / notice title |
| `heading` | 24.0 | 1.25 | −0.05, tighter | page or section heading |
| `display` | 32.0 | 1.2 | −0.10, tighter | rare major page title |

**Two roles are looser than the default, one matches it exactly, and
three are tighter — deliberately, because larger text tolerates (and
typically wants) less relative leading, the same reason the ladder
tightens as size grows.** The tighter roles are not a defect. `title`
is worth knowing about specifically: `title_line_height` restates the
renderer's own default, so calling it changes nothing observable — see
its own doc comment. See [Readability](../guides/readability.md) for
what this means in practice when picking a role for text that wraps.

**These are defaults a preset supplies, not constants.** All four built-in
presets (`light`, `dark`, `high_contrast_light`, `high_contrast_dark`) share
`Typography::default_roles()`, but `Typography` is a plain, non-`#[non_exhaustive]`
struct — an application supplying its own `Tokens` can set every field to
whatever its design calls for. See [Design tokens](tokens.md#customizing).

## Applying a role to your own text

```rust,ignore
{{#include ../../../examples/book_snippets/src/typography.rs:typography_applying_a_role}}
```

`snora::design::style::text::*` exposes one size helper and one
line-height helper per role — `body_size`/`body_line_height`,
`body_small_size`/`body_small_line_height`, `label_size`/`label_line_height`,
`title_size`/`title_line_height`, `heading_size`/`heading_line_height`,
`display_size`/`display_line_height` — returning `iced::Pixels` and
`iced::widget::text::LineHeight::Relative` respectively (RFC-068). Either
can still be bypassed by reading `tokens.typography.<role>.size` /
`.line_height` directly (`tokens.typography` is `pub`), but the helpers
are the recommended form: same module, same shape, discoverable
alongside each other.

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

**The floor is 12 logical pixels.** Nothing else — using a role
(`body`, `body_small`, or any other) is separate guidance about
keeping sizes in one place, not a second floor. See
[Readability](../guides/readability.md) for the full guidance,
including what's actually asserted for the four built-in presets
(RFC-081) and what stays the application's own responsibility on a
custom `Tokens`.
