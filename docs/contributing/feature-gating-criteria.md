# When to introduce per-widget feature gates

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
(8-core laptop, SSD, 16 GB RAM, no other heavy work).

Reasoning: snora's selling point includes a fast iteration cycle.
If `cargo check` of the widget set on its own approaches the cost
of recompiling iced, the per-widget gate becomes worth its
documentation cost.

How to measure: run `cargo clean -p snora-widgets && time cargo
build -p snora-widgets --release`. Use `--release` so we are
measuring optimization workload, not debug-info layout. Repeat the
measurement at each release; track the trend.

### 2. Binary size measurably increases for engine-only consumers

**Threshold:** the difference between

```bash
cargo build --release -p snora-example-hello
cargo build --release -p snora-example-hello --no-default-features
```

exceeds **150 KB stripped** on Linux x86_64.

Reasoning: at a small absolute size the noise from iced itself
swamps any saving. The threshold reflects "noticeable in a
discriminating distribution" rather than "the largest possible
absolute saving".

How to measure: build both binaries, strip them
(`strip --strip-all`), `wc -c` each. Re-measure on each release.

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

## Current status (snora 0.7.0)

| Indicator | Status |
|---|---|
| 1. Compile time | Within budget — `snora-widgets` builds in seconds when iced is cached. |
| 2. Binary size | Untested as of 0.7.0; no baseline. Establish in 0.8 if convenient. |
| 3. Heavy optional dep | None — all widgets share `iced` and `snora-core` only. |
| 4. Platform-specific dep | None. |
| 5. Field requests | None. |

Re-evaluate at each release. Update this table as part of the
release process if anything changed.
