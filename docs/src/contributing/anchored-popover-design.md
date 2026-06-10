# Anchored popover design

This page records the design study for a potential anchored-popover overlay
in Snora. It answers the eight internal design questions from RFC-013-A and
records the decision to defer implementation until a concrete consuming
application exists.

## What a popover would be

An anchored popover is a lightweight, non-modal overlay positioned relative
to a widget, point, or rectangle. It fills the gap between:

- `header_menu` / `context_menu` (lightweight, not anchored to a specific
  widget);
- `Dialog` (centered modal);
- `Sheet` (edge-anchored modal).

Use cases: combobox dropdowns, hover cards, inspector bubbles, inline
contextual detail panels.

## Why it is not implemented yet

The trigger is unmet: no concrete consuming application has demonstrated a
need that `context_menu` does not cover. The workbench example uses
`context_menu` successfully for its floating menu surface. Until a real
app hits a wall, implementation is premature.

## Design questions and answers

### 1. Can iced provide anchor geometry?

Yes. iced 0.14 exposes `UserInterface::operate` (the same mechanism
`iced_test::Simulator::find` uses) which returns a widget's
`visible_bounds: Rectangle` given a `widget::Id`. An application assigns
`widget::Id` to the anchor widget, runs an `operate` pass in a `Task`, and
receives the bounds it needs.

This requires a one-frame async round-trip; the popover is positioned on
the frame after the anchor bounds are received. Applications must handle
this delay.

### 2. Should Snora accept only application-provided geometry?

Yes, initially. Snora does not have access to widget layout outside of
`render()`. `Popover` would accept only `PopoverAnchor::Point` or
`PopoverAnchor::Rect` — both application-provided. Auto-derivation from
a widget reference would couple Snora to iced's layout phase in fragile ways.

### 3. Does popover need its own close sink?

Yes — a new `on_close_popovers: Option<Message>` field on `AppLayout`.
Popover dismiss semantics are distinct from menus (`on_close_menus`) and
modals (`on_close_modals`). A popover may coexist with an open menu; the
menu backdrop must not dismiss it. This adds a third close channel — a
tradeoff against the current two-channel simplicity.

### 4. Modal or non-modal?

Non-modal. Popovers are lightweight by nature. The modal dim must not
render behind a popover. Applications needing a modal popover should use
`Dialog`.

### 5. One or many?

One at a time, initially: `pub popover: Option<Popover<Node, Message>>` on
`AppLayout`. Multiple simultaneous popovers are an advanced case without
downstream evidence.

### 6. Collision detection?

Explicitly out of scope initially. `PopoverPlacement::Auto` is rejected.
Applications choose `Above`, `Below`, `Start`, or `End`; Snora positions
the popover there. Viewport-edge correction requires a second `operate`
round-trip — complexity without a proven need.

### 7. Impact on `AppLayout` construction stability?

None. RFC-011-C (`#[non_exhaustive]` on `AppLayout`) is in effect since
v0.11. A new `popover` field would be a non-breaking additive change,
provided a `#[must_use]` `.popover()` builder ships in the same PR.

### 8. Layer assignment?

Between context_menu (layer 3) and modal dim (layer 4), with its own
transparent backdrop. If implemented, the full z-stack becomes:

```text
0. skeleton
1. menu backdrop          (on_close_menus)
2. header_menu
3. context_menu
4. popover backdrop       (on_close_popovers, transparent)
5. popover content
6. modal dim              (on_close_modals)
7. dialog
8. sheet
9. toasts
```

Modal state (layers 6–8) still dominates popover state.

## Proposed vocabulary (for reference, not adopted)

```rust,no_run
pub struct Popover<Node, Message>
where
    Message: Clone,
{
    pub content: Node,
    pub anchor: PopoverAnchor,
    pub placement: PopoverPlacement,
}

pub enum PopoverAnchor {
    Point { x: f32, y: f32 },
    Rect { x: f32, y: f32, width: f32, height: f32 },
}

pub enum PopoverPlacement {
    Above,
    Below,
    Start,  // logical — mirrors under RTL
    End,    // logical — mirrors under RTL
    // Auto intentionally absent
}
```

`Start` and `End` must resolve through `LayoutDirection` per ABDD.

## Trigger condition for implementation

At least one concrete consuming application that needs widget-anchored
positioning and cannot be reasonably served by `context_menu`. Open an
issue with a concrete scenario.

## Related

- [Adding a new overlay kind](adding-an-overlay.md)
- [Overlay interaction semantics](../reference/overlay-interaction-semantics.md)
- RFC-013-A in `rfcs/proposed/`
