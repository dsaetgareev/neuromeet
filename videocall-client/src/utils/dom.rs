use wasm_bindgen::JsValue;
use web_sys::Window;
use web_sys::Element;

pub fn global_window() -> Window {
    web_sys::window().expect("there was no window global object!")
}


pub fn get_url_pathname() -> Result<String, JsValue> {
    global_window().location().pathname()
}

pub fn get_document() -> web_sys::Document{
    global_window().document().expect("cannot get document")
}

pub fn get_element(id: &str) -> Option<Element> {
    get_document()
    .get_element_by_id(id)
}

pub fn remove_element_by_id(id: &str) {
    match get_element(id) {
        Some(element) => {
            element.remove();
        },
        None => {
            
        }
    }
}

