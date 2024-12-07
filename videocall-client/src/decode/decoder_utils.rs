use std::{fmt::Debug, sync::Arc};
use protobuf::Message;
use types::protos::media_packet::MediaPacket;
use wasm_bindgen::{prelude::Closure, JsValue, JsCast };
use wasm_bindgen_futures::JsFuture;
use log::error;
use web_sys::{AudioData, AudioDecoder, AudioDecoderConfig, AudioDecoderInit, 
    HtmlVideoElement, MediaStream, MediaStreamTrackGenerator, MediaStreamTrackGeneratorInit, VideoDecoder, VideoDecoderConfig, VideoDecoderInit, VideoFrame, WritableStream 
};
use js_sys::Array;

use crate::constants::{AUDIO_CHANNELS, AUDIO_CODEC, AUDIO_SAMPLE_RATE, VIDEO_CODEC};

use super::{config::configure_audio_context, DecodeStatus, PeerDecodeError};

pub trait Decode: Debug {
    fn decode(&mut self, packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error> ; 
}

pub struct FakeDecoder {

}

impl FakeDecoder {
    pub fn new() -> Self {
        Self {  }
    }
}

impl Decode for FakeDecoder {
    fn decode(&mut self, _packet: &Vec<u8>) -> Result<DecodeStatus, anyhow::Error>  {
        Ok(DecodeStatus {
            _rendered: true,
            first_frame: true
        })
    }
}

impl Debug for FakeDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeDecoder").finish()
    }
}

#[derive(Debug)]
pub enum ThreadType {
    Single,
    Multithread
}

pub fn _configure_video_decoder() -> (VideoDecoder, VideoDecoderConfig, MediaStream, WritableStream) {
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);

    let video_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("video")).unwrap();
    let js_tracks = Array::new();
    js_tracks.push(&video_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();

    let writable_stream = video_stream_generator.writable();
    let writable = writable_stream.clone();
    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();              
        if writable.locked() {
            return;
        }
        if let Err(e) = writable.get_writer().map(|writer| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = JsFuture::from(writer.ready()).await {
                    error!("write chunk error {:?}", e);
                }
                if let Err(e) = JsFuture::from(writer.write_with_chunk(&video_chunk)).await {
                    error!("write chunk error {:?}", e);
                };
                video_chunk.unchecked_into::<VideoFrame>().close();
                writer.release_lock();
            });
        }) {
            error!("error {:?}", e);
        }        
    }) as Box<dyn FnMut(JsValue)>);

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC); 
    local_video_decoder.configure(&video_config);
    (local_video_decoder, video_config, media_stream, writable_stream)
}

pub fn configure_video_decoder_for_worker(writable: WritableStream) -> (VideoDecoder, VideoDecoderConfig) {
    
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        log::error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);

    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();              

        if writable.locked() {
            return;
        }
        if let Err(e) = writable.get_writer().map(|writer| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = JsFuture::from(writer.ready()).await {
                    error!("write chunk error {:?}", e);
                }
                if let Err(e) = JsFuture::from(writer.write_with_chunk(&video_chunk)).await {

                    error!("write chunk error {:?}", e);
                };
                video_chunk.unchecked_into::<VideoFrame>().close();
                writer.release_lock();
            });
        }) {
            web_sys::console::error_1(&e);
            error!("error {:?}", e);
        }        
    }) as Box<dyn FnMut(JsValue)>);

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC); 
    local_video_decoder.configure(&video_config);
    log::info!("docoder created");
    (local_video_decoder, video_config)
}

pub fn configure_audio_decoder_for_worker(writable: WritableStream) -> AudioDecoder {

    let error = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);

    let output = Closure::wrap(Box::new(move |audio_data: AudioData| {
        if writable.locked() {
            return;
        }
        if let Err(e) = writable.get_writer().map(|writer| {
            wasm_bindgen_futures::spawn_local(async move {

                if let Err(e) = JsFuture::from(writer.ready()).await {
                    error!("write chunk error {:?}", e);
                }

                if let Err(e) = JsFuture::from(writer.write_with_chunk(&audio_data)).await {
                    error!("write chunk error {:?}", e);
                };
                writer.release_lock();
            });
        }) {
            error!("error {:?}", e);
        }
    }) as Box<dyn FnMut(AudioData)>);
    let decoder = AudioDecoder::new(&AudioDecoderInit::new(
        error.as_ref().unchecked_ref(),
        output.as_ref().unchecked_ref(),
    ))
    .unwrap();

    decoder.configure(&AudioDecoderConfig::new(
        AUDIO_CODEC,
        AUDIO_CHANNELS,
        AUDIO_SAMPLE_RATE,
    )); 
    error.forget();
    output.forget();
    decoder
}

pub fn configure_video_stream() -> (MediaStream, MediaStreamTrackGenerator) {
    let video_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("video")).unwrap();
    let js_tracks = Array::new();
    js_tracks.push(&video_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();
    (media_stream, video_stream_generator)
}

pub fn configure_audio_stream() ->  MediaStreamTrackGenerator {
    let audio_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("audio")).unwrap();
    let _audio_context = configure_audio_context(&audio_stream_generator).unwrap();
    audio_stream_generator
}

pub fn parse_media_packet(data: &[u8]) -> Result<Arc<MediaPacket>, PeerDecodeError> {
    Ok(Arc::new(
        MediaPacket::parse_from_bytes(data).map_err(|_| PeerDecodeError::PacketParseError)?,
    ))
}