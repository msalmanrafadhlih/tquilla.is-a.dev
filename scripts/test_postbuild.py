"""Test postbuild.py di atas output palsu mirip `dx bundle`.

    python3 -m unittest discover -s scripts -p "test_*.py" -v
"""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "postbuild.py"
INDEX = "<!DOCTYPE html><html><head><title>x</title></head><body><div id=main></div></body></html>"


def make_public(root: Path, index: str = INDEX, avatar: bool = True) -> Path:
    public = root / "public"
    (public / "assets").mkdir(parents=True)
    if index is not None:
        (public / "index.html").write_text(index, encoding="utf-8")
    if avatar:
        (public / "assets" / "avatar-dxh1234.jpg").write_bytes(b"jpg")
    return public


def run(public: Path) -> subprocess.CompletedProcess:
    return subprocess.run([sys.executable, "-I", str(SCRIPT), str(public)], capture_output=True, text=True)


class PostbuildTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def test_injects_everything(self):
        public = make_public(self.root)
        r = run(public)
        self.assertEqual(r.returncode, 0, r.stderr)
        html = (public / "index.html").read_text(encoding="utf-8")
        self.assertIn('<html lang="en">', html)
        self.assertIn("og:image", html)
        self.assertIn("/assets/avatar-dxh1234.jpg", html)
        self.assertNotIn("{{AVATAR}}", html)
        self.assertIn("<noscript>", html)
        for name in ("robots.txt", "sitemap.xml", "og-image.jpg"):
            self.assertTrue((public / name).is_file(), name)

    def test_is_idempotent(self):
        public = make_public(self.root)
        self.assertEqual(run(public).returncode, 0)
        first = (public / "index.html").read_text(encoding="utf-8")
        self.assertEqual(run(public).returncode, 0)
        self.assertEqual((public / "index.html").read_text(encoding="utf-8"), first)

    def test_keeps_existing_lang(self):
        public = make_public(self.root, index=INDEX.replace("<html>", '<html lang="id">'))
        self.assertEqual(run(public).returncode, 0)
        html = (public / "index.html").read_text(encoding="utf-8")
        self.assertIn('lang="id"', html)
        self.assertNotIn('lang="en"', html)

    def test_fails_without_index(self):
        public = make_public(self.root, index=None)
        r = run(public)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("index.html", r.stderr)

    def test_fails_without_head_or_body(self):
        for broken in (INDEX.replace("</head>", ""), INDEX.replace("</body>", "")):
            with tempfile.TemporaryDirectory() as d:
                public = make_public(Path(d), index=broken)
                self.assertNotEqual(run(public).returncode, 0)

    def test_fails_without_hashed_avatar(self):
        public = make_public(self.root, avatar=False)
        r = run(public)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("avatar", r.stderr)


if __name__ == "__main__":
    unittest.main()
