use iced::widget::{column, container, row};
use iced::{Element, Length};
use snora_core::contract::{app::AppLayout, rtl::LayoutDirection};

/// AppLayout の規約を iced の Element に変換する
pub fn render_app<'a, Message>(layout: AppLayout<Element<'a, Message>>) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    // メインコンテンツエリア（Header -> Body -> Footer の縦積み）
    let mut main_col = column![];

    if let Some(header) = layout.header {
        main_col = main_col.push(header);
    }

    // サイドバー + Body は残りの高さをすべて埋める
    // サイドバーとメインエリアの横並び（Ltr / Rtl 対応）
    let mut body_row = row![].width(Length::Fill).height(Length::Fill);

    let body = container(layout.body)
        .width(Length::Fill)
        .height(Length::Fill);

    match layout.direction {
        LayoutDirection::Ltr => {
            if let Some(sidebar) = layout.side_bar {
                body_row = body_row.push(sidebar);
            }
            body_row = body_row.push(body);
        }
        LayoutDirection::Rtl => {
            body_row = body_row.push(body);
            if let Some(sidebar) = layout.side_bar {
                body_row = body_row.push(sidebar);
            }
        }
    }

    main_col = main_col.push(body_row);

    if let Some(footer) = layout.footer {
        main_col = main_col.push(footer);
    }

    let main_container = container(main_col).width(Length::Fill).height(Length::Fill);

    main_container.into()
}
