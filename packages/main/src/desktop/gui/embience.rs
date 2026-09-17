use dioxus::prelude::*;
use serde::Deserialize;

use super::audio::set_audio_volume;
use crate::desktop::js_util::{eval_js, js_string_escape};

const EMBIENCE_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/embience.json"));

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
    idx: usize,
    sound_id: u32,
    url: String,
) {
    let mut arr = active();
    arr[idx] = true;
    active.set(arr);
    let vol = volumes()[idx];
    let audio_id = audio_id_for(sound_id);
    eval_js(format!(
        "window.__audio && (window.__audio.load('{audio_id}', '{}'), window.__audio.setVolume('{audio_id}', {vol}), window.__audio.play('{audio_id}'));",
        js_string_escape(&url)
    ));
}

fn turn_off(mut active: Signal<Vec<bool>>, idx: usize, sound_id: u32) {
    let mut arr = active();
    arr[idx] = false;
    active.set(arr);
    let audio_id = audio_id_for(sound_id);
    eval_js(format!("window.__audio && window.__audio.pause('{audio_id}');"));
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

    let any_active = active().iter().any(|a| *a);
    let master_label = if master_playing() && any_active { "\u{275A}\u{275A}" } else { "\u{25B6}" };

    rsx! {
        div { class: "flex flex-col h-full text-[11px]",
            for sound in sounds.iter() {
                audio { key: "{sound.embiences_id}", id: "{audio_id_for(sound.embiences_id)}", r#loop: true }
            }

            div { class: "flex-1 min-h-0 overflow-y-auto p-4",
                div { class: "grid grid-cols-3 gap-3",
                    for (idx , sound) in sounds.iter().enumerate() {
                        {
                            let sound_id = sound.embiences_id;
                            let url = sound.embiences_stream.clone();
                            let name = sound.embiences_name.clone();
                            let icon = sound.embiences_icon.clone();
                            let is_active = active()[idx];
                            let vol = volumes()[idx];
                            let url_for_toggle = url.clone();
                            rsx! {
                                div {
                                    key: "{sound_id}",
                                    class: "flex flex-col items-center gap-1.5 border px-2 py-3",
                                    class: if is_active { "border-white bg-white text-black" } else { "border-white/15 text-white/70 hover:border-white/30" },
                                    button {
                                        r#type: "button",
                                        class: "flex flex-col items-center gap-1.5 w-full cursor-pointer",
                                        onclick: move |_| {
                                            if is_active {
                                                turn_off(active, idx, sound_id);
                                            } else {
                                                turn_on(active, volumes, idx, sound_id, url_for_toggle.clone());
                                            }
                                        },
                                        img { src: "{icon}", alt: "{name}", class: "w-6 h-6" }
                                        span { class: "text-[10px] text-center", "{name}" }
                                    }
                                    if is_active {
                                        input {
                                            r#type: "range",
                                            min: "0",
                                            max: "1",
                                            step: "0.01",
                                            value: "{vol}",
                                            class: "w-full accent-black",
                                            oninput: move |evt: FormEvent| {
                                                let v: f64 = evt.value().parse().unwrap_or(0.6);
                                                let mut arr = volumes();
                                                arr[idx] = v;
                                                volumes.set(arr);
                                                set_audio_volume(&audio_id_for(sound_id), v);
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "shrink-0 border-t border-white/15 px-4 py-3 flex flex-col items-center gap-2",
                if !presets.is_empty() {
                    div { class: "flex items-center gap-2",
                        for preset in presets.iter() {
                            {
                                let preset_names = preset.embience_list.clone();
                                let label = preset.preset_label.clone();
                                let sounds_for_preset = sounds.clone();
                                rsx! {
                                    button {
                                        key: "{label}",
                                        r#type: "button",
                                        class: "border border-white/20 px-2.5 py-1 text-[10px] text-white/60 hover:text-white hover:border-white/40 transition-colors duration-150",
                                        onclick: move |_| {
                                            for (i , s) in sounds_for_preset.iter().enumerate() {
                                                if preset_names.contains(&s.embiences_name) {
                                                    turn_on(active, volumes, i, s.embiences_id, s.embiences_stream.clone());
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
                {
                    let sounds_for_master = sounds.clone();
                    rsx! {
                        button {
                            r#type: "button",
                            class: "w-9 h-9 flex items-center justify-center border border-white/30 hover:bg-white hover:text-black transition-colors duration-150",
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
                            "{master_label}"
                        }
                    }
                }
            }
        }
    }
}
