use dioxus::prelude::*;
use serde::Deserialize;

use super::audio::set_audio_volume;
use crate::nixdesk::js_util::{eval_js, js_string_escape};

const EMBIENCE_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/embience.json"));

// Path icon footer (sama seperti di embience.html).
// Ganti ke `asset!("/assets/icon-xxx.svg")` kalau project kamu memakai asset pipeline Dioxus.
const ICON_VOLUME: Asset = asset!("/assets/icon-volume.svg");
const ICON_PLAY: Asset = asset!("/assets/icon-play.svg");
const ICON_PAUSE: Asset = asset!("/assets/icon-pause.svg");
const ICON_PLAYLIST: Asset = asset!("/assets/icon-playlist.svg");

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Sound {
    embiences_id: u32,
    embiences_name: String,
    embiences_icon: String,
    embiences_stream: String,
}

#[derive(Debug, Deserialize, Clone)]
struct EmbienceFile {
    embiences: Vec<Sound>,
    presets: Vec<Preset>,
}

#[derive(Debug, Deserialize, Clone)]
struct Preset {
    preset_label: String,
    embience_list: Vec<String>,
}

fn load_data() -> EmbienceFile {
    // Jaga-jaga kalau ada baris komentar `//` di file JSON, karena plain JSON
    // gak support komentar.
    let cleaned: String = EMBIENCE_JSON
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::from_str(&cleaned).unwrap_or(EmbienceFile { embiences: vec![], presets: vec![] })
}

fn audio_id_for(sound_id: u32) -> String {
    format!("emb-{sound_id}")
}

fn turn_on(
    mut active: Signal<Vec<bool>>,
    volumes: Signal<Vec<f64>>,
    muted: Signal<bool>,
    mut master_playing: Signal<bool>,
    idx: usize,
    sound_id: u32,
    url: String,
) {
    let mut arr = active();
    arr[idx] = true;
    active.set(arr);
    let vol = if muted() { 0.0 } else { volumes()[idx] };
    let audio_id = audio_id_for(sound_id);
    eval_js(format!(
        "window.__audio && (window.__audio.load('{audio_id}', '{}'), window.__audio.setVolume('{audio_id}', {vol}), window.__audio.play('{audio_id}'));",
        js_string_escape(&url)
    ));
    master_playing.set(true);
}

fn turn_off(
    mut active: Signal<Vec<bool>>,
    mut master_playing: Signal<bool>,
    idx: usize,
    sound_id: u32,
) {
    let mut arr = active();
    arr[idx] = false;
    let none_left = !arr.iter().any(|a| *a);
    active.set(arr);
    let audio_id = audio_id_for(sound_id);
    eval_js(format!("window.__audio && window.__audio.pause('{audio_id}');"));
    if none_left {
        master_playing.set(false);
    }
}

#[component]
pub fn EmbienceWindowContent() -> Element {
    let data = use_hook(load_data);
    let sounds = data.embiences;
    let presets = data.presets;
    let sound_count = sounds.len();

    let mut active = use_signal(|| vec![false; sound_count]);
    let mut volumes = use_signal(|| vec![0.6f64; sound_count]);
    let mut master_playing = use_signal(|| false);
    let mut muted = use_signal(|| false);
    let mut show_presets = use_signal(|| false);

    let any_active = active().iter().any(|a| *a);
    let is_playing = master_playing() && any_active;

    let sounds_for_mute = sounds.clone();
    let sounds_for_master = sounds.clone();

    rsx! {
        main {
            class: "relative flex flex-col w-full h-full items-start gap-5 bg-[var(--bg-secondary)] text-[var(--fg-main,#fff)]",
            "aria-label": "Embience",

            for sound in sounds.iter() {
                audio { key: "{sound.embiences_id}", id: "{audio_id_for(sound.embiences_id)}", r#loop: true }
            }

            // MAIN CONTENTS : AVAILABLE AUDIOS
            section {
                class: "grid w-full h-full min-h-0 content-start grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2.5 border-b border-solid border-[var(--fg-secondary,#808080)] justify-center overflow-y-scroll [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
                "aria-label": "Available ambient sound themes",

                for (idx , sound) in sounds.iter().enumerate() {
                    {
                        let sound_id = sound.embiences_id;
                        let url = sound.embiences_stream.clone();
                        let name = sound.embiences_name.clone();
                        let icon = sound.embiences_icon.clone();
                        let is_active = active()[idx];
                        let vol = volumes()[idx];
                        let bar_width = if is_active { (vol * 100.0).round() } else { 0.0 };
                        rsx! {
                            div {
                                key: "{sound_id}",
                                class: "relative w-full min-w-0 h-max flex flex-col items-center justify-center gap-5 p-2.5 text-inherit",
                                class: if is_active { "bg-[image:var(--conic-gradient)]" } else { "bg-[var(--bg-secondary)]" },

                                // toggle on/off: logo + title
                                button {
                                    r#type: "button",
                                    class: "w-full flex flex-col items-center justify-center gap-5 appearance-none border-0 p-0 bg-transparent text-inherit cursor-pointer",
                                    "aria-label": "Select {name} ambience",
                                    "aria-pressed": "{is_active}",
                                    onclick: move |_| {
                                        if is_active {
                                            turn_off(active, master_playing, idx, sound_id);
                                        } else {
                                            turn_on(active, volumes, muted, master_playing, idx, sound_id, url.clone());
                                        }
                                    },

                                    // logo audio
                                    span {
                                        class: "flex flex-col w-[100px] h-[100px] items-center justify-center gap-2.5 p-2.5 relative aspect-[1]",
                                        class: if is_active { "bg-[var(--fg-main,#fff)]" } else { "bg-[var(--bg-secondary,#000)]" },
                                        img {
                                            class: "relative self-stretch w-full aspect-[1]",
                                            class: if is_active { "invert" } else { "" },
                                            src: "{icon}",
                                            alt: if is_active { String::new() } else { name.clone() },
                                        }
                                    }

                                    // title audio
                                    span {
                                        class: "relative flex items-center justify-center w-fit [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-main,#fff)] text-xs text-center tracking-[0] leading-[normal]",
                                        "{name}"
                                    }
                                }

                                // volume bar (aktif: sesuai volume, nonaktif: 0% dengan lebar minimum 3%)
                                span {
                                    class: "relative self-stretch w-full h-[5px] bg-[var(--fg-secondary,#808080)]",
                                    class: if is_active { "" } else { "opacity-50" },
                                    span {
                                        class: "block h-full min-w-[3%] bg-[var(--fg-main,#fff)]",
                                        style: "width: {bar_width}%;",
                                    }
                                    if is_active {
                                        // slider transparan di atas bar supaya volume tetap bisa diatur
                                        input {
                                            r#type: "range",
                                            min: "0",
                                            max: "1",
                                            step: "0.01",
                                            value: "{vol}",
                                            "aria-label": "{name} volume",
                                            class: "absolute left-0 -top-2 m-0 w-full h-[21px] opacity-0 cursor-pointer",
                                            oninput: move |evt: FormEvent| {
                                                let v: f64 = evt.value().parse().unwrap_or(0.6);
                                                let mut arr = volumes();
                                                arr[idx] = v;
                                                volumes.set(arr);
                                                if !muted() {
                                                    set_audio_volume(&audio_id_for(sound_id), v);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // PLAYLISTS (preset), tampil saat tombol playlist ditekan
            if show_presets() && !presets.is_empty() {
                div { class: "flex flex-wrap items-center justify-center gap-2.5 w-full",
                    for preset in presets.iter() {
                        {
                            let preset_names = preset.embience_list.clone();
                            let label = preset.preset_label.clone();
                            let sounds_for_preset = sounds.clone();

                            // preset aktif = sound yang menyala persis sama dengan isi preset
                            let current = active();
                            let is_preset_active = any_active
                                && sounds.iter().enumerate().all(|(i, s)| {
                                    current[i] == preset_names.contains(&s.embiences_name)
                                });

                            rsx! {
                                button {
                                    key: "{label}",
                                    r#type: "button",
                                    "aria-pressed": "{is_preset_active}",
                                    class: if is_preset_active {
                                        "appearance-none px-2.5 py-1 border border-solid border-[var(--fg-main,#fff)] bg-[var(--fg-main,#fff)] [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-xs text-[var(--bg-secondary,#000)] cursor-pointer"
                                    } else {
                                        "appearance-none bg-[var(--bg-secondary)] px-2.5 py-1 border border-solid border-[var(--fg-secondary,#808080)] [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-xs text-[var(--fg-main,#fff)] cursor-pointer"
                                    },
                                    onclick: move |_| {
                                        let currently_active = active();
                                        let was_playing = master_playing();

                                        for (i, s) in sounds_for_preset.iter().enumerate() {
                                            let in_preset = preset_names.contains(&s.embiences_name);

                                            if in_preset {
                                                if !(currently_active[i] && was_playing) {
                                                    turn_on(
                                                        active,
                                                        volumes,
                                                        muted,
                                                        master_playing,
                                                        i,
                                                        s.embiences_id,
                                                        s.embiences_stream.clone(),
                                                    );
                                                }
                                            } else if currently_active[i] {
                                                turn_off(active, master_playing, i, s.embiences_id);
                                            }
                                        }
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }

            // FOOTER: AUDIO PLAYBACK
            footer {
                class: "flex items-center justify-center gap-[25px] relative self-stretch w-full flex-[0_0_auto]",
                "aria-label": "Playback controls",

                // MUTE/UNMUTE BUTTON
                button {
                    r#type: "button",
                    class: "relative w-6 h-6 aspect-[1] border-0 p-0 bg-[var(--bg-secondary)] cursor-pointer",
                    "aria-label": "Toggle volume",
                    "aria-pressed": "{muted()}",
                    onclick: move |_| {
                        let now_muted = !muted();
                        muted.set(now_muted);
                        let current_active = active();
                        let current_volumes = volumes();
                        for (i , s) in sounds_for_mute.iter().enumerate() {
                            if current_active[i] {
                                let v = if now_muted { 0.0 } else { current_volumes[i] };
                                set_audio_volume(&audio_id_for(s.embiences_id), v);
                            }
                        }
                    },
                    img {
                        class: "w-[100%] h-[100%] aspect-[1]",
                        class: if muted() { "opacity-50" } else { "" },
                        src: ICON_VOLUME,
                        alt: "",
                    }
                }

                // PLAYER BUTTON
                button {
                    r#type: "button",
                    class: "group flex w-[50px] h-[50px] items-center justify-center relative bg-[var(--bg-secondary)] aspect-[1] border-0 p-0 cursor-pointer hover:bg-[var(--fg-main)] disabled:opacity-40 disabled:cursor-default",
                    "aria-label": if is_playing { "Pause ambience" } else { "Play ambience" },
                    disabled: !any_active,
                    onclick: move |_| {
                        let now_playing = !master_playing();
                        master_playing.set(now_playing);
                        let current_active = active();
                        for (i , s) in sounds_for_master.iter().enumerate() {
                            if current_active[i] {
                                let audio_id = audio_id_for(s.embiences_id);
                                if now_playing {
                                    eval_js(format!("window.__audio && window.__audio.play('{audio_id}');"));
                                } else {
                                    eval_js(format!("window.__audio && window.__audio.pause('{audio_id}');"));
                                }
                            }
                        }
                    },
                    img {
                        class: "group-hover:invert relative w-[18px] h-[18px] aspect-[1]",
                        src: if is_playing { ICON_PAUSE } else { ICON_PLAY },
                        alt: "",
                    }
                }

                // PLAYLISTS
                button {
                    r#type: "button",
                    class: "relative w-6 h-6 aspect-[1] border-0 p-0 bg-[var(--bg-secondary)] cursor-pointer",
                    "aria-label": "Open playlist",
                    "aria-expanded": "{show_presets()}",
                    onclick: move |_| show_presets.set(!show_presets()),
                    img {
                        class: "w-[100%] h-[100%] aspect-[1]",
                        src: ICON_PLAYLIST,
                        alt: "",
                    }
                }
            }
        }
    }
}
