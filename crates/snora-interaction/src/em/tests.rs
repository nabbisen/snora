use super::*;
use iced::widget::space;
use iced_core::widget::Tree;

#[derive(Debug, Clone)]
enum Msg {}

fn renderer(text: f32) -> iced::Renderer {
    use iced_core::renderer::Headless;
    iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        Font::DEFAULT,
        Pixels(text),
        Some("tiny-skia"),
    ))
    .expect("a tiny-skia renderer needs no GPU")
}

fn laid_out(element: Element<'static, Msg>, text: f32) -> layout::Node {
    let renderer = renderer(text);
    let mut tree = Tree::new(&element);
    let mut element = element;
    element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(400.0, 400.0)),
    )
}

/// A glyph's height is its text size times the line height, so it is
/// proportional to the size it was given, whatever the font.
fn glyph_height(em: f32, text: f32) -> f32 {
    laid_out(Glyph::new("M", em).into(), text).size().height
}

#[test]
fn a_glyph_is_one_em_of_the_hosts_text() {
    // At 16 px host text the height is the 16 px size times iced's
    // default 1.3 line height; at 24 it is 1.5 x that. A glyph that
    // ignored the host would not move.
    let at_16 = glyph_height(1.0, 16.0);
    let at_24 = glyph_height(1.0, 24.0);
    assert!((at_16 - 20.8).abs() < 0.5, "1 em at 16 px measured {at_16}");
    assert!(
        (at_24 / at_16 - 1.5).abs() < 0.02,
        "1 em went from {at_16} to {at_24} as the host went 16 -> 24; it should scale 1.5x"
    );
}

#[test]
fn a_close_mark_is_one_and_an_eighth_em() {
    let ratio = glyph_height(1.125, 16.0) / glyph_height(1.0, 16.0);
    assert!(
        (ratio - 1.125).abs() < 0.02,
        "1.125 em is {ratio}x 1 em, not 1.125x"
    );
}

#[test]
fn a_glyph_announces_its_text_to_operations() {
    // `text` does, and a selector that finds text must find this.
    struct Seen(Vec<String>);
    impl Operation for Seen {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _id: Option<&iced_core::widget::Id>, _b: Rectangle, text: &str) {
            self.0.push(text.to_owned());
        }
    }
    let renderer = renderer(16.0);
    let mut element: Element<'static, Msg> = Glyph::new("X", 1.0).into();
    let mut tree = Tree::new(&element);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(400.0, 400.0)),
    );
    let mut seen = Seen(Vec::new());
    element
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut seen);
    assert_eq!(seen.0, vec!["X".to_owned()]);
}

#[test]
fn an_em_box_is_a_square_of_the_hosts_text_size() {
    for text in [16.0_f32, 24.0] {
        let boxed: Element<'static, Msg> =
            EmBox::new(space().width(Length::Fill).height(Length::Fill), 1.0).into();
        let node = laid_out(boxed, text);
        assert_eq!(node.size(), Size::new(text, text), "at {text} px host text");
        // And the filling content took the whole square.
        assert_eq!(node.children()[0].size(), Size::new(text, text));
    }
}
