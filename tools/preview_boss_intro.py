#!/usr/bin/env python3
"""
Render storyboard mockup PNGs of the v0.3.2 boss break-in cinematic so
the user can preview each phase before the live deployment.

Output: docs/preview/boss_intro/phase_*.png

These are NOT loaded by the wasm runtime; they exist solely as design
references viewable on GitHub. The actual cinematic is rendered live by
src/render/sprites.rs::draw_boss_intro.
"""
from __future__ import annotations

import math
import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "docs" / "preview" / "boss_intro"
OUT_DIR.mkdir(parents=True, exist_ok=True)

W, H = 1280, 400


def new_canvas(w: int, h: int, fill=(40, 38, 60, 255)) -> Image.Image:
    return Image.new("RGBA", (w, h), fill)


def get_font(size: int) -> ImageFont.FreeTypeFont:
    # Pillow ships a default bitmap; for monospace try DejaVu if present.
    candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    ]
    for c in candidates:
        if os.path.isfile(c):
            return ImageFont.truetype(c, size)
    return ImageFont.load_default()


def text_dim(d: ImageDraw.ImageDraw, text: str, font) -> tuple[int, int]:
    bbox = d.textbbox((0, 0), text, font=font)
    return bbox[2] - bbox[0], bbox[3] - bbox[1]


def draw_factory_bg(d: ImageDraw.ImageDraw) -> None:
    """Cheap mock of the AeiROBOT Factory backdrop -- desaturated grays
    so the cinematic overlay reads on top."""
    d.rectangle((0, 0, W, 200), fill=(80, 88, 100, 255))   # sky
    d.rectangle((0, 200, W, 320), fill=(72, 78, 90, 255))  # mid
    d.rectangle((0, 320, W, 400), fill=(58, 62, 72, 255))  # floor
    # Some far-layer building blocks
    for i, fx in enumerate(range(-40, W, 220)):
        d.rectangle((fx, 110 + (i % 2) * 30, fx + 180, 200), fill=(64, 70, 86, 255))
        d.rectangle((fx + 4, 114 + (i % 2) * 30, fx + 176, 200), fill=(86, 92, 110, 255))
    # EDIE silhouette at PLAYER_X
    edie_x, edie_y = 200, 248
    d.ellipse((edie_x - 22, edie_y - 12, edie_x + 22, edie_y + 32), fill=(248, 244, 232, 255), outline=(20, 20, 20, 255), width=2)
    d.point((edie_x - 8, edie_y + 4), fill=(20, 20, 20, 255))
    d.point((edie_x + 8, edie_y + 4), fill=(20, 20, 20, 255))


def draw_boss(d: ImageDraw.ImageDraw, cx: int, cy: int, r: int = 90) -> None:
    """Stylised mungchi boss with zigzag eyes (mocks the new sprite)."""
    out = (20, 20, 20, 255)
    # Spike halo
    for i in range(24):
        a = i / 24 * math.tau
        x1 = cx + math.cos(a) * (r * 0.6)
        y1 = cy + math.sin(a) * (r * 0.6)
        x2 = cx + math.cos(a) * (r * 1.05)
        y2 = cy + math.sin(a) * (r * 1.05)
        d.line((x1, y1, x2, y2), fill=(30, 110, 40, 255), width=4)
        d.ellipse((x2 - 6, y2 - 6, x2 + 6, y2 + 6), fill=(80, 180, 90, 255), outline=out)
    # Body
    d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=(60, 200, 80, 255), outline=out, width=3)
    d.ellipse((cx - r * 0.85, cy - r * 0.85, cx + r * 0.6, cy + r * 0.6), fill=(120, 230, 140, 255))
    d.ellipse((cx - r * 0.6, cy - r * 0.6, cx + r * 0.6, cy + r * 0.6), fill=(60, 200, 80, 255))
    # Zigzag pumpkin slit eyes
    for ex in (cx - 30, cx + 30):
        flip = ex > cx
        xs = [-20, -10, -2, 8, 16, 20, 14, 6, -2, -8, -14, -18, -20]
        ys = [-2,  -8,  -2, -10, -2, -8, 10, 4, 12, 4, 12, 4, 10]
        if flip:
            xs = [-v for v in reversed(xs)]
            ys = list(reversed(ys))
        pts = [(ex + xs[i], cy - 4 + ys[i]) for i in range(len(xs))]
        d.polygon(pts, fill=(20, 20, 24, 255), outline=out)
        inner = [(ex + int(xs[i] * 0.7), cy - 4 + int(ys[i] * 0.7)) for i in range(len(xs))]
        d.polygon(inner, fill=(255, 90, 30, 255))
        inner2 = [(ex + int(xs[i] * 0.4), cy - 4 + int(ys[i] * 0.4)) for i in range(len(xs))]
        d.polygon(inner2, fill=(255, 220, 100, 255))


def save_phase(name: str, im: Image.Image) -> None:
    out = OUT_DIR / f"{name}.png"
    im.save(out)
    print(f"  OK {out.relative_to(ROOT)}")


# ============================================================
# Phase frames (one PNG per phase, illustrating the key moment).
# ============================================================
def phase_1_alert() -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    # Red warning border
    d.rectangle((0, 0, W, 12), fill=(220, 40, 50, 255))
    d.rectangle((0, H - 12, W, H), fill=(220, 40, 50, 255))
    d.rectangle((0, 0, 12, H), fill=(220, 40, 50, 255))
    d.rectangle((W - 12, 0, W, H), fill=(220, 40, 50, 255))
    # Dim
    d.rectangle((0, 0, W, H), fill=(0, 0, 0, 70))
    f = get_font(64)
    txt = "!! WARNING !!"
    tw, th = text_dim(d, txt, f)
    d.text((W // 2 - tw // 2 + 4, 110 + 4), txt, fill=(0, 0, 0, 200), font=f)
    d.text((W // 2 - tw // 2, 110), txt, fill=(255, 80, 90, 255), font=f)
    f2 = get_font(22)
    note = "PHASE 1 / 8 -- 0.0s..0.8s"
    nw, _ = text_dim(d, note, f2)
    d.text((W // 2 - nw // 2, 200), note, fill=(255, 220, 220, 200), font=f2)
    return im


def phase_2_glitch() -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    # Dim
    d.rectangle((0, 0, W, H), fill=(0, 0, 0, 130))
    # Scanlines
    for y in range(0, H, 6):
        d.rectangle((0, y, W, y + 2), fill=(40, 220, 80, 80))
    # RGB-split bars
    for by in (60, 110, 180, 240, 290, 340):
        d.rectangle((0, by, W, by + 8), fill=(255, 60, 70, 70))
        d.rectangle((6, by, W, by + 8), fill=(60, 255, 100, 50))
    f2 = get_font(22)
    note = "PHASE 2 / 8 -- 0.8s..1.6s -- GLITCH WIPE"
    d.text((40, H - 40), note, fill=(255, 255, 220, 220), font=f2)
    return im


def phase_3_slam() -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    d.rectangle((0, 0, W, H), fill=(0, 0, 0, 170))
    # Boss mid-fall
    draw_boss(d, W // 2, 130, r=80)
    # Trail glow above
    for i in range(1, 4):
        a = 80 - i * 18
        d.rectangle((W // 2 - 80, 130 - 80 - i * 22, W // 2 + 80, 130 - 80 - i * 22 + 24), fill=(70, 220, 110, a))
    # Shockwave starting
    for r in (60, 100, 140):
        d.ellipse((W // 2 - r, 200 - r // 3, W // 2 + r, 200 + r // 3), outline=(255, 240, 180, 220), width=4)
    f2 = get_font(22)
    note = "PHASE 3 / 8 -- 1.6s..2.6s -- BOSS SLAM"
    d.text((40, H - 40), note, fill=(255, 255, 220, 220), font=f2)
    return im


def phase_4_dialog1() -> Image.Image:
    return _dialog_phase("AEIROBOT IS MINE NOW...", "PHASE 4 / 8 -- 2.6s..4.8s -- DIALOG 1")


def phase_5_dialog2() -> Image.Image:
    return _dialog_phase("SUBMIT AND BE INFECTED!", "PHASE 5 / 8 -- 4.8s..6.8s -- DIALOG 2")


def _dialog_phase(line: str, label: str) -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    d.rectangle((0, 0, W, H), fill=(0, 0, 0, 200))
    # Boss in centre
    draw_boss(d, W // 2, 130, r=80)
    # Dialog box
    box = (60, 260, 60 + 1160, 260 + 120)
    d.rectangle(box, fill=(10, 10, 20, 235))
    d.rectangle(box, outline=(255, 240, 180, 240), width=4)
    d.rectangle((box[0] + 4, box[1] + 4, box[2] - 4, box[3] - 4), outline=(80, 220, 100, 200), width=2)
    # Portrait
    pbox = (80, 278, 80 + 84, 278 + 84)
    d.rectangle(pbox, fill=(20, 50, 24, 255))
    d.rectangle(pbox, outline=(80, 220, 100, 230), width=3)
    draw_boss(d, (pbox[0] + pbox[2]) // 2, (pbox[1] + pbox[3]) // 2 + 2, r=32)
    # Name label
    fname = get_font(22)
    nw, _ = text_dim(d, "MUNGCHI", fname)
    d.text(((pbox[0] + pbox[2]) // 2 - nw // 2, pbox[1] - 26), "MUNGCHI", fill=(80, 220, 100, 255), font=fname)
    # Dialog text
    fline = get_font(36)
    d.text((190 + 3, 320 + 3), line, fill=(0, 0, 0, 200), font=fline)
    d.text((190, 320), line, fill=(245, 245, 215, 255), font=fline)
    # Phase label
    f2 = get_font(20)
    d.text((40, 16), label, fill=(255, 200, 200, 220), font=f2)
    return im


def phase_6_charge() -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    d.rectangle((0, 0, W, H), fill=(0, 0, 0, 200))
    draw_boss(d, W // 2, 130, r=80)
    # EDIE charge streak
    edie_x = 480
    edie_y = 252
    for k in range(6):
        sx = edie_x - (k + 1) * 30
        a = max(0, 200 - k * 30)
        d.rectangle((sx, edie_y - 8, sx + 60, edie_y + 40), fill=(255, 255, 255, a))
    # EDIE
    d.ellipse((edie_x - 22, edie_y - 12, edie_x + 22, edie_y + 32), fill=(248, 244, 232, 255), outline=(20, 20, 20, 255), width=2)
    d.point((edie_x - 8, edie_y + 4), fill=(20, 20, 20, 255))
    d.point((edie_x + 8, edie_y + 4), fill=(20, 20, 20, 255))
    # "TAKE THIS!"
    f = get_font(34)
    txt = "TAKE THIS!"
    tw, _ = text_dim(d, txt, f)
    d.text((edie_x + 30 + 2, 200 + 2), txt, fill=(0, 0, 0, 220), font=f)
    d.text((edie_x + 30, 200), txt, fill=(255, 220, 80, 255), font=f)
    f2 = get_font(20)
    d.text((40, 16), "PHASE 6 / 8 -- 6.8s..7.6s -- EDIE CHARGE", fill=(255, 255, 220, 220), font=f2)
    return im


def phase_7_impact() -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    d.rectangle((0, 0, W, H), fill=(255, 255, 230, 200))  # White flash
    draw_boss(d, W // 2 + 30, 140, r=80)
    f = get_font(96)
    txt = "THWACK!!"
    tw, _ = text_dim(d, txt, f)
    d.text((W // 2 - tw // 2 + 5, 150 + 5), txt, fill=(0, 0, 0, 230), font=f)
    d.text((W // 2 - tw // 2, 150), txt, fill=(255, 70, 70, 255), font=f)
    f2 = get_font(20)
    d.text((40, 16), "PHASE 7 / 8 -- 7.6s..8.4s -- IMPACT", fill=(40, 40, 40, 230), font=f2)
    return im


def phase_8_fight() -> Image.Image:
    im = new_canvas(W, H)
    d = ImageDraw.Draw(im)
    draw_factory_bg(d)
    d.rectangle((0, 0, W, H), fill=(0, 0, 0, 100))
    draw_boss(d, W // 2, 140, r=85)
    f = get_font(56)
    txt = "-- FIGHT! --"
    tw, _ = text_dim(d, txt, f)
    d.text((W // 2 - tw // 2 + 3, 320 + 3), txt, fill=(0, 0, 0, 220), font=f)
    d.text((W // 2 - tw // 2, 320), txt, fill=(255, 220, 80, 255), font=f)
    f2 = get_font(20)
    d.text((40, 16), "PHASE 8 / 8 -- 8.4s..9.6s -- FIGHT START", fill=(255, 255, 220, 220), font=f2)
    return im


def main() -> None:
    print(f"Rendering boss intro storyboard to {OUT_DIR}")
    save_phase("phase_1_alert", phase_1_alert())
    save_phase("phase_2_glitch", phase_2_glitch())
    save_phase("phase_3_slam", phase_3_slam())
    save_phase("phase_4_dialog1", phase_4_dialog1())
    save_phase("phase_5_dialog2", phase_5_dialog2())
    save_phase("phase_6_charge", phase_6_charge())
    save_phase("phase_7_impact", phase_7_impact())
    save_phase("phase_8_fight", phase_8_fight())


if __name__ == "__main__":
    main()
