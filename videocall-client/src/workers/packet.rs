use std::sync::Arc;

use js_sys::Uint8Array;
use serde::{Serialize, Deserialize};
use types::protos::media_packet::MediaPacket;
use web_sys::{EncodedAudioChunkInit, EncodedAudioChunk, EncodedVideoChunk, EncodedVideoChunkInit};

use crate::wrappers::EncodedVideoChunkTypeWrapper;
#[derive(Serialize, Deserialize, Clone, PartialEq)]

pub struct VideoPacket {
    pub data: Vec<u8>,
    pub chunk_type: String,
    pub timestamp: f64,
    pub duration: f64,
    pub sequence_number: u64,
}

impl VideoPacket {
    pub fn new(
        chunk: web_sys::EncodedVideoChunk,
        sequence_number: u64,
    ) -> Self {
        let duration = chunk.duration().expect("no duration video chunk");
        let mut buffer: [u8; 1000000] = [0; 1000000];
        let byte_length = chunk.byte_length() as usize;
        chunk.copy_to_with_u8_array(&mut buffer);
        let data = buffer[0..byte_length].to_vec();
        let chunk_type = EncodedVideoChunkTypeWrapper(chunk.type_()).to_string();
        let timestamp = chunk.timestamp();

        Self {
            data,
            chunk_type,
            timestamp,
            duration,
            sequence_number,
        }
    }

    pub fn from(media_packet: Arc<MediaPacket>) -> Self {
        let data = media_packet.data.clone();
        // let chunk_type = EncodedVideoChunkTypeWrapper(media_packet.frame_type).to_string();
        let chunk_type = media_packet.frame_type.clone();
        let timestamp = media_packet.timestamp;
        let duration = media_packet.duration;
        let sequence_number = media_packet.video_metadata.sequence;
        Self {
            data,
            chunk_type,
            timestamp,
            duration,
            sequence_number,
        }
    }

    pub fn get_encoded_video_chunk_from_data(video_data: Arc<VideoPacket>) -> EncodedVideoChunk {
        let data = Uint8Array::from(video_data.data.as_ref());
        let chunk_type = EncodedVideoChunkTypeWrapper::from(video_data.chunk_type.as_str()).0;
        
        let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, video_data.timestamp, chunk_type);
        encoded_chunk_init.duration(video_data.duration);
        let encoded_video_chunk = EncodedVideoChunk::new(
            &encoded_chunk_init
        ).unwrap();
        encoded_video_chunk
    }

}