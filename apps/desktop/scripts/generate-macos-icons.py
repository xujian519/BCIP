#!/usr/bin/env python3
"""按 macOS App Icon 规范从源图生成 Tauri 图标资源。

- 将主体缩放到 1024 画布内约 824px 安全区并居中
- 四角保持透明，便于系统在 Dock 上套用圆角（squircle）遮罩
- 对最终 alpha 套用近似 squircle，避免 dev/未套系统遮罩时显示为正矩形
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

# Apple macOS 图标：1024 画布上约 824×824 内容区
CANVAS = 1024
SAFE = 824
# Big Sur 起 Dock 圆角半径约为边长的 ~22.37%
SQUIRCLE_RADIUS = int(CANVAS * 0.2237)

DESKTOP_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = DESKTOP_ROOT / "public" / "logo.png"
ICONS_DIR = DESKTOP_ROOT / "src-tauri" / "icons"
ICONSET_DIR = ICONS_DIR / "icon.iconset"

# iconutil 要求的文件名
ICONSET_SIZES: list[tuple[str, int]] = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]


def content_bbox(image: Image.Image) -> tuple[int, int, int, int]:
    """取非近白、或有可见 alpha 的内容区域。"""
    rgba = image.convert("RGBA")
    w, h = rgba.size
    pixels = rgba.load()
    xs: list[int] = []
    ys: list[int] = []
    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if a < 16:
                continue
            if r > 250 and g > 250 and b > 250:
                continue
            xs.append(x)
            ys.append(y)
    if not xs:
        alpha = rgba.split()[3]
        box = alpha.getbbox()
        if box is None:
            return 0, 0, w, h
        return box
    return min(xs), min(ys), max(xs) + 1, max(ys) + 1


def squircle_mask(size: int, radius: int) -> Image.Image:
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, size - 1, size - 1), radius=radius, fill=255)
    return mask


def compose_macos_icon(source: Path) -> Image.Image:
    src = Image.open(source).convert("RGBA")
    left, top, right, bottom = content_bbox(src)
    cropped = src.crop((left, top, right, bottom))
    cw, ch = cropped.size
    scale = min(SAFE / cw, SAFE / ch)
    nw, nh = max(1, int(cw * scale)), max(1, int(ch * scale))
    resized = cropped.resize((nw, nh), Image.Resampling.LANCZOS)

    canvas = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    ox = (CANVAS - nw) // 2
    oy = (CANVAS - nh) // 2
    canvas.paste(resized, (ox, oy), resized)

    mask = squircle_mask(CANVAS, SQUIRCLE_RADIUS)
    r, g, b, a = canvas.split()
    a = Image.composite(a, Image.new("L", (CANVAS, CANVAS), 0), mask)
    return Image.merge("RGBA", (r, g, b, a))


def write_iconset(master: Image.Image, iconset_dir: Path) -> None:
    if iconset_dir.exists():
        shutil.rmtree(iconset_dir)
    iconset_dir.mkdir(parents=True)
    for name, edge in ICONSET_SIZES:
        resized = master.resize((edge, edge), Image.Resampling.LANCZOS)
        resized.save(iconset_dir / name, format="PNG", optimize=True)


def run_iconutil(iconset_dir: Path, icns_path: Path) -> None:
    subprocess.run(
        ["iconutil", "-c", "icns", str(iconset_dir), "-o", str(icns_path)],
        check=True,
    )


def write_ico(master: Image.Image, ico_path: Path) -> None:
    sizes = (16, 32, 48, 64, 128, 256)
    master.save(
        ico_path,
        format="ICO",
        sizes=[(edge, edge) for edge in sizes],
    )


def copy_bundle_pngs(master: Image.Image, icons_dir: Path) -> None:
    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 1024,
    }
    for name, edge in sizes.items():
        master.resize((edge, edge), Image.Resampling.LANCZOS).save(
            icons_dir / name,
            format="PNG",
            optimize=True,
        )


def verify_corners(image: Image.Image) -> None:
    w, h = image.size
    for label, xy in [("TL", (0, 0)), ("TR", (w - 1, 0)), ("BR", (w - 1, h - 1))]:
        if image.getpixel(xy)[3] != 0:
            raise SystemExit(f"角点 {label} 仍不透明: {image.getpixel(xy)}")


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SOURCE
    if not source.is_file():
        raise SystemExit(f"源图不存在: {source}")

    master = compose_macos_icon(source)
    verify_corners(master)

    ICONS_DIR.mkdir(parents=True, exist_ok=True)
    copy_bundle_pngs(master, ICONS_DIR)
    write_iconset(master, ICONSET_DIR)
    run_iconutil(ICONSET_DIR, ICONS_DIR / "icon.icns")
    write_ico(master, ICONS_DIR / "icon.ico")

    # 关于页与 Vite 静态资源
    app_icon = DESKTOP_ROOT / "public" / "app-icon.png"
    shutil.copy2(ICONS_DIR / "icon.png", app_icon)

    icns_size = (ICONS_DIR / "icon.icns").stat().st_size
    print(f"已生成 macOS 图标: {ICONS_DIR}")
    print(f"  icon.icns 大小: {icns_size / 1024:.1f} KiB")
    print(f"  已同步: {app_icon}")


if __name__ == "__main__":
    main()
