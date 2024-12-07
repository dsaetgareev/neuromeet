use std::{cell::RefCell, collections::HashMap, rc::Rc, str};
use videocall_client::{workers::{worker_new, AgentMsg, ConnectionAgentMsg}, ReadableType, Sender};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{MediaStreamTrack, MessageEvent, Worker, WritableStream};
use yewdux::Dispatch;

use crate::{stores::media_store::{MediaMsg, MediaStore}, utils::dom::get_origin_url};

use super::Peer;



#[derive(Clone, PartialEq, Debug)]
pub struct ConnectionAgent {
    worker: Option<Worker>,
    is_start: Rc<RefCell<bool>>,
    dispatch: Option<Rc<RefCell<Dispatch<MediaStore>>>>,
    user_id: String,
    meeting_id: String,
    peers: HashMap<String, Peer>
}

impl ConnectionAgent {
    pub fn new() -> Self {
        Self {
            worker: None,
            is_start: Rc::new(RefCell::new(false)),
            dispatch: None,
            user_id: String::default(),
            meeting_id: String::default(),
            peers: HashMap::new(),
        }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_peers(&self) -> &HashMap<String, Peer> {
        &self.peers
    }

    pub fn get_mut_peers(&mut self) -> &mut HashMap<String, Peer> {
        &mut self.peers
    }

    pub fn is_start(&self) -> bool {
        self.is_start.borrow().clone()
    }

    pub fn configure(&mut self, url: &str, user_name: String, meeting_id: String, dispatch: Dispatch<MediaStore>) {
        self.user_id = user_name;
        self.meeting_id = meeting_id;
        let worker = worker_new(None, url);
        self.worker = Some(worker.clone());
        let is_start = self.is_start.clone();
        self.dispatch = Some(Rc::new(RefCell::new(dispatch)));
        let dispatch = self.dispatch.clone();
        let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
            let dispatch = dispatch.clone();
            let data: JsValue = msg.data();
            web_sys::console::log_1(&data);
            let message: AgentMsg = serde_wasm_bindgen::from_value(data).unwrap();
            match message {
                AgentMsg::Start => {
                    web_sys::console::log_1(&"Start".into());
                    is_start.replace(true);
                    if let Some(dispatch) = dispatch {
                        dispatch.borrow().apply(MediaMsg::AgentInit);
                    }
                },
                AgentMsg::OnConnected => {
                    if let Some(dispatch) = dispatch {
                        dispatch.borrow().apply(MediaMsg::SetConnected(true));
                    }
                },
                AgentMsg::OnConnectionLost => {
                    if let Some(dispatch) = dispatch {
                        dispatch.borrow().apply(MediaMsg::Connect);
                    }
                },
                AgentMsg::OnPeerAdded {peer_id} => {
                    
                    
                    if let Some(dispatch) = dispatch {
                        dispatch.borrow().apply(MediaMsg::AddPeer(peer_id));
                    }
                },
            }
    
        }) as Box<dyn Fn(MessageEvent)>);
    
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    }

    pub fn init(&mut self) {
        let user_name = &self.user_id;
        let meeting_id = &self.meeting_id;
        let origin_url = get_origin_url();
        let message = ConnectionAgentMsg::Init { user_name: user_name.to_string(), meeting_id: meeting_id.to_string(), origin_url };
        let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
        self.worker.as_ref().expect("cannot get worker").post_message(&js_value).expect("cannot post message");
    }

    pub fn connect(&self) {
        let message = ConnectionAgentMsg::Connect;
        let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
        self.worker.as_ref().expect("cannot get worker").post_message(&js_value).expect("cannot post message");
    }

    pub fn send_writables(&self, peer_id: String, audio_ws: WritableStream, video_ws: WritableStream, screen_ws: WritableStream) {
        if video_ws.locked() {
            web_sys::console::log_1(&"im locked".into());
            return;
        }
       // Обернуть `WritableStream` в `JsValue`
       let peer_id_js = JsValue::from(peer_id.clone());
       let message = js_sys::Object::new();
        js_sys::Reflect::set(&message, &"peerId".into(), &peer_id_js).unwrap();
       let audio_writable_stream_js = JsValue::from(audio_ws.clone());
       let video_writable_stream_js = JsValue::from(video_ws.clone());
       let screen_writable_stream_js = JsValue::from(screen_ws.clone());

       let arr = js_sys::Array::new();
       arr.push(&message);
       arr.push(&audio_writable_stream_js); 
       arr.push(&video_writable_stream_js); 
       arr.push(&screen_writable_stream_js); 

       // Создать массив для передачи transferable-объектов
       let transferables = js_sys::Array::new();
    //    transferables.push(&message);
       transferables.push(&audio_writable_stream_js); 
       transferables.push(&video_writable_stream_js); 
       transferables.push(&screen_writable_stream_js); 

        let _ = self.worker.as_ref().expect("cannot get worker").post_message_with_transfer(&arr, &transferables).expect("sending message error");
    }

    pub fn send_readables(&self, readable_type: ReadableType, readable: web_sys::ReadableStream, media_track: MediaStreamTrack) {
        if let Some(dispatch) = &self.dispatch {
            dispatch.borrow().apply(MediaMsg::SetMediaTrack(readable_type.clone(), readable.clone(), media_track));
        }
        let readable_type_js = serde_wasm_bindgen::to_value(&readable_type).unwrap();

        let readable_js = JsValue::from(readable.clone());

        let message_arr = js_sys::Array::new();
        message_arr.push(&readable_type_js);
        message_arr.push(&readable_js);
 
        let transferables = js_sys::Array::new();
        transferables.push(&readable_js);
        let _ = self.worker.as_ref().expect("cannot get worker").post_message_with_transfer(&message_arr, &transferables).expect("sending message error");
    }

    pub fn stop_encoder(&self, readable_type: ReadableType) {
        let message = ConnectionAgentMsg::StopEncoder { readable_type: readable_type };
        let js_value = serde_wasm_bindgen::to_value(&message).unwrap();
        self.worker.as_ref().expect("cannot get worker").post_message(&js_value).expect("cannot post message");
    }
}

impl Sender for ConnectionAgent {
    fn send_readable(&self, readable_type: ReadableType, readable: web_sys::ReadableStream, media_track: MediaStreamTrack) {
        self.send_readables(readable_type, readable, media_track);
    }
}