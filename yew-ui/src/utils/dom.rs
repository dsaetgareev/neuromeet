use wasm_bindgen::JsValue;
use web_sys::{Window, window};

pub fn global_window() -> Window {
    window().expect("there was no window global object!")
}


pub fn get_url_pathname() -> Result<String, JsValue> {
    global_window().location().pathname()
}

pub fn get_origin_url() -> String {
    let origin = window()
        .expect("window to be available")
        .location()
        .origin()
        .expect("origin to be available");
    origin
}