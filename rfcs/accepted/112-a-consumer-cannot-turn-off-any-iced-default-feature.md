# RFC-112 — A consumer cannot turn off any of iced's default features

**Status.** Accepted 2026-10-21. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/112-…`](../handoffs/112-a-consumer-cannot-turn-off-any-iced-default-feature/implementation-handoff.md).
**Raised.** 2026-10-21, architect, from aaai's reply to the 0.53.0 letter
(`.git-exclude/upstream/receive/0.53.0/aaai-to-snora-2026-10-21.md`). aaai tried our
advice and reported that it could not be applied. The architect confirmed the report
against 0.53.0 and tried the design below in a scratch copy.
**Revised.** 2026-10-21, after the architect's own review against ABDD (see "Review
against snora's principles"). R-1 no longer lets a consumer drop `web-colors` or
`crisp`.
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

**Two of the defaults change what snora draws, and snora's guarantees rest on
them.** Without `web-colors`, iced's wgpu renderer picks an sRGB surface and blends
in linear light (`iced_graphics-0.14.0/src/color.rs`,
`iced_wgpu-0.14.0/src/window/compositor.rs:111`). tiny-skia is unaffected. snora's
composited ratios were measured with sRGB blending. Computed for the unstyled modal
dim at alpha 0.44:

| Page | sRGB blending (today) | Linear blending (no `web-colors`, wgpu) |
|---|---|---|
| White, black dim | 3.24 | **1.72**, under the 3.0 floor |
| Black, white dim | 4.25 | 9.80 |

The two "today" figures match the top of the measured ranges (3.05–3.24 and
3.74–4.36). An application that could turn `web-colors` off could break a floor snora
publishes, without being told.

`crisp` sets the default `snap` for quads, buttons and containers. **Its effect on
snora's elements is unmeasured.** In 0.53.0, snora's own explicit `snap` changed 0
pixels at scale 2. It is kept by the rule below, because it changes drawing and every
snora measurement was taken with it on, not because a known floor depends on it.

**snora never asks the OS for its colour scheme.** Nothing in `crates/*/src` reads
the system theme, so `linux-theme-detection` was only ever on snora's behalf by
accident.

## Proposal

**The rule, stated once:** *an iced feature that changes how a colour or an edge is
drawn is snora's, and always on. A feature that chooses what draws it, or where it
runs, is the application's.* The first group is `web-colors` and `crisp`. The second
is `wgpu`, `tiny-skia`, `thread-pool`, `linux-theme-detection`, `x11` and `wayland`.
`tokio` is snora's for RFC-088's reason: `snora::toast::subscription` does not compile
without an executor.

**R-1 — each crate that uses iced takes it without defaults, keeps its own three
features, and passes the rest through a default-on feature, `iced-default` (Q-1).**

- Workspace line: `iced = { version = "0.14", default-features = false, features =
  ["tokio", "web-colors", "crisp"] }`.
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

and gets their own line's features, plus snora's three. **snora never chooses a
renderer.** aaai's condition, that tiny-skia must not be dropped silently, is met:
it is the consumer's own line that keeps it. If a consumer leaves `web-colors` or
`crisp` out of their line, snora keeps them on anyway (tried: the probe's feature
list still shows both).

`iced-default` stays on by default, so an application that follows the OS's light,
dark or high-contrast preference through `linux-theme-detection` keeps it without
doing anything.

**R-2 — binaries in the workspace choose their own iced features.** Members that
inherit `iced = { workspace = true }` would lose iced's defaults when they also set
`default-features = false` on snora: `responsive_body`, `size_probe_engine` and
`size_probe_design_engine`. Each adds `features = ["default"]` to its iced line,
which is what a consumer does. **The size probes must come out byte-identical before
and after**, which proves that a default build doesn't change.

**R-3 — CI proves it from a consumer's position.**
- A `publish = false` probe crate, shaped like the footnote above, that CI builds.
  CI asserts that `cargo tree -i zbus` and `-i mundy` find nothing on Linux, and that
  `-i tiny-skia` and `-i wgpu` find each. The probe's own iced line leaves out
  `web-colors` and `crisp`, and CI asserts that `cargo tree -e features -i iced`
  still lists both. **Each assertion must fail first:** `zbus` is present on
  today's manifests, and both features are missing from the probe if R-1 is
  applied without the workspace line's three features.
- Every feature-matrix row that passes `--no-default-features` (to `snora`, and to
  `snora-widgets` in the design-isolation job) adds `iced-default`. Without it those
  rows fail on winit, as measured.
- `check-workspace-iced-features.sh` must still see the workspace line, with
  `default-features = false` added. `web-colors` and `crisp` are not used by name
  in code. Give them a named exemption next to `tokio`'s, stating the rule above,
  so they are never removed as unused.

**R-4 — docs.**
- The 0.53 → 0.54 guide states:
  - who could notice: a consumer that sets `default-features = false` on `snora`,
    `snora-widgets` or `snora-style` **and** whose own iced dependency also turns
    defaults off;
  - how to opt out, with the snippet above;
  - the rule above, and that `web-colors` and `crisp` stay on whatever the
    consumer writes, because snora's measured floors depend on them.
- The threat model's 0.53.0 paragraph and the three `deny.toml` reasons gain one
  clause: a consumer can now keep these crates out of its graph by leaving
  `linux-theme-detection` off.
- `feature-flags.md` documents `iced-default` and the rule.

**R-5 — a warning CI cannot see.** With `design` off, RFC-110's `DimFill::Fixed` is
never constructed, and `cargo check -p snora` warns `dead_code`. CI misses it: the
feature-matrix rows run `cargo check` without denying warnings, and clippy runs with
all features only. Fix the warning, and deny warnings in the matrix rows (Q-3).

## Open questions

**Q-1 — the feature's name.** **Suggest `iced-default`.** It turns on `iced/default`,
so a reader can tell what it does from the name alone. It stays accurate with
snora's three features on regardless: `iced/default` contains them.

**Q-2 — `snora-style` is under the additive-only covenant.** Its forbidden list
covers items, signatures, `Palette` roles and token meanings. A new Cargo feature
that is on by default touches none of these, and a default build is identical.
**Suggest: not a forbidden change, so D-3 and D-4 stay closed.** Record the reasoning
in `api-freeze-review.md`. One narrow case changes. Today `default-features = false`
on `snora-style` or `snora-widgets` does nothing, because neither crate has a
default feature. Afterwards it drops iced's defaults. R-4's guide names this case.

**Q-3 — deny warnings in the feature matrix.** **Suggest yes**, on every check row,
as a fixed rule rather than a choice per row (`RUSTFLAGS="-D warnings"` on the
matrix's check step and on the design-isolation job). The dev team runs every row
first. If other rows warn, they list them and fix them in the same change, or stop
and report if a fix is not small.

**Q-4 — release.** **Suggest 0.54.0.** For the narrow cases in R-4 and Q-2 it can
stop a build compiling, the same shape as RFC-083 and RFC-088. The compiler names the
cause: a winit platform error, or a missing renderer.

**Q-5 — who hears about this, and when.** **Suggest one short reply, to aaai only,
once this RFC is accepted.** They asked a question, which clears the correspondence
bar. The reply says that the option will exist from 0.54.0, that `web-colors` and
`crisp` stay on and why, and that until then they keep accepting the three by ID.
Two things **not** to send:
- **No letter to the other four adopters or to logolig** about the withdrawn advice.
  The guide is corrected, trying the advice did no harm, and none of them has said
  they acted on it.
- **No letter when 0.54.0 ships.** The migration guide carries the recipe.

## Review against snora's principles

The architect reviewed the first draft (`96722ec`) against the owner's standing
rules before any ruling. Two changes resulted:

- **ABDD (accessible by default and by design).** The first draft passed all of
  `iced/default` through one feature, so an application could turn off
  `web-colors` and take the light-theme dim from 3.24 to 1.72. Its docs only said
  such an application was "outside what snora measured". That is accessible by
  default but not by design. R-1 now keeps the two drawing features on
  unconditionally.
- **Rules a reader can predict.** The split between snora's features and the
  application's is now one stated rule (drawing vs choosing and running), not a list
  to look up. The same goes for Q-3: warnings are denied on every row, not on rows
  someone selects.

The other rules hold as drafted:
- **Verify consumer claims:** aaai's claim was reproduced.
- **Try remedies downstream:** the recipe was built and checked from a consumer's
  position.
- **Failing first:** acceptance criterion 1.
- **Mark deprecated when superseded:** nothing is superseded. No path or item moves.
  `iced-default` adds a dimension and replaces nothing.

## Alternatives considered

- **Only a platform feature (for example `iced/wayland`) on by default.** Consumers
  would still get that platform forced on them, and the renderers and `thread-pool`
  would need decisions of their own. Passing `default` through keeps today's build
  exactly as it is.
- **Pass all of `iced/default` through, with nothing fixed.** This was the first
  draft. It lets an application break the dim floor (see the finding).
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
   passing. The `web-colors`/`crisp` assertion is shown failing with the workspace
   line at `["tokio"]` only. All outputs are quoted.
2. The size probes are byte-identical before and after (R-2).
3. Every feature-matrix row passes, with warnings denied (R-5, Q-3).
4. `cargo publish --dry-run` (or `cargo package`) verifies for `snora-style`,
   `snora-widgets` and `snora` on Linux.
5. CHANGELOG under Changed, with the narrow breaking case named. The migration guide
   follows R-4.

---

## Rulings, 2026-10-21

All suggestions accepted, as revised in `e98d8c6`.

- **The rule:** drawing features (`web-colors`, `crisp`) are snora's and always on.
  Features that choose what draws, or where it runs, are the application's.
- **Q-1:** the feature is named `iced-default`.
- **Q-2:** not a forbidden change under the covenant, so D-3 and D-4 stay closed. The
  reasoning is recorded in `api-freeze-review.md`.
- **Q-3:** warnings are denied on every feature-matrix row and on the
  design-isolation job.
- **Q-4:** ships in 0.54.0.
- **Q-5:** one reply, to aaai only. No letter to the other adopters or logolig.
  **Amended by the owner, 2026-10-21: the reply goes when 0.54.0 is published, not on
  acceptance.** The draft waits in `.git-exclude/upstream/send/draft/`. At the cut,
  rewrite it from "from 0.54.0" to "0.54.0 is out", and check it against what
  shipped.
