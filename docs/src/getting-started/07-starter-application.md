# 7 — Starter application

The starter example (`snora-example-starter`) is a minimal but complete
Snora application you can copy and expand. It is intentionally smaller
than the [workbench](06-workbench.md): everything you need, nothing you
don't.

## What it demonstrates

| Pattern | Where |
|---|---|
| `AppLayout` assembly | `view()` — assembles header, sidebar, body, footer |
| Header menu with close sink | `menu_open` state + `on_close_menus` |
| Dialog with Escape close | `show_dialog` state + `on_close_modals` + `keyboard::dismiss_on_escape` |
| Transient toast | `toasts: Vec<Toast<Message>>` + sweep subscription |
| LTR/RTL toggle | `direction: LayoutDirection` + `ToggleDirection` message |
| Tab bar | `TabBar` + `TabAction` message |
| Sidebar navigation | `SideBar` + `SideBarItem` |

## Running it

```bash
cargo run -p snora-example-starter
```

## Code walkthrough

The 177 ELOC are split into four sections with inline comments:

1. **State** (`struct App`) — only overlay flags, navigation state, and
   the toast queue. Nothing Snora-specific beyond `LayoutDirection`.
2. **Message** — one variant per user action. `CloseMenus` / `CloseModals`
   are the Snora close-sink messages; everything else is app logic.
3. **Update** — pure state mutation. Shows the Law 2 pattern (clear menus
   before opening a modal) and `dismiss_on_escape` wiring.
4. **View** — builds slots with prefab widgets, assembles `AppLayout`,
   hands it to `snora::render`. The comment on `direction` explains why
   one field drives all mirroring.

## Expanding the starter

| Want to add | Look at |
|---|---|
| Sheet overlay | `examples/sheet` |
| Context menu | `examples/context_menu` |
| All surfaces together | `examples/workbench` |
| Custom icons | `docs/src/guides/icons.md` |
| RTL deep-dive | `docs/src/guides/direction.md` |

The starter is a **recommended pattern**, not a required architecture.
Snora does not own your domain state, routing, persistence, or async
work — only the skeleton and its overlay stack.
