use dioxus::prelude::*;

const CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn Booting() -> Element {
   rsx! {
      document::Link { rel: "stylesheet", href: CSS }

      "Hello World!!" 
   } 
}
