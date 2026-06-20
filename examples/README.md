# Snora examples

Each directory is an independent Cargo workspace member. Run any example with:

```bash
cargo run -p <crate-name>
```

## Acceptance matrix

The following examples collectively form the release acceptance matrix. Every
example in this table must compile (`cargo check --workspace --all-features`)
before a release is tagged.

| Crate | Purpose | Surface demonstrated |
|---|---|---|
| `snora-example-hello` | Minimal app | `AppLayout::new(body)` + `render` — smallest working Snora app |
| **`snora-example-starter`** | **Recommended starting point** | Header menu, dialog, toast, LTR/RTL toggle, Escape wiring — minimal but complete |
| `snora-example-skeleton` | Full skeleton | Prefab `app_header`, `app_side_bar`, `app_footer`; slot injection |
| `snora-example-header-menu` | Header menus | `Menu` + `header_menu` slot; click-outside close sink |
| `snora-example-context-menu` | Context menus | `context_menu` slot; transparent backdrop |
| `snora-example-dialog` | Dialog overlay | `Dialog`; modal dim; outside-click and button close |
| `snora-example-sheet` | Sheet overlay | `Sheet` at all four `SheetEdge` values; `SheetSize` variants |
| `snora-example-toast` | Toast lifecycle | All five `ToastIntent` values; all six `ToastPosition` anchors; TTL subscription |
| `snora-example-rtl` | ABDD / direction | Live LTR↔RTL toggle; sidebar/sheet/toast anchor mirroring |
| `snora-example-tab` | Tab bar | `TabBar`, `Tab`, `TabAction`; direction-aware tab order |
| `snora-example-breadcrumb` | Breadcrumb | `Crumb::ancestor`/`leaf`; `BreadcrumbAction`; RTL separator flip |
| `snora-example-multi-view` | Sidebar navigation | `SideBar`, `SideBarItem`; active-view highlighting |
| `snora-example-workbench` | **All surfaces together** | Header, sidebar, menus, dialog, sheet, toasts (all intents/positions), tab bar, breadcrumb, Escape wiring, LTR/RTL toggle |
| `snora-example-design-workbench` | **Snora Design visual QA** | Token presets, high contrast, buttons (all variants/states), cards, notices (all tones), chips (filter/removable), progress (tones, indeterminate), typography |

## Manual QA: design workbench

The design workbench is the primary manual regression target for Snora Design.
Run with:

```bash
cargo run -p snora-example-design-workbench
```

QA checklist (inspect visually for all four preset tokens):

- Switch presets (Light / Dark / High-contrast Light / High-contrast Dark).
- Verify all button variants render: primary, secondary, ghost, danger, disabled.
- Verify card variants: surface, raised, selected.
- Verify all notice tones (Info, Success, Warning, Danger, Accent) render
  with legible text; verify dismiss and action buttons are reachable.
- Verify filter chips: selected (solid accent bg) vs unselected state.
- Verify removable chips: label and × button both reachable.
- Verify progress: determinate at various values; indeterminate ("…") suffix
  visible; all six tone variants.
- Inspect typography scale for visible line-height rendering.
- Confirm high-contrast presets show strong border and text contrast.

## Manual QA: workbench

The workbench is the primary manual regression target. See the
[workbench page](../docs/src/getting-started/06-workbench.md) for the
full QA checklist, including:

- open/close File menu and context menu (Escape + outside-click);
- open/close dialog (Escape + backdrop + close button);
- open/close sheet at End edge (mirrors under RTL);
- add toasts for all five intents;
- cycle through all six toast positions;
- toggle LTR↔RTL and verify sidebar/sheet/toast mirror;
- confirm toast dismiss fires above an open dialog.

## Icons gap

No example currently exercises the `lucide-icons` or `svg-icons` optional
features. All examples use `Icon::Text`. The feature-gated icon paths are
covered by the CI feature matrix (`--features widgets,lucide-icons` etc.)
and documented in `docs/src/guides/icons.md`. A dedicated icon example will
be added when the workbench or a downstream app grows icon-feature usage.

## Adding a new example

1. Create `examples/<name>/Cargo.toml` following the pattern of existing
   examples (`publish = false`, `version.workspace = true`).
2. Add the crate to the workspace `Cargo.toml` `members` list.
3. Add a `//!` doc comment at the top of `src/main.rs` listing the surfaces
   demonstrated and the `cargo run` command.
4. Add a row to the acceptance matrix above.
5. If the example is direction-sensitive, complete the
   [ABDD checklist](../docs/src/contributing/abdd-checklist.md).
