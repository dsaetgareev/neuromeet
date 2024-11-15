use std::cell::RefCell;
use std::rc::Rc;
use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use gloo_worker::{HandlerId, Worker, WorkerScope};
use js_sys::Uint8Array;
use serde::{Serialize, Deserialize};
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CodecState, VideoDecoder, VideoDecoderConfig, VideoDecoderInit, EncodedAudioChunkInit, EncodedAudioChunk, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType};

use crate::decode::DecodeStatus;
use crate::{constants::VIDEO_CODEC, decode::{create_video_decoder, create_video_decoder_for_worker}};

use super::{IdWrapper, VideoPacket, VideoWorker};
use crate::wrappers::EncodedVideoChunkTypeWrapper;
const MAX_BUFFER_SIZE: usize = 100;

#[derive(Clone)]
pub struct VideoWorkerDecoder {
    cache: BTreeMap<u64, Arc<VideoPacket>>,
    video_decoder: VideoDecoder,
    video_config: VideoDecoderConfig,
    pub sequence: Option<u64>,
    require_key: bool,
    scope: WorkerScope<VideoWorker>, 
    id: Option<Rc<RefCell<IdWrapper>>>,
}

impl VideoWorkerDecoder {

    pub fn new(
        video_decoder: VideoDecoder,
        video_config: VideoDecoderConfig,
        scope: &WorkerScope<VideoWorker>, 
        id: Option<Rc<RefCell<IdWrapper>>>,
    ) -> Self {
        Self {
            cache: BTreeMap::new(),
            video_decoder,
            video_config,
            sequence: None,
            require_key: false,
            scope: scope.clone(),
            id
        }
    }

    pub fn decode(&mut self, packet: Arc<VideoPacket>) -> Result<DecodeStatus, anyhow::Error> {
        let new_sequence_number = packet.sequence_number;
        let frame_type = packet.chunk_type.as_str();
        let cache_size = self.cache.len();
        if frame_type == "key" {
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

    pub fn decode_packet(&mut self, packet: Arc<VideoPacket>) {
        let encoded_video_chunk = get_encoded_video_chunk(packet);
        match self.state() {
            CodecState::Unconfigured => {
                log::info!("video decoder unconfigured");
            },
            CodecState::Configured => {
                log::info!("decodddddderrr");
                let _ = self.video_decoder.decode(&encoded_video_chunk);
            },
            CodecState::Closed => {
                log::error!("video decoder closed");
                self.require_key = true;

                let (video_decoder, video_config) = create_video_decoder_for_worker(self.scope.clone(), self.id.clone());     
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


pub fn get_encoded_video_chunk(packet: Arc<VideoPacket>) -> EncodedVideoChunk {
    get_encoded_video_chunk_from_data(packet)
}

pub fn get_encoded_video_chunk_from_data(video_data: Arc<VideoPacket>) -> EncodedVideoChunk {
    let data = Uint8Array::from(video_data.data.as_ref());
    // let frame_type = EncodedVideoChunkTypeWrapper(video_data.chunk_type);
    let frame_type = EncodedVideoChunkTypeWrapper::from(video_data.chunk_type.as_str()).0;
    let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, video_data.timestamp, frame_type);
    encoded_chunk_init.duration(video_data.duration);
    let encoded_video_chunk = EncodedVideoChunk::new(
        &encoded_chunk_init
    ).unwrap();
    encoded_video_chunk
}