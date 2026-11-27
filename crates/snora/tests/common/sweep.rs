//! The drawn-contrast sweep's machinery (RFC-131, and RFC-135 after it): which scenes snora draws,
//! what each is expected to contain, and how a colour is read from the pixels.
//!
//! # The inventory is made by drawing
//!
//! A source search for colours is what missed the inactive tab (RFC-128 R-2 searched for non-text
//! colours, and a mix of two text colours passes such a search). So nothing here lists a colour. A
//! scene is **drawn**, and every text run it holds is enumerated from the widget tree with
//! `Operation::text`: the place `iced_selector` uses for `Candidate::Text`, and the one snora's own
//! `Glyph` and `Underlined` announce their text to. The run's colour is then **read from the frame**.
//! (The renderer's own text list is private to iced; the tree is the place that needs no public API.)
//!
//! # Words and glyphs (RFC-131 M-1)
//!
//! WCAG's own split: a **word** (a label, a title, a crumb, a number) is text, held to its palette's
//! level (1.4.3, 1.4.6: 4.5, or 7). A **glyph that is not a word** (an icon, the `×`, the breadcrumb's
//! `›`/`‹`, a tab's `★`) is a graphical object, held to 3:1 (1.4.11, held by `nontext_sweep.rs`, RFC-135). **Each scene lists its
//! runs and says which is which, here, in writing; the classification is never inferred from a
//! string**, so a label cannot be waved through as a glyph, and a run the table does not name fails
//! (tripwire 1 in `text_sweep.rs`).
//!
//! # Reading a colour from pixels
//!
//! A thin stroke is anti-aliased, so at the scale it is drawn at its pixels are blends that read low
//! against any background: the `×` on a toast reads 5.89:1 at 1x against a declared 6.54, and a `›` at
//! 10 px declared 4.51 reads 2.79. **Drawn at twice the pixel density a 1 px stroke is two solid pixels
//! wide**, and the pixel furthest in contrast from the background beside it is the glyph's colour
//! (RFC-131 M-2). Measured on glyphs of known colour (2.62 to 7.03:1, five shapes, 10 to 20 px) it
//! reads the declared ratio to two decimals, and on all 3,210 cells of Unit 0's sweep 2x and 4x agree
//! to 0.00. **Its limit:** a stroke thinner than half a logical pixel would still not reach its colour
//! at 2x. None of snora's glyphs is.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). Unit 0 compared both on 4,860 cells:
//! no verdict changed.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use super::removal::Rig;
use iced::widget::{column, container, text};
use iced::{Element, Length, Rectangle, Theme};
use iced_runtime::core::widget::{Id, Operation};
use snora::design::{Color as DColor, Tokens, Tone, button, chip, notice::Notice, widget as dw};
use snora::{
    AppLayout, BreadcrumbAction, Crumb, Icon, LayoutDirection, Menu, MenuAction, MenuItem, SideBar,
    SideBarItem, Tab, TabAction, TabBar, Toast, ToastIntent,
};

/// The pixel density the sweep reads at. See the module documentation.
pub const SCALE: usize = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Go,
    Menu(MenuAction<u8, u8>),
    Tab(TabAction<u8>),
    Crumb(BreadcrumbAction<u8>),
}
pub static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;
pub static ON_TAB: fn(TabAction<u8>) -> Msg = Msg::Tab;
pub static ON_CRUMB: fn(BreadcrumbAction<u8>) -> Msg = Msg::Crumb;

// ---- the levels -------------------------------------------------------------------------------------------------------------------

/// The book's text level per preset (`aaa-presets.md`), written here and **never read from a
/// palette's token**: a guard does not read what it guards (RFC-129 Q-4). A preset that is not
/// here panics, naming it.
pub const LEVELS: [(&str, f32); 6] = [
    ("light", 4.5),
    ("dark", 4.5),
    ("light_aaa", 7.0),
    ("dark_aaa", 7.0),
    ("high_contrast_light", 7.0),
    ("high_contrast_dark", 7.0),
];

pub fn level(preset: &str) -> f32 {
    LEVELS
        .iter()
        .find(|(name, _)| *name == preset)
        .unwrap_or_else(|| panic!("`{preset}` is a new preset: say its text level in LEVELS"))
        .1
}

/// WCAG 1.4.11's floor for a graphical object.
pub const NON_TEXT: f32 = 3.0;

/// 8-bit rounding of the ink and the background moves a ratio by about 0.02.
pub const TOLERANCE: f32 = 0.05;

// ---- classification ---------------------------------------------------------------------------------------------------------------

/// What a run is. **Written in the scene's table, never inferred.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// A word: text, held to the palette's level.
    Word,
    /// A glyph that is not a word, with what it is: held to 3:1 by `nontext_sweep.rs`.
    Glyph(&'static str),
    /// The application's own content in the scene (a snora widget does not draw it): not measured.
    App,
}

/// `Send + Sync`: a sweep's presets are drawn on threads of their own (`par.rs`), each reading the same scenes.
pub type Make =
    Box<dyn Fn(&'static Tokens, LayoutDirection) -> Element<'static, Msg> + Send + Sync>;

/// One scene snora draws, and every run it is expected to contain.
pub struct Scene {
    pub name: &'static str,
    /// The `snora::design` builders this scene draws (tripwire 3 checks every export is in some scene).
    pub covers: &'static [&'static str],
    /// Drawn by `design::render`, which paints its own page: the neutral surface does not apply.
    pub own_page: bool,
    /// The widget takes a direction, so it is drawn in both.
    pub directional: bool,
    /// Hovered and pressed apply to its controls.
    pub states: bool,
    /// Every run the scene holds, with its kind. A run in the frame that is not here fails.
    pub runs: &'static [(&'static str, Kind)],
    pub make: Make,
}

/// The close mark: lucide's `X` under the `lucide-icons` feature, a plain `×` without it.
pub const CLOSE: [(&str, Kind); 2] = [
    ("\u{e1b2}", Kind::Glyph("close mark")),
    ("\u{d7}", Kind::Glyph("close mark")),
];

pub fn menu(id: u8, label: &str, items: u8) -> Menu<u8, u8> {
    Menu {
        id,
        label: label.into(),
        icon: Some(Icon::Text("#".into())),
        items: (0..items)
            .map(|i| MenuItem {
                menu_id: id,
                id: i,
                label: format!("{label} item {i}"),
                icon: (i == 0).then(|| Icon::Text("@".into())),
            })
            .collect(),
    }
}

pub fn rail(n: u8) -> SideBar<Msg, u8> {
    SideBar {
        items: (0..n)
            .map(|i| SideBarItem {
                view_id: i,
                icon: Icon::Text(format!("{}", i + 1)),
                tooltip: format!("Item {i}"),
                on_press: Msg::Go,
            })
            .collect(),
        active: 1,
    }
}

pub fn page() -> Element<'static, Msg> {
    column![text("Page text")].into()
}

pub const TONES: [(&str, Tone); 6] = [
    ("info", Tone::Info),
    ("success", Tone::Success),
    ("warning", Tone::Warning),
    ("danger", Tone::Danger),
    ("accent", Tone::Accent),
    ("neutral", Tone::Neutral),
];

use Kind::{App, Glyph, Word};

/// The tick: lucide's `Check` under the `lucide-icons` feature, a text glyph without (RFC-100's rule for a mark).
pub const TICK: [(&str, Kind); 2] = [("\u{e06c}", Glyph("tick")), ("\u{2713}", Glyph("tick"))];
/// The select's chevron: lucide's `ChevronDown`, or a text glyph.
pub const CHEVRON: [(&str, Kind); 2] = [
    ("\u{e06d}", Glyph("chevron")),
    ("\u{25be}", Glyph("chevron")),
];

/// Every scene the sweep draws. **A new design widget adds a scene here** (tripwire 3 fails until it
/// does), and a new text run in a scene adds a row to its `runs` (tripwire 1).
pub fn scenes() -> Vec<Scene> {
    let mut v: Vec<Scene> = Vec::new();
    v.push(Scene {
        name: "header",
        covers: &["app_header"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[
            ("App title", Word),
            ("File", Word),
            ("Flip", Word),
            ("#", Glyph("icon")),
        ],
        make: Box::new(|t, dir| {
            dw::app_header(
                t,
                "App title",
                vec![menu(0, "File", 2)],
                &ON_MENU,
                None,
                Some(button::ghost(t, "Flip", Msg::Go)),
                dir,
            )
        }),
    });
    v.push(Scene {
        name: "header with an open menu",
        covers: &[],
        own_page: true,
        directional: true,
        states: true,
        runs: &[
            ("App title", Word),
            ("File", Word),
            ("File item 0", Word),
            ("File item 1", Word),
            ("#", Glyph("icon")),
            ("@", Glyph("icon")),
            ("Page text", App),
        ],
        make: Box::new(|t, dir| {
            let h = dw::app_header(
                t,
                "App title",
                vec![menu(0, "File", 2)],
                &ON_MENU,
                Some(&0u8),
                None,
                dir,
            );
            snora::design::render(AppLayout::new(page()).header(h).direction(dir), t)
        }),
    });
    v.push(Scene {
        name: "footer",
        covers: &["app_footer"],
        own_page: false,
        directional: true,
        states: false,
        runs: &[("Ready", Word)],
        make: Box::new(|t, dir| dw::app_footer(t, text("Ready").into(), dir)),
    });
    v.push(Scene {
        name: "tab bar",
        covers: &["app_tab_bar"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[
            ("Tab0", Word),
            ("Tab1", Word),
            ("Tab2", Word),
            ("Tab3", Word),
            ("\u{2605}", Glyph("icon")),
        ],
        make: Box::new(|t, dir| {
            let bar = TabBar {
                tabs: (0..4u8)
                    .map(|i| Tab {
                        id: i,
                        label: format!("Tab{i}"),
                        icon: (i == 1 || i == 2).then(|| Icon::Text("\u{2605}".into())),
                    })
                    .collect(),
                active: 1,
            };
            dw::app_tab_bar(t, bar, &ON_TAB, dir)
        }),
    });
    v.push(Scene {
        name: "breadcrumb",
        covers: &["app_breadcrumb"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[
            ("Root", Word),
            ("Mid", Word),
            ("Here", Word),
            ("\u{203a}", Glyph("separator")),
            ("\u{2039}", Glyph("separator")),
        ],
        make: Box::new(|t, dir| {
            dw::app_breadcrumb(
                t,
                vec![
                    Crumb::ancestor(0u8, "Root"),
                    Crumb::ancestor(1, "Mid"),
                    Crumb::leaf(2, "Here"),
                ],
                &ON_CRUMB,
                dir,
            )
        }),
    });
    for (name, labels, runs) in [
        (
            "sidebar, names beneath",
            dw::SideBarLabels::Beneath,
            &[
                ("Item 0", Word),
                ("Item 1", Word),
                ("Item 2", Word),
                ("1", Glyph("icon")),
                ("2", Glyph("icon")),
                ("3", Glyph("icon")),
            ][..],
        ),
        (
            "sidebar, names beside",
            dw::SideBarLabels::Beside,
            &[
                ("Item 0", Word),
                ("Item 1", Word),
                ("Item 2", Word),
                ("1", Glyph("icon")),
                ("2", Glyph("icon")),
                ("3", Glyph("icon")),
            ][..],
        ),
        (
            "sidebar, names in tooltips",
            dw::SideBarLabels::Tooltip,
            &[
                ("1", Glyph("icon")),
                ("2", Glyph("icon")),
                ("3", Glyph("icon")),
                ("Page text", App),
            ][..],
        ),
    ] {
        v.push(Scene {
            name,
            covers: if name.ends_with("beneath") {
                &["app_side_bar", "app_side_bar_with"]
            } else {
                &[]
            },
            // The rail whose names are in tooltips is drawn in a window: its tooltip opens on the end
            // side and iced snaps it into the window, so on a rail with no room beside it (alone, at
            // the left under RTL) the tooltip would land over the item and take the pointer.
            own_page: labels == dw::SideBarLabels::Tooltip,
            directional: true,
            states: true,
            runs,
            make: Box::new(move |t, dir| {
                let rail = dw::app_side_bar_with(t, rail(3), dir, labels);
                if labels == dw::SideBarLabels::Tooltip {
                    snora::design::render(AppLayout::new(page()).side_bar(rail).direction(dir), t)
                } else {
                    rail
                }
            }),
        });
    }
    for (tn, tone) in TONES {
        let name: &'static str = Box::leak(format!("notice, {tn}").into_boxed_str());
        v.push(Scene {
            name,
            covers: if tn == "info" { &["Notice"] } else { &[] },
            own_page: false,
            directional: true,
            states: true,
            runs: &[
                ("Notice title", Word),
                ("Body text of the notice.", Word),
                ("Act", Word),
                CLOSE[0],
                CLOSE[1],
            ],
            make: Box::new(move |t, dir| {
                Notice::new(t, tone, "Body text of the notice.", dir)
                    .title("Notice title")
                    .action("Act", Msg::Go)
                    .dismiss(Msg::Go)
                    .render()
            }),
        });
    }
    v.push(Scene {
        name: "filter chip, selected",
        covers: &["filter"],
        own_page: false,
        directional: false,
        states: true,
        runs: &[("Filter", Word)],
        make: Box::new(|t, _| chip::filter(t, "Filter", true, Msg::Go)),
    });
    v.push(Scene {
        name: "filter chip, unselected",
        covers: &[],
        own_page: false,
        directional: false,
        states: true,
        runs: &[("Filter", Word)],
        make: Box::new(|t, _| chip::filter(t, "Filter", false, Msg::Go)),
    });
    for (name, selected) in [
        ("removable chip, selected", true),
        ("removable chip, unselected", false),
    ] {
        v.push(Scene {
            name,
            covers: if selected {
                &["removable", "removable_with_tooltip"]
            } else {
                &[]
            },
            own_page: false,
            directional: true,
            states: true,
            runs: &[("Tag", Word), CLOSE[0], CLOSE[1]],
            make: Box::new(move |t, dir| {
                chip::removable(t, "Tag", selected, Msg::Go, Msg::Go, dir)
            }),
        });
    }
    type Btn = fn(&Tokens, &str, Msg) -> Element<'static, Msg>;
    let buttons: [(&str, &[&str], Btn); 5] = [
        (
            "button, primary",
            &["primary", "primary_maybe"],
            |t, l, m| button::primary(t, l, m),
        ),
        (
            "button, secondary",
            &["secondary", "secondary_maybe"],
            |t, l, m| button::secondary(t, l, m),
        ),
        ("button, ghost", &["ghost", "ghost_maybe"], |t, l, m| {
            button::ghost(t, l, m)
        }),
        ("button, danger", &["danger", "danger_maybe"], |t, l, m| {
            button::danger(t, l, m)
        }),
        ("button, link", &["link"], |t, l, m| button::link(t, l, m)),
    ];
    for (name, covers, make) in buttons {
        v.push(Scene {
            name,
            covers,
            own_page: false,
            directional: false,
            states: true,
            runs: &[("Save", Word)],
            make: Box::new(move |t, _| make(t, "Save", Msg::Go)),
        });
    }
    v.push(Scene {
        name: "progress row",
        covers: &["row"],
        own_page: false,
        directional: true,
        states: false,
        runs: &[("Progress  50%", Word)],
        make: Box::new(|t, dir| {
            snora::design::progress::row(t, "Progress", Some(0.5), Tone::Accent, dir)
        }),
    });
    v.push(Scene {
        name: "progress card",
        covers: &["card"],
        own_page: false,
        directional: true,
        states: false,
        runs: &[("Progress  50%", Word)],
        make: Box::new(|t, dir| {
            snora::design::progress::card(t, "Progress", Some(0.5), Tone::Accent, dir)
        }),
    });
    for (name, covers) in [
        ("card, surface", &["surface"][..]),
        ("card, raised", &["raised"][..]),
        ("card, selected", &["selected"][..]),
    ] {
        v.push(Scene {
            name,
            covers,
            own_page: false,
            directional: false,
            states: false,
            runs: &[("Card", App)],
            make: Box::new(move |t, _| {
                let card = match name {
                    "card, surface" => snora::design::card::surface(t, text("Card")),
                    "card, raised" => snora::design::card::raised(t, text("Card")),
                    _ => snora::design::card::selected(t, text("Card")),
                };
                container(card).width(Length::Fixed(300.0)).into()
            }),
        });
    }
    for (iname, intent) in [
        ("debug", ToastIntent::Debug),
        ("info", ToastIntent::Info),
        ("success", ToastIntent::Success),
        ("warning", ToastIntent::Warning),
        ("error", ToastIntent::Error),
    ] {
        let name: &'static str = Box::leak(format!("toast, {iname}").into_boxed_str());
        v.push(Scene {
            name,
            covers: &[],
            own_page: true,
            directional: true,
            states: false,
            runs: &[
                ("Toast title", Word),
                ("Toast message", Word),
                ("Page text", App),
                CLOSE[0],
                CLOSE[1],
            ],
            make: Box::new(move |t, dir| {
                let l = AppLayout::new(page())
                    .toasts(vec![Toast::new(
                        7,
                        intent,
                        "Toast title",
                        "Toast message",
                        Msg::Go,
                    )])
                    .direction(dir);
                snora::design::render(l, t)
            }),
        });
    }
    // RFC-134: the five form controls. Hovered and pressed apply to each (a tint of the label's colour
    // behind it), and both directions are drawn.
    v.push(Scene {
        name: "checkbox, checked",
        covers: &["checkbox"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[("Remember me", Word), TICK[0], TICK[1]],
        make: Box::new(|t, dir| {
            snora::design::checkbox(t, true, dir)
                .label("Remember me")
                .on_toggle(|_| Msg::Go)
                .into()
        }),
    });
    v.push(Scene {
        name: "checkbox, unchecked",
        covers: &[],
        own_page: false,
        directional: true,
        states: true,
        runs: &[("Subscribe", Word)],
        make: Box::new(|t, dir| {
            snora::design::checkbox(t, false, dir)
                .label("Subscribe")
                .on_toggle(|_| Msg::Go)
                .into()
        }),
    });
    for (name, on) in [("toggler, on", true), ("toggler, off", false)] {
        v.push(Scene {
            name,
            covers: if on { &["toggler"] } else { &[] },
            own_page: false,
            directional: true,
            states: true,
            runs: &[("Dark mode", Word)],
            make: Box::new(move |t, dir| {
                snora::design::toggler(t, on, dir)
                    .label("Dark mode")
                    .on_toggle(|_| Msg::Go)
                    .into()
            }),
        });
    }
    v.push(Scene {
        name: "radio group",
        covers: &["radio_group"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[("Small", Word), ("Medium", Word), ("Large", Word)],
        make: Box::new(|t, dir| {
            snora::design::radio_group(
                t,
                ["Small", "Medium", "Large"],
                Some(&"Medium"),
                |_| Msg::Go,
                dir,
            )
            .into()
        }),
    });
    // RFC-138: the same group laid out in a row, the three options on one line with the middle one selected (so that a
    // finder can name it by position whichever way the line is mirrored). Its marks and labels are the column's.
    v.push(Scene {
        name: "radio group, horizontal",
        covers: &[],
        own_page: false,
        directional: true,
        states: true,
        runs: &[("Small", Word), ("Medium", Word), ("Large", Word)],
        make: Box::new(|t, dir| {
            container(
                snora::design::radio_group(
                    t,
                    ["Small", "Medium", "Large"],
                    Some(&"Medium"),
                    |_| Msg::Go,
                    dir,
                )
                .horizontal(true),
            )
            .width(Length::Fixed(320.0))
            .into()
        }),
    });
    v.push(Scene {
        name: "slider",
        covers: &["slider"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[],
        make: Box::new(|t, dir| {
            snora::design::slider(t, 0.0..=100.0, 35.0, |_| Msg::Go, dir)
                .width(Length::Fixed(300.0))
                .into()
        }),
    });
    v.push(Scene {
        name: "select, closed",
        covers: &["select"],
        own_page: false,
        directional: true,
        states: true,
        runs: &[("Banana", Word), CHEVRON[0], CHEVRON[1]],
        make: Box::new(|t, dir| {
            snora::design::select(
                t,
                ["Apple", "Banana", "Cherry"],
                Some(&"Banana"),
                |_| Msg::Go,
                dir,
            )
            .on_open_change(|_| Msg::Go)
            .into()
        }),
    });
    v.push(Scene {
        name: "select, open",
        covers: &[],
        own_page: true,
        directional: true,
        states: true,
        runs: &[
            ("Apple", Word),
            ("Banana", Word),
            ("Cherry", Word),
            ("Page text", App),
            TICK[0],
            TICK[1],
            CHEVRON[0],
            CHEVRON[1],
        ],
        make: Box::new(|t, dir| {
            let select = snora::design::select(
                t,
                ["Apple", "Banana", "Cherry"],
                Some(&"Banana"),
                |_| Msg::Go,
                dir,
            )
            .opened(true)
            .on_open_change(|_| Msg::Go);
            let body: Element<'static, Msg> = column![select, page()].into();
            snora::design::render(AppLayout::new(body).direction(dir), t)
        }),
    });
    v
}

// ---- drawing a scene -------------------------------------------------------------------------------------------------------------

pub fn leak(t: Tokens) -> &'static Tokens {
    Box::leak(Box::new(t))
}

pub fn design_theme(t: &'static Tokens) -> Theme {
    snora::design::theme(t)
}

fn iced_colour(c: DColor) -> iced::Color {
    iced::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

pub const SURFACES: [&str; 3] = ["background", "surface", "surface_raised"];

pub fn surface_colour(t: &Tokens, name: &str) -> DColor {
    match name {
        "background" => t.palette.background,
        "surface" => t.palette.surface,
        _ => t.palette.surface_raised,
    }
}

/// The scene's element, on the neutral surface `surface` unless it paints its own page.
pub fn build(
    sc: &Scene,
    t: &'static Tokens,
    surface: &str,
    dir: LayoutDirection,
) -> Element<'static, Msg> {
    let el = (sc.make)(t, dir);
    if sc.own_page {
        return el;
    }
    let fill = iced_colour(surface_colour(t, surface));
    container(el)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(fill.into()),
            ..Default::default()
        })
        .into()
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Rest,
    Hovered,
    Pressed,
}

pub fn apply_state(rig: &mut Rig<Msg>, target: Rectangle, state: State) {
    match state {
        State::Rest => {}
        State::Hovered => {
            rig.point_at(Some(target.center()));
            rig.redraw();
        }
        State::Pressed => {
            rig.point_at(Some(target.center()));
            rig.left_button(true);
            rig.redraw();
        }
    }
}

pub fn inside(b: Rectangle, outer: Rectangle) -> bool {
    let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    cx >= outer.x && cx <= outer.x + outer.width && cy >= outer.y && cy <= outer.y + outer.height
}

/// Every text run in the tree, with its bounds: `Operation::text`.
pub fn runs(rig: &mut Rig<Msg>) -> Vec<(String, Rectangle)> {
    struct Collect(Arc<Mutex<Vec<(String, Rectangle)>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
            self.0.lock().unwrap().push((text.to_owned(), bounds));
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    out.lock().unwrap().clone()
}

// ---- reading a colour -------------------------------------------------------------------------------------------------------------

fn lum(p: [u8; 3]) -> f32 {
    let f = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(p[0]) + 0.7152 * f(p[1]) + 0.0722 * f(p[2])
}

/// WCAG's contrast ratio, written out here (the sweep does not use the crate's).
pub fn ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
    let (la, lb) = (lum(a), lum(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn hex(p: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", p[0], p[1], p[2])
}

/// What a run is drawn in, and on: the **background** is the modal colour inside the run's box, the
/// **ink** the pixel in the box furthest in contrast from it (RFC-131 M-2). `None` if the box holds
/// no ink (an empty or blank run).
pub fn read_run(frame: &[u8], b: Rectangle) -> Option<([u8; 3], [u8; 3], f32)> {
    let s = SCALE as f32;
    let get = |x: usize, y: usize| {
        let i = (y * 800 * SCALE + x) * 4;
        [frame[i], frame[i + 1], frame[i + 2]]
    };
    let (x0, y0) = ((b.x.max(0.0) * s) as usize, (b.y.max(0.0) * s) as usize);
    let (x1, y1) = (
        (((b.x + b.width) * s).ceil() as usize).min(800 * SCALE - 1),
        (((b.y + b.height) * s).ceil() as usize).min(600 * SCALE - 1),
    );
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let mut counts: std::collections::HashMap<[u8; 3], usize> = std::collections::HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            *counts.entry(get(x, y)).or_default() += 1;
        }
    }
    let bg = counts
        .iter()
        .max_by_key(|(c, n)| (**n, **c))
        .map(|(c, _)| *c)?;
    let (mut ink, mut best) = (bg, 1.0f32);
    for y in y0..y1 {
        for x in x0..x1 {
            let r = ratio(get(x, y), bg);
            if r > best {
                best = r;
                ink = get(x, y);
            }
        }
    }
    (best > 1.05).then_some((ink, bg, best))
}

// ---- the sweep ---------------------------------------------------------------------------------------------------------------------

/// One measured cell.
#[derive(Clone, Debug)]
pub struct Cell {
    pub preset: &'static str,
    pub scene: &'static str,
    pub surface: &'static str,
    pub direction: LayoutDirection,
    pub state: &'static str,
    /// The control the pointer is on (hovered, pressed), by its place among the scene's focus targets.
    pub target: Option<usize>,
    pub text: String,
    pub kind: Option<Kind>,
    pub ink: [u8; 3],
    pub bg: [u8; 3],
    pub ratio: f32,
}

/// Every run of every scene, in every preset, on every neutral surface, in each state and each
/// direction the widget has. **A run that the scene's table does not name is returned in `unlisted`**
/// (tripwire 1), not skipped.
pub struct Sweep {
    pub cells: Vec<Cell>,
    pub unlisted: Vec<String>,
}

pub fn kind_of(sc: &Scene, text: &str) -> Option<Kind> {
    sc.runs.iter().find(|(s, _)| *s == text).map(|x| x.1)
}

pub fn sweep() -> Sweep {
    sweep_of(scenes(), None)
}

/// The same for the given scenes, and for one preset if `only` names it (a plant needs one, not six).
pub fn sweep_of(scenes: Vec<Scene>, only: Option<&str>) -> Sweep {
    // The presets are drawn on threads of their own, and put back in preset order (`par.rs`): the cells and the
    // `unlisted` runs come out as they did when one thread drew them all (a message names its preset, so none repeats).
    let parts = super::par::by_preset(|preset, tokens| {
        let mut cells = Vec::new();
        let mut unlisted = Vec::new();
        if only.is_some_and(|o| o != preset) {
            return (cells, unlisted);
        }
        let t = leak(tokens);
        let theme = design_theme(t);
        for sc in &scenes {
            let dirs: &[LayoutDirection] = if sc.directional {
                &[LayoutDirection::Ltr, LayoutDirection::Rtl]
            } else {
                &[LayoutDirection::Ltr]
            };
            let surfaces: &[&'static str] = if sc.own_page {
                &["(own page)"]
            } else {
                &SURFACES
            };
            for &dir in dirs {
                for &surface in surfaces {
                    let scene_name: &'static str = sc.name;
                    // Rest: every run in the scene.
                    let mut rig = Rig::new(build(sc, t, surface, dir));
                    let n_targets = rig.targets().len();
                    let found = runs(&mut rig);
                    let frame = rig.frame_scaled(&theme, SCALE as f32);
                    let mut record =
                        |state: &'static str,
                         target: Option<usize>,
                         found: &[(String, Rectangle)],
                         frame: &[u8],
                         only: Option<Rectangle>| {
                            for (s, b) in found {
                                if only.is_some_and(|tb| !inside(*b, tb)) {
                                    continue;
                                }
                                let kind = kind_of(sc, s);
                                if kind.is_none() {
                                    let msg = format!("{preset} / {scene_name} / {dir:?}: {s:?}");
                                    if !unlisted.contains(&msg) {
                                        unlisted.push(msg);
                                    }
                                }
                                if let Some((ink, bg, r)) = read_run(frame, *b) {
                                    cells.push(Cell {
                                        preset,
                                        scene: scene_name,
                                        surface,
                                        direction: dir,
                                        state,
                                        target,
                                        text: s.clone(),
                                        kind,
                                        ink,
                                        bg,
                                        ratio: r,
                                    });
                                }
                            }
                        };
                    record("rest", None, &found, &frame, None);
                    if !sc.states {
                        continue;
                    }
                    // Hovered and pressed, control by control: the runs inside that control.
                    for i in 0..n_targets {
                        for (label, state) in
                            [("hovered", State::Hovered), ("pressed", State::Pressed)]
                        {
                            let mut rig = Rig::new(build(sc, t, surface, dir));
                            let Some(tb) = rig.targets().get(i).map(|x| x.bounds) else {
                                continue;
                            };
                            apply_state(&mut rig, tb, state);
                            let found = runs(&mut rig);
                            let frame = rig.frame_scaled(&theme, SCALE as f32);
                            record(label, Some(i), &found, &frame, Some(tb));
                        }
                    }
                }
            }
        }
        (cells, unlisted)
    });
    let mut cells = Vec::new();
    let mut unlisted = Vec::new();
    for (c, u) in parts {
        cells.extend(c);
        unlisted.extend(u);
    }
    Sweep { cells, unlisted }
}
