#!/usr/bin/env python3
"""
EDIE Runner -- extras generator.

Runs AFTER tools/generate_art.py. Produces:

  - Four additional Pangyo Pop-up Store shopfronts (clothes / shoes /
    desserts / phone) so the mall rotates through a real sequence of
    distinct stores instead of one repeating watch window.
  - One Hanyang ERICA main-gate variant for the Ansan stage far layer.
  - A short looping BGM WAV (sfx_bgm.wav).
  - A short countdown-beep SFX (sfx_beep.wav) and a stage-transition
    whoosh (sfx_whoosh.wav).

Everything is saved into assets/gen/ and picked up by the build.rs bundle
step automatically.
"""
from __future__ import annotations

import os
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
GEN = ROOT / "assets" / "gen"
GEN.mkdir(parents=True, exist_ok=True)

EDIE_OUTLINE = (26, 26, 26, 255)
TRANSPARENT = (0, 0, 0, 0)


def new_canvas(w: int, h: int) -> Image.Image:
    return Image.new("RGBA", (w, h), TRANSPARENT)


def save(im: Image.Image, name: str) -> None:
    im.save(GEN / name)
    print(f"  OK {name} {im.size}")


# ============================================================
# Shared helpers
# ============================================================
def shop_frame(d: ImageDraw.ImageDraw, frame: tuple[int, int, int, int]) -> None:
    """Wood shop frame + inner panel, generic to all store variants."""
    d.rectangle(frame, fill=(90, 60, 35, 255), outline=EDIE_OUTLINE, width=2)
    x1, y1, x2, y2 = frame
    d.rectangle((x1 + 2, y1 + 2, x2 - 2, y2 - 2), fill=(120, 80, 45, 255))
    # Marble pillars on the outer edges of every shop tile so tiles chain
    # together without an obvious seam when cycled.
    d.rectangle((0, 0, 8, 100), fill=(210, 200, 180, 255))
    d.rectangle((6, 0, 8, 100), fill=(180, 170, 150, 255))
    d.rectangle((248, 0, 256, 100), fill=(210, 200, 180, 255))
    d.rectangle((248, 0, 250, 100), fill=(180, 170, 150, 255))


def shop_sign(
    d: ImageDraw.ImageDraw,
    band_color: tuple[int, int, int, int],
    trim_color: tuple[int, int, int, int],
) -> None:
    """Top header band + fake letters, unique per shop."""
    d.rectangle((20, 4, 236, 16), fill=band_color, outline=EDIE_OUTLINE, width=1)
    d.rectangle((24, 7, 232, 13), fill=trim_color)
    for lx in range(40, 220, 16):
        d.rectangle((lx, 8, lx + 10, 12), fill=band_color)


def mall_floor(d: ImageDraw.ImageDraw) -> None:
    """Common shop-floor strip along the bottom."""
    d.rectangle((0, 86, 256, 94), fill=(170, 150, 130, 255))
    d.rectangle((0, 92, 256, 94), fill=(110, 90, 70, 255))


# ============================================================
# Store variant 2: Clothes / Fashion boutique
# ============================================================
def make_store_clothes() -> Image.Image:
    im = new_canvas(256, 100)
    d = ImageDraw.Draw(im)
    # Wall backdrop (warm beige)
    d.rectangle((0, 0, 256, 100), fill=(232, 218, 194, 255))
    # Shop window frame
    shop_frame(d, (16, 18, 240, 86))
    # Glass interior (pale pink)
    d.rectangle((22, 24, 234, 80), fill=(245, 228, 232, 255))
    # Mannequin on the left
    mx, my = 34, 30
    d.ellipse((mx, my, mx + 12, my + 14), fill=(230, 210, 185, 255), outline=EDIE_OUTLINE, width=1)
    d.polygon(
        [(mx - 4, my + 14), (mx + 16, my + 14), (mx + 20, my + 50), (mx - 8, my + 50)],
        fill=(200, 80, 120, 255),
        outline=EDIE_OUTLINE,
    )
    d.rectangle((mx + 3, my + 28, mx + 9, my + 32), fill=(150, 40, 80, 255))
    # Clothing racks with hanging dresses
    for rx, colors in (
        (84, [(220, 110, 140, 255), (120, 150, 210, 255), (240, 210, 110, 255)]),
        (144, [(140, 200, 170, 255), (220, 170, 200, 255), (110, 130, 180, 255)]),
        (196, [(240, 190, 110, 255), (200, 100, 130, 255), (150, 180, 220, 255)]),
    ):
        d.line((rx - 14, 28, rx + 14, 28), fill=(90, 80, 90, 255))
        for i, c in enumerate(colors):
            cx = rx - 12 + i * 9
            d.rectangle((cx, 30, cx + 7, 56), fill=c, outline=EDIE_OUTLINE, width=1)
            d.rectangle((cx, 55, cx + 7, 58), fill=(80, 80, 80, 255))
            d.line((cx + 3, 28, cx + 3, 30), fill=(180, 170, 170, 255))
    # Shop floor
    mall_floor(d)
    # Sign band
    shop_sign(d, (110, 40, 80, 255), (220, 160, 190, 255))
    return im


# ============================================================
# Store variant 3: Shoes
# ============================================================
def make_store_shoes() -> Image.Image:
    im = new_canvas(256, 100)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 256, 100), fill=(225, 228, 238, 255))
    shop_frame(d, (16, 18, 240, 86))
    # Glass interior (cool blue)
    d.rectangle((22, 24, 234, 80), fill=(222, 230, 244, 255))
    # Three tiered display shelves
    for sy in (34, 50, 66):
        d.rectangle((26, sy, 230, sy + 4), fill=(140, 120, 95, 255), outline=EDIE_OUTLINE, width=1)
        d.rectangle((28, sy + 1, 228, sy + 3), fill=(190, 170, 140, 255))
    # Shoes on each shelf (mixture of sneakers and heels)
    shoe_palette = [
        (230, 230, 235, 255),
        (230, 90, 80, 255),
        (80, 130, 200, 255),
        (60, 60, 60, 255),
        (240, 200, 90, 255),
        (200, 140, 200, 255),
    ]
    for shelf_y in (34, 50, 66):
        for i, col in enumerate(shoe_palette):
            sx = 36 + i * 32
            # Sneaker body
            d.rectangle((sx, shelf_y - 8, sx + 22, shelf_y), fill=col, outline=EDIE_OUTLINE, width=1)
            # Sole
            d.rectangle((sx, shelf_y - 2, sx + 22, shelf_y), fill=(250, 250, 250, 255))
            # Laces / detail
            d.line((sx + 4, shelf_y - 7, sx + 14, shelf_y - 7), fill=EDIE_OUTLINE)
            d.line((sx + 4, shelf_y - 5, sx + 14, shelf_y - 5), fill=EDIE_OUTLINE)
    mall_floor(d)
    shop_sign(d, (20, 50, 100, 255), (140, 180, 220, 255))
    return im


# ============================================================
# Store variant 4: Desserts / Patisserie
# ============================================================
def make_store_desserts() -> Image.Image:
    im = new_canvas(256, 100)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 256, 100), fill=(250, 230, 220, 255))
    shop_frame(d, (16, 18, 240, 86))
    # Display-case glass (cream)
    d.rectangle((22, 24, 234, 80), fill=(252, 244, 232, 255))
    # Counter top (marble)
    d.rectangle((22, 58, 234, 68), fill=(230, 220, 210, 255), outline=EDIE_OUTLINE, width=1)
    # Cupcakes row on the counter
    for i in range(10):
        cx = 30 + i * 20
        # Wrapper (brown)
        d.polygon(
            [(cx, 58), (cx + 14, 58), (cx + 12, 52), (cx + 2, 52)],
            fill=(140, 80, 40, 255),
            outline=EDIE_OUTLINE,
        )
        # Frosting (pastel pink / mint / yellow)
        frosting = [(240, 170, 200, 255), (170, 220, 190, 255), (245, 215, 130, 255)][i % 3]
        d.ellipse((cx + 1, 44, cx + 13, 54), fill=frosting, outline=EDIE_OUTLINE, width=1)
        # Cherry on top
        d.ellipse((cx + 5, 40, cx + 9, 44), fill=(220, 60, 70, 255), outline=EDIE_OUTLINE)
    # Macarons on upper shelf
    d.rectangle((22, 32, 234, 36), fill=(200, 180, 160, 255), outline=EDIE_OUTLINE, width=1)
    mac_colors = [
        (250, 200, 210, 255),
        (180, 220, 230, 255),
        (240, 220, 160, 255),
        (200, 240, 200, 255),
        (220, 180, 230, 255),
    ]
    for i in range(10):
        mx = 28 + i * 20
        c = mac_colors[i % 5]
        d.ellipse((mx, 24, mx + 14, 32), fill=c, outline=EDIE_OUTLINE, width=1)
        d.rectangle((mx + 1, 27, mx + 13, 29), fill=(240, 240, 240, 255))
    mall_floor(d)
    shop_sign(d, (180, 90, 120, 255), (255, 210, 220, 255))
    return im


# ============================================================
# Store variant 5: Phone / Mobile store
# ============================================================
def make_store_phone() -> Image.Image:
    im = new_canvas(256, 100)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 256, 100), fill=(220, 228, 236, 255))
    shop_frame(d, (16, 18, 240, 86))
    # Glass interior (tech cool blue)
    d.rectangle((22, 24, 234, 80), fill=(232, 240, 252, 255))
    # Phone display stands -- 6 phones in a row
    for i in range(6):
        cx = 34 + i * 34
        # Stand
        d.rectangle((cx - 2, 62, cx + 16, 66), fill=(150, 150, 160, 255), outline=EDIE_OUTLINE, width=1)
        d.rectangle((cx + 4, 60, cx + 10, 66), fill=(120, 120, 130, 255))
        # Phone body (tall rectangle)
        d.rectangle((cx, 28, cx + 14, 60), fill=(40, 40, 50, 255), outline=EDIE_OUTLINE, width=1)
        # Screen (glowing)
        screen_cols = [
            (90, 170, 240, 255),
            (230, 130, 160, 255),
            (150, 220, 180, 255),
            (240, 200, 120, 255),
            (180, 140, 220, 255),
            (100, 200, 220, 255),
        ]
        d.rectangle((cx + 2, 32, cx + 12, 56), fill=screen_cols[i % len(screen_cols)])
        # Camera / speaker
        d.rectangle((cx + 6, 30, cx + 8, 31), fill=(200, 200, 210, 255))
    # Tech accent LEDs along the top of the case
    for lx in range(28, 232, 16):
        d.rectangle((lx, 24, lx + 8, 26), fill=(90, 180, 240, 255))
    mall_floor(d)
    shop_sign(d, (30, 80, 140, 255), (150, 200, 240, 255))
    return im


# ============================================================
# Store mid layer variants (shop-floor / velvet rope strip)
# These are simpler -- one unified mid layer that reads as a mall
# corridor floor with rope dividers. We produce 4 variants with
# slightly different rope color per shop type so consecutive tiles
# still feel visually connected but not identical.
# ============================================================
def make_store_mid(rope: tuple[int, int, int, int]) -> Image.Image:
    im = new_canvas(256, 60)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 44, 256, 60), fill=(225, 215, 195, 255))
    d.line((0, 44, 256, 44), fill=(180, 165, 140, 255))
    for sx in (20, 88, 156, 224):
        d.rectangle((sx, 26, sx + 4, 48), fill=(190, 155, 60, 255), outline=EDIE_OUTLINE, width=1)
        d.ellipse((sx - 2, 22, sx + 6, 28), fill=(230, 190, 80, 255), outline=EDIE_OUTLINE, width=1)
    for a, b in ((22, 90), (90, 158), (158, 226)):
        d.line((a + 4, 32, b - 2, 32), fill=rope)
        d.line((a + 4, 33, b - 2, 33), fill=rope)
    return im


# ============================================================
# Hanyang University ERICA main gate (Ansan stage far variant)
# ============================================================
def make_ansan_hanyang_gate() -> Image.Image:
    im = new_canvas(256, 100)
    d = ImageDraw.Draw(im)
    # Sky / wall backdrop
    d.rectangle((0, 0, 256, 100), fill=(210, 220, 238, 255))

    # Trees flanking the gate
    for tx in (12, 236):
        d.rectangle((tx, 60, tx + 4, 100), fill=(70, 50, 30, 255))
        d.ellipse((tx - 14, 38, tx + 18, 78), fill=(60, 120, 70, 255), outline=EDIE_OUTLINE)
        d.ellipse((tx - 10, 42, tx + 14, 70), fill=(105, 165, 90, 255))
        d.ellipse((tx - 6, 50, tx + 6, 62), fill=(150, 200, 110, 255))

    # Left pillar
    d.rectangle((40, 22, 76, 100), fill=(238, 238, 242, 255), outline=EDIE_OUTLINE, width=2)
    d.rectangle((44, 26, 72, 96), fill=(208, 214, 226, 255))
    # Vertical groove lines
    for gx in (50, 60, 66):
        d.line((gx, 26, gx, 96), fill=(180, 188, 200, 255))
    # Right pillar
    d.rectangle((180, 22, 216, 100), fill=(238, 238, 242, 255), outline=EDIE_OUTLINE, width=2)
    d.rectangle((184, 26, 212, 96), fill=(208, 214, 226, 255))
    for gx in (190, 200, 206):
        d.line((gx, 26, gx, 96), fill=(180, 188, 200, 255))

    # Crossbar header (Hanyang navy)
    d.rectangle((32, 6, 224, 34), fill=(24, 42, 96, 255), outline=EDIE_OUTLINE, width=2)
    d.rectangle((34, 8, 222, 32), fill=(36, 64, 130, 255))
    # Gold trim
    d.rectangle((34, 8, 222, 10), fill=(220, 180, 60, 255))
    d.rectangle((34, 30, 222, 32), fill=(220, 180, 60, 255))

    # "HANYANG ERICA" as fake block letters
    for lx in range(44, 124, 10):
        d.rectangle((lx, 14, lx + 6, 24), fill=(255, 255, 255, 255))
        d.rectangle((lx, 18, lx + 6, 20), fill=(210, 210, 220, 255))
    # Central red "H" crest (Hanyang accent red)
    d.rectangle((120, 12, 136, 28), fill=(220, 50, 60, 255), outline=EDIE_OUTLINE, width=1)
    d.rectangle((123, 12, 125, 28), fill=(255, 255, 255, 255))
    d.rectangle((131, 12, 133, 28), fill=(255, 255, 255, 255))
    d.rectangle((123, 18, 133, 20), fill=(255, 255, 255, 255))
    # "ERICA" block letters on the right of the crest
    for lx in range(142, 212, 10):
        d.rectangle((lx, 14, lx + 6, 24), fill=(255, 255, 255, 255))
        d.rectangle((lx, 18, lx + 6, 20), fill=(210, 210, 220, 255))

    # Lamps on top of the pillars
    for lx in (52, 192):
        d.ellipse((lx, 16, lx + 12, 24), fill=(255, 240, 180, 255), outline=EDIE_OUTLINE, width=1)
        d.rectangle((lx + 4, 18, lx + 8, 24), fill=(255, 250, 200, 255))

    # Paved road between the pillars
    d.rectangle((76, 50, 180, 100), fill=(108, 108, 112, 255))
    d.rectangle((76, 50, 180, 52), fill=(70, 70, 72, 255))
    # Road center dashes
    for dy in (60, 72, 84, 96):
        d.rectangle((124, dy, 132, dy + 3), fill=(230, 230, 230, 255))

    return im


# ============================================================
# BGM + extra SFX
# ============================================================
def make_audio_extras() -> None:
    print("[sfx] generating BGM and extra SFX")
    import wave
    import numpy as np

    SR = 22050

    def write_wav(name: str, samples: np.ndarray) -> None:
        samples = np.clip(samples, -1.0, 1.0)
        pcm = (samples * 32000).astype(np.int16)
        with wave.open(str(GEN / name), "w") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes(pcm.tobytes())
        print(f"  OK {name} {len(pcm)/SR:.2f}s")

    def env(n: int, attack: float = 0.01, decay: float = 0.3) -> np.ndarray:
        t = np.arange(n) / SR
        a = np.clip(t / attack, 0, 1)
        dc = np.exp(-t / decay)
        return a * dc

    # --- Looping chiptune BGM ---------------------------------------
    # 8 seconds of a simple happy major-key loop with bass + melody.
    dur_loop = 8.0
    n = int(dur_loop * SR)
    t = np.arange(n) / SR
    bpm = 132
    beat = 60.0 / bpm  # seconds per beat

    # Melody pattern: C E G C E G E C (up-down arpeggio) repeated
    melody_notes = [
        523.25, 659.25, 783.99, 1046.50,
        783.99, 659.25, 523.25, 659.25,
    ]  # C5 E5 G5 C6 G5 E5 C5 E5
    bass_notes = [
        130.81, 130.81, 164.81, 164.81,
        196.00, 196.00, 130.81, 164.81,
    ]  # C3-C3-E3-E3-G3-G3-C3-E3

    melody = np.zeros(n)
    bass = np.zeros(n)
    note_n = int(beat * SR / 2.0)  # 8th notes -> quarter note has two
    for i in range(16):
        note_start = i * note_n
        if note_start >= n:
            break
        idx = i % len(melody_notes)
        mf = melody_notes[idx]
        bf = bass_notes[idx]
        seg_t = np.arange(note_n) / SR
        seg_env = np.clip(seg_t / 0.01, 0, 1) * np.exp(-seg_t / 0.25)
        mel_wave = 0.18 * np.sign(np.sin(2 * np.pi * mf * seg_t)) * seg_env
        bass_env = np.clip(seg_t / 0.02, 0, 1) * np.exp(-seg_t / 0.4)
        bass_wave = 0.22 * np.sin(2 * np.pi * bf * seg_t) * bass_env
        end = min(note_start + note_n, n)
        cut = end - note_start
        melody[note_start:end] += mel_wave[:cut]
        bass[note_start:end] += bass_wave[:cut]
    # Hat: ticking noise on each 8th
    hat = np.zeros(n)
    for i in range(16):
        note_start = i * note_n
        if note_start >= n:
            break
        seg = np.random.RandomState(10 + i).uniform(-1, 1, note_n)
        seg *= np.exp(-np.arange(note_n) / SR / 0.03) * 0.06
        end = min(note_start + note_n, n)
        hat[note_start:end] += seg[:end - note_start]
    bgm = melody + bass + hat
    # Gentle fade at the loop boundary to avoid a pop
    fade = int(0.04 * SR)
    fade_env = np.linspace(0.0, 1.0, fade)
    bgm[:fade] *= fade_env
    bgm[-fade:] *= fade_env[::-1]
    write_wav("sfx_bgm.wav", bgm)

    # --- Countdown beep ---------------------------------------------
    dur = 0.18
    n = int(dur * SR)
    t = np.arange(n) / SR
    beep = 0.35 * np.sin(2 * np.pi * 880 * t) * env(n, 0.002, 0.14)
    write_wav("sfx_beep.wav", beep)

    # --- Stage transition whoosh ------------------------------------
    dur = 0.45
    n = int(dur * SR)
    t = np.arange(n) / SR
    freq = 120 + 900 * (t / dur)
    noise = np.random.RandomState(42).uniform(-1, 1, n)
    # Lowpass-like smoothing
    alpha = 0.06
    smooth = np.zeros(n)
    y = 0.0
    for i in range(n):
        y = y * (1 - alpha) + noise[i] * alpha
        smooth[i] = y
    whoosh = (
        0.35 * np.sin(2 * np.pi * np.cumsum(freq) / SR) * env(n, 0.01, 0.28)
        + 0.45 * smooth * env(n, 0.005, 0.22)
    )
    write_wav("sfx_whoosh.wav", whoosh)


# ============================================================
# Main
# ============================================================
def main() -> None:
    # ---- Store shop variants ----
    save(make_store_clothes(), "bg_store_far_v2.png")
    save(make_store_shoes(), "bg_store_far_v3.png")
    save(make_store_desserts(), "bg_store_far_v4.png")
    save(make_store_phone(), "bg_store_far_v5.png")
    save(make_store_mid((170, 30, 40, 255)), "bg_store_mid_v2.png")
    save(make_store_mid((40, 70, 130, 255)), "bg_store_mid_v3.png")
    save(make_store_mid((200, 120, 150, 255)), "bg_store_mid_v4.png")
    save(make_store_mid((40, 100, 160, 255)), "bg_store_mid_v5.png")

    # ---- Hanyang ERICA main gate (Ansan) ----
    save(make_ansan_hanyang_gate(), "bg_ansan_far_v2.png")

    # ---- BGM + extra SFX ----
    make_audio_extras()

    print("extras generated.")


if __name__ == "__main__":
    main()
