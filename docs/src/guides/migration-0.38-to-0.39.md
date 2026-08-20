# Migration 0.38 → 0.39

> **Nothing is required.** Everything in 0.39.0 is additive or a correction to
> our own documentation. No public item was renamed, removed, or retyped, and
> nothing snora renders changed.

## Who is affected

Nobody is *required* to change anything.

Two groups may **choose** to:

- **Applications using F6 zone navigation** (`snora::keyboard::cycle_zones`) can
  now drop a direct `snora-core` dependency — see below.
- **Anyone who read our explanation of what makes the dialog card visible**
  against the modal dim: that explanation was wrong, and the corrected version
  may change a conclusion you drew from it.

## What changed

### `snora::focus` — you can drop a workaround

`snora::keyboard::cycle_zones` returns `Option<Cycle>`, and until 0.39.0 the
`snora` facade did not export `Cycle`. A consumer depending only on `snora`
could call the function and match `Some(_)`, but could not name its return
type. Our own doc comments told you to reach into `snora_core` instead — which
is how the gap survived from 0.35.0.

```rust
// 0.38 and earlier — needed a direct `snora-core` dependency:
let cycle: Option<snora_core::focus::Cycle> =
    snora::keyboard::cycle_zones(key, modifiers);

// 0.39.0 — `snora` alone is enough:
let cycle: Option<snora::focus::Cycle> =
    snora::keyboard::cycle_zones(key, modifiers);
```

`snora::focus` carries `Cycle`, `FocusZone`, `ZonePresence` and `next_zone`.

**Keeping your direct `snora-core` dependency also works** and breaks nothing.
This is a simplification you may take or ignore.

Reported by **arama**, who hit it while shipping F6 navigation and told us
before other teams each added the same edge silently.

### The dialog card's border — a correction to what we told you

**No colour changed.** This is a correction to our explanation, and it matters
only if you relied on the explanation.

We previously described the dialog card's **border** as what identifies the card
against the modal dim, citing WCAG 2.1 SC 1.4.11. **Measured, it is not.** The
border reaches **1.00:1 against the dim** — invisible — in *every* preset, at
some content luminance. What separates the card from the dimmed page is the
**dim-to-fill step**, which measures 3.16:1 or better everywhere.

The border repair shipped in 0.34.0 is still real and still necessary — it does
its work at the card's **inner** edge, border against the card's own fill, at
3.38:1 (`light`) and 3.17:1 (`dark`). It simply does not do the job we said it
did.

**What to re-check:** if you recorded, in an accessibility audit or a design
note, that snora's dialog card is *border-defined against the dim*, that
statement came from us and is wrong. The card is fill-defined against the dim
and border-defined against its own surface. Nothing about your application needs
to change; the record does.

Measured by **arama** over photographic content, re-derived across the full
content range for all four presets.

## What did not change

- No palette value, no `DIM_ALPHA`, no contrast assertion.
- No rendered output. If you hold reference images or screenshots, 0.39.0 does
  not invalidate them.
- No public item renamed, removed, or retyped.

## If you are jumping more than one minor

0.38 → 0.39 requires nothing. If you are arriving from further back, read the
guides for the jumps in between — several of them carry real changes, and the
[migration index](migrations.md) lists them.
