use iced::{
    Alignment::Center,
    Element, Length,
    widget::{button, column, container, row, scrollable, space, text},
};
use snora::{
    AppLayout, AppSideBar, AppSideBarItem, BottomSheet, Icon, Menu, MenuItem, PageLayout,
    components::app::{header::app_header, side_bar::app_side_bar},
    icons,
    layout::{app::render_app, page::render_page},
    style::container_box_style,
};

use super::{App, MenuItemId, ViewId, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // --- 1. Page Level の構築 ---

        let page_body = container(
            column![
                text(format!("Current View: {}", self.active_view_id)).size(32),
                text("This is the main page content area.").size(18),
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
        let app_header = app_header(
            "Snora App",
            vec![
                Menu {
                    label: "File".into(),
                    icon: Some(Icon::Lucide(icons::FileText)),
                    items: vec![MenuItem {
                        menu_id: MenuItemId::FileDummy,
                        label: "Dummy".into(),
                        icon: Some(Icon::Lucide(icons::FileText)),
                    }],
                },
                Menu {
                    label: "Settings".into(),
                    icon: Some(Icon::Lucide(icons::Settings)),
                    items: vec![MenuItem {
                        menu_id: MenuItemId::SettingsDummy,
                        label: "Settings".into(),
                        icon: Some(Icon::Lucide(icons::Settings)),
                    }],
                },
            ],
            &Message::MenuAction,
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
            active_view_id: self.active_view_id.clone(),
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
                button(if self.is_bottom_sheet_open {
                    "Hide Logs ▲"
                } else {
                    "Show Logs ▼"
                })
                .on_press(Message::ToggleLogSheet)
            ]
            .spacing(16)
            .align_y(Center),
        )
        .padding(16)
        .into();
        let footer = Some(footer_content);

        let bottom_sheet = if self.is_bottom_sheet_open {
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
                        button("Close").on_press(Message::ToggleLogSheet)
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
                on_close: Some(Message::ToggleLogSheet),
            })
        } else {
            None
        };

        // 最終的な AppLayout の組み立て
        let app_layout = AppLayout {
            body: page_node,
            header: Some(app_header),
            side_bar: Some(sidebar_node),
            footer,
            dialog: None,
            bottom_sheet,
            toasts: vec![],
            direction: self.direction,
        };

        render_app(app_layout)
    }
}
