use yew::{ html, function_component, Html};
use yewdux::prelude::use_store;
use crate::stores::app_store::AppStore;
use crate::AttendantsFunc;
use crate::TopBar;
use crate::Home;

#[function_component(Middleware)]
pub fn middleware() -> Html {
    let (state, _dispatch) = use_store::<AppStore>();

    html! {
        {if state.name.is_empty() {
            html! {
                <Home/>
            }
        } else {
            html! {
                <>
                    <TopBar room_id={state.id.clone()}/>
                    <AttendantsFunc />
                </>

            }
        }}
    }
}