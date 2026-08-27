# Migration 0.39 → 0.40

> **Possibly breaking, for a narrow and specific case.** If your own code
> calls `iced::advanced::` and you never enable iced's `advanced` feature
> yourself, you relied on `snora`'s `lucide-icons` feature enabling it for
> you. That reliance stops working in 0.40.0. The fix is one line in your
> own `Cargo.toml`. Nobody has told us they do this, and it was never
> documented as something you could rely on — but it was possible, so this
> guide says so rather than assuming it did not happen.

## Who is affected

**Only applications that both:**

1. Enable snora's `lucide-icons` feature, **and**
2. Call `iced::advanced::` directly in their own code, without separately
   enabling iced's `advanced` feature themselves.

If either is false, nothing changes for you.

## What changed

### `lucide-icons` no longer pulls in `iced`, `winit`, or `advanced`

The workspace's `lucide-icons` dependency declared `features = ["iced"]`.
`lucide-icons`' own manifest maps that feature to `iced` **with
`features = ["advanced"]` and no default features** — so every consumer
who turned on snora's `lucide-icons` feature transitively compiled all of
iced (including `winit`, a full windowing backend) into `snora-core`, the
crate whose entire identity is "no GUI dependency," and silently got
`iced`'s `advanced` feature enabled along with it.

**Nothing in snora used the integration that feature provided.**
`snora-core` uses `lucide_icons::Icon`, a plain enum; `snora-widgets` uses
that plus a byte slice (`LUCIDE_FONT_BYTES`). Neither calls the
iced-returning method `lucide-icons`' `iced` feature exists to provide —
`snora-widgets`' own source carries a comment instructing us specifically
**not** to call it, for reasons unrelated to this fix. We had enabled a
feature to get an integration we had separately told ourselves never to
use.

**This is also what broke `snora-core`'s published documentation.**
docs.rs builds every crate `--all-features`, and `snora-core`
`--all-features` pulled in `winit`, which fails to compile on docs.rs's
build target. `snora-core` 0.39.3's own docs.rs page did not build.

**The fix:**

```toml
# workspace Cargo.toml — before
lucide-icons = { version = "1", features = ["iced"] }

# after
lucide-icons = { version = "1", default-features = false }
```

### The good news, which is the actual point of the change

- **`snora-core` now has zero dependencies under every feature
  combination**, including `--all-features` — the property its own
  documentation always claimed, now true and CI-enforced (see below).
- **iced's `advanced` feature is enabled by snora nowhere, in no feature
  combination.** `design-decisions.md`'s "not stable-by-default" statement
  used to be true in the narrow sense and misleading in the way that
  matters, since `lucide-icons` enabled it regardless of any default. That
  gap is closed.

### If you were relying on the transitive `advanced` feature

Enable it yourself:

```toml
iced = { version = "0.14", features = ["advanced"] }
```

Cargo's feature unification means this is additive and safe to add
regardless of whether you actually needed the old transitive edge — if you
didn't need it, this line does nothing observable; if you did, it restores
exactly what you had.

Found from a live docs.rs build failure on the published `snora-core`
0.39.3, before any application reported hitting the `advanced` edge
itself.

## What did not change

- `Icon` and `LUCIDE_FONT_BYTES` — `lucide-icons`'s vocabulary that
  snora actually uses — are unaffected.
- No new dependency was added anywhere.
- No public snora item was renamed, removed, or retyped.
- `RFC-078` (measuring whether to enable `advanced` as an opt-in feature)
  stays archived; this fix does not reopen it — it makes the archived
  ruling ("`advanced` is never a default") true rather than nearly true.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
