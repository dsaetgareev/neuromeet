use crate::constants::VIDEO_ELEMENT_ID;
use crate::stores::media_store::{MediaMsg, MediaStore};
use web_sys::*;
use yew::prelude::*;
use yew::{html, Html};
use yewdux::use_store;
use crate::components::{Devices, VideoButton};

#[function_component(AttendantsFunc)]
pub fn attendats_func() -> Html {

    let (media_state, media_dispatch) = use_store::<MediaStore>();
    let ws_client = media_state.get_client();
    let user_name = ws_client.userid();
    let peers = ws_client.sorted_peer_keys();
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
    
    let (media_state, _media_dispatch) = use_store::<MediaStore>();
    let video_ref = use_node_ref();
    let screen_ref = use_node_ref();
    use_effect({
        let video_ref = video_ref.clone();
        let screen_ref = screen_ref.clone();
        let video_media_stream = media_state.get_client().get_video_media_stream_by_key(key);
        let screen_media_stream = media_state.get_client().get_screen_media_stream_by_key(key);
        move || {
            if let Some(video_element) = video_ref.cast::<HtmlVideoElement>() {
                if let Some(stream) = video_media_stream {
                    video_element.set_src_object(Some(&stream));
                }
            }
            if let Some(video_element) = screen_ref.cast::<HtmlVideoElement>() {
                if let Some(stream) = screen_media_stream {
                    video_element.set_src_object(Some(&stream));
                }
            }
        }
    });

    html! {
        <div class="bg-gray-700 shadow-2xl rounded-xl p-4 flex flex-col items-center">
            <p>{ key.clone() }</p>
            <video class="rounded-lg w-32 h-32 mb-2" ref={video_ref} autoplay=true />
            {
                if media_state.is_screen_share() {
                    html! {
                        <video class="rounded-lg w-32 h-32 mb-2" ref={screen_ref} autoplay=true />
                    }
                } else {
                    html!(<></>)
                }
            }
        </div> 
    }
}