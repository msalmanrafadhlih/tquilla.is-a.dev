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
            SkeletonGroup { class: "app-window__loading",
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
        div { class: "radio",
            audio {
                id: AUDIO_ID,
                onmounted: move |_| prime_station(AUDIO_ID, &initial_url),
            }

            // SIDE PANEL
            nav {
                class: "radio__list",
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
                                class: "radio__station",
                                class: if is_selected { "radio__station--on" } else { "radio__station--off" },
                                onclick: move |_| play_station(idx),
                                if let Some(src) = image.as_deref() {
                                    img {
                                        src: "{src}",
                                        alt: "",
                                        class: "radio__station-img",
                                    }
                                }
                                span { class: "radio__station-name", "{label}" }
                            }
                        }
                    }
                }
            }

            // CONTROL PLAYER
            div { class: "radio__stage",
                if let Some(src) = current.image_url.as_deref() {
                    img {
                        src: "{src}",
                        alt: "{current.label}",
                        class: "radio__cover",
                    }
                }

                div {
                    p { class: "radio__title", "{current.label}" }
                    p { class: "radio__subtitle",
                        "{s.radio.slogan_line(&current.slogan)}"
                    }
                }

                main { class: "radio__player",
                    header { class: "radio__brand",
                        div { class: "radio__logo-box",
                            img {
                                class: "radio__logo",
                                src: WAVE_ICON,
                                alt: "",
                                "aria-hidden": "true",
                            }
                        }

                        section {
                            class: "radio__controls",
                            "aria-label": s.radio.controls_label,

                            // PLAYBACK CONTROLS
                            nav {
                                class: "radio__buttons",
                                "aria-label": s.radio.playback_controls,
                                button {
                                    r#type: "button",
                                    class: "radio__btn",
                                    title: s.radio.previous_station,
                                    "aria-label": s.radio.previous_station,
                                    onclick: move |_| {
                                        if total > 0 {
                                            play_station((selected() + total - 1) % total);
                                        }
                                    },
                                    img {
                                        class: "radio__btn-icon",
                                        src: SKIP_BACK_ICON,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "group radio__play",
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
                                        class: "radio__play-icon",
                                        src: if playing() { PAUSE_ICON } else { PLAY_ICON },
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "radio__btn",
                                    title: s.radio.next_station,
                                    "aria-label": s.radio.next_station,
                                    onclick: move |_| {
                                        if total > 0 {
                                            play_station((selected() + 1) % total);
                                        }
                                    },
                                    img {
                                        class: "radio__btn-icon",
                                        src: SKIP_FORWARD_ICON,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                            }

                            // VOLUME
                            div { class: "radio__volume",
                                div { class: "radio__volume-icon-box",
                                    img {
                                        class: "radio__volume-icon",
                                        src: VOLUME_ICON,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                }
                                div { class: "radio__volume-track",
                                    div {
                                        class: "radio__volume-fill",
                                        style: "width: {volume_pct}%;",
                                        "aria-hidden": "true",
                                    }
                                    input {
                                        class: "radio__volume-input",
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
                        class: "radio__progress",
                        "aria-label": s.radio.playback_status,
                        div { class: "radio__progress-track",
                            div {
                                class: if playing() { "radio__progress-bar radio__progress-bar--on" } else { "radio__progress-bar radio__progress-bar--off" },
                                "aria-hidden": "true",
                            }
                        }
                        div { class: "radio__meta",
                            span { class: "radio__meta-text",
                                "{s.radio.live}"
                            }
                            span { class: "radio__meta-text",
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
                        class: "radio__link",
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
