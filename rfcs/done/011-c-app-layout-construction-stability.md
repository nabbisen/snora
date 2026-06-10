# RFC-011-C — AppLayout Construction Stability

**Status.** Implemented (v0.11.0)
**Tracks.** Public API stability. Fixes the stable construction path for
`AppLayout` before any new top-level field (popover, focus, etc.) is added.
**Touches.** `crates/snora-core/src/layout.rs`,
`docs/src/guides/migration-0.10-to-0.11.md` (new),
`docs/src/guides/migrations.md`,
`docs/src/contributing/design-decisions.md`,
`docs/src/reference/vocabulary.md`.

> Project-adopted version of planning-pack RFC-011-C. The planning draft
> presented Options A/B/C and *recommended* B; this version makes B
> **final**, backed by an in-tree audit, and specifies the exact edit and
> migration.

## 1. Summary

Decide — and implement — the long-term construction contract for
`AppLayout`. Decision: **Option B**. `AppLayout::new(body)` plus chainable
builders is the canonical, stable construction path; `AppLayout` becomes
`#[non_exhaustive]` during the pre-1.0 period so future top-level fields can
be added without breaking downstream construction.

## 2. Why this must be decided in v0.11

Three later RFCs explicitly depend on this decision and must not land a new
`AppLayout` field before it is made:

- RFC-013-A (anchored popover) — "may add a top-level field … depends on
  RFC-011-C."
- RFC-014-B (focus/accessibility) — "Do not add focus-related fields to
  `Dialog` or `Sheet` before resolving `AppLayout` construction stability."
- RFC-015-B (re-export/docs.rs) — lists RFC-011-C as a dependency.

Deciding now keeps every one of those unblocked and prevents a post-1.0
breaking change.

## 3. [Audit] In-tree impact of `#[non_exhaustive]`

Performed against the v0.10.0 tree:

- **Struct-literal construction `AppLayout { … }` outside the defining
  crate:** none. Every example uses `AppLayout::new(...)` + builders
  (`examples/{hello,skeleton,dialog,sheet,tab,multi_view,rtl,breadcrumb,
  header_menu,context_menu}`).
- **Direct field writes on an `AppLayout` value (`layout.field = …`):** none.
- **Engine field access:** `crates/snora/src/render.rs` reads
  `layout.direction`, `layout.header`, … by **field access**, never by
  exhaustive struct pattern. `#[non_exhaustive]` restricts *literal
  construction* and *exhaustive patterns* by foreign crates, not field
  reads, so the engine is unaffected.

Conclusion: `#[non_exhaustive]` breaks **no** in-tree code. The only
affected population is hypothetical external struct-literal users, of which
there are none (pre-1.0, examples-and-maintainer usage only). No new
accessors are required.

## 4. Goals

- Make the builder path the stable construction contract.
- Keep public fields readable (transparency is a Snora value).
- Unblock future top-level fields without a future breaking change.
- Provide a clear, short migration note.

## 5. Non-Goals

- No trait-based page contract (explicitly rejected in v0.4).
- No generic `Overlay` list.
- No new overlay features.
- No getters that merely mirror fields.

## 6. Decision (Option B) and rationale

| Option | Verdict |
|---|---|
| A — keep literals stable forever | Rejected. Freezes the field set; any future field is breaking; blocks popover/focus RFCs pre-1.0. |
| **B — builders canonical + `#[non_exhaustive]`** | **Adopted.** Matches the already-documented canonical path; future fields become additive; zero in-tree breakage (§3). |
| C — public fields + private extension bag | Rejected. Adds hidden state to a framework whose value is transparency; solves nothing A/B don't. |

## 7. Internal design

### 7.1 Code change

In `crates/snora-core/src/layout.rs`, add the attribute to the struct:

```rust
/// ...
/// Construction is via [`AppLayout::new`] plus chainable builder methods;
/// this is the stable, canonical path. The struct is `#[non_exhaustive]`
/// during the pre-1.0 period so future top-level surfaces can be added
/// without breaking downstream construction. Fields remain `pub` for
/// readability and in-crate use.
#[non_exhaustive]
pub struct AppLayout<Node, Message>
where
    Message: Clone,
{
    pub body: Node,
    // ... existing fields unchanged ...
}
```

`AppLayout::new` (inside `snora-core`) keeps using `Self { … }` — literal
construction is always allowed within the defining crate. No other code
change is needed.

### 7.2 Builder completeness (verified)

Every field already has a chainable setter; `new(body)` covers the required
field. No gaps. **Rule going forward (normative):** any PR adding an
`AppLayout` field must add its `#[must_use]` builder in the same PR and cite
this RFC.

| Field | Builder | Field | Builder |
|---|---|---|---|
| `body` | `new` | `sheet` | yes |
| `header` | yes | `toasts` | yes |
| `side_bar` | yes | `toast_position` | yes |
| `footer` | yes | `direction` | yes |
| `header_menu` | yes | `on_close_menus` | yes |
| `context_menu` | yes | `on_close_modals` | yes |
| `dialog` | yes | | |

### 7.3 Accessor policy

No mirror getters. Fields stay `pub` and readable. Add an accessor only if
it clarifies semantics beyond raw field access (none needed now).

## 8. Migration

New page `docs/src/guides/migration-0.10-to-0.11.md`, linked from
`docs/src/guides/migrations.md`. Core content:

```rust
// Before (only possible for the rare external struct-literal user):
let layout = AppLayout { body, header: Some(header), .. };

// After — canonical, already used by every example:
let layout = AppLayout::new(body).header(header);
```

Who is affected: only code constructing `AppLayout` by struct literal from
*outside* `snora-core`. In-tree code and all examples already use builders,
so most users see no change. Field *reads* continue to work.

## 9. Tests

- Existing examples compile unchanged (they already use builders).
- Add a `snora-core` doctest demonstrating canonical construction so the
  documented path is itself tested.
- No in-tree struct literals to migrate.

## 10. Risks and mitigations

| Risk | Mitigation |
|---|---|
| A user prefers struct literals. | Pre-1.0; builder path documented as stable; field reads still work. |
| `#[non_exhaustive]` makes field mutation awkward. | Audit (§3) shows no in-tree mutation; builders cover all fields. |
| Over-conservative stabilization hurts ergonomics. | Builder API is fluent and complete. |

## 11. Acceptance criteria

- `AppLayout` is `#[non_exhaustive]`.
- Decision recorded in `design-decisions.md` with the Option B rationale.
- Construction policy documented in reference docs.
- Migration note exists and is linked.
- Future RFCs adding layout fields cite this policy and add a builder.
