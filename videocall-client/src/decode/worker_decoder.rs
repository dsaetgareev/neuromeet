use wasm_bindgen::JsValue;
use web_sys::{MessageEvent, Worker, WritableStream};
use js_sys::{wasm_bindgen::{prelude::Closure, JsCast}, Array, Uint8Array};
use crate::workers::worker_new;
use crate::decode::DecodeStatus;

use super::Decode;

#[derive(PartialEq, Debug)]
pub struct WorkerDecoder {
    worker: Worker
}

impl WorkerDecoder {

    pub fn new(orgign_url: &str, url: &str, writable_stream: WritableStream) -> Self {
        let worker = worker_new(Some(orgign_url), url);
        let worker_clone = worker.clone();
        let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
            let data = Array::from(&msg.data());
            if data.length() == 0 {
                web_sys::console::log_1(&"len 0".into());
                if writable_stream.locked() {
                    web_sys::console::log_1(&"im locked".into());
                    return;
                }
               // Обернуть `WritableStream` в `JsValue`
               let writable_stream_js = JsValue::from(writable_stream.clone());

               // Создать массив для передачи transferable-объектов
               let transferables = js_sys::Array::new();
               transferables.push(&writable_stream_js); 

                let _ = worker_clone.post_message_with_transfer(&writable_stream_js, &transferables).expect("sending message error");
            }
    
        }) as Box<dyn Fn(MessageEvent)>);
    
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        Self {
            worker
        }
    }

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        let uint8_array = Uint8Array::from(media_packet.as_slice());
        let _ = self.worker.post_message(&uint8_array);
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }
    
}

impl Decode for WorkerDecoder {
    fn decode(&mut self, packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decode(&packet)
    }
}

