# Implementation handoff — RFC-126: F6 moves focus between zones, from where focus is

**RFC.** `rfcs/accepted/126-f6-zone-navigation-cannot-move-focus.md`. Read the finding, R-1 to
R-5 and the Rulings.
**Release target.** 0.61.0, **after RFC-127 lands**: Unit 0's first measure uses its ids.
**Branch.** `rfc126-zones`, off `main` after RFC-127. Unit 0 is its own review. Land as `main`
requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## The rules

1. **F6 and Shift+F6 move focus to the first control of the next and the previous zone,**
   counted from the zone that holds focus **when the key is pressed.** The order is Header →
   SideBar → Body → Footer, wrapping. It is logical, and does not mirror under RTL.
2. **A zone with no control that can take focus is skipped,** whether it is absent or holds
   none.
3. **With focus in no zone** (nothing focused, or a toast): F6 goes to the first zone with a
   control, and Shift+F6 to the last.
4. **An open menu's dropdown belongs to its trigger's zone.**
5. **While the page is covered, F6 does nothing:** under a dialog or a sheet, a menu with its
   backdrop, or an open `app_header` dropdown with a close message.
6. **F6 never presses anything.** Held, it keeps moving, one zone per repeat, as Tab does
   (corrected at Unit 0's review).
7. **The application routes the key, as it routes Tab:**

```rust,ignore
if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
    return snora::keyboard::focus_zone(cycle);
}
```

## What exists to build on

- **The four regions are marked:** `render.rs`'s `build_skeleton` (`:419`) wraps each slot in a
  container with `identifiers::HEADER_REGION`, `SIDEBAR_REGION`, `BODY_REGION` or
  `FOOTER_REGION` (`:435`–`:465`), which reach an operation through `container(id, …)`. **Check
  `snora::design::render`** (`crates/snora/src/design/render.rs:115`): F6 must work on both
  paths.
- **`Inert` hides a covered layer's focus targets from every operation** (RFC-119). So a
  window-wide operation run under a cover sees no zone targets, and rule 5 may need no code of
  its own. **Measure it; do not assume it.**
- **A task's operation runs through `UserInterface::operate`,** which visits the root and then
  the overlays (RFC-123's units). An open dropdown's items are therefore visited **outside**
  the header's container. That is rule 4's problem: propose how the operation knows they are
  the header's. The anchor already marks itself for `Inert` and `MenuKeys`.
- **`traverse`** (`crates/snora/src/keyboard.rs`) is the pattern for a key that returns a
  `Task`.

## Unit 0 — measure the finding, then prototype

**On `main`, before any change:**

| Measure | Expected (the RFC's finding) |
|---|---|
| following the accessibility guide and `cycle_zones`'s rustdoc as written, F6 from the body | no focus moves |
| with RFC-127's `.id()` on a `snora::control` of the application's own in the sidebar, `focus(id)` for "the sidebar" | it works for that control, and **cannot** reach a prefab sidebar item, a tab or the header's trigger |
| `next_zone` after a Tab from the body into the sidebar | counts from the body, the stale variable |
| `next_zone` with a menu with its backdrop open | answers as if no menu were open |

Then **prototype `focus_zone`** and measure rules 1 to 6 on both renderers:
- all four zones, wrapping, both directions, with the ring read from **pixels**;
- a Tab into another zone, then F6, which counts from where focus now is;
- a footer of plain text, skipped;
- a toast focused, then F6;
- nothing focused, then F6 and Shift+F6;
- each cover in rule 5, where nothing moves;
- a dropdown item focused (a menu without a close message), then F6, which goes to the sidebar;
- RTL, with the same order;
- a held F6;
- the design path.

**Propose rule 4's mechanism,** and say whether rule 5 needed code.

## Unit 1

- **`snora::keyboard::focus_zone(cycle: Cycle) -> Task<Message>`,** with rustdoc showing the
  two-line pattern and the rules.
- **Deprecate,** with notes naming `focus_zone` and **"removed in 0.64.0"**:
  `snora_core::focus::next_zone`, `FocusZone` and `ZonePresence`, and their re-exports (check
  `snora::focus`, and both crates' `lib.rs`). Keep `Cycle` and `cycle_zones`. `cycle_zones`'s
  rustdoc example becomes the two-line pattern.
- **Tests:** a test per rule and a breakage row each, checked first that it can fail (named `cp`
  backups, `sha256sum -c`; never `git checkout` or `restore`). The suites already in the tree
  pass **unmodified**: `render_semantics.rs`, `modal_keyboard.rs`, `menu_arrows.rs`,
  `covered_pointer.rs` and `focus_follows_removal.rs`. The deprecated items' own unit tests may
  carry `#[expect(deprecated)]`, as in 0.57.
- **`every_control.rs`:** a subject per zone, each reached by F6 with its ring.
- **CHANGELOG `[Unreleased]`:**
  - **Fixed:** F6 zone navigation moved no focus, counted from a stale zone, ignored a
    covering menu, and stopped on empty zones (RFC-126);
  - **Added:** `snora::keyboard::focus_zone`;
  - **Deprecated:** the three items, each with its replacement and the removal version.
- **The book is the architect's** (R-5).

## Acceptance criteria

1. Unit 0's tables, reviewed.
2. Unit 1: every breakage row shown failing.
3. All of `ci.yaml`'s Rust quality gate commands (16), the Docs job's commands, the feature
   matrix, both rendered jobs, and parity. Then the branch CI.
