# RFC-012-B — Showcase / Workbench Example

Status: Proposed  
Target release: v0.12  
Priority: Medium-high  
Type: Example / Dogfood / Manual QA

## 1. Summary

Add a comprehensive `examples/workbench` application that exercises all major Snora surfaces together: header, sidebar,
body, footer, menus, context menu, dialog, sheet, toasts, tab bar, breadcrumb, icons, and LTR/RTL switching.

## 2. Motivation

Small examples are excellent for learning individual features, but they do not prove that surfaces compose well. Snora's
framework value is composition. A single workbench example should become:

- a manual QA app;
- a docs screenshot/source-of-truth app;
- a dogfood target for z-stack behavior;
- a demonstration that Snora remains small while enabling real app shells.

## 3. Goals

- Demonstrate the complete application skeleton.
- Demonstrate all overlay classes coexisting under documented semantics.
- Demonstrate LTR/RTL toggling.
- Demonstrate toast lifecycle helpers.
- Demonstrate default `widgets` feature usage.
- Keep the example comprehensible and not domain-heavy.

## 4. Non-Goals

- Do not build a production application.
- Do not introduce persistent storage.
- Do not add network calls.
- Do not benchmark performance.
- Do not create a new crate outside `examples/`.

## 5. External Design

Example path:

```text
examples/workbench/
  Cargo.toml
  src/main.rs
```

App layout:

```text
+--------------------------------------------------------------------------------+
| Header: Snora Workbench      [File ▼] [View ▼] [Toggle RTL] [Open Sheet]       |
+----------+---------------------------------------------------------------------+
| Sidebar  | Breadcrumb: Home / Project / Surface                                 |
|          | Tabs: Overview | Overlay Lab | Toast Lab | Direction Lab              |
| - Home   |                                                                     |
| - Layout | Main body changes by active tab.                                    |
| - Toasts |                                                                     |
| - RTL    |                                                                     |
+----------+---------------------------------------------------------------------+
| Footer: active view, direction, overlay state, toast count                     |
+--------------------------------------------------------------------------------+
```

Surfaces demonstrated:

| Surface | Workbench interaction |
|---|---|
| Header | static chrome with menu buttons |
| Sidebar | navigation items with tooltips |
| Body | tab-specific content |
| Footer | status summary |
| Header menu | File/View menu dropdown |
| Context menu | right-click or button-triggered floating menu |
| Dialog | “About Snora” or “Confirm action” |
| Sheet | settings/details side panel |
| Toasts | success/info/warning/error/debug buttons |
| Toast positions | selector for six positions |
| Direction | LTR/RTL toggle |
| Breadcrumb | current location trail |
| Tab bar | surface lab sections |

## 6. Internal Design

### 6.1 State Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Home,
    Layout,
    Toasts,
    Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Overview,
    OverlayLab,
    ToastLab,
    DirectionLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenMenu {
    File,
    View,
    Context,
}

struct Workbench {
    direction: LayoutDirection,
    view: View,
    tab: ActiveTab,
    open_menu: Option<OpenMenu>,
    show_dialog: bool,
    show_sheet: bool,
    toasts: Vec<Toast<Message>>,
    next_toast_id: u64,
    toast_position: ToastPosition,
}
```

### 6.2 Message Model

```rust
#[derive(Debug, Clone)]
enum Message {
    SelectView(View),
    SelectTab(ActiveTab),
    ToggleDirection,
    OpenFileMenu,
    OpenViewMenu,
    OpenContextMenu,
    CloseMenus,
    OpenDialog,
    OpenSheet,
    CloseModals,
    AddToast(ToastIntent),
    DismissToast(u64),
    SetToastPosition(ToastPosition),
    ToastTick,
}
```

### 6.3 Update Rules

- Opening any modal closes menus first.
- Opening any menu leaves modals unchanged only if no modal is open; otherwise ignore or close modal depending on chosen sample policy.
- `CloseMenus` clears `open_menu`.
- `CloseModals` clears `show_dialog` and `show_sheet`.
- `ToastTick` calls `snora::toast::sweep_expired`.
- `AddToast(intent)` appends to the queue and increments `next_toast_id`.

### 6.4 View Construction

`view()` builds slots first, then assembles:

```rust
let layout = AppLayout::new(body)
    .header(header)
    .side_bar(sidebar)
    .footer(footer)
    .toasts(self.toasts.clone())
    .toast_position(self.toast_position)
    .direction(self.direction)
    .on_close_menus(Message::CloseMenus)
    .on_close_modals(Message::CloseModals);

let layout = if let Some(menu) = self.menu_element() {
    layout.header_menu(menu)
} else {
    layout
};

let layout = if self.show_dialog {
    layout.dialog(Dialog::new(dialog_content))
} else {
    layout
};

let layout = if self.show_sheet {
    layout.sheet(Sheet::new(sheet_content).at(SheetEdge::End).with_size(SheetSize::OneThird))
} else {
    layout
};

snora::render(layout)
```

## 7. Documentation Changes

- Add `docs/src/getting-started/06-workbench.md` or link from examples index.
- Add screenshots later if project policy accepts generated images.
- Mention workbench in README as the “all surfaces together” example.

## 8. Testing Plan

- `cargo check --example workbench` in CI if examples are configured as examples rather than package subcrates.
- If the examples are independent packages, include them in workspace checks.
- Manual QA checklist:
  - open/close menus;
  - open/close dialog;
  - open/close sheet;
  - show all toast intents;
  - switch all toast positions;
  - toggle LTR/RTL;
  - confirm sidebar/sheet/toast placement mirrors.

## 9. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Example becomes too large. | Keep domain fake and tabs small; prefer clarity over completeness. |
| Example tempts feature additions. | Use existing surfaces only. |
| Workbench breaks often. | This is useful; it exposes integration drift. |
| Too many dependencies. | Use only `snora`, `iced`, and existing workspace dependencies. |

## 10. Acceptance Criteria

- `examples/workbench` builds.
- Workbench demonstrates all major current surfaces.
- Workbench includes LTR/RTL toggle.
- Workbench includes toast lifecycle subscription.
- README or docs link to the workbench as the full composition example.
