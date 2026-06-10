# RFC-014-E — Examples Acceptance Matrix

**Status.** Implemented (v0.14.0)
**Tracks.** Examples / QA / release acceptance.
**Touches.** `examples/README.md` (new),
`.github/workflows/ci.yaml` (examples step added),
`docs/src/contributing/release-process.md` (checklist item).

> Workbench dependency is satisfied: RFC-012-B shipped in v0.12.

## 1. [Decision] Map RFC matrix to actual example crate names

The planning RFC used generic names. The actual tree uses workspace-member
crates. The matrix is adapted to match reality, plus one gap addressed.

### Gap: no dedicated icons example

The RFC matrix includes an `icons` entry. No example exercises
`lucide-icons` or `svg-icons`. Creating one would require the `lucide-icons`
feature, adding a non-default dependency to the workspace check unless
scoped carefully. **Decision: omit the icons example from the v0.14
acceptance matrix**; document the gap and add it when the workbench
grows icon-feature usage or when a dedicated small example is added.
The existing icon guide (`docs/src/guides/icons.md`) covers the feature-
gating aspects as documentation.

## 2. Acceptance matrix (actual crate names)

| Crate | Purpose | Feature set | Demonstrates |
|---|---|---|---|
| `snora-example-hello` | Minimal app | default | Smallest working app |
| `snora-example-skeleton` | Header+sidebar+footer | default+widgets | Slot injection, prefab chrome |
| `snora-example-header-menu` + `snora-example-context-menu` | Menus | default+widgets | Header/context menu, close sink, backdrop |
| `snora-example-dialog` + `snora-example-sheet` | Modal overlays | default+widgets | Modal dim, outside-close, sheet edges |
| `snora-example-toast` | Toast lifecycle | default | Transient/persistent toasts, 6 positions |
| `snora-example-rtl` | ABDD/direction | default+widgets | LTR/RTL mirroring |
| `snora-example-workbench` | Integrated dogfood | default+widgets | All major surfaces + Escape wiring |

## 3. CI step

Add to the `rust-quality` job in `ci.yaml`:

```yaml
- name: Check examples (all-features)
  run: cargo check --workspace --all-features
```

This step already runs — workspace check covers all members. What the
RFC adds is making the *intent* explicit in a dedicated CI job name or
comment, and the `examples/README.md` documenting the matrix.

The `feature-matrix` job already covers `snora` crate feature combinations.
No additional per-example feature-matrix job is needed; the workspace check
at `--all-features` is the gate.

## 4. examples/README.md

New file at `examples/README.md` with:
- the acceptance matrix table (crate names, purpose, surface covered);
- how to run any example (`cargo run -p <crate-name>`);
- the manual QA checklist for the workbench;
- the "icons gap" note.

## 5. Release checklist addition

Add to `release-process.md`:

```text
[ ] cargo check --workspace --all-features passes (covers all examples).
[ ] Workbench manual QA checklist completed (docs/src/getting-started/06-workbench.md).
```

## 6. ABDD check

`examples/README.md` is not direction-sensitive. ABDD does not apply.

## 7. Acceptance criteria

- `examples/README.md` exists with the acceptance matrix.
- Release checklist references examples.
- `workbench` is the workbench manual QA reference.
- Icons gap is documented.
