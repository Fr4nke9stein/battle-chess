"""Картинка главного меню: толстая книга на деревянном столе при свече — в стиле ITB.

Сцена 480×270 (при 1920×1080 — ровно ×4, как пиксели нынешнего меню): стол из досок с
волокнами, сучками и стыками; на нём толстая книга в бордовой коже — по краю обложки
плетёный кожаный кант между золотыми линиями, латунные уголки с заклёпками, в центре золотой
медальон с королём, тиснение «БОЕВЫЕ ШАХМАТЫ»; спереди виден толстый блок страниц, из него
свисает лента-ляссе. Слева свеча на латунной подставке с потёками воска; свет от неё
полосами гаснет к краям (виньетка), книга и вещи отбрасывают тени вправо. На столе —
шахматные фигуры у свечи, монеты, чернильница с пером (слева вверху, в свете свечи).

Кнопки меню — закладки, торчащие из книги справа (отдельные спрайты, их кладёт игра):
лента с раздвоенным концом, прострочка, надпись пиксельным шрифтом 5×7 (`FONT`). Состояния:
`normal`, `hover` (выдвинута на HOVER_PULL пикселей, светлее), `pressed` (выдвинута на
PRESS_PULL, темнее, надпись ниже на пиксель). Холст закладки TAB_W×TAB_H (тень внутри),
левый край — у правого края обложки: закладка k — в (BOOK_RIGHT − 1, TAB_Y0 + k·TAB_PITCH).

`py tools/breach_menu.py [папка]` — по умолчанию assets/breach/menu:
- menu_background.png (сцена со свечой), menu_background_noflame.png (без огонька — под
  анимацию), menu_flame.png (FLAME_FRAMES кадров FLAME_W×FLAME_H в ряд, 90 мс; точка фитиля —
  FLAME_ANCHOR в кадре, на фоне — FLAME_AT);
- tab_<имя>_<состояние>.png — закладки с надписями (`TABS`), tab_blank_<цвет>_<состояние>.png —
  пустые закладки всех цветов (`TAB_COLORS`) под свои надписи;
- preview_menu.png (×2, первая закладка под курсором), preview_menu.gif (огонёк и курсор по
  закладкам).
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_coin as coin  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "menu")

W, H = 480, 270
YY, XX = np.mgrid[0:H, 0:W]
PX, PY = XX + 0.5, YY + 0.5
BAYER = np.array([[0, 2], [3, 1]]) / 4.0 - 0.375
DITHER = BAYER[YY % 2, XX % 2]
OUTLINE = hexc("#0b0604")

# книга: обложка (вид сверху), толщина спереди
BOOK = (120, 10, 300, 234)  # x0, y0, x1, y1 обложки — высокий классический том (шире — просьба пользователя)
BOOK_RIGHT = BOOK[2]
STRIP = 30  # толщина книги: снизу видны доска, страницы, доска; слева — корешок
KX = 0.3  # на сколько пикселей влево уходит пиксель толщины (виден корешок слева)
CANDLE_X = 46
FLAME_AT = (CANDLE_X, 33)  # основание огонька (верх фитиля) на фоне
LIGHT_GROUND = (CANDLE_X, 74)  # точка стола под огоньком — от неё свет гаснет

# ---------------------------------------------------------------- палитры (от тёмного)
def pal(*hexes):
    return np.array([hexc(h) for h in hexes], np.uint8)


TABLE, LEATHER, BRAID, BRASS, PAGES, WAX, RIBBON, GLASS, FEATHER = range(9)
PALETTES = {
    TABLE: (pal("#0c0705", "#1a100a", "#2a1910", "#3c2516", "#52331d", "#6b4425", "#87582f", "#a6703c"), 0.9),
    LEATHER: (pal("#110506", "#220a0b", "#381012", "#521917", "#70261f", "#913829", "#b25239"), 0.35),
    BRAID: (pal("#1a0e07", "#38200f", "#5c381b", "#835428", "#a97338", "#cf9a52"), 0.0),
    BRASS: (pal("#1c1104", "#462d08", "#7a5313", "#ac7e29", "#d9aa46", "#f8dd86", "#fff5c6"), 0.0),
    PAGES: (pal("#271d14", "#4b3b28", "#776346", "#a58d6c", "#cdb892", "#e9d8b0", "#fbefcc"), 0.0),
    WAX: (pal("#382c20", "#6b5944", "#a58d6e", "#d2c09c", "#ecdfbd", "#fbf2d6", "#fffbe8"), 0.4),
    RIBBON: (pal("#280507", "#560c12", "#88161c", "#b8262a", "#e04a40"), 0.0),
    GLASS: (pal("#05070c", "#0d1420", "#182638", "#294056", "#4a6a86", "#8fb2c8"), 0.0),
    FEATHER: (pal("#3a3734", "#6c6762", "#a29b92", "#cfc8bc", "#ece6da", "#fdfaf2"), 0.0),
}


def quantize(material, value):
    colors, amp = PALETTES[material]
    n = len(colors)
    idx = np.floor(value * (n - 1) + 0.5 + DITHER[: value.shape[0], : value.shape[1]] * amp)
    return colors[np.clip(idx, 0, n - 1).astype(int)]


# ---------------------------------------------------------------- шрифт 5×7 (кириллица)
FONT = {
    "А": [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    "Б": ["#####", "#....", "#....", "####.", "#...#", "#...#", "####."],
    "В": ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
    "Г": ["#####", "#....", "#....", "#....", "#....", "#....", "#...."],
    "Д": [".###.", ".#.#.", ".#.#.", ".#.#.", ".#.#.", "#####", "#...#"],
    "Е": ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
    "Ж": ["#.#.#", "#.#.#", ".###.", "..#..", ".###.", "#.#.#", "#.#.#"],
    "З": [".###.", "#...#", "....#", "..##.", "....#", "#...#", ".###."],
    "И": ["#...#", "#...#", "#..##", "#.#.#", "##..#", "#...#", "#...#"],
    "Й": [".#.#.", "..#..", "#...#", "#..##", "#.#.#", "##..#", "#...#"],
    "К": ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
    "Л": ["..###", ".#..#", ".#..#", ".#..#", ".#..#", ".#..#", "#...#"],
    "М": ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"],
    "Н": ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    "О": [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "П": ["#####", "#...#", "#...#", "#...#", "#...#", "#...#", "#...#"],
    "Р": ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
    "С": [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."],
    "Т": ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
    "У": ["#...#", "#...#", "#...#", ".####", "....#", "#...#", ".###."],
    "Ф": ["..#..", ".###.", "#.#.#", "#.#.#", "#.#.#", ".###.", "..#.."],
    "Х": ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
    "Ц": ["#..#.", "#..#.", "#..#.", "#..#.", "#..#.", "#####", "....#"],
    "Ч": ["#...#", "#...#", "#...#", ".####", "....#", "....#", "....#"],
    "Ш": ["#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", "#.#.#", "#####"],
    "Щ": ["#.#.#", "#.#.#", "#.#.#", "#.#.#", "#.#.#", "#####", "....#"],
    "Ъ": ["##...", ".#...", ".#...", ".###.", ".#..#", ".#..#", ".###."],
    "Ы": ["#...#", "#...#", "#...#", "###.#", "#.#.#", "#.#.#", "###.#"],
    "Ь": ["#....", "#....", "#....", "####.", "#...#", "#...#", "####."],
    "Э": [".###.", "#...#", "....#", "..###", "....#", "#...#", ".###."],
    "Ю": ["#..#.", "#.#.#", "#.#.#", "###.#", "#.#.#", "#.#.#", "#..#."],
    "Я": [".####", "#...#", "#...#", ".####", "..#.#", ".#..#", "#...#"],
    "R": ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
    "G": [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".###."],
    "U": ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "L": ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
    "I": [".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "0": [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
    "1": ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "2": [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
    "3": [".###.", "#...#", "....#", "..##.", "....#", "#...#", ".###."],
    "-": ["...", "...", "...", "###", "...", "...", "..."],
    ":": [".", ".", "#", ".", ".", "#", "."],
    ".": [".", ".", ".", ".", ".", ".", "#"],
    " ": ["...", "...", "...", "...", "...", "...", "..."],
}
FONT.update({"O": FONT["О"], "E": FONT["Е"], "K": FONT["К"]})


def text_mask(text, scale=1):
    """Маска надписи шрифтом 5×7 (заглавные), между знаками пиксель."""
    text = text.upper()
    width = sum(len(FONT[c][0]) + 1 for c in text) - 1
    m = np.zeros((7, width), bool)
    x = 0
    for c in text:
        rows = FONT[c]
        for dy, row in enumerate(rows):
            for dx, ch in enumerate(row):
                if ch == "#":
                    m[dy, x + dx] = True
        x += len(rows[0]) + 1
    if scale > 1:
        m = np.kron(m, np.ones((scale, scale), bool))
    return m


# ---------------------------------------------------------------- свет
def vignette():
    r = np.hypot((PX - 240) / 255, (PY - 132) / 160)
    return 1 - 0.55 * np.clip((r - 0.5) / 0.6, 0, 1) ** 1.3


VIGNETTE = vignette()


def light(height=0.0, gain=1.0):
    """Свет свечи: от точки стола под огоньком гаснет; поверхности выше — ближе к огню."""
    gx, gy = LIGHT_GROUND[0], LIGHT_GROUND[1] - height
    d = np.hypot(PX - gx, (PY - gy) * 1.12)
    return (0.2 + 0.98 / (1 + (d / 150) ** 2)) * gain * VIGNETTE


def rrect_sdf(px, py, box, r):
    x0, y0, x1, y1 = box
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    hx, hy = (x1 - x0) / 2 - r, (y1 - y0) / 2 - r
    qx, qy = np.abs(px - cx) - hx, np.abs(py - cy) - hy
    return np.hypot(np.maximum(qx, 0), np.maximum(qy, 0)) + np.minimum(np.maximum(qx, qy), 0) - r


def neighbours(m):
    out = np.zeros_like(m)
    out[1:] |= m[:-1]
    out[:-1] |= m[1:]
    out[:, 1:] |= m[:, :-1]
    out[:, :-1] |= m[:, 1:]
    return out


# ---------------------------------------------------------------- сцена
class Scene:
    def __init__(self):
        self.mat = np.full((H, W), TABLE, int)
        self.alb = np.zeros((H, W))
        self.lit = light()
        self.shade = np.ones((H, W))
        self.z = np.zeros((H, W), int)
        self.fixed = np.zeros((H, W, 4), np.uint8)  # готовые цвета поверх (контуры, спрайты)
        self.outlined = np.ones((H, W), bool)  # вокруг этих пикселей рисовать контур на заднем
        self.zc = 0

    def paint(self, mask, material, albedo, lit=None, outline=True):
        self.zc += 1
        self.mat[mask] = material
        self.alb[mask] = albedo[mask] if np.ndim(albedo) else albedo
        if lit is not None:
            self.lit[mask] = lit[mask] if np.ndim(lit) else lit
        self.z[mask] = self.zc
        self.fixed[mask] = 0
        self.outlined[mask] = outline
        return self.zc

    def sprite(self, img, x0, y0, factor=1.0):
        """Готовый спрайт (свои цвета и контур) с затемнением по свету."""
        self.zc += 1
        h, w = img.shape[:2]
        ys, xs = np.nonzero(img[..., 3] > 0)
        for y, x in zip(ys, xs):
            X, Y = x0 + x, y0 + y
            if 0 <= X < W and 0 <= Y < H:
                f = factor if np.ndim(factor) == 0 else factor[Y, X]
                self.fixed[Y, X] = (*np.clip(img[y, x, :3] * f, 0, 255).astype(np.uint8), 255)
                self.z[Y, X] = self.zc
                self.outlined[Y, X] = False
                self.mat[Y, X] = -1

    def cast(self, mask, strength=0.55):
        """Тень на стол: затемнение там, где стол (с мягким краем через пиксель)."""
        edge = neighbours(mask) & ~mask
        table = self.mat == TABLE
        self.shade[mask & table] *= strength
        soft = edge & table & ((XX + YY) % 2 == 0)
        self.shade[soft] *= (1 + strength) / 2

    def render(self):
        img = np.zeros((H, W, 3), np.uint8)
        value = self.alb * self.lit * self.shade
        for m in PALETTES:
            sel = self.mat == m
            if sel.any():
                img[sel] = quantize(m, value)[sel]
        # контур: пиксели заднего объекта рядом с передним (у которого контур включён)
        front = np.zeros((H, W), bool)
        for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
            zn = np.roll(np.roll(self.z, dy, 0), dx, 1)
            on = np.roll(np.roll(self.outlined, dy, 0), dx, 1)
            front |= (zn > self.z) & on
        img[front & (self.mat != -1)] = OUTLINE
        m = self.fixed[..., 3] > 0
        img[m] = self.fixed[m, :3]
        return img


def smooth_noise(n, count, rng):
    return np.interp(np.arange(n), np.linspace(0, n, count), rng.uniform(-1, 1, count))


def value_noise(cell, rng):
    gh, gw = H // cell + 2, W // cell + 2
    g = rng.random((gh, gw))
    y, x = YY / cell, XX / cell
    y0, x0 = y.astype(int), x.astype(int)
    fy, fx = y - y0, x - x0
    fy, fx = fy * fy * (3 - 2 * fy), fx * fx * (3 - 2 * fx)
    a = g[y0, x0] * (1 - fx) + g[y0, x0 + 1] * fx
    b = g[y0 + 1, x0] * (1 - fx) + g[y0 + 1, x0 + 1] * fx
    return a * (1 - fy) + b * fy


def paint_table(sc, rng):
    """Доски поперёк стола: волокна, сучки, стыки, щели между досками."""
    a = np.zeros((H, W))
    y = -7
    while y < H:
        h = int(rng.integers(20, 27))
        y0, y1 = y, y + h
        r0, r1 = max(0, y0), min(H, y1)
        base = rng.uniform(0.6, 0.74)
        n1, n2 = smooth_noise(W, 14, rng), smooth_noise(W, 70, rng)
        t = (YY[r0:r1] - y0) / h
        g = np.sin(2 * np.pi * (t * 2.3 + 0.75 * n1 + 0.14 * n2))
        streak = np.abs(np.sin(2 * np.pi * (t * 4.6 + 1.1 * n1 + 0.3 * n2))) < 0.11
        a[r0:r1] = base + 0.055 * g - 0.09 * streak
        if 0 <= y0 < H:
            a[y0] += 0.07
        if 0 <= y1 - 1 < H:
            a[y1 - 1] = 0.12  # щель
        if 0 <= y1 - 2 < H:
            a[y1 - 2] -= 0.05
        for jx in rng.choice(np.arange(40, W - 40), size=int(rng.integers(0, 2)), replace=False):
            a[r0 : r1 - 1, jx] = 0.14
            a[r0 : r1 - 1, jx + 1] += 0.06
        if rng.random() < 0.5 and h > 8:
            kx, ky = rng.uniform(20, W - 20), rng.uniform(y0 + 6, y1 - 6)
            rx = rng.uniform(5, 9)
            d = np.hypot((PX - kx) / rx, (PY - ky) / (rx * 0.42))
            m = (d < 1) & (YY >= r0) & (YY < r1 - 1)
            a[m & (np.sin(d * 9) > 0.5)] -= 0.1
            a[m & (d < 0.33)] -= 0.22
        y = y1
    sc.alb[:] = a
    sc.mat[:] = TABLE


def bevel(sdf_fn, box, r, e, width=3.0, amount=0.2):
    """Скругление края: освещённые стороны (к свече — влево-вверх) светлее."""
    gx = sdf_fn(PX + 1, PY, box, r) - sdf_fn(PX - 1, PY, box, r)
    gy = sdf_fn(PX, PY + 1, box, r) - sdf_fn(PX, PY - 1, box, r)
    ln = np.hypot(gx, gy) + 1e-9
    to_light = np.array([-0.94, -0.34])
    s = (gx / ln) * to_light[0] + (gy / ln) * to_light[1]
    return np.where(e < width, amount * s * (1 - e / width), 0.0)


def paint_book(sc, rng):
    """Классический том: обложка с рамкой-плетёнкой (сетка между двойными золотыми линиями) и
    медальоном с королём, снизу толщина со страницами, слева круглый корешок с бинтами."""
    x0, y0, x1, y1 = BOOK
    r = 3
    sdf = rrect_sdf(PX, PY, BOOK, r)
    cover = sdf < 0
    e = -sdf
    hf = PY - y1  # толщина под нижним краем обложки
    front = (hf >= 0) & (hf < STRIP) & (PX >= x0 - KX * hf) & (PX < x1 - KX * hf)
    hl = (x0 - PX) / KX  # толщина левее левого края обложки
    left = (hl > 0) & (hl <= STRIP) & (PY >= y0 + hl) & (PY < y1 + hl) & ~front
    foot = cover | front | left
    shifted = np.zeros_like(foot)
    shifted[7:, 9:] = foot[:-7, :-9]
    sc.cast(shifted & ~foot, 0.5)
    lit_cover = light(30, 1.08)
    tex = 0.66 + 0.08 * (value_noise(3, rng) - 0.5) + 0.07 * (value_noise(13, rng) - 0.5)
    tex += bevel(rrect_sdf, BOOK, r, e, 3.0, 0.22)
    sc.paint(cover, LEATHER, tex, lit_cover)
    dmin = np.minimum(np.minimum(PY - y0, y1 - PY), np.minimum(PX - x0, x1 - PX))
    # рамка-плетёнка: косая сетка тонких золотых нитей
    band = cover & (dmin >= 8) & (dmin < 22)
    d1, d2 = (XX + YY) % 6 == 0, (XX - YY) % 6 == 0
    sc.paint(band, LEATHER, tex - 0.1, lit_cover)
    sc.paint(band & (d1 | d2), BRASS, np.where(d1 & d2, 0.86, 0.6), lit_cover, outline=False)
    for inset in (5, 7, 23, 25):  # двойные золотые линии снаружи и внутри сетки
        line = cover & (np.floor(dmin) == inset)
        sc.paint(line, BRASS, 0.82, lit_cover, outline=False)
    # медальон с королём посередине обложки
    mcx, mcy = (x0 + x1) / 2, (y0 + y1) / 2
    d = np.hypot(PX - mcx, PY - mcy)
    ang = np.arctan2(PY - mcy, PX - mcx)
    to_l = np.cos(ang - math.atan2(-0.7, -0.7))
    ring = (d < 18) & (d >= 14.5)
    sc.paint(ring, BRASS, 0.72 + 0.22 * to_l * np.clip((d - 16.25) / 1.75, -1, 1), lit_cover)
    sc.paint(ring & (np.abs(d - 16.2) < 0.8) & (np.cos(ang * 12) > 0.55), BRASS, 0.95, lit_cover, outline=False)
    sc.paint(d < 14.5, LEATHER, 0.3, lit_cover, outline=False)
    king = np.array([[c == "X" for c in row] for row in coin.KING[24]])
    king = np.kron(king, np.ones((2, 2), bool))
    kh, kw = king.shape
    km = np.zeros((H, W), bool)
    kx0, ky0 = int(round(mcx - kw / 2)), int(round(mcy - kh / 2)) + 1
    km[ky0 : ky0 + kh, kx0 : kx0 + kw] = king
    emboss(sc, km, lit_cover)
    # веточки над и под медальоном
    sprig = np.zeros((H, W), bool)
    scx = int(mcx)
    for scy in (int(mcy) - 30, int(mcy) + 30):
        sprig[scy, scx - 15 : scx + 16] = True
        for dx in (-12, -8, -4, 4, 8, 12):
            sprig[scy - 1, scx + dx] = True
            sprig[scy - 2, scx + dx + (1 if dx > 0 else -1)] = True
        sprig |= (np.abs(PX - (scx + 0.5)) + np.abs(PY - (scy + 0.5))) <= 2.6
    emboss(sc, sprig, lit_cover)
    # толщина снизу: доски и страницы; у левого края — закругление корешка
    lit_front = light(12, 0.82)
    fx0 = x0 - KX * hf  # левый край нижней грани в этой строке
    spine_end = front & (PX < fx0 + 5)
    top_board = front & (hf < 3) & ~spine_end
    bottom_board = front & (hf >= STRIP - 3) & ~spine_end
    pages = front & ~spine_end & ~top_board & ~bottom_board & (PX >= fx0 + 7) & (PX < x1 - KX * hf - 3)
    gaps = front & ~spine_end & ~top_board & ~bottom_board & ~pages
    board_alb = 0.5 + 0.06 * (value_noise(3, rng) - 0.5) - 0.1 * (PX - x0) / (x1 - x0)
    sc.paint(top_board, LEATHER, np.where(hf < 1, 0.62, board_alb), lit_front)
    rows = (hf - 3).astype(int)
    ends = np.minimum(PX - (fx0 + 7), (x1 - KX * hf - 3) - PX)
    page_alb = 0.86 - 0.09 * (rows % 2) - 0.07 * (rows % 6 == 5) - 0.1 * (rows == 0) - 0.1 * (rows >= STRIP - 8)
    page_alb = page_alb - 0.1 * (PX - x0) / (x1 - x0) - 0.14 * np.clip(1 - ends / 5, 0, 1)
    sc.paint(pages, PAGES, page_alb, lit_front)
    sc.paint(gaps, LEATHER, 0.14, lit_front)
    sc.paint(bottom_board, LEATHER, np.where(hf >= STRIP - 1, 0.34, board_alb - 0.06), lit_front)
    sc.paint(spine_end, LEATHER, 0.5 + 0.12 * ((PX - fx0) < 2), lit_front)
    # корешок слева: круглый, обращён к свече, с выпуклыми бинтами и золотыми линиями
    c = np.clip(hl / STRIP, 0, 1)
    sp = PY - hl - y0  # координата вдоль корешка: 0 — верх, y1 − y0 — низ
    spine_alb = 0.48 + 0.3 * np.sin(np.pi * c) + 0.06 * (value_noise(3, rng) - 0.5)
    lit_spine = light(15, 1.0)
    sc.paint(left, LEATHER, spine_alb, lit_spine)
    for b in (26, 52, 150, 176, 202):
        ridge = left & (sp >= b) & (sp < b + 3)
        sc.paint(ridge, LEATHER, np.where(sp < b + 1, spine_alb + 0.2, spine_alb - 0.18), lit_spine, outline=False)
        for g in (b - 2, b + 4):
            sc.paint(left & (np.floor(sp) == g), BRASS, 0.78, lit_spine, outline=False)
    panel = left & (sp >= 70) & (sp < 132)
    sc.paint(panel, LEATHER, spine_alb - 0.2, lit_spine, outline=False)
    for g in (69, 132):
        sc.paint(left & (np.floor(sp) == g), BRASS, 0.82, lit_spine, outline=False)


def emboss(sc, mask, lit):
    """Золотое тиснение: верх-лево кромки светлее, низ-право темнее, тень вдавлена в кожу."""
    up = np.zeros_like(mask)
    up[1:] = mask[:-1]
    left = np.zeros_like(mask)
    left[:, 1:] = mask[:, :-1]
    down = np.zeros_like(mask)
    down[:-1] = mask[1:]
    right = np.zeros_like(mask)
    right[:, :-1] = mask[:, 1:]
    shadow = np.zeros_like(mask)
    shadow[1:, 1:] = mask[:-1, :-1]
    sc.paint(shadow & ~mask, LEATHER, 0.12, lit, outline=False)
    alb = np.full((H, W), 0.74)
    alb[mask & (~up | ~left)] = 0.98
    alb[mask & (~down | ~right)] = 0.5
    sc.paint(mask, BRASS, alb, lit, outline=False)


def paint_candle(sc, rng, flame=True):
    cx = CANDLE_X
    # подставка: латунная тарелочка с ушком
    dcx, dcy, rx, ry = cx, 80, 22, 7
    e_top = np.hypot((PX - dcx) / rx, (PY - dcy) / ry)
    top = e_top <= 1
    front = (np.hypot((PX - dcx) / rx, (PY - dcy - 3) / ry) <= 1) & ~top & (PY > dcy)
    ear_d = np.hypot((PX - (dcx - rx - 5)) / 6.5, (PY - dcy) / 5)
    ear = (ear_d <= 1) & (ear_d >= 0.5) & ~top
    shadow = np.zeros_like(top)
    shadow[2:, 4:] = (top | front | ear)[:-2, :-4]
    sc.cast(shadow & ~(top | front | ear), 0.6)
    lit_dish = light(4, 1.0)
    ang = np.arctan2(PY - dcy, PX - dcx)
    to_l = np.cos(ang - math.atan2(-0.34, -0.94))
    alb = np.where(e_top > 0.82, 0.72 + 0.2 * to_l, 0.58 - 0.1 * to_l * (e_top > 0.5))
    sc.paint(ear, BRASS, 0.7 + 0.2 * np.cos(np.arctan2(PY - dcy, PX - (dcx - rx - 5)) + 2.8), lit_dish)
    sc.paint(top, BRASS, alb, lit_dish)
    sc.paint(front, BRASS, 0.45 - 0.15 * (PX - dcx) / rx, lit_dish)
    # лужица воска у основания
    pool = np.hypot((PX - cx - 1) / 12, (PY - 77) / 4) <= 1
    sc.paint(pool, WAX, 0.72, light(6, 1.0))
    # свеча: цилиндр, верх светится от огня, потёки
    r = 10.5
    top_y = 40
    u = (PX - cx) / r
    body = (np.abs(u) <= 1) & (PY >= top_y) & (PY < 78)
    lit_c = 0.66 + 0.36 * np.clip((top_y + 44 - PY) / 44, 0, 1)
    alb = 0.42 + 0.32 * np.sqrt(np.clip(1 - u ** 2, 0, 1)) - 0.1 * u
    alb = alb + 0.22 * np.clip(1 - (PY - top_y) / 12, 0, 1)  # воск у огня просвечивает
    sc.paint(body, WAX, alb, lit_c)
    # потёки: светлее тела, к низу закругляются; крайние выпирают за силуэт
    drips = [(-11.4, 13, 3), (-5.0, 7, 2), (1.5, 20, 3), (6.0, 9, 2), (9.6, 15, 3)]
    for xr, ln, w in drips:
        dx0 = int(round(cx + xr))
        m = np.zeros((H, W), bool)
        for y in range(top_y - 1, top_y + ln):
            ww = w if y < top_y + ln - 1 else max(1, w - 1)
            off = 1 if (y >= top_y + ln - 1 and w > 2) else 0
            m[y, dx0 + off : dx0 + off + ww] = True
        uu = (PX - cx) / r
        dalb = 0.62 + 0.26 * np.sqrt(np.clip(1 - np.clip(uu, -1, 1) ** 2, 0.15, 1)) - 0.08 * uu
        dalb = dalb + 0.1 + 0.2 * np.clip(1 - (PY - top_y) / 14, 0, 1)
        dalb = np.where(YY == top_y + ln - 1, dalb - 0.14, dalb)
        sc.paint(m, WAX, dalb, lit_c, outline=False)
    # верх: кромка и светящаяся лужица
    rim = np.hypot((PX - cx) / r, (PY - top_y) / 3.4)
    cap = rim <= 1
    sc.paint(cap, WAX, np.where(rim > 0.72, 0.95, 1.1), lit_c, outline=False)
    # фитиль
    wick = (np.abs(PX - cx - 0.0) < 0.6) & (PY >= FLAME_AT[1] - 1) & (PY < top_y)
    sc.paint(wick, LEATHER, 0.05, 1.0, outline=False)
    if flame:
        put_flame(sc, flame_image(0))


def flame_image(k, frames=8):
    """Огонёк свечи (кадр k): вытянутая капля, внутри жёлтое и белое, у основания синее;
    чуть качается и дышит. Холст FLAME_W×FLAME_H, верх фитиля — FLAME_ANCHOR."""
    w, h = FLAME_W, FLAME_H
    ax, ay = FLAME_ANCHOR
    ph = 2 * math.pi * k / frames
    lean = 1.1 * math.sin(ph) + 0.4 * math.sin(2 * ph + 1)
    tall = 25 + 2.2 * math.sin(ph * 2 + 0.6)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    t = (ay + 1 - (yy + 0.5)) / tall  # 0 — низ, 1 — кончик
    cxs = ax + 0.5 + lean * np.clip(t, 0, 1) ** 2
    hw = 2.35 * 4.4 * np.clip(t, 0, 1) ** 0.55 * np.clip(1 - t, 0, 1) ** 1.05
    dx = np.abs(xx + 0.5 - cxs)
    outer = (dx <= hw) & (t > 0) & (t < 1)
    layers = [
        (outer, "#ff8a2a"),
        (outer & (dx <= hw * 0.68) & (t < 0.85), "#ffc14a"),
        (outer & (dx <= hw * 0.45) & (t > 0.08) & (t < 0.6), "#ffeaa0"),
        (outer & (dx <= hw * 0.22) & (t > 0.12) & (t < 0.42), "#fffbe6"),
        (outer & (t < 0.1) & (dx >= hw * 0.45), "#5a74e8"),  # синяя кайма у основания
        (outer & (t < 0.07) & (dx < hw * 0.45), "#c84a1c"),
    ]
    for m, c in layers:
        img[m] = (*hexc(c), 255)
    # уголёк на фитиле
    img[ay, ax] = (*hexc("#ff5a1e"), 255)
    return img


FLAME_W, FLAME_H = 18, 32
FLAME_ANCHOR = (9, 29)  # верх фитиля в кадре огонька
FLAME_FRAMES = 8


def put_flame(sc, img):
    x0, y0 = FLAME_AT[0] - FLAME_ANCHOR[0], FLAME_AT[1] - FLAME_ANCHOR[1]
    sc.sprite(img, x0, y0)


def glow(rgb):
    """Тёплое свечение воздуха вокруг огонька: три кольца, край — через пиксель."""
    out = rgb.astype(float)
    d = np.hypot(PX - FLAME_AT[0], (PY - (FLAME_AT[1] - 10)) * 1.05)
    warm = np.array(hexc("#ffc878"), float)
    for r, a in ((34, 0.07), (24, 0.11), (15, 0.16)):
        m = (d < r) | ((d < r + 1.5) & ((XX + YY) % 2 == 0))
        out[m] = out[m] * (1 - a) + warm * a
    return np.clip(out, 0, 255).astype(np.uint8)


def scene(flame=True, seed=4):
    rng = np.random.default_rng(seed)
    sc = Scene()
    paint_table(sc, rng)
    paint_book(sc, rng)
    paint_candle(sc, rng, flame)
    rgb = sc.render()
    if flame:
        rgb = glow(rgb)
    return rgb


# ---------------------------------------------------------------- закладки
TAB_COLORS = {
    "wine": ["#1a0608", "#36100f", "#561c18", "#742a22", "#8f3e30", "#a85844"],
    "plum": ["#140a16", "#28142c", "#3c2142", "#523058", "#6a4470", "#86608a"],
    "teal": ["#071412", "#0f2622", "#1a3a34", "#285048", "#3a685e", "#548478"],
    "ochre": ["#1a1206", "#33240e", "#4e3716", "#6b4c20", "#87632c", "#a37e40"],
    "forest": ["#0c140a", "#1a2a14", "#2a3f20", "#3b552c", "#506e3c", "#6a8a52"],
    "navy": ["#080c18", "#121b30", "#1e2c4a", "#2c3f64", "#3e557e", "#58709a"],
    "umber": ["#120c08", "#241812", "#38261c", "#4e3626", "#664834", "#80604a"],
    "slate": ["#0e1012", "#1c2024", "#2c3238", "#3e464e", "#535d66", "#6e7a84"],
}
TABS = [  # имя, надпись, цвет — кнопки нынешнего меню и «Настройки»
    ("play", "Начать игру", "wine"),
    ("rogue", "Rogue-like режим", "plum"),
    ("multiplayer", "Мультиплеер", "teal"),
    ("boost_off", "Компьютер: обычный", "ochre"),
    ("boost_on", "Компьютер: усиленный", "ochre"),
    ("advance_off", "Взятие: на месте", "forest"),
    ("advance_on", "Взятие: с перемещением", "forest"),
    ("balance", "Править баланс", "navy"),
    ("settings", "Настройки", "umber"),
    ("quit", "Выйти", "slate"),
]
SLOTS = ["play", "rogue", "multiplayer", "boost", "advance", "balance", "settings", "quit"]  # порядок сверху
TAB_LEN = 140  # видимая длина закладки (обычное состояние)
HOVER_PULL, PRESS_PULL = 5, 2
TAB_BODY_H = 13
TAB_WAVE, TAB_PERIOD = 2, 64  # лента извивается: размах и шаг волны; у книги волны нет
TAB_W, TAB_H = TAB_LEN + HOVER_PULL + 3, TAB_BODY_H + 2 * TAB_WAVE + 3
TAB_Y0, TAB_PITCH = 54, 17  # верх ленты k-й закладки — TAB_Y0 + k·TAB_PITCH (холст выше на TAB_WAVE)
TEXT_X = 8  # начало надписи от левого края закладки
TEXT_MAX = TAB_LEN - TEXT_X - 9  # до раздвоенного конца


def tab_phase(name):
    """Фаза волны закладки: у каждой своя, у вариантов одной кнопки — одинаковая."""
    return 1.3 * SLOTS.index(name.rsplit("_", 1)[0] if name not in SLOTS else name)


def tab_flat(color, state, label, extra):
    """Прямая лента (тона сдвинуты на extra — для светлых и тёмных изгибов) и её форма."""
    tones = [hexc(c) for c in TAB_COLORS[color]]
    shift = {"normal": -1, "hover": 0, "pressed": -2}[state] + extra  # приглушённые: основа на тон темнее
    length = TAB_LEN + {"normal": 0, "hover": HOVER_PULL, "pressed": PRESS_PULL}[state]

    def tone(i):
        return tones[int(np.clip(i + shift, 0, len(tones) - 1))]

    h = TAB_BODY_H
    yy, xx = np.mgrid[0:h, 0:TAB_W]
    shape = xx < length
    notch = (xx >= length - 7) & (np.abs(yy - (h - 1) / 2) <= (xx - (length - 7)) * (h / 2) / 7)
    shape &= ~notch
    img = np.zeros((h, TAB_W, 4), np.uint8)
    edge = shape & ~(np.roll(shape, 1, 1) & np.roll(shape, -1, 1))
    edge |= shape & ((yy == 0) | (yy == h - 1))
    body = shape & ~edge
    img[body] = (*tone(2), 255)
    img[body & (yy == 1)] = (*tone(3), 255)  # верхняя кромка на свету
    img[body & (yy == h - 2)] = (*tone(1), 255)
    stitch = body & ((yy == 2) | (yy == h - 3)) & (xx % 4 < 2) & (xx > 5) & (xx < length - 9)
    img[stitch & (yy == 2)] = (*tone(3), 255)
    img[stitch & (yy == h - 3)] = (*tone(1), 255)
    img[body & (xx < 2)] = (*tone(0), 255)  # у книги лента уходит под обложку — в тени
    img[body & (xx >= 2) & (xx < 5)] = (*tone(1), 255)
    img[edge] = (*OUTLINE, 255)
    return img, shape


def tab_wave(x, phase):
    """Сдвиг столбца x по вертикали и наклон волны в нём."""
    a = TAB_WAVE * min(1.0, x / 30)
    ang = 2 * math.pi * x / TAB_PERIOD + phase
    return int(round(a * math.sin(ang))), a * math.cos(ang)


def tab_image(color, state="normal", label=None, phase=0.0):
    """Закладка-лента: от книги прямая, дальше извивается волной (столбцы сдвинуты по
    вертикали, изгибы к свету светлее, от него — темнее), с тенью на стол."""
    flats = {k: tab_flat(color, state, label, k) for k in (-1, 0, 1)}
    shape0 = flats[0][1]
    h = TAB_BODY_H
    img = np.zeros((TAB_H, TAB_W, 4), np.uint8)
    sm = np.zeros((TAB_H, TAB_W), bool)
    for x in range(TAB_W):
        dy, slope = tab_wave(x, phase)
        k = 1 if slope < -1.2 else (-1 if slope > 1.2 else 0)
        col, shape = flats[k]
        y0 = TAB_WAVE + dy
        img[y0 : y0 + h, x] = col[:, x]
        sm[y0 : y0 + h, x] = shape0[:, x]
    if label:  # буквы целиком, каждая сдвинута по волне в своей середине — не рвутся
        tones = [hexc(c) for c in TAB_COLORS[color]]
        light_c = hexc("#fff4dc") if state == "hover" else hexc("#d8c8a2")
        th = 7
        ty = (h - th) // 2 + (1 if state == "pressed" else 0)
        text = label.upper()
        width = sum(len(FONT[c][0]) + 1 for c in text) - 1
        assert width <= TEXT_MAX, f"надпись «{label}» длиннее закладки: {width} > {TEXT_MAX}"
        x = TEXT_X
        for ch in text:
            g = text_mask(ch)
            gw = g.shape[1]
            dy, _ = tab_wave(x + gw // 2, phase)
            for (oy, ox), col in (((1, 1), tones[0]), ((0, 0), light_c)):
                ys, xs = np.nonzero(g)
                for yy_, xx_ in zip(ys, xs):
                    Y, X = TAB_WAVE + dy + ty + yy_ + oy, x + xx_ + ox
                    if sm[Y, X]:
                        img[Y, X] = (*col, 255)
            x += gw + 1
    sh = np.zeros_like(sm)
    sh[2:, 1:] = sm[:-2, :-1]
    img[sh & ~sm] = (0, 0, 0, 90)
    return img


# ---------------------------------------------------------------- сохранение и превью
def composite(bg, tabs, hover=None, flame_k=None):
    im = Image.fromarray(bg, "RGB").convert("RGBA")
    if flame_k is not None:
        im.alpha_composite(Image.fromarray(flame_image(flame_k), "RGBA"),
                           (FLAME_AT[0] - FLAME_ANCHOR[0], FLAME_AT[1] - FLAME_ANCHOR[1]))
    for k, (name, label, color) in enumerate(tabs):
        state = "hover" if k == hover else "normal"
        im.alpha_composite(Image.fromarray(tab_image(color, state, label, tab_phase(name)), "RGBA"),
                           (BOOK_RIGHT - 1, TAB_Y0 + k * TAB_PITCH - TAB_WAVE))
    return im


# ---------------------------------------------------------------- открытие книги
OPEN_FRAMES = 30  # кадров открытия, по OPEN_MS
OPEN_MS = 70
OPEN_MAX = 9.0  # на сколько градусов приоткрывается обложка
RAY = np.array(hexc("#ffe2a0"), float)


def ease(t):
    t = min(1.0, max(0.0, t))
    return t * t * (3 - 2 * t)


def bands(b, steps=8, amp=0.8):
    """Свет полосами, на границах — через пиксель (как остальной свет сцены)."""
    return np.clip(np.floor(b * steps + 0.5 + DITHER * amp) / steps, 0, 1)


def warped(theta, px, py, cover):
    """Где на приподнятой обложке пиксель экрана (px, py): обложка поворачивается на
    корешке, её правый край поднимается к зрителю — на экране выше и чуть правее."""
    x0 = BOOK[0]
    c, sn = math.cos(theta), math.sin(theta)
    u = (px - x0) / (c + KX * sn)
    yl = py + u * sn
    sx = np.floor(x0 + u).astype(int)
    sy = np.floor(yl).astype(int)
    ok = (u >= 0) & (sx >= 0) & (sx < W) & (sy >= 0) & (sy < H)
    sxc, syc = np.clip(sx, 0, W - 1), np.clip(sy, 0, H - 1)
    return ok & cover[syc, sxc], sxc, syc, u


def open_frame(bare, k):
    """Кадр k открытия: обложка приоткрывается, из щели под ней идёт свет, лучи по
    страницам и столу переливаются (экран светом не заливается, закладки остаются)."""
    x0, y0, x1, y1 = BOOK
    theta = math.radians(OPEN_MAX) * ease((k - 3) / 13)
    glow_k = ease((k - 2) / 14)
    cover = rrect_sdf(PX, PY, BOOK, 3) < 0
    img = bare.astype(float)
    lid = cover.copy()  # что из кадра — обложка (рисуется поверх закладок)
    # под обложкой — страницы, залитые светом изнутри (ярче к щели)
    g = np.clip(0.35 + 0.45 * (PX - x0) / (x1 - x0) + 0.4 * (PY - y0) / (y1 - y0), 0, 1)
    under = np.array(hexc("#c89a52"), float) * (1 - g[..., None]) + np.array(hexc("#fff4d0"), float) * g[..., None]
    if theta > 0:
        img[cover] = under[cover]
        top, sx, sy, u = warped(theta, PX, PY, cover)
        board = np.zeros((H, W), bool)
        for d in (1, 2, 3):  # толщина доски обложки: видна снизу, подсвечена светом из щели
            b, _, _, _ = warped(theta, PX + KX * d, PY - d, cover)
            board |= b
        board &= ~top
        lit_b = np.array(hexc("#3a1a12"), float) * (1 - glow_k) + np.array(hexc("#b06a30"), float) * glow_k
        img[board] = lit_b
        img[top] = bare[sy[top], sx[top]]
        shape = top | board
        ring = neighbours(shape) & ~shape
        img[ring & (PY < y1 + 2)] = OUTLINE
        lid = shape | (ring & (PY < y1 + 2))
        # лучи из щели вниз-вправо: по толщине книги и по столу
        gap = np.clip((PX - (PY - y1) * 0.25 - x0) / (x1 - x0), 0, 1)  # где на щели начинается луч
        opening = gap * math.sin(theta) / math.sin(math.radians(OPEN_MAX))
        d = PY - (y1 - (PX - x0) * math.sin(theta))  # от нижнего края приподнятой обложки
        sv = PX - (PY - y1) * 0.25
        stripe = (np.sin(sv * 0.5 + 0.35 * k) + np.sin(sv * 0.21 + 1.3 - 0.2 * k)) > 0.4  # лучи переливаются
        beam = (d > 0) & (sv > x0 + 4) & (sv < x1 + 4)
        b = opening * np.exp(-np.maximum(d, 0) / 70) * (0.45 + 0.55 * stripe) * (0.6 + 0.4 * glow_k)
        b = np.where(beam, b, 0)
        near = (d > -1) & (d < 10) & (PX > x0) & (PX < x1 + 12) & ~top  # свечение у самой щели
        b = np.maximum(b, np.where(near, opening * (1 - np.abs(d) / 10) * 0.8, 0))
        bq = bands(b * 0.9)[..., None]
        img = img + (RAY - img) * bq
    img = np.clip(img, 0, 255).astype(np.uint8)
    over = np.zeros((H, W, 4), np.uint8)
    over[lid, :3] = img[lid]
    over[lid, 3] = 255
    return img, over


def open_strip(bare):
    """Кадры открытия с огоньком свечи (без закладок — их игра рисует поверх, на месте)."""
    frames = []
    fx, fy = FLAME_AT[0] - FLAME_ANCHOR[0], FLAME_AT[1] - FLAME_ANCHOR[1]
    for k in range(OPEN_FRAMES):
        base = Image.fromarray(bare, "RGB").convert("RGBA")
        base.alpha_composite(Image.fromarray(flame_image(k % FLAME_FRAMES), "RGBA"), (fx, fy))
        frames.append(open_frame(np.array(base.convert("RGB")), k))
    return [f for f, _ in frames], [o for _, o in frames]


def save_open(bare, shown):
    frames, lids = open_strip(bare)
    Image.fromarray(np.concatenate(frames, axis=1), "RGB").save(os.path.join(OUT, "menu_open.png"))
    # обложка отдельно — игра рисует её поверх закладок: они выходят из-под приподнятой обложки
    Image.fromarray(np.concatenate(lids, axis=1), "RGBA").save(os.path.join(OUT, "menu_open_cover.png"))
    seq = [composite(bare, shown, flame_k=k % FLAME_FRAMES).convert("RGB") for k in range(8)]
    for f, lid in zip(frames, lids):  # закладки на месте, под обложкой
        im = composite(f, shown)
        im.alpha_composite(Image.fromarray(lid, "RGBA"))
        seq.append(im.convert("RGB"))
    seq = [im.resize((W * 2, H * 2), Image.NEAREST) for im in seq]
    allpix = Image.new("RGB", (W * 2, H * 2 * 4))
    for n, k in enumerate((0, 14, 22, 34)):
        allpix.paste(seq[k], (0, n * H * 2))
    palette = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [f.quantize(palette=palette, dither=Image.Dither.NONE) for f in seq]
    gif[0].save(os.path.join(OUT, "preview_menu_open.gif"), save_all=True, append_images=gif[1:],
                duration=OPEN_MS, loop=0, disposal=1)
    sheet = Image.new("RGB", (W * 3, H * 2))
    for n, k in enumerate((6, 12, 17, 21, 25, 29)):
        sheet.paste(Image.fromarray(frames[k], "RGB"), ((n % 3) * W, (n // 3) * H))
    sheet.save(os.path.join(OUT, "preview_menu_open_frames.png"))


def main():
    os.makedirs(OUT, exist_ok=True)
    bg = scene(True)
    Image.fromarray(bg, "RGB").save(os.path.join(OUT, "menu_background.png"))
    bare = scene(False)
    bare = glow(bare)
    Image.fromarray(bare, "RGB").save(os.path.join(OUT, "menu_background_noflame.png"))
    flames = [flame_image(k) for k in range(FLAME_FRAMES)]
    Image.fromarray(np.concatenate(flames, axis=1), "RGBA").save(os.path.join(OUT, "menu_flame.png"))
    for name, label, color in TABS:
        for state in ("normal", "hover", "pressed"):
            Image.fromarray(tab_image(color, state, label, tab_phase(name)), "RGBA").save(
                os.path.join(OUT, f"tab_{name}_{state}.png"))
    for color in TAB_COLORS:
        for state in ("normal", "hover", "pressed"):
            Image.fromarray(tab_image(color, state), "RGBA").save(
                os.path.join(OUT, f"tab_blank_{color}_{state}.png"))
    shown = [t for t in TABS if t[0] not in ("boost_on", "advance_on")]
    still = composite(bg, shown, hover=0)
    still.resize((W * 2, H * 2), Image.NEAREST).save(os.path.join(OUT, "preview_menu.png"))
    frames = []
    for i in range(len(shown) * 6):
        frames.append(composite(bare, shown, hover=i // 6, flame_k=i % FLAME_FRAMES)
                      .convert("RGB").resize((W * 2, H * 2), Image.NEAREST))
    allpix = Image.new("RGB", (W * 2, H * 2 * 3))
    for n, k in enumerate((0, 10, 30)):
        allpix.paste(frames[k], (0, n * H * 2))
    palette = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [f.quantize(palette=palette, dither=Image.Dither.NONE) for f in frames]
    gif[0].save(os.path.join(OUT, "preview_menu.gif"), save_all=True, append_images=gif[1:], duration=90,
                loop=0, disposal=1)
    save_open(bare, shown)
    print("done", OUT)


if __name__ == "__main__":
    main()
