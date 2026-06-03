#!/usr/bin/env python3
"""按 macOS App Icon 规范从源图生成 Tauri 图标资源。

- 去除 logo 源图黑边/白底，仅保留品牌图形
- 1024 画布铺满 macOS 风格渐变背景（Big Sur 全出血图标）
- 图形主体缩放到约 824px 安全区并居中
- 不预烘焙圆角；由系统在 Dock/Finder 套用 squircle 遮罩
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

DESKTOP_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = DESKTOP_ROOT / "public" / "logo.png"
ICONS_DIR = DESKTOP_ROOT / "src-tauri" / "icons"
ICONSET_DIR = ICONS_DIR / "icon.iconset"

# 与 YunPat 品牌色一致的深色渐变背景
BG_TOP = (34, 34, 48)
BG_BOTTOM = (8, 8, 12)

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


def is_logo_pixel(r: int, g: int, b: int, a: int) -> bool:
    """识别品牌图形像素，排除透明、黑边与白底。"""
    if a < 16:
        return False
    if r < 40 and g < 40 and b < 40:
        return False
    if r > 245 and g > 245 and b > 245:
        return False
    return True


def logo_bbox(image: Image.Image) -> tuple[int, int, int, int]:
    rgba = image.convert("RGBA")
    w, h = rgba.size
    pixels = rgba.load()
    xs: list[int] = []
    ys: list[int] = []
    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if is_logo_pixel(r, g, b, a):
                xs.append(x)
                ys.append(y)
    if not xs:
        raise SystemExit("源图中未检测到品牌图形，请检查 logo.png")
    return min(xs), min(ys), max(xs) + 1, max(ys) + 1


def make_background(size: int) -> Image.Image:
    bg = Image.new("RGBA", (size, size))
    draw = ImageDraw.Draw(bg)
    for y in range(size):
        t = y / max(size - 1, 1)
        r = int(BG_TOP[0] + (BG_BOTTOM[0] - BG_TOP[0]) * t)
        g = int(BG_TOP[1] + (BG_BOTTOM[1] - BG_TOP[1]) * t)
        b = int(BG_TOP[2] + (BG_BOTTOM[2] - BG_TOP[2]) * t)
        draw.line([(0, y), (size, y)], fill=(r, g, b, 255))
    return bg


def extract_logo_mark(source: Image.Image) -> Image.Image:
    left, top, right, bottom = logo_bbox(source)
    mark = source.convert("RGBA").crop((left, top, right, bottom))
    pixels = mark.load()
    w, h = mark.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if not is_logo_pixel(r, g, b, a):
                pixels[x, y] = (0, 0, 0, 0)
    return mark


def compose_macos_icon(source: Path) -> Image.Image:
    src = Image.open(source)
    mark = extract_logo_mark(src)
    cw, ch = mark.size
    scale = min(SAFE / cw, SAFE / ch) * 0.92
    nw, nh = max(1, int(cw * scale)), max(1, int(ch * scale))
    resized = mark.resize((nw, nh), Image.Resampling.LANCZOS)

    canvas = make_background(CANVAS)
    ox = (CANVAS - nw) // 2
    oy = (CANVAS - nh) // 2
    canvas.paste(resized, (ox, oy), resized)
    return canvas


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


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SOURCE
    if not source.is_file():
        raise SystemExit(f"源图不存在: {source}")

    master = compose_macos_icon(source)

    ICONS_DIR.mkdir(parents=True, exist_ok=True)
    copy_bundle_pngs(master, ICONS_DIR)
    write_iconset(master, ICONSET_DIR)
    run_iconutil(ICONSET_DIR, ICONS_DIR / "icon.icns")
    write_ico(master, ICONS_DIR / "icon.ico")

    app_icon = DESKTOP_ROOT / "public" / "app-icon.png"
    shutil.copy2(ICONS_DIR / "icon.png", app_icon)

    icns_size = (ICONS_DIR / "icon.icns").stat().st_size
    print(f"已生成 macOS 图标: {ICONS_DIR}")
    print(f"  icon.icns 大小: {icns_size / 1024:.1f} KiB")
    print(f"  已同步: {app_icon}")


if __name__ == "__main__":
    main()
