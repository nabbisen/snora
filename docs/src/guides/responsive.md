# Responsive layout width

`snora::responsive_render` exposes the layout's available width to your
application. It does not decide anything about what you do with it —
**thresholds are your decision, not snora's.**

```rust,ignore
use snora::{AppLayout, responsive_render};

fn view(state: &State) -> Element<'_, Message> {
    responsive_render(move |width| {
        let layout = AppLayout::new(body(state));

        if width < 600.0 {
            layout // your own choice: no sidebar below 600px
        } else {
            layout.side_bar(sidebar(state))
        }
    })
}
```

`build` is called with the width available to the layout, in logical
pixels, and returns the `AppLayout` to render at that width. It may be
called again whenever the available size changes.

## Why exposure, not behavior

snora has consistently declined to make decisions like this on your
behalf — no theming layer beyond opt-in tokens, no form widgets, no
prescribed layout beyond the skeleton itself. A `Breakpoint` enum or an
auto-collapsing sidebar would decide *for* you: which threshold, which
element, what "collapsed" means. `responsive_render` prescribes nothing.
You supply the content and the thresholds; snora supplies the number.

If your application converges on thresholds that feel like they should
be shared vocabulary, that is useful evidence for a future RFC — but it
requires this shipping first, and no breakpoint behavior is planned
without a concrete downstream case to draw thresholds from.

## `f32` width, not the full `Size`

`iced::widget::Responsive` (which this wraps) hands its closure the full
available `Size`; `responsive_render` narrows that to `.width` only,
since width is what motivated this in the first place and is the
narrower, more conservative contract. If you need height too, use
`iced::widget::Responsive` directly and build your `AppLayout` inside
its closure — the same shared z-stack (`snora::render`) is available to
call from there.

## A sibling to `render`, not a replacement

`snora::render` is unchanged; applications that do not call
`responsive_render` are unaffected. This is engine capability, not
`design`-gated — it belongs in the default surface next to `render`,
the same way `snora::design::render` (RFC-039) sits beside it for
token-derived styling.

## Try it

```text
cargo run -p snora-example-responsive
```

Resize the window narrower than ~600px and back — the example's sidebar
drops and returns. The threshold and what happens at it are the
example's own choice, stated in its own source, not snora's.

`examples/responsive` is for slot-based chrome — an `AppLayout::side_bar`
that collapses. If your chrome is composed into `body` instead (no
`snora::widget::*`, no `side_bar`), see `examples/responsive_body` — the
same width number, varying a tab bar instead of a slot.
