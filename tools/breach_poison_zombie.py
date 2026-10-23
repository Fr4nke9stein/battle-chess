"""Ядовитые зомби — зомби `breach_zombie.py` и ядовитые фигуры `breach_poison.py` вместе, в стиле Into the Breach.

Фигура — зомби целиком (трупная кожа с гнилью, тряпьё с лохмотьями, вырванные куски с рёбрами, шрамы, земля на
основании, огни), сверху облита ядовитой жидкостью, как ядовитые фигуры: шапка с неровной кромкой, потёки вниз
разной длины, лужицы на уступах, набухшие капли на концах длинных потёков и блики (те же зерно и потёки, что у
ядовитой фигуры этого вида и цвета). Жидкость ложится на кожу, гниль, тряпьё, бронзу и гриву; огни, рот, глазница, шрамы, раны
(мясо и кость) и земля основания — поверх неё не заливаются.

Материалы зомби — 5–12, жидкость — 13 (`SLIME`; у ядовитых фигур она 5).

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_poison_zombie.py [папка]` — по умолчанию assets/breach/pieces: poison_zombie/<цвет><фигура>.png и
_shadow.png, превью preview_poison_zombie.png (ряды: ядовитые, зомби, вместе) и preview_poison_zombie_board.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_ice_pieces as ip  # noqa: E402
import breach_poison as bp  # noqa: E402
import breach_vampire as vp  # noqa: E402
import breach_zombie as zb  # noqa: E402
from breach_art import MAIN, TRIM, hexc, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

SLIME = 13
WET = [MAIN, TRIM, zb.ROT, zb.RAG, zb.RUST, zb.HAIR]  # что заливает жидкость (рот, глазница и шрамы видны сквозь неё)


def drips_of(kind, color):
    """Потёки — как у ядовитой фигуры (`bp.poisoned`): то же зерно, те же азимуты и длины."""
    seed = 31 + "PNBRQK".index(kind) * 7 + (0 if color == "w" else 3)
    rng = np.random.default_rng(seed)
    top = bp.top_of(art.PIECES[kind]())
    drips = []
    for a in np.linspace(bp.FRONT - 85, bp.FRONT + 85, 6):
        drips.append((a + rng.uniform(-9, 9), rng.uniform(0.12, 0.55) * top, rng.uniform(1.1, 2.0)))
    for a in (bp.FRONT + 150, bp.FRONT + 210):
        drips.append((a, rng.uniform(0.1, 0.4) * top, rng.uniform(1.2, 2.0)))
    return seed, top, drips


def slimed(part, seed, top, drips):
    """Деталь зомби под жидкостью: шапка, потёки и лужицы, как в `bp.slimy`, но поверх материалов зомби."""
    f, m = part

    def mat(p):
        out = m(p) if callable(m) else np.full(len(p), m)
        out = out.copy()
        free = np.isin(out, WET)
        az = np.degrees(np.arctan2(p[:, 2], p[:, 0]))
        r = np.maximum(np.hypot(p[:, 0], p[:, 2]), 2.0)
        cap = bp.COAT * top
        depth = cap + 1.6 * (ip.noise3(p, 3.0, seed) - 0.5)
        for a, length, width in drips:
            d = (((az - a + 180) % 360) - 180) * math.pi / 180 * r
            k = np.clip(1 - (d / width) ** 2, 0, 1)
            depth = np.maximum(depth, cap + length * np.sqrt(k))
        coat = p[:, 1] > top - depth
        e = 0.08
        ny = (f(p + [0, e, 0]) - f(p - [0, e, 0])) / (2 * e)
        pool = (ny > 0.85) & (ip.noise3(p, 2.5, seed + 3) > 0.66) & (p[:, 1] > zb.BASE_TOP)
        out[free & (coat | pool)] = SLIME
        return out

    return f, mat


def poison_zombie(kind, color):
    piece = zb.zombie(kind, color)
    seed, top, drips = drips_of(kind, color)
    parts = [slimed(p, seed + k, top, drips) for k, p in enumerate(piece.parts)]
    plain = art.PIECES[kind]()
    for a, length, _ in drips:  # набухшие капли на концах длинных потёков
        if length < 0.3 * top:
            continue
        y = top - bp.COAT * top - length + 0.7
        r = bp.surface_r(plain, y, a)
        if r is not None:
            parts.append((sphere(ip.at(r - 0.25, y, a), 1.05), SLIME))
    return art.Piece(parts, stretch=piece.stretch)


def render(kind, color):
    key = f"poison_zombie_{color}"
    pal = dict(art.PALETTES[color])
    pal[MAIN] = zb.SKIN[color]
    pal[art.GLOW] = zb.GLOWS[color]
    pal.update(zb.COMMON)
    pal.update(zb.SIDE[color])
    pal[SLIME] = bp.SLIME_RAMP
    art.PALETTES[key] = pal
    body, shadow = art.render_piece(poison_zombie(kind, color), key, 0.86)
    img = np.array(body)  # блики на самой светлой жидкости, как у ядовитых
    bright = (img[..., :3] == np.array(hexc(bp.SLIME_RAMP[4]))).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:4]:
        img[y, x, :3] = bp.GLINT
    return Image.fromarray(img, "RGBA"), shadow


def main():
    folder = os.path.join(OUT, "poison_zombie")
    os.makedirs(folder, exist_ok=True)
    load = lambda sub: {c + k: Image.open(os.path.join(OUT, sub, f"{c}{k}.png")).convert("RGBA")  # noqa: E731
                        for c in "wb" for k in "PNBRQK"}
    sprites, shadows = {}, {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(folder, f"{color}{kind}.png"))
            shadow.save(os.path.join(folder, f"{color}{kind}_shadow.png"))
            sprites[color + kind], shadows[color + kind] = body, shadow
            print("poison_zombie", color + kind, flush=True)
    vp.preview([load("poison"), load("zombie"), sprites], os.path.join(OUT, "preview_poison_zombie.png"))
    board = {code: (sprites[code], shadows[code]) for code in sprites}
    art.board_preview(board, scale=2, demo=False).save(os.path.join(OUT, "preview_poison_zombie_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
