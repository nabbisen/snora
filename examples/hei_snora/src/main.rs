use iced::{
    Element, Length, Task,
    widget::{button, column, container, text},
};
use snora::ToastIntent;
use snora::{
    AppLayout, AppSideBar, AppSideBarItem, Icon, LayoutDirection, MenuItem, PageLayout,
    components::app::{
        footer::{LogEntry, app_footer},
        header::app_header,
        side_bar::app_side_bar,
    },
    icons,
    layout::{app::render_app, page::render_page},
    style::container_box_style,
};

pub fn main() -> iced::Result {
    iced::application(HeiSnora::new, HeiSnora::update, HeiSnora::view)
        .title(HeiSnora::title)
        .run()
}

struct HeiSnora {
    direction: LayoutDirection,
    active_view: String, // 現在選択されているAppレベルのビューID
    is_log_expanded: bool,
    logs: Vec<LogEntry>,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleDirection,
    ToggleLog,
    SelectView(String),
    MenuAction(&'static str),
}

impl HeiSnora {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                direction: LayoutDirection::Ltr,
                active_view: "home".into(),
                is_log_expanded: false,
                logs: vec![],
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        "Snora Framework - Modern Layout".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleDirection => {
                self.direction = match self.direction {
                    LayoutDirection::Ltr => LayoutDirection::Rtl,
                    LayoutDirection::Rtl => LayoutDirection::Ltr,
                };
            }
            Message::ToggleLog => {
                self.is_log_expanded = !self.is_log_expanded;
            }
            Message::SelectView(id) => {
                self.active_view = id.clone();
                self.logs.push(LogEntry {
                    intent: ToastIntent::Info,
                    timestamp: "Now".into(),
                    message: format!("Switched to view: {}", id),
                });
            }
            Message::MenuAction(name) => {
                self.logs.push(LogEntry {
                    intent: ToastIntent::Info,
                    timestamp: "Just now".into(),
                    message: format!("Clicked: {}", name),
                });
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // --- 1. Page Level の構築 ---

        let page_body = container(
            column![
                text(format!("Current View: {}", self.active_view)).size(32),
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
            dialog: None,
            toasts: vec![],
            direction: self.direction,
        };

        // PageLayout を Node (Element) に変換
        let page_node = render_page(page_layout);

        // --- 2. App Level の構築 ---

        // App Header
        let app_header = app_header(
            "Snora App",
            vec![
                MenuItem {
                    label: "File".into(),
                    icon: Some(Icon::Lucide(icons::FileText)),
                    action: Some(Message::MenuAction("File")),
                },
                MenuItem {
                    label: "Settings".into(),
                    icon: Some(Icon::Lucide(icons::Settings)),
                    action: Some(Message::MenuAction("Settings")),
                },
            ],
            None,
        );

        // App Side Bar (VS Codeのアクティビティバーに相当)
        let sidebar_data = AppSideBar {
            items: vec![
                AppSideBarItem {
                    id: "home".into(),
                    icon: Icon::Lucide(icons::Home),
                    tooltip: "Home".into(),
                    action: Message::SelectView("home".into()),
                },
                AppSideBarItem {
                    id: "search".into(),
                    icon: Icon::Lucide(icons::Search),
                    tooltip: "Search".into(),
                    action: Message::SelectView("search".into()),
                },
            ],
            active_id: self.active_view.clone(),
        };
        let sidebar_node = app_side_bar(sidebar_data);

        // App Footer
        let app_footer = app_footer(
            "Ready",
            &self.logs,
            self.is_log_expanded,
            None,
            Message::ToggleLog,
        );

        // 最終的な AppLayout の組み立て
        let app_layout = AppLayout {
            body: page_node,
            header: Some(app_header),
            side_bar: Some(sidebar_node),
            footer: Some(app_footer),
            direction: self.direction,
        };

        render_app(app_layout)
    }
}
