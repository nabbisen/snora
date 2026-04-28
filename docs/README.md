# Snora documentation

Welcome. This directory holds long-form documentation for snora —
tutorials, guides, architectural reference, and contributor notes. The
generated API reference (per-function signatures and short doc comments)
lives on [docs.rs](https://docs.rs/snora).

## I am new to snora

Read these in order. Each is short and self-contained.

1. [Install](getting-started/01-install.md) — add snora to your `Cargo.toml`.
2. [Hello, snora](getting-started/02-hello-world.md) — the smallest working app.
3. [Add a header, sidebar, footer](getting-started/03-add-a-header.md) — fill the skeleton.
4. [Toasts](getting-started/04-toasts.md) — notifications with framework-managed lifetime.
5. [When to use snora](getting-started/05-when-to-use.md) — fit and non-fit guidance.

## I have used snora before

Pick the topic you need.

- [Overlays — dialogs, bottom sheets, context menus](guides/overlays.md)
- [Header and context menus](guides/menus.md)
- [Direction and ABDD](guides/direction.md)
- [Icons — text, Lucide, SVG](guides/icons.md)
- [Testing UI logic without a renderer](guides/testing.md)
- [Migrating from 0.5 to 0.6](guides/migration-0.5-to-0.6.md)
- [Migrating from 0.4 to 0.5](guides/migration-0.4-to-0.5.md)

## I want to look up a specific symbol or layout

- API reference (per-symbol): [docs.rs/snora](https://docs.rs/snora) / [docs.rs/snora-core](https://docs.rs/snora-core)
- [Architecture overview](reference/architecture.md) — what `snora-core` and `snora` each contribute
- [Vocabulary cheatsheet](reference/vocabulary.md) — every public enum at a glance
- [Built-in widgets](reference/widgets.md) — the prefab `app_header` / `app_side_bar` / etc.

## I want to contribute

Welcome — see the contributor docs:

- [Internal architecture](contributing/architecture.md)
- [Design decisions](contributing/design-decisions.md) — why the API looks the way it does
- [Adding a new overlay kind](contributing/adding-an-overlay.md)
- [Release process](contributing/release-process.md)
