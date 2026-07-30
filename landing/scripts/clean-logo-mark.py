"""Lift muddy shading on the existing logo-mark.png — no new artwork."""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src" / "assets" / "logo-mark.png"
OUT = ROOT / "src" / "assets" / "logo-mark-clean.png"


def main() -> None:
    im = Image.open(SRC).convert("RGBA")
    arr = np.array(im).astype(np.float32)
    r, g, b, a = [arr[:, :, i] for i in range(4)]
    lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
    chroma = np.maximum(np.maximum(r, g), b) - np.minimum(np.minimum(r, g), b)

    # Soft matte: near-black plate → transparent
    alpha = np.clip((lum - 14) / 28.0, 0, 1) * 255.0
    alpha = np.minimum(alpha, a)
    alpha = np.where(lum < 14, 0.0, alpha)

    # Kill pale fringe / halo on soft edges
    fringe = (alpha > 0) & (alpha < 230) & (lum > 90)
    alpha = np.where(fringe, alpha * 0.06, alpha)

    alpha_img = Image.fromarray(np.clip(alpha, 0, 255).astype(np.uint8), mode="L")
    alpha_blur = np.array(
        alpha_img.filter(ImageFilter.GaussianBlur(radius=0.7))
    ).astype(np.float32)
    rim = (alpha > 6) & (alpha < 245)
    alpha = np.where(rim, alpha_blur, alpha)
    alpha = np.where(alpha < 10, 0.0, alpha)

    opaque = alpha > 30

    # Flatten muddy greys (low chroma midtones) toward cleaner silver
    muddy = opaque & (chroma < 45) & (lum >= 18) & (lum < 150)
    # Target luminance curve: lift floor so snout reads silver, not soot
    target = 95 + (lum - 18) * (155 / 132.0)  # 18→95, 150→~250
    target = np.clip(target, 0, 250)
    scale = np.where(lum > 1, target / np.maximum(lum, 1.0), 1.0)
    scale = np.clip(scale, 1.0, 3.2)
    r = np.where(muddy, np.clip(r * scale, 0, 255), r)
    g = np.where(muddy, np.clip(g * scale, 0, 255), g)
    b = np.where(muddy, np.clip(b * scale, 0, 255), b)

    # Second pass: remaining midtones get a gentler gamma lift
    lum2 = 0.2126 * r + 0.7152 * g + 0.0722 * b
    mid = opaque & (lum2 >= 30) & (lum2 < 120)
    gamma = 0.72
    lifted = 255.0 * np.power(np.clip(lum2 / 255.0, 0, 1), gamma)
    mix = np.where(mid, 0.55, 0.0)
    scale2 = np.where(lum2 > 1, lifted / np.maximum(lum2, 1.0), 1.0)
    r = np.where(opaque, r * (1 - mix) + r * scale2 * mix, r)
    g = np.where(opaque, g * (1 - mix) + g * scale2 * mix, g)
    b = np.where(opaque, b * (1 - mix) + b * scale2 * mix, b)

    # Whiten light stripes
    lum3 = 0.2126 * r + 0.7152 * g + 0.0722 * b
    bright = opaque & (lum3 > 100)
    r = np.where(bright, np.clip(r * 1.1 + 10, 0, 255), r)
    g = np.where(bright, np.clip(g * 1.1 + 10, 0, 255), g)
    b = np.where(bright, np.clip(b * 1.1 + 12, 0, 255), b)

    # Warm/gold → cool silver
    gold = opaque & (r > 105) & (g > 65) & ((r - b) > 18)
    lum4 = 0.2126 * r + 0.7152 * g + 0.0722 * b
    t = np.clip((lum4 - 50) / 170.0, 0, 1)
    silver = 140 + t * 100
    r = np.where(gold, silver, r)
    g = np.where(gold, silver * 0.99 + 2, g)
    b = np.where(gold, np.minimum(255, silver * 1.06 + 10), b)

    out = np.stack(
        [
            np.clip(r, 0, 255),
            np.clip(g, 0, 255),
            np.clip(b, 0, 255),
            np.clip(alpha, 0, 255),
        ],
        axis=-1,
    ).astype(np.uint8)
    result = Image.fromarray(out, "RGBA")

    bbox = result.getbbox()
    if bbox:
        result = result.crop(bbox)

    w, h = result.size
    crop_w = int(w * 0.78)
    right = np.array(result)[:, crop_w:, 3]
    left = np.array(result)[:, :crop_w, 3]
    if right.mean() < left.mean() * 0.55 and float(right.mean()) < 40:
        result = result.crop((0, 0, crop_w, h))
        bb = result.getbbox()
        if bb:
            result = result.crop(bb)

    w, h = result.size
    side = max(w, h)
    pad = int(side * 0.1)
    canvas = Image.new("RGBA", (side + pad * 2, side + pad * 2), (0, 0, 0, 0))
    canvas.paste(result, ((canvas.size[0] - w) // 2, (canvas.size[1] - h) // 2), result)

    canvas = canvas.resize(
        (canvas.size[0] * 2, canvas.size[1] * 2),
        Image.Resampling.LANCZOS,
    )

    canvas.save(OUT, optimize=True)

    a2 = np.array(canvas)
    lum_f = 0.2126 * a2[:, :, 0] + 0.7152 * a2[:, :, 1] + 0.0722 * a2[:, :, 2]
    muddy2 = (a2[:, :, 3] > 40) & (lum_f < 100) & (lum_f > 20)
    print(
        f"wrote {OUT} size={canvas.size} muddy_left={int(muddy2.sum())} "
        f"avg_muddy_lum={float(lum_f[muddy2].mean()) if muddy2.any() else 'n/a'}"
    )


if __name__ == "__main__":
    main()
