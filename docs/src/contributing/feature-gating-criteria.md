## Supported feature combinations (CI matrix)

The following combinations of `snora` features are explicitly supported and
verified by CI on every PR and push to `main`:

| Combination | What it builds |
|---|---|
| default (no flags) | Engine + `widgets`. |
| `--no-default-features` | Engine only. No widget re-exports. |
| `--no-default-features --features widgets` | Engine + `widgets`. |
| `--no-default-features --features widgets,lucide-icons` | Engine + widgets + Lucide icon constants. |
| `--no-default-features --features widgets,svg-icons` | Engine + widgets + SVG icon support. |
| `--all-features` | All public optional features. |

`lucide-icons` and `svg-icons` are **subordinate** to `widgets`: they gate
widget-side rendering that requires `snora-widgets` and have no meaningful
effect without it. The CI matrix therefore does not test `lucide-icons` or
`svg-icons` in isolation. This policy is recorded here so it is visible when
the matrix is read (see RFC-014-D for the planned v2 icon-gating policy).

## When to introduce per-widget feature gates

Snora's current widget feature gating is **coarse**: a single
`widgets` feature on the `snora` crate switches the entire
`snora-widgets` set on or off. There is no `widget-tab-bar` /
`widget-breadcrumb` / `widget-header` distinction.

This page records the criteria that would justify revisiting that
decision and introducing per-widget feature gates. **Do not split
the `widgets` feature into multiple features unless at least one of
the indicators below applies.**

## Background

The wider the feature matrix, the more combinations have to compile,
test, and stay coherent in documentation. Five widgets with five
toggleable features yields 32 combinations; ten widgets, 1024. Each
combination is a potential bug surface (one widget references
another's helper, breaks when the helper is gated out) and a piece
of documentation surface (which combinations are supported, which
are not).

We accepted the cost of *one* on/off (`widgets`) because engine-only
builds are a real, named use case. We deferred everything finer.

## Indicators that justify revisiting

If two or more of these become true, open a discussion to introduce
per-widget feature gates.

### 1. Compile time grows past acceptable

**Threshold:** `cargo build -p snora-widgets` from cold cache
exceeds **30 seconds on a developer's machine of average specs**
(8-core laptop, SSD, 16 GB RAM, no other heavy work). **Unchanged by
RFC-062** — the number is not wrong, only what has been read as
measuring it was.

**`build_widgets_ms` (`reference/build-cost-budget/compile-time.csv`) is
not indicator 1's proxy, and RFC-062 retires the claim that it was.**
The threshold is written against *a developer's machine*;
`build_widgets_ms` measures GitHub CI, which since RFC-043 rebuilds
iced's entire transitive closure from scratch on a shared runner. Those
are different quantities — CI cold-build time under contended shared
hardware is not developer-machine build time — and RFC-050 additionally
showed the CI column carries 36–60% spread between identical-runner
releases, so even restated in CI's own terms no single reading would be
trustworthy. **The proxy was always measuring something else; RFC-043
only made the mismatch visible** by removing the warm-cache artifact
that had coincidentally kept early readings looking plausible.

**How indicator 1 is actually assessed:** run
`cargo build -p snora-widgets --release` from a `cargo clean -p
snora-widgets` state, timed, on a machine matching the threshold's own
description. **Current status: unassessed.** No such run has been
recorded as of 2026-08-18 (RFC-062). This is stated as a fact, not
softened — an indicator can be honestly *unassessed*, and that is a
better state than one silently assessed by a proxy measuring a different
quantity.

`build_widgets_ms` stays tracked in
[`reference/build-cost-budget/compile-time.csv`](../reference/build-cost-budget/compile-time.csv)
(appended on every release tag by the `build-cost` GitHub Actions
workflow) for context and as RFC-050's own trend material via
`design_overhead_ratio` — but it is **not** indicator 1's assessment,
and a reader should not treat a CSV row as one.

### 2. Binary size measurably increases for engine-only consumers

**Threshold:** `widgets_diff_bytes` exceeds **150 KB stripped** on Linux
x86_64. Unchanged by RFC-062 — only the method description below was
stale.

Reasoning: at a small absolute size the noise from iced itself
swamps any saving. The threshold reflects "noticeable in a
discriminating distribution" rather than "the largest possible
absolute saving".

**How to measure — corrected (RFC-062): this is not a diff of two
`snora-example-hello` builds.** That description predates RFC-041/043;
since RFC-041 the actual method is three probe crates
(`size_probe_engine`, `size_probe_widgets`, `size_probe_design`) sharing
a common baseline application, each adding exactly one minimal,
representative call to the feature it measures — see
[`binary-size-budget.md`](../reference/binary-size-budget.md) for why a
naive feature-on/feature-off diff undercounts (RFC-041 found the
original two-binary approach measured **0** bytes, because the
`widgets`-enabled binary never *called* a widget, so the linker stripped
the whole unused feature). `widgets_diff_bytes` is recorded per release
in
[`reference/binary-size-budget/binary-size.csv`](../reference/binary-size-budget/binary-size.csv),
appended by the `binary-size` GitHub Actions workflow. **Current status:
46,464 B (~45 KB) as of 0.38.1 — comfortably under the 150 KB
threshold, not met.**

### 3. A widget gains a heavy optional dependency

**Threshold:** any single widget pulls in a crate larger than
**500 KB compiled** that is not already required by the rest of
`snora-widgets`.

Examples that would qualify (none have shipped):

- A `markdown_view` widget pulling in a markdown parser.
- A `data_table` widget pulling in a sortable-table or virtualized-list crate.
- A `chart` widget pulling in `plotters`.

When this happens, the widget should ship behind its own feature
flag *immediately* — that is the only way users who do not need
it can avoid paying for it. Do not wait for two indicators.

This is the only indicator that, taken alone, justifies a new
feature gate. The others require corroboration.

### 4. A widget needs a new platform-specific dependency

**Threshold:** any single widget links a system library that the
rest of snora does not (e.g. `libnotify` for desktop notification
fallback, a system clipboard binding beyond what iced provides).

Reasoning: optional system bindings are exactly what feature flags
are for. Engine-only builds and CI cross-compile builds need to
opt out cleanly.

### 5. A widget category is requested for distinct opt-in

**Threshold:** at least **three independent applications** in the
field tell us they want a specific subset of widgets without the
rest. "I only use the chrome widgets, not navigation" or "I only
want icons and menus, no tab bar".

Reasoning: this is the user-experience signal that the coarse gate
no longer matches actual usage patterns. It is a soft indicator —
two reports could be a coincidence; three suggests a structural
mismatch.

## What "revisiting" looks like

If the criteria justify per-widget gates, the work is:

1. Add features named after their widget (`widget-header`,
   `widget-sidebar`, `widget-tab-bar`, …) to `snora-widgets/Cargo.toml`.
2. Gate each module declaration in `snora-widgets/src/lib.rs` with
   `#[cfg(feature = "widget-X")]`.
3. Make the existing `widgets` feature on `snora` enable all of
   them, so the *default* user experience is unchanged.
4. Document the new features in `docs/getting-started/01-install.md`
   and `docs/guides/feature-gating.md`.
5. Bump the minor version (these are additive features).

The `widgets` umbrella feature should remain. We never want users
who do not care about the partition to face a long feature list.

## What this document is not

This is not a checklist that *forces* a split when an indicator is
met. It is a list of inputs to a judgment call. If compile time
grows but the cause is a transitive iced bump that affects all
crates equally, splitting widget features will not help; the right
fix is elsewhere. Indicators trigger a discussion, not a refactor.

## Documentation scope when a capability arrives, leaves, a standing answer is invisible, or a claim is withdrawn

**When a capability lands, every default-path page that states it is
absent is part of the change scope. When a capability — or a path to
one — is removed, every page that states it still resolves is part of
the change scope, symmetrically. When a governance decision or policy
answers a question a consumer would ask — a stability guarantee, the
true scope of a constraint — recording it only in a contributor
document is the same defect as either of the above. And when a release
withdraws or narrows a claim consumers may have relied on, correcting
it in our own docs is *again* the same defect, in a fourth shape:**
the answer (or the correction) exists, and the consumer who needs it
cannot reach it, or reaches it without knowing what to do about it.
Documenting the change (or the standing answer) in `docs/src/design/`
or a migration guide is necessary and not sufficient: a consumer who
never reads that page, or never reads `contributing/`, is left without
the answer, or worse, with a stale one.

**The fourth case has a distinction the other three do not need:
announcing a correction is not the same as prompting the action a
consumer who relied on the withdrawn claim needs to take (RFC-067).**
snora's 0.34.0 release withdrew two consumer-facing claims and
explained each withdrawal thoroughly — the `text_muted` entry runs to a
paragraph including "that exemption was ours, invented, and it is
withdrawn." Neither explanation named what a consumer who had **already
acted on the claim** should now do. The same release proves we already
know the difference: its *rendered* change (a border-color repair)
carries "re-check any screenshot tests or visual regression baselines
that include card or dialog borders…"; its *documentary* withdrawal
carries nothing. A withdrawal note must do what the rendered-change
note already does: name the re-check, not only the correction. A page
that states "X is withdrawn" without stating "if you relied on X,
re-check Y" has announced a correction, not retracted the claim from
where it landed.

This is not hypothetical. Ten downstream/review reports, across all
four cases, were the same omission:

| Report | Change | Page or record that still carried the old claim |
|---|---|---|
| apimokka (2026-08-04) | focus/AT limitation documented under `contributing/` | consumer-facing docs had no route to it |
| arama (2026-08-15) | dialog card, RFC-039, v0.27.0 (**arrival**) | `guides/overlays.md` still said "positioner, not a styler" with no scope |
| RFC-056 review (2026-08-15) | `snora_widgets::design::{style, theme}` removed (**removal**) | `design/feature-flags.md`, `contributing/api-governance.md`, `contributing/semantic-accessibility.md` still named the removed path — one of them asserting it "keeps resolving" |
| tekstide, RFC-059 (1) (2026-08-17) | the `BLOCKED` focus-ring label over-scoped a constraint (**standing answer, mis-scoped**) | `design/iced-style-bridge.md` — consumer-facing — repeated the same over-scoped claim the contributor copy was fixed for |
| tekstide, RFC-059 (2) (2026-08-17) | the token-surface stability covenant answers "does it churn?" (**standing answer, undiscoverable**) | no consumer-facing page stated it; only `contributing/api-governance.md` and two passing mentions existed |
| orbok (2026-08-19) | `text_muted` "exempt from mandatory contrast" invented and withdrawn, 0.34.0 (**claim withdrawn**) | orbok's WCAG conformance record cited the exemption as justification |
| knotra (2026-08-19) | same withdrawal (**claim withdrawn**) | excluded the role from their WCAG AA suite, naming our doc comment as the authority; renders it as select-widget placeholder text |
| aaai (2026-08-19) | same withdrawal (**claim withdrawn**) | excluded it from their contrast test across **28 call sites** — diff line numbers, a "Not selected" status, onboarding steps |
| apimokka (2026-08-19) | focus ring "cannot be rendered" on iced 0.14, over-scoped, withdrawn, 0.34.0 (**claim withdrawn**) | written into **RFC MK-023**, their accessibility contract, as a reason full Tab traversal might be unachievable |
| orbok (2026-08-19) | same withdrawal (**claim withdrawn**) | carried the same statement "almost verbatim" |
| knotra, RFC-072 (2026-08-19) | contrast thresholds are floors, never ceilings — never stated to consumers (**standing answer, undiscoverable**) | knotra asserted `border` against `surface` *stays below* AA (4.5) to justify excluding a notice tone — a bound snora has never held and does not guarantee; no consumer-facing page said thresholds are one-directional until this RFC |

Eleven downstream/review reports, across all four cases, and every one of
the five withdrawal-propagation instances found by the consumer — two
of them only while reading a
seven-release migration bundle. Naming five instances is not a claim
that the propagation list is complete: four consumers told us; the
others were not asked.

RFC-039 correctly documented the dialog card in `docs/src/design/`. What
neither that RFC nor its review caught was that
`docs/src/guides/overlays.md` — the page a consumer actually reads about
dialogs — still stated the opposite. Adding the new page was not the
whole job; retracting the old denial was the missed half.

RFC-056 was the mirror-image miss: the implementation grepped `crates/`
for stale references to the removed shim and fixed what it found there,
but the rule as originally written only named capabilities *landing*,
so the equivalent sweep of `docs/src/` for the capability *leaving*
wasn't in scope until review caught it.

RFC-059's two instances are neither arrival nor removal — nothing about
snora changed. A review control (`BLOCKED`) closed a question with an
over-scoped claim, and a governance covenant answered a question that was
never asked outside `contributing/`. Both are **standing answers**: true
facts about snora that already existed, reachable only from the half of
the book a consumer does not read. tekstide declined to adopt snora
partly for want of the second one — the cost of this omission is not
hypothetical either.

When gating, ungating, or removing a capability (or a specific path to
one) behind `design` (or any feature) — or when recording a governance
decision that answers a question a consumer would ask — grep the
default-path docs for the specific claim the change contradicts, or add
a consumer-facing page/line stating the answer, before considering the
documentation done. Arrival claims ("X is not available") when adding,
resolution claims ("X resolves at this path") when removing, and a
reachable statement of the answer when it is a standing one. An empty
grep is not the acceptance signal by itself — read each surviving hit
and confirm it is still true for the path it describes; a claim can
also hide across a line wrap that a naive single-line grep pattern
misses.

**And when a release withdraws or narrows a claim consumers may have
relied on**, the release note names what to re-check, not only what
changed — in the same voice and the same place as the re-check line an
appearance change already carries (see, e.g., the modal-dim migration
guides' "re-check any screenshot tests…" convention). There is no grep
for "we stopped asserting something" — this is not a check to
mechanise, it is a question the release-note author answers, because
they are the person who knows: *did this release withdraw, narrow, or
correct anything we previously told consumers? If so, does the note say
what a consumer who acted on it should now do?*

## Current status (snora 0.38.2, re-derived 2026-08-20, RFC-062)

**Every row states a measured value against its threshold and whether
the threshold is met — a prose verdict alone is what let "Within
budget" sit beside a 3.2×-over-threshold figure for ten minors
(RFC-062).**

| Indicator | Threshold | Current | Met? |
|---|---|---|---|
| 1. Compile time | 30 000 ms, developer machine, cold | **Unassessed** — see indicator 1 above; the CI proxy previously cited here measured a different quantity and has been retired | Unknown |
| 2. Binary size | 150 KB stripped (`widgets_diff_bytes`) | **46,464 B (~45 KB)** — `binary-size.csv`'s 0.38.1 row. **Back inside the long-running band, and that corrects last release's reading:** 0.38.0 was recorded here as "+128 B, a new high" outside a seven-release 46,336–46,464 range; 0.38.1 returned to 46,464 having changed no executable code at all — documentation, one test constant, one compiled snippet. A ±128 B swing across a release that could not have moved code is the measurement's noise, not a trend, so 0.38.0's "new high" was over-read. Treat moves at this magnitude as noise; 0.3% of a 150 KB bar either way | **No** |
| 3. Heavy optional dep | >500 KB compiled crate, not already shared | None — re-checked against current manifests, not inherited: `snora-widgets` depends on `snora-core`, `snora-design` (optional), `snora-style` (optional, arrived RFC-055), `iced`, `lucide-icons` (optional); `snora-style` itself depends only on `snora-design` and `iced` — no new heavy dependency. 0.38.0 added one workspace member, `examples/book_snippets` (RFC-069), which is `publish = false` and ships to nobody | **No** |
| 4. Platform-specific dep | Any system library not already required | None — same manifest check as indicator 3 | **No** |
| 5. Field requests | Three independent applications | None received | **No** |

**At most one indicator could be met** (indicator 1, if a
developer-machine measurement were taken and found over threshold) —
short of the "two or more" the trigger requires. See
[design decisions § coarse `widgets` feature gate](design-decisions.md#why-widget-feature-gating-is-coarse-not-per-widget)
for the full not-fired statement.

Re-evaluate at each release — `release-process.md`'s checklist now
points here (RFC-062; previously nothing did, which is how this table
went ten minors without an update).

## Icon and asset feature policy

The icon feature policy is intentionally separate from the widget-splitting
criteria above. It is documented in `docs/src/guides/icons.md` (the
"Why icons are feature-gated" section). Key rules:

- `Icon::Text` is always available (no feature flag).
- `lucide-icons` and `svg-icons` are optional and subordinate to `widgets`.
- Adding a new icon ecosystem (a third icon pack) requires an RFC and
  evidence of repeated demand.
- Raster asset helpers are application responsibility.
