#![feature(future_join)]
use yew_ui::App;


fn main() {
    #[cfg(feature = "debugAssertions")]
    {
        _ = console_log::init_with_level(log::Level::Debug);
    }
    #[cfg(not(feature = "debugAssertions"))]
    {
        _ = console_log::init_with_level(log::Level::Info);
    }

    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
