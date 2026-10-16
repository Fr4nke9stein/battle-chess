"""Курсор в стиле Into the Breach: стрелка-дротик и её варианты со значками.

Стрелка — тёмно-синяя заливка, светлая кромка в пиксель и почти чёрный контур (видна и на
песке, и на траве), жёсткая тень 30% вправо-вниз, как у фигур и карт. Варианты: наведение
(кромка цвета выделения), ход (голубой ромб-клетка), удар (красный ромб-прицел), карта
(мини-карта — взять/перетащить), нельзя (серый ромб, перечёркнутый красным).

`py tools/breach_cursor.py [папка]` — по умолчанию assets/breach/cursor: PNG 1x
(точка нажатия — `HOTSPOT`), preview_cursor.png. В игре выводить в том же целом
масштабе, что и доску.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # breach_art читает папку вывода из argv
import breach_art as art  # noqa: E402
import breach_cards as bc  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "cursor")

W, H = 22, 24  # холст курсора 1x
HOTSPOT = (1, 1)  # кончик стрелки (контур — на нулевых строке и столбце)
OUTLINE = "#020201"
SHADOW = art.SHADOW
SHADOW_SHIFT = (2, 3)

# Дротик остриём вверх-влево: правая кромка спускается наискось, спинка вогнута к оси.
DART = [
    "#..........",
    "##.........",
    "###........",
    "####.......",
    "#####......",
    "######.....",
    "#######....",
    "########...",
    "#########..",
    "##########.",
    "#######....",
    "#####......",
    "####.......",
    "##.........",
    "#..........",
]
RING = {"default": "#f4f1df", "hover": "#d9ebc8"}
FILL = ["#181c29", "#232a3b", "#2f3a50"]  # заливка, внутренний блик, свет у кромки


def bitmap(rows):
    return np.array([[ch == "#" for ch in r] for r in rows])


def place(mask, x, y, w=W, h=H):
    out = np.zeros((h, w), bool)
    out[y : y + mask.shape[0], x : x + mask.shape[1]] = mask
    return out


def paint(img, mask, color, alpha=255):
    img[mask] = (*hexc(color), alpha)


def dart(img, ring="default"):
    """Стрелка с кончиком в HOTSPOT; возвращает её маску вместе с контуром."""
    body = place(bitmap(DART), *HOTSPOT)
    edge = body & ~(
        np.roll(body, 1, 0) & np.roll(body, -1, 0) & np.roll(body, 1, 1) & np.roll(body, -1, 1)
    )
    inner = body & ~edge
    lit, _ = bc.shade_mask(inner)
    paint(img, body, RING[ring])
    paint(img, inner, FILL[0])
    paint(img, lit, FILL[2])
    # полоска блика вдоль левой кромки, как фаска на рамках карт
    spine = inner & np.roll(lit, 1, 1) & ~lit
    paint(img, spine, FILL[1])
    ring_px = bc.outline4(body)
    paint(img, ring_px, OUTLINE)
    return body | ring_px


# ---------------------------------------------------------------- значки
def badge_layer(draw, x, y):
    """Значок рисуется на отдельном слое со своим контуром; x, y — левый верх."""
    layer = np.zeros((H, W, 4), np.uint8)
    draw(layer, x, y)
    solid = layer[..., 3] > 0
    paint(layer, bc.outline4(solid), OUTLINE)
    return layer


def diamond_mask(cx, cy, hw, hh):
    yy, xx = np.mgrid[0:H, 0:W]
    return np.abs(xx + 0.5 - cx) / hw + np.abs(yy + 0.5 - cy) / hh


def draw_move(layer, x, y):
    """Голубой ромб-клетка — сюда можно сходить."""
    d = diamond_mask(x + 4.5, y + 3.5, 4.6, 3.6)
    paint(layer, d <= 1.0, "#2aa9d6")
    paint(layer, (d <= 1.0) & (np.mgrid[0:H, 0:W][0] + 0.5 < y + 3.5), "#57d9ff")
    paint(layer, d <= 0.42, "#a6f2ff")


def draw_attack(layer, x, y):
    """Красный ромб-прицел с рисками — удар по фигуре."""
    d = diamond_mask(x + 4.5, y + 4.5, 4.8, 4.8)
    paint(layer, (d <= 1.0) & (d > 0.62), "#ff4a4a")
    paint(layer, (d <= 0.25), "#ffb0a8")
    cx, cy = x + 4, y + 4
    for k in (0, 1, 7, 8):  # риски по осям, выступают за ромб
        layer[cy, x + k] = (*hexc("#c0242a"), 255)
        layer[y + k, cx] = (*hexc("#c0242a"), 255)


def draw_card(layer, x, y):
    """Мини-карта в стальной рамке с голубым огоньком — взять и перетащить карту."""
    paint(layer, place(np.ones((8, 6), bool), x, y), "#495c79")
    paint(layer, place(np.ones((6, 4), bool), x + 1, y + 1), "#181c29")
    paint(layer, place(np.ones((1, 4), bool), x + 1, y + 3), "#34425a")
    layer[y + 1, x + 1] = (*hexc("#57d9ff"), 255)
    layer[y, x : x + 6] = (*hexc("#6b80a3"), 255)
    layer[y : y + 8, x] = (*hexc("#6b80a3"), 255)


def draw_off(layer, x, y):
    """Серый ромб, перечёркнутый красным, — сюда нельзя."""
    d = diamond_mask(x + 4.5, y + 3.5, 4.6, 3.6)
    paint(layer, d <= 1.0, "#6e6a62")
    paint(layer, (d <= 1.0) & (np.mgrid[0:H, 0:W][0] + 0.5 < y + 3.5), "#8c877d")
    for k in range(9):
        yy = y + 7 - int(k * 7 / 8)
        layer[yy, x + k] = (*hexc("#ef4b3c"), 255)


BADGES = {"move": draw_move, "attack": draw_attack, "card": draw_card, "off": draw_off}
BADGE_AT = (9, 11)  # левый верх значка: у спинки стрелки, справа внизу


def cursor(kind="default"):
    """Курсор: тело и тень на одном холсте 1x (тень уже под телом)."""
    img = np.zeros((H, W, 4), np.uint8)
    solid = dart(img, ring="hover" if kind == "hover" else "default")
    if kind in BADGES:
        layer = badge_layer(BADGES[kind], *BADGE_AT)
        m = layer[..., 3] > 0
        img[m] = layer[m]
        solid |= m
    out = np.zeros_like(img)
    sx, sy = SHADOW_SHIFT
    shadow = np.zeros_like(solid)
    shadow[sy:, sx:] = solid[:-sy, :-sx]
    out[shadow] = SHADOW
    body = img[..., 3] > 0
    out[body] = img[body]
    return out


KINDS = ["default", "hover", "move", "attack", "card", "off"]


def put_cursor(canvas, kind, tip):
    x, y = tip[0] - HOTSPOT[0], tip[1] - HOTSPOT[1]
    canvas.alpha_composite(Image.fromarray(cursor(kind), "RGBA"), (int(x), int(y)))


def preview(scale=3):
    """Все курсоры на песке и на траве (середина клетки, без краёв ромба)."""
    cell = 30
    canvas = Image.new("RGBA", (len(KINDS) * cell + 12, 2 * (cell + 4) + 12), hexc("#1b1e27") + (255,))
    for row, name in enumerate(("light_0", "dark_0")):
        tile = Image.open(os.path.join(art.ROOT, "assets", "breach", "tiles", f"{name}.png")).convert("RGBA")
        patch = tile.crop((20, 15, 36, 27))
        bg = Image.new("RGBA", (cell, cell))
        for py in range(0, cell, patch.height):
            for px in range(0, cell, patch.width):
                bg.alpha_composite(patch, (px, py))
        for k, kind in enumerate(KINDS):
            x, y = 6 + k * cell, 6 + row * (cell + 4)
            canvas.alpha_composite(bg, (x, y))
            put_cursor(canvas, kind, (x + 6, y + 4))
    return canvas.resize((canvas.width * scale, canvas.height * scale), Image.NEAREST)


def table_preview(scale=2):
    """Курсоры в деле на столе из breach_cards.table_preview (координаты — его раскладка)."""
    canvas = bc.table_preview(scale=1)
    side, top = 100, 70  # как в breach_cards.table_preview
    ox, oy = side + 4 * art.TW - art.TW // 2, top

    def tile_center(file, rank):
        i, j = file, 7 - rank
        return ox + (i - j) * (art.TW // 2) + art.TW // 2, oy + (i + j) * (art.TH // 2) + art.TH // 2

    gx, gy = tile_center(6, 0)  # белый конь g1 — наведение
    put_cursor(canvas, "hover", (gx + 2, gy - 22))
    mx, my = tile_center(4, 3)  # пустая e4 — ход
    put_cursor(canvas, "move", (mx - 4, my - 4))
    ax, ay = tile_center(3, 6)  # чёрная пешка d7 — удар
    put_cursor(canvas, "attack", (ax, ay - 14))
    put_cursor(canvas, "card", (530, 392))  # над рукой
    put_cursor(canvas, "off", (50, 172))  # над колодой соперника
    put_cursor(canvas, "default", (150, 430))  # пустое место
    return canvas.resize((canvas.width * scale, canvas.height * scale), Image.NEAREST)


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for kind in KINDS:
        Image.fromarray(cursor(kind), "RGBA").save(os.path.join(OUT, f"cursor_{kind}.png"))
    preview().save(os.path.join(OUT, "preview_cursor.png"))
    table_preview().save(os.path.join(OUT, "preview_cursor_table.png"))
    print("done", OUT, "hotspot", HOTSPOT)
