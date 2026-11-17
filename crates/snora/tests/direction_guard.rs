//! RFC-133 R-3: **every snora widget function that returns an element takes a direction, or says why
//! it has no reading order.**
//!
//! Without this, the next widget with a reading order ships left-to-right under RTL, which is how
//! eight functions came to (RFC-132's second audit found them). The guard is built from the
//! **exports**, not from rows: a list of "public functions with a `row`" misses the footers (a
//! container whose content hugs the left) and the progress widgets (a column over a bar), which is
//! four of the eight (RFC-133 Unit 0). What a widget *is*, is what it returns.
//!
//! # How the list is built
//!
//! Every non-test source file of `snora-widgets` is read. For each `pub fn` it takes the **return
//! type after the first `->` that sits outside every bracket** (a `where F: Fn(..) -> X` clause is
//! not it: the first version of this parser read it as the return type and missed `render_menu`),
//! and the type the function is an `impl` of. A function is a **widget function** if it returns an
//! `Element` or a `LogicalRow`, or if it is the `new` of a type whose `render` returns an
//! `Element` (a builder: `Notice`), **or if it returns a type that converts into an `Element`**
//! (`impl From<Checkbox> for Element`: RFC-134's five, whose constructors are `checkbox(is_checked,
//! direction)` and the others, and whose builder methods return `Self` and are not constructors).
//!
//! Each widget function either has a `LayoutDirection` in its signature, or is in
//! [`DIRECTION_FREE`] with the reason written beside it. A function that is neither fails.
//!
//! **What is not scanned, and why:** the engine's `render` and `responsive_render` (in `snora`, not
//! `snora-widgets`) carry the direction in `AppLayout`, which is the one place an application sets
//! it, so they take none of their own.
//!
//! # What the guard cannot see
//!
//! A function that takes a `LayoutDirection` and ignores it passes. Each widget's own test
//! (`five_widgets_mirrored.rs`, `mirrored_layout.rs`) is what reads the pixels and the bounds.
//! And a widget that is built from a function this list calls "direction-free" and has a reading
//! order after all is a wrong entry: every entry is checked to be live and to carry a reason, so a
//! reviewer reads each reason once, here.

use std::collections::BTreeSet;

#[derive(Debug, Clone)]
struct Func {
    /// Path under `snora-widgets/src`.
    file: String,
    owner: Option<String>,
    name: String,
    sig: String,
}

impl Func {
    fn id(&self) -> String {
        match &self.owner {
            Some(o) => format!("{}::{o}::{}", self.file, self.name),
            None => format!("{}::{}", self.file, self.name),
        }
    }
    fn takes_a_direction(&self) -> bool {
        self.sig.contains("LayoutDirection")
    }
}

/// The widget functions with **no reading order**, each with why. A reader checks each reason once.
const DIRECTION_FREE: [(&str, &str); 16] = [
    (
        "design/button.rs::primary",
        "one centred label in one button",
    ),
    (
        "design/button.rs::primary_maybe",
        "one centred label in one button",
    ),
    (
        "design/button.rs::secondary",
        "one centred label in one button",
    ),
    (
        "design/button.rs::secondary_maybe",
        "one centred label in one button",
    ),
    ("design/button.rs::ghost", "one centred label in one button"),
    (
        "design/button.rs::ghost_maybe",
        "one centred label in one button",
    ),
    ("design/button.rs::link", "one centred label in one button"),
    (
        "design/button.rs::danger",
        "one centred label in one button",
    ),
    (
        "design/button.rs::danger_maybe",
        "one centred label in one button",
    ),
    (
        "design/card.rs::surface",
        "a container around the caller's content; it lays out nothing of its own",
    ),
    (
        "design/card.rs::raised",
        "a container around the caller's content; it lays out nothing of its own",
    ),
    (
        "design/card.rs::selected",
        "a container around the caller's content; it lays out nothing of its own",
    ),
    ("design/chip.rs::filter", "one label in one button"),
    (
        "design/notice.rs::Notice::render",
        "the builder: `Notice::new` takes the direction",
    ),
    ("icon.rs::icon_element", "one glyph"),
    ("icon.rs::icon_element_sized", "one glyph"),
];

// ---- reading the source ---------------------------------------------------------------------------------------------------------

/// Every `fn` in a source file with its visibility, owner type (the `impl` it sits in) and signature.
fn parse(file: &str, src: &str) -> Vec<Func> {
    // `impl` blocks: (start of body, end of body, owner type).
    let mut impls: Vec<(usize, usize, String)> = Vec::new();
    for (i, _) in src.match_indices("impl") {
        let line_start = src[..i].rfind('\n').map_or(0, |p| p + 1);
        if !src[line_start..i].trim().is_empty() {
            continue;
        }
        let Some(brace) = src[i..].find('{') else {
            continue;
        };
        let header = &src[i..i + brace];
        let owner: String = if header.contains(" for ") {
            header.rsplit(" for ").next().unwrap_or("")
        } else {
            // `impl<'a, M> Name<'a, M>`: the first identifier after the generics.
            let after = header.trim_start_matches("impl").trim_start();
            if after.starts_with('<') {
                let mut depth = 0;
                let mut end = 0;
                for (k, c) in after.char_indices() {
                    match c {
                        '<' => depth += 1,
                        '>' => {
                            depth -= 1;
                            if depth == 0 {
                                end = k + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                &after[end..]
            } else {
                after
            }
        }
        .trim()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
        let start = i + brace;
        impls.push((start, matching_brace(src, start), owner));
    }

    let mut out = Vec::new();
    for (i, _) in src.match_indices("fn ") {
        let before = &src[..i];
        if before
            .chars()
            .last()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let line_start = before.rfind('\n').map_or(0, |p| p + 1);
        let prefix = src[line_start..i].trim();
        if prefix.contains("//") {
            continue;
        }
        // `pub fn` and `pub const fn`, but not `pub(crate) fn` or `pub(super) fn`.
        let public = prefix == "pub" || (prefix.starts_with("pub ") && !prefix.starts_with("pub("));
        if !public {
            continue;
        }
        let rest = &src[i + 3..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let Some(open) = rest.find(['{', ';']) else {
            continue;
        };
        let owner = impls
            .iter()
            .find(|(s, e, _)| i > *s && i < *e)
            .map(|x| x.2.clone());
        out.push(Func {
            file: file.to_owned(),
            owner,
            name,
            sig: rest[..open].to_owned(),
        });
    }
    out
}

fn matching_brace(src: &str, start: usize) -> usize {
    let mut depth = 0i32;
    for (j, c) in src[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return start + j;
                }
            }
            _ => {}
        }
    }
    src.len()
}

/// The return type: after the first `->` outside every bracket (a `where F: Fn(..) -> X` is not it).
fn return_type(sig: &str) -> String {
    let cs: Vec<char> = sig.chars().collect();
    let (mut depth, mut seen_params) = (0i32, false);
    for i in 0..cs.len() {
        match cs[i] {
            '(' | '<' | '[' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    seen_params = true;
                }
            }
            '>' if i > 0 && cs[i - 1] == '-' => {
                if depth == 0 && seen_params {
                    let rest: String = cs[i + 1..].iter().collect();
                    return rest
                        .split("where")
                        .next()
                        .unwrap_or("")
                        .trim()
                        .replace('\n', " ");
                }
            }
            '>' | ']' => depth -= 1,
            _ => {}
        }
    }
    String::new()
}

fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let name = p.to_string_lossy().to_string();
            // Test modules: `tests.rs`, `tests/`, and the registers the tests read.
            if name.contains("/tests")
                || name.ends_with("tests.rs")
                || name.contains("contrast_tests")
                || name.contains("channel_register")
                || name.contains("state_register")
            {
                continue;
            }
            out.push(p);
        }
    }
}

fn widget_crate_functions() -> Vec<Func> {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("snora-widgets")
        .join("src");
    let mut files = Vec::new();
    walk(&src, &mut files);
    assert!(!files.is_empty(), "no source files under {}", src.display());
    let mut all = Vec::new();
    for p in files {
        let rel = p
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        all.extend(parse(&rel, &std::fs::read_to_string(&p).unwrap()));
    }
    all
}

/// The types that convert into an `Element`: `impl<..> From<Name<..>> for Element<..>`, read from the
/// source text.
fn element_types_in(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, _) in src.match_indices("for Element<") {
        // The header is what precedes it on the `impl` line(s): `impl<'a, M> From<Name<'a, M>> `.
        let header_start = src[..i].rfind("impl").unwrap_or(0);
        let header = &src[header_start..i];
        if let Some(at) = header.find("From<") {
            let name: String = header[at + 5..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.insert(name);
            }
        }
    }
    out
}

fn element_types() -> BTreeSet<String> {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("snora-widgets")
        .join("src");
    let mut files = Vec::new();
    walk(&src, &mut files);
    files
        .iter()
        .flat_map(|p| element_types_in(&std::fs::read_to_string(p).unwrap()))
        .collect()
}

/// The widget functions: those that return an element (or a `LogicalRow`, or a type that becomes an
/// element), and the constructor of a builder whose `render` does.
fn widget_functions(all: &[Func]) -> Vec<Func> {
    widget_functions_of(all, &element_types())
}

fn widget_functions_of(all: &[Func], types: &BTreeSet<String>) -> Vec<Func> {
    let builders: BTreeSet<(String, String)> = all
        .iter()
        .filter(|f| f.name == "render" && return_type(&f.sig).contains("Element"))
        .filter_map(|f| f.owner.clone().map(|o| (f.file.clone(), o)))
        .collect();
    all.iter()
        .filter(|f| {
            let r = return_type(&f.sig);
            r.contains("Element")
                || r.contains("LogicalRow")
                || types
                    .iter()
                    .any(|t| r == *t || r.starts_with(&format!("{t}<")))
                || (f.name == "new"
                    && f.owner
                        .clone()
                        .is_some_and(|o| builders.contains(&(f.file.clone(), o))))
        })
        .cloned()
        .collect()
}

/// Every widget function that takes no direction and is not on the direction-free list.
fn unclassified(widgets: &[Func]) -> Vec<String> {
    widgets
        .iter()
        .filter(|f| !f.takes_a_direction())
        .filter(|f| !DIRECTION_FREE.iter().any(|(id, _)| *id == f.id()))
        .map(Func::id)
        .collect()
}

// ---- the guard --------------------------------------------------------------------------------------------------------------------

#[test]
fn every_widget_function_takes_a_direction_or_says_why_it_has_none() {
    let widgets = widget_functions(&widget_crate_functions());
    assert!(
        widgets.len() >= 30,
        "the parser found {} widget functions; it found 36 when it was written, and 46 with RFC-134's \
         ten. It has stopped reading the source",
        widgets.len()
    );
    let missing = unclassified(&widgets);
    println!(
        "{} widget functions: {} take a direction, {} are direction-free",
        widgets.len(),
        widgets.iter().filter(|f| f.takes_a_direction()).count(),
        DIRECTION_FREE.len()
    );
    assert!(
        missing.is_empty(),
        "these widget functions return an element and take no `LayoutDirection`, and are not on \
         `DIRECTION_FREE`:\n  {}\n\
         A widget with a reading order (a row, a bar that fills, content at one edge, text that \
         aligns) takes a required `direction: LayoutDirection` as its last argument and lays out \
         with `LogicalRow` (RFC-133). A widget with none (one label, one glyph, a container around \
         the caller's content) goes on `DIRECTION_FREE` with its reason.",
        missing.join("\n  ")
    );
}

/// An entry that names no function, or a function that now takes a direction, is stale; and every
/// entry carries a reason.
#[test]
fn the_direction_free_list_is_live_and_every_entry_has_a_reason() {
    let widgets = widget_functions(&widget_crate_functions());
    let ids: Vec<(String, bool)> = widgets
        .iter()
        .map(|f| (f.id(), f.takes_a_direction()))
        .collect();
    for (id, reason) in DIRECTION_FREE {
        assert!(
            !reason.trim().is_empty(),
            "{id}: a direction-free entry needs a reason"
        );
        let found = ids.iter().find(|(i, _)| i == id);
        let (_, takes) = found.unwrap_or_else(|| {
            panic!(
                "`DIRECTION_FREE` names `{id}`, which is not a widget function: remove the entry"
            )
        });
        assert!(
            !takes,
            "`{id}` takes a `LayoutDirection` now: remove it from `DIRECTION_FREE`"
        );
    }
}

/// R-3 asks the guard to be shown able to fail: a ninth function with a reading order and no
/// direction is reported, with the id a reader looks for. (The same check on the real tree is
/// planted in the breakage script, which edits a source file.)
#[test]
fn a_planted_widget_with_no_direction_is_found() {
    let mut all = widget_crate_functions();
    let before = unclassified(&widget_functions(&all));
    all.push(Func {
        file: "design/toolbar.rs".into(),
        owner: None,
        name: "toolbar".into(),
        sig: "toolbar<'a, Message>(items: Vec<Element<'a, Message>>) -> Element<'a, Message> "
            .into(),
    });
    let after = unclassified(&widget_functions(&all));
    assert_eq!(after.len(), before.len() + 1, "{after:?}");
    assert_eq!(
        after.last().map(String::as_str),
        Some("design/toolbar.rs::toolbar")
    );

    // A builder whose `render` returns an element is a widget too, and its constructor is where the
    // direction goes.
    let mut all = widget_crate_functions();
    all.push(Func {
        file: "design/banner.rs".into(),
        owner: Some("Banner".into()),
        name: "render".into(),
        sig: "render(self) -> Element<'a, Message> ".into(),
    });
    all.push(Func {
        file: "design/banner.rs".into(),
        owner: Some("Banner".into()),
        name: "new".into(),
        sig: "new(tokens: &Tokens) -> Self ".into(),
    });
    let after = unclassified(&widget_functions(&all));
    assert!(
        after.contains(&"design/banner.rs::Banner::new".to_owned()),
        "a builder's constructor without a direction is not found: {after:?}"
    );
}

/// RFC-134: a function that returns a **builder type** that converts into an element is a widget
/// function, though it does not return an `Element` itself, and so is checked for a direction. The real
/// tree's ten (five on each path) are found, and a planted eleventh with no direction is reported.
#[test]
fn a_constructor_returning_a_type_that_becomes_an_element_is_a_widget_function() {
    let types = element_types();
    for t in ["Checkbox", "Toggler", "RadioGroup", "Select", "Slider"] {
        assert!(
            types.contains(t),
            "{t} is not read as converting into an element: {types:?}"
        );
    }
    let all = widget_crate_functions();
    let found = widget_functions_of(&all, &types);
    for f in [
        "form.rs::checkbox",
        "form.rs::toggler",
        "form.rs::radio_group",
        "form.rs::select",
        "form.rs::slider",
        "design/form.rs::checkbox",
        "design/form.rs::toggler",
        "design/form.rs::radio_group",
        "design/form.rs::select",
        "design/form.rs::slider",
    ] {
        assert!(
            found.iter().any(|w| w.id() == f && w.takes_a_direction()),
            "{f} is not read as a widget function that takes a direction"
        );
    }
    // THE PLANT: a sixth builder, with a reading order and no direction.
    let mut types = types;
    types.extend(element_types_in(
        "impl<'a, M> From<Dial<'a, M>> for Element<'a, M> { fn from(d: Dial<'a, M>) -> Self { todo!() } }",
    ));
    let mut all = all;
    all.push(Func {
        file: "form.rs".into(),
        owner: None,
        name: "dial".into(),
        sig: "dial<'a, M>(value: f32) -> Dial<'a, M> ".into(),
    });
    let missing = unclassified(&widget_functions_of(&all, &types));
    assert_eq!(missing, vec!["form.rs::dial".to_owned()], "{missing:?}");
}

/// The parser reads the return type after the first `->` outside every bracket, not the last one: a
/// `where` clause with an `Fn(..) -> X` follows it. (`render_menu` was missed by the first version of
/// this parser, which took the `where` clause's.)
#[test]
fn the_return_type_is_not_a_where_clauses_closure() {
    let sig = "render_menu<'a, Message, F>(menu: Menu, on_action: &'a F, is_active: bool, \
               direction: LayoutDirection) -> Element<'a, Message> where F: Fn(MenuAction) -> Message + 'a ";
    assert_eq!(return_type(sig), "Element<'a, Message>");
}
