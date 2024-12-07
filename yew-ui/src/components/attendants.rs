use crate::constants::VIDEO_ELEMENT_ID;
use crate::stores::media_store::MediaStore;
use web_sys::*;
use yew::prelude::*;
use yew::{html, Html};
use yewdux::use_store;
use crate::components::{Devices, VideoButton};

#[function_component(AttendantsFunc)]
pub fn attendats_func() -> Html {

    let (media_state, _media_dispatch) = use_store::<MediaStore>();
    let ws_agent = media_state.get_agent();
    let user_name = ws_agent.get_user_id();
    let peers = ws_agent.get_peers();
    let rows = || {
        
        peers.keys()
        .map(|key| {
            html! {
                <>
                    <VideoComponent key_id={ key.clone() } />
                </>
            }
        }).collect::<Html>()
    };

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
                    <h4 class="floating-name">{user_name}</h4>

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

        let video_media_stream = media_state.get_agent().get_peers().get(key).unwrap().get_video_ms_cloned();
        let screen_media_stream = media_state.get_agent().get_peers().get(key).unwrap().get_screen_ms_cloned();
        move || {
            if let Some(video_element) = video_ref.cast::<HtmlVideoElement>() {
                video_element.set_src_object(Some(&video_media_stream));
            }
            if let Some(video_element) = screen_ref.cast::<HtmlVideoElement>() {
                video_element.set_src_object(Some(&screen_media_stream));
            }
        }
    });

    html! {
        <div class="bg-gray-700 shadow-2xl rounded-xl p-4 flex flex-col items-center">
            <p>{ key.clone() }</p>
            <video class="rounded-lg w-32 h-32 mb-2" ref={video_ref} autoplay=true />
            <video class="rounded-lg w-32 h-32 mb-2" ref={screen_ref} autoplay=true />
            // {
            //     if media_state.is_screen_share() {
            //         html! {
            //             <video class="rounded-lg w-32 h-32 mb-2" ref={screen_ref} autoplay=true />
            //         }
            //     } else {
            //         html!(<></>)
            //     }
            // }
        </div> 
    }
}