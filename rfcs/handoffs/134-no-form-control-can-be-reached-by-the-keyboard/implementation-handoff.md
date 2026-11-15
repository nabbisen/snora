# Implementation handoff — RFC-134: keyboard-reachable form controls

**RFC.** `rfcs/accepted/134-no-form-control-can-be-reached-by-the-keyboard.md`. Read the finding,
R-1 to R-3 (R-2's key table especially) and the Rulings.
**Release target.** 0.65.0, as additions.
**Starts after** 0.63.0 has shipped. Unit 0 may run during 0.64. Unit 1 lands in 0.65.
**Where.** On `main`, by the one-`main` route. Probes are scratch, described and not committed.

## Unit 0 — feasibility, control by control (no product code committed)

For each of the **checkbox, toggle, radio group, select and slider**, prototyped on
`snora-interaction`'s `Control` (no wrapper over iced's):

1. **R-2's keys, working,** each measured:
   - Tab reaches it once;
   - the radio group is one stop;
   - its keys do what R-2's table says;
   - ← → follow the screen under RTL;
   - a disabled control is no stop.
2. **R-3:** a click and a key publish the same message.
3. **The floors:** the ring at 3:1, the 24 px target, the text at its palette's level and the
   non-text parts at 3:1.
4. **The select:** whether it can reuse the menu machinery (RFC-123 keys, RFC-124's anchored
   overlay, the cover) as it is, and what it needs that the menu does not have.
5. **Its size,** and **any public API** beyond the new constructors.
6. **Q-3's verdict per control: reasonable, or not and why.** "Not" needs a reason a reader can
   check, such as iced 0.14 offering no way to do X, and names the iced change that would end it.

## Unit 1 (0.65.0)

- Each control Unit 0 found reasonable:
  - on both paths (`snora::…` and `snora::design::…`);
  - with R-2's keys, R-3 and every floor;
  - each with a breakage row.
- **The sweep tests take them in:** `every_control.rs` and the drawn-contrast sweep (RFC-131).
- **CHANGELOG `[Unreleased]`, Added.**

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout`
   or `git restore`.
2. `render_semantics.rs` is unmodified.
3. Every `ci.yaml` command, run locally, before pushing to `main`.
4. **The book is the architect's:** the keyboard guide's *Form controls*, the scope page, and the
   guides.
