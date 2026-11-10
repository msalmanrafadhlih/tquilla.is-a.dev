use dioxus::prelude::*;

use super::audio::set_audio_volume;
use crate::api::{self, Embience, Preset};
use crate::languages::use_strings;
use crate::nixdesk::js_util::{eval_js, js_string_escape};
use crate::remote::LoadFailed;
use crate::skeleton::{SkeletonGroup, SkeletonRows};

// Path icon footer (sama seperti di embience.html).
// Ganti ke `asset!("/assets/icon-xxx.svg")` kalau project kamu memakai asset pipeline Dioxus.
const ICON_VOLUME: Asset = asset!("/assets/icon-volume.svg");
const ICON_PLAY: Asset = asset!("/assets/icon-play.svg");
const ICON_PAUSE: Asset = asset!("/assets/icon-pause.svg");
const ICON_PLAYLIST: Asset = asset!("/assets/icon-playlist.svg");

// Ikon tiap suara ada sebagai aset lokal. Sebelumnya `embiences_icon` di JSON
// menunjuk ke raw.githubusercontent.com, jadi tiap ikon = 1 request ke host
// pihak ketiga. Dicocokkan lewat nama file di URL (mis. `.../logo-rain.svg`);
// URL yang tidak dikenal tetap dipakai apa adanya.
const ICON_BIRDS: Asset = asset!("/assets/logo-birds.svg");
const ICON_BOAT: Asset = asset!("/assets/logo-boat.svg");
const ICON_CITY: Asset = asset!("/assets/logo-city.svg");
const ICON_COFFEE_SHOP: Asset = asset!("/assets/logo-coffee_shop.svg");
const ICON_FIREPLACE: Asset = asset!("/assets/logo-fireplace.svg");
const ICON_RAIN: Asset = asset!("/assets/logo-rain.svg");
const ICON_STORM: Asset = asset!("/assets/logo-storm.svg");
const ICON_STREAM: Asset = asset!("/assets/logo-stream.svg");
const ICON_SUMMER_NIGHT: Asset = asset!("/assets/logo-summer_night.svg");
const ICON_TRAIN: Asset = asset!("/assets/logo-train.svg");
const ICON_WAVE: Asset = asset!("/assets/logo-wave.svg");
const ICON_WIND: Asset = asset!("/assets/logo-wind.svg");

fn icon_src(url: &str) -> String {
    let file = url.rsplit('/').next().unwrap_or("");
    let local = match file {
        "logo-birds.svg" => ICON_BIRDS,
        "logo-boat.svg" => ICON_BOAT,
        "logo-city.svg" => ICON_CITY,
        "logo-coffee_shop.svg" => ICON_COFFEE_SHOP,
        "logo-fireplace.svg" => ICON_FIREPLACE,
        "logo-rain.svg" => ICON_RAIN,
        "logo-storm.svg" => ICON_STORM,
        "logo-stream.svg" => ICON_STREAM,
        "logo-summer_night.svg" => ICON_SUMMER_NIGHT,
        "logo-train.svg" => ICON_TRAIN,
        "logo-wave.svg" => ICON_WAVE,
        "logo-wind.svg" => ICON_WIND,
        _ => return url.to_string(),
    };
    local.to_string()
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
    eval_js(format!(
        "window.__audio && window.__audio.pause('{audio_id}');"
    ));
    if none_left {
        master_playing.set(false);
    }
}

/// Loads the sounds and the presets, then hands them to [`EmbienceBoard`].
///
/// The sounds are required. The presets are a convenience: if only they fail,
/// the board still opens, without the playlist buttons.
#[component]
pub fn EmbienceWindowContent() -> Element {
    let mut sounds = use_resource(api::embiences);
    let mut presets = use_resource(api::presets);
    let sounds_state = sounds.read_unchecked();
    let presets_state = presets.read_unchecked();
    match (&*sounds_state, &*presets_state) {
        (Some(Err(e)), _) => rsx! {
            LoadFailed {
                error: e.clone(),
                on_retry: move |_| {
                    sounds.restart();
                    presets.restart();
                },
            }
        },
        (Some(Ok(list)), Some(loaded)) => {
            let presets = api::sanitize_presets(loaded.clone().unwrap_or_default(), list);
            rsx! {
                EmbienceBoard { sounds: list.clone(), presets }
            }
        }
        _ => rsx! {
            SkeletonGroup { class: "app-window__loading",
                SkeletonRows { rows: 5 }
            }
        },
    }
}

#[component]
fn EmbienceBoard(sounds: Vec<Embience>, presets: Vec<Preset>) -> Element {
    let s = use_strings();
    let sound_count = sounds.len();

    let active = use_signal(|| vec![false; sound_count]);
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
            class: "embience",
            "aria-label": s.embience.label,

            for sound in sounds.iter() {
                audio { key: "{sound.id}", id: "{audio_id_for(sound.id)}", r#loop: true }
            }

            // MAIN CONTENTS : AVAILABLE AUDIOS
            section {
                class: "embience__grid",
                "aria-label": s.embience.themes_label,

                for (idx , sound) in sounds.iter().enumerate() {
                    {
                        let sound_id = sound.id;
                        let url = sound.stream_url.clone();
                        let name = sound.name.clone();
                        let icon = icon_src(&sound.icon_url);
                        let is_active = active()[idx];
                        let vol = volumes()[idx];
                        let bar_width = if is_active { (vol * 100.0).round() } else { 0.0 };
                        rsx! {
                            div {
                                key: "{sound_id}",
                                class: "embience__card",
                                class: if is_active { "embience__card--on" } else { "embience__card--off" },

                                // toggle on/off: logo + title
                                button {
                                    r#type: "button",
                                    class: "embience__card-btn",
                                    "aria-label": "{s.embience.select_ambience(&name)}",
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
                                        class: "embience__thumb-box",
                                        class: if is_active { "embience__thumb--on" } else { "embience__thumb--off" },
                                        img {
                                            class: "embience__thumb",
                                            class: if is_active { "embience__thumb-img--invert" } else { "" },
                                            src: "{icon}",
                                            alt: if is_active { String::new() } else { name.clone() },
                                        }
                                    }

                                    // title audio
                                    span {
                                        class: "embience__name",
                                        "{name}"
                                    }
                                }

                                // volume bar (aktif: sesuai volume, nonaktif: 0% dengan lebar minimum 3%)
                                span {
                                    class: "embience__volume",
                                    class: if is_active { "" } else { "embience__dim" },
                                    span {
                                        class: "embience__volume-fill",
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
                                            "aria-label": "{s.embience.volume_of(&name)}",
                                            class: "embience__volume-input",
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
                div { class: "embience__presets",
                    for preset in presets.iter() {
                        {
                            let preset_ids = preset.embience_ids.clone();
                            let label = preset.label.clone();
                            let sounds_for_preset = sounds.clone();

                            // preset aktif = sound yang menyala persis sama dengan isi preset
                            let current = active();
                            let is_preset_active = any_active
                                && sounds.iter().enumerate().all(|(i, s)| {
                                    current[i] == preset_ids.contains(&s.id)
                                });

                            rsx! {
                                button {
                                    key: "{preset.id}",
                                    r#type: "button",
                                    "aria-pressed": "{is_preset_active}",
                                    class: if is_preset_active {
                                        "embience__chip embience__chip--on"
                                    } else {
                                        "embience__chip embience__chip--off"
                                    },
                                    onclick: move |_| {
                                        let currently_active = active();
                                        let was_playing = master_playing();

                                        for (i, s) in sounds_for_preset.iter().enumerate() {
                                            let in_preset = preset_ids.contains(&s.id);

                                            if in_preset {
                                                if !(currently_active[i] && was_playing) {
                                                    turn_on(
                                                        active,
                                                        volumes,
                                                        muted,
                                                        master_playing,
                                                        i,
                                                        s.id,
                                                        s.stream_url.clone(),
                                                    );
                                                }
                                            } else if currently_active[i] {
                                                turn_off(active, master_playing, i, s.id);
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
                class: "embience__footer",
                "aria-label": s.embience.playback_controls,

                // MUTE/UNMUTE BUTTON
                button {
                    r#type: "button",
                    class: "embience__btn",
                    "aria-label": s.embience.toggle_volume,
                    "aria-pressed": "{muted()}",
                    onclick: move |_| {
                        let now_muted = !muted();
                        muted.set(now_muted);
                        let current_active = active();
                        let current_volumes = volumes();
                        for (i , s) in sounds_for_mute.iter().enumerate() {
                            if current_active[i] {
                                let v = if now_muted { 0.0 } else { current_volumes[i] };
                                set_audio_volume(&audio_id_for(s.id), v);
                            }
                        }
                    },
                    img {
                        class: "embience__btn-icon",
                        class: if muted() { "embience__dim" } else { "" },
                        src: ICON_VOLUME,
                        alt: "",
                    }
                }

                // PLAYER BUTTON
                button {
                    r#type: "button",
                    class: "group embience__play",
                    "aria-label": if is_playing { s.embience.pause_ambience } else { s.embience.play_ambience },
                    disabled: !any_active,
                    onclick: move |_| {
                        let now_playing = !master_playing();
                        master_playing.set(now_playing);
                        let current_active = active();
                        for (i , s) in sounds_for_master.iter().enumerate() {
                            if current_active[i] {
                                let audio_id = audio_id_for(s.id);
                                if now_playing {
                                    eval_js(format!("window.__audio && window.__audio.play('{audio_id}');"));
                                } else {
                                    eval_js(format!("window.__audio && window.__audio.pause('{audio_id}');"));
                                }
                            }
                        }
                    },
                    img {
                        class: "embience__play-icon",
                        src: if is_playing { ICON_PAUSE } else { ICON_PLAY },
                        alt: "",
                    }
                }

                // PLAYLISTS
                button {
                    r#type: "button",
                    class: "embience__btn",
                    "aria-label": s.embience.open_playlist,
                    "aria-expanded": "{show_presets()}",
                    onclick: move |_| show_presets.set(!show_presets()),
                    img {
                        class: "embience__btn-icon",
                        src: ICON_PLAYLIST,
                        alt: "",
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// File names of the icons that ship with the app (see the `ICON_*`
    /// constants above); the server stores them as `.../logo-<name>.svg`.
    const LOCAL_ICONS: [&str; 12] = [
        "birds",
        "boat",
        "city",
        "coffee_shop",
        "fireplace",
        "rain",
        "storm",
        "stream",
        "summer_night",
        "train",
        "wave",
        "wind",
    ];

    #[test]
    fn every_shipped_icon_resolves_to_a_local_asset() {
        // icon_src falls back to the remote URL for unknown file names, so a
        // result equal to the input means a missing local mapping.
        for name in LOCAL_ICONS {
            let url = format!("https://raw.githubusercontent.com/x/y/main/logo-{name}.svg");
            assert_ne!(icon_src(&url), url, "no local icon for {name}");
        }
    }

    #[test]
    fn unknown_icon_urls_are_kept_as_is() {
        assert_eq!(
            icon_src("https://example.com/other.svg"),
            "https://example.com/other.svg"
        );
    }

    #[test]
    fn audio_ids_follow_the_sound_id() {
        assert_eq!(audio_id_for(7), "emb-7");
    }
}
