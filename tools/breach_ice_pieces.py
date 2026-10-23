"""Ледяные фигуры: иссиня-голубые, с участками наледи — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`, с новыми материалами прямо на теле (свет и тени —
как на остальном): корпус и отделка — синие (у белых светлый ледниково-голубой, у чёрных —
глубокий тёмно-синий: стороны не путаются), огоньки — как у сторон (голубые у белых,
оранжевые у чёрных). Наледь: пятна прозрачного льда по бокам (`ICE`, по 3D-шуму), иней
шапками на верхних поверхностях (`RIME`, по нормали вверх), лёд наползает от основания.
Объёмные детали: сосульки свисают из-под поясов и короны (цепочки шариков, к низу тоньше),
у основания торчат ледяные кристаллы. Самые светлые пиксели льда — белые искры.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_ice_pieces.py [папка]` — по умолчанию assets/breach/pieces:
ice/<цвет><фигура>.png и <цвет><фигура>_shadow.png, превью preview_ice_pieces.png (обычные и
ледяные) и preview_ice_pieces_board.png (армии на доске).
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
from breach_art import MAIN, TRIM, GLOW, R, T, Y, hexc, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

ICE, RIME = 5, 6  # новые материалы: прозрачный лёд и иней

PALETTES = {
    "w": {
        MAIN: ["#0f2a54", "#1d4f98", "#377fd4", "#6cb1f0", "#b4e0ff"],  # иссиня-голубой
        TRIM: ["#0a1830", "#173462", "#28579a", "#4682c4", "#7fb4e6"],
        GLOW: art.PALETTES["w"][GLOW],
    },
    "b": {
        MAIN: ["#060a1c", "#0f1a40", "#1d306c", "#304c9a", "#5272c4"],  # глубокий тёмно-синий
        TRIM: ["#03050d", "#0a112c", "#152156", "#253a80", "#4560aa"],
        GLOW: art.PALETTES["b"][GLOW],
    },
}
ICE_RAMP = ["#4d84ad", "#86bddf", "#bde5f7", "#e4f7ff", "#ffffff"]
RIME_RAMP = ["#8ea9bf", "#b9cddd", "#d8e6f0", "#eef5fa", "#ffffff"]
for side in PALETTES.values():
    side[ICE] = ICE_RAMP
    side[RIME] = RIME_RAMP
    side[art.SEAM] = ["#020201"] * 5

BASE_R = {"P": 13.5, "N": 14.0, "B": 13.8, "R": 14.0, "Q": 14.2, "K": 14.6}
FRONT = 45.0  # азимут, смотрящий на зрителя
# откуда свисают сосульки: (высота, радиус) кромок поясов и короны
ICICLES = {
    "P": [(13.6, 8.0)],
    "R": [(27.0, 10.6), (29.2, 11.6)],
    "B": [(21.4, 8.6)],
    "Q": [(25.4, 8.8), (42.2, 10.6)],
    "K": [(28.0, 9.3), (46.5, 10.6)],
    "N": [],
}


def at(r, y, az):
    a = math.radians(az)
    return np.array([r * math.cos(a), y, r * math.sin(a)])


def hash3(ix, iy, iz, seed):
    h = (ix * 73856093) ^ (iy * 19349663) ^ (iz * 83492791) ^ (seed * 2654435761)
    h = (h ^ (h >> 13)) * 1274126177
    return ((h ^ (h >> 16)) & 0xFFFF) / 65535.0


def noise3(p, cell, seed):
    """Гладкий 3D-шум 0…1 по решётке с шагом cell."""
    q = p / cell
    i = np.floor(q).astype(np.int64)
    fr = q - i
    fr = fr * fr * (3 - 2 * fr)
    out = np.zeros(len(p))
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (fr[:, 0] if dx else 1 - fr[:, 0]) * (fr[:, 1] if dy else 1 - fr[:, 1]) * \
                    (fr[:, 2] if dz else 1 - fr[:, 2])
                out += w * hash3(i[:, 0] + dx, i[:, 1] + dy, i[:, 2] + dz, seed)
    return out


def iced(part, seed, frost_h):
    """Часть фигуры с наледью: пятна льда, иней на верхних поверхностях, лёд от основания."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        e = 0.08
        ny = (f(p + [0, e, 0]) - f(p - [0, e, 0])) / (2 * e)
        n = noise3(p, 6.5, seed)  # крупные цельные пятна
        n2 = noise3(p, 2.2, seed + 7)
        out = base.copy()
        free = (base == MAIN) | (base == TRIM)
        v = n + 0.12 * n2
        patch = free & (v > 0.8)
        out[patch] = ICE
        out[free & (v > 0.76) & ~patch] = RIME  # иней по краю пятна
        cap = free & (ny > 0.7) & (n2 > 0.55)
        out[cap] = RIME
        edge = frost_h + 2.0 * (n2 - 0.5)
        low = free & (p[:, 1] < edge)
        out[low] = ICE
        out[low & (p[:, 1] > edge - 0.6)] = RIME
        return out

    return f, mat


def icicle(top, length, rad=0.95):
    """Сосулька: цепочка шариков вниз, к концу тоньше."""
    parts = []
    k = 4
    for i in range(k):
        t = i / (k - 1)
        parts.append((sphere(top - Y * (length * t), rad * (1 - 0.65 * t)), ICE))
    return parts


def shard(base, out_dir, length, thick):
    """Ледяной кристалл у основания: вытянутый скруглённый брусок, наклонён наружу и вверх."""
    up = norm(out_dir * 0.55 + Y)
    side = norm(np.cross(up, out_dir))
    fwd = norm(np.cross(side, up))
    c = base + up * (length / 2)
    return [(art.rbox(c, [thick, length / 2, thick], [side, up, fwd], 0.35), ICE)]


def adorned(kind, color):
    piece = getattr(art, {"P": "pawn", "N": "knight", "B": "bishop", "R": "rook", "Q": "queen", "K": "king"}[kind])()
    seed = 11 + "PNBRQK".index(kind) * 5 + (0 if color == "w" else 2)
    rng = np.random.default_rng(seed)
    parts = [iced(p, seed + k, 1.8) for k, p in enumerate(piece.parts)]
    extra = []
    for y, r in ICICLES[kind]:
        for az in (FRONT - 50, FRONT - 18, FRONT + 14, FRONT + 46, FRONT + 80):
            if rng.random() < 0.8:
                extra += icicle(at(r, y, az + rng.uniform(-6, 6)), rng.uniform(2.5, 4.8))
    if kind == "N":  # сосульки из-под челюсти коня
        a = math.radians(18)
        fwd = norm(-R * math.cos(a) + T * math.sin(a))
        side = norm(np.cross(Y, fwd))
        if side @ art.D < 0:
            side = -side
        for u, ln in ((9.4, 3.6), (11.8, 2.8)):
            extra += icicle(fwd * u + Y * 23.4 + side * 2.6, ln, 0.85)
    r0 = BASE_R[kind]
    for az, ln in ((FRONT - 55, 4.5), (FRONT + 8, 5.5), (FRONT + 62, 4.0)):
        d = at(1, 0, az)
        extra += shard(at(r0 - 1.0, 1.5, az), d, ln, 1.1)
    return art.Piece(parts + extra, stretch=piece.stretch)


def render(kind, color):
    key = f"ice_{color}"
    art.PALETTES[key] = PALETTES[color]
    body, shadow = art.render_piece(adorned(kind, color), key, 0.86)
    img = np.array(body)
    # искры: самые светлые пиксели льда с открытым верхом-слева — белые
    top = np.array(hexc(ICE_RAMP[3]))
    bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:3]:
        img[y, x, :3] = (255, 255, 255)
    return Image.fromarray(img, "RGBA"), shadow


def preview(sprites, path):
    """Ряды: обычные и ледяные; в ряду белые и чёрные фигуры."""
    cw, ch = 40, 72
    sheet = Image.new("RGBA", (12 * cw + 8, 2 * ch + 8), hexc("#2c3a2c") + (255,))
    sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), hexc("#3d4a3a") + (255,)), (0, 4 + ch))
    for r in range(2):
        for c, code in enumerate(color + k for color in "wb" for k in "PNBRQK"):
            if r == 0:
                body = Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA")
            else:
                body = sprites[code][0]
            sheet.alpha_composite(body.crop((12, 6, 52, 78)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def main():
    os.makedirs(os.path.join(OUT, "ice"), exist_ok=True)
    sprites = {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(OUT, "ice", f"{color}{kind}.png"))
            shadow.save(os.path.join(OUT, "ice", f"{color}{kind}_shadow.png"))
            sprites[color + kind] = (body, shadow)
            print(color + kind, flush=True)
    preview(sprites, os.path.join(OUT, "preview_ice_pieces.png"))
    art.board_preview(sprites, scale=2, demo=False).save(os.path.join(OUT, "preview_ice_pieces_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
