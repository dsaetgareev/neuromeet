use std::{cell::RefCell, rc::Rc, time::{SystemTime, UNIX_EPOCH}};
use protobuf::Message;
use gloo_worker::{HandlerId, WorkerScope};
use wasm_bindgen::{prelude::Closure, JsValue, JsCast };
use wasm_bindgen_futures::JsFuture;
use gloo_utils::format::JsValueSerdeExt;
use log::error;
use web_sys::{ 
    MediaStream, MediaStreamTrackGenerator, MediaStreamTrackGeneratorInit, 
    VideoDecoder, VideoDecoderConfig, VideoDecoderInit, VideoFrame,
    HtmlVideoElement 
};
use js_sys::{Array, Uint8Array};

use crate::{constants::VIDEO_CODEC, workers::IdWrapper, VideoWorker, VideoWorkerOutput};

use super::video::Video;



pub fn create_video_decoder(video_elem_id: &str) -> (VideoDecoder, VideoDecoderConfig, MediaStream) {
    let error_id = video_elem_id.to_string();
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
        error!("error from id: {}", error_id);
    }) as Box<dyn FnMut(JsValue)>);

    let video_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("video")).unwrap();
    let js_tracks = Array::new();
    js_tracks.push(&video_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();

    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();              
        let writable = video_stream_generator.writable();
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
    (local_video_decoder, video_config, media_stream)
}

pub fn create_video_decoder_for_worker(scope: WorkerScope<VideoWorker>, id: HandlerId) -> (VideoDecoder, VideoDecoderConfig) {
    
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        log::error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);

    let id  = id.clone();
    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        // let aaa = format!("shcunk {:?}", original_chunk);
        let uint8_array = Uint8Array::new(&original_chunk);
        let data = uint8_array.to_vec();
        let length = data.len();
        let id = id;
        scope.respond(
            id,
            VideoWorkerOutput {
                data: data, 
                id: length.to_string(),
            });
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

pub fn create_video_stream() -> (MediaStream, MediaStreamTrackGenerator) {
    let video_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("video")).unwrap();
    let js_tracks = Array::new();
    js_tracks.push(&video_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();
    (media_stream, video_stream_generator)
}

pub fn video_handle(video_stream_generator: MediaStreamTrackGenerator, original_chunk: JsValue) {
    let chunk = Box::new(original_chunk);
    let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();              
    let writable = video_stream_generator.writable();
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
}