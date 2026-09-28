//! Laying out toasts stays roughly linear in their number (RFC-105 R-4).
//!
//! # Why this test is `#[ignore]`d and run in release
//!
//! It is a timing ratio, and a timing ratio in a debug build under a
//! parallel test runner measures the machine more than the code. One CI
//! step runs it on its own, in release:
//!
//! ```text
//! cargo test --release -p snora --test bounded_work -- --ignored
//! ```
//!
//! # What it asserts, and what it deliberately does not
//!
//! It asserts a **bound**, not a number: laying out ten times as many
//! toasts must take less than thirty times as long. Linear work lands
//! near ten. The bound has that much room because the quantity being
//! measured is noisy by nature, and the failure it exists to catch is a
//! change in *complexity* — an accidental quadratic — which overshoots
//! by orders of magnitude rather than by a factor of two. Measured with
//! a deliberate quadratic in place, the ratio was far outside it (see
//! the RFC-105 review package).
//!
//! It is not a performance budget: it says nothing about whether the
//! absolute time is acceptable. The absolute numbers, and the machine
//! they came from, are reported in the RFC's review package for the
//! performance-envelope document.
//!
//! Each size is measured as the **minimum of several runs**, because the
//! minimum is the run least disturbed by whatever else the machine was
//! doing; an average would fold that noise in.

#![cfg(feature = "widgets")]

use std::time::{Duration, Instant};

use iced::{Element, Size};
use iced_test::Simulator;

use snora::{AppLayout, Toast, ToastIntent, render};

/// The smaller size. Large enough that one run is stable — at a hundred
/// the whole measurement is a few hundred microseconds and dominated by
/// setup — and small enough that ten times it stays quick.
const N: usize = 1_000;

/// Linear is about 10. See the module doc for why the bound is this
/// loose and what it is really for.
const MAX_RATIO: f64 = 30.0;

/// Runs per size; the minimum of these is taken.
const RUNS: usize = 5;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Dismiss(u64),
}

/// Build and lay out `count` toasts, and return how long that took.
///
/// The timed region is the part that scales: composing the layer stack
/// and laying it out. Building the toast vector is done first, outside
/// the clock.
fn layout_time(count: usize) -> Duration {
    let toasts: Vec<Toast<Msg>> = (0..count)
        .map(|i| {
            Toast::new(
                i as u64,
                ToastIntent::Info,
                "Saved",
                "All good.",
                Msg::Dismiss(i as u64),
            )
        })
        .collect();
    let body: Element<'_, Msg> = iced::widget::text("body").into();
    let layout = AppLayout::new(body).toasts(toasts);

    let start = Instant::now();
    let element = render(layout);
    let ui = Simulator::with_size(iced::Settings::default(), Size::new(1024.0, 768.0), element);
    let elapsed = start.elapsed();
    drop(ui);
    elapsed
}

fn best_of(count: usize) -> Duration {
    (0..RUNS)
        .map(|_| layout_time(count))
        .min()
        .expect("RUNS is non-zero")
}

#[test]
#[ignore = "timing ratio: run in release, on its own — see the module documentation"]
fn laying_out_toasts_is_not_quadratic() {
    // One untimed pass, so neither size pays for first-touch costs the
    // other does not.
    let _ = layout_time(N);

    let small = best_of(N);
    let large = best_of(N * 10);
    let ratio = large.as_secs_f64() / small.as_secs_f64();

    println!(
        "bounded_work: {N} toasts {:.1}ms, {} toasts {:.1}ms, ratio {ratio:.2} (linear is ~10)",
        small.as_secs_f64() * 1e3,
        N * 10,
        large.as_secs_f64() * 1e3,
    );

    assert!(
        ratio < MAX_RATIO,
        "laying out {} toasts took {ratio:.2}x as long as {N} ({:.1}ms against {:.1}ms). Linear \
         is about 10 and the bound is {MAX_RATIO}; a ratio past it means the work per toast grows \
         with the number of toasts",
        N * 10,
        large.as_secs_f64() * 1e3,
        small.as_secs_f64() * 1e3,
    );
}
