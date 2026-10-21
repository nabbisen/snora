//! RFC-112's opt-out probe. See `Cargo.toml` for what it is for.
//!
//! The body is deliberately tiny: what is under test is what Cargo
//! resolves for this manifest, not anything this code does. It still
//! has to *use* snora, or the dependency could be dropped as unused
//! and the resolution would stop meaning anything.

fn main() {
    let body: iced::Element<'_, ()> = iced::widget::text("opt-out probe").into();
    // `render` is the engine's entry point and needs no renderer to
    // compile; the probe asserts the dependency graph, not a frame.
    let _element: iced::Element<'_, ()> = snora::render(snora::AppLayout::new(body));
    println!("snora-opt-out-probe: resolved");
}
