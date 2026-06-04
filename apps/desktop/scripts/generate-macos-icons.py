#!/usr/bin/env python3
"""从 src-tauri/icons/logo.png 生成 macOS / Windows / Tauri 全套桌面图标。

- 去除 logo 源图黑边/白底，仅保留品牌图形
- 1024 画布铺满 macOS 风格渐变背景（Big Sur 全出血图标）
- 图形主体缩放到约 824px 安全区并居中
- 不预烘焙圆角；由系统在 Dock/Finder 套用 squircle 遮罩
- 所有 PNG 统一 72 DPI，iconset / 外层 PNG / icns / ico 同源
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageCms

# Apple macOS 图标：1024 画布上约 824×824 内容区
CANVAS = 1024
SAFE = 824
PNG_DPI = (72, 72)

DESKTOP_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = DESKTOP_ROOT / "src-tauri" / "icons" / "logo.png"
ICONS_DIR = DESKTOP_ROOT / "src-tauri" / "icons"
ICONSET_DIR = ICONS_DIR / "icon.iconset"

# 与 YunPat 品牌色一致的深色渐变背景
BG_TOP = (34, 34, 48)
BG_BOTTOM = (8, 8, 12)

# macOS iconset 标准尺寸（用于 iconutil 生成 .icns）
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

# Tauri bundle.icon 引用（tauri.conf.json）
TAURI_BUNDLE_PNGS: list[str] = [
    "icon.png",
    "icon_32x32.png",
    "icon_128x128.png",
    "icon_128x128@2x.png",
    "icon_256x256.png",
    "icon_512x512.png",
]

# 历史错误命名，生成后清理
LEGACY_ICON_FILES: list[str] = [
    "32x32.png",
    "128x128.png",
    "128x128@2x.png",
    "16x16.png",
    "256x256.png",
    "512x512.png",
]

# Windows Store 遗留资源（Tauri bundle 未引用）
LEGACY_WINDOWS_STORE_FILES: list[str] = [
    "Square30x30Logo.png",
    "Square44x44Logo.png",
    "Square71x71Logo.png",
    "Square89x89Logo.png",
    "Square107x107Logo.png",
    "Square142x142Logo.png",
    "Square150x150Logo.png",
    "Square284x284Logo.png",
    "Square310x310Logo.png",
    "StoreLogo.png",
]

_srgb_profile: bytes | None = None


def srgb_icc_profile() -> bytes:
    global _srgb_profile
    if _srgb_profile is None:
        _srgb_profile = ImageCms.ImageCmsProfile(
            ImageCms.createProfile("sRGB")
        ).tobytes()
    return _srgb_profile


def save_png(image: Image.Image, path: Path) -> None:
    rgb = image.convert("RGBA")
    rgb.save(
        path,
        format="PNG",
        optimize=True,
        dpi=PNG_DPI,
        icc_profile=srgb_icc_profile(),
    )


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
        save_png(resized, iconset_dir / name)


def sync_iconset_to_bundle(iconset_dir: Path, icons_dir: Path) -> None:
    """将 iconset 内全部尺寸同步到 icons/，保证 icns 与外层 PNG 同源。"""
    for name, _ in ICONSET_SIZES:
        shutil.copy2(iconset_dir / name, icons_dir / name)
    save_png(Image.open(iconset_dir / "icon_512x512@2x.png"), icons_dir / "icon.png")


def run_iconutil(iconset_dir: Path, icns_path: Path) -> None:
    subprocess.run(
        ["iconutil", "-c", "icns", str(iconset_dir), "-o", str(icns_path)],
        check=True,
    )


def write_ico(master: Image.Image, ico_path: Path) -> None:
    """Windows ICO：嵌入多尺寸以满足任务栏 / 资源管理器 / NSIS 安装器。"""
    sizes = (16, 24, 32, 48, 64, 128, 256)
    master.save(
        ico_path,
        format="ICO",
        sizes=[(edge, edge) for edge in sizes],
    )


def remove_legacy_files(icons_dir: Path) -> None:
    for name in LEGACY_ICON_FILES + LEGACY_WINDOWS_STORE_FILES:
        path = icons_dir / name
        if path.is_file():
            path.unlink()


def sync_public_assets(icons_dir: Path) -> None:
    public_dir = DESKTOP_ROOT / "public"
    public_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(icons_dir / "icon.png", public_dir / "app-icon.png")
    shutil.copy2(DEFAULT_SOURCE, public_dir / "logo.png")


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SOURCE
    if not source.is_file():
        raise SystemExit(f"源图不存在: {source}")

    master = compose_macos_icon(source)

    ICONS_DIR.mkdir(parents=True, exist_ok=True)
    remove_legacy_files(ICONS_DIR)
    write_iconset(master, ICONSET_DIR)
    sync_iconset_to_bundle(ICONSET_DIR, ICONS_DIR)
    run_iconutil(ICONSET_DIR, ICONS_DIR / "icon.icns")
    write_ico(master, ICONS_DIR / "icon.ico")
    sync_public_assets(ICONS_DIR)

    icns_size = (ICONS_DIR / "icon.icns").stat().st_size
    print(f"源图: {source}")
    print(f"已生成桌面图标: {ICONS_DIR}")
    print(f"  icon.icns: {icns_size / 1024:.1f} KiB")
    print(f"  Tauri bundle PNG: {', '.join(TAURI_BUNDLE_PNGS)}")
    print(f"  已同步 public/app-icon.png 与 public/logo.png")


if __name__ == "__main__":
    main()
