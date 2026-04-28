# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the crate version is below 1.0, breaking changes are signaled by a
**minor** bump (e.g. `0.7.0` → `0.8.0`); patch releases (`0.7.0` → `0.7.1`)
are bug fixes and additive changes only.

This file begins its history at the 0.7.0 release. Earlier release notes
are recorded in the per-version migration guides under
[`docs/guides/`](docs/src/guides/).

## [Unreleased]

Nothing yet.

## [0.8.0] — 2026-04-29

### Added

- **Documentation is now an mdBook.** The `docs/` directory has been
  reorganized into a standard mdBook layout (`docs/book.toml`,
  `docs/src/`, `docs/src/SUMMARY.md`). All long-form documentation is
  authored as before; the new structure adds a searchable, themed,
  hosted view at <https://nabbisen.github.io/snora/>.
  - The Markdown source remains GitHub-readable. Internal cross-links
    use relative paths so both render targets work.
  - `docs/book/` is git-ignored; only the source under `docs/src/` is
    versioned.
- **GitHub Actions docs workflow.** A new
  `.github/workflows/docs.yaml` builds the mdBook on every push to
  `main` and deploys the result to GitHub Pages. The workflow
  status is exposed as a Docs badge in the README.
- **Project-level GitHub conventions.** `.github/` now ships:
  - `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`.
  - Issue templates (`ISSUE_TEMPLATE/{bug_report,feature_request,question}.yml`)
    and `config.yml`.
- **README Quick start now points to runnable examples.** A direct
  link to <https://github.com/nabbisen/snora/tree/main/examples>
  sits alongside the existing pointer to the getting-started chapter,
  so readers who want to skim working code rather than tutorials
  get there in one click.

### Changed

- The `docs/README.md` entry page was rewritten as the entry to the
  full snora documentation (not as an mdBook welcome). It links into
  `docs/src/...` and explains how to read the docs locally
  (`mdbook serve docs --open`) or in CI-published form on GitHub
  Pages.
- `docs/src/contributing/release-process.md` gained an `mdbook build
  docs` step in the release checklist so the book is validated as
  part of every release.

### Tests

- 17 unit tests in `snora-core` (unchanged from 0.7.0).

[Unreleased]: https://github.com/nabbisen/snora/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/nabbisen/snora/releases/tag/v0.8.0
[0.7.0]: https://github.com/nabbisen/snora/releases/tag/v0.7.0

### Removed

- The deprecated 0.6 aliases for the renamed sheet API are gone:
  - `BottomSheet` (use `Sheet`).
  - `SheetHeight` (use `SheetSize`).
  - `AppLayout::bottom_sheet(...)` (use `AppLayout::sheet(...)`).

  Code that compiled cleanly under 0.6 (without `#[deprecated]`
  warnings) compiles unchanged on 0.7.

### Added

- **Tab bar widget.** New navigation primitive for peer-level views.
  - Vocabulary in `snora-core`: `Tab<TabId>`, `TabBar<TabId>`,
    `TabAction<TabId>`.
  - Renderer in `snora-widgets`: `app_tab_bar(bar, on_action, direction)`.
  - Re-exported through `snora` and `snora::widget`.
  - Direction-aware: tab order mirrors under `LayoutDirection::Rtl`.
  - Active tab is rendered with a colored underline drawn from the
    theme's primary palette.
- **Breadcrumb widget.** New navigation primitive for ancestor-level
  navigation.
  - Vocabulary in `snora-core`: `Crumb<CrumbId>`, `BreadcrumbAction<CrumbId>`,
    plus the `Crumb::ancestor(...)` and `Crumb::leaf(...)` constructors.
  - Renderer in `snora-widgets`: `app_breadcrumb(crumbs, on_action, direction)`.
  - Re-exported through `snora` and `snora::widget`.
  - Direction-aware: order mirrors *and* the separator glyph flips
    (`›` under LTR, `‹` under RTL).
- New focused examples: `snora-example-tab` and `snora-example-breadcrumb`.
- New contributor doc:
  [`docs/contributing/feature-gating-criteria.md`](docs/src/contributing/feature-gating-criteria.md)
  records the indicators that would justify splitting the coarse
  `widgets` feature into per-widget gates. The decision for 0.7 is
  to keep the coarse gate; the document captures the criteria for
  revisiting it in future releases.
- New migration guide:
  [`docs/guides/migration-0.6-to-0.7.md`](docs/src/guides/migration-0.6-to-0.7.md).

### Changed

- `docs/contributing/design-decisions.md` gained two sections:
  - "Why `Tab` and `Crumb` are separate vocabulary, not one navigation type."
  - "Why widget feature gating is coarse, not per-widget."
- `docs/contributing/architecture.md` source-layout diagram updated for
  the two new modules (`snora-core/src/{tab.rs, crumb.rs}` and
  `snora-widgets/src/{tab.rs, crumb.rs}`).

### Tests

- 17 unit tests in `snora-core` (12 inherited from 0.6 + 2 tab + 3 crumb).
