use iced::widget::{center, column, container, mouse_area, opaque, row, space, stack, text};
use iced::{Background, Color, Element, Length};
use snora_core::contract::page::PageLayout;
use snora_core::contract::rtl::LayoutDirection;

/// PageLayout の規約を iced の Element に変換する
pub fn render_page<'a, Message>(
    layout: PageLayout<Element<'a, Message>, Message>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    // ==========================================
    // 1. Base Layer (Header -> [Aside + Body] -> Footer)
    // ==========================================
    let mut page_col = column![];

    if let Some(header) = layout.header {
        page_col = page_col.push(header);
    }

    let body_area = match layout.aside {
        Some(aside) => {
            let mut content_row = row![];
            let aside = container(aside).height(Length::Fill);
            let content = container(layout.body)
                .width(Length::Fill)
                .height(Length::Fill);

            content_row = match layout.direction {
                LayoutDirection::Ltr => {
                    content_row = content_row.push(aside);
                    content_row = content_row.push(content);
                    content_row
                }
                LayoutDirection::Rtl => {
                    content_row = content_row.push(content);
                    content_row = content_row.push(aside);
                    content_row
                }
            };

            container(content_row)
        }
        None => container(layout.body)
            .width(Length::Fill)
            .height(Length::Fill),
    };

    page_col = page_col.push(body_area);

    if let Some(footer) = layout.footer {
        page_col = page_col.push(footer);
    }

    let base_layer = container(page_col).width(Length::Fill).height(Length::Fill);

    // ==========================================
    // 2. Stack Layer (Base -> Dialog -> Toasts)
    // ==========================================
    let mut ui_stack = stack![base_layer];

    // Dialog オーバーレイ
    if let Some(dialog) = layout.dialog {
        // 半透明の黒背景（クリックキャッチ用）
        let backdrop = container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..Default::default()
            });

        // 領域外クリックで閉じるアクションの設定
        let backdrop_interactive = if let Some(on_close) = dialog.on_outside_click {
            mouse_area(opaque(backdrop)).on_press(on_close)
        } else {
            mouse_area(opaque(backdrop))
        };

        let dialog_content = center(dialog.content);
        ui_stack = ui_stack.push(stack![backdrop_interactive, dialog_content]);
    }

    // Toast オーバーレイ（右下配置の想定）
    if !layout.toasts.is_empty() {
        let mut toasts_col = column![].spacing(8);

        for toast in layout.toasts {
            // ※ここでは簡易的な描画。実際は専用の render_toast 関数などを呼ぶ
            let toast_ui = container(column![
                text(toast.title).size(16),
                text(toast.message).size(14)
            ])
            .padding(12)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            toasts_col = toasts_col.push(toast_ui);
        }

        let toasts_container = container(toasts_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .align_x(iced::alignment::Horizontal::Right) // 右寄せ
            .align_y(iced::alignment::Vertical::Bottom); // 下寄せ

        ui_stack = ui_stack.push(toasts_container);
    }

    ui_stack.width(Length::Fill).height(Length::Fill).into()
}
