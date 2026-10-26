//! Riwayat chat AI: model data, ekspor JSON, dan jembatan ke IndexedDB.
//!
//! - Riwayat disimpan **hanya di browser pengguna** (IndexedDB, lewat
//!   `js/chat_store.js`). Tidak ada yang dikirim ke server, dan tiap pengguna
//!   punya riwayatnya sendiri. Kunci API tidak punya tempat di sini: tidak ada
//!   kolomnya, jadi tidak bisa ikut tersimpan atau terekspor.
//! - Semua logika yang bisa dites tanpa browser (judul, urutan, batas jumlah,
//!   nama berkas, format ekspor, tanggal) ada di bagian atas sebagai fungsi murni.
//! - Bagian bawah (`list`, `load`, `save`, ...) hanya membungkus skrip JS dan
//!   tidak bisa jalan di host; tes-nya ada di `js/tests/chat_store.test.js`.

use std::sync::atomic::{AtomicU32, Ordering};

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Jumlah chat tersimpan maksimum; yang paling lama tidak diperbarui dibuang.
pub const MAX_CHATS: usize = 100;
/// Panjang judul (karakter) sebelum dipotong.
const TITLE_MAX_CHARS: usize = 40;
/// Panjang bagian nama di nama berkas ekspor.
const SLUG_MAX_CHARS: usize = 40;
/// Penanda dan versi berkas ekspor; naikkan versi bila bentuknya berubah.
pub const EXPORT_FORMAT: &str = "tquilla-ai-chat";
pub const EXPORT_VERSION: u32 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    /// Dinamai `model` seperti di Gemini; di berkas ekspor tampil `assistant`.
    #[serde(rename = "model")]
    Assistant,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: Role,
    pub text: String,
}

/// Satu percakapan lengkap, persis seperti yang disimpan.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    /// Kosong = belum ada judul (UI menampilkan teks "tanpa judul").
    pub title: String,
    pub model: String,
    /// Milidetik sejak epoch (`Date.now()`).
    pub created_at: i64,
    pub updated_at: i64,
    pub turns: Vec<ChatTurn>,
}

/// Ringkasan untuk daftar riwayat (tanpa isi pesan).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub title: String,
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub count: usize,
}

impl Conversation {
    /// Merakit percakapan dari keadaan jendela. Judul diambil dari pesan
    /// pengguna pertama, jadi tetap sama untuk chat yang sudah dibuka lagi.
    pub fn assemble(
        id: &str,
        created_at: i64,
        model: &str,
        turns: Vec<ChatTurn>,
        now: i64,
    ) -> Self {
        let title = turns
            .iter()
            .find(|t| t.role == Role::User)
            .map(|t| derive_title(&t.text))
            .unwrap_or_default();
        Self {
            id: id.to_owned(),
            title,
            model: model.to_owned(),
            created_at,
            updated_at: now,
            turns,
        }
    }

    pub fn summary(&self) -> Summary {
        Summary {
            id: self.id.clone(),
            title: self.title.clone(),
            model: self.model.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            count: self.turns.len(),
        }
    }
}

/// Judul dari teks pertama: spasi dan baris baru dipadatkan jadi satu spasi,
/// dipotong di [`TITLE_MAX_CHARS`] karakter (bukan byte) dengan `…` di ujung.
pub fn derive_title(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= TITLE_MAX_CHARS {
        return flat;
    }
    let mut out: String = flat.chars().take(TITLE_MAX_CHARS).collect();
    while out.ends_with(' ') {
        out.pop();
    }
    out.push('…');
    out
}

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

/// Kunci unik chat dari waktu dan penghitung (dua chat dalam milidetik yang
/// sama tetap berbeda).
pub fn new_id(now_ms: i64, counter: u32) -> String {
    format!("{now_ms:x}-{counter:x}")
}

/// [`new_id`] dengan waktu sekarang dan penghitung proses.
pub fn fresh_id() -> String {
    let now = crate::platform::Date::now() as i64;
    new_id(now, NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

/// Memasukkan atau mengganti ringkasan (menurut `id`), mengurutkan terbaru
/// dulu, dan memangkas ke [`MAX_CHATS`]. Mengembalikan id yang terbuang, supaya
/// pemanggil juga menghapusnya dari penyimpanan.
pub fn upsert(list: &mut Vec<Summary>, summary: Summary) -> Vec<String> {
    match list.iter_mut().find(|s| s.id == summary.id) {
        Some(existing) => *existing = summary,
        None => list.push(summary),
    }
    // Stabil: yang sama-sama terbaru tetap pada urutan masuknya.
    list.sort_by_key(|s| std::cmp::Reverse(s.updated_at));
    if list.len() > MAX_CHATS {
        list.split_off(MAX_CHATS)
            .into_iter()
            .map(|s| s.id)
            .collect()
    } else {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Tanggal (tanpa crate tambahan; UTC)
// ---------------------------------------------------------------------------

/// Hari sejak 1970-01-01 -> (tahun, bulan 1-12, tanggal). Algoritma "civil from
/// days" (Howard Hinnant), berlaku juga untuk tanggal sebelum 1970.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// `YYYY-MM-DDTHH:MM:SS.mmmZ` (UTC), seperti `Date.prototype.toISOString`.
pub fn iso_from_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let milli = ms.rem_euclid(1000);
    let (year, month, day) = civil_from_days(secs.div_euclid(86_400));
    let rest = secs.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{milli:03}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// `YYYYMMDD` (UTC) untuk nama berkas.
fn compact_date(ms: i64) -> String {
    let (year, month, day) = civil_from_days(ms.div_euclid(1000).div_euclid(86_400));
    format!("{year:04}{month:02}{day:02}")
}

// ---------------------------------------------------------------------------
// Ekspor JSON
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ExportMessage<'a> {
    role: &'static str,
    text: &'a str,
}

#[derive(Serialize)]
struct ExportChat<'a> {
    id: &'a str,
    title: &'a str,
    model: &'a str,
    created_at: String,
    updated_at: String,
    messages: Vec<ExportMessage<'a>>,
}

#[derive(Serialize)]
struct ExportFile<'a> {
    format: &'static str,
    version: u32,
    exported_at: String,
    chats: Vec<ExportChat<'a>>,
}

/// Isi berkas unduhan. Bentuknya sama untuk satu chat maupun semua chat:
/// `{format, version, exported_at, chats: [{id, title, model, created_at,
/// updated_at, messages: [{role: "user" | "assistant", text}]}]}`.
pub fn export_json(chats: &[Conversation], exported_at_ms: i64) -> String {
    let file = ExportFile {
        format: EXPORT_FORMAT,
        version: EXPORT_VERSION,
        exported_at: iso_from_ms(exported_at_ms),
        chats: chats
            .iter()
            .map(|c| ExportChat {
                id: &c.id,
                title: &c.title,
                model: &c.model,
                created_at: iso_from_ms(c.created_at),
                updated_at: iso_from_ms(c.updated_at),
                messages: c
                    .turns
                    .iter()
                    .map(|t| ExportMessage {
                        role: match t.role {
                            Role::User => "user",
                            Role::Assistant => "assistant",
                        },
                        text: &t.text,
                    })
                    .collect(),
            })
            .collect(),
    };
    // Tipe-tipe di atas selalu bisa diserialisasi; `{}` hanya jaring pengaman.
    serde_json::to_string_pretty(&file).unwrap_or_else(|_| String::from("{}"))
}

/// Huruf kecil ASCII dan angka; urutan karakter lain jadi satu `-`.
fn slug(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
        if out.len() >= SLUG_MAX_CHARS {
            break;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Nama berkas: `chat-<judul>-<YYYYMMDD>.json` untuk satu chat (tanggal
/// pembaruan terakhirnya), `chats-<YYYYMMDD>.json` untuk semua.
pub fn export_filename(chat: Option<&Conversation>, now_ms: i64) -> String {
    match chat {
        Some(c) => {
            let name = slug(&c.title);
            let date = compact_date(c.updated_at);
            if name.is_empty() {
                format!("chat-{date}.json")
            } else {
                format!("chat-{name}-{date}.json")
            }
        }
        None => format!("chats-{}.json", compact_date(now_ms)),
    }
}

// ---------------------------------------------------------------------------
// Jembatan ke IndexedDB (lewat document::eval)
// ---------------------------------------------------------------------------

const STORE_JS: &str = include_str!("../../../js/chat_store.js");
const DOWNLOAD_JS: &str = include_str!("../../../js/download_file.js");

/// Skrip untuk satu operasi penyimpanan; `op` dan `arg` ditulis sebagai
/// literal JSON sebelum badan skrip.
fn store_script(op: &str, arg: &Value) -> String {
    format!("const OP = {}; const ARG = {arg};\n{STORE_JS}", json!(op))
}

/// Membuka balasan skrip: `{ok: true, data}` atau `{ok: false, error}`.
fn unwrap_reply(reply: Value) -> Result<Value, String> {
    match reply.get("ok").and_then(Value::as_bool) {
        Some(true) => Ok(reply.get("data").cloned().unwrap_or(Value::Null)),
        Some(false) => Err(reply
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("storage error")
            .to_owned()),
        None => Err(String::from("unreadable reply")),
    }
}

async fn call(op: &str, arg: Value) -> Result<Value, String> {
    let mut eval = document::eval(&store_script(op, &arg));
    let reply = eval.recv::<Value>().await.map_err(|e| e.to_string())?;
    unwrap_reply(reply)
}

fn decode<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| e.to_string())
}

/// Ringkasan semua chat, terbaru dulu.
pub async fn list() -> Result<Vec<Summary>, String> {
    decode(call("list", json!({})).await?)
}

/// Satu chat lengkap, atau `None` bila sudah tidak ada.
pub async fn load(id: &str) -> Result<Option<Conversation>, String> {
    decode(call("get", json!({ "id": id })).await?)
}

/// Semua chat lengkap (untuk "unduh semua").
pub async fn load_all() -> Result<Vec<Conversation>, String> {
    decode(call("all", json!({})).await?)
}

/// Menyimpan (atau mengganti) satu chat beserta ringkasannya.
pub async fn save(chat: &Conversation) -> Result<(), String> {
    let arg = serde_json::to_value(chat).map_err(|e| e.to_string())?;
    call("put", arg).await.map(|_| ())
}

pub async fn delete(ids: &[String]) -> Result<(), String> {
    call("delete", json!({ "ids": ids })).await.map(|_| ())
}

/// Meminta browser mengunduh `content` sebagai berkas `filename` (JSON).
pub fn download_json(filename: &str, content: &str) {
    let script = format!(
        "const FILENAME = {}; const CONTENT = {}; const MIME = \"application/json\";\n{DOWNLOAD_JS}",
        json!(filename),
        json!(content),
    );
    spawn(async move {
        document::eval(&script).await.ok();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(role: Role, text: &str) -> ChatTurn {
        ChatTurn {
            role,
            text: text.to_owned(),
        }
    }

    fn summary(id: &str, updated_at: i64) -> Summary {
        Summary {
            id: id.to_owned(),
            title: format!("t-{id}"),
            model: String::from("m"),
            created_at: 0,
            updated_at,
            count: 2,
        }
    }

    fn conv(id: &str, title: &str, updated_at: i64) -> Conversation {
        Conversation {
            id: id.to_owned(),
            title: title.to_owned(),
            model: String::from("gemini-3.8-flash"),
            created_at: 1_761_436_800_000,
            updated_at,
            turns: vec![
                turn(Role::User, "hi \"there\"\nline 2"),
                turn(Role::Assistant, "hello ✓"),
            ],
        }
    }

    #[test]
    fn the_title_is_the_first_user_message_flattened() {
        assert_eq!(derive_title("  hello \n\n  world\t!  "), "hello world !");
        assert_eq!(derive_title(""), "");
        assert_eq!(derive_title(" \n "), "");
        let exactly = "a".repeat(TITLE_MAX_CHARS);
        assert_eq!(derive_title(&exactly), exactly);
    }

    #[test]
    fn a_long_title_is_cut_by_characters_not_bytes() {
        let long = "é".repeat(TITLE_MAX_CHARS + 10);
        let title = derive_title(&long);
        assert_eq!(title.chars().count(), TITLE_MAX_CHARS + 1);
        assert!(title.ends_with('…'));
        assert!(title.starts_with("éé"));
        // Tidak berakhir dengan spasi sebelum "…".
        let spaced = format!("{} {}", "a".repeat(TITLE_MAX_CHARS - 1), "b".repeat(20));
        let cut = derive_title(&spaced);
        assert!(!cut.contains(" …"));
        assert!(cut.ends_with('…'));
    }

    #[test]
    fn assemble_takes_the_title_from_the_first_user_turn_only() {
        let turns = vec![
            turn(Role::Assistant, "ignored"),
            turn(Role::User, "first question"),
            turn(Role::User, "second question"),
        ];
        let c = Conversation::assemble("id1", 5, "m", turns.clone(), 9);
        assert_eq!(c.title, "first question");
        assert_eq!((c.created_at, c.updated_at), (5, 9));
        assert_eq!(c.turns, turns);
        let empty = Conversation::assemble("id2", 5, "m", vec![], 9);
        assert_eq!(empty.title, "");
        assert_eq!(c.summary().count, 3);
        assert_eq!(c.summary().id, "id1");
    }

    #[test]
    fn ids_differ_within_the_same_millisecond() {
        assert_ne!(new_id(1_000, 0), new_id(1_000, 1));
        assert_ne!(new_id(1_000, 1), new_id(1_001, 1));
        assert_eq!(new_id(255, 10), "ff-a");
    }

    #[test]
    fn upsert_orders_newest_first_and_replaces_by_id() {
        let mut list = Vec::new();
        assert!(upsert(&mut list, summary("a", 10)).is_empty());
        upsert(&mut list, summary("b", 30));
        upsert(&mut list, summary("c", 20));
        let ids = |l: &[Summary]| l.iter().map(|s| s.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&list), ["b", "c", "a"]);
        // Chat lama dibuka dan dibalas lagi: naik ke atas, tidak ganda.
        upsert(&mut list, summary("a", 99));
        assert_eq!(ids(&list), ["a", "b", "c"]);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn upsert_drops_the_least_recently_updated_beyond_the_limit() {
        let mut list: Vec<Summary> = (0..MAX_CHATS as i64)
            .map(|i| summary(&format!("c{i}"), 1000 + i))
            .collect();
        // Persis di batas: tidak ada yang terbuang.
        assert!(upsert(&mut list, summary("c5", 5000)).is_empty());
        assert_eq!(list.len(), MAX_CHATS);
        // Satu baru: yang paling lama (c0) terbuang.
        let dropped = upsert(&mut list, summary("new", 6000));
        assert_eq!(dropped, ["c0"]);
        assert_eq!(list.len(), MAX_CHATS);
        assert_eq!(list[0].id, "new");
        assert!(list.iter().all(|s| s.id != "c0"));
        // Chat yang paling lama tapi baru saja dibalas tidak ikut terbuang.
        let dropped = upsert(&mut list, summary("c1", 7000));
        assert!(dropped.is_empty());
        assert_eq!(list[0].id, "c1");
    }

    #[test]
    fn a_new_chat_that_is_older_than_all_others_is_the_one_dropped() {
        let mut list: Vec<Summary> = (0..MAX_CHATS as i64)
            .map(|i| summary(&format!("c{i}"), 1000 + i))
            .collect();
        let dropped = upsert(&mut list, summary("ancient", 1));
        assert_eq!(dropped, ["ancient"]);
        assert_eq!(list.len(), MAX_CHATS);
    }

    #[test]
    fn iso_dates_match_the_javascript_format() {
        assert_eq!(iso_from_ms(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_from_ms(1_761_436_800_123), "2025-10-26T00:00:00.123Z");
        assert_eq!(iso_from_ms(951_782_400_000), "2000-02-29T00:00:00.000Z");
        assert_eq!(iso_from_ms(1_782_864_000_999), "2026-07-01T00:00:00.999Z");
        assert_eq!(iso_from_ms(-1000), "1969-12-31T23:59:59.000Z");
        assert_eq!(iso_from_ms(1_761_523_199_999), "2025-10-26T23:59:59.999Z");
        assert_eq!(compact_date(1_761_523_199_999), "20251026");
        assert_eq!(compact_date(1_761_523_200_000), "20251027");
    }

    #[test]
    fn the_export_has_a_stable_documented_shape() {
        let json = export_json(
            &[conv("abc", "Hello", 1_761_436_800_123)],
            1_782_864_000_000,
        );
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["format"], "tquilla-ai-chat");
        assert_eq!(v["version"], 1);
        assert_eq!(v["exported_at"], "2026-07-01T00:00:00.000Z");
        let chat = &v["chats"][0];
        assert_eq!(chat["id"], "abc");
        assert_eq!(chat["title"], "Hello");
        assert_eq!(chat["model"], "gemini-3.8-flash");
        assert_eq!(chat["created_at"], "2025-10-26T00:00:00.000Z");
        assert_eq!(chat["updated_at"], "2025-10-26T00:00:00.123Z");
        assert_eq!(
            chat["messages"][0],
            json!({"role": "user", "text": "hi \"there\"\nline 2"})
        );
        assert_eq!(
            chat["messages"][1],
            json!({"role": "assistant", "text": "hello ✓"})
        );
        // Hanya kolom ini: tidak ada kunci API atau data lain yang ikut.
        // (`Value` mengurutkan kunci menurut abjad; urutan di teks dicek di bawah.)
        let sorted_keys = |value: &Value| {
            let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
            keys.sort();
            keys
        };
        assert_eq!(
            sorted_keys(chat),
            [
                "created_at",
                "id",
                "messages",
                "model",
                "title",
                "updated_at"
            ]
        );
        assert_eq!(
            sorted_keys(&v),
            ["chats", "exported_at", "format", "version"]
        );
        let at = |needle: &str| json.find(needle).unwrap();
        assert!(at("\"format\"") < at("\"version\"") && at("\"version\"") < at("\"chats\""));
    }

    #[test]
    fn exporting_all_chats_and_none() {
        let all = export_json(&[conv("a", "A", 1), conv("b", "B", 2)], 0);
        let v: Value = serde_json::from_str(&all).unwrap();
        assert_eq!(v["chats"].as_array().unwrap().len(), 2);
        let none: Value = serde_json::from_str(&export_json(&[], 0)).unwrap();
        assert_eq!(none["chats"], json!([]));
    }

    #[test]
    fn file_names_are_safe_and_readable() {
        let day = 1_761_436_800_000; // 2025-10-26
        assert_eq!(
            export_filename(Some(&conv("a", "How do I fix: Rust/WASM?", day)), 0),
            "chat-how-do-i-fix-rust-wasm-20251026.json"
        );
        // Judul kosong atau tanpa huruf/angka ASCII: tidak ada bagian nama.
        assert_eq!(
            export_filename(Some(&conv("a", "", day)), 0),
            "chat-20251026.json"
        );
        assert_eq!(
            export_filename(Some(&conv("a", "日本語 ✓", day)), 0),
            "chat-20251026.json"
        );
        // Tidak ada pemisah jalur atau titik dari judul.
        let name = export_filename(Some(&conv("a", "../../etc/passwd", day)), 0);
        assert_eq!(name, "chat-etc-passwd-20251026.json");
        assert!(!name.contains('/') && !name.contains(".."));
        // Panjang dibatasi.
        let long = export_filename(Some(&conv("a", &"word ".repeat(40), day)), 0);
        assert!(long.len() <= "chat-".len() + SLUG_MAX_CHARS + "-20251026.json".len());
        // Semua chat.
        assert_eq!(export_filename(None, day), "chats-20251026.json");
    }

    #[test]
    fn the_stored_shape_round_trips_with_the_script() {
        // Persis bentuk yang ditulis dan dibaca `js/chat_store.js`.
        let c = conv("abc", "Hello", 5);
        let value = serde_json::to_value(&c).unwrap();
        assert_eq!(value["turns"][0]["role"], "user");
        assert_eq!(value["turns"][1]["role"], "model");
        assert_eq!(value["created_at"], 1_761_436_800_000_i64);
        let back: Conversation = serde_json::from_value(value).unwrap();
        assert_eq!(back, c);
        // Ringkasan dari skrip.
        let s: Summary = serde_json::from_value(
            json!({"id":"a","title":"t","model":"m","created_at":1,"updated_at":2,"count":3}),
        )
        .unwrap();
        assert_eq!((s.count, s.updated_at), (3, 2));
    }

    #[test]
    fn replies_from_the_script_are_unwrapped() {
        assert_eq!(
            unwrap_reply(json!({"ok": true, "data": [1]})),
            Ok(json!([1]))
        );
        assert_eq!(unwrap_reply(json!({"ok": true})), Ok(Value::Null));
        assert_eq!(
            unwrap_reply(json!({"ok": false, "error": "open blocked"})),
            Err(String::from("open blocked"))
        );
        assert_eq!(
            unwrap_reply(json!({"ok": false})),
            Err(String::from("storage error"))
        );
        assert!(unwrap_reply(json!("garbage")).is_err());
        assert!(unwrap_reply(Value::Null).is_err());
    }

    #[test]
    fn the_script_gets_op_and_arg_as_json_literals() {
        let arg = json!({"id": "x\"; alert(1); //", "n": 1});
        let script = store_script("get", &arg);
        assert!(script.starts_with("const OP = \"get\"; const ARG = {"));
        // Tanda kutip dalam argumen ter-escape: tidak bisa keluar dari literal.
        assert!(script.contains(r#""x\"; alert(1); //""#));
        assert!(script.contains("function openDb"));
        // Teks pengguna (yang bisa memuat apa saja) tetap satu nilai JSON.
        let hostile = json!({"turns": [{"text": "</script>\u{2028}`${1}`"}]});
        let script = store_script("put", &hostile);
        let literal = script
            .strip_prefix("const OP = \"put\"; const ARG = ")
            .and_then(|rest| rest.split(";\n").next())
            .unwrap();
        assert_eq!(serde_json::from_str::<Value>(literal).unwrap(), hostile);
    }
}
