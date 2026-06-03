#!/usr/bin/env python3
"""生成 README 用桌面端展示图（替代旧 Codex CLI 截图）。"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
ICON = ROOT / "public" / "app-icon.png"
OUT = ROOT.parents[1] / ".github" / "yunxi-desktop-splash.png"

WIDTH, HEIGHT = 1600, 900
MARGIN = 80


def load_font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ]
    for path in candidates:
        if Path(path).exists():
            try:
                return ImageFont.truetype(path, size=size, index=1 if bold else 0)
            except OSError:
                try:
                    return ImageFont.truetype(path, size=size)
                except OSError:
                    continue
    return ImageFont.load_default()


def draw_window(draw: ImageDraw.ImageDraw, box: tuple[int, int, int, int]) -> None:
    x0, y0, x1, y1 = box
    draw.rounded_rectangle(box, radius=18, fill=(18, 18, 22, 255), outline=(60, 60, 68, 255), width=2)
    title_h = 42
    draw.rounded_rectangle((x0, y0, x1, y0 + title_h), radius=18, fill=(28, 28, 34, 255))
    draw.rectangle((x0, y0 + title_h - 18, x1, y0 + title_h), fill=(28, 28, 34, 255))
    lights = [(x0 + 18, y0 + 16), (x0 + 38, y0 + 16), (x0 + 58, y0 + 16)]
    colors = [(255, 95, 87), (255, 189, 46), (40, 200, 64)]
    for (lx, ly), color in zip(lights, colors, strict=True):
        draw.ellipse((lx - 6, ly - 6, lx + 6, ly + 6), fill=color)


def main() -> None:
    if not ICON.is_file():
        raise SystemExit(f"缺少图标: {ICON}，请先运行 npm run icons:macos")

    canvas = Image.new("RGBA", (WIDTH, HEIGHT), (12, 12, 18, 255))
    draw = ImageDraw.Draw(canvas)

    # 背景渐变
    for y in range(HEIGHT):
        t = y / HEIGHT
        r = int(20 + (8 - 20) * t)
        g = int(18 + (10 - 18) * t)
        b = int(34 + (24 - 34) * t)
        draw.line([(0, y), (WIDTH, y)], fill=(r, g, b, 255))

    win = (MARGIN, MARGIN, WIDTH - MARGIN, HEIGHT - MARGIN)
    draw_window(draw, win)

    icon = Image.open(ICON).convert("RGBA").resize((28, 28), Image.Resampling.LANCZOS)
    canvas.paste(icon, (win[0] + 88, win[1] + 7), icon)

    title_font = load_font(18, bold=True)
    sub_font = load_font(13)
    draw.text((win[0] + 124, win[1] + 10), "云熙智能体", fill=(240, 240, 245, 255), font=title_font)
    draw.text((win[0] + 124, win[1] + 30), "YunPat Agent · 专利智能体", fill=(150, 150, 160, 255), font=sub_font)

    # 三栏布局示意
    inner_x0, inner_y0 = win[0] + 16, win[1] + 58
    inner_x1, inner_y1 = win[2] - 16, win[3] - 16
    sidebar_w = 220
    panel_w = 320
    draw.rounded_rectangle(
        (inner_x0, inner_y0, inner_x0 + sidebar_w, inner_y1),
        radius=12,
        fill=(24, 24, 30, 255),
        outline=(45, 45, 55, 255),
        width=1,
    )
    draw.rounded_rectangle(
        (inner_x0 + sidebar_w + 12, inner_y0, inner_x1 - panel_w - 12, inner_y1),
        radius=12,
        fill=(20, 20, 26, 255),
        outline=(45, 45, 55, 255),
        width=1,
    )
    draw.rounded_rectangle(
        (inner_x1 - panel_w, inner_y0, inner_x1, inner_y1),
        radius=12,
        fill=(24, 24, 30, 255),
        outline=(45, 45, 55, 255),
        width=1,
    )

    body_font = load_font(15)
    small_font = load_font(13)
    accent = (99, 102, 241, 255)

    draw.text((inner_x0 + 18, inner_y0 + 18), "资源管理器", fill=(210, 210, 220, 255), font=body_font)
    draw.text((inner_x0 + 18, inner_y0 + 52), "新建任务", fill=(150, 150, 160, 255), font=small_font)
    draw.text((inner_x0 + 18, inner_y0 + 78), "技能", fill=(150, 150, 160, 255), font=small_font)
    draw.text((inner_x0 + 18, inner_y0 + 104), "AI 助手", fill=(150, 150, 160, 255), font=small_font)

    cx = inner_x0 + sidebar_w + 28
    draw.text((cx, inner_y0 + 24), "你好，我是云熙专利智能体。", fill=(230, 230, 235, 255), font=body_font)
    draw.rounded_rectangle(
        (cx, inner_y0 + 58, cx + 520, inner_y0 + 110),
        radius=10,
        fill=(34, 34, 42, 255),
    )
    draw.text((cx + 14, inner_y0 + 74), "我可以帮你检索、分析、撰写与答复专利事务。", fill=(180, 180, 190, 255), font=small_font)

    draw.rounded_rectangle(
        (cx, inner_y1 - 72, inner_x1 - panel_w - 28, inner_y1 - 20),
        radius=10,
        fill=(28, 28, 36, 255),
        outline=(55, 55, 65, 255),
        width=1,
    )
    draw.text((cx + 14, inner_y1 - 58), "输入消息，或使用 / 命令…", fill=(120, 120, 130, 255), font=small_font)

    px = inner_x1 - panel_w + 18
    draw.text((px, inner_y0 + 18), "Agent", fill=(210, 210, 220, 255), font=body_font)
    draw.text((px, inner_y0 + 52), "● 已连接", fill=accent, font=small_font)
    draw.text((px, inner_y0 + 78), "模型 · 推理 · MCP", fill=(150, 150, 160, 255), font=small_font)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    canvas.convert("RGB").save(OUT, format="PNG", optimize=True)
    print(f"已生成 README 展示图: {OUT}")


if __name__ == "__main__":
    main()
