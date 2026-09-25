//! Rendered-layout tests for the notice dismiss control and the removable
//! chip's remove control (RFC-100).
//!
//! Both controls render one shared glyph: lucide `X` under `lucide-icons`,
//! text `"×"` otherwise. Every test here runs in both feature states — the
//! expected glyph is chosen by `cfg` below — for both controls and all four
//! built-in presets.
//!
//! # The lucide font must actually be loaded
//!
//! Lucide glyphs render through `Font::with_name("lucide")`, which resolves
//! only once `lucide_icons::LUCIDE_FONT_BYTES` has been loaded. A plain
//! `iced_test::simulator` never loads it, and the codepoint then renders as
//! a fallback glyph. A natural-size-then-centre comparison would still pass,
//! because the control's glyph and the reference glyph would be the same
//! fallback, measured consistently. So every simulator here is built by
//! [`ui`], which loads the font under `lucide-icons`, and
//! `lucide::font_is_loaded` fails if it did not.
//!
//! iced's font system is **process-global**: once any simulator in this
//! binary loads the font, every later simulator sees it. A "with the font
//! versus without it" control inside one binary would therefore measure
//! whichever test happened to run first. The control instead asserts a
//! property the loaded font has and the fallback does not — see
//! `lucide::font_is_loaded`.
//!
//! # Tooltips are observed through rendered frames
//!
//! iced 0.14's tooltip overlay does not take part in widget operations, so
//! `Simulator::find` never sees tooltip text, whether or not the tooltip is
//! open. A test that looked for the text with `find` would fail against a
//! correct implementation. The tooltip tests compare hashes of rendered
//! frames instead; see `tooltip_is_drawn_on_hover_and_only_on_hover`.

#![cfg(all(feature = "widgets", feature = "design"))]

use iced::{Element, Event, Rectangle, mouse, widget::text};
use iced_test::Simulator;
use iced_test::selector::Candidate;

mod common;
#[cfg(feature = "lucide-icons")]
use common::assert_lucide_font_loaded;
use common::{MIN_TARGET, TOLERANCE, frame_hash, pointer_target, simulator as ui};

use snora::design::style::text::label_size;
use snora::design::{Tokens, Tone, chip, notice::Notice};

/// The glyph the controls must render in this feature state, and the one
/// they must not.
#[cfg(feature = "lucide-icons")]
const GLYPH: &str = "\u{e1b2}";
#[cfg(feature = "lucide-icons")]
const NOT_GLYPH: &str = "×";
#[cfg(not(feature = "lucide-icons"))]
const GLYPH: &str = "×";
#[cfg(not(feature = "lucide-icons"))]
const NOT_GLYPH: &str = "\u{e1b2}";

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Act,
    Dismiss,
    Toggle,
    Remove,
}

#[derive(Debug, Clone, Copy)]
enum Control {
    NoticeDismiss,
    ChipRemove,
}

impl Control {
    const ALL: [Control; 2] = [Control::NoticeDismiss, Control::ChipRemove];

    /// The message the control under test emits.
    fn message(self) -> Msg {
        match self {
            Control::NoticeDismiss => Msg::Dismiss,
            Control::ChipRemove => Msg::Remove,
        }
    }

    /// Renders the control in a realistic setting: the notice carries an
    /// action button beside its dismiss button, and the chip its label
    /// button beside its remove button, so the tests must pick the right
    /// one of two adjacent buttons.
    fn render(self, tokens: &Tokens) -> Element<'_, Msg> {
        self.render_with_tooltip(tokens, None)
    }

    /// As [`Control::render`], through the tooltip API when `tooltip` is
    /// given and the plain API otherwise.
    fn render_with_tooltip<'a>(
        self,
        tokens: &'a Tokens,
        tooltip: Option<&str>,
    ) -> Element<'a, Msg> {
        match (self, tooltip) {
            (Control::NoticeDismiss, tooltip) => {
                let notice = Notice::new(tokens, Tone::Info, "Index rebuilt.")
                    .action("Undo", Msg::Act)
                    .dismiss(Msg::Dismiss);
                match tooltip {
                    Some(tooltip) => notice.dismiss_tooltip(tooltip).render(),
                    None => notice.render(),
                }
            }
            (Control::ChipRemove, None) => {
                chip::removable(tokens, "Rust", false, Msg::Toggle, Msg::Remove)
            }
            (Control::ChipRemove, Some(tooltip)) => chip::removable_with_tooltip(
                tokens,
                "Rust",
                false,
                Msg::Toggle,
                Msg::Remove,
                tooltip,
            ),
        }
    }
}

fn presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

/// The glyph rendered alone at the given size, in the font this feature
/// state uses — built independently of the production helper, so a wrong
/// glyph or font there cannot also be the reference.
fn reference_glyph<'a>(size: iced::Pixels) -> Element<'a, Msg> {
    let glyph = text(GLYPH).size(size);
    #[cfg(feature = "lucide-icons")]
    let glyph = glyph.font(iced::Font::with_name("lucide"));
    glyph.into()
}

fn glyph_bounds(ui: &mut Simulator<'_, Msg>, label: &str) -> Rectangle {
    ui.find(GLYPH)
        .unwrap_or_else(|_| panic!("{label}: glyph {GLYPH:?} is not rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{label}: glyph {GLYPH:?} is not visible"))
}

/// [`common::pointer_target`] for one of these controls: the glyph's
/// centre is a point known to be inside it, and the control's own message
/// is what a click on it must produce.
fn control_target(control: Control, tokens: &Tokens, label: &str) -> Rectangle {
    let inside = {
        let mut ui = ui(control.render(tokens));
        glyph_bounds(&mut ui, label).center()
    };
    let message = control.message();
    pointer_target(
        label,
        || control.render(tokens),
        inside,
        |m: &Msg| *m == message,
    )
}

// ---------------------------------------------------------------------------
// (a) The glyph
// ---------------------------------------------------------------------------

#[test]
fn controls_render_the_feature_state_glyph() {
    for control in Control::ALL {
        for (preset, tokens) in presets() {
            let label = format!("{control:?} / {preset}");
            let mut ui = ui(control.render(&tokens));
            assert!(
                ui.find(GLYPH).is_ok(),
                "{label}: expected glyph {GLYPH:?} is not rendered"
            );
            assert!(
                ui.find(NOT_GLYPH).is_err(),
                "{label}: glyph {NOT_GLYPH:?} is rendered, but this feature state must render {GLYPH:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// (b) Pointer target
// ---------------------------------------------------------------------------

#[test]
fn controls_clear_the_24px_pointer_target() {
    for control in Control::ALL {
        for (preset, tokens) in presets() {
            let label = format!("{control:?} / {preset}");
            let target = control_target(control, &tokens, &label);
            assert!(
                target.width >= MIN_TARGET && target.height >= MIN_TARGET,
                "{label}: pointer target is {}x{}, below the {MIN_TARGET}x{MIN_TARGET} floor",
                target.width,
                target.height,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// (c) Centring
// ---------------------------------------------------------------------------

/// Natural size first, then centre (RFC-099): a glyph box that fills its
/// button is centred by construction while the glyph itself is drawn at
/// the box's top-left, so the size assertion is what gives the centre
/// assertion meaning.
#[test]
fn glyph_is_natural_size_and_centred_in_its_target() {
    for control in Control::ALL {
        for (preset, tokens) in presets() {
            let label = format!("{control:?} / {preset}");

            let natural = {
                let mut reference = ui(reference_glyph(label_size(&tokens)));
                glyph_bounds(&mut reference, &label).size()
            };
            let target = control_target(control, &tokens, &label);
            let glyph = {
                let mut ui = ui(control.render(&tokens));
                glyph_bounds(&mut ui, &label)
            };

            assert!(
                (glyph.width - natural.width).abs() <= TOLERANCE
                    && (glyph.height - natural.height).abs() <= TOLERANCE,
                "{label}: glyph box is {}x{} but the glyph's natural size is {}x{}",
                glyph.width,
                glyph.height,
                natural.width,
                natural.height,
            );
            let (g, t) = (glyph.center(), target.center());
            assert!(
                (g.x - t.x).abs() <= TOLERANCE && (g.y - t.y).abs() <= TOLERANCE,
                "{label}: glyph centre {g:?} is not within {TOLERANCE}px of the target centre {t:?} \
                 (target {target:?}, glyph {glyph:?})",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Lucide font control
// ---------------------------------------------------------------------------

#[cfg(feature = "lucide-icons")]
mod lucide {
    use super::*;

    /// Fails if the lucide font was not loaded, at each preset's label
    /// size. The check itself is [`common::assert_lucide_font_loaded`];
    /// see its documentation for why it measures a 1em advance rather
    /// than comparing a loaded glyph with an unloaded one.
    #[test]
    fn font_is_loaded() {
        for (preset, tokens) in presets() {
            assert_lucide_font_loaded(preset, label_size(&tokens).0);
        }
    }
}

// ---------------------------------------------------------------------------
// (d) Tooltips
// ---------------------------------------------------------------------------

const TOOLTIP: &str = "Remove this";
const OTHER_TOOLTIP: &str = "Close";

/// Renders the control, optionally hovers its pointer target, and hashes
/// the resulting frame.
fn frame(
    control: Control,
    preset: &str,
    tokens: &Tokens,
    tooltip: Option<&str>,
    hover: bool,
) -> String {
    let label = format!("{control:?} / {preset}");
    let target = control_target(control, tokens, &label);

    let mut ui = ui(control.render_with_tooltip(tokens, tooltip));
    if hover {
        let position = target.center();
        ui.point_at(position);
        let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved { position })]);
    }
    let name = format!(
        "{control:?}-{preset}-{}-{}",
        tooltip.unwrap_or("none").replace(' ', "_"),
        if hover { "hover" } else { "idle" },
    );
    frame_hash(&mut ui, &snora::design::theme(tokens), &name)
}

/// A tooltip draws nothing until the control is hovered, draws something
/// when it is, and what it draws depends on the tooltip text.
///
/// Hovering also changes the button's own background, so "the hovered
/// frame changed" alone would prove nothing; every comparison here is
/// between two frames in the same hover state.
#[test]
fn tooltip_is_drawn_on_hover_and_only_on_hover() {
    for control in Control::ALL {
        for (preset, tokens) in presets() {
            let label = format!("{control:?} / {preset}");

            assert_eq!(
                frame(control, preset, &tokens, Some(TOOLTIP), false),
                frame(control, preset, &tokens, None, false),
                "{label}: with no hover, a control with a tooltip must render exactly like one without",
            );
            assert_ne!(
                frame(control, preset, &tokens, Some(TOOLTIP), true),
                frame(control, preset, &tokens, None, true),
                "{label}: hovering a control with a tooltip drew nothing that a control without one \
                 does not also draw — the tooltip is not shown",
            );
            assert_ne!(
                frame(control, preset, &tokens, Some(TOOLTIP), true),
                frame(control, preset, &tokens, Some(OTHER_TOOLTIP), true),
                "{label}: hovered frames for two different tooltip texts are identical — \
                 the tooltip does not show the text it was given",
            );
        }
    }
}

/// The plain API (`Notice::dismiss` without `dismiss_tooltip`, and
/// `chip::removable`) attaches no tooltip at all.
///
/// Checked structurally, because frames cannot show absence on their own:
/// iced's tooltip widget reports itself to widget operations as a container
/// with exactly its content's bounds. With a tooltip there is one more
/// container at the pointer target's bounds than without.
#[test]
fn no_tooltip_is_attached_when_none_is_given() {
    fn containers_at(
        control: Control,
        tokens: &Tokens,
        tooltip: Option<&str>,
        target: Rectangle,
    ) -> usize {
        let mut ui = ui(control.render_with_tooltip(tokens, tooltip));
        let mut count = 0;
        let _ = ui.find(|candidate: Candidate<'_>| {
            if matches!(candidate, Candidate::Container { .. }) && candidate.bounds() == target {
                count += 1;
            }
            None::<()>
        });
        count
    }

    for control in Control::ALL {
        for (preset, tokens) in presets() {
            let label = format!("{control:?} / {preset}");
            let target = control_target(control, &tokens, &label);
            let without = containers_at(control, &tokens, None, target);
            let with = containers_at(control, &tokens, Some(TOOLTIP), target);
            assert_eq!(
                with,
                without + 1,
                "{label}: expected exactly one tooltip wrapper more with a tooltip than without \
                 (without: {without}, with: {with}) — the plain API attaches a tooltip, or the \
                 tooltip API does not",
            );
        }
    }
}
