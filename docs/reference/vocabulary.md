# Vocabulary cheatsheet

Every public enum in snora-core, with one-line semantics. Use this as
a quick scan when you forget a variant name.

## Direction and edges

```rust
pub enum LayoutDirection { Ltr, Rtl }
pub enum Edge            { Start, End }
```

`LayoutDirection` is the framework-wide reading direction. `Edge`
expresses logical position on the horizontal axis; resolve to physical
left/right via `Edge::is_left_under(direction)`.

## Toasts

```rust
pub enum ToastIntent {
    Debug,
    Info,
    Success,
    Warning,
    Error,
}

pub enum ToastLifetime {
    Transient(std::time::Duration),
    Persistent,
}

pub enum ToastPosition {
    TopEnd,        // default
    TopStart,
    TopCenter,
    BottomEnd,
    BottomStart,
    BottomCenter,
}
```

`ToastIntent` maps to theme colors (intent → palette pair). Helpers:

- `ToastLifetime::DEFAULT` — 4-second transient.
- `ToastLifetime::seconds(n)` / `ToastLifetime::millis(ms)`.
- `ToastPosition::is_top()` / `is_bottom()` — partition helpers.

## Bottom sheet

```rust
pub enum SheetHeight {
    OneThird,           // default
    Half,
    TwoThirds,
    Ratio(f32),         // clamped 0.0..=1.0
    Pixels(f32),
}
```

Helpers: `SheetHeight::DEFAULT`, `as_ratio()`, `as_pixels()`.

## Icons

```rust
pub enum Icon {
    Text(String),
    #[cfg(feature = "lucide-icons")] Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]    Svg(std::path::PathBuf),
}
```

## Menu actions

```rust
pub enum MenuAction<MenuId, MenuItemId> {
    MenuPressed(MenuId),
    MenuItemPressed { menu_id: MenuId, menu_item_id: MenuItemId },
}
```

`MenuId` and `MenuItemId` are application-defined. snora is generic
over both.

## Defaults at a glance

```text
LayoutDirection::default()  → Ltr
ToastPosition::default()    → TopEnd
ToastLifetime::DEFAULT      → Transient(4 s)
SheetHeight::DEFAULT        → OneThird
```
