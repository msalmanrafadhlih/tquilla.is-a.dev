use dioxus::prelude::*;

const GEN_CSS: Asset = asset!("/assets/styling/generation.css");

#[component]
pub fn GenerationMenu() -> Element {
    // struct generations {
    //     gen: String,
    //     nixos_version: String,
    //     kernel: String,
    //     Built_on: String
    // }

    rsx! {
        document::Link { rel: "stylesheet", href: GEN_CSS }

        div {
            div { 
                ul { id: "generation",
                    li { a {  "NixOs (Generation 11 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 10 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 9 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 8 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 7 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 6 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 5 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 4 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 3 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 2 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                    li { a {  "NixOs (Generation 1 NixOS 26.11.2979.47c003416129, Linux Kernel 5.15.97, Built on 2026-03-07)"  } }
                }
                p {  "Reboot Into Firmware Interface" }
            }
            p {  "Boot in 20s" }
        }
    }
}
