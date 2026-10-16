"""Огненная волна в стиле Into the Breach и арт карты, которая её вызывает.

Волна расходится от клетки фигуры на одну клетку во все стороны (3×3 клетки): по
верхним граням бежит ромбовидная кромка огня — бело-жёлтая спереди, за ней оранжевая
и тёмно-красная; над кромкой — языки пламени с чёрным контуром, позади остаются
тлеющие подпалины, в конце — искры и клубы дыма. Кадры делятся на два слоя: `under`
(земля и огонь на трёх клетках позади фигуры — рисовать до фигур) и `over` (остальное
пламя, искры, дым — после фигур).

Кадр FW×FH, центр верхней грани клетки фигуры — (CX, CY). Арт карты — картинка окна
лица карты (ART_AT, ART_SIZE).

`py tools/breach_firewave.py [папка]` — по умолчанию assets/breach/effects:
fire_wave_under.png, fire_wave_over.png (кадры в ряд), preview_fire_wave.gif,
preview_fire_wave_frames.png; арт — assets/breach/cards/art/fire_wave.png и
preview_fire_wave_card.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_cards as bc  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")
ART_DIR = os.path.join(art.ROOT, "assets", "breach", "cards", "art")

FIRE, OUTLINE, SMOKE = burn.FIRE, burn.OUTLINE, burn.SMOKE
TW, TH = art.TW, art.TH
FW, FH = 184, 168
CX, CY = 92, 98  # центр верхней грани клетки фигуры
FRAMES = 20
FRAME_MS = 60
WAVE_START, WAVE_END = 2, 10  # кадры, за которые кромка доходит до края соседних клеток
ART_AT, ART_SIZE = (6, 17), (38, 25)  # окно под картинку на лице карты


def grid_coords():
    """Для каждого пикселя кадра — координаты в клетках относительно клетки фигуры."""
    yy, xx = np.mgrid[0:FH, 0:FW]
    dx, dy = xx + 0.5 - CX, yy + 0.5 - CY
    u = (dx / (TW / 2) + dy / (TH / 2)) / 2
    v = (dy / (TH / 2) - dx / (TW / 2)) / 2
    return u, v


U, V = grid_coords()
CHEB = np.maximum(np.abs(U), np.abs(V))  # 0.5 — край своей клетки, 1.5 — край соседних
AREA = CHEB <= 1.5
BACK = (np.round(U) + np.round(V)) < 0  # три клетки позади фигуры


def front_radius(f):
    """Радиус кромки в клетках: быстро вначале, медленнее к краю."""
    t = np.clip((f - WAVE_START) / (WAVE_END - WAVE_START), 0, 1)
    r = 0.35 + (1.62 - 0.35) * (1 - (1 - t) ** 2)
    # дойдя до края, кромка уходит за него и гаснет за три кадра
    return r + 0.13 * max(0, f - WAVE_END)


def paint_flames(layer_under, layer_over, fl_back, fl_front):
    """Раскраска пламени как в анимации сжигания: светлое нутро, оранжевая и тёмная
    кайма, тёмные кончики, чёрный контур; задние клетки — в слой под фигурами."""
    for fl, layer in ((fl_back, layer_under), (fl_front, layer_over)):
        if not fl.any():
            continue
        edge = burn.boundary(fl)
        inner_edge = burn.boundary(fl & ~edge)
        c = np.zeros((FH, FW, 4), np.uint8)
        c[fl] = (*FIRE[1], 255)
        c[inner_edge] = (*FIRE[3], 255)
        c[edge] = (*FIRE[5], 255)
        for x in range(FW):
            col = np.nonzero(fl[:, x])[0]
            if len(col) > 4:
                c[col[0], x, :3] = FIRE[7]
                c[col[1], x, :3] = FIRE[6]
        ring = burn.neighbours_any(fl) & ~fl
        layer[ring] = (*OUTLINE, 255)
        layer[fl] = c[fl]


def flame_wall(lead, rng, base_h=3.5, tongue=(8.0, 14.0), spacing=4):
    """Огненная стена над кромкой: по столбцам — низкое основание и зубчатые языки.
    Задняя половина кольца (выше центра) и передняя считаются отдельно."""
    fl_back = np.zeros((FH, FW), bool)
    fl_front = np.zeros((FH, FW), bool)
    for upper in (True, False):
        base = np.full(FW, -1)
        for x in range(FW):
            ys = np.nonzero(lead[:, x] & ((np.arange(FH) < CY) == upper))[0]
            if len(ys):
                base[x] = ys.max() + 1  # пламя растёт от нижнего края полосы
        cols = np.nonzero(base >= 0)[0]
        if not len(cols):
            continue
        height = np.where(base >= 0, base_h, 0.0)
        taken = []
        for cx in rng.permutation(cols):
            if any(abs(cx - t) < spacing for t in taken):
                continue
            taken.append(cx)
            ht, wt = rng.uniform(*tongue), rng.uniform(2.5, 4.5)
            for x in range(int(cx - wt), int(cx + wt) + 1):
                if 0 <= x < FW and base[x] >= 0:
                    height[x] = max(height[x], ht * (1 - abs(x - cx) / (wt + 0.5)))
        for x in cols:
            top = max(0, int(round(base[x] - height[x])))
            target = fl_back if BACK[min(base[x], FH - 1), x] else fl_front
            target[top : min(FH, base[x] + 1), x] = True
    return fl_back, fl_front


def small_fires(points, rng):
    """Догорающие огоньки на соседних клетках: короткие широкие язычки."""
    fl_back = np.zeros((FH, FW), bool)
    fl_front = np.zeros((FH, FW), bool)
    for bx, by, scale in points:
        ht, wt = rng.uniform(4, 8) * scale, rng.uniform(2.0, 3.2)
        target = fl_back if BACK[by, bx] else fl_front
        for x in range(int(bx - wt), int(bx + wt) + 1):
            if 0 <= x < FW:
                h = ht * (1 - abs(x - bx) / (wt + 0.5))
                if h > 0.8:
                    target[max(0, int(round(by - h))) : min(FH, by + 1), x] = True
    return fl_back, fl_front


def puff(layer, p):
    """Клуб дыма ITB: комок из трёх кругов, светлый верх, контур, 70% непрозрачности."""
    r = p["r"]
    if r < 0.8:
        return
    yy, xx = np.mgrid[0:FH, 0:FW]
    px, py = xx + 0.5, yy + 0.5
    body = np.zeros((FH, FW), bool)
    for ox, oy, k in ((-0.45, 0.1, 0.72), (0.45, -0.05, 0.68), (0.0, -0.45, 0.62)):
        body |= np.hypot(px - (p["x"] + ox * r), (py - (p["y"] + oy * r)) * 1.15) <= r * k
    lit = body & (np.hypot(px - (p["x"] - r * 0.35), (py - (p["y"] - r * 0.45)) * 1.15) <= r * 0.5)
    shade = body & ~np.roll(np.roll(body, -2, 0), -1, 1)
    a = burn.SMOKE_ALPHA
    paint = np.zeros((FH, FW, 4), np.uint8)
    paint[body] = (*SMOKE[1], a)
    paint[shade] = (*SMOKE[2], a)
    paint[lit & ~shade] = (*SMOKE[0], a)
    ring = burn.neighbours_any(body) & ~body
    paint[ring] = (*OUTLINE, a)
    m = paint[..., 3] > 0
    layer[m] = paint[m]


def render(seed=11):
    rng = np.random.default_rng(seed)
    noise = burn.value_noise(FH, FW, 5, rng)
    residue = burn.value_noise(FH, FW, 3, rng)
    jag = CHEB + 0.11 * (noise - 0.5)  # неровная кромка
    embers, smoke = burn.Particles(), burn.Particles()
    unders, overs = [], []
    for f in range(FRAMES):
        under = np.zeros((FH, FW, 4), np.uint8)
        over = np.zeros((FH, FW, 4), np.uint8)
        R = front_radius(f)
        dc = R - jag
        ring_zone = AREA & (CHEB > 0.42)

        # вспышка по краю своей клетки перед волной
        if f < WAVE_START + 1:
            flash = (CHEB > 0.40) & (CHEB <= 0.47 + 0.03 * f)
            under[burn.neighbours_any(flash) & ~flash & AREA] = (*OUTLINE, 255)
            under[flash] = (*FIRE[0 if f == 1 else 2], 255)

        # подпалины позади кромки: тёмные, с тлеющими точками, к концу гаснут
        fade = np.clip((FRAMES - 1 - f) / 7, 0, 1)
        scorch = ring_zone & (dc >= 0.36)
        under[scorch] = (0, 0, 0, int(75 * fade))
        glow = scorch & (residue > 0.66 + 0.3 * (1 - fade)) & (rng.random((FH, FW)) < 0.3)
        under[glow] = (*FIRE[7], 255)
        under[glow & (residue > 0.8)] = (*FIRE[5], 255)

        # кромка огня по земле
        band = ring_zone & (dc >= 0) & (dc < 0.36)
        for lo, hi, col in ((0.0, 0.07, FIRE[1]), (0.07, 0.15, FIRE[3]), (0.15, 0.25, FIRE[5]), (0.25, 0.36, FIRE[7])):
            under[band & (dc >= lo) & (dc < hi)] = (*col, 255)
        ahead = burn.neighbours_any(band) & ~band & ring_zone & (dc < 0)
        under[ahead] = (*OUTLINE, 255)

        # огненная стена на кромке, а когда кромка прошла — огоньки по соседним клеткам
        lead = band & (dc < 0.15)
        ys, xs = np.nonzero(lead & (dc < 0.07))
        fb, ff = flame_wall(lead, rng) if lead.any() else (np.zeros((FH, FW), bool),) * 2
        if f >= WAVE_END - 1:
            left = max(0, 10 - 2 * (f - WAVE_END + 1))
            zone = np.nonzero(AREA & (CHEB > 0.62) & (CHEB < 1.38))
            pts = [(zone[1][k], zone[0][k], 1.0) for k in rng.choice(len(zone[0]), size=left, replace=False)]
            sb, sf = small_fires(pts, rng)
            fb, ff = fb | sb, ff | sf
        paint_flames(under, over, fb, ff)

        # искры летят наружу и вверх, дым поднимается от догорающего огня
        if len(ys):
            for k in rng.choice(len(ys), size=min(8, len(ys)), replace=False):
                dx, dy = xs[k] - CX, ys[k] - CY
                dl = math.hypot(dx, dy) + 1e-6
                embers.items.append(dict(x=xs[k] + 0.5, y=ys[k] - 2.0, vx=1.4 * dx / dl + rng.uniform(-0.4, 0.4),
                                         vy=rng.uniform(-3.4, -1.6), drag=0.88, life=int(rng.integers(5, 9)),
                                         age=0, wob=0.3, ph=rng.uniform(0, 6)))
        if WAVE_END - 2 <= f <= FRAMES - 6 and f % 2 == 0:
            zone = np.nonzero(AREA & (CHEB > 0.7) & (CHEB < 1.4))
            k = rng.integers(len(zone[0]))
            smoke.items.append(dict(x=zone[1][k] + 0.5, y=zone[0][k] - 8.0, vx=rng.uniform(-0.3, 0.3),
                                    vy=rng.uniform(-1.8, -1.2), drag=0.94, life=int(rng.integers(7, 11)), age=0,
                                    r0=rng.uniform(2.0, 2.8), wob=0.3, ph=rng.uniform(0, 6)))
        for p in smoke.items:
            life = p["age"] / p["life"]
            grow = p["r0"] + 3.0 * min(life / 0.6, 1.0)
            p["r"] = grow * (1.0 if life < 0.65 else max(0.0, 1 - (life - 0.65) / 0.35))
            puff(over, p)
        for p in embers.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                life = p["age"] / p["life"]
                over[y, x] = (*FIRE[[1, 3, 5, 7][min(int(life * 4), 3)]], 255)
                if p["age"] < 2 and y + 1 < FH:
                    over[y + 1, x] = (*FIRE[5], 255)
        embers.step()
        smoke.step()
        unders.append(under)
        overs.append(over)
    return unders, overs


# ---------------------------------------------------------------- превью на доске
def load_sprites():
    pieces = os.path.join(art.ROOT, "assets", "breach", "pieces")
    sprites = {}
    for color in "wb":
        for k in "PNBRQK":
            body = Image.open(os.path.join(pieces, f"{color}{k}.png")).convert("RGBA")
            shadow = Image.open(os.path.join(pieces, f"{color}{k}_shadow.png")).convert("RGBA")
            sprites[color + k] = (body, shadow)
    return sprites


def board_scene(position, center, size=(300, 236)):
    """Кусок доски с фигурами вокруг клетки `center`; слои: клетки, фигуры по рядам."""
    ox = size[0] // 2 - TW // 2
    oy = size[1] // 2 - TH // 2 + 10
    cf, cr = center

    def tile_xy(f, r):
        i, j = f - cf, (7 - r) - (7 - cr)
        x, y = art.iso(i, j)
        return ox + x, oy + y

    tiles = Image.new("RGBA", size, hexc("#151821") + (255,))
    cells = sorted(((f, r) for f in range(8) for r in range(8)), key=lambda c: c[0] + 7 - c[1])
    for f, r in cells:
        kind = "dark" if (f + r) % 2 == 0 else "light"
        x, y = tile_xy(f, r)
        if -TW < x < size[0] and -80 < y < size[1]:
            tiles.alpha_composite(art.block(kind, 11 * ((f * 7 + r * 3) % 4) + (0 if kind == "dark" else 5)), (x, y))
    sprites = load_sprites()
    pieces = Image.new("RGBA", size)
    shadows = Image.new("RGBA", size)
    for (f, r), code in sorted(position.items(), key=lambda kv: kv[0][0] + 7 - kv[0][1]):
        x, y = tile_xy(f, r)
        body, shadow = sprites[code]
        at = (x + TW // 2 - art.AX, y + TH // 2 - art.AY)
        shadows.alpha_composite(shadow, at)
        pieces.alpha_composite(body, at)
    cx, cy = tile_xy(cf, cr)
    return tiles, shadows, pieces, (cx + TW // 2 - CX, cy + TH // 2 - CY)


def save_effect(unders, overs, out, scale=2):
    os.makedirs(out, exist_ok=True)
    Image.fromarray(np.concatenate(unders, axis=1), "RGBA").save(os.path.join(out, "fire_wave_under.png"))
    Image.fromarray(np.concatenate(overs, axis=1), "RGBA").save(os.path.join(out, "fire_wave_over.png"))
    position = {(4, 3): "wQ", (3, 4): "bP", (4, 4): "bP", (5, 4): "bN", (5, 3): "bP", (3, 2): "bB",
                (6, 5): "bR", (2, 3): "wP", (4, 1): "wK"}
    tiles, shadows, pieces, at = board_scene(position, (4, 3))
    frames = []
    for k in range(-4, FRAMES + 6):  # пауза до и после
        f = min(max(k, 0), FRAMES - 1)
        im = tiles.copy()
        im.alpha_composite(shadows)
        if 0 <= k < FRAMES:
            im.alpha_composite(Image.fromarray(unders[f], "RGBA"), at)
        im.alpha_composite(pieces)
        if 0 <= k < FRAMES:
            im.alpha_composite(Image.fromarray(overs[f], "RGBA"), at)
        frames.append(im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * len(frames)))
    for k, p in enumerate(frames):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_fire_wave.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    # лист кадров (каждый второй кадр)
    picks = list(range(0, FRAMES, 2))
    w, h = frames[0].width // scale, frames[0].height // scale
    sheet = Image.new("RGB", (5 * (w + 4) + 4, 2 * (h + 4) + 4), (27, 30, 39))
    for n, f in enumerate(picks):
        sheet.paste(frames[f + 4].resize((w, h), Image.NEAREST), (4 + (n % 5) * (w + 4), 4 + (n // 5) * (h + 4)))
    sheet.save(os.path.join(out, "preview_fire_wave_frames.png"))


# ---------------------------------------------------------------- арт карты
# Белая пешка для арта, нарисованная по пикселям в палитре белых фигур: голова с голубым
# визором, воротничок и основание цвета вороненой стали, свет слева.
PAWN_ICON = [
    "...hll...",
    "..hlllm..",
    "..Ccccc..",
    "..llmmd..",
    "...lmd...",
    "..tttTT..",
    "...lmd...",
    "...lmd...",
    "..llmmd..",
    ".ttttTTT.",
    "hllllmmmd",
    "tttttTTTT",
]
PAWN_COLORS = {
    "h": "#f4f1df", "l": "#d3cdb1", "m": "#a49e80", "d": "#6e6a55",
    "t": "#7d8881", "T": "#535d57", "c": "#57d9ff", "C": "#a6f2ff",
}


def pawn_icon(colors=PAWN_COLORS):
    """RGBA пешки с контуром в пиксель."""
    grid = np.array([list(r) for r in PAWN_ICON])
    h, w = grid.shape
    img = np.zeros((h + 2, w + 2, 4), np.uint8)
    body = np.pad(grid != ".", 1)
    img[burn.neighbours_any(body) & ~body] = (*OUTLINE, 255)
    for ch, col in colors.items():
        img[1:-1, 1:-1][grid == ch] = (*hexc(col), 255)
    return img


def card_art(seed=5):
    """Картинка окна карты 38×25: белая пешка в кольце огня на тёмной земле."""
    w, h = ART_SIZE
    rng = np.random.default_rng(seed)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    px, py = xx + 0.5, yy + 0.5
    horizon = 9
    # небо: ночная синева полосами, у горизонта — отсвет огня с дизерингом
    for y0, y1, col in ((0, 3, "#11141f"), (3, 6, "#161a27"), (6, horizon, "#2a1a1e")):
        img[(yy >= y0) & (yy < y1)] = (*hexc(col), 255)
    img[(yy == 5) & ((xx + yy) % 2 == 0)] = (*hexc("#2a1a1e"), 255)
    # земля-доска: тёмные клетки ромбами
    ground = yy >= horizon
    gu = (px - w / 2) / 8 + (py - 17) / 4
    gv = (py - 17) / 4 - (px - w / 2) / 8
    checker = (np.floor(gu).astype(int) + np.floor(gv).astype(int)) % 2 == 0
    img[ground & checker] = (*hexc("#2b2320"), 255)
    img[ground & ~checker] = (*hexc("#221c1b"), 255)
    img[(yy == horizon)] = (*hexc("#3d2420"), 255)
    # кольцо огня: ромб 2:1 вокруг фигуры, внутри — подпалина
    cx, cy = w / 2, 17.0
    d = np.abs(px - cx) / 15.5 + np.abs(py - cy) / 7.0
    img[(d < 0.78) & ground] = (*hexc("#1a1312"), 255)
    band = (d >= 0.78) & (d <= 1.0)
    img[band & (d > 0.92)] = (*FIRE[1], 255)
    img[band & (d <= 0.92) & (d > 0.85)] = (*FIRE[3], 255)
    img[band & (d <= 0.85)] = (*FIRE[5], 255)
    # фигура в центре: белая пешка по пикселям — приземистая, с широким основанием
    piece = pawn_icon()
    ph, pw = piece.shape[:2]
    # языки пламени на кольце: сзади невысокие, спереди выше, но перед пешкой низкие —
    # основание фигуры должно быть видно целиком
    flames = np.zeros((h, w), bool)
    fcol = np.zeros((h, w, 4), np.uint8)
    for x in range(w):
        col = np.nonzero(band[:, x])[0]
        if not len(col):
            continue
        for base, k in ((col.min(), 0.7), (col.max(), 0.8)):
            ht = (2.5 + 3.5 * abs(math.sin(x * 0.9 + rng.uniform(0, 0.6)))) * k
            if x % 3 == 0:
                ht += 2
            if base == col.max() and abs(x + 0.5 - cx) < pw / 2 + 2:
                ht = min(ht, 2.0)
            top = max(0, int(round(base - ht)))
            flames[top : base + 1, x] = True
    edge = burn.boundary(flames)
    fcol[flames] = (*FIRE[2], 255)
    fcol[edge] = (*FIRE[5], 255)
    for x in range(w):
        col = np.nonzero(flames[:, x])[0]
        if len(col) > 3:
            fcol[col[0], x, :3] = FIRE[7]
    # сначала задняя половина кольца с пламенем, потом пешка, потом передняя половина
    behind = py < cy
    ring_out = burn.neighbours_any(flames) & ~flames
    for part in (behind, ~behind):
        m = flames & part
        img[ring_out & part] = (*OUTLINE, 255)
        img[m] = fcol[m]
        if part is behind:
            x0, y0 = int(cx - pw / 2), int(cy + 2 - ph)  # основание — в середине кольца
            m2 = piece[..., 3] > 0
            img[y0 : y0 + ph, x0 : x0 + pw][m2] = piece[m2]
    # искры
    for _ in range(9):
        x, y = rng.integers(2, w - 2), rng.integers(1, horizon + 3)
        img[y, x] = (*FIRE[rng.choice([1, 3, 5])], 255)
    return img


def save_card_art(art_img):
    os.makedirs(ART_DIR, exist_ok=True)
    Image.fromarray(art_img, "RGBA").save(os.path.join(ART_DIR, "fire_wave.png"))
    faces = []
    for kind in ("w", "b", "victory", "sacrifice"):
        for sel in (False, True):
            face = bc.card_front(kind, selected=sel).copy()
            x0, y0 = ART_AT
            face[y0 : y0 + ART_SIZE[1], x0 : x0 + ART_SIZE[0]] = art_img
            faces.append(face)
    gap = 10
    sheet = Image.new("RGBA", (4 * (bc.CW + gap) + gap, 2 * (bc.CH + gap) + gap), hexc("#1b1e27") + (255,))
    for n, face in enumerate(faces):
        col, row = n // 2, n % 2
        x, y = gap + col * (bc.CW + gap), gap + row * (bc.CH + gap)
        sheet.alpha_composite(Image.fromarray(bc.shadow_of(face), "RGBA"), (x, y))
        sheet.alpha_composite(Image.fromarray(face, "RGBA"), (x, y))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(
        os.path.join(ART_DIR, "preview_fire_wave_card.png")
    )


if __name__ == "__main__":
    unders, overs = render()
    save_effect(unders, overs, OUT)
    save_card_art(card_art())
    print("done", OUT, ART_DIR)
