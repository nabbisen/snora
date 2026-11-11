# RFC-126 — F6 zone navigation cannot move focus, and does not know where focus is

**Status.** Proposed 2026-11-12.
**Raised.** 2026-11-12, architect, scoping 0.61. It began as one finding while scoping 0.60:
`next_zone` ignores an open menu. Reading the whole feature found that it cannot work as
documented.
**Release target.** 0.61.0, with RFC-127.
**Priority.** A documented feature that does nothing visible, and a rule ("covered") it
contradicts. Keyboard compatibility is required (owner, 2026-11-04).

---

## The finding

RFC-060 (0.35) gave snora **frame-level zone navigation:** F6 and Shift+F6 move between the
header, the sidebar, the body and the footer, as the desktop convention has it. It shipped as
a pure decision, and the application does the rest:

```rust,ignore
if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
    let next = snora::focus::next_zone(self.focus_zone, cycle, self.zone_presence(),
                                       self.show_dialog || self.show_sheet,
                                       self.open_menu.is_some());
    if let Some(zone) = next {
        self.focus_zone = zone;
        return self.focus_task_for(zone); // application-owned
    }
}
```

**Four things are wrong with it.** Each is read from the code and the book, and Unit 0
measures them.

1. **An application cannot move focus into a zone.** `focus_task_for` is left to the
   application, and it has nothing to write it with:
   - every snora control's `widget::Id` is made inside it (`Id::unique()`) and cannot be set,
     so the header's trigger, a tab, a sidebar item or a `design::button` cannot be the
     target of iced's `focus(id)`;
   - iced 0.14 has no "focus the first control inside this container" without its `advanced`
     feature, which snora does not enable.

   The accessibility guide's example stops at `self.focus_zone = zone;`, so **following the
   book, F6 changes a variable and moves no focus.**
2. **It does not know where focus is.** The current zone is the application's own variable,
   changed only by F6. A reader who Tabs or clicks into the sidebar and then presses F6 moves
   from the zone the variable remembers, not from the sidebar. RFC-060 chose this because
   finding focus needed iced's `advanced` feature. **Since 0.55 that is no longer true for
   snora:** its own widgets run focus operations (`Inert`, `MenuKeys`) on `iced_core`.
3. **It contradicts "covered".** Since 0.55, a menu with its backdrop covers the page for the
   keyboard (RFC-119), and since 0.60 for the pointer too (RFC-125). `next_zone` documents that
   "menus do not own focus" and **ignores `has_menu`**. Its parameter is kept "only to mirror
   `dismiss_on_escape`'s shape", and that function was removed in 0.60.
4. **Zones without a control are not skipped.** `ZonePresence` says whether a slot exists, not
   whether anything in it can take focus. A footer of plain text is a stop where F6 lands
   nowhere.

## Proposal

**R-1. F6 and Shift+F6 move focus to the first control of the next and the previous zone,**
counted from **the zone that holds focus now**, read from the tree when the key is pressed.
- The order is Header → SideBar → Body → Footer, wrapping. It is logical, so it does not mirror
  under RTL (RFC-060).
- **A zone with no control that can take focus is skipped,** whether it is absent or holds
  none.
- **With focus in no zone** (nothing focused, or a toast): F6 goes to the first zone that has
  a control, and Shift+F6 to the last.
- **An open menu's dropdown belongs to its trigger's zone.**
- **F6 never presses anything.** A held F6 moves once per press, as Tab does.

**R-2. While the page is covered, F6 does nothing:** under a dialog or a sheet, a menu with its
backdrop, or an open `app_header` dropdown with a close message. That is "the keyboard does
what the pointer can": the zones are beneath the cover. Inside a dialog, F6 has no zones to
move between.

**R-3. snora does the work, and the application routes the key, as it does Tab.**
- **The action:** `snora::keyboard::focus_zone(cycle) -> Task<Message>` (name: Q-1). It runs one
  window-wide operation that finds the zone with focus and focuses the target. The engine
  already marks the four regions (`snora-header` … `snora-footer`, RFC-047).
- **The binding is unchanged:** `snora::keyboard::cycle_zones(key, modifiers)` still turns F6 and
  Shift+F6 into a `Cycle`. An application that wants another key maps that key itself.

```rust,ignore
if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
    return snora::keyboard::focus_zone(cycle);
}
```

- **Why the application routes it,** and the engine does not answer F6 by itself: an
  application may already use F6 (a debugger's "step over"), and an upgrade must not take a
  key away silently. The rule a reader can predict: **snora answers keys inside its own
  controls and overlays (Enter, Space, arrows in a menu, Escape). Keys that move between
  controls (Tab, F6) are routed to it by the application.**

**R-4. The application's zone vocabulary is deprecated, and removed in 0.64.0:**
`snora_core::focus::next_zone`, `FocusZone` and `ZonePresence`, and their re-exports. Each
note names `focus_zone` and the removal version. `Cycle` and `cycle_zones` stay: they are the
binding, and still used.
- **This is superseded, not merely old.** The zone with focus, the zones present, and what is
  covered are now read from the tree. An application can no longer pass a stale answer, and
  it has nothing left to pass.

**R-5. The book:** the keyboard map gains an F6 row, the accessibility guide's example is
replaced, and Law 8's zone row is rewritten.

## Open questions, with suggested rulings

**Q-1 — the action's name.** **Suggest `snora::keyboard::focus_zone(cycle)`:** a verb for what it
does, next to `traverse`. `next_zone` would collide in meaning with the deprecated
`focus::next_zone`, which returns a zone rather than moving focus.

**Q-2 — where focus lands in a zone: the first control, or the one last focused there?**
**Suggest the first control.** Windows returns to the last focused control in a pane. That
needs a memory per zone, which goes stale when the zone's controls change (RFC-119's opener
memory and RFC-120 show how much care that takes). "The first control" is one sentence a
reader can predict. The memory goes to RFC-116's long-term plan as **future wanted**.

**Q-3 — the 1.0 gate 2 ("two consecutive minors without vocabulary churn").** R-4 removes
`snora-core` vocabulary in 0.64. **The gate's row was last re-derived at 0.54,** and `snora-core`
items have been removed since: `ToastLifetime::DEFAULT` (0.57), and `ToastHover` and
`AppLayout::on_toast_hover` (0.59). Whether a later removal reopens a gate ticked at
v0.13–v0.16 is a definition, like gate 3's two words. **Suggest:** the architect re-derives the
row with these facts at 0.61, and the owner rules on the definition. R-4 does not wait for
that ruling: keeping a superseded API alive to protect a tick would be the wrong way round.

## Review against snora's principles

- **Nobody is confused.** F6 does what the book says, from where the reader is, and lands on
  something. A reader who Tabs and then presses F6 gets the zone after the one they are in.
- **Safe.** It moves focus only. It never presses, and never reaches beneath a cover.
- **Predictable.** One rule for F6, and one sentence for which keys snora answers by itself.
- **Clean.** It replaces three items and an application-held variable with one function. The
  application writes two lines and no state.
- **Robust.** The answer comes from the tree at the key press. Nothing is remembered, so nothing
  can go stale.
- **The initial cost is not the deciding factor.** The cheap fix, honouring `has_menu`, would
  leave points 1, 2 and 4: F6 would still move no focus.
- **ABDD.** Logical order, unchanged under RTL, as RFC-060 ruled.
- **Keyboard.** Completes the second keyboard axis RFC-060 promised.

## What this is not

- Not a new binding: F6 and Shift+F6, as since 0.35.
- Not a memory of the last control per zone (Q-2).
- Not zones inside the body (an application's own panes). The four skeleton regions only.
- Not RFC-127, which lets an application point focus at a snora control of its own.

## Acceptance criteria

1. **Unit 0** measures findings 1, 2 and 4 on `main`:
   - whether any focus task an application can write reaches a snora control in a zone;
   - F6 after a Tab into another zone;
   - a zone with no control.

   It also prototypes R-1 and R-2 on both renderers.
2. **Unit 1:** R-1 to R-4, with a test and a breakage row for each rule. `render_semantics.rs` is
   unmodified, and `every_control.rs` takes a subject per zone.
3. **The book,** by the architect (R-5).
