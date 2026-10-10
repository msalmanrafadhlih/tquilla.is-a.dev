#!/usr/bin/env python3
"""Pasca-build untuk output `dx bundle` (dipakai CI, bisa dijalankan manual).

    python3 scripts/postbuild.py target/dx/web/release/web/public

Yang dilakukan (semuanya idempoten, aman dijalankan dua kali):
  1. `<html lang="en">`            -> bahasa dokumen untuk screen reader / mesin pencari
  2. site/head.html -> `</head>`   -> meta description, canonical, Open Graph, favicon.
     Crawler preview link (WhatsApp, Discord, Facebook) tidak menjalankan
     JS/WASM, jadi tag ini harus ada di HTML statis. `{{AVATAR}}` diganti
     dengan path aset avatar yang di-hash oleh dx (assets/avatar-<hash>.jpg).
  3. site/noscript.html -> `</body>`
  4. site/robots.txt, site/sitemap.xml, dan site/og-image.jpg (gambar kartu
     preview 1200x630, URL-nya harus stabil/tanpa hash) disalin ke root output.

Dirancang sebagai langkah terpisah (bukan template index.html kustom) supaya
tidak bergantung pada placeholder internal template dx yang bisa berubah
antar versi. Script gagal keras (exit != 0) kalau struktur index.html tidak
seperti yang diharapkan, supaya deploy tidak diam-diam kehilangan metadata.
"""
import re
import shutil
import sys
from pathlib import Path

MARK = "<!-- site-meta:"
ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"


def fail(msg: str) -> None:
    print(f"postbuild: ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: postbuild.py <public-dir>")
    public = Path(sys.argv[1])
    index = public / "index.html"
    if not index.is_file():
        fail(f"{index} not found (did `dx bundle` run?)")

    avatars = sorted((public / "assets").glob("avatar-*.jpg"))
    if not avatars:
        fail(f"no hashed avatar-*.jpg found in {public / 'assets'}")
    avatar_path = "/assets/" + avatars[0].name

    html = index.read_text(encoding="utf-8")

    # 1. lang
    m = re.search(r"<html\b([^>]*)>", html, re.I)
    if not m:
        fail("no <html> tag in index.html")
    if not re.search(r"\blang\s*=", m.group(1), re.I):
        html = html[: m.start()] + f'<html lang="en"{m.group(1)}>' + html[m.end():]

    # 2. head meta
    if MARK not in html:
        if "</head>" not in html:
            fail("no </head> in index.html")
        head = (SITE / "head.html").read_text(encoding="utf-8").replace("{{AVATAR}}", avatar_path)
        html = html.replace("</head>", head + "</head>", 1)

    # 3. noscript
    if "<noscript>" not in html:
        if "</body>" not in html:
            fail("no </body> in index.html")
        noscript = (SITE / "noscript.html").read_text(encoding="utf-8")
        html = html.replace("</body>", noscript + "</body>", 1)

    index.write_text(html, encoding="utf-8")

    # 4. static files
    for name in ("robots.txt", "sitemap.xml", "og-image.jpg"):
        shutil.copyfile(SITE / name, public / name)

    print(f"postbuild: ok ({index}, avatar={avatar_path})")


if __name__ == "__main__":
    main()
