use std::rc::Rc;
use gloo_timers::callback::Timeout;
use types::protos::packet_wrapper::PacketWrapper;
use videocall_client::{request_permissions, CameraEncoder, MediaDeviceAccess, MicrophoneEncoder, VideoCallClient, VideoCallClientOptions};
// use yewdux::{Dispatch, Reducer, Store};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{components::{host::_MeetingProps::{client, video_enabled}, middleware::Middleware}, constants::{ACTIX_WEBSOCKET, WEBTRANSPORT_HOST}};

const VIDEO_ELEMENT_ID: &str = "webcam";

#[derive(Clone, PartialEq, Store)]
pub struct MediaStore {
    rerender: bool,
    client: Option<VideoCallClient>,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
    media_device_access: Option<MediaDeviceAccess>,
    is_device_access: bool,
    pub is_connected: bool,
}

impl Default for MediaStore {
    fn default() -> Self {
        Self {
            rerender: Default::default(),
            client: None,
            camera: CameraEncoder::new(VIDEO_ELEMENT_ID),
            microphone: MicrophoneEncoder::new(),
            media_device_access: Self::create_media_device_access(),
            is_device_access: false,
            is_connected: false,
        }
    }
}

impl MediaStore {
    pub fn rerender(&mut self) {
        let rerender = self.rerender;
        self.rerender = !rerender;
    }
    pub fn get_client(&self) -> &VideoCallClient {
        &self.client.as_ref().unwrap()
    }
    pub fn get_mut_client(&mut self) -> &mut VideoCallClient {
        self.client.as_mut().unwrap()
    }
    pub fn get_camera(&self) -> &CameraEncoder {
        &self.camera
    }

    pub fn get_mut_camera(&mut self) -> &mut CameraEncoder {
        &mut self.camera
    }

    pub fn get_mic(&self) -> &MicrophoneEncoder {
        &self.microphone
    }

    pub fn get_mut_mic(&mut self) -> &mut MicrophoneEncoder {
        &mut self.microphone
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

    fn create_video_call_client(&mut self, user_name: String, meeting_id: String, dispatch: Dispatch<MediaStore>) -> VideoCallClient {
        let opts = VideoCallClientOptions {
            userid: user_name.clone(),
            websocket_url: format!("{ACTIX_WEBSOCKET}/{user_name}/{meeting_id}"),
            webtransport_url: format!("{WEBTRANSPORT_HOST}/{user_name}/{meeting_id}"),
            enable_e2ee: false,
            enable_webtransport: true,
            on_connected: {
                let dispatch = dispatch.clone();
                Callback::from(move |_| {
                    dispatch.apply(MediaMsg::SetConnected(true));
                })
            },
            on_connection_lost: {
                // let link = ctx.link().clone();
                Callback::from(move |_| {
                    // link.send_message(Msg::from(WsAction::Lost(None))
                })
            },
            on_peer_added: {
                let dispatch = dispatch.clone();
                Callback::from(move |_| {
                    log::info!("rerererererer");
                    dispatch.apply(MediaMsg::Rerender);
                })
            },
            on_peer_first_frame: {
                Callback::from(move |(email, media_type)| {

                })
            },
            get_peer_video_canvas_id: Callback::from(|email| email),
            get_peer_screen_canvas_id: Callback::from(|email| format!("screen-share-{}", &email)),
        };
        VideoCallClient::new(opts)
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
    MediaDeviceAccessRequest,
    ClientInit(String, String),
    AudioDeviceInit(String),
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    SwitchMic(bool),
    VideoDeviceInit(String),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    SwitchVedeo(bool),
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
                if !state.get_mut_client().is_connected() {
                    match state.get_mut_client().connect() {
                        Ok(_) => {

                        },
                        Err(_) => {
                            log::error!("connected field");
                        },
                    }
                    log::info!("Connected in attendants");
                }
            },
            MediaMsg::SetConnected(is_connected) => {
                state.set_connected(is_connected);
            }
            MediaMsg::MediaDeviceAccessRequest => {
                let future = request_permissions();
            },
            MediaMsg::ClientInit(user_name, meeting_id) => {
                state.client = Some(state.create_video_call_client(user_name, meeting_id, dispatch));
            },
            MediaMsg::AudioDeviceInit(_) => todo!(),
            MediaMsg::AudioDeviceChanged(audio) => {
                if state.microphone.select(audio) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::EnableMicrophone(should_enable) => {
                if should_enable && state.get_media_device_access().is_granted() {
                    if state.client.is_some() {
                        let ws_client = state.client.as_ref().unwrap().clone();
                        let user_id = ws_client.userid().to_string();
                        let aes = ws_client.aes();
                        let on_audio = move |chunk: PacketWrapper| {
                            ws_client.send_packet(chunk);                        
                        }; 
                        state.get_mut_mic().start(on_audio, user_id, aes);
                        dispatch.apply(MediaMsg::Connect);
                    } else {
                        state.microphone.run(); 
                    }
                }               
            },
            MediaMsg::SwitchMic(_) => {
                let on_mic= state.get_mic().get_enabled();
                log::info!("on video {}", on_mic);
                if state.get_mut_mic().set_enabled(!on_mic) {
                    state.get_mut_mic().run();
                }
            },
            MediaMsg::VideoDeviceInit(_) => todo!(),
            MediaMsg::VideoDeviceChanged(video) => {
                if state.camera.select(video) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableVideo(true));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::EnableVideo(should_enable) => {
                if should_enable {
                    if state.client.is_some() {
                        
                        let ws_client = state.get_client().clone();
                        let user_id = ws_client.userid().to_string();
                        let aes = ws_client.aes();
                        let on_video = move |chunk: PacketWrapper| {
                            ws_client.send_packet(chunk);                        
                        };  
                        state.get_mut_camera().start(on_video, user_id, aes);
                        dispatch.apply(MediaMsg::Connect);
                    } else {
                        state.get_mut_camera().run();
                    }
                }
            },
            MediaMsg::SwitchVedeo(_) => {
                let on_video= state.get_camera().get_enabled();
                log::info!("on video {}", on_video);
                if state.get_mut_camera().set_enabled(!on_video) {
                    state.get_mut_camera().run();
                }
            },
        }
        store
    }
}

