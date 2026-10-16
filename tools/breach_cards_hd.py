"""Карты вдвое детальнее: лицо 100×140 вместо 50×70 (`breach_cards.py`).

На столе карта того же размера, но каждый пиксель арта — 2×2 пикселя карты: рамки, скобки
окна, огоньки и рубашка нарисованы линиями в пиксель карты, поэтому тоньше и подробнее, а в
увеличенной карте под курсором — резкие. Цвета и устройство те же, что у карт 1x.

Лица двух раскладок: с окном под картинку (огненная волна, обмен, финансирование,
исцеление) и текстовые — у карт без картинки окна нет, и всё поле под описание.
Знаки типа (кристалл, череп, венок) — знаки 1x, увеличенные scale2x (диагонали плавнее).
Картинки окна 76×50 — пока картинки 1x через scale2x. Сжигание: карта горит по той же карте
прогорания, что и 1x (вдвое подробнее), огонь, дым и искры — пиксель-арт игры (1x вдвое).

`py tools/breach_cards_hd.py [папка]` — по умолчанию assets/breach/cards/hd.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_bless as bl  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_cards as bc  # noqa: E402
import breach_firewave as fw  # noqa: E402
import breach_portal as pt  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "cards", "hd")
ART_1X = os.path.join(art.ROOT, "assets", "breach", "cards", "art")

S = 2
CW, CH = bc.CW * S, bc.CH * S  # 100×140
CHAMFER = bc.CHAMFER * S
ART_AT, ART_SIZE = (6 * S, 17 * S), (38 * S, 25 * S)  # окно 76×50 в точке (12, 34)
ARTS = ["fire_wave", "swap", "contract_blood", "bless"]

paint = bc.paint


def frame_hd(img, d, side, frame):
    """Рамка 7 пикселей: контур, кромка в два пикселя (свет сверху-слева с бликом), средняя
    полоса, тёмная линия и контур внутри — как `bevel_frame`, только вдвое тоньше линии."""
    light, mid, dark = side == 1, side == 0, side == -1
    paint(img, d == 0, frame[0])
    paint(img, (d == 1) & light, frame[4])
    paint(img, (d == 2) & light, frame[3])
    paint(img, (d >= 1) & (d <= 2) & mid, frame[2])
    paint(img, (d >= 1) & (d <= 2) & dark, frame[1])
    paint(img, (d >= 3) & (d <= 4), frame[2])
    paint(img, d == 5, frame[1])
    paint(img, d == 6, frame[0])


def emblem_hd(t, frame):
    """Знак типа 1x на прозрачном листе, увеличенный scale2x: в карте 2x — на месте 1x."""
    small = np.zeros((bc.CH, bc.CW, 4), np.uint8)
    bc.EMBLEMS[t["emblem"]](small, t, frame)
    return bc.scale2x(small)


def recess(img, x0, x1, y0, y1, shadow, light):
    """Утопленное поле: тень сверху-слева и свет снизу-справа в два пикселя."""
    y, x = np.mgrid[0:CH, 0:CW]
    top = (y >= y0 - 2) & (y < y0) & (x >= x0 - 2) & (x <= x1)
    left = (x >= x0 - 2) & (x < x0) & (y >= y0 - 2) & (y <= y1)
    bottom = (y > y1) & (y <= y1 + 2) & (x >= x0) & (x <= x1 + 2)
    right = (x > x1) & (x <= x1 + 2) & (y >= y0) & (y <= y1 + 2)
    paint(img, top | left, shadow)
    paint(img, bottom | right, light)


def card_front(kind, selected=False):
    """Лицо 100×140: `kind` — "w"/"b", "sacrifice" или "victory"."""
    t = bc.CARD_TYPES[kind]
    frame = bc.SELECT if selected else t["frame"]
    acc = t["accent"]
    img = np.zeros((CH, CW, 4), np.uint8)
    d, side = bc.rings(CW, CH, CHAMFER)
    inner = d >= 7
    paint(img, inner, t["fill"])
    frame_hd(img, d, side, frame)
    y, x = np.mgrid[0:CH, 0:CW]

    # шапка под название и двойной разделитель
    paint(img, inner & (y >= 8) & (y <= 25), t["head"])
    paint(img, inner & (y >= 26) & (y <= 27), t["divider"][0])
    paint(img, inner & (y >= 28) & (y <= 29), t["divider"][1])

    # окно под картинку — у всех карт (у карты без картинки оно пустое): облик единый
    x0, x1, y0, y1 = 12, CW - 13, 34, 83
    win = (x >= x0) & (x <= x1) & (y >= y0) & (y <= y1)
    bands = [y0 + (y1 - y0 + 1) * k // 3 for k in range(4)]
    for k, color in enumerate(t["window"]):
        paint(img, win & (y >= bands[k]) & (y < bands[k + 1]), color)
        # на границе полос — шахматный переход в пиксель
        if k:
            paint(img, win & (y == bands[k]) & ((x + y) % 2 == 0), t["window"][k - 1])
    recess(img, x0, x1, y0, y1, *t["recess"])
    # уголки-скобки, как у прицелов ITB: линии в пиксель длиной 7
    bracket = hexc(bc.SELECT[2] if selected else frame[3])
    for cx, cy, sx, sy in ((x0 + 2, y0 + 2, 1, 1), (x1 - 2, y0 + 2, -1, 1), (x0 + 2, y1 - 2, 1, -1), (x1 - 2, y1 - 2, -1, -1)):
        for k in range(7):
            img[cy, cx + sx * k] = (*bracket, 255)
            img[cy + sy * k, cx] = (*bracket, 255)
    # поле под описание
    ty0, ty1 = 90, 123
    paint(img, (x >= x0) & (x <= x1) & (y >= ty0) & (y <= ty1), t["text"])
    recess(img, x0, x1, ty0, ty1, *t["text_recess"])

    # нижний язычок с тремя огоньками
    tab = (y >= CH - 12) & (y <= CH - 5) & (np.abs(x + 0.5 - CW / 2) <= 18 - (y - (CH - 12)))
    paint(img, tab, t["head"])
    edge = tab & ~(np.roll(tab, 1, 1) & np.roll(tab, -1, 1) & np.roll(tab, 1, 0))
    paint(img, edge, frame[2])
    for k in (-8, 0, 8):
        px = CW // 2 + k - 2
        img[CH - 10 : CH - 8, px : px + 4] = (*hexc(acc[3]), 255)
        img[CH - 8 : CH - 6, px : px + 4] = (*hexc(acc[1]), 255)
        img[CH - 10, px] = (*hexc(acc[4]), 255)

    emblem = emblem_hd(t, frame)
    m = emblem[..., 3] > 0
    img[m] = emblem[m]
    return img


def card_back(f):
    """Рубашка 100×140: пластина с распорками и ромбом-доской, линии в пиксель."""
    pal = bc.FACTIONS[f]
    p, acc = pal["plate"], pal["accent"]
    img = np.zeros((CH, CW, 4), np.uint8)
    d, side = bc.rings(CW, CH, CHAMFER)
    y, x = np.mgrid[0:CH, 0:CW]
    paint(img, d >= 7, p[2])
    frame_hd(img, d, side, pal["frame"])
    paint(img, (d >= 9) & (d <= 10) & (side >= 0), p[0])
    paint(img, (d >= 9) & (d <= 10) & (side < 0), p[3])
    inside = d >= 11
    paint(img, inside, p[1])
    ix0, iy0, ix1, iy1 = 12, 12, CW - 13, CH - 13
    for sgn in (1, -1):
        ax, ay = (ix0, iy0) if sgn == 1 else (ix1, iy0)
        bx, by = (ix1, iy1) if sgn == 1 else (ix0, iy1)
        nx, ny = by - ay, -(bx - ax)
        nl = np.hypot(nx, ny)
        dist = ((x + 0.5 - ax) * nx + (y + 0.5 - ay) * ny) / nl
        paint(img, inside & (np.abs(dist) <= 5.2), p[3])
        paint(img, inside & (np.abs(dist) <= 5.2) & (np.abs(dist) > 3.2) & (dist * sgn > 0), p[4])
        paint(img, inside & (np.abs(dist) > 5.2) & (np.abs(dist) <= 7.0), p[0])
    cx, cy = CW / 2, CH / 2
    hw, hh = 32.0, 24.0
    dm = bc.diamond(cx, cy, hw, hh, CW, CH)
    paint(img, dm <= 1.0 + 5.2 / hh, pal["frame"][0])
    paint(img, dm <= 1.0 + 2.6 / hh, acc[2])
    paint(img, dm <= 1.0 + 1.2 / hh, acc[3])
    paint(img, dm <= 1.0, pal["frame"][0])
    core = dm <= 1.0 - 1.0 / hh
    cells = 3
    u = ((x + 0.5 - cx) / hw + (y + 0.5 - cy) / hh + 1) / 2 * cells
    v = ((y + 0.5 - cy) / hh - (x + 0.5 - cx) / hw + 1) / 2 * cells
    checker = (np.floor(u).astype(int) + np.floor(v).astype(int)) % 2 == 0
    light, dark = art.TOPS["light"], art.TOPS["dark"]
    paint(img, core & checker, light["base"])
    paint(img, core & ~checker, dark["base"])
    du = cells / (2 * hh)
    grid = core & ((np.abs(u - np.round(u)) < du * 0.5) | (np.abs(v - np.round(v)) < du * 0.5))
    paint(img, grid & checker, light["edge"])
    paint(img, grid & ~checker, dark["edge"])
    # заклёпки 4×4 с бликом
    for rx, ry in ((16, 16), (CW - 20, 16), (16, CH - 20), (CW - 20, CH - 20)):
        img[ry : ry + 4, rx : rx + 4] = (*hexc(p[3]), 255)
        img[ry : ry + 2, rx : rx + 2] = (*hexc(p[4]), 255)
        img[ry + 3, rx + 1 : rx + 4] = (*hexc(p[0]), 255)
        img[ry + 1 : ry + 4, rx + 3] = (*hexc(p[0]), 255)
    return img


def art_upscaled(name):
    """Картинка окна 76×50 — картинка 1x, увеличенная scale2x."""
    small = np.array(Image.open(os.path.join(ART_1X, f"{name}.png")).convert("RGBA"))
    return bc.scale2x(small)


def piece_sprite(code):
    """Спрайт фигуры игры (`pieces/<код>.png`), обрезанный по непрозрачному."""
    a = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA"))
    ys, xs = np.nonzero(a[..., 3])
    return a[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]


def put_sprite(img, spr, x0, y0):
    m = spr[..., 3] > 0
    h, w = img.shape[:2]
    for y, x in zip(*np.nonzero(m)):
        if 0 <= y0 + y < h and 0 <= x0 + x < w:
            img[y0 + y, x0 + x] = spr[y, x]


def night_sky(img, horizon, glow):
    """Ночное небо тремя полосами до горизонта, на стыках — шахматный переход в пиксель."""
    yy, xx = np.mgrid[0 : img.shape[0], 0 : img.shape[1]]
    colors = ["#11141f", "#161a27", glow]
    bounds = [0, horizon // 3, 2 * horizon // 3, horizon]
    for k, col in enumerate(colors):
        img[(yy >= bounds[k]) & (yy < bounds[k + 1])] = (*hexc(col), 255)
        if k:
            img[(yy == bounds[k]) & ((xx + yy) % 2 == 0)] = (*hexc(colors[k - 1]), 255)
            img[(yy == bounds[k] - 1) & ((xx + yy) % 2 == 1)] = (*hexc(col), 255)


def dark_board(img, horizon, cy, edge):
    """Тёмная доска ромбами от горизонта вниз (клетка 32×16 — вдвое мельче, чем у доски 1x)."""
    h, w = img.shape[:2]
    yy, xx = np.mgrid[0:h, 0:w]
    ground = yy >= horizon
    gu = (xx + 0.5 - w / 2) / 16 + (yy + 0.5 - cy) / 8
    gv = (yy + 0.5 - cy) / 8 - (xx + 0.5 - w / 2) / 16
    checker = (np.floor(gu).astype(int) + np.floor(gv).astype(int)) % 2 == 0
    img[ground & checker] = (*hexc("#2b2320"), 255)
    img[ground & ~checker] = (*hexc("#221c1b"), 255)
    # кромки клеток — тонкие светлые линии, как стыки блоков на доске
    du = 1 / 16
    seam = ground & ((np.abs(gu - np.round(gu)) < du) | (np.abs(gv - np.round(gv)) < du))
    img[seam] = (*hexc("#33292a"), 255)
    img[yy == horizon] = (*hexc(edge), 255)
    return ground


def art_fire_wave(seed=5):
    """Окно 76×50: белая пешка (спрайт игры) в кольце огня на тёмной доске, вдвое подробнее."""
    w, h = ART_SIZE
    rng = np.random.default_rng(seed)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    px, py = xx + 0.5, yy + 0.5
    horizon = 18
    night_sky(img, horizon, "#2a1a1e")
    ground = dark_board(img, horizon, 34, "#3d2420")
    cx, cy = w / 2, 34.0
    d = np.abs(px - cx) / 31.0 + np.abs(py - cy) / 14.0
    img[(d < 0.78) & ground] = (*hexc("#1a1312"), 255)
    img[(d < 0.7) & ground & ((xx + yy) % 2 == 0)] = (*hexc("#221715"), 255)
    band = (d >= 0.78) & (d <= 1.0)
    img[band & (d > 0.94)] = (*fw.FIRE[1], 255)
    img[band & (d <= 0.94) & (d > 0.89)] = (*fw.FIRE[2], 255)
    img[band & (d <= 0.89) & (d > 0.83)] = (*fw.FIRE[3], 255)
    img[band & (d <= 0.83)] = (*fw.FIRE[5], 255)
    pawn = piece_sprite("wP")
    ph, pw = pawn.shape[:2]
    flames = np.zeros((h, w), bool)
    for x in range(w):
        col = np.nonzero(band[:, x])[0]
        if not len(col):
            continue
        for base, k in ((col.min(), 0.7), (col.max(), 0.8)):
            ht = (5 + 7 * abs(np.sin(x * 0.45 + rng.uniform(0, 0.6)))) * k
            if x % 6 in (0, 1):
                ht += 4
            if base == col.max() and abs(x + 0.5 - cx) < pw / 2 + 4:
                ht = min(ht, 4.0)
            top = max(0, int(round(base - ht)))
            flames[top : base + 1, x] = True
    edge = burn.boundary(flames)
    inner = burn.boundary(flames & ~edge)
    fcol = np.zeros((h, w, 4), np.uint8)
    fcol[flames] = (*fw.FIRE[1], 255)
    fcol[inner] = (*fw.FIRE[3], 255)
    fcol[edge] = (*fw.FIRE[5], 255)
    for x in range(w):
        col = np.nonzero(flames[:, x])[0]
        if len(col) > 5:
            fcol[col[0], x, :3] = fw.FIRE[7]
            fcol[col[1], x, :3] = fw.FIRE[6]
    behind = py < cy
    ring_out = burn.neighbours_any(flames) & ~flames
    for part in (behind, ~behind):
        m = flames & part
        img[ring_out & part] = (*fw.OUTLINE, 255)
        img[m] = fcol[m]
        if part is behind:
            put_sprite(img, pawn, int(cx - pw / 2), int(cy + 5 - ph))
    for _ in range(16):  # искры: светлое ядро и тёмный хвост вниз
        x, y = int(rng.integers(2, w - 2)), int(rng.integers(1, horizon + 6))
        img[y, x] = (*fw.FIRE[rng.choice([1, 3])], 255)
        img[y + 1, x] = (*fw.FIRE[5], 255)
    return img


def arrow_mask(h, w, cx, cy, length, right):
    """Стрелка знака обмена: древко в два пикселя, наконечник-треугольник."""
    yy, xx = np.mgrid[0:h, 0:w]
    x0, x1 = cx - length / 2, cx + length / 2
    tip = x1 if right else x0
    head = 7.0
    along = (xx + 0.5 - tip) * (1 if right else -1)  # 0 — у острия, отрицательно — к древку
    shaft = (np.abs(yy + 0.5 - cy) <= 1.0) & (xx + 0.5 >= x0) & (xx + 0.5 <= x1)
    tri = (along <= 0) & (along >= -head) & (np.abs(yy + 0.5 - cy) <= -along * 0.62 + 0.4)
    return shaft | tri


def art_swap(seed=9):
    """Окно 76×50: две белые пешки (спрайты игры) на порталах и знак обмена ⇄ между ними."""
    w, h = ART_SIZE
    rng = np.random.default_rng(seed)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    horizon = 24
    night_sky(img, horizon, "#1e1636")
    dark_board(img, horizon, 36, "#2e2244")
    for _ in range(10):  # звёзды — в верхней части неба
        x, y = int(rng.integers(1, w - 1)), int(rng.integers(0, 10))
        img[y, x] = (*hexc("#a6f2ff" if rng.random() < 0.6 else "#57d9ff"), 255)
    feet = [(15.0, 39.0), (61.0, 39.0)]
    for fx, fy in feet:
        ex, ey = (xx + 0.5 - fx) / 13.0, (yy + 0.5 - fy) / 5.2
        r = np.hypot(ex, ey)
        ang = np.arctan2(ey, ex)
        img[r <= 1.18] = (*hexc("#0b1622"), 255)
        img[r <= 1.0] = (*hexc(pt.RIM["outer"]), 255)
        img[r <= 0.82] = (*hexc(pt.VORTEX[2]), 255)
        # вихрь: две спиральные полосы к центру
        swirl = (r <= 0.82) & (((ang * 2 + r * 7) % (2 * np.pi)) < 1.3)
        img[swirl] = (*hexc(pt.VORTEX[1]), 255)
        img[r <= 0.4] = (*hexc(pt.VORTEX[0]), 255)
        img[(r > 0.82) & (r <= 1.0) & (ey < 0)] = (*hexc(pt.RIM["inner"]), 255)
    pawn = piece_sprite("wP")
    ph, pw = pawn.shape[:2]
    for fx, fy in feet:
        put_sprite(img, pawn, int(round(fx - pw / 2)), int(round(fy + 3 - ph)))
    for right, cy in ((True, 19.0), (False, 30.0)):
        shape = arrow_mask(h, w, w / 2, cy, 17.0, right)
        img[bc.outline4(shape)] = (*hexc("#08090d"), 255)
        img[shape] = (*hexc("#57d9ff"), 255)
        top = shape & ~np.vstack([np.zeros((1, w), bool), shape[:-1]])
        img[top] = (*hexc("#a6f2ff"), 255)
    return img


def art_bless(seed=21):
    """Окно 76×50: белый слон с крыльями (крылья — `spread_wing` вдвое крупнее) и нимбом,
    лучи и кольцо света."""
    w, h = ART_SIZE
    rng = np.random.default_rng(seed)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    horizon = 24
    night_sky(img, horizon, "#2e2820")
    ang = np.degrees(np.arctan2(yy + 0.5 - 16, xx + 0.5 - 38)) % 360
    rays = (yy < horizon) & ((ang // 12) % 2 == 0) & (np.hypot(xx - 38, (yy - 16) * 1.3) < 40)
    img[rays & (yy < 8)] = (*hexc("#1a1c28"), 255)
    img[rays & (yy >= 8) & (yy < 16)] = (*hexc("#262230"), 255)
    img[rays & (yy >= 16)] = (*hexc("#3d3424"), 255)
    ground = dark_board(img, horizon, 36, "#4a3c24")
    d = np.abs(xx + 0.5 - 38) / 30.0 + np.abs(yy + 0.5 - 39) / 7.2
    img[ground & (d < 0.8) & ((xx + yy) % 2 == 0)] = (*hexc("#4a3c24"), 255)
    img[ground & (d >= 0.8) & (d <= 1.0)] = (*hexc(bl.LIGHT[2]), 255)
    img[ground & (d >= 0.9) & (d <= 1.0)] = (*hexc(bl.LIGHT[1]), 255)
    img[ground & (d >= 0.96) & (d <= 1.0)] = (*hexc(bl.LIGHT[0]), 255)
    bishop = bc.scale2x(bl.icon(bl.BISHOP_ICON, fw.PAWN_COLORS))
    bh, bw = bishop.shape[:2]
    bx, by = 38 - bw // 2, 43 - bh
    for right, sx in ((False, bx + 8), (True, bx + bw - 8)):
        spr, (ax, ay) = bl.spread_wing(right, bl.WING_CARD, scale=2.0)
        put_sprite(img, spr, sx - ax, by + 18 - ay)
    put_sprite(img, bishop, bx, by)
    # нимб — прямо над митрой
    top = np.nonzero(bishop[..., 3].any(axis=1))[0].min() + by
    hy = top - 2.5
    r = np.hypot((xx + 0.5 - 39) / 7.4, (yy + 0.5 - hy) / 2.6)
    halo = np.abs(r - 1.0) < 0.2
    img[halo] = (*hexc(bl.LIGHT[2]), 255)
    img[halo & (yy + 0.5 < hy)] = (*hexc(bl.LIGHT[1]), 255)
    for _ in range(14):
        x, y = int(rng.integers(1, w - 1)), int(rng.integers(1, horizon + 10))
        if img[y, x, :3].sum() < 200:
            img[y, x] = (*hexc(bl.SPARK[int(rng.integers(0, 3))]), 255)
    return img


def art_hd(name):
    """Картинка окна 76×50: огненная волна, обмен и исцеление нарисованы заново вдвое
    подробнее, договор в крови — картинка 1x через scale2x."""
    draw = {"fire_wave": art_fire_wave, "swap": art_swap, "bless": art_bless}.get(name)
    return draw() if draw else art_upscaled(name)


def with_art(face, art_img):
    out = face.copy()
    x0, y0 = ART_AT
    out[y0 : y0 + art_img.shape[0], x0 : x0 + art_img.shape[1]] = art_img
    return out


# ---------------------------------------------------------------- сжигание
def up2(img):
    return img.repeat(2, axis=0).repeat(2, axis=1)


def over(dst, src):
    """Наложение src на dst по прозрачности (без премультипликации)."""
    a = src[..., 3:4] / 255.0
    da = dst[..., 3:4] / 255.0
    out_a = a + da * (1 - a)
    rgb = np.where(out_a > 0, (src[..., :3] * a + dst[..., :3] * da * (1 - a)) / np.maximum(out_a, 1e-6), 0)
    dst[..., :3] = rgb.astype(np.uint8)
    dst[..., 3] = (out_a[..., 0] * 255).round().astype(np.uint8)


def puff_layer(p):
    """Клуб дыма отдельным слоем 1x (как `burn.draw_puff`, но без смешивания)."""
    layer = np.zeros((burn.FH, burn.FW, 4), np.uint8)
    canvas = np.zeros((burn.FH, burn.FW, 4), np.uint8)
    canvas[..., 3] = 0
    burn.draw_puff(canvas, p)
    # draw_puff смешивает с прозрачным — восстанавливаем цвет: альфа слоя известна
    m = canvas[..., 3] > 0
    k = canvas[..., 3:4] / 255.0
    layer[..., :3] = np.where(m[..., None], (canvas[..., :3] / np.maximum(k, 1e-6)).clip(0, 255), 0).astype(np.uint8)
    layer[..., 3] = canvas[..., 3]
    return layer


def render_burn(face1, face2, seed=7):
    """Кадры сжигания 148×220: карта 2x горит по карте прогорания 1x (увеличенной), огонь,
    дым, пепел и искры — как в `burn.render`, пикселями 1x."""
    rng = np.random.default_rng(seed)
    rng2 = np.random.default_rng(seed + 1)
    tb, (ix, iy) = burn.burn_times(seed)
    tb2 = up2(tb)
    total_t = 1.0 + burn.GLOW_T + burn.CHAR_T + burn.SCORCH_T
    embers, smoke, ash = burn.Particles(), burn.Particles(), burn.Particles()
    FW, FH, PX, PT = burn.FW, burn.FH, burn.PAD_X, burn.PAD_TOP
    frames = []
    for f in range(burn.FRAMES):
        t = (f - burn.IGNITE) / burn.BURN_FRAMES * total_t - burn.SCORCH_T if f >= burn.IGNITE else -1.0
        _, _, burning, _ = burn.card_layer(face1, tb, t, rng)
        card2, _, _, _ = burn.card_layer(face2, tb2, t, rng2)
        frame = np.zeros((FH * 2, FW * 2, 4), np.uint8)
        frame[PT * 2 : PT * 2 + CH, PX * 2 : PX * 2 + CW] = card2
        fx = np.zeros((FH, FW, 4), np.uint8)  # искра поджога, огонь, пепел и искры — 1x
        if f in (burn.IGNITE - 1, burn.IGNITE):
            r = 1 if f == burn.IGNITE - 1 else 3
            yy, xx = np.mgrid[0:FH, 0:FW]
            dd = np.abs(xx - (ix + PX)) + np.abs(yy - (iy + PT))
            star = dd <= r
            if f == burn.IGNITE:
                star |= ((xx == ix + PX) & (np.abs(yy - iy - PT) <= 5)) | ((yy == iy + PT) & (np.abs(xx - ix - PX) <= 5))
            fx[burn.neighbours_any(star) & ~star] = (*burn.OUTLINE, 255)
            fx[star] = (*burn.FIRE[0], 255)
        bys, bxs = np.nonzero(burning)
        if len(bys):
            for _ in range(min(4, len(bys) // 6 + 1)):
                k = rng.integers(len(bys))
                embers.items.append(dict(x=bxs[k] + PX + 0.5, y=bys[k] + PT - 1.0, vx=rng.uniform(-0.5, 0.5),
                                         vy=rng.uniform(-5.5, -3.6), drag=0.88, life=int(rng.integers(6, 11)), age=0,
                                         wob=0.35, ph=rng.uniform(0, 6)))
            if f % 2 == 0:
                top = bys.min()
                xs_top = bxs[bys <= top + 3]
                smoke.items.append(dict(x=rng.choice(xs_top) + PX + rng.uniform(-2, 2), y=top + PT - 9.0,
                                        vx=rng.uniform(-0.4, 0.4), vy=rng.uniform(-4.2, -3.2), drag=0.86,
                                        life=int(rng.integers(10, 15)), age=0, r0=rng.uniform(2.0, 3.0), wob=0.3,
                                        ph=rng.uniform(0, 6)))
        if f >= burn.IGNITE:
            prev_t = (f - 1 - burn.IGNITE) / burn.BURN_FRAMES * total_t - burn.SCORCH_T
            fresh = (face1[..., 3] > 0) & (t - tb >= burn.GLOW_T + burn.CHAR_T) & (prev_t - tb < burn.GLOW_T + burn.CHAR_T)
            fys, fxs = np.nonzero(fresh)
            for k in range(len(fys)):
                if rng.random() < 0.008:
                    ash.items.append(dict(x=fxs[k] + PX + 0.5, y=fys[k] + PT + 0.5, vx=rng.uniform(-0.3, 0.3),
                                          vy=rng.uniform(0.4, 1.0), life=int(rng.integers(6, 11)), age=0, wob=0.6,
                                          ph=rng.uniform(0, 6), big=rng.random() < 0.6))
        for p in smoke.items:
            life = p["age"] / p["life"]
            grow = p["r0"] + 4.2 * min(life / 0.6, 1.0)
            p["r"] = grow * (1.0 if life < 0.65 else max(0.0, 1 - (life - 0.65) / 0.35))
            if p["r"] >= 0.8:
                over(frame, up2(puff_layer(p)))
        flames = burn.flame_layer(burning, f, rng)
        m = flames[..., 3] > 0
        fx[m] = flames[m]
        for p in ash.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW - 1 and 0 <= y < FH:
                col = burn.ASH[0] if p["age"] < p["life"] * 0.6 else burn.ASH[1]
                fx[y, x] = (*col, 255)
                if p["big"]:
                    fx[y, x + 1] = (*burn.ASH[1], 255)
        for p in embers.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                life = p["age"] / p["life"]
                fx[y, x] = (*burn.FIRE[[1, 3, 5, 7][min(int(life * 4), 3)]], 255)
                if p["age"] < 2 and y + 1 < FH:
                    fx[y + 1, x] = (*burn.FIRE[5], 255)
        big = up2(fx)
        m = big[..., 3] > 0
        frame[m] = big[m]
        frames.append(frame)
        for ps in (embers, smoke, ash):
            ps.step()
    return frames


# ---------------------------------------------------------------- сохранение
def save(img, *path):
    full = os.path.join(OUT, *path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    Image.fromarray(img, "RGBA").save(full)


def main():
    faces = {}
    for kind in bc.CARD_KINDS:
        for selected in (False, True):
            name = f"front_{kind}" + ("_selected" if selected else "")
            faces[name] = card_front(kind, selected)
            save(faces[name], f"{name}.png")
    for f in "wb":
        save(card_back(f), f"back_{f}.png")
    arts = {name: art_hd(name) for name in ARTS}
    for name, img in arts.items():
        save(img, "art", f"{name}.png")
    # сжигание: лица без картинки (пустое окно) и лица с картинкой
    burns = {f"plain_{k}": (k, None) for k in bc.CARD_KINDS}
    burns.update({
        "fire_wave_w": ("w", "fire_wave"),
        "fire_wave_b": ("b", "fire_wave"),
        "swap_victory": ("victory", "swap"),
        "funding_sacrifice": ("sacrifice", "contract_blood"),
        "heal_victory": ("victory", "bless"),
    })
    for name, (kind, art_name) in burns.items():
        face1 = bc.card_front(kind)
        face2 = faces[f"front_{kind}"] if art_name is None else with_art(faces[f"front_{kind}"], arts[art_name])
        strip = np.concatenate(render_burn(face1, face2), axis=1)
        save(strip, "burn", f"burn_strip_{name}.png")
    # превью: лица 1x и 2x рядом, увеличено вдвое
    sheet_faces = [faces["front_w"], faces["front_b"], faces["front_sacrifice"], faces["front_victory"],
                   with_art(faces["front_w"], arts["fire_wave"]), with_art(faces["front_victory"], arts["swap"]),
                   with_art(faces["front_sacrifice"], arts["contract_blood"]), card_back("w")]
    gap = 10
    sheet = Image.new("RGBA", (len(sheet_faces) * (CW + gap) + gap, CH + 2 * gap), hexc("#1b1e27") + (255,))
    for n, img in enumerate(sheet_faces):
        sheet.alpha_composite(Image.fromarray(img, "RGBA"), (gap + n * (CW + gap), gap))
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(os.path.join(OUT, "preview_cards_hd.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
