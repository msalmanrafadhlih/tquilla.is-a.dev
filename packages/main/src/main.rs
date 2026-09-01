use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            div { 
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
