# 6 — The workbench example

The **Snora Workbench** is a single application that exercises snora's
chrome and overlays together: the header, menus, dialog, sheet, toasts,
sidebar, tab bar, breadcrumb, footer and the reading direction. It is the
recommended starting point for manual QA of those, and its source is a
copy-paste reference for wiring them.

**What it does not show:** the form controls, `snora::control`, the design
path (`snora::design`) and `responsive_render`. The
[keyboard guide](../guides/keyboard.md) and the design pages cover those.

*(Corrected 2026-11-28, from the 0.67 audit: this said the workbench
exercises every major framework surface, and listed a "File ▾" button, a
4-second toast and a dialog and a sheet open together.)*

## What it demonstrates

| Surface | Where to find it |
|---|---|
| Header with menu button | Top bar — "File" button |
| File menu dropdown | Click "File" |
| Context menu | Click "Context menu" in the header |
| Dialog | Click "Dialog" in the header |
| Sheet (End-anchored) | Click "Sheet" in the header |
| Toasts — all five intents | "Toast Lab" tab |
| Toasts — all six positions | "Toast Lab" tab |
| Toast TTL lifecycle | Transient toasts dismiss themselves after a reading time: two seconds plus one per six characters, never under six (RFC-113). An Error toast stays until closed |
| LTR ↔ RTL toggle | "→ RTL" / "→ LTR" button in the header |
| Sidebar navigation | Left rail (mirrors under RTL) |
| Tab bar | Body area below breadcrumb |
| Breadcrumb | Above tabs |
| Footer status bar | Bottom bar — shows direction, overlays, toast count |

## Running the workbench

```bash
cargo run -p snora-example-workbench
```

## Manual QA checklist

Use this checklist when verifying a Snora release:

```text
Overlays
[ ] File menu opens and closes (header button + outside click).
[ ] Context menu opens and closes.
[ ] Dialog opens, dim layer visible, close button works.
[ ] Outside click on dim dismisses dialog.
[ ] Sheet opens at the End edge, close button works.
[ ] Toasts are visible above an open dialog.
[ ] Toast × button fires even while dialog is open.

Toasts
[ ] All five intent buttons add a toast with the correct color.
[ ] All six position buttons reanchor the toast stack.
[ ] Newest toast is closest to the anchor edge.
[ ] Transient toasts dismiss themselves after their reading time (the
    workbench's are 8 s); an Error toast stays until closed.

Direction (ABDD)
[ ] "→ RTL" button toggles the label to "→ LTR" and vice versa.
[ ] Sidebar moves to the right edge under RTL.
[ ] Sheet End edge appears on the left under RTL.
[ ] Toast TopEnd anchor appears on the left under RTL.
[ ] Header start/end controls swap sides.
[ ] Breadcrumb separator flips from › to ‹ under RTL.
```

## Source

The workbench is a single file (`examples/workbench/src/main.rs`).
It intentionally avoids domain logic — every tab body is minimal so
the focus stays on framework surface demonstration. If you are
starting a new Snora application, copy the state model and the
`AppLayout` assembly pattern from the workbench.
