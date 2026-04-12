use std::fmt::Debug;

use iced::widget::{column, container, row, stack};
use iced::{Element, Length};
use snora_core::contract::{app::AppLayout, rtl::LayoutDirection};

use crate::stack::{bottom_sheet::render_bottom_sheet, dialog::render_dialog, toast::render_toast};

pub fn render_app<'a, Message, MenuId>(
    layout: AppLayout<Element<'a, Message>, Message, MenuId>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
    MenuId: Clone + Debug + PartialEq,
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
    if let Some(dialog_stack) = render_dialog(layout.dialog) {
        ui_stack = ui_stack.push(dialog_stack);
    }

    // -- BottomSheet オーバーレイ --
    if let Some(bottom_sheet_stack) = render_bottom_sheet(layout.bottom_sheet) {
        ui_stack = ui_stack.push(bottom_sheet_stack);
    }

    // -- Toast オーバーレイ --
    if let Some(toast_stack) = render_toast(layout.toasts) {
        ui_stack = ui_stack.push(toast_stack);
    }

    ui_stack.width(Length::Fill).height(Length::Fill).into()
}
