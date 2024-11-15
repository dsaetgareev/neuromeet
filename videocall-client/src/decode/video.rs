use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use gloo_worker::{Spawnable, WorkerBridge};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::JsValue;
use web_sys::{CodecState, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType, MediaStream, VideoDecoder, VideoDecoderConfig};
use js_sys::Uint8Array;
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

        let worker = VideoWorker::spawner()
            .callback(move |output| {
                let data = output.data;
                // let js_value: JsValue = JsValue::from_serde(&data).unwrap();

                // // video_handle(media_stream_generator, original_chunk);
                log::info!("from woker {:?}", data);
                log::info!("iddddd {:?}", output.id);
            })
            .spawn("../worker.js");

        Self {
            media_stream,
            worker
        }
    }

    pub fn decode(&mut self, packet: Arc<MediaPacket>) -> Result<DecodeStatus, anyhow::Error> {
        // log::info!("data len {}", packet.data.len());
        self.worker.send(VideoWorkerInput {data: VideoPacket::from(packet)});
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }

}