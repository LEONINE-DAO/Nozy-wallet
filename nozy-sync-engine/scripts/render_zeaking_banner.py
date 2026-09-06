#!/usr/bin/env python3
"""zcashd-style terminal art from the Nozy *icon* (zebra + gold shield, no wordmark).

Uses truecolor half-blocks (▀/▄), same idea as zcashd's img2txt splash.
Source is the black-background brand mark, not the photo lockup.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parent
# Prefer the high-res brand icon (profile zebra + gold shield, no type).
CANDIDATES = [
    ROOT / "assets" / "zeaking-mark.png",
    REPO / "landing" / "src" / "assets" / "logo-icon.png",
    REPO / "logo-icon.png",
]
PREVIEW = ROOT / "assets" / "banner-pixel-preview.png"
OUT_RS = ROOT / "src" / "banner_ansi.rs"

# ~zcashd metrics art is ~40–50 cols; we use half-blocks so this is 2px tall per row.
WIDTH = 76
HEIGHT_PX = 64  # even
SCALE = 4
BLACK_LUMA = 22  # treat near-black as terminal background


def find_src() -> Path:
    for p in CANDIDATES:
        if p.exists():
            return p
    raise SystemExit("missing zeaking-mark / logo-icon.png")


def luma(rgb: tuple[int, int, int]) -> float:
    r, g, b = rgb
    return 0.299 * r + 0.587 * g + 0.114 * b


def content_bbox(im: Image.Image, pad: int = 8) -> tuple[int, int, int, int]:
    """Tight box around non-black pixels."""
    rgb = im.convert("RGB")
    w, h = rgb.size
    px = rgb.load()
    top, left, right, bottom = h, w, 0, 0
    found = False
    for y in range(h):
        for x in range(w):
            if luma(px[x, y]) > BLACK_LUMA + 8:
                found = True
                top = min(top, y)
                left = min(left, x)
                right = max(right, x)
                bottom = max(bottom, y)
    if not found:
        return (0, 0, w, h)
    return (
        max(0, left - pad),
        max(0, top - pad),
        min(w, right + 1 + pad),
        min(h, bottom + 1 + pad),
    )


def to_rgba_keyed(small: Image.Image) -> Image.Image:
    rgb = small.convert("RGB")
    w, h = rgb.size
    out = Image.new("RGBA", (w, h))
    sp = rgb.load()
    dp = out.load()
    for y in range(h):
        for x in range(w):
            r, g, b = sp[x, y]
            if luma((r, g, b)) <= BLACK_LUMA:
                dp[x, y] = (0, 0, 0, 0)
            else:
                dp[x, y] = (r, g, b, 255)
    return out


def ansi_half_blocks(small: Image.Image) -> str:
    w, h = small.size
    px = small.load()

    def cell(x: int, y: int) -> tuple[int, int, int] | None:
        r, g, b, a = px[x, y]
        if a < 128:
            return None
        return (r, g, b)

    lines: list[str] = []
    for y in range(0, h, 2):
        parts: list[str] = ["  "]
        for x in range(w):
            top = cell(x, y)
            bot = cell(x, y + 1) if y + 1 < h else None
            if top is None and bot is None:
                parts.append(" ")
                continue
            if top is None:
                r, g, b = bot
                parts.append(f"\x1b[38;2;{r};{g};{b}m▄\x1b[0m")
            elif bot is None:
                r, g, b = top
                parts.append(f"\x1b[38;2;{r};{g};{b}m▀\x1b[0m")
            else:
                tr, tg, tb = top
                br, bg, bb = bot
                parts.append(
                    f"\x1b[38;2;{tr};{tg};{tb}m\x1b[48;2;{br};{bg};{bb}m▀\x1b[0m"
                )
        lines.append("".join(parts).rstrip())
    return "\n".join(lines)


def ascii_fallback(small: Image.Image) -> str:
    w, h = small.size
    px = small.load()
    lines: list[str] = []
    for y in range(h):
        chars: list[str] = ["  "]
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < 128:
                chars.append(" ")
            elif r >= 160 and g >= 110 and b <= 120:
                chars.append("$")
            else:
                chars.append("#" if luma((r, g, b)) >= 140 else "=")
        lines.append("".join(chars).rstrip())
    return "\n".join(lines)


def rust_string(name: str, body: str) -> str:
    escaped = (
        body.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\x1b", "\\x1b")
        .replace("\n", "\\n\\\n")
    )
    return f'pub const {name}: &str = "{escaped}";\n'


def save_preview(small: Image.Image) -> None:
    w, h = small.size
    canvas = Image.new("RGB", (w, h), (0, 0, 0))
    canvas.paste(small, mask=small.split()[3])
    preview = canvas.resize((w * SCALE, h * SCALE), Image.Resampling.NEAREST)
    PREVIEW.parent.mkdir(parents=True, exist_ok=True)
    preview.save(PREVIEW, "PNG")


def main() -> None:
    src = find_src()
    mark_copy = ROOT / "assets" / "zeaking-mark.png"
    im = Image.open(src).convert("RGBA")
    if src != mark_copy:
        mark_copy.parent.mkdir(parents=True, exist_ok=True)
        im.save(mark_copy, "PNG")

    cropped = im.crop(content_bbox(im))
    # LANCZOS keeps the stripe curves; no 2-color snap (that made the photo unreadable).
    small_rgb = cropped.convert("RGB").resize((WIDTH, HEIGHT_PX), Image.Resampling.LANCZOS)
    small = to_rgba_keyed(small_rgb)
    save_preview(small)

    ansi = ansi_half_blocks(small)
    ascii_small = small.resize((WIDTH, HEIGHT_PX // 2), Image.Resampling.NEAREST)
    fallback = ascii_fallback(ascii_small)

    header = """//! Generated by `scripts/render_zeaking_banner.py` — do not hand-edit.
//! Source: `assets/zeaking-mark.png` (brand icon: zebra + gold shield, no wordmark)

"""
    OUT_RS.write_text(
        header + rust_string("BANNER_ANSI", ansi) + "\n" + rust_string("BANNER_ASCII", fallback),
        encoding="utf-8",
    )
    print(f"source {src}")
    print(f"wrote {PREVIEW}")
    print(f"wrote {OUT_RS}  ansi_lines={ansi.count(chr(10))+1}")


if __name__ == "__main__":
    main()
