//! # Example: workbench
//!
//! The **Snora Workbench** exercises all major framework surfaces in a
//! single application. Use it as a manual QA reference, composition proof,
//! and dogfood target for z-stack behavior.
//!
//! Surfaces demonstrated:
//! - Skeleton: header, sidebar, body, footer.
//! - Overlays: File menu, context menu, dialog, bottom sheet.
//! - Toasts: all five intents, all six positions, TTL lifecycle.
//! - Direction: live LTR ↔ RTL toggle (ABDD demonstration).
//! - Navigation: tab bar, breadcrumb.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-workbench
//! ```

use std::time::Instant;

use iced::{
    Alignment::Center,
    Element, Length, Subscription, Task,
    widget::{button, column, container, row, space, text},
};
use snora::{
    AppLayout, BreadcrumbAction, Crumb, Dialog, LayoutDirection,
    Menu, MenuAction, Sheet, SheetEdge, SheetSize,
    SideBar, SideBarItem, Tab, TabAction, TabBar, Toast, ToastIntent, ToastPosition,
    render,
    widget::{app_breadcrumb, app_footer, app_header, app_side_bar, app_tab_bar},
};

// ---------------------------------------------------------------------------
// State types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab { Overview, OverlayLab, ToastLab, DirectionLab }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SideView { Home, Layout, Toasts, Direction }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuId { File }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants reserved for menu item expansion
enum MenuItemId { New, Close }

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Message {
    SelectTab(TabAction<ActiveTab>),
    SelectView(BreadcrumbAction<SideView>),
    SidebarPressed(SideView),
    HeaderAction(MenuAction<MenuId, MenuItemId>),
    ToggleDirection,
    OpenContextMenu,
    CloseMenus,
    OpenDialog,
    CloseModals,
    OpenSheet,
    AddToast(ToastIntent),
    DismissToast(u64),
    SetToastPosition(ToastPosition),
    ToastTick,
    KeyPressed(iced::keyboard::Key),
    NoOp,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct Workbench {
    direction:        LayoutDirection,
    active_tab:       ActiveTab,
    side_view:        SideView,
    file_menu_open:   bool,
    context_menu_open:bool,
    show_dialog:      bool,
    show_sheet:       bool,
    toasts:           Vec<Toast<Message>>,
    next_toast_id:    u64,
    toast_position:   ToastPosition,
}

impl Default for Workbench {
    fn default() -> Self {
        Self {
            direction:         LayoutDirection::Ltr,
            active_tab:        ActiveTab::Overview,
            side_view:         SideView::Home,
            file_menu_open:    false,
            context_menu_open: false,
            show_dialog:       false,
            show_sheet:        false,
            toasts:            Vec::new(),
            next_toast_id:     1,
            toast_position:    ToastPosition::TopEnd,
        }
    }
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

impl Workbench {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectTab(TabAction::Pressed(tab)) => self.active_tab = tab,
            Message::SelectView(BreadcrumbAction::Pressed(v)) => self.side_view = v,
            Message::SidebarPressed(v) => self.side_view = v,
            Message::HeaderAction(MenuAction::MenuPressed(MenuId::File)) => {
                if !self.show_dialog && !self.show_sheet {
                    self.file_menu_open = !self.file_menu_open;
                    self.context_menu_open = false;
                }
            }
            Message::HeaderAction(_) => {}
            Message::ToggleDirection => {
                self.direction = self.direction.flipped();
            }
            Message::OpenContextMenu => {
                if !self.show_dialog && !self.show_sheet {
                    self.context_menu_open = !self.context_menu_open;
                    self.file_menu_open = false;
                }
            }
            Message::CloseMenus => {
                self.file_menu_open = false;
                self.context_menu_open = false;
            }
            Message::OpenDialog => {
                // Law 2: clear menus before opening a modal.
                self.file_menu_open = false;
                self.context_menu_open = false;
                self.show_dialog = true;
            }
            Message::OpenSheet => {
                self.file_menu_open = false;
                self.context_menu_open = false;
                self.show_sheet = true;
            }
            Message::CloseModals => {
                self.show_dialog = false;
                self.show_sheet = false;
            }
            Message::AddToast(intent) => {
                let id = self.next_toast_id;
                self.next_toast_id += 1;
                self.toasts.push(Toast::new(
                    id, intent,
                    format!("{intent} toast"),
                    "Added from workbench.",
                    Message::DismissToast(id),
                ));
            }
            Message::DismissToast(id) => self.toasts.retain(|t| t.id != id),
            Message::NoOp => {}
            Message::SetToastPosition(pos) => self.toast_position = pos,
            Message::ToastTick => {
                snora::toast::sweep_expired(&mut self.toasts, Instant::now());
            }
            // Escape key: dismiss modals first, then menus — via the helper.
            Message::KeyPressed(key) => {
                let has_menu = self.file_menu_open || self.context_menu_open;
                if let Some(msg) = snora::keyboard::dismiss_on_escape(
                    self.show_dialog || self.show_sheet,
                    has_menu,
                    Some(Message::CloseModals),
                    Some(Message::CloseMenus),
                    key,
                ) {
                    return self.update(msg);
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let toast_sub = snora::toast::subscription(&self.toasts, || Message::ToastTick);
        // Listen to all keyboard events; filter to key-press in update.
        let key_sub = iced::keyboard::listen().map(|event| {
            if let iced::keyboard::Event::KeyPressed { key, .. } = event {
                Message::KeyPressed(key)
            } else {
                Message::NoOp
            }
        });
        Subscription::batch([toast_sub, key_sub])
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

impl Workbench {
    fn view(&self) -> Element<'_, Message> {
        let dir_label = match self.direction {
            LayoutDirection::Ltr => "→ RTL",
            LayoutDirection::Rtl => "→ LTR",
        };

        let header = app_header(
            "Snora Workbench",
            vec![Menu { id: MenuId::File, label: "File".into(), icon: None, items: vec![] }],
            &Message::HeaderAction,
            if self.file_menu_open { Some(&MenuId::File) } else { None },
            Some(row![
                btn(dir_label, Message::ToggleDirection),
                btn("Dialog", Message::OpenDialog),
                btn("Sheet", Message::OpenSheet),
                btn("Context menu", Message::OpenContextMenu),
            ].spacing(6).into()),
            self.direction,
        );

        let sidebar = app_side_bar(
            SideBar {
                items: vec![
                    SideBarItem { view_id: SideView::Home,      icon: "🏠".into(), tooltip: "Home".into(),      on_press: Message::SidebarPressed(SideView::Home) },
                    SideBarItem { view_id: SideView::Layout,    icon: "⬜".into(), tooltip: "Layout".into(),    on_press: Message::SidebarPressed(SideView::Layout) },
                    SideBarItem { view_id: SideView::Toasts,    icon: "🔔".into(), tooltip: "Toasts".into(),    on_press: Message::SidebarPressed(SideView::Toasts) },
                    SideBarItem { view_id: SideView::Direction, icon: "↔".into(),  tooltip: "Direction".into(), on_press: Message::SidebarPressed(SideView::Direction) },
                ],
                active: self.side_view,
            },
            self.direction,
        );

        let breadcrumb = app_breadcrumb(
            vec![
                Crumb::ancestor(SideView::Home, "Home"),
                Crumb::leaf(self.side_view, format!("{:?}", self.side_view)),
            ],
            &Message::SelectView,
            self.direction,
        );

        let tabs = app_tab_bar(
            TabBar {
                tabs: vec![
                    Tab { id: ActiveTab::Overview,     label: "Overview".into(),     icon: None },
                    Tab { id: ActiveTab::OverlayLab,   label: "Overlay Lab".into(),  icon: None },
                    Tab { id: ActiveTab::ToastLab,     label: "Toast Lab".into(),    icon: None },
                    Tab { id: ActiveTab::DirectionLab, label: "Direction Lab".into(),icon: None },
                ],
                active: self.active_tab,
            },
            &Message::SelectTab,
            self.direction,
        );

        let tab_body: Element<'_, Message> = match self.active_tab {
            ActiveTab::Overview     => self.tab_overview(),
            ActiveTab::OverlayLab   => self.tab_overlay_lab(),
            ActiveTab::ToastLab     => self.tab_toast_lab(),
            ActiveTab::DirectionLab => self.tab_direction_lab(),
        };

        let body = container(
            column![breadcrumb, tabs, tab_body].spacing(8).padding(16),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        let dir_str = match self.direction { LayoutDirection::Ltr => "LTR", LayoutDirection::Rtl => "RTL" };
        let overlay_str = match (self.show_dialog, self.show_sheet) {
            (true, true)   => "dialog+sheet",
            (true, false)  => "dialog",
            (false, true)  => "sheet",
            (false, false) => "none",
        };
        let footer = app_footer(
            row![
                text(format!("Dir: {dir_str}  |  Overlays: {overlay_str}  |  Toasts: {}", self.toasts.len())).size(13),
                container(space()).width(Length::Fill),
                text("Snora Workbench").size(13),
            ]
            .align_y(Center)
            .spacing(12)
            .into(),
        );

        let mut layout = AppLayout::new(body)
            .header(header)
            .side_bar(sidebar)
            .footer(footer)
            .toasts(self.toasts.clone())
            .toast_position(self.toast_position)
            .direction(self.direction)
            .on_close_menus(Message::CloseMenus)
            .on_close_modals(Message::CloseModals);

        // File menu dropdown overlay (below modal dim — Law 2).
        if self.file_menu_open {
            layout = layout.header_menu(self.build_file_menu_overlay());
        }
        // Context menu overlay.
        if self.context_menu_open {
            layout = layout.context_menu(self.build_context_menu_overlay());
        }
        if self.show_dialog {
            layout = layout.dialog(Dialog::new(self.build_dialog()));
        }
        if self.show_sheet {
            layout = layout.sheet(
                Sheet::new(self.build_sheet())
                    .at(SheetEdge::End)   // logical End — mirrors under RTL (ABDD)
                    .with_size(SheetSize::OneThird),
            );
        }
        render(layout)
    }

    fn build_file_menu_overlay(&self) -> Element<'_, Message> {
        container(column![
            btn("New",        Message::CloseMenus),
            btn("Open",       Message::CloseMenus),
            btn("Close menu", Message::CloseMenus),
        ].spacing(4).padding(8))
        .style(menu_style)
        .into()
    }

    fn build_context_menu_overlay(&self) -> Element<'_, Message> {
        container(column![
            btn("Action A",   Message::CloseMenus),
            btn("Action B",   Message::CloseMenus),
            btn("Close menu", Message::CloseMenus),
        ].spacing(4).padding(8))
        .style(menu_style)
        .into()
    }

    fn build_dialog(&self) -> Element<'_, Message> {
        container(column![
            text("About Snora Workbench").size(18),
            text("Demonstrates the centered modal surface (layer 5)."),
            text("The dim layer (layer 4) blocks menu access while this is open."),
            row![space().width(Length::Fill), btn("Close", Message::CloseModals)],
        ].spacing(12).padding(24))
        .width(400)
        .into()
    }

    fn build_sheet(&self) -> Element<'_, Message> {
        column![
            text("Settings Sheet").size(16),
            text("Anchored to the logical End edge (layer 6)."),
            text("Under RTL the sheet appears on the left — try toggling direction."),
            space().height(Length::Fill),
            btn("Close sheet", Message::CloseModals),
        ]
        .spacing(12)
        .padding(16)
        .into()
    }

    // --- Tab bodies -------------------------------------------------------

    fn tab_overview(&self) -> Element<'_, Message> {
        column![
            text("Overview").size(16),
            text("All major Snora surfaces are wired in this example."),
            text("Use the header buttons to open overlays."),
            text("Toggle RTL in the header to verify all surfaces mirror."),
        ].spacing(8).into()
    }

    fn tab_overlay_lab(&self) -> Element<'_, Message> {
        column![
            text("Overlay Lab").size(16),
            text("Open overlays via the header buttons. Observe z-stack behavior:"),
            text("  • File menu: layer 2 (below modal dim)."),
            text("  • Dialog: layer 5. Sheet: layer 6. Toasts: layer 7."),
            text("  • Toast dismiss is reachable even while a modal is open."),
            row![
                btn("Open dialog", Message::OpenDialog),
                btn("Open sheet",  Message::OpenSheet),
            ].spacing(6),
        ].spacing(8).into()
    }

    fn tab_toast_lab(&self) -> Element<'_, Message> {
        let intent_btns = row![
            btn("Debug",   Message::AddToast(ToastIntent::Debug)),
            btn("Info",    Message::AddToast(ToastIntent::Info)),
            btn("Success", Message::AddToast(ToastIntent::Success)),
            btn("Warning", Message::AddToast(ToastIntent::Warning)),
            btn("Error",   Message::AddToast(ToastIntent::Error)),
        ].spacing(4);

        let pos_btns = row![
            btn("↖",  Message::SetToastPosition(ToastPosition::TopStart)),
            btn("↑",  Message::SetToastPosition(ToastPosition::TopCenter)),
            btn("↗",  Message::SetToastPosition(ToastPosition::TopEnd)),
            btn("↙",  Message::SetToastPosition(ToastPosition::BottomStart)),
            btn("↓",  Message::SetToastPosition(ToastPosition::BottomCenter)),
            btn("↘",  Message::SetToastPosition(ToastPosition::BottomEnd)),
        ].spacing(4);

        column![
            text("Toast Lab").size(16),
            text(format!("Position: {:?}", self.toast_position)),
            text("Add by intent:"),
            intent_btns,
            text("Change anchor:"),
            pos_btns,
            text("Newest toast is always closest to the anchor edge."),
        ].spacing(8).into()
    }

    fn tab_direction_lab(&self) -> Element<'_, Message> {
        let lbl = match self.direction {
            LayoutDirection::Ltr => "Currently LTR",
            LayoutDirection::Rtl => "Currently RTL",
        };
        column![
            text("Direction Lab").size(16),
            text(lbl),
            btn("Toggle LTR ↔ RTL", Message::ToggleDirection),
            text("After toggling, observe:"),
            text("  • Sidebar moves to the opposite edge."),
            text("  • Sheet End edge mirrors (right ↔ left)."),
            text("  • Toast Start/End anchor mirrors."),
            text("  • Header start/end controls swap."),
            text("ABDD — Accessible By Default and by Design."),
        ].spacing(8).into()
    }
}

// ---------------------------------------------------------------------------
// Style helper
// ---------------------------------------------------------------------------

fn menu_style(theme: &iced::Theme) -> iced::widget::container::Style {
    let p = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(p.background.base.color)),
        border: iced::Border { width: 1.0, color: p.background.weak.color, radius: 4.0.into() },
        ..Default::default()
    }
}

fn btn<'a>(label: &'static str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(13)).on_press(msg).padding([4, 10]).into()
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> iced::Result {
    iced::application(Workbench::default, Workbench::update, Workbench::view)
        .subscription(Workbench::subscription)
        .run()
}
