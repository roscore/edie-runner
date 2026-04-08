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
def make_coiled_cable() -> None:
    w, h = 32, 32
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    # Main coil — dark grey
    d.ellipse((4, 8, 27, 27), fill=(80, 80, 86, 255), outline=EDIE_OUTLINE, width=1)
    d.ellipse((9, 13, 22, 22), fill=(60, 60, 66, 255), outline=EDIE_OUTLINE, width=1)
    # Cool accent dot
    d.rectangle((14, 16, 16, 18), fill=COOL_ACCENT)
    save_png(im, "obstacle_cable.png")


def make_charging_dock() -> None:
    w, h = 32, 64
    frames = []
    for frame_idx, lit in enumerate([False, True]):
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        # Base
        d.rectangle((4, h - 8, w - 5, h - 1), fill=(70, 70, 76, 255), outline=EDIE_OUTLINE, width=1)
        # Pole
        d.rectangle((13, 8, 18, h - 8), fill=(90, 90, 98, 255), outline=EDIE_OUTLINE, width=1)
        # Top pad
        d.rectangle((6, 4, 25, 12), fill=(60, 60, 66, 255), outline=EDIE_OUTLINE, width=1)
        # LED
        led_color = COOL_ACCENT if lit else (40, 50, 60, 255)
        d.rectangle((14, 7, 17, 9), fill=led_color)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_dock.png")


def make_tool_cart() -> None:
    w, h = 80, 40
    im = new_canvas(w, h)
    d = ImageDraw.Draw(im)
    # Body
    d.rectangle((4, 10, w - 5, h - 10), fill=(110, 80, 50, 255), outline=EDIE_OUTLINE, width=1)
    # Top shelf
    d.rectangle((10, 5, w - 11, 12), fill=(140, 100, 60, 255), outline=EDIE_OUTLINE, width=1)
    # Cool accent stripe
    d.rectangle((6, 18, w - 7, 22), fill=COOL_ACCENT)
    # Wheels
    d.ellipse((6, h - 12, 16, h - 2), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=1)
    d.ellipse((w - 17, h - 12, w - 7, h - 2), fill=(40, 40, 46, 255), outline=EDIE_OUTLINE, width=1)
    save_png(im, "obstacle_cart.png")


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


def make_quad_drone() -> None:
    w, h = 56, 32
    frames = []
    for f in range(4):
        im = new_canvas(w, h)
        d = ImageDraw.Draw(im)
        # Body — charcoal
        d.ellipse((18, 10, 38, 22), fill=(60, 60, 66, 255), outline=EDIE_OUTLINE, width=1)
        # Eye / lens
        d.rectangle((26, 14, 30, 17), fill=COOL_ACCENT)
        # Arms
        d.line((20, 12, 6, 6), fill=EDIE_OUTLINE, width=1)
        d.line((36, 12, 50, 6), fill=EDIE_OUTLINE, width=1)
        d.line((20, 20, 6, 26), fill=EDIE_OUTLINE, width=1)
        d.line((36, 20, 50, 26), fill=EDIE_OUTLINE, width=1)
        # Rotors — animated blur
        for cx, cy in [(6, 6), (50, 6), (6, 26), (50, 26)]:
            if f % 2 == 0:
                d.line((cx - 4, cy, cx + 4, cy), fill=(140, 140, 150, 255), width=1)
            else:
                d.line((cx, cy - 4, cx, cy + 4), fill=(140, 140, 150, 255), width=1)
        frames.append(im)
    sheet = tile_horizontal(frames)
    save_png(sheet, "obstacle_drone.png")


def make_spark_burst() -> None:
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
    save_png(sheet, "obstacle_spark.png", palette_lock=False)


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
def make_background() -> None:
    print("[bg] generating layers")
    # Sky — solid 1280×200
    sky = Image.new("RGBA", (1280, 200), BG_SKY)
    save_png(sky, "bg_sky.png", palette_lock=False)

    # Far — 256×100, server silhouettes
    far = new_canvas(256, 100)
    d = ImageDraw.Draw(far)
    silhouette_x = 0
    heights = [60, 80, 50, 90, 70, 55, 85, 65, 75, 50, 95, 60]
    widths = [24, 20, 28, 18, 22, 30, 16, 24, 20, 26, 18, 28]
    for hh, ww in zip(heights, widths):
        d.rectangle((silhouette_x, 100 - hh, silhouette_x + ww, 100), fill=BG_FAR)
        # window dots
        for wy in range(100 - hh + 8, 100 - 4, 6):
            for wx in range(silhouette_x + 4, silhouette_x + ww - 2, 4):
                d.point((wx, wy), fill=(160, 170, 175, 255))
        silhouette_x += ww
    save_png(far, "bg_far.png", palette_lock=False)

    # Mid — 256×60 workbench silhouette
    mid = new_canvas(256, 60)
    d = ImageDraw.Draw(mid)
    d.rectangle((0, 30, 256, 60), fill=BG_MID)
    # legs
    for x in (10, 50, 90, 130, 170, 210, 240):
        d.rectangle((x, 35, x + 4, 60), fill=(100, 95, 80, 255))
    # surface highlight
    d.line((0, 30, 256, 30), fill=(170, 160, 140, 255), width=1)
    save_png(mid, "bg_mid.png", palette_lock=False)

    # Floor — 256×80
    floor = new_canvas(256, 80)
    d = ImageDraw.Draw(floor)
    d.rectangle((0, 0, 256, 80), fill=FLOOR)
    d.line((0, 0, 256, 0), fill=FLOOR_LINE, width=2)
    # Subtle floor lines
    for x in range(0, 256, 32):
        d.line((x, 4, x, 78), fill=(60, 55, 46, 255), width=1)
    save_png(floor, "bg_floor.png", palette_lock=False)


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
    make_coiled_cable()
    make_charging_dock()
    make_tool_cart()
    make_sensor_cone()
    make_quad_drone()
    make_spark_burst()
    print()
    print("[pickups]")
    make_aurora()
    print()
    make_background()
    print()
    print("Done.")


if __name__ == "__main__":
    main()
