//! The target-size rule's inputs and centring (RFC-115's amendment,
//! RFC-116 R-1 Unit 3), measured on `control`'s layout directly.
//!
//! `sizing_tests.rs` measures the text-relative term through the
//! simulator, by clicking. These need the content's position inside
//! the control as well, which only the layout tree has, so they lay
//! the control out against a headless renderer and read the nodes.
//! The renderer's default text size is the one setting that moves the
//! `1.5 x text` term, and it is passed to the renderer at build.

use super::*;
use iced::Font;
use iced::widget::space;
use iced_core::layout::Limits;

#[derive(Debug, Clone)]
enum Msg {
    Press,
}

/// A headless renderer whose default text size is `text`.
fn renderer(text: f32) -> iced::Renderer {
    use iced_core::renderer::Headless;
    iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        Font::DEFAULT,
        Pixels(text),
        Some("tiny-skia"),
    ))
    .expect("a tiny-skia renderer needs no GPU")
}

fn control_of(side: f32) -> Control<'static, Msg> {
    Control::new(
        space()
            .width(Length::Fixed(side))
            .height(Length::Fixed(side)),
        Box::new(|_theme, _status| button::Style::default()),
        Ring {
            color: Box::new(|_| Color::BLACK),
            width: 0.0,
            offset: 0.0,
        },
    )
    .on_press(Msg::Press)
}

/// The control's node, laid out at host text size `text` in a 400 x 400
/// window.
fn laid_out(control: Control<'static, Msg>, text: f32) -> layout::Node {
    let renderer = renderer(text);
    let element: Element<'static, Msg> = control.into();
    let mut tree = Tree::new(&element);
    let limits = Limits::new(Size::ZERO, Size::new(400.0, 400.0));
    // The element, not the control: `layout` is the trait method.
    let mut element = element;
    element
        .as_widget_mut()
        .layout(&mut tree, &renderer, &limits)
}

/// Where the content sits inside the control, as its offset from the
/// control's top-left corner.
fn content_offset(node: &layout::Node) -> (f32, f32) {
    let child = &node.children()[0];
    (child.bounds().x, child.bounds().y)
}

#[test]
fn the_token_sets_the_target_when_it_is_the_largest_input() {
    // 30 px of content, 16 px text (floor 24), a 44 px token.
    let node = laid_out(control_of(30.0).min_target(44.0), 16.0);
    assert_eq!(
        node.size(),
        Size::new(44.0, 44.0),
        "a 44 px minimum-target token did not make a 30 px control 44 px",
    );
}

#[test]
fn a_token_under_24_changes_nothing() {
    // WCAG's own 24 is not something a token can take a target under.
    let node = laid_out(control_of(1.0).min_target(10.0), 16.0);
    assert_eq!(node.size(), Size::new(24.0, 24.0));
}

#[test]
fn the_text_relative_term_still_applies_beside_a_token() {
    // 24 px host text makes the text term 36; a 30 px token is under it.
    let node = laid_out(control_of(1.0).min_target(30.0), 24.0);
    assert_eq!(node.size(), Size::new(36.0, 36.0));
}

#[test]
fn a_control_carrying_its_own_text_size_is_measured_against_that() {
    // Host text is 16 (floor 24), but this control's label is 30 px:
    // 1.5 x 30 = 45. Without `text_size` the control would be 24.
    let node = laid_out(control_of(1.0).text_size(30.0), 16.0);
    assert_eq!(node.size(), Size::new(45.0, 45.0));
}

#[test]
fn content_is_centred_in_the_growth_the_floor_causes() {
    // A 10 px mark in a 24 px floor: 7 px each side.
    let node = laid_out(control_of(10.0), 16.0);
    assert_eq!(node.size(), Size::new(24.0, 24.0));
    assert_eq!(content_offset(&node), (7.0, 7.0));
}

#[test]
fn padding_is_kept_and_the_remainder_split() {
    // 10 px content + 5 px padding each side = 20; the floor adds 4,
    // so 2 more each side: 5 + 2.
    let node = laid_out(control_of(10.0).padding(5.0), 16.0);
    assert_eq!(node.size(), Size::new(24.0, 24.0));
    assert_eq!(content_offset(&node), (7.0, 7.0));
}

/// An application that widened a control with `width` meant its content
/// to start at the edge (a menu item); only the floor's growth is split.
#[test]
fn width_the_application_set_is_not_split() {
    let node = laid_out(control_of(10.0).width(Length::Fixed(100.0)), 16.0);
    assert_eq!(node.size().width, 100.0);
    let (x, y) = content_offset(&node);
    assert_eq!(x, 0.0, "a widened control's content must stay at the start");
    assert_eq!(y, 7.0, "the height was the floor's, so it is centred");
}

#[test]
fn a_control_already_above_the_floor_is_laid_out_as_before() {
    let node = laid_out(control_of(40.0).min_target(30.0), 16.0);
    assert_eq!(node.size(), Size::new(40.0, 40.0));
    assert_eq!(content_offset(&node), (0.0, 0.0));
}

/// `button` reports its whole box as a container, and selectors and tests
/// find "the region that answers a click" through that. A control that did
/// not would be a drop-in only until something looked for its box.
#[test]
fn the_controls_whole_box_is_reported_as_a_container() {
    struct Containers(Vec<Rectangle>);
    impl iced_core::widget::Operation for Containers {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced_core::widget::Operation)) {
            operate(self);
        }
        fn container(&mut self, _id: Option<&iced_core::widget::Id>, bounds: Rectangle) {
            self.0.push(bounds);
        }
    }

    let renderer = renderer(16.0);
    let mut element: Element<'static, Msg> = control_of(40.0).padding(5.0).into();
    let mut tree = Tree::new(&element);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(400.0, 400.0)),
    );
    let mut seen = Containers(Vec::new());
    element.as_widget_mut().operate(
        &mut tree,
        iced_core::layout::Layout::new(&node),
        &renderer,
        &mut seen,
    );
    assert_eq!(
        seen.0.first().copied(),
        Some(Rectangle::new(iced::Point::ORIGIN, Size::new(50.0, 50.0))),
        "the control's own box (40 + 5 + 5 each way) is not what it reports as a container",
    );
}
