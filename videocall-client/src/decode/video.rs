use std::fmt::Debug;

use web_sys::WritableStream;

use crate::workers::VideoWorkerDecoder;

use super::{configure_video_decoder_for_worker, decoder_utils::ThreadType, peer_decoder::DecodeStatus, Decode, FakeDecoder, WorkerDecoder};


pub struct Video {
    thread_type: ThreadType,
    pub decoder: Box<dyn Decode>,
}

impl Debug for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Video").field("thread_type", &self.thread_type).field("decoder", &self.decoder).finish()
    }
}

impl Video {
    pub fn new(thread_type: ThreadType) -> Self {
        let decoder = Box::new(FakeDecoder::new());
        Self {
            thread_type,
            decoder,
        }
    }

    pub fn set_media(&mut self, origin_url: &str, video_ws: WritableStream) {
        let decoder: Box<dyn Decode> = match self.thread_type {
            ThreadType::Single => {
                let (video_decoder, video_config) = configure_video_decoder_for_worker(video_ws.clone());
                let decoder = VideoWorkerDecoder::new(video_decoder, video_config, video_ws);
                Box::new(decoder)
            },
            ThreadType::Multithread => {
                let decoder = WorkerDecoder::new(origin_url, "v_worker", video_ws);
                Box::new(decoder)
            },
        };
        self.decoder = decoder;
    }

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decoder.decode(media_packet)
    }

}