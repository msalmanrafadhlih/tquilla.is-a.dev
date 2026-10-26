// Riwayat chat AI di IndexedDB milik browser (tidak pernah dikirim ke server).
// Dijalankan lewat `document::eval`; sebelum badan skrip ini ditulis:
//   OP   salah satu dari list, all, get, put, delete
//   ARG  argumen op (JSON): get {id}; put {id, title, model, created_at,
//        updated_at, turns: [{role, text}]}; delete {ids: [id, ...]}
// Mengirim tepat SATU pesan lewat `dioxus.send`:
//   { ok: true, data: ... }  atau  { ok: false, error: "..." }
// `list` mengembalikan ringkasan (tanpa isi pesan) dari store `meta`, terbaru
// dulu; `all` mengembalikan chat lengkap, terbaru dulu (untuk ekspor semua).
// Dua store (`meta` kecil, `chats` lengkap) ditulis dalam SATU transaksi,
// supaya daftar riwayat tidak perlu membaca ratusan isi chat.
// Kunci API tidak pernah lewat sini: tidak ada kolomnya.

const DB_NAME = 'tquilla-ai-chat';
const DB_VERSION = 1;
const META = 'meta';
const CHATS = 'chats';
const MAX_TEXT = 200000; // batas pengaman per pesan (karakter)
const MAX_TURNS = 2000; // batas pengaman per chat

function asRequest(request) {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error || new Error('request failed'));
  });
}

function asDone(tx) {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve(true);
    tx.onerror = () => reject(tx.error || new Error('transaction failed'));
    tx.onabort = () => reject(tx.error || new Error('transaction aborted'));
  });
}

function openDb() {
  return new Promise((resolve, reject) => {
    const idb = window.indexedDB;
    if (!idb) {
      reject(new Error('indexedDB unavailable'));
      return;
    }
    const request = idb.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(META)) db.createObjectStore(META, { keyPath: 'id' });
      if (!db.objectStoreNames.contains(CHATS)) db.createObjectStore(CHATS, { keyPath: 'id' });
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error || new Error('open failed'));
    request.onblocked = () => reject(new Error('open blocked'));
  });
}

// Bentuk yang disimpan: hanya kolom yang dikenal, teks dipotong di batas.
function normalize(chat) {
  if (!chat || typeof chat.id !== 'string' || chat.id === '' || !Array.isArray(chat.turns)) {
    throw new Error('invalid chat');
  }
  const turns = chat.turns.slice(0, MAX_TURNS).map((t) => ({
    role: t && t.role === 'user' ? 'user' : 'model',
    text: String((t && t.text) || '').slice(0, MAX_TEXT),
  }));
  return {
    id: chat.id,
    title: String(chat.title || ''),
    model: String(chat.model || ''),
    created_at: Number(chat.created_at) || 0,
    updated_at: Number(chat.updated_at) || 0,
    turns,
  };
}

function summarize(chat) {
  return {
    id: chat.id,
    title: chat.title,
    model: chat.model,
    created_at: chat.created_at,
    updated_at: chat.updated_at,
    count: chat.turns.length,
  };
}

function newestFirst(a, b) {
  return b.updated_at - a.updated_at;
}

async function perform(op, arg) {
  const db = await openDb();
  try {
    if (op === 'list') {
      const rows = await asRequest(db.transaction([META], 'readonly').objectStore(META).getAll());
      return rows.slice().sort(newestFirst);
    }
    if (op === 'all') {
      const rows = await asRequest(db.transaction([CHATS], 'readonly').objectStore(CHATS).getAll());
      return rows.slice().sort(newestFirst);
    }
    if (op === 'get') {
      const row = await asRequest(db.transaction([CHATS], 'readonly').objectStore(CHATS).get(String(arg.id)));
      return row || null;
    }
    if (op === 'put') {
      const chat = normalize(arg);
      const tx = db.transaction([META, CHATS], 'readwrite');
      tx.objectStore(CHATS).put(chat);
      tx.objectStore(META).put(summarize(chat));
      await asDone(tx);
      return true;
    }
    if (op === 'delete') {
      const ids = Array.isArray(arg.ids) ? arg.ids : [];
      const tx = db.transaction([META, CHATS], 'readwrite');
      for (const id of ids) {
        tx.objectStore(CHATS).delete(String(id));
        tx.objectStore(META).delete(String(id));
      }
      await asDone(tx);
      return true;
    }
    throw new Error('unknown op');
  } finally {
    db.close();
  }
}

return perform(OP, ARG).then(
  (data) => {
    dioxus.send({ ok: true, data });
  },
  (error) => {
    dioxus.send({ ok: false, error: String((error && error.message) || error) });
  },
);
