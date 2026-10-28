# tquilla.is-a.dev: panduan khusus project ini

Bagian ini mendahului panduan umum Dioxus 0.7 di bawah. Baca ini dulu.

- **Web adalah satu-satunya platform yang sudah diuji di runtime.** `platform/desktop` sudah dijalankan di sandbox (Xvfb + WebKitGTK) tetapi belum dicoba Moch di mesin sungguhan, dan `platform/mobile` hanya compile + clippy (`platforms.yml`); jangan klaim "sudah jalan" untuk keduanya sebelum Moch mengujinya (README, bagian "Platform desktop dan mobile"). `build.rs` di keduanya hanya membuat `assets/tailwind.css` kosong kalau belum ada; `platform/web` sengaja tidak punya `build.rs` itu.
- **Fullstack = dua repo.** Frontend ini tetap di GitHub Pages. Backend (Axum di Cloudflare Workers lewat `workers-rs`, DB Turso) ada di repo terpisah dan publik; klien hanya memakai REST + JSON, **tanpa `#[server]`**. Repo backend: `msalmanrafadhlih/deisktify-worker.rs`; kontrak bersama (`api-types`) masuk lewat git dependency yang dikunci ke **tag** di `[workspace.dependencies]` (naikkan tag sengaja, lalu `cargo update -p api-types` dan commit `Cargo.lock`). Semua panggilan API lewat `components::api` (`get`/`post`/`health`/`messages`/`post_message`/`report_message`/`journal`, base URL dari variable build `API_BASE_URL`; semua variable build dibaca dengan `option_env!`, **jangan `env!`**, supaya CI, `nix build` dan `cargo test` tanpa variable tetap compile; nilai kosong/salah = state "belum dikonfigurasi", bukan host tebakan). Jurnal GitHub (`/profile`) ikut lewat `components::api::journal()` (`GET /api/v1/journal`); tidak ada data contoh bawaan dan tidak ada `STATS_API_URL` lagi. Jangan memanggil `platform::http` langsung untuk endpoint API. Server tidak boleh percaya data klien (validasi ulang panjang dan URL, rate limit), dan tidak ada unggah gambar (R2 dibatalkan karena butuh kartu): avatar adalah avatar bawaan dari username, jangan menyimpan base64 di DB. Rencana dan keputusan: README, bagian "Fullstack dan database". Kerjakan satu tahap per balasan dan centang README.
- **Teks UI: Inggris, semuanya di `components/src/languages/language.rs`.** Jangan menulis teks tampilan langsung di komponen: tambahkan kunci di tabel `strings!`, lalu pakai `let s = use_strings();` (`s.grup.kunci`). Teks bertemplate memakai `{nama}` dan method bertipe di bagian "Template". Pesan error berupa enum yang diterjemahkan saat tampil. Test `languages::tests::no_display_strings_left_in_components` menggagalkan CI kalau ada teks tampilan tertinggal; pengecualian (bukan teks UI) didaftarkan dengan alasan di `languages/tests.rs`.
- **Kolom teks tidak boleh diikat ke signal (`value: "{signal}"`).** Di webview desktop/seluler itu menghilangkan ketikan (peristiwa `input` tiba asinkron, DOM ditimpa nilai usang). DOM pemilik teks; `oninput` hanya menyalin ke signal; mengosongkan/mengisi dari kode lewat `nixdesk::js_util::set_field(id, nilai, hanya_jika)` (isi `Some(teks_terkirim)` supaya ketikan baru tidak terhapus). Elemen yang dibuat ulang (tab) memakai `initial_value`. Slider (`type=range`) boleh tetap terikat.
- **Rilis app (`release-apps.yml`, tag `app-v*`):** per arsitektur, jangan paksa semua arsitektur; yang dicakup dan yang sengaja tidak ada di komentar kepala workflow. Biner desktop `nixos-desktop`, mobile `nixos-mobile` (lewat `[[bin]]`; nama paket Cargo tetap `desktop`/`mobile`, dan `flake.nix` memanggil biner dengan nama itu). Tailwind harus dijalankan eksplisit sebelum `dx bundle` (dx tidak menjamin; tanpa itu app tanpa gaya). Android wajib `--target aarch64-linux-android`. Jangan menambah PNG kecil ke `[bundle].icon` desktop (bentrok dengan konversi `.ico`, RPM gagal). Job rilis yang belum pernah jalan di runner sungguhan harus ditulis "belum terverifikasi".
- **Tata letak workspace:** `crates/components` (semua UI), `crates/markdown` (mesin Markdown, Rust murni), `platform/{web,desktop,mobile}` (hanya launch + Tailwind + `PlatformServices`). Crate `markdown` **tidak boleh** bergantung pada Dioxus dan tidak boleh memuat teks UI (tes pemindai di `components` ikut memindai `crates/markdown/src`).
- **Perintah yang harus hijau sebelum commit** (sama dengan `.github/workflows/ci.yml`):
  ```sh
  cargo fmt --all -- --check
  cargo clippy -p components -p markdown --all-targets -- -D warnings
  cargo test -p components -p markdown
  node --test "crates/components/js/tests/*.test.js"
  python3 -m unittest discover -s scripts -p "test_*.py"
  ```
  Target `wasm32` tidak dikompilasi oleh perintah di atas (kode `cfg(target_arch = "wasm32")` hanya terkompilasi lewat `dx`).
- **JS yang dijalankan lewat `document::eval` ada di `crates/components/js/*.js`** dan di-embed dengan `include_str!`. Jangan menulis blok JS panjang di dalam string Rust. Isi file diperlakukan sebagai badan fungsi (boleh `return` di level atas). Tiap file punya test di `js/tests/` (DOM tiruan, tanpa browser).
- **Riwayat chat AI** (`nixdesk/gui/ai_history.rs`, `js/chat_store.js`): disimpan **hanya di IndexedDB browser pengguna**, tidak pernah ke server atau backend; jangan menambah kolom untuk kunci API atau data lain di sana (ekspor JSON dan penyimpanan hanya berisi judul, model, waktu, dan pesan). Logika murni (judul, urutan, batas `MAX_CHATS`, nama berkas, format ekspor) tetap di fungsi murni berikut tesnya; bentuk data yang ditulis/dibaca skrip diuji di kedua sisi (`ai_history::tests` dan `js/tests/chat_store.test.js`). Naikkan `EXPORT_VERSION` bila bentuk ekspor berubah.
- **Gambar di Guestbook (D10):** gambar dari host di luar daftar `markdown::images` (`TRUSTED_HOSTS`/`TRUSTED_SUFFIXES`) **tidak boleh dirender sebagai `<img>`** sebelum pembaca mengeklik; tidak ada permintaan jaringan tanpa klik. Pencocokan host hanya lewat `images::is_trusted_image`/`image_host` (jangan membandingkan string alamat sendiri), dan jangan menambah host yang bisa dipakai melacak (penghitung kunjungan, dll.). Tetap tidak ada unggah gambar atau base64 di DB.
- **Skeleton loading:** pakai `components::skeleton` (`SkeletonGroup` + `Skeleton*`) untuk keadaan memuat, jangan menulis kotak abu-abu sendiri di tiap layar. Bentuk di dalam `SkeletonGroup` otomatis disembunyikan dari pembaca layar; labelnya dari `Strings`.
- **Logika window manager itu murni** (`window.rs`: `open_or_focus_in`, `close_in`, ... atas `Vec<OpenWindow>`); wrapper `Signal` tipis. Tambah perilaku di fungsi murni lalu tes di sana.
- **Dua `Dioxus.toml`** (root dan `platform/web/`) sengaja dua-duanya dipertahankan karena belum terverifikasi mana yang dibaca `dx` untuk build web. Ubah **keduanya** bersamaan (bagian `[web.wasm_opt]`).
- **Ejaan `deisktify` disengaja** (URL, judul, komentar). `DISCORD_URL` sengaja invite komunitas. Brightness di navbar ditunda (nanti diganti popup daftar tema).
- Versi Rust dikunci di `rust-toolchain.toml`; saat menaikkannya, ubah juga `dtolnay/rust-toolchain@<versi>` di kedua workflow.

---

You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

# Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

# Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

* Props must be owned values, not references. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
* Props must implement `PartialEq` and `Clone`.
* To make props reactive and copy, you can wrap the type in `ReadOnlySignal`. Any reactive state like memos and resources that read `ReadOnlySignal` props will automatically re-run when the prop changes.

# State

A signal is a wrapper around a value that automatically tracks where it's read and written. Changing a signal's value causes code that relies on the signal to rerun.

## Local State

The `use_signal` hook creates state that is local to a single component. You can call the signal like a function (e.g. `my_signal()`) to clone the value, or use `.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its dependencies change. Memos are useful for expensive calculations that you don't want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

## Context API

The Context API allows you to share state down the component tree. A parent provides the state using `use_context_provider`, and any child can access it with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

# Async

For state that depends on an asynchronous operation (like a network request), Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of the async task and provides the result to your component.

* The `use_resource` hook takes an `async` closure. It re-runs this closure whenever any signals it depends on (reads) are updated
* The `Resource` object returned can be in several states when read:
1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

# Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`. Each variant represents a route and is annotated with `#[route("/path")]`. Dynamic Segments can capture parts of the URL path as parameters by using `:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and place an `Outlet<Route> {}` inside your layout component. The child routes will be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

# Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features (`server` and a client feature like `web`) to split the code into a server and client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

## Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only run on the server. On the server, this macro generates an API endpoint. On the client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

## Hydration

Hydration is the process of making a server-rendered HTML page interactive on the client. The server sends the initial HTML, and then the client-side runs, attaches event listeners, and takes control of future rendering.

### Errors
The initial UI rendered by the component on the client must be identical to the UI rendered on the server.

* Use the `use_server_future` hook instead of `use_resource`. It runs the future on the server, serializes the result, and sends it to the client, ensuring the client has the data immediately for its first render.
* Any code that relies on browser-specific APIs (like accessing `localStorage`) must be run *after* hydration. Place this code inside a `use_effect` hook.
- **Guestbook dan Markdown.** Entri adalah Markdown yang disimpan apa adanya dan diurai oleh crate `markdown` (`pulldown-cmark` → pohon sendiri) dan digambar di klien oleh `components::markdown_view` (pohon → komponen Dioxus). **Jangan pernah membuat string HTML atau memakai `dangerous_inner_html`** untuk isi entri. HTML di entri didukung hanya sebagai **subset daftar-putih**: `crates/markdown/src/html.rs` cuma memecah teks jadi token, dan `Builder::html_open` memutuskan tag mana yang berarti (lihat README D4). Tag/atribut baru = tambah di daftar putih *dan* test-nya; jangan pernah menyalin atribut mentah ke elemen (tidak ada `style`, `class`, `id`, `on*`, `srcset`). Aturan yang harus tetap berlaku (ada test-nya): tag di luar daftar dibuang dan `<script>`/`<style>`/`<iframe>` dibuang beserta isinya; gambar hanya dimuat dari `https://` dengan `referrerpolicy="no-referrer"`, lazy, dan batas ukuran; hanya `http(s)` dan `mailto:` polos yang jadi tautan; `rel="noopener noreferrer nofollow ugc"`; teks hasil dekode entitas tidak boleh memuat karakter kontrol/pembalik arah; kedalaman daftar/kutipan/`<div>`/inline dibatasi dan builder tidak boleh panik. Logika non-UI di `guestbook.rs`, UI di `nixdesk/gui/guestbook.rs`, toolbar editor di `js/guestbook_editor.js` (dengan test node). Tidak ada polling; muat saat dibuka, Refresh dan Load older manual.
