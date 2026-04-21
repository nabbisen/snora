//! The top-level view: assemble an `AppLayout` and hand it to `render_app`.
//!
//! This is the one place in the app that actually speaks to snora's render
//! engine. Everywhere else either mutates state (`update.rs`) or builds small
//! `Element`s (`chrome.rs`, `pages.rs`, `overlay.rs`).
//!
//! The pattern here is: wrap the four layout slots in `Section`, pull every
//! overlay surface off the body via `PageContract`, and feed the whole lot to
//! `render_app`. The framework then owns composition (RTL mirror, backdrop
//! layering, toast stacking, etc.) and we just declare what each surface is.

use iced::{
    Element,
    widget::{container, space},
};
use snora::{AppLayout, PageContract, layout::app::render_app};

use super::{
    App, Message,
    section::{self, Section},
};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // Four sections, one `PageContract` shape — which is what `AppLayout`
        // requires. The role tag inside each Section dispatches to the right
        // slot-specific renderer.
        let header = Section::header(self);
        let body = Section::body(self);
        let sidebar = Section::sidebar(self);
        let footer = Section::footer(self);

        // Pull every "extra" surface off the body per the framework contract.
        // These borrow only from `self` (via `body.app: &'a App`), not from
        // `body` itself, so moving `body` into the layout below is fine: the
        // returned `Element`s outlive `body` because their lifetime is tied to
        // the App reference, not to Section.
        let dialog = body.dialog();
        let context_menu = body.context_menu();
        let toasts = body.toasts();
        let on_close_menus = body.on_close_menus();
        let on_close_modals = body.on_close_modals();
        let bottom_sheet = section::bottom_sheet(self);

        // The header dropdown renders *inline* inside the header itself —
        // snora's `render_menu` appends items under the menu button when its
        // id matches `active_menu_id`. But we still want the framework's
        // transparent click-outside backdrop installed so an outside click
        // closes the menu, so we hand `header_menu` a zero-size stand-in
        // whenever a menu is open. `render_app` keys off `Some(_)`, not on
        // what the element actually paints. We wrap `space()` in a container
        // to match framework idiom (`container(space()).into()`, see
        // `layout/app.rs`) and dodge any Into-inference edge cases.
        let header_menu: Option<Element<'_, Message>> = self
            .active_menu_id
            .as_ref()
            .map(|_| container(space()).into());

        let layout = AppLayout {
            header: Some(header),
            body,
            side_bar: Some(sidebar),
            footer: Some(footer),
            header_menu,
            context_menu,
            toasts,
            dialog,
            bottom_sheet,
            direction: self.direction,
            menu_id: self.active_menu_id.clone(),
        };

        render_app(layout, on_close_menus, on_close_modals)
    }
}
