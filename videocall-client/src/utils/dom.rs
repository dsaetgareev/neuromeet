use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{ Window, HtmlVideoElement };
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

pub fn create_video_element(id: &str) -> HtmlVideoElement {
    // remove_element_by_id(id);
    let video_element = get_document()
        .create_element("video")
        .expect("cannot create video element")
        .dyn_into::<web_sys::HtmlVideoElement>()
        .expect("cannot cast video element");
    // video_element.set_id(id);
    video_element
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

