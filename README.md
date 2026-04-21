# Snora iced-based GUI Framework Documentation

## 1. Overview
**Snora** is a declarative and structured UI framework built atop the pure Rust GUI library `iced`.

It is designed for **local-first applications**, particularly those running heavy background processes (such as AI inference or local database searches), keeping the UI thread clean and allowing developers to focus solely on "state management" and the "logical structure of layouts."

## 2. Philosophy
Snora is grounded in three philosophies aimed at removing "friction" for both developers and users:

* **Accessible by Default and by Design (ABDD)**
    Accessibility is not an afterthought. Snora supports layout switching not only from Left-to-Right (LTR) but also Right-to-Left (RTL) at the foundational level of the framework. By constructing UIs using logical "Start" and "End" rather than physical "Left" and "Right," it naturally adapts to users of any language region or environments requiring temporary cognitive shifts.
* **Frictionless Architecture**
    We do not tolerate bloated build times caused by massive `match` statements or unused icon assets. Required icons and features are selectively opted-in via Cargo `feature` flags, maximizing the compiler's Dead Code Elimination (DCE).
* **Pure Separation of Concerns (Logical Layouts)**
    We strictly separate "what a page is" (Contract) from "where it is placed" (Layout) and "how it is rendered" (Render). Snora defers UI instantiation (reification) to the deepest layer of the framework, ensuring domain logic remains completely decoupled from physical pixel rendering.

## 3. Design Principles

* **Trait-Driven Rendering (`PageContract`)**: The `snora-core` crate has **no dependency** on `iced`. UI elements implement the `PageContract` trait, declaring their visual output (`view()`) and overlay behaviors (`dialog()`, `toasts()`, etc.). The framework trusts this contract and handles the heavy lifting of composition.
* **Coexistence of Flexibility and Strict Typing**: For aspects like icon specification, we adopt a design that allows intuitive fallbacks (e.g., `Option<String>`) even when features are disabled, reducing developer typing burden through aggressive use of `From<&str>` implementations.
* **Unified Feedback Loop**: Standard mechanisms like `Toast`, `Dialog`, and `BottomSheet` overlays are inherently tied to the page contract, ensuring asynchronous processing status and system errors are naturally bubbled up and rendered with consistent design.

## 4. Structure
Snora is organized as a workspace divided into the following layers:

```text
snora-workspace/
├── snora-core/   # [Contract Layer] Traits (PageContract) and structural definitions (AppLayout, LayoutDirection)
└── snora/        # [Render Layer] The engine that consumes AppLayout and renders physical pixels using iced
```

The dependency arrow always points `snora` → `snora-core`, never the reverse.

---

## 5. Tutorial: Your First Snora App
Here is how to build a simple application leveraging Snora's `PageContract` and logical layout engine.

### Step 1: Add Dependencies
Add `snora` to your app's `Cargo.toml` and enable the necessary features.

```toml
[dependencies]
iced = { version = "0.14", features = ["tokio"] }
snora-core = { version = "0" }
snora = { version = "0", features = ["lucide-icons"] }
```

### Step 2: Implement the Page Contract
Define your component or page by implementing `PageContract`. This tells Snora exactly how to render the node and handle its specific overlays or events.

```rust
use iced::{Element, widget::text};
use snora_core::contract::page::PageContract;

// Your domain message
#[derive(Clone)]
pub enum Message {
    CloseMenus,
}

pub struct MyPage;

impl PageContract for MyPage {
    type Node = Element<'static, Message>;
    type Message = Message;

    // The single source of truth for visual representation
    fn view(&self) -> Self::Node {
        text("Hello, Snora Frictionless Architecture!").into()
    }

    // Optional: Let the page define how to close backdrops
    fn on_close_menus(&self) -> Option<Self::Message> {
        Some(Message::CloseMenus)
    }
}
```

### Step 3: Construct the Logical AppLayout
Place your page objects into the `AppLayout`. Notice that you are passing the **objects themselves**, not pre-rendered elements. Snora handles the RTL/LTR layout mathematically.

```rust
use snora_core::contract::app::AppLayout;
use snora_core::contract::rtl::LayoutDirection;

let direction = LayoutDirection::Rtl; // Instantly mirror the UI for Arabic support

let layout = AppLayout {
    direction,
    header: None,
    body: MyPage, // Fulfills PageContract
    side_bar: None,
    footer: None,
    header_menu: None,
    context_menu: None,
    toasts: vec![],
    dialog: None,
    bottom_sheet: None,
    menu_id: None::<()>,
};
```

### Step 4: Render the App
Pass the layout to the rendering engine. Snora will lazily evaluate `.view()` and automatically wire up backdrop clicks and overlays based on the provided contracts.

```rust
use snora::layout::app::render_app;

// Final conversion to iced::Element
let ui = render_app(
    layout, 
    layout.body.on_close_menus(), // Autonomously gather close intentions
    None
);
```

We hope this robust, strict, and scalable UI foundation serves you well.
