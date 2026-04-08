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
    _make_cat_variant("orange", (235, 150, 70, 255), (180, 100, 40, 255), (255, 220, 170, 255))
    _make_cat_variant("white", (252, 252, 252, 255), (200, 200, 210, 255), (240, 240, 245, 255))


def _make_cat_variant(variant: str, body, body_d, belly) -> None:
    """Chunky chibi kitten. Oversized head, tiny body, big round eyes."""
    frames = []
    pink = (255, 155, 180, 255)
    pink_d = (230, 110, 140, 255)
    out = EDIE_OUTLINE
    for f in range(2):
        w, h = 48, 40
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        bob = f  # 1-pixel up/down bob

        # Body coords
        body_cx = w // 2
        body_top = 22 + bob
        body_bot = 38
        # Main body blob (shorter than head)
        d.ellipse((body_cx - 12, body_top, body_cx + 12, body_bot), fill=body, outline=out, width=1)
        # Belly patch
        d.ellipse((body_cx - 9, body_top + 3, body_cx + 9, body_bot - 1), fill=belly)
        # Tiny front legs tucked in
        d.rectangle((body_cx - 8, body_bot - 4, body_cx - 4, body_bot - 1), fill=body_d, outline=out, width=1)
        d.rectangle((body_cx + 4, body_bot - 4, body_cx + 8, body_bot - 1), fill=body_d, outline=out, width=1)
        # Paws
        d.point((body_cx - 6, body_bot - 1), fill=out)
        d.point((body_cx + 6, body_bot - 1), fill=out)
        # Tail: long curved, wrapping around right side
        tail_swing = f
        tail_pts = [
            (body_cx + 11, body_top + 6),
            (body_cx + 16, body_top + 2 + tail_swing),
            (body_cx + 18, body_top - 4 + tail_swing),
            (body_cx + 16, body_top - 10),
            (body_cx + 11, body_top - 13),
        ]
        for i in range(len(tail_pts) - 1):
            d.line([tail_pts[i], tail_pts[i + 1]], fill=body, width=4)
            d.line([tail_pts[i], tail_pts[i + 1]], fill=out, width=1)

        # HUGE head (takes most of the canvas)
        head_cx = body_cx
        head_cy = 14 + bob
        head_r = 13
        d.ellipse(
            (head_cx - head_r, head_cy - head_r, head_cx + head_r, head_cy + head_r),
            fill=body,
            outline=out,
            width=1,
        )
        # Head highlight
        d.ellipse(
            (head_cx - 9, head_cy - 11, head_cx + 2, head_cy - 2),
            fill=belly,
        )
        # Pointy ears (bigger, well-defined)
        d.polygon(
            [(head_cx - 13, head_cy - 3), (head_cx - 11, head_cy - 15), (head_cx - 5, head_cy - 9)],
            fill=body,
            outline=out,
        )
        d.polygon(
            [(head_cx + 13, head_cy - 3), (head_cx + 11, head_cy - 15), (head_cx + 5, head_cy - 9)],
            fill=body,
            outline=out,
        )
        # Inner pink ears
        d.polygon(
            [(head_cx - 11, head_cy - 5), (head_cx - 10, head_cy - 12), (head_cx - 7, head_cy - 8)],
            fill=pink,
        )
        d.polygon(
            [(head_cx + 11, head_cy - 5), (head_cx + 10, head_cy - 12), (head_cx + 7, head_cy - 8)],
            fill=pink,
        )

        # --- Big round eyes (oval, 6x8) ---
        eye_y = head_cy + 1
        # Left eye white
        d.ellipse((head_cx - 8, eye_y - 4, head_cx - 2, eye_y + 4), fill=(255, 255, 255, 255), outline=out, width=1)
        # Iris (teal-green)
        d.ellipse((head_cx - 7, eye_y - 3, head_cx - 3, eye_y + 3), fill=(80, 180, 160, 255))
        # Pupil (black oval)
        d.ellipse((head_cx - 6, eye_y - 2, head_cx - 4, eye_y + 2), fill=out)
        # Sparkles
        d.point((head_cx - 5, eye_y - 2), fill=(255, 255, 255, 255))
        d.point((head_cx - 4, eye_y + 2), fill=(200, 240, 255, 255))
        # Right eye
        d.ellipse((head_cx + 2, eye_y - 4, head_cx + 8, eye_y + 4), fill=(255, 255, 255, 255), outline=out, width=1)
        d.ellipse((head_cx + 3, eye_y - 3, head_cx + 7, eye_y + 3), fill=(80, 180, 160, 255))
        d.ellipse((head_cx + 4, eye_y - 2, head_cx + 6, eye_y + 2), fill=out)
        d.point((head_cx + 5, eye_y - 2), fill=(255, 255, 255, 255))
        d.point((head_cx + 4, eye_y + 2), fill=(200, 240, 255, 255))

        # Tiny triangular pink nose
        nose_y = eye_y + 5
        d.polygon(
            [(head_cx - 1, nose_y), (head_cx + 1, nose_y), (head_cx, nose_y + 2)],
            fill=pink_d,
        )
        # Tiny 'w' mouth
        d.point((head_cx - 2, nose_y + 3), fill=out)
        d.point((head_cx - 1, nose_y + 4), fill=out)
        d.point((head_cx, nose_y + 3), fill=out)
        d.point((head_cx + 1, nose_y + 4), fill=out)
        d.point((head_cx + 2, nose_y + 3), fill=out)

        # Rosy cheeks (soft pink dots)
        d.ellipse((head_cx - 12, eye_y + 2, head_cx - 9, eye_y + 5), fill=(255, 180, 200, 200))
        d.ellipse((head_cx + 9, eye_y + 2, head_cx + 12, eye_y + 5), fill=(255, 180, 200, 200))

        # Whiskers
        d.line((head_cx - 14, eye_y + 4, head_cx - 9, eye_y + 3), fill=out)
        d.line((head_cx - 14, eye_y + 6, head_cx - 9, eye_y + 5), fill=out)
        d.line((head_cx + 9, eye_y + 3, head_cx + 14, eye_y + 4), fill=out)
        d.line((head_cx + 9, eye_y + 5, head_cx + 14, eye_y + 6), fill=out)

        # Variant-specific markings
        if variant == "orange":
            # Tabby stripes on forehead
            d.line((head_cx - 5, head_cy - 12, head_cx - 5, head_cy - 9), fill=body_d)
            d.line((head_cx, head_cy - 13, head_cx, head_cy - 9), fill=body_d)
            d.line((head_cx + 5, head_cy - 12, head_cx + 5, head_cy - 9), fill=body_d)
            # Back stripes
            d.line((body_cx - 8, body_top + 3, body_cx - 2, body_top + 2), fill=body_d)
            d.line((body_cx + 2, body_top + 3, body_cx + 8, body_top + 2), fill=body_d)
        else:
            # Subtle grey shading around head edge
            d.line((head_cx - 12, head_cy, head_cx - 11, head_cy + 5), fill=body_d)
            d.line((head_cx + 11, head_cy, head_cx + 12, head_cy + 5), fill=body_d)

        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, f"obstacle_cat_{variant}.png", palette_lock=False)


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


def process_robot_refs() -> None:
    """Downsample the user-provided AeiROBOT reference PNGs into game-ready
    sprites, preserving their silhouette and palette."""
    print("[robots] downsampling AeiROBOT reference PNGs")

    def downsample(src_name: str, target_h: int, out_name: str) -> None:
        p = SOURCE / src_name
        im = Image.open(p).convert("RGBA")
        a = np.array(im)
        alpha = a[:, :, 3]
        rows = np.any(alpha > 40, axis=1)
        cols = np.any(alpha > 40, axis=0)
        if not rows.any():
            return
        y0, y1 = np.where(rows)[0][[0, -1]]
        x0, x1 = np.where(cols)[0][[0, -1]]
        cropped = im.crop((x0, y0, x1 + 1, y1 + 1))
        cw, ch = cropped.size
        new_w = max(1, round(cw * target_h / ch))
        small = cropped.resize((new_w, target_h), Image.LANCZOS)
        # Quantize to a small palette to tighten edges
        pal_im = small.convert("RGB").quantize(colors=10, method=Image.FASTOCTREE)
        quant_rgb = pal_im.convert("RGB")
        out = Image.new("RGBA", small.size)
        orig = np.array(small)
        q = np.array(quant_rgb)
        for y in range(small.size[1]):
            for x in range(small.size[0]):
                if orig[y, x, 3] < 80:
                    out.putpixel((x, y), (0, 0, 0, 0))
                else:
                    r, g, b = q[y, x]
                    out.putpixel((x, y), (int(r), int(g), int(b), 255))
        save_png(out, out_name, palette_lock=False)

    # Target heights chosen to fit game world (Alice3 ~64, Alice4 ~68,
    # AliceM1 ~64, Amy ~60) -- matches ObstacleKind::size().
    downsample("alice3.png", 64, "obstacle_alice3.png")
    downsample("alice4.png", 68, "obstacle_alice4.png")
    downsample("alice_m1.png", 64, "obstacle_alicem1.png")
    downsample("aimy.png", 60, "obstacle_amy.png")


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


def make_car() -> None:
    """Charging car - wide ground obstacle, bright red."""
    w, h = 96, 40
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    body = (220, 60, 60, 255)
    body_d = (160, 30, 40, 255)
    glass = (140, 200, 230, 255)
    # Lower body
    d.rectangle((4, 18, w - 5, h - 8), fill=body, outline=EDIE_OUTLINE, width=2)
    # Upper cabin
    d.rectangle((22, 6, w - 22, 20), fill=body_d, outline=EDIE_OUTLINE, width=2)
    # Windshield
    d.rectangle((26, 9, w - 26, 18), fill=glass)
    # Front lights
    d.rectangle((w - 8, 22, w - 4, 28), fill=(255, 230, 120, 255), outline=EDIE_OUTLINE, width=1)
    d.rectangle((4, 22, 8, 28), fill=(200, 40, 40, 255), outline=EDIE_OUTLINE, width=1)
    # Grille
    for gx in range(w - 14, w - 5, 2):
        d.line((gx, 24, gx, 30), fill=(40, 40, 46, 255))
    # Wheels
    d.ellipse((12, h - 14, 26, h - 1), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=2)
    d.ellipse((14, h - 12, 24, h - 3), fill=(120, 120, 130, 255))
    d.ellipse((w - 27, h - 14, w - 13, h - 1), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=2)
    d.ellipse((w - 25, h - 12, w - 15, h - 3), fill=(120, 120, 130, 255))
    # Motion streaks (implies charging)
    for sy in (14, 24, 34):
        d.line((0, sy, 3, sy), fill=(255, 255, 255, 180))
    save_png(im, "obstacle_car.png", palette_lock=False)


def make_deer() -> None:
    """Leaping deer - ground mid-height obstacle, tan with antlers."""
    w, h = 48, 52
    frames = []
    for f in range(2):
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        tan = (200, 130, 70, 255)
        tan_d = (150, 90, 40, 255)
        out = EDIE_OUTLINE
        # Leap height varies slightly per frame for a "jumping in" feel
        dy = f * -1
        # Body
        d.ellipse((6, 18 + dy, 38, 36 + dy), fill=tan, outline=out, width=1)
        # Chest/belly
        d.ellipse((10, 22 + dy, 34, 34 + dy), fill=tan_d)
        # Head
        d.ellipse((28, 10 + dy, 44, 22 + dy), fill=tan, outline=out, width=1)
        # Snout
        d.rectangle((40, 15 + dy, 46, 19 + dy), fill=tan, outline=out, width=1)
        d.point((44, 16 + dy), fill=out)
        # Eye
        d.point((36, 14 + dy), fill=out)
        # Ears
        d.polygon([(30, 10 + dy), (28, 4 + dy), (34, 10 + dy)], fill=tan, outline=out)
        d.polygon([(36, 9 + dy), (34, 3 + dy), (40, 9 + dy)], fill=tan, outline=out)
        # Antlers
        d.line((32, 5 + dy, 30, 0 + dy), fill=out, width=1)
        d.line((30, 3 + dy, 27, 1 + dy), fill=out, width=1)
        d.line((38, 4 + dy, 40, -1 + dy), fill=out, width=1)
        d.line((40, 2 + dy, 43, 0 + dy), fill=out, width=1)
        # Legs (in leap pose - folded/stretched)
        d.line((12, 34 + dy, 10, 48), fill=out, width=2)
        d.line((18, 34 + dy, 20, 48), fill=out, width=2)
        d.line((28, 34 + dy, 26, 48), fill=out, width=2)
        d.line((34, 34 + dy, 36, 48), fill=out, width=2)
        # Hooves
        d.rectangle((9, 48, 12, 50), fill=out)
        d.rectangle((19, 48, 22, 50), fill=out)
        d.rectangle((25, 48, 28, 50), fill=out)
        d.rectangle((35, 48, 38, 50), fill=out)
        # Tail
        d.ellipse((4, 20 + dy, 8, 24 + dy), fill=(250, 245, 230, 255))
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_deer.png", palette_lock=False)


def make_balloon_drone() -> None:
    """Balloon drone - floating pastel balloon with small drone body below."""
    w, h = 40, 48
    frames = []
    for f in range(4):
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        # Balloon (pink/orange)
        balloon = (240, 130, 160, 255)
        balloon_hi = (255, 200, 215, 255)
        d.ellipse((6, 2, w - 7, 28), fill=balloon, outline=EDIE_OUTLINE, width=1)
        # Shine
        d.ellipse((12, 6, 16, 10), fill=balloon_hi)
        # Tie
        d.polygon([(18, 27), (22, 27), (20, 30)], fill=(200, 90, 120, 255), outline=EDIE_OUTLINE)
        # String with wiggle (animated)
        wiggle = (f % 2) - 0  # 0..1
        d.line((20, 30, 20 + wiggle, 36), fill=EDIE_OUTLINE)
        d.line((20 + wiggle, 36, 20, 40), fill=EDIE_OUTLINE)
        # Small drone body
        d.rectangle((12, 38, 28, 46), fill=(80, 90, 110, 255), outline=EDIE_OUTLINE, width=1)
        # Eye
        d.rectangle((16, 40, 18, 42), fill=EDIE_ORANGE)
        d.rectangle((22, 40, 24, 42), fill=EDIE_ORANGE)
        # Rotor blur hint
        if f % 2 == 0:
            d.line((8, 40, 12, 40), fill=(140, 140, 150, 255))
            d.line((28, 40, 32, 40), fill=(140, 140, 150, 255))
        else:
            d.line((6, 42, 12, 42), fill=(140, 140, 150, 255))
            d.line((28, 42, 34, 42), fill=(140, 140, 150, 255))
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_balloon.png", palette_lock=False)


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
    # Stage 0: Pangyo Department Store - luxury retail floor
    # Big glass shop windows with mannequins + gold watches
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Marble back wall
    d.rectangle((0, 0, 256, 100), fill=(232, 222, 205, 255))
    # Ceiling darker strip
    d.rectangle((0, 0, 256, 8), fill=(200, 185, 155, 255))
    # Warm ceiling spotlights
    for lx in range(28, 256, 48):
        d.ellipse((lx - 4, 2, lx + 4, 8), fill=(255, 235, 160, 255))
        d.line((lx, 8, lx - 3, 14), fill=(255, 230, 140, 100))
        d.line((lx, 8, lx + 3, 14), fill=(255, 230, 140, 100))
    # TWO large shop windows, each ~120 wide
    for wx_base in (8, 136):
        wx = wx_base
        ww = 112
        wy = 14
        wh = 74
        # Thick wood frame
        d.rectangle((wx, wy, wx + ww, wy + wh), fill=(90, 60, 35, 255), outline=EDIE_OUTLINE, width=2)
        d.rectangle((wx + 2, wy + 2, wx + ww - 2, wy + wh - 2), fill=(120, 80, 45, 255))
        # Glass interior
        d.rectangle((wx + 5, wy + 5, wx + ww - 5, wy + wh - 5), fill=(220, 230, 238, 255))
        # Glass reflection streaks
        d.line((wx + 10, wy + 8, wx + 22, wy + 38), fill=(245, 250, 255, 255))
        d.line((wx + ww - 18, wy + 12, wx + ww - 10, wy + 26), fill=(245, 250, 255, 255))
        # Mannequin on the left of the window
        mx = wx + 20
        my = wy + 14
        # Head
        d.ellipse((mx, my, mx + 12, my + 14), fill=(220, 205, 180, 255), outline=EDIE_OUTLINE, width=1)
        # Neck
        d.rectangle((mx + 4, my + 13, mx + 8, my + 18), fill=(200, 185, 160, 255))
        # Formal suit torso (charcoal)
        d.polygon(
            [(mx - 4, my + 18), (mx + 16, my + 18), (mx + 18, my + 44), (mx - 6, my + 44)],
            fill=(50, 55, 65, 255),
            outline=EDIE_OUTLINE,
        )
        # White shirt triangle
        d.polygon([(mx + 4, my + 18), (mx + 8, my + 18), (mx + 6, my + 28)], fill=(240, 240, 245, 255))
        # Red tie
        d.polygon([(mx + 5, my + 22), (mx + 7, my + 22), (mx + 6, my + 34)], fill=(180, 40, 50, 255))
        # Base stand
        d.rectangle((mx - 2, my + 44, mx + 14, my + 46), fill=(110, 110, 120, 255))
        # Gold watch display on the right side
        gwx = wx + ww - 42
        gwy = wy + 18
        # Pedestal
        d.rectangle((gwx, gwy + 22, gwx + 32, gwy + 32), fill=(140, 120, 90, 255), outline=EDIE_OUTLINE, width=1)
        d.rectangle((gwx + 2, gwy + 24, gwx + 30, gwy + 30), fill=(200, 180, 140, 255))
        # Watch 1 (gold)
        d.ellipse((gwx + 4, gwy + 6, gwx + 16, gwy + 18), fill=(245, 200, 80, 255), outline=EDIE_OUTLINE, width=1)
        d.ellipse((gwx + 6, gwy + 8, gwx + 14, gwy + 16), fill=(255, 235, 130, 255))
        d.point((gwx + 10, gwy + 12), fill=(40, 40, 40, 255))
        d.line((gwx + 10, gwy + 10, gwx + 10, gwy + 12), fill=(40, 40, 40, 255))
        d.line((gwx + 10, gwy + 12, gwx + 13, gwy + 12), fill=(40, 40, 40, 255))
        d.rectangle((gwx + 6, gwy + 16, gwx + 14, gwy + 22), fill=(200, 160, 50, 255))
        # Watch 2 (rose gold)
        d.ellipse((gwx + 18, gwy + 8, gwx + 30, gwy + 20), fill=(220, 150, 120, 255), outline=EDIE_OUTLINE, width=1)
        d.ellipse((gwx + 20, gwy + 10, gwx + 28, gwy + 18), fill=(250, 200, 180, 255))
        d.point((gwx + 24, gwy + 14), fill=(40, 40, 40, 255))
        d.rectangle((gwx + 20, gwy + 18, gwx + 28, gwy + 23), fill=(180, 120, 90, 255))
        # Price tag
        d.rectangle((gwx + 10, gwy + 34, gwx + 22, gwy + 40), fill=(255, 250, 230, 255), outline=EDIE_OUTLINE, width=1)
        # Window top luxury brand name bar
        d.rectangle((wx + 10, wy - 2, wx + ww - 10, wy + 10), fill=(30, 30, 35, 255), outline=(200, 170, 80, 255), width=1)
        # Letter dots as gold lettering
        for gx in range(wx + 14, wx + ww - 14, 8):
            d.rectangle((gx, wy + 2, gx + 4, wy + 7), fill=(230, 190, 80, 255))
    # Between windows: marble pillar
    d.rectangle((122, 0, 134, 100), fill=(210, 200, 180, 255))
    d.rectangle((120, 0, 122, 100), fill=(180, 170, 150, 255))
    d.rectangle((134, 0, 136, 100), fill=(180, 170, 150, 255))
    # Pillar base
    d.rectangle((118, 88, 138, 100), fill=(170, 155, 130, 255))
    save_png(far, "bg_store_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    # Marble floor rim
    d.rectangle((0, 44, 256, 60), fill=(225, 215, 195, 255))
    d.line((0, 44, 256, 44), fill=(180, 165, 140, 255))
    # Low velvet rope stanchions + ropes
    for sx in (20, 88, 156, 224):
        # Post
        d.rectangle((sx, 26, sx + 4, 48), fill=(190, 155, 60, 255), outline=EDIE_OUTLINE, width=1)
        d.ellipse((sx - 2, 22, sx + 6, 28), fill=(230, 190, 80, 255), outline=EDIE_OUTLINE, width=1)
    # Velvet ropes between posts (red)
    for a, b in ((22, 90), (90, 158), (158, 226)):
        d.line((a + 4, 32, b - 2, 32), fill=(170, 30, 40, 255))
        d.line((a + 4, 33, b - 2, 33), fill=(140, 20, 30, 255))
        # Sag curve
        d.point(((a + b) // 2, 35), fill=(170, 30, 40, 255))
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
    # Stage 1b: Pangyo Tech Park (generic fictional IT campus)
    # Logos are abstract color shapes, not real brand marks.
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Glass tower block 1 (tall, blue-grey)
    d.rectangle((8, 8, 96, 100), fill=(155, 175, 195, 255), outline=EDIE_OUTLINE, width=1)
    # Window grid
    for wy in range(14, 98, 7):
        for wx in range(12, 94, 8):
            d.rectangle((wx, wy, wx + 6, wy + 4), fill=(95, 125, 160, 255))
    # Abstract rooftop mark: simple triangle, no letters
    d.polygon([(46, 22), (58, 22), (52, 12)], fill=(240, 210, 90, 255), outline=EDIE_OUTLINE)
    # Glass tower block 2 (mid-height, muted teal)
    d.rectangle((104, 24, 176, 100), fill=(165, 195, 180, 255), outline=EDIE_OUTLINE, width=1)
    for wy in range(30, 98, 6):
        for wx in range(108, 174, 7):
            d.rectangle((wx, wy, wx + 5, wy + 3), fill=(80, 140, 110, 255))
    # Rooftop accent: two stacked bars, no text
    d.rectangle((120, 28, 160, 32), fill=(90, 170, 130, 255))
    d.rectangle((126, 34, 154, 38), fill=(120, 190, 150, 255))
    # Glass tower block 3 (short, warm peach)
    d.rectangle((184, 40, 254, 100), fill=(220, 180, 150, 255), outline=EDIE_OUTLINE, width=1)
    for wy in range(46, 98, 6):
        for wx in range(188, 252, 7):
            d.rectangle((wx, wy, wx + 5, wy + 3), fill=(160, 110, 80, 255))
    # Rooftop dome
    d.ellipse((208, 32, 230, 44), fill=(200, 160, 130, 255), outline=EDIE_OUTLINE, width=1)
    # Sky between buildings
    d.rectangle((96, 8, 104, 24), fill=(200, 225, 240, 255))
    d.rectangle((176, 24, 184, 40), fill=(200, 225, 240, 255))
    save_png(far, "bg_techpark_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    # Wide plaza with fountain and benches
    d.rectangle((0, 40, 256, 60), fill=(210, 210, 200, 255))
    d.rectangle((0, 38, 256, 41), fill=(170, 170, 160, 255))
    # Fountain (center)
    d.ellipse((108, 26, 148, 50), fill=(180, 190, 210, 255), outline=EDIE_OUTLINE, width=1)
    d.ellipse((112, 30, 144, 46), fill=(120, 170, 200, 255))
    # Water spout
    d.line((128, 10, 128, 30), fill=(200, 220, 240, 255), width=2)
    d.line((124, 14, 126, 28), fill=(200, 220, 240, 255))
    d.line((130, 14, 132, 28), fill=(200, 220, 240, 255))
    # Food truck (left)
    d.rectangle((12, 18, 58, 44), fill=(220, 80, 60, 255), outline=EDIE_OUTLINE, width=1)
    d.rectangle((14, 22, 56, 32), fill=(255, 240, 220, 255))
    # Awning stripes
    for sx in range(12, 58, 4):
        d.line((sx, 18, sx, 20), fill=(240, 180, 60, 255))
    d.rectangle((18, 34, 52, 42), fill=(40, 40, 50, 255))
    d.ellipse((18, 40, 28, 48), fill=(40, 40, 50, 255), outline=EDIE_OUTLINE, width=1)
    d.ellipse((44, 40, 54, 48), fill=(40, 40, 50, 255), outline=EDIE_OUTLINE, width=1)
    # Trees (right side)
    for tx in (180, 210, 240):
        d.rectangle((tx + 2, 30, tx + 4, 44), fill=(90, 60, 40, 255))
        d.ellipse((tx - 4, 16, tx + 10, 32), fill=(70, 150, 80, 255), outline=EDIE_OUTLINE, width=1)
        d.ellipse((tx - 2, 18, tx + 8, 28), fill=(90, 170, 100, 255))
    save_png(mid, "bg_techpark_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    # Modern granite plaza
    d.rectangle((0, 0, 256, 80), fill=(175, 175, 180, 255))
    d.rectangle((0, 0, 256, 4), fill=(110, 110, 120, 255))
    # Large diagonal stone tile pattern
    for tx in range(0, 256, 32):
        d.line((tx, 4, tx, 80), fill=(145, 145, 155, 255))
    for ty in range(16, 80, 16):
        d.line((0, ty, 256, ty), fill=(145, 145, 155, 255))
    # Speckle
    import random as _rr
    rng = _rr.Random(42)
    for _ in range(120):
        sx = rng.randint(0, 255)
        sy = rng.randint(6, 79)
        d.point((sx, sy), fill=(130, 130, 140, 255))
    save_png(floor, "bg_techpark_floor.png", palette_lock=False)

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
    # Stage 4a: AeiROBOT Office (open-plan workspace)
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Bright office wall
    d.rectangle((0, 0, 256, 100), fill=(235, 235, 242, 255))
    d.rectangle((0, 0, 256, 12), fill=(200, 205, 215, 255))  # ceiling band
    # Ceiling lights
    for lx in range(22, 256, 36):
        d.rectangle((lx, 4, lx + 16, 8), fill=(255, 250, 210, 255))
    # Window strip at top with blinds
    d.rectangle((0, 14, 256, 40), fill=(180, 210, 235, 255))
    for bx in range(0, 256, 8):
        d.line((bx, 14, bx, 40), fill=(140, 170, 200, 255))
    # AeiROBOT wall logo centered - corrupted with virus intrusion
    d.rectangle((96, 48, 160, 74), fill=(30, 40, 60, 255), outline=EDIE_OUTLINE, width=2)
    d.rectangle((102, 54, 154, 68), fill=EDIE_ORANGE)
    # "AEI" letters with glitch offsets
    d.rectangle((106, 58, 110, 64), fill=EDIE_WHITE)
    d.rectangle((114, 59, 118, 65), fill=EDIE_WHITE)  # 1px glitch
    d.rectangle((122, 58, 126, 64), fill=EDIE_WHITE)
    # Red ERROR bar across logo
    d.rectangle((96, 60, 160, 63), fill=(220, 50, 50, 255))
    # Cubicle dividers
    d.rectangle((10, 72, 50, 96), fill=(200, 180, 150, 255), outline=EDIE_OUTLINE, width=1)
    d.rectangle((180, 72, 240, 96), fill=(200, 180, 150, 255), outline=EDIE_OUTLINE, width=1)
    # Error warning triangles floating in the top
    def _warn_tri(cx, cy):
        d.polygon(
            [(cx, cy - 6), (cx - 6, cy + 4), (cx + 6, cy + 4)],
            fill=(230, 200, 40, 255),
            outline=EDIE_OUTLINE,
        )
        d.rectangle((cx - 1, cy - 3, cx + 1, cy + 1), fill=EDIE_OUTLINE)
        d.point((cx, cy + 2), fill=EDIE_OUTLINE)
    _warn_tri(30, 24)
    _warn_tri(72, 30)
    _warn_tri(204, 26)
    _warn_tri(244, 32)
    # Red horizontal glitch strips (virus signal)
    for gy in (18, 42, 64, 88):
        for gx in range(0, 256, 16):
            d.rectangle((gx, gy, gx + 8, gy + 1), fill=(230, 60, 60, 150))
    # "ERROR" red tag box
    d.rectangle((8, 52, 40, 62), fill=(180, 30, 30, 255), outline=EDIE_WHITE, width=1)
    d.point((13, 56), fill=EDIE_WHITE)
    d.point((20, 56), fill=EDIE_WHITE)
    d.point((27, 56), fill=EDIE_WHITE)
    d.point((34, 56), fill=EDIE_WHITE)
    save_png(far, "bg_office_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 44, 256, 60), fill=(210, 205, 195, 255))
    # Desks with corrupted monitors
    for i, dx in enumerate((16, 88, 160, 228)):
        # Desk top
        d.rectangle((dx - 22, 30, dx + 22, 40), fill=(160, 130, 90, 255), outline=EDIE_OUTLINE, width=1)
        # Desk legs
        d.rectangle((dx - 20, 40, dx - 17, 50), fill=(110, 85, 55, 255))
        d.rectangle((dx + 17, 40, dx + 20, 50), fill=(110, 85, 55, 255))
        # Monitor stand
        d.rectangle((dx - 2, 22, dx + 2, 30), fill=(80, 80, 90, 255))
        # Monitor (some have red error screens)
        d.rectangle((dx - 12, 10, dx + 12, 24), fill=(40, 50, 70, 255), outline=EDIE_OUTLINE, width=1)
        if i % 2 == 0:
            # Corrupted red BSOD screen
            d.rectangle((dx - 10, 12, dx + 10, 22), fill=(180, 30, 30, 255))
            # "!!!" pixel warning
            d.point((dx - 4, 16), fill=EDIE_WHITE)
            d.point((dx - 4, 17), fill=EDIE_WHITE)
            d.point((dx, 16), fill=EDIE_WHITE)
            d.point((dx, 17), fill=EDIE_WHITE)
            d.point((dx + 4, 16), fill=EDIE_WHITE)
            d.point((dx + 4, 17), fill=EDIE_WHITE)
            # Static lines
            d.line((dx - 10, 19, dx + 10, 19), fill=(100, 20, 20, 255))
        else:
            # Glitched blue screen
            d.rectangle((dx - 10, 12, dx + 10, 22), fill=(120, 200, 240, 255))
            d.line((dx - 10, 15, dx + 10, 15), fill=(255, 80, 80, 255))
            d.line((dx - 10, 18, dx + 10, 18), fill=(255, 80, 80, 255))
        # Chair back
        d.rectangle((dx - 8, 42, dx + 8, 55), fill=(60, 80, 120, 255), outline=EDIE_OUTLINE, width=1)
    save_png(mid, "bg_office_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=(160, 150, 135, 255))  # office carpet
    d.rectangle((0, 0, 256, 4), fill=(100, 90, 75, 255))
    # Carpet fiber dither
    for ty in range(8, 80, 4):
        for tx in range((ty // 4) * 3 % 6, 256, 6):
            d.point((tx, ty), fill=(140, 130, 115, 255))
    save_png(floor, "bg_office_floor.png", palette_lock=False)

    # ============================================================
    # Stage 4b: AeiROBOT CEO Room (dark luxe, intimidating)
    # ============================================================
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    # Dark wood panel wall
    d.rectangle((0, 0, 256, 100), fill=(40, 30, 25, 255))
    for px in range(0, 256, 32):
        d.rectangle((px, 0, px + 1, 100), fill=(80, 55, 40, 255))
    # Ceiling crown molding
    d.rectangle((0, 0, 256, 8), fill=(80, 55, 40, 255))
    d.rectangle((0, 8, 256, 10), fill=(120, 85, 60, 255))
    # Giant floor-to-ceiling window with city night view
    d.rectangle((80, 14, 176, 90), fill=(20, 25, 50, 255), outline=(150, 110, 70, 255), width=2)
    # Distant city lights
    import random as _r
    rng = _r.Random(77)
    for _ in range(60):
        cx = rng.randint(82, 174)
        cy = rng.randint(20, 60)
        col = rng.choice([(255, 230, 120, 255), (200, 220, 255, 255), (255, 180, 100, 255)])
        d.point((cx, cy), fill=col)
    # Cracked glass pattern (virus breach)
    d.line((80, 40, 128, 30), fill=(220, 220, 240, 200))
    d.line((128, 30, 176, 60), fill=(220, 220, 240, 200))
    d.line((128, 30, 100, 70), fill=(220, 220, 240, 200))
    d.line((128, 30, 160, 75), fill=(220, 220, 240, 200))
    # Window divider
    d.line((128, 14, 128, 90), fill=(150, 110, 70, 255))
    # Red ALERT band across top
    d.rectangle((0, 0, 256, 6), fill=(180, 30, 30, 255))
    d.rectangle((0, 2, 256, 3), fill=(255, 80, 80, 255))
    # Warning triangles
    for cx, cy in ((50, 28), (218, 28), (50, 82), (218, 82)):
        d.polygon(
            [(cx, cy - 5), (cx - 5, cy + 4), (cx + 5, cy + 4)],
            fill=(230, 200, 40, 255),
            outline=EDIE_OUTLINE,
        )
        d.rectangle((cx - 1, cy - 2, cx + 1, cy + 1), fill=EDIE_OUTLINE)
    # Red glitch scan lines
    for gy in (20, 45, 68):
        for gx in range(0, 80, 10):
            d.rectangle((gx, gy, gx + 5, gy + 1), fill=(220, 50, 50, 180))
        for gx in range(180, 256, 10):
            d.rectangle((gx, gy, gx + 5, gy + 1), fill=(220, 50, 50, 180))
    # CEO portrait frames (left/right of window)
    for fx in (20, 212):
        d.rectangle((fx, 30, fx + 40, 70), fill=(120, 85, 60, 255), outline=EDIE_OUTLINE, width=2)
        d.rectangle((fx + 4, 34, fx + 36, 66), fill=(80, 70, 60, 255))
        # Silhouette
        d.ellipse((fx + 14, 40, fx + 26, 52), fill=(200, 180, 160, 255))
        d.rectangle((fx + 12, 52, fx + 28, 64), fill=(50, 50, 60, 255))
    save_png(far, "bg_ceo_far.png", palette_lock=False)

    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 40, 256, 60), fill=(30, 22, 18, 255))
    # Executive desk centered
    d.rectangle((64, 22, 192, 50), fill=(70, 40, 25, 255), outline=EDIE_OUTLINE, width=2)
    # Desk top highlight
    d.rectangle((64, 22, 192, 26), fill=(140, 90, 55, 255))
    # Desktop computer
    d.rectangle((100, 10, 156, 24), fill=(30, 30, 40, 255), outline=EDIE_OUTLINE, width=1)
    d.rectangle((102, 12, 154, 22), fill=(120, 200, 240, 255))
    # Lamp
    d.line((76, 8, 76, 22), fill=(80, 80, 90, 255))
    d.polygon([(70, 2), (82, 2), (78, 8), (74, 8)], fill=(240, 200, 80, 255), outline=EDIE_OUTLINE)
    # Office chair behind desk
    d.rectangle((120, 42, 136, 55), fill=(40, 40, 50, 255), outline=EDIE_OUTLINE, width=1)
    # Leather couches on sides
    d.rectangle((2, 32, 60, 48), fill=(120, 60, 40, 255), outline=EDIE_OUTLINE, width=1)
    d.rectangle((196, 32, 254, 48), fill=(120, 60, 40, 255), outline=EDIE_OUTLINE, width=1)
    save_png(mid, "bg_ceo_mid.png", palette_lock=False)

    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    # Plush red carpet
    d.rectangle((0, 0, 256, 80), fill=(120, 40, 40, 255))
    d.rectangle((0, 0, 256, 4), fill=(180, 60, 50, 255))
    d.rectangle((0, 4, 256, 6), fill=(220, 180, 60, 255))  # gold trim
    # Diamond pattern
    for ty in range(12, 80, 12):
        for tx in range((ty // 12) * 8 % 16, 256, 16):
            d.point((tx, ty), fill=(160, 60, 55, 255))
            d.point((tx + 1, ty), fill=(160, 60, 55, 255))
            d.point((tx, ty + 1), fill=(160, 60, 55, 255))
    save_png(floor, "bg_ceo_floor.png", palette_lock=False)


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


def _virus_frames(core, core_d, core_hi):
    import math
    w, h = 40, 40
    frames = []
    for f in range(4):
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        cx, cy = 20, 20
        d.ellipse((cx - 10, cy - 10, cx + 10, cy + 10), fill=core, outline=EDIE_OUTLINE, width=1)
        d.ellipse((cx - 7, cy - 7, cx + 4, cy + 4), fill=core_hi)
        for i in range(8):
            angle = (i / 8) * math.tau + (f * 0.2)
            sx1 = cx + int(math.cos(angle) * 10)
            sy1 = cy + int(math.sin(angle) * 10)
            sx2 = cx + int(math.cos(angle) * 16)
            sy2 = cy + int(math.sin(angle) * 16)
            d.line((sx1, sy1, sx2, sy2), fill=core_d, width=2)
            d.rectangle((sx2 - 1, sy2 - 1, sx2 + 1, sy2 + 1), fill=core_d)
        d.point((cx - 3, cy - 1), fill=core_d)
        d.point((cx + 2, cy + 3), fill=core_d)
        d.point((cx - 1, cy + 4), fill=core_d)
        frames.append(im)
    return frames


def make_virus() -> None:
    """Green + purple corona virus sprites for boss mode rain."""
    green = _virus_frames(
        (60, 200, 80, 255), (40, 140, 50, 255), (150, 240, 160, 255)
    )
    save_png(tile_horizontal(green), "virus_green.png", palette_lock=False)
    purple = _virus_frames(
        (157, 107, 255, 255), (90, 50, 180, 255), (211, 184, 255, 255)
    )
    save_png(tile_horizontal(purple), "virus_purple.png", palette_lock=False)


def make_boss_virus() -> None:
    """Giant central boss virus with yellow eyes and virus-shaped spikes.
    Spikes mirror the falling mini-virus crown pattern but scaled up."""
    import math
    w, h = 220, 220
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    cx, cy = 110, 110
    core = (60, 200, 80, 255)
    core_d = (30, 120, 40, 255)
    core_hi = (120, 230, 140, 255)
    out = EDIE_OUTLINE
    # Spike proteins -- same style as small virus, just 24 of them and bigger knobs
    num_spikes = 24
    inner_r = 58
    outer_r = 100
    for i in range(num_spikes):
        angle = (i / num_spikes) * math.tau
        sx1 = cx + int(math.cos(angle) * inner_r)
        sy1 = cy + int(math.sin(angle) * inner_r)
        sx2 = cx + int(math.cos(angle) * outer_r)
        sy2 = cy + int(math.sin(angle) * outer_r)
        # Thick stalk
        d.line((sx1, sy1, sx2, sy2), fill=core_d, width=5)
        d.line((sx1, sy1, sx2, sy2), fill=(50, 160, 60, 255), width=2)
        # Knob at tip (mirrors small-virus crown knob)
        d.ellipse((sx2 - 9, sy2 - 9, sx2 + 9, sy2 + 9), fill=core_d, outline=out, width=1)
        d.ellipse((sx2 - 6, sy2 - 6, sx2 + 6, sy2 + 6), fill=(80, 180, 90, 255))
        d.ellipse((sx2 - 3, sy2 - 3, sx2 + 3, sy2 + 3), fill=core_hi)
    # Main body (bigger)
    body_r = 62
    d.ellipse((cx - body_r, cy - body_r, cx + body_r, cy + body_r), fill=core, outline=out, width=2)
    # Inner highlight
    d.ellipse((cx - 50, cy - 50, cx + 30, cy + 30), fill=core_hi)
    d.ellipse((cx - 34, cy - 34, cx + 34, cy + 34), fill=core)
    # Inner dots
    for (dx, dy) in ((-24, 12), (26, -6), (-14, 32), (16, 28), (-34, -10), (32, 18)):
        d.ellipse((cx + dx - 3, cy + dy - 3, cx + dx + 3, cy + dy + 3), fill=core_d)
    # Yellow eyes (2 large menacing eyes)
    eye_y = cy - 6
    # Left eye
    d.ellipse((cx - 34, eye_y - 16, cx - 6, eye_y + 12), fill=(255, 255, 255, 255), outline=out, width=2)
    d.ellipse((cx - 30, eye_y - 12, cx - 10, eye_y + 8), fill=(255, 225, 50, 255))
    d.ellipse((cx - 26, eye_y - 8, cx - 14, eye_y + 4), fill=(255, 175, 30, 255))
    d.ellipse((cx - 24, eye_y - 6, cx - 16, eye_y + 2), fill=(0, 0, 0, 255))
    d.rectangle((cx - 22, eye_y - 5, cx - 20, eye_y - 3), fill=(255, 255, 255, 255))
    # Right eye
    d.ellipse((cx + 6, eye_y - 16, cx + 34, eye_y + 12), fill=(255, 255, 255, 255), outline=out, width=2)
    d.ellipse((cx + 10, eye_y - 12, cx + 30, eye_y + 8), fill=(255, 225, 50, 255))
    d.ellipse((cx + 14, eye_y - 8, cx + 26, eye_y + 4), fill=(255, 175, 30, 255))
    d.ellipse((cx + 16, eye_y - 6, cx + 24, eye_y + 2), fill=(0, 0, 0, 255))
    d.rectangle((cx + 18, eye_y - 5, cx + 20, eye_y - 3), fill=(255, 255, 255, 255))
    # Jagged mouth
    for i, mx in enumerate(range(-26, 27, 7)):
        top = cy + 20
        if i % 2 == 0:
            d.polygon([(cx + mx, top), (cx + mx + 4, top + 10), (cx + mx + 7, top)], fill=out)
        else:
            d.polygon([(cx + mx, top), (cx + mx + 4, top + 8), (cx + mx + 7, top)], fill=(40, 20, 20, 255))
    save_png(im, "boss_virus.png", palette_lock=False)


def make_heart_pickup() -> None:
    """Classic red heart pickup (4-frame pulse)."""
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
def _extract_gif_slice(gif_name: str, out_name: str, start: int, end_exclusive: int) -> None:
    """Extract a specific frame range from a GIF into a sprite sheet."""
    p = SOURCE / gif_name
    im = Image.open(p)
    n = getattr(im, "n_frames", 1)
    end = min(end_exclusive, n)
    # Union bbox across selected frames
    union = None
    for i in range(start, end):
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
        raise SystemExit(f"{gif_name}: empty slice")
    frames: list[Image.Image] = []
    for i in range(start, end):
        im.seek(i)
        fr = im.convert("RGBA").crop(union)
        frames.append(fr)
    sheet = tile_horizontal(frames)
    save_png(sheet, out_name, palette_lock=False)
    print(f"    sliced frames [{start}..{end}) -> {len(frames)}f")


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
    # Running cycle (7f): bright-eyed idle blink - also used as boss-mode EDIE
    extract_gif_to_sheet("1000027545.gif", "edie_run_anim.png")
    # Full smile loop from 1000027555.gif (17 frames)
    extract_gif_to_sheet("1000027555.gif", "edie_happy_run.png")
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
    make_car()
    make_deer()
    make_balloon_drone()
    make_vacuum_bot()
    # Procedural fallbacks first -- then real PNGs overwrite if present.
    make_amy()
    make_alice_m1()
    make_alice3()
    make_alice4()
    process_robot_refs()
    print()
    print("[pickups]")
    make_aurora()
    make_heart_pickup()
    make_virus()
    make_boss_virus()
    print()
    make_background()
    make_stage_backgrounds()
    print()
    make_sfx()
    print()
    print("Done.")


if __name__ == "__main__":
    main()
