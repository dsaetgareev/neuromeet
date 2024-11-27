use std::{cell::RefCell, rc::Rc};

use js_sys::{wasm_bindgen::{prelude::Closure, JsCast, JsValue}, Array, Uint8Array};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, WritableStream};

use super::{DecoderType, WorkerHandler};

pub fn worker_start(decoder_type: DecoderType) {
    web_sys::console::log_1(&"worker starting".into());

    let scope = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()));
    let scope_clone = scope.clone();
    
    let worker_handler = Rc::new(RefCell::new(WorkerHandler::new(decoder_type)));
    let handler = worker_handler.clone();

    let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
        let handler = handler.clone();
        let data = msg.data();
        if data.is_instance_of::<WritableStream>() {
            web_sys::console::log_1(&scope_clone.name().into());
            web_sys::console::log_1(&"init".into());
            let ws = data.unchecked_into::<WritableStream>();
            if let Ok(mut worker) = handler.as_ref().try_borrow_mut() {
                worker.init(ws);
            } else {
                web_sys::console::log_1(&"cannot borrow_mut handler for init".into());
            }
        } else {
            let data = data.unchecked_into::<Uint8Array>();
            let vec_data: Vec<u8> = data.to_vec();
            if let Ok(mut worker) = handler.as_ref().try_borrow_mut() {
                worker.handle(vec_data); 
            } else {
                scope_clone.close();
                web_sys::console::log_1(&"cannot borrow_mut handler for delta".into());
            }
        }

    }) as Box<dyn Fn(MessageEvent)>);

    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // The worker must send a message to indicate that it's ready to receive messages.
    scope
        .post_message(&Array::new().into())
        .expect("posting ready message succeeds");
}

