# Developer Handoff — RFC-076 the facade's unexported return type

**Governing RFC.** **RFC-076** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-076 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.0 — **minor.** Public API addition.
**Implementation units.** Two: the re-export, then the sweep.

---

## 1. Purpose

`snora::keyboard::cycle_zones` returns `Option<snora_core::focus::Cycle>` and
`snora` does not export `focus`. A consumer on the facade cannot name the return
type of a function the facade provides.

Verified live at the **0.38.3** tag: the re-export list
(`crates/snora/src/lib.rs:84-88`, **22** items) is byte-identical to 0.38.0.

## 2. Q-1 ruled — re-export the module, not the items

```rust
pub use snora_core::focus;
```

`snora::focus::Cycle` names the type. Nothing new lands in `snora`'s root
namespace, and it matches how `keyboard` already appears as a module.

**Do not add the four names to the existing item list.** `next_zone` as a bare
name in the crate root reads worse than `snora::focus::next_zone`, and the four
are a coherent unit.

## 3. Unit 1 — the re-export, plus the docs that route around it

`keyboard.rs`'s own doc comments tell readers to reach past the facade:

- `:82` "([`snora_core::focus::next_zone`])"
- `:88` "bind a different key to [`snora_core::focus::next_zone`] directly"
- `:107` an example calling `snora_core::focus::next_zone(`

**All three become the facade path.** They are how the gap survived: we
documented the workaround instead of noticing it.

**Acceptance is a compiling example that depends only on `snora`** — no
`snora-core` in its `Cargo.toml`. A doctest is the natural home. If it needs
`snora-core` to compile, the fix is not done.

## 4. Unit 2 — Q-2's sweep, and it is the reason this RFC is worth a minor

`cycle_zones` was found because a consumer used it. **Enumerate every public
signature in `snora` and confirm each named type is reachable through `snora`
alone.**

Mechanisable: extract the types appearing in public fn signatures, struct
fields, and trait bounds, and check each resolves under a `snora`-only
dependency. Do it by whatever means is simplest — a scratch crate that depends
solely on `snora` and names each type is a legitimate implementation.

**State the result either way.** "No others found" is a result and must appear
in the report; an absent finding reads as an unperformed check.

## 5. Explicit non-change scope

- **Additive only.** Nothing existing is renamed, removed, or retyped.
- **No new focus API.** This exposes what `snora-core` already has; it does not
  design anything.
- **Do not rule RFC-060 Q-1.** RFC-076 Q-3 observes that arama shipped zone
  navigation without iced's `advanced` feature — a fourth adopter declining
  trapping — and explicitly leaves the ruling to the owner. **It is still
  open. Do not close it, and do not enable the feature.**

## 6. Required evidence

- The `snora`-only compiling example (§3)
- Unit 2's sweep, its method, and its result
- `cargo test --workspace --all-features`, `cargo doc -p snora --no-deps`
- `cargo check -p snora --no-default-features` — the re-export must not depend
  on any feature
- `git diff -- crates/` reviewed: the only functional change is the re-export

## 7. Acceptance criteria

1. `snora::focus::Cycle` names the return type of `snora::keyboard::cycle_zones`.
2. A compiling example proves it **without** a `snora-core` dependency.
3. All three `keyboard.rs` doc references use the facade path.
4. Q-2's sweep run and its result stated, including a null result.
5. Additive only; `--no-default-features` still builds.
6. `CHANGELOG.md` `[Unreleased]` under **Added**, crediting arama.

## 8. Required review-request format

`.git-exclude/review-request/076-the-facade-publishes-a-type-it-does-not-export/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: Unit 2's sweep.** The one-line re-export is trivial;
whether this is the only instance is the question worth the release.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
