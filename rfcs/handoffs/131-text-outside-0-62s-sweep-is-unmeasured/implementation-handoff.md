# Implementation handoff — RFC-131: every text snora draws holds its palette's level

**RFC.** `rfcs/accepted/131-text-outside-0-62s-sweep-is-unmeasured.md`. Read the finding, R-1 to
R-3, the Rulings and the principles review's additions.
**Release target.** 0.63.0.
**Where.** On `main`, by the one-`main` route. Probes are scratch tests, described and not
committed.

## The rule

> **Every text a snora widget draws on the design path holds its palette's `text_contrast` level**
> (4.5 or 7, **whatever its size**), at rest and in every state it has, on every neutral surface it
> can sit on, in every preset. Disabled is exempt. A colour that holds today does not move.

## Order

1. **Unit 0 is one sweep with the non-text unit** (`rfcs/handoffs/non-text-contrast-0-63/`).
   - It uses the same frames: every text run at its level, every non-text element at 3:1.
   - **One review request:** `.git-exclude/review-request/drawn-contrast-unit0/`.
2. **It starts after:**
   - the audit follow-up unit (`../../../.git-exclude/reviewed/audit-0-63-b/review-result.md`, "Order");
   - RFC-132's Unit 0.

## Unit 0 — measure (no product code committed)

1. **The inventory, from frames.**
   - For each snora widget on the design path, in both directions, list every text run the
     renderer receives: the string, its colour, the widget that drew it, and what it is drawn on.
   - Find the place in `iced_test` or the renderer path where text runs can be enumerated, and say
     which you used. **If no such place exists without public API, stop and report.**
   - **A source search does not count.** That is what missed the tab.
2. **Every text measured:**
   - in each state it has (rest, hovered, pressed, active, focused);
   - on each neutral surface it can sit on;
   - in every preset, against its palette's level.
   - **List every cell under its level.** The inactive tab on `light_aaa` (6.58) is expected, and
     is the premise.
3. **The completeness check, prototyped:** a text run that appears in a frame and is not in the
   sweep's table fails a test. Plant one (a new label in a scratch widget) and show it fails.
4. **Q-1's route:**
   - how each failing widget's style learns its palette's level;
   - which style functions would move to `snora-style` (as the chips did), or how the design path
     passes the level in;
   - **any public API it needs.** The owner rules on that before Unit 1.

## Unit 1

- **The sweep and the completeness check** in `crates/snora/tests/`.
  - The sweep fails on today's tab (the breakage row).
  - The check fails on a planted unmeasured text run.
- **The fixes, by RFC-129's rule** (`snora_style::state`, which stays `pub(crate)`), each shown to
  hold the level.
- **Nothing that passes moves:** a table of today's colours, as RFC-129's Q-2 test is.
- **CHANGELOG `[Unreleased]`, Fixed,** with every changed cell.

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout`
   or `git restore`.
2. `render_semantics.rs` is unmodified.
3. Every `ci.yaml` command, run locally, before pushing to `main`.
