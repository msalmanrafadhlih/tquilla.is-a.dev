use crate::languages::use_strings;
use dioxus::prelude::*;

const NIXOS_ICON: Asset = asset!("/assets/logo-nixos.svg");

#[component]
pub fn InfoBlock() -> Element {
    let s = use_strings();
    // Blok kedua (`s.about.extra_info()`: Philosophy, Design, ...) belum dipakai
    // di UI; blok pertama dirender dua kali seperti sebelumnya.
    let system_info = s.about.system_info();
    rsx! {
        header {
            class: "flex items-center justify-center gap-2.5 relative w-full flex-[0_0_auto]",
            div { class: "flex w-[75px] h-16 items-center justify-center gap-2.5 relative aspect-[1.16]",
                img { src: NIXOS_ICON, class: "relative flex-1 grow aspect-[1.15]", width: "75", height: "64", alt: s.about.logo_alt }
            }
            div { class: "flex flex-col w-[67px] items-start relative",
                h2 { class: "relative flex items-center w-max [font:'Inter-Regular',Helvetica] font-normal text-fg-main text-xs tracking-[0] leading-[normal]",
                    "{s.about.title}"
                }
                a {
                    href: "https://nixos.org",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "relative flex items-center w-fit mr-[-11.00px] [font:'Inter-Regular',Helvetica] font-normal text-[#7db1ff] text-[10px] tracking-[0] leading-[normal] whitespace-nowrap",
                    "https://nixos.org",
                }
            }
        }
        div { class: "w-full text-left text-[11px] leading-relaxed",
            for (label , value) in system_info {
                div {
                    key: "{label}",
                    class: "flex justify-between gap-4 py-0.5 border-b border-white/5",
                    span { class: "text-white/40", "{label}:" }
                    span { class: "text-white/80 text-right", "{value}" }
                }
            }
        }
        div { class: "w-full text-left text-[11px] leading-relaxed",
            for (label , value) in system_info {
                div {
                    key: "{label}",
                    class: "flex justify-between gap-4 py-0.5 border-b border-white/5",
                    span { class: "text-white/40", "{label}:" }
                    span { class: "text-white/80 text-right", "{value}" }
                }
            }
        }
    }
}

/// "About" — a standalone info card.
#[component]
pub fn AboutWindowContent() -> Element {
    rsx! {
        div { class: "flex w-full h-full items-center justify-center overflow-auto",
            section {
                class: "max-w-[500px] flex w-full h-full flex-col items-center justify-start gap-[25px] pb-5 relative flex-1 grow self-stretch",
                class: "overflow-auto [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",

                InfoBlock {}
            }
        }
    }
}
