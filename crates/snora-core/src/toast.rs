//! Toast notifications.
//!
//! A toast is a small, auto-stackable notification that appears anchored to
//! one corner of the window. snora's toast contract carries **both** the
//! visible payload (title, body, intent) and the **lifetime policy**,
//! moving TTL management from user code into the framework.
//!
//! # Lifetime policy
//!
//! Each [`Toast`] declares a [`ToastLifetime`]:
//!
//! * [`ToastLifetime::Transient`] — the toast auto-dismisses after the
//!   given `Duration`. The engine provides a subscription helper that
//!   wakes the runtime periodically and the `snora::toast::Holds::sweep`
//!   helper removes entries whose deadlines have passed.
//! * [`ToastLifetime::Persistent`] — the toast remains until the user
//!   clicks the close button.
//!
//! # Design note — why does the toast own its creation time?
//!
//! Keeping `created_at` inside the struct, rather than outside in an
//! auxiliary `expires_at` field, means the Toast is a self-describing unit:
//! sweep logic is one pure function on a `Toast`, and test code can fabricate
//! a toast with a specific creation time without touching any other state.

use std::time::{Duration, Instant};

/// Semantic intent of a notification.
///
/// Engines map intents to colors using the current theme. `Debug` is kept
/// intentionally separate from `Info` so that diagnostic noise can be styled
/// distinctly (or suppressed) without changing intent at every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToastIntent {
    /// Diagnostic information for developers. Lower priority than `Info`,
    /// styled distinctly so that diagnostic noise can be visually separated
    /// (or filtered out) without changing intent at every call site.
    Debug,
    /// Neutral information.
    Info,
    /// A positive outcome — completed action, saved file, sent message.
    Success,
    /// Something the user should notice but is not an error.
    Warning,
    /// A failure. [`Toast::new`] makes this [`ToastLifetime::Persistent`]
    /// by default, so the user must acknowledge it rather than risk
    /// missing it (RFC-113).
    Error,
}

impl std::fmt::Display for ToastIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ToastIntent::Debug => "Debug",
            ToastIntent::Info => "Info",
            ToastIntent::Success => "Success",
            ToastIntent::Warning => "Warning",
            ToastIntent::Error => "Error",
        };
        f.write_str(s)
    }
}

/// Where the toast stack anchors within the window.
///
/// Positions are expressed in **logical** terms (`Start` / `End`) along the
/// horizontal axis, so they automatically mirror under
/// [`crate::LayoutDirection::Rtl`] without per-application changes — the
/// same ABDD principle that governs sidebars and header end-controls.
///
/// # Choosing a position
///
/// * [`TopEnd`] (default) — top-right under LTR, top-left under RTL.
///   Recommended for application-internal notifications because the bottom
///   half of the window is typically reserved for primary content
///   (previews, editors, lists).
/// * [`BottomEnd`] — bottom-right under LTR. Matches the OS-level
///   notification center convention on macOS / GNOME / Windows.
/// * [`TopStart`] / [`BottomStart`] — opposite horizontal edges from the
///   `End` variants.
/// * [`TopCenter`] / [`BottomCenter`] — centered horizontally. Useful for
///   modal-feeling messages.
///
/// # Stack growth direction
///
/// New toasts are inserted so that the **most recent toast is closest to
/// the anchor edge**:
///
/// * `Top*`: new toasts appear *below* older ones; the newest sits closest
///   to the top edge.
/// * `Bottom*`: new toasts appear *above* older ones; the newest sits
///   closest to the bottom edge.
///
/// The engine is responsible for honoring this invariant; applications
/// only push to the back of their `Vec<Toast<_>>` in chronological order.
///
/// [`TopEnd`]: ToastPosition::TopEnd
/// [`BottomEnd`]: ToastPosition::BottomEnd
/// [`TopStart`]: ToastPosition::TopStart
/// [`BottomStart`]: ToastPosition::BottomStart
/// [`TopCenter`]: ToastPosition::TopCenter
/// [`BottomCenter`]: ToastPosition::BottomCenter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToastPosition {
    /// LTR=top-right, RTL=top-left. The default.
    #[default]
    TopEnd,
    /// LTR=top-left, RTL=top-right.
    TopStart,
    /// Horizontally centered, anchored to the top edge.
    TopCenter,
    /// LTR=bottom-right, RTL=bottom-left.
    BottomEnd,
    /// LTR=bottom-left, RTL=bottom-right.
    BottomStart,
    /// Horizontally centered, anchored to the bottom edge.
    BottomCenter,
}

impl ToastPosition {
    /// Whether this position anchors to the *top* edge of the window.
    /// Engines use this to decide stack growth direction (top anchors grow
    /// downward; bottom anchors grow upward).
    ///
    /// # Example
    ///
    /// ```
    /// use snora_core::ToastPosition;
    ///
    /// assert!(ToastPosition::TopEnd.is_top());
    /// assert!(ToastPosition::TopCenter.is_top());
    /// assert!(!ToastPosition::BottomEnd.is_top());
    /// ```
    #[must_use]
    pub fn is_top(self) -> bool {
        matches!(
            self,
            ToastPosition::TopEnd | ToastPosition::TopStart | ToastPosition::TopCenter
        )
    }

    /// Whether this position anchors to the *bottom* edge of the window.
    ///
    /// Always the inverse of [`Self::is_top`].
    #[must_use]
    pub fn is_bottom(self) -> bool {
        !self.is_top()
    }
}

/// The floor under [`ToastLifetime::reading_time`], in seconds.
///
/// Below this a toast is a flicker rather than a message, however
/// little it says: the reader has to notice it, move their eyes to it
/// and read it, and the first two of those do not get shorter when the
/// text does (RFC-113).
const MINIMUM_READING_SECONDS: usize = 6;

/// Which way the pointer crossed a toast's edge (RFC-113).
///
/// What [`AppLayout::on_toast_hover`] hands an application, and what
/// `snora::toast::Holds::hover` takes back.
///
/// # Why this is not a `bool`
///
/// It was one, for one review round. Nothing in
/// `Message::ToastHover(7, true)` says what `true` meant, so a reader
/// had to leave the line and go and look — and an application that
/// passed the two the wrong way round would hold the toast it meant to
/// release and release the one it meant to hold, which no type would
/// have caught. `ToastHover::Left` says it where it is written.
///
/// There is deliberately no `Default`: a pointer either arrived or
/// left, and neither is the resting state of anything.
///
/// ```
/// use snora_core::ToastHover;
///
/// // The whole point: this reads without a lookup.
/// let crossing = ToastHover::Left;
/// assert_ne!(crossing, ToastHover::Entered);
/// ```
///
/// [`AppLayout::on_toast_hover`]: crate::AppLayout::on_toast_hover
#[deprecated(
    since = "0.56.0",
    note = "use `ToastAttention`, which also reports keyboard focus: \
            `AppLayout::on_toast_attention` and `snora::toast::Holds::attend` (RFC-120). \
            `ToastHover::Entered` is `ToastAttention::PointerEntered` and `Left` is `PointerLeft`."
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToastHover {
    /// The pointer came onto the toast. Its countdown is **held**: the
    /// toast will not expire while the pointer is on it.
    Entered,
    /// The pointer left the toast. Its countdown **restarts in full**,
    /// so the reader gets the whole reading time rather than whatever
    /// was left when they arrived.
    Left,
}

/// What a reader's attention did to a toast: the pointer crossed its edge, or
/// keyboard focus arrived on or left its controls (RFC-120 R-3).
///
/// What [`AppLayout::on_toast_attention`] hands an application, and what
/// `snora::toast::Holds::attend` takes back. **One rule:** a toast waits while
/// the reader is attending to it, by pointer or by keyboard.
///
/// - `PointerEntered` and `FocusGained` **hold** the toast's countdown.
/// - `PointerLeft` and `FocusLost` release their half of the hold. When
///   **neither** is left, the countdown **restarts in full**.
///
/// `#[non_exhaustive]`: a third source of attention may come (a screen reader's
/// cursor, say), and adding it must not break an application's `match`. That is
/// why this replaces [`ToastHover`], which could not take a focus variant
/// without breaking every application that matched it.
///
/// ```
/// use snora_core::ToastAttention;
///
/// let a = ToastAttention::FocusLost;
/// assert_ne!(a, ToastAttention::FocusGained);
/// ```
///
/// [`AppLayout::on_toast_attention`]: crate::AppLayout::on_toast_attention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ToastAttention {
    /// The pointer came onto the toast.
    PointerEntered,
    /// The pointer left the toast.
    PointerLeft,
    /// Keyboard focus arrived on a control in the toast.
    FocusGained,
    /// Keyboard focus left the toast's controls.
    FocusLost,
}

/// Auto-dismiss policy for a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLifetime {
    /// Toast vanishes once `created_at + duration < now`.
    Transient(Duration),
    /// Toast stays until the user clicks the close button.
    /// Use sparingly — reserved for errors that must be acknowledged.
    Persistent,
}

impl ToastLifetime {
    /// The former default: a 4-second transient.
    ///
    /// Four seconds is how long a toast lasted for every intent until
    /// 0.54.0, and it is not long enough. A reader who is slower than
    /// average, reading in a second language, or using a screen
    /// magnifier can lose the message before finishing it — and an
    /// error could vanish unread (RFC-113).
    #[deprecated(
        since = "0.54.0",
        note = "`Toast::new` no longer uses this: an Error is `Persistent`, and every \
                other intent gets `ToastLifetime::reading_time(title, message)`, which \
                scales with how much there is to read. This constant still means a \
                4-second transient if you set it deliberately."
    )]
    pub const DEFAULT: ToastLifetime = ToastLifetime::Transient(Duration::from_secs(4));

    /// How long a toast of this length should stay, by RFC-113's rule:
    ///
    /// > Two seconds to notice it, plus one second for every six
    /// > characters of title and message, rounded up, and never less
    /// > than six seconds.
    ///
    /// `n` counts `char`s, not bytes, so the figure is about how much
    /// there is to read rather than how it is encoded — which matters
    /// most for the scripts that say the most per character.
    ///
    /// The arithmetic is integral, so the rounding is exact rather than
    /// a float that happens to land right.
    ///
    /// # Examples
    ///
    /// RFC-113's four worked cases:
    ///
    /// ```
    /// use snora_core::ToastLifetime;
    /// use std::time::Duration;
    ///
    /// // 29 characters -> 2 + ceil(29 / 6) = 7s
    /// assert_eq!(
    ///     ToastLifetime::reading_time("Saved", "Your changes are stored."),
    ///     ToastLifetime::Transient(Duration::from_secs(7)),
    /// );
    ///
    /// // 18 characters -> 2 + ceil(18 / 6) = 5s, raised to the 6s floor
    /// assert_eq!(
    ///     ToastLifetime::reading_time("保存しました", "変更を保存しました。"),
    ///     ToastLifetime::Transient(Duration::from_secs(6)),
    /// );
    ///
    /// // 28 characters -> 2 + ceil(28 / 6) = 7s
    /// assert_eq!(
    ///     ToastLifetime::reading_time("接続できませんでした", "ネットワークを確認してください。"),
    ///     ToastLifetime::Transient(Duration::from_secs(7)),
    /// );
    ///
    /// // 120 characters -> 2 + ceil(120 / 6) = 22s
    /// assert_eq!(
    ///     ToastLifetime::reading_time(&"x".repeat(20), &"y".repeat(100)),
    ///     ToastLifetime::Transient(Duration::from_secs(22)),
    /// );
    /// ```
    #[must_use]
    pub fn reading_time(title: &str, message: &str) -> Self {
        let n = title.chars().count() + message.chars().count();
        // `div_ceil` rather than a float: the rule says "rounded up",
        // and this is that, not an approximation of it.
        let seconds = (2 + n.div_ceil(6)).max(MINIMUM_READING_SECONDS);
        ToastLifetime::Transient(Duration::from_secs(seconds as u64))
    }

    /// Convenience constructor for a transient lifetime in whole seconds.
    #[must_use]
    pub const fn seconds(secs: u64) -> Self {
        ToastLifetime::Transient(Duration::from_secs(secs))
    }

    /// Convenience constructor for a transient lifetime in milliseconds.
    #[must_use]
    pub const fn millis(ms: u64) -> Self {
        ToastLifetime::Transient(Duration::from_millis(ms))
    }
}

/// A toast notification.
///
/// **Where focus goes when a toast goes (RFC-120).** A toast's × is its only
/// focus target. If it had focus and the toast is closed, focus moves to the
/// **next toast's ×**, or else the **previous toast's ×**; with no toast left,
/// nothing is focused, and never a control on the page beneath. Toasts are known
/// by their `id`, so **a toast that expires or is closed elsewhere leaves focus on
/// the toast it was on.** (Which toast is "next" is Tab order: the stack's
/// layout order.)
///
/// If your container is left with no control in it, snora cannot tell that it is still
/// there, and treats it as gone: focus moves to the nearest control in the container
/// around it.
///
/// `Message` is your application's top-level message type. The `on_dismiss`
/// field is fired when the user clicks the toast's close button. It is *not*
/// fired when a transient toast expires; expiration is a silent sweep.
#[derive(Debug, Clone)]
pub struct Toast<Message: Clone> {
    /// Application-assigned id. snora does not interpret or generate ids;
    /// the application is the source of truth. Typically a monotonically
    /// increasing `u64`.
    pub id: u64,
    /// Bold heading line — typically a few words.
    pub title: String,
    /// Body text — one or two short sentences, explaining the situation.
    pub message: String,
    /// Semantic category (`Info`, `Success`, `Warning`, …). Resolved to a
    /// theme color by the engine.
    pub intent: ToastIntent,
    /// Auto-dismiss policy.
    ///
    /// [`Toast::new`] sets this by intent (RFC-113): an
    /// [`ToastIntent::Error`] is [`ToastLifetime::Persistent`], and
    /// every other intent gets
    /// [`ToastLifetime::reading_time`] for its own title and message.
    pub lifetime: ToastLifetime,
    /// When this toast was enqueued. Used with `lifetime` to compute
    /// expiration.
    pub created_at: Instant,
    /// Emitted when the user clicks the close button.
    pub on_dismiss: Message,
}

impl<Message: Clone> Toast<Message> {
    /// Build a new toast with `created_at` set to [`Instant::now()`].
    ///
    /// This constructor takes the mandatory fields positionally and
    /// chooses the lifetime by intent (RFC-113):
    ///
    /// - an [`ToastIntent::Error`] is [`ToastLifetime::Persistent`],
    ///   because an error that vanishes unread is an error the user
    ///   never had;
    /// - every other intent gets [`ToastLifetime::reading_time`] for
    ///   its own title and message, so a longer message stays longer.
    ///
    /// Use [`Self::with_lifetime`] or [`Self::persistent`] to override.
    ///
    /// # Example
    ///
    /// ```
    /// use snora_core::{Toast, ToastIntent};
    ///
    /// #[derive(Clone, Debug)]
    /// enum Msg { DismissToast(u64) }
    ///
    /// let toast = Toast::new(
    ///     /* id */       1,
    ///     /* intent */   ToastIntent::Success,
    ///     /* title */    "Saved",
    ///     /* message */  "Your changes are stored.",
    ///     /* on_dismiss */ Msg::DismissToast(1),
    /// );
    /// assert_eq!(toast.id, 1);
    /// assert_eq!(toast.title, "Saved");
    /// assert_eq!(toast.intent, ToastIntent::Success);
    /// ```
    pub fn new(
        id: u64,
        intent: ToastIntent,
        title: impl Into<String>,
        message: impl Into<String>,
        on_dismiss: Message,
    ) -> Self {
        let title = title.into();
        let message = message.into();
        // Computed after the conversions, so the count is of the text
        // that will actually be shown.
        let lifetime = match intent {
            // An error that vanishes unread is an error the user never
            // had (RFC-113).
            ToastIntent::Error => ToastLifetime::Persistent,
            _ => ToastLifetime::reading_time(&title, &message),
        };
        Self {
            id,
            title,
            message,
            intent,
            lifetime,
            created_at: Instant::now(),
            on_dismiss,
        }
    }

    /// Override the lifetime.
    #[must_use]
    pub fn with_lifetime(mut self, lifetime: ToastLifetime) -> Self {
        self.lifetime = lifetime;
        self
    }

    /// Make this toast persistent (never auto-dismiss).
    ///
    /// Use for notifications the user must explicitly acknowledge —
    /// "Export complete", "Disk almost full", confirmations of long
    /// operations.
    ///
    /// # Example
    ///
    /// ```
    /// use snora_core::{Toast, ToastIntent, ToastLifetime};
    ///
    /// #[derive(Clone, Debug)]
    /// enum Msg { DismissToast(u64) }
    ///
    /// let toast = Toast::new(
    ///     1, ToastIntent::Info, "Exported", "Done.", Msg::DismissToast(1),
    /// )
    /// .persistent();
    /// assert_eq!(toast.lifetime, ToastLifetime::Persistent);
    /// ```
    #[must_use]
    pub fn persistent(mut self) -> Self {
        self.lifetime = ToastLifetime::Persistent;
        self
    }

    /// Override the creation timestamp. Mainly useful for tests.
    #[must_use]
    pub fn with_created_at(mut self, created_at: Instant) -> Self {
        self.created_at = created_at;
        self
    }

    /// True when this toast has outlived its transient deadline.
    /// Persistent toasts always return `false`.
    #[must_use]
    pub fn is_expired(&self, now: Instant) -> bool {
        match self.lifetime {
            ToastLifetime::Persistent => false,
            ToastLifetime::Transient(d) => now.saturating_duration_since(self.created_at) >= d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC-113 R-1: an error must not vanish on its own.
    #[test]
    fn a_default_error_is_persistent() {
        let t = Toast::new(1, ToastIntent::Error, "Failed", "Could not save.", ());
        assert_eq!(t.lifetime, ToastLifetime::Persistent);
        assert!(
            !t.is_expired(t.created_at + Duration::from_secs(60)),
            "a default Error toast expired within a minute: lifetime is {:?}",
            t.lifetime,
        );
    }

    /// RFC-113 R-1: the countdown follows how much there is to read.
    ///
    /// 120 characters, so the rule gives 22s and the toast is still
    /// there at 21.
    #[test]
    fn a_default_toast_lasts_as_long_as_it_takes_to_read() {
        let title = "x".repeat(20);
        let message = "y".repeat(100);
        let t = Toast::new(1, ToastIntent::Info, title, message, ());
        assert!(
            !t.is_expired(t.created_at + Duration::from_secs(21)),
            "a 120-character toast expired within 21s: lifetime is {:?}",
            t.lifetime,
        );
        assert!(t.is_expired(t.created_at + Duration::from_secs(22)));
    }

    /// Every non-Error intent takes the reading time, not a constant.
    #[test]
    fn every_other_intent_takes_its_reading_time() {
        for intent in [
            ToastIntent::Debug,
            ToastIntent::Info,
            ToastIntent::Success,
            ToastIntent::Warning,
        ] {
            let t = Toast::new(1, intent, "Saved", "Your changes are stored.", ());
            assert_eq!(
                t.lifetime,
                ToastLifetime::reading_time("Saved", "Your changes are stored."),
                "{intent:?} did not take its reading time",
            );
        }
    }

    /// The floor, and that it is a floor rather than the whole rule.
    #[test]
    fn reading_time_has_a_floor_of_six_seconds() {
        assert_eq!(
            ToastLifetime::reading_time("", ""),
            ToastLifetime::Transient(Duration::from_secs(6)),
        );
        // 18 characters would give 5s without the floor.
        assert_eq!(
            ToastLifetime::reading_time("保存しました", "変更を保存しました。"),
            ToastLifetime::Transient(Duration::from_secs(6)),
        );
        // 25 characters clears it: 2 + ceil(25/6) = 7.
        assert_eq!(
            ToastLifetime::reading_time(&"x".repeat(25), ""),
            ToastLifetime::Transient(Duration::from_secs(7)),
        );
    }

    /// `n` counts characters, not bytes: the same number of characters
    /// gets the same time whatever the script costs to encode.
    #[test]
    fn reading_time_counts_characters_not_bytes() {
        let ascii = "x".repeat(30);
        let japanese = "あ".repeat(30);
        assert_eq!(
            japanese.len(),
            ascii.len() * 3,
            "the fixture is not what it claims"
        );
        assert_eq!(
            ToastLifetime::reading_time(&japanese, ""),
            ToastLifetime::reading_time(&ascii, ""),
        );
    }

    #[test]
    fn persistent_never_expires() {
        let t = Toast::new(1, ToastIntent::Info, "t", "m", ()).persistent();
        assert!(!t.is_expired(Instant::now() + Duration::from_secs(3600)));
    }

    #[test]
    fn transient_expires_past_deadline() {
        let base = Instant::now();
        let t = Toast::new(1, ToastIntent::Info, "t", "m", ())
            .with_lifetime(ToastLifetime::millis(100))
            .with_created_at(base);
        assert!(!t.is_expired(base));
        assert!(!t.is_expired(base + Duration::from_millis(50)));
        assert!(t.is_expired(base + Duration::from_millis(100)));
        assert!(t.is_expired(base + Duration::from_millis(200)));
    }

    #[test]
    fn default_toast_position_is_top_end() {
        assert_eq!(ToastPosition::default(), ToastPosition::TopEnd);
    }

    #[test]
    fn top_positions_classify_as_top() {
        assert!(ToastPosition::TopEnd.is_top());
        assert!(ToastPosition::TopStart.is_top());
        assert!(ToastPosition::TopCenter.is_top());
        assert!(!ToastPosition::BottomEnd.is_top());
        assert!(!ToastPosition::BottomStart.is_top());
        assert!(!ToastPosition::BottomCenter.is_top());
    }

    #[test]
    fn is_top_and_is_bottom_partition() {
        for pos in [
            ToastPosition::TopEnd,
            ToastPosition::TopStart,
            ToastPosition::TopCenter,
            ToastPosition::BottomEnd,
            ToastPosition::BottomStart,
            ToastPosition::BottomCenter,
        ] {
            assert_ne!(pos.is_top(), pos.is_bottom());
        }
    }
}
