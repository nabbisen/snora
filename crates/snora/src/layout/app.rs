use iced::widget::{center, column, container, mouse_area, opaque, row, space, stack, text};
use iced::{Background, Color, Element, Length};
use snora_core::contract::{app::AppLayout, rtl::LayoutDirection};

pub fn render_app<'a, Message>(
    layout: AppLayout<Element<'a, Message>, Message>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    // ==========================================
    // 1. Base Layer (Header -> Body/Sidebar -> Footer)
    // ==========================================
    let mut main_col = column![];

    if let Some(header) = layout.header {
        main_col = main_col.push(header);
    }

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

    let base_layer = container(main_col).width(Length::Fill).height(Length::Fill);

    // ==========================================
    // 2. Stack Layer (Base -> BottomSheet -> Dialog -> Toasts)
    // ==========================================
    let mut ui_stack = stack![base_layer];

    // -- Dialog オーバーレイ --
    if let Some(dialog) = layout.dialog {
        let backdrop = container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..Default::default()
            });

        let backdrop_interactive = if let Some(on_close) = dialog.on_outside_click {
            mouse_area(opaque(backdrop)).on_press(on_close)
        } else {
            mouse_area(opaque(backdrop))
        };

        let dialog_content = center(dialog.content);
        ui_stack = ui_stack.push(stack![backdrop_interactive, dialog_content]);
    }

    // -- BottomSheet オーバーレイ --
    if let Some(sheet) = layout.bottom_sheet {
        // 1. 背景の視覚効果（暗転）のみ。ここではクリックを拾わない
        let backdrop_visual = container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.3))),
                ..Default::default()
            });

        // 2. インタラクティブ・レイヤー
        // column 自体は画面全体を覆うが、上部の space 部分でクリックを拾うようにする
        let sheet_layer = column![
            // 上部 2/3：ここをクリックしたら閉じる
            if let Some(msg) = sheet.on_close {
                mouse_area(space().width(Length::Fill).height(Length::FillPortion(2))).on_press(msg)
            } else {
                mouse_area(space().height(Length::FillPortion(2)))
            },
            // 下部 1/3：シート本体（ここはクリックを透過させない）
            container(sheet.content)
                .width(Length::Fill)
                .height(Length::FillPortion(1)) // シート自体のクリックが背面に抜けないよう opaque に包む
                                                //（中身が opaque なら不要ですが、念のため）
        ];

        // 視覚効果の上に操作レイヤーを重ねる
        ui_stack = ui_stack.push(stack![backdrop_visual, sheet_layer]);
    }

    // -- Toast オーバーレイ --
    if !layout.toasts.is_empty() {
        let mut toasts_col = column![].spacing(8);

        for toast in layout.toasts {
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
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom);

        ui_stack = ui_stack.push(toasts_container);
    }

    ui_stack.width(Length::Fill).height(Length::Fill).into()
}
