# Migration guide: 0.10 → 0.11

## Who is affected

This guide covers users upgrading from snora 0.10.x to 0.11.0.

## What changed

### `AppLayout` is now `#[non_exhaustive]`

`AppLayout` gained the `#[non_exhaustive]` attribute. This means that
struct literal construction of `AppLayout` from outside the `snora-core`
crate is no longer permitted by the compiler.

### Why

`#[non_exhaustive]` lets future minor releases add new top-level fields
(such as a popover slot or a focus-policy field) without breaking any
downstream code. It is applied during the pre-1.0 period as part of
establishing a stable construction contract before 1.0.

## Behavioral changes

None. The engine, overlays, toasts, and ABDD mirroring all behave the
same.

### Toast ordering fix

The rendered order of toasts has been corrected. The newest toast now
correctly appears **closest to the anchor edge**, matching the documented
`ToastPosition` invariant. If your application relied on the previous
inverted order, update the position or the order in which you push toasts.

## Mechanical migration

### `AppLayout` struct literal → builder

This change only affects you if you constructed `AppLayout` using a struct
literal *outside* `snora-core`. Every example and the recommended usage
pattern already use the builder, so most users need no change.

```rust
// Before (struct literal — no longer compiles from outside snora-core):
let layout = AppLayout {
    body,
    header: Some(header),
    direction: LayoutDirection::Rtl,
    on_close_modals: Some(Message::CloseModals),
    // ... all fields ...
};

// After (builder — already the canonical path):
let layout = AppLayout::new(body)
    .header(header)
    .direction(LayoutDirection::Rtl)
    .on_close_modals(Message::CloseModals);
```

Field *reads* (`layout.direction`, `layout.header`, etc.) continue to
work unchanged.

## Deprecated aliases

None in this release.
