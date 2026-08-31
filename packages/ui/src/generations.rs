use dioxus::prelude::*;

const GEN_CSS: Asset = asset!("/assets/styling/hero.css");

#[component]
pub fn GenerationMenu() -> Element {
    struct generations {
        gen: String,
        nixos_version: String,
        kernel: String,
        Built_on: String
    }

    rsx! {
        document::Link { rel: "stylesheet", href: GEN_CSS }

        div {
            div { id: "links",
                ul { 
                    li {
                        a { 
                            href: "https://msalmanrafadhlih.github.io/tquilla.is-a.dev/generation_2/",
                            "NixOs (Generation 2 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"
                        }
                    }
                    li {
                        a { 
                            href: "https://msalmanrafadhlih.github.io/tquilla.is-a.dev/generation_1/",
                            "NixOs (Generation 1 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"
                        }
                    }
                }
                p {  "Reboot Into Firmware Interface" }
            }
            p {  "Boot in 20s" }
        }
    }
}
