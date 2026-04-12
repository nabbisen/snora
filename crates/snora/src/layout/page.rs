use iced::widget::{column, container, row};
use iced::{Element, Length};
use snora_core::contract::page::PageLayout;
use snora_core::contract::rtl::LayoutDirection;

pub fn render_page<'a, Message>(layout: PageLayout<Element<'a, Message>>) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
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
                LayoutDirection::Ltr => content_row.push(aside).push(content),
                LayoutDirection::Rtl => content_row.push(content).push(aside),
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

    container(page_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
