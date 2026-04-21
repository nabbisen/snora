use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill},
    Padding,
    widget::{button, column, container, row, scrollable, space, text},
};
use snora::{
    AppLayout, AppSideBar, AppSideBarItem, BottomSheet, Dialog, Icon, Menu, MenuItem, PageLayout,
    components::app::{header::app_header, side_bar::app_side_bar},
    icons,
    layout::{app::render_app, page::render_page},
    style::container_box_style,
};

use super::{
    App, FileMenuItemId, MenuId, MenuItemId, SettingsMenuItemId, ViewId, message::Message,
};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // --- 1. Page Level の構築 ---

        let page_body = container(
            column![
                text(format!("Current View: {}", self.view_id)).size(32),
                text("This is the main page content area.").size(18),
                button("Open context").on_press(Message::OpenContext(
                    self.context_menu_pos.unwrap_or_default()
                )),
                button("Toggle Dialog").on_press(Message::ToggleDialog),
                button("Add toast").on_press(Message::AddDummyToast),
                button("Toggle LTR / RTL").on_press(Message::ToggleDirection)
            ]
            .spacing(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        // Page Aside (VS Codeのエクスプローラー領域に相当)
        let page_aside = container(
            column![
                text("Page Aside").size(20),
                text("Contextual info for the current view.")
            ]
            .spacing(10),
        )
        .width(200)
        .height(Length::Fill)
        .padding(10)
        .style(container_box_style)
        .into();

        let page_layout = PageLayout {
            body: page_body,
            header: None, // 今回はAppレベルにHeaderを持つためNone
            aside: Some(page_aside),
            footer: None, // 今回はAppレベルにFooterを持つためNone
            direction: self.direction,
        };

        // PageLayout を Node (Element) に変換
        let page_node = render_page(page_layout);

        // --- 2. App Level の構築 ---

        // App Header
        let menus = vec![
            Menu {
                id: MenuId::File,
                label: "File".into(),
                icon: Some(Icon::Lucide(icons::FileText)),
                items: vec![MenuItem {
                    menu_id: MenuId::File,
                    id: MenuItemId::File(FileMenuItemId::New),
                    label: "New".into(),
                    icon: None,
                }],
            },
            Menu {
                id: MenuId::Settings,
                label: "Settings".into(),
                icon: Some(Icon::Lucide(icons::Settings)),
                items: vec![MenuItem {
                    menu_id: MenuId::Settings,
                    id: MenuItemId::Settings(SettingsMenuItemId::About),
                    label: "About".into(),
                    icon: None,
                }],
            },
        ];

        let app_header = app_header(
            "Snora App",
            menus,
            &|action| Message::HeaderAction(action),
            self.menu_id.as_ref(),
            None,
        );

        // App Side Bar (VS Codeのアクティビティバーに相当)
        let sidebar_data = AppSideBar {
            items: vec![
                AppSideBarItem {
                    view_id: ViewId::Home,
                    icon: Icon::Lucide(icons::Home),
                    tooltip: "Home".into(),
                    action: Message::SelectView(ViewId::Home),
                },
                AppSideBarItem {
                    view_id: ViewId::Search,
                    icon: Icon::Lucide(icons::Search),
                    tooltip: "Search".into(),
                    action: Message::SelectView(ViewId::Search),
                },
                AppSideBarItem {
                    view_id: ViewId::Settings,
                    icon: Icon::Lucide(icons::Settings),
                    tooltip: "Settings".into(),
                    action: Message::SelectView(ViewId::Settings),
                },
            ],
            view_id: self.view_id.clone(),
        };
        let sidebar_node = app_side_bar(sidebar_data);

        // App Footer
        // Footer にシートを開閉するトグルボタンを配置
        let footer_content = container(
            row![
                text(format!("Total Logs: {}", self.logs.len())).size(14),
                container(space()).width(Length::Fill),
                button("Add Log").on_press(Message::AddDummyLog),
                // 状態に応じてボタンのラベルを変更
                button(if self.show_bottom_sheet {
                    "Hide Logs ▲"
                } else {
                    "Show Logs ▼"
                })
                .on_press(Message::ToggleSheet)
            ]
            .spacing(16)
            .align_y(Center),
        )
        .padding(16)
        .into();
        let footer = Some(footer_content);

        // 最終的な AppLayout の組み立て
        let app_layout = AppLayout {
            body: page_node,
            header: Some(app_header),
            header_menu: None.into(),
            menu_id: self.menu_id.clone(),
            side_bar: Some(sidebar_node),
            footer,
            context_menu: self.render_context_menu().into(),
            dialog: self.render_dialog().into(),
            bottom_sheet: self.render_bottom_sheet().into(),
            toasts: self.toasts,
            direction: self.direction,
        };

        render_app(
            app_layout,
            self.menu_id.as_ref().map(|_| Message::CloseMenus), // 同期
            Some(Message::CloseModals),                         // None か Some(Message)
        )
    }

    fn render_context_menu(&self) -> Option<Element<'_, Message>> {
        self.context_menu_pos.map(|pos| {
            let menu_content = container(
                column![
                    button("Copy").on_press(Message::MenuPressed(MenuId::File)),
                    button("Paste").on_press(Message::MenuPressed(MenuId::Settings)),
                ]
                .spacing(5),
            )
            .padding(8);

            container(menu_content)
                // Padding 修正: 構造体で直接指定
                .padding(Padding {
                    top: pos.y,
                    left: pos.x,
                    ..Padding::ZERO
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        })
    }

    fn render_dialog(&self) -> Option<Dialog<Element<'_, Message>, Message>> {
        if !self.show_dialog {
            return None;
        }

        let dialog_card = container(
            column![
                text("Important Action Required").size(24),
                button("Close").on_press(Message::CloseModals),
            ]
            .spacing(20)
            .align_x(Center),
        )
        .padding(40);

        Some(Dialog {
            content: container(dialog_card).center_x(Fill).center_y(Fill).into(),
            on_outside_click: Some(Message::CloseModals),
        })
    }

    fn render_bottom_sheet(&self) -> Option<BottomSheet<Element<'_, Message>, Message>> {
        if !self.show_bottom_sheet {
            return None;
        }

        // 💡 所有権を奪わず、参照から UI を組み立てる
        let log_list = self.logs.iter().fold(column![].spacing(8), |col, log| {
            col.push(
                row![
                    text(&log.timestamp).size(12), // 参照を渡す
                    text(log.intent.to_string()).size(12),
                    text(&log.message).size(13),
                ]
                .spacing(8),
            )
        });

        let sheet_content = container(
            column![
                row![
                    text("System Logs").size(18),
                    container(space()).width(Length::Fill),
                    button("Close").on_press(Message::ToggleSheet)
                ]
                .align_y(Center),
                scrollable(log_list)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .spacing(16),
        )
        .padding(24)
        .style(container_box_style);

        Some(BottomSheet {
            content: sheet_content.into(), // ここで Element<'a> に変換
            on_close: Some(Message::ToggleSheet),
        })
    }
}
