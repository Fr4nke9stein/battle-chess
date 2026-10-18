"""Иконка золотой монеты под углом — объёмная, в стиле Into the Breach.

Монета повёрнута вокруг вертикали: лицо — вытянутый эллипс, справа видно толстое ребро с
насечкой. Всё собрано из чистых форм в сетке пикселей: лицо — эллипс, ребро — тот же
эллипс, сдвинутый вправо (заметённая область минус лицо), бортик — кольцо у края лица
(свет сверху-слева: верх-лево бортика светлый, низ-право — в тени, внутренняя стенка —
наоборот), поле с бликом-полосой, чеканный король (выпуклая: светлая кромка сверху-слева,
тень снизу-справа), блик-звёздочка. Шесть тонов золота, контур #020201.

`py tools/breach_coin.py [папка]` — по умолчанию assets/breach/ui: coin_<размер>.png
(16, 24, 32 пикселя) и preview_coin.png (на тёмном фоне и на табличке, крупно).
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "ui")

# от блика к тени
GOLD = [hexc(c) for c in ["#fffbe0", "#ffe27a", "#f5b638", "#d07d1c", "#8a4510", "#4a1f06"]]
OUTLINE = hexc("#020201")
SQUASH = 0.7  # ширина лица к высоте: насколько монета повёрнута
THICK = 0.17  # видимая толщина ребра в долях размера


def shifted(m, dy, dx):
    """Маска, сдвинутая на (dy, dx) без заворота через край."""
    out = np.zeros_like(m)
    h, w = m.shape
    out[max(dy, 0) : h + min(dy, 0), max(dx, 0) : w + min(dx, 0)] = m[max(-dy, 0) : h - max(dy, 0),
                                                                      max(-dx, 0) : w - max(dx, 0)]
    return out


# Король для чеканки, нарисован по пикселям под размер монеты: крест, корона, воротник,
# тело, основание.
KING = {
    32: [
        "....X....",
        "...XXX...",
        "....X....",
        "..XXXXX..",
        ".XXXXXXX.",
        ".XXXXXXX.",
        "..XXXXX..",
        ".XXXXXXX.",
        "...XXX...",
        "...XXX...",
        "..XXXXX..",
        "..XXXXX..",
        ".XXXXXXX.",
        "XXXXXXXXX",
        "XXXXXXXXX",
    ],
    24: [
        "...X...",
        "..XXX..",
        "...X...",
        ".XXXXX.",
        ".XXXXX.",
        "..XXX..",
        ".XXXXX.",
        "..XXX..",
        "..XXX..",
        ".XXXXX.",
        "XXXXXXX",
    ],
}


def king_mask(size, cx, cy):
    """Маска короля size×size с центром около (cx, cy)."""
    grid = np.array([[ch == "X" for ch in row] for row in KING[size]])
    h, w = grid.shape
    m = np.zeros((size, size), bool)
    x0, y0 = int(round(cx - w / 2)), int(round(cy - h / 2))
    m[y0 : y0 + h, x0 : x0 + w] = grid
    return m


def render(size, emblem=True):
    """RGBA монеты size×size."""
    t = max(2, round(size * THICK))
    ry = size / 2 - 1.5
    rx = ry * SQUASH
    cx = 1.5 + rx  # лицо — слева, ребро — правее на t
    cy = size / 2
    yy, xx = np.mgrid[0:size, 0:size]
    px, py = xx + 0.5, yy + 0.5

    def ell(ox):
        return np.hypot((px - ox) / rx, (py - cy) / ry)

    e = ell(cx)
    face = e <= 1
    side = np.zeros_like(face)
    for k in np.arange(0.5, t + 0.01, 0.5):
        side |= ell(cx + k) <= 1
    side &= ~face
    u, v = (px - cx) / rx, (py - cy) / ry
    tl = u + v  # < 0 — к свету (верх-лево)
    img = np.zeros((size, size, 4), np.uint8)

    def put(mask, tone):
        img[mask] = (*GOLD[tone], 255)

    # ребро: сверху светлее, насечка — каждая вторая строка темнее, шов у лица — тёмный
    put(side, 3)
    put(side & (v > 0.35), 4)
    put(side & (yy % 2 == 0), 4)
    put(side & (yy % 2 == 0) & (v > 0.35), 5)
    put(side & (v < -0.55) & (yy % 2 == 1), 2)  # верх ребра ловит свет
    seam = side & shifted(face, 0, 1)
    put(seam, 5)
    # лицо: поле, блик-полоса наискосок, бортик с внутренней стенкой
    rim_w = 1.6 / ry * (size / 24) ** 0.3  # ширина бортика — около 1.5–2 пикселей
    put(face, 2)
    put(face & (tl < -0.55) & (tl > -0.95), 1)  # широкая полоса света
    put(face & (tl > 0.7), 3)  # низ-право поля темнее
    rim = face & (e > 1 - rim_w)
    inner = face & (e > 1 - 2 * rim_w) & ~rim
    put(rim, 2)
    put(rim & (tl < -0.2), 1)
    put(rim & (tl > 0.5), 3)
    put(inner & (tl < -0.1), 3)  # стенка бортика вверху-слева — в тени
    put(inner & (tl > 0.6), 1)  # внизу-справа — на свету
    # чеканный король
    if emblem:
        king = king_mask(size, cx, cy + 0.4) & face
        around = (shifted(king, 1, 0) | shifted(king, -1, 0) | shifted(king, 0, 1) | shifted(king, 0, -1))
        put(around & ~king & face & ~rim, 3)  # кромка — отделяет короля от блика на поле
        put(shifted(king, 1, 1) & ~king & face & ~rim, 4)  # тень снизу-справа
        put(king, 1)
        put(king & ~shifted(king, 0, 1), 0)  # левая кромка на свету
    else:  # мелкая монета: вместо короля — выпуклая точка
        dot = face & (np.hypot(u / 0.28, v / 0.22) <= 1)
        put(dot, 1)
        put(shifted(dot, 1, 1) & ~dot & face, 3)
    # блик на бортике сверху-слева
    glint = rim & (tl < -1.05) & (np.abs(u - v * 0.4) < 0.5)
    put(glint, 0)
    # контур
    solid = face | side
    ring = (shifted(solid, 1, 0) | shifted(solid, -1, 0) | shifted(solid, 0, 1) | shifted(solid, 0, -1)) & ~solid
    img[ring] = (*OUTLINE, 255)
    return img


def sparkle(img, x, y, r):
    """Звёздочка блика: крест из светлых пикселей с центром (x, y)."""
    h, w = img.shape[:2]
    for k in range(-r, r + 1):
        for xx, yy in ((x + k, y), (x, y + k)):
            # только по самой монете: контур не трогаем
            if 0 <= xx < w and 0 <= yy < h and img[yy, xx, 3] and tuple(img[yy, xx, :3]) != OUTLINE:
                img[yy, xx] = (*GOLD[0 if abs(k) < r else 1], 255)


def preview(icons, path):
    """Крупно на тёмном фоне доски и на табличке, как в игре."""
    zoom = 6
    gap = 10
    w = sum(i.shape[1] for i in icons) + gap * (len(icons) + 1)
    h = max(i.shape[0] for i in icons) + 2 * gap
    sheet = Image.new("RGBA", (w, h * 2), hexc("#151821") + (255,))
    plate = Image.new("RGBA", (w - 2 * gap + 8, h - 8), hexc("#2b2f3b") + (255,))
    sheet.alpha_composite(plate, (gap - 4, h + 4))
    x = gap
    for icon in icons:
        im = Image.fromarray(icon, "RGBA")
        for row in (0, 1):
            sheet.alpha_composite(im, (x, row * h + (h - icon.shape[0]) // 2))
        x += icon.shape[1] + gap
    sheet.resize((sheet.width * zoom, sheet.height * zoom), Image.NEAREST).save(path)


def main():
    os.makedirs(OUT, exist_ok=True)
    icons = []
    for size in (16, 24, 32):
        img = render(size, emblem=size >= 24)
        if size >= 24:
            sparkle(img, round(size * 0.16), round(size * 0.19), r=1 if size == 24 else 2)
        Image.fromarray(img, "RGBA").save(os.path.join(OUT, f"coin_{size}.png"))
        icons.append(img)
    preview(icons, os.path.join(OUT, "preview_coin.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
