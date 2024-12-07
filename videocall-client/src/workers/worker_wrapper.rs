use std::{cell::RefCell, rc::Rc};

use js_sys::{wasm_bindgen::{prelude::Closure, JsCast, JsValue}, Array, Uint8Array};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, ReadableStream, WritableStream};

use crate::ReadableType;

use super::{agent_handler::{AgentHandler, AgentMsg}, ConnectionAgentMsg, DecoderType, WorkerHandler};

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


pub fn service_worker_start() {
    web_sys::console::log_1(&"service worker starting".into());

    let scope = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()));
    let scope_clone = scope.clone();
    let agent_handler = Rc::new(RefCell::new(AgentHandler::new(scope_clone.clone())));    
    let handler = agent_handler.clone();

    let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
        let data: JsValue = msg.data();
        let message = serde_wasm_bindgen::from_value::<ConnectionAgentMsg>(data.clone());
        if let Ok(message) = message {

            match message {
                ConnectionAgentMsg::Init { user_name, meeting_id, origin_url } => {
                    if let Ok(mut handler) = handler.as_ref().try_borrow_mut() {
                        web_sys::console::log_1(&"init from init".into());
                        handler.init(user_name, meeting_id, origin_url);
                    } else {
                        web_sys::console::log_1(&"cannot borrow_mut handler for init".into());
                    }
                
                },
                ConnectionAgentMsg::Connect => {
                    if let Ok(mut handler) = handler.as_ref().try_borrow_mut() {
                        web_sys::console::log_1(&"connect from connect".into());
                        handler.connect();
                    } else {
                        web_sys::console::log_1(&"cannot borrow_mut handler for connect".into());
                    }
                },
                ConnectionAgentMsg::StopEncoder { readable_type } => {
                    if let Ok(handler) = handler.as_ref().try_borrow_mut() {
                        web_sys::console::log_1(&"connect from connect".into());
                        handler.stop_encoder(readable_type);
                    } else {
                        web_sys::console::log_1(&"cannot borrow_mut handler for connect".into());
                    } 
                },
            }
        } else {

            if data.is_instance_of::<js_sys::Array>() {
                let arr = data.clone().unchecked_into::<js_sys::Array>();
                if arr.length() == 4 {
                    web_sys::console::log_1(&"writable stream init".into());
                    let peer_id = if let Some(peer_id_value) = js_sys::Reflect::get(&arr.get(0), &"peerId".into()).ok() {
                        peer_id_value.as_string().expect("cannot get peer_id")
                    } else {
                        "user".to_string()
                    };
                    let audio_ws = arr.get(1).unchecked_into::<WritableStream>();
                    let video_ws = arr.get(2).unchecked_into::<WritableStream>();
                    let screen_ws = arr.get(3).unchecked_into::<WritableStream>();

                    if let Ok(worker) = handler.as_ref().try_borrow_mut() {
                        worker.set_media(peer_id, audio_ws, video_ws, screen_ws);
                    } else {
                        web_sys::console::log_1(&"cannot borrow_mut handler for init".into());
                    }
                } else if arr.length() == 2 {
                    let message = serde_wasm_bindgen::from_value::<ReadableType>(arr.get(0));
                    if let Ok(readable_type) = message {
                        let readable = arr.get(1).unchecked_into::<ReadableStream>(); 
                        if let Ok(worker) = handler.as_ref().try_borrow_mut() {
                            worker.configure_encoder(readable_type, readable);
                        } else {
                            web_sys::console::log_1(&"cannot borrow_mut handler for get readable".into());
                        }
                    }
                }
            }
        }

    }) as Box<dyn Fn(MessageEvent)>);

    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let message = AgentMsg::Start;
    let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
    web_sys::console::log_1(&js_value);
    scope
        .post_message(&js_value)
        .expect("posting ready message succeeds");
}
