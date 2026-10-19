# RFC-111 — The unstyled toast misses AA on six of iced's built-in themes

**Status.** Proposed.
**Raised.** 2026-10-19, architect, from RFC-108's review. The dev team reported one
case as unasserted, without claiming a defect: the Info toast at 1.40–1.66:1 when an
application renders through `snora::render` under `snora::design::theme(tokens)`. The
architect then measured every intent over every built-in theme.
**Release target.** 0.53.0 (Q-3). **Priority.** A defect on the default path, for any
application that uses a theme other than iced's stock Light and Dark.

---

## The finding

`intent_colors` (`crates/snora/src/toast.rs`) pairs each fill with a text colour:

| Intent | Fill | Text |
|---|---|---|
| Debug | `background.strong.color` | `background.strong.text` |
| Info | `primary.strong.color` | `Color::BLACK`, a literal |
| Success | `success.base.color` | `success.base.text` |
| Warning | `warning.base.color` | `warning.base.text` |
| Error | `danger.base.color` | `danger.base.text` |

**The toast contrast suite asserts this on two themes only**, iced's stock Light and
Dark (`theme_contexts()`). Measured over all 22 themes in `Theme::ALL` (scratch
test, mirroring `intent_colors`), **six pairs miss 4.5:1, on six themes**:

| Theme | Intent | Ratio |
|---|---|---|
| Kanagawa Dragon | Info (literal black) | **2.42** |
| Nord | Error | 4.25 |
| Oxocarbon | Error | 4.36 |
| Kanagawa Lotus | Success | 4.40 |
| Solarized Light | Error | 4.44 |
| Solarized Dark | Error | 4.44 |

**The dev team's case is worse.** Under a token-derived theme on the unstyled path,
Info draws black text on a deep blue fill at 1.40–1.66:1. Rendering through
`snora::design::render` does not have the problem (6.70:1), but an application is
free to use `snora::design::theme` with the unstyled renderer.

**Two published statements overclaim:**
- REQ-002's 0.52.0 status: "contrast asserted for every intent on both paths". It is
  now corrected with a dated note.
- The 0.52.0 CHANGELOG: "every intent's text clears AA against its own fill on both
  paths". It gets a `*Corrected*` line under RFC-107's rule (R-3).

The Info literal is also the last fixed colour in the toast, which is REQ-002 again.

**Why iced's pairs are not enough on their own.** `Pair.text` is iced's "readable"
choice for that background. It does not guarantee 4.5:1, as Nord's Error shows at
4.25. RFC-085 picked the Info literal because iced's own Info pair measured 4.43 on
stock themes. That fixed two themes and left the other twenty unmeasured.

## Proposal

**R-1 — keep the pair where it clears AA, and fall back to the stronger pole where
it does not.** For each intent, use today's text when it measures ≥ 4.5:1 against its
fill. Otherwise use black or white, whichever contrasts more with the fill. **The
fallback always clears AA:** for any opaque colour, the better of black and white
reaches at least √(1.05 / 0.05) ≈ **4.58:1**, the point where they contrast equally.
So the rule holds for every theme, including ones that do not exist yet.

- On the stock themes nothing changes, since every pair clears today.
- Info's literal becomes the theme's `primary.strong.text` when that clears, and the
  stronger pole otherwise (Q-1).

**R-2 — assert it everywhere a theme can come from.** Every intent, ≥ 4.5:1 text
against fill, over:
- all of `Theme::ALL`;
- the four presets' themes rendered through the **unstyled** path, which is the dev
  team's configuration.

The toast's dismiss mark, which has the 3.0 floor, gets the same sweep.
`theme_contexts()` grows rather than gaining a second list. Add one drawn check, via
RFC-108's `frame_pixels`, on Kanagawa Dragon / Info.

**R-3 — corrections.**
- The architect adds a `*Corrected*` line under the 0.52.0 entry.
- The dev team fixes `toast_style`'s doc comment, which still says iced's extended
  palette has no `warning` pair (stale since RFC-104), and the comment above the
  Info literal, which cites stock-only measurements as if they were general.

## Open questions

**Q-1 — Info on the stock themes.** RFC-085 chose black on `primary.strong` for its
margin: 5.63 on Light and Dark, against 4.57 for `primary.strong.text`. **Suggest:
keep a pinned pole for Info only where it beats the pair.** In practice, apply R-1
with "the better of the pair and the stronger pole" for Info. Stock Light and Dark
then keep black (5.63), and Kanagawa Dragon gets at least its pair's 7.13. That is R-1's
rule, read as "maximise", for the one intent whose pair was already rejected for
thin margin.

**Q-2 — the design path.** It takes its pairs from tokens and is asserted on all
four presets. **Suggest no change**, but include it in R-2's sweep so the two paths
are asserted the same way.

**Q-3 — release.** **Suggest 0.53.0.** A few toasts on six non-stock themes change
text colour. It is a migration-guide line, and no stock-theme baseline changes.

## What this is not

- Not a change to any fill, nor to the stock themes' appearance.
- Not a covenant change: `snora-design` and `snora-style` are untouched.

## Acceptance criteria

1. R-2's sweep **shown failing** on today's code, listing the six pairs above plus
   the token-theme case, then passing.
2. On stock Light and Dark, each intent's text is unchanged: quote the before and
   after values.
3. The drawn check on Kanagawa Dragon / Info, ≥ 4.5:1.
4. CHANGELOG under Fixed, with the six themes named. The migration guide notes which
   themes' toasts change.
