use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use gloo_worker::{Spawnable, WorkerBridge};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::JsValue;
use web_sys::{CodecState, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType, MediaStream, VideoDecoder, VideoDecoderConfig};
use js_sys::Uint8Array;
use protobuf::Message;
use crate::{decode::video_decoder::{create_video_decoder, video_handle}, workers::VideoPacket, wrappers::EncodedVideoChunkTypeWrapper, VideoWorker, VideoWorkerInput};

use super::{create_video_stream, peer_decoder::DecodeStatus};

const MAX_BUFFER_SIZE: usize = 100;

#[derive(PartialEq, Debug)]
pub struct Video {
    pub media_stream: MediaStream,
    pub worker: WorkerBridge<VideoWorker>,
}

impl Video {
    pub fn new() -> Self {
        let (media_stream, media_stream_generator) = create_video_stream();

        let media_stream_generator = media_stream_generator.clone();
        let worker = VideoWorker::spawner()
            .callback(move |output| {
                let data = output.data;
                let uint8 = Uint8Array::new_with_length(data.len() as u32);
                uint8.copy_from(&data);
                let js_value = JsValue::from(uint8);

                // video_handle(media_stream_generator.clone(), js_value);
                log::info!("from woker {:?}", data);
                log::info!("iddddd {:?}", output.id);
            })
            .spawn("../worker.js");

        Self {
            media_stream,
            worker
        }
    }

    pub fn decode(&mut self, media_packet: Arc<MediaPacket>) -> Result<DecodeStatus, anyhow::Error> {
        let data = media_packet.write_to_bytes().unwrap();
        // log::info!("data len {}", packet.data.len());
        self.worker.send(VideoWorkerInput {data});
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }

}