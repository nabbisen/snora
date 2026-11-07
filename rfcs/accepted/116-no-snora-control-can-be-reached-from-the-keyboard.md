# RFC-116 — No snora control can be reached from the keyboard

**Status.** Accepted 2026-11-04. Every open question ruled as suggested (see Rulings).
**R-1 shipped in v0.55.0** (merged 2026-11-07, `952e1a2`): every control snora
draws is `control`, and 1.0 gate 11 is met. **R-2, the `Shortcuts` router, is open.** The
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
| L-1 | **Focus after a control removes itself.** Dismissing a notice, closing a toast, removing a chip, or pressing an ancestor crumb, which becomes the current page: focus goes nowhere, and the next Tab starts from the top of the window | everyone who dismisses from the keyboard | **fired 2026-11-07** (RFC-119 Unit 1 accepted). Next: an RFC, after 0.55.0. RFC-119's position-and-count memory is the likely mechanism |
| L-2 | ~~Focus returns to the opener when a modal closes~~ **Met by RFC-119** (merged 2026-11-07): focus returns, unless the page's number of controls changed or the application focused something itself | — | — |
| L-3 | **No hold for a keyboard-focused toast.** Focusing the toast's × does not hold it, as a resting pointer does (RFC-113) | keyboard users reading a toast | with L-1, since both concern the toast's control |
| L-4 | **No public control for an application's own content.** An application's own iced `button` (an icon button, a card as a target) is pointer and touch only; `control` serves snora's crates only | applications that build their own controls | the first adopter that asks, or 1.0's API freeze review, whichever comes first |
| L-5 | **A focused item on the `Tooltip` rail shows no name:** iced opens tooltips only under the pointer | keyboard users of `SideBarLabels::Tooltip` | an iced release with focus-driven tooltips; until then, the book says so (`reference/widgets.md`), and `Beneath` is the default; the next rustdoc pass on `SideBarLabels::Tooltip` says it too |
| L-6 | **Native Wayland.** The real-window Tab check ran under X11 only | Wayland users | the next real-window check, or any Wayland report |
