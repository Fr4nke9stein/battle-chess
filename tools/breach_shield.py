"""Фигуры со щитом — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`, держат перед собой большой треугольный щит в правой руке
и смотрят на противника, к середине доски (`FACING`): белые — вверх по доске (от зрителя,
вверх-вправо по экрану), чёрные — вниз (к зрителю, вниз-влево). Поэтому у белых щит справа
впереди и к зрителю повёрнут тыльной стороной (доски `WOOD`, латунная кайма, кожаные ремни
`LEATHER`), у чёрных — слева впереди и лицом к зрителю (поле `FIELD`, кайма, светящийся ромб).
При перевёрнутой доске (игрок за чёрных) стороны смотрели бы не туда.

Щит — отдельная объёмная деталь (`shield`): треугольник с прямым верхом, бока почти прямые до
середины, дальше дугой сходятся к острию снизу (`heater`), лицевая сторона выпуклая (`BULGE`),
толщина `THICK`, развёрнут поперёк корпуса (как в стойке: плоскостью к зрителю, а не ребром) и
отклонён назад; у белых вынесен правее — иначе его закрывает корпус. `SHIELDS`: высота
центра, вынос вперёд и вправо (от фигуры, по её взгляду), высота и ширина щита — по ширине
спрайт почти не выходит за обычный (соседние фигуры не загораживает). Поле — геральдическое
(`FIELD`: у белых синее, у чёрных багровое), кайма — латунь (`BRASS`, у чёрных медь), посередине —
светящийся ромб-эмблема (`GLOW`, огонёк стороны).

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_shield.py [папка]` — по умолчанию assets/breach/pieces:
shield/<цвет><фигура>.png и <цвет><фигура>_shadow.png, превью preview_shield_pieces.png (обычные и
со щитом) и preview_shield_board.png (армии на доске).
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
from breach_art import GLOW, MAIN, R, T, Y, norm  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

BRASS, FIELD = 5, 6
FIELD_RAMP = {  # поле щита — геральдический цвет: в цвете стороны щит сливался с фигурой
    "w": ["#0c1640", "#1a3080", "#2f52c0", "#5a82e8", "#a2c0ff"],  # синее
    "b": ["#2a0610", "#560e20", "#8c1a30", "#c0344a", "#ec7a86"],  # багровое
}
BRASS_RAMP = {
    "w": ["#3a2408", "#74501a", "#b48434", "#e0b860", "#fff0b0"],
    "b": ["#2e1006", "#5e2814", "#94482a", "#c87a4e", "#f0b48a"],  # у чёрных — медь
}
WOOD, WOOD_DARK, LEATHER = 7, 8, 9
BACK = {
    WOOD: ["#2e1a0c", "#5a3418", "#86542a", "#ae7a44", "#d4a46a"],
    WOOD_DARK: ["#24140a", "#462814", "#6a4220", "#8c6034", "#ae8250"],
    LEATHER: ["#140a06", "#2a160c", "#442616", "#5e3a22", "#7a5034"],
}
FACING = {"w": norm(R - T), "b": norm(T - R)}  # куда смотрит фигура: к середине доски
BULGE = 1.1  # насколько выпукла середина щита
THICK = 1.0
RIM = 1.1  # ширина латунной каймы
# высота центра щита, вынос вперёд и вправо (по взгляду фигуры), высота и ширина щита
SHIELDS = {
    "P": (12.5, 9.0, 3.0, 21.0, 15.0),
    "N": (16.5, 10.4, 3.0, 23.0, 16.5),
    "B": (15.5, 9.4, 3.2, 23.0, 16.5),
    "R": (16.5, 12.0, 3.0, 23.5, 17.0),
    "Q": (16.5, 10.0, 3.2, 24.0, 17.0),
    "K": (17.5, 10.2, 3.4, 24.5, 17.0),
}


def heater(a, b, height, width):
    """Треугольный щит в плоскости (a — вбок, b — вверх от центра): прямой верх, бока почти
    прямые до середины, ниже дугой к острию. Знаковое расстояние (приближённо)."""
    v = (height / 2 - b) / height  # 0 — верх, 1 — остриё
    half = width / 2 * np.where(v < 0.42, 1 - 0.06 * v, (1 - 0.06 * 0.42) * np.clip(1 - ((v - 0.42) / 0.58) ** 1.7, 0, 1) ** 0.6)
    side = np.abs(a) - half
    top = b - height / 2
    bottom = -b - height / 2
    return np.maximum.reduce([side, top, bottom])


def shield(centre, height, width, n):
    """Щит лицом в сторону n: выпуклая пластина; спереди — поле, кайма, светящийся ромб; сзади —
    доски, кайма, два кожаных ремня."""
    up = norm(Y - n * 0.18)  # чуть отклонён назад
    side = norm(np.cross(up, n))
    c = np.asarray(centre, float)

    def f(p):
        d = p - c
        a, b, w = d @ side, d @ up, d @ n
        bulge = BULGE * (1 - np.clip((a / (width / 2)) ** 2, 0, 1)) * (1 - np.clip(((b + height * 0.1) / height) ** 2, 0, 1))
        plate = np.abs(w - bulge + THICK / 2) - THICK / 2
        return np.maximum(heater(a, b, height, width), plate) - 0.15

    def mat(p):
        d = p - c
        a, b, w = d @ side, d @ up, d @ n
        bulge = BULGE * (1 - np.clip((a / (width / 2)) ** 2, 0, 1)) * (1 - np.clip(((b + height * 0.1) / height) ** 2, 0, 1))
        front = w > bulge - THICK / 2
        inner = heater(a, b, height - 2 * RIM, width - 2 * RIM) < 0
        out = np.full(len(p), BRASS)
        out[inner & front] = FIELD
        gem = np.abs(a) / 2.2 + np.abs(b - height * 0.05) / 3.0 < 1
        out[gem & inner & front] = GLOW
        back = inner & ~front
        out[back] = np.where((np.floor(a[back] / 2.4) % 2) == 0, WOOD, WOOD_DARK)
        straps = back & (np.abs(a) < width * 0.34) & ((np.abs(b - height * 0.16) < 0.9) | (np.abs(b + height * 0.1) < 0.9))
        out[straps] = LEATHER
        return out

    return f, mat


def with_shield(kind, color):
    piece = art.PIECES[kind]()
    y, ahead, right, height, width = SHIELDS[kind]
    look = FACING[color]
    hand = norm(np.cross(look, Y))  # правая рука фигуры, смотрящей в сторону look
    if color == "w":  # белые смотрят от зрителя: щит перед ними закрыт корпусом — выносим вправо
        ahead, right = ahead * 0.3, right + 5.6
    centre = look * ahead + hand * right + Y * y
    n = norm(look - hand * 0.45)  # щит развёрнут поперёк корпуса — плоскостью к зрителю, а не ребром
    return art.Piece(list(piece.parts) + [shield(centre, height, width, n)], stretch=piece.stretch)


def render(kind, color):
    key = f"shield_{color}"
    pal = dict(art.PALETTES[color])
    pal[BRASS] = BRASS_RAMP[color]
    pal[FIELD] = FIELD_RAMP[color]
    pal.update(BACK)
    art.PALETTES[key] = pal
    return art.render_piece(with_shield(kind, color), key, 0.86)


def preview(sprites, path):
    """Ряды: обычные и со щитом; в ряду белые и чёрные фигуры."""
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
    os.makedirs(os.path.join(OUT, "shield"), exist_ok=True)
    sprites = {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(OUT, "shield", f"{color}{kind}.png"))
            shadow.save(os.path.join(OUT, "shield", f"{color}{kind}_shadow.png"))
            sprites[color + kind] = (body, shadow)
            print(color + kind, flush=True)
    preview(sprites, os.path.join(OUT, "preview_shield_pieces.png"))
    art.board_preview(sprites, scale=2, demo=False).save(os.path.join(OUT, "preview_shield_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
