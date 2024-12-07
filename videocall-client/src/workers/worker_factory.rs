use web_sys::{window, Blob, BlobPropertyBag, Url, Worker};
use js_sys::Array;

pub fn worker_new(origin: Option<&str>, name: &str) -> Worker {
    let origin = &get_origin_url(origin);

    let script = Array::new();
    script.push(
        &format!(r#"importScripts("{origin}/{name}.js");wasm_bindgen("{origin}/{name}_bg.wasm");"#)
            .into(),
    );

    let blob = Blob::new_with_str_sequence_and_options(
        &script,
        BlobPropertyBag::new().type_("text/javascript"),
    )
    .expect("blob creation succeeds");

    let url = Url::create_object_url_with_blob(&blob).expect("url creation succeeds");

    Worker::new(&url).expect("failed to spawn worker")
}

fn get_origin_url(origin: Option<&str>) -> String {
    let origin = match origin {
        Some(origin_url) => {
            origin_url.to_string()
        },
        None => {
            let origin = window()
                .expect("window to be available")
                .location()
                .origin()
                .expect("origin to be available");
            origin
        },
    };
    origin
}