use crate::constants::AUDIO_SAMPLE_RATE;
use js_sys::Array;
use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioContextOptions, BiquadFilterType};
use web_sys::{MediaStream, MediaStreamTrackGenerator};

pub fn configure_audio_context(
    audio_stream_generator: &MediaStreamTrackGenerator,
) -> anyhow::Result<AudioContext> {
    let js_tracks = Array::new();
    js_tracks.push(audio_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();
    let audio_context_options = AudioContextOptions::new();
    audio_context_options.set_sample_rate(AUDIO_SAMPLE_RATE as f32);
    audio_context_options.set_latency_hint(&JsValue::from("balanced"));
    let audio_context = AudioContext::new_with_context_options(&audio_context_options).unwrap();
    let gain_node = audio_context.create_gain().unwrap();
    gain_node.set_channel_count(1);
    let source = audio_context
        .create_media_stream_source(&media_stream)
        .unwrap();
    let _ = source.connect_with_audio_node(&gain_node).unwrap();
    let _ = gain_node
        .connect_with_audio_node(&audio_context.destination())
        .unwrap();
    let biquad_filter = audio_context.create_biquad_filter().expect("cannot get buquad_filter");
    biquad_filter.set_type(BiquadFilterType::Lowpass);
    biquad_filter.frequency().set_value(3000.0);
    let _ = source.connect_with_audio_node(&gain_node).unwrap();
    let _ = gain_node.connect_with_audio_node(&biquad_filter).unwrap();
    let _ = biquad_filter.connect_with_audio_node(&audio_context.destination()).unwrap();
    Ok(audio_context)
}
