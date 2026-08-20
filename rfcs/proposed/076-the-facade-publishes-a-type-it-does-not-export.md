# RFC 076 — `snora` publishes a function whose return type it does not export

**Status.** Proposed
**Tracks.** Public API / facade completeness.
**Found by** **arama**, 2026-08-20, while shipping F6 zone navigation.
**Touches.** `crates/snora/src/lib.rs`, `crates/snora/src/keyboard.rs`.
**Release target.** 0.39.0 — **minor**, public API addition.

## Summary

`snora::keyboard::cycle_zones` returns `Option<snora_core::focus::Cycle>`.
**`snora` does not re-export `focus`.**

`crates/snora/src/lib.rs:84-88` re-exports 21 `snora_core` items by name —
`AppLayout`, `Dialog`, `SideBar`, `Toast` and the rest. `FocusZone`, `Cycle`,
`ZonePresence` and `next_zone` are not among them, and there is no blanket
`pub use snora_core;` anywhere in the crate.

So a consumer using the facade **cannot name the return type of a function the
facade provides.** They can call it and match `Some(_)`; they cannot bind it,
store it, or pass it on.

## It is worse than an omission — our own docs route around it

`keyboard.rs`'s doc comments tell the reader to reach past the facade:

- `:82` — "([`snora_core::focus::next_zone`])"
- `:88` — "bind a different key to [`snora_core::focus::next_zone`] directly"
- `:107` — an example calling `snora_core::focus::next_zone(`

We documented the workaround instead of noticing the gap. **arama added a direct
`snora-core` dependency**, which works and which our own examples imply — but
RFC-060's 0.35.0 note presented `cycle_zones` as *the consumer-facing mapping*,
and as published it is not usable as one.

Their closing line is why this is a minor rather than a backlog entry: *"Worth
telling you before three consumers each add the same edge and none of them
mentions it."*

## Open questions

**Q-1 — re-export the module, or the items?** The existing list is item-by-item,
which argues for adding the four names to it. But `focus` is a coherent unit,
and `next_zone` as a bare name in `snora`'s root reads worse than
`snora::focus::next_zone`.

**Suggest `pub use snora_core::focus;`** — a module re-export, matching how
`keyboard` already appears as a module. `snora::focus::Cycle` then names the
type and nothing new lands in the root namespace.

**Q-2 — does anything else in the facade have this shape?** `cycle_zones` was
found because a consumer used it. **Enumerate every public signature in `snora`
and confirm each named type is reachable through `snora`.** Mechanisable, and it
should be done before assuming this is the only instance.

**Q-3 — is this the moment to close RFC-060 Q-1?** That question — enable iced's
`advanced` feature for focus querying — has had zero demand. arama has now
shipped zone navigation **without** it, on F6, with a permanent footer hint. A
fourth adopter declining trapping. **Suggest closing it as a decision record**,
but that is the owner's ruling, not this RFC's.

## Acceptance criteria

1. `cycle_zones`' return type is nameable using only a `snora` dependency —
   demonstrated by a compiling example that does **not** depend on `snora-core`.
2. `keyboard.rs`'s doc comments reference the facade path, not `snora_core::`.
3. Q-2's sweep run and its result stated — including "no others found", which is
   a result.
4. Additive only; nothing existing renamed or removed.

## Compatibility and security

**Compatibility.** Purely additive — a new public path in `snora`. Consumers who
added a direct `snora-core` dependency keep working. **Minor**, because it adds
public API. **Security.** None.
