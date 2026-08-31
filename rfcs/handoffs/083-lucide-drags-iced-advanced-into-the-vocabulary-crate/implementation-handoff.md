# Developer Handoff — RFC-083 the lucide feature chain

**Governing RFC.** [RFC-083](../../done/083-lucide-drags-iced-advanced-into-the-vocabulary-crate.md)
**Status.** Inherited from RFC-083 — Accepted (owner, 2026-08-21).
**Release target.** 0.40.0 — **minor.** It changes which features a consumer's
build resolves.
**Implementation units.** Three.

---

## 1. Purpose

docs.rs cannot build the published `snora-core` 0.39.3. One workspace line is
the cause, and it does three kinds of damage — see the RFC. **Verify the chain
yourself before changing it**, because the fix is one line and the reasoning is
the whole content:

```
cargo tree -p snora-core --all-features        # expect: → lucide-icons → iced → …
cargo tree -p snora-core --no-default-features # expect: no dependencies at all
cargo tree -p snora --all-features -e features | grep 'advanced'
```

## 2. Unit 1 — the one line

```toml
lucide-icons = { version = "1", default-features = false }
```

**Do not instead add a per-crate override in `snora-core`.** The feature is
unused by *every* member — `snora-core` uses `lucide_icons::Icon`,
`snora-widgets` uses that and `LUCIDE_FONT_BYTES`, and neither is gated on
lucide's `iced` feature. Fixing it in one crate would leave the same defect in
the others and split one fact across five manifests.

**Read `crates/snora-widgets/src/icon.rs:28-29` before you touch this.** It
tells us not to call the method lucide's `iced` feature exists to provide. If
that comment has changed, stop — the premise has moved.

Expected after, all measured on the working tree already:

| check | expected |
|---|---|
| `cargo check --workspace --all-features` | passes |
| `cargo test --workspace --all-features` | 34 `ok` result lines |
| `clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| `iced feature "advanced"` in any feature tree | **0 occurrences** |
| `cargo tree -p snora-core --all-features` | `lucide-icons` only |

**These are a cross-check, not a source.** If yours differ, yours win.

## 3. Unit 2 — the migration guide (Q-1 ruled: treat it as breaking)

0.40.0 is a minor and gets a guide regardless (RFC-079). This one has content:

**Name the reliance that can break.** A consumer whose own code uses
`iced::advanced::` and got that feature through snora's `lucide-icons` edge will
stop compiling. It was never documented and nobody has said they do it — **say
it anyway.** "Undocumented and unlikely" is not "cannot have happened", and the
fix is one line in their manifest: enable `iced`'s `advanced` feature
themselves.

Also state the good news plainly, because it is the reason for the change:
`snora-core` regains zero dependencies under every feature combination, and
`advanced` is no longer enabled behind anyone's back.

## 4. Unit 3 — the governance statement

`docs/src/contributing/design-decisions.md` says `advanced` is **"not
stable-by-default"**. That was true and read as *"snora never enables it"*,
which was false for every `lucide-icons` user.

Correct it to say what is now true — snora enables `advanced` **nowhere**, in no
feature combination — and **record that it was not always so**, with the version
it became true. A governance claim that quietly starts being accurate teaches a
reader nothing about how much to trust the next one.

## 5. Q-2 — ruled: add the gate, and prove it fires

`snora-design`'s iced-free property is CI-enforced (RFC-021/022 Q3).
`snora-core`'s equivalent is documented and unguarded, **which is exactly why
this stood until docs.rs found it.**

Add the same-shaped check for `snora-core`: **no iced in its dependency tree, in
any feature combination.** Match the existing gate's mechanism rather than
inventing a second style — find it first and say which you followed.

**A perturbation demo is required**, not a green run: re-add
`features = ["iced"]`, watch the gate fail, restore. A gate that has never
failed is not known to work, and the one we did not have is why this shipped.

## 6. Explicit non-change scope

- **No per-crate override** (§2).
- **Do not un-archive RFC-078.** Q-3: the ruling stands and this makes it true
  rather than nearly-true. Record in the archived file only that lucide users
  had been paying the `advanced` cost, so some data already exists if the
  opt-in feature is ever built.
- **No change to what `lucide-icons` provides consumers** — `Icon` and
  `LUCIDE_FONT_BYTES` are unaffected; only the unused iced integration goes.
- **No new dependency anywhere.**

## 7. Required evidence

- The three `cargo tree` outputs, before and after
- Full gate suite, and the four cross-checks in §2 with your own numbers
- The perturbation demo for the new gate, with the restore
- **docs.rs is not verifiable pre-release** — say so rather than claiming it.
  It is checked after publication, and acceptance criterion 3 belongs to the
  release, not to this package.

## 8. Acceptance criteria

1. `cargo tree -p snora-core --all-features` shows no iced.
2. `iced feature "advanced"` appears in no feature tree.
3. Migration guide names the `iced::advanced::` reliance and its one-line fix.
4. `design-decisions.md` says `advanced` is enabled nowhere, and records that
   this became true at 0.40.0.
5. The `snora-core` gate exists, matches the existing mechanism, and its
   perturbation demo is captured.
6. RFC-078 stays archived; the note added, nothing else.

## 9. Required review-request format

`.git-exclude/review-request/083-lucide-drags-iced-advanced-into-the-vocabulary-crate/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: the gate and its perturbation.** The one-line fix is
already measured. Whether the property stays fixed after everyone forgets why
is the part worth the release.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
