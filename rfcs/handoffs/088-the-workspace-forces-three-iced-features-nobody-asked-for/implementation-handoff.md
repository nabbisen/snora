# Developer Handoff — RFC-088 workspace iced features

**Governing RFC.** **RFC-088** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Accepted (owner, 2026-09-01). High.
**Release target.** 0.42.0 — **minor.** Changes what a consumer's build
resolves.
**Implementation units.** Two.

---

## 1. Verify the usage yourself first

`Cargo.toml:55` — `iced = { version = "0.14", features = ["canvas", "svg", "tokio"] }`.

Architect's findings, to confirm not inherit:

| feature | claim |
|---|---|
| `canvas` | **zero occurrences** in `crates/` |
| `svg` | used only at `snora-widgets/src/icon.rs:45`, **under our own `svg-icons` feature — which already declares `"iced/svg"`** at `crates/snora/Cargo.toml:59` |
| `tokio` | **no direct use** |

**If any of those has changed, stop and report.** This is RFC-083 one line
later in the same file, and that RFC was safe only because its premise was
re-checked before the edit.

## 2. Unit 1 — remove what is unused

`canvas` goes. `svg` goes **from the workspace line only** — confirm
`svg-icons` consumers still get it via `crates/snora/Cargo.toml:59`, by building
with and without that feature.

**Q-1 ruled: `tokio` is measured, not assumed.** iced needs *an* executor;
`tokio` is one choice, forced on every consumer. **Build the workspace and run
the examples without it.** If something needs it, gate it behind a snora feature
rather than forcing it — and if it cannot be gated, say why rather than
restoring it silently.

## 3. Unit 2 — the numbers are the deliverable

**Re-derive the audit's −1.83 MiB / −34 crates.** Before and after:

```
cargo tree --workspace --all-features | wc -l
```

plus the binary-size probe the release process already uses. **State both
figures as your own.** A dependency reduction nobody measured is a claim.

## 4. Q-2 ruled — treat it as breaking

A consumer relying on `iced/canvas` or `iced/tokio` arriving transitively stops
compiling. **Identical shape to RFC-083, one release ago, which was treated as
breaking on the same reasoning:** undocumented and unlikely is not cannot have
happened. Migration guide names the reliance and the one-line fix.

## 5. Q-3 ruled — add the general gate

RFC-083's gate asserts *a crate has no iced*. It says nothing about **features
iced is asked for that nothing uses** — which is why this survived RFC-083 by
one line.

**Add a check: for each feature in the workspace `iced` declaration, the
workspace uses it.** Derivable by grep, same committed-runnable shape as the
existing scripts — and per RFC-087 it is now wired into CI rather than manual.

**A perturbation demo is required:** re-add `canvas`, watch it fail, restore.

## 6. Explicit non-change scope

- **Do not touch line 56.** RFC-083 settled `lucide-icons`.
- **No behaviour change** for `svg-icons` consumers.
- **No new snora feature** unless Q-1 forces one for `tokio`, in which case say
  so before adding it.

## 7. Required evidence

- Usage verification for all three features
- Before/after dependency count and binary size, your own numbers
- `svg-icons` on and off, both building
- The gate's perturbation demo with its restore
- `cargo test --workspace --all-features` and the examples unaffected

## 8. Acceptance criteria

1. `canvas` removed; `svg` removed from the workspace line with `svg-icons`
   consumers verified unaffected; `tokio` resolved per Q-1 with a measurement.
2. Dependency and size deltas stated as your own.
3. Migration guide names the breaking reliance.
4. Q-3's gate exists, is in CI, and its perturbation is captured.
5. Line 56 untouched.

## 9. Required review-request format

`.git-exclude/review-request/088-the-workspace-forces-three-iced-features-nobody-asked-for/`,
`README.md` entry point, evidence under `evidence/`.

**Requested review focus: the gate.** The removals are three words. That the
class cannot recur a third time on a third line is the release's value.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
