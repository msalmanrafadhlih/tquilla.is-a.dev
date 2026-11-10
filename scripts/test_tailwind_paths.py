"""Entry Tailwind tiap platform + susunan `assets/styles/` harus konsisten.

Kenapa ada tes ini: `build.rs` di desktop/mobile membuat `assets/tailwind.css`
kosong bila belum ada, jadi `cargo check`/`clippy` di `platforms.yml` tetap
hijau walau `@import` di `tailwind.css` salah. Kesalahannya baru ketahuan
di `release-apps.yml` ("Can't resolve '../components/theme.css'", build
Android 2026-10-28, sisa tata letak lama `packages/`).

    python3 -m unittest discover -s scripts -p "test_*.py" -v
"""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PLATFORMS = ("web", "desktop", "mobile")
ASSETS = ROOT / "crates" / "components" / "assets"
STYLES = ASSETS / "styles"
IMPORT = re.compile(r'@import\s+"([^"]+)"')
SOURCE = re.compile(r'@source\s+"([^"]+)"')


def css_files():
    return [ROOT / "platform" / name / "tailwind.css" for name in PLATFORMS]


def read_css(css):
    return re.sub(r"/\*.*?\*/", "", css.read_text(encoding="utf-8"), flags=re.S)  # tanpa komentar


def local_imports(css):
    return [
        (css.parent / target).resolve()
        for target in IMPORT.findall(read_css(css))
        if target.startswith(".")  # "tailwindcss" dan paket lain bukan path lokal
    ]


class TailwindPaths(unittest.TestCase):
    def test_every_platform_has_a_tailwind_entry(self):
        for css in css_files():
            self.assertTrue(css.is_file(), f"{css.relative_to(ROOT)} tidak ada")

    def test_local_imports_exist(self):
        for css in css_files():
            for target in IMPORT.findall(read_css(css)):
                if not target.startswith("."):
                    continue
                path = (css.parent / target).resolve()
                self.assertTrue(
                    path.is_file(),
                    f"{css.relative_to(ROOT)}: @import \"{target}\" tidak ditemukan",
                )

    def test_source_globs_point_at_existing_directories(self):
        # Tailwind diam-diam mengabaikan @source yang salah. Saat ini entry
        # memakai `source(none)` (tanpa @source), tapi kalau ada yang menambah
        # lagi, jalurnya harus benar.
        for css in css_files():
            for glob in SOURCE.findall(read_css(css)):
                if "*" not in glob:
                    continue
                base = glob.split("*", 1)[0].rstrip("/")
                path = (css.parent / base).resolve()
                self.assertTrue(
                    path.exists(),
                    f"{css.relative_to(ROOT)}: @source \"{glob}\" menunjuk ke tempat yang tidak ada",
                )

    def test_all_platforms_share_the_same_style_entry(self):
        # Satu sumber CSS untuk semua platform.
        entries = {tuple(local_imports(css)) for css in css_files()}
        self.assertEqual(len(entries), 1, f"entry CSS berbeda-beda: {entries}")
        self.assertEqual(entries.pop(), ((STYLES / "index.css").resolve(),))

    def test_tailwind_does_not_scan_sources(self):
        # Class di rsx! adalah nama semantik; memindai Rust/HTML hanya menambah
        # utility liar ke bundel. Hapus `source(none)` hanya dengan sengaja.
        for css in css_files():
            self.assertIn("source(none)", read_css(css), css.name)


class StylesLayout(unittest.TestCase):
    def test_index_imports_global_first_then_every_file_once(self):
        index = STYLES / "index.css"
        imports = [Path(t).name for t in IMPORT.findall(read_css(index))]
        self.assertEqual(imports[0], "global.css", "global.css harus diimpor pertama")
        on_disk = sorted(p.name for p in STYLES.glob("*.css") if p.name != "index.css")
        self.assertEqual(sorted(imports), on_disk, "index.css harus mengimpor semua file styles/ (tepat sekali)")
        for name in imports:
            self.assertTrue((STYLES / name).is_file(), f"{name} diimpor tapi tidak ada")

    def test_no_stray_css_outside_styles(self):
        # Jalur utama CSS adalah assets/styles/. Tailwind output (assets/tailwind.css
        # per platform) di-generate dan di-.gitignore, jadi dikecualikan.
        stray = [p.relative_to(ROOT) for p in ASSETS.rglob("*.css") if STYLES not in p.parents]
        self.assertEqual(stray, [], "CSS harus di assets/styles/, bukan assets/*.css")
        self.assertFalse((ROOT / "crates" / "components" / "theme.css").exists(), "theme.css sudah pindah ke styles/global.css")


if __name__ == "__main__":
    unittest.main()
