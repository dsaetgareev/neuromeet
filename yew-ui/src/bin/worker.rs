use std::{cell::RefCell, rc::Rc};

use js_sys::{wasm_bindgen::{prelude::Closure, JsCast, JsValue}, Array, Uint8Array};
use videocall_client::VideoWorker;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, WritableStream};

fn main() {
    web_sys::console::log_1(&"worker starting".into());

    let scope = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()));
    let scope_clone = scope.clone();
    
    let video_worker_one = Rc::new(RefCell::new(VideoWorker::new()));
    let video_worker = video_worker_one.clone();

    let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
        let data = msg.data();
        if data.is_instance_of::<WritableStream>() {
            web_sys::console::log_1(&"init".into());
            let ws = data.unchecked_into::<WritableStream>();
            let mut worker = video_worker.borrow_mut();
            worker.init(ws);
        } else {
            let data = data.unchecked_into::<Uint8Array>();

            let vec_data: Vec<u8> = data.to_vec();
            let mut worker = video_worker.as_ref().borrow_mut();
            worker.decode(vec_data); 
        }

    }) as Box<dyn Fn(MessageEvent)>);

    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // The worker must send a message to indicate that it's ready to receive messages.
    scope
        .post_message(&Array::new().into())
        .expect("posting ready message succeeds");
}