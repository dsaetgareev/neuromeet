use std::fmt::Debug;

use crate::workers::AudioWorkerDecoder;

use super::{configure_audio_decoder_for_worker, configure_audio_stream, decoder_utils::ThreadType, peer_decoder::DecodeStatus, Decode, WorkerDecoder};


pub struct Audio {
    pub decoder: Box<dyn Decode>,
}

impl Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio").field("decoder", &self.decoder).finish()
    }
}


impl Audio {
    pub fn new(thread_type: ThreadType) -> Self {
        let audio_stream_generator = configure_audio_stream();
        let writable = audio_stream_generator.writable();

        let decoder: Box<dyn Decode> = match thread_type {
            ThreadType::Single => {
                let audio_decoder = configure_audio_decoder_for_worker(writable);
                let decoder = AudioWorkerDecoder::new(audio_decoder);
                Box::new(decoder)
            },
            ThreadType::Multithread => {
                let decoder = WorkerDecoder::new("a_worker", writable);
                Box::new(decoder)
            },
        };

        Self {
            decoder
        }
    }    

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decoder.decode(media_packet)
    }
}