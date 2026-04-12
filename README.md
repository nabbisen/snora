# Snora Framework Documentation

## 1. Overview
**Snora** is a declarative and structured UI framework built atop the pure Rust GUI library `iced`.

It is designed for **local-first applications**, particularly those running heavy background processes (such as AI inference or local database searches), keeping the UI thread clean and allowing developers to focus solely on "state management" and the "logical structure of layouts."

## 2. Philosophy
Snora is grounded in three philosophies aimed at removing "friction" for both developers and users:

*   **Accessible by Default and by Design (ABDD)**
    Accessibility is not an afterthought. Snora supports layout switching not only from Left-to-Right (LTR) but also Right-to-Left (RTL) at the foundational level of the framework. By constructing UIs using logical "Start" and "End" rather than physical "Left" and "Right," it naturally adapts to users of any language region or environments requiring temporary cognitive shifts.
*   **Frictionless Architecture**
    We do not tolerate bloated build times caused by massive `match` statements or unused icon assets. Required icons and features are selectively opted-in via Cargo `feature` flags, maximizing the compiler's Dead Code Elimination (DCE).
*   **Pure Separation of Concerns**
    We strictly separate "how data should be" (Contract) from "how it should be rendered" (Render). This makes testing domain logic extremely straightforward.

## 3. Design Principles

*   **Separation of Core and Render**: The `snora-core` crate has **no dependency** on `iced`. It consists purely of Rust enums and structs, defining the "contracts" of the UI.
*   **Coexistence of Flexibility and Strict Typing**: For aspects like icon specification, we adopt a design that allows intuitive fallbacks (e.g., `Option<String>`) even when features are disabled, reducing developer typing burden through aggressive use of `From<&str>` implementations.
*   **Unified Feedback Loop**: Standard mechanisms like `Toast` and `Footer` logging ensure asynchronous processing status and system errors (Failures) are notified to the user with a consistent design.

## 4. Structure
Snora is organized as a workspace divided into the following layers:

```
snora-workspace/
├── snora-core/   # [Contract Layer] Data structures for UI components (LayoutDirection, Icon, Toast)
└── snora/        # [Render Layer] Conversion to physical pixels and layout using iced
```

The dependency arrow always points `snora` → `snora-core`, never the reverse.

---

## 5. Tutorial: Your First Snora App
Here is how to build a simple application with Snora that supports RTL layouts.

### Step 1: Add Dependencies
Add `snora` to your app's `Cargo.toml` and enable the necessary features (here, Lucide icons).

```toml
[dependencies]
iced = { version = "0.14", features = ["tokio"] }
snora = { version = "0", features = ["lucide-icons"] }
```

### Step 2: Define Menus and Icons
Construct type-safe menu items with icons via `snora::icons`. Thanks to feature flags, you don't need to declare dependencies directly.

```rust
use snora::{Icon, MenuItem, icons};

let items = vec![
    MenuItem {
        label: "Settings".into(),
        icon: Some(Icon::Lucide(icons::Settings)), // Lucide icon
        action: Some(Message::OpenSettings),
    },
    MenuItem {
        label: "Home".into(),
        icon: Some("🏠".into()), // Automatic conversion from string
        action: Some(Message::GoHome),
    },
];
```

### Step 3: Determine Logical Layout Direction
Specify whether the layout is RTL (Right-to-Left) or LTR (Left-to-Right). A single flag optimizes sidebar positioning and element ordering automatically.

```rust
use snora::LayoutDirection;

let direction = LayoutDirection::Rtl; // For Arabic support or UI inversion testing
```

### Step 4: Assemble and Render Pages
Map widgets for each area into the `PageLayout` struct and pass it to the `build_layout` function. All elements will be placed logically correctly.

```rust
use snora::PageLayout;
use snora::components::header::app_header;
use snora::layout::build_layout;

// Generate header
let header = app_header("My Snora App", items, None);

// Define skeleton
let layout = PageLayout {
    direction,
    header: Some(header),
    body: main_content_widget,
    aside: Some(sidebar_widget),
    footer: None,
    dialog: None,
    bottom_sheet: None,
    toasts: vec![],
};

// Final conversion to iced::Element
build_layout(layout)
```

We hope this robust and scalable UI foundation serves you well.
