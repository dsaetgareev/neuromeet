use yew::prelude::*;


#[derive(Properties, Debug, PartialEq)]
pub struct TopBarProps {
    pub room_id: String,
}

#[function_component(TopBar)]
pub fn top_bar(props: &TopBarProps) -> Html {
    html! {
        <div class="top-bar">
            <span>{ props.room_id.clone() }</span>
        </div>
    }
}
