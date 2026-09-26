# Consumer requirements register

Requirements stated by application teams, recorded as they were stated and
assessed against the tree. **This is not a commitment list.** It is the place a
requirement lives between "a team told us" and "an RFC decided it", so that it
is neither lost in correspondence nor mistaken for a promise.

Each entry records the team's own words, its status at the version it was
assessed against, the evidence, and what would satisfy it. When an RFC acts on
one, the entry links to it. When the status changes, the entry is updated in the
same change.

*Status terms:* **met** (true, and asserted by a test) · **met by inspection**
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

### REQ-003 — Display text rendered verbatim

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

### REQ-004 — Never colour alone

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

> *"Our file explorer caps a directory at 256 drawn entries and must not stall on
> a 100,000-entry one."*

- **Status: met by inspection.** `performance-envelope.md` commits to linear
  rendering and no hidden work. Prefabs render every item the caller passes, and
  capping is the caller's job; virtualised lists are out of scope. Nothing
  asserts linearity at scale.
- **Satisfied by:** a scale smoke test that would fail on superlinear behaviour.
  *(Theme E.)*
