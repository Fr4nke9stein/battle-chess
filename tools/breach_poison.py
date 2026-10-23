"""Ядовитые фигуры: облиты зелёной жидкостью, капают в лужу, оставляют след — в стиле Into the Breach.

Спрайты — те же 3D-фигуры `breach_art.py` с материалом `SLIME` прямо на теле (свет и тени —
как на остальном): жидкость облила фигуру сверху — шапка на макушке (`COAT` высоты) с неровной
кромкой, вниз по бокам стекают потёки разной длины (`drips`: полуэллипс по азимуту, к концу
уже), на концах длинных — набухшие капли-шарики; на верхних уступах — лужицы. Отделка и
огоньки — как у сторон, сквозь жидкость светятся огни. Самые светлые пиксели жидкости — блики.

Анимация (замена спрайта, холст FW×FH = 120×120, опора (FAX, FAY) = (60, 92), как у
`electric_*`, 60 мс): под фигурой лужа (`puddle`: неровный эллипс 4:3 по клетке, не выходит за
ромб, кромка тёмная, дальний край светлее, блики), вступление [0, INTRO) — лужа растекается;
повтор [INTRO, FRAMES): с концов потёков набухают и падают капли (до трёх, `drop_sources`),
в луже — всплеск и круги, по потёкам сбегает блик, иногда в луже лопается пузырь, кромка
лужи медленно колышется. Тень — обычная тень фигуры (`poison/<код>_shadow.png`).

След (шаг на клетку): полоса TRW×TRH = 96×80, середина между центрами клеток — (TRX, TRY) =
(48, 40); `poison_trail_<направление>.png` — ne (вверх-вправо, к восьмой горизонтали без
переворота доски), sw, nw, se; на экране сдвиг клетки (±28, ±21). Рисовать до фигур (как
`bless_under`). След — капля (по рисунку пользователя): большая круглая голова там, куда фигура
встала (под её лужей), хвост плавно сужается к исходной клетке и кончается остриём чуть за её
центром. Кадры [0, TRAIL_MOVE) — фигура скользит (SLIDE_TIME игры 0,2 с), капля вытягивается за
ней; дальше [TRAIL_MOVE, TRAIL_FRAMES) хвост втягивается от острия к голове, след темнеет,
распадается на пятна и пропадает, от него поднимается зелёный пар. Лужа самой фигуры едет
вместе с ней (она в кадрах анимации).

`py tools/breach_poison.py [папка ассетов]` — по умолчанию assets/breach: pieces/poison/<код>.png и
_shadow.png, effects/poison_<код>.png (все 12), effects/poison_trail_<ne|sw|nw|se>.png; превью
pieces/preview_poison_pieces.png, effects/preview_poison.gif (все фигуры),
effects/preview_poison_step.gif (волна ядовитых шагает вперёд), effects/preview_poison_trail.png (кадры
следа), effects/preview_poison_frames.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_arcs as ba  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_firewave as fw  # noqa: E402
import breach_ice_pieces as ip  # noqa: E402
from breach_art import MAIN, TRIM, hexc, sphere  # noqa: E402

sys.argv = _argv
ROOT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach")
PIECES_OUT = os.path.join(ROOT, "pieces")
EFFECTS_OUT = os.path.join(ROOT, "effects")

SLIME = 5  # новый материал — ядовитая жидкость
SLIME_RAMP = ["#0b2a0e", "#1f6a18", "#3fae22", "#7ce83a", "#c8ff7a"]
GLINT = hexc("#efffc8")
# лужа и след: контур, глубина, жидкость, светлая кромка, блик, искра
P_OUT, P_DEEP, P_MID, P_LIGHT, P_SHINE = (hexc(c) for c in ["#0b2a0e", "#1f6a18", "#3fae22", "#7ce83a", "#c8ff7a"])
COAT = 0.2  # шапка жидкости — доля высоты фигуры
FRONT = 45.0  # азимут к зрителю

FW, FH = ba.FW, ba.FH  # 120×120
FAX, FAY = ba.FAX, ba.FAY  # центр клетки
SX, SY = FAX - art.AX, FAY - art.AY
FRAME_MS = 60
INTRO = 8
LOOP = 32  # 1,9 с
LOOP_START, FRAMES = INTRO, INTRO + LOOP
PUDDLE_RX = 17.0  # полуось лужи по x; по y — ×0.75 (ромб клетки 56×42), с волнами кромки — в ромбе
HALF_TW, HALF_TH = art.TW / 2, art.TH / 2
GRAVITY = 1.5  # пикселей за кадр²

TRW, TRH = 96, 80
TRX, TRY = 48, 40
TRAIL_MOVE = 4  # кадров скольжения (0,2 с)
TRAIL_FRAMES = 30
TRAIL_HEAD = 14.0  # голова капли (в осях клетки) — под лужей фигуры, видно хвост
TAIL_POW = 1.4  # хвост сужается вогнуто, как на рисунке
TAIL_OVER = 0.25  # остриё — за центром исходной клетки, на эту долю шага
DIRECTIONS = {"ne": (28, -21), "sw": (-28, 21), "nw": (-28, -21), "se": (28, 21)}


def put(img, mask, col, alpha=255):
    img[mask] = (*col, alpha)


def shifted(m, dy, dx):
    out = np.zeros_like(m)
    h, w = m.shape
    out[max(dy, 0) : h + min(dy, 0), max(dx, 0) : w + min(dx, 0)] = m[max(-dy, 0) : h - max(dy, 0),
                                                                      max(-dx, 0) : w - max(dx, 0)]
    return out


# ---------------------------------------------------------------- спрайты
def part_dist(piece, p):
    return np.min(np.stack([f(p) for f, _ in piece.parts]), axis=0)


def top_of(piece):
    """Высота макушки фигуры (в координатах частей, без вытягивания)."""
    xs = np.arange(-16, 16.01, 1.0)
    ys = np.arange(0, 70, 0.25)
    gx, gy, gz = np.meshgrid(xs, ys, xs, indexing="ij")
    p = np.stack([gx.ravel(), gy.ravel(), gz.ravel()], axis=1)
    inside = part_dist(piece, p) < 0
    return p[inside, 1].max()


def surface_r(piece, y, az):
    """Радиус поверхности фигуры на высоте y по азимуту az (None — ось снаружи)."""
    rs = np.arange(0, 20, 0.1)
    pts = np.stack([ip.at(r, y, az) for r in rs])
    d = part_dist(piece, pts)
    if d[0] > 0:
        return None
    out = np.nonzero(d > 0)[0]
    return rs[out[0]] if len(out) else None


def slimy(part, seed, top, drips):
    """Часть фигуры, облитая сверху: шапка с неровной кромкой, потёки, лужицы на уступах."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        out = base.copy()
        free = (base == MAIN) | (base == TRIM)  # огни светят сквозь жидкость
        az = np.degrees(np.arctan2(p[:, 2], p[:, 0]))
        r = np.maximum(np.hypot(p[:, 0], p[:, 2]), 2.0)
        cap = COAT * top
        depth = cap + 1.6 * (ip.noise3(p, 3.0, seed) - 0.5)
        for a, length, width in drips:
            d = (((az - a + 180) % 360) - 180) * math.pi / 180 * r
            k = np.clip(1 - (d / width) ** 2, 0, 1)
            depth = np.maximum(depth, cap + length * np.sqrt(k))
        coat = p[:, 1] > top - depth
        e = 0.08
        ny = (f(p + [0, e, 0]) - f(p - [0, e, 0])) / (2 * e)
        pool = (ny > 0.85) & (ip.noise3(p, 2.5, seed + 3) > 0.66)
        out[free & (coat | pool)] = SLIME
        return out

    return f, mat


def poisoned(kind, color):
    piece = art.PIECES[kind]()
    seed = 31 + "PNBRQK".index(kind) * 7 + (0 if color == "w" else 3)
    rng = np.random.default_rng(seed)
    top = top_of(piece)
    drips = []
    for a in np.linspace(FRONT - 85, FRONT + 85, 6):  # к зрителю — разной длины
        drips.append((a + rng.uniform(-9, 9), rng.uniform(0.12, 0.55) * top, rng.uniform(1.1, 2.0)))
    for a in (FRONT + 150, FRONT + 210):  # сзади — для силуэта по краям
        drips.append((a, rng.uniform(0.1, 0.4) * top, rng.uniform(1.2, 2.0)))
    parts = [slimy(p, seed + k, top, drips) for k, p in enumerate(piece.parts)]
    extra = []
    for a, length, _ in drips:  # набухшие капли на концах длинных потёков
        if length < 0.3 * top:
            continue
        y = top - COAT * top - length + 0.7
        r = surface_r(piece, y, a)
        if r is not None:
            extra.append((sphere(ip.at(r - 0.25, y, a), 1.05), SLIME))
    return art.Piece(parts + extra, stretch=piece.stretch)


def render_sprite(kind, color):
    key = f"poison_{color}"
    pal = dict(art.PALETTES[color])
    pal[SLIME] = SLIME_RAMP
    art.PALETTES[key] = pal
    body, shadow = art.render_piece(poisoned(kind, color), key, 0.86)
    img = np.array(body)
    top = np.array(hexc(SLIME_RAMP[4]))
    bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:4]:
        img[y, x, :3] = GLINT
    return Image.fromarray(img, "RGBA"), shadow


# ---------------------------------------------------------------- лужа
def in_diamond(xx, yy, cx, cy, inset=1.0):
    return np.abs(xx + 0.5 - cx) / HALF_TW + np.abs(yy + 0.5 - cy) / HALF_TH <= inset


def blob(xx, yy, cx, cy, rx, phases, wobble=0.0):
    """Неровный эллипс 4:3: радиус колеблется по углу; wobble — фаза медленного колыхания."""
    dx, dy = (xx + 0.5 - cx) / rx, (yy + 0.5 - cy) / (rx * 0.75)
    th = np.arctan2(dy, dx)
    r = 1 + 0.09 * np.sin(3 * th + phases[0]) + 0.06 * np.sin(5 * th + phases[1]) \
        + 0.035 * np.sin(2 * th + phases[2] + wobble)
    return np.hypot(dx, dy) < r


def paint_liquid(img, mask, cy, shine=1.0, dark=0.0):
    """Жидкость на земле: тёмный контур, дальняя кромка светлее, ближняя — глубже."""
    if not mask.any():
        return
    yy = np.mgrid[0 : mask.shape[0], 0 : mask.shape[1]][0]
    edge = burn.boundary(mask)
    inner = mask & ~edge
    rim = burn.boundary(inner)
    put(img, inner, P_MID if dark < 0.5 else P_DEEP)
    put(img, rim & (yy < cy), P_LIGHT if shine > 0.5 else P_MID)
    put(img, rim & (yy >= cy), P_DEEP)
    put(img, edge, P_OUT)


def ring(img, under, cx, cy, rx, ry, col):
    """Круг по луже (контур эллипса), только поверх жидкости."""
    yy, xx = np.mgrid[0 : img.shape[0], 0 : img.shape[1]]
    d = np.hypot((xx + 0.5 - cx) / rx, (yy + 0.5 - cy) / ry)
    m = (np.abs(d - 1) < 0.5 / max(rx, ry) * 1.6) & under
    put(img, m, col)


# ---------------------------------------------------------------- анимация фигуры
def drop_sources(canvas, slime, sil, rng):
    """Концы потёков, откуда падают капли: нижние пиксели жидкости над телом, разнесены по x."""
    h, w = slime.shape
    tips = slime & ~shifted(slime, -1, 0) & shifted(sil, -1, 0)
    base_y = np.nonzero(sil)[0].max()
    ys, xs = np.nonzero(tips)
    # концы на теле (выше основания) — сначала нижние; лужицы на уступах основания — нет
    body_tips = sorted(((y, x) for y, x in zip(ys, xs) if y < base_y - 13 and slime[y, x + 1]), key=lambda q: -q[0])
    out = []
    for y, x in body_tips:
        if all(abs(x - ox) >= 6 for _, ox in out):
            out.append((y, x))
            if len(out) == 3:
                break
    return out


def render_idle(code, sprite):
    rng = np.random.default_rng(101 + "PNBRQK".index(code[1]) + (0 if code[0] == "w" else 50))
    body = np.zeros((FH, FW, 4), np.uint8)
    body[SY : SY + sprite.shape[0], SX : SX + sprite.shape[1]] = sprite
    sil = body[..., 3] > 0
    colors = [np.array(hexc(c)) for c in SLIME_RAMP] + [np.array(GLINT)]
    slime = np.zeros(sil.shape, bool)
    for c in colors:
        slime |= (body[..., :3] == c).all(axis=-1) & sil
    sources = drop_sources(body, slime, sil, rng)
    land = {x: np.nonzero(sil[:, x])[0].max() + 2 for _, x in sources}
    # блик сбегает по потёку: столбец жидкости над концом
    runs = []
    for y, x in sources[:2]:
        col = np.nonzero(slime[: y + 1, x])[0]
        runs.append((x, col.min(), y))
    yy, xx = np.mgrid[0:FH, 0:FW]
    phases = rng.uniform(0, 2 * math.pi, 3)
    # брызги рядом с лужей (спереди и по бокам): лужа — пролитая жидкость, а не подставка
    splats = [(a, rng.uniform(1.12, 1.3), rng.uniform(1.4, 2.3))
              for a in rng.uniform(0.15, math.pi - 0.15, 3)]
    cx, cy = FAX, FAY + 1.5  # чуть вперёд: спереди лужу видно, сзади её закрывает фигура
    diamond = in_diamond(xx, yy, FAX, FAY)
    visible = ~sil
    frames = []
    for f in range(FRAMES):
        lf = f - LOOP_START
        grow = 0.3 + 0.7 * (1 - (1 - (f + 1) / INTRO) ** 2) if f < INTRO else 1.0
        wob = 2 * math.pi * (lf % LOOP) / LOOP if lf >= 0 else 0.0
        img = np.zeros((FH, FW, 4), np.uint8)
        pool = blob(xx, yy, cx, cy, PUDDLE_RX * grow, phases, wob)
        for a, dist, rad in splats:
            sx = cx + math.cos(a) * PUDDLE_RX * dist * grow
            sy = cy + math.sin(a) * PUDDLE_RX * 0.75 * dist * grow
            pool |= np.hypot(xx + 0.5 - sx, (yy + 0.5 - sy) / 0.75) < rad * min(1.0, grow * 1.2)
        pool &= diamond
        paint_liquid(img, pool, cy)
        # блики на луже по бокам от фигуры — мерцают
        for bx, by, ln in ((-15, 1, 3), (12, 5, 2), (-7, 9, 2)):
            seg = pool & (yy == round(cy + by)) & (xx >= cx + bx) & (xx < cx + bx + ln) & visible
            on = lf < 0 or (lf + bx) % 16 < 11
            put(img, seg & ~burn.boundary(pool), GLINT if on else P_SHINE)
        over = []  # (y, x, цвет) поверх тела
        if lf >= 0:
            # пузырь в луже: где видно жидкость, раз за повтор
            spots = np.argwhere(pool & visible & ~burn.boundary(pool) & ~burn.boundary(pool & ~burn.boundary(pool)))
            if len(spots):
                by, bx = spots[(len(spots) * 3) // 7]
                t = (lf - 19) % LOOP
                if t == 0:
                    over.append((by, bx, P_SHINE))
                elif t in (1, 2, 3):
                    ring(img, pool, bx + 0.5, by + 0.5, 1.6, 1.2, P_SHINE)
                    img[by, bx] = (*P_DEEP, 255)
                elif t == 4:
                    ring(img, pool, bx + 0.5, by + 0.5, 2.6, 1.9, P_LIGHT)
                elif t == 5:
                    for dx, dy in ((-2, -1), (2, -1), (-1, -2), (1, -2)):
                        over.append((by + dy, bx + dx, P_SHINE))
            # капли: шириной 2 пикселя, с тёмной обводкой по бокам — видны и на светлых фигурах
            for k, (ty, tx) in enumerate(sources):
                t = (lf - k * 11) % LOOP
                gy = land[tx]
                fall_h = gy - (ty + 3)
                tf = math.ceil(math.sqrt(2 * max(fall_h, 1) / GRAVITY))

                def blob_row(y, col, edge=True):
                    over.extend([(y, tx, col), (y, tx + 1, col)])
                    if edge:
                        over.extend([(y, tx - 1, P_OUT), (y, tx + 2, P_OUT)])

                if t < 9:  # набухает на конце потёка
                    if t >= 2:
                        blob_row(ty + 1, P_MID)
                    if t >= 5:
                        blob_row(ty + 1, SLIME_C[3])
                        blob_row(ty + 2, P_MID)
                    if t >= 7:
                        blob_row(ty + 3, P_DEEP)
                elif t < 9 + tf:  # падает: в полёте вытягивается
                    s = t - 9
                    y = round(ty + 3 + 0.5 * GRAVITY * s * s)
                    if y < gy:
                        tail = 3 if s * GRAVITY > 4 else 2
                        for k2 in range(tail):
                            col = SLIME_C[4] if k2 == 0 else P_MID if k2 == tail - 1 else SLIME_C[3]
                            blob_row(y - tail + 1 + k2, col)
                        over.append((y - tail, tx, P_OUT))
                        over.append((y - tail, tx + 1, P_OUT))
                        over.append((y + 1, tx, P_OUT))
                        over.append((y + 1, tx + 1, P_OUT))
                else:  # всплеск и круги
                    s = t - 9 - tf
                    cxs = tx + 1.0
                    if s == 0:
                        over += [(gy, tx - 1, SLIME_C[4]), (gy, tx, GLINT), (gy, tx + 1, GLINT),
                                 (gy, tx + 2, SLIME_C[4]), (gy - 1, tx, SLIME_C[3]), (gy - 1, tx + 1, SLIME_C[3])]
                    elif s == 1:
                        ring(img, pool, cxs, gy + 0.5, 2.5, 1.6, P_SHINE)
                        over += [(gy - 2, tx - 2, SLIME_C[4]), (gy - 2, tx + 3, SLIME_C[4]), (gy - 3, tx, SLIME_C[4])]
                    elif s == 2:
                        ring(img, pool, cxs, gy + 0.5, 3.5, 2.4, P_SHINE)
                        over += [(gy - 1, tx - 3, SLIME_C[3]), (gy - 1, tx + 4, SLIME_C[3]), (gy - 2, tx, SLIME_C[3])]
                    elif s == 3:
                        ring(img, pool, cxs, gy + 0.5, 4.5, 3.2, P_LIGHT)
                    elif s == 4:
                        ring(img, pool, cxs, gy + 0.5, 5.5, 3.8, P_LIGHT)
            # блик сбегает по потёку
            for k, (x, y0, y1) in enumerate(runs):
                n = y1 - y0 + 1
                if n >= 3:
                    pos = y0 + ((lf + k * 13) * n // 24) % (n + 6)
                    if pos <= y1:
                        over.append((pos, x, GLINT))
                        if pos - 1 >= y0:
                            over.append((pos - 1, x, SLIME_C[4]))
        img[sil] = body[sil]
        for y, x, col in over:
            if 0 <= y < FH and 0 <= x < FW:
                img[y, x] = (*col, 255)
        frames.append(img)
    return frames


SLIME_C = [hexc(c) for c in SLIME_RAMP]


# ---------------------------------------------------------------- след
def smooth_noise(h, w, cell, rng):
    gh, gw = h // cell + 2, w // cell + 2
    g = rng.random((gh, gw))
    yy, xx = np.mgrid[0:h, 0:w]
    y, x = yy / cell, xx / cell
    y0, x0 = y.astype(int), x.astype(int)
    fy, fx = y - y0, x - x0
    fy, fx = fy * fy * (3 - 2 * fy), fx * fx * (3 - 2 * fx)
    a = g[y0, x0] * (1 - fx) + g[y0, x0 + 1] * fx
    b = g[y0 + 1, x0] * (1 - fx) + g[y0 + 1, x0 + 1] * fx
    return a * (1 - fy) + b * fy


def render_trail(direction, seed=7):
    rng = np.random.default_rng(seed + list(DIRECTIONS).index(direction))
    dx, dy = DIRECTIONS[direction]
    ox, oy = TRX - dx / 2, TRY - dy / 2  # центр исходной клетки
    tx, ty = TRX + dx / 2, TRY + dy / 2
    yy, xx = np.mgrid[0:TRH, 0:TRW]
    tiles = in_diamond(xx, yy, ox, oy, 1.02) | in_diamond(xx, yy, tx, ty, 1.02)  # без щели на общей кромке
    phases = rng.uniform(0, 2 * math.pi, 3)
    noise = smooth_noise(TRH, TRW, 5, rng)
    wob = smooth_noise(TRH, TRW, 7, rng)
    k = 4 / 3  # по y клетка сжата: 56×42
    vapour = [(rng.uniform(0, 1), rng.uniform(0, 1), rng.integers(0, 14)) for _ in range(7)]
    frames = []
    for f in range(TRAIL_FRAMES):
        p = min(1.0, (f + 1) / TRAIL_MOVE)
        p = p * p * (3 - 2 * p)
        cx, cy = ox + dx * p, oy + dy * p
        # капля (рисунок пользователя): круглая голова там, куда фигура встала, хвост плавно
        # сужается к исходной клетке и кончается остриём чуть за её центром. Считается в осях клетки
        # (y × 4/3 — ромб 56×42 становится квадратом), голова — круг R, хвост — R · t^TAIL_POW,
        # t — от острия (0) до головы (1)
        sx, sy = ox - dx * TAIL_OVER, oy - dy * TAIL_OVER  # остриё
        vx, vy = cx - sx, (cy - sy) * k
        px, py = xx + 0.5 - sx, (yy + 0.5 - sy) * k
        ll = vx * vx + vy * vy
        t = (px * vx + py * vy) / ll if ll > 0 else np.zeros_like(px)
        perp = np.abs(px * vy - py * vx) / math.sqrt(ll) if ll > 0 else np.hypot(px, py)
        edge = 1.6 * (wob - 0.5)
        tail = (t >= 0) & (t <= 1) & (perp < TRAIL_HEAD * np.clip(t, 0, 1) ** TAIL_POW + edge * np.clip(t, 0, 1))
        head = np.hypot(px - vx, py - vy) < TRAIL_HEAD + edge
        mask = (tail | head) & tiles
        img = np.zeros((TRH, TRW, 4), np.uint8)
        if f >= TRAIL_MOVE:
            fade = (f - TRAIL_MOVE) / (TRAIL_FRAMES - TRAIL_MOVE - 1)
            # тает от острия к голове: хвост втягивается, края — раньше середины
            depth = np.minimum(ba.distance(~mask, 6), 6) / 6
            score = 0.62 * np.clip(t, 0, 1) + 0.22 * noise + 0.16 * depth
            mask &= score > fade * 1.02
        else:
            fade = 0.0
        paint_liquid(img, mask, TRY + 2, shine=1.0 - fade * 1.6, dark=fade * 1.4)
        if fade < 0.45:  # блики
            for at, ln in ((0.45, 2), (0.62, 3)):  # на хвосте, у верхнего края
                gx, gy = ox + dx * at, oy + dy * at
                seg = mask & (yy == round(gy - 1)) & (xx >= gx - 1) & (xx < gx - 1 + ln) & ~burn.boundary(mask)
                put(img, seg, GLINT if f % 8 < 5 else P_SHINE)
        # пар: точки поднимаются от пятен
        if fade > 0:
            pts = np.argwhere(mask & ~burn.boundary(mask))
            if len(pts):
                for u, v, start in vapour:
                    age = f - TRAIL_MOVE - start
                    if 0 <= age < 9:
                        y0, x0 = pts[int(u * (len(pts) - 1))]
                        y = y0 - age // 2 - 1
                        x = x0 + (1 if (age // 3) % 2 else 0)
                        if 0 <= y < TRH:
                            img[y, x] = (*(SLIME_C[4] if age < 4 else SLIME_C[3]), 200 - age * 18)
        frames.append(img)
    return frames


# ---------------------------------------------------------------- превью
def preview_sprites(sprites, path):
    cw, ch = 40, 72
    sheet = Image.new("RGBA", (12 * cw + 8, 2 * ch + 8), hexc("#2c3a2c") + (255,))
    sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), hexc("#3d4a3a") + (255,)), (0, 4 + ch))
    for r in range(2):
        for c, code in enumerate(color + k for color in "wb" for k in "PNBRQK"):
            if r == 0:
                body = Image.open(os.path.join(PIECES_OUT, f"{code}.png")).convert("RGBA")
            else:
                body = sprites[code][0]
            sheet.alpha_composite(body.crop((12, 6, 52, 78)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def save_gif(frames, path):
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 4))
    for i, k in enumerate(np.linspace(0, len(frames) - 1, 4).astype(int)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(path, save_all=True, append_images=gif[1:], duration=FRAME_MS, loop=0, disposal=1)


def tile_center(at, center, sq):
    i, j = sq[0] - center[0], center[1] - sq[1]
    x, y = art.iso(i, j)
    return at[0] + fw.CX + x, at[1] + fw.CY + y


def preview_all(idle, sprites, path, scale=2):
    """Все фигуры на клетке: верхний ряд — белые, нижний — чёрные."""
    crop = (70, 20, 230, 200)
    cw, ch = crop[2] - crop[0], crop[3] - crop[1]
    order = list(range(FRAMES)) + list(range(LOOP_START, FRAMES)) * 2
    scenes = {}
    for code in idle:
        tiles, shadows, _, at = fw.board_scene({}, (4, 3))
        c = tile_center(at, (4, 3), (4, 3))
        tiles.alpha_composite(sprites[code][1], (c[0] - art.AX, c[1] - art.AY))
        scenes[code] = (tiles, c)
    frames = []
    for f in order:
        im = Image.new("RGB", (6 * cw + 5 * 4, 2 * ch + 4), (27, 30, 39))
        for r, color in enumerate("wb"):
            for col, k in enumerate("PNBRQK"):
                tiles, c = scenes[color + k]
                sc = tiles.copy()
                sc.alpha_composite(Image.fromarray(idle[color + k][f], "RGBA"), (c[0] - FAX, c[1] - FAY))
                im.paste(sc.crop(crop).convert("RGB"), (col * (cw + 4), r * (ch + 4)))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    save_gif(frames, path)


def preview_step(idle, sprites, trails, path, scale=2):
    """Rogue-like: волна ядовитых чёрных шагает вперёд (вниз-влево) два раза."""
    center = (4, 4)
    walkers = {(2, 6): "bP", (4, 6): "bN", (6, 6): "bQ"}
    others = {(1, 1): "wP", (3, 1): "wK", (5, 1): "wP", (4, 0): "wR"}
    size = (420, 330)
    tiles, _, _, at = fw.board_scene({}, center, size=size)
    normal = fw.load_sprites()
    hold = 14
    timeline = []  # (шаг, кадр внутри шага); шаг = сколько раз уже шагнули
    for step in range(3):
        timeline += [(step, None, k) for k in range(hold)]
        if step < 2:
            timeline += [(step, k, None) for k in range(TRAIL_FRAMES)]
    frames = []
    trail_age = {}
    for n, (step, mv, _) in enumerate(timeline):
        im = tiles.copy()
        ground = Image.new("RGBA", size)
        layers = []
        for (f0, r0), code in walkers.items():
            r = r0 - step
            a = tile_center(at, center, (f0, r))
            if mv is not None:  # след и скольжение на клетку вперёд
                b = tile_center(at, center, (f0, r - 1))
                mid = ((a[0] + b[0]) // 2, (a[1] + b[1]) // 2)
                ground.alpha_composite(Image.fromarray(trails["sw"][mv], "RGBA"), (mid[0] - TRX, mid[1] - TRY))
                p = min(1.0, (mv + 1) / TRAIL_MOVE)
                p = p * p * (3 - 2 * p)
                pos = (round(a[0] + (b[0] - a[0]) * p), round(a[1] + (b[1] - a[1]) * p))
                depth = f0 + 7 - (r - p)
            else:
                pos, depth = a, f0 + 7 - r
            fi = LOOP_START + (n + f0 * 5) % LOOP
            layers.append((depth, pos, Image.fromarray(idle[code][fi], "RGBA"), sprites[code][1], True))
        for (f0, r0), code in others.items():
            pos = tile_center(at, center, (f0, r0))
            layers.append((f0 + 7 - r0, pos, normal[code][0], normal[code][1], False))
        im.alpha_composite(ground)
        for _, pos, _, shadow, _ in layers:
            im.alpha_composite(shadow, (pos[0] - art.AX, pos[1] - art.AY))
        for _, pos, body, _, anim in sorted(layers, key=lambda q: q[0]):
            if anim:
                im.alpha_composite(body, (pos[0] - FAX, pos[1] - FAY))
            else:
                im.alpha_composite(body, (pos[0] - art.AX, pos[1] - art.AY))
        frames.append(im.convert("RGB").resize((size[0] * scale, size[1] * scale), Image.NEAREST))
    save_gif(frames, path)


def preview_frames(idle, trails, path):
    cols = 10
    rows = []
    for code in ("wP", "bQ"):
        rows.append(idle[code])
    rows.append([np.pad(fr, ((20, 20), (12, 12), (0, 0))) for fr in trails["sw"]])
    sheet_rows = sum((len(r) + cols - 1) // cols for r in rows)
    sheet = Image.new("RGBA", (cols * (FW + 2), sheet_rows * (FH + 2)), (40, 44, 56, 255))
    y = 0
    for r in rows:
        for k, fr in enumerate(r):
            sheet.alpha_composite(Image.fromarray(fr, "RGBA"), ((k % cols) * (FW + 2), (y + k // cols) * (FH + 2)))
        y += (len(r) + cols - 1) // cols
    sheet.save(path)


def main():
    os.makedirs(os.path.join(PIECES_OUT, "poison"), exist_ok=True)
    os.makedirs(EFFECTS_OUT, exist_ok=True)
    sprites, idle = {}, {}
    for color in "wb":
        for kind in "PNBRQK":
            code = color + kind
            body, shadow = render_sprite(kind, color)
            body.save(os.path.join(PIECES_OUT, "poison", f"{code}.png"))
            shadow.save(os.path.join(PIECES_OUT, "poison", f"{code}_shadow.png"))
            sprites[code] = (body, shadow)
            idle[code] = render_idle(code, np.array(body))
            Image.fromarray(np.concatenate(idle[code], axis=1), "RGBA").save(
                os.path.join(EFFECTS_OUT, f"poison_{code}.png"))
            print(code, flush=True)
    trails = {d: render_trail(d) for d in DIRECTIONS}
    for d, frames in trails.items():
        Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(
            os.path.join(EFFECTS_OUT, f"poison_trail_{d}.png"))
    preview_sprites(sprites, os.path.join(PIECES_OUT, "preview_poison_pieces.png"))
    preview_all(idle, sprites, os.path.join(EFFECTS_OUT, "preview_poison.gif"))
    preview_step(idle, sprites, trails, os.path.join(EFFECTS_OUT, "preview_poison_step.gif"))
    preview_frames(idle, trails, os.path.join(EFFECTS_OUT, "preview_poison_frames.png"))
    sheet = Image.new("RGBA", (TRW * 5, TRH * 2), hexc("#3d4a3a") + (255,))
    for i, f in enumerate((0, 1, 3, 6, 9, 12, 15, 18, 22, 26)):
        sheet.alpha_composite(Image.fromarray(trails["sw"][f], "RGBA"), ((i % 5) * TRW, (i // 5) * TRH))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(
        os.path.join(EFFECTS_OUT, "preview_poison_trail.png"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
