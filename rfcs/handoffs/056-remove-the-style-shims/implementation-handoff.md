# Developer Handoff — RFC-056 remove the style shims

**Governing RFC.** [RFC-056](../../proposed/056-remove-the-style-shims.md)
**Status.** Inherited from RFC-056 — Accepted (owner, 2026-08-15).
**Release target.** 0.33.0 (minor — a public path is removed from
`snora-widgets`).
**Implementation units.** One.

---

## 1. Task title

Delete `snora_widgets::design::{style, theme}`, re-pointing snora-widgets' own
four consumers at `snora-style` first.

## 2. Purpose

RFC-055 kept those two paths as compatibility re-exports so nothing broke
while the style layer moved to `snora-style`. Its precondition for retiring
them is met: `snora::design::style` and `::theme` now point at `snora_style`
directly, so removing the shims does not affect the documented consumer route.

Removal rather than deprecation, per RFC-056: `#[deprecated]` on a `pub use`
emits no warning at all, the audience is hypothetical either way, and a
compile error serves a hypothetical user better than a warning they may not
read.

## 3. Order — this is the whole task

**Re-point first, remove second.** Removing the module while snora-widgets
still imports it breaks our own build, and CI runs `clippy -D warnings`.

Found by experiment: deprecating the module produced **five** warnings from
snora-widgets itself.

## 4. The four consumers, and the one that will not be fixed by changing the import

```text
crates/snora-widgets/src/design/chip.rs:54     use super::style;
crates/snora-widgets/src/design/notice.rs:41   use super::style;
crates/snora-widgets/src/design/card.rs:41     use super::style;
crates/snora-widgets/src/design/button.rs:48   use super::style;
```

Call sites read `style::container::card_raised`, `style::button::primary`, and
so on. **`snora-style` exposes exactly those modules at its crate root**
(`color`, `button`, `container`, `text`, `progress`, `theme`), so the minimal
change is:

```rust,ignore
use snora_style as style;      // was: use super::style;
```

Every `style::…` call site then resolves identically.

**One call site is fully qualified and will not be caught by that:**

```text
crates/snora-widgets/src/design/button.rs:59
    button(text(label.into()).size(super::style::text::label_size(tokens)))
```

`super::style::…`, written out. Changing the import does nothing for it.
**Grep the file bodies for `super::style`, not just the import lines** — this
is the third time in this project that a fully-qualified call has hidden from
an import scan (RFC-054's `card_raised`, RFC-055's `theme`).

## 5. What to remove

| Item | Where |
|---|---|
| `pub mod style;` | `crates/snora-widgets/src/design.rs` |
| the shim file itself | `crates/snora-widgets/src/design/style.rs` |
| `pub use snora_style::theme;` | `crates/snora-widgets/src/design.rs` |

`snora-widgets` keeps `snora-style` as a dependency — its widgets still use
the style functions, now via `snora_style` directly.

## 6. Also required

**A migration guide** — `docs/src/guides/migration-0.32-to-0.33.md`, registered
in `SUMMARY.md` and the migrations index. Short: a two-row old→new table plus
a sentence that `snora::design::*` consumers are unaffected. This is the first
guide since 0.29.0 and the release breaks something, so the convention applies.

**Correct the feature comment.** `crates/snora-widgets/Cargo.toml`'s `design`
feature says it "exposes the Snora Design style bridge". After this it gates
the prefab design *widgets* only. Fix the comment; **do not change the
feature.**

## 7. Explicit non-change scope

Do **not**:

- **Change `snora::design::*`.** That is the documented consumer route and
  must resolve exactly as today.
- **Remove anything from `snora-style`.**
- **Add a deprecation.** Removal is the alternative to one, not a step after
  it — see RFC-056 §"Why removal, not deprecation".
- **Change `snora-widgets`' widget surface** — `widget`, `button`, `card`,
  `notice`, `chip`, `progress` stay.
- **Change the `design` feature's contents**, only its comment.
- Modify `render_semantics.rs`.

## 8. Required tests

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora-widgets --all-features
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features
cargo check -p snora --no-default-features --features design
cargo check --workspace --all-features
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

## 9. Acceptance criteria

RFC-056 §Acceptance criteria 1–6. The one most likely to be asserted rather
than shown:

**Criterion 3** — that a design-path consumer sees no difference. Demonstrate
it: a probe using `snora::design::style::container::card_raised` and
`snora::design::theme` must still compile. Show the probe and its build, then
remove it.

Symmetrically, show that the removed path is *gone* — a probe referencing
`snora_widgets::design::style` must fail to compile, with the error. That is
the same technique RFC-053's absence check used, and it is stronger than
reporting that a build passed.

## 10. Prohibited shortcuts

- Do not remove before re-pointing (§3).
- Do not rely on the import lines to find every use (§4).
- Do not skip the migration guide because "nothing consumers use is broken" —
  that is the claim the guide exists to make explicit.
- Do not modify `render_semantics.rs`.

## 11. Compatibility and security

**Compatibility.** Breaking for direct `snora-widgets` consumers only, of
which none are known. `snora` consumers are unaffected.

**Security.** None.

## 12. Required evidence

- Diffs of the four re-pointed modules, including `button.rs:59`.
- The removal diff.
- **Both probes from §9** — the one that must still compile, and the one that
  must not, with its error.
- Clippy output at `-D warnings`.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it empty.
- The migration guide.

## 13. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under `.git-exclude/review-request/056-remove-the-style-shims/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** §4 — whether every use of the shim was found,
including fully-qualified ones. The removal is mechanical; missing a call site
turns it into a build failure discovered at release time.
