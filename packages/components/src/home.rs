use api_types::validate::is_http_url;
use dioxus::prelude::*;

use crate::api::{self, ApiError, Generation};
use crate::app::Route;
use crate::languages::use_strings;
use crate::platform::{use_platform, PlatformServices, TimeoutFuture};
use crate::remote::{LoadFailed, Loading};
use crate::shared::PageMeta;

/// What a menu entry's `link` turns into.
// No `Debug`: `Route` does not implement it.
#[derive(PartialEq)]
enum LinkTarget {
    /// A page of this app, opened through the router on every platform.
    Route(Route),
    /// A web address, handed to the platform package (same tab on web,
    /// system browser on desktop, ...).
    External(String),
    /// Anything else: not opened. The link comes from the server, so an
    /// unknown path or `javascript:` must not fall through to "open
    /// externally".
    Blocked,
}

fn link_target(link: &str) -> LinkTarget {
    if link.starts_with('/') {
        // `//host` and `/\host` are absolute URLs for a browser, not paths.
        if link.starts_with("//") || link.starts_with("/\\") {
            return LinkTarget::Blocked;
        }
        return link
            .parse::<Route>()
            .map_or(LinkTarget::Blocked, LinkTarget::Route);
    }
    if is_http_url(link) {
        LinkTarget::External(link.to_owned())
    } else {
        LinkTarget::Blocked
    }
}

fn open_link(platform: PlatformServices, url: &str) {
    match link_target(url) {
        LinkTarget::Route(route) => {
            navigator().push(route);
        }
        LinkTarget::External(url) => (platform.open_external)(&url),
        LinkTarget::Blocked => {}
    }
}

/// The boot menu comes from `GET /api/v1/generations`. While it loads, or if
/// it cannot be loaded, the page shows that state plus two plain links, so the
/// site stays reachable when the server is down.
#[component]
pub fn Home() -> Element {
    let s = use_strings();
    let mut generations = use_resource(api::generations);
    let state = generations.read_unchecked();

    let body = match &*state {
        None => rsx! {
            Loading { label: s.home.loading }
        },
        Some(Ok(list)) if !list.is_empty() => rsx! {
            BootMenu { generations: list.clone() }
        },
        other => {
            // An empty list is as useless as an error: nothing to boot.
            let error = match other {
                Some(Err(e)) => e.clone(),
                _ => ApiError::Decode("empty list".into()),
            };
            rsx! {
                LoadFailed {
                    error,
                    headline: s.home.failed,
                    on_retry: move |_| generations.restart(),
                    nav {
                        class: "flex flex-col gap-1 text-[var(--fg-main,#fff)] underline",
                        "aria-label": s.home.list_label,
                        Link { to: Route::DesktopPage {}, "{s.home.open_desktop}" }
                        Link { to: Route::JournalPage {}, "{s.home.open_journal}" }
                    }
                }
            }
        }
    };

    rsx! {
        PageMeta {
            title: s.meta.home_title,
            description: s.meta.home_description,
            path: "/",
        }
        if matches!(&*state, Some(Ok(list)) if !list.is_empty()) {
            {body}
        } else {
            section {
                class: "min-h-screen w-full bg-[var(--bg-main)] text-neutral-200 font-mono flex items-center justify-center px-3 py-10",
                {body}
            }
        }
    }
}

#[component]
fn BootMenu(generations: Vec<Generation>) -> Element {
    let s = use_strings();
    let platform = use_platform();
    let open = use_callback(move |url: String| open_link(platform, &url));
    let total = generations.len();
    // `links` dipisah supaya handler keyboard bisa mengambil URL yang sedang
    // dipilih tanpa meminjam `generations` ke dalam closure 'static.
    let links: Vec<String> = generations.iter().map(|g| g.link.clone()).collect();
    let mut number: f32 = 1.01;
    let mut selected = use_signal(|| 0usize);
    // Tracks which row the mouse is currently over, independent of
    // keyboard `selected`, so the label swap only reacts to hover.
    let mut hovered = use_signal(|| Option::<usize>::None);

    let mut counting = use_signal(|| 10i32);
    // Seperti timeout GRUB: interaksi apa pun (tombol, hover, sentuh, klik)
    // membatalkan auto-boot, supaya halaman tidak berpindah sendiri saat
    // pengunjung masih membaca.
    let mut cancelled = use_signal(|| false);
    // Elemen daftar, disimpan agar fokus bisa dikembalikan setelah klik di
    // area kosong (keyboard handler hidup di daftar, bukan di <section>).
    let mut list_ref = use_signal(|| Option::<std::rc::Rc<MountedData>>::None);
    let last_link = links.first().cloned();
    // Jalan sekali saat mount. Setiap 1 detik kurangi counting; begas 0,
    // buka link generasi terakhir dan hentikan loop-nya. Berhenti seketika
    // kalau dibatalkan.
    use_effect(move || {
        let last_link = last_link.clone();
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                if cancelled() {
                    break;
                }
                let remaining = counting() - 1;
                counting.set(remaining);
                if remaining <= 0 {
                    if let Some(url) = &last_link {
                        open.call(url.clone());
                    }
                    break;
                }
            }
        });
    });
    let onkeydown = move |evt: KeyboardEvent| {
        cancelled.set(true);
        match evt.key() {
            Key::ArrowDown => {
                evt.prevent_default();
                if total > 0 {
                    selected.set((selected() + 1) % total);
                }
            }
            Key::ArrowUp => {
                evt.prevent_default();
                if total > 0 {
                    selected.set((selected() + total - 1) % total);
                }
            }
            Key::Character(c) if c.eq_ignore_ascii_case("j") => {
                if total > 0 {
                    selected.set((selected() + 1) % total);
                }
            }
            Key::Character(c) if c.eq_ignore_ascii_case("k") => {
                if total > 0 {
                    selected.set((selected() + total - 1) % total);
                }
            }
            Key::Enter => {
                if let Some(url) = links.get(selected()) {
                    open.call(url.clone());
                }
            }
            Key::Character(c) if c == " " => {
                if let Some(url) = links.get(selected()) {
                    open.call(url.clone());
                }
            }
            _ => {}
        }
    };

    // id elemen opsi yang sedang dipilih, untuk `aria-activedescendant`.
    let active_id = generations
        .get(selected())
        .map(|g| format!("gen-{}", g.number))
        .unwrap_or_default();

    rsx! {
        section {
            class: "min-h-screen w-full bg-[var(--bg-main)] text-neutral-200 font-mono flex flex-col items-center justify-center px-3 py-10 outline-none select-none",
            // Klik di area kosong menghilangkan fokus dari daftar; kembalikan.
            onclick: move |_| {
                cancelled.set(true);
                if let Some(el) = list_ref() {
                    spawn(async move {
                        let _ = el.set_focus(true).await;
                    });
                }
            },
            ontouchstart: move |_| cancelled.set(true),

            div {
                class: "w-full max-w-3xl",
                // Pola listbox ARIA: fokus tetap di <ul>, opsi aktif
                // diumumkan lewat `aria-activedescendant`. Ini juga mencegah
                // Enter/Space terpicu dua kali (sekali oleh tombol yang
                // fokus, sekali oleh handler keyboard).
                ul {
                    role: "listbox",
                    "aria-label": s.home.list_label,
                    "aria-activedescendant": "{active_id}",
                    tabindex: "0",
                    class: "flex flex-col outline-none",
                    onkeydown,
                    onmounted: move |evt| {
                        let data = evt.data();
                        list_ref.set(Some(data.clone()));
                        spawn(async move {
                            let _ = data.set_focus(true).await;
                        });
                    },
                    for (idx, gen) in generations.iter().enumerate() {
                        li {
                            key: "{gen.number}",
                            id: "gen-{gen.number}",
                            role: "option",
                            "aria-selected": "{selected() == idx}",
                            onmouseenter: move |_| {
                                cancelled.set(true);
                                selected.set(idx);
                                hovered.set(Some(idx));
                            },
                            onmouseleave: move |_| {
                                if hovered() == Some(idx) {
                                    hovered.set(None);
                                }
                            },
                            onclick: {
                                let url = gen.link.clone();
                                move |_| open.call(url.clone())
                            },
                            class: if selected() == idx {
                                "bg-neutral-200 text-black px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 text-center"
                            } else {
                                "bg-[var(--bg-main)] text-neutral-200 px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 hover:bg-neutral-800 text-center"
                            },
                            if hovered() == Some(idx) {
                                "{gen.label}"
                            } else {
                                "{s.home.generation_line(gen.number, &number, &gen.kernel, &gen.built_on)}"
                                { number += 0.01 }
                            }
                        }
                    }
                }
                p {
                    class: "text-center text-[11px] sm:text-sm py-2 border-t border-neutral-700 text-neutral-500",
                    "{s.home.reboot_firmware}"
                }
            }
            p {
                class: "mt-6 text-center text-[10px] sm:text-xs text-neutral-600 max-w-md",
                if cancelled() {
                    "{s.home.auto_boot_cancelled}"
                } else {
                    "{s.home.auto_boot_countdown(counting())}"
                }
            }
            p {
                class: "mt-6 text-center text-[10px] sm:text-xs text-neutral-600 max-w-md",
                "{s.home.hint}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_links_go_through_the_router() {
        assert!(link_target("/deisktify") == LinkTarget::Route(Route::DesktopPage {}));
        assert!(link_target("/profile") == LinkTarget::Route(Route::JournalPage {}));
        assert!(link_target("/") == LinkTarget::Route(Route::Home {}));
    }

    #[test]
    fn web_links_are_external() {
        assert!(
            link_target("https://github.com/x")
                == LinkTarget::External("https://github.com/x".into())
        );
    }

    #[test]
    fn links_from_the_server_never_fall_through_to_the_browser() {
        // An unknown path used to be opened as an "external" URL.
        for bad in [
            "/no-such-page",
            "//evil.example",
            "/\\evil.example",
            "javascript:alert(1)",
            "data:text/html,x",
            "file:///etc/passwd",
            "HTTPS://shout.example",
            "",
            "github.com",
        ] {
            assert!(link_target(bad) == LinkTarget::Blocked, "{bad:?}");
        }
    }
}
