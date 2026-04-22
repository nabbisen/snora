# Snora — iced GUI Framework

## 1. Overview

**Snora** is a minimal iced-based GUI framework that provides the
**skeleton** of a desktop application and lets you inject your own UI
and background processing into each slot.

It targets **local-first applications** — tools that run heavy work
(AI inference, local database search, file indexing) alongside an
interactive UI. Snora takes care of the compositional scaffolding
(overlay stacking, backdrop dismissal, toast lifecycles, LTR/RTL
mirroring) so your application code stays focused on state management
and domain logic.

Snora is not a component library and does not try to be. It provides
the minimum UX affordances that a desktop app cannot ship without —
header, sidebar, footer, dialog, bottom sheet, toast, context menu —
and leaves everything else to your application.

## 2. Philosophy

Three commitments shape every API decision:

### 2.1 Accessible by Default and by Design (ABDD)

Accessibility is not an afterthought. Snora's layout is expressed in
**logical edges** (`Edge::Start` / `Edge::End`) rather than physical
directions (left / right). Pick a `LayoutDirection` once at the
framework level and every built-in widget — header controls, sidebar
side, toast anchor, sheet position — mirrors correctly. Applications
supporting Arabic / Hebrew users, or providing a temporary
cognitive-shift setting, do not need to re-author their UI.

### 2.2 Frictionless Build

Unused code is not compiled. Icon backends (`lucide-icons`,
`svg-icons`) are opt-in Cargo features — when disabled, the
corresponding `Icon` variant does not exist in the enum, so dead-code
elimination removes all references at compile time. No bundled asset
blobs you aren't using.

### 2.3 Skeleton + Injected Content

Snora owns the **skeleton** — the slot topology, the z-ordering of
overlays, backdrop installation, toast lifetimes. Your application
owns the **content** of each slot, which is always a plain
`iced::Element`. The framework never forces you to implement a trait
or wrap your view code in a dispatcher enum just to participate. Any
function returning `Element<'a, Message>` is a valid slot provider.

## 3. Design Principles

* **Declarative layout.** `AppLayout` is a plain data structure.
  Building one is a chain of `.new(body).header(…).footer(…)` calls;
  rendering it is a single `render(layout)` call. There is no view
  trait to implement, no associated type to resolve.

* **Vocabulary over flags.** Framework choices that matter
  semantically are expressed as enums, not booleans or magic numbers:
  `ToastIntent`, `ToastLifetime`, `SheetHeight`, `Edge`. You pick from
  a defined set of options; the engine resolves each to pixels and
  colors using the current iced `Theme`.

* **One wiring point per concern.** Outside-click dismissal is wired
  exactly once, via `AppLayout::on_close_menus` and
  `AppLayout::on_close_modals`. Dialogs and bottom sheets do not carry
  their own close messages. Toasts do not need hand-written
  `Subscription` logic — `snora::toast::subscription` + `sweep_expired`
  are the two lines you write.

* **Graceful degradation, not silent drops.** If you populate an
  overlay but leave its `on_close_*` sink `None`, the overlay still
  renders. The engine simply omits the click-outside backdrop, and the
  application is expected to provide an explicit close button inside
  the overlay content. Snora never silently swallows declared UI.

## 4. Architecture

```text
snora-workspace/
├── snora-core/   # Vocabulary & contract layer (no iced dependency)
│                 # - AppLayout, Toast, Dialog, BottomSheet
│                 # - LayoutDirection, Edge, ToastIntent, ToastLifetime,
│                 #   SheetHeight, Icon
│                 # - Menu / MenuItem / MenuAction, SideBar / SideBarItem
│
└── snora/        # Engine layer — iced implementation of the contracts
                  # - render(AppLayout) -> Element
                  # - toast::subscription, toast::sweep_expired
                  # - direction::row_dir, direction::row_dir_three
                  # - widget::{app_header, app_footer, app_side_bar,
                  #   render_menu, icon_element}
```

The dependency arrow is strictly `snora → snora-core`. An alternative
engine (a test double, a WGPU frontend, a WASM/HTML frontend) could be
built against `snora-core` without touching iced.

The split is meaningful, not ornamental:

| Layer | Responsibility | iced dependency |
|-------|----------------|-----------------|
| `snora-core` | What choices exist. The shape of the skeleton. | None |
| `snora` | How those choices become pixels. | Yes |

## 5. Quick Start

### 5.1 Dependencies

```toml
[dependencies]
iced  = { version = "0.14", features = ["tokio"] }
snora = { version = "0.4", features = ["lucide-icons"] }
```

You do not usually depend on `snora-core` directly. The `snora` crate
re-exports the vocabulary, so `use snora::{AppLayout, Toast, ...};` is
the canonical import style.

### 5.2 Minimum app

```rust
use iced::{Element, widget::text};
use snora::{AppLayout, render};

#[derive(Debug, Clone)]
enum Message { Noop }

struct App;

impl App {
    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = text("Hello, Snora!").into();
        render(AppLayout::new(body))
    }
}
```

No traits. No dispatcher enum. No associated types. The `body` is
whatever `Element` your code produced.

### 5.3 Adding slots incrementally

```rust
use snora::{AppLayout, LayoutDirection, render};

let layout = AppLayout::new(self.body())
    .header(self.header())
    .side_bar(self.sidebar())
    .footer(self.footer())
    .direction(LayoutDirection::Ltr)
    .on_close_menus(Message::CloseMenus)
    .on_close_modals(Message::CloseModals);

render(layout)
```

Every setter is chainable and takes the raw slot content (an
`Element`, a `Dialog`, a `BottomSheet`, or a `Vec<Toast>`). You can
also construct `AppLayout` via direct struct-literal syntax — its
fields are all `pub` — but the builder is the canonical path.

### 5.4 Toasts with lifecycle management

```rust
use std::time::Instant;
use iced::{Subscription, Task};
use snora::{Toast, ToastIntent, ToastLifetime};

struct App {
    toasts: Vec<Toast<Message>>,
    next_id: u64,
    // …
}

#[derive(Debug, Clone)]
enum Message {
    ToastTick,
    DismissToast(u64),
    ShowSaved,
    // …
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        snora::toast::subscription(&self.toasts, || Message::ToastTick)
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ToastTick => {
                snora::toast::sweep_expired(&mut self.toasts, Instant::now());
            }
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
            }
            Message::ShowSaved => {
                let id = self.next_id;
                self.next_id += 1;
                self.toasts.push(
                    Toast::new(
                        id,
                        ToastIntent::Success,
                        "Saved",
                        "Document saved to disk.",
                        Message::DismissToast(id),
                    )
                    .with_lifetime(ToastLifetime::seconds(3)),
                );
            }
        }
        Task::none()
    }
}
```

Three observations:

* Toast TTL is framework-owned. The app only calls
  `snora::toast::subscription` and `snora::toast::sweep_expired` —
  both one-liners.
* The `on_dismiss` message is fired when the user clicks the toast's
  close button. Expiration is silent (no message is sent).
* For persistent toasts (errors that must be acknowledged), call
  `.persistent()` instead of `.with_lifetime(...)`.

### 5.5 Dialogs and bottom sheets

```rust
use snora::{AppLayout, BottomSheet, Dialog, SheetHeight};

let dialog_content: Element<'_, Message> = /* your card element */;
let sheet_content:  Element<'_, Message> = /* your drawer content */;

let layout = AppLayout::new(body)
    .dialog(Dialog::new(dialog_content))
    .bottom_sheet(
        BottomSheet::new(sheet_content)
            .with_height(SheetHeight::Half),
    )
    .on_close_modals(Message::CloseModals);
```

Both overlays share the same dim backdrop (painted once by the
engine). Clicking outside fires `Message::CloseModals`. There is no
`on_outside_click` field on `Dialog` or `BottomSheet` — the one sink
at `AppLayout` level is authoritative.

### 5.6 Direction-aware custom widgets

Use `snora::direction::row_dir` (or `row_dir_three`) anywhere you
would otherwise write `row![left, right]`. The helper resolves the
order from the application's `LayoutDirection`:

```rust
use snora::direction::row_dir;

let bar = row_dir(
    self.direction,
    text("File: untitled"),                  // start
    button("Save").on_press(Message::Save),  // end
);
```

For built-in snora widgets (`app_header`, `app_side_bar`), pass the
direction as an argument — they mirror accordingly.

## 6. Reference

### 6.1 `AppLayout`

The application skeleton. Fields:

| Field | Type | Notes |
|-------|------|-------|
| `body` | `Node` | **Required.** Main content. |
| `header`, `side_bar`, `footer` | `Option<Node>` | Skeleton slots. |
| `header_menu`, `context_menu` | `Option<Node>` | Light overlays above the skeleton. |
| `dialog` | `Option<Dialog<Node, Message>>` | Modal card. |
| `bottom_sheet` | `Option<BottomSheet<Node, Message>>` | Modal drawer. |
| `toasts` | `Vec<Toast<Message>>` | Bottom-end stack. |
| `direction` | `LayoutDirection` | LTR / RTL. |
| `on_close_menus` | `Option<Message>` | Outside-click sink for light overlays. |
| `on_close_modals` | `Option<Message>` | Outside-click sink for modals. |

Canonical construction is via `AppLayout::new(body)` plus chained
setters. Direct struct-literal syntax is also supported.

### 6.2 `Toast` and lifetime helpers

| API | Purpose |
|-----|---------|
| `Toast::new(id, intent, title, msg, on_dismiss)` | Build with default 4s transient lifetime. |
| `.with_lifetime(ToastLifetime::seconds(n))` | Override duration. |
| `.persistent()` | No auto-dismiss; user must click close. |
| `Toast::is_expired(now)` | Pure expiry check. |
| `snora::toast::subscription(&toasts, || msg)` | Periodic tick while transient toasts exist. |
| `snora::toast::sweep_expired(&mut toasts, now)` | Drop expired entries. |

`ToastIntent` (`Debug` / `Info` / `Success` / `Warning` / `Error`)
maps to colors via the current iced `Theme`. Warning uses a stable
orange because iced's extended palette does not expose a warning pair.

### 6.3 `Dialog` and `BottomSheet`

* `Dialog::new(content)` — centers `content` in the window.
* `BottomSheet::new(content)` — drawer at `SheetHeight::OneThird`.
* `BottomSheet::with_height(SheetHeight::…)` — pick from
  `OneThird`, `Half`, `TwoThirds`, `Ratio(f32)`, or `Pixels(f32)`.

Neither carries a close message. Wire `on_close_modals` on
`AppLayout` once.

### 6.4 `LayoutDirection` and `Edge`

* `LayoutDirection::{Ltr, Rtl}` — the framework-level setting.
* `LayoutDirection::flipped()` — toggle.
* `Edge::{Start, End}` — logical position along the primary axis.
* `Edge::is_left_under(direction)` — physical resolution when needed.

See `snora::direction::row_dir` for the day-to-day helper.

### 6.5 `Icon`

Three variants, each gated by a feature:

```rust
Icon::Text(String)                       // always available
Icon::Lucide(lucide_icons::Icon)         // feature = "lucide-icons"
Icon::Svg(PathBuf)                       // feature = "svg-icons"
```

`From<&str>` and `From<String>` are implemented for the `Text`
variant, so `icon: "★".into()` works even with every feature off.

### 6.6 `Menu` / `SideBar`

Application-defined `MenuId` and `ViewId` enum types parameterise the
menus and sidebar. The engine renders them as data; interaction comes
back as `MenuAction<MenuId, MenuItemId>` or the `SideBarItem::on_press`
message respectively.

Use `snora::widget::app_header` and `snora::widget::app_side_bar` for
the prefab renderers, or build your own chrome and place it in the
relevant `AppLayout` slot — both paths are equally supported.

## 7. Examples

Snora ships a gallery of small, self-contained example binaries. Each
one focuses on a **single UI/UX theme** so you can read just the one
you care about and copy it into your own app without picking apart a
monolithic showcase.

All examples live under `examples/` as independent crates. Run any one
with:

```text
cargo run -p snora-example-<name>
```

| Name | Theme | What it demonstrates |
|------|-------|----------------------|
| `hello` | Minimum-viable app | `AppLayout::new(body)` + `render`. No header, no footer, no overlays. |
| `skeleton` | Four-slot chrome | Prefab `app_header` / `app_side_bar` / `app_footer` composed with a hand-written body. |
| `toast` | Notifications | All five `ToastIntent` colors and all three `ToastLifetime` policies, with framework-owned TTL (`snora::toast::subscription` + `sweep_expired`). |
| `dialog` | Modal card | `Dialog::new` + a single `on_close_modals` sink driving click-outside dismissal. |
| `bottom_sheet` | Drawers | Every `SheetHeight` variant (`OneThird`, `Half`, `TwoThirds`, `Ratio`, `Pixels`). |
| `context_menu` | Floating menus | `context_menu` slot + `on_close_menus` transparent backdrop. Menu and modal close channels are independent. |
| `header_menu` | Drop-down menu bar | `Menu` / `MenuItem` / `MenuAction` with application-defined id enums. |
| `multi_view` | Sidebar-driven navigation | `SideBar` with a `ViewId` enum; each view is a plain function returning `Element`. |
| `rtl` | LTR ↔ RTL flip | Framework-wide direction switch; header, sidebar, toast anchor, and custom `row_dir` rows all mirror together. |

Examples are deliberately **not glued together** by a shared helper
crate. Each one is the smallest complete program that exercises its
theme — so the file you open is the tutorial for that feature.

## 8. Migration from 0.3.x

Breaking changes in 0.4:

| 0.3.x | 0.4 |
|-------|-----|
| `PageContract` trait | Removed. Overlays are `AppLayout` fields. |
| `AppLayout<P, Message, MenuId>` with single `P` for all slots | `AppLayout<Node, Message>` — each slot is a raw `Node`. The Section-enum workaround is no longer needed. |
| `render_app(layout, on_close_menus, on_close_modals)` | `render(layout)` — close sinks are fields on the layout. |
| `Dialog.on_outside_click`, `BottomSheet.on_close` | Removed. Use `AppLayout::on_close_modals`. |
| `BottomSheet` hardcoded to 1/3 height, black background | `BottomSheet::with_height(SheetHeight::…)`, theme-aware background. |
| Toast intent ignored in rendering | Intent drives toast color via the theme. |
| Toast position hardcoded bottom-right | Anchors at bottom-end (RTL-aware). |
| Toast TTL handled in application code | `snora::toast::{subscription, sweep_expired}` + `Toast::is_expired`. |
| `PageLayout`, `render_page`, `AppLayout.menu_id` | Removed (dead code). |
| `AppSideBar` renamed to `SideBar`; fields `view_id: ViewId` → `active: ViewId`, `action` → `on_press` | Clearer semantics. |
