#!/usr/bin/env python3
"""
EDIE Runner — Phase 2 art generator.

Reads source assets from `assets/source/`, produces game-ready PNGs in
`assets/gen/`. Re-running is deterministic and idempotent.

Usage:
    python tools/generate_art.py
"""
from __future__ import annotations

import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter
import numpy as np

# ============================================================
# Locked palette (spec §6.2)
# ============================================================
EDIE_OUTLINE = (26, 26, 26, 255)
EDIE_WHITE = (255, 255, 255, 255)
EDIE_SHADE = (216, 216, 216, 255)
EDIE_ORANGE = (232, 146, 60, 255)
EDIE_ORANGE_DEEP = (184, 106, 31, 255)

BG_SKY = (245, 239, 228, 255)
BG_FAR = (194, 194, 184, 255)  # cool-shifted from #C9C2B2 per reviewer
BG_MID = (142, 134, 118, 255)
FLOOR = (74, 68, 56, 255)
FLOOR_LINE = (46, 42, 34, 255)

AURORA_PURPLE = (157, 107, 255, 255)
AURORA_PURPLE_HI = (211, 184, 255, 255)
AURORA_GREEN = (91, 227, 168, 255)
AURORA_GREEN_HI = (184, 245, 221, 255)
AURORA_GLOW = (255, 255, 255, 80)

HAZARD = (230, 57, 70, 255)
COOL_ACCENT = (74, 165, 200, 255)  # cool teal-blue used on obstacles
TRANSPARENT = (0, 0, 0, 0)

EDIE_PALETTE = [
    (26, 26, 26),
    (255, 255, 255),
    (216, 216, 216),
    (232, 146, 60),
    (184, 106, 31),
]

# ============================================================
# Paths
# ============================================================
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "source"
GEN = ROOT / "assets" / "gen"
GEN.mkdir(parents=True, exist_ok=True)


# ============================================================
# Helpers
# ============================================================
def new_canvas(w: int, h: int) -> Image.Image:
    return Image.new("RGBA", (w, h), TRANSPARENT)


def save_png(im: Image.Image, name: str, palette_lock: bool = True) -> None:
    out = GEN / name
    im.save(out)
    n_colors = count_unique_colors(im)
    if palette_lock and n_colors > 16:
        raise SystemExit(f"PALETTE LOCK FAIL: {name} has {n_colors} unique colors")
    print(f"  OK {name} {im.size}  ({n_colors} colors)")


def count_unique_colors(im: Image.Image) -> int:
    a = np.array(im.convert("RGBA"))
    opaque = a[a[:, :, 3] > 0]
    if len(opaque) == 0:
        return 0
    return len({tuple(p[:3]) for p in opaque})


def quantize_to_palette(rgba: np.ndarray, palette: list[tuple[int, int, int]]) -> np.ndarray:
    h, w = rgba.shape[:2]
    out = np.zeros_like(rgba)
    out[:, :, 3] = rgba[:, :, 3]
    rgb = rgba[:, :, :3].astype(np.int32)
    pal = np.array(palette)
    for y in range(h):
        for x in range(w):
            if rgba[y, x, 3] < 128:
                out[y, x] = (0, 0, 0, 0)
                continue
            d = ((pal - rgb[y, x]) ** 2).sum(axis=1)
            idx = int(d.argmin())
            out[y, x, :3] = pal[idx]
            out[y, x, 3] = 255
    return out


def tile_horizontal(frames: list[Image.Image], padding: int = 1) -> Image.Image:
    if not frames:
        raise ValueError("no frames")
    h = max(f.height for f in frames)
    w = sum(f.width for f in frames) + padding * (len(frames) - 1)
    sheet = new_canvas(w, h)
    x = 0
    for f in frames:
        sheet.paste(f, (x, h - f.height), f)
        x += f.width + padding
    return sheet


def outline(im: Image.Image, color=(26, 26, 26, 255)) -> Image.Image:
    """Add 1-pixel hard outline around opaque pixels."""
    a = np.array(im)
    alpha = a[:, :, 3]
    h, w = alpha.shape
    out = a.copy()
    for y in range(h):
        for x in range(w):
            if alpha[y, x] > 0:
                continue
            for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                ny, nx = y + dy, x + dx
                if 0 <= ny < h and 0 <= nx < w and alpha[ny, nx] > 0:
                    out[y, x] = color
                    break
    return Image.fromarray(out, "RGBA")


# ============================================================
# EDIE — from user-provided references
# ============================================================
def process_edie_refs(target_h: int = 48) -> tuple[Image.Image, Image.Image]:
    print("[EDIE] processing source refs")
    out = {}
    for src in ("edie_ref_run.png", "edie_ref_jump.png"):
        im = Image.open(SOURCE / src).convert("RGBA")
        a = np.array(im)
        alpha = a[:, :, 3]
        rows = np.any(alpha > 128, axis=1)
        cols = np.any(alpha > 128, axis=0)
        y0, y1 = np.where(rows)[0][[0, -1]]
        x0, x1 = np.where(cols)[0][[0, -1]]
        cropped = im.crop((x0, y0, x1 + 1, y1 + 1))
        cw, ch = cropped.size
        new_w = max(1, round(cw * target_h / ch))
        small = cropped.resize((new_w, target_h), Image.LANCZOS)
        arr = np.array(small)
        quant = quantize_to_palette(arr, EDIE_PALETTE)
        result = Image.fromarray(quant, "RGBA")
        key = src.replace("_ref_", "_").replace(".png", ".png")
        out[key] = result
        save_png(result, key, palette_lock=True)
    return out["edie_run.png"], out["edie_jump.png"]


def derive_edie_states(run_im: Image.Image) -> None:
    print("[EDIE] deriving duck/dash/hit/shadow")
    w, h = run_im.size

    # Duck — dramatic vertical squash to ~50% height for clear visual feedback
    duck_h = int(h * 0.50)
    duck = run_im.resize((w, duck_h), Image.NEAREST)
    save_png(duck, "edie_duck.png")

    # Dash — replace white→orange, shade→deep orange
    a = np.array(run_im)
    dash = a.copy()
    mask_white = (a[:, :, :3] == [255, 255, 255]).all(axis=2) & (a[:, :, 3] > 0)
    mask_shade = (a[:, :, :3] == [216, 216, 216]).all(axis=2) & (a[:, :, 3] > 0)
    dash[mask_white] = (232, 146, 60, 255)
    dash[mask_shade] = (184, 106, 31, 255)
    save_png(Image.fromarray(dash, "RGBA"), "edie_dash.png")

    # Hit — overlay red 50% blend on white
    hit = a.copy()
    mask_body = ((a[:, :, :3] == [255, 255, 255]) | (a[:, :, :3] == [216, 216, 216])).all(axis=2) & (a[:, :, 3] > 0)
    hit[mask_body] = (230, 57, 70, 255)
    save_png(Image.fromarray(hit, "RGBA"), "edie_hit.png")

    # Shadow — flat ellipse, separate file
    shadow = new_canvas(20, 6)
    d = ImageDraw.Draw(shadow)
    d.ellipse((0, 0, 19, 5), fill=(0, 0, 0, 110))
    save_png(shadow, "edie_shadow.png", palette_lock=False)


# ============================================================
# Obstacles
# ============================================================
def make_coffee_cup() -> None:
    w, h = 24, 32
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    # Steam wisps
    d.line((10, 1, 10, 4), fill=(220, 220, 220, 200))
    d.line((13, 1, 13, 5), fill=(220, 220, 220, 200))
    # Lid
    d.rectangle((4, 6, w - 5, 9), fill=(170, 70, 30, 255), outline=EDIE_OUTLINE, width=1)
    d.point((10, 7), fill=EDIE_WHITE)
    # Cup body — Starbucks-like brown
    d.polygon(
        [(5, 9), (w - 6, 9), (w - 7, h - 2), (6, h - 2)],
        fill=(120, 80, 50, 255),
        outline=EDIE_OUTLINE,
    )
    # Sleeve / band
    d.rectangle((5, 18, w - 6, 22), fill=(180, 130, 80, 255))
    # Logo dot
    d.point((11, 20), fill=EDIE_WHITE)
    save_png(im, "obstacle_coffee.png", palette_lock=False)


def make_shopping_cart() -> None:
    w, h = 80, 44
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    # Frame outline
    metal = (180, 180, 190, 255)
    metal_dark = (110, 110, 120, 255)
    # Basket
    d.rectangle((6, 12, w - 6, h - 14), outline=EDIE_OUTLINE, width=1, fill=metal)
    # Vertical bars
    for bx in range(10, w - 8, 6):
        d.line((bx, 14, bx, h - 16), fill=metal_dark)
    # Horizontal bar
    d.line((6, 22, w - 6, 22), fill=metal_dark)
    # Handle
    d.line((w - 6, 12, w - 2, 4), fill=EDIE_OUTLINE)
    d.line((w - 5, 12, w - 1, 4), fill=metal_dark)
    # Wheels
    d.ellipse((10, h - 12, 18, h - 4), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=1)
    d.ellipse((w - 18, h - 12, w - 10, h - 4), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=1)
    save_png(im, "obstacle_cart.png", palette_lock=False)


def make_cat() -> None:
    """Sleeping orange cat — 2-frame breathing."""
    frames = []
    for f in range(2):
        w, h = 40, 24
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        orange = (220, 130, 50, 255)
        orange_d = (160, 80, 30, 255)
        white = EDIE_WHITE
        outline = EDIE_OUTLINE
        # Body — curled lump
        breathe = f
        d.ellipse((4, 8 - breathe, w - 5, h - 2), fill=orange, outline=outline, width=1)
        # Tummy fade
        d.ellipse((10, 14, w - 12, h - 4), fill=orange_d)
        # Head — left side
        d.ellipse((2, 4 - breathe, 16, 18 - breathe), fill=orange, outline=outline, width=1)
        # Ears
        d.polygon([(4, 5 - breathe), (6, 1 - breathe), (8, 5 - breathe)], fill=orange, outline=outline)
        d.polygon([(11, 5 - breathe), (13, 1 - breathe), (15, 5 - breathe)], fill=orange, outline=outline)
        # Closed eye (sleeping ^)
        d.line((6, 10 - breathe, 8, 9 - breathe), fill=outline)
        d.line((8, 9 - breathe, 10, 10 - breathe), fill=outline)
        # Stripes on body
        d.line((20, 14, 24, 12), fill=orange_d)
        d.line((26, 15, 30, 13), fill=orange_d)
        # Tail curling around
        d.line((w - 6, 18, w - 2, 14), fill=orange, width=2)
        d.line((w - 6, 18, w - 2, 14), fill=outline)
        # White whisker spot
        d.point((4, 12 - breathe), fill=white)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_cat.png", palette_lock=False)


def make_vacuum_bot() -> None:
    """Generic round vacuum robot — 4-frame indicator blink."""
    frames = []
    for f in range(4):
        w, h = 40, 20
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        # Disc body
        d.ellipse((1, 4, w - 2, h - 1), fill=(220, 220, 230, 255), outline=EDIE_OUTLINE, width=1)
        # Bumper
        d.ellipse((3, 6, w - 4, h - 3), fill=(180, 180, 190, 255))
        # Top sensor
        d.rectangle((w // 2 - 3, 1, w // 2 + 3, 5), fill=(70, 70, 80, 255), outline=EDIE_OUTLINE, width=1)
        # Indicator LED — animated
        led_colors = [
            (60, 230, 120, 255),
            (60, 230, 120, 255),
            (40, 100, 60, 255),
            (40, 100, 60, 255),
        ]
        d.rectangle((w // 2 - 1, 2, w // 2, 3), fill=led_colors[f])
        save_png_no_lock = True
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_vacuum.png", palette_lock=False)


def make_amy() -> None:
    """Amy — small white flying AeiROBOT (round body, single eye, hover ring)."""
    frames = []
    for f in range(4):
        w, h = 44, 32
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        outline = EDIE_OUTLINE
        white = EDIE_WHITE
        # Body — egg shape
        d.ellipse((10, 4, w - 11, h - 10), fill=white, outline=outline, width=1)
        # Eye lens — large orange
        d.ellipse((16, 9, 28, 19), fill=outline)
        d.ellipse((18, 11, 26, 17), fill=EDIE_ORANGE)
        d.point((20, 13), fill=white)
        # Antenna
        d.line((22, 4, 22, 1), fill=outline)
        d.point((22, 0), fill=EDIE_ORANGE)
        # Hover ring (animated stretch)
        ring_y = h - 6 + (f % 2)
        d.ellipse((6, ring_y - 2, w - 7, ring_y + 2), outline=(120, 200, 255, 255))
        # Side thrusters
        d.rectangle((4, 14, 9, 18), fill=(100, 180, 230, 255), outline=outline, width=1)
        d.rectangle((w - 10, 14, w - 5, 18), fill=(100, 180, 230, 255), outline=outline, width=1)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_amy.png", palette_lock=False)


def make_alice_m1() -> None:
    """Alice-M1 — wheeled mobile AeiROBOT (compact, antenna)."""
    frames = []
    for f in range(2):
        w, h = 36, 36
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        outline = EDIE_OUTLINE
        white = EDIE_WHITE
        # Body — rectangular with rounded corners
        d.rectangle((4, 8, w - 5, h - 10), fill=white, outline=outline, width=1)
        # Top dome
        d.ellipse((6, 2, w - 7, 12), fill=white, outline=outline, width=1)
        # Eye strip
        d.rectangle((9, 14, w - 10, 18), fill=outline)
        d.rectangle((11, 15, 13, 17), fill=EDIE_ORANGE)
        d.rectangle((w - 14, 15, w - 12, 17), fill=EDIE_ORANGE)
        # Body label badge
        d.rectangle((12, 22, w - 13, 26), fill=(60, 80, 120, 255))
        # Antenna with blinking tip
        d.line((w // 2, 2, w // 2, -1), fill=outline)
        tip = EDIE_ORANGE if f == 0 else (200, 80, 30, 255)
        d.point((w // 2, 0), fill=tip)
        # Wheels
        d.ellipse((4, h - 10, 14, h - 1), fill=(40, 40, 46, 255), outline=outline, width=1)
        d.ellipse((w - 14, h - 10, w - 4, h - 1), fill=(40, 40, 46, 255), outline=outline, width=1)
        d.ellipse((7, h - 7, 11, h - 4), fill=(120, 120, 130, 255))
        d.ellipse((w - 11, h - 7, w - 7, h - 4), fill=(120, 120, 130, 255))
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_alicem1.png", palette_lock=False)


def make_alice3() -> None:
    """Alice3 — humanoid AeiROBOT v3, white panels with orange accent."""
    frames = []
    for f in range(2):
        w, h = 32, 64
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        outline = EDIE_OUTLINE
        white = EDIE_WHITE
        accent = EDIE_ORANGE
        # Head
        d.rectangle((9, 2, w - 10, 14), fill=white, outline=outline, width=1)
        # Visor
        d.rectangle((11, 6, w - 12, 10), fill=outline)
        d.rectangle((12, 7, 14, 9), fill=accent)
        d.rectangle((17, 7, 19, 9), fill=accent)
        # Neck
        d.rectangle((14, 14, w - 15, 16), fill=(140, 140, 150, 255))
        # Torso
        d.rectangle((6, 16, w - 7, 36), fill=white, outline=outline, width=1)
        # Chest accent
        d.rectangle((11, 20, w - 12, 26), fill=accent)
        d.rectangle((12, 22, w - 13, 24), fill=(180, 70, 20, 255))
        # Body label
        d.rectangle((9, 30, w - 10, 34), fill=(60, 80, 120, 255))
        # Arms — slight idle sway
        sway = f
        d.rectangle((2, 18, 5, 36 + sway), fill=white, outline=outline, width=1)
        d.rectangle((w - 6, 18, w - 3, 36 - sway), fill=white, outline=outline, width=1)
        # Hips
        d.rectangle((8, 36, w - 9, 40), fill=(140, 140, 150, 255), outline=outline, width=1)
        # Legs
        d.rectangle((9, 40, 14, h - 4), fill=white, outline=outline, width=1)
        d.rectangle((w - 15, 40, w - 10, h - 4), fill=white, outline=outline, width=1)
        # Feet
        d.rectangle((8, h - 5, 16, h - 1), fill=outline)
        d.rectangle((w - 17, h - 5, w - 9, h - 1), fill=outline)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_alice3.png", palette_lock=False)


def make_alice4() -> None:
    """Alice4 — newer humanoid AeiROBOT v4, sleeker silhouette."""
    frames = []
    for f in range(2):
        w, h = 36, 68
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        outline = EDIE_OUTLINE
        white = EDIE_WHITE
        accent = EDIE_ORANGE
        teal = (60, 180, 200, 255)
        # Head — taller helmet
        d.rectangle((10, 2, w - 11, 16), fill=white, outline=outline, width=1)
        # Curved visor
        d.rectangle((11, 6, w - 12, 11), fill=outline)
        d.rectangle((12, 7, w - 13, 10), fill=teal)
        d.point((14, 8), fill=EDIE_WHITE)
        # Forehead emblem
        d.point((w // 2, 4), fill=accent)
        d.point((w // 2 - 1, 4), fill=accent)
        # Neck
        d.rectangle((15, 16, w - 16, 18), fill=(120, 120, 130, 255))
        # Torso — broader shoulders
        d.polygon(
            [(4, 18), (w - 5, 18), (w - 7, 40), (6, 40)],
            fill=white,
            outline=outline,
        )
        # Chest panel
        d.rectangle((11, 22, w - 12, 30), fill=accent)
        d.rectangle((12, 24, w - 13, 26), fill=(255, 200, 100, 255))
        # AeiROBOT badge
        d.rectangle((10, 32, w - 11, 38), fill=(40, 60, 100, 255))
        d.point((w // 2, 35), fill=teal)
        # Arms with shoulder pads
        sway = f
        d.rectangle((1, 19, 5, 22), fill=white, outline=outline, width=1)
        d.rectangle((w - 6, 19, w - 2, 22), fill=white, outline=outline, width=1)
        d.rectangle((2, 22, 5, 40 + sway), fill=white, outline=outline, width=1)
        d.rectangle((w - 6, 22, w - 3, 40 - sway), fill=white, outline=outline, width=1)
        # Hips
        d.rectangle((8, 40, w - 9, 44), fill=(120, 120, 130, 255), outline=outline, width=1)
        # Legs — longer
        d.rectangle((10, 44, 16, h - 4), fill=white, outline=outline, width=1)
        d.rectangle((w - 17, 44, w - 11, h - 4), fill=white, outline=outline, width=1)
        # Knee accent
        d.rectangle((11, 54, 15, 56), fill=accent)
        d.rectangle((w - 16, 54, w - 12, 56), fill=accent)
        # Feet
        d.rectangle((8, h - 5, 18, h - 1), fill=outline)
        d.rectangle((w - 19, h - 5, w - 9, h - 1), fill=outline)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_alice4.png", palette_lock=False)


def make_sensor_cone() -> None:
    w, h = 24, 32
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    # Cone shape — orange
    d.polygon([(w // 2, 2), (w - 3, h - 4), (3, h - 4)], fill=(232, 146, 60, 255), outline=EDIE_OUTLINE)
    # White chevron stripe
    d.line((6, h // 2 + 2, w // 2, h // 2 - 2), fill=EDIE_WHITE, width=1)
    d.line((w // 2, h // 2 - 2, w - 7, h // 2 + 2), fill=EDIE_WHITE, width=1)
    # Base
    d.rectangle((2, h - 5, w - 3, h - 2), fill=(60, 60, 66, 255), outline=EDIE_OUTLINE, width=1)
    save_png(im, "obstacle_cone.png")


def make_sign_board() -> None:
    w, h = 24, 24
    frames = []
    for f in range(4):
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        cx, cy = 12, 12
        radius = 2 + f * 2
        d.ellipse((cx - radius, cy - radius, cx + radius, cy + radius),
                  fill=(255, 230, 100, 255), outline=EDIE_OUTLINE)
        if f >= 2:
            for ang in range(0, 360, 45):
                import math
                rx = cx + int(math.cos(math.radians(ang)) * (radius + 2))
                ry = cy + int(math.sin(math.radians(ang)) * (radius + 2))
                d.line((cx, cy, rx, ry), fill=HAZARD, width=1)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_sign.png", palette_lock=False)


# ============================================================
# Aurora Stones
# ============================================================
def make_aurora() -> None:
    for variant, core, hi in [
        ("purple", AURORA_PURPLE, AURORA_PURPLE_HI),
        ("green", AURORA_GREEN, AURORA_GREEN_HI),
    ]:
        frames = []
        for f in range(6):
            w, h = 48, 48
            im = new_canvas(w, h)
            d = ImageDraw.Draw(im)
            cx, cy = 24, 24
            # Outer soft halo — large, low alpha
            for r_off, a in [(22, 30), (18, 55), (14, 85)]:
                d.ellipse(
                    (cx - r_off, cy - r_off, cx + r_off, cy + r_off),
                    fill=(255, 255, 255, a),
                )
            # Mid pulse ring
            ring_r = 10 + (f % 3) * 2
            d.ellipse(
                (cx - ring_r, cy - ring_r, cx + ring_r, cy + ring_r),
                fill=hi,
                outline=EDIE_OUTLINE,
                width=1,
            )
            # Core orb
            r = 6 + (f % 2)
            d.ellipse(
                (cx - r, cy - r, cx + r, cy + r),
                fill=core,
                outline=EDIE_OUTLINE,
                width=1,
            )
            # Highlight glint
            d.ellipse((cx - 3, cy - 4, cx - 1, cy - 2), fill=EDIE_WHITE)
            frames.append(im)
        sheet = tile_horizontal(frames)
        save_png(sheet, f"aurora_{variant}.png", palette_lock=False)


# ============================================================
# Background tiles
# ============================================================
def _pixel_dither(d, x0, y0, x1, y1, base, accent, step=4):
    """Sparse pixel dither to add texture without exploding color count."""
    for y in range(y0, y1, step):
        for x in range(x0 + ((y // step) % 2) * (step // 2), x1, step):
            d.point((x, y), fill=accent)


def make_stage_backgrounds() -> None:
    """Generate 5 parallax background sets for the 5 stages of the journey."""
    print("[bg] generating stage backgrounds")

    # ============================================================
    # Stage 0: Department Store interior (Pangyo pop-up scene)
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Ceiling / ambient glow
    d.rectangle((0, 0, 256, 16), fill=(250, 236, 210, 255))
    # Ceiling recessed lights
    for lx in range(20, 256, 32):
        d.rectangle((lx, 6, lx + 10, 9), fill=(255, 230, 140, 255))
    # Shopfront wall
    d.rectangle((0, 16, 256, 72), fill=(234, 216, 186, 255))
    # Shop sign band
    d.rectangle((0, 16, 256, 26), fill=(60, 50, 45, 255))
    # Letter bars (big "STORE" letters as abstract color blocks)
    letters = [
        ((10, 30, 30, 230, 60, 80), "A"),
        ((42, 60, 80, 220, 120, 70), "B"),
        ((74, 230, 190, 60, 60, 60), "C"),
        ((106, 60, 160, 220, 220, 60), "D"),
        ((138, 220, 100, 150, 80, 80), "E"),
        ((170, 100, 200, 180, 60, 70), "F"),
        ((202, 220, 180, 60, 80, 60), "G"),
        ((234, 60, 200, 160, 70, 60), "H"),
    ]
    for (lx, r, g, b, h, w), _ in letters:
        d.rectangle((lx, 18, lx + 22, 24), fill=(r, g, b, 255))
    # Glass storefront windows with mannequin silhouettes
    for wx in (12, 84, 156, 228):
        # Window frame
        d.rectangle((wx, 30, wx + 56, 68), fill=(200, 190, 175, 255), outline=(80, 60, 40, 255), width=1)
        # Glass
        d.rectangle((wx + 3, 33, wx + 53, 65), fill=(225, 235, 240, 255))
        # Mannequin silhouette inside
        d.ellipse((wx + 24, 37, wx + 34, 47), fill=(170, 160, 150, 255))
        d.rectangle((wx + 26, 46, wx + 32, 62), fill=(170, 160, 150, 255))
        # Window price tag
        d.rectangle((wx + 8, 56, wx + 20, 62), fill=(255, 230, 80, 255))
    # Escalator balustrade running across the back
    d.rectangle((0, 72, 256, 82), fill=(180, 180, 195, 255))
    d.line((0, 72, 256, 72), fill=(100, 100, 115, 255))
    # Escalator rail with diagonal step hint
    for ex in range(0, 256, 12):
        d.line((ex, 82, ex + 6, 76), fill=(140, 140, 160, 255))
    # Floor edge
    d.rectangle((0, 90, 256, 100), fill=(210, 195, 170, 255))
    save_png(far, "bg_store_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 40, 256, 60), fill=(225, 210, 185, 255))
    # Brand display cases / shelves
    for i, dx in enumerate((8, 72, 136, 200)):
        # Cabinet
        d.rectangle((dx, 12, dx + 48, 42), fill=(240, 230, 210, 255), outline=(90, 70, 50, 255), width=1)
        # Shelf lines
        d.line((dx + 2, 22, dx + 46, 22), fill=(150, 120, 90, 255))
        d.line((dx + 2, 32, dx + 46, 32), fill=(150, 120, 90, 255))
        # Product silhouettes
        colors = [(230, 80, 70, 255), (70, 130, 220, 255), (240, 200, 60, 255), (80, 200, 140, 255)]
        for j, pc in enumerate(colors):
            px = dx + 4 + j * 11
            d.rectangle((px, 14, px + 8, 20), fill=pc)
            d.rectangle((px, 24, px + 8, 30), fill=pc)
        # Price dot
        d.rectangle((dx + 2, 36, dx + 10, 40), fill=(255, 220, 60, 255))
    # Potted plants between cabinets
    for pt in (56, 120, 184, 248):
        d.rectangle((pt, 35, pt + 8, 42), fill=(120, 80, 50, 255))
        d.ellipse((pt - 4, 22, pt + 12, 38), fill=(70, 140, 80, 255), outline=EDIE_OUTLINE)
    save_png(mid, "bg_store_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    # Polished marble — light cream with veins
    d.rectangle((0, 0, 256, 80), fill=(230, 220, 200, 255))
    d.rectangle((0, 0, 256, 3), fill=(170, 140, 100, 255))
    # Large tile grid
    for tx in range(0, 256, 64):
        d.line((tx, 3, tx, 80), fill=(195, 180, 155, 255))
    d.line((0, 40, 256, 40), fill=(195, 180, 155, 255))
    # Marble veining
    for i, (vx, vy, length) in enumerate(((12, 20, 40), (80, 55, 32), (150, 10, 28), (200, 50, 44), (30, 68, 22))):
        d.line((vx, vy, vx + length, vy + 3), fill=(205, 190, 165, 255))
    # Reflection highlight
    d.line((0, 6, 256, 6), fill=(255, 250, 235, 255))
    save_png(floor, "bg_store_floor.png", palette_lock=False)

    # ============================================================
    # Stage 1: Pangyo Street (day)
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Building silhouettes (tech office towers)
    building_palette = [(180, 180, 200, 255), (165, 170, 190, 255), (195, 195, 215, 255)]
    x = 0
    buildings = [(28, 75, 0), (22, 90, 1), (34, 60, 2), (20, 82, 0), (30, 95, 1), (26, 72, 2), (24, 85, 0), (32, 68, 1), (28, 90, 2)]
    for ww, hh, pi in buildings:
        d.rectangle((x, 100 - hh, x + ww, 100), fill=building_palette[pi])
        # Windows
        for wy in range(100 - hh + 6, 100 - 4, 5):
            for wx in range(x + 3, x + ww - 2, 5):
                d.point((wx, wy), fill=(90, 110, 140, 255))
        x += ww
    save_png(far, "bg_street_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 40, 256, 60), fill=(120, 130, 115, 255))  # grass strip
    # Street lamps
    for lx in (20, 100, 180, 240):
        d.rectangle((lx, 10, lx + 3, 45), fill=(60, 60, 70, 255))
        d.rectangle((lx - 2, 10, lx + 5, 14), fill=(200, 200, 120, 255))
    # Benches
    for bx in (50, 140, 220):
        d.rectangle((bx, 35, bx + 20, 40), fill=(120, 80, 40, 255), outline=EDIE_OUTLINE, width=1)
        d.line((bx + 2, 40, bx + 2, 48), fill=(100, 70, 35, 255))
        d.line((bx + 17, 40, bx + 17, 48), fill=(100, 70, 35, 255))
    save_png(mid, "bg_street_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=(145, 140, 130, 255))  # sidewalk grey
    d.rectangle((0, 0, 256, 3), fill=(90, 85, 75, 255))
    # Paving stone pattern
    for ty in range(6, 80, 12):
        d.line((0, ty, 256, ty), fill=(110, 105, 95, 255))
        for tx in range(0, 256, 24):
            offset = 12 if (ty // 12) % 2 == 0 else 0
            d.line((tx + offset, ty, tx + offset, ty + 12), fill=(110, 105, 95, 255))
    save_png(floor, "bg_street_floor.png", palette_lock=False)

    # ============================================================
    # Stage 2: Highway
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Distant mountains / overpass
    d.rectangle((0, 60, 256, 100), fill=(170, 185, 190, 255))
    for ox, oh in ((10, 20), (55, 35), (110, 28), (160, 42), (210, 25)):
        d.polygon([(ox, 60), (ox + 20, 60 - oh), (ox + 40, 60)], fill=(140, 155, 165, 255))
    # Highway sign posts
    for sx in (40, 130, 210):
        d.rectangle((sx, 55, sx + 3, 100), fill=(100, 100, 110, 255))
        d.rectangle((sx - 8, 45, sx + 24, 60), fill=(60, 130, 80, 255), outline=EDIE_OUTLINE, width=1)
        d.rectangle((sx - 5, 48, sx + 21, 57), fill=(240, 240, 240, 255))
    save_png(far, "bg_highway_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    # Guardrail
    d.rectangle((0, 48, 256, 56), fill=(180, 180, 190, 255))
    d.rectangle((0, 44, 256, 48), fill=(140, 140, 150, 255))
    for px in range(8, 256, 24):
        d.rectangle((px, 46, px + 4, 60), fill=(100, 100, 110, 255))
    # Passing car silhouettes
    for cx, cc in ((30, (200, 70, 60, 255)), (150, (70, 110, 180, 255))):
        d.rectangle((cx, 30, cx + 36, 45), fill=cc, outline=EDIE_OUTLINE, width=1)
        d.rectangle((cx + 4, 24, cx + 32, 32), fill=cc)
        d.ellipse((cx + 3, 41, cx + 11, 49), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=1)
        d.ellipse((cx + 25, 41, cx + 33, 49), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=1)
    save_png(mid, "bg_highway_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=(65, 65, 72, 255))  # asphalt
    d.rectangle((0, 0, 256, 4), fill=(240, 230, 80, 255))  # yellow edge line
    # Dashed lane markings
    for mx in range(10, 256, 32):
        d.rectangle((mx, 38, mx + 18, 44), fill=(240, 240, 240, 255))
    # Asphalt speckle
    for ty in range(10, 80, 6):
        for tx in range((ty // 6) * 3 % 8, 256, 8):
            d.point((tx, ty), fill=(90, 90, 98, 255))
    save_png(floor, "bg_highway_floor.png", palette_lock=False)

    # ============================================================
    # Stage 3: Ansan / Hanyang University ERICA campus
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # University building row (red brick + glass)
    d.rectangle((10, 30, 110, 100), fill=(170, 90, 70, 255))
    d.rectangle((10, 30, 110, 34), fill=(130, 65, 50, 255))
    # Windows grid
    for wy in range(40, 95, 10):
        for wx in range(18, 104, 12):
            d.rectangle((wx, wy, wx + 6, wy + 5), fill=(220, 220, 240, 255))
    # Central tower
    d.rectangle((120, 15, 170, 100), fill=(200, 200, 215, 255))
    for wy in range(25, 95, 8):
        d.rectangle((128, wy, 162, wy + 4), fill=(110, 140, 180, 255))
    # Right wing
    d.rectangle((180, 40, 256, 100), fill=(160, 85, 65, 255))
    for wy in range(48, 95, 10):
        for wx in range(186, 252, 12):
            d.rectangle((wx, wy, wx + 6, wy + 5), fill=(220, 220, 240, 255))
    save_png(far, "bg_ansan_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 40, 256, 60), fill=(80, 130, 70, 255))  # green lawn
    # Trees
    for tx in (30, 80, 140, 200):
        d.rectangle((tx + 4, 25, tx + 8, 45), fill=(90, 60, 40, 255))
        d.ellipse((tx - 4, 5, tx + 16, 30), fill=(60, 130, 70, 255), outline=EDIE_OUTLINE)
        d.ellipse((tx - 2, 8, tx + 14, 26), fill=(80, 150, 80, 255))
    # Stone path marker
    for bx in (110, 170):
        d.rectangle((bx, 35, bx + 6, 50), fill=(180, 180, 175, 255))
    save_png(mid, "bg_ansan_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=(175, 165, 140, 255))  # tan path
    d.rectangle((0, 0, 256, 3), fill=(130, 120, 95, 255))
    # Cobble dots
    for ty in range(8, 80, 6):
        for tx in range((ty // 6) * 4 % 12, 256, 12):
            d.rectangle((tx, ty, tx + 2, ty + 1), fill=(140, 130, 105, 255))
    save_png(floor, "bg_ansan_floor.png", palette_lock=False)

    # ============================================================
    # Stage 4: AeiROBOT HQ interior (sleek future tech)
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Glass wall with LED accent strips
    d.rectangle((0, 0, 256, 100), fill=(40, 50, 75, 255))
    d.rectangle((0, 0, 256, 100), fill=(40, 50, 75, 255))
    # Vertical light columns
    for cx in range(16, 256, 32):
        d.rectangle((cx, 10, cx + 4, 100), fill=(80, 120, 160, 255))
        d.rectangle((cx + 1, 12, cx + 3, 98), fill=(120, 200, 240, 255))
    # AeiROBOT logo panel
    d.rectangle((96, 20, 160, 50), fill=(20, 25, 40, 255), outline=(200, 220, 240, 255), width=2)
    d.rectangle((104, 28, 152, 42), fill=EDIE_ORANGE)
    d.rectangle((108, 32, 148, 38), fill=(255, 200, 120, 255))
    save_png(far, "bg_hq_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 40, 256, 60), fill=(60, 75, 100, 255))
    # Robot assembly pods
    for px in (30, 120, 210):
        d.rectangle((px - 12, 20, px + 12, 45), fill=(90, 110, 140, 255), outline=EDIE_OUTLINE, width=1)
        d.rectangle((px - 8, 24, px + 8, 40), fill=(150, 200, 240, 255))
        d.rectangle((px - 2, 12, px + 2, 20), fill=(120, 130, 150, 255))
        d.point((px, 30), fill=EDIE_ORANGE)
    save_png(mid, "bg_hq_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=(30, 40, 60, 255))  # dark high-tech floor
    d.rectangle((0, 0, 256, 4), fill=(100, 180, 220, 255))
    # Hex grid pattern
    for ty in range(6, 80, 10):
        off = 12 if (ty // 10) % 2 == 0 else 0
        for tx in range(off, 256, 24):
            d.rectangle((tx, ty, tx + 3, ty + 3), fill=(60, 90, 130, 255))
    # Center glow line
    d.line((0, 40, 256, 40), fill=(80, 140, 180, 255))
    save_png(floor, "bg_hq_floor.png", palette_lock=False)


def make_background() -> None:
    print("[bg] generating layers")

    # Sky — 1280×200 base warm cream with pixel cloud smears
    sky = Image.new("RGBA", (1280, 200), BG_SKY)
    d = ImageDraw.Draw(sky)
    cloud = (228, 221, 207, 255)
    # A handful of horizontal cloud streaks at pixel-friendly positions
    cloud_positions = [
        (40, 22, 100, 5),
        (180, 40, 120, 4),
        (340, 15, 80, 3),
        (520, 60, 140, 5),
        (700, 30, 90, 4),
        (860, 55, 110, 4),
        (1020, 20, 100, 5),
        (1160, 45, 90, 3),
    ]
    for cx, cy, cw, ch in cloud_positions:
        d.rectangle((cx, cy, cx + cw, cy + ch), fill=cloud)
        d.rectangle((cx + 4, cy - 1, cx + cw - 4, cy), fill=cloud)
    save_png(sky, "bg_sky.png", palette_lock=False)

    # Stars — 1280×200 transparent layer with scattered pixel stars
    # (rendered over sky during night portion of day/night cycle)
    stars = new_canvas(1280, 200)
    ds = ImageDraw.Draw(stars)
    import random
    rng = random.Random(42)
    for _ in range(90):
        sx = rng.randint(0, 1279)
        sy = rng.randint(0, 110)
        if rng.random() < 0.3:
            # Bright star — 3-pixel cross
            ds.point((sx, sy), fill=(255, 255, 255, 255))
            ds.point((sx + 1, sy), fill=(220, 220, 230, 255))
            ds.point((sx - 1, sy), fill=(220, 220, 230, 255))
            ds.point((sx, sy + 1), fill=(220, 220, 230, 255))
            ds.point((sx, sy - 1), fill=(220, 220, 230, 255))
        else:
            ds.point((sx, sy), fill=(255, 255, 255, 255))
    save_png(stars, "bg_stars.png", palette_lock=False)

    # Far — 256×100, server silhouettes with pixel window rows + dither
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    silhouette_x = 0
    heights = [60, 80, 50, 90, 70, 55, 85, 65, 75, 50, 95, 60]
    widths = [24, 20, 28, 18, 22, 30, 16, 24, 20, 26, 18, 28]
    far_shade = (174, 174, 164, 255)
    for hh, ww in zip(heights, widths):
        # Main silhouette
        d.rectangle((silhouette_x, 100 - hh, silhouette_x + ww, 100), fill=BG_FAR)
        # Shaded right edge
        d.rectangle(
            (silhouette_x + ww - 2, 100 - hh, silhouette_x + ww, 100),
            fill=far_shade,
        )
        # Top accent line
        d.line(
            (silhouette_x, 100 - hh, silhouette_x + ww, 100 - hh),
            fill=(210, 208, 200, 255),
        )
        # Pixel window rows
        for wy in range(100 - hh + 6, 100 - 6, 5):
            for wx in range(silhouette_x + 3, silhouette_x + ww - 3, 4):
                d.point((wx, wy), fill=(120, 130, 140, 255))
        silhouette_x += ww
    save_png(far, "bg_far.png", palette_lock=False)

    # Mid — 256×60 workbench with pixel highlights and bolts
    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    # Surface band
    d.rectangle((0, 28, 256, 60), fill=BG_MID)
    d.rectangle((0, 26, 256, 30), fill=(170, 160, 140, 255))  # brighter top
    d.rectangle((0, 28, 256, 29), fill=(205, 195, 175, 255))  # bright edge
    # Legs
    leg_color = (90, 85, 70, 255)
    leg_hl = (120, 114, 94, 255)
    for x in (8, 48, 88, 128, 168, 208, 248):
        d.rectangle((x, 32, x + 5, 60), fill=leg_color)
        d.line((x, 32, x, 60), fill=leg_hl)
    # Bolts on surface
    bolt = (70, 65, 52, 255)
    for bx in range(14, 256, 22):
        d.point((bx, 35), fill=bolt)
        d.point((bx + 1, 35), fill=bolt)
    save_png(mid, "bg_mid.png", palette_lock=False)

    # Floor — 256×80 warm brown with dither + rivets + accent stripe
    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=FLOOR)
    # Accent stripe near top
    d.rectangle((0, 0, 256, 3), fill=FLOOR_LINE)
    d.rectangle((0, 4, 256, 5), fill=(94, 85, 70, 255))
    # Dither dots for pixel feel
    dither_color = (60, 55, 46, 255)
    for y in range(8, 80, 3):
        for x in range((y // 3) % 2 * 2, 256, 4):
            d.point((x, y), fill=dither_color)
    # Panel seams every 32 pixels
    for px in range(0, 256, 32):
        d.line((px, 6, px, 78), fill=(62, 56, 45, 255))
        d.line((px + 1, 6, px + 1, 78), fill=(82, 74, 60, 255))
    # Rivets
    rivet = (106, 96, 76, 255)
    for px in range(0, 256, 32):
        for py in (16, 40, 64):
            d.rectangle((px + 14, py, px + 15, py + 1), fill=rivet)
    save_png(floor, "bg_floor.png", palette_lock=False)


def make_sfx() -> None:
    """Generate a handful of simple WAV sound effects via numpy synth."""
    print("[sfx] generating sound effects")
    import struct
    import wave

    SR = 22050

    def write_wav(name: str, samples: np.ndarray) -> None:
        samples = np.clip(samples, -1.0, 1.0)
        pcm = (samples * 32000).astype(np.int16)
        out = GEN / name
        with wave.open(str(out), "w") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes(pcm.tobytes())
        print(f"  OK {name} {len(pcm)/SR:.2f}s")

    def env(n: int, attack: float = 0.01, decay: float = 0.3) -> np.ndarray:
        t = np.arange(n) / SR
        a = np.clip(t / attack, 0, 1)
        d = np.exp(-t / decay)
        return a * d

    # Jump: short upward chirp
    dur = 0.18
    n = int(dur * SR)
    t = np.arange(n) / SR
    freq = 320 + 700 * t / dur
    jump = 0.35 * np.sin(2 * np.pi * np.cumsum(freq) / SR) * env(n, 0.005, 0.15)
    write_wav("sfx_jump.wav", jump)

    # Hit: low thud with noise
    dur = 0.25
    n = int(dur * SR)
    t = np.arange(n) / SR
    hit = (
        0.5 * np.sin(2 * np.pi * 80 * t) * env(n, 0.005, 0.12)
        + 0.35 * (np.random.RandomState(1).uniform(-1, 1, n)) * env(n, 0.002, 0.08)
    )
    write_wav("sfx_hit.wav", hit)

    # Pickup: bright two-tone ding
    dur = 0.22
    n = int(dur * SR)
    t = np.arange(n) / SR
    pickup = (
        0.32 * np.sin(2 * np.pi * 880 * t) * env(n, 0.002, 0.18)
        + 0.22 * np.sin(2 * np.pi * 1320 * t) * env(n, 0.005, 0.12)
    )
    write_wav("sfx_pickup.wav", pickup)

    # Dash: whoosh (band-limited noise sweeping up)
    dur = 0.32
    n = int(dur * SR)
    t = np.arange(n) / SR
    noise = np.random.RandomState(2).uniform(-1, 1, n)
    # Simple one-pole lowpass with time-varying alpha
    dash = np.zeros(n)
    y = 0.0
    for i in range(n):
        alpha = 0.02 + 0.15 * (i / n)
        y = y * (1 - alpha) + noise[i] * alpha
        dash[i] = y
    dash *= env(n, 0.002, 0.22) * 0.6
    write_wav("sfx_dash.wav", dash)

    # Smash: short crunch
    dur = 0.22
    n = int(dur * SR)
    t = np.arange(n) / SR
    smash = (
        0.45 * np.sin(2 * np.pi * 180 * t) * env(n, 0.002, 0.08)
        + 0.5 * np.random.RandomState(3).uniform(-1, 1, n) * env(n, 0.001, 0.06)
    )
    write_wav("sfx_smash.wav", smash)

    # Heart pickup: warm chime
    dur = 0.3
    n = int(dur * SR)
    t = np.arange(n) / SR
    heart = (
        0.3 * np.sin(2 * np.pi * 660 * t) * env(n, 0.01, 0.25)
        + 0.2 * np.sin(2 * np.pi * 990 * t) * env(n, 0.02, 0.2)
    )
    write_wav("sfx_heart.wav", heart)


def make_heart_pickup() -> None:
    """Pixel-art heart sprite sheet (4-frame pulse)."""
    frames = []
    red_dark = (180, 25, 40, 255)
    red = (232, 50, 60, 255)
    red_light = (255, 140, 140, 255)
    white = (255, 255, 255, 255)
    outline = EDIE_OUTLINE
    for f in range(4):
        w = h = 36
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        # Heart shape centered at (18, 18), pulse by +/-1 px
        pulse = (f % 2)
        cx, cy = 18, 18 - pulse
        # Two top lobes
        d.ellipse((cx - 10, cy - 8, cx - 1, cy + 2), fill=red, outline=outline, width=1)
        d.ellipse((cx + 1, cy - 8, cx + 10, cy + 2), fill=red, outline=outline, width=1)
        # Bottom triangle/point
        d.polygon(
            [(cx - 10, cy - 1), (cx + 10, cy - 1), (cx, cy + 12)],
            fill=red,
            outline=outline,
        )
        # Dark inner
        d.polygon(
            [(cx - 6, cy + 2), (cx + 6, cy + 2), (cx, cy + 8)],
            fill=red_dark,
        )
        # Highlight
        d.ellipse((cx - 7, cy - 6, cx - 4, cy - 3), fill=red_light)
        d.point((cx - 6, cy - 5), fill=white)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "heart.png", palette_lock=False)


# ============================================================
# main
# ============================================================
def extract_gif_to_sheet(gif_name: str, out_name: str, target_h: int | None = None) -> None:
    """Extract every frame of a GIF into a horizontal sprite sheet.

    Frames are cropped to the union bounding box of all non-transparent
    content, then tiled with 1 px transparent padding between frames.
    """
    p = SOURCE / gif_name
    im = Image.open(p)
    n = getattr(im, "n_frames", 1)
    frames: list[Image.Image] = []
    # Union bbox across all frames
    union = None
    for i in range(n):
        im.seek(i)
        fr = im.convert("RGBA")
        bbox = fr.getbbox()
        if bbox is None:
            continue
        if union is None:
            union = bbox
        else:
            union = (
                min(union[0], bbox[0]),
                min(union[1], bbox[1]),
                max(union[2], bbox[2]),
                max(union[3], bbox[3]),
            )
    if union is None:
        raise SystemExit(f"{gif_name}: empty frames")
    for i in range(n):
        im.seek(i)
        fr = im.convert("RGBA").crop(union)
        if target_h is not None and fr.height != target_h:
            new_w = max(1, round(fr.width * target_h / fr.height))
            fr = fr.resize((new_w, target_h), Image.NEAREST)
        frames.append(fr)
    sheet = tile_horizontal(frames)
    save_png(sheet, out_name, palette_lock=False)
    print(f"    ({n} frames, frame size {frames[0].size})")


def process_gif_assets() -> None:
    print("[EDIE] extracting gif animations")
    # Running cycle (7f): bright-eyed idle blink
    extract_gif_to_sheet("1000027545.gif", "edie_run_anim.png")
    # Title idle variant 1 (7f): looking around curiously
    extract_gif_to_sheet("1000027548.gif", "edie_title_idle.png")
    # Sad closed eyes (7f): GameOver alternate
    extract_gif_to_sheet("1000027549.gif", "edie_sad_alt.png")
    # Drowsy / sleepy (7f): Pause screen
    extract_gif_to_sheet("1000027550.gif", "edie_sleepy.png")
    # Hit / dazed (17f): X-eye dizzy
    extract_gif_to_sheet("1000027551.gif", "edie_hit_anim.png")
    # Title idle variant 2 (11f): looking around
    extract_gif_to_sheet("1000027552.gif", "edie_look.png")
    # Game over overlay (11f): sad teardrop
    extract_gif_to_sheet("1000027553.gif", "edie_gameover_anim.png")
    # Title idle variant 3 (7f): clean blink
    extract_gif_to_sheet("1000027554.gif", "edie_blink_alt.png")
    # Celebration / cheer (17f): happy laugh — Dash state
    extract_gif_to_sheet("1000027555.gif", "edie_cheer_anim.png")


def main() -> None:
    print(f"== EDIE Runner art generator ==")
    print(f"Source: {SOURCE}")
    print(f"Output: {GEN}")
    print()
    run_im, jump_im = process_edie_refs(target_h=48)
    derive_edie_states(run_im)
    print()
    process_gif_assets()
    print()
    print("[obstacles]")
    make_coffee_cup()
    make_shopping_cart()
    make_sensor_cone()
    make_sign_board()
    make_cat()
    make_vacuum_bot()
    make_amy()
    make_alice_m1()
    make_alice3()
    make_alice4()
    print()
    print("[pickups]")
    make_aurora()
    make_heart_pickup()
    print()
    make_background()
    make_stage_backgrounds()
    print()
    make_sfx()
    print()
    print("Done.")


if __name__ == "__main__":
    main()
