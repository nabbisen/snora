# RFC-115 — Nothing in snora grows for a reader who needs it larger

**Status.** Accepted 2026-11-04. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/115-…`](../handoffs/115-nothing-in-snora-grows-for-a-reader-who-needs-it-larger/implementation-handoff.md).
**Raised.** 2026-11-04, architect, from the owner's question about older users (Theme
E). The owner approved opening it.
**Release target.** Not fixed until Unit 0 reports (Q-4). **Priority.** An
improvement, not a defect: snora meets its stated AA floors. This RFC asks what an
application serving older users can turn up, and makes sure everything snora draws
turns up with it.

---

## The finding

**Text can already grow, on both paths.**
- Since 0.52 (RFC-104), chrome labels on the unstyled path follow the host's
  `default_text_size`.
- On the design path they follow `Typography`, whose fields an application can set.

**The things a hand has to hit do not grow with the text.** These are literals that no
setting reaches:

| What | Size | Where |
|---|---|---|
| Toast close target | 24 × 24 | `crates/snora/src/toast.rs:89` |
| Toast close glyph | 18 | `toast.rs:92` |
| Sidebar button | 48 × 48 | `crates/snora-widgets/src/sidebar.rs:25` |
| Sidebar rail | 64 wide | `sidebar.rs:23` |
| Icon default, including the sidebar's icons | 14 | `crates/snora-widgets/src/icon.rs:8` |
| Toast width | 340 | `toast.rs:79` |

Chip dismiss, tab and breadcrumb targets are derived partly from text and partly from
literals. Unit 0 lists them all; the table above is what a grep found, not an audit.

**At 24 px host text,** a size an older user might choose, the toast's close target is
still 24 px beside text that is 24 px, and the sidebar's icons are still 14 px. A
reader who needs the text larger usually needs what they click larger too: the same
eyes find it, and the same hand hits it.

**The floor is AA.**
- Pointer targets: 24 px (WCAG 2.5.8). On the design path, 25 of the 36 role ×
  padding combinations also meet the preferred 44 px
  (`accessibility-checklist.md`). On the unstyled path the bar is unasserted.
- Text: 4.5:1. Only the two high-contrast presets assert 7:1 for body text
  (`snora-design/src/tests.rs`).

Both floors are right for a general default. Neither is what WCAG's AAA criteria
(2.5.5, 44 px targets; 1.4.6, 7:1 text) ask, and those are the bars commonly
recommended for older users.

## Proposal

**R-1 — Unit 0: measure before ruling.** For every pointer target and every icon snora
draws, on both paths, record:
- the rendered size at the default text size, and at a host text size of 24 px;
- whether it follows text size, tokens or neither.

Measure with `iced_test` bounds, as RFC-105 did. Nothing is decided until this
inventory exists, because the table above came from a grep.

**R-2 — one rule: targets follow text (Q-1).** Every snora pointer target's minimum
becomes a function of the text it sits beside, so that turning up the text turns up
the targets. The same goes for icons that stand in for text, such as the sidebar's.
The application needs no new setting: it already has one.

**R-3 — a statement for applications serving older users** (docs, the architect's),
in `accessibility.md`:
- set a larger text size;
- on the design path, use the high-contrast presets, which assert 7:1;
- prefer the labelled sidebar (RFC-114) and `persistent()` toasts (RFC-113).

Each item names what snora then guarantees, from Unit 0's figures.

## Open questions

**Q-1 — the multiplier, and whether the default moves.**
**The rule always keeps the AA floor:** target ≥ max(24 px, k × text). Without
the `max`, a host text size under 16 px would shrink targets below 24 px, under WCAG
2.5.8. *Added in review.*

- At today's default text of 16 px, a rule of "target ≥ 1.5 × text" keeps every
  default exactly as it is (24 px), and gives 36 px at 24 px text.
- "Target ≥ 2.75 × text" gives 44 px at the default and 66 px at 24 px. That makes
  AAA's 44 px the default, but changes every application's appearance.

**Suggest deciding after Unit 0**, with the first as the starting position: it changes
nothing for anyone who changes nothing. Whether 44 px should become the general
default is a larger question than older users, and it needs adopter evidence.

**Q-2 — 7:1 outside the high-contrast presets.** **Suggest not in this RFC.** On the
unstyled path, colours come from the application's iced theme, which snora does not
choose. The design path already offers 7:1 through two presets. A third, "large"
preset would mix two independent needs, size and contrast, in one switch.

**Q-3 — `Density`.** `Density::{Comfortable, Compact}` is a `Tokens` field that
nothing reads. **Suggest leaving it alone.** R-2 needs no token, and it works on the
unstyled path, which has no tokens. Adding a variant would also break any exhaustive
`match` on `Density`, because it is not `#[non_exhaustive]`. Whether the covenant
counts that as forbidden is not settled, and this RFC does not need to settle it.
*Corrected in review: the first draft called it forbidden outright.*

**Q-4 — release.** **Suggest:** Unit 0 lands as its own review, and R-2's target
release is set then. RFC-113 and RFC-114 do not wait for it.

**Q-5 — ask the adopters?** Whether any of the six teams serves mainly older users
would turn Q-1 from guidelines into evidence. **Suggest: put the question in the
0.53 → 0.54 migration guide** (the book, at no relay cost), not a letter. The
guide is where adopters look when they upgrade. A changelog entry records changes,
and is the wrong place for a question. It is not broken
now, and they did not ask.

## Review against snora's principles

- **ABDD:** today's defaults meet AA. This RFC makes "larger" reach everything snora
  draws (R-2). Whether the *default* should be AAA is left open, with the reason: it
  needs evidence, not guidelines alone (Q-1).
- **Rules a reader can predict:** one rule, "targets follow text", and no new
  setting.
- **Verify claims:** the size table is labelled as a grep, and Unit 0 replaces it with
  measurements.
- **Correspondence cost:** the question to adopters goes in the migration guide
  (Q-5).
- **Second pass:**
  - the rule now keeps the 24 px AA floor below 16 px text;
  - the `Density` covenant claim is no longer overstated;
  - the adopters' question moved from the changelog to the guide.

## What this is not

- Not a new token, preset or `Density` variant (Q-2, Q-3).
- Not a colour change.

## Acceptance criteria

1. Unit 0's inventory, every row measured and none taken from the grep.
2. R-2 (after the Q-1 ruling): a rendered test at 16 px and 24 px host text that
   asserts each target's minimum, failing first on today's literals.
3. R-3's statement, with every guarantee in it traced to a test.

---

## Rulings, 2026-11-04

All suggestions accepted, as revised in `393c753`.

- **Unit 0 first,** as its own review, after RFC-114 lands (it changes the sidebar).
- **Q-1:** the rule is target ≥ max(24 px, k × text). k is ruled after Unit 0, starting
  from 1.5, which leaves today's defaults unchanged.
- **Q-2:** no 7:1 change here.
- **Q-3:** `Density` is left alone.
- **Q-4:** the release target is set when Unit 0 reports.
- **Q-5:** the question to adopters goes in the 0.53 → 0.54 migration guide.
