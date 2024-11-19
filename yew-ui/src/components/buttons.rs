use yew::prelude::*;
use yewdux::use_store;

use crate::stores::media_store::{MediaMsg, MediaStore};



#[function_component(VideoButton)]
pub fn video_button() -> Html {

    let (media_state, media_dispatch) = use_store::<MediaStore>();
    let camera_enabled = use_state(|| media_state.get_camera().get_enabled());
    let mic_enabled = use_state(|| media_state.get_mic().get_enabled());

    // let onclick = {
    //     let media_dispatch = media_dispatch.clone();
    //     let camera_enabled = camera_enabled.clone();
    //     Callback::from(move |_event| {
    //         camera_enabled.set(!*camera_enabled);
    //         media_dispatch.apply(MediaMsg::SwitchVedeo(false));
    //     })
    // };

    // let mic_onclick = {
    //     let media_dispatch = media_dispatch.clone();
    //     let mic_enabled = mic_enabled.clone();
    //     Callback::from(move |_event| {
    //         mic_enabled.set(!*mic_enabled);
    //         media_dispatch.apply(MediaMsg::SwitchMic(false));
    //     })
    // };

    let screen_onclick = {
        let media_state = media_state.clone();
        let is_screen_share = media_state.is_screen_share();
        Callback::from(move |_event| {
            if !is_screen_share {
                media_dispatch.apply(MediaMsg::EnableScreenShare);
            } else {
                media_dispatch.apply(MediaMsg::DisableScreenShare);
            }
        })
    };

    html! {
        <div class="parent flex items-center justify-center">
            <div class="child w-1/5 bg-blue-500 text-white text-center p-4">
                // <button
                //     class="bg-yew-blue p-2 rounded-md text-white" 
                //     onclick={ onclick }>
                //     { 
                //         if *camera_enabled {
                //             html! { <n-icon ><svg role="presentation" viewBox="0 0 24 24" style="vertical-align: top; overflow: hidden; width: 24px; height: 24px;"><path class="n-icon__fill" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" vector-effect="non-scaling-stroke" d="m21.443 6.354-5.075 3.383v4.525l5.075 3.384a1 1 0 0 0 1.555-.832V7.186a1 1 0 0 0-1.555-.832Zm-8.075-1.37H5.002a3 3 0 0 0-3 3v8.024a3 3 0 0 0 3 3h8.366a3 3 0 0 0 3-3V7.984a3 3 0 0 0-3-3Z" stroke-width="1.6"></path></svg></n-icon> }
                //         } else {
                //             html! { <n-icon class=""><svg role="presentation" viewBox="0 0 24 24" style="vertical-align: top; overflow: hidden; width: 24px; height: 24px;"><path  d="M12.368 19.008H5.002a3 3 0 0 1-3-3V7.984m6.998-3h4.368a3 3 0 0 1 3 3V12.5m2.132 8L2 3m14.368 6.737 5.075-3.383a1 1 0 0 1 1.555.832v9.628a1 1 0 0 1-1.555.832l-5.075-3.384V9.737Z" stroke-width="1.6"></path></svg></n-icon>  }
                //         }
                //     }
                // </button>
                // <button
                //     class="bg-yew-blue p-2 rounded-md text-white" 
                //     onclick={ mic_onclick }
                // >
                //     { 
                //         if *mic_enabled {
                //             html! { <n-icon><svg role="presentation" viewBox="0 0 24 24" style="vertical-align: top; overflow: hidden; width: 24px; height: 24px;"><path class="n-icon__fill" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" vector-effect="non-scaling-stroke" d="M4.536 11.268a7.464 7.464 0 1 0 14.928 0M12 19v2.689M12 15a4 4 0 0 1-4-4V7a4 4 0 1 1 8 0v4a4 4 0 0 1-4 4Z" stroke-width="1.6"></path></svg></n-icon> }
                //         } else {
                //             html! { <n-icon><svg role="presentation" viewBox="0 0 24 24" style="vertical-align: top; overflow: hidden; width: 24px; height: 24px;"><path class="n-icon__fill" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" vector-effect="non-scaling-stroke" d="M16 11V7a4 4 0 0 0-6.994-2.653m5.27 9.942A4 4 0 0 1 8 11V8.034m-3.463 3.235a7.464 7.464 0 0 0 12.348 5.645m1.833-2.387a7.407 7.407 0 0 0 .747-3.258M20 20 4 4m8 15v2.689" stroke-width="1.6"></path></svg></n-icon> }
                //         }
                //     }
                // </button>
                <button
                    class="bg-yew-blue p-2 rounded-md text-white" 
                    onclick={ screen_onclick }
                >
                    { "s" }
                    // <n-icon><svg role="presentation" viewBox="0 0 24 24" style="vertical-align: top; overflow: hidden; width: 24px; height: 24px;"><path class="n-icon__fill" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" vector-effect="non-scaling-stroke" d="M16 11V7a4 4 0 0 0-6.994-2.653m5.27 9.942A4 4 0 0 1 8 11V8.034m-3.463 3.235a7.464 7.464 0 0 0 12.348 5.645m1.833-2.387a7.407 7.407 0 0 0 .747-3.258M20 20 4 4m8 15v2.689" stroke-width="1.6"></path></svg></n-icon>
                </button>
            </div>

        </div>
    }
}