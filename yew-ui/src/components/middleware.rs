use yew::{ html, function_component, Html};
use yewdux::prelude::use_store;
use crate::stores::app_store::AppStore;
use crate::AttendantsFunc;
use crate::AttendantsComponent;
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
                    // <AttendantsComponent email={state.name.clone()} id={state.id.clone()} webtransport_enabled={true} e2ee_enabled={false} />
                    <AttendantsFunc />
                </>

            }
        }}
    }
}