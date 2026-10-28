"""Setiap `platform/*/tailwind.css` harus menunjuk berkas yang ada.

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
IMPORT = re.compile(r'@import\s+"([^"]+)"')
SOURCE = re.compile(r'@source\s+"([^"]+)"')


def css_files():
    return [ROOT / "platform" / name / "tailwind.css" for name in PLATFORMS]


class TailwindPaths(unittest.TestCase):
    def test_every_platform_has_a_tailwind_entry(self):
        for css in css_files():
            self.assertTrue(css.is_file(), f"{css.relative_to(ROOT)} tidak ada")

    def test_local_imports_exist(self):
        for css in css_files():
            for target in IMPORT.findall(css.read_text(encoding="utf-8")):
                if not target.startswith("."):
                    continue  # "tailwindcss" dan paket lain bukan path lokal
                path = (css.parent / target).resolve()
                self.assertTrue(
                    path.is_file(),
                    f"{css.relative_to(ROOT)}: @import \"{target}\" tidak ditemukan",
                )

    def test_source_globs_point_at_existing_directories(self):
        # Tailwind diam-diam mengabaikan @source yang salah: CSS jadi, tetapi
        # kelas dari crate `components` hilang (app tanpa gaya).
        for css in css_files():
            for glob in SOURCE.findall(css.read_text(encoding="utf-8")):
                if "*" not in glob:
                    continue  # berkas tunggal (web: ./index.html, dibuat dx saat build)
                base = glob.split("*", 1)[0].rstrip("/")
                path = (css.parent / base).resolve()
                self.assertTrue(
                    path.exists(),
                    f"{css.relative_to(ROOT)}: @source \"{glob}\" menunjuk ke tempat yang tidak ada",
                )

    def test_all_platforms_share_the_same_theme_import(self):
        # Satu sumber tema untuk semua platform: bedanya hanya @source khusus
        # platform (web punya index.html).
        themes = set()
        for css in css_files():
            for target in IMPORT.findall(css.read_text(encoding="utf-8")):
                if target.startswith("."):
                    themes.add((css.parent / target).resolve())
        self.assertEqual(len(themes), 1, f"theme.css berbeda-beda: {themes}")


if __name__ == "__main__":
    unittest.main()
