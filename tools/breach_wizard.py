"""Фигуры в шляпах мага — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py` (корпус, отделка и огоньки — в цветах сторон); макушка
срезана (`HATS`: высота среза), сверху — остроконечная шляпа мага: широкие поля (скруглённый диск),
высокая тулья цепочкой шаров вдоль изогнутой оси — к концу тоньше, кончик заломлен вбок и чуть
назад (`crown`). У тульи золотая лента (`GOLD`), на тулье — золотые звёздочки (`STAR`, маленькие
выпуклые шарики к зрителю). Шляпа белых — синяя, чёрных — багровая (`HAT_COLORS`). Кончик: у
ферзя — светящийся камень, у короля — золотой крестик, у слона — звезда; у коня шляпа на макушке
между ушами.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_wizard.py [папка]` — по умолчанию assets/breach/pieces:
wizard/<цвет><фигура>.png и <цвет><фигура>_shadow.png, превью preview_wizard_pieces.png (обычные
и маги) и preview_wizard_board.png (армии на доске).
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
from breach_art import GLOW, R, T, Y, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

HAT, GOLD, STAR = 5, 6, 7
HAT_COLORS = {
    "w": ["#0c1640", "#1a2e78", "#2f4fb8", "#5a7ee0", "#9db8ff"],  # синяя
    "b": ["#2a0610", "#560e22", "#8a1a34", "#c03450", "#ec7088"],  # багровая
}
GOLD_RAMP = ["#3e2604", "#7a4e0c", "#c08a1e", "#ecc04a", "#fff0a0"]
STAR_RAMP = ["#a08020", "#e0c040", "#ffe070", "#fff4b0", "#ffffff"]

# срез макушки (None — не срезать), поля: высота, радиус; тулья: радиус у основания, высота
HATS = {
    "P": (27.0, 27.0, 11.0, 6.4, 22.0),
    "B": (30.5, 30.5, 12.0, 7.6, 25.0),
    "R": (None, 37.6, 14.5, 8.6, 23.0),
    "Q": (44.0, 44.0, 12.6, 7.6, 26.0),
    "K": (49.6, 49.6, 13.0, 8.0, 25.0),
}
LEAN = norm(-R * 0.8 - T * 0.35)  # куда заломлен кончик: влево по экрану и чуть назад
FRONT = 45.0


def disc(cy, r, half, rad=0.5, centre=(0.0, 0.0)):
    """Скруглённый диск (поля шляпы)."""
    cx, cz = centre

    def f(p):
        q0 = np.hypot(p[:, 0] - cx, p[:, 2] - cz) - r + rad
        q1 = np.abs(p[:, 1] - cy) - half + rad
        return np.hypot(np.maximum(q0, 0), np.maximum(q1, 0)) + np.minimum(np.maximum(q0, q1), 0) - rad

    return f


def crown(base, r0, height, bend):
    """Тулья: шары вдоль изогнутой оси, к концу тоньше; у основания — золотая лента.
    Возвращает части и функцию точки на оси (u от 0 до 1)."""

    def axis(u):
        return base + Y * (height * u) + LEAN * (bend * height * u ** 2.4) - Y * (0.22 * height * u ** 5)

    def radius(u):
        return r0 * (1 - u) ** 0.95 + 0.35

    parts = []
    n = 46
    for i in range(n):
        u = i / (n - 1)
        parts.append((sphere(axis(u), radius(u)), GOLD if u < 0.11 else HAT))
    return parts, axis, radius


def stars(axis, radius, spots):
    """Звёздочки на тулье: (u, азимут) — выпуклые золотые точки."""
    out = []
    for u, az in spots:
        c = axis(u)
        d = ip.at(1, 0, az)
        out.append((sphere(c + d * (radius(u) - 0.15), 0.85), STAR))
    return out


def cut_above(f, y):
    return lambda p: np.maximum(f(p), p[:, 1] - y)


def wizard(kind):
    piece = art.PIECES[kind]()
    rng = np.random.default_rng(61 + "PNBRQK".index(kind))
    if kind == "N":
        return wizard_knight(piece)
    cut, brim_y, brim_r, r0, height = HATS[kind]
    body_f, body_m = piece.parts[0]
    if cut is not None:
        body_f = cut_above(body_f, cut)
    parts = [(body_f, body_m)]
    if kind == "R":
        parts += piece.parts[1:]
    # ферзь — без шаров короны, король — без креста: их место заняла шляпа
    parts.append((disc(brim_y + 0.6, brim_r, 0.75), HAT))
    cparts, axis, radius = crown(Y * (brim_y + 1.0), r0, height, 0.5)
    parts += cparts
    parts += stars(axis, radius, [(0.28, FRONT - 30 + rng.uniform(-8, 8)), (0.42, FRONT + 35),
                                  (0.6, FRONT - 5), (0.2, FRONT + 70)])
    tip = axis(1.0)
    if kind == "Q":
        parts.append((sphere(tip, 2.6), GLOW))
    elif kind == "K":
        axes = [R, Y, T]
        parts.append((art.rbox(tip + Y * 3.4, [1.4, 4.2, 1.4], axes, 0.4), GOLD))
        parts.append((art.rbox(tip + Y * 4.6, [3.6, 1.4, 1.4], axes, 0.4), GOLD))
    elif kind == "B":
        parts.append((sphere(tip, 1.8), STAR))
    return art.Piece(parts, stretch=piece.stretch)


def wizard_knight(piece):
    """Конь: шляпа на макушке, поля над глазами, кончик заломлен назад."""
    a = math.radians(18)
    fwd = norm(-R * math.cos(a) + T * math.sin(a))
    centre = fwd * 1.2
    parts = list(piece.parts)
    brim_y = 46.0
    parts.append((disc(brim_y, 9.5, 0.7, centre=(centre[0], centre[2])), HAT))
    cparts, axis, radius = crown(centre + Y * (brim_y + 0.4), 5.6, 18.0, 0.5)
    parts += cparts
    parts += stars(axis, radius, [(0.3, FRONT - 20), (0.55, FRONT + 30)])
    return art.Piece(parts, stretch=piece.stretch)


def render(kind, color):
    key = f"wizard_{color}"
    pal = dict(art.PALETTES[color])
    pal.update({HAT: HAT_COLORS[color], GOLD: GOLD_RAMP, STAR: STAR_RAMP})
    art.PALETTES[key] = pal
    return art.render_piece(wizard(kind), key, 0.86)


def preview(sprites, path):
    """Ряды: обычные и маги; в ряду белые и чёрные фигуры."""
    cw, ch = 44, 82
    sheet = Image.new("RGBA", (12 * cw + 8, 2 * ch + 8), art.hexc("#2c3a2c") + (255,))
    sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), art.hexc("#3d4a3a") + (255,)), (0, 4 + ch))
    for r in range(2):
        for c, code in enumerate(color + k for color in "wb" for k in "PNBRQK"):
            if r == 0:
                body = Image.open(os.path.join(OUT, f"{code}.png")).convert("RGBA")
            else:
                body = sprites[code][0]
            sheet.alpha_composite(body.crop((10, 0, 54, 82)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def main():
    os.makedirs(os.path.join(OUT, "wizard"), exist_ok=True)
    sprites = {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(OUT, "wizard", f"{color}{kind}.png"))
            shadow.save(os.path.join(OUT, "wizard", f"{color}{kind}_shadow.png"))
            sprites[color + kind] = (body, shadow)
            print(color + kind, flush=True)
    preview(sprites, os.path.join(OUT, "preview_wizard_pieces.png"))
    art.board_preview(sprites, scale=2, demo=False).save(os.path.join(OUT, "preview_wizard_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
