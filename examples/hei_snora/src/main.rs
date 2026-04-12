mod app;

use app::HeiSnora;

pub fn main() -> iced::Result {
    iced::application(HeiSnora::new, HeiSnora::update, HeiSnora::view)
        .title(HeiSnora::title)
        .run()
}
