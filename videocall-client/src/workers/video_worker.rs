use std::{borrow::Borrow, cell::RefCell, rc::Rc, sync::Arc};

use gloo_worker::{HandlerId, Worker, WorkerScope};
use js_sys::Uint8Array;
use protobuf::Message;
use serde::{Serialize, Deserialize};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, VideoDecoder, VideoDecoderConfig, VideoDecoderInit};

use crate::{constants::VIDEO_CODEC, decode::{create_video_decoder, create_video_decoder_for_worker, PeerDecodeError}};

use super::{video_worker_decoder::VideoWorkerDecoder, VideoPacket};


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerInput {
    pub(crate) data: Vec<u8>,
}


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerOutput {
    pub data: Vec<u8>,
    pub timestamp: f64,
}

pub struct IdWrapper {
    pub id: HandlerId,
}

pub struct VideoWorker {
    decoder: Option<VideoWorkerDecoder>,        
    scope: WorkerScope<VideoWorker>,
}

impl Worker for VideoWorker {
    type Message = ();

    type Input = VideoWorkerInput;

    type Output = VideoWorkerOutput;

    fn create(scope: &WorkerScope<Self>) -> Self {

        Self {
            decoder: None,
            scope: scope.clone(),
        }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        todo!()
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {

        if let Some(decoder) = &self.decoder {
            let packet = msg.data;
            if let Ok(media_packet) = parse_media_packet(&packet) {
                &self.decoder.as_mut().expect("cannot get decoder").decode(media_packet);
            }
        } else {
            let (video_worker_decoder, video_config) = create_video_decoder_for_worker(scope.clone(), id.clone());
            self.decoder = Some(VideoWorkerDecoder::new(video_worker_decoder, video_config, &scope, id.clone()));
        }
    }

}

fn parse_media_packet(data: &[u8]) -> Result<Arc<MediaPacket>, PeerDecodeError> {
    Ok(Arc::new(
        MediaPacket::parse_from_bytes(data).map_err(|_| PeerDecodeError::PacketParseError)?,
    ))
}
    