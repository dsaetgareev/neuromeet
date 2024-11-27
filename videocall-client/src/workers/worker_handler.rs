use std::sync::Arc;

use protobuf::Message;
use types::protos::media_packet::MediaPacket;
use web_sys::WritableStream;

use crate::decode::{configure_audio_decoder_for_worker, create_video_decoder_for_worker, PeerDecodeError};

use super::{video_worker_decoder::VideoWorkerDecoder, AudioWorkerDecoder};

pub enum DecoderType {
    Audio,
    Video,
}

pub trait PeerDecode {
    fn decode(&mut self, packet: Arc<MediaPacket>); 
}

// #[derive(Clone)]
pub struct WorkerHandler {
    decoder_type: DecoderType,
    worker_decoder: Option<Box<dyn PeerDecode>>,
    writable: Option<WritableStream>
}

impl WorkerHandler {
    pub fn new(decoder_type: DecoderType) -> WorkerHandler {
        Self {
            decoder_type,
            worker_decoder: None,
            writable: None,
        }
    }

    pub fn init(&mut self, writable: WritableStream) {
        let decoder_type = &self.decoder_type;
        self.writable = Some(writable.clone());
        match decoder_type {
            DecoderType::Audio => {
                let audio_decoder = configure_audio_decoder_for_worker(writable);
                let audio_worker_decoder = AudioWorkerDecoder::new(audio_decoder);
                self.worker_decoder = Some(Box::new(audio_worker_decoder));
            },
            DecoderType::Video => {
                let (video_decoder, video_config) = create_video_decoder_for_worker(writable.clone());
                let video_worker_decoder = VideoWorkerDecoder::new(video_decoder, video_config, writable);
                self.worker_decoder = Some(Box::new(video_worker_decoder));
            },
        }
    }

    pub fn handle(&mut self, data: Vec<u8>) {
        let packet = parse_media_packet(&data).expect("cannot parse media packet");
        if let Some(worker_decoder) = self.worker_decoder.as_mut() {
            worker_decoder.decode(packet);
        }
    }

    pub fn reset(&mut self) {
        if let Some(writable) = &self.writable {
            self.init(writable.clone());
        }
    }
}


fn parse_media_packet(data: &[u8]) -> Result<Arc<MediaPacket>, PeerDecodeError> {
    Ok(Arc::new(
        MediaPacket::parse_from_bytes(data).map_err(|_| PeerDecodeError::PacketParseError)?,
    ))
}
    