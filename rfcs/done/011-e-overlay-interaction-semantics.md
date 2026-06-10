# RFC-011-E — Overlay Interaction Semantics

**Status.** Implemented (v0.11.0)
**Tracks.** Documentation / semantics. Writes the normative overlay-coexistence
and dismissal policy that future runtime RFCs extend.
**Touches.** `docs/src/reference/overlay-interaction-semantics.md` (new),
`docs/src/SUMMARY.md`, `crates/snora/src/render.rs` (doc comments only).

> Project-adopted version of planning-pack RFC-011-E. Content is largely
> unchanged because it is already detailed and correct; this version pins
> the wording so the v0.14 RFCs can extend it without a rewrite (see §12)
> and specifies the exact `render.rs` comment that must match it.

## 1. Summary

Write a normative overlay-interaction policy. The z-stack is already
fixed in code; this makes the *intended behavior of overlay combinations*
explicit so future changes and application usage stay consistent.

## 2. Goals

- Define overlay coexistence semantics and the eight "laws".
- Clarify click-outside dismissal, modal-vs-menu precedence, and
  toast-vs-modal layering.
- State what remains application-owned (Escape, focus).
- Add no code beyond doc-comment alignment unless a mismatch is found.

## 3. Non-Goals

- No focus trapping (RFC-014-B territory).
- No keyboard/Escape handling in the engine (RFC-014-A territory).
- No new overlay kinds, overlay manager, or per-overlay close hooks.

## 4. The overlay laws (normative)

### Law 1 — Z-stack is deterministic

Bottom → top:

```text
0. skeleton
1. menu backdrop
2. header_menu
3. context_menu
4. modal dim
5. dialog
6. sheet
7. toasts
```

Part of the framework contract. (Matches `crates/snora/src/render.rs`.)

### Law 2 — Menus are lightweight and below modal state

If a modal exists, modal state dominates menus visually and interactively.
Apps should close menus before opening a modal.

### Law 3 — Dialog and sheet may coexist (advanced)

Both render; the sheet is above the dialog. Supported because the z-stack
says so, but documented as advanced. Prefer one modal surface at a time.

### Law 4 — Close sinks are global per overlay class

Exactly two outside-click sinks: `on_close_menus` (header/context menus) and
`on_close_modals` (dialog/sheet). Individual overlay values carry no close
message. Intentional — it makes wiring impossible to get subtly wrong.

### Law 5 — Missing close sink does not hide content

If an overlay is populated but its sink is `None`, the engine still renders
the content. Modal overlays still paint the dim layer but install no
outside-click dismissal; menus render without the transparent backdrop.
Applications must then supply explicit close controls.

### Law 6 — Toasts are above modal state

Toasts render above modals so operational feedback stays visible during a
modal workflow. Use persistent error toasts sparingly.

### Law 7 — Keyboard dismissal is application-owned (for now)

Snora does not own `Escape` in this RFC. **Snora does not own application
shortcut routing.** Apps may map `Escape` to `CloseMenus` / `CloseModals`
via iced subscriptions/events. (RFC-014-A may later add a documented recipe
or a small optional helper; this wording is chosen so that RFC extends, not
contradicts, this section.)

### Law 8 — Focus management is out of scope until a concrete path exists

The modal dim/backdrop represents **visual modality and pointer blocking**,
**not** keyboard focus trapping or screen-reader modal semantics. Snora does
not overclaim accessibility. (RFC-014-B builds on exactly this distinction.)

## 5. Combination table

| Combination | Supported? | Recommended? | Notes |
|---|:--:|:--:|---|
| header menu only | yes | yes | Normal. |
| context menu only | yes | yes | Normal. |
| header + context menu | yes | rare | Usually show one menu surface. |
| dialog only | yes | yes | Normal modal. |
| sheet only | yes | yes | Normal workflow panel. |
| dialog + sheet | yes | advanced | Sheet renders above dialog. |
| menu + dialog/sheet | yes | discouraged | Modal dim dominates menus; close menus first. |
| toast + anything | yes | yes | Toasts always top; persistent toasts sparingly. |

## 6. Recommended state transitions (docs examples)

```rust
match msg {
    Message::OpenSettingsDialog => {
        self.header_menu = None;       // Law 2: close menus before modal
        self.context_menu = None;
        self.dialog = Some(DialogState::Settings);
    }
    Message::CloseModals => { self.dialog = None; self.sheet = None; }
    Message::CloseMenus  => { self.header_menu = None; self.context_menu = None; }
    _ => {}
}
```

App-owned Escape recipe (Law 7), prioritizing modal over menu:

```rust
match msg {
    Message::EscapePressed if self.dialog.is_some() || self.sheet.is_some() => {
        self.dialog = None; self.sheet = None;
    }
    Message::EscapePressed => { self.header_menu = None; self.context_menu = None; }
    _ => {}
}
```

## 7. New doc page outline

`docs/src/reference/overlay-interaction-semantics.md`:

1. Z-stack diagram (Law 1).
2. The eight laws.
3. Combination table.
4. Recommended state transitions.
5. "What Snora does not do" — Escape ownership (Law 7), focus/accessibility
   limits (Law 8), cross-linking the ABDD-is-layout-only statement.

Add under **Reference** in `docs/src/SUMMARY.md`, after
"Built-in widgets":

```markdown
- [Overlay interaction semantics](reference/overlay-interaction-semantics.md)
```

## 8. `render.rs` doc-comment alignment

The module header in `crates/snora/src/render.rs` already lists the layer
order; it must match Law 1 verbatim, and the "toasts … bottom-end" phrasing
in the layer comment must be corrected to "toasts — stacked at the
configured `ToastPosition` (RTL-aware), newest closest to the anchor edge"
to stay consistent with RFC-011-B. No behavioral code change.

## 9. [Forward-compat] Wording contract for RFC-014-A / -014-B

These two later RFCs *depend on this page*. To avoid a rewrite:

- Law 7 must say Snora does **not** own shortcut routing and that any future
  Escape support is a documented recipe / optional helper, never per-overlay
  hooks. (014-A extends here.)
- Law 8 must keep the precise four-way distinction — visual modality,
  pointer blocking, keyboard dismissal, focus trapping — and assert only the
  first two. (014-B extends here.)

The wording in §4 already satisfies both.

## 10. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Docs constrain future implementation. | Intentional; changes go through explicit RFCs. |
| Users expect a focus trap from "modal". | Law 8 states the limit plainly. |
| Dialog+sheet encourages complex UX. | Marked advanced, not default. |

## 11. Acceptance criteria

- `overlay-interaction-semantics.md` exists with z-stack, laws, table,
  transitions, and the "what Snora does not do" section.
- `SUMMARY.md` links it.
- `render.rs` doc comments match Law 1 and the corrected toast wording.
- Laws 7 and 8 use the forward-compatible wording in §9.
