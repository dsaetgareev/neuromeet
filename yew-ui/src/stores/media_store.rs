use std::rc::Rc;
use gloo_timers::callback::Timeout;
use videocall_client::{configure_audio_stream, configure_video_stream, CameraDevice, MediaDeviceAccess, MicrophoneEncoder, ReadableType, ScreenEncoder};
use web_sys::{MediaStreamTrack, ReadableStream};
// use yewdux::{Dispatch, Reducer, Store};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::agents::{ConnectionAgent, Peer};

const VIDEO_ELEMENT_ID: &str = "webcam";

#[derive(Clone, PartialEq, Store)]
pub struct MediaStore {
    rerender: bool,
    camera: CameraDevice,
    screen: ScreenEncoder,
    microphone: MicrophoneEncoder,
    media_device_access: Option<MediaDeviceAccess>,
    is_device_access: bool,
    pub is_connected: bool,
    is_agent_started: bool,
    is_screen_share: bool,
    connection_agent: ConnectionAgent,
}

impl Default for MediaStore {
    fn default() -> Self {
        let connection_agent = ConnectionAgent::new();
        Self {
            rerender: Default::default(),
            camera: CameraDevice::new(VIDEO_ELEMENT_ID),
            screen: ScreenEncoder::new(),
            microphone: MicrophoneEncoder::new(),
            media_device_access: Self::create_media_device_access(),
            is_device_access: false,
            is_connected: false,
            is_agent_started: false,
            is_screen_share: Default::default(),
            connection_agent,
        }
    }
}

impl MediaStore {
    pub fn rerender(&mut self) {
        let rerender = self.rerender;
        self.rerender = !rerender;
    }

    pub fn get_agent(&self) -> &ConnectionAgent {
        &self.connection_agent
    }
    pub fn get_mut_agent(&mut self) -> &mut ConnectionAgent {
        &mut self.connection_agent
    }
    pub fn get_camera(&self) -> &CameraDevice {
        &self.camera
    }

    pub fn get_mut_camera(&mut self) -> &mut CameraDevice {
        &mut self.camera
    }

    pub fn get_mic(&self) -> &MicrophoneEncoder {
        &self.microphone
    }

    pub fn get_mut_mic(&mut self) -> &mut MicrophoneEncoder {
        &mut self.microphone
    }

    pub fn get_screen(&self) -> &ScreenEncoder {
        &self.screen
    }

    pub fn get_mut_screen(&mut self) -> &mut ScreenEncoder {
        &mut self.screen
    }

    pub fn get_media_device_access(&self) -> &MediaDeviceAccess {
        &self.media_device_access.as_ref().unwrap() 
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
    }
    pub fn is_agent_started(&self) -> bool {
        self.is_agent_started
    }

    pub fn set_agent_started(&mut self, started: bool) {
        self.is_agent_started = started;
    }

    pub fn is_screen_share(&self) -> bool {
        self.is_screen_share
    }

    fn create_media_device_access() -> Option<MediaDeviceAccess> {
        let mut media_device_access = MediaDeviceAccess::new();
        media_device_access.on_granted = {
            Callback::from(move |_| {
                log::info!("on granted11111");
            })
        };
        media_device_access.on_denied = {
            Callback::from(move |_| {
                log::info!("on denied");
            })
        };
        Some(media_device_access)
    }

}

pub enum MediaMsg {
    Rerender,
    Connect,
    SetConnected(bool),
    ClientConfigure(String, String),
    AgentInit,
    AddPeer(String),
    SetMediaTrack(ReadableType, ReadableStream, MediaStreamTrack),
    // divices
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    SwitchMic(bool),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    SwitchVedeo(bool),
    EnableScreenShare,
    DisableScreenShare,
}


impl Reducer<MediaStore> for MediaMsg {
    fn apply(self, mut store: Rc<MediaStore>) -> Rc<MediaStore> {
        let state = Rc::make_mut(&mut store); 
        let dispatch = Dispatch::<MediaStore>::global();
        match self {
            MediaMsg::Rerender => {
                state.rerender();
            },
            MediaMsg::Connect => {
                state.get_agent().connect();

            },
            MediaMsg::SetConnected(is_connected) => {
                state.set_connected(is_connected);
            }
            MediaMsg::ClientConfigure(user_name, meeting_id) => {
                state.get_mut_agent().configure("s_worker", user_name, meeting_id, dispatch);
            },
            MediaMsg::AgentInit => {
                state.set_agent_started(true);
                state.get_mut_agent().init();
            },
            MediaMsg::AddPeer(peer_id) => {

                let audio_stream_generator = configure_audio_stream();
                let audio_writable = audio_stream_generator.writable();

                let (video_media_stream, video_media_stream_generator) = configure_video_stream();
                let video_writable = video_media_stream_generator.writable();

                let (screen_media_stream, screen_media_stream_generator) = configure_video_stream();
                let screen_writable = screen_media_stream_generator.writable();
                
                let peer = Peer::new(peer_id.clone(), video_media_stream, screen_media_stream);

                state.get_mut_agent().get_mut_peers().insert(peer_id.clone(), peer);

                state.get_mut_agent().send_writables(peer_id.clone(), audio_writable, video_writable, screen_writable);


            },
            MediaMsg::SetMediaTrack(readable_type, readable_stream, media_track, ) => {
                match readable_type {
                    ReadableType::Audio => {
                        state.get_mut_mic().set_media_track(readable_stream, media_track);                        
                    },
                    ReadableType::Video => {
                        state.get_mut_camera().set_media_track(readable_stream, media_track); 
                    },
                    ReadableType::Screen => {
                        state.get_mut_screen().set_media_track(readable_stream, media_track);
                    },
                }
            },
            
            MediaMsg::AudioDeviceChanged(audio) => {
                if state.microphone.select(audio) {
                    state.get_agent().stop_encoder(ReadableType::Audio);
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::EnableMicrophone(should_enable) => {
                if should_enable {
                    if state.get_agent().is_start() {
                        log::info!("mic start");
                        let ws_agent = state.get_agent();
                        let agent = ws_agent.clone();
                        state.get_mut_mic().start(Box::new(agent));
                    }
                }            
            },
            MediaMsg::SwitchMic(_) => {
                if state.get_mut_mic().switch_enabled() {
                    state.get_mut_mic().stop();
                    state.get_agent().stop_encoder(ReadableType::Audio);
                } else {
                    let agent = state.get_agent();
                    let agent = agent.clone();
                    state.get_mut_mic().start(Box::new(agent));
                }
            },
            MediaMsg::VideoDeviceChanged(video) => {
                if state.camera.select(video) {
                    state.get_agent().stop_encoder(ReadableType::Video);
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableVideo(true));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::EnableVideo(should_enable) => {
                if should_enable {
                    if state.is_agent_started() {
                        log::info!("video start");
                        let agent = state.get_agent();
                        let agent = agent.clone();
                        state.get_mut_camera().start(Box::new(agent));
                    }
                }
            },
            MediaMsg::SwitchVedeo(_) => {
                if state.get_mut_camera().switch_enabled() {
                    state.get_mut_camera().stop();
                    state.get_agent().stop_encoder(ReadableType::Video);
                } else {
                    let agent = state.get_agent();
                    let agent = agent.clone();
                    state.get_mut_camera().start(Box::new(agent));
                }
            },
            MediaMsg::EnableScreenShare => {
                let ws_agent = state.get_mut_agent();
                let agent = ws_agent.clone();
                state.is_screen_share = true; 
                state.screen.start(Box::new(agent));
            },
            MediaMsg::DisableScreenShare => {
                state.is_screen_share = false;
                state.screen.stop();
                state.get_agent().stop_encoder(ReadableType::Screen);
            },
        }
        store
    }
}

