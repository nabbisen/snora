# Implementation handoff — RFC-129: a state never takes a label under its floor

**RFC.** `rfcs/accepted/129-a-buttons-hovered-and-pressed-label-falls-under-the-floor.md`. Read the
finding, R-1 to R-3 and the Rulings.
**Release target.** 0.62.0. **After RFC-128 Unit 1 lands,** because its sweep at rest is what this
extends. Then the audit follow-up's Unit 3.
**Branch.** `rfc129-state-contrast`, off `main`. Unit 0 is its own review. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## The rule

> **A hovered or pressed control never takes its label under its palette's text level**
> (`Palette::text_contrast`: `Aa` is 4.5, `Aaa` is 7), **on any neutral surface it is drawn on**
> (`background`, `surface`, `surface_raised`). **It still draws visibly differently from rest**
> (ΔE76 ≥ 3.0). Disabled is exempt.

## What exists

- `crates/snora-style/src/button.rs`: `primary`, `secondary`, `ghost` and `danger`, with their
  `Hovered` and `Pressed` arms (`:81`, `:105`, `:144`, `:172`). The helpers are `lighten` (`:37`,
  additive) and `darken` (`:107`, toward black). Each shift is fixed in direction on every preset.
- `crates/snora-widgets/src/design/chip.rs:224-225, :254-255`: the chips' hovered and pressed
  fills (R-3).
- RFC-128 Unit 0's state sweep (`.git-exclude/review-request/rfc128-unit0/evidence/
  unit0-button-variants.test.rs.txt`) and its tables. **Start from it.**
- RFC-128 Unit 1's sweep at rest, on `main` by the time this starts.

## Unit 0 — measure, and propose the rule (Q-1, Q-3)

1. **The full table on today's code:** every variant (the four buttons, the notice's ghost, the
   chips selected and unselected) × Active / Hovered / Pressed × the three neutral surfaces × the
   six presets, against the floor. The cells under it, listed.
2. **Q-1's rule, prototyped:** for a state, of the two shifts (today's, and the opposite), take
   today's when it keeps the label at or above the floor, else the other. Then measure:
   - whether every cell then passes;
   - **which cells change** (the migration guide's list). Q-2 says nothing that passes today
     moves: confirm it;
   - whether a simpler rule also holds, for example "shift away from the label's luminance". If
     one does, propose it, since one sentence a reader can predict is better than a search.
3. **Q-3, the hover cue:** for every changed cell, a frame comparison against rest shows the state
   still differs. Propose the measure (a minimum ΔE, or a fill contrast step between rest and the
   state), and say what today's states already give, so the floor is not invented.
4. **If no rule keeps every cell above its floor while staying visibly different** (for example, a
   fill with no room left on one side), **stop and report** the cells. Those go back to the owner.

## Unit 1 (Unit 0 accepted; Q-4 ruled 2026-11-14, revision 2)

Read the RFC's "What Unit 0 found", and **Q-4 revision 2 in full**: its points 1 to 5 and its
"For Unit 1".

**The token (Q-4):**
- **`TextContrast { Aa, Aaa }` in `snora-design`, and `Palette::text_contrast`:**
  - `light` and `dark` are `Aa`;
  - `light_aaa`, `dark_aaa` and both high-contrast presets are `Aaa`.
- **`Palette`'s rustdoc** gains "and the contrast level its text is held to".
- **`usages()` binds the field** as `text_contrast: _`, with a comment saying it is not a role.
  Do not add `..`.
- **The field's rustdoc** says it is the level snora holds every hovered and pressed label to. At
  rest the colours are the application's, and changing `accent` on `light_aaa` leaves `Aaa`.
- **No `Default`** for `Palette`, `Tokens` or `TextContrast`.
- **The suites keep their own table** of each preset's level:
  - they assert the token equals it;
  - they assert the resting pairs meet it;
  - **they never read the token to choose their floor.**
- **The style code alone reads the token.**

**The shift (Q-1, Q-3):**
- **One function in `snora-style`,** shared by every variant and the chips, not copied into each
  arm.
- **The candidates are a fixed, finite list:** today's shift, the opposite, then larger steps up
  to a fixed maximum. The result is measured against the worst of the three neutral surfaces.
- **It is total:** if no candidate clears the level, it takes the highest-contrast candidate that
  is still ΔE76 ≥ 3.0 from rest. Never the resting colour, and never a panic.
- **`secondary`'s border moves with its label:** measure it at 3:1 (non-text).
- **The high-contrast presets at 7:** measure their states. Any cell under 7 joins the migration
  list by the same rule. Say how many.
- **The cost of the search:** measure it per state per frame. If it is noticeable, say so before
  optimising.

**The breakage rows, in addition to those below:** Q-4's points 1, 2, 4 and 5, as the RFC lists
them. Point 3 has no row.

- **The rule in `snora-style`'s shift**, shared by every variant and the chips, not copied into
  each arm.
- **RFC-128's sweep extended to every state,** at the floor, every neutral surface. **It must fail
  on today's code** (the breakage row: the fixed shifts).
- **A frame test** that each variant's hovered and pressed states differ from rest by Q-3's measure,
  with a breakage row (a state that equals rest).
- **The existing state tests pass unmodified,** or say which assert today's exact colours and why
  they must change.
- **CHANGELOG `[Unreleased]`, Fixed:** a hovered or pressed label could fall under the preset's
  floor (RFC-129), with Unit 0's list of the changed cells.
- **CHANGELOG `[Unreleased]`, Added:** `Palette::text_contrast` and `TextContrast` (RFC-129 Q-4).
- **The book is the architect's:** `aaa-presets.md` (every state), `buttons.md`, `tokens.md` (the field).

## Acceptance criteria

1. Unit 0's tables and the proposed rule, reviewed.
2. Unit 1: every breakage row shown failing. `render_semantics.rs` is unmodified.
3. All of `ci.yaml`'s Rust quality gate commands (17, and Unit 2's claims step), the Docs job's
   commands, the feature matrix, both rendered jobs, and parity. Then the branch CI.
