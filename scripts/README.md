# scripts/

Repository-level utility scripts.  Each script is self-documenting in
its header comment; this README is a brief inventory.

| Script | Purpose | Invoked by |
|---|---|---|
| [`measure-binary-size.sh`](measure-binary-size.sh) | Build `examples/hello` with and without the `widgets` feature, strip both, emit a single CSV row describing the sizes. | The `binary-size` GitHub Actions workflow on every push and tag. |
| [`append-binary-size-row.sh`](append-binary-size-row.sh) | Append a single measurement row to `docs/src/reference/binary-size-budget/binary-size.csv`. | The `binary-size` workflow, only on release tag pushes. |

Together these implement the [binary size budget][budget] documented
in the snora docs.

[budget]: ../docs/src/reference/binary-size-budget.md

## Conventions

- Scripts are POSIX `bash`, run with `set -euo pipefail`.
- All paths are resolved relative to the workspace root, regardless
  of where the script is invoked from.
- Output destined for downstream pipelines goes to stdout; logging
  goes to stderr.
- No script writes outside of `target/` and the explicit data files
  it owns (e.g. `binary-size.csv`).  Adding new behaviour belongs in
  a new script rather than in extending an existing one.
