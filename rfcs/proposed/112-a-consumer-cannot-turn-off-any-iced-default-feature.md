# RFC-112 — A consumer cannot turn off any of iced's default features

**Status.** Proposed.
**Raised.** 2026-10-21, architect, from aaai's reply to the 0.53.0 letter
(`.git-exclude/upstream/receive/0.53.0/aaai-to-snora-2026-10-21.md`). aaai tried our
advice and reported that it could not be applied. The architect confirmed the report
against 0.53.0 and tried the design below in a scratch copy.
**Release target.** 0.54.0 (Q-4). **Priority.** Not a defect in what snora draws.
Today snora decides part of every consumer's iced build, and one piece of advice we
published can't be followed.

---

## The finding

**Every snora crate that uses iced takes iced's default features.** The workspace
declares `iced = { version = "0.14", features = ["tokio"] }`, with defaults on.
`snora`, `snora-widgets` and `snora-style` all inherit that line. iced 0.14's
defaults are:

```
default = ["wgpu", "tiny-skia", "crisp", "web-colors",
           "thread-pool", "linux-theme-detection", "x11", "wayland"]
```

Cargo merges features across the whole graph and never removes one. So whatever an
application writes in its own iced dependency, all eight are on. aaai found this by
trying to drop `linux-theme-detection`, which is the only route by which the three
unmaintained-crate advisories of 0.53.0 (`zvariant`, `zbus_names`,
`zvariant_derive`) reach a Linux build.

**We told five adopters, and logolig, that they could do this.** The 0.52 → 0.53
guide §5 said "accept them by ID, or turn off iced's `linux-theme-detection`". It was
corrected on 2026-10-21 (`dc62565`). The letter to the five adopters said the same
thing.

**Measured, 2026-10-21, in a scratch copy:**

| Check | Today (0.53.0) | With R-1 |
|---|---|---|
| Consumer probe¹: `cargo tree -i zbus` on Linux | `zbus v5.16.0` | not in the graph |
| Consumer probe: `-i mundy` | present | not in the graph |
| Consumer probe: `-i tiny-skia`, `-i wgpu` | present | present (the consumer chose them) |
| `cargo check -p snora-style`, `-p snora-widgets`, `-p snora`, each on its own | builds | builds |
| `snora` with its own defaults: `-i zbus` | present | present (unchanged) |
| `cargo check -p snora --no-default-features` | builds | **fails**: winit needs `x11` or `wayland` on Linux |

¹ A crate outside the workspace with `snora = { default-features = false, features =
["widgets", "design"] }` and its own `iced = { default-features = false, features =
[every default except linux-theme-detection] }`.

**snora's library code needs none of iced's defaults.** With iced at `tokio` only,
plus a platform feature from elsewhere, all three crates check with every feature on.
They need no renderer: renderers are the application's choice. The one thing iced
cannot build without on Linux is a windowing platform, `x11` or `wayland`. An
application's own iced dependency always provides one, but a snora crate built **on
its own** does not. That covers our CI, `cargo publish`'s verification build and
docs.rs. This is why R-1 passes iced's defaults through instead of simply dropping
them.

**snora never asks the OS for its colour scheme.** Nothing in `crates/*/src` reads
the system theme, so `linux-theme-detection` was only ever on snora's behalf by
accident.

## Proposal

**R-1 — each crate that uses iced takes it without defaults and passes them through
a default-on feature, `iced-default` (Q-1).**

- Workspace line: `iced = { version = "0.14", default-features = false, features =
  ["tokio"] }`. `tokio` stays, for RFC-088's measured reason.
- `snora-style`: `default = ["iced-default"]`, `iced-default = ["iced/default"]`.
- `snora-widgets`: `default = ["iced-default"]`, `iced-default = ["iced/default",
  "snora-style?/iced-default"]`.
- `snora`: `default = ["widgets", "iced-default"]`, `iced-default = ["iced/default",
  "snora-widgets?/iced-default", "snora-style?/iced-default"]`.
- The workspace's `snora-widgets` and `snora-style` entries get
  `default-features = false`. Without that, `snora`'s own dependencies would turn
  `iced-default` back on, and nothing could be opted out.

**Nothing changes for anyone who keeps snora's defaults.** That is almost everyone.
A consumer who wants to choose iced's features writes:

```toml
snora = { version = "0.54", default-features = false, features = ["widgets", "design"] }
iced  = { version = "0.14", default-features = false, features = [
    "wgpu", "tiny-skia", "crisp", "web-colors", "thread-pool", "x11", "wayland",
] }
```

and gets exactly the iced features in their own line. **snora never chooses a
renderer.** aaai's condition, that tiny-skia must not be dropped silently, is met:
it is the consumer's own line that keeps it.

**R-2 — binaries in the workspace choose their own iced features.** Members that
inherit `iced = { workspace = true }` would lose iced's defaults when they also set
`default-features = false` on snora: `responsive_body`, `size_probe_engine` and
`size_probe_design_engine`. Each adds `features = ["default"]` to its iced line,
which is what a consumer does. **The size probes must come out byte-identical before
and after**, which proves that a default build doesn't change.

**R-3 — CI proves it from a consumer's position.**
- A `publish = false` probe crate, shaped like the footnote above, that CI builds.
  CI asserts that `cargo tree -i zbus` and `-i mundy` find nothing on Linux, and that
  `-i tiny-skia` and `-i wgpu` find each. **It must fail first** on today's manifests,
  where `zbus` is present.
- Every feature-matrix row that passes `--no-default-features` (to `snora`, and to
  `snora-widgets` in the design-isolation job) adds `iced-default`. Without it those
  rows fail on winit, as measured.
- `check-workspace-iced-features.sh` must still see the workspace line, with
  `default-features = false` added.

**R-4 — docs.**
- The 0.53 → 0.54 guide states:
  - who could notice: a consumer that sets `default-features = false` on `snora`,
    `snora-widgets` or `snora-style` **and** whose own iced dependency also turns
    defaults off;
  - how to opt out, with the snippet above;
  - that snora's measured contrasts and composited ratios assume iced's defaults.
    `web-colors` changes how alpha is blended, which moves the modal dim and hover
    overlays. `crisp` sets the default `snap` for quads. An application that turns
    either off is outside what snora measured.
- The threat model's 0.53.0 paragraph and the three `deny.toml` reasons gain one
  clause: a consumer can now keep these crates out of its graph by leaving
  `linux-theme-detection` off.
- `feature-flags.md` documents `iced-default`.

**R-5 — a warning CI cannot see.** With `design` off, RFC-110's `DimFill::Fixed` is
never constructed, and `cargo check -p snora` warns `dead_code`. CI misses it: the
feature-matrix rows run `cargo check` without denying warnings, and clippy runs with
all features only. Fix the warning, and deny warnings in the matrix rows (Q-3).

## Open questions

**Q-1 — the feature's name.** **Suggest `iced-default`**, after `iced/default`, which
is what it turns on. Alternatives: `iced-defaults`, or a platform-only feature, which
is rejected below.

**Q-2 — `snora-style` is under the additive-only covenant.** Its forbidden list
covers items, signatures, `Palette` roles and token meanings. A new Cargo feature that
is on by default touches none of these, and a default build is identical. **Suggest:
not a forbidden change, so D-3 and D-4 stay closed.** Record the reasoning in
`api-freeze-review.md`, so nobody has to work it out again.

**Q-3 — deny warnings in the feature matrix.** **Suggest yes** (`RUSTFLAGS="-D
warnings"` on the matrix's check step). The dev team runs every row first. If other
rows warn, they list them and fix them in the same change, or stop and report if a
fix is not small.

**Q-4 — release.** **Suggest 0.54.0.** For the narrow case in R-4 it can stop a build
compiling, the same shape as RFC-083 and RFC-088. The compiler names the cause: a
winit platform error, or a missing renderer.

**Q-5 — answer aaai before it ships.** **Suggest yes, once this is accepted:** "yes,
in 0.54.0; until then, keep accepting the three by ID". They asked to know whether
the option is open, and the answer changes now, not at release.

## Alternatives considered

- **Only a platform feature (for example `iced/wayland`) on by default.** Consumers
  would still get that platform forced on them, and the four other defaults (`crisp`,
  `web-colors`, `thread-pool`, the renderers) would need decisions of their own.
  Passing `default` through keeps today's build exactly as it is.
- **Drop the defaults with no feature, and publish with `--no-verify`.** That breaks
  standalone builds on Linux for contributors and docs.rs, and gives up a check
  `cargo publish` makes for us.
- **Keep everything and answer "no".** That is honest, but snora would keep deciding
  eight iced features for every application, including a renderer pair aaai depends
  on and a D-Bus client none of us asked for.

## What this is not

- Not a change to any style function, colour, token or rendered output under default
  features.
- Not a new dependency.

## Acceptance criteria

1. R-3's probe is **shown failing** on today's manifests (`zbus` present), then
   passing. Both outputs are quoted.
2. The size probes are byte-identical before and after (R-2).
3. Every feature-matrix row passes, with warnings denied (R-5, Q-3).
4. `cargo publish --dry-run` (or `cargo package`) verifies for `snora-style`,
   `snora-widgets` and `snora` on Linux.
5. CHANGELOG under Changed, with the narrow breaking case named. The migration guide
   follows R-4.
