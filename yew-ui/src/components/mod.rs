pub mod attendants;
pub mod device_selector;
pub mod host;
pub mod icons;
pub mod middleware;
pub mod top_bar;

mod canvas_generator;
mod peer_list;
mod peer_list_item;
mod buttons;
pub use buttons::VideoButton;
pub use attendants::AttendantsFunc;
pub use device_selector::Devices;