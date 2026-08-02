# RFC 037 — Coherent defaults for snora-rendered surfaces

**Status.** Proposed
**Tracks.** Snora Design positioning. Extends RFC-020's boundary to
snora's own rendered surfaces and narrowly amends the theme-owning non-goal.
Authorizes RFC-038 … RFC-040.
**Touches.** `docs/src/design/overview.md`,
`docs/src/contributing/design-decisions.md` (index row + theme section),
`docs/src/contributing/feedback-and-scope.md` (scope table row),
`README.md` (design-notes bullet). No source changes.

## Summary

A snora application that opts into Snora Design today gets token-derived
styling on the primitives *it* builds, and nothing else. Snora's own
rendered surfaces — header, sidebar, footer, menus, dialog, sheet, modal
dim, toasts — do not participate in the token system. The observable
result is an application whose framework-supplied chrome reads as flat,
square, hairline-bordered and arhythmic regardless of which preset is
selected, including the high-contrast presets.

This RFC does two narrow things:

1. **Extends RFC-020's boundary** to cover surfaces snora itself renders,
   not only primitives the application builds.
2. **Amends the theme-owning non-goal** to permit snora to *emit* an
   `iced::Theme` derived from tokens, under the `design` feature only.

It establishes one load-bearing compatibility invariant: **with `design`
inactive, rendering is unchanged.** The skeleton remains the default; the
coherent appearance is what activating a feature buys.

## Motivation

### The observable defect

| Location | Current behavior |
|---|---|
| `crates/snora-widgets/src/style.rs:35-48` | Chrome containers: `radius: 0.0`, border `width: 1.0`, `background: None`, `Shadow::default()` — square, hairline, unfilled, unelevated |
| `crates/snora/src/overlay/dialog.rs:14` | `center(dialog.content)` — a dialog has **no card**: no background, padding, radius, or shadow |
| `crates/snora/src/render.rs:192,210` | Modal dim hardcoded `rgba(0,0,0,0.4)`, independent of theme and tokens |
| `crates/snora/src/toast.rs:46` | `WARNING_COLOR` hardcoded; toast padding literals `24/12/8/4` |
| `crates/snora-widgets/src/{header,sidebar}.rs` | Spacing literals `12`, `16` with no shared rhythm |

`snora-design` defines `Radius`, `Spacing`, `Typography`, and
`FocusTokens` scales. **None of them reaches any surface above.**

This is not a matter of taste and not a limitation of iced. It is that
snora's own chrome was written before the token system existed and was
never connected to it.

### What RFC-020 already settled

RFC-020 amended the philosophy and remains in force. Its boundary:

> Snora positions and stacks. Optionally, Snora Design provides a small,
> coherent desktop application design system for local-first productivity
> tools. Applications still own content, domain behavior, complex widgets,
> workflow logic, and final brand identity.

RFC-020 already authorizes "accessible visual **defaults**", already lists
"No forced design-system adoption" as a non-goal, and already preserves
the minimal path. `README.md:110` already reads "Skeleton first, optional
design defaults." **The opt-in-defaults position is ratified**; this RFC
does not reopen it.

### The two genuine gaps

1. **Scope.** RFC-020's scope table classifies candidates that
   *applications* use — palette tokens, typography, button helper, card
   helper. It never contemplated snora styling *its own* rendered
   surfaces. That is the larger half of what a user actually sees, and it
   sits outside every existing classification.
2. **Theme emission.** DEC-02 ("theme-aware, not theme-owning", *Firm
   boundary*) and the `feedback-and-scope.md` row ("Snora-owned theming
   layer — Firm non-goal") block deriving an `iced::Theme` from tokens.
   Without it, stock iced widgets (`text_input`, `pick_list`,
   `scrollable`) and the window background cannot follow the selected
   preset — so a high-contrast preset reaches only part of the screen.

### A defect inherited from RFC-020

RFC-020's acceptance criteria include "Boundary statement is added to
docs." It was not: neither the boundary statement nor the longer
non-widget-framework statement appears anywhere under `docs/`.
`design/overview.md` carries a paraphrase only. This RFC satisfies that
criterion as part of restating the boundary.

## Goals

- G-1. Bring snora-rendered surfaces into scope for token-derived styling
  when `design` is active.
- G-2. Permit token-derived `iced::Theme` emission under `design`.
- G-3. Guarantee that `design`-inactive rendering is unchanged.
- G-4. Restate the boundary in `docs/`, satisfying RFC-020's open criterion.
- G-5. Leave the declined scope untouched and visibly so.

## Non-goals

- **N-1. No component catalog.** N-3/N-4/N-5 stand: no form or validation
  widgets, no tables/charts/grids, no avatar/badge/spinner.
- **N-2. `design` does not become default-on.** DEC-11 stands; that needs
  its own size and build-cost review.
- **N-3. Snora does not own theme state.** It emits a value; the
  application holds it and passes it to iced. No registry, no cascade, no
  global.
- **N-4. No source changes in this RFC.** It authorizes RFC-038 … RFC-040.
- **N-5. Brand identity remains the application's.** Snora supplies a
  coherent neutral default, not a visual identity.

## Amended boundary statement

To be added to `docs/src/design/overview.md`, replacing the paraphrase:

> Snora is not becoming a general widget component framework. Snora remains
> a small layout and overlay framework for iced-based desktop applications.
>
> By default, snora positions and stacks: applications supply content and
> styling, and snora's own chrome carries only the minimum needed to be
> legible.
>
> When the optional `design` feature is active, Snora Design additionally
> supplies a coherent visual default for the surfaces snora itself renders
> — chrome, overlays, and notification surfaces — derived from tokens the
> application owns and may replace. Applications still own their domain
> behavior, complex widgets, validation, data presentation, navigation, and
> final brand identity.

## Amendment: theme-producing, not theme-owning

DEC-02 is amended, not revoked. The distinction that makes this coherent:

- **Theme-owning** — snora defines a parallel theming abstraction, holds
  theme state, and requires applications to configure appearance through
  snora rather than through iced. **Still declined, permanently.**
- **Theme-producing** — snora offers a pure function from tokens to an
  `iced::Theme`. The application decides whether to call it, owns the
  result, and hands it to iced through iced's own `.theme()` hook. Snora
  holds no state and intercepts nothing. **Permitted under `design`.**

The original DEC-02 reasoning was that a parallel theme layer "would
duplicate iced's system, force applications to configure theming twice,
and create a maintenance surface with no commensurate value." Emission
does not duplicate iced's system — it *feeds* it — and it removes a
double-configuration that exists today, where an application must
separately configure tokens and an iced `Theme` and keep them manually in
agreement.

DEC-02's status changes from *Firm boundary* to **Accepted**, with the
theme-owning half restated as the firm part. The
`feedback-and-scope.md` row changes from "Snora-owned theming layer — Firm
non-goal" to distinguish owning (declined) from emitting (available under
`design`).

## The gating invariant

**With `design` inactive, snora's rendered output is unchanged from
v0.25.** This is a compatibility promise, not an aspiration:

- No existing public function changes signature or behavior.
- Existing downstream applications see no visual change on upgrade unless
  they opt in.
- The `default = ["widgets"]` build continues to style chrome from
  `iced::Theme` exactly as it does today.

RFC-038 … RFC-040 must each demonstrate compliance. Any proposal that
cannot preserve this invariant is out of scope for this milestone and
requires a separate owner decision.

## Consequences for dependent RFCs

This RFC authorizes, under RFC-036's additive-only covenant:

| RFC | Authorized to |
|---|---|
| **038** | Add a pure `Tokens → iced::Theme` emission function, with contrast verification of the *generated* palette |
| **039** | Style engine-owned surfaces (dialog card, modal dim) via a `design`-gated render path |
| **040** | Style chrome geometry (radius, spacing rhythm, elevation) and interaction affordance (hover/pressed clarity, hit targets) |

Each must cite RFC-036's covenant and state its compliance, and must not
modify `snora-core` or any existing widget signature.

## Compatibility and migration

No API change in this RFC. For the milestone as a whole: additive and
opt-in, with the gating invariant above. No migration guide is required
for applications that do not enable `design`. Applications that *do*
enable it will see a visual change and must be told so in the v0.26
migration guide — that is a **Changed** entry, not **Fixed**, because it
alters documented appearance rather than restoring it.

## Security

No new data flow, dependency, integration, or auth logic. Existing
controls (requirements §1.4; cross-cutting S-1 … S-6) remain valid. Of
note: this milestone deliberately avoids third-party styling or plug-in
dependencies, so the supply-chain surface (S-4) is unchanged — see
Alternatives.

## Alternatives considered

- **Status quo (unstyled skeleton).** Honest, and cheap. Rejected because
  the project already ships a token system that reaches almost nothing,
  and because the observed result is an application users described as
  unpleasant to look at and to operate. If chosen, honesty would require
  documenting that outcome as intended.
- **Mediator for third-party styling crates / plug-ins.** Rejected on
  three grounds. (a) Supply chain: `lucide-icons` declares `iced = "0.*"`,
  which produced the v0.18.1 type-parameter mismatch (DEC-17); every
  additional iced-typed third-party dependency reopens that failure mode,
  and a mediator's purpose is to have many. (b) A plug-in surface is a
  trait/registry abstraction — the shape deleted in v0.4 as `PageContract`
  (N-1). (c) S-4 commits to a minimal dependency surface. For the scope
  needed, owning the defaults is smaller than the architecture required to
  broker them.
- **Default-on coherent appearance.** Rejected: it would silently restyle
  every existing downstream application on upgrade, and contradicts
  DEC-11.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Two visual paths to maintain (design on/off) | Certain | Medium | The off path is frozen at current behavior — no new work, no regression surface. Visual QA doubles only for the on path. |
| Emitted theme fails WCAG where tokens pass | Medium | **High** | RFC-038 owns this: control extended-palette generation rather than relying on iced's, and extend contrast tests to the generated palette. Blocking for 038. |
| Scope creep toward a component catalog | Medium | High | N-1 restated; RFC-020's scope-gate question applies unchanged to every addition |
| "Coherent default" becomes a de facto brand | Low | Medium | Defaults are neutral and fully replaceable; slots still accept any `Element` |
| Accessibility promise outruns iced | Medium | Medium | Focus visibility on standard controls is **not** deliverable in iced 0.14 (`crates/snora/src/lib.rs:133-139`); recorded as deferred with an upstream trigger, not claimed |

## Open questions

- **Q-1.** Does the v0.26 migration guide need a visual before/after for
  `design` adopters? Recommended: yes, using the design workbench. Non-blocking.
- **Q-2.** Should `WARNING_COLOR` be retired once tokens supply the warning
  role under `design`? Deferred to RFC-039; it interacts with the stale
  iced-warning-pair claim under audit in RFC-035 F-4.

## Acceptance criteria

1. `docs/src/design/overview.md` carries the amended boundary statement
   verbatim, satisfying RFC-020's open criterion.
2. `design-decisions.md` theme section distinguishes theme-owning
   (declined) from theme-producing (permitted under `design`); the index
   row's status is updated with a reconsideration trigger.
3. `feedback-and-scope.md` row distinguishes owning from emitting.
4. `README.md:110` reflects that `design` supplies defaults for snora's own
   surfaces, not only for app-built primitives.
5. The gating invariant is stated in `design/overview.md` as a
   compatibility promise.
6. No declined-scope item (N-3/N-4/N-5, `snora-test`, i18n, game-loop) is
   weakened.
7. No source file changes.

## Release implications

Documentation and governance only. Ships with the v0.26 milestone it
authorizes. Advances no 1.0 gate directly; it does make D-7 ("no component
catalog creep") a sharper review each minor, since the boundary now
extends further.
