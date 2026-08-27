# RFC 083 — One workspace feature breaks docs.rs, pulls iced into the vocabulary crate, and silently enables `advanced`

**Status.** Proposed
**Tracks.** Dependency layering / governance / published-artifact integrity.
**Found by** the owner, 2026-08-21, from a **live docs.rs build failure** on
`snora-core` 0.39.3.
**Touches.** `Cargo.toml` (one line), `docs/src/contributing/design-decisions.md`.
**Release target.** 0.40.0 — **minor.** It changes which features a consumer's
build resolves.

## The failure, and the one line under it

docs.rs builds `--all-features` and fails:

> `error: The platform you're compiling for is not supported by winit`

**Why `snora-core` — the dependency-free vocabulary crate — is compiling winit
at all** is the whole finding. The workspace declares:

```toml
lucide-icons = { version = "1", features = ["iced"] }
```

Every member inherits that. And `lucide-icons`' own manifest declares its `iced`
feature as `iced = ["dep:iced"]` with **`features = ["advanced"]`** on the
dependency. So the chain is:

`snora-core --all-features` → `lucide-icons/iced` → `iced` **with `advanced`
and no default features** → `winit` with no platform backend → `compile_error!`.

**Verified:** `cargo tree -p snora-core --all-features` shows
`snora-core → lucide-icons → iced → …`. With no features, `snora-core` has
**zero dependencies**.

## Three consequences, all live on the published 0.39.3

**1. `snora-core`'s docs.rs page does not build.** A published crate whose
documentation is broken for anyone who looks.

**2. The vocabulary crate is not dependency-free under all features.**
`snora-core`'s entire identity is plain data with no GUI dependency —
`snora-design` has a **CI gate** enforcing the equivalent property. `snora-core`
has no such gate, and does not hold the property.

**3. `iced/advanced` is enabled for every consumer using `lucide-icons`.**
Confirmed by `cargo tree -e features`: `iced feature "advanced"` is present with
`lucide-icons` on, and **absent without it**.

Consequence 3 is the one with governance weight. `design-decisions.md` says
`advanced` is *"not stable-by-default"* — **true, and easily misread.** It is
not a default feature; it *is* silently enabled for anyone who turns on
`lucide-icons`, which is a feature we document and recommend. A reader concludes
snora never enables `advanced`. For lucide users, it always has.

## Nothing in snora uses the feature that causes all of this

`snora-core` uses `lucide_icons::Icon` — a plain enum. `snora-widgets` uses
`Icon` and `LUCIDE_FONT_BYTES` — a byte slice. **Neither is gated behind
lucide's `iced` feature.**

And `snora-widgets/src/icon.rs:28-29` instructs us, in our own words, not to use
the thing the feature provides:

> **Do NOT call `lucide_const.widget()`** — that method returns
> `iced::widget::Text` parameterised against lucide-icons' own …

**We enabled a feature to get an integration we then wrote a comment forbidding
ourselves from using.**

## The fix, tested before proposing

```toml
lucide-icons = { version = "1", default-features = false }
```

Measured on the working tree:

| | before | after |
|---|---|---|
| `cargo check --workspace --all-features` | passes | **passes** |
| `cargo test --workspace --all-features` | 34 ok | **34 ok** |
| `clippy -D warnings` | clean | **clean** |
| `iced feature "advanced"` occurrences | present | **0** |
| `cargo tree -p snora-core --all-features` | pulls iced, winit | **`lucide-icons` only** |

## Open questions

**Q-1 — is removing a transitively-enabled feature a breaking change?** A
consumer whose own code uses `iced::advanced::` and relied on snora's lucide
dependency to enable it would stop compiling. **Suggest treating it as one** —
minor bump, migration guide, and a release note that says exactly this. Nobody
has told us they do it, and it was never documented, but "undocumented and
unlikely" is not the same as "cannot have happened", and the guide costs a
paragraph.

**Q-2 — does `snora-core` need a dependency gate like `snora-design`'s?**
`snora-design`'s iced-free property is CI-enforced (RFC-021/022 Q3);
`snora-core`'s equivalent is documented and unguarded, which is why this stood.
**Suggest yes, same shape** — and note it would have caught this at the commit
that introduced it rather than at a docs.rs page nobody was watching.

**Q-3 — does this change the archived RFC-078's picture?** It does not reopen
it: the ruling was that `advanced` must never be a **default**, and after this
fix that is true rather than merely nearly-true. But the archived RFC says
enabling `advanced` is an unmeasured cost, and **lucide users have been paying
it all along** — so if the opt-in feature is ever built, some of the cost data
already exists. Record it there; do not un-archive.

## Acceptance criteria

1. `cargo tree -p snora-core --all-features` shows **no iced**.
2. `iced feature "advanced"` appears in **no** feature tree.
3. docs.rs builds `snora-core` — verified on the published release, not assumed.
4. Q-1's note names the reliance that could break, in the migration guide.
5. `design-decisions.md` no longer lets *"not stable-by-default"* be read as
   *"never enabled"*.
6. Q-2 ruled; if a gate is added, a perturbation demo proves it fires.

## Compatibility and security

**Compatibility.** Removes a transitively-enabled feature. **Minor.**
**Security.** None — it removes a dependency edge rather than adding one.
