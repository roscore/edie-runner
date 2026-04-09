#!/usr/bin/env python3
"""
Generate 3 additional variant tiles for each stage's far / mid parallax
layer. Each variant is a horizontally shift-wrapped copy of the base tile
so consecutive tiles show *different* parts of the same art, producing a
non-repeating rhythm when chained.

Run AFTER tools/generate_art.py has produced the base tiles. The build
script picks the new files up automatically because it globs assets/gen/.
"""

import os
from PIL import Image

STAGES = [
    "store",
    "street",
    "techpark",
    "highway",
    "ansan",
    "office",
    "factory",
]
LAYERS = ["far", "mid"]

# Shift fractions relative to the tile width. These are chosen so that the
# same feature (e.g. a rooftop sign) is never directly against itself when
# four tiles run in sequence.
SHIFTS = [0.25, 0.5, 0.75]

GEN_DIR = os.path.join(os.path.dirname(__file__), "..", "assets", "gen")


def shift_wrap(im: Image.Image, amount: int) -> Image.Image:
    """Horizontally shift `im` by `amount` pixels, wrapping the overflow
    back to the opposite edge. Produces a seamless loop variant."""
    w, h = im.size
    amount %= w
    if amount == 0:
        return im.copy()
    out = Image.new("RGBA", (w, h))
    # right part of the source ends up at the left of the output
    right = im.crop((w - amount, 0, w, h))
    left = im.crop((0, 0, w - amount, h))
    out.paste(right, (0, 0))
    out.paste(left, (amount, 0))
    return out


def tint(im: Image.Image, r: float, g: float, b: float) -> Image.Image:
    """Multiply each RGB channel by the given factor (alpha untouched)."""
    src = im.convert("RGBA")
    rb, gb, bb, ab = src.split()
    rb = rb.point(lambda p: min(255, int(p * r)))
    gb = gb.point(lambda p: min(255, int(p * g)))
    bb = bb.point(lambda p: min(255, int(p * b)))
    return Image.merge("RGBA", (rb, gb, bb, ab))


def generate_variants_for(stage: str, layer: str) -> int:
    base_name = f"bg_{stage}_{layer}.png"
    base_path = os.path.join(GEN_DIR, base_name)
    if not os.path.isfile(base_path):
        return 0
    src = Image.open(base_path).convert("RGBA")
    w, _ = src.size

    # v2 / v3 / v4: horizontal shift-wrap at 1/4, 1/2, 3/4 of the tile
    # width. Each gets a tiny tint wobble so the colour reads different
    # even when the silhouette still matches.
    tints = [
        (1.02, 0.98, 0.95),  # warm-ish
        (0.94, 1.00, 1.04),  # cool-ish
        (1.00, 1.03, 0.96),  # greenish warm
    ]
    written = 0
    for idx, shift_frac in enumerate(SHIFTS):
        shifted = shift_wrap(src, int(w * shift_frac))
        r, g, b = tints[idx]
        tinted = tint(shifted, r, g, b)
        out_name = f"bg_{stage}_{layer}_v{idx + 2}.png"
        out_path = os.path.join(GEN_DIR, out_name)
        tinted.save(out_path)
        written += 1
    return written


def main() -> None:
    total = 0
    for stage in STAGES:
        for layer in LAYERS:
            total += generate_variants_for(stage, layer)
    print(f"Generated {total} variant background tiles.")


if __name__ == "__main__":
    main()
