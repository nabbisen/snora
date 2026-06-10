# 6 — The workbench example

The **Snora Workbench** is a single application that exercises every
major framework surface together. It is the recommended starting point
for manual QA, and its source is useful as a copy-paste reference for
wiring surfaces you haven't used before.

## What it demonstrates

| Surface | Where to find it |
|---|---|
| Header with menu button | Top bar — "File ▾" button |
| File menu dropdown | Click "File ▾" |
| Context menu | Click "Context menu" in the header |
| Dialog | Click "Dialog" in the header |
| Sheet (End-anchored) | Click "Sheet" in the header |
| Toasts — all five intents | "Toast Lab" tab |
| Toasts — all six positions | "Toast Lab" tab |
| Toast TTL lifecycle | Transient toasts auto-dismiss after 4 s |
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
[ ] Dialog and sheet can be open simultaneously (advanced, Law 3).
[ ] Toasts are visible above an open dialog.
[ ] Toast × button fires even while dialog is open.

Toasts
[ ] All five intent buttons add a toast with the correct color.
[ ] All six position buttons reanchor the toast stack.
[ ] Newest toast is closest to the anchor edge.
[ ] Transient toasts auto-dismiss after ~4 seconds.

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
