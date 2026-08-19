# scripts/

Repository-level utility scripts. Each script is self-documenting in
its header comment; this README is a brief inventory.

| Script | Purpose | Invoked by |
|---|---|---|
| [`measure-binary-size.sh`](measure-binary-size.sh) | Build three size-probe binaries (`snora-size-probe-engine`, `snora-size-probe-widgets`, `snora-size-probe-design`), strip each, emit a single 9-field CSV row. Measures the marginal binary cost of `widgets` and `design` in isolation. | The `binary-size` GitHub Actions workflow on every push and tag. |
| [`append-binary-size-row.sh`](append-binary-size-row.sh) | Append a single measurement row to `docs/src/reference/binary-size-budget/binary-size.csv`. Validates 9 fields before appending. | The `binary-size` workflow, only on release tag pushes. |
| [`measure-compile-time.sh`](measure-compile-time.sh) | Measure six cold-build durations (engine-only check, widgets build, engine-only build, hello example, widgets+design build, design-workbench example), emit a single 11-field CSV row. | The `build-cost` GitHub Actions workflow on pushes and tags. |
| [`measure-render-cost.sh`](measure-render-cost.sh) | Local performance envelope reference; not invoked by CI. | Manual: `scripts/measure-render-cost.sh` |
| [`check-built-links.py`](check-built-links.py) | Confirms every internal link in the built `docs/book/` HTML resolves to a file that exists — catches a page that exists on disk but was never wired into `SUMMARY.md` (RFC-073), which a source-level `.md`-link check cannot see. | Manual: `mdbook build docs && python3 scripts/check-built-links.py` |

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
