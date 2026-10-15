use crate::languages::{use_strings, Strings};
use dioxus::prelude::*;

const IMG: Asset = asset!("/assets/logo-FileManager.svg");

/// Folder di bawah `home/tquilla`. Nama tampilannya mengikuti bahasa aktif
/// (`Strings::filemanager`); identitasnya tetap varian enum ini.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Folder {
    Documents,
    Downloads,
    Musics,
    Pictures,
    Videos,
}

impl Folder {
    const ALL: [Folder; 5] = [
        Folder::Documents,
        Folder::Downloads,
        Folder::Musics,
        Folder::Pictures,
        Folder::Videos,
    ];

    fn name(self, s: &Strings) -> &'static str {
        match self {
            Folder::Documents => s.filemanager.documents,
            Folder::Downloads => s.filemanager.downloads,
            Folder::Musics => s.filemanager.musics,
            Folder::Pictures => s.filemanager.pictures,
            Folder::Videos => s.filemanager.videos,
        }
    }

    /// Stable key for rsx `key`s (not translated).
    fn key(self) -> &'static str {
        match self {
            Folder::Documents => "documents",
            Folder::Downloads => "downloads",
            Folder::Musics => "musics",
            Folder::Pictures => "pictures",
            Folder::Videos => "videos",
        }
    }
}

#[component]
pub fn FileManagerWindowContent() -> Element {
    const NAV_BUTTON_CLASS: &str = "relative flex items-end justify-center w-fit mt-[-1.00px] appearance-none border-0 bg-[var(--bg-secondary)] p-0 cursor-pointer [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-fg-main text-xs text-center tracking-[0] leading-[normal]";
    let s = use_strings();
    let mut current = use_signal(|| Option::<Folder>::None);
    let selected = current();
    let breadcrumb_suffix = selected.map(|f| f.name(s)).unwrap_or("");

    rsx! {
        section {
            "aria-label": s.filemanager.file_browser,
            class: "flex flex-row w-full h-full items-start justify-start flex relative overflow-hidden",

            // side panel
            aside {
                "aria-label": s.filemanager.places,
                class: "inline-flex flex-col w-[30%] h-full min-w-max max-w-[150px] items-center relative border-r border-solid border-[var(--fg-secondary)] ",

                button {
                    r#type: "button",
                    class: nav_item_class(selected.is_none()),
                    "aria-current": if selected.is_none() { "page" } else { "false" },
                    onclick: move |_| current.set(None),
                    "{s.filemanager.desktops}"
                }
                for folder in Folder::ALL {
                    button {
                        key: "{folder.key()}",
                        r#type: "button",
                        class: nav_item_class(selected == Some(folder)),
                        "aria-current": if selected == Some(folder) { "page" } else { "false" },
                        onclick: move |_| current.set(Some(folder)),
                        "{folder.name(s)}"
                    }
                }
            }

            // current directory
            section {
                class: "flex-col items-start gap-2.5 flex-1 self-stretch grow flex relative",
                "aria-label": s.filemanager.current_directory,

                // top bar: navigation + breadcrumb
                header {
                    class: "flex items-start gap-2.5 px-2.5 pb-2.5 relative self-stretch w-full flex-[0_0_auto] border-b [border-bottom-style:solid] border-[var(--fg-secondary)]",

                    button {
                        r#type: "button",
                        class: NAV_BUTTON_CLASS,
                        "aria-label": s.filemanager.go_back,
                        title: s.filemanager.go_back,
                        "<"
                    }
                    button {
                        r#type: "button",
                        class: NAV_BUTTON_CLASS,
                        "aria-label": s.filemanager.go_forward,
                        title: s.filemanager.go_forward,
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
                    "aria-label": s.filemanager.folder_contents,
                    class: "flex flex-col min-[400px]:grid min-[400px]:grid-cols-[repeat(auto-fill,minmax(90px,max-content))] justify-start content-start items-start gap-[25px] p-5 w-full h-full max-h-full overflow-y-auto min-h-[200px] relative",

                    if selected.is_none() {
                        for folder in Folder::ALL {
                            button {
                                key: "{folder.key()}",
                                r#type: "button",
                                role: "listitem",
                                class: "flex flex-col shrink-0 items-center gap-1.5 p-0 cursor-pointer hover:opacity-50 border-0 bg-[var(--bg-secondary)] w-[90px] opacity-70",
                                onclick: move |_| current.set(Some(folder)),
                                img {
                                    src: IMG,
                                    alt: folder.name(s),
                                    class: "hidden min-[400px]:block w-full",
                                }
                                span {
                                    class: "text-[var(--fg-main)] text-left min-[400px]:text-center text-xs w-full break-words",
                                    "{folder.name(s)}"
                                }
                            }
                        }
                    } else {
                        p {
                            class: "text-fg-secondary",
                            "{s.filemanager.empty}"
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
