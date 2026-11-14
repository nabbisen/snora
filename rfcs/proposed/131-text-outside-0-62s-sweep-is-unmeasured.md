# RFC-131 — Text snora draws outside 0.62's sweep is unmeasured, and an inactive tab reads 6.58:1 on `light_aaa`

**Status.** Proposed 2026-11-15, for the owner.
**Raised.** 2026-11-15, architect, from the 0.63 audit's request A (its "7:1 boundary" doubtful row),
verified by the architect against the code.
**Release target.** 0.63.0.
**Priority.** A broken promise on a preset whose purpose is the promise. It is also a claim in
0.62.0's CHANGELOG, now corrected, and adopters cite contrast.

---

## The finding

An inactive tab's label is drawn as `mix(background.base.text, background.base.color, 0.3)`
(`crates/snora-widgets/src/tab.rs`, `tab_button_style`). That is 70% text and 30% page, a colour
**no palette declares**. Against the page:

| Preset | Inactive tab label | Level |
|---|---|---|
| `light` | 6.58:1 | 4.5, holds |
| **`light_aaa`** | **6.58:1** | **7, under** |
| `dark`, `dark_aaa` | 8.59:1 | holds |
| `high_contrast_light` | 8.52:1 | holds |
| `high_contrast_dark` | 9.96:1 | holds |

(Computed from the presets; `light_aaa` keeps Light's background and `text_primary`. The audit's
hand figure agrees. Unit 0 measures it as drawn.)

**The cause is RFC-128's R-2, done in part:**
- R-2 promised "a test that draws every text a design widget draws with a colour of its own".
- Its inventory was a **source search for non-text colours**, run by the architect. A mix of two
  text-side colours passes such a search, so the tab was missed.
- Its sweep (`crates/snora/tests/text_floor.rs`) draws buttons, chips, links and a notice's
  action. **Tab labels, crumbs, sidebar names, menu items, tooltips, a notice's title and body,
  and toasts are not in it.**
- 0.62.0's CHANGELOG lead said "every text snora draws holds its preset's contrast level". It is
  corrected (2026-11-15) to what the sweep covers.

## Proposal

**R-1. The sweep R-2 promised, completed, from an inventory made by drawing rather than by
searching.**
- **Every text a snora widget draws on the design path** (`snora::design::render` with
  `design::theme(&tokens)`):
  - tab labels, active and inactive;
  - crumbs;
  - sidebar names, active and not;
  - menu items;
  - the tooltip;
  - a notice's title and body;
  - toasts, per intent;
  - the header's title;
  - the footer;
  - and anything else Unit 0 finds in a frame.
- **Each is measured as drawn,** at rest and in each state it has (hovered, pressed, active), on
  each neutral surface it can sit on, in every preset, at its palette's `text_contrast` level.
- **The inventory is made from frames:** for each widget, every text run that appears is listed
  with the widget that drew it. A source search does not count. That is the method that missed
  the tab.
- **The unstyled path is out of scope,** as for RFC-129: its colours are the application's iced
  theme.

**R-2. Every text under its level is fixed by RFC-129's rule:**
- keep today's colour where it holds the level;
- otherwise take the nearest colour that does;
- keep any state cue visibly different (ΔE76 ≥ 3.0).

**Nothing that passes today moves.** On Light, the inactive tab keeps its 6.58:1, which clears
4.5.

**R-3. The book:** `aaa-presets.md`'s *In every state* lists what the sweep covers, now every text
snora draws on the design path, and the CHANGELOG says so.

## Open questions, with suggested rulings

**Q-1 — how does a widget styled from an iced `Theme` know its palette's level?** The tab's style
takes `&Theme`, and `text_contrast` lives on the `Palette`.

**Suggest: Unit 0 finds the route and proposes it, with these constraints:**
- **One rule:** RFC-129's `snora_style::state`, which stays `pub(crate)`.
  - If a widget in `snora-widgets` needs the rule, its style function moves to `snora-style`, as
    the chips' did (RFC-129 Q-5).
  - Or the design path passes the level in. Unit 0 says which, and the owner rules if either adds
    public API.
- **No level inferred from colours** (RFC-129 Q-4's rejected B).

**Q-2 — the inactive tab's muted cue.** It distinguishes inactive from active a little; the
underline carries the active state at 3:1.

**Suggest: keep the cue,** muted only as far as the level allows. On `light_aaa` that means a
mix a little nearer the text. Removing the muting everywhere would move Light, which passes, and
RFC-129's ruling is that nothing passing moves.

**Q-3 — 0.63.0.** **Suggest: yes.** It is the contrast work of the release, and Unit 0's sweep
shares the drawing machinery with the non-text unit.

## Review against snora's principles

- **Nobody is confused.** "Every text snora draws holds its level" becomes a sentence that is
  true, and the test says what "every" covers.
- **Robust.** The inventory comes from frames, so a widget that adds text joins the sweep
  without anyone remembering to.
- **Predictable.** It is RFC-129's rule, with no new one.
- **ABDD.** Nothing directional is involved. Both directions draw the same colours; Unit 0 checks
  RTL frames too.
- **Keyboard.** Unchanged.

## What this is not

- Not glyphs or borders: those are the non-text unit (`rfcs/handoffs/non-text-contrast-0-63/`).
- Not the unstyled path.
- Not a palette change.

## Acceptance criteria

1. **Unit 0:**
   - the inventory from frames;
   - every text measured;
   - every cell under its level listed;
   - Q-1's route proposed.
2. **Unit 1:**
   - the sweep, failing on today's `light_aaa` tab (the breakage row);
   - the fixes;
   - nothing that passed moves (a test, as RFC-129's Q-2 table).
3. **The book,** by the architect.
