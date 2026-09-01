# Developer Handoff — RFC-063 derived contrast pairs

**Governing RFC.** **RFC-063** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-063 — Accepted (owner, 2026-08-18).
**Release target.** 0.36.0, alongside RFC-061 and RFC-062.
**Implementation units.** One.

---

## 1. Task title

Make `Palette` roles declare their intended surfaces and threshold class under
compiler enforcement, and derive `mandatory_pairs` from that declaration instead
of maintaining it by hand.

## 2. Purpose

RFC-058 added seven contrast assertions. **All of them are hand-written entries
in a hand-maintained list.** tekstide's diagnosis:

> The list did not fail because it was short; it failed because **nothing about
> adding a role forces anyone to measure it.**

RFC-058 fixed two instances. This closes the class.

## 3. What is already enforced — do not rebuild it

Probed by adding a nineteenth field to `Palette` and compiling (2026-08-18,
files restored, tree verified clean):

| Mechanism | Adding a role… |
|---|---|
| the four preset initializers | **fails to compile** — `E0063: missing field` ×4 |
| `Palette::roles()` | compiles; role silently omitted |
| `mandatory_pairs` | compiles; role silently unasserted |

**The compiler already forces a *value* in all four presets.** That half works.
What nothing forces is declaring **where the role renders**, and therefore what
it must be measured against.

Your job is the second half only. Do not add machinery for the first.

## 4. The mechanism, verified — use this one

Exhaustive struct destructuring inside `snora-design` fails to compile when a
field is added, **despite `Palette` being `#[non_exhaustive]`** — that attribute
constrains *other* crates, not the defining one. Captured from the probe:

```text
error[E0027]: pattern does not mention field `probe_role`
  --> crates/snora-design/src/palette.rs:77:13
   | missing field `probe_role`
```

So: a function that destructures `Palette` exhaustively and maps each field to
its intended surfaces and threshold class **cannot be left stale**. The
nineteenth field breaks the build until someone answers *where does this
render?*

Two ways to get this wrong:

- **A `match` on a role enum is not equivalent** unless something ties the enum
  to the struct's fields. `Palette` is a struct of 18 `pub` fields; an enum
  beside it can drift silently, which is the defect you are removing.
- **`..` in the pattern destroys the enforcement entirely.** The compiler
  helpfully suggests it (`"if you don't care about this missing field"`). Do not
  take that suggestion, and say so in a comment so a future reader does not.

## 5. What to build

### 5.1 The declaration — `crates/snora-design/src/palette.rs`

Per Q-2, it lives in `palette.rs`, beside the fields it constrains — that is
where a contributor adding a field will already be. A tidier separate module
reintroduces exactly the distance that caused the problem.

Each role declares:

- **the surfaces it is intended to render on**, and
- **its threshold class** — `AA_TEXT` for body text, `NON_TEXT_MIN` for non-text
  boundaries.

Per Q-3, **make the threshold explicit, not inferred from the name.** A
`*_text` → `AA_TEXT` rule is concise and brittle: `focus` and `border` are both
non-text and share no naming convention, and this RFC exists because something
was implicit.

Keep it crate-private and test-only, matching `roles()`'s existing visibility.
This is not a public API addition.

### 5.2 Intended usage, not the cross-product

tekstide anticipated the wrong fix, and the handoff repeats their warning
because it is the failure mode here:

> `accent_text` on `background` is meaningless — `accent_text` exists to sit on
> `accent` — so 18 × 3 would be mostly noise, and **noise in an accessibility
> gate is how gates get ignored.**

`accent_text` declares `accent`. `text_primary` declares all three surfaces.
`focus` and `border` declare the surfaces they are drawn over. A role that
genuinely renders nowhere measurable declares that explicitly, with a reason —
an empty declaration must be a deliberate statement, not a default.

### 5.3 Derive `mandatory_pairs` from it

The assertions become a consequence of the declaration. **No hand-written pair
list may remain** — if one does, the RFC has not been implemented, only added to.

### 5.4 `Palette::roles()`

Derive it from the same declaration or delete it. It is `#[cfg(test)]` and
`pub(crate)`, so removal costs nothing externally. Deleting is preferable if
nothing needs it once pairs are derived.

### 5.5 Record the rule

- `docs/src/contributing/accessibility-checklist.md` — adding a `Palette` role
  requires declaring its surfaces; the compiler enforces it.
- `docs/src/contributing/api-governance.md` — beside the additive-only covenant,
  since that is where role additions are already governed.

## 6. Q-1 is the part most likely to go wrong

Deriving the list should produce a **superset**: the twelve original pairs,
RFC-058's seven additions, and any pair implied by a declaration that nobody had
written an assertion for.

**Enumerate the before/after pair set explicitly.** Then:

- **A newly-covered pair that passes** — fine, that is the ratchet working.
- **A newly-covered pair that FAILS** — that is a defect this RFC found.
  **Report it. Do not repair it in this change.**

That second case is the one to get right. Repairing it in passing would consume
RFC-036's accessibility carve-out without its failing-first proof, which is
precisely the discipline RFC-058 established and the reason its evidence is
trustworthy. Stop, report the pair and its measured ratio, and let it be scoped
deliberately.

If the derived set is *smaller* than today's anywhere, that is a declaration
error, not a discovery — a pair asserted today must remain asserted.

## 7. Change scope

| File | Purpose |
|---|---|
| `crates/snora-design/src/palette.rs` | the declaration (§5.1) |
| `crates/snora-design/src/tests.rs` | derive `mandatory_pairs` (§5.3) |
| `docs/src/contributing/accessibility-checklist.md` | the rule (§5.5) |
| `docs/src/contributing/api-governance.md` | the rule (§5.5) |
| `CHANGELOG.md` | **Changed** |

## 8. Explicit non-change scope

Do **not**:

- **Add, rename or retype a `Palette` role.** RFC-036's covenant forbids it
  without reopening D-3/D-4.
- **Change any preset value** (§6).
- **Change `AA_TEXT`, `FOCUS_MIN` or `NON_TEXT_MIN`.**
- **Emit the cross-product** (§5.2).
- **Use `..` in the destructuring pattern** (§4).
- **Make the declaration public API.**
- Modify `render_semantics.rs`.

## 9. Required tests

```bash
cargo test -p snora-design
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

**Plus the enforcement probe, which is the acceptance evidence** (§10.1): add a
nineteenth field, compile, capture the error, revert, and prove the tree is
byte-identical afterwards (`git diff --stat -- crates/snora-design/` empty).
This project does not accept a passing test as proof that a guard works — the
guard must be seen to fire.

## 10. Acceptance criteria

RFC-063 §Acceptance criteria 1–6. The two that carry the task:

- **1** — the probe, with the compiler error captured and the revert proven.
  Without it, "the compiler enforces this" is a claim.
- **4** — Q-1's before/after pair set enumerated, and any newly-failing pair
  **reported, not repaired** (§6).

## 11. Required evidence

- The declaration, and the derived `mandatory_pairs`.
- **The enforcement probe**: the added field, the `E0027` error, the revert, and
  `git diff --stat` empty.
- The before/after pair set, as an explicit list, with any additions marked
  pass/fail.
- `roles()` derived or deleted, and which.
- The two documentation diffs.
- `render_semantics` output and `git diff --stat -- crates/snora/tests/` empty.

## 12. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/063-contrast-pairs-are-a-hand-maintained-list/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the before/after pair set, and whether any
declaration is *narrower* than the role's real usage. The failure mode this RFC
must not introduce is a role declaring one surface, deriving one assertion, and
looking rigorous while covering less than the hand-written list did.
