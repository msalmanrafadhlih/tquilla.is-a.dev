use dioxus::prelude::*;

use super::audio::{load_and_play, pause_audio, resume_audio, set_audio_volume};
use crate::api::{self, RadioStation};
use crate::languages::{use_strings, Strings};
use crate::nixdesk::js_util::{eval_js, js_string_escape};
use crate::remote::LoadFailed;
use crate::skeleton::{SkeletonGroup, SkeletonRows};

const AUDIO_ID: &str = "radio-audio";
const PLAY_ICON: Asset = asset!("/assets/icon-play.svg");
const PAUSE_ICON: Asset = asset!("/assets/icon-pause.svg");
const SKIP_BACK_ICON: Asset = asset!("/assets/icon-skip-backward.svg");
const SKIP_FORWARD_ICON: Asset = asset!("/assets/icon-skip-forward.svg");
const VOLUME_ICON: Asset = asset!("/assets/icon-volume.svg");
const WAVE_ICON: Asset = asset!("/assets/logo-soundwave.svg");

/// Stand-in for "no station selected" (empty list): no stream, no link, no
/// image, so the player shows its label and does nothing else.
fn empty_station(s: &Strings) -> RadioStation {
    RadioStation {
        id: 0,
        label: s.radio.no_station.to_string(),
        slogan: String::new(),
        stream_url: String::new(),
        official_link: None,
        image_url: None,
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

/// Loads the stations, then hands them to [`RadioPlayer`].
#[component]
pub fn RadioWindowContent() -> Element {
    let mut resource = use_resource(api::radio);
    let state = resource.read_unchecked();
    match &*state {
        None => rsx! {
            SkeletonGroup { class: "p-3",
                SkeletonRows { rows: 5 }
            }
        },
        Some(Err(e)) => rsx! {
            LoadFailed { error: e.clone(), on_retry: move |_| resource.restart() }
        },
        Some(Ok(list)) => rsx! {
            RadioPlayer { initial: list.clone() }
        },
    }
}

#[component]
fn RadioPlayer(initial: Vec<RadioStation>) -> Element {
    let s = use_strings();
    // Signal supaya Copy -> bisa dipakai di banyak closure tanpa clone manual.
    let stations = use_signal(move || initial.clone());
    let mut selected = use_signal(|| 0usize);
    let mut playing = use_signal(|| false);
    let mut volume = use_signal(|| 0.7f64);

    let total = stations.read().len();
    let current = stations
        .read()
        .get(selected())
        .cloned()
        .unwrap_or_else(|| empty_station(s));
    let initial_url = current.stream_url.clone();
    let station_list = stations.read().clone();
    let play_title = if playing() {
        s.radio.pause
    } else {
        s.radio.play
    };
    let volume_pct = (volume() * 100.0).round() as u32;

    // Satu-satunya tempat logika "pindah station + play", dipakai oleh
    // list station, tombol previous, dan tombol next.
    let mut play_station = move |idx: usize| {
        let Some(url) = stations.read().get(idx).map(|s| s.stream_url.clone()) else {
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
                "aria-label": s.radio.stations_label,
                for (idx , station) in station_list.iter().enumerate() {
                    {
                        let image = station.image_url.clone();
                        let label = station.label.clone();
                        let is_selected = selected() == idx;
                        rsx! {
                            button {
                                key: "{idx}",
                                r#type: "button",
                                class: "w-full max-w-[300px] flex flex-col @min-[643px]:flex-row items-center gap-2 px-3 py-2 text-left border-0",
                                class: if is_selected { "bg-white text-black" } else { "bg-[var(--bg-secondary)] text-white/30 hover:text-white hover:bg-conic-gradient" },
                                onclick: move |_| play_station(idx),
                                if let Some(src) = image.as_deref() {
                                    img {
                                        src: "{src}",
                                        alt: "",
                                        class: "@min-[643px]:w-6 @min-[643px]:h-6 w-[70px] h-[70px] object-cover shrink-0",
                                    }
                                }
                                span { class: "w-[70px] @min-[643px]:w-full text-center @min-[643px]:text-left truncate", "{label}" }
                            }
                        }
                    }
                }
            }

            // CONTROL PLAYER
            div { class: "@container flex-1 w-full h-full min-w-0 flex flex-col items-center justify-start @min-[643px]:justify-center gap-4 px-6 py-6 text-center overflow-scroll [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
                if let Some(src) = current.image_url.as_deref() {
                    img {
                        src: "{src}",
                        alt: "{current.label}",
                        class: "w-28 h-28 object-cover border border-white/30",
                    }
                }

                div {
                    p { class: "text-sm text-white font-semibold", "{current.label}" }
                    p { class: "text-white/30 text-[10px] mt-1",
                        "{s.radio.slogan_line(&current.slogan)}"
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
                            "aria-label": s.radio.controls_label,

                            // PLAYBACK CONTROLS
                            nav {
                                class: "flex items-center justify-center gap-5 relative w-max flex-[0_0_auto]",
                                "aria-label": s.radio.playback_controls,
                                button {
                                    r#type: "button",
                                    class: "w-7 h-7 flex items-center bg-[var(--bg-secondary)] border-0 justify-center relative appearance-none p-0",
                                    title: s.radio.previous_station,
                                    "aria-label": s.radio.previous_station,
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
                                    title: s.radio.next_station,
                                    "aria-label": s.radio.next_station,
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
                                        "aria-label": s.radio.volume,
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
                        "aria-label": s.radio.playback_status,
                        div { class: "flex h-1 items-start w-full bg-white/30 relative self-stretch",
                            div {
                                class: if playing() { "w-full bg-white relative self-stretch animate-pulse" } else { "w-0 bg-white relative self-stretch" },
                                "aria-hidden": "true",
                            }
                        }
                        div { class: "flex items-end justify-between relative self-stretch w-full flex-[0_0_auto]",
                            span { class: "relative w-fit mt-[-1.00px] [font-family:'JetBrains_Mono-Regular',Helvetica] font-normal text-white text-[10px] tracking-[0] leading-[normal]",
                                "{s.radio.live}"
                            }
                            span { class: "relative w-fit mt-[-1.00px] [font-family:'JetBrains_Mono-Regular',Helvetica] font-normal text-white text-[10px] tracking-[0] leading-[normal]",
                                if playing() {
                                    "{s.radio.on_air}"
                                } else {
                                    "{s.radio.paused}"
                                }
                            }
                        }
                    }
                }

                if let Some(href) = current.official_link.as_deref() {
                    a {
                        class: "text-white/30 hover:text-white transition-colors duration-150 text-[10px]",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        href: "{href}",
                        "{s.radio.stations_link}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;

    #[test]
    fn the_placeholder_station_has_nothing_to_play_or_open() {
        let st = empty_station(Language::default().strings());
        assert!(st.stream_url.is_empty());
        assert!(st.official_link.is_none() && st.image_url.is_none());
        assert!(!st.label.is_empty());
    }
}
