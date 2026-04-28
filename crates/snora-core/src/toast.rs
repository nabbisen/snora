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
//!   given [`Duration`]. The engine provides a subscription helper that
//!   wakes the runtime periodically and the `snora::toast::sweep_expired`
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
    /// A failure. Often paired with [`ToastLifetime::Persistent`] so the
    /// user must acknowledge before the toast disappears.
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
    #[must_use]
    pub fn is_top(self) -> bool {
        matches!(
            self,
            ToastPosition::TopEnd | ToastPosition::TopStart | ToastPosition::TopCenter
        )
    }

    /// Whether this position anchors to the *bottom* edge of the window.
    #[must_use]
    pub fn is_bottom(self) -> bool {
        !self.is_top()
    }
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
    /// The default "normal channel" duration (4 seconds). Long enough to
    /// read a short message, short enough not to stack up if the user is
    /// busy with something else.
    pub const DEFAULT: ToastLifetime = ToastLifetime::Transient(Duration::from_secs(4));

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
    /// Auto-dismiss policy. Defaults to [`ToastLifetime::DEFAULT`]
    /// (4-second transient).
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
    /// This constructor takes the mandatory fields positionally and uses
    /// [`ToastLifetime::DEFAULT`] for the lifetime. Use builder-style
    /// methods below to customize further.
    pub fn new(
        id: u64,
        intent: ToastIntent,
        title: impl Into<String>,
        message: impl Into<String>,
        on_dismiss: Message,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            message: message.into(),
            intent,
            lifetime: ToastLifetime::DEFAULT,
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
