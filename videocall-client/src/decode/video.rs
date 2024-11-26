use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::JsValue;
use web_sys::{CodecState, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType, HtmlVideoElement, MediaStream, MessageEvent, VideoDecoder, VideoDecoderConfig, VideoFrame, Worker};
use js_sys::{wasm_bindgen::{prelude::Closure, JsCast}, Array, Uint8Array};
use protobuf::Message;
use crate::{constants::{VIDEO_HEIGHT, VIDEO_WIDTH}, decode::video_decoder::{create_video_decoder, video_handle}, workers::{worker_new, VideoPacket}, wrappers::EncodedVideoChunkTypeWrapper, VideoWorker};

use super::{create_video_stream, peer_decoder::DecodeStatus};

const MAX_BUFFER_SIZE: usize = 100;

#[derive(PartialEq, Debug)]
pub struct Video {
    pub media_stream: MediaStream,
    pub worker: Worker,
}

impl Video {
    pub fn new() -> Self {
        let (media_stream, media_stream_generator) = create_video_stream();

        let worker = worker_new("worker");
        let worker_clone = worker.clone();
        let writable = media_stream_generator.writable();
        let ws  = writable.clone();
        let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
            let worker_clone = worker_clone.clone();
            let data = Array::from(&msg.data());
            if data.length() == 0 {
                web_sys::console::log_1(&"len 0".into());

               // Обернуть `WritableStream` в `JsValue`
               let writable_stream_js = JsValue::from(ws.clone());

               // Создать массив для передачи transferable-объектов
               let mut transferables = js_sys::Array::new();
               transferables.push(&writable_stream_js); 

                let _ = worker_clone.post_message_with_transfer(&writable_stream_js, &transferables).expect("sending message error");
            }
    
        }) as Box<dyn Fn(MessageEvent)>);
    
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        Self {
            media_stream,
            worker
        }
    }

    pub fn decode(&mut self, media_packet: Arc<MediaPacket>) -> Result<DecodeStatus, anyhow::Error> {
        // web_sys::console::log_1(&"jfkdjfkdfj".into());
        let data = media_packet.write_to_bytes().unwrap();
        let uint8_array = Uint8Array::from(data.as_slice());

        // Создаём transferable объект
        let transferable = Array::of1(&uint8_array);
        // log::info!("jfdjfdkf");
        // web_sys::console::log_1(&"jfkdjfkdfj".into());
        // let _ = self.worker.post_message_with_transfer(&"decode".into(), &js_sys::Array::of1(&transferable)).expect("sending message error");
        self.worker.post_message(&uint8_array);
        // let mut json_message = js_sys::Object::new();
        // js_sys::Reflect::set(&json_message, &JsValue::from_str("type"), &JsValue::from_str("decode")).unwrap();
        // js_sys::Reflect::set(&json_message, &JsValue::from_str("data"), &&uint8_array.into()).unwrap();
        // self.worker.post_message(&json_message.into()).unwrap();
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }

}