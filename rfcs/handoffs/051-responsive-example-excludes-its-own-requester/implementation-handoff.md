# Developer Handoff — RFC-051 engine-only responsive example

**Governing RFC.** [RFC-051](../../done/051-responsive-example-excludes-its-own-requester.md)
**Status.** Inherited from RFC-051 — Implemented (v0.30.0).
**Release target.** 0.30.0 (minor — new workspace member).
**Implementation units.** One. Independent of RFC-050; either may land first.

---

## 1. Task title

Add a responsive example that uses no `snora::widget::*` and no `side_bar`,
varying `body`'s own composition by width — the adoption pattern both known
consumers actually have.

## 2. Purpose

`responsive_render` shipped in v0.28.0 because **apimokka asked for it**.
Every demonstration snora provides teaches it through `AppLayout::side_bar`,
built with `snora::widget::app_side_bar`. apimokka uses **neither** — zero
`snora::widget::*` call sites, and `side_bar` is on their ignored list.

The feature's own requester cannot copy the example that teaches it.

They are now sequencing the UX work that produces breakpoint thresholds, and
**those thresholds are the evidence deciding whether snora ever ships
breakpoint behaviour** (RFC-046's deliberate deferral). They will generate it
against whatever example they work from.

## 3. Background — read first

- `rfcs/done/051-responsive-example-excludes-its-own-requester.md` in full.
- `examples/responsive/src/main.rs` — what you are *complementing*, not
  replacing.
- `examples/dialog/` or `examples/hello/` — existing engine-only examples, for
  manifest and structure shape.
- `docs/src/guides/responsive.md` — gains two sentences (§7).

Conventions: English only. `cargo fmt --all --check` is enforced in CI as of
0.28.1.

## 4. The manifest trap — verified, do not re-derive

The example must be **compiler-enforced** engine-only, not engine-only by
discipline. That means `default-features = false` on the `snora` dependency.

**`snora = { workspace = true, default-features = false }` does not work.**
Tested against this workspace:

```text
error inheriting `snora` from workspace root manifest's
`workspace.dependencies.snora`
Caused by:
  `default-features = false` cannot override workspace's `default-features`
```

So use the explicit form, exactly as `examples/size_probe_engine` does:

```toml
[dependencies]
snora = { path = "../../crates/snora", version = "0.30", default-features = false }
iced  = { workspace = true }
```

**This pin does not follow the workspace table and must be hand-edited on every
minor bump.** Missing it fails *every* `cargo` command in the workspace with
`failed to select a version for the requirement snora = "^0.NN"`. That has
already happened once, on the 0.26.0 bump.

You are adding the **second** such example, so:

- Update the comment in the root `Cargo.toml` (currently at the
  `workspace.dependencies` block) to name **both** examples, not just
  `size_probe_engine`.
- Add a line to `docs/src/contributing/release-process.md`'s version-bump
  section naming both. **The checklist currently does not mention this at
  all** — the warning lives only in the `Cargo.toml` comment, which is the
  wrong place to find it while following a release checklist.

No workspace `members` edit is needed: the root manifest globs `examples/*`.

## 5. Change scope

| File | Purpose |
|---|---|
| `examples/<name>/Cargo.toml` (new) | manifest — see §4 |
| `examples/<name>/src/main.rs` (new) | the example |
| `examples/README.md` | a row in the examples table |
| `docs/src/guides/responsive.md` | two sentences naming both examples |
| `Cargo.toml` | comment: name both hand-pinned examples (§4) |
| `docs/src/contributing/release-process.md` | checklist line (§4) |
| `CHANGELOG.md` | `[Unreleased]` **Added** |

Name at your discretion; `responsive_body` describes it.

## 6. Required implementation

### Step 1 — Decide the pattern, and say why (RFC-051 Q-1)

Two candidates. **Pick one; do not build both.**

- **A horizontal tab bar** composed into `body`, becoming compact below a
  threshold. Mirrors apimokka's actual layout — that is the argument for it.
- **A two-column body that becomes one column.** More generic; less likely to
  read as "snora is about tab bars".

My lean is the tab bar, because the point of the example is to match a real
adoption pattern rather than to be maximally generic — but the decision is
yours and the reasoning goes in the review request.

### Step 2 — The example

Hard constraints, inherited from RFC-046:

- **The example picks its own threshold**, in a named constant, with a comment
  saying the number is the application's decision and snora prescribes none.
  A reader must not come away thinking it is a snora default.
- **Correct under both `LayoutDirection` values.** Chrome composed into `body`
  is still direction-sensitive. Demonstrate it — a runtime toggle, or at
  minimum build the layout through `LayoutDirection` rather than hardcoding
  left/right.
- **No `snora::widget::*`, no `snora::design::widget::*`, no `side_bar`, no
  `footer`.** Build the chrome from `iced` widgets composed into `body`.
- Runnable and readable. This is documentation that compiles.

### Step 3 — Guide

`docs/src/guides/responsive.md` gains a short paragraph naming both examples
and who each is for: slot-based chrome (`examples/responsive`) versus chrome
composed into `body` (yours).

**Two sentences, not a section** (RFC-051 Q-2). A reader landing on the guide
should learn that two patterns exist without the page growing a taxonomy.

### Step 4 — Examples table

`examples/README.md` has a one-row-per-example table. Match its column shape.
State plainly that this one uses no prefab widgets — that is the reason it
exists and the reason a reader would choose it.

## 7. Explicit non-change scope

Do **not**:

- **Modify `examples/responsive`.** It stays as-is. Sidebar collapse is a fair
  archetype.
- **Add a `Breakpoint` type, thresholds, or adaptive behaviour to snora.**
  Unchanged from RFC-046. This task exists partly to protect that deferral.
- **Change `responsive_render` or any library code.** No `crates/` edits at
  all beyond none.
- **Add a third example.** Two patterns is the point.
- **Add a dependency** beyond `snora` and `iced`.
- Claim prefab widgets are wrong. They have no demonstrated downstream
  adoption, which is a different statement.

## 8. Required tests

```bash
cargo build -p snora-example-<name>
cargo run  -p snora-example-<name>          # look at it, both directions
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo test -p snora --test render_semantics   # MUST pass unmodified
mdbook build docs && mdbook test docs
```

**The engine-only property must be demonstrated, not asserted** (RFC-051
AC-2). The manifest's `default-features = false` makes the compiler enforce
it — so a successful `cargo build -p snora-example-<name>` *is* the proof,
provided the manifest is right. Show both the manifest and the build output.

Additionally confirm the workspace still resolves: after adding a
hand-pinned dependency, run a bare `cargo check --workspace --all-features`
and confirm it does not fail version selection (§4).

## 9. Acceptance criteria

RFC-051 §Acceptance criteria 1–7:

1. New engine-only example exists as a workspace member; no `snora::widget::*`,
   no `side_bar`/`footer`.
2. Compiler-enforced engine-only via `default-features = false`, with the build
   shown.
3. Varies `body` composition by width; own threshold in a named constant;
   comment states it is the application's choice.
4. Correct under both `LayoutDirection` values.
5. `guides/responsive.md` names both examples and who each is for.
6. `examples/responsive` unchanged.
7. All gates in §8 pass; `render_semantics` unmodified.

Plus, from §4: root `Cargo.toml` comment and `release-process.md` both name
**both** hand-pinned examples.

## 10. Prohibited shortcuts

- Do not use `workspace = true` and simply avoid widgets by discipline. The
  compiler must enforce it, or the property decays on the first edit.
- Do not skip the `Cargo.toml` comment and checklist updates (§4). Adding a
  second hand-pinned example without recording it is how the 0.26.0 breakage
  recurs, and next time with two candidates to find.
- Do not "improve" `examples/responsive` while you are in there.
- Do not hardcode a threshold without the comment explaining whose decision it
  is. That is the single most likely thing for a reader to copy wrongly.

## 11. Compatibility and security

**Compatibility.** Purely additive — a new example crate and two sentences.
No library API, no rendering change, nothing existing modified except comments
and docs.

**Security.** No new dependency or data flow.

## 12. Required evidence

- The new example's `Cargo.toml` and `main.rs` in full.
- Build output showing it compiles with `default-features = false`.
- A screenshot or description of both width states, and of both
  `LayoutDirection` values.
- `cargo check --workspace --all-features` output, confirming the new pin did
  not break version resolution.
- Diffs of `Cargo.toml`, `release-process.md`, `examples/README.md`,
  `guides/responsive.md`.
- Your Q-1 decision and reasoning.

## 13. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/051-responsive-example-excludes-its-own-requester/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** whether the example is genuinely copyable by a
consumer who has no prefab widgets and no `side_bar` — read it as if you were
apimokka. If any line requires translating before it helps them, it has not
done its job.
