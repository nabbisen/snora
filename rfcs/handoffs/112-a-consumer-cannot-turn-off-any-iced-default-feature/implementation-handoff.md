# Implementation handoff — RFC-112: a consumer cannot turn off any of iced's default features

**RFC.** `rfcs/accepted/112-a-consumer-cannot-turn-off-any-iced-default-feature.md`
**Release target.** 0.54.0.
**Rulings (2026-10-21), all as suggested:**
- **The rule:** `web-colors` and `crisp` are snora's and always on. Everything else
  in `iced/default` passes through `iced-default`.
- **Q-1:** the feature is named `iced-default`.
- **Q-2:** not a covenant break. The architect records it in `api-freeze-review.md`.
- **Q-3:** deny warnings on every matrix row and on the design-isolation job.
- **Q-4:** 0.54.0.
- **Q-5:** correspondence is the architect's.

**Touch list.**
- `Cargo.toml` (workspace): the iced line, plus `default-features = false` on the
  `snora-widgets` and `snora-style` entries;
- `crates/snora/Cargo.toml`, `crates/snora-widgets/Cargo.toml`,
  `crates/snora-style/Cargo.toml`: `[features]`;
- `examples/responsive_body/Cargo.toml`, `examples/size_probe_engine/Cargo.toml`,
  `examples/size_probe_design_engine/Cargo.toml`: their iced lines;
- a new `publish = false` probe crate (Unit 3);
- `.github/workflows/ci.yaml`;
- `scripts/check-workspace-iced-features.sh`, plus `scripts/README.md` if the probe
  gets a script;
- `crates/snora/src/render.rs` (`DimFill::Fixed`, R-5);
- `CHANGELOG.md` (`[Unreleased]`, under Changed).

**Not:** any style function, colour, token or `src/` behaviour beyond R-5's warning
fix. The architect writes the 0.53 → 0.54 guide, `feature-flags.md`, the threat-model
and `deny.toml` clauses, and the `api-freeze-review.md` note.

## Already established (architect's scratch probe)

The scratch copy is in `.git-exclude/tmp/no-default-iced/`: a `git archive` of
`dc62565`'s crates (unchanged since), with R-1 applied by hand. `consumer/` is the opt-out probe
against R-1, and `consumer-today/` is the same probe against the real `crates/`.
Use it for reference, and re-derive anything you quote.

- **Today:** `consumer-today` gets `zbus v5.16.0` on `x86_64-unknown-linux-gnu`.
- **With R-1 and the workspace line at `["tokio", "web-colors", "crisp"]`:**
  - `consumer` builds;
  - `zbus` and `mundy` are gone;
  - `tiny-skia` and `wgpu` are present;
  - `cargo tree -e features -i iced` lists `crisp` and `web-colors`, though the
    probe's own line omits both.
- `cargo check -p snora-style`, `-p snora-widgets` and `-p snora`, each on its own
  with defaults, build.
- `cargo check -p snora --no-default-features` **fails**: `winit` reports "The
  platform you're compiling for is not supported by winit". Adding `--features
  iced-default` builds.
- With iced at `tokio` only plus a platform, all three crates check with
  `--all-features`. snora's library code needs no renderer.
- The probe in R-3 that I used was **outside** the workspace, so nothing else in the
  workspace could add features to it.

## Unit 1 — failing-first

Before touching any manifest:

1. Create the probe (Unit 3's shape) and run its assertions against today's
   manifests. **Record the failure:** `zbus` is present.
2. Record the stripped size of every size probe that `measure-binary-size.sh` builds,
   for R-2's byte-identical check.
3. Run every feature-matrix row locally with `RUSTFLAGS="-D warnings"`. **Record
   every row that fails.** At least the `default` row should fail on
   `DimFill::Fixed`.

## Unit 2 — R-1 and R-2

**Workspace `Cargo.toml`:**
```toml
iced = { version = "0.14", default-features = false, features = ["tokio", "web-colors", "crisp"] }
snora-widgets = { version = "0.53", path = "crates/snora-widgets", default-features = false }
snora-style   = { version = "0.53", path = "crates/snora-style",   default-features = false }
```

**Each crate's `[features]`:**
- `snora-style`: `default = ["iced-default"]`, `iced-default = ["iced/default"]`;
- `snora-widgets`: `default = ["iced-default"]`, `iced-default = ["iced/default",
  "snora-style?/iced-default"]`;
- `snora`: `default = ["widgets", "iced-default"]`, `iced-default = ["iced/default",
  "snora-widgets?/iced-default", "snora-style?/iced-default"]`.

**Comments:**
- Above the workspace iced line, comment the rule once: drawing features are
  snora's; features that choose what draws, or where it runs, are the application's.
  Name RFC-112.
- Each `iced-default` gets a one-line comment saying what turning it off is for.

**R-2:** `responsive_body`, `size_probe_engine` and `size_probe_design_engine` add
`features = ["default"]` to their `iced = { workspace = true }` lines. **Check every
other member** that inherits iced and turns snora's defaults off. Those three are
what I found by grepping for `default-features = false`.

## Unit 3 — the probe and CI (R-3, R-5, Q-3)

**The probe:**
- `snora = { path = …, default-features = false, features = ["widgets", "design"] }`;
- its own iced line, **not** `workspace = true`: `default-features = false, features
  = ["wgpu", "tiny-skia", "thread-pool", "x11", "wayland"]`. That leaves out
  `linux-theme-detection`, `web-colors` and `crisp`.

**Where it lives is your call.** A workspace member is simpler. It is valid only if
`cargo tree -p <probe>` resolves the probe's features without other members adding to
them. **Show that with a breakage** (row 4 below). If you can't show it, put the
probe outside the workspace as I did.

**CI job (Linux),** asserting with `--target x86_64-unknown-linux-gnu`:
- the probe builds;
- `cargo tree -e normal -i zbus` and `-i mundy` find nothing;
- `-i tiny-skia` and `-i wgpu` each find something;
- `cargo tree -e features -i iced` lists `web-colors` and `crisp`.

**Feature matrix:**
- Every row that passes `--no-default-features` adds `iced-default` to its features.
  So does the design-isolation job's `cargo check -p snora-widgets
  --no-default-features`.
- `RUSTFLAGS="-D warnings"` on the matrix check step and on the design-isolation job.

**R-5:** fix `DimFill::Fixed`'s dead-code warning when `design` is off. Choose the
smallest fix that keeps the variant honest. A `cfg` on the variant is acceptable, and
so is `#[cfg_attr(not(feature = "design"), allow(dead_code))]` with a comment saying
why. Fix any other row Unit 1 found in the same way. **If a fix is not small, stop
and report.**

**`check-workspace-iced-features.sh`:** it must parse the new line. Give `web-colors`
and `crisp` a named exemption next to `tokio`'s, stating RFC-112's rule.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. **Never
`git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| probe: `zbus` absent | today's manifests (Unit 1) |
| probe: `web-colors` and `crisp` present | workspace iced line at `["tokio"]` only |
| probe: `tiny-skia` and `wgpu` present | remove `tiny-skia` from the probe's own iced line: it must fail **only** on tiny-skia |
| probe isolated (workspace member only) | add `linux-theme-detection` to one other member's iced line: the probe must stay free of `zbus` |
| matrix rows need `iced-default` | remove it from one `--no-default-features` row: the winit error |
| warnings denied | Unit 1's `default` row, before R-5's fix |
| exemption in the gate | re-add `canvas` to the workspace line: the gate still fails, as RFC-088 proved |

**If a breakage cannot make its row fail, replace the assertion and say so.**

## Acceptance criteria

1. Every row above shown failing, then passing, with the outputs quoted.
2. Size probes byte-identical before and after (Unit 1, step 2).
3. Every matrix row green with warnings denied, and CI green on `main`.
4. `cargo package --workspace` (or `cargo publish --dry-run` per crate, in publish
   order) verifies `snora-style`, `snora-widgets` and `snora` on Linux.
5. CHANGELOG under Changed:
   - `iced-default`, and the rule;
   - the narrow case that stops a build compiling: `default-features = false` on
     `snora`, `snora-widgets` or `snora-style`, together with an own iced line that
     also turns defaults off;
   - "no change for a default build", backed by criterion 2.

   **Do not write the migration guide or touch `deny.toml`.** The architect does
   both, at the cut.
