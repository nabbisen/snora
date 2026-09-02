# scripts/

Repository-level utility scripts. Each script is self-documenting in
its header comment; this README is a brief inventory.

| Script | Purpose | Invoked by |
|---|---|---|
| [`measure-binary-size.sh`](measure-binary-size.sh) | Build three size-probe binaries (`snora-size-probe-engine`, `snora-size-probe-widgets`, `snora-size-probe-design`), strip each, emit a single 9-field CSV row. Measures the marginal binary cost of `widgets` and `design` in isolation. | The `binary-size` GitHub Actions workflow on every push and tag. |
| [`append-binary-size-row.sh`](append-binary-size-row.sh) | Append a single measurement row to `docs/src/reference/binary-size-budget/binary-size.csv`. Validates 9 fields before appending. | The `binary-size` workflow, only on release tag pushes. |
| [`measure-compile-time.sh`](measure-compile-time.sh) | Measure six cold-build durations (engine-only check, widgets build, engine-only build, hello example, widgets+design build, design-workbench example), emit a single 11-field CSV row. | The `build-cost` GitHub Actions workflow on pushes and tags. |
| [`measure-render-cost.sh`](measure-render-cost.sh) | Local performance envelope reference; not invoked by CI. | Manual: `scripts/measure-render-cost.sh` |
| [`check-repo-links.py`](check-repo-links.py) | Every relative Markdown link in the repository resolves — including `rfcs/`, which `check-built-links.py` cannot see because it reads the built book. Fourteen handoff back-links had gone stale there unnoticed (2026-09-01). | Manual: `python3 scripts/check-repo-links.py` |
| [`check-built-links.py`](check-built-links.py) | Confirms every internal link in the built `docs/book/` HTML resolves to a file that exists — catches a page that exists on disk but was never wired into `SUMMARY.md` (RFC-073), which a source-level `.md`-link check cannot see. | The `docs` GitHub Actions job, after `mdbook build` (RFC-087). Its condition (passes on a clean tree) held for several releases before being gated — RFC-073, RFC-074, RFC-079. |
| [`check-version-snippets.sh`](check-version-snippets.sh) | Finds every `snora`/`snora-*` version-bearing Cargo snippet under `docs/` and crate doc comments and reports any whose minor doesn't match `[workspace.package].version` in `Cargo.toml` (RFC-074) — replaces a release-checklist line that named two files by hand while every other snippet drifted. | The `docs` GitHub Actions job (RFC-087); also part of the release checklist. |
| [`check-migration-guides.sh`](check-migration-guides.sh) | Derives every consecutive minor pair from `git tag` and confirms `docs/src/guides/migration-X.Y-to-X.Z.md` exists for each — reports every gap found, but exits non-zero only for gaps at or after the one `ADOPTION_MINOR` constant (RFC-079), so known pre-adoption gaps don't fail a check nobody expects to be clean yet. | The `docs` GitHub Actions job (RFC-087); also part of the release checklist. |
| [`check-workspace-iced-features.sh`](check-workspace-iced-features.sh) | Every feature the workspace's own `iced` dependency line turns on must be used somewhere, or it forces build cost on every consumer for nothing (RFC-088) — the general property RFC-083's iced-free gates don't check. One feature (`tokio`) is a named, commented exemption: it supplies iced's async executor, a structural requirement of `snora::toast::subscription` that no source-level grep can see. | The `design-isolation` GitHub Actions job (RFC-088). |
| [`check-tag-matches-version.sh`](check-tag-matches-version.sh) | Refuses unless the given tag matches `[workspace.package].version` in `Cargo.toml` exactly, printing both values (RFC-090). One of `release.yaml`'s three pre-upload refusals. | The `release` GitHub Actions workflow, on every tag push, before `cargo publish`. |
| [`check-commit-ci-green.sh`](check-commit-ci-green.sh) | Refuses to publish unless the given commit has an existing, completed, successful run of the CI workflow — the *existing* run, never a fresh re-run (RFC-090 Q-5): re-running is slow and can pass on a commit whose earlier run failed for a reason the re-run doesn't reproduce. Fails closed when no run is found at all, rather than treating "nothing said no" as a pass — the exact gap RFC-087 D-1 found in the migration-guide gate. | The `release` GitHub Actions workflow, on every tag push, before `cargo publish`. |

Together `measure-binary-size.sh` and `measure-compile-time.sh` implement
the [binary size budget][bin-budget] and [build cost budget][build-budget]
documented in the snora docs.

[bin-budget]: ../docs/src/reference/binary-size-budget.md
[build-budget]: ../docs/src/reference/build-cost-budget.md

## Conventions

- Scripts are POSIX `bash`, run with `set -euo pipefail` — **except
  `check-built-links.py`**, which parses HTML/URLs and is Python for
  that reason alone; it follows every other convention below.
- All paths are resolved relative to the workspace root.
- Output destined for downstream pipelines goes to stdout; logging goes to stderr.
- No script writes outside of `target/` and the explicit data files it owns.
