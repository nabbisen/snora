# RFC-014-B — Focus and Modal Accessibility Boundary

**Status.** Implemented (v0.14.0)
**Tracks.** Accessibility boundary / modal semantics. Documentation only.
**Touches.** `docs/src/guides/overlays.md` (accessibility section),
`docs/src/contributing/design-decisions.md` (focus-trap record).

> Builds on RFC-011-E Law 8 (already states focus trap is out of scope).
> This RFC adds the application-responsibility checklist to the overlays
> guide and records the design decision more explicitly.

## 1. Summary

Add an accessibility boundary section to the overlays guide. No code.
The four-way distinction (visual modality, pointer blocking, keyboard
dismissal, focus trapping) from RFC-011-E Law 8 is already normative;
this RFC surfaces it in the guide that overlay users read first.

## 2. [Decision] No focus vocabulary this release

iced 0.14's focus primitives exist but are not stable enough at the
framework level to promise cross-platform behavior. The `operate`
machinery (used for `iced_test`) exposes `is_focused` selectors, but
building a reliable focus-trap on top requires platform-specific
attention. Defer until a concrete app need and a proven iced path exist.

`ModalAccessibilityPolicy` enum — deferred (planning draft §5).

## 3. Application responsibility checklist (overlay guide section)

```text
When building a dialog or sheet, your application should:

[ ] Provide a visible close/cancel button inside the overlay content.
[ ] Set `on_close_modals` when outside-click dismissal is intended.
[ ] Wire `Escape` via `snora::keyboard::dismiss_on_escape` if desired.
    (See the keyboard section of overlay-interaction-semantics.md.)
[ ] Use descriptive text for destructive actions; do not trigger them
    via backdrop click alone.
[ ] Consider focus: Snora does not trap keyboard focus. If your app
    needs initial focus inside a dialog, use iced's widget::Id and
    the `operate` mechanism.
```

## 4. Design-decisions.md record

Add a "Why no focus trapping yet" decision entry explaining:
- iced 0.14 exposes `operate`/`widget::Id` but a reliable cross-platform
  focus trap is unproven at the framework level.
- Law 8 (RFC-011-E) already states the boundary publicly.
- Reconsideration trigger: a concrete downstream app demonstrates the
  need and iced provides a stable API.

## 5. Acceptance criteria

- `overlays.md` has an accessibility section with the checklist.
- `design-decisions.md` records the focus-trap deferred decision.
- No new public API.
