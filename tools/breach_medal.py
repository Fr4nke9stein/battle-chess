"""Медали rogue-like (0.126, просьба пользователя: сундук медалей — вид фигур на выбор, выбранный
до конца забега получает мульти +1) в стиле игры: пиксель-арт Into the Breach, свет сверху слева,
тёмный контур, как у монеты и гербов.

Медаль — золотой диск на красной ленте с белой полосой, на лице выбит золотой знак фигуры (те же
знаки по пикселям, что на гербах, — `breach_crest.PAWN`, `KNIGHT`, `BISHOP`, `ROOK`, `QUEEN_SMALL`,
`KING_SMALL`). Картинка 32×44 (медаль 30×42 и контур), тени нет:
`ui/medal_<p|n|b|r|q|k>.png`, превью — `ui/preview_medals.png` (×6).

    py tools/breach_medal.py [папка assets/breach/ui]
"""

import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
import breach_crest as crest  # noqa: E402


def hexc(s):
    s = s.lstrip("#")
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "assets", "breach", "ui")
OUTLINE = hexc("#020201")
GOLD = [hexc(c) for c in ["#fffbe0", "#ffe27a", "#f5b638", "#d07d1c", "#8a4510"]]
RED = [hexc(c) for c in ["#ff8a7a", "#e8402e", "#b0201a", "#701010"]]
CREAM = [hexc(c) for c in ["#fff8e6", "#e8dcc0", "#c2b494"]]

W, H = 30, 42
CX, CY, R = 15.0, 27.5, 13.5  # середина и радиус диска
SIGNS = {
    "p": crest.PAWN,
    "n": crest.KNIGHT,
    "b": crest.BISHOP,
    "r": crest.ROOK,
    "q": crest.QUEEN_SMALL,
    "k": crest.KING_SMALL,
}


def medal(sign):
    img = np.zeros((H, W, 4), np.uint8)

    def put(x, y, col):
        if 0 <= x < W and 0 <= y < H:
            img[y, x] = (*col, 255)

    # лента: от верха к диску сужается, три полосы — красная, белая, красная
    for y in range(0, 17):
        half = 7 - y // 6
        for x in range(int(CX) - half, int(CX) + half):
            k = x - (int(CX) - half)
            width = 2 * half
            if width // 3 <= k < width - width // 3:
                col = CREAM[0] if k < width // 2 else CREAM[1]
            else:
                col = RED[1] if k < width // 2 else RED[2]
            if k == 0:
                col = RED[0] if col in (RED[1], RED[2]) else col
            if k == width - 1:
                col = RED[3]
            put(x, y, col)
    # складка ленты над диском
    for x in range(int(CX) - 5, int(CX) + 5):
        put(x, 13, RED[3])
    # диск: обод со светом сверху слева, желобок, лицо темнее знака
    for y in range(H):
        for x in range(W):
            dx, dy = x + 0.5 - CX, y + 0.5 - CY
            d = (dx * dx + dy * dy) ** 0.5
            if d > R:
                continue
            light = -(dx + dy) / max(d, 0.001)
            if d > R - 2:
                col = GOLD[1] if light > 0.35 else GOLD[2] if light > -0.35 else GOLD[3]
                if d > R - 2 and light > 0.8:
                    col = GOLD[0]
            elif d > R - 3:
                col = GOLD[4]
            else:
                col = GOLD[3] if light < 0.2 else hexc("#e09a2a")
            put(x, y, col)
    # знак фигуры, выбитый на лице: тень вправо вниз, светлый верх и левый край
    rows = SIGNS[sign]
    ph, pw = len(rows), len(rows[0])
    ox, oy = round(CX - pw / 2), round(CY - ph / 2)
    cells = {(i, j) for j, line in enumerate(rows) for i, ch in enumerate(line) if ch == "#"}
    for i, j in cells:
        if (i + 1, j + 1) not in cells:
            put(ox + i + 1, oy + j + 1, GOLD[4])
    for i, j in cells:
        top = (i, j - 1) not in cells or (i - 1, j) not in cells
        bottom = (i + 1, j) not in cells or (i, j + 1) not in cells
        col = GOLD[0] if top else GOLD[2] if bottom else GOLD[1]
        put(ox + i, oy + j, col)
    # блик на ободе
    put(int(CX - R * 0.55), int(CY - R * 0.6), hexc("#ffffff"))
    # контур
    out = np.zeros((H + 2, W + 2, 4), np.uint8)
    out[1:-1, 1:-1] = img
    alpha = out[:, :, 3] > 0
    grown = alpha.copy()
    grown[1:, :] |= alpha[:-1, :]
    grown[:-1, :] |= alpha[1:, :]
    grown[:, 1:] |= alpha[:, :-1]
    grown[:, :-1] |= alpha[:, 1:]
    out[grown & ~alpha] = (*OUTLINE, 255)
    return out


def main():
    os.makedirs(OUT, exist_ok=True)
    images = []
    for sign in SIGNS:
        img = medal(sign)
        Image.fromarray(img, "RGBA").save(os.path.join(OUT, f"medal_{sign}.png"))
        images.append(img)
    h, w = images[0].shape[:2]
    sheet = Image.new("RGBA", (len(images) * (w + 4) + 4, h + 8), (24, 28, 41, 255))
    for k, img in enumerate(images):
        sheet.alpha_composite(Image.fromarray(img, "RGBA"), (4 + k * (w + 4), 4))
    sheet.resize((sheet.width * 6, sheet.height * 6), Image.NEAREST).save(
        os.path.join(OUT, "preview_medals.png"))


if __name__ == "__main__":
    main()
