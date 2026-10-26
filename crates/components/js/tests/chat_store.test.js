const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

// IndexedDB tiruan secukupnya untuk skrip ini: open, object store dengan
// keyPath, transaksi (getAll/get/put/delete) dan oncomplete. Data hidup di
// `world.stores`, jadi beberapa "pembukaan" melihat isi yang sama.
function fakeIndexedDb(world = { stores: {}, opens: 0, closes: 0 }) {
  function store(name) {
    return world.stores[name] || (world.stores[name] = new Map());
  }
  function request(fn) {
    const r = { result: undefined, error: null, onsuccess: null, onerror: null };
    queueMicrotask(() => {
      try {
        r.result = fn();
        r.onsuccess && r.onsuccess();
      } catch (e) {
        r.error = e;
        r.onerror && r.onerror();
      }
    });
    return r;
  }
  const idb = {
    world,
    open(name, version) {
      world.opens += 1;
      world.opened = { name, version };
      const r = { result: null, error: null };
      queueMicrotask(() => {
        const db = {
          objectStoreNames: { contains: (n) => n in world.stores },
          createObjectStore(n) {
            store(n);
          },
          close() {
            world.closes += 1;
          },
          transaction(names, mode) {
            const tx = { oncomplete: null, onerror: null, onabort: null, mode, names };
            // Seperti IndexedDB: transaksi selesai sendiri setelah permintaannya
            // habis, juga bila tidak ada permintaan sama sekali.
            if (mode === 'readwrite') setTimeout(() => tx.oncomplete && tx.oncomplete(), 0);
            tx.objectStore = (n) => {
              if (!names.includes(n)) throw new Error('store not in transaction');
              const s = store(n);
              return {
                getAll: () => request(() => [...s.values()]),
                get: (k) => request(() => s.get(k)),
                put: (v) => {
                  if (mode !== 'readwrite') throw new Error('read-only transaction');
                  s.set(v.id, JSON.parse(JSON.stringify(v)));
                },
                delete: (k) => {
                  if (mode !== 'readwrite') throw new Error('read-only transaction');
                  s.delete(k);
                },
              };
            };
            return tx;
          },
        };
        const fresh = !('meta' in world.stores);
        if (fresh && r.onupgradeneeded) {
          r.result = db;
          r.onupgradeneeded();
        }
        r.result = db;
        r.onsuccess && r.onsuccess();
      });
      return r;
    },
  };
  return idb;
}

async function call(idb, op, arg) {
  const sent = [];
  const dioxus = { send: (m) => sent.push(m) };
  await run('chat_store.js', { indexedDB: idb }, {}, { OP: op, ARG: arg, dioxus });
  assert.equal(sent.length, 1, 'exactly one message per call');
  return sent[0];
}

const chat = (id, updated, extra = {}) => ({
  id,
  title: `t-${id}`,
  model: 'gemini-3.8-flash',
  created_at: 1,
  updated_at: updated,
  turns: [
    { role: 'user', text: 'hi' },
    { role: 'model', text: 'hello' },
  ],
  ...extra,
});

test('put then get returns the same chat; list holds a summary without messages', async () => {
  const idb = fakeIndexedDb();
  assert.deepEqual(await call(idb, 'put', chat('a', 10)), { ok: true, data: true });
  assert.deepEqual((await call(idb, 'get', { id: 'a' })).data, chat('a', 10));
  const list = (await call(idb, 'list', {})).data;
  assert.deepEqual(list, [
    { id: 'a', title: 't-a', model: 'gemini-3.8-flash', created_at: 1, updated_at: 10, count: 2 },
  ]);
  assert.equal('turns' in list[0], false);
  assert.equal(idb.world.opened.name, 'tquilla-ai-chat');
  assert.equal(idb.world.opened.version, 1);
});

test('list and all are newest first; get of an unknown id is null', async () => {
  const idb = fakeIndexedDb();
  await call(idb, 'put', chat('old', 1));
  await call(idb, 'put', chat('new', 30));
  await call(idb, 'put', chat('mid', 20));
  assert.deepEqual((await call(idb, 'list', {})).data.map((c) => c.id), ['new', 'mid', 'old']);
  assert.deepEqual((await call(idb, 'all', {})).data.map((c) => c.id), ['new', 'mid', 'old']);
  assert.equal((await call(idb, 'all', {})).data[0].turns.length, 2);
  assert.deepEqual(await call(idb, 'get', { id: 'nope' }), { ok: true, data: null });
});

test('put again replaces the chat and its summary', async () => {
  const idb = fakeIndexedDb();
  await call(idb, 'put', chat('a', 10));
  await call(idb, 'put', chat('a', 11, { turns: [{ role: 'user', text: 'x' }] }));
  const list = (await call(idb, 'list', {})).data;
  assert.equal(list.length, 1);
  assert.equal(list[0].count, 1);
  assert.equal(list[0].updated_at, 11);
});

test('delete removes the chat and its summary; unknown ids are ignored', async () => {
  const idb = fakeIndexedDb();
  await call(idb, 'put', chat('a', 1));
  await call(idb, 'put', chat('b', 2));
  await call(idb, 'put', chat('c', 3));
  assert.deepEqual(await call(idb, 'delete', { ids: ['a', 'c', 'zzz'] }), { ok: true, data: true });
  assert.deepEqual((await call(idb, 'list', {})).data.map((c) => c.id), ['b']);
  assert.equal((await call(idb, 'get', { id: 'a' })).data, null);
  assert.deepEqual(await call(idb, 'delete', { ids: [] }), { ok: true, data: true });
});

test('only known fields are stored; roles are normalised; text is capped', async () => {
  const idb = fakeIndexedDb();
  const dirty = chat('a', 5, {
    api_key: 'AIza-secret',
    turns: [
      { role: 'user', text: 'x'.repeat(250000), extra: 1 },
      { role: 'weird', text: 5 },
      { role: 'model' },
    ],
  });
  await call(idb, 'put', dirty);
  const stored = (await call(idb, 'get', { id: 'a' })).data;
  assert.equal('api_key' in stored, false);
  assert.equal(JSON.stringify(stored).includes('AIza'), false);
  assert.equal(stored.turns[0].text.length, 200000);
  assert.deepEqual(stored.turns[0], { role: 'user', text: 'x'.repeat(200000) });
  assert.deepEqual(stored.turns[1], { role: 'model', text: '5' });
  assert.deepEqual(stored.turns[2], { role: 'model', text: '' });
});

test('invalid chats and unknown operations are errors, nothing is written', async () => {
  const idb = fakeIndexedDb();
  for (const bad of [null, {}, { id: '', turns: [] }, { id: 'a' }, { id: 'a', turns: 'x' }]) {
    const r = await call(idb, 'put', bad);
    assert.equal(r.ok, false);
    assert.match(r.error, /invalid chat/);
  }
  assert.equal((await call(idb, 'list', {})).data.length, 0);
  const r = await call(idb, 'drop-everything', {});
  assert.deepEqual(r, { ok: false, error: 'unknown op' });
});

test('the database is closed after every call, also after an error', async () => {
  const idb = fakeIndexedDb();
  await call(idb, 'list', {});
  await call(idb, 'put', { nope: 1 });
  await call(idb, 'whatever', {});
  assert.equal(idb.world.closes, idb.world.opens);
  assert.equal(idb.world.opens, 3);
});

test('without indexedDB the answer is an error, not a hang or a throw', async () => {
  const sent = [];
  await run('chat_store.js', {}, {}, { OP: 'list', ARG: {}, dioxus: { send: (m) => sent.push(m) } });
  assert.deepEqual(sent, [{ ok: false, error: 'indexedDB unavailable' }]);
});

test('a blocked or failing open is reported as an error', async () => {
  const blocked = {
    open() {
      const r = {};
      queueMicrotask(() => r.onblocked());
      return r;
    },
  };
  const sent = [];
  await run('chat_store.js', { indexedDB: blocked }, {}, { OP: 'list', ARG: {}, dioxus: { send: (m) => sent.push(m) } });
  assert.deepEqual(sent, [{ ok: false, error: 'open blocked' }]);

  const failing = {
    open() {
      const r = { error: new Error('denied') };
      queueMicrotask(() => r.onerror());
      return r;
    },
  };
  const sent2 = [];
  await run('chat_store.js', { indexedDB: failing }, {}, { OP: 'list', ARG: {}, dioxus: { send: (m) => sent2.push(m) } });
  assert.deepEqual(sent2, [{ ok: false, error: 'denied' }]);
});
