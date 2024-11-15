use std::{borrow::Borrow, cell::RefCell, rc::Rc, sync::Arc};

use gloo_worker::{HandlerId, Worker, WorkerScope};
use js_sys::Uint8Array;
use serde::{Serialize, Deserialize};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{VideoDecoder, VideoDecoderConfig, VideoDecoderInit};

use crate::{constants::VIDEO_CODEC, decode::{create_video_decoder, create_video_decoder_for_worker}};

use super::{video_worker_decoder::VideoWorkerDecoder, VideoPacket};


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerInput {
    pub(crate) data: VideoPacket,
}


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerOutput {
    pub data: Vec<u8>,
    pub id: String,
}

pub struct IdWrapper {
    pub id: HandlerId,
}

pub struct VideoWorker {
    decoder: Option<VideoWorkerDecoder>,        
    last_id: Option<Rc<RefCell<IdWrapper>>>,
    scope: WorkerScope<VideoWorker>,
    count: u64,
}

impl Worker for VideoWorker {
    type Message = ();

    type Input = VideoWorkerInput;

    type Output = VideoWorkerOutput;

    fn create(scope: &WorkerScope<Self>) -> Self {

        Self {
            decoder: None,
            last_id: None,
            scope: scope.clone(),
            count: 0,
        }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        todo!()
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {

        if let Some(decoder) = &self.decoder {
            self.count = self.count + 1;
            let packet = msg.data;
            &self.decoder.as_mut().expect("cannot get decoder").decode(Arc::new(packet));
        } else {
            self.last_id = Some(Rc::new(RefCell::new(IdWrapper { id })));
            let id = self.last_id.clone();
            let (video_worker_decoder, video_config) = create_video_decoder_for_worker(scope.clone(), id.clone());
            self.decoder = Some(VideoWorkerDecoder::new(video_worker_decoder, video_config, &scope, id.clone()));
        }
        // self.scope.respond(id.clone(), VideoWorkerOutput {
        //     data: msg.data.data,
        //     id: id
        // });

    }

}





    