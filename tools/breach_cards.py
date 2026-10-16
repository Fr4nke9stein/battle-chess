"""Карты в стиле Into the Breach: пустое лицо карты, рубашка, колода-стопка и веер в руке.

Три типа лиц: обычные карты (по сторонам, кристалл в углу), карты жертвы (чёрные с
красным, череп) и карты победы (белые с желтизной и багровым, лавровый венок).
Рубашки у всех типов общие — по сторонам.

Рамки — как панели ITB: тёмно-синяя заливка, кромка 1px со светлым верхом-левом и тёмным
низом-правом, выбранная карта — бледно-зелёная кромка. Рубашка — металлическая пластина
с распорками и ромбом-доской. У белых сталь и голубые огни, у чёрных хитин и оранжевые.
Повёрнутые карты в веере — RotSprite (scale2x ×3, поворот, выборка), без размытия.

`py tools/breach_cards.py [папка]` — по умолчанию пишет в assets/breach/cards.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import breach_art as art  # noqa: E402
from breach_art import hexc  # noqa: E402

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "cards")

CW, CH = 50, 70  # карта 1x, пропорции игральной карты
CHAMFER = 2

FACTIONS = {
    "w": {
        # кромка: контур, тёмная, средняя, светлая, блик
        "frame": ["#0d0f17", "#34425a", "#495c79", "#6b80a3", "#9fb3d1"],
        "fill": "#22222a",
        "head": "#181c29",
        "window": ["#11141f", "#161a27", "#1c2130"],
        "text": "#2a2f3d",
        "accent": art.PALETTES["w"][art.GLOW],
        "plate": ["#151a26", "#1f2636", "#283146", "#343f58", "#4a5a7a"],
    },
    "b": {
        "frame": ["#0d0a12", "#3a2418", "#643f29", "#94613f", "#c8975a"],
        "fill": "#221d2c",
        "head": "#17131f",
        "window": ["#0f0c16", "#15111e", "#1c1727"],
        "text": "#2c2539",
        "accent": art.PALETTES["b"][art.GLOW],
        "plate": ["#120e1b", "#1c172a", "#262038", "#332b4a", "#4a4068"],
    },
}
SELECT = ["#0d0f17", "#7f9472", "#b4c9a3", "#d9ebc8", "#f4fbe9"]
SHADOW = art.SHADOW


# ---------------------------------------------------------------- геометрия карты
def rings(w=CW, h=CH, c=CHAMFER):
    """Номер кольца от края (0 — контур, <0 — снаружи) и сторона: 1 светлая, -1 тёмная, 0 нейтр."""
    y, x = np.mgrid[0:h, 0:w]
    terms = {
        "l": x,
        "t": y,
        "r": w - 1 - x,
        "b": h - 1 - y,
        "tl": x + y - c,
        "tr": (w - 1 - x) + y - c,
        "bl": x + (h - 1 - y) - c,
        "br": (w - 1 - x) + (h - 1 - y) - c,
    }
    names = list(terms)
    stack = np.stack([terms[n] for n in names])
    d = stack.min(axis=0)
    which = stack.argmin(axis=0)
    side = np.zeros((h, w), int)
    for k, n in enumerate(names):
        if n in ("l", "t", "tl"):
            side[which == k] = 1
        elif n in ("r", "b", "br"):
            side[which == k] = -1
    return d, side


def paint(img, mask, color):
    img[mask] = (*hexc(color), 255)


def bevel_frame(img, d, side, frame):
    """Контур, двухпиксельная кромка с фаской и тёмная линия внутри."""
    paint(img, d == 0, frame[0])
    paint(img, (d == 1) & (side == 1), frame[3])
    paint(img, (d == 1) & (side == 0), frame[2])
    paint(img, (d == 1) & (side == -1), frame[1])
    paint(img, d == 2, frame[2])
    paint(img, d == 3, frame[0])


def diamond(cx, cy, hw, hh, w=CW, h=CH):
    y, x = np.mgrid[0:h, 0:w]
    return np.abs(x + 0.5 - cx) / hw + np.abs(y + 0.5 - cy) / hh


# ---------------------------------------------------------------- типы карт
def _normal(f):
    p = FACTIONS[f]
    fr = p["frame"]
    return dict(p, recess=(fr[0], fr[2]), text_recess=(fr[0], fr[1]), divider=(fr[2], fr[0]), emblem="gem")


# Обычные карты — по сторонам (сталь и голубое у белых, хитин и оранжевое у чёрных).
# Карты жертвы — чёрные с красным, вместо кристалла череп. Карты победы — белые с
# желтизной и багровым, вместо кристалла багровый лавровый венок.
CARD_TYPES = {
    "w": _normal("w"),
    "b": _normal("b"),
    "sacrifice": {
        "frame": ["#080405", "#2e0a0d", "#5e1218", "#8e1e26", "#c4343e"],
        "fill": "#141012",
        "head": "#0b0809",
        "window": ["#0c0809", "#120c0d", "#1a1012"],
        "text": "#1e1517",
        "accent": ["#3a0a0c", "#7a1418", "#c0242a", "#ff4a4a", "#ffb0a8"],
        "recess": ("#030202", "#5e1218"),
        "text_recess": ("#030202", "#2e0a0d"),
        "divider": ("#8e1e26", "#030202"),
        "emblem": "skull",
    },
    "victory": {
        "frame": ["#24060c", "#5c0c18", "#8a1426", "#b8283c", "#e0566a"],
        "fill": "#efe6c6",
        "head": "#e3d6ab",
        "window": ["#e9e0c3", "#e1d6b3", "#d8cba4"],
        "text": "#f8f2de",
        "accent": ["#4a0810", "#7a1020", "#a81a30", "#e04858", "#ffd0c8"],
        "recess": ("#a8956a", "#fffbec"),
        "text_recess": ("#a8956a", "#fffbec"),
        "divider": ("#8a1426", "#c9b98c"),
        "emblem": "wreath",
    },
}
CARD_KINDS = list(CARD_TYPES)


def outline4(mask):
    """Кольцо в 1px вокруг маски (4-соседство)."""
    p = np.pad(mask, 1)
    ring = p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:]
    return ring & ~mask


def close8(mask):
    """Морфологическое закрытие 3×3: заполняет щели шириной в пиксель."""
    def grow(m):
        p = np.pad(m, 1)
        return np.max(np.stack([p[1 + dy : p.shape[0] - 1 + dy, 1 + dx : p.shape[1] - 1 + dx]
                                for dy in (-1, 0, 1) for dx in (-1, 0, 1)]), axis=0)

    return ~grow(~grow(mask))


def shade_mask(mask):
    """Свет сверху-слева: край без соседа сверху/слева — светлый, снизу/справа — тёмный."""
    p = np.pad(mask, 1)
    up, left = p[:-2, 1:-1], p[1:-1, :-2]
    down, right = p[2:, 1:-1], p[1:-1, 2:]
    light = mask & (~up | ~left)
    dark = mask & (~down | ~right) & ~light
    return light, dark


def emblem_gem(img, t, frame):
    """Кристалл обычной карты — ромб маны, светится цветом стороны. Крупный и вытянутый
    вверх (12×14, заходит на рамку, как значок): на нём белой цифрой с контуром пишется
    цена в мане (тёмная сливалась с кромкой ромба)."""
    acc = t["accent"]
    y, x = np.mgrid[0:CH, 0:CW]
    dm = diamond(8.0, 8.0, 6.0, 7.5)
    paint(img, dm <= 1.0, frame[0])
    paint(img, dm <= 0.78, acc[3])
    paint(img, dm <= 0.56, acc[1])
    paint(img, (dm <= 0.56) & (y < 8), acc[2])
    img[5, 7] = (*hexc(acc[4]), 255)


SKULL = [
    "..#####..",
    ".#######.",
    "#########",
    "#########",
    "#ee###ee#",
    "#ee###ee#",
    "####n####",
    ".###n###.",
    "..#####..",
    "..#t#t#..",
]
BONE = ["#f2ece0", "#d6ccbb", "#a39684"]  # свет, середина, тень


def emblem_skull(img, t, frame, x0=3, y0=3):
    """Череп карты жертвы: кость с тенью, глаза светятся красным, чёрный контур."""
    acc = t["accent"]
    h, w = len(SKULL), len(SKULL[0])
    grid = np.array([list(r) for r in SKULL])
    bone = grid == "#"
    holes = (grid == "e") | (grid == "n") | (grid == "t")
    shape = bone | holes
    lit, shaded = shade_mask(bone)
    sub = img[y0 : y0 + h, x0 : x0 + w]
    big = np.zeros((CH, CW), bool)
    big[y0 : y0 + h, x0 : x0 + w] = shape
    paint(img, outline4(big), frame[0])
    sub[bone] = (*hexc(BONE[1]), 255)
    sub[shaded] = (*hexc(BONE[2]), 255)
    sub[lit] = (*hexc(BONE[0]), 255)
    sub[(grid == "n") | (grid == "t")] = (*hexc("#1a0a0c"), 255)
    sub[grid == "e"] = (*hexc(acc[1]), 255)
    # верхний левый пиксель каждого глаза — яркий огонёк
    for ey, ex in ((4, 1), (4, 6)):
        sub[ey, ex] = (*hexc(acc[3]), 255)
        sub[ey + 1, ex + 1] = (*hexc(acc[0]), 255)


# Знак победы — багровый лавровый венок. Ветвь нарисована по пикселям прямой
# (стебель s, у листа кончик h, тело m и основание d; внутренние листья в пиксель) и
# изогнута сдвигом строк к оси; правая ветвь — зеркало, внизу ветви связывает узелок k.
LAUREL_BRANCH = [
    "..h..",
    ".hmd.",
    "h.s..",
    "mmsm.",
    ".ds..",
    "h.s..",
    "mmsm.",
    ".ds..",
    "..s..",
]
LAUREL_SHIFT = [2, 1, 1, 0, 0, 0, 1, 1, 2]  # сдвиг строк к оси: изгиб ветви
WREATH_W, WREATH_H = 13, 10  # последняя строка — узелок
WREATH_COLORS = {
    False: {"h": "#e86070", "m": "#b4223a", "d": "#7a1020", "s": "#5c0c18"},
    True: {"h": "#c4364a", "m": "#9a1a2e", "d": "#6a0e1c", "s": "#4e0812"},
}


def wreath_grid():
    """Сетка венка: символ роли пикселя и признак правой (затенённой) ветви."""
    w, h = WREATH_W, WREATH_H
    grid = np.full((h, w), ".", dtype="<U1")
    right = np.zeros((h, w), bool)
    for r, (row, shift) in enumerate(zip(LAUREL_BRANCH, LAUREL_SHIFT)):
        for c, ch in enumerate(row):
            if ch == ".":
                continue
            x = shift + c
            grid[r, x] = ch
            grid[r, w - 1 - x] = ch
            right[r, w - 1 - x] = True
    # узелок: связывает стебли внизу
    grid[h - 1, w // 2 - 2 : w // 2 + 3] = "k"
    grid[h - 2, w // 2 - 1 : w // 2 + 2] = "k"
    return grid, right


def fill_holes(mask):
    """Маска вместе с замкнутыми дырами: всё, что не достижимо снаружи."""
    outside = np.zeros_like(mask)
    outside[0, :], outside[-1, :], outside[:, 0], outside[:, -1] = True, True, True, True
    outside &= ~mask
    while True:
        p = np.pad(outside, 1)
        grown = (p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:] | outside) & ~mask
        if (grown == outside).all():
            return ~outside
        outside = grown


def emblem_wreath(img, t, frame, x0=3, y0=3):
    """Багровый лавровый венок — знак карты победы."""
    grid, right = wreath_grid()
    h, w = grid.shape
    leaves = grid != "."
    solid = fill_holes(close8(np.pad(leaves, 1)))[1:-1, 1:-1]
    big = np.zeros((CH, CW), bool)
    big[y0 : y0 + h, x0 : x0 + w] = solid
    # контур — только по внешнему силуэту: щели между листьями и середина венка светлые
    paint(img, outline4(big), frame[0])
    sub = img[y0 : y0 + h, x0 : x0 + w]
    for side in (False, True):
        for ch, color in WREATH_COLORS[side].items():
            sub[(grid == ch) & (right == side)] = (*hexc(color), 255)
    sub[grid == "k"] = (*hexc("#4e0812"), 255)
    sub[h - 2, w // 2] = (*hexc("#b4223a"), 255)


EMBLEMS = {"gem": emblem_gem, "skull": emblem_skull, "wreath": emblem_wreath}


def card_front(kind, selected=False):
    """Лицо карты: `kind` — "w"/"b" (обычная карта стороны), "sacrifice" или "victory"."""
    t = CARD_TYPES[kind]
    frame = SELECT if selected else t["frame"]
    acc = t["accent"]
    img = np.zeros((CH, CW, 4), np.uint8)
    d, side = rings()
    paint(img, d >= 4, t["fill"])
    bevel_frame(img, d, side, frame)
    y, x = np.mgrid[0:CH, 0:CW]
    inner = d >= 4

    # шапка под название и разделитель
    head = inner & (y >= 4) & (y <= 12)
    paint(img, head, t["head"])
    paint(img, inner & (y == 13), t["divider"][0])
    paint(img, inner & (y == 14), t["divider"][1])

    # окно под картинку: утоплено (тень сверху-слева, свет снизу-справа), градиент
    x0, x1, y0, y1 = 6, CW - 7, 17, 41
    win = (x >= x0) & (x <= x1) & (y >= y0) & (y <= y1)
    # ступенчатый градиент: три полосы сверху вниз
    bands_y = [y0 + (y1 - y0 + 1) * k // 3 for k in range(4)]
    for k, color in enumerate(t["window"]):
        paint(img, win & (y >= bands_y[k]) & (y < bands_y[k + 1]), color)
    shadow_c, light_c = t["recess"]
    paint(img, ((x == x0 - 1) & (y >= y0 - 1) & (y <= y1)) | ((y == y0 - 1) & (x >= x0 - 1) & (x <= x1)), shadow_c)
    paint(img, ((x == x1 + 1) & (y >= y0) & (y <= y1 + 1)) | ((y == y1 + 1) & (x >= x0) & (x <= x1 + 1)), light_c)
    # уголки-скобки, как у прицелов ITB
    bracket = hexc(SELECT[2] if selected else frame[3])
    for cx, cy, sx, sy in ((x0 + 1, y0 + 1, 1, 1), (x1 - 1, y0 + 1, -1, 1), (x0 + 1, y1 - 1, 1, -1), (x1 - 1, y1 - 1, -1, -1)):
        for k in range(3):
            img[cy, cx + sx * k] = (*bracket, 255)
            img[cy + sy * k, cx] = (*bracket, 255)

    # поле под текст
    tx0, tx1, ty0, ty1 = 6, CW - 7, 45, 61
    tb = (x >= tx0) & (x <= tx1) & (y >= ty0) & (y <= ty1)
    paint(img, tb, t["text"])
    shadow_c, light_c = t["text_recess"]
    paint(img, ((y == ty0 - 1) & (x >= tx0 - 1) & (x <= tx1 + 1)) | ((x == tx0 - 1) & (y >= ty0) & (y <= ty1)), shadow_c)
    paint(img, ((y == ty1 + 1) & (x >= tx0 - 1) & (x <= tx1 + 1)) | ((x == tx1 + 1) & (y >= ty0) & (y <= ty1)), light_c)

    # нижний язычок с тремя огоньками
    tab = (y >= CH - 6) & (y <= CH - 3) & (np.abs(x + 0.5 - CW / 2) <= 9 - (y - (CH - 6)))
    paint(img, tab, t["head"])
    edge = tab & ~(np.roll(tab, 1, 1) & np.roll(tab, -1, 1) & np.roll(tab, 1, 0))
    paint(img, edge, frame[2])
    for k in (-4, 0, 4):
        px = CW // 2 + k - 1
        img[CH - 5, px : px + 2] = (*hexc(acc[3]), 255)
        img[CH - 4, px : px + 2] = (*hexc(acc[1]), 255)

    # знак типа в левом верхнем углу
    EMBLEMS[t["emblem"]](img, t, frame)
    return img


def card_back(f):
    pal = FACTIONS[f]
    p = pal["plate"]
    acc = pal["accent"]
    img = np.zeros((CH, CW, 4), np.uint8)
    d, side = rings()
    y, x = np.mgrid[0:CH, 0:CW]
    paint(img, d >= 4, p[2])
    bevel_frame(img, d, side, pal["frame"])
    # утопленная панель
    paint(img, (d == 5) & (side >= 0), p[0])
    paint(img, (d == 5) & (side < 0), p[3])
    paint(img, d >= 6, p[1])
    # диагональные распорки, как на дверях капсул
    ix0, iy0, ix1, iy1 = 6, 6, CW - 7, CH - 7
    inside = d >= 6
    for sgn in (1, -1):
        # прямая через углы внутренней панели
        ax, ay = (ix0, iy0) if sgn == 1 else (ix1, iy0)
        bx, by = (ix1, iy1) if sgn == 1 else (ix0, iy1)
        nx, ny = by - ay, -(bx - ax)
        nl = math.hypot(nx, ny)
        dist = ((x + 0.5 - ax) * nx + (y + 0.5 - ay) * ny) / nl
        band = inside & (np.abs(dist) <= 2.6)
        paint(img, band, p[3])
        paint(img, inside & (np.abs(dist) <= 2.6) & (np.abs(dist) > 1.6) & (dist * sgn > 0), p[4])
        paint(img, inside & (np.abs(dist) > 2.6) & (np.abs(dist) <= 3.5), p[0])
    # ромб-доска в центре: изометрическая клетчатая плашка в цветах клеток доски
    cx, cy = CW / 2, CH / 2
    hw, hh = 16.0, 12.0
    dm = diamond(cx, cy, hw, hh)
    paint(img, dm <= 1.0 + 2.6 / hh, pal["frame"][0])
    paint(img, dm <= 1.0 + 1.3 / hh, acc[2])
    paint(img, dm <= 1.0, pal["frame"][0])
    core = dm <= 1.0 - 1.0 / hh
    cells = 3
    u = ((x + 0.5 - cx) / hw + (y + 0.5 - cy) / hh + 1) / 2 * cells
    v = ((y + 0.5 - cy) / hh - (x + 0.5 - cx) / hw + 1) / 2 * cells
    checker = (np.floor(u).astype(int) + np.floor(v).astype(int)) % 2 == 0
    light, dark = art.TOPS["light"], art.TOPS["dark"]
    paint(img, core & checker, light["base"])
    paint(img, core & ~checker, dark["base"])
    # тонкая сетка между клетками, как кромки блоков на доске
    du, dv = cells / (2 * hh), cells / (2 * hh)
    grid = core & ((np.abs(u - np.round(u)) < du * 0.5) | (np.abs(v - np.round(v)) < dv * 0.5))
    paint(img, grid & checker, light["edge"])
    paint(img, grid & ~checker, dark["edge"])

    # заклёпки по углам
    for rx, ry in ((8, 8), (CW - 10, 8), (8, CH - 10), (CW - 10, CH - 10)):
        img[ry, rx] = (*hexc(p[4]), 255)
        img[ry, rx + 1] = (*hexc(p[3]), 255)
        img[ry + 1, rx] = (*hexc(p[3]), 255)
        img[ry + 1, rx + 1] = (*hexc(p[0]), 255)
    return img


# ---------------------------------------------------------------- колода-стопка
def deck(f, count=12, seed=1):
    """Стопка рубашкой вверх: под верхней картой видны торцы нижних, с лёгким разбросом."""
    pal = FACTIONS[f]
    fr = pal["frame"]
    rng = np.random.default_rng(seed)
    back = card_back(f)
    pad = 2
    w, h = CW + 2 * pad, CH + count + pad + 1
    img = np.zeros((h, w, 4), np.uint8)
    shifts = [0] + [int(s) for s in rng.choice([-1, 0, 0, 0, 1], size=count - 1)]
    # торцы снизу вверх
    for k in range(count - 1, 0, -1):
        top = k  # смещение вниз относительно верхней карты
        dx = pad + shifts[k]
        row = CH - 1 + top
        x0, x1 = dx + CHAMFER, dx + CW - 1 - CHAMFER
        color = fr[2] if k % 2 else fr[1]
        img[row, x0 : x1 + 1] = (*hexc(color), 255)
        img[row, x0 - 1] = (*hexc(fr[0]), 255)
        img[row, x1 + 1] = (*hexc(fr[0]), 255)
    last = CH - 1 + count
    img[last, pad + CHAMFER : pad + CW - CHAMFER] = (*hexc(fr[0]), 255)
    # верхняя карта
    mask = back[..., 3] > 0
    region = img[0:CH, pad : pad + CW]
    region[mask] = back[mask]
    return img


# ---------------------------------------------------------------- RotSprite
def scale2x(img):
    p = np.pad(img, ((1, 1), (1, 1), (0, 0)))
    B, D, E, F, H = p[:-2, 1:-1], p[1:-1, :-2], p[1:-1, 1:-1], p[1:-1, 2:], p[2:, 1:-1]

    def eq(a, b):
        return np.all(a == b, axis=2)

    cond = ~eq(B, H) & ~eq(D, F)
    out = np.zeros((img.shape[0] * 2, img.shape[1] * 2, 4), np.uint8)
    out[0::2, 0::2] = np.where((cond & eq(D, B))[..., None], D, E)
    out[0::2, 1::2] = np.where((cond & eq(B, F))[..., None], F, E)
    out[1::2, 0::2] = np.where((cond & eq(D, H))[..., None], D, E)
    out[1::2, 1::2] = np.where((cond & eq(H, F))[..., None], F, E)
    return out


def rotate(img, deg, outline):
    """Поворот пиксель-арта без размытия; центр картинки — центр исходной."""
    big = scale2x(scale2x(scale2x(img)))
    h, w = img.shape[:2]
    a = math.radians(deg)
    ca, sa = math.cos(a), math.sin(a)
    rw = int(math.ceil(abs(w * ca) + abs(h * sa))) + 4
    rh = int(math.ceil(abs(w * sa) + abs(h * ca))) + 4
    yy, xx = np.mgrid[0:rh, 0:rw]
    dx, dy = xx + 0.5 - rw / 2, yy + 0.5 - rh / 2
    lx = ca * dx + sa * dy + w / 2
    ly = -sa * dx + ca * dy + h / 2
    sx, sy = np.floor(lx * 8).astype(int), np.floor(ly * 8).astype(int)
    ok = (sx >= 0) & (sy >= 0) & (sx < w * 8) & (sy < h * 8)
    out = np.zeros((rh, rw, 4), np.uint8)
    out[ok] = big[sy[ok], sx[ok]]
    # чистый контур: край силуэта всегда цвета контура
    solid = out[..., 3] > 0
    edge = solid & ~(np.roll(solid, 1, 0) & np.roll(solid, -1, 0) & np.roll(solid, 1, 1) & np.roll(solid, -1, 1))
    out[edge] = (*hexc(outline), 255)
    return out


def shadow_of(img, dx=2, dy=3):
    sh = np.zeros((img.shape[0] + dy, img.shape[1] + dx, 4), np.uint8)
    sh[dy:, dx:][img[..., 3] > 0] = SHADOW
    return sh


def put(canvas, arr, x, y):
    canvas.alpha_composite(Image.fromarray(arr, "RGBA"), (int(x), int(y)))


# ---------------------------------------------------------------- веер в руке
def hand(canvas, f, cx, base_y, n=5, hover=None, step=9.0, radius=170, mirror=False, kinds=None):
    """Карты по дуге вокруг точки ниже руки; `hover` — приподнятая выбранная карта.

    `kinds` — типы карт по порядку (по умолчанию все обычные карты стороны `f`).
    `mirror` — рука соперника, повёрнутая на 180°: точка поворота выше руки, поверх
    лежат левые карты, карты рубашкой вверх (свет и тени не переворачиваются).
    """
    kinds = kinds or [f] * n
    sign = -1 if mirror else 1
    pivot_y = base_y + sign * radius
    order = [k for k in range(n) if k != hover]
    if mirror:
        order.reverse()
    # выбранная карта рисуется последней: она поверх соседей
    for k in order + ([hover] if hover is not None else []):
        ang = (k - (n - 1) / 2) * step
        if k == hover:
            rot, lift = card_front(kinds[k], selected=True), 18
        else:
            face = card_back(f) if mirror else card_front(kinds[k])
            frame0 = FACTIONS[f]["frame"][0] if mirror else CARD_TYPES[kinds[k]]["frame"][0]
            rot, lift = rotate(face, sign * ang, frame0), 0
        a = math.radians(ang)
        px = cx + math.sin(a) * radius
        py = pivot_y - sign * (math.cos(a) * radius + lift)
        x, y = px - rot.shape[1] / 2, py - rot.shape[0] / 2
        put(canvas, shadow_of(rot), x, y)
        put(canvas, rot, x, y)

# ---------------------------------------------------------------- превью
def sheet(scale=3):
    """Лист: строки — обычные белых, обычные чёрных, победы, жертвы (лицо и выбранное);
    справа — рубашки, колоды и веер из карт всех типов."""
    rows = ["w", "b", "victory", "sacrifice"]
    gap = 12
    cell_w, cell_h = CW + 4 + gap, CH + 6 + gap
    w = gap + 2 * cell_w + gap + 4 * cell_w
    h = gap + 4 * cell_h
    canvas = Image.new("RGBA", (w, h), hexc("#1b1e27") + (255,))
    for r, kind in enumerate(rows):
        for c, sel in enumerate((False, True)):
            it = card_front(kind, selected=sel)
            x, y = gap + c * cell_w, gap + r * cell_h
            put(canvas, shadow_of(it), x, y)
            put(canvas, it, x, y)
    right = gap + 2 * cell_w + gap
    for k, it in enumerate([card_back("w"), card_back("b"), deck("w", 12), deck("b", 12, seed=4)]):
        x, y = right + k * cell_w, gap
        put(canvas, shadow_of(it), x, y)
        put(canvas, it, x, y)
    hand(canvas, "w", right + 2 * cell_w, gap + 2.6 * cell_h, n=6, hover=None, step=8, radius=170,
         kinds=["w", "victory", "w", "sacrifice", "w", "victory"])
    return canvas.resize((w * scale, h * scale), Image.NEAREST)


def table_preview(scale=2, deck_at=None):
    pieces = os.path.join(art.ROOT, "assets", "breach", "pieces")
    sprites = {}
    for color in "wb":
        for k in "PNBRQK":
            body = Image.open(os.path.join(pieces, f"{color}{k}.png")).convert("RGBA")
            shadow = Image.open(os.path.join(pieces, f"{color}{k}_shadow.png")).convert("RGBA")
            sprites[color + k] = (body, shadow)
    # Своя рука — у правого угла доски, своя колода — в левом нижнем углу экрана; у
    # соперника — то же, повёрнутое на 180° вокруг центра доски. Кнопка «Конец хода» —
    # справа по центру экрана.
    import breach_ui  # здесь, а не наверху: breach_ui сам импортирует этот модуль

    side, top = 100, 70
    canvas = art.board_preview(
        sprites, scale=1, demo=False, margin_x=side, margin_right=side, margin_top=top
    )
    cw, ch = canvas.size
    mid_x, mid_y = side + 4 * art.TW, top + 4 * art.TH  # центр верхней грани доски
    corner_x, corner_y = mid_x + 4 * art.TW, mid_y + art.SIDE  # правый угол, низ бока
    dw, db = deck("w", 12), deck("b", 12, seed=4)
    dk_h, dk_w = dw.shape[:2]
    left_x, left_y = side, mid_y  # левый угол доски
    deck_x, deck_y = deck_at or (left_x - 76, left_y + 152)  # колода в левом нижнем углу экрана
    hand_x, hand_y = corner_x - 24, corner_y + 135  # рука под краем доски, у цифр

    put(canvas, shadow_of(dw), deck_x, deck_y)
    put(canvas, dw, deck_x, deck_y)
    hand(canvas, "w", hand_x, hand_y, n=6, hover=3, step=8.5, radius=220,
         kinds=["w", "sacrifice", "w", "victory", "w", "w"])
    # соперник: точка (x, y) переходит в (2·mid_x − x, 2·mid_y − y)
    ox, oy = 2 * mid_x - (deck_x + dk_w), 2 * mid_y - (deck_y + dk_h)
    put(canvas, shadow_of(db), ox, oy)
    put(canvas, db, ox, oy)
    hand(canvas, "b", 2 * mid_x - hand_x, 2 * mid_y - hand_y, n=6, step=8.5, radius=220, mirror=True)
    # кнопка «Конец хода»: по вертикали — центр экрана, по горизонтали — середина поля
    # между правым углом доски и краем экрана
    bh, bw = breach_ui.end_turn().shape[:2]
    put(canvas, breach_ui.button_png("normal"), (corner_x + cw) // 2 - bw // 2, ch // 2 - bh // 2)
    return canvas.resize((cw * scale, ch * scale), Image.NEAREST)

if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)

    def save(arr, name):
        Image.fromarray(arr, "RGBA").save(os.path.join(OUT, name))

    # обычные карты сторон — прежние имена файлов (их грузит игра), остальные типы — свои
    for kind in CARD_KINDS:
        save(card_front(kind), f"front_{kind}.png")
        save(card_front(kind, selected=True), f"front_{kind}_selected.png")
    for f in "wb":
        save(card_back(f), f"back_{f}.png")
        save(deck(f, 12, seed=1 if f == "w" else 4), f"deck_{f}.png")
    sheet().save(os.path.join(OUT, "preview_cards.png"))
    table_preview().save(os.path.join(OUT, "preview_table.png"))
    print("done", OUT)
