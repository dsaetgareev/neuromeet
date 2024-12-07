use videocall_client::MediaDeviceList;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yewdux::use_store;
use videocall_client::request_permissions;
use yew::suspense::use_future;
use crate::{constants::VIDEO_ELEMENT_ID, stores::media_store::{MediaMsg, MediaStore}};
use crate::components::VideoButton;

pub struct DeviceSelector {
    media_devices: MediaDeviceList,
}

pub enum Msg {
    DevicesLoaded,
    OnCameraSelect(String),
    OnMicSelect(String),
    LoadDevices(),
}

#[derive(Properties, Debug, PartialEq)]
pub struct DeviceSelectorProps {
    pub on_camera_select: Callback<String>,
    pub on_microphone_select: Callback<String>,
}

impl DeviceSelector {
    fn create_media_device_list(ctx: &Context<DeviceSelector>) -> MediaDeviceList {
        let mut media_devices = MediaDeviceList::new();
        let link = ctx.link().clone();
        let on_microphone_select = ctx.props().on_microphone_select.clone();
        let on_camera_select = ctx.props().on_camera_select.clone();
        media_devices.on_loaded = Callback::from(move |_| link.send_message(Msg::DevicesLoaded));
        media_devices.audio_inputs.on_selected =
            Callback::from(move |device_id| on_microphone_select.emit(device_id));
        media_devices.video_inputs.on_selected =
            Callback::from(move |device_id| on_camera_select.emit(device_id));
        media_devices
    }
}

impl Component for DeviceSelector {
    type Message = Msg;
    type Properties = DeviceSelectorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            link.send_message(Msg::LoadDevices());
        });
        Self {
            media_devices: Self::create_media_device_list(ctx),
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::LoadDevices());
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadDevices() => {
                self.media_devices.load();
                false
            }
            Msg::DevicesLoaded => true,
            Msg::OnCameraSelect(camera) => {
                self.media_devices.video_inputs.select(&camera);
                true
            }
            Msg::OnMicSelect(mic) => {
                self.media_devices.audio_inputs.select(&mic);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mics = self.media_devices.audio_inputs.devices();
        let cameras = self.media_devices.video_inputs.devices();
        let selected_mic = self.media_devices.audio_inputs.selected();
        let selected_camera = self.media_devices.video_inputs.selected();
        fn selection(event: Event) -> String {
            event
                .target()
                .expect("Event should have a target when dispatched")
                .unchecked_into::<HtmlSelectElement>()
                .value()
        }

        html! {
            <div class={"device-selector-wrapper"}>
                <label for={"audio-select"}>{ "Audio:" }</label>
                <select id={"audio-select"} class={"device-selector"}
                        onchange={ctx.link().callback(|e: Event| Msg::OnMicSelect(selection(e)))}
                >
                    { for mics.iter().map(|device| html! {
                        <option value={device.device_id()} selected={selected_mic == device.device_id()}>
                            { device.label() }
                        </option>
                    }) }
                </select>
                <br/>
                <label for={"video-select"}>{ "Video:" }</label>
                <select id={"video-select"} class={"device-selector"}
                        onchange={ctx.link().callback(|e:Event| Msg::OnCameraSelect(selection(e))) }
                >
                    { for cameras.iter().map(|device| html! {
                        <option value={device.device_id()} selected={selected_camera == device.device_id()}>
                            { device.label() }
                        </option>
                    }) }
                </select>
            </div>
        }
    }
}

#[function_component(Devices)]
pub fn devices() -> Html {
    let (_state, dispatch) = use_store::<MediaStore>();
    let mic_callback: Callback<String> = {
        let dispatch = dispatch.clone();
        Callback::from(move |audio| {
            dispatch.apply(MediaMsg::AudioDeviceChanged(audio))
        })
    };
    let cam_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |video| {
            dispatch.apply(MediaMsg::VideoDeviceChanged(video));
        })
    };
    html! {
        <>
            <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
        </>
    }
}

#[function_component(PermissionsDevices)]
pub fn permissions_devices() -> Html {
    let future = use_future(|| async {
        match request_permissions().await {
            Ok(_res) => {
                html!{
                    <>
                        <Devices />
                        <div>
                            <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                            <VideoButton />
                        </div>      
                    </>
                }
            }
            Err(e) => {
                html!(
                    <>
                        { format!("{:?}", e.to_owned()) }
                        <p> { "Дайте разрешение браузеру использовать медиа ресурсы (камера, микрофон)" } </p>
                    </>
                )
            } 
        }
    });
    let devices = match future {
        Ok(res) => {
            res.to_owned()
        }
        Err(e) => {
            html! {
                <>
                    { format!("{:?}", e.to_owned()) }
                </>
            }
        } 
        
    };

    html! {
        <>
            { devices }
        </>
    }
}