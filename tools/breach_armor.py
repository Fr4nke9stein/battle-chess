"""Фигуры в латах — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`, закованы в стальные латы (свет и тени — как на остальном).
Латы объёмные: корпус от основания до шеи покрыт полосами высотой `LAME` (`lames`) — каждая
полоса толще тела и к нижнему краю расходится (`FLARE`), нижняя кромка нависает над следующей
полосой и отбрасывает на неё тень, как у настоящих лат; по нижней кромке — латунная кайма
(`EDGE`). У коня латы — только на шее, морда цельная. Голова — стальной шлем, прорези и огоньки
светятся как были. Отделка (пояса, корона, грива коня) — латунь (`BRASS`, у чёрных медь), на
плечах — ступенчатые наплечники (купол и под ним две полосы, `spaulder`) с латунной
заклёпкой. Сталь у белых светлая полированная, у чёрных — тёмная воронёная (`PLATE`);
основание — в цветах сторон.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_armor.py [папка]` — по умолчанию assets/breach/pieces:
armor/<цвет><фигура>.png и <цвет><фигура>_shadow.png, превью preview_armor_pieces.png (обычные и
в латах) и preview_armor_board.png (армии на доске).
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
from breach_art import GLOW, MAIN, R, TRIM, Y, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

PLATE, BRASS = 5, 6
ARMOR = {
    "w": {
        PLATE: ["#2a2e38", "#56607a", "#8e9ab4", "#c8d2e6", "#f6faff"],  # полированная сталь
        BRASS: ["#3a2408", "#74501a", "#b48434", "#e0b860", "#fff0b0"],
    },
    "b": {
        PLATE: ["#0c0d16", "#22253a", "#3c4262", "#626a94", "#9aa4d4"],  # воронёная сталь
        BRASS: ["#2e1006", "#5e2814", "#94482a", "#c87a4e", "#f0b48a"],  # медь
    },
}
LAME = 5.2  # высота полосы лат (ниже — похоже на пружину)
FLARE = (0.1, 2.0)  # насколько полоса толще тела: у верхнего края и у нижнего
EDGE = 0.75  # латунная кайма по нижней кромке полосы
# латы: от низа (над основанием) до шеи; голова выше — шлем
LAMES = {"P": (7.0, 13.0), "B": (8.6, 20.8), "R": (10.0, 26.0), "Q": (8.8, 25.0), "K": (9.0, 27.4), "N": (9.6, 30.0)}
# наплечники: высота, отступ от оси, размер
SPAULDERS = {"P": (14.6, 8.4, 4.0), "B": (21.8, 8.6, 4.2), "Q": (26.0, 8.6, 4.3), "K": (28.4, 9.0, 4.6)}


def lames(f, low, high):
    """Полосы лат по телу f: каждая полоса — тело, раздутое от FLARE[0] у верха до FLARE[1] у
    низа; нижняя кромка нависает над следующей полосой."""
    parts = []
    y = low
    while y < high - 0.5:
        y0, y1 = y, min(y + LAME, high)

        def band(p, y0=y0, y1=y1):
            t = np.clip((y1 - p[:, 1]) / (y1 - y0), 0, 1)  # 0 — верх полосы, 1 — низ
            grow = FLARE[0] + (FLARE[1] - FLARE[0]) * t
            return np.maximum.reduce([f(p) - grow, y0 - p[:, 1], p[:, 1] - y1])

        def mat(p, y0=y0):
            out = np.full(len(p), PLATE)
            out[p[:, 1] < y0 + EDGE] = BRASS
            return out

        parts.append((band, mat))
        y = y1
    return parts


def steel(part):
    """Тело под латами и шлем: корпус — сталь, отделка — латунь; основание и огни — как были."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        out = base.copy()
        above = p[:, 1] > art.BASE_BANDS[1] + 0.2
        out[(base == MAIN) & above] = PLATE
        out[(base == TRIM) & above] = BRASS
        return out

    return f, mat


def spaulder(centre, out_dir, size):
    """Наплечник: купол, под ним две полосы — каждая шире и ниже предыдущей; заклёпка сверху."""
    up = norm(Y + out_dir * 0.5)
    c = np.asarray(centre, float)
    parts = [(sphere(c, size), PLATE)]
    for k in (1, 2):
        cc = c - up * (size * 0.42 * k) + out_dir * (0.35 * k)
        rad = size * (0.86 - 0.12 * k)

        def ring(p, cc=cc, rad=rad):
            d = p - cc
            h = d @ up
            radial = np.linalg.norm(d - h[:, None] * up, axis=1)
            return np.hypot(np.maximum(radial - rad, 0), np.maximum(np.abs(h) - 0.55, 0)) - 0.5

        parts.append((ring, PLATE))
    parts.append((sphere(c + up * (size * 0.8), 0.7), BRASS))
    return parts


def armour(kind):
    piece = art.PIECES[kind]()
    low, high = LAMES[kind]
    if kind == "N":
        base_part, (head_f, head_m), (mane_f, _) = piece.parts
        parts = [steel(base_part), steel((head_f, head_m)), (mane_f, BRASS)]
        parts += lames(head_f, low, high)
        return art.Piece(parts, stretch=piece.stretch)
    body = piece.parts[0]
    parts = [steel(body)]
    if kind == "Q":  # шары короны — латунь, верхний камень светится
        parts += [(f, BRASS) for f, _ in piece.parts[1:9]] + piece.parts[9:]
    elif kind == "K":  # крест — латунь, камень в нём светится
        def cross_mat(m):
            return lambda p: np.where(m(p) == GLOW, GLOW, BRASS)
        parts += [(f, cross_mat(m)) for f, m in piece.parts[1:]]
    else:
        parts += piece.parts[1:]
    parts += lames(body[0], low, high)
    if kind in SPAULDERS:
        y, r, size = SPAULDERS[kind]
        for s in (R, -R):
            parts += spaulder(s * r + Y * y, s, size)
    return art.Piece(parts, stretch=piece.stretch)


def render(kind, color):
    key = f"armor_{color}"
    pal = dict(art.PALETTES[color])
    pal.update(ARMOR[color])
    art.PALETTES[key] = pal
    return art.render_piece(armour(kind), key, 0.86)


def preview(sprites, path):
    """Ряды: обычные и в латах; в ряду белые и чёрные фигуры."""
    cw, ch = 44, 76
    sheet = Image.new("RGBA", (12 * cw + 8, 2 * ch + 8), art.hexc("#2c3a2c") + (255,))
    sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), art.hexc("#3d4a3a") + (255,)), (0, 4 + ch))
    for r in range(2):
        for c, code in enumerate(color + k for color in "wb" for k in "PNBRQK"):
            if r == 0:
                body = Image.open(os.path.join(OUT, f"{code}.png")).convert("RGBA")
            else:
                body = sprites[code][0]
            sheet.alpha_composite(body.crop((10, 2, 54, 78)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def main():
    os.makedirs(os.path.join(OUT, "armor"), exist_ok=True)
    sprites = {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(OUT, "armor", f"{color}{kind}.png"))
            shadow.save(os.path.join(OUT, "armor", f"{color}{kind}_shadow.png"))
            sprites[color + kind] = (body, shadow)
            print(color + kind, flush=True)
    preview(sprites, os.path.join(OUT, "preview_armor_pieces.png"))
    art.board_preview(sprites, scale=2, demo=False).save(os.path.join(OUT, "preview_armor_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
