# Migration 0.48 → 0.49

> **Not breaking — no API, appearance, feature-resolution or MSRV change.**
> But unlike the last two releases this one is worth reading, and it contains
> **one thing to check on your own side** that has nothing to do with snora's
> code.

## Who is affected

**For code: nobody.** Upgrading is a version bump.

**If you run `cargo-deny` in your own CI, read §2.** You may have the same
blind spot we did, independently of snora.

## 1. Our advisory gate was blind to a whole class, and it hid three

Advisory scanning arrived in 0.47.0. From then until 0.48.0 it reported
`advisories ok` on every run, and **it could not report an `unsound` advisory
at all.**

`cargo-deny`'s `[advisories] unsound` key takes a **scope** — `"all"`,
`"workspace"`, `"transitive"`, `"none"` — **not** a lint level, and its default
excludes transitive dependencies. snora declares five direct dependencies and
resolves 394; everything the gate exists to watch is transitive, so the default
excluded all of it. We did not read the default.

**Two of the three hidden advisories had published fixes and are taken in this
release:**

| Advisory | Crate | Now |
|---|---|---|
| `RUSTSEC-2026-0186` | `memmap2` — unchecked pointer offset | 0.9.10 → **0.9.11** |
| `RUSTSEC-2026-0221` | `event-listener` — `!Send` tags crossing thread boundaries | 5.4.1 → **5.4.2** |

**The third is accepted, and it is not the same kind of acceptance as the
others.** `RUSTSEC-2026-0253` — a use-after-free / double-free in `lru`'s
`LruCache::pop()` when a stored key's `Drop` panics — is **memory corruption
on the default runtime path**, reached through `cryoglyph`'s glyph cache under
the wgpu renderer. It is not an unmaintained-crate advisory, and it is named
explicitly in [the threat model](../reference/threat-model.md), which the
other accepted advisories are not.

**Neither you nor we can clear it today.** `cargo update -p lru` will not move
it: `cryoglyph` 0.1.0 — its only published release — declares `lru ^0.16`,
which cannot admit the patched 0.18.2. Verified against `lru`, its parent
`cryoglyph`, its grandparent `iced_wgpu`, a full fresh resolve, and
`--ignore-rust-version`. It retires when a `cryoglyph` release takes
`lru >= 0.18.2`, and our gate will fail and name the entry the moment that
happens.

For most applications the trigger is not a realistic path — it needs a stored
key whose `Drop` panics, with unwinding enabled. We are telling you because it
is in your graph too if you use the default renderer, and because you should
not learn it from your own scanner.

## 2. Check this on your side

**If your CI runs `cargo-deny` and you have not set `unsound` explicitly, you
have the same blind spot** — regardless of anything about snora. One line:

```toml
[advisories]
unsound = "all"   # a SCOPE, not a lint level; the default excludes transitive deps
```

If you use `cargo audit` instead, it does not carry this default and this
section does not apply to you.

The broader lesson, which cost us two releases: **an unset key is not "off",
it is that key's own default** — and a default written for a workspace's own
code is the wrong default for a dependency graph.

## 3. What we changed so this shape cannot recur

Setting the scope fixes today's graph. It does not fix the mistake, so
`scripts/check-advisories.sh` **now refuses to run at all against a
configuration that leaves any advisory class unset.** The failure was never a
wrong value — it was an absent key behaving permissively — so checking values
would not have caught it, and only checking presence does.

## 4. This was found externally, and we would rather say so

**tekstide found it**, and they are not a consumer — they had asked to be
dropped from compatibility letters, and we had excluded them from the 0.46 →
0.48 note. They replied anyway, checked our published advisory list against
their own graph, and wrote the sentence that turned one advisory into this
release:

> *"If your gate does not surface it, that is worth knowing more than the
> advisory itself is."*

That was right. A team with no dependency on us audited our disclosure more
effectively than our own tooling did.

## What did not change

- No public API, no rendered appearance, no feature resolution, no MSRV (still
  `1.88`).
- **No visual-regression baseline is invalidated** — the sixth release running.
- **`Cargo.lock` did move, and this time third-party packages moved too** —
  `memmap2` and `event-listener`, plus snora's own 26 crate versions. Checked:
  no package entry was added or removed, and no other dependency changed
  version. Your own resolution is unaffected; a library's lockfile does not
  bind the applications that depend on it.

## If you are jumping more than one minor

**0.48** ruled the non-colour cue question closed rather than deferred — if you
were waiting on an icon or text prefix for toast intents, supply your own.
**0.47** enforced `#![forbid(unsafe_code)]` across all five crates and adopted
the advisory scanning this release repairs. **0.45** removed
`snora_design::{Emphasis, Size}`. **0.41** and **0.42** carry real behavioural
and dependency changes. The [migration index](migrations.md) lists them all.
