use std::sync::Arc;

use types::protos::media_packet::MediaPacket;
use wasm_bindgen::JsValue;

use web_sys::AudioDecoder;
use web_sys::{
    EncodedAudioChunk, EncodedAudioChunkInit, EncodedAudioChunkType
};

use crate::decode::{parse_media_packet, Decode, DecodeStatus};

use super::PeerDecode;

#[derive(Clone, Debug)]
pub struct AudioWorkerDecoder {
    audio_decoder: AudioDecoder
}

impl AudioWorkerDecoder {

    pub fn new(audio_decoder: AudioDecoder) -> Self {
        Self {
            audio_decoder
        }
    }

    pub fn decode(&mut self, packet: Arc<MediaPacket>) {
        let chunk_type = get_chunk_type(&packet);
        let encoded_audio_chunk = get_chunk(&packet, chunk_type);
        let state = self.audio_decoder.state();
        match state {
            web_sys::CodecState::Unconfigured => {
                web_sys::console::log_1(&"audio decoder unconfigured".into());
                log::info!("audio decoder unconfigured");
            },
            web_sys::CodecState::Configured => {
                if let Err(_err) = self.audio_decoder.decode(&encoded_audio_chunk) {
                    web_sys::console::error_1(&JsValue::from("error decode audio"));
                }
            },
            web_sys::CodecState::Closed => {
                log::info!("audio_decoder closed");
            },
            _ => {}
        }
        
    }
    
}

impl PeerDecode for AudioWorkerDecoder {
    fn decode(&mut self, packet: Arc<MediaPacket>) {
        self.decode(packet);
    }
}

impl Decode for AudioWorkerDecoder {
    fn decode(&mut self, packet: &Vec<u8>) -> Result<crate::decode::DecodeStatus, anyhow::Error>  {
        let packet = parse_media_packet(&packet).expect("cannot parse media packet");
        self.decode(packet);
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }
}

fn get_chunk_type(packet: &Arc<MediaPacket>) -> EncodedAudioChunkType {
    EncodedAudioChunkType::from_js_value(&JsValue::from(packet.frame_type.clone())).unwrap()
}

fn get_chunk(
    packet: &Arc<MediaPacket>,
    chunk_type: EncodedAudioChunkType,
) -> EncodedAudioChunk {
    let audio_data = &packet.data;
    let audio_data_js: js_sys::Uint8Array =
        js_sys::Uint8Array::new_with_length(audio_data.len() as u32);
    audio_data_js.copy_from(audio_data.as_slice());
    let audio_chunk =
        EncodedAudioChunkInit::new(&audio_data_js.into(), packet.timestamp, chunk_type);
    audio_chunk.set_duration(packet.duration);
    EncodedAudioChunk::new(&audio_chunk).unwrap()
}