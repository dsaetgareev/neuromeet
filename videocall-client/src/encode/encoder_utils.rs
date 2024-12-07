use std::cell::RefCell;
use std::rc::Rc;
use log::error;

use types::protos::packet_wrapper::PacketWrapper;
use wasm_bindgen::{prelude::Closure, JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::AudioData;
use web_sys::AudioEncoder;
use web_sys::AudioEncoderConfig;
use web_sys::AudioEncoderInit;
use web_sys::CodecState;
use web_sys::MediaStreamTrack;
use web_sys::ReadableStream;
use web_sys::ReadableStreamDefaultReader;
use web_sys::VideoEncoder;
use web_sys::VideoEncoderConfig;
use web_sys::VideoEncoderEncodeOptions;
use web_sys::VideoEncoderInit;
use web_sys::VideoFrame;
use web_sys::LatencyMode;
use js_sys::JsString;
use js_sys::Reflect;
use serde::{Deserialize, Serialize};

use crate::constants::AUDIO_BITRATE;
use crate::constants::AUDIO_CHANNELS;
use crate::constants::AUDIO_CODEC;
use crate::constants::AUDIO_SAMPLE_RATE;
use crate::constants::SCREEN_HEIGHT;
use crate::constants::SCREEN_WIDTH;
use crate::constants::VIDEO_CODEC;
use crate::constants::VIDEO_HEIGHT;
use crate::constants::VIDEO_WIDTH;
use crate::crypto::aes::Aes128State;

use super::transform::transform_audio_chunk;
use super::transform::transform_screen_chunk;
use super::transform::transform_video_chunk;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ReadableType {
    Audio,
    Video,
    Screen,
}

pub trait Sender {
    fn send_readable(&self, readable_type: ReadableType, readable: ReadableStream, media_track: MediaStreamTrack);
}

pub struct EncoderHandler {
    audio_encoder: Option<Box<AudioEncoder>>,
    video_encoder: Option<Box<VideoEncoder>>,
    screen_encoder: Option<Box<VideoEncoder>>,
}

impl EncoderHandler {
    pub fn new() -> Self {
        Self {
            audio_encoder: None,
            video_encoder: None,
            screen_encoder: None,
        }
    }

    pub fn set_audio(&mut self, audio_encoder: Box<AudioEncoder>) {
        self.audio_encoder = Some(audio_encoder);
    }

    pub fn set_video(&mut self, video_encoder: Box<VideoEncoder>) {
        self.video_encoder = Some(video_encoder);
    }

    pub fn set_screen(&mut self, screen_encoder: Box<VideoEncoder>) {
        self.screen_encoder = Some(screen_encoder);
    }

    pub fn audio_close(&mut self) {
        if let Some(audio_encoder) = &self.audio_encoder {
            audio_encoder.close();
        }
    }

    pub fn video_close(&mut self) {
        if let Some(video_encoder) = &self.video_encoder {
            video_encoder.close()
        }
    }

    pub fn screen_close(&mut self) {
        if let Some(screen_encoder) = &self.screen_encoder {
            screen_encoder.close()
        }
    }
}



pub fn configure_camera_encoder(
    readable: ReadableStream,
    on_frame: impl Fn(PacketWrapper) + 'static,
    user_id: String,
    aes: Rc<Aes128State>,
    encoder_handler: Rc<RefCell<EncoderHandler>>,
) {
    let userid = user_id;
    let aes = aes;

    let video_output_handler = {
        let mut buffer: [u8; 100000] = [0; 100000];
        let mut sequence_number = 0;
        let on_frame = on_frame;
        Box::new(move |chunk: JsValue| {
            let chunk = web_sys::EncodedVideoChunk::from(chunk);
            let packet: PacketWrapper = transform_video_chunk(
                chunk,
                sequence_number,
                &mut buffer,
                &userid,
                aes.clone(),
            );
            on_frame(packet);
            sequence_number += 1;
        })
    };

    wasm_bindgen_futures::spawn_local(async move {

        let video_error_handler = Closure::wrap(Box::new(move |e: JsValue| {
            error!("error_handler error {:?}", e);
        }) as Box<dyn FnMut(JsValue)>);

        let video_output_handler =
            Closure::wrap(video_output_handler as Box<dyn FnMut(JsValue)>);

        let video_encoder_init = VideoEncoderInit::new(
            video_error_handler.as_ref().unchecked_ref(),
            video_output_handler.as_ref().unchecked_ref(),
        );

        let video_encoder = Box::new(VideoEncoder::new(&video_encoder_init).unwrap());

        let mut video_encoder_config =
            VideoEncoderConfig::new(VIDEO_CODEC, VIDEO_HEIGHT as u32, VIDEO_WIDTH as u32);

        video_encoder_config.bitrate(100_000f64);
        video_encoder_config.latency_mode(LatencyMode::Realtime);
        video_encoder.configure(&video_encoder_config);
        encoder_handler.borrow_mut().set_video(video_encoder.clone());
        let video_reader = readable
            .get_reader()
            .unchecked_into::<ReadableStreamDefaultReader>();

        // Start encoding video and audio.
        let mut video_frame_counter = 0;
        let poll_video = async {
            loop {
                match JsFuture::from(video_reader.read()).await {
                    Ok(js_frame) => {
                        let video_frame = Reflect::get(&js_frame, &JsString::from("value"))
                            .unwrap()
                            .unchecked_into::<VideoFrame>();
                        let mut opts = VideoEncoderEncodeOptions::new();
                        video_frame_counter = (video_frame_counter + 1) % 50;
                        opts.key_frame(video_frame_counter == 0);
                        let state: CodecState = video_encoder.state();
                        match state {
                            CodecState::Unconfigured => {
                                web_sys::console::log_1(&"Video encoder unconfigured".into()); 
                            },
                            CodecState::Configured => {
                                if video_frame.is_undefined() || !video_frame.is_instance_of::<VideoFrame>() {
                                    return;
                                }
                                video_encoder.encode_with_options(&video_frame, &opts);
                            },
                            CodecState::Closed => {
                                web_sys::console::log_1(&"Video encoder closed".into());
                                return;
                            },
                            _ => todo!(),
                        }
                        video_frame.close();
                    }
                    Err(e) => {
                        error!("error {:?}", e);
                    }
                }
            }
        };
        poll_video.await;
    });

}

pub fn configure_screen_encoder(
    readable: ReadableStream,
    on_frame: impl Fn(PacketWrapper) + 'static,
    user_id: String,
    aes: Rc<Aes128State>,
    encoder_handler: Rc<RefCell<EncoderHandler>>
) {
    let userid = user_id;
    let aes = aes;
    let screen_output_handler = {
        let mut buffer: [u8; 150000] = [0; 150000];
        let mut sequence_number = 0;
        Box::new(move |chunk: JsValue| {
            let chunk = web_sys::EncodedVideoChunk::from(chunk);
            let packet: PacketWrapper = transform_screen_chunk(
                chunk,
                sequence_number,
                &mut buffer,
                &userid,
                aes.clone(),
            );
            on_frame(packet);
            sequence_number += 1;
        })
    };
    wasm_bindgen_futures::spawn_local(async move {

        let screen_error_handler = Closure::wrap(Box::new(move |e: JsValue| {
            error!("error_handler error {:?}", e);
        }) as Box<dyn FnMut(JsValue)>);

        let screen_output_handler =
            Closure::wrap(screen_output_handler as Box<dyn FnMut(JsValue)>);

        let screen_encoder_init = VideoEncoderInit::new(
            screen_error_handler.as_ref().unchecked_ref(),
            screen_output_handler.as_ref().unchecked_ref(),
        );

        let screen_encoder = Box::new(VideoEncoder::new(&screen_encoder_init).unwrap());
        let mut screen_encoder_config =
            VideoEncoderConfig::new(VIDEO_CODEC, SCREEN_HEIGHT, SCREEN_WIDTH);
        screen_encoder_config.bitrate(64_000f64);
        screen_encoder_config.latency_mode(LatencyMode::Realtime);
        screen_encoder.configure(&screen_encoder_config);
        encoder_handler.borrow_mut().set_screen(screen_encoder.clone());

        let screen_reader = readable
            .get_reader()
            .unchecked_into::<ReadableStreamDefaultReader>();

        let mut screen_frame_counter = 0;

        let poll_screen = async {
            loop {
                match JsFuture::from(screen_reader.read()).await {
                    Ok(js_frame) => {
                        let video_frame = Reflect::get(&js_frame, &JsString::from("value"))
                            .unwrap()
                            .unchecked_into::<VideoFrame>();
                        let mut opts = VideoEncoderEncodeOptions::new();
                        screen_frame_counter = (screen_frame_counter + 1) % 50;
                        opts.key_frame(screen_frame_counter == 0);
                        let state: CodecState = screen_encoder.state();
                        match state {
                            CodecState::Unconfigured => {
                                web_sys::console::log_1(&"Screen encoder unconfigured".into());
                            },
                            CodecState::Configured => {
                                if video_frame.is_undefined() || !video_frame.is_instance_of::<VideoFrame>() {
                                    return;
                                }
                                screen_encoder.encode_with_options(&video_frame, &opts);
                            },
                            CodecState::Closed => {
                                web_sys::console::log_1(&"Screen encoder closed".into());
                                return;
                            },
                            _ => todo!(),
                        }
                        screen_encoder.encode_with_options(&video_frame, &opts);
                        video_frame.close();
                    }
                    Err(e) => {
                        error!("error {:?}", e);
                    }
                }
            }
        };
        poll_screen.await;
    });
}

pub fn configure_audio_encoder(
    readable: ReadableStream,
    on_audio: impl Fn(PacketWrapper) + 'static,
    user_id: String,
    aes: Rc<Aes128State>,
    encoder_handler: Rc<RefCell<EncoderHandler>>
) {
    let userid = user_id;
    let aes = aes;
    let audio_output_handler = {
        let mut buffer: [u8; 100000] = [0; 100000];
        let mut sequence = 0;
        let on_audio = on_audio;
        Box::new(move |chunk: JsValue| {
            let chunk = web_sys::EncodedAudioChunk::from(chunk);
            let packet: PacketWrapper =
                transform_audio_chunk(&chunk, &mut buffer, &userid, sequence, aes.clone());
            on_audio(packet);
            sequence += 1;
        })
    };

    wasm_bindgen_futures::spawn_local(async move {

        let audio_error_handler = Closure::wrap(Box::new(move |e: JsValue| {
            error!("error_handler error {:?}", e);
        }) as Box<dyn FnMut(JsValue)>);

        let audio_output_handler =
            Closure::wrap(audio_output_handler as Box<dyn FnMut(JsValue)>);

        let audio_encoder_init = AudioEncoderInit::new(
            audio_error_handler.as_ref().unchecked_ref(),
            audio_output_handler.as_ref().unchecked_ref(),
        );
        let audio_encoder = Box::new(AudioEncoder::new(&audio_encoder_init).unwrap());
        let mut audio_encoder_config = AudioEncoderConfig::new(AUDIO_CODEC);
        audio_encoder_config.bitrate(AUDIO_BITRATE);
        audio_encoder_config.sample_rate(AUDIO_SAMPLE_RATE);
        audio_encoder_config.number_of_channels(AUDIO_CHANNELS);
        audio_encoder.configure(&audio_encoder_config);

        let audio_reader = readable
            .get_reader()
            .unchecked_into::<ReadableStreamDefaultReader>();
        encoder_handler.borrow_mut().set_audio(audio_encoder.clone());
        let poll_audio = async {
            loop {
                match JsFuture::from(audio_reader.read()).await {
                    Ok(js_frame) => {
                        let audio_frame = Reflect::get(&js_frame, &JsString::from("value"))
                            .unwrap()
                            .unchecked_into::<AudioData>();
                        let state = audio_encoder.state();
                        match state {
                            web_sys::CodecState::Unconfigured => {
                                web_sys::console::log_1(&"Audio encoder uncofigured".into());
                            },
                            web_sys::CodecState::Configured => {
                                audio_encoder.encode(&audio_frame);
                            },
                            web_sys::CodecState::Closed => {
                                web_sys::console::log_1(&"Audio encoder closed".into());
                                return;
                            },
                            _ => {
                                return;
                            },
                        }
                        audio_frame.close();
                    }
                    Err(e) => {
                        error!("error {:?}", e);
                    }
                }
            }
        };
        poll_audio.await;
    });
}