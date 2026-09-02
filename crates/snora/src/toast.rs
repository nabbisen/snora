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
//! 3. `sweep_expired` — a public helper that drops expired transient
//!    toasts from the application's toast queue.
//!
//! Together, (2) and (3) move toast TTL bookkeeping from application code
//! into the framework — the app only stores a `Vec<Toast<Message>>` and
//! calls two one-liners.

use std::time::{Duration, Instant};

use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Shadow, Subscription,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, opaque, row, text},
};

use snora_core::{LayoutDirection, Toast, ToastIntent, ToastLifetime, ToastPosition};

/// Fixed toast width so stacked toast edges align cleanly regardless of
/// content length. The value is chosen to comfortably hold two lines of
/// 14pt text at default font sizes.
const TOAST_WIDTH: f32 = 340.0;

/// Default sweep interval. Half-second resolution is imperceptible to users
/// and keeps idle wakeups low.
const SWEEP_INTERVAL: Duration = Duration::from_millis(500);

/// Fallback color for [`ToastIntent::Warning`].
///
/// iced's extended palette has no `warning` semantic pair (unlike `primary`,
/// `success`, and `danger`). This stable amber/orange is chosen to remain
/// readable against both light and dark iced themes. It is a Snora
/// implementation detail — applications cannot configure it through the
/// theme API, and it may change when iced eventually adds a warning
/// semantic. See RFC-014-C.
const WARNING_COLOR: Color = Color::from_rgb(0.851, 0.467, 0.024);

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
) -> Option<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    if toasts.is_empty() {
        return None;
    }

    let mut stack_col = column![].spacing(8);
    match render_order_for(position) {
        // Top anchors: newest must be at the top of the column (first child),
        // so iterate in reverse — newest (back of queue) is pushed first.
        ToastRenderOrder::ReverseChronological => {
            for toast in toasts.into_iter().rev() {
                stack_col = stack_col.push(render_single_toast(toast));
            }
        }
        // Bottom anchors: newest must be at the bottom of the column (last child),
        // so iterate in chronological order — newest (back of queue) is pushed last.
        ToastRenderOrder::Chronological => {
            for toast in toasts {
                stack_col = stack_col.push(render_single_toast(toast));
            }
        }
    }

    let horizontal_anchor = horizontal_align(position, direction);
    let vertical_anchor = if position.is_top() {
        Vertical::Top
    } else {
        Vertical::Bottom
    };

    Some(
        container(stack_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .align_x(horizontal_anchor)
            .align_y(vertical_anchor)
            .id(crate::identifiers::TOAST_STACK)
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

fn render_single_toast<'a, Message>(toast: Toast<Message>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let intent = toast.intent;

    let text_col = column![text(toast.title).size(16), text(toast.message).size(14),].spacing(4);

    let close_btn = button(text("×").size(18))
        .on_press(toast.on_dismiss)
        .padding([0, 8])
        .style(move |theme, status| close_button_style(theme, intent, status));

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
            .style(move |theme| toast_style(theme, intent))
            .id(crate::identifiers::toast_id(toast.id)),
    )
}

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
/// - `Warning` — F-05. `WARNING_COLOR` vs `Color::WHITE` measured
///   **3.18:1**, matching the audit. Two repairs were measured, not
///   assumed: darkening the fill ~20-25% while keeping white text reaches
///   AA (4.73-5.26:1) but changes the amber's own identity; switching the
///   text to `Color::BLACK` against the **unchanged** fill measures
///   **6.60:1** — real margin, and preserves `WARNING_COLOR` exactly.
///   Chose the text swap: the identity property traded is "how Warning
///   reads at a glance" (white-on-color to black-on-color), not the
///   colour itself. `Success` and `Error` both keep white text, so
///   `Warning`'s black text reads as *more* distinct from them, not a
///   collision — Q-1 named that as a risk to check, not a certainty.
/// - `Debug` — unchanged; already the correct pairing (`13.28:1` /
///   `6.02:1`, light/dark). F-06 was in [`close_button_style`], not here.
/// - `Info` — **not named by the audit, found by measuring all five
///   (Q-3).** `primary.base.color`/`.text` measured **4.43:1** in both
///   stock themes — under AA, and iced's own derivation, not a snora
///   literal. Neither `primary.base` nor `primary.strong`'s own paired
///   `.text` clears with real margin (4.43-4.58:1, all within 2% of the
///   floor) because `primary.base.color`'s luminance sits almost exactly
///   at the point where black and white text contrast it equally.
///   `primary.strong.color` paired with `Color::BLACK` measures
///   **5.64:1** — real margin, still recognizably the same blue family.
/// - `Success` — unchanged; already passes with margin (`6.61:1` /
///   `6.91:1`).
/// - `Error` — unchanged; passes (`4.83:1` both themes) — thinner than
///   `Success`'s margin but does clear the floor, and Q-3 asked to
///   report passes, not to re-tune ones that already hold.
fn intent_colors(theme: &iced::Theme, intent: ToastIntent) -> (Color, Color) {
    let ep = theme.extended_palette();
    match intent {
        ToastIntent::Debug => (ep.background.strong.color, ep.background.strong.text),
        // Fill widened from `base` to `strong` and text overridden to
        // black — see this function's doc comment for the measured
        // figures behind both changes.
        ToastIntent::Info => (ep.primary.strong.color, Color::BLACK),
        ToastIntent::Success => (ep.success.base.color, ep.success.base.text),
        // iced's extended palette has no `warning` semantic pair; use the
        // private fallback constant. See RFC-014-C and WARNING_COLOR
        // above. Text corrected white -> black (RFC-086 F-05); fill
        // unchanged.
        ToastIntent::Warning => (WARNING_COLOR, Color::BLACK),
        ToastIntent::Error => (ep.danger.base.color, ep.danger.base.text),
    }
}

/// Style a toast surface based on its intent. Colors are pulled from the
/// theme's extended palette where available, with a hand-picked warning
/// color (iced's extended palette has no `warning` pair of its own).
fn toast_style(theme: &iced::Theme, intent: ToastIntent) -> iced::widget::container::Style {
    use iced::widget::container::Style;

    let (background, text_color) = intent_colors(theme, intent);

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
) -> button::Style {
    let (_, text_color) = intent_colors(theme, intent);
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
///                 snora::toast::sweep_expired(&mut self.toasts, std::time::Instant::now());
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

/// Drop expired transient toasts from the queue.
///
/// Persistent toasts are always retained. Call this from your update
/// function when handling the tick message produced by [`subscription`].
pub fn sweep_expired<Message: Clone>(toasts: &mut Vec<Toast<Message>>, now: Instant) {
    toasts.retain(|t| !t.is_expired(now));
}

#[cfg(test)]
mod contrast_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_drops_only_expired_transient() {
        let base = Instant::now();

        let live_transient = Toast::new(1, ToastIntent::Info, "a", "b", ())
            .with_lifetime(ToastLifetime::seconds(10))
            .with_created_at(base);
        let dead_transient = Toast::new(2, ToastIntent::Info, "a", "b", ())
            .with_lifetime(ToastLifetime::millis(100))
            .with_created_at(base);
        let persistent = Toast::new(3, ToastIntent::Error, "a", "b", ())
            .persistent()
            .with_created_at(base);

        let mut v = vec![live_transient, dead_transient, persistent];
        sweep_expired(&mut v, base + Duration::from_secs(1));

        let remaining_ids: Vec<u64> = v.iter().map(|t| t.id).collect();
        assert_eq!(remaining_ids, vec![1, 3]);
    }

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
}
