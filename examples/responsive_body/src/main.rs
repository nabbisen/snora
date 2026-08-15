//! # Example: responsive_body
//!
//! Demonstrates `snora::responsive_render` (RFC-046) varying **`body`'s
//! own composition** by width, rather than a slot `AppLayout` names.
//! This is the pattern snora's two known consumers actually use:
//! application-owned chrome — here, a tab bar — composed *into* `body`,
//! not built from `snora::widget::*` and not placed in `side_bar`.
//!
//! Complements `examples/responsive` (slot-based: `AppLayout::side_bar`
//! collapsing below a width) rather than replacing it — see
//! `docs/src/guides/responsive.md` for which reader each is for.
//!
//! **Engine-only, compiler-enforced**: this crate's `Cargo.toml` pins
//! `snora` with `default-features = false`. There is no
//! `snora::widget::*`, no `side_bar`, and no `footer` anywhere in this
//! file — the header and tab bar below are built from plain `iced`
//! widgets, the same building blocks an application with its own design
//! system would use.
//!
//! This example picks its own threshold — **TAB_BAR_STACK_WIDTH**, in
//! logical pixels — and its own response to crossing it: the tab bar
//! switches from a horizontal row to a vertical stack. Both the number
//! and what happens at it are this example's decision, not snora's; a
//! different application would reasonably pick a different threshold, or
//! change something else at it entirely.
//!
//! Also correct under both `LayoutDirection` values: press "Flip LTR ↔
//! RTL" and the header and the *horizontal* tab bar mirror.
//! `snora::direction::row_dir` — the helper that would normally do this
//! swap — lives in `snora-widgets` and is unavailable in an engine-only
//! build, so this file reorders by hand: the same one-line `match`
//! every direction-aware row needs, without the helper.
//!
//! **RTL mirrors the horizontal axis only.** When the tab bar is
//! stacked vertically (below `TAB_BAR_STACK_WIDTH`), its order does
//! *not* reverse under `Rtl` — a vertical list reads top-to-bottom in
//! Arabic and Hebrew exactly as it does in English. `AppLayout` itself
//! follows the same rule: header stays above footer under both
//! directions, and only a horizontal row (sidebar beside body) ever
//! swaps. A reader hand-rolling direction support should reverse a
//! *row*'s element order, never a *column*'s.
//!
//! **Try it:** run this example and resize the window narrower than
//! ~640px — the tab bar stacks vertically. Press "Flip LTR ↔ RTL" at any
//! width: the header mirrors at both widths, and so does the tab bar
//! while it is horizontal — but once stacked, its order stays
//! top-to-bottom.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-responsive-body
//! ```

use iced::widget::{button, column, container, row, text};
use iced::{Alignment::Center, Element, Length};
use snora::{AppLayout, LayoutDirection, responsive_render};

/// Below this width, the tab bar switches from a horizontal row to a
/// vertical stack. This example's own choice — snora exposes the
/// available width and prescribes nothing about what "narrow" means or
/// what should happen at it. A different application would reasonably
/// pick a different number, or a different response entirely.
const TAB_BAR_STACK_WIDTH: f32 = 640.0;

const TABS: [&str; 3] = ["Overview", "Settings", "About"];

/// The tab bar's element order. **Reverses under `Rtl` only when `row`
/// is `true`** (horizontal) — a vertical stack's reading order does not
/// depend on `LayoutDirection`, only a horizontal row's physical
/// left/right does. This is the one property that made this example
/// wrong once (a prior version reversed the vertical stack too); see
/// the `#[cfg(test)]` module below for the check that would have caught
/// it, and that a change to this function should keep passing.
fn tab_order(direction: LayoutDirection, row: bool) -> Vec<usize> {
    let indices: Vec<usize> = (0..TABS.len()).collect();
    if row && direction == LayoutDirection::Rtl {
        indices.into_iter().rev().collect()
    } else {
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All four combinations this example claims to handle correctly
    /// (RFC-051 AC-4): both widths (`row` = horizontal / not-`row` =
    /// stacked), both `LayoutDirection` values. Run with
    /// `cargo test -p snora-example-responsive-body`.
    #[test]
    fn row_order_reverses_under_rtl_only() {
        assert_eq!(tab_order(LayoutDirection::Ltr, true), vec![0, 1, 2]);
        assert_eq!(tab_order(LayoutDirection::Rtl, true), vec![2, 1, 0]);
    }

    #[test]
    fn column_order_is_unaffected_by_direction() {
        assert_eq!(tab_order(LayoutDirection::Ltr, false), vec![0, 1, 2]);
        assert_eq!(
            tab_order(LayoutDirection::Rtl, false),
            vec![0, 1, 2],
            "a vertical stack reads top-to-bottom under Rtl exactly as under Ltr — \
             mirroring it was this example's original bug"
        );
    }
}

#[derive(Debug, Clone)]
enum Message {
    Flip,
    SelectTab(usize),
}

struct App {
    direction: LayoutDirection,
    active_tab: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            direction: LayoutDirection::Ltr,
            active_tab: 0,
        }
    }
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Flip => self.direction = self.direction.flipped(),
            Message::SelectTab(i) => self.active_tab = i,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let direction = self.direction;
        let active_tab = self.active_tab;

        responsive_render(move |width| {
            // The header: title at the logical start, "Flip" button at
            // the logical end. Under `Rtl` the physical order reverses —
            // `row_dir` would do this in one call; here it's a `match`.
            let title = text("snora — responsive body").size(18);
            let flip_button = button(text("Flip LTR ↔ RTL").size(12)).on_press(Message::Flip);
            let header_row: Element<'_, Message> = match direction {
                LayoutDirection::Ltr => row![title, flip_button],
                LayoutDirection::Rtl => row![flip_button, title],
            }
            .align_y(Center)
            .spacing(12)
            .into();
            let header: Element<'_, Message> =
                container(header_row).padding(12).width(Length::Fill).into();

            // The tab bar — application-owned chrome composed *into*
            // `body`, not a slot `AppLayout` names. Below
            // `TAB_BAR_STACK_WIDTH` it stacks vertically instead of
            // sitting in a horizontal row. `tab_order` (above) is the
            // single place that decides element order for both cases —
            // see its doc and the `#[cfg(test)]` module below for why
            // only the horizontal row reverses under `Rtl`.
            let is_narrow = width < TAB_BAR_STACK_WIDTH;
            let tab_button = |i: usize| -> Element<'_, Message> {
                let style = if i == active_tab {
                    button::primary
                } else {
                    button::secondary
                };
                button(text(TABS[i]).size(14))
                    .on_press(Message::SelectTab(i))
                    .style(style)
                    .into()
            };
            let tab_bar: Element<'_, Message> = if is_narrow {
                column(tab_order(direction, false).into_iter().map(tab_button))
                    .spacing(4)
                    .into()
            } else {
                row(tab_order(direction, true).into_iter().map(tab_button))
                    .spacing(4)
                    .into()
            };

            let panel = container(
                text(format!(
                    "{} — available width: {width:.0}px ({} tab bar, {:?})",
                    TABS[active_tab],
                    if is_narrow { "stacked" } else { "horizontal" },
                    direction,
                ))
                .size(14),
            )
            .padding(16);

            let body: Element<'_, Message> = container(column![tab_bar, panel].spacing(16))
                .padding(32)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();

            AppLayout::new(body).header(header).direction(direction)
        })
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — responsive body"))
        .run()
}
