use dioxus::prelude::*;

const IMG: Asset = asset!("/assets/logo-FileManager.svg");
const FOLDER_NAMES: [&str; 5] = ["Documents", "Downloads", "Musics", "Pictures", "Videos"];

#[component]
pub fn FileManagerWindowContent() -> Element {
    const NAV_BUTTON_CLASS: &str = "relative flex items-end justify-center w-fit mt-[-1.00px] appearance-none border-0 bg-[var(--bg-secondary)] p-0 cursor-pointer [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-fg-main text-xs text-center tracking-[0] leading-[normal]";
    let mut current = use_signal(|| Option::<&'static str>::None);
    let selected = current();
    let breadcrumb_suffix = selected.unwrap_or("");

    rsx! {
        section {
            "aria-label": "File browser",
            class: "flex flex-row w-full h-full items-start justify-start flex relative overflow-hidden",

            // side panel
            aside {
                "aria-label": "Places",
                class: "inline-flex flex-col w-[30%] h-full min-w-max max-w-[150px] items-center relative border-r border-solid border-[var(--fg-secondary)] ",

                button {
                    r#type: "button",
                    class: nav_item_class(selected.is_none()),
                    "aria-current": if selected.is_none() { "page" } else { "false" },
                    onclick: move |_| current.set(None),
                    "Desktops"
                }
                for name in FOLDER_NAMES {
                    button {
                        key: "{name}",
                        r#type: "button",
                        class: nav_item_class(selected == Some(name)),
                        "aria-current": if selected == Some(name) { "page" } else { "false" },
                        onclick: move |_| current.set(Some(name)),
                        "{name}"
                    }
                }
            }

            // current directory
            section {
                class: "flex-col items-start gap-2.5 flex-1 self-stretch grow flex relative",
                "aria-label": "Current directory",

                // top bar: navigation + breadcrumb
                header {
                    class: "flex items-start gap-2.5 px-2.5 pb-2.5 relative self-stretch w-full flex-[0_0_auto] border-b [border-bottom-style:solid] border-[var(--fg-secondary)]",

                    button {
                        r#type: "button",
                        class: NAV_BUTTON_CLASS,
                        "aria-label": "Go back",
                        title: "Go back",
                        "<"
                    }
                    button {
                        r#type: "button",
                        class: NAV_BUTTON_CLASS,
                        "aria-label": "Go forward",
                        title: "Go forward",
                        ">"
                    }

                    div {
                        class: "items-center gap-2.5 flex-1 self-stretch grow flex relative",
                        p {
                            class: "items-end justify-center w-fit [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-secondary)]  text-xs text-center tracking-[0] leading-[normal] flex relative whitespace-nowrap",
                            title: "home/tquilla/{breadcrumb_suffix}",
                            "home/tquilla/{breadcrumb_suffix}"
                        }
                    }
                }

                // workspace
                div {
                    role: "list",
                    "aria-label": "Folder contents",
                    class: "flex flex-col min-[400px]:grid min-[400px]:grid-cols-[repeat(auto-fill,minmax(90px,max-content))] justify-start content-start items-start gap-[25px] p-5 w-full h-full max-h-full overflow-y-auto min-h-[200px] relative",

                    if selected.is_none() {
                        for name in FOLDER_NAMES {
                            button {
                                key: "{name}",
                                r#type: "button",
                                role: "listitem",
                                class: "flex flex-col shrink-0 items-center gap-1.5 p-0 cursor-pointer hover:opacity-50 border-0 bg-[var(--bg-secondary)] w-[90px] opacity-70",
                                onclick: move |_| current.set(Some(name)),
                                img {
                                    src: IMG,
                                    alt: name,
                                    class: "hidden min-[400px]:block w-full",
                                }
                                span {
                                    class: "text-[var(--fg-main)] text-left min-[400px]:text-center text-xs w-full break-words",
                                    "{name}"
                                }
                            }
                        }
                    } else {
                        p {
                            class: "text-fg-secondary",
                            "(empty)"
                        }
                    }
                }
            }
        }
    }
}

fn nav_item_class(active: bool) -> String {
    const NAV_ITEM_BASE: &str = "w-full p-2 border-0 cursor-pointer [font:'JetBrains_Mono-ExtraLight',Helvetica] font-normal text-xs text-left tracking-[0] leading-[normal] flex relative";

    if active {
        format!("{NAV_ITEM_BASE} text-[var(--bg-secondary)] bg-[var(--fg-main)]")
    } else {
        format!(
            "{NAV_ITEM_BASE} bg-[var(--bg-secondary)] text-[var(--fg-secondary)] hover:text-[var(--fg-main)]"
        )
    }
}
