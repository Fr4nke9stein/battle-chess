"""Фигуры варваров с рогатыми шлемами — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`, с новыми материалами прямо на теле (свет и тени — как
на остальном): корпус и огоньки — в цветах сторон (стороны не путаются), голова — под шлемом
из тёмного железа (`HELM`) с бронзовым ободом (`BRONZE`) и заклёпками, по бокам шлема —
изогнутые рога (`HORN`, к концу темнеют — `TIP`): выходят наружу и загибаются вверх, к концу
тоньше. Отделка корпуса (пояса, грива коня) — меховая (`FUR` с тёмными клоками `FUR_DARK` по
шуму), основание — как было. У ферзя шары короны — бронзовые шипы, верхний камень светится;
у короля крест над шлемом; у ладьи зубцы — железо, рога по бокам башни; у коня — налобник
с рогами у ушей.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_barbarian.py [папка]` — по умолчанию assets/breach/pieces:
barbarian/<цвет><фигура>.png и <цвет><фигура>_shadow.png, превью preview_barbarian_pieces.png
(обычные и варвары) и preview_barbarian_board.png (армии на доске).
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
from breach_art import GLOW, MAIN, R, T, TRIM, Y, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

HELM, BRONZE, HORN, TIP, FUR, FUR_DARK = 5, 6, 7, 8, 9, 10
EXTRA = {
    HELM: ["#16181e", "#30333c", "#525663", "#7e8494", "#b9bfcc"],  # тёмное железо
    BRONZE: ["#2e1a08", "#5e3812", "#93601e", "#c99236", "#f0cf72"],
    HORN: ["#4a3a26", "#7f6a4a", "#b49e78", "#ddcfae", "#f6efdc"],  # кость
    TIP: ["#1e1610", "#3a2c1e", "#5e4a34", "#86704f", "#a8946e"],
    FUR: ["#2a1608", "#4e2c12", "#7a4a22", "#a26c3a", "#c8935c"],
    FUR_DARK: ["#1a0d05", "#33190a", "#522f14", "#704622", "#8c5e34"],
}

# шлем: (высота, с которой начинается купол, обод (от, до)); голова — (центр по высоте, радиус)
HELMETS = {
    "P": (24.6, (22.6, 24.6), (23.0, 7.4)),
    "B": (29.0, (27.0, 29.0), (33.0, 8.4)),
    "R": (29.6, (27.0, 29.6), (33.0, 12.0)),
    "Q": (44.0, (41.2, 44.0), (44.4, 8.6)),
    "K": (49.0, (46.4, 49.0), (49.6, 8.4)),
}
# рога: высота основания над центром головы (доля радиуса), вынос наружу и вверх (доли радиуса), толщина
HORNS = {
    "P": (0.25, 0.95, 1.15, 0.36),
    "B": (0.0, 1.0, 1.25, 0.34),
    "R": (0.1, 0.7, 1.0, 0.27),
    "Q": (0.3, 1.05, 1.2, 0.32),
    "K": (0.3, 1.1, 1.25, 0.34),
}


def helmeted(part, cap, band, seed, keep=()):
    """Часть фигуры в шлеме: выше cap — железо, обод — бронза; отделка корпуса ниже — мех."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        out = base.copy()
        free = (base == MAIN) | (base == TRIM)
        y = p[:, 1]
        out[free & (y >= cap)] = HELM
        out[free & (y >= band[0]) & (y < band[1])] = BRONZE
        fur = (base == TRIM) & (y < band[0]) & (y > art.BASE_BANDS[1] + 0.5)
        for lo, hi in keep:
            fur &= ~((y >= lo) & (y < hi))
        out[fur] = FUR
        out[fur & (ip.noise3(p, 1.3, seed) > 0.56)] = FUR_DARK
        return out

    return f, mat


def horn(base, out_dir, r, rise_k, out_k, up_k, thick_k):
    """Рог цепочкой шаров: от шлема наружу, потом вверх; к концу тоньше, кончик темнее."""
    parts = []
    n = 22
    b = base + Y * (r * rise_k)
    for i in range(n):
        u = i / (n - 1)
        pos = b + out_dir * (r * out_k * (1 - (1 - u) ** 2)) + Y * (r * up_k * u ** 1.8)
        rad = r * thick_k * (1 - 0.82 * u)
        parts.append((sphere(pos, rad), TIP if u > 0.72 else HORN))
    return parts


def rivets(y, r, azimuths, rad=0.65):
    return [(sphere(ip.at(r, y, az), rad), BRONZE) for az in azimuths]


def barbarian(kind, color):
    piece = art.PIECES[kind]()
    seed = 51 + "PNBRQK".index(kind) * 5 + (0 if color == "w" else 2)
    if kind == "N":
        return barbarian_knight(piece, seed)
    cap, band, (hy, hr) = HELMETS[kind]
    parts = list(piece.parts)
    if kind == "Q":  # шары короны — бронзовые шипы, верхний камень оставляем
        parts = [parts[0]] + [(f, BRONZE) for f, _ in parts[1:9]] + parts[9:]
    if kind == "K":  # крест — над шлемом, как был
        parts = [helmeted(parts[0], cap, band, seed)] + parts[1:]
    else:
        parts = [helmeted(parts[0], cap, band, seed)] + parts[1:]
    rise, out_k, up_k, thick = HORNS[kind]
    centre = Y * hy
    extra = []
    for s in (R, -R):  # по бокам — видно в силуэте
        start = centre + s * (hr * 0.78)
        extra += horn(start, s, hr, rise, out_k, up_k, thick)
    band_r = {"P": 7.3, "B": 9.25, "R": 12.1, "Q": 9.6, "K": 10.9}[kind]
    extra += rivets((band[0] + band[1]) / 2, band_r, (5, 25, 45, 65, 85))
    return art.Piece(parts + extra, stretch=piece.stretch)


def barbarian_knight(piece, seed):
    """Конь: налобник из железа с бронзовой кромкой, рога у ушей, грива — мех."""
    a = math.radians(18)
    fwd = norm(-R * math.cos(a) + T * math.sin(a))
    side = norm(np.cross(Y, fwd))
    base_part, (head_f, head_m), (mane_f, _) = piece.parts

    def head_mat(p):
        m = head_m(p)
        u, y = p @ fwd, p[:, 1]
        free = m == MAIN
        cap = 41.5 - 0.35 * np.maximum(u - 2.0, 0.0)  # налобник спускается ко лбу
        m[free & (y >= cap)] = HELM
        m[free & (y >= cap - 1.4) & (y < cap)] = BRONZE
        return m

    def mane_mat(p):
        m = np.full(len(p), FUR)
        m[ip.noise3(p, 1.3, seed) > 0.56] = FUR_DARK
        return m

    parts = [base_part, (head_f, head_mat), (mane_f, mane_mat)]
    extra = []
    for sgn in (1, -1):  # рога у ушей: назад (по экрану — от морды) и вверх, чуть врозь — видно над головой
        s = norm(-fwd * 0.85 + side * sgn * 0.5)
        extra += horn(-fwd * 1.0 + Y * 45.0 + side * sgn * 3.4, s, 9.0, 0.0, 1.05, 1.2, 0.3)
    return art.Piece(parts + extra, stretch=piece.stretch)


def render(kind, color):
    key = f"barbarian_{color}"
    pal = dict(art.PALETTES[color])
    pal.update(EXTRA)
    art.PALETTES[key] = pal
    return art.render_piece(barbarian(kind, color), key, 0.86)


def preview(sprites, path):
    """Ряды: обычные и варвары; в ряду белые и чёрные фигуры."""
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
    os.makedirs(os.path.join(OUT, "barbarian"), exist_ok=True)
    sprites = {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(OUT, "barbarian", f"{color}{kind}.png"))
            shadow.save(os.path.join(OUT, "barbarian", f"{color}{kind}_shadow.png"))
            sprites[color + kind] = (body, shadow)
            print(color + kind, flush=True)
    preview(sprites, os.path.join(OUT, "preview_barbarian_pieces.png"))
    art.board_preview(sprites, scale=2, demo=False).save(os.path.join(OUT, "preview_barbarian_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
