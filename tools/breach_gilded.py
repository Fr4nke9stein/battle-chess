"""Роскошные фигуры: позолоченные и целиком золотые — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`, но с украшениями прямо на теле (свет и тени
ложатся как на остальном): ободок отделки — золото, на поясе и короне — крупные
камни, по уступу основания — нитка жемчуга, на корпусе — пояс филиграни (два тонких
кольца, между ними выпуклые бусины). У ферзя шары короны — жемчуг, сверху
большой камень; у короля камень в кресте; у коня — розетка с камнем на ремне.

Два варианта, чтобы стороны не путались:
- `gilded` — корпус в цвете стороны (белые — слоновая кость, чёрные — фиолетовый
  панцирь), отделка и филигрань — золото; камни у белых — сапфиры, у чёрных — рубины;
- `gold` — корпус целиком золотой: у белых жёлтое золото с белой эмалью (ободки и узор) и
  сапфирами, у чёрных красное золото с чёрной эмалью и рубинами.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_gilded.py [папка]` — по умолчанию assets/breach/pieces:
<вариант>/<цвет><фигура>.png и <цвет><фигура>_shadow.png, превью preview_gilded.png (ряды:
обычные, позолоченные, золотые) и preview_gilded_board.png (армии на доске).
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

FILI, GEM, PEARL = 5, 6, 7  # новые материалы: узор на корпусе, камни, жемчуг

GOLD = ["#3b2106", "#7a4a0e", "#c08a1e", "#f0c64a", "#fff4b8"]
RED_GOLD = ["#2a0f06", "#5e2a0e", "#a35a1e", "#d98f3a", "#f7cf86"]
SAPPHIRE = ["#0d2160", "#1c47b8", "#3f78ff", "#9cc0ff", "#eef4ff"]
RUBY = ["#5a0814", "#a3142a", "#e2323f", "#ff8a84", "#fff0ee"]
PEARLS = ["#56525c", "#958f9e", "#d4d0da", "#f0eef4", "#ffffff"]
WHITE_ENAMEL = ["#4a4a55", "#8a8a98", "#cfd0da", "#eceef4", "#ffffff"]
BLACK_ENAMEL = ["#050407", "#121018", "#23202d", "#3b3849", "#66637a"]
IVORY = art.PALETTES["w"][MAIN]
PURPLE = art.PALETTES["b"][MAIN]

PALETTES = {
    "gilded": {
        "w": {MAIN: IVORY, TRIM: GOLD, GLOW: art.PALETTES["w"][GLOW], FILI: GOLD, GEM: SAPPHIRE, PEARL: PEARLS},
        "b": {MAIN: PURPLE, TRIM: GOLD, GLOW: art.PALETTES["b"][GLOW], FILI: GOLD, GEM: RUBY, PEARL: PEARLS},
    },
    "gold": {
        "w": {MAIN: GOLD, TRIM: WHITE_ENAMEL, GLOW: art.PALETTES["w"][GLOW], FILI: WHITE_ENAMEL, GEM: SAPPHIRE,
              PEARL: PEARLS},
        "b": {MAIN: RED_GOLD, TRIM: BLACK_ENAMEL, GLOW: art.PALETTES["b"][GLOW], FILI: BLACK_ENAMEL, GEM: RUBY,
              PEARL: PEARLS},
    },
}
for pal in PALETTES.values():
    for side in pal.values():
        side[art.SEAM] = ["#020201"] * 5

BASE_R = {"P": 13.5, "N": 14.0, "B": 13.8, "R": 14.0, "Q": 14.2, "K": 14.6}
FRONT = 45.0  # азимут, смотрящий на зрителя (направление T)


def at(r, y, az):
    a = math.radians(az)
    return [r * math.cos(a), y, r * math.sin(a)]


def ring(y, r, n, rad, mat, az0=FRONT):
    """n шариков по кругу: камни на поясе или жемчуг."""
    return [(sphere(at(r, y, az0 + 360 * k / n), rad), mat) for k in range(n)]


def azimuth(p):
    return np.degrees(np.arctan2(p[:, 2], p[:, 0]))


def belt_rings(y0, y1, w=0.5):
    """Пояс филиграни: два тонких кольца по краям (между ними — бусины, см. studs)."""
    def f(p):
        y = p[:, 1]
        return (np.abs(y - y0) < w) | (np.abs(y - y1) < w)
    return f


def studs(y, r, n, rad=0.85, az0=FRONT):
    """Выпуклые бусины на поясе — ловят свет."""
    return ring(y, r, n, rad, FILI, az0)


def rook_frame(p):
    """Золотая рамка бойницы ладьи и край зубцов."""
    r = np.hypot(p[:, 0], p[:, 2])
    front = art.front_mask(p, 0.93) & ~art.front_mask(p, 0.975)
    frame = front & (p[:, 1] > 12.2) & (p[:, 1] < 21.8) & (r > 7.5)
    frame |= art.front_mask(p, 0.93) & (r > 7.5) & ((np.abs(p[:, 1] - 12.6) < 0.6) | (np.abs(p[:, 1] - 21.4) < 0.6))
    top = (p[:, 1] > 36.6) & (r > 8.8)
    return frame | top


def with_pattern(part, *patterns):
    """Часть фигуры, у которой корпус (MAIN) там, где узор, — FILI."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        hit = np.zeros(len(p), bool)
        for pat in patterns:
            hit |= pat(p)
        return np.where((base == MAIN) & hit, FILI, base)

    return f, mat


def remap(part, old, new):
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        return np.where(base == old, new, base)

    return f, mat


def pearls(kind):
    r0 = BASE_R[kind]
    return ring(3.35, r0 - 1.5, 26, 0.72, PEARL, FRONT + 7)


def adorned(kind):
    """Фигура с украшениями."""
    piece = getattr(art, {"P": "pawn", "N": "knight", "B": "bishop", "R": "rook", "Q": "queen", "K": "king"}[kind])()
    parts = list(piece.parts)
    extra = pearls(kind)
    if kind == "P":
        parts[0] = with_pattern(parts[0], belt_rings(18.0, 20.8, 0.45))
        extra += studs(19.4, 7.25, 9)
        extra += ring(14.85, 8.9, 5, 1.75, GEM)
    elif kind == "R":
        parts[0] = with_pattern(parts[0], belt_rings(22.2, 25.0), rook_frame)
        extra += studs(23.6, 9.1, 12)
        extra += [(sphere(at(12.25, 32.6, az), 1.8), GEM) for az in (0, 90, 180, 270)]
    elif kind == "B":
        parts[0] = with_pattern(parts[0], belt_rings(26.4, 29.4))
        extra += studs(27.9, 8.35, 10)
        extra += ring(22.6, 9.5, 5, 1.8, GEM)
        extra += [(sphere(at(8.3, 31.4, FRONT + 40), 1.6), GEM)]
    elif kind == "Q":
        parts[0] = with_pattern(parts[0], belt_rings(34.6, 38.2))
        parts = [parts[0]] + [(f, PEARL) if m == GLOW else (f, m) for f, m in parts[1:-1]] + [(parts[-1][0], GEM)]
        extra += studs(36.4, 9.15, 12)
        extra += ring(26.7, 9.7, 5, 1.8, GEM)
        extra += ring(42.6, 11.2, 8, 1.5, GEM)
    elif kind == "K":
        # на чаше под короной бусины уходят в тень — только одно тонкое кольцо
        parts[0] = with_pattern(parts[0], belt_rings(41.0, 41.0))
        parts[1:] = [remap(p, GLOW, GEM) for p in parts[1:]]
        extra += ring(29.4, 10.1, 5, 1.85, GEM)
        extra += ring(47.7, 11.25, 8, 1.5, GEM)
    elif kind == "N":
        extra += ring(8.6, 9.9, 6, 1.7, GEM, FRONT + 20)
        # розетка с камнем на ремне уздечки — с видимой стороны головы
        a = math.radians(18)
        fwd = norm(-R * math.cos(a) + T * math.sin(a))
        side = norm(np.cross(Y, fwd))
        if side @ art.D < 0:
            side = -side
        y = 33.0
        u = 1.2 + (y - 24.0) * 0.16
        c = fwd * u + Y * y + side * 4.4
        extra += [(sphere(c, 2.2), TRIM), (sphere(c + side * 1.2, 1.5), GEM)]
    return art.Piece(parts + extra, stretch=piece.stretch)


def render(kind, color, variant):
    key = f"{variant}_{color}"
    art.PALETTES[key] = PALETTES[variant][color]
    body, shadow = art.render_piece(adorned(kind), key, 0.86)
    img = np.array(body)
    # блеск: самые светлые пиксели золота с открытым верхом-слева — белые
    gold = PALETTES[variant][color][TRIM if variant == "gilded" else MAIN]
    top = np.array(hexc(gold[4]))
    bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:2]:
        img[y, x, :3] = (255, 255, 255)
    return Image.fromarray(img, "RGBA"), shadow


def preview(sprites, path):
    """Ряды: обычные, позолоченные, золотые; в ряду белые и чёрные фигуры."""
    kinds = "PNBRQK"
    variants = (None, "gilded", "gold")
    cw, ch = 40, 72
    sheet = Image.new("RGBA", (12 * cw + 8, 3 * ch + 8), hexc("#2c3a2c") + (255,))
    for r, variant in enumerate(variants):
        if r % 2:
            sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), hexc("#3d4a3a") + (255,)), (0, 4 + r * ch))
        for c, code in enumerate(color + k for color in "wb" for k in kinds):
            if variant is None:
                body = Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA")
            else:
                body = sprites[variant][code][0]
            sheet.alpha_composite(body.crop((12, 6, 52, 78)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def board(sprites, path):
    """Обе армии в начальной расстановке на доске игры: слева позолоченные, справа золотые."""
    boards = [art.board_preview(sprites[v], scale=1, demo=False) for v in ("gilded", "gold")]
    w, h = boards[0].size
    sheet = Image.new("RGBA", (2 * w, h))
    for i, b in enumerate(boards):
        sheet.alpha_composite(b, (i * w, 0))
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(path)


def main():
    sprites = {}
    for variant in ("gilded", "gold"):
        os.makedirs(os.path.join(OUT, variant), exist_ok=True)
        sprites[variant] = {}
        for color in "wb":
            for kind in "PNBRQK":
                body, shadow = render(kind, color, variant)
                body.save(os.path.join(OUT, variant, f"{color}{kind}.png"))
                shadow.save(os.path.join(OUT, variant, f"{color}{kind}_shadow.png"))
                sprites[variant][color + kind] = (body, shadow)
                print(variant, color + kind, flush=True)
    preview(sprites, os.path.join(OUT, "preview_gilded.png"))
    board(sprites, os.path.join(OUT, "preview_gilded_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
