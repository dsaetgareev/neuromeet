use gloo_utils::window;
use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::MediaStream;
use web_sys::MediaStreamTrack;
use web_sys::MediaStreamTrackProcessor;
use web_sys::MediaStreamTrackProcessorInit;
use web_sys::ReadableStream;
use web_sys::VideoTrack;

use super::device_state::DeviceState;
use super::ReadableType;
use super::Sender;

#[derive(Clone, PartialEq)]
pub struct ScreenEncoder {
    state: DeviceState,
    media_track: Option<MediaStreamTrack>,
    readable_stream: Option<ReadableStream>,
}

impl ScreenEncoder {
    pub fn new() -> Self {
        Self {
            state: DeviceState::new(),
            media_track: None,
            readable_stream: None,
        }
    }

    pub fn set_media_track(&mut self, readable_stream: ReadableStream, media_track: MediaStreamTrack) {
        self.media_track = Some(media_track);
        self.readable_stream = Some(readable_stream);
    }

    pub fn get_enabled(&self) -> bool {
        self.state.is_enabled()
    }

    pub fn set_enabled(&mut self, value: bool) -> bool {
        self.state.set_enabled(value)
    }

    pub fn stop(&mut self) {
        if let Some(meida_track) = &self.media_track {
            meida_track.stop();
        }
        if let Some(readable_stream) = &self.readable_stream {
            let _ = readable_stream.cancel();
        }
    }

    pub fn switch_enabled(&mut self) -> bool {
        let is_enabled = self.get_enabled();
        self.set_enabled(!is_enabled); 
        is_enabled
    }

    pub fn start(
        &mut self,
        sender: Box<dyn Sender>
    ) {

        self.stop();

        wasm_bindgen_futures::spawn_local(async move {
            let navigator = window().navigator();
            let media_devices = navigator.media_devices().unwrap();
            let screen_to_share: MediaStream =
                JsFuture::from(media_devices.get_display_media().unwrap())
                    .await
                    .unwrap()
                    .unchecked_into::<MediaStream>();

            // TODO: How can we determine the actual width and height of the screen to set the encoder config?
            let screen_track = Box::new(
                screen_to_share
                    .get_video_tracks()
                    .find(&mut |_: JsValue, _: u32, _: Array| true)
                    .unchecked_into::<VideoTrack>(),
            );

            let screen_processor =
                MediaStreamTrackProcessor::new(&MediaStreamTrackProcessorInit::new(
                    &screen_track.clone().unchecked_into::<MediaStreamTrack>(),
                ))
                .unwrap();

            let screen_readable = screen_processor
                .readable();
            let media_track = &screen_track
                .clone()
                .unchecked_into::<MediaStreamTrack>();
            sender.send_readable(ReadableType::Screen, screen_readable, media_track.clone());
        });
    }
}
