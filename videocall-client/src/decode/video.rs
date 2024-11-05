use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use types::protos::media_packet::MediaPacket;
use web_sys::{ VideoDecoder, VideoDecoderConfig, CodecState, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType };
use js_sys::Uint8Array;
use crate::{decode::video_decoder::create_video_decoder, wrappers::EncodedVideoChunkTypeWrapper};

use super::peer_decoder::DecodeStatus;

const MAX_BUFFER_SIZE: usize = 100;

#[derive(Clone, PartialEq, Debug)]
pub struct Video {
    pub cache: BTreeMap<u64, Arc<MediaPacket>>,
    pub video_decoder: VideoDecoder,
    pub video_config: VideoDecoderConfig,
    pub sequence: Option<u64>,
    pub require_key: bool,
    pub video_elem_id: String, 
}

impl Video {
    pub fn new(
        video_decoder: VideoDecoder,
        video_config: VideoDecoderConfig,
        video_elem_id: String,
    ) -> Self {
        Self {
            cache: BTreeMap::new(),
            video_decoder,
            video_config,
            sequence: None,
            require_key: false,
            video_elem_id
        }
    }

    pub fn decode(&mut self, packet: Arc<MediaPacket>) -> Result<DecodeStatus, anyhow::Error> {
        let new_sequence_number = packet.video_metadata.sequence;
        let frame_type = EncodedVideoChunkTypeWrapper::from(packet.frame_type.as_str()).0;
        let cache_size = self.cache.len();
        if frame_type == EncodedVideoChunkType::Key {
            self.require_key = false;
            self.decode_packet(packet);
            self.sequence = Some(new_sequence_number);
            self.prune_older_frames_from_buffer(new_sequence_number);
        } else if let Some(sequence) = self.sequence {
            if self.require_key {
                return Ok(DecodeStatus {
                    _rendered: true,
                    first_frame: self.require_key
                });
            }
            let is_next_frame = new_sequence_number == sequence + 1;
            if is_next_frame {
                self.decode_packet(packet);
                self.sequence = Some(new_sequence_number);
                self.play_queued_follow_up_frames();
                self.prune_older_frames_from_buffer(sequence);
            } else {
                let next_frame_already_cached = self.cache.contains_key(&(sequence + 1));
                if next_frame_already_cached {
                    self.play_queued_follow_up_frames();
                    self.prune_older_frames_from_buffer(sequence);
                }
                let is_future_frame = new_sequence_number > sequence;
                if is_future_frame {
                    self.cache.insert(new_sequence_number, packet);
                    if cache_size + 1 > MAX_BUFFER_SIZE {
                        // self.fast_forward_frames_and_then_prune_buffer();
                    }
                }
            }
        }
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: self.require_key
        })
    }

    pub fn decode_packet(&mut self, packet: Arc<MediaPacket>) {
        let encoded_video_chunk = get_encoded_video_chunk(packet);
        match self.state() {
            CodecState::Unconfigured => {
                log::info!("video decoder unconfigured");
            },
            CodecState::Configured => {
                let _ = self.video_decoder.decode(&encoded_video_chunk);
            },
            CodecState::Closed => {
                log::error!("video decoder closed");
                self.require_key = true;

                let (video_decoder, video_config) = create_video_decoder(&self.video_elem_id);     
                self.video_decoder = video_decoder;
                self.video_config = video_config;
                self.video_decoder.configure(&self.video_config);
            },
            _ => {},
        }
    }

    pub fn state(&self) -> CodecState {
        self.video_decoder.state()
    }

    fn prune_older_frames_from_buffer(&mut self, sequence_number: u64) {
        self.cache
            .retain(|sequence, _| *sequence >= sequence_number)
    }

    fn play_queued_follow_up_frames(&mut self) {
        let sorted_frames = self.cache.keys().collect::<Vec<_>>();
        if self.sequence.is_none() || sorted_frames.is_empty() {
            return;
        }
        for current_sequence in sorted_frames {
            let next_sequence = self.sequence.unwrap() + 1;
            match current_sequence.cmp(&next_sequence) {
                Ordering::Less => continue,
                Ordering::Equal => {
                    if let Some(next_image) = self.cache.get(current_sequence) {
                        let encoded_video_chunk = get_encoded_video_chunk(next_image.clone());
                        let _ = self.video_decoder.decode(&encoded_video_chunk);
                        // self.decode_packet(next_image.clone());
                        self.sequence = Some(next_sequence);
                    }
                }
                Ordering::Greater => break,
            }
        }
    }
}

pub fn get_encoded_video_chunk(packet: Arc<MediaPacket>) -> EncodedVideoChunk {
    // let video_data = get_video_data(packet);
    get_encoded_video_chunk_from_data(packet)
}

pub fn get_encoded_video_chunk_from_data(video_data: Arc<MediaPacket>) -> EncodedVideoChunk {
    let data = Uint8Array::from(video_data.data.as_ref());
    let chunk_type = EncodedVideoChunkTypeWrapper::from(video_data.frame_type.as_str()).0;
    // let chunk_type = EncodedVideoChunkTypeWrapper::from(video_data.chunk_type.as_str()).0;
    let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, video_data.timestamp, chunk_type);
    encoded_chunk_init.duration(video_data.duration);
    let encoded_video_chunk = EncodedVideoChunk::new(
        &encoded_chunk_init
    ).unwrap();
    encoded_video_chunk
}

// pub fn get_video_data(packet: Arc<VideoPacket>) -> VideoPacket{
//     let chunk_type = EncodedVideoChunkTypeWrapper::from(packet.chunk_type.as_str()).0;
//     let video_data = Uint8Array::new_with_length(packet.data.len().try_into().unwrap());
//     video_data.copy_from(&packet.data);
//     let video_chunk = EncodedVideoChunkInit::new(&video_data, packet.timestamp, chunk_type);
//     let chunk = EncodedVideoChunk::new(&video_chunk).unwrap();
    
//     let mut video_vector = vec![0u8; chunk.byte_length() as usize];
//     let video_message = video_vector.as_mut();
//     chunk.copy_to_with_u8_array(video_message);
//     let data = VideoPacket {
//         data: video_message.to_vec(),
//         chunk_type: packet.chunk_type.clone(),
//         timestamp: packet.timestamp,
//         duration: packet.duration,
//         sequence_number: packet.sequence_number
//     };
//     data
// }