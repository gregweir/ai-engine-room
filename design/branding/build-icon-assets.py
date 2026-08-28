"""Deterministically install the owner-approved C2R1 production icon assets.

Run from any directory with Python and Pillow already available. PNG production
inputs are copied byte-for-byte from approved size-specific sources. ICO and ICNS
containers embed those sources without aesthetic changes.
"""

from __future__ import annotations

import io
import struct
from pathlib import Path

from PIL import Image


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ICONS = ROOT / "src-tauri" / "icons"

SOURCES = {
    size: HERE / ("ai-engine-room-icon-master.png" if size == 1024 else f"ai-engine-room-icon-{size}.png")
    for size in (16, 32, 48, 64, 128, 256, 512, 1024)
}


def approved(size: int) -> Image.Image:
    image = Image.open(SOURCES[size])
    image.load()
    if image.format != "PNG" or image.mode != "RGBA" or image.size != (size, size):
        raise RuntimeError(f"Invalid approved source: {SOURCES[size]}")
    if image.getchannel("A").getextrema() != (0, 255):
        raise RuntimeError(f"Approved source lacks expected transparency: {SOURCES[size]}")
    return image


def png_bytes(image: Image.Image) -> bytes:
    output = io.BytesIO()
    image.save(output, "PNG", optimize=True)
    return output.getvalue()


def write_ico(path: Path) -> None:
    # Tauri requires 16, 24, 32, 48, 64 and 256. Its documentation recommends
    # the 32 px layer first for optimal development display.
    layers: list[tuple[int, bytes]] = []
    for size in (32, 16, 24, 48, 64, 256):
        image = approved(size) if size != 24 else approved(32).resize((24, 24), Image.Resampling.LANCZOS)
        layers.append((size, png_bytes(image)))

    header = struct.pack("<HHH", 0, 1, len(layers))
    offset = 6 + 16 * len(layers)
    directory = bytearray()
    payload = bytearray()
    for size, data in layers:
        encoded_size = 0 if size == 256 else size
        directory.extend(struct.pack("<BBBBHHII", encoded_size, encoded_size, 0, 0, 1, 32, len(data), offset))
        payload.extend(data)
        offset += len(data)
    path.write_bytes(header + directory + payload)


def write_icns(path: Path) -> None:
    # Pillow's ICNS writer accepts exact per-size source images via append_images.
    # This keeps approved 32/64 treatments and approved large treatments intact.
    base = approved(1024)
    variants = [approved(size) for size in (32, 64, 128, 256, 512)]
    base.save(path, "ICNS", append_images=variants)


def main() -> None:
    for size in SOURCES:
        approved(size)
    # These are the three PNG paths referenced by the current Tauri bundle config.
    (ICONS / "32x32.png").write_bytes(SOURCES[32].read_bytes())
    (ICONS / "128x128.png").write_bytes(SOURCES[128].read_bytes())
    (ICONS / "128x128@2x.png").write_bytes(SOURCES[256].read_bytes())
    write_ico(ICONS / "icon.ico")
    write_icns(ICONS / "icon.icns")


if __name__ == "__main__":
    main()
