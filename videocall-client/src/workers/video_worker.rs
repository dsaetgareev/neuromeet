use std::{borrow::Borrow, cell::RefCell, rc::Rc, sync::Arc};

use js_sys::Uint8Array;
use protobuf::Message;
use serde::{Serialize, Deserialize};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, VideoDecoder, VideoDecoderConfig, VideoDecoderInit, WritableStream};

use crate::{constants::VIDEO_CODEC, decode::{create_video_decoder, create_video_decoder_for_worker, PeerDecodeError}};

use super::{video_worker_decoder::{self, VideoWorkerDecoder}, VideoPacket};

#[derive(Clone)]
pub struct VideoWorker {
    video_worker_decoder: Option<VideoWorkerDecoder>
}

impl VideoWorker {
    pub fn new() -> VideoWorker {
        Self {
            video_worker_decoder: None
        }
    }

    pub fn init(&mut self, writable: WritableStream) {
        let (video_decoder, video_config) = create_video_decoder_for_worker(writable.clone());
        let video_worker_decoder = VideoWorkerDecoder::new(video_decoder, video_config, writable);
        self.video_worker_decoder = Some(video_worker_decoder);
    }

    pub fn decode(&mut self, data: Vec<u8>) {
        let packet = parse_media_packet(&data).unwrap();

        self.video_worker_decoder.as_mut().unwrap().decode(packet);
    }
}


fn parse_media_packet(data: &[u8]) -> Result<Arc<MediaPacket>, PeerDecodeError> {
    Ok(Arc::new(
        MediaPacket::parse_from_bytes(data).map_err(|_| PeerDecodeError::PacketParseError)?,
    ))
}
    