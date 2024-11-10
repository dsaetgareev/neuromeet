use crate::components::{canvas_generator, peer_list::PeerList};
use crate::constants::{CANVAS_LIMIT, USERS_ALLOWED_TO_STREAM, VIDEO_ELEMENT_ID, WEBTRANSPORT_HOST};
use crate::stores::media_store::{MediaMsg, MediaStore};
use crate::{components::host::Host, constants::ACTIX_WEBSOCKET};
use log::{error, warn};
use types::protos::media_packet::media_packet::MediaType;
use videocall_client::{MediaDeviceAccess, VideoCallClient, VideoCallClientOptions};
use wasm_bindgen::JsValue;
use web_sys::*;
use yew::prelude::*;
use yew::{html, Component, Context, Html};
use yewdux::use_store;
use crate::components::{Devices, VideoButton};

#[derive(Debug)]
pub enum WsAction {
    Connect,
    Connected,
    Lost(Option<JsValue>),
    RequestMediaPermissions,
    MediaPermissionsGranted,
    MediaPermissionsError(String),
    Log(String),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum MeetingAction {
    ToggleScreenShare,
    ToggleMicMute,
    ToggleVideoOnOff,
}

#[derive(Debug)]
pub enum UserScreenAction {
    TogglePeerList,
}

#[derive(Debug)]
pub enum Msg {
    WsAction(WsAction),
    MeetingAction(MeetingAction),
    OnPeerAdded(String),
    OnFirstFrame((String, MediaType)),
    UserScreenAction(UserScreenAction),
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WsAction(action)
    }
}

impl From<UserScreenAction> for Msg {
    fn from(action: UserScreenAction) -> Self {
        Msg::UserScreenAction(action)
    }
}

impl From<MeetingAction> for Msg {
    fn from(action: MeetingAction) -> Self {
        Msg::MeetingAction(action)
    }
}

#[derive(Properties, Debug, PartialEq)]
pub struct AttendantsComponentProps {
    #[prop_or_default]
    pub id: String,

    #[prop_or_default]
    pub email: String,

    pub e2ee_enabled: bool,

    pub webtransport_enabled: bool,
}

pub struct AttendantsComponent {
    pub client: VideoCallClient,
    pub media_device_access: MediaDeviceAccess,
    pub share_screen: bool,
    pub mic_enabled: bool,
    pub video_enabled: bool,
    pub peer_list_open: bool,
    pub error: Option<String>,
}

impl AttendantsComponent {
    fn create_video_call_client(ctx: &Context<Self>) -> VideoCallClient {
        let email = ctx.props().email.clone();
        // let email = uuid::Uuid::new_v4().to_string();
        let id = ctx.props().id.clone();
        let opts = VideoCallClientOptions {
            userid: email.clone(),
            websocket_url: format!("{ACTIX_WEBSOCKET}/{email}/{id}"),
            webtransport_url: format!("{WEBTRANSPORT_HOST}/{email}/{id}"),
            enable_e2ee: false,
            enable_webtransport: true,
            on_connected: {
                let link = ctx.link().clone();
                Callback::from(move |_| link.send_message(Msg::from(WsAction::Connected)))
            },
            on_connection_lost: {
                let link = ctx.link().clone();
                Callback::from(move |_| link.send_message(Msg::from(WsAction::Lost(None))))
            },
            on_peer_added: {
                let link = ctx.link().clone();
                Callback::from(move |email| link.send_message(Msg::OnPeerAdded(email)))
            },
            on_peer_first_frame: {
                let link = ctx.link().clone();
                Callback::from(move |(email, media_type)| {
                    link.send_message(Msg::OnFirstFrame((email, media_type)))
                })
            },
            get_peer_video_canvas_id: Callback::from(|email| email),
            get_peer_screen_canvas_id: Callback::from(|email| format!("screen-share-{}", &email)),
        };
        VideoCallClient::new(opts)
    }

    fn create_media_device_access(ctx: &Context<Self>) -> MediaDeviceAccess {
        let mut media_device_access = MediaDeviceAccess::new();
        media_device_access.on_granted = {
            let link = ctx.link().clone();
            Callback::from(move |_| link.send_message(WsAction::MediaPermissionsGranted))
        };
        media_device_access.on_denied = {
            let link = ctx.link().clone();
            Callback::from(move |_| {
                link.send_message(WsAction::MediaPermissionsError("Error requesting permissions. Please make sure to allow access to both camera and microphone.".to_string()))
            })
        };
        media_device_access
    }
}

impl Component for AttendantsComponent {
    type Message = Msg;
    type Properties = AttendantsComponentProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            client: Self::create_video_call_client(ctx),
            media_device_access: Self::create_media_device_access(ctx),
            share_screen: false,
            mic_enabled: true,
            video_enabled: true,
            peer_list_open: false,
            error: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(WsAction::RequestMediaPermissions);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("AttendantsComponent update: {:?}", msg);
        match msg {
            Msg::WsAction(action) => match action {
                WsAction::Connect => {
                    if self.client.is_connected() {
                        return false;
                    }
                    if let Err(e) = self.client.connect() {
                        ctx.link()
                            .send_message(WsAction::Log(format!("Connection failed: {e}")));
                    }
                    log::info!("Connected in attendants");
                    true
                }
                WsAction::Connected => true,
                WsAction::Log(msg) => {
                    warn!("{}", msg);
                    false
                }
                WsAction::Lost(reason) => {
                    warn!("Lost with reason {:?}", reason);
                    ctx.link().send_message(WsAction::Connect);
                    true
                }
                WsAction::RequestMediaPermissions => {
                    self.media_device_access.request();
                    ctx.link().send_message(WsAction::Connect);
                    false
                }
                WsAction::MediaPermissionsGranted => {
                    self.error = None;
                    ctx.link().send_message(WsAction::Connect);
                    true
                }
                WsAction::MediaPermissionsError(error) => {
                    self.error = Some(error);
                    true
                }
            },
            Msg::OnPeerAdded(_email) => true,
            Msg::OnFirstFrame((_email, media_type)) => matches!(media_type, MediaType::SCREEN),
            Msg::MeetingAction(action) => {
                match action {
                    MeetingAction::ToggleScreenShare => {
                        self.share_screen = !self.share_screen;
                    }
                    MeetingAction::ToggleMicMute => {
                        self.mic_enabled = !self.mic_enabled;
                    }
                    MeetingAction::ToggleVideoOnOff => {
                        self.video_enabled = !self.video_enabled;
                    }
                }
                true
            }
            Msg::UserScreenAction(action) => {
                match action {
                    UserScreenAction::TogglePeerList => {
                        self.peer_list_open = !self.peer_list_open;
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let email = ctx.props().email.clone();
        let media_access_granted = self.media_device_access.is_granted();

        let toggle_peer_list = ctx.link().callback(|_| UserScreenAction::TogglePeerList);

        let peers = self.client.sorted_peer_keys();
        let _rows = canvas_generator::generate(
            &self.client,
            peers.iter().take(CANVAS_LIMIT).cloned().collect(),
        );

        html! {
            <div id="main-container">
                <div id="grid-container" style={if self.peer_list_open {"width: 80%;"} else {"width: 100%;"}}>
                    { self.error.as_ref().map(|error| html! { <p>{ error }</p> }) }
                    // { rows }
                    {
                        if USERS_ALLOWED_TO_STREAM.iter().any(|host| host == &email) || USERS_ALLOWED_TO_STREAM.is_empty() {
                            html! {
                                <nav class="host">
                                    <div class="controls">
                                        <button
                                            class="bg-yew-blue p-2 rounded-md text-white"
                                            onclick={ctx.link().callback(|_| MeetingAction::ToggleScreenShare)}>
                                            { if self.share_screen { "Stop Screen Share"} else { "Share Screen"} }
                                        </button>
                                        <button
                                            class="bg-yew-blue p-2 rounded-md text-white"
                                            onclick={ctx.link().callback(|_| MeetingAction::ToggleVideoOnOff)}>
                                            { if !self.video_enabled { "Start Video"} else { "Stop Video"} }
                                        </button>
                                        <button
                                            class="bg-yew-blue p-2 rounded-md text-white"
                                            onclick={ctx.link().callback(|_| MeetingAction::ToggleMicMute)}>
                                            { if !self.mic_enabled { "Unmute"} else { "Mute"} }
                                        </button>
                                        <button
                                            class="bg-yew-blue p-2 rounded-md text-white"
                                            onclick={toggle_peer_list.clone()}>
                                            { if !self.peer_list_open { "Open Peers"} else { "Close Peers"} }
                                        </button>
                                    </div>
                                    {
                                        if media_access_granted {
                                            html! {<Host client={self.client.clone()} share_screen={self.share_screen} mic_enabled={self.mic_enabled} video_enabled={self.video_enabled} />}
                                        } else {
                                            html! {<></>}
                                        }
                                    }
                                    <h4 class="floating-name">{email}</h4>

                                    {if !self.client.is_connected() {
                                        html! {<h4>{"Connecting"}</h4>}
                                    } else {
                                        html! {<h4>{"Connected"}</h4>}
                                    }}

                                    // {if ctx.props().e2ee_enabled {
                                    //     html! {<h4>{"End to End Encryption Enabled"}</h4>}
                                    // } else {
                                    //     html! {<h4>{"End to End Encryption Disabled"}</h4>}
                                    // }}
                                </nav>
                            }
                        } else {
                            error!("User not allowed to stream");
                            error!("allowed users {}", USERS_ALLOWED_TO_STREAM.join(", "));
                            html! {}
                        }
                    }
                </div>
                <div id="peer-list-container" class={if self.peer_list_open {"visible"} else {""}}>
                    <PeerList peers={peers} onclose={toggle_peer_list} />
                </div>
            </div>
        }
    }
}

#[function_component(AttendantsFunc)]
pub fn attendats_func() -> Html {

    let (media_state, media_dispatch) = use_store::<MediaStore>();
    let camera_enabled = use_state(|| media_state.get_camera().get_enabled());
    let mic_enabled = use_state(|| media_state.get_mic().get_enabled());
    let ws_client = media_state.get_client();
    let user_name = ws_client.userid();
    let peers = media_state.get_client().sorted_peer_keys();
    let rows = || {
        
        peers.iter()
        .map(|key| {
            html! {
                <>
                    <VideoComponent key_id={ key.clone() } />
                </>
            }
        }).collect::<Html>()
    };

    use_effect({
        let media_dispatch = media_dispatch.clone();
        move || {
            media_dispatch.apply(MediaMsg::Connect);
        }
    });
    html! {
            <div id="main-container">
                <div id="grid-container" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 w-full max-w-6xl">
                    <div class="bg-gray-700 shadow-2xl rounded-xl p-4 flex flex-col items-center">
                        <video class="rounded-lg w-32 h-32 mb-2" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                    </div>
                    { rows() }
                </div>
                <nav class="host">
                    // <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                    <Devices />
                    <VideoButton />
                    <h4 class="floating-name">{(*user_name).clone()}</h4>

                    {if !media_state.is_connected() {
                        html! {<h4>{"Connecting"}</h4>}
                    } else {
                        html! {<h4>{"Connected"}</h4>}
                    }}

                    // {if ctx.props().e2ee_enabled {
                    //     html! {<h4>{"End to End Encryption Enabled"}</h4>}
                    // } else {
                    //     html! {<h4>{"End to End Encryption Disabled"}</h4>}
                    // }}
                </nav>
            </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ItemPorps {
    pub key_id: String,
}


#[function_component(VideoComponent)]
pub fn video_component(props: &ItemPorps) -> Html {
    let key = &props.key_id;
    log::info!("key {}", key);
    
    let (media_state, media_dispatch) = use_store::<MediaStore>();
    let video_ref = use_node_ref();
    use_effect({
        let video_ref = video_ref.clone();
        let media_stream = media_state.get_client().get_media_stream_by_key(key);
        move || {
            if let Some(video_element) = video_ref.cast::<HtmlVideoElement>() {
                if let Some(stream) = media_stream {
                    video_element.set_src_object(Some(&stream));
                }
            }
        }
    });

    html! {
        <div class="bg-gray-700 shadow-2xl rounded-xl p-4 flex flex-col items-center">
            <p>{ key.clone() }</p>
            <video class="rounded-lg w-32 h-32 mb-2" ref={video_ref} autoplay=true />
        </div> 
    }
}