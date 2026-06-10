# RFC-013-A — Anchored Popover Design Study

**Status.** Implemented (v0.13.0)
implementation**. Design study complete; all eight internal design questions
answered and recorded in `docs/src/contributing/anchored-popover-design.md`.

> v0.11 propagation: RFC-011-C satisfied — a future `popover` field on
> `AppLayout` would be additive and non-breaking.

**Tracks.** Future feature design / design study.
**Target release.** v0.13 (design study complete; implementation on demand).
**Touches.** `docs/src/contributing/anchored-popover-design.md` (new),
`docs/src/SUMMARY.md`.

## 1. Summary

Design study for an anchored popover overlay. Deliverable is a written
decision with the eight internal design questions answered.
Implementation deferred until a concrete consuming application exists.

## 2. [Decisions] Eight internal design questions answered

### Q1: Can iced reliably provide anchor geometry to the application?

**Yes, via `widget::Id` + the `operate` machinery.** iced 0.14 exposes
`UserInterface::operate` (the same mechanism `iced_test::Simulator::find`
uses) which traverses the widget tree and can return a widget's
`visible_bounds: Rectangle`. An application can assign `widget::Id` to
the anchor widget, run an `operate` pass in a `Task`, and receive the
`Point`/`Rectangle` it needs.

This is not a hack — it is the intended iced path for programmatic widget
queries. However, it requires an async `Task` round-trip: the application
sets up the popover anchor on one frame and the bounds are available on the
next. Applications must be prepared to handle this one-frame delay.

### Q2: Should Snora accept only application-provided geometry?

**Yes, initially.** The iced `operate` round-trip is application-owned.
Snora does not have access to the widget tree layout outside of `render()`.
`Popover` therefore accepts only `PopoverAnchor::Point` or
`PopoverAnchor::Rect` — application-provided values. Snora cannot auto-
derive the anchor from a widget reference; that would require coupling to
iced's internal layout phase in ways that would make the API fragile.

### Q3: Does popover belong to the menu close sink or a new close sink?

**New close sink `on_close_popovers`.** A popover's dismiss semantics are
distinct from both menus (`on_close_menus`) and modals (`on_close_modals`).
Conflating with either causes user confusion:
- Menu backdrop dismisses all open menus at once; a popover may coexist
  with an open menu.
- Modal dim is visually heavy; popovers are typically lightweight.

The cost is a third close channel — a genuine tradeoff against the
current two-channel simplicity. This study affirms Option B from the
planning draft when/if implementation proceeds.

### Q4: Should popovers be modal or non-modal?

**Non-modal.** A popover anchored to a widget is lightweight by nature.
The modal dim layer must not render behind a popover. If an application
needs a modal popover, it should use `Dialog` instead.

### Q5: Should multiple popovers be supported or only one?

**One at a time initially.** A single `Option<Popover<…>>` field on
`AppLayout`. Multiple simultaneous popovers are not evidenced by any
downstream app.

### Q6: Should collision detection be explicitly out of scope?

**Yes.** `Auto` placement is rejected for the initial implementation.
Applications choose `Above`, `Below`, `Start`, or `End`; Snora positions
accordingly. Viewport-edge correction would require a second `operate`
round-trip — added complexity without a proven need.

### Q7: How does popover interact with `AppLayout` construction stability?

No blocker. RFC-011-C is in effect since v0.11. A `pub popover:
Option<Popover<…>>` field with a `#[must_use]` `.popover()` builder is a
non-breaking additive change per RFC-011-C §7.2.

### Q8: What layer does popover occupy?

**Between context_menu (layer 3) and modal dim (layer 4)**, with its own
transparent backdrop. Full z-stack if popover is added:

```text
0. skeleton
1. menu backdrop        (on_close_menus)
2. header_menu
3. context_menu
4. popover backdrop     (on_close_popovers, transparent)
5. popover content
6. modal dim            (on_close_modals)
7. dialog
8. sheet
9. toasts
```

## 3. Decision: defer implementation

**Condition for implementation:** at least one concrete consuming application
demonstrating a use case that `context_menu` does not naturally cover.

The workbench example uses `context_menu` for its floating menu — the
primary popover use case. Until a real downstream app hits a wall with
`context_menu` or needs the widget-anchored positioning specifically,
implementation is premature.

## 4. Deliverable: design page

`docs/src/contributing/anchored-popover-design.md` records the above
analysis. Linked from the contributing SUMMARY and the adding-an-overlay
guide.

## 5. Acceptance criteria

- Design page exists with all eight questions answered.
- Decision (defer) is recorded with the trigger condition.
- Z-stack with hypothetical popover layer documented.
- No implementation code produced.
