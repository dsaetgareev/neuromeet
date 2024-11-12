#![feature(future_join)]
#[allow(non_camel_case_types)]
mod components;
mod constants;
mod pages;
mod utils;
mod stores;

use constants::LOGIN_URL;

use log::info;
use yew::prelude::*;
#[macro_use]
extern crate lazy_static;
use components::{middleware::Middleware, top_bar::TopBar, AttendantsFunc};
use enum_display::EnumDisplay;
use gloo_utils::window;
use pages::home::Home;
use yew_router::prelude::*;

use crate::constants::ENABLE_OAUTH;

#[derive(Clone, Routable, PartialEq, Debug, EnumDisplay)]
enum Route {
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

struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        App {}
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(());
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        info!("OAuth enabled: {}", *ENABLE_OAUTH);
        html! {
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        }
    }
}

fn main() {
    #[cfg(feature = "debugAssertions")]
    {
        _ = console_log::init_with_level(log::Level::Debug);
    }
    #[cfg(not(feature = "debugAssertions"))]
    {
        _ = console_log::init_with_level(log::Level::Info);
    }

    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
