"""Rasterises the icon sources with headless Chrome and writes the Windows .ico.

Kept in the repo rather than run by hand: the sizes below are what the shell
asks for, and getting one wrong shows up as a blurry tray icon nobody traces
back to a build step.
"""

import io
import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "..", "app", "src-tauri", "icons")
CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"

ICO_SIZES = [16, 20, 24, 32, 48, 64, 128, 256]
TRAY_SIZES = [16, 20, 24, 32]


def render(svg: str, size: int, target: str) -> None:
    with io.open(os.path.join(HERE, svg), encoding="utf-8") as handle:
        body = handle.read()
    page = (
        "<style>html,body{margin:0;padding:0;background:transparent}"
        f"svg{{width:{size}px;height:{size}px;display:block}}</style>{body}"
    )
    with tempfile.NamedTemporaryFile("w", suffix=".html", delete=False, encoding="utf-8") as tmp:
        tmp.write(page)
        page_path = tmp.name

    work = tempfile.mkdtemp()
    subprocess.run(
        [
            CHROME,
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
    os.makedirs(OUT, exist_ok=True)
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

    for variant in ("tray-dark", "tray-light"):
        for size in TRAY_SIZES:
            render(f"{variant}.svg", size, os.path.join(OUT, f"{variant}-{size}.png"))

    print(f"escritos en {os.path.normpath(OUT)}")


if __name__ == "__main__":
    if not os.path.isfile(CHROME):
        sys.exit(f"no encuentro Chrome en {CHROME}")
    main()
