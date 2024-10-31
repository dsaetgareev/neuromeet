use wasm_bindgen::JsValue;
use web_sys::Window;

pub fn global_window() -> Window {
    web_sys::window().expect("there was no window global object!")
}


pub fn get_url_pathname() -> Result<String, JsValue> {
    global_window().location().pathname()
}