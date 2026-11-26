//! Scenes for the disclosure tests (RFC-137): a control that stays where it is when pressed and
//! shows or hides other controls after it, built on `snora::control` (engine path) and
//! `snora::design::control` (design path), inside the engine's page.
//!
//! A scene is a function of the items present (`&[u8]`), so a test names the view before a press and
//! the view after it:
//!
//! - `0..=9` are options after the header (the disclosure's content);
//! - [`ABOVE`] is a control inserted **before** the header;
//! - [`GONE`] says the header itself is not there (a misdeclared control that did go away);
//! - [`TWIN`] is a second control with the header's id (a doubled id);
//! - [`INPUT`] is a text input after everything else.
//!
//! Every control publishes its own name, so "what has focus" reads as a name three ways (the tree's
//! flags, a ring read from pixels, and what one more Enter publishes), as RFC-120's tests do.

#![allow(dead_code)]

use iced::widget::{column, container, row, space, text};
use iced::{Element, Length};

use snora::LayoutDirection;

use super::removal::Rig;
use super::removal_scenes::{Landing, Msg, msg, names, page, read, tokens};

pub const ABOVE: u8 = 100;
pub const GONE: u8 = 200;
pub const TWIN: u8 = 150;
/// A text input after everything else (the application's own focus can come from the pointer).
pub const INPUT: u8 = 120;

#[derive(Clone, Copy, Debug)]
pub enum Path {
    Engine,
    Design,
}

#[derive(Clone, Copy, Debug)]
pub enum Decl {
    /// `keeps_focus(id)`.
    Keeps,
    /// The same id and no declaration: what an application did before 0.66, and what
    /// "undeclared" means now.
    Undeclared,
}

pub const PATHS: [Path; 2] = [Path::Engine, Path::Design];
pub const DIRECTIONS: [LayoutDirection; 2] = [LayoutDirection::Ltr, LayoutDirection::Rtl];

/// A control around `content` on `path`, publishing `name`.
fn control(path: Path, name: &str, content: Element<'static, Msg>) -> snora::Control<'static, Msg> {
    match path {
        Path::Engine => snora::control(content).on_press(msg(name)),
        Path::Design => snora::design::control(tokens(), content).on_press(msg(name)),
    }
}

/// An ordinary control, which declares nothing.
fn plain(path: Path, name: &str) -> Element<'static, Msg> {
    control(path, name, text(name.to_owned()).into()).into()
}

fn declared(
    path: Path,
    decl: Decl,
    name: &str,
    id: &'static str,
    content: Element<'static, Msg>,
) -> Element<'static, Msg> {
    let c = control(path, name, content);
    match decl {
        Decl::Keeps => c.keeps_focus(id),
        Decl::Undeclared => c.id(id),
    }
    .into()
}

/// "Advanced settings": a header, the options after it, a control after those.
pub fn disclosure(
    path: Path,
    decl: Decl,
    direction: LayoutDirection,
) -> impl Fn(&[u8]) -> Element<'static, Msg> {
    move |items: &[u8]| {
        let mut col = column![].spacing(8);
        if items.contains(&ABOVE) {
            col = col.push(plain(path, "above"));
        }
        if !items.contains(&GONE) {
            col = col.push(declared(
                path,
                decl,
                "header",
                "advanced-settings",
                text("Advanced settings").into(),
            ));
        }
        for i in items.iter().filter(|i| **i < 10) {
            col = col.push(plain(path, &format!("option {i}")));
        }
        if items.contains(&TWIN) {
            let twin: Element<'static, Msg> = control(path, "twin", text("Twin").into())
                .id("advanced-settings")
                .into();
            col = col.push(twin);
        }
        col = col.push(plain(path, "after"));
        if items.contains(&INPUT) {
            col = col.push(iced::widget::text_input("search", "").on_input(|_| msg("typed")));
        }
        page(container(col).padding(24).into(), direction)
    }
}

/// aaai's file tree: rows of `width(Fill)`, an indent, an icon and a name. Rows `1` and `2` are the
/// files inside `src`; they are drawn only while `src` is open.
pub fn file_tree(
    path: Path,
    decl: Decl,
    direction: LayoutDirection,
) -> impl Fn(&[u8]) -> Element<'static, Msg> {
    move |items: &[u8]| {
        let node = |name: &str,
                    id: &'static str,
                    depth: u16,
                    folder: bool,
                    kept: bool|
         -> Element<'static, Msg> {
            let content: Element<'static, Msg> = row![
                space().width(Length::Fixed(f32::from(depth) * 24.0)),
                text(if folder { "▸" } else { "·" }),
                text(name.to_owned()),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into();
            let c = control(path, name, content)
                .width(Length::Fill)
                .padding([6, 12]);
            match (kept, decl) {
                (true, Decl::Keeps) => c.keeps_focus(id),
                _ => c.id(id),
            }
            .into()
        };
        let mut col = column![].spacing(2);
        col = col.push(node("src", "node:src", 0, true, true));
        for i in items.iter().filter(|i| **i < 10) {
            let name = ["main.rs", "lib.rs"][usize::from(*i - 1) % 2];
            let id = ["node:src/main.rs", "node:src/lib.rs"][usize::from(*i - 1) % 2];
            col = col.push(node(name, id, 1, false, false));
        }
        col = col.push(node("tests", "node:tests", 0, true, true));
        page(
            container(col)
                .padding(24)
                .width(Length::Fixed(360.0))
                .into(),
            direction,
        )
    }
}

/// Presses `target` **from the keyboard** in `view(before)` (Tab to it, Enter), rebuilds as `view(after)`
/// does, and reads where focus is, three ways.
pub fn press(
    view: &impl Fn(&[u8]) -> Element<'static, Msg>,
    before: &[u8],
    target: &str,
    after: &[u8],
) -> Landing {
    let mut rig = pressed(view, before, target);
    let mut rig = {
        rig.messages.clear();
        rig.rebuilt(view(after))
    };
    read(&mut rig, after, view)
}

/// `view(before)` with `target` focused by Tab and pressed by Enter, **not yet rebuilt**.
pub fn pressed(
    view: &impl Fn(&[u8]) -> Element<'static, Msg>,
    before: &[u8],
    target: &str,
) -> Rig<Msg> {
    let order = names(view, before);
    let index = order
        .iter()
        .position(|n| n == target)
        .unwrap_or_else(|| panic!("no control publishes {target:?} in {order:?}"));
    let mut rig = Rig::new(view(before));
    for _ in 0..=index {
        rig.tab(false);
    }
    assert_eq!(
        rig.focused(),
        vec![index],
        "the premise: {target} has focus"
    );
    rig.enter();
    assert_eq!(
        rig.messages,
        vec![msg(target)],
        "the premise: Enter pressed it"
    );
    rig
}

/// A **click** on `target` in `view(before)`, then the rebuild as `view(after)`. Returns the rig, so the
/// test can read where focus is and press Tab.
pub fn clicked(
    view: &impl Fn(&[u8]) -> Element<'static, Msg>,
    before: &[u8],
    target: &str,
    after: &[u8],
) -> Rig<Msg> {
    let order = names(view, before);
    let index = order
        .iter()
        .position(|n| n == target)
        .unwrap_or_else(|| panic!("no control publishes {target:?} in {order:?}"));
    let mut rig = Rig::new(view(before));
    let at = rig.targets()[index].bounds.center();
    rig.click_at(at);
    assert_eq!(
        rig.messages,
        vec![msg(target)],
        "the premise: the click pressed it"
    );
    rig.messages.clear();
    let mut rig = rig.rebuilt(view(after));
    // The pointer leaves, as it does when the user moves on: a hover tint under it would be read
    // from the pixels as a ring (the frame is compared with one never focused).
    rig.point_at(None);
    rig.redraw();
    rig
}
