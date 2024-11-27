use web_sys::MediaStream;

use super::{create_video_stream, peer_decoder::DecodeStatus, WorkerDecoder};

#[derive(PartialEq, Debug)]
pub struct Video {
    pub media_stream: MediaStream,
    pub decoder: WorkerDecoder,
}

impl Video {
    pub fn new() -> Self {
        let (media_stream, media_stream_generator) = create_video_stream();

        let writable = media_stream_generator.writable();
        let ws  = writable.clone();
        let decoder = WorkerDecoder::new("v_worker", ws);
        Self {
            media_stream,
            decoder
        }
    }

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decoder.decode(media_packet)
    }

}