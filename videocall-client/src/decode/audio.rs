use std::fmt::Debug;

use web_sys::WritableStream;

use crate::workers::AudioWorkerDecoder;

use super::{configure_audio_decoder_for_worker, decoder_utils::ThreadType, peer_decoder::DecodeStatus, Decode, FakeDecoder, WorkerDecoder};


pub struct Audio {
    thread_type: ThreadType,
    pub decoder: Box<dyn Decode>,
}

impl Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio").field("decoder", &self.decoder).finish()
    }
}


impl Audio {
    pub fn new(thread_type: ThreadType) -> Self {
        let decoder = Box::new(FakeDecoder::new());
        Self {
            thread_type,
            decoder,
        }
    }

    pub fn set_media(&mut self, origin_url: &str, audio_ws: WritableStream) {

        let decoder: Box<dyn Decode> = match self.thread_type {
            ThreadType::Single => {
                let audio_decoder = configure_audio_decoder_for_worker(audio_ws);
                let decoder = AudioWorkerDecoder::new(audio_decoder);
                Box::new(decoder)
            },
            ThreadType::Multithread => {
                let decoder = WorkerDecoder::new(origin_url, "a_worker", audio_ws);
                Box::new(decoder)
            },
        };
        self.decoder = decoder;
    }    

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decoder.decode(media_packet)
    }
}