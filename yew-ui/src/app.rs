#[allow(non_camel_case_types)]

use yew::prelude::*;
use enum_display::EnumDisplay;
use gloo_utils::window;
use yew_router::prelude::*;
use yew::{html, Html, function_component};

use crate::{components::Middleware, constants::LOGIN_URL, pages::Home};

#[derive(Clone, Routable, PartialEq, Debug, EnumDisplay)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/m/:id")]
    Middleware { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login/> },
        Route::Middleware {
            id: _ 
        } => html! {
            <>
                <Middleware />
            </>
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Login)]
fn login() -> Html {
    let login = Callback::from(|_: MouseEvent| {
        window().location().set_href(LOGIN_URL).ok();
    });
    html! {<>
        <input type="image" onclick={login.clone()} src="/assets/btn_google.png" />
    </>}
}

#[function_component(App)]
pub fn app() -> Html {
   
    html! {
        <BrowserRouter>
            <main>
                <Switch<Route> render={switch} />
            </main>
        </BrowserRouter>
    }
}

