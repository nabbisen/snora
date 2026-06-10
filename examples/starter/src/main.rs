//! # Example: starter
//!
//! The recommended starting point for a new Snora application.
//! Demonstrates the minimal patterns every Snora app needs:
//!
//! * `AppLayout` assembled from prefab widgets + your own slots.
//! * Header menu with click-outside close sink.
//! * Dialog with button close and Escape dismissal.
//! * Transient toast added by a button.
//! * Live LTR ↔ RTL toggle (ABDD).
//! * Tab bar for simple view switching.
//! * `snora::keyboard::dismiss_on_escape` for keyboard handling.
//!
//! The workbench example (`snora-example-workbench`) covers every surface
//! in depth. This starter stays small so you can read it in one sitting.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-starter
//! ```

use std::time::Instant;

use iced::{Alignment::Center, Element, Length, Subscription, Task, widget::{button, column, container, row, space, text}};
use snora::{
    AppLayout, Dialog, LayoutDirection, Menu, MenuAction, SideBar, SideBarItem,
    Tab, TabAction, TabBar, Toast, ToastIntent, ToastPosition,
    render,
    widget::{app_footer, app_header, app_side_bar, app_tab_bar},
};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View { Home, Settings }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuId { File }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum MenuItemId {}

struct App {
    // ---- Snora-managed state ----
    direction:    LayoutDirection,   // Snora uses this to mirror the skeleton
    // ---- App-managed overlay state ----
    menu_open:    bool,
    show_dialog:  bool,
    // ---- App-managed navigation ----
    view:         View,
    // ---- App-managed toasts ----
    toasts:       Vec<Toast<Message>>,
    next_id:      u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            direction:   LayoutDirection::Ltr,
            menu_open:   false,
            show_dialog: false,
            view:        View::Home,
            toasts:      Vec::new(),
            next_id:     1,
        }
    }
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Message {
    // ---- Snora overlay plumbing ----
    HeaderAction(MenuAction<MenuId, MenuItemId>),
    CloseMenus,
    OpenDialog,
    CloseModals,
    // ---- Navigation ----
    SelectTab(TabAction<View>),
    // ---- Direction ----
    ToggleDirection,
    // ---- Toasts ----
    AddToast,
    DismissToast(u64),
    ToastTick,
    // ---- Keyboard ----
    KeyPressed(iced::keyboard::Key),
    NoOp,
}

// ---------------------------------------------------------------------------
// Update — app logic lives here, not in view()
// ---------------------------------------------------------------------------

impl App {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // Open the File menu (app clears it before opening a modal).
            Message::HeaderAction(MenuAction::MenuPressed(MenuId::File)) => {
                if !self.show_dialog { self.menu_open = !self.menu_open; }
            }
            Message::HeaderAction(_) => {}
            Message::CloseMenus  => self.menu_open = false,
            // Law 2: clear menus before opening a modal.
            Message::OpenDialog  => { self.menu_open = false; self.show_dialog = true; }
            Message::CloseModals => self.show_dialog = false,
            Message::SelectTab(TabAction::Pressed(v)) => self.view = v,
            Message::ToggleDirection => self.direction = self.direction.flipped(),
            Message::AddToast => {
                let id = self.next_id; self.next_id += 1;
                self.toasts.push(Toast::new(id, ToastIntent::Info, "Hello", "Toast from the starter.", Message::DismissToast(id)));
            }
            Message::DismissToast(id) => self.toasts.retain(|t| t.id != id),
            Message::ToastTick => snora::toast::sweep_expired(&mut self.toasts, Instant::now()),
            // Escape: dismiss modal first, then menu (modal priority).
            Message::KeyPressed(key) => {
                if let Some(m) = snora::keyboard::dismiss_on_escape(
                    self.show_dialog, self.menu_open,
                    Some(Message::CloseModals), Some(Message::CloseMenus), key,
                ) { return self.update(m); }
            }
            Message::NoOp => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            snora::toast::subscription(&self.toasts, || Message::ToastTick),
            iced::keyboard::listen().map(|ev| {
                if let iced::keyboard::Event::KeyPressed { key, .. } = ev {
                    Message::KeyPressed(key)
                } else { Message::NoOp }
            }),
        ])
    }
}

// ---------------------------------------------------------------------------
// View — assemble the AppLayout and hand it to snora::render
// ---------------------------------------------------------------------------

impl App {
    fn view(&self) -> Element<'_, Message> {
        // Snora prefab header. Owns the File menu open/close interaction.
        let header = app_header(
            "My App",
            vec![Menu { id: MenuId::File, label: "File".into(), icon: None, items: vec![] }],
            &Message::HeaderAction,
            if self.menu_open { Some(&MenuId::File) } else { None },
            Some(row![
                button(text("Toggle RTL").size(13)).on_press(Message::ToggleDirection).padding([4,10]),
                button(text("Open dialog").size(13)).on_press(Message::OpenDialog).padding([4,10]),
                button(text("Add toast").size(13)).on_press(Message::AddToast).padding([4,10]),
            ].spacing(6).into()),
            self.direction,
        );

        // Snora prefab sidebar. Direction is passed so it mirrors under RTL.
        let sidebar = app_side_bar(
            SideBar {
                items: vec![
                    SideBarItem { view_id: View::Home,     icon: "🏠".into(), tooltip: "Home".into(),     on_press: Message::SelectTab(TabAction::Pressed(View::Home)) },
                    SideBarItem { view_id: View::Settings, icon: "⚙".into(),  tooltip: "Settings".into(), on_press: Message::SelectTab(TabAction::Pressed(View::Settings)) },
                ],
                active: self.view,
            },
            self.direction,
        );

        // Body: tab bar + per-tab content.
        let tabs = app_tab_bar(
            TabBar {
                tabs: vec![
                    Tab { id: View::Home,     label: "Home".into(),     icon: None },
                    Tab { id: View::Settings, label: "Settings".into(), icon: None },
                ],
                active: self.view,
            },
            &Message::SelectTab,
            self.direction,
        );

        let body_content: Element<'_, Message> = match self.view {
            View::Home =>
                column![text("Home view").size(20), text("Use the header to open menus, dialog, and toasts."), text("Press Escape to close overlays.")]
                    .spacing(8).into(),
            View::Settings =>
                column![text("Settings view").size(20), text("Your app settings would live here.")]
                    .spacing(8).into(),
        };

        let body = container(column![tabs, body_content].spacing(12).padding(16))
            .width(Length::Fill).height(Length::Fill).into();

        // Snora prefab footer.
        let dir = match self.direction { LayoutDirection::Ltr => "LTR", LayoutDirection::Rtl => "RTL" };
        let footer = app_footer(
            row![text(format!("Dir: {dir}  |  Toasts: {}", self.toasts.len())).size(13),
                 container(space()).width(Length::Fill)]
            .align_y(Center).into(),
        );

        // Dialog content (app-owned: close button is required).
        let dialog_el: Element<'_, Message> = container(column![
            text("Dialog").size(18),
            text("Close with the button below or press Escape."),
            row![space().width(Length::Fill), button(text("Close").size(13)).on_press(Message::CloseModals).padding([4,10])],
        ].spacing(12).padding(24)).width(380).into();

        // File menu overlay (app-owned element).
        let menu_el: Element<'_, Message> = container(column![
            button(text("New").size(13)).on_press(Message::CloseMenus).padding([4,8]).width(Length::Fill),
            button(text("Close menu").size(13)).on_press(Message::CloseMenus).padding([4,8]).width(Length::Fill),
        ].spacing(2).padding(4))
        .style(|t: &iced::Theme| {
            let p = t.extended_palette();
            iced::widget::container::Style {
                background: Some(iced::Background::Color(p.background.base.color)),
                border: iced::Border { width: 1.0, color: p.background.weak.color, radius: 4.0.into() },
                ..Default::default()
            }
        })
        .into();

        // Assemble AppLayout — every field is optional; add what you need.
        let mut layout = AppLayout::new(body)
            .header(header)
            .side_bar(sidebar)
            .footer(footer)
            .toasts(self.toasts.clone())
            .toast_position(ToastPosition::TopEnd)
            .direction(self.direction)           // ← drives all mirroring
            .on_close_menus(Message::CloseMenus) // ← outside-click close sink
            .on_close_modals(Message::CloseModals);

        if self.menu_open    { layout = layout.header_menu(menu_el); }
        if self.show_dialog  { layout = layout.dialog(Dialog::new(dialog_el)); }

        render(layout)  // ← the single Snora entry point
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Snora Starter")
        .subscription(App::subscription)
        .run()
}
