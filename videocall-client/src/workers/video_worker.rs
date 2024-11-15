use std::sync::Arc;

use gloo_worker::{HandlerId, Worker, WorkerScope};
use js_sys::Uint8Array;
use serde::{Serialize, Deserialize};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{VideoDecoder, VideoDecoderConfig, VideoDecoderInit};

use crate::{constants::VIDEO_CODEC, decode::create_video_decoder};

use super::VideoPacket;


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerInput {
    pub(crate) data: VideoPacket,
}


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerOutput {
    pub data: Vec<u8>,
}

pub struct VideoWorker {
    decoder: Option<VideoDecoder>,        
    last_id: Option<HandlerId>,
}

impl Worker for VideoWorker {
    type Message = ();

    type Input = VideoWorkerInput;

    type Output = VideoWorkerOutput;

    fn create(scope: &WorkerScope<Self>) -> Self {


        Self {
            decoder: None,
            last_id: None,
        }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        todo!()
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {

        log::info!("jfkdfjdkfjdlksl");
        self.last_id = Some(id);
        
        if self.decoder.is_none() {

            let scope = scope.clone();
            let id = self.last_id;

            let error_video = Closure::wrap(Box::new(move |e: JsValue| {
                log::error!("{:?}", e);
            }) as Box<dyn FnMut(JsValue)>);

            let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
                let uint8_array = Uint8Array::new(&original_chunk);
                let mut chunk_data = vec![0; uint8_array.length() as usize];
                uint8_array.copy_to(&mut chunk_data);

                if let Some(id) = id {
                    scope.respond(
                        id,
                        VideoWorkerOutput {
                            data: chunk_data,
                        },
                    );
                } else {
                    log::warn!("No handler ID found for response");
                }
            }) as Box<dyn FnMut(JsValue)>);

            let local_video_decoder = VideoDecoder::new(
                &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
            ).unwrap();
            error_video.forget();
            output.forget();
            let video_config = VideoDecoderConfig::new(&VIDEO_CODEC); 
            local_video_decoder.configure(&video_config);

        }

        if let Some(decoder) = &self.decoder {
            let packet = Arc::new(msg.data);
            let chunk = VideoPacket::get_encoded_video_chunk_from_data(packet);

            // decoder.decode(&chunk);
        }

    }

}





    