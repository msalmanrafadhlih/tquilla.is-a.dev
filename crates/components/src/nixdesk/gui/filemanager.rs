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
    const NAV_BUTTON_CLASS: &str = "files__nav-btn";
    let s = use_strings();
    let mut current = use_signal(|| Option::<Folder>::None);
    let selected = current();
    let breadcrumb_suffix = selected.map(|f| f.name(s)).unwrap_or("");

    rsx! {
        section {
            "aria-label": s.filemanager.file_browser,
            class: "files",

            // side panel
            aside {
                "aria-label": s.filemanager.places,
                class: "files__sidebar ",

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
                class: "files__main",
                "aria-label": s.filemanager.current_directory,

                // top bar: navigation + breadcrumb
                header {
                    class: "files__header",

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
                        class: "files__path",
                        p {
                            class: "files__path-text",
                            title: "home/tquilla/{breadcrumb_suffix}",
                            "home/tquilla/{breadcrumb_suffix}"
                        }
                    }
                }

                // workspace
                div {
                    role: "list",
                    "aria-label": s.filemanager.folder_contents,
                    class: "files__grid",

                    if selected.is_none() {
                        for folder in Folder::ALL {
                            button {
                                key: "{folder.key()}",
                                r#type: "button",
                                role: "listitem",
                                class: "files__item",
                                onclick: move |_| current.set(Some(folder)),
                                img {
                                    src: IMG,
                                    alt: folder.name(s),
                                    class: "files__item-icon",
                                }
                                span {
                                    class: "files__item-name",
                                    "{folder.name(s)}"
                                }
                            }
                        }
                    } else {
                        p {
                            "{s.filemanager.empty}"
                        }
                    }
                }
            }
        }
    }
}

fn nav_item_class(active: bool) -> String {
    const NAV_ITEM_BASE: &str = "files__nav-item";

    if active {
        format!("{NAV_ITEM_BASE} files__nav-item--active")
    } else {
        format!("{NAV_ITEM_BASE} files__nav-item--idle")
    }
}
