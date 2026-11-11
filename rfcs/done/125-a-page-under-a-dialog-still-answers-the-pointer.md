# RFC-125 — A page under a dialog still answers the pointer

**Status.** Done — shipped v0.60.0. Accepted 2026-11-11 (owner, scoping 0.60: the proposal, Q-1 and Q-2 as
recommended).
**Handoff.** [`handoffs/125-…`](../handoffs/125-a-page-under-a-dialog-still-answers-the-pointer/implementation-handoff.md).
**Raised.** 2026-11-11, architect, while scoping 0.60. The hover half was measured by the dev
team in RFC-124 Unit 1 (`.git-exclude/reviewed/124-unit1/`). The wheel half was found in
the code while scoping.
**Release target.** **0.60.0, first** (owner, Q-2). RFC-123 follows, because both change
`Inert`'s cover.
**Priority.** The wheel half can throw away what a reader typed. The hover half is the
confusion RFC-124's ruling 3 already removed under menus.

---

## The finding

Since 0.59, **a page covered by an open header dropdown gets no pointer**: no hover, no
pointer shape, no tooltip, no wheel (RFC-124, ruling 3). The keyboard cannot reach it, and
neither can the pointer. "Covered" means one thing there.

Under a dialog or a sheet it means something else.

1. **The page shows hover under the dim.** Measured
   (`menu_overlay.rs`, `does_a_page_under_a_modal_dim_show_hover`): pointing at an iced
   button under a dialog's dim changes 3,373 px, against 3,385 px with no dialog. The
   tint promises a click that the dim will not let through.
2. **A wheel scroll over the dim closes the dialog.** *Read in the code, not yet run:*
   `crates/snora/src/render.rs:514-519` gives the dim's `mouse_area` an `on_scroll` that
   publishes the same message as `on_press`, which is `on_close_modals`.
   - **Where it came from.** RFC-084 (0.41) asked only that the dim *block* scrolling
     (F-03). Publishing the close message was how the implementation captured the wheel. It
     was never ruled, it is in no book page, and the tests assert only that the page
     receives nothing (`render_semantics.rs:311`).
   - **Why it matters.** A reader scrolls a long dialog with a two-finger swipe. The
     pointer strays off the card onto the dim, and the dialog closes, with whatever they
     typed. The same applies to a sheet. No desktop platform's modal dialog closes on a
     wheel.
   - **And once is not once.** A swipe sends many wheel events, and each one publishes the
     close message. An application whose close handler toggles would reopen the dialog,
     then close it again.
3. **The hand cursor shows over a page control under the dim** (measured in Unit 0).
4. **Under a context menu, or an application's `header_menu`, with its backdrop, the page
   answers the pointer as if nothing covered it** (measured in Unit 0): the full hover, the
   hand cursor, and **a wheel over the backdrop scrolls the page.** That page is covered by
   `Inert`'s flag, and only the header dropdown's cover withholds the pointer
   (`inert/menu_cover.rs`). The book's Law 8 row overclaimed for these two, and was corrected
   on 2026-11-11.
5. **A drag that began on the page keeps moving under a cover** (measured in Unit 0: a
   slider, under a dialog and under a context menu). The release does end it.

**What is already right** (measured in Unit 0):
- **a page tooltip is not drawn under any cover,** because `Inert` hides a covered layer's
  overlays;
- **a dialog with no close message** already gives the page no hover and the default cursor,
  because its dim is `opaque`. Its wheel status is `Ignored`, though, not `Captured`.

## The rule

> **A covered layer gets no pointer.** Whatever the keyboard cannot reach (RFC-119), the
> pointer does not reach either: no hover, no pointer shape, no tooltip, no wheel. What
> covers it is a surface. A press on it (a click or a tap) and Escape close what is open
> (Law 7). Nothing else it receives does anything.

## Proposal

**R-1. Covered means one thing.** Every layer `Inert` covers gives its content no cursor,
for `update` and `mouse_interaction` alike, as the menu cover does today:
- under a dialog or a sheet, **with or without** `on_close_modals`;
- under a menu with its backdrop (`on_close_menus`), context menus and `header_menu`
  included;
- under an open header dropdown with a close message (unchanged from 0.59).

**R-2. The wheel over a backdrop does nothing.** Over a dim or a menu's backdrop, a wheel
event publishes nothing, reaches nothing beneath, and is captured (`event::Status::Captured`),
as a press on it is.

**R-3. Unchanged:**
- a press on the dim publishes `on_close_modals`; a press on a menu's backdrop publishes
  `on_close_menus`;
- Escape follows Law 7;
- a dim with no close message blocks the pointer and publishes nothing;
- the dialog, the sheet and the open menu get the pointer as before;
- toasts get the pointer and the keyboard (Law 6);
- the keyboard: RFC-119 is untouched;
- window events and redraws reach a covered layer, so an animation beneath keeps
  running.

**R-4. Nothing beneath acts on the pointer once it is covered.** A pointer resting on a page
control when an application opens a dialog shows no hover from the first frame under the
dim. A drag that started on the page before the cover appeared does not move anything while
covered. It ends at the release, as it already does.

**Where R-2 lives** (architect, at Unit 0's review): in `Inert`'s cover, beside the rule that
withholds the cursor. A covered layer captures a wheel that nothing above it took. The layer
above (a dialog's card, a menu's list) is offered the wheel first. From outside, this is R-2:
nothing beneath moves, nothing is published, and the status is `Captured`. That includes the
dim with no close message.

## What Unit 0 changed (2026-11-11)

- **The tooltip inference was wrong:** no page tooltip is drawn under any cover. It is now
  under "already right".
- **Under a menu's backdrop the page scrolls.** That is stronger than hover, and it is now
  finding 4.
- **R-4's release half already holds.** Only "does not move while covered" is new.

None of the rules changed.

## Review against snora's principles

- **Nobody is confused.** A hover tint is a promise that a click will act. Under a dim it
  was a false one. One rule now covers every cover, so a reader who has learned one has
  learned them all.
- **Safe.** A dialog no longer closes because a scroll strayed off it, so what the reader
  typed is not lost. One gesture can no longer publish a burst of close messages.
- **Predictable.** The rule is one sentence, and the same as the keyboard's: what the
  keyboard cannot reach, the pointer cannot reach.
- **Platforms.** Windows, GNOME, KDE and macOS dialogs do not close on a wheel, and do not
  hover the window behind a modal.
- **The initial cost is not the deciding factor.** The cheaper fix (hover only, leaving the
  wheel as it is) would keep the unsafe half, which no reader can predict.
- **ABDD.** No direction is involved. RTL is unchanged.
- **Keyboard.** Unchanged. This brings the pointer into line with it.
- **Clean.** No public API changes. It is a behaviour change, named under **Changed**.
  An application that relied on a wheel closing a dialog relied on something no page
  documented. The migration guide names it.
- **Gate 5.** `render_semantics.rs` passes **unmodified**. Its two wheel tests assert that
  the page receives nothing, and that still holds.

## What this is not

- **Not a change to touch.** A finger that lands on the dim is a press, and closes the
  dialog (Law 7), as before.
- **Not a change to what is drawn.** The dim, its colour, and the page beneath look the
  same, except that no hover shows.
- **Not RFC-123.** Arrow keys in menus follow this RFC.

## Acceptance criteria

1. **Unit 0** measures each finding on both renderers before anything changes: the wheel
   over the dim (and the messages it publishes), the pointer shape and a tooltip under the
   dim, and the three for a page under a context menu and a `header_menu` with its backdrop.
2. **Unit 1:** R-1 to R-4, each with a test and a breakage row shown failing. Hover is read
   from **pixels**.
3. `render_semantics.rs` is unmodified, and passes.
4. **The book,** by the architect: Law 7, Law 8's pointer row, the dialogs and menus guides.

## Rulings, 2026-11-11 (owner, scoping 0.60)

- **The proposal is accepted:** a page under a dialog gets no hover, tooltip or pointer
  shape, and the wheel over the dim does nothing.
- **Q-1:** the wheel over a dialog's dim does nothing, rather than closing the dialog.
- **Q-2:** 0.60.0 waits for this RFC, RFC-123 and the `dismiss_on_escape` removal. This RFC
  goes first. If RFC-123's measurement shows rule 4 cannot hold, it returns to the owner
  before anything ships.
