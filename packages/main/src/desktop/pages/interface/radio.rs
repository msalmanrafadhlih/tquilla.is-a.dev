use dioxus::prelude::*;
use serde::Deserialize;

use super::audio::{load_and_play, pause_audio, resume_audio, set_audio_volume};
use super::js_util::{eval_js, js_string_escape};

const RADIO_JSON: &str = include_str!("../../../../data/radio.json");
const AUDIO_ID: &str = "radio-audio";
const SKIP_BACK_ICON: Asset = asset!("/assets/skip-back.svg");
const SKIP_FORWARD_ICON: Asset = asset!("/assets/skip-forward.svg");
const WAVE_ICON: Asset = asset!("/assets/sound_wave.svg");

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Station {
    label: String,
    slogan: String,
    url_stream: String,
    official_link: String,
    image: String,
}

fn load_stations() -> Vec<Station> {
    serde_json::from_str(RADIO_JSON).unwrap_or_default()
}

fn empty_station() -> Station {
    Station {
        label: "No station".to_string(),
        slogan: String::new(),
        url_stream: String::new(),
        official_link: "#".to_string(),
        image: String::new(),
    }
}

/// Loads a station's stream without starting playback — used on mount so
/// the player is primed without fighting browser autoplay restrictions.
fn prime_station(audio_id: &str, url: &str) {
    eval_js(format!(
        "window.__audio && window.__audio.load('{audio_id}', '{}');",
        js_string_escape(url)
    ));
}

#[component]
pub fn RadioWindowContent() -> Element {
    let stations = load_stations();
    let mut selected = use_signal(|| 0usize);
    let mut playing = use_signal(|| false);
    let mut volume = use_signal(|| 0.7f64);

    let total = stations.len();
    let current = stations.get(selected()).cloned().unwrap_or_else(empty_station);
    let initial_url = current.url_stream.clone();
    let play_label = if playing() { "\u{275A}\u{275A}" } else { "\u{25B6}" };
    let play_title = if playing() { "Pause" } else { "Play" };

    rsx! {
        div { class: "flex h-full text-[11px]",
            audio {
                id: AUDIO_ID,
                onmounted: move |_| prime_station(AUDIO_ID, &initial_url),
            }

            div { class: "w-40 shrink-0 border-r border-white/15 overflow-y-auto py-2",
                for (idx , station) in stations.iter().enumerate() {
                    {
                        let url = station.url_stream.clone();
                        let image = station.image.clone();
                        let label = station.label.clone();
                        let is_selected = selected() == idx;
                        rsx! {
                            button {
                                key: "{label}",
                                r#type: "button",
                                class: "w-full flex items-center gap-2 px-3 py-2 text-left",
                                class: if is_selected { "bg-white text-black" } else { "text-white/70 hover:text-white" },
                                onclick: move |_| {
                                    selected.set(idx);
                                    playing.set(true);
                                    load_and_play(AUDIO_ID, &url);
                                    set_audio_volume(AUDIO_ID, volume());
                                },
                                img { src: image, alt: "", class: "w-6 h-6 object-cover shrink-0" }
                                span { class: "truncate", "{label}" }
                            }
                        }
                    }
                }
            }

            div { class: "flex-1 min-w-0 flex flex-col items-center justify-center gap-4 px-6 py-6 text-center",
                img {
                    src: current.image.clone(),
                    alt: "{current.label}",
                    class: "w-28 h-28 object-cover border border-white/20",
                }
                div {
                    p { class: "text-sm font-semibold", "{current.label}" }
                    p { class: "text-white/40 text-[10px] mt-1", "Radio Indonesia - {current.label}" }
                }
                img { src: WAVE_ICON, alt: "", class: "w-40 opacity-60" }

                div { class: "flex items-center gap-5",
                    button {
                        r#type: "button",
                        title: "Previous station",
                        onclick: move |_| {
                            if total > 0 {
                                let next = (selected() + total - 1) % total;
                                let list = load_stations();
                                if let Some(s) = list.get(next) {
                                    selected.set(next);
                                    playing.set(true);
                                    load_and_play(AUDIO_ID, &s.url_stream);
                                    set_audio_volume(AUDIO_ID, volume());
                                }
                            }
                        },
                        img { src: SKIP_BACK_ICON, alt: "Previous", class: "w-4 h-4 opacity-80" }
                    }
                    button {
                        r#type: "button",
                        title: "{play_title}",
                        class: "w-9 h-9 flex items-center justify-center border border-white/30 hover:bg-white hover:text-black transition-colors duration-150",
                        onclick: move |_| {
                            if playing() {
                                playing.set(false);
                                pause_audio(AUDIO_ID);
                            } else {
                                playing.set(true);
                                resume_audio(AUDIO_ID);
                            }
                        },
                        "{play_label}"
                    }
                    button {
                        r#type: "button",
                        title: "Next station",
                        onclick: move |_| {
                            if total > 0 {
                                let next = (selected() + 1) % total;
                                let list = load_stations();
                                if let Some(s) = list.get(next) {
                                    selected.set(next);
                                    playing.set(true);
                                    load_and_play(AUDIO_ID, &s.url_stream);
                                    set_audio_volume(AUDIO_ID, volume());
                                }
                            }
                        },
                        img { src: SKIP_FORWARD_ICON, alt: "Next", class: "w-4 h-4 opacity-80" }
                    }
                }

                div { class: "flex items-center gap-2 w-full max-w-[220px]",
                    span { class: "text-white/40", "Vol" }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "1",
                        step: "0.01",
                        value: "{volume}",
                        class: "flex-1 accent-white",
                        oninput: move |evt: FormEvent| {
                            let v: f64 = evt.value().parse().unwrap_or(0.7);
                            volume.set(v);
                            set_audio_volume(AUDIO_ID, v);
                        },
                    }
                }

                a {
                    href: "{current.official_link}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "text-white/40 hover:text-white transition-colors duration-150 text-[10px]",
                    "Radio Stations ↗"
                }
            }
        }
    }
}
