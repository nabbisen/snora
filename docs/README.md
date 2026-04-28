# Snora documentation

This directory contains the full long-form documentation for snora —
tutorials, guides, architectural reference, and contributor notes. The
generated API reference (per-symbol signatures and short doc comments)
lives on [docs.rs](https://docs.rs/snora).

The hosted, searchable version of these pages is published from this
directory at **<https://nabbisen.github.io/snora/>**. If you are
reading this on GitHub, every link below also resolves to the source
Markdown.

Two top-level files complement these docs:
[CHANGELOG](../CHANGELOG.md) records what changed in each release, and
[ROADMAP](../ROADMAP.md) sketches what is expected next.

## I am new to snora

Read these in order. Each is short and self-contained.

1. [Install](src/getting-started/01-install.md) — add snora to your `Cargo.toml`.
2. [Hello, snora](src/getting-started/02-hello-world.md) — the smallest working app.
3. [Add a header, sidebar, footer](src/getting-started/03-add-a-header.md) — fill the skeleton.
4. [Toasts](src/getting-started/04-toasts.md) — notifications with framework-managed lifetime.
5. [When to use snora](src/getting-started/05-when-to-use.md) — fit and non-fit guidance.

## I have used snora before

Pick the topic you need.

- [Overlays — dialogs and sheets](src/guides/overlays.md)
- [Header and context menus](src/guides/menus.md)
- [Direction and ABDD](src/guides/direction.md)
- [Icons — text, Lucide, SVG](src/guides/icons.md)
- [Testing UI logic without a renderer](src/guides/testing.md)
- [Migrating from 0.6 to 0.7](src/guides/migration-0.6-to-0.7.md)
- [Migrating from 0.5 to 0.6](src/guides/migration-0.5-to-0.6.md)
- [Migrating from 0.4 to 0.5](src/guides/migration-0.4-to-0.5.md)

## I want to look up a specific symbol or layout

- API reference (per-symbol): [docs.rs/snora](https://docs.rs/snora) /
  [docs.rs/snora-core](https://docs.rs/snora-core) /
  [docs.rs/snora-widgets](https://docs.rs/snora-widgets)
- [Architecture overview](src/reference/architecture.md) — what each crate contributes
- [Vocabulary cheatsheet](src/reference/vocabulary.md) — every public enum at a glance
- [Built-in widgets](src/reference/widgets.md) — the prefab `app_header` / `app_side_bar` / etc.

## I want to contribute

Welcome — see the contributor docs:

- [Internal architecture](src/contributing/architecture.md)
- [Design decisions](src/contributing/design-decisions.md) — why the API looks the way it does
- [Adding a new overlay kind](src/contributing/adding-an-overlay.md)
- [Feature-gating criteria](src/contributing/feature-gating-criteria.md) — when to split the `widgets` feature
- [Release process](src/contributing/release-process.md)

## Building this documentation locally

These pages are also published as an [mdBook](https://rust-lang.github.io/mdBook/).
To preview locally:

```text
cargo install mdbook --no-default-features --features search --locked
mdbook serve docs --open
```

The `docs/book/` output directory is git-ignored; the
[Docs CI workflow](../.github/workflows/docs.yaml) regenerates and
deploys it on every push to `main`.
