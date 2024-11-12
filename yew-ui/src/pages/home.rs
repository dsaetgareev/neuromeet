use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::components::PermissionsDevices;
use crate::stores::app_store::AppMsg;
use crate::stores::app_store::AppStore;
use crate::stores::media_store::MediaMsg;
use crate::stores::media_store::MediaStore;
use crate::utils::dom::get_url_pathname;
use crate::Route;

const TEXT_INPUT_CLASSES: &str = "rounded-md mx-2 p-2 text-black required:ring-2 required:ring-red-500 required:valid:ring-2 required:valid:ring-green-500";

#[function_component(Home)]
pub fn home() -> Html {
    let (_state, dispatch) = use_store::<AppStore>();
    let (_media_state, media_dispatch) = use_store::<MediaStore>();
    let navigator = use_navigator().unwrap();

    let username_ref = use_node_ref();
    let session_id = use_state(|| {
        let url = get_url_pathname();
        let session_id = match url {
            Ok(url) => {
                let id = if url == "/" || url.is_empty() {
                    let generated_session_id = uuid::Uuid::new_v4().to_string(); 
                    generated_session_id
                } else {
                    let id = match url.split("/m/").last() {
                        Some(session_id) => {
                            session_id.to_string()
                        },
                        _ => {
                            let generated_session_id = uuid::Uuid::new_v4().to_string(); 
                            generated_session_id
                        },
                    };
                    id
                };
                id
            },
            Err(_) => todo!(),
        };
        session_id
    });

    let onsubmit = {
        let username_ref = username_ref.clone();
        let dispatch = dispatch.clone();
        let media_dispatch = media_dispatch.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = username_ref.cast::<HtmlInputElement>().unwrap().value();
            let meeting_id = session_id.to_string();
            media_dispatch.apply(MediaMsg::ClientInit(username.clone(), meeting_id.clone()));
            dispatch.apply(AppMsg::SetName(username));
            dispatch.apply(AppMsg::SetId(meeting_id.clone()));
            navigator.push(&Route::Middleware {
                id: meeting_id,
            })
        })
    };
    html! {
        <div class="flex justify-center items-center content-center flex-col m-auto">
            <div class="flex items-center flex-col">
                <h1 class="text-xl">{ "Neuromeet" }</h1>
                <p class="text-xs">{ "Создайте комнату видеоконференции, указав логин" }</p>
                <p class="text-xs">{ "Допускаются символы: a-z, A-Z, 0-9, и _" }</p>
            </div>
            <form {onsubmit}>
                <div class="py-4">
                    <input
                        class={TEXT_INPUT_CLASSES}
                        label="username"
                        type="text"
                        placeholder="Username"
                        ref={username_ref}
                        required={true}
                        pattern="^[a-zA-Z0-9_]*$"
                        value="User"
                    />
                </div>
                <input type="submit" value="Подключиться" class="py-2 px-4 pointer bg-yew-blue rounded-md w-full cursor-pointer" />
            </form>
            <PermissionsDevices /> 
        </div>
    }
}
