use std::fmt::Debug;

use web_sys::MediaStream;

use crate::workers::VideoWorkerDecoder;

use super::{configure_video_stream, decoder_utils::{configure_video_decoder, ThreadType}, peer_decoder::DecodeStatus, Decode, WorkerDecoder};


pub struct Video {
    pub media_stream: MediaStream,
    pub decoder: Box<dyn Decode>,
}

impl Debug for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Video").field("media_stream", &self.media_stream).field("decoder", &self.decoder).finish()
    }
}

impl Video {
    pub fn new(video_type: ThreadType) -> Self {
        let (decoder, media_stream): (Box<dyn Decode>, MediaStream) = match video_type {
            ThreadType::Single => {
                let (video_decoder, video_config, media_stream, writable) = configure_video_decoder();   
                let decoder = VideoWorkerDecoder::new(video_decoder, video_config, writable);
                (Box::new(decoder), media_stream)
            },
            ThreadType::Multithread => {
                let (media_stream, media_stream_generator) = configure_video_stream();
                let writable = media_stream_generator.writable();
                let ws  = writable.clone();
                let decoder = WorkerDecoder::new("v_worker", ws);
                (Box::new(decoder), media_stream)
            },
        };
        Self {
            media_stream,
            decoder
        }
    }

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decoder.decode(media_packet)
    }

}