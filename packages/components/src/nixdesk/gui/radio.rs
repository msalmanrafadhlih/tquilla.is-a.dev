use dioxus::prelude::*;
use serde::Deserialize;

use super::audio::{load_and_play, pause_audio, resume_audio, set_audio_volume};
use crate::nixdesk::js_util::{eval_js, js_string_escape};

const RADIO_JSON: &str = include_str!("../../../data/radio.json");
const AUDIO_ID: &str = "radio-audio";
const PLAY_ICON: Asset = asset!("/assets/icon-play.svg");
const PAUSE_ICON: Asset = asset!("/assets/icon-pause.svg");
const SKIP_BACK_ICON: Asset = asset!("/assets/icon-skip-backward.svg");
const SKIP_FORWARD_ICON: Asset = asset!("/assets/icon-skip-forward.svg");
const VOLUME_ICON: Asset = asset!("/assets/icon-volume.svg");
const WAVE_ICON: Asset = asset!("/assets/logo-soundwave.svg");

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
    // Signal supaya Copy -> bisa dipakai di banyak closure tanpa clone manual.
    let stations = use_signal(load_stations);
    let mut selected = use_signal(|| 0usize);
    let mut playing = use_signal(|| false);
    let mut volume = use_signal(|| 0.7f64);

    let total = stations.read().len();
    let current = stations
        .read()
        .get(selected())
        .cloned()
        .unwrap_or_else(empty_station);
    let initial_url = current.url_stream.clone();
    let station_list = stations.read().clone();
    let play_title = if playing() { "Pause" } else { "Play" };
    let volume_pct = (volume() * 100.0).round() as u32;

    // Satu-satunya tempat logika "pindah station + play", dipakai oleh
    // list station, tombol previous, dan tombol next.
    let mut play_station = move |idx: usize| {
        let Some(url) = stations.read().get(idx).map(|s| s.url_stream.clone()) else {
            return;
        };
        selected.set(idx);
        playing.set(true);
        load_and_play(AUDIO_ID, &url);
        set_audio_volume(AUDIO_ID, volume());
    };

    rsx! {
        // MAIN CONTENTS : RADIO
        div { class: "flex h-full w-full flex-col @min-[643px]:flex-row items-center justify-center gap-2.5 relative overflow-hidden",
            audio {
                id: AUDIO_ID,
                onmounted: move |_| prime_station(AUDIO_ID, &initial_url),
            }

            // SIDE PANEL
            nav {
                class: "flex items-center @min-[643px]:flex-col justify-start @min-[643px]:p-0 p-2 relative self-stretch w-full h-max @min-[643px]:max-w-max max-w-full @min-[643px]:h-full overflow-scroll [scrollbar-width:none] [&::-webkit-scrollbar]:hidden border-b-[0.5px] @min-[643px]:border-0 @min-[643px]:border-r-[0.5px] border-solid border-white/30",
                "aria-label": "Radio stations",
                for (idx , station) in station_list.iter().enumerate() {
                    {
                        let image = station.image.clone();
                        let label = station.label.clone();
                        let is_selected = selected() == idx;
                        rsx! {
                            button {
                                key: "{idx}",
                                r#type: "button",
                                class: "w-full max-w-[300px] flex flex-col @min-[643px]:flex-row items-center gap-2 px-3 py-2 text-left border-0",
                                class: if is_selected { "bg-white text-black" } else { "bg-[var(--bg-secondary)] text-white/30 hover:text-white hover:bg-conic-gradient" },
                                onclick: move |_| play_station(idx),
                                img {
                                    src: "{image}",
                                    alt: "",
                                    class: "@min-[643px]:w-6 @min-[643px]:h-6 w-[70px] h-[70px] object-cover shrink-0",
                                }
                                span { class: "w-[70px] @min-[643px]:w-full text-center @min-[643px]:text-left truncate", "{label}" }
                            }
                        }
                    }
                }
            }

            // CONTROL PLAYER
            div { class: "@container flex-1 w-full h-full min-w-0 flex flex-col items-center justify-start @min-[643px]:justify-center gap-4 px-6 py-6 text-center overflow-scroll [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
                img {
                    src: "{current.image}",
                    alt: "{current.label}",
                    class: "w-28 h-28 object-cover border border-white/30",
                }

                div {
                    p { class: "text-sm text-white font-semibold", "{current.label}" }
                    p { class: "text-white/30 text-[10px] mt-1",
                        "Radio Indonesia - {current.slogan}"
                    }
                }

                main { class: "flex flex-col items-center justify-center gap-[15px] relative",
                    header { class: "flex-col flex items-center justify-center gap-5 relative self-stretch w-full flex-[0_0_auto]",
                        div { class: "relative max-w-[200px] w-full max-h-[83.4px] aspect-[2.4]",
                            img {
                                class: "absolute w-[95.09%] h-[82.80%] top-[17.20%] left-[4.91%]",
                                src: WAVE_ICON,
                                alt: "",
                                "aria-hidden": "true",
                            }
                        }

                        section {
                            class: "flex flex-col @min-[643px]:flex-row w-[284px] items-center justify-center gap-5 relative flex-[0_0_auto]",
                            "aria-label": "Audio player controls",

                            // PLAYBACK CONTROLS
                            nav {
                                class: "flex items-center justify-center gap-5 relative w-max flex-[0_0_auto]",
                                "aria-label": "Playback controls",
                                button {
                                    r#type: "button",
                                    class: "w-7 h-7 flex items-center bg-[var(--bg-secondary)] border-0 justify-center relative appearance-none p-0",
                                    title: "Previous station",
                                    "aria-label": "Previous station",
                                    onclick: move |_| {
                                        if total > 0 {
                                            play_station((selected() + total - 1) % total);
                                        }
                                    },
                                    img {
                                        class: "relative w-4 h-4",
                                        src: SKIP_BACK_ICON,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "group w-10 h-10 bg-[var(--bg-secondary)] hover:bg-white aspect-[1] flex items-center justify-center relative appearance-none border-0 p-0",
                                    title: "{play_title}",
                                    "aria-label": "{play_title}",
                                    onclick: move |_| {
                                        if playing() {
                                            playing.set(false);
                                            pause_audio(AUDIO_ID);
                                        } else {
                                            playing.set(true);
                                            resume_audio(AUDIO_ID);
                                        }
                                    },
                                    img {
                                        class: "group-hover:invert relative w-[18px] h-[18px] aspect-[1]",
                                        src: if playing() { PAUSE_ICON } else { PLAY_ICON },
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "w-7 h-7 flex items-center bg-[var(--bg-secondary)] border-0 justify-center relative appearance-none p-0",
                                    title: "Next station",
                                    "aria-label": "Next station",
                                    onclick: move |_| {
                                        if total > 0 {
                                            play_station((selected() + 1) % total);
                                        }
                                    },
                                    img {
                                        class: "relative w-4 h-4",
                                        src: SKIP_FORWARD_ICON,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                            }

                            // VOLUME
                            div { class: "flex w-32 h-10 items-center justify-center gap-3 relative",
                                div { class: "flex flex-col w-4 items-start gap-8 relative",
                                    img {
                                        class: "relative w-4 h-4 aspect-[1]",
                                        src: VOLUME_ICON,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                                div { class: "flex w-[100px] h-1 items-start relative bg-white/30",
                                    div {
                                        class: "relative self-stretch bg-white",
                                        style: "width: {volume_pct}%;",
                                        "aria-hidden": "true",
                                    }
                                    input {
                                        class: "absolute inset-0 w-full h-full opacity-0 cursor-pointer",
                                        r#type: "range",
                                        min: "0",
                                        max: "1",
                                        step: "0.01",
                                        value: "{volume}",
                                        "aria-label": "Volume",
                                        oninput: move |evt: FormEvent| {
                                            let v: f64 = evt.value().parse().unwrap_or(0.7);
                                            volume.set(v);
                                            set_audio_volume(AUDIO_ID, v);
                                        },
                                    }
                                }
                            }
                        }
                    }

                    // STATUS (radio = live stream, jadi tidak ada seek/durasi)
                    section {
                        class: "flex flex-col h-10 items-center justify-end gap-2 w-full relative self-stretch",
                        "aria-label": "Playback status",
                        div { class: "flex h-1 items-start w-full bg-white/30 relative self-stretch",
                            div {
                                class: if playing() { "w-full bg-white relative self-stretch animate-pulse" } else { "w-0 bg-white relative self-stretch" },
                                "aria-hidden": "true",
                            }
                        }
                        div { class: "flex items-end justify-between relative self-stretch w-full flex-[0_0_auto]",
                            span { class: "relative w-fit mt-[-1.00px] [font-family:'JetBrains_Mono-Regular',Helvetica] font-normal text-white text-[10px] tracking-[0] leading-[normal]",
                                "LIVE"
                            }
                            span { class: "relative w-fit mt-[-1.00px] [font-family:'JetBrains_Mono-Regular',Helvetica] font-normal text-white text-[10px] tracking-[0] leading-[normal]",
                                if playing() {
                                    "ON AIR"
                                } else {
                                    "PAUSED"
                                }
                            }
                        }
                    }
                }

                a {
                    class: "text-white/30 hover:text-white transition-colors duration-150 text-[10px]",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    href: "{current.official_link}",
                    "Radio Stations ↗"
                }
            }
        }
    }
}
