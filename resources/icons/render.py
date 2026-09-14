"""Rasterises the icon sources with headless Chrome and writes the Windows .ico
and the macOS .icns.

Kept in the repo rather than run by hand: the sizes below are what the shell
asks for, and getting one wrong shows up as a blurry tray icon nobody traces
back to a build step.
"""

import io
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "..", "app", "src-tauri", "icons")

CHROMIUMS = [
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
    "/Applications/Vivaldi.app/Contents/MacOS/Vivaldi",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
]


def chromium() -> str:
    for candidate in CHROMIUMS:
        if os.path.isfile(candidate):
            return candidate
    found = shutil.which("chromium") or shutil.which("google-chrome")
    if found:
        return found
    sys.exit("no encuentro ningún Chromium con el que rasterizar")


# What `iconutil` expects inside the .iconset, by the names it expects them with.
ICNS_SIZES = [16, 32, 128, 256, 512]


ICO_SIZES = [16, 20, 24, 32, 48, 64, 128, 256]
TRAY_SIZES = [16, 20, 24, 32]
# What an MSIX manifest names, by the names it names them with. The Store rejects a package whose
# logo is the wrong size, and it rejects it after the upload rather than before.
STORE_LOGOS = {
    "StoreLogo": 50,
    "Square44x44Logo": 44,
    "Square71x71Logo": 71,
    "Square150x150Logo": 150,
    "Square310x310Logo": 310,
}


def source(svg: str) -> str:
    with io.open(os.path.join(HERE, svg), encoding="utf-8") as handle:
        return handle.read()


def render(svg: str, size: int, target: str) -> None:
    if sys.platform == "darwin":
        quicklook(svg, size, target)
    else:
        headless(svg, size, target)


def quicklook(svg: str, size: int, target: str) -> None:
    """Quick Look draws an SVG at the size it declares and never scales it up, so
    a 32-wide glyph asked for at 256 lands in a corner of an empty square."""
    body = re.sub(
        r'<svg\s+width="\d+"\s+height="\d+"',
        f'<svg width="{size}" height="{size}"',
        source(svg),
        count=1,
    )
    work = tempfile.mkdtemp()
    drawn = os.path.join(work, "icon.svg")
    with io.open(drawn, "w", encoding="utf-8") as handle:
        handle.write(body)

    subprocess.run(
        ["qlmanage", "-t", "-s", str(size), "-o", work, drawn],
        check=True,
        capture_output=True,
    )
    # It answers zero whether or not it drew anything.
    made = f"{drawn}.png"
    if not os.path.isfile(made):
        sys.exit(f"Quick Look no dibujó {svg} a {size}px")
    shutil.move(made, target)


def headless(svg: str, size: int, target: str) -> None:
    page = (
        "<style>html,body{margin:0;padding:0;background:transparent}"
        f"svg{{width:{size}px;height:{size}px;display:block}}</style>{source(svg)}"
    )
    with tempfile.NamedTemporaryFile("w", suffix=".html", delete=False, encoding="utf-8") as tmp:
        tmp.write(page)
        page_path = tmp.name

    work = tempfile.mkdtemp()
    subprocess.run(
        [
            chromium(),
            "--headless",
            "--disable-gpu",
            "--default-background-color=00000000",
            f"--screenshot={target}",
            f"--window-size={size},{size}",
            f"--user-data-dir={work}",
            page_path,
        ],
        check=True,
        capture_output=True,
    )
    os.unlink(page_path)


def write_ico(pngs: list[tuple[int, bytes]], target: str) -> None:
    """PNG-in-ICO, which every Windows since Vista reads and keeps the file small."""
    header = struct.pack("<HHH", 0, 1, len(pngs))
    offset = len(header) + 16 * len(pngs)
    entries, blobs = b"", b""
    for size, blob in pngs:
        entries += struct.pack(
            "<BBBBHHII", size if size < 256 else 0, size if size < 256 else 0,
            0, 0, 1, 32, len(blob), offset,
        )
        blobs += blob
        offset += len(blob)
    with io.open(target, "wb") as handle:
        handle.write(header + entries + blobs)


def main() -> None:
    asked = sys.argv[1] if len(sys.argv) > 1 else ("macos" if sys.platform == "darwin" else "windows")
    if asked not in ("windows", "macos", "all"):
        sys.exit("uso: render.py [windows|macos|all]")

    os.makedirs(OUT, exist_ok=True)
    # Two rasterisers draw the same source differently, so a run meant for one
    # system must not rewrite the other's icons: the Store rejects a package
    # whose logos changed, and the tray pictures are compiled into the binary.
    if asked in ("windows", "all"):
        windows_icons()
    if asked in ("macos", "all"):
        macos_icons()
    print(f"escritos en {os.path.normpath(OUT)}")


def macos_icons() -> None:
    # No menu bar picture is drawn here: Quick Look fits a bare glyph to its own
    # bounding box rather than to the viewBox, and the one the tray already
    # ships is the same drawing with the alpha a template needs.
    write_icns(tempfile.mkdtemp())


def windows_icons() -> None:
    work = tempfile.mkdtemp()

    frames = []
    for size in ICO_SIZES:
        path = os.path.join(work, f"app-{size}.png")
        render("app-windows.svg", size, path)
        with io.open(path, "rb") as handle:
            frames.append((size, handle.read()))
        if size in (32, 128, 256):
            render("app-windows.svg", size, os.path.join(OUT, f"{size}x{size}.png"))
    write_ico(frames, os.path.join(OUT, "icon.ico"))
    render("app-windows.svg", 512, os.path.join(OUT, "icon.png"))

    for name, size in STORE_LOGOS.items():
        render("app-windows.svg", size, os.path.join(OUT, f"{name}.png"))

    for variant in ("tray-dark", "tray-light"):
        for size in TRAY_SIZES:
            render(f"{variant}.svg", size, os.path.join(OUT, f"{variant}-{size}.png"))


def write_icns(work: str) -> None:
    """`iconutil` is macOS only; elsewhere the .icns already in the tree stands."""
    iconset = os.path.join(work, "icon.iconset")
    os.makedirs(iconset, exist_ok=True)
    for size in ICNS_SIZES:
        render("app-macos.svg", size, os.path.join(iconset, f"icon_{size}x{size}.png"))
        render("app-macos.svg", size * 2, os.path.join(iconset, f"icon_{size}x{size}@2x.png"))

    # Declared by tauri.conf.json beside the .icns, and the bundler reads it.
    render("app-macos.svg", 256, os.path.join(OUT, "128x128@2x.png"))

    if not shutil.which("iconutil"):
        print("sin iconutil: el .icns se queda como estaba")
        return
    subprocess.run(
        ["iconutil", "-c", "icns", iconset, "-o", os.path.join(OUT, "icon.icns")],
        check=True,
        capture_output=True,
    )


if __name__ == "__main__":
    main()
