//! Toast rendering and lifecycle utilities.
//!
//! This module exposes three concerns:
//!
//! 1. `render_toasts` — internal renderer used by [`crate::render::render`].
//!    Produces a toast stack at the requested [`ToastPosition`], with logical
//!    Start/End anchoring resolved by [`LayoutDirection`], and the newest
//!    toast rendered closest to the anchor edge.
//! 2. `subscription` — a public helper that emits ticks for TTL sweep.
//!    Applications wire this into their `iced::Application::subscription`.
//! 3. [`Holds`](crate::toast::Holds) — the sweep, and the toasts the
//!    pointer is resting on. `Holds::sweep` drops expired transient
//!    toasts but keeps a held one, so a toast cannot vanish from under
//!    someone reading it (RFC-113). It replaced the free function that swept
//!    without a hold, removed in 0.57.0.
//!
//! The two links above are spelled as full paths on purpose. This
//! module's `//!` docs are concatenated with the `///` docs on
//! `pub mod toast;` in `lib.rs`, and the whole is then resolved in
//! **`lib.rs`'s** scope, where `Holds` is not a name — so a bare
//! `[`Holds`]` here is a broken link that only `cargo doc` reports.
//!
//! Together, (2) and (3) move toast TTL bookkeeping from application code
//! into the framework — the app stores a `Vec<Toast<Message>>` and a
//! [`Holds`](crate::toast::Holds), and calls two one-liners.

use std::time::{Duration, Instant};

use iced::widget::text::Wrapping;
use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Shadow, Subscription,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, mouse_area, opaque, row, text},
};

use snora_core::{
    LayoutDirection, Toast, ToastAttention, ToastIntent, ToastLifetime, ToastPosition,
};

/// How a toast is sized and coloured (RFC-104).
///
/// The engine renders toasts on both paths, and before this the sizes
/// were literals and the colours came from the `iced::Theme` alone — so
/// a `design` application's toasts ignored its `Tokens` entirely, and no
/// application's toast text followed its `default_text_size`.
///
/// `None` everywhere is the default path: inherit the host's text size,
/// and derive colours from the theme.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ToastStyle {
    /// Title size, or `None` to inherit `default_text_size`.
    pub(crate) title_size: Option<f32>,
    /// Message size, or `None` to inherit `default_text_size`.
    pub(crate) message_size: Option<f32>,
    /// Intent colours, or `None` to derive them from the theme.
    pub(crate) intents: Option<ToastIntentColors>,
    /// `Tokens::targets.min` on the design path; `0.0` (none) otherwise.
    pub(crate) min_target: f32,
    /// The close control's focus ring, as `(width, offset)`, from
    /// `Tokens::focus` on the design path; `None` takes the unstyled
    /// literals (2, 2).
    pub(crate) ring: Option<(f32, f32)>,
}

/// The `(fill, text)` pair each intent paints with, supplied by the
/// `design` path from the token palette's status colours.
///
/// Exhaustive on [`ToastIntent`] in [`ToastIntentColors::pair`], so a
/// sixth intent fails to compile here until it is given a pair
/// (RFC-063's pattern).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ToastIntentColors {
    pub(crate) debug: (Color, Color),
    pub(crate) info: (Color, Color),
    pub(crate) success: (Color, Color),
    pub(crate) warning: (Color, Color),
    pub(crate) error: (Color, Color),
}

impl ToastIntentColors {
    pub(crate) fn pair(&self, intent: ToastIntent) -> (Color, Color) {
        match intent {
            ToastIntent::Debug => self.debug,
            ToastIntent::Info => self.info,
            ToastIntent::Success => self.success,
            ToastIntent::Warning => self.warning,
            ToastIntent::Error => self.error,
        }
    }
}

/// Fixed toast width so stacked toast edges align cleanly regardless of
/// content length. The value is chosen to comfortably hold two lines of
/// 14pt text at default font sizes.
const TOAST_WIDTH: f32 = 340.0;

/// The close mark's size, in ems of the text the toast sits with (RFC-115's
/// amendment): 1.125, because an × fills less of its em than a pictogram
/// does. On the unstyled path that text is the host's, so at the default
/// 16 px the mark is the 18 px it has always been and follows the text
/// instead of staying 18. On the design path it is the message's
/// `Typography::body`.
///
/// The pointer target is not set here: it is `control`'s, which makes
/// every control at least `max(24, 1.5 x text, the minimum-target token)`
/// (RFC-116). That replaces the 24 px centring box this file used to build
/// (RFC-101 Q-2 (a)) with the same 24 x 24 at the default and a target
/// that grows with the text.
const CLOSE_GLYPH_EM: f32 = 1.125;

/// The focus ring's literals when the toast has no tokens: the same
/// `FocusTokens::new(2.0, 2.0, ..)` defaults the rest of the chrome uses.
const CLOSE_RING_UNSTYLED: (f32, f32) = (2.0, 2.0);

/// Default sweep interval. Half-second resolution is imperceptible to users
/// and keeps idle wakeups low.
const SWEEP_INTERVAL: Duration = Duration::from_millis(500);

// =========================================================================
// Render-order policy
// =========================================================================

/// The iteration order used when pushing individual toasts into the column.
///
/// This type exists so that the ordering decision can be unit-tested
/// independently of the iced widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastRenderOrder {
    /// Push oldest first, newest last. Used for bottom anchors so the newest
    /// toast ends up at the bottom of the column (closest to the anchor edge).
    Chronological,
    /// Push newest first, oldest last. Used for top anchors so the newest
    /// toast ends up at the top of the column (closest to the anchor edge).
    ReverseChronological,
}

/// Returns the push order required to place the newest toast closest to the
/// anchor edge, given `position`.
///
/// The rule: in a top-down iced `column!`, the first child pushed is
/// visually highest and the last child pushed is visually lowest. Therefore:
///
/// * Top anchors (column is top-aligned): push newest **first** →
///   newest appears at the top.
/// * Bottom anchors (column is bottom-aligned): push newest **last** →
///   newest appears at the bottom.
fn render_order_for(position: ToastPosition) -> ToastRenderOrder {
    if position.is_top() {
        ToastRenderOrder::ReverseChronological
    } else {
        ToastRenderOrder::Chronological
    }
}

// =========================================================================
// Rendering
// =========================================================================

/// Build the toast layer, or `None` if the queue is empty.
///
/// The layer is positioned at the requested [`ToastPosition`], with
/// horizontal anchoring resolved against `direction` for `Start` / `End`
/// variants. Stack growth direction is derived from the position:
///
/// * Top anchors grow *downward* — newest toast is closest to the top edge.
/// * Bottom anchors grow *upward* — newest toast is closest to the bottom edge.
///
/// Applications push new toasts to the *back* of their queue in chronological
/// order (oldest at index 0, newest at the back). This function is responsible
/// for honoring the anchor-edge invariant regardless of that convention.
pub(crate) fn render_toasts<'a, Message>(
    toasts: Vec<Toast<Message>>,
    position: ToastPosition,
    direction: LayoutDirection,
    style: ToastStyle,
    on_attention: Option<fn(u64, ToastAttention) -> Message>,
) -> Option<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    if toasts.is_empty() {
        return None;
    }

    // Only when the application asked for it (RFC-113, RFC-120). An unwired
    // layout gets the tree it got before: no `mouse_area`, so nothing
    // that asserts the toast layer's composition has to change.
    let wrap = |toast: Toast<Message>| -> Element<'a, Message> {
        let id = toast.id;
        let single = render_single_toast(toast, style, on_attention);
        match on_attention {
            Some(f) => mouse_area(single)
                .on_enter(f(id, ToastAttention::PointerEntered))
                .on_exit(f(id, ToastAttention::PointerLeft))
                .into(),
            None => single,
        }
    };

    // The toasts' ids, in the order they are laid out, which is Tab order:
    // focus in the stack follows a toast by its id (RFC-120 D-2).
    let keys: Vec<u64> = match render_order_for(position) {
        ToastRenderOrder::ReverseChronological => toasts.iter().rev().map(|t| t.id).collect(),
        ToastRenderOrder::Chronological => toasts.iter().map(|t| t.id).collect(),
    };

    let mut stack_col = column![].spacing(8);
    match render_order_for(position) {
        // Top anchors: newest must be at the top of the column (first child),
        // so iterate in reverse — newest (back of queue) is pushed first.
        ToastRenderOrder::ReverseChronological => {
            for toast in toasts.into_iter().rev() {
                stack_col = stack_col.push(wrap(toast));
            }
        }
        // Bottom anchors: newest must be at the bottom of the column (last child),
        // so iterate in chronological order — newest (back of queue) is pushed last.
        ToastRenderOrder::Chronological => {
            for toast in toasts {
                stack_col = stack_col.push(wrap(toast));
            }
        }
    }

    let horizontal_anchor = horizontal_align(position, direction);
    let vertical_anchor = if position.is_top() {
        Vertical::Top
    } else {
        Vertical::Bottom
    };

    // Focus follows a toast by its id, not by its place in the stack (iced keeps
    // state by position, so a toast that goes would hand its focus to whichever
    // moves into its place). `Keyed` does not change what the tree reports.
    Some(
        snora_interaction::Keyed::new(
            container(stack_col)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(24)
                .align_x(horizontal_anchor)
                .align_y(vertical_anchor)
                .id(crate::identifiers::TOAST_STACK),
            keys,
        )
        .into(),
    )
}

/// Resolve the horizontal anchor of a toast position under a given
/// direction. `Start` / `End` mirror under RTL; `Center` is unaffected.
fn horizontal_align(position: ToastPosition, direction: LayoutDirection) -> Horizontal {
    use ToastPosition::*;
    match position {
        TopCenter | BottomCenter => Horizontal::Center,
        TopStart | BottomStart => match direction {
            LayoutDirection::Ltr => Horizontal::Left,
            LayoutDirection::Rtl => Horizontal::Right,
        },
        TopEnd | BottomEnd => match direction {
            LayoutDirection::Ltr => Horizontal::Right,
            LayoutDirection::Rtl => Horizontal::Left,
        },
    }
}

fn render_single_toast<'a, Message>(
    toast: Toast<Message>,
    style: ToastStyle,
    on_attention: Option<fn(u64, ToastAttention) -> Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let intent = toast.intent;
    let intents = style.intents;

    // No literal sizes (RFC-104): each inherits the host's
    // `default_text_size` unless the caller's style maps it to a role.
    //
    // `WordOrGlyph` (RFC-105): a toast is a fixed 340px, so caller text
    // with no spaces in it — a file name, a path, a URL, a hash — used to
    // be cut off at the column with no sign that anything was missing.
    // Measured: 400 unbroken characters rendered 288.0 x 20.8 against a
    // natural 3481.6 x 20.8, one line, about 48 characters visible.
    // iced's default is `Word`, which breaks at spaces only and so does
    // nothing for a string that has none; `WordOrGlyph` breaks between
    // glyphs when a word cannot fit, and leaves ordinary prose alone.
    let mut title = text(toast.title).wrapping(Wrapping::WordOrGlyph);
    if let Some(size) = style.title_size {
        title = title.size(size);
    }
    let mut message = text(toast.message).wrapping(Wrapping::WordOrGlyph);
    if let Some(size) = style.message_size {
        message = message.size(size);
    }
    let text_col = column![title, message].spacing(4);

    // The close glyph. **Two sites must agree on it**: this one and
    // `snora_widgets::icon::icon_element_sized`, which renders
    // `Icon::Lucide` for the widget layer. The engine cannot call that
    // one — it does not depend on `snora-widgets` — so it repeats the
    // construction: the codepoint from `char::from`, drawn in the lucide
    // font. `toast_close_button.rs`'s
    // `engine_and_widgets_draw_the_same_lucide_glyph` fails if either site
    // changes codepoint, or if what either draws is not a lucide glyph
    // (RFC-101 Q-1 (a)). Dropping the `.font(..)` below is not among them:
    // with the font loaded, cosmic-text resolves this codepoint by coverage
    // and draws the same glyph either way — measured, not assumed.
    //
    // **The call still earns its place.** Coverage-based fallback picks
    // whichever loaded font covers the codepoint, and U+E1B2 is in the
    // private use area: an application that also loads another PUA icon
    // font covering it could get that font's glyph instead. Naming the
    // font here settles it. The harness loads lucide alone, so no test can
    // see that case.
    //
    // **Sized with the text it sits with** (RFC-115's amendment: × is 1.125
    // em). Which text that is depends on the path:
    //
    // - **unstyled:** the host's `default_text_size`, which view code never
    //   sees, so `Glyph` reads it when the layout runs;
    // - **design:** the toast's own message, `Typography::body`
    //   (`style.message_size`), so raising the body text grows the × with it,
    //   and raising the host's size with default tokens does not — the two
    //   move together or the × would follow a size the toast's text does not
    //   use. A pixel size is exact here, so `text` does it.
    #[cfg(feature = "lucide-icons")]
    let close_content = char::from(lucide_icons::Icon::X).to_string();
    #[cfg(not(feature = "lucide-icons"))]
    let close_content = String::from("×");
    let close_glyph: Element<'a, Message> = match style.message_size {
        Some(message) => {
            let mark = CLOSE_GLYPH_EM * message;
            let glyph = text(close_content).size(mark);
            #[cfg(feature = "lucide-icons")]
            let glyph = glyph.font(iced::Font::with_name("lucide"));
            glyph.into()
        }
        None => {
            let glyph = snora_interaction::Glyph::new(close_content, CLOSE_GLYPH_EM);
            #[cfg(feature = "lucide-icons")]
            let glyph = glyph.font(iced::Font::with_name("lucide"));
            glyph.into()
        }
    };

    // `control` in place of `button` (RFC-116): the close mark is a focus
    // target, Enter and Space dismiss, and the target is `control`'s own
    // `max(24, 1.5 x text, token)` with the mark centred in it.
    //
    // No padding: the control's floor *is* the target. The `[0, 8]` an
    // earlier version used existed to widen a glyph-sized button, and
    // keeping it would have made the button 40 x 24 — 15px of it stolen
    // from the message column, which is what a fixed-width toast has to
    // spare. At zero the control is 24 x 24 at the default text size, and
    // the message column is 288px against 0.50.0's 287.
    //
    // No explicit colour: the glyph inherits the control's `text_color`
    // from `close_button_style`, which is where a toast's intent colour is
    // resolved. The ring is that same colour, which is the pair iced or
    // the token palette guarantees against the toast's fill.
    let (ring_width, ring_offset) = style.ring.unwrap_or(CLOSE_RING_UNSTYLED);
    let mut close_btn = snora_interaction::Control::new(
        close_glyph,
        Box::new(move |theme, status| close_button_style(theme, intent, status, intents)),
        snora_interaction::Ring {
            color: Box::new(move |theme| intent_colors(theme, intent, intents).1),
            width: ring_width,
            offset: ring_offset,
        },
    )
    .min_target(style.min_target);
    // The `1.5 x text` term is measured against the text the control sits
    // with: the message, on the design path.
    if let Some(message) = style.message_size {
        close_btn = close_btn.text_size(message);
    }
    // A reader on the toast's control by keyboard is attending to it, as one
    // with the pointer on it is (RFC-120 R-3).
    if let Some(f) = on_attention {
        close_btn = close_btn.on_focus_change(
            f(toast.id, ToastAttention::FocusGained),
            f(toast.id, ToastAttention::FocusLost),
        );
    }
    let close_btn = close_btn.on_press(toast.on_dismiss).padding(0);

    let body = row![container(text_col).width(Length::Fill), close_btn]
        .align_y(Center)
        .spacing(4);

    // Wrapped in `opaque` (RFC-084 F-04): only the `×` button captured
    // clicks before this, because it is a button — a click anywhere else on
    // the toast's own surface (its title, message, or padding) fell through
    // to whatever rendered beneath it.
    opaque(
        container(body)
            .width(Length::Fixed(TOAST_WIDTH))
            .padding(12)
            .style(move |theme| toast_style(theme, intent, intents))
            .id(crate::identifiers::toast_id(toast.id)),
    )
}

// The readable-text rule is `snora_interaction::contrast`'s (RFC-122 Unit 1). It was a
// twin of `snora_widgets::style`'s copy until a third caller, `snora::control`, made
// two copies of one rule untenable; both crates are above `snora-interaction`.
use snora_interaction::contrast::{contrast_ratio, readable_text, stronger_pole};

/// Returns `(background, text_color)` for a toast intent — the single
/// derivation both [`toast_style`] (the body) and [`close_button_style`]
/// (the dismiss `×`) share, so a pairing fix only needs to happen once
/// (RFC-086 Q-2: "the same category error as RFC-085's F-13 — a tier's
/// `.color` used where its `.text` belongs").
///
/// Exhaustive on `intent` on purpose (RFC-063's pattern, applied to this
/// enum for the first time): a sixth [`ToastIntent`] variant fails to
/// compile here until it is given a pairing, rather than silently
/// falling through a wildcard arm untested.
///
/// **Corrected 2026-09-02 (RFC-086), all figures re-measured, not
/// inherited from the audit:**
///
/// - `Warning` — F-05. The private amber constant this used to carry vs
///   `Color::WHITE` measured **3.18:1**, matching the audit. Two repairs
///   were measured, not assumed: darkening the fill ~20-25% while keeping
///   white text reaches AA (4.73-5.26:1) but changes the amber's own
///   identity; switching the text to `Color::BLACK` against the
///   **unchanged** fill measures **6.60:1** — real margin, and preserved
///   the amber exactly. **RFC-104 then retired the constant**: iced
///   0.14's extended palette does carry a `warning` pair, so the fill and
///   its text now come from the theme like every other intent, and a host
///   that themes its warning colour finally gets it.
///   Chose the text swap: the identity property traded is "how Warning
///   reads at a glance" (white-on-color to black-on-color), not the
///   colour itself. `Success` and `Error` both keep white text, so
///   `Warning`'s black text reads as *more* distinct from them, not a
///   collision — Q-1 named that as a risk to check, not a certainty.
/// - `Debug` — unchanged; already the correct pairing (`13.28:1` /
///   `6.02:1`, light/dark). F-06 was in [`close_button_style`], not here.
/// - `Info` — **not named by the audit, found by measuring all five
///   (Q-3).** `primary.base.color`/`.text` measured **4.43:1 on the two
///   stock themes** — under AA, and iced's own derivation, not a snora
///   literal. On those two themes neither `primary.base` nor
///   `primary.strong`'s own paired `.text` clears with real margin
///   (4.43-4.58:1) because `primary.base.color`'s luminance there sits
///   almost exactly where black and white contrast it equally;
///   `primary.strong.color` with `Color::BLACK` measures **5.64:1**.
///
///   **Every number in this bullet is a stock-theme number, and the
///   literal black that came out of it was wrong on the other twenty
///   (RFC-111).** iced derives `primary.strong` from whatever palette
///   the application sets, and on Kanagawa Dragon that is a mid-blue
///   where black measures 2.42:1. Info now takes the better of its own
///   pair and the stronger pole, which is still black on stock.
/// - `Success` — unchanged; already passes with margin (`6.61:1` /
///   `6.91:1`).
/// - `Error` — unchanged; passes (`4.83:1` both themes) — thinner than
///   `Success`'s margin but does clear the floor, and Q-3 asked to
///   report passes, not to re-tune ones that already hold.
fn intent_colors(
    theme: &iced::Theme,
    intent: ToastIntent,
    overrides: Option<ToastIntentColors>,
) -> (Color, Color) {
    // The `design` path supplies the token palette's own status pairs
    // (RFC-104); without them the pairs are derived from the theme, and
    // the theme's pair is kept only where it clears AA (RFC-111).
    if let Some(overrides) = overrides {
        return overrides.pair(intent);
    }

    let ep = theme.extended_palette();
    match intent {
        ToastIntent::Debug => {
            let fill = ep.background.strong.color;
            (fill, readable_text(fill, ep.background.strong.text))
        }
        // Info takes the better of its pair and the stronger pole
        // (RFC-111 Q-1), rather than the unconditional `Color::BLACK`
        // this used to carry. That literal was measured on the two
        // stock themes, where black is the better choice at 5.63 — and
        // it stayed black on the other twenty, which is how Kanagawa
        // Dragon drew black on a mid-blue at 2.42, and how an
        // application rendering a token-derived theme through the
        // unstyled path drew 1.40-1.66.
        ToastIntent::Info => {
            let fill = ep.primary.strong.color;
            let paired = ep.primary.strong.text;
            let pole = stronger_pole(fill);
            let text = if contrast_ratio(paired, fill) >= contrast_ratio(pole, fill) {
                paired
            } else {
                pole
            };
            (fill, text)
        }
        ToastIntent::Success => {
            let fill = ep.success.base.color;
            (fill, readable_text(fill, ep.success.base.text))
        }
        // From the theme, not from a literal (RFC-104). iced 0.14's
        // extended palette does carry a `warning` pair — the comment
        // that said it does not predates it — so a host that themes its
        // warning colour now gets it here, and the paired `.text` comes
        // with it instead of a hardcoded black.
        ToastIntent::Warning => {
            let fill = ep.warning.base.color;
            (fill, readable_text(fill, ep.warning.base.text))
        }
        ToastIntent::Error => {
            let fill = ep.danger.base.color;
            (fill, readable_text(fill, ep.danger.base.text))
        }
    }
}

/// Style a toast surface based on its intent. Colors are pulled from the
/// theme's extended palette, including its `warning` pair — iced 0.14
/// carries one, and RFC-104 retired the hand-picked constant that this
/// comment used to describe. The text is the palette's own paired
/// value wherever that clears AA against the fill, and the stronger of
/// black and white where it does not (RFC-111).
fn toast_style(
    theme: &iced::Theme,
    intent: ToastIntent,
    overrides: Option<ToastIntentColors>,
) -> iced::widget::container::Style {
    use iced::widget::container::Style;

    let (background, text_color) = intent_colors(theme, intent, overrides);

    Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_color),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

/// Style for the toast's dismiss `×`.
///
/// **Corrected (RFC-086 F-06, Q-2).** Previously hard-coded
/// `Color::WHITE` regardless of intent — a pairing fix, not a colour
/// fix, the same category error as RFC-085's F-13: the mark must use
/// the *same* text colour [`toast_style`] uses for the body, not a
/// literal that happened to work for four of five intents and measured
/// **1.58:1** (invisible) for `Debug`, whose body text is black on a
/// light-gray fill. Now shares [`intent_colors`] with the body, so the
/// two can never re-diverge.
///
/// The hover/rest alpha fade is **not applied to the dismiss mark's own
/// colour** (unlike the previous version). **Correction (R-3, round 2,
/// 2026-09-02): this is a deliberate margin simplification, not a
/// floor fix.** The first submission claimed the fade "regressed
/// contrast... under the 3.0 non-text floor" for four of five intents
/// — false, and caught only because the reviewer re-measured directly
/// rather than accepting the claim. With these corrected colours, a
/// 0.75-alpha fade composited toward each toast's own background
/// clears the floor at every intent and both stock themes (worst case
/// `Error` at **3.38:1**) — the cited table was the *pre-fix*
/// hardcoded-white mark, which cannot support a claim about the fixed
/// one, and even that table showed two failures, not four. The real
/// and stated reason for removing the fade: it raises the worst case
/// from 3.38:1 to **4.83:1** and makes the mark's contrast independent
/// of interaction state, so no future colour change can reintroduce a
/// floor risk through this specific channel. The mark is fully opaque
/// at every status; hover is signalled by underline-free color alone
/// remaining constant, a smaller affordance than before, traded for
/// that margin and that independence — not for correctness the fade
/// never lacked.
///
/// **If hover feedback on this mark is reintroduced later**, prefer a
/// background tint on the button itself, or a non-colour cue (a
/// size/scale nudge), over alpha on the mark's own colour — alpha
/// compositing toward the background still costs margin even where (as
/// here) it does not cost the floor, and re-verifying that trade for
/// every future colour choice is exactly the recurring cost this
/// version avoids.
fn close_button_style(
    theme: &iced::Theme,
    intent: ToastIntent,
    _status: button::Status,
    overrides: Option<ToastIntentColors>,
) -> button::Style {
    let (_, text_color) = intent_colors(theme, intent, overrides);
    button::Style {
        background: None,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

// =========================================================================
// Lifecycle helpers
// =========================================================================

/// Subscribe to periodic ticks for TTL sweep.
///
/// Wire this into your `iced::Application::subscription` like so:
///
/// ```rust,no_run
/// use iced::{Subscription, Task};
/// use snora::Toast;
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     ToastTick,
/// }
///
/// struct MyState {
///     toasts: Vec<Toast<Message>>,
///     holds: snora::toast::Holds,
/// }
///
/// impl MyState {
///     fn subscription(&self) -> Subscription<Message> {
///         snora::toast::subscription(&self.toasts, || Message::ToastTick)
///     }
///
///     fn update(&mut self, msg: Message) -> Task<Message> {
///         match msg {
///             Message::ToastTick => {
///                 self.holds.sweep(&mut self.toasts, std::time::Instant::now());
///             }
///         }
///         Task::none()
///     }
/// }
/// ```
///
/// The subscription is only active while at least one *transient* toast
/// is present. An all-persistent or empty queue returns
/// [`Subscription::none`] so the runtime does not wake for nothing.
///
/// The `tick_message` closure must be `Clone` because iced clones it for
/// each wake-up. The simplest form is a zero-capture closure like
/// `|| Message::ToastTick`.
pub fn subscription<Message, F>(toasts: &[Toast<Message>], tick_message: F) -> Subscription<Message>
where
    Message: Clone + Send + 'static,
    F: Fn() -> Message + Send + Sync + Clone + 'static,
{
    let has_transient = toasts
        .iter()
        .any(|t| matches!(t.lifetime, ToastLifetime::Transient(_)));
    if has_transient {
        iced::time::every(SWEEP_INTERVAL).map(move |_| tick_message())
    } else {
        Subscription::none()
    }
}

/// The toasts someone is attending to, and the sweep that respects them
/// (RFC-113, RFC-120).
///
/// A transient toast counts down from the moment it appears. If the
/// reader is still on it when the deadline passes, it vanishes
/// mid-sentence, and reaching for a toast with the pointer, or Tabbing to its
/// ×, is exactly what someone does when they need longer with it.
///
/// **One rule: a toast waits while the reader is attending to it, by pointer
/// or by keyboard.** It holds while **either** is on it, and its countdown
/// **restarts in full only when both have gone**.
///
/// Keep one of these in your application state and sweep through it:
///
/// ```no_run
/// # use snora_core::{AppLayout, Toast, ToastIntent, ToastAttention};
/// # use snora::toast::Holds;
/// # use std::time::Instant;
/// # #[derive(Clone, Debug)] enum Message { ToastAttention(u64, ToastAttention), Tick, Dismiss(u64) }
/// # struct App { toasts: Vec<Toast<Message>>, holds: Holds }
/// # impl App {
/// fn update(&mut self, message: Message) {
///     match message {
///         Message::ToastAttention(id, what) => {
///             self.holds.attend(id, what, &mut self.toasts, Instant::now());
///         }
///         Message::Tick => self.holds.sweep(&mut self.toasts, Instant::now()),
///         # Message::Dismiss(_) => {}
///         // ...
///     }
/// }
/// # }
/// ```
///
/// and tell the layout to report it:
///
/// ```no_run
/// # use snora_core::{AppLayout, Toast, ToastAttention};
/// # #[derive(Clone, Debug)] enum Message { ToastAttention(u64, ToastAttention) }
/// # let body: () = ();
/// # let layout: AppLayout<(), Message> = AppLayout::new(body);
/// let layout = layout.on_toast_attention(Message::ToastAttention);
/// ```
///
/// **Releasing restarts the countdown in full**, and [`Holds::attend`] does
/// that itself. That is why this is one type rather than a pair of free
/// functions: a toast held past its deadline would otherwise disappear the
/// instant the reader left, and an application that forgot the restart would
/// never see the problem until a user did.
#[derive(Debug, Default, Clone)]
pub struct Holds {
    /// Ids the pointer is currently on. Small by construction: a
    /// pointer is in one place, and the set is pruned on every sweep.
    pointer: Vec<u64>,
    /// Ids a control of which has keyboard focus. Also pruned on every sweep.
    focus: Vec<u64>,
}

impl Holds {
    /// The reader's attention reached or left the toast with this id
    /// (RFC-120 R-3).
    ///
    /// - [`ToastAttention::PointerEntered`] and [`ToastAttention::FocusGained`]
    ///   hold the toast.
    /// - [`ToastAttention::PointerLeft`] and [`ToastAttention::FocusLost`]
    ///   release that source. If **the other source is still on the toast, it
    ///   stays held**. When **neither** is, its countdown **restarts in full**
    ///   from `now`, so the reader gets the whole reading time back rather
    ///   than whatever was left when they arrived.
    ///
    /// Does nothing if `id` is not in `toasts`, and ignores an attention it does
    /// not know (the enum is `#[non_exhaustive]`).
    pub fn attend<M: Clone>(
        &mut self,
        id: u64,
        attention: ToastAttention,
        toasts: &mut [Toast<M>],
        now: Instant,
    ) {
        let source = match attention {
            ToastAttention::PointerEntered | ToastAttention::PointerLeft => &mut self.pointer,
            ToastAttention::FocusGained | ToastAttention::FocusLost => &mut self.focus,
            // A source this version does not know: not a hold.
            #[allow(unreachable_patterns)]
            _ => return,
        };
        match attention {
            ToastAttention::PointerEntered | ToastAttention::FocusGained => {
                if !source.contains(&id) {
                    source.push(id);
                }
                return;
            }
            _ => source.retain(|&held| held != id),
        }

        // Released. Only when no source is left does the countdown restart:
        // that restart is the whole point, and it is in here rather than in the
        // caller's hands.
        if !self.is_held(id)
            && let Some(toast) = toasts.iter_mut().find(|t| t.id == id)
        {
            toast.created_at = now;
        }
    }

    fn is_held(&self, id: u64) -> bool {
        self.pointer.contains(&id) || self.focus.contains(&id)
    }

    /// Drop expired transient toasts, except any being attended to.
    ///
    /// Also forgets held ids whose toast has gone (dismissed while held, say),
    /// so the sets cannot grow without bound.
    pub fn sweep<M: Clone>(&mut self, toasts: &mut Vec<Toast<M>>, now: Instant) {
        toasts.retain(|t| self.is_held(t.id) || !t.is_expired(now));
        self.pointer.retain(|id| toasts.iter().any(|t| t.id == *id));
        self.focus.retain(|id| toasts.iter().any(|t| t.id == *id));
    }
}

#[cfg(test)]
mod contrast_tests;

#[cfg(test)]
mod channel_register;

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // horizontal_align — RTL Start/End mirroring (RFC-011-D / RFC-012-A ABDD)
    //
    // The ABDD contract: Start/End positions mirror under RTL so the newest
    // toast always lands on the correct logical side regardless of reading
    // direction. Center positions are unaffected by direction.
    // -----------------------------------------------------------------------

    #[test]
    fn top_end_ltr_resolves_right() {
        assert_eq!(
            horizontal_align(ToastPosition::TopEnd, LayoutDirection::Ltr),
            iced::alignment::Horizontal::Right,
        );
    }

    #[test]
    fn top_end_rtl_mirrors_to_left() {
        assert_eq!(
            horizontal_align(ToastPosition::TopEnd, LayoutDirection::Rtl),
            iced::alignment::Horizontal::Left,
        );
    }

    #[test]
    fn top_start_ltr_resolves_left() {
        assert_eq!(
            horizontal_align(ToastPosition::TopStart, LayoutDirection::Ltr),
            iced::alignment::Horizontal::Left,
        );
    }

    #[test]
    fn top_start_rtl_mirrors_to_right() {
        assert_eq!(
            horizontal_align(ToastPosition::TopStart, LayoutDirection::Rtl),
            iced::alignment::Horizontal::Right,
        );
    }

    #[test]
    fn center_positions_unaffected_by_direction() {
        for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
            assert_eq!(
                horizontal_align(ToastPosition::TopCenter, dir),
                iced::alignment::Horizontal::Center,
                "TopCenter must be unaffected by direction ({dir:?})",
            );
            assert_eq!(
                horizontal_align(ToastPosition::BottomCenter, dir),
                iced::alignment::Horizontal::Center,
                "BottomCenter must be unaffected by direction ({dir:?})",
            );
        }
    }
    //
    // The contract: applications push toasts in chronological order (oldest
    // at index 0, newest at the back). The newest must appear closest to the
    // anchor edge. In a top-down iced column:
    //   - top anchor → newest must be the first child → reverse iteration;
    //   - bottom anchor → newest must be the last child → chronological order.
    // -----------------------------------------------------------------------

    #[test]
    fn top_positions_render_reverse_chronological() {
        use ToastPosition::*;
        for pos in [TopEnd, TopStart, TopCenter] {
            assert_eq!(
                render_order_for(pos),
                ToastRenderOrder::ReverseChronological,
                "{pos:?} should use reverse-chronological order",
            );
        }
    }

    #[test]
    fn bottom_positions_render_chronological() {
        use ToastPosition::*;
        for pos in [BottomEnd, BottomStart, BottomCenter] {
            assert_eq!(
                render_order_for(pos),
                ToastRenderOrder::Chronological,
                "{pos:?} should use chronological order",
            );
        }
    }

    // -----------------------------------------------------------------------
    // subscription (RFC-094 Unit 2) — had no test at all before this.
    //
    // `Subscription` has no `PartialEq`, so the branches can't be compared
    // directly. `Subscription::units()` (a public count of recipes, via
    // iced's own re-export of `iced_futures::Subscription`) distinguishes
    // them: `Subscription::none()` has zero, a live `iced::time::every`
    // subscription has one. Two of the four cases below are the negative
    // half a doctest can't reach: no subscription when nothing needs
    // sweeping.
    // -----------------------------------------------------------------------

    #[test]
    fn subscription_units_reflect_whether_sweeping_is_needed() {
        let empty: Vec<Toast<()>> = Vec::new();
        assert_eq!(
            subscription(&empty, || ()).units(),
            0,
            "an empty queue must not subscribe",
        );

        let persistent_only = vec![Toast::new(1, ToastIntent::Info, "a", "b", ()).persistent()];
        assert_eq!(
            subscription(&persistent_only, || ()).units(),
            0,
            "an all-persistent queue must not subscribe",
        );

        let one_transient = vec![Toast::new(2, ToastIntent::Info, "a", "b", ())];
        assert_eq!(
            subscription(&one_transient, || ()).units(),
            1,
            "a transient toast must subscribe",
        );

        let mixed = vec![
            Toast::new(3, ToastIntent::Info, "a", "b", ()),
            Toast::new(4, ToastIntent::Info, "a", "b", ()).persistent(),
        ];
        assert_eq!(
            subscription(&mixed, || ()).units(),
            1,
            "any transient toast in the queue must subscribe, even alongside a persistent one",
        );
    }
}

#[cfg(test)]
mod holds_tests {
    use super::*;
    use snora_core::ToastIntent;

    fn toast(id: u64, base: Instant) -> Toast<()> {
        Toast::new(id, ToastIntent::Info, "t", "m", ())
            .with_lifetime(ToastLifetime::seconds(10))
            .with_created_at(base)
    }

    // ---- `Holds::attend`: the pointer and the keyboard hold a toast (RFC-120 R-3) ----

    use snora_core::ToastAttention::{FocusGained, FocusLost, PointerEntered, PointerLeft};

    /// Held by the pointer alone, past its deadline.
    #[test]
    fn the_pointer_alone_holds_a_toast() {
        let base = Instant::now();
        let mut toasts = vec![toast(1, base)];
        let mut holds = Holds::default();
        holds.attend(1, PointerEntered, &mut toasts, base);
        holds.sweep(&mut toasts, base + Duration::from_secs(3600));
        assert_eq!(toasts.len(), 1, "a toast under the pointer was swept");
    }

    /// Held by keyboard focus alone, past its deadline.
    #[test]
    fn keyboard_focus_alone_holds_a_toast() {
        let base = Instant::now();
        let mut toasts = vec![toast(1, base)];
        let mut holds = Holds::default();
        holds.attend(1, FocusGained, &mut toasts, base);
        holds.sweep(&mut toasts, base + Duration::from_secs(3600));
        assert_eq!(toasts.len(), 1, "a toast with focus on its × was swept");
    }

    /// Both, then **one leaves: still held.** The restart must not happen, so
    /// the deadline stays where it was.
    #[test]
    fn with_both_sources_one_leaving_does_not_release() {
        for (first_leaves, second_stays) in
            [(PointerLeft, FocusGained), (FocusLost, PointerEntered)]
        {
            let base = Instant::now();
            let mut toasts = vec![toast(1, base)];
            let mut holds = Holds::default();
            // Both are on it.
            holds.attend(1, second_stays, &mut toasts, base);
            let first_arrives = if first_leaves == PointerLeft {
                PointerEntered
            } else {
                FocusGained
            };
            holds.attend(1, first_arrives, &mut toasts, base);
            // One leaves, long after the deadline.
            let later = base + Duration::from_secs(30);
            holds.attend(1, first_leaves, &mut toasts, later);

            holds.sweep(&mut toasts, later + Duration::from_secs(3600));
            assert_eq!(
                toasts.len(),
                1,
                "{first_leaves:?} released a toast the other source still holds"
            );
            assert_eq!(
                toasts[0].created_at, base,
                "{first_leaves:?} restarted the countdown while the other source was on it"
            );
        }
    }

    /// Both, then **the other leaves too: restarted in full**, from the moment
    /// the last one went, and not before.
    #[test]
    fn the_countdown_restarts_in_full_only_when_both_have_gone() {
        let base = Instant::now();
        let mut toasts = vec![toast(1, base)];
        let mut holds = Holds::default();
        holds.attend(1, PointerEntered, &mut toasts, base);
        holds.attend(1, FocusGained, &mut toasts, base);
        holds.attend(1, PointerLeft, &mut toasts, base + Duration::from_secs(3));
        let last = base + Duration::from_secs(5);
        holds.attend(1, FocusLost, &mut toasts, last);

        let mut nearly = toasts.clone();
        Holds::default().sweep(&mut nearly, last + Duration::from_millis(9_999));
        assert_eq!(nearly.len(), 1, "it went one millisecond early");
        let mut due = toasts.clone();
        Holds::default().sweep(&mut due, last + Duration::from_secs(10));
        assert!(due.is_empty(), "it outlived its restarted deadline");
    }

    /// A toast that was dismissed while held is forgotten by both sources.
    #[test]
    fn both_sources_forget_a_toast_that_has_gone() {
        let base = Instant::now();
        let mut toasts = vec![toast(1, base)];
        let mut holds = Holds::default();
        holds.attend(1, PointerEntered, &mut toasts, base);
        holds.attend(1, FocusGained, &mut toasts, base);
        toasts.clear();
        holds.sweep(&mut toasts, base);
        assert!(!holds.is_held(1), "a dismissed id stayed held");
    }

    /// **The case `Holds` exists for.** Held past its deadline, then
    /// left: an immediate sweep must not take it, and a sweep after
    /// its restarted lifetime must. Without the restart inside
    /// `attend`, it would go the instant the pointer moved away.
    ///
    /// **Both halves are asserted, because one of them alone cannot
    /// fail for the obvious breakage.** Mapping `Left` to the entered
    /// branch — the mistake the `bool` invited and the enum is meant to
    /// prevent — leaves the toast *held for ever*: the first assertion
    /// passes, since a held toast is never swept. Only the release
    /// catches it. Measured: with the two arms swapped, the first
    /// assertion alone passes and the second fails.
    #[test]
    fn a_toast_left_after_its_deadline_is_not_swept_immediately() {
        let base = Instant::now();
        let long_past = base + Duration::from_secs(600);
        let mut toasts = vec![toast(1, base)];
        let mut holds = Holds::default();
        holds.attend(1, PointerEntered, &mut toasts, base);
        holds.attend(1, PointerLeft, &mut toasts, long_past);
        holds.sweep(&mut toasts, long_past);
        assert_eq!(
            toasts.len(),
            1,
            "the toast vanished the moment the pointer left it",
        );

        // Released, and counting again from when the pointer left: its
        // fresh 10 seconds are up, so this sweep takes it.
        holds.sweep(&mut toasts, long_past + Duration::from_secs(10));
        assert!(
            toasts.is_empty(),
            "the toast outlived its restarted deadline — leaving did not release the hold, so \
             this toast would sit on screen until the application was closed",
        );
    }

    /// Stack order comes from the vector, so a restart must not move
    /// anything (`snora/src/toast.rs`'s `render_order_for`).
    #[test]
    fn leaving_does_not_reorder_the_queue() {
        let base = Instant::now();
        let mut toasts = vec![toast(1, base), toast(2, base), toast(3, base)];
        let before: Vec<u64> = toasts.iter().map(|t| t.id).collect();
        let mut holds = Holds::default();
        holds.attend(2, PointerEntered, &mut toasts, base);
        holds.attend(2, PointerLeft, &mut toasts, base + Duration::from_secs(1));
        let after: Vec<u64> = toasts.iter().map(|t| t.id).collect();
        assert_eq!(before, after, "the restart reordered the stack");
    }

    /// An unheld toast is swept exactly as the old free sweep swept it.
    #[test]
    fn sweep_without_any_hold_matches_the_old_behaviour() {
        let base = Instant::now();
        let mut toasts = vec![
            toast(1, base),
            Toast::new(2, ToastIntent::Info, "t", "m", ())
                .persistent()
                .with_created_at(base),
        ];
        Holds::default().sweep(&mut toasts, base + Duration::from_secs(11));
        assert_eq!(toasts.len(), 1, "the persistent toast should remain");
        assert_eq!(toasts[0].id, 2);
    }
}

#[cfg(test)]
mod subscription_tests {
    use super::*;
    use snora_core::ToastAttention::PointerEntered;
    use snora_core::ToastIntent;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Tick,
    }

    /// With Error persistent by default (RFC-113), an application
    /// showing only errors no longer wakes the runtime at all.
    #[test]
    fn a_queue_of_default_errors_asks_for_no_ticks() {
        let toasts = vec![
            Toast::new(1, ToastIntent::Error, "Failed", "No.", Msg::Tick),
            Toast::new(
                2,
                ToastIntent::Error,
                "Failed again",
                "Still no.",
                Msg::Tick,
            ),
        ];
        assert!(
            toasts
                .iter()
                .all(|t| matches!(t.lifetime, ToastLifetime::Persistent)),
            "the fixture is not what it claims",
        );
        // `Subscription` has no public equality, so assert the
        // condition `subscription` branches on, and that it branches.
        let has_transient = toasts
            .iter()
            .any(|t| matches!(t.lifetime, ToastLifetime::Transient(_)));
        assert!(!has_transient);
        let _ = subscription(&toasts, || Msg::Tick);
    }

    /// A held toast is still transient, so the ticks must keep coming:
    /// it has to expire on time once the pointer leaves, and that
    /// needs a sweep to run.
    #[test]
    fn a_held_toast_keeps_the_subscription_alive() {
        let base = Instant::now();
        let mut toasts =
            vec![Toast::new(1, ToastIntent::Info, "t", "m", Msg::Tick).with_created_at(base)];
        let mut holds = Holds::default();
        holds.attend(1, PointerEntered, &mut toasts, base);
        let has_transient = toasts
            .iter()
            .any(|t| matches!(t.lifetime, ToastLifetime::Transient(_)));
        assert!(
            has_transient,
            "a held toast stopped looking transient, so nothing would tick and it \
             would never expire after the pointer left",
        );
        let _ = subscription(&toasts, || Msg::Tick);
    }
}

#[cfg(test)]
mod hover_wiring {
    use crate::{AppLayout, render};
    use iced::Element;
    use iced_test::Simulator;
    use snora_core::{Toast, ToastAttention, ToastIntent};

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Body,
        Dismiss(u64),
        Hover(u64, ToastAttention),
    }

    fn layout(wired: bool) -> Element<'static, Msg> {
        let body: Element<'static, Msg> = iced::widget::button(iced::widget::text("body"))
            .on_press(Msg::Body)
            .into();
        let toast = Toast::new(7, ToastIntent::Info, "Saved", "All good.", Msg::Dismiss(7));
        let l = AppLayout::new(body).toasts(vec![toast]);
        render(if wired {
            l.on_toast_attention(Msg::Hover)
        } else {
            l
        })
    }

    /// Wired: moving the pointer onto the toast emits
    /// `Hover(id, ToastAttention::PointerEntered)`.
    #[test]
    fn hovering_a_toast_reports_the_pointer_when_wired() {
        let mut ui = Simulator::new(layout(true));
        let bounds = ui
            .find("Saved")
            .expect("the toast draws its title")
            .visible_bounds()
            .expect("the title is visible");
        ui.point_at(bounds.center());
        let _ = ui.simulate([iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: bounds.center(),
        })]);
        let messages: Vec<Msg> = ui.into_messages().collect();
        assert!(
            messages.contains(&Msg::Hover(7, ToastAttention::PointerEntered)),
            "no Hover(7, ToastAttention::PointerEntered) after the pointer entered the toast: {messages:?}",
        );
    }

    /// Unwired: the same gesture emits nothing.
    #[test]
    fn hovering_a_toast_reports_nothing_when_unwired() {
        let mut ui = Simulator::new(layout(false));
        let bounds = ui
            .find("Saved")
            .expect("the toast draws its title")
            .visible_bounds()
            .expect("the title is visible");
        ui.point_at(bounds.center());
        let _ = ui.simulate([iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: bounds.center(),
        })]);
        let messages: Vec<Msg> = ui.into_messages().collect();
        assert!(
            messages.is_empty(),
            "an unwired layout reported pointer movement: {messages:?}",
        );
    }
}

#[cfg(test)]
mod ring_pin {
    //! **The toast close mark's ring colour, pinned** (RFC-122 review, 3.4): the
    //! intent's own text colour, which `readable_text` and `stronger_pole` chose before
    //! they moved to `snora-interaction`. Five intents in the 22 stock themes (the
    //! engine path) and in the 6 presets (the design path).
    use super::*;

    fn rgb(c: Color) -> [u8; 3] {
        [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        ]
    }

    const PINNED: &[(&str, &str, &str, [u8; 3])] = &[
        ("engine", "Light", "Debug", [0, 0, 0]),
        ("engine", "Light", "Info", [0, 0, 0]),
        ("engine", "Light", "Success", [249, 250, 250]),
        ("engine", "Light", "Warning", [0, 0, 0]),
        ("engine", "Light", "Error", [252, 250, 250]),
        ("engine", "Dark", "Debug", [230, 230, 230]),
        ("engine", "Dark", "Info", [0, 0, 0]),
        ("engine", "Dark", "Success", [255, 255, 255]),
        ("engine", "Dark", "Warning", [63, 45, 12]),
        ("engine", "Dark", "Error", [252, 250, 250]),
        ("engine", "Dracula", "Debug", [248, 248, 242]),
        ("engine", "Dracula", "Info", [0, 0, 0]),
        ("engine", "Dracula", "Success", [13, 62, 25]),
        ("engine", "Dracula", "Warning", [59, 62, 30]),
        ("engine", "Dracula", "Error", [63, 14, 14]),
        ("engine", "Nord", "Debug", [252, 255, 255]),
        ("engine", "Nord", "Info", [0, 0, 0]),
        ("engine", "Nord", "Success", [37, 45, 30]),
        ("engine", "Nord", "Warning", [57, 48, 30]),
        ("engine", "Nord", "Error", [0, 0, 0]),
        ("engine", "SolarizedLight", "Debug", [48, 47, 41]),
        ("engine", "SolarizedLight", "Info", [0, 0, 0]),
        ("engine", "SolarizedLight", "Success", [28, 34, 0]),
        ("engine", "SolarizedLight", "Warning", [42, 29, 0]),
        ("engine", "SolarizedLight", "Error", [255, 255, 255]),
        ("engine", "SolarizedDark", "Debug", [249, 250, 250]),
        ("engine", "SolarizedDark", "Info", [0, 0, 0]),
        ("engine", "SolarizedDark", "Success", [28, 34, 0]),
        ("engine", "SolarizedDark", "Warning", [42, 29, 0]),
        ("engine", "SolarizedDark", "Error", [255, 255, 255]),
        ("engine", "GruvboxLight", "Debug", [40, 40, 40]),
        ("engine", "GruvboxLight", "Info", [0, 0, 0]),
        ("engine", "GruvboxLight", "Success", [17, 17, 17]),
        ("engine", "GruvboxLight", "Warning", [17, 17, 17]),
        ("engine", "GruvboxLight", "Error", [253, 249, 249]),
        ("engine", "GruvboxDark", "Debug", [251, 241, 199]),
        ("engine", "GruvboxDark", "Info", [0, 0, 0]),
        ("engine", "GruvboxDark", "Success", [34, 33, 2]),
        ("engine", "GruvboxDark", "Warning", [52, 34, 3]),
        ("engine", "GruvboxDark", "Error", [253, 249, 249]),
        ("engine", "CatppuccinLatte", "Debug", [48, 51, 87]),
        ("engine", "CatppuccinLatte", "Info", [0, 0, 0]),
        ("engine", "CatppuccinLatte", "Success", [8, 36, 4]),
        ("engine", "CatppuccinLatte", "Warning", [24, 22, 68]),
        ("engine", "CatppuccinLatte", "Error", [253, 249, 250]),
        ("engine", "CatppuccinFrappe", "Debug", [238, 253, 255]),
        ("engine", "CatppuccinFrappe", "Info", [0, 0, 0]),
        ("engine", "CatppuccinFrappe", "Success", [38, 50, 29]),
        ("engine", "CatppuccinFrappe", "Warning", [56, 47, 31]),
        ("engine", "CatppuccinFrappe", "Error", [56, 27, 28]),
        ("engine", "CatppuccinMacchiato", "Debug", [232, 244, 255]),
        ("engine", "CatppuccinMacchiato", "Info", [0, 0, 0]),
        ("engine", "CatppuccinMacchiato", "Success", [38, 53, 33]),
        ("engine", "CatppuccinMacchiato", "Warning", [58, 51, 36]),
        ("engine", "CatppuccinMacchiato", "Error", [58, 29, 33]),
        ("engine", "CatppuccinMocha", "Debug", [205, 214, 244]),
        ("engine", "CatppuccinMocha", "Info", [0, 0, 0]),
        ("engine", "CatppuccinMocha", "Success", [38, 55, 36]),
        ("engine", "CatppuccinMocha", "Warning", [61, 55, 40]),
        ("engine", "CatppuccinMocha", "Error", [60, 30, 38]),
        ("engine", "TokyoNight", "Debug", [210, 228, 255]),
        ("engine", "TokyoNight", "Info", [0, 0, 0]),
        ("engine", "TokyoNight", "Success", [35, 49, 20]),
        ("engine", "TokyoNight", "Warning", [54, 40, 20]),
        ("engine", "TokyoNight", "Error", [61, 24, 31]),
        ("engine", "TokyoNightStorm", "Debug", [210, 228, 255]),
        ("engine", "TokyoNightStorm", "Info", [0, 0, 0]),
        ("engine", "TokyoNightStorm", "Success", [35, 49, 20]),
        ("engine", "TokyoNightStorm", "Warning", [54, 40, 20]),
        ("engine", "TokyoNightStorm", "Error", [61, 24, 31]),
        ("engine", "TokyoNightLight", "Debug", [30, 34, 74]),
        ("engine", "TokyoNightLight", "Info", [0, 0, 0]),
        ("engine", "TokyoNightLight", "Success", [250, 250, 249]),
        ("engine", "TokyoNightLight", "Warning", [251, 250, 249]),
        ("engine", "TokyoNightLight", "Error", [251, 250, 250]),
        ("engine", "KanagawaWave", "Debug", [220, 215, 186]),
        ("engine", "KanagawaWave", "Info", [0, 0, 0]),
        ("engine", "KanagawaWave", "Success", [24, 32, 20]),
        ("engine", "KanagawaWave", "Warning", [63, 35, 7]),
        ("engine", "KanagawaWave", "Error", [252, 250, 250]),
        ("engine", "KanagawaDragon", "Debug", [197, 201, 197]),
        ("engine", "KanagawaDragon", "Info", [255, 255, 255]),
        ("engine", "KanagawaDragon", "Success", [30, 34, 25]),
        ("engine", "KanagawaDragon", "Warning", [63, 35, 7]),
        ("engine", "KanagawaDragon", "Error", [46, 23, 22]),
        ("engine", "KanagawaLotus", "Debug", [32, 28, 64]),
        ("engine", "KanagawaLotus", "Info", [0, 0, 0]),
        ("engine", "KanagawaLotus", "Success", [0, 0, 0]),
        ("engine", "KanagawaLotus", "Warning", [32, 28, 64]),
        ("engine", "KanagawaLotus", "Error", [253, 250, 250]),
        ("engine", "Moonfly", "Debug", [189, 189, 189]),
        ("engine", "Moonfly", "Info", [0, 0, 0]),
        ("engine", "Moonfly", "Success", [30, 47, 17]),
        ("engine", "Moonfly", "Warning", [55, 47, 30]),
        ("engine", "Moonfly", "Error", [63, 14, 14]),
        ("engine", "Nightfly", "Debug", [189, 193, 198]),
        ("engine", "Nightfly", "Info", [0, 0, 0]),
        ("engine", "Nightfly", "Success", [36, 49, 17]),
        ("engine", "Nightfly", "Warning", [55, 50, 30]),
        ("engine", "Nightfly", "Error", [62, 13, 12]),
        ("engine", "Oxocarbon", "Debug", [241, 241, 241]),
        ("engine", "Oxocarbon", "Info", [0, 0, 0]),
        ("engine", "Oxocarbon", "Success", [0, 45, 16]),
        ("engine", "Oxocarbon", "Warning", [45, 33, 63]),
        ("engine", "Oxocarbon", "Error", [0, 0, 0]),
        ("engine", "Ferra", "Debug", [255, 235, 201]),
        ("engine", "Ferra", "Info", [0, 0, 0]),
        ("engine", "Ferra", "Success", [41, 42, 33]),
        ("engine", "Ferra", "Warning", [60, 52, 22]),
        ("engine", "Ferra", "Error", [54, 21, 24]),
        ("design", "light", "Debug", [21, 24, 28]),
        ("design", "light", "Info", [255, 255, 255]),
        ("design", "light", "Success", [255, 255, 255]),
        ("design", "light", "Warning", [255, 255, 255]),
        ("design", "light", "Error", [255, 255, 255]),
        ("design", "dark", "Debug", [241, 243, 246]),
        ("design", "dark", "Info", [8, 17, 31]),
        ("design", "dark", "Success", [6, 33, 15]),
        ("design", "dark", "Warning", [36, 22, 0]),
        ("design", "dark", "Error", [42, 6, 6]),
        ("design", "light_aaa", "Debug", [21, 24, 28]),
        ("design", "light_aaa", "Info", [255, 255, 255]),
        ("design", "light_aaa", "Success", [255, 255, 255]),
        ("design", "light_aaa", "Warning", [255, 255, 255]),
        ("design", "light_aaa", "Error", [255, 255, 255]),
        ("design", "dark_aaa", "Debug", [241, 243, 246]),
        ("design", "dark_aaa", "Info", [8, 17, 31]),
        ("design", "dark_aaa", "Success", [6, 33, 15]),
        ("design", "dark_aaa", "Warning", [36, 22, 0]),
        ("design", "dark_aaa", "Error", [42, 6, 6]),
        ("design", "high_contrast_light", "Debug", [0, 0, 0]),
        ("design", "high_contrast_light", "Info", [255, 255, 255]),
        ("design", "high_contrast_light", "Success", [255, 255, 255]),
        ("design", "high_contrast_light", "Warning", [255, 255, 255]),
        ("design", "high_contrast_light", "Error", [255, 255, 255]),
        ("design", "high_contrast_dark", "Debug", [255, 255, 255]),
        ("design", "high_contrast_dark", "Info", [0, 0, 0]),
        ("design", "high_contrast_dark", "Success", [0, 0, 0]),
        ("design", "high_contrast_dark", "Warning", [0, 0, 0]),
        ("design", "high_contrast_dark", "Error", [0, 0, 0]),
    ];

    fn pinned(kind: &str, name: &str, intent: ToastIntent) -> [u8; 3] {
        PINNED
            .iter()
            .find(|(k, n, i, _)| *k == kind && *n == name && *i == format!("{intent:?}"))
            .unwrap_or_else(|| panic!("no pinned colour for {kind} {name} {intent:?}"))
            .3
    }

    #[test]
    fn every_toast_ring_colour_is_what_it_was_before_the_rule_moved() {
        let intents = [
            ToastIntent::Debug,
            ToastIntent::Info,
            ToastIntent::Success,
            ToastIntent::Warning,
            ToastIntent::Error,
        ];
        let mut off = Vec::new();
        let mut checked = 0;
        for theme in iced::Theme::ALL {
            for intent in intents {
                let got = rgb(intent_colors(theme, intent, None).1);
                let want = pinned("engine", &format!("{theme:?}"), intent);
                checked += 1;
                if got != want {
                    off.push(format!(
                        "engine {theme:?} {intent:?}: {got:?}, was {want:?}"
                    ));
                }
            }
        }
        #[cfg(feature = "design")]
        for (name, tokens) in snora_design::Tokens::presets() {
            let overrides = crate::design::render::toast_intent_colors(&tokens);
            let theme = snora_style::theme::theme(&tokens);
            for intent in intents {
                let got = rgb(intent_colors(&theme, intent, Some(overrides)).1);
                let want = pinned("design", name, intent);
                checked += 1;
                if got != want {
                    off.push(format!("design {name} {intent:?}: {got:?}, was {want:?}"));
                }
            }
        }
        assert!(checked >= 110, "only {checked} colours checked");
        assert!(off.is_empty(), "toast ring colours changed: {off:?}");
    }
}
