"""Стрелка прицеливания картой (как в Slay the Spire) в стиле Into the Breach.

От взятой карты к цели тянется цепочка звеньев-шевронов по кривой Безье: у карты звенья
мелкие, к цели крупнее, на конце — наконечник. Звенья нарисованы остриём вправо со светлым
гребнем по оси (свет не зависит от поворота) и чёрным контуром; по касательной к кривой их
поворачивает RotSprite. Цвета: idle — белая сталь (цели нет), enemy — красный (чужая
фигура), ally — голубой (своя фигура). Цель отмечают четыре шеврона вокруг ромба клетки,
сходящиеся к ней (как метки ITB), — target_<состояние>.png.

`py tools/breach_arrow.py [папка]` — по умолчанию assets/breach/ui: arrow_seg_<s|m|l>_<состояние>,
arrow_head_<состояние>, target_<состояние> (1x, остриём вправо, без тени), превью и GIF.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_cards as bc  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "ui")

STATES = {
    # заливка, гребень, кромка, контур
    "idle": ("#cfd5df", "#f4f6fa", "#8d96a8", "#0d0f17"),
    "enemy": ("#e0452f", "#ff9a6b", "#a8231c", "#1a0505"),
    "ally": ("#2aa9d6", "#a6f2ff", "#16627f", "#06141c"),
}
# Шеврон остриём вправо: спинка, острие, спинка, выемка (в долях коробки w×h).
SIZES = {"s": (12, 10), "m": (15, 12), "l": (18, 14), "head": (28, 26)}
NOTCH = {"s": 0.32, "m": 0.34, "l": 0.36, "head": 0.3}
SPACING = {"s": 8.5, "m": 10.5, "l": 12.5}  # шаг звеньев вдоль кривой


def chevron(size, state):
    """Звено или наконечник остриём вправо, с контуром в пиксель."""
    w, h = SIZES[size]
    fill, ridge, edge_c, outline = STATES[state]
    yy, xx = np.mgrid[0 : h + 2, 0 : w + 2]
    px, py = xx + 0.5 - 1, yy + 0.5 - 1
    poly = [(0.0, 0.0), (w, h / 2), (0.0, h), (w * NOTCH[size], h / 2)]
    body = art.sd_polygon(px, py, poly) <= 0.15
    img = np.zeros((h + 2, w + 2, 4), np.uint8)
    img[body] = (*hexc(fill), 255)
    edge = body & ~(np.roll(body, 1, 0) & np.roll(body, -1, 0) & np.roll(body, 1, 1) & np.roll(body, -1, 1))
    img[edge] = (*hexc(edge_c), 255)
    # гребень по оси — светлый, не зависит от поворота
    axis = body & ~edge & (np.abs(py - h / 2) < (1.2 if size == "head" else 0.8))
    img[axis] = (*hexc(ridge), 255)
    img[bc.outline4(body)] = (*hexc(outline), 255)
    return img


def marker_pad(gap=5):
    """Поля холста метки вокруг клетки: шеврон целиком помещается снаружи ромба."""
    return gap + (max(SIZES["m"]) + 3) // 2 + 1


def target_marker(state, tw=art.TW, th=art.TH, gap=5):
    """Четыре шеврона вокруг ромба клетки, остриями к ней. Холст: клетка + поля marker_pad()."""
    pad = marker_pad(gap)
    w, h = tw + 2 * pad, th + 2 * pad
    img = np.zeros((h, w, 4), np.uint8)
    base = chevron("m", state)
    cx, cy = pad + tw / 2, pad + th / 2
    spots = [  # центр шеврона и куда он смотрит (градусы, 0 — вправо)
        ((cx, pad - gap), 90),  # сверху, остриём вниз
        ((pad + tw + gap, cy), 180),  # справа, остриём влево
        ((cx, pad + th + gap), 270),  # снизу, остриём вверх
        ((pad - gap, cy), 0),  # слева, остриём вправо
    ]
    for (x, y), deg in spots:
        spr = np.rot90(base, k={0: 0, 90: 3, 180: 2, 270: 1}[deg]) if deg % 90 == 0 else base
        sh, sw = spr.shape[:2]
        x0, y0 = int(round(x - sw / 2)), int(round(y - sh / 2))
        m = spr[..., 3] > 0
        img[y0 : y0 + sh, x0 : x0 + sw][m] = spr[m]
    return img


# ---------------------------------------------------------------- кривая
def bezier(start, end, lift=0.28):
    """Квадратичная кривая: выходит из карты вверх и подходит к цели сверху-сбоку."""
    (sx, sy), (ex, ey) = start, end
    dist = math.hypot(ex - sx, ey - sy)
    cx, cy = sx, min(sy, ey) - lift * dist
    t = np.linspace(0, 1, 600)
    x = (1 - t) ** 2 * sx + 2 * (1 - t) * t * cx + t**2 * ex
    y = (1 - t) ** 2 * sy + 2 * (1 - t) * t * cy + t**2 * ey
    seg = np.hypot(np.diff(x), np.diff(y))
    s = np.concatenate([[0], np.cumsum(seg)])
    return x, y, s


def point_at(x, y, s, d):
    """Точка и угол касательной на расстоянии d вдоль кривой."""
    i = int(np.clip(np.searchsorted(s, d), 1, len(s) - 1))
    ang = math.degrees(math.atan2(y[i] - y[i - 1], x[i] - x[i - 1]))
    return x[i], y[i], ang


ROT_CACHE = {}


def rotated(size, state, ang, step=5):
    """Повёрнутое звено; угол округляется до `step` градусов (так же стоит кэшировать в игре)."""
    a = int(round(ang / step) * step) % 360
    key = (size, state, a)
    if key not in ROT_CACHE:
        ROT_CACHE[key] = bc.rotate(chevron(size, state), a, STATES[state][3])
    return ROT_CACHE[key]


def draw_arrow(canvas, start, end, state):
    """Стрелка от start (верх карты) до end (острие наконечника) с жёсткой тенью."""
    x, y, s = bezier(start, end)
    total = s[-1]
    head_len = SIZES["head"][0] * 0.8
    sprites = []
    d = 6.0
    while d < total - head_len:
        frac = d / max(total, 1)
        size = "s" if frac < 0.3 else "m" if frac < 0.62 else "l"
        px, py, ang = point_at(x, y, s, d)
        sprites.append((px, py, rotated(size, state, ang)))
        d += SPACING[size]
    hx, hy, ang = point_at(x, y, s, total - head_len / 2)
    sprites.append((hx, hy, rotated("head", state, ang)))
    # сначала все тени, потом звенья от карты к наконечнику (ближние к цели — поверх)
    for px, py, spr in sprites:
        bc.put(canvas, bc.shadow_of(spr), px - spr.shape[1] / 2, py - spr.shape[0] / 2)
    for px, py, spr in sprites:
        bc.put(canvas, spr, px - spr.shape[1] / 2, py - spr.shape[0] / 2)


# ---------------------------------------------------------------- превью
SIDE, TOP = 100, 70  # как в breach_cards.table_preview
OX = SIDE + 4 * art.TW - art.TW // 2


def tile_xy(file, rank):
    i, j = file, 7 - rank
    return OX + (i - j) * (art.TW // 2), TOP + (i + j) * (art.TH // 2)


def held_card_top():
    """Верх приподнятой карты руки в превью стола (как считает breach_cards.hand)."""
    corner_x, corner_y = SIDE + 8 * art.TW, TOP + 4 * art.TH + art.SIDE
    hand_x, hand_y, radius, step, n, k = corner_x - 24, corner_y + 135, 220, 8.5, 6, 3
    a = math.radians((k - (n - 1) / 2) * step)
    cx = hand_x + math.sin(a) * radius
    cy = hand_y + radius - (math.cos(a) * radius + 18)
    return cx, cy - bc.CH / 2 + 1


def scene(base, end, state, target=None):
    img = base.copy()
    if target is not None:
        tx, ty = tile_xy(*target)
        bc.put(img, target_marker(state), tx - marker_pad(), ty - marker_pad())
    draw_arrow(img, held_card_top(), end, state)
    return img


def save_previews(out):
    base = bc.table_preview(scale=1)
    # на чёрную пешку d7 — красная стрелка и метки цели
    target = (3, 6)
    tx, ty = tile_xy(*target)
    enemy_end = (tx + art.TW // 2, ty + art.TH // 2 - 14)
    still = scene(base, enemy_end, "enemy", target=target)
    still.resize((still.width * 2, still.height * 2), Image.NEAREST).save(os.path.join(out, "preview_arrow.png"))
    # GIF: стрелка ведёт по пустым клеткам (белая), потом ловит пешку (красная)
    crop = (230, 40, 648, 497)
    path = [(330, 285), (324, 245), (332, 214), (352, 194), (370, 176), enemy_end]
    frames = []
    for k in range(28):
        u = min(k / 18, 1.0)
        pos = u * (len(path) - 1)
        i = min(int(pos), len(path) - 2)
        f = pos - i
        end = (path[i][0] * (1 - f) + path[i + 1][0] * f, path[i][1] * (1 - f) + path[i + 1][1] * f)
        locked = math.hypot(end[0] - enemy_end[0], end[1] - enemy_end[1]) < 12
        im = scene(base, enemy_end if locked else end, "enemy" if locked else "idle",
                   target=target if locked else None)
        im = im.crop(crop)
        frames.append(im.convert("RGB").resize((im.width * 2, im.height * 2), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * len(frames)))
    for k, p in enumerate(frames):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    durations = [60] * len(gif)
    durations[-1] = 900
    gif[0].save(os.path.join(out, "preview_arrow.gif"), save_all=True, append_images=gif[1:],
                duration=durations, loop=0, disposal=1)
    # детали: звенья, наконечники и метки во всех состояниях
    gap = 8
    row_h = max(SIZES["head"][1] + 2, art.TH + 2 * marker_pad()) + gap
    sheet = Image.new("RGBA", (gap + 4 * 32 + art.TW + 2 * marker_pad() + gap, gap + len(STATES) * row_h), hexc("#1b1e27") + (255,))
    for r, state in enumerate(STATES):
        x, y = gap, gap + r * row_h
        for size in ("s", "m", "l", "head"):
            spr = chevron(size, state)
            bc.put(sheet, spr, x, y + (SIZES["head"][1] - SIZES[size][1]) // 2)
            x += 32
        bc.put(sheet, target_marker(state), x, y)
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(out, "preview_arrow_parts.png"))


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for state in STATES:
        for size in ("s", "m", "l"):
            Image.fromarray(chevron(size, state), "RGBA").save(os.path.join(OUT, f"arrow_seg_{size}_{state}.png"))
        Image.fromarray(chevron("head", state), "RGBA").save(os.path.join(OUT, f"arrow_head_{state}.png"))
        Image.fromarray(target_marker(state), "RGBA").save(os.path.join(OUT, f"target_{state}.png"))
    save_previews(OUT)
    print("done", OUT)
