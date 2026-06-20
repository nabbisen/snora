//! # Example: design-workbench
//!
//! Live visual-fit QA workbench for the Snora Design token system (RFC-030).
//!
//! **What to inspect:**
//! - Button variants: vertical centering, text clipping, disabled readability.
//! - Card variants: padding, border visibility, shadow on raised card.
//! - High-contrast mode: border clarity, text-on-background legibility.
//! - Focus: tab through buttons; iced 0.14 does not expose focus state to
//!   button styles so no custom ring is drawn — documented limitation
//!   (RFC-025, RFC-027).
//! - Typography: size scale, line-height fit, no clipping.
//! - Palette swatches: swatch colors at each preset.
//!
//! **Controls:**
//! - Header preset buttons: Light / Dark / HC Light / HC Dark.
//! - All four semantic button variants shown enabled and disabled.
//!
//! Run with:
//! ```text
//! cargo run -p snora-example-design-workbench
//! ```

use iced::{
    Alignment, Element, Length, Task,
    widget::{column, container, row, scrollable, text},
};
use snora::{AppLayout, LayoutDirection, MenuAction, render, widget::app_header};
use snora::design::{Color, Tone, Tokens, button, card, chip, notice::Notice, progress, style};

// ---------------------------------------------------------------------------
// Preset selector
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preset { Light, Dark, HcLight, HcDark }

impl Preset {
    fn label(self) -> &'static str {
        match self {
            Self::Light   => "Light",
            Self::Dark    => "Dark",
            Self::HcLight => "HC Light",
            Self::HcDark  => "HC Dark",
        }
    }

    fn tokens(self) -> Tokens {
        match self {
            Self::Light   => Tokens::light(),
            Self::Dark    => Tokens::dark(),
            Self::HcLight => Tokens::high_contrast_light(),
            Self::HcDark  => Tokens::high_contrast_dark(),
        }
    }
}

// ---------------------------------------------------------------------------
// State / Messages / Update
// ---------------------------------------------------------------------------

// Tokens are stored in state so that view() borrows them via &self,
// avoiding the lifetime issue of a locally-constructed Tokens value.
struct App {
    preset: Preset,
    tokens: Tokens,
    notice_dismissed: bool,
}

impl Default for App {
    fn default() -> Self {
        Self { preset: Preset::Light, tokens: Preset::Light.tokens(), notice_dismissed: false }
    }
}

#[derive(Debug, Clone)]
enum Msg {
    SetPreset(Preset),
    Noop,
    DismissNotice,
}

impl App {
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::SetPreset(p) => {
                self.preset = p;
                self.tokens = p.tokens();
                self.notice_dismissed = false;
            }
            Msg::DismissNotice => self.notice_dismissed = true,
            Msg::Noop => {}
        }
        Task::none()
    }
}

// ---------------------------------------------------------------------------
// View helpers
// ---------------------------------------------------------------------------

fn heading<'a>(t: &'a Tokens, s: &'a str) -> Element<'a, Msg> {
    text(s)
        .size(style::text::title_size(t))
        .color(style::color::to_iced_color(t.palette.text_primary))
        .into()
}

fn note<'a>(t: &'a Tokens, s: &'a str) -> Element<'a, Msg> {
    text(s)
        .size(style::text::body_small_size(t))
        .color(style::color::to_iced_color(t.palette.text_secondary))
        .into()
}

fn swatch_text_color(c: Color) -> iced::Color {
    let b = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
    if b < 0.5 { iced::Color::WHITE } else { iced::Color::BLACK }
}

// ---------------------------------------------------------------------------
// Preset toolbar
// ---------------------------------------------------------------------------

fn preset_toolbar<'a>(t: &'a Tokens, active: Preset) -> Element<'a, Msg> {
    let presets = [Preset::Light, Preset::Dark, Preset::HcLight, Preset::HcDark];
    let btns: Vec<Element<'a, Msg>> = presets.iter().map(|&p| {
        if p == active {
            button::primary(t, p.label(), Msg::SetPreset(p))
        } else {
            button::secondary(t, p.label(), Msg::SetPreset(p))
        }
    }).collect();
    row(btns).spacing(t.spacing.sm).into()
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

fn buttons_section<'a>(t: &'a Tokens) -> Element<'a, Msg> {
    let g = t.spacing.sm;
    card::surface(t, column![
        heading(t, "Buttons"),
        note(t, "Inspect: vertical centering, text clipping, disabled alpha."),
        note(t, "Focus: tab through — iced 0.14 has no focused Status, no custom ring drawn (RFC-025)."),
        column![
            note(t, "Enabled"),
            row![
                button::primary(t, "Primary", Msg::Noop),
                button::secondary(t, "Secondary", Msg::Noop),
                button::ghost(t, "Ghost", Msg::Noop),
                button::danger(t, "Danger", Msg::Noop),
            ].spacing(g),
        ].spacing(g),
        column![
            note(t, "Disabled (on_press = None)"),
            row![
                button::primary_maybe(t, "Primary", None::<Msg>),
                button::secondary_maybe(t, "Secondary", None::<Msg>),
                button::ghost_maybe(t, "Ghost", None::<Msg>),
                button::danger_maybe(t, "Danger", None::<Msg>),
            ].spacing(g),
        ].spacing(g),
    ].spacing(t.spacing.md))
}

fn cards_section<'a>(t: &'a Tokens) -> Element<'a, Msg> {
    let mk = |label: &'a str, desc: &'a str| -> Element<'a, Msg> {
        column![
            text(label)
                .size(style::text::label_size(t))
                .color(style::color::to_iced_color(t.palette.text_primary)),
            text(desc)
                .size(style::text::body_small_size(t))
                .color(style::color::to_iced_color(t.palette.text_secondary)),
        ].spacing(t.spacing.xs).into()
    };

    card::surface(t, column![
        heading(t, "Cards"),
        note(t, "Inspect: padding, border clarity at high contrast, shadow on Raised."),
        row![
            card::surface(t, mk("Surface", "Default card")),
            card::raised(t, mk("Raised", "Elevated + shadow")),
            card::selected(t, mk("Selected", "Accent border")),
        ].spacing(t.spacing.md),
    ].spacing(t.spacing.md))
}

fn typography_section<'a>(t: &'a Tokens) -> Element<'a, Msg> {
    let pairs: &[(&str, iced::Pixels)] = &[
        ("Display",    style::text::display_size(t)),
        ("Heading",    style::text::heading_size(t)),
        ("Title",      style::text::title_size(t)),
        ("Body",       style::text::body_size(t)),
        ("Label",      style::text::label_size(t)),
        ("Body small", style::text::body_small_size(t)),
    ];

    let rows: Vec<Element<'a, Msg>> = pairs.iter().map(|(name, size)| {
        row![
            text(format!("{name} — The quick brown fox"))
                .size(*size)
                .color(style::color::to_iced_color(t.palette.text_primary))
                .width(Length::Fill),
            text(format!("{:.0}px", size.0))
                .size(style::text::body_small_size(t))
                .color(style::color::to_iced_color(t.palette.text_secondary)),
        ].align_y(Alignment::Center).into()
    }).collect();

    card::surface(t, column(
        std::iter::once(heading(t, "Typography"))
            .chain(std::iter::once(note(t, "Inspect: no clipping, readable scale.")))
            .chain(rows)
            .collect::<Vec<_>>(),
    ).spacing(t.spacing.sm))
}

fn notices_section<'a>(t: &'a Tokens, dismissed: bool) -> Element<'a, Msg> {
    let mut notices: Vec<Element<'a, Msg>> = Vec::new();
    for tone in [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger] {
        let label = match tone {
            Tone::Info    => "Info notice — background sync running.",
            Tone::Success => "Success notice — export complete.",
            Tone::Warning => "Warning notice — disk space low.",
            Tone::Danger  => "Danger notice — action cannot be undone.",
            _             => "",
        };
        notices.push(
            Notice::new(t, tone, label)
                .action("View", Msg::Noop)
                .render(),
        );
    }
    // One dismissible notice
    if !dismissed {
        notices.push(
            Notice::new(t, Tone::Accent, "This notice can be dismissed.")
                .title("Dismissible")
                .action("Learn more", Msg::Noop)
                .dismiss(Msg::DismissNotice)
                .render(),
        );
    }
    card::surface(t, iced::widget::column![
        heading(t, "Notices"),
        note(t, "Inspect: tone colors, border clarity at HC, action/dismiss button focus."),
        iced::widget::column(notices).spacing(t.spacing.sm),
    ].spacing(t.spacing.md))
}

fn chips_section<'a>(t: &'a Tokens) -> Element<'a, Msg> {
    card::surface(t, iced::widget::column![
        heading(t, "Chips"),
        note(t, "Inspect: selected vs unselected tint at HC, tap target size."),
        iced::widget::row![
            chip::filter(t, "All",     true,  Msg::Noop),
            chip::filter(t, "Draft",   false, Msg::Noop),
            chip::filter(t, "Active",  false, Msg::Noop),
            chip::filter(t, "Done",    false, Msg::Noop),
        ].spacing(t.spacing.sm),
        iced::widget::row![
            chip::removable(t, "Rust",   true,  Msg::Noop, Msg::Noop),
            chip::removable(t, "Design", false, Msg::Noop, Msg::Noop),
        ].spacing(t.spacing.sm),
    ].spacing(t.spacing.md))
}

fn progress_section<'a>(t: &'a Tokens) -> Element<'a, Msg> {
    card::surface(t, iced::widget::column![
        heading(t, "Progress"),
        note(t, "Inspect: bar fill at 0%/60%/100%, indeterminate (…), tone colors."),
        progress::row(t, "Indexing files",    Some(0.6),  Tone::Accent),
        progress::row(t, "Upload complete",   Some(1.0),  Tone::Success),
        progress::row(t, "Sync in progress",  None,       Tone::Info),
        progress::row(t, "Low disk — backup", Some(0.85), Tone::Warning),
    ].spacing(t.spacing.md))
}

fn palette_section<'a>(t: &'a Tokens) -> Element<'a, Msg> {
    let p = &t.palette;
    let pairs: &[(Color, &str)] = &[
        (p.background,     "background"),
        (p.surface,        "surface"),
        (p.surface_raised, "surface_raised"),
        (p.accent,         "accent"),
        (p.success,        "success"),
        (p.warning,        "warning"),
        (p.danger,         "danger"),
        (p.info,           "info"),
        (p.focus,          "focus"),
    ];

    let swatches: Vec<Element<'a, Msg>> = pairs.iter().map(|(color, name)| {
        let bg       = style::color::to_iced_color(*color);
        let text_col = swatch_text_color(*color);
        let r        = t.radius.sm;
        let pad      = t.spacing.sm;
        container(
            text(*name).size(style::text::body_small_size(t)).color(text_col),
        )
        .padding(pad)
        .width(100.0)
        .height(56.0)
        .style(move |_| iced::widget::container::Style {
            background: Some(bg.into()),
            border: iced::Border::default().rounded(r),
            ..Default::default()
        })
        .into()
    }).collect();

    card::surface(t, column![
        heading(t, "Palette"),
        note(t, "Inspect at HC presets: all swatches must be visually distinct."),
        row(swatches).spacing(t.spacing.xs),
    ].spacing(t.spacing.md))
}

// ---------------------------------------------------------------------------
// Main view
// ---------------------------------------------------------------------------

impl App {
    fn view(&self) -> Element<'_, Msg> {
        let t = &self.tokens;
        let bg = style::color::to_iced_color(t.palette.background);

        let toolbar = preset_toolbar(t, self.preset);
        let header  = app_header(
            "Snora Design Workbench",
            vec![],
            &|_: MenuAction<(), ()>| Msg::Noop,
            None::<&()>,
            Some(toolbar),
            LayoutDirection::Ltr,
        );

        let body: Element<'_, Msg> = scrollable(
            container(
                column![
                    text("Visual-fit QA for Snora Design. \
                          Use the header buttons to switch presets. \
                          Tab through controls to test native focus handling.")
                        .size(style::text::body_size(t))
                        .color(style::color::to_iced_color(t.palette.text_primary)),
                    buttons_section(t),
                    cards_section(t),
                    notices_section(t, self.notice_dismissed),
                    chips_section(t),
                    progress_section(t),
                    typography_section(t),
                    palette_section(t),
                ]
                .spacing(t.spacing.lg)
                .padding(t.spacing.lg),
            )
            .width(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(bg.into()),
                ..Default::default()
            }),
        ).into();

        render(AppLayout::new(body).header(header))
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("Snora Design Workbench"))
        .run()
}
