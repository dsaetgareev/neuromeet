use super::{create_audio_stream, peer_decoder::DecodeStatus, WorkerDecoder};


#[derive(PartialEq, Debug)]
pub struct Audio {
    pub decoder: WorkerDecoder,
}


impl Audio {
    pub fn new() -> Self {
        let audio_stream_generator = create_audio_stream();
        let writable = audio_stream_generator.writable();
        let ws  = writable.clone();
        let decoder = WorkerDecoder::new("a_worker", ws);
        Self {
            decoder
        }
    }    

    pub fn decode(&mut self, media_packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> {
        self.decoder.decode(media_packet)
    }
}