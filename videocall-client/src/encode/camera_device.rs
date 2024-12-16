use gloo_utils::window;
use js_sys::Array;
use js_sys::Boolean;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlVideoElement;
use web_sys::MediaStream;
use web_sys::MediaStreamConstraints;
use web_sys::MediaStreamTrack;
use web_sys::MediaStreamTrackProcessor;
use web_sys::MediaStreamTrackProcessorInit;
use web_sys::ReadableStream;
use web_sys::VideoTrack;

use super::device_state::DeviceState;
use super::ReadableType;
use super::Sender;

#[derive(Clone, PartialEq)]
pub struct CameraDevice {
    video_elem_id: String,
    state: DeviceState,
    media_track: Option<MediaStreamTrack>,
    readable_stream: Option<ReadableStream>,
}

impl CameraDevice {

    pub fn new(video_elem_id: &str) -> Self {
        Self {
            video_elem_id: video_elem_id.to_string(),
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

    pub fn select(&mut self, device_id: String) -> bool {
        self.state.select(device_id)
    }

    pub fn get_enabled(&self) -> bool {
        self.state.is_enabled()
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

    pub fn start(&mut self, sender: Box<dyn Sender>)  {
        self.stop();
        let video_elem_id = self.video_elem_id.clone();
        let device_id = if let Some(vid) = &self.state.selected {
            vid.to_string()
        } else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            let navigator = window().navigator();
            let video_element = window()
                .document()
                .unwrap()
                .get_element_by_id(&video_elem_id)
                .unwrap()
                .unchecked_into::<HtmlVideoElement>();

            let media_devices = navigator.media_devices().unwrap();
            let constraints = MediaStreamConstraints::new();
            let media_info = web_sys::MediaTrackConstraints::new();
            media_info.set_device_id(&device_id.into());

            constraints.set_video(&media_info.into());
            constraints.set_audio(&Boolean::from(false));

            let devices_query = media_devices
                .get_user_media_with_constraints(&constraints)
                .unwrap();
            let device = JsFuture::from(devices_query)
                .await
                .unwrap()
                .unchecked_into::<MediaStream>();
            video_element.set_src_object(Some(&device));
            video_element.set_muted(true);

            let video_track = Box::new(
                device
                    .get_video_tracks()
                    .find(&mut |_: JsValue, _: u32, _: Array| true)
                    .unchecked_into::<VideoTrack>(),
            );

            let video_processor =
                MediaStreamTrackProcessor::new(&MediaStreamTrackProcessorInit::new(
                    &video_track.clone().unchecked_into::<MediaStreamTrack>(),
                ))
                .unwrap();
            let video_reader = video_processor.readable();
            let media_track = video_track
                .clone()
                .unchecked_into::<MediaStreamTrack>();
            sender.send_readable(ReadableType::Video, video_reader, media_track);
        });
    }
}
