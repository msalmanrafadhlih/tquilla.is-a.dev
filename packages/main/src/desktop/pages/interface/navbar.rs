use dioxus::prelude::*;

const NIXOS_ICON: Asset = asset!("/assets/Icon-Nixos.svg");
const WIFI_ICON: Asset = asset!("/assets/button-wifi.svg");
const VOLUME_ICON: Asset = asset!("/assets/button-volume.svg");
const BRIGHTNESS_ICON: Asset = asset!("/assets/button-brightness.svg");
const BATTERY_ICON: Asset = asset!("/assets/button-battery.svg");
const NOTIFICATION_ICON: Asset = asset!("/assets/button-notification.svg");
const TERMINAL_ICON: Asset = asset!("/assets/button-terminal.svg");

// Decorative only for now — a real dropdown per item is more than this
// pass covers, matching classic desktop chrome without the behavior.
const MENU_ITEMS: [&str; 6] = ["File", "Edit", "View", "Go", "Tools", "Settings"];

#[component]
pub fn Navbar(time: Signal<String>, date: Signal<String>, on_toggle: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex items-center justify-between h-10 px-4 border-b border-white/15 shrink-0 text-xs",
            div { class: "flex items-center gap-4 min-w-0",
                img { src: NIXOS_ICON, alt: "NixDesktop", class: "w-4 h-4" }
                span { class: "font-semibold whitespace-nowrap", "NixDesktop" }
                div { class: "hidden md:flex items-center gap-3 text-white/50",
                    for item in MENU_ITEMS {
                        span {
                            key: "{item}",
                            class: "hover:text-white transition-colors duration-150 cursor-default",
                            "{item}"
                        }
                    }
                }
            }
            div { class: "hidden sm:flex flex-col items-center leading-tight",
                span { class: "tracking-wide", "{time}" }
                span { class: "text-[10px] text-white/40", "{date}" }
            }
            div { class: "flex items-center gap-3 shrink-0",
                img { src: WIFI_ICON, alt: "Wi-Fi", class: "w-3.5 h-3.5 opacity-70" }
                img { src: VOLUME_ICON, alt: "Volume", class: "w-3.5 h-3.5 opacity-70" }
                img { src: BRIGHTNESS_ICON, alt: "Brightness", class: "w-3.5 h-3.5 opacity-70" }
                img { src: BATTERY_ICON, alt: "Battery", class: "w-3.5 h-3.5 opacity-70" }
                img { src: NOTIFICATION_ICON, alt: "Notifications", class: "w-3.5 h-3.5 opacity-70" }
                button {
                    r#type: "button",
                    class: "flex items-center gap-1.5 border border-white/15 px-2 py-1 hover:bg-white hover:text-black transition-colors duration-150",
                    onclick: move |_| on_toggle.call(()),
                    img { src: TERMINAL_ICON, alt: "", class: "w-3 h-3" }
                    span { class: "text-[10px]", "Switch TTY" }
                }
            }
        }
    }
}
