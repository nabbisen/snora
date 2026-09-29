# Consumer requirements register

Requirements stated by application teams, recorded as they were stated and
assessed against the tree. **This is not a commitment list.** It is the place a
requirement lives between "a team told us" and "an RFC decided it", so that it
is neither lost in correspondence nor mistaken for a promise.

Each entry records the team's own words, its status at the version it was
assessed against, the evidence, and what would satisfy it. When an RFC acts on
one, the entry links to it. When the status changes, the entry is updated in the
same change.

*Status terms* (entries keep their original status line, with later statuses added beneath it, so the history stays visible): **met** (true, and asserted by a test) · **met by inspection**
(true, but nothing would fail if it stopped being true) · **partly** · **not
met** · **blocked** (snora cannot act alone).

---

## From tekstide — 2026-09-26, assessed at 0.51.0

tekstide evaluated snora at 0.33.1 and did not adopt it. This letter states what
would have to be true for adoption, as *"requirements, not requests"*.

### REQ-001 — Accessible names, roles and states

> *"If snora shipped accessible names, roles and states on its interactive
> components — backed by an accessibility tree rather than by tooltips — …
> it alone would reopen adoption here."*

- **Status: blocked.** iced 0.14 exposes no accessibility tree, and no AccessKit
  or AT-SPI crate is in snora's resolved graph. snora's position (RFC-045) is to
  integrate one when iced exposes it and not to build a parallel abstraction.
- **Satisfied by:** an iced release with an accessibility tree, followed by
  snora exposing name, role and state for every interactive component.
- **Possible now:** an inventory of each component's name, role and state and
  their source (caller label, snora text, none), so the work is sized before
  iced moves. *(Theme D.)*
- **Decisive for tekstide's adoption.**

### REQ-002 — Nothing hardcoded that users configure

*Proposed as **RFC-104** (2026-09-26).*

**Status at 0.52.0: met (RFC-104).** Every chrome label size comes from the host: `default_text_size` on the unstyled path, and `Typography` roles on the styled path (tab, crumb and menu labels → `label`; header and toast title → `title`; toast message → `body`). The toast's Warning fill comes from the theme or tokens, with contrast asserted for every intent on both paths. **`scripts/check-literal-sizes.sh` fails the build on a literal text size** in any non-test source file. It allow-lists one named constant, `CLOSE_GLYPH_SIZE`, with its reason, and rejects path-qualified wrappers such as `iced::Pixels(13.0)`. This is the scan tekstide described.

> *"Font size, font family and every theme colour come from the host."*

- **Status: not met.** Chrome label sizes are literals **in both the unstyled and
  the styled variants**: tab 13, crumb 13, menu 14, header 16 (bold), and toast
  title and message 16 / 14. The styled variants pass no text size, so a host's
  `Typography` does not reach them. Colour exceptions: the toast's Warning fill
  (`WARNING_COLOR`), the fixed black/white text on toast fills, and the
  default-path modal dim. Font family is met; snora sets none except for lucide
  glyphs.
- **Also a defect for current consumers:** an application's text-size setting
  does not move snora's chrome labels.
- **Satisfied by:** every chrome label size taken from the host (tokens on the
  styled path; an explicit input or an accepted default on the unstyled path,
  since iced 0.14 sizes are absolute), and the toast's colours from the theme.
  *(Theme A.)*
- **tekstide, second letter:** this is *"the single item that would keep us out
  regardless of the others"*. The shape that worked for them: *"no component reads
  a size or a colour directly — every one takes them from a theme value the host
  supplies"*, plus **a scan that fails the build on a literal size in a surface
  file**, which is *"what keeps the rule true a year later"*. That fits snora's
  existing `scripts/check-*.sh` family.

### REQ-003 — Display text rendered verbatim

*Proposed as **RFC-105** (2026-09-26).*

**Status at 0.52.0: met (RFC-105).** Caller text wraps inside a word when it must (`Wrapping::WordOrGlyph`) on the toast title and message, the notice title and body, and every tooltip, which now has a shared 320 px maximum width. That was measured on each surface, and asserted by a rendered-area check, since `visible_bounds()` cannot see the clip. **Verbatim** is asserted with tekstide's fixture (a bidi override, a newline and a control character), which is found by exact content in every caller-text surface. **Not asserted:** the glyphs drawn for those characters, which are iced's and cosmic-text's; the harness exposes frame hashes, not pixel reads.

> *"A component must not re-escape, re-wrap or re-interpret them."*

- **Status: partly.** No caller string is transformed (no trim, truncate,
  replace, case change or escaping), and no shaping or wrapping override is set,
  so iced's `Shaping::Auto` handles bidi. **But text that cannot be broken at a
  word is silently cut off:** a 400-character unbroken string in a toast
  measured as one line clipped at the 288 px message column, and nothing marks
  the cut. File names, paths, URLs and hashes are the usual cases.
- **Satisfied by:** caller text shown whole (breaking inside a word when it must),
  and a test that renders escaped markers and finds each by **exact** content.
  *(Theme E.)*
- **tekstide's acceptance criterion** (second letter, 2026-09-26): *"a fixture
  string containing a bidi override, a newline and a control character, passed in
  and read back from what the component draws."*
- **Mechanism note, measured.** A clipped text is **not** detectable by
  `visible_bounds() ≠ bounds()`. The unbroken string measured 288.0 × 18.2 for
  both, because the text widget's box is its constrained layout and the glyphs
  overflow inside it. What detects it is comparing the string's natural
  (unconstrained) size with its rendered box, the natural-size technique the
  RFC-099/100 tests already use.

### REQ-004 — Never colour alone

*Proposed as **RFC-103** (2026-09-26).*

**Status at 0.52.0: met by snora's rule, with one qualification (RFC-103, RFC-106).** Every state snora draws carries an asserted cue, held by a state register that fails to compile without one (see the accessibility guide's table), and the disabled unselected chip, which looked enabled, is fixed. **Qualification:** a selected chip's cue is luminance (≥ 3.0:1, measured 6.19–11.75:1), not a word or shape. tekstide's word-or-shape bar is met for chips by the caller putting a mark in the label, which is documented. Variants remain the caller's to put into words, by ruling (RFC-093).

> *"Every state that matters also carries a word or shape."*

- **Status: partly.** The active tab carries a shape (the underline). The sidebar
  active item is a fill, asserted at 3.0:1 or more against the rail. **A chip's
  selected state differs from unselected in colour only.** Its luminance
  difference measures 6.19–11.75:1, but it has no word or shape, and no test
  asserts it. Toast intent, notice tone and progress tone are colour alone by
  ruling (RFC-093: the caller supplies the words). That division of labour is
  documented for variants, **not for states**.
- **Satisfied by:** every snora-drawn state carrying an asserted non-colour cue,
  or a documented, asserted division of labour for it. *(Theme B.)*

### REQ-005 — Contrast asserted on both renderers

> *"We ship where the GPU path may not start, so a figure from one renderer does
> not transfer."*

- **Status: partly.** Contrast is asserted on style functions, which does not
  depend on the renderer, provided no primitive the renderers draw differently
  alters what sits under text. RFC-102 removed the known one, but no general guard
  exists. The rendered suite passes under wgpu (measured locally, serialised), but
  CI renders tiny-skia only, and no test reads contrast from pixels.
- **Satisfied by:** a guard against renderer-divergent primitives, plus the
  rendered suite running on both renderers in CI. *(Theme C.)*

### REQ-006 — Bounded work on hostile input

*Proposed as **RFC-105** (2026-09-26).*

**Status at 0.52.0: met (RFC-105).** Asserted by `bounded_work.rs` in release in CI: 10× the toasts must cost under 30× the layout time. It measured 11.63–11.96, and 92.18 against a deliberate quadratic edit. The numbers are recorded in `performance-envelope.md`: 656 ms layout at 100,000 toasts on the reference machine.

> *"Our file explorer caps a directory at 256 drawn entries and must not stall on
> a 100,000-entry one."*

- **Status: met by inspection, and now measured.** `performance-envelope.md`
  commits to linear rendering and no hidden work. Prefabs render every item the
  caller passes, and capping is the caller's job; virtualised lists are out of
  scope. **Measured 2026-09-26** (toasts, release build, tiny-skia simulator,
  architect's machine): layout **0.6 / 5.9 / 62.9 / 656 ms** for **100 / 1,000 /
  10,000 / 100,000** toasts, which is linear. Nothing asserts it yet.
- **tekstide's acceptance criterion:** *"one input far past any plausible use …
  with the number written down, not the word 'fast'."*
- **Satisfied by:** that measurement recorded in `performance-envelope.md`, plus
  a ratio assertion (e.g. 10× input costs well under 100× time), which stays
  machine-independent. *(Theme E.)*

### From tekstide's second letter — substrate note, not a requirement

tekstide pointed at iced 0.14's `iced_selector` (`id`, `is_focused`,
`Target::bounds`, `visible_bounds`). **snora already uses it:** `iced_test`
re-exports it as `iced_test::selector`, and the rendered tests (sidebar fit,
close controls, toast target, tab bar edges) are built on those calls. **The part
not yet used is `is_focused()`**, which could assert keyboard-focus reachability.

