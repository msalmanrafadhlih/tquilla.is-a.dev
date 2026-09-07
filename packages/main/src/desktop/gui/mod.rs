use dioxus::prelude::*;

mod calculator;
use calculator::CalculatorApp;

/// Every application launchable from the dock. `label()` drives both the
/// window title bar and the `title` tooltip on its dock icon.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppId {
    Dvd,
    Chill,
    FileManager,
    Browser,
    AiAssistant,
    Calculator,
}

impl AppId {
    pub const ALL: [AppId; 6] = [
        AppId::Dvd,
        AppId::Chill,
        AppId::FileManager,
        AppId::Browser,
        AppId::AiAssistant,
        AppId::Calculator,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            AppId::Dvd => "Screensaver",
            AppId::Chill => "Chill",
            AppId::FileManager => "File Manager",
            AppId::Browser => "Browser",
            AppId::AiAssistant => "AI Assistant",
            AppId::Calculator => "Calculator",
        }
    }
}

fn open_in_new_tab(url: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url_and_target(url, "_blank");
    }
}

/// The rounded-square icon used on the dock, in the window title bar, and
/// nowhere else — one source of truth for every app glyph.
#[component]
pub fn AppIcon(app: AppId, size: u32) -> Element {
    let s = size.to_string();

    let glyph = match app {
        AppId::Dvd => rsx! {
            g {
                path { d: "M20 37.9166C29.8951 37.9166 37.9166 29.8951 37.9166 20C37.9166 10.1049 29.8951 2.08331 20 2.08331C10.1049 2.08331 2.08331 10.1049 2.08331 20C2.08331 29.8951 10.1049 37.9166 20 37.9166Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M20 24.1625C22.2989 24.1625 24.1625 22.2989 24.1625 20C24.1625 17.7011 22.2989 15.8375 20 15.8375C17.7011 15.8375 15.8375 17.7011 15.8375 20C15.8375 22.2989 17.7011 24.1625 20 24.1625Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M20 2.08331V13.3658M20 26.4858V37.9166M9.45834 5.51248L16.0808 14.6133M23.8275 25.2591L30.5417 34.4866M5.57251 9.37665L14.6917 16.0916M25.2817 23.89L34.4275 30.6233", stroke: "white" }
                path { d: "M20 26.6108C23.6511 26.6108 26.6108 23.6511 26.6108 20C26.6108 16.3489 23.6511 13.3892 20 13.3892C16.3489 13.3892 13.3892 16.3489 13.3892 20C13.3892 23.6511 16.3489 26.6108 20 26.6108Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
            }
        },
        AppId::Chill => rsx! {
            g {
                path { d: "M4.32001 9.51333C4.32001 6.33 9.47168 3.75 15.8267 3.75C22.1817 3.75 27.3325 6.33 27.3325 9.51333", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M7.53917 33.7541C7.53917 33.7541 7.46417 33.68 7.34167 33.5316C6.34072 32.2341 5.72426 30.6816 5.5625 29.0508C5.17667 25.9075 4.16417 12.5158 4.32084 9.51331", stroke: "white" }
                path { d: "M27.3333 9.51331C27.3333 9.51331 26.7725 22.19 26.5017 25.9133C26.2308 29.6366 26.2267 31.885 24.3783 33.4816C22.53 35.0783 19.645 36.25 15.8267 36.25C12.085 36.25 8.85834 35.1583 7.36584 33.58", stroke: "white" }
                path { d: "M26.9916 16.8667C28.0116 14.9475 31.3183 13.1583 32.9816 13.9C35.9183 15.2083 36.6091 21.9617 34.3775 25.2775C32.1458 28.5933 29.4016 30.5792 26.2441 29.1375", stroke: "white" }
                path { d: "M27.13 13.9501C27.9456 13.4596 28.8731 13.1857 29.8243 13.1545C30.7756 13.1233 31.719 13.3357 32.565 13.7717M26.8333 20.0967C27.2794 20.2396 27.7614 20.2215 28.1954 20.0455C28.6294 19.8694 28.9879 19.5466 29.2083 19.1334C29.95 17.6542 29.9508 16.6209 31.9917 17.1351C34.0325 17.6492 34.6967 24.1084 30.5067 26.2617C26.3167 28.4151 28.925 24.9334 26.5892 24.5659", stroke: "white" }
                path { d: "M30.3308 17.2C31.1025 17.0083 33.0558 23.6192 27.7475 25.8183", stroke: "white" }
                path { d: "M4.32001 9.51331C4.32001 12.6966 9.47168 15.2766 15.8267 15.2766C22.1817 15.2766 27.3325 12.6958 27.3325 9.51331", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M17.955 15.1783C17.955 15.1783 18.3134 13.6166 19.4625 13.6908C21.7842 13.8408 21.7709 21.6733 20.0692 21.68", stroke: "white", stroke_linecap: "round" }
                path { d: "M19.9875 19.9109L16.8666 22.8359L17 31.0359L23.8808 29.1709L23.3725 22.2667L19.9875 19.9109Z", stroke: "white", stroke_linejoin: "round" }
            }
        },
        AppId::FileManager => rsx! {
            g {
                path { d: "M33.4683 12.4966V7.77328L15.9391 7.77162C15.9085 7.77118 15.8782 7.76472 15.8501 7.75259C15.822 7.74046 15.7965 7.7229 15.7751 7.70093C15.7538 7.67895 15.7369 7.65298 15.7256 7.6245C15.7143 7.59602 15.7087 7.56559 15.7091 7.53495L15.665 6.56245C15.5908 5.41578 14.8066 4.63745 13.7441 4.63745H6.50665C5.99697 4.63789 5.50827 4.84044 5.14772 5.20068C4.78717 5.56092 4.5842 6.04944 4.58331 6.55912V35.3625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M7.79081 32.4084C7.69665 34.0875 5.80998 35.1767 4.58331 35.3634L33.4933 35.3609C34.0034 35.3604 34.4925 35.1575 34.8531 34.7968C35.2137 34.436 35.4164 33.9468 35.4166 33.4367V13.1384C35.4166 12.9682 35.349 12.805 35.2287 12.6846C35.1084 12.5643 34.9452 12.4967 34.775 12.4967H8.43498C8.2648 12.4967 8.10159 12.5643 7.98125 12.6846C7.86092 12.805 7.79331 12.9682 7.79331 13.1384L7.79081 32.4084Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
            }
        },
        AppId::Browser => rsx! {
            g {
                path { d: "M20.6358 33.6275C20.1408 33.6797 19.6444 33.7058 19.1467 33.7058M19.1467 33.7058C11.3333 33.7067 5 27.3725 5 19.5583C5 11.7442 11.3333 5.40916 19.1467 5.40916C26.6 5.40916 32.7075 11.1742 33.2533 18.49M19.1467 33.7058V5.40833M32.1908 14.7867H6.13M21.4667 24.3283H6.13M30.3233 28.1833C30.3233 28.845 29.7867 29.3817 29.1242 29.3817C28.8062 29.3814 28.5013 29.255 28.2765 29.0302C28.0516 28.8053 27.9252 28.5005 27.925 28.1825C27.9252 27.8644 28.0517 27.5594 28.2768 27.3345C28.5018 27.1096 28.8069 26.9833 29.125 26.9833C29.443 26.9838 29.7478 27.1104 29.9725 27.3354C30.1971 27.5604 30.3233 27.8654 30.3233 28.1833Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M20.6592 33.4208C20.1765 33.6071 19.664 33.7037 19.1467 33.7058C14.9675 33.7058 11.5792 27.3717 11.5792 19.5575C11.5792 11.7433 14.9675 5.40918 19.1458 5.40918C23.0008 5.40918 26.2 10.8067 26.6575 17.83M29.125 29.55V31.6217", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M24.9225 25.2834V23.9734C24.9214 23.4483 25.024 22.9282 25.2244 22.4429C25.4249 21.9577 25.7192 21.5167 26.0904 21.1455C26.4617 20.7742 26.9026 20.4799 27.3879 20.2795C27.8732 20.0791 28.3933 19.9764 28.9184 19.9775H29.3309C29.8558 19.9766 30.3758 20.0792 30.861 20.2797C31.3462 20.4802 31.7871 20.7745 32.1582 21.1457C32.5294 21.517 32.8236 21.9579 33.024 22.4431C33.2244 22.9284 33.327 23.4484 33.3259 23.9734V25.285M24.7467 25.3942H33.5017C34.3317 25.3942 35 26.0625 35 26.8925V33.0925C35 33.9225 34.3317 34.5909 33.5017 34.5909H24.7467C24.5498 34.5913 24.3548 34.5529 24.1728 34.4777C23.9908 34.4026 23.8254 34.2922 23.6862 34.153C23.547 34.0138 23.4367 33.8484 23.3615 33.6665C23.2864 33.4845 23.2479 33.2894 23.2484 33.0925V26.8925C23.2484 26.0625 23.9167 25.3942 24.7467 25.3942Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
            }
        },
        AppId::AiAssistant => rsx! {
            g {
                path { d: "M20 37.0834C24.4348 37.0834 28.03 29.4349 28.03 20C28.03 10.5652 24.4348 2.91669 20 2.91669C15.5651 2.91669 11.97 10.5652 11.97 20C11.97 29.4349 15.5651 37.0834 20 37.0834Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M34.7951 28.5417C37.0126 24.7009 32.1859 17.7626 24.0151 13.0459C19.1117 10.2142 14.1176 8.82672 10.4284 9.00005C7.97089 9.11505 6.09255 9.92255 5.20589 11.4584C2.98839 15.3001 7.81422 22.2367 15.9851 26.9542C24.1559 31.6717 32.5776 32.3826 34.7951 28.5417Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M17.0251 17.7092H22.9751M16.0951 20H23.9051M18.2842 22.2909H21.7159M24.0151 26.9542C15.8442 31.6717 7.42339 32.3825 5.20589 28.5417C2.98839 24.7009 7.81422 17.7634 15.9851 13.0459C20.7034 10.3209 25.5051 8.93338 29.1476 8.98671C31.8126 9.02504 33.8576 9.83504 34.7951 11.4584C37.0117 15.2992 32.1867 22.2367 24.0151 26.9542Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
            }
        },
        AppId::Calculator => rsx! {
            g {
                path { d: "M20 4.58337V35.4167M4.58337 20H35.4167M23.9167 12.8334H31.4167M9.41671 24.4167L16.0834 31M16 24.4167L9.33337 31.0834M12.75 9.08337V16.5834M9.00004 12.8334H16.5M7.91671 4.58337H32.0834C32.9674 4.58337 33.8153 4.93456 34.4404 5.55968C35.0655 6.18481 35.4167 7.03265 35.4167 7.91671V32.0834C35.4167 32.9674 35.0655 33.8153 34.4404 34.4404C33.8153 35.0655 32.9674 35.4167 32.0834 35.4167H7.91671C7.03265 35.4167 6.18481 35.0655 5.55968 34.4404C4.93456 33.8153 4.58337 32.9674 4.58337 32.0834V7.91671C4.58337 7.03265 4.93456 6.18481 5.55968 5.55968C6.18481 4.93456 7.03265 4.58337 7.91671 4.58337Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M23.75 23.3334L21.9166 29L25.9166 33.3334L31.6666 32.0834L33.5 26.4167L29.5 22.0834L23.75 23.3334Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M23.75 23.3334L25 28.5L25.9167 33.3334L29.75 29.75L33.5 26.4167L28.5833 24.9167L23.75 23.3334Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M29.75 29.75L25 28.5M29.75 29.75L28.5833 24.9167M29.75 29.75L31.6666 32.0834M25 28.5L28.5833 24.9167M25 28.5L21.9166 29M28.5833 24.9167L29.5 22.0834", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
            }
        },
    };

    rsx! {
        svg {
            width: "{s}",
            height: "{s}",
            view_box: "0 0 40 40",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",
            rect { x: "0.5", y: "0.5", width: "39", height: "39", rx: "9.5", stroke: "white" }
            {glyph}
        }
    }
}

/// The window chrome shared by every app: title bar with the app's icon +
/// name + a close button, then the app-specific body underneath. Keyed by
/// `app` from the caller so switching apps remounts the body and replays
/// the `window-pop-in` entrance animation.
#[component]
pub fn GuiApps(app: AppId, on_close: EventHandler<()>) -> Element {
    let body = match app {
        AppId::Dvd => rsx! { DvdApp {} },
        AppId::Chill => rsx! { ChillApp {} },
        AppId::FileManager => rsx! { FileManagerApp {} },
        AppId::Browser => rsx! { BrowserApp {} },
        AppId::AiAssistant => rsx! { AiAssistantApp {} },
        AppId::Calculator => rsx! { CalculatorApp {} },
    };

    rsx! {
        div {
            key: "{app:?}",
            class: "window-pop-in flex h-full w-full flex-col overflow-hidden",

            div { class: "flex shrink-0 items-center justify-between gap-2 border-b border-white/25 px-3 py-2",
                div { class: "flex min-w-0 items-center gap-2",
                    AppIcon { app, size: 20 }
                    span { class: "truncate text-sm tracking-wide font-['JetBrains_Mono']", "{app.label()}" }
                }
                button {
                    r#type: "button",
                    "aria-label": "Tutup jendela",
                    class: "grid h-6 w-6 shrink-0 place-items-center rounded-sm border border-white/30 text-xs text-white/60 transition-all duration-150 ease-out hover:border-white/80 hover:bg-white/10 hover:text-white active:scale-90",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div { class: "min-h-0 flex-1 overflow-auto p-4", {body} }
        }
    }
}

// --------------------------------------------------------------------
// Individual dock apps. Calculator is the one fully-functional app (see
// calculator.rs) — the rest are lightweight, honestly-labelled previews.
// --------------------------------------------------------------------

#[component]
fn DvdApp() -> Element {
    rsx! {
        div { class: "flex h-full flex-col gap-3",
            div { class: "relative min-h-[200px] flex-1 overflow-hidden rounded-md border border-white/15 bg-white/[0.02]",
                div {
                    class: "dvd-bounce absolute drop-shadow-[0_0_6px_rgba(255,255,255,0.25)]",
                    style: "left: 2%; top: 6%;",
                    AppIcon { app: AppId::Dvd, size: 36 }
                }
            }
            p { class: "text-center text-xs text-white/40 font-['JetBrains_Mono']",
                "Klasik. Belum pernah sekalipun nabrak pojokan dengan sempurna."
            }
        }
    }
}

const BAR_HEIGHTS: [(u32, u32, u32); 4] = [(0, 8, 0), (1, 13, 120), (2, 18, 240), (3, 11, 360)];

#[component]
fn ChillApp() -> Element {
    let mut playing = use_signal(|| false);

    rsx! {
        div { class: "flex h-full flex-col items-center justify-center gap-5 text-center",
            AppIcon { app: AppId::Chill, size: 56 }
            div {
                p { class: "text-sm tracking-wide font-['JetBrains_Mono']", "Lo-fi beats to refactor to" }
                p { class: "text-xs text-white/40 font-['JetBrains_Mono']", "tquilla radio · episode 12" }
            }

            div { class: "flex h-6 items-end gap-1",
                for (i , bar_height , bar_delay) in BAR_HEIGHTS {
                    span {
                        key: "{i}",
                        class: if playing() { "w-1 rounded-full bg-white/70 animate-pulse" } else { "w-1 rounded-full bg-white/20" },
                        style: "height: {bar_height}px; animation-delay: {bar_delay}ms;",
                    }
                }
            }

            div { class: "h-1 w-48 overflow-hidden rounded-full bg-white/10",
                div {
                    class: if playing() { "h-full w-1/3 bg-white/70 transition-all duration-700" } else { "h-full w-1/3 bg-white/30 transition-all duration-700" },
                }
            }

            button {
                r#type: "button",
                class: "rounded-full border border-white/40 px-5 py-1.5 text-xs tracking-wide transition-all duration-150 ease-out hover:border-white/80 hover:bg-white/10 active:scale-90 font-['JetBrains_Mono']",
                onclick: move |_| playing.set(!playing()),
                if playing() { "❚❚ Pause" } else { "▶ Play" }
            }
        }
    }
}

#[component]
fn FileManagerApp() -> Element {
    let entries: [(&str, &str); 5] = [
        ("drwxr-xr-x", "nixos-config"),
        ("drwxr-xr-x", "projects"),
        ("-rw-r--r--", "journal.md"),
        ("-rw-r--r--", "resume.pdf"),
        ("drwxr-xr-x", "notes"),
    ];
    let mut selected = use_signal(|| Option::<usize>::None);

    rsx! {
        div { class: "flex h-full flex-col gap-2",
            p { class: "text-xs text-white/40 font-['JetBrains_Mono']", "~/ $ ls -la" }
            div { class: "flex flex-col gap-0.5",
                for (i , (perm , name)) in entries.into_iter().enumerate() {
                    button {
                        key: "{name}",
                        r#type: "button",
                        class: if selected() == Some(i) { "flex items-center gap-3 rounded px-2 py-1.5 text-left text-sm bg-white/10 border border-white/30 transition-colors duration-150 font-['JetBrains_Mono']" } else { "flex items-center gap-3 rounded px-2 py-1.5 text-left text-sm bg-transparent border border-transparent hover:bg-white/5 transition-colors duration-150 font-['JetBrains_Mono']" },
                        onclick: move |_| {
                            selected.set(if selected() == Some(i) { None } else { Some(i) });
                        },
                        span { class: "text-white/40", "{perm}" }
                        span { "{name}" }
                    }
                }
            }
            p { class: "mt-auto text-xs text-white/30 font-['JetBrains_Mono']",
                "Pratinjau isi berkas belum tersedia di simulasi ini."
            }
        }
    }
}

#[component]
fn BrowserApp() -> Element {
    rsx! {
        div { class: "flex h-full flex-col gap-4",
            div { class: "flex items-center gap-2 rounded-md border border-white/25 px-3 py-2",
                span { class: "text-white/40", "🔒" }
                span { class: "truncate text-sm font-['JetBrains_Mono']", "tquilla.is-a.dev" }
            }

            p { class: "text-xs text-white/40 font-['JetBrains_Mono']",
                "Pratinjau di dalam jendela ini belum tersedia — buka situs aslinya di tab baru:"
            }

            div { class: "flex flex-wrap gap-2",
                button {
                    r#type: "button",
                    class: "rounded-md border border-white/70 bg-white/90 px-3 py-1.5 text-xs font-medium text-black transition-all duration-150 ease-out hover:bg-white active:scale-90 font-['JetBrains_Mono']",
                    onclick: move |_| open_in_new_tab("https://tquilla.is-a.dev"),
                    "Buka situs asli ↗",
                }
                button {
                    r#type: "button",
                    class: "rounded-md border border-white/30 px-3 py-1.5 text-xs transition-all duration-150 ease-out hover:border-white/80 hover:bg-white/10 active:scale-90 font-['JetBrains_Mono']",
                    onclick: move |_| open_in_new_tab("https://github.com/msalmanrafadhlih"),
                    "GitHub ↗",
                }
            }
        }
    }
}

#[component]
fn AiAssistantApp() -> Element {
    const REPLIES: [&str; 4] = [
        "Menarik! Sayangnya aku cuma AI tiruan di dalam simulasi ini.",
        "Fitur ini masih dalam pengembangan — nantikan update selanjutnya.",
        "Coba tanya Moch langsung ya, aku cuma dekorasi di sini.",
        "Hmm, aku belum terhubung ke otak yang asli. Masih placeholder!",
    ];

    let mut messages = use_signal(|| {
        vec![(false, "Halo! Aku masih dalam tahap pengembangan, tapi coba tanya sesuatu.".to_string())]
    });
    let mut draft = use_signal(String::new);
    let mut reply_cursor = use_signal(|| 0usize);

    let mut send = move || {
        let text = draft().trim().to_string();
        if text.is_empty() {
            return;
        }
        messages.with_mut(|m| m.push((true, text)));
        draft.set(String::new());

        let reply = REPLIES[reply_cursor() % REPLIES.len()];
        reply_cursor.set(reply_cursor() + 1);
        messages.with_mut(|m| m.push((false, reply.to_string())));
    };

    rsx! {
        div { class: "flex h-full flex-col gap-3",
            div { class: "flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto pr-1",
                for (i , (is_user , text)) in messages().into_iter().enumerate() {
                    div {
                        key: "{i}",
                        class: if is_user { "self-end max-w-[85%] rounded-md border border-white/30 bg-white/10 px-3 py-1.5 text-sm font-['JetBrains_Mono']" } else { "self-start max-w-[85%] rounded-md border border-white/15 bg-transparent px-3 py-1.5 text-sm text-white/80 font-['JetBrains_Mono']" },
                        "{text}"
                    }
                }
            }

            form {
                class: "flex shrink-0 gap-2",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    send();
                },
                input {
                    r#type: "text",
                    class: "min-w-0 flex-1 rounded-md border border-white/25 bg-transparent px-2.5 py-1.5 text-sm outline-none transition-colors duration-150 focus:border-white/70 font-['JetBrains_Mono']",
                    placeholder: "Ketik pesan…",
                    value: "{draft()}",
                    oninput: move |evt| draft.set(evt.value()),
                }
                button {
                    r#type: "submit",
                    class: "shrink-0 rounded-md border border-white/70 bg-white/90 px-3 py-1.5 text-xs font-medium text-black transition-all duration-150 ease-out hover:bg-white active:scale-90 font-['JetBrains_Mono']",
                    "Kirim",
                }
            }
        }
    }
}
