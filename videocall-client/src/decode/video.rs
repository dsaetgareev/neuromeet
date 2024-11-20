use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use gloo_worker::{Spawnable, WorkerBridge};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{VideoFrame, CodecState, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType, HtmlVideoElement, MediaStream, VideoDecoder, VideoDecoderConfig};
use js_sys::Uint8Array;
use protobuf::Message;
use crate::{constants::{VIDEO_HEIGHT, VIDEO_WIDTH}, decode::video_decoder::{create_video_decoder, video_handle}, workers::VideoPacket, wrappers::EncodedVideoChunkTypeWrapper, VideoWorker, VideoWorkerInput};

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
                let buf = output.data;

                let data = Uint8Array::from(buf.as_ref());
                let vfbi = web_sys::VideoFrameBufferInit::new(
                    VIDEO_HEIGHT as u32,
                    VIDEO_WIDTH as u32,
                    web_sys::VideoPixelFormat::I420,
                    output.timestamp,
                );

                let chunk = web_sys::VideoFrame::new_with_buffer_source_and_video_frame_buffer_init(&data, &vfbi).unwrap();
        
        
                let original_chunk = chunk.unchecked_into::<VideoFrame>();

                video_handle(media_stream_generator.clone(), original_chunk);
            })
            .spawn("../worker.js");

        Self {
            media_stream,
            worker
        }
    }

    pub fn decode(&mut self, media_packet: Arc<MediaPacket>) -> Result<DecodeStatus, anyhow::Error> {
        let data = media_packet.write_to_bytes().unwrap();
        self.worker.send(VideoWorkerInput {data});
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }

}