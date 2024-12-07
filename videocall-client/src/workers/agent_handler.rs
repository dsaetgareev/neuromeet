use std::{cell::RefCell, rc::Rc};

use types::protos::packet_wrapper::PacketWrapper;
use web_sys::{DedicatedWorkerGlobalScope, ReadableStream, WritableStream};
use yew::Callback;
use serde::{Deserialize, Serialize};

use crate::{constants::{ACTIX_WEBSOCKET, WEBTRANSPORT_HOST}, encode::{configure_audio_encoder, configure_camera_encoder, configure_screen_encoder, EncoderHandler}, ReadableType, VideoCallClient, VideoCallClientOptions};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum AgentMsg {
    Start,
    OnConnected,
    OnConnectionLost,
    OnPeerAdded {peer_id: String},
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ConnectionAgentMsg {
    Init {user_name: String, meeting_id: String, origin_url: String},
    Connect,
    StopEncoder {readable_type: ReadableType},
}


pub struct AgentHandler {
    client: Option<VideoCallClient>,
    scope: DedicatedWorkerGlobalScope,
    encoder_handler: Rc<RefCell<EncoderHandler>>,
}

impl AgentHandler {

    pub fn new(scope: DedicatedWorkerGlobalScope) -> Self {
        Self { 
            client: None,
            scope,
            encoder_handler: Rc::new(RefCell::new(EncoderHandler::new())),
        }
    }

    pub fn get_client(&self) -> &VideoCallClient {
        &self.client.as_ref().unwrap()
    }
    pub fn get_mut_client(&mut self) -> &mut VideoCallClient {
        self.client.as_mut().unwrap()
    }

    pub fn init(&mut self, user_name: String, meeting_id: String, origin_url: String) {
        let scope = &self.scope;
        let client = create_video_call_client(user_name, meeting_id, origin_url, scope.clone());
        self.client = Some(client);
        self.connect();
    }
    
    pub fn connect(&mut self) {
        if !self.get_mut_client().is_connected() {
            match self.get_mut_client().connect() {
                Ok(_) => {

                },
                Err(_) => {
                    log::error!("connected field");
                },
            }
            log::info!("Connected WT to server");
        }    
    }

    pub fn set_media(&self, peer_id: String, audio_ws: WritableStream, video_ws: WritableStream, screen_ws: WritableStream) {
        self.get_client().set_media(peer_id, audio_ws, video_ws, screen_ws);
    }

    pub fn configure_encoder(&self, readable_type: ReadableType, readable: ReadableStream) {
        let ws_client = self.get_client().clone();
        let user_id = ws_client.userid().to_string();
        let aes = ws_client.aes();
        let on_frame = move |chunk: PacketWrapper| {
            ws_client.send_packet(chunk);                        
        };
        let encoder_handler = self.encoder_handler.clone();
        match readable_type {
            ReadableType::Audio => {
                configure_audio_encoder(readable, on_frame, user_id, aes, encoder_handler);                
            },
            ReadableType::Video => {
                configure_camera_encoder(readable, on_frame, user_id, aes, encoder_handler);
            },
            ReadableType::Screen => {
                configure_screen_encoder(readable, on_frame, user_id, aes, encoder_handler);
            },
        }
    }

    pub fn stop_encoder(&self, readable_type: ReadableType) {
        match readable_type {
            ReadableType::Audio => {
                self.encoder_handler.as_ref().borrow_mut().audio_close();                
            },
            ReadableType::Video => {
               self.encoder_handler.as_ref().borrow_mut().video_close();
            },
            ReadableType::Screen => {
                self.encoder_handler.as_ref().borrow_mut().screen_close();
            },
        }
    }


}

fn create_video_call_client(user_name: String, meeting_id: String, origin_url: String, scope: DedicatedWorkerGlobalScope) -> VideoCallClient {
    let opts = VideoCallClientOptions {
        origin_url,
        userid: user_name.clone(),
        websocket_url: format!("{ACTIX_WEBSOCKET}/{user_name}/{meeting_id}"),
        webtransport_url: format!("{WEBTRANSPORT_HOST}/{user_name}/{meeting_id}"),
        enable_e2ee: false,
        enable_webtransport: true,
        on_connected: {
            let scope = scope.clone();
            Callback::from(move |_| {
                let message = AgentMsg::OnConnected;
                let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
                let _ = &scope.clone()
                    .post_message(&js_value)
                    .expect("posting ready message succeeds");
            })
        },
        on_connection_lost: {
            let scope = scope.clone();
            Callback::from(move |_| {
                let message = AgentMsg::OnConnectionLost;
                let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
                let _ = &scope.clone()
                    .post_message(&js_value)
                    .expect("posting ready message succeeds");
            })
        },
        on_peer_added: {
            let scope = scope.clone();
            Callback::from(move |peer_id: String| {
                let message = AgentMsg::OnPeerAdded {peer_id};
                let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
                let _ = &scope.clone()
                    .post_message(&js_value)
                    .expect("posting ready message succeeds");
            })
        },
        on_peer_first_frame: {
            Callback::from(move |_| {

            })
        },
        get_peer_video_canvas_id: Callback::from(|email| email),
        get_peer_screen_canvas_id: Callback::from(|email| format!("screen-share-{}", &email)),
    };
    VideoCallClient::new(opts)
}