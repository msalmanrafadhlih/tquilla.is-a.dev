"""Class di rsx! harus punya definisi di `assets/styles/*.css`, dan sebaliknya.

Class di rsx! berupa nama semantik (BEM: `blok__elemen--modifier`) yang
didefinisikan lewat `@apply` di `crates/components/assets/styles/`. Tailwind
tidak memindai berkas Rust (`source(none)`), jadi utility yang ditulis
langsung di rsx! (mis. `class: "flex gap-2"`) TIDAK menghasilkan CSS. Tes ini
menangkapnya sebelum sampai ke layar.

    python3 -m unittest discover -s scripts -p "test_*.py" -v
"""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STYLES = ROOT / "crates" / "components" / "assets" / "styles"
SRC = ROOT / "crates" / "components" / "src"

# Class tanpa aturan sendiri di styles/: penanda untuk variant Tailwind
# (`group-hover:`, `peer-focus-visible:`) dan font ikon dari Google Fonts.
EXTERNAL = {"group", "peer", "material-symbols-outlined"}
# Class yang cuma diubah JS (window_manager.js / scroll_reveal.js); aturannya
# ada di CSS, tapi namanya tidak muncul sebagai literal Rust.
JS_ONLY = {"is-visible"}

BEM = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*(?:__[a-z0-9]+(?:-[a-z0-9]+)*)?(?:--[a-z0-9]+(?:-[a-z0-9]+)*)?$")
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')


def strip_css_comments(text):
    return re.sub(r"/\*.*?\*/", "", text, flags=re.S)


def defined_classes():
    found = {}
    for css in sorted(STYLES.glob("*.css")):
        text = strip_css_comments(css.read_text(encoding="utf-8"))
        text = re.sub(r'url\([^)]*\)|"[^"]*"|\'[^\']*\'', "", text)
        for chunk in re.split(r"[{}]", text):
            header = chunk.split(";")[-1].strip()
            if not header or header.startswith("@"):
                continue
            for name in re.findall(r"\.([A-Za-z_][\w-]*)", header):
                found.setdefault(name, css.name)
    return found


def rust_files():
    for path in sorted(SRC.rglob("*.rs")):
        if "languages" in path.parts:  # teks UI, bukan class
            continue
        yield path


def strip_rust_comments(text):
    return re.sub(r"(?m)^\s*//.*$|(?<=[;,{}\s])//[^\n\"]*$", "", text)


def chain_literals(text, i):
    """String literal di dalam rantai `if .. { "a" } else { "b" }` yang mulai di `i`."""
    out, depth, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c == '"':
            j = i + 1
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            if depth >= 1:
                out.append(text[i + 1: j])
            i = j + 1
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth <= 0 and not re.match(r"\s*else\b", text[i + 1:]):
                break
        i += 1
    return out


def class_attribute_tokens():
    """Token dari `class: "..."` dan cabang `class: if c { "a" } else { "b" }`.

    Nilai dinamis (`format!`, variabel, bagian `{x}`) tidak bisa dicek statis dan dilewati.
    """
    for path in rust_files():
        text = strip_rust_comments(path.read_text(encoding="utf-8"))
        for m in re.finditer(r"\bclass:\s*", text):
            i = m.end()
            if text.startswith('"', i):
                lits = [STRING.match(text, i).group(1)]
            elif re.match(r"if\b", text[i:]):
                lits = chain_literals(text, i)
            else:
                continue
            for lit in lits:
                lit = re.sub(r"\{[^{}]*\}", "\0", lit)  # `{x}` (boleh berisi spasi) = bagian dinamis
                for tok in lit.split():
                    if "\0" not in tok:
                        yield path, tok


def all_rust_text():
    return "\n".join(strip_rust_comments(p.read_text(encoding="utf-8")) for p in rust_files())


class CssClasses(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.defined = defined_classes()

    def test_styles_define_something(self):
        self.assertGreater(len(self.defined), 300)

    def test_class_attributes_use_defined_classes(self):
        missing = {}
        for path, tok in class_attribute_tokens():
            if tok in self.defined or tok in EXTERNAL:
                continue
            missing.setdefault(tok, path.relative_to(ROOT))
        self.assertEqual(
            missing, {},
            "class di rsx! tanpa definisi di assets/styles/*.css "
            "(utility Tailwind tidak dipindai; tulis class semantik + @apply):",
        )

    def test_bem_names_in_rust_are_defined(self):
        missing = {}
        for path in rust_files():
            text = strip_rust_comments(path.read_text(encoding="utf-8"))
            for lit in STRING.findall(text):
                for tok in lit.split():
                    if ("__" in tok or "--" in tok) and BEM.match(tok) and tok not in self.defined:
                        missing.setdefault(tok, path.relative_to(ROOT))
        self.assertEqual(missing, {}, "nama BEM di kode Rust tanpa definisi CSS:")

    def test_every_defined_class_is_used(self):
        # CSS mati dibuang, bukan dibiarkan menumpuk.
        text = all_rust_text()
        unused = sorted(
            name for name in self.defined
            if name not in JS_ONLY and not re.search(r"(?<![\w-])" + re.escape(name) + r"(?![\w-])", text)
        )
        self.assertEqual(unused, [], "class didefinisikan di styles/ tapi tidak dipakai di Rust:")


if __name__ == "__main__":
    unittest.main()
