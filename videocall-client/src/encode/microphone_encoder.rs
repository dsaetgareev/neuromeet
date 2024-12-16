use gloo_utils::window;
use js_sys::Array;
use js_sys::Boolean;
use web_sys::ReadableStream;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::AudioTrack;
use web_sys::MediaStream;
use web_sys::MediaStreamConstraints;
use web_sys::MediaStreamTrack;
use web_sys::MediaStreamTrackProcessor;
use web_sys::MediaStreamTrackProcessorInit;

use super::device_state::DeviceState;
use super::ReadableType;
use super::Sender;

#[derive(Clone, PartialEq)]
pub struct MicrophoneEncoder {
    state: DeviceState,
    media_track: Option<MediaStreamTrack>,
    readable_stream: Option<ReadableStream>,
}

impl MicrophoneEncoder {
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

    pub fn set_enabled(&mut self, value: bool) -> bool {
        self.state.set_enabled(value)
    }

    pub fn get_enabled(&self) -> bool {
        self.state.is_enabled()
    }

    pub fn select(&mut self, device: String) -> bool {
        self.state.select(device)
    }

    pub fn stop(&mut self) {
        if let Some(meida_track) = &self.media_track {
            web_sys::console::log_1(&"media_track stop".into());
            meida_track.stop();
        }
        if let Some(readable_stream) = &self.readable_stream {
            web_sys::console::log_1(&"readable_stream stop".into());
            let _ = readable_stream.cancel();
        }
    }

    pub fn switch_enabled(&mut self) -> bool {
        let is_enabled = self.get_enabled();
        self.set_enabled(!is_enabled); 
        is_enabled
    }

    pub fn start(&mut self, sender: Box<dyn Sender>) {
        self.stop();
        let device_id = if let Some(mic) = &self.state.selected {
            mic.to_string()
        } else {
            return;
        };

        wasm_bindgen_futures::spawn_local(async move {
            let navigator = window().navigator();
            let media_devices = navigator.media_devices().unwrap();
            // TODO: Add dropdown so that user can select the device that they want to use.
            let constraints = MediaStreamConstraints::new();
            let media_info = web_sys::MediaTrackConstraints::new();
            media_info.set_device_id(&device_id.into());

            constraints.set_audio(&media_info.into());
            constraints.set_video(&Boolean::from(false));
            let devices_query = media_devices
                .get_user_media_with_constraints(&constraints)
                .unwrap();
            let device = JsFuture::from(devices_query)
                .await
                .unwrap()
                .unchecked_into::<MediaStream>();

            let audio_track = Box::new(
                device
                    .get_audio_tracks()
                    .find(&mut |_: JsValue, _: u32, _: Array| true)
                    .unchecked_into::<AudioTrack>(),
            );

            let audio_processor =
                MediaStreamTrackProcessor::new(&MediaStreamTrackProcessorInit::new(
                    &audio_track.clone().unchecked_into::<MediaStreamTrack>(),
                ))
                .unwrap();
            let audio_readable = audio_processor.readable();
            let media_track = &audio_track.clone().unchecked_into::<MediaStreamTrack>();
            sender.send_readable(ReadableType::Audio, audio_readable, media_track.clone());
        });
    }

}
