# Migrating from 0.20 to 0.21

v0.21 adds the **notice, chip, and progress** shallow primitives to Snora
Design. No breaking changes. All existing applications compile unchanged.

## What changed

### New: notice, chip, and progress primitives

Three new modules under `snora::design` when the `design` feature is enabled:

**`snora::design::notice::Notice`** — builder-style notice banner:

```rust,ignore
use snora::design::{Tokens, Tone, notice::Notice};

Notice::new(&tokens, Tone::Warning, "Disk space low.")
    .title("Storage warning")
    .action("Free space", Message::FreeSpace)
    .dismiss(Message::DismissNotice)
    .render()
```

**`snora::design::chip::{filter, removable}`** — toggle and removable chips:

```rust,ignore
use snora::design::chip;

chip::filter(&tokens, "Draft", self.show_drafts, Message::Toggle)
chip::removable(&tokens, "Rust", true, Message::Toggle, Message::Remove)
```

**`snora::design::progress::{row, card}`** — progress indicators:

```rust,ignore
use snora::design::{Tone, progress};

progress::row(&tokens, "Indexing…", Some(0.6), Tone::Accent)
progress::card(&tokens, "Syncing",  None,      Tone::Info)   // indeterminate
```

Pass `None` for indeterminate state (renders as 0% with "…" suffix —
iced 0.14 has no native indeterminate animation; documented limitation).

**`snora::design::style::progress::toned`** — progress bar style function
for manual wiring.

### Design workbench updated

`snora-example-design-workbench` now includes notice, chip, and progress
sections for all four token presets.

## Upgrade steps

1. Change `snora = "0.20"` to `snora = "0.21"` in `Cargo.toml`.
2. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | No |
| Does any type rename or move? | No |
| Does a default behavior change? | No |
| Does a new public item require downstream action? | No — all new items are additive, behind the `design` feature |
