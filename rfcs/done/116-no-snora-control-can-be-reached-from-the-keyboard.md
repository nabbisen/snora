# RFC-116 — No snora control can be reached from the keyboard

**Status.** Done — R-1 shipped v0.55.0, R-2 shipped v0.57.0. **Its long-term plan (L-1 to L-8, at the
end) stays live here.** Accepted 2026-11-04. Every open question ruled as suggested (see Rulings).
**R-1 shipped in v0.55.0** (merged 2026-11-07, `952e1a2`): every control snora
draws is `control`, and 1.0 gate 11 is met. **R-2, the `Shortcuts` router and the Escape field,
merged 2026-11-08 (`51331bc`), shipping in 0.57.0.** The
long-term plan is at the end. RFC-119 covers the keyboard beneath an open modal.
**Handoff.** [`handoffs/116-…`](../handoffs/116-no-snora-control-can-be-reached-from-the-keyboard/implementation-handoff.md).
**Revised** 2026-11-04, before acceptance, after the architect reviewed it against
the owner's fundamental philosophy (see "Review against snora's principles"). The
first draft (`b24a8f5`) treated real focus as a distant hope and gave keyboard users
four ad-hoc helpers. Two facts checked in review made the final design reachable
now.
**Raised.** 2026-11-04, architect, from the finding behind RFC-114's correction
(`.git-exclude/reviewed/iced-buttons-take-no-keyboard-input/`).
**Owner's direction, 2026-11-04:** *"Keyboard compatibility is required. Even if
immediate action is not possible, it must be managed as a long-term plan."*
**Release target.** Unit 0 first, as its own review (Q-5). **Priority.** WCAG 2.1.1
(Keyboard) is level A, the lowest level. snora's controls do not meet it.

---

## The finding

**iced 0.14's button takes no keyboard input.** `update` handles mouse and touch
only, and `operate` registers no focus target. A probe confirmed it: Tab, Enter and
Space never press a button, while a click does and typing reaches a text input.

**So every control snora draws is pointer and touch only:**
- sidebar items, tabs and breadcrumbs;
- toast and notice dismiss buttons, and the chip ×;
- menu items, and every `design::button` variant, including RFC-117's `link`.

**iced does not move focus on Tab either.** Nothing in `iced_winit`, `iced` or
`iced_runtime` binds Tab. An application must call
`iced::widget::operation::focus_next()` itself (`iced_runtime/src/widget/operation.rs:55`).
Today, even a text input is reachable only if the application wires that.

**Two facts change what is possible** (both checked in review):
1. **A focusable widget needs no iced feature.** The `Widget` trait
   (`iced_core/src/lib.rs:84`) and the `Focusable` operation
   (`iced_core/src/widget/operation/focusable.rs:7`) are public in `iced_core`,
   unconditionally. iced's `advanced` feature only *re-exports* them, and adds a few
   small methods. snora can build a focusable control on `iced_core` directly,
   **on by default, without enabling `advanced` for anyone.**
2. **The owner's 2026-08-20 ruling on `advanced` was conditional:** *"If snora does
   not require iced's `advanced` feature, snora should not introduce it as default"*
   (RFC-078). That condition is now in question, because keyboard compatibility is
   required (Q-1).

**Keys a focused widget uses never reach the application.** `iced::keyboard::listen()`
"listens to ignored keyboard events" (`iced_futures/src/keyboard.rs:6`). So
characters a text input consumes do not trigger shortcuts. Keys it ignores still
reach them, which matters for choosing bindings.

## Proposal

The design separates three things that a keyboard user needs:
- **reach** a control (focus);
- **operate** it (activate);
- **jump** straight to a destination (accelerators).

The first two are what WCAG 2.1.1 asks. The third is a convenience that stays even
when the first two exist, so it is never a stopgap to deprecate later.

**R-1 — a focusable control: reach and operate, on by default.**
- **One internal widget,** `snora_widgets::control` (name per Q-2). It wraps the
  content, and:
  - registers as `Focusable` in `operate`;
  - activates on Enter and Space when focused, emitting **exactly the message a click
    emits**;
  - takes clicks and touches as iced's button does.
- **Every snora control is built on it:** sidebar items, tabs, crumbs, toast and
  notice dismiss buttons, the chip ×, menu items, and the `design::button` variants.
  No public signature changes.
- **A visible focus ring, drawn by snora.** snora owns the focus state, so it can
  finally draw the ring that `accessibility.md` lists as impossible on iced-owned
  widgets. It uses `FocusTokens` on the design path, and a theme-derived colour on
  the unstyled path, with the 3:1 non-text floor asserted on drawn pixels.
- **Traversal:** `snora::keyboard::traverse(key, modifiers)` → `Option<Task>`, with
  Tab → `focus_next`, Shift+Tab → `focus_previous`. It is the one line an application
  adds to its keyboard subscription. Getting-started wires it, and so does every
  example.
- **It also owns sizing (RFC-115's amendment, 2026-11-05).** `control`'s layout
  enforces the target rule, target ≥ max(24 px, 1.5 × text, the minimum target
  token), and the icon rule (a pictogram is 1 em, the × is 1.125 em). It reads the
  text size through `text::Renderer::default_size()`. One widget owns reach, operate,
  focus and size.
- **Built on `iced_core`, not on iced's `advanced` feature,** so no consumer's feature
  set changes (RFC-112's lesson). `iced_core` is already in every snora graph; it
  becomes a direct dependency at the same `0.14` requirement, so it unifies with
  iced's own.

**R-2 — accelerators: jump, through one router.**
- **`snora::keyboard::Shortcuts`**, a builder that takes the controls as the
  application already passes them to their widgets:
  - `.side_bar(&side_bar)`;
  - `.tabs(&tab_bar, &on_action)`;
  - `.crumbs(&crumbs, &on_action)`;
  - `.escape(Escape { modal, menu })`.
- `.route(key, modifiers) -> Option<Message>` answers in **one place**, with one
  precedence order, so two routes can never both fire.
- **The rule a reader can predict:** *a shortcut emits exactly the message a click on
  that control emits.*
- **Platform-conventional bindings** (Q-3), chosen at compile time with
  `cfg!(target_os = "macos")`:

  | Destination | Windows and Linux | macOS |
  |---|---|---|
  | Sidebar item *n* | `Ctrl` + `1`…`9` | `Cmd` + `1`…`9` |
  | Next / previous tab | `Ctrl+Tab` / `Ctrl+Shift+Tab` | the same |
  | Breadcrumb parent | `Alt+↑` | `Cmd+↑` |
  | Close modal, then menu | `Escape` | `Escape` |

- **Escape is now named fields, not positional arguments.** `dismiss_on_escape` takes
  two `bool`s and two `Option`s in a row (`keyboard.rs:61`). That is easy to call in
  the wrong order without any compiler error. The router's `Escape { modal, menu }`
  says which is which. **`dismiss_on_escape` is deprecated**, as superseded, and keeps
  working.
- **Escape does not dismiss toasts** (revised in review). A stray Escape would clear
  an unread persistent error, which RFC-113 exists to prevent. A keyboard user
  reaches the toast's close button through R-1 instead.

**R-3 — the plan, where an upgrade will read it.** A new 1.0 gate: *"Every snora
control is reachable, operable and visibly focused from the keyboard (WCAG 2.1.1,
2.4.7)."* It closes when R-1 ships and its probe passes. Gate 1's row, which an iced
upgrade reads, gains two instructions:
- **re-run the keyboard probe;**
- **if iced's own button has become focusable, compare it with snora's `control`.**
  If upstream is equivalent, `control` is superseded: deprecate it and adopt
  upstream.

**R-4 — the book.**
- `accessibility.md` states WCAG 2.1.1's status at each stage.
- A keyboard map page lists every binding.
- `cycle_zones`'s doc says which zones contain something focusable.
- The "no focus ring on snora's own widgets" limitation is withdrawn when R-1 ships.

## Open questions

**Q-1 — the owner's `advanced` ruling.** R-1 needs no iced feature. It does depend on
the widget API that `advanced` re-exports, which is the API iced changes most between
versions. **Suggest:** the 2026-08-20 ruling is met: snora introduces no iced feature
at all. The real cost is maintenance, porting `control` at each iced upgrade, and Unit
0 measures it. **The owner decides** whether that cost is acceptable for a level-A
criterion.

**Q-2 — the name.** The widget is internal. Only `traverse`, `Shortcuts` and `Escape`
are public. **Suggest** keeping `control` private until an adopter needs it. A public
"button that is really focusable" next to iced's `button` would leave every reader
wondering which one to use.

**Q-3 — bindings.** **Suggest the table.** Where a platform has a convention, follow
it; where it has none, use the common one. `Cmd+↑` is "enclosing folder" in macOS
Finder, and `Alt+↑` is "up one level" in Windows Explorer. Collisions with an
application's own shortcuts are resolved by not adding that route to the router: each
route is opt-in per control.

**Q-4 — is the router opt-in?** It must be, since snora captures no keys. But under
R-1 the router is a convenience, not the accessibility route. **Reach and operate
work without it,** once the application adds `traverse`, which every example and
getting-started does. So ABDD's "by default" now holds for the part WCAG requires.

**Q-5 — release and order.**
- **Unit 0, a prototype:** one focusable `control` replacing the sidebar item. Tab
  reaches it, Enter presses it, the ring draws, and the existing sidebar tests pass.
  Measure the size and compile cost, and the porting surface (which `iced_core` items
  it touches). It is its own review.
- **R-1 for every control, R-2, R-3 and R-4:** the release after Unit 0 is accepted.
- **R-3's gate row and R-4's status statement go in now**, as docs, because they are
  the plan the owner asked for.

## Review against snora's principles

- **Finally clean, robust, sophisticated (owner, 2026-11-04):**
  - reach, operate and jump are separated, so nothing built now becomes debt later;
  - one router replaces four helpers of three different shapes;
  - real focus is the plan, not a hope.
- **Nobody can misunderstand it (owner, 2026-11-04):**
  - one rule, "a key emits exactly what a click emits";
  - named fields replace swappable positional arguments;
  - each platform's own conventions;
  - no second public button type to choose between.
- **Safe:**
  - no Escape on toasts, so an unread error is never cleared by accident;
  - shortcuts never see keys a focused input used.
- **ABDD and keyboard (owner):** reach and operate on by default, needing only one
  traversal line, which every example carries.
- **Mark deprecated when superseded:** `dismiss_on_escape`, and later `control` if
  upstream catches up.
- **Verify claims:** every iced fact above is cited to its source line, and the probe
  is reused.
- **Correspondence cost:** no letter. The book carries the status.

## What this is not

- Not enabling iced's `advanced` feature.
- Not snora capturing keys.
- Not focus trapping inside modals. That needs to *query* focus, and stays RFC-060's
  separate question.

## Acceptance criteria

1. **Unit 0:** the prototype passes the button probe's three cases inverted: Tab
   reaches it, and Enter and Space press it, each assertion shown failing on today's
   sidebar. The ring is drawn at ≥ 3:1. The costs are measured.
2. **R-1 and R-2:** every control passes the same probe. The router's precedence is
   tested. `dismiss_on_escape` is deprecated, and its callers have moved.
3. **R-3 and R-4:** the gate row, the gate-1 instructions and the keyboard map,
   traced to the probe.

---

## Rulings, 2026-11-04

All suggestions accepted, as revised in `387d29f`.

- **Q-1:** snora introduces no iced feature. `control` is built on `iced_core`, and
  Unit 0 measures the porting cost before the rest proceeds.
- **Q-2:** `control` stays private. The public API is `traverse`, `Shortcuts` and
  `Escape`.
- **Q-3:** platform-conventional bindings, as tabled.
- **Q-4:** the router is opt-in per route. Reach and operate need only `traverse`.
- **Q-5:** Unit 0 first, as its own review. R-3's gate row and R-4's status statement
  go in now, written by the architect.

## Ruling, 2026-11-07: R-1 proceeds

Unit 0 (`.git-exclude/reviewed/116-focusable-control/`) measured the cost Q-1 was to
be ruled on:
- engine binary byte-identical, widgets +9,728 bytes;
- no detectable compile-time change;
- porting breaks loudly, in one file;
- the focus ring at ≥ 4.65:1.

**The owner ruled that R-1 proceeds.**

**One placement question, raised while writing R-1's handoff,** waits for its own
ruling. The toast's close button lives in the `snora` engine crate
(`toast.rs:314`), whose `snora-widgets` dependency is optional, so that engine-only
builds stay small. `control`, in `snora-widgets`, cannot sit under it. R-1 Unit 1,
the prerequisites, does not depend on the answer. Units 2 and 3 do.

## Ruling, 2026-11-07: `control` lives in a new crate, `snora-control`

**The owner ruled for the architect's recommendation.** A new crate, `snora-control`,
holds `control`. Both `snora` (the engine, for the toast's close button) and
`snora-widgets` depend on it, non-optionally.
- **One implementation.** No twin.
- **Layering kept.** `snora-core` stays iced-free, and the engine's dependency on
  `snora-widgets` stays optional.
- **The cost:** a sixth published crate. Its crate doc says it exists to serve
  snora's own crates, and that it is versioned in lockstep with them.

**Rejected:**
- making `snora-widgets` mandatory for the engine, which would drop small engine-only
  builds;
- placing `control` in `snora-core`, which would break its iced-free design.

## Rulings, 2026-11-07: the branch push, and the new crate's first publish

**The branch push (owner).** The rebased `rfc116-unit0` is force-pushed with a lease
(`--force-with-lease=rfc116-unit0:d6d6a00`). The dev team's local backup branch is kept
until 0.55.0 ships. The auto-mode permission system denied the push to the dev team and
to the architect alike, so the owner runs it.

**The first publish of `snora-control` (owner, confirmed).** Trusted Publishing is
configured per crate, and cannot be configured for a crate that does not yet exist on
crates.io. So **the first version is published from the local machine:**
1. At the 0.55.0 cut, from a clean worktree at the release commit, before the tag is
   pushed: `cargo publish -p snora-control`.
2. The owner adds the crate's trusted publisher on crates.io (`release.yaml`, this
   repository).
3. The tag is pushed, and `release.yaml` publishes the other five. It **skips any crate
   whose version is already on crates.io** (the Units 2–3 handoff, Unit 2b), so the
   pre-published crate does not fail the run.

The skip is general, not a one-off exclusion: it also makes a re-run after a partly
failed publish safe. The procedure is recorded in `release-process.md` for any future
new crate.

## Long-term plan: the keyboard gaps that remain after R-1 (2026-11-07)

Recorded under the owner's ruling of 2026-11-04: *"Even if immediate action is not
possible, it must be managed as a long-term plan."* Each item has a trigger. When the
trigger fires, the item becomes an RFC or a unit, and nobody waits for a re-read.

| # | Gap | Who meets it | Trigger |
|---|---|---|---|
| L-1 | ~~Focus after a control removes itself~~ **Met by RFC-120** (shipped v0.56.0). Dismissing a notice, closing a toast, removing a chip, or pressing an ancestor crumb, which becomes the current page: focus goes nowhere, and the next Tab starts from the top of the window | everyone who dismisses from the keyboard | **fired 2026-11-07** (RFC-119 Unit 1 accepted). Next: an RFC, after 0.55.0. RFC-119's position-and-count memory is the likely mechanism |
| L-2 | ~~Focus returns to the opener when a modal closes~~ **Met by RFC-119** (merged 2026-11-07): focus returns, unless the page's number of controls changed or the application focused something itself | — | — |
| L-3 | ~~No hold for a keyboard-focused toast~~ **Met by RFC-120** (`on_toast_attention`, `Holds::attend`). Focusing the toast's × does not hold it, as a resting pointer does (RFC-113) | keyboard users reading a toast | with L-1, since both concern the toast's control |
| L-4 | ~~No public control for an application's own content~~ **Met by RFC-122** (merged 2026-11-09, ships 0.58.0: `snora::control`, `snora::design::control`). An application's own iced `button` (an icon button, a card as a target) is pointer and touch only; `control` serves snora's crates only | applications that build their own controls | the first adopter that asks, or 1.0's API freeze review, whichever comes first |
| L-5 | **A focused item on the `Tooltip` rail shows no name:** iced opens tooltips only under the pointer | keyboard users of `SideBarLabels::Tooltip` | an iced release with focus-driven tooltips; until then, the book says so (`reference/widgets.md`), and `Beneath` is the default; the next rustdoc pass on `SideBarLabels::Tooltip` says it too |
| L-6 | ~~Native Wayland~~ **Met 2026-11-08** (housekeeping-0-56: niri, `DISPLAY` unset; the same frames as X11). The real-window Tab check ran under X11 only | Wayland users | the next real-window check, or any Wayland report |
| L-7 | **A removal elsewhere moves focus, in an application's own list.** iced 0.14 keeps widget state by tree position, so removing row 0 while row 1 is focused hands focus to row 2 (RFC-120 Unit 0, 1b). snora knows its own toasts by id and fixes it there (RFC-120); it has no identity for an application's widgets | keyboard users of lists that change under them | iced gains keyed widget state, or an adopter reports it; until then the book tells applications to set focus explicitly in such lists |

**L-5 is *future wanted* (owner, 2026-11-08).** It is not scheduled, and stays on this
list. snora could draw a name on focus itself. But the labelled rail is the default, and
the `Tooltip` form's cost is stated in the book. It is revisited with L-4, or when iced's
tooltips can open on focus.

## Proposed amendment to R-2, for 0.57 (2026-11-08): awaiting the owner's rulings

**Owner, 2026-11-08:** R-2 moves to 0.57, *"carefully and with consideration."* R-2 was
accepted before RFC-119. Read again after it, two of its parts no longer hold, and its
placement needs deciding afresh.

### What no longer holds

**F-1. As specified, shortcuts would fire beneath an overlay.**
- R-2's `Shortcuts::route(key, modifiers)` runs in the application's `update`, fed by
  `iced::keyboard::listen()`.
- That subscription receives every key nothing captured. With a dialog open, `Ctrl+1`
  is captured by nothing (RFC-119's `Inert` only stops it reaching the page's widgets),
  so the router would switch the sidebar's view beneath the dialog.
- That breaks the rule 0.55 shipped: *the keyboard does only what the pointer can.*

**F-2. "`dismiss_on_escape` is deprecated" now conflicts with RFC-119.**
- RFC-119 answers Escape in the engine for every overlay that closes on a click outside.
- `dismiss_on_escape` was kept for the one case left: an overlay that should close on
  Escape but **not** on a click outside, such as a destructive confirmation.
- R-2's `.escape(Escape { modal, menu })` was to replace it. Named fields are right:
  `dismiss_on_escape`'s two `bool`s and two `Option`s in a row are easy to call in the
  wrong order, with no compiler error.
- But an Escape answered inside the page would be silenced by `Inert` while the modal
  it should close is open. **That Escape has to live with the overlay, not the page.**

### Where shortcuts live: three options

| | The application's `update` (R-2 as accepted) | Each snora widget answers its own keys | **One router, placed by the engine inside the page layer** |
|---|---|---|---|
| Silent under an overlay | only if the application passes the overlay state, and it can forget | yes, through `Inert` | **yes, through `Inert`**: nothing to remember |
| One precedence order, so two routes never both fire (R-2's own guarantee) | yes | no: two tab bars would each answer Ctrl+Tab | **yes** |
| Takes no key the application did not ask for | yes | only if every widget is opt-in | **yes: only the routes the builder names** |
| No double message | the application's own concern | yes, captured | **yes, captured** |

**Suggest the third.** The application builds `Shortcuts` in `view`, from the same
side bar, tab bar and crumbs it hands its widgets, and passes it with
`AppLayout::shortcuts(…)`. The engine places it inside the page's `Inert`. Every
property in the table then comes from mechanisms 0.55 already has and tests.

### The Escape that does not close on a click outside

**Suggest an `AppLayout` field, beside `on_close_modals` and `on_close_menus`.** For
example `.escape(Escape { modal, menu })`, with named fields and no positional
arguments. The engine places it on the overlay layers, as `OnEscape` already is, so it
works while the overlay is open.
- `dismiss_on_escape` is then **superseded**, and is deprecated in the release that
  ships this, per the owner's rule;
- the router (above) has **no** Escape route, so Escape has one home, the overlay;
- the exact name, and how `Escape` combines with a close message that is also set,
  are Q-R2-2.

### Questions for the owner

- **Q-R2-1 — placement.** Suggest the engine-placed router. The alternatives are the
  application-side router of R-2 as accepted, or keys owned by each widget.
- **Q-R2-2 — Escape without a click-outside close.** Suggest an `AppLayout` field with
  named fields, and `dismiss_on_escape` deprecated when it ships. The alternative is
  to keep `dismiss_on_escape` undeprecated, and live with its positional arguments.
- **Q-R2-3 — bindings.** Suggest R-2's table unchanged, with each route opt-in through
  the builder:
  - `Ctrl`/`Cmd` + `1`…`9` for sidebar items;
  - `Ctrl+Tab` for tabs;
  - `Alt+↑`/`Cmd+↑` for the breadcrumb parent.

  A text editor that needs `Ctrl+Tab` for itself then leaves that route out.

### Additions, 2026-11-08, from re-checking against 0.56

- **B1 — a shortcut does exactly what a click on that control does.** That means the
  same message, focus taken away from wherever it was, and the Tab starting point set on
  that control (RFC-121); a disabled control does nothing. Without it, a shortcut that
  swaps the view would leave keyboard focus at a tree position the new view fills with
  another control, which would inherit it (RFC-120's finding).
- **B2 — a shortcut works unless the focused control uses that key itself.** The router
  answers only keys nothing else took. The keyboard map page lists which keys snora's own
  controls use.
- **B3 — the digit binding.** Browsers and most tabbed applications use Ctrl/Cmd + 1…9 for
  *tabs*. R-2 gave it to *sidebar items*. Unit 0 surveys the conventions, and the binding
  table is ruled on them.
- **Every binding has a modifier** (Ctrl, Alt or Cmd). WCAG 2.1.4 governs single-character
  shortcuts, so the router stays outside it by design.

**Owner, 2026-11-08 (Q-3):** R-2 stays in 0.57, with B1 to B3. Unit 0 runs first
(`handoffs/116-…/implementation-handoff-r2-unit0.md`). If it shows the router does not fit,
the owner decides at its review whether it moves to 0.58.

**Before ruling,** a Unit 0 for 0.57 measures two things: that a router inside `Inert`
is silent under each overlay of RFC-119's table, and that it captures what it answers.
This amendment is reviewed against the principles as part of that unit's review. It is
not ruled now.

## Ruling, 2026-11-08: `snora-control` is renamed `snora-interaction` (0.57)

**Owner, agreed.** Beside `snora-widgets`, "control" reads as another word for widget.
The crate also now holds far more than the control: everything in it is about *how the
reader operates snora*, by pointer, keyboard and focus.
- In 0.57.0 the crate becomes `snora-interaction`. The type `Control` keeps its name.
- **The last release of `snora-control`, 0.57.0, declares the crate deprecated** (Q-1):
  no items, only a notice naming `snora-interaction`.
- 0.55.0 and 0.56.0 stay published and are never yanked, so existing lockfiles keep
  working.
- The notice crate leaves the workspace in 0.58.
- The first publish of `snora-interaction` follows `release-process.md`'s first-version
  procedure.

Handoff: `handoffs/housekeeping-0-57/implementation-handoff.md`.

## Rulings, 2026-11-08: R-2, after Unit 0 (owner, confirmed after a check against the principles)

Unit 0 (`.git-exclude/reviewed/116-r2-unit0/`) measured a prototype router on both
renderers. Nothing proposed was infeasible.
- It is silent under every overlay that blocks the pointer, and captures what it answers.
- A focused control that uses the key wins.
- A shortcut does what a click does. Without that, Enter after a view swap pressed a
  control nobody chose.

**Placement (Q-R2-1):** one router, built by the application in `view`, passed with
`AppLayout::shortcuts(…)`, and placed by the engine around the skeleton, inside its
`Inert`. It sees a key only after its content, and answers only if nothing captured it.

**The rules a reader can predict:**
- **A shortcut does exactly what a click on that control does.** That covers the
  message, focus taken away, and the Tab starting point (RFC-121); a disabled or absent
  target does nothing.
- **A shortcut works unless the focused control uses that key, and a focused text field
  is taken to use every arrow key, with any modifier.** The text-field clause was added
  in review: on macOS, Cmd+↑ in a field means "start of text".
- **Every binding has a modifier,** so WCAG 2.1.4 does not apply.
- **The digit routes match the physical digit row; named keys match the logical key.**
  This was added in review, because on AZERTY the "1" key types "&".

**R2-1, the digits.** `Ctrl/Cmd + 1…9` go to **exactly one destination, chosen by the
application in the builder** (tabs or sidebar items), and never change at run time. `9`
is the last. The book recommends tabs when the application has a tab bar (the browsers'
convention), and the sidebar when it has none.

**The bindings:**

| Destination | Windows / Linux | macOS |
|---|---|---|
| Next / previous tab | `Ctrl+Tab` / `Ctrl+Shift+Tab`, and `Ctrl+PgDn` / `Ctrl+PgUp` | `Control+Tab` / `Control+Shift+Tab`, and `Shift+Cmd+]` / `[` |
| Destination *n* (tabs or sidebar, as chosen) | `Ctrl+1…9` | `Cmd+1…9` |
| Breadcrumb parent | `Alt+↑` | `Cmd+↑` |

**R2-2, Escape.** `AppLayout` gains an escape field with named fields. Escape publishes
the overlay's escape message if one is set, and otherwise its click-outside message.
The click-outside message still answers clicks. **`dismiss_on_escape` is deprecated in
0.57.0, and removed in 0.60.0.**

**R2-3, macOS.** The macOS table ships. The book states that it follows Apple's
conventions but has not been checked on a Mac. The text-field rule keeps Cmd+↑ from ever
firing in a field. **L-8** is added to the long-term plan.

**Found in review:** `traverse` claimed Tab with any modifier, so Ctrl+Tab moved focus.
It now claims plain Tab and Shift+Tab only.

| # | Gap | Who meets it | Trigger |
|---|---|---|---|
| L-8 | **The macOS bindings have not been checked on a Mac** (how `iced_winit` reports Cmd+↑, and whether a field keeps it) | Mac users | the first check on macOS, by an adopter: the owner has no Mac (2026-11-09) |
| L-9 | ~~Arrow keys do nothing inside snora's menus~~ **Met by RFC-123** (shipped v0.60.0: ↓ ↑ Home End, wrapping; opening a menu moves no focus, by ruling). Originally: Since 0.55, Tab, Enter and Escape work in an open menu. Native menus also answer ↓/↑ (and Home/End), and opening one from the keyboard moves focus to its first item; snora's do neither (found 2026-11-09, scoping 0.59) | keyboard users of `app_header`'s and `render_menu`'s menus | **RFC-123, in 0.59's scope (owner, 2026-11-09)** |
| L-10 | **Type-ahead in menus** (a letter jumps to the item that starts with it). It needs a timing rule and a matching rule across scripts (RFC-123 Q-4) | keyboard users of long menus | *future wanted*: an adopter asks |
| L-11 | **Arrow keys in the tab bar,** the "roving" tab pattern, where Tab stops once at the bar. It changes Tab everywhere (RFC-123 Q-5) | keyboard users of tab bars | *future wanted*: an adopter asks |
