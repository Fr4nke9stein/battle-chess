"""Благословение слона в стиле Into the Breach: слон светится, раскрывает белые ангельские
крылья, вокруг на клетку во все стороны (3×3) расходится волна света.

Слон разгорается золотисто-белым, вокруг силуэта — ореол; за спиной раскрываются крылья
(кроющие перья по верхней кромке, веер маховых перьев вниз-наружу, контур, правое
крыло в тени). На пике — вспышка, по соседним клеткам бежит кольцо света, над каждой
клеткой вспыхивает столб, взлетают искры-звёздочки. В конце крылья рассыпаются искрами,
свет гаснет.

Слои (24 кадра, 60 мс):
- `bless_under` — земля (свет под слоном, кольцо волны) — до фигур; кадр 184×168,
  центр верхней грани клетки слона — (CX, CY) = (92, 98), как у огненной волны;
- `bless_figure_<w|b>` — слон с крыльями и ореолом вместо обычного спрайта слона; кадр
  FIG_W×FIG_H, точка опоры (центр клетки) — (FAX, FAY); тень — обычная тень слона;
- `bless_over` — искры, столбы света — после фигур; кадр как у `under`.

`py tools/breach_bless.py [папка]` — по умолчанию assets/breach/effects: полосы кадров,
preview_bless.gif, preview_bless_frames.png.
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
import breach_firewave as fw  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

FW, FH, CX, CY = fw.FW, fw.FH, fw.CX, fw.CY
FIG_W, FIG_H = 120, 112
FAX, FAY = 60, 92  # центр клетки в кадре фигуры
FRAMES = 24
FRAME_MS = 60
SHOULDER = (9, 40)  # плечи в координатах спрайта слона: ±x от оси, y
GOLD = np.array(hexc("#fff6d2"), float)  # цвет свечения
AURA = ["#fff1b0", "#ffd966"]
OUTLINE = "#020201"
WING = {  # перо: свет, основа, тень, шов; правое крыло темнее
    False: ["#ffffff", "#f4f1df", "#d3cdb1", "#9e967f"],
    True: ["#f4f1df", "#e2ddc8", "#c2bba2", "#857d68"],
}
LIGHT = ["#ffffff", "#fff3c4", "#ffe27a", "#e0b44c"]  # волна: кромка → хвост
SPARK = ["#ffffff", "#fff3c4", "#ffe27a", "#e0b44c"]


def ease_out(u):
    return 1 - (1 - u) ** 2


def glow_level(f):
    """Сила свечения слона по кадрам: разгорается, вспышка, держится, гаснет."""
    if f <= 5:
        return 0.45 * f / 5
    if f <= 9:
        return 0.45
    if f == 10:
        return 0.95
    if f <= 15:
        return 0.6 - 0.2 * (f - 11) / 4
    return 0.4 * max(0.0, 1 - (f - 15) / 8)


def wing_unfold(f):
    """Раскрытие крыльев 0…1: раскрываются в кадрах 3–10, держатся, 18+ — рассыпаются."""
    if f < 3:
        return 0.0
    if f <= 10:
        return ease_out((f - 3) / 7)
    return 1.0


def wing_fade(f):
    """Доля рассыпавшихся пикселей крыла."""
    return 0.0 if f < 18 else min(1.0, (f - 17) / 6)


# ---------------------------------------------------------------- крылья
# Крыло по образцу (классические раскрытые крылья): от плеча вверх-наружу идёт гладкая
# кромка, под ней ряды округлых перьев-лепестков — кроющие, средние, длинные маховые; их
# концы дают фестончатый нижний край, к кончику крыло заостряется. Строится прямо в
# пикселях в любом масштабе и наклоне (`spread_wing`), поэтому каждый кадр чёткий.
# Свои координаты левого крыла: плечо — (0, 0), наружу — −x, вверх — −y.
WING_ANIM = {
    "edge": [(0, 0), (-5, -14), (-16, -30), (-30, -38)],
    # ряды от заднего к переднему: число, t начала/конца по кромке, длина у плеча/у
    # кончика, полуширина, отступ от кромки, доворот к кончику (градусы)
    "rows": [(8, 0.0, 1.0, 15, 24, 3.6, 2.0, 20),
             (8, 0.04, 0.96, 10, 15, 3.3, 2.0, 15),
             (9, 0.06, 0.96, 6, 8, 2.8, 1.2, 10)],
    "band": (3.0, 1.6),  # толщина кромки у плеча и у кончика
}
WING_OPEN_SCALE = 0.62  # раскрытое крыло в кадре фигуры
WING_CARD = {  # для окна карты: меньше лепестков, они крупнее — мелко не слипаются
    "edge": [(0, 0), (-2, -5), (-6, -9), (-11, -11)],
    "rows": [(4, 0.1, 1.0, 4, 6, 1.7, 0.6, 20),
             (3, 0.2, 0.85, 2.5, 3, 1.4, 0.4, 10)],
    "band": (1.3, 0.9),
}


def bezier3(p, t):
    u = 1 - t
    return (u**3 * p[0][0] + 3 * u * u * t * p[1][0] + 3 * u * t * t * p[2][0] + t**3 * p[3][0],
            u**3 * p[0][1] + 3 * u * u * t * p[1][1] + 3 * u * t * t * p[2][1] + t**3 * p[3][1])


def spread_wing(right, geom, scale=1.0, rot=0.0):
    """RGBA крыла и точка плеча в нём. rot > 0 — крыло наклонено к оси и вверх (сложено),
    правое крыло — зеркало левого."""
    c = WING[right]
    reach = max(math.hypot(*p) for p in geom["edge"]) + 34
    r_px = int(math.ceil(reach * scale)) + 3
    size = 2 * r_px + 1
    yy, xx = np.mgrid[0:size, 0:size]
    dx, dy = xx + 0.5 - r_px, yy + 0.5 - r_px
    if right:
        dx = -dx
    a = math.radians(rot)
    ox = (dx * math.cos(a) + dy * math.sin(a)) / scale
    oy = (-dx * math.sin(a) + dy * math.cos(a)) / scale
    edge = geom["edge"]
    ids = np.full((size, size), -1)
    n_id = 0
    for count, t0, t1, l0, l1, half, off, tilt in geom["rows"]:
        for t in np.linspace(t0, t1, count):
            bx, by = bezier3(edge, t)
            ax_, ay_ = bezier3(edge, min(1.0, t + 0.02))
            bx0, by0 = bezier3(edge, max(0.0, t - 0.02))
            tx, ty = ax_ - bx0, ay_ - by0
            tl = math.hypot(tx, ty)
            tx, ty = tx / tl, ty / tl  # вдоль кромки, к кончику
            nx, ny = ty, -tx  # вниз от кромки
            g = math.radians(tilt * t)
            fx, fy = nx * math.cos(g) + tx * math.sin(g), ny * math.cos(g) + ty * math.sin(g)
            fl = math.hypot(fx, fy)
            fx, fy = fx / fl, fy / fl
            ln = l0 + (l1 - l0) * t
            cx, cy = bx + nx * off + fx * ln / 2, by + ny * off + fy * ln / 2
            u = (ox - cx) * fx + (oy - cy) * fy
            v = -(ox - cx) * fy + (oy - cy) * fx
            lobe = (u / (ln / 2)) ** 2 + (v / half) ** 2 <= 1.0
            ids[lobe] = n_id
            n_id += 1
    # кромка — гладкая полоса поверх перьев
    ts = np.linspace(0, 1, 60)
    ex, ey = bezier3(edge, ts)
    d = np.hypot(ox[..., None] - ex, oy[..., None] - ey)
    thick = geom["band"][0] + (geom["band"][1] - geom["band"][0]) * ts
    band = (d <= thick).any(axis=-1)
    ids[band] = n_id
    body = ids >= 0
    grid = np.where(body, "w", ".").astype("<U1")
    # шов — по нижнему краю пера, лежащего поверх соседнего: у каждого ряда фестончатый низ
    p = np.pad(ids, 1, constant_values=-1)
    below = p[2:, 1:-1]
    seam = body & (below >= 0) & (below < ids)
    lit, shaded = bc.shade_mask(body)
    grid[shaded] = "g"
    grid[band & ~lit] = "h"  # кромка светлее перьев
    grid[seam] = "s"
    grid[lit] = "h"
    img = np.zeros((size, size, 4), np.uint8)
    for ch, col in (("w", c[1]), ("g", c[2]), ("s", c[3]), ("h", c[0])):
        img[grid == ch] = (*hexc(col), 255)
    img[bc.outline4(body)] = (*hexc(OUTLINE), 255)
    return img, (r_px, r_px)


def place_wing(img, right, s, shoulder, fade, noise):
    """Крыло в кадре фигуры: раскрытие s, рассыпание fade (по карте шума, кромка — золото)."""
    # сложено — поднято и прижато к оси, раскрыто — разведено вверх-наружу
    out, (ax, ay) = spread_wing(right, WING_ANIM, scale=(0.45 + 0.55 * s) * WING_OPEN_SCALE, rot=40 * (1 - s))
    x0, y0 = int(round(shoulder[0] - ax)), int(round(shoulder[1] - ay))
    h, w = out.shape[:2]
    ys, xs = slice(max(0, y0), min(FIG_H, y0 + h)), slice(max(0, x0), min(FIG_W, x0 + w))
    part = out[ys.start - y0 : ys.stop - y0, xs.start - x0 : xs.stop - x0]
    m = part[..., 3] > 0
    if fade > 0:
        nz = noise[ys, xs]
        edge = m & (nz >= fade) & (nz < fade + 0.12)
        m &= nz >= fade
        part = part.copy()
        part[edge] = (*hexc(SPARK[2]), 255)
    region = img[ys, xs]
    region[m] = part[m]


def figure_frames(color, seed=5):
    """Слон с крыльями и ореолом: кадры FIG_W×FIG_H."""
    rng = np.random.default_rng(seed)
    noise = burn.value_noise(FIG_H, FIG_W, 3, rng)  # карта выгорания крыльев
    body = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{color}B.png")).convert("RGBA"))
    ox, oy = FAX - art.AX, FAY - art.AY
    frames = []
    for f in range(FRAMES):
        img = np.zeros((FIG_H, FIG_W, 4), np.uint8)
        s = wing_unfold(f)
        fade = wing_fade(f)
        if s > 0 and fade < 1:
            for right in (False, True):
                sx = FAX + (SHOULDER[0] if right else -SHOULDER[0])
                bob = 1 if f > 10 and (f // 3) % 2 else 0  # лёгкий взмах — сдвиг на пиксель
                place_wing(img, right, s, (sx, oy + SHOULDER[1] - bob), fade, noise)
        # слон, подкрашенный светом
        k = glow_level(f)
        piece = body.astype(float)
        solid = body[..., 3] > 0
        dark = body[..., :3].sum(axis=2) < 60  # контур — светлеет слабее, форма читается
        mix = np.where(dark, k * 0.45, k)[..., None]
        piece[..., :3] = piece[..., :3] * (1 - mix) + GOLD * mix
        region = img[oy : oy + body.shape[0], ox : ox + body.shape[1]]
        region[solid] = piece.astype(np.uint8)[solid]
        # ореол вокруг силуэта слона
        if k > 0.15:
            sil = np.zeros((FIG_H, FIG_W), bool)
            sil[oy : oy + body.shape[0], ox : ox + body.shape[1]] = solid
            ring1 = bc.outline4(sil)
            a1 = int(255 * min(1.0, k * 1.6))
            img[ring1 & (img[..., 3] == 0)] = (*hexc(AURA[0]), a1)
            if k > 0.35:
                ring2 = bc.outline4(sil | ring1)
                yy, xx = np.mgrid[0:FIG_H, 0:FIG_W]
                dith = ring2 & ((xx + yy + f) % 2 == 0) & (img[..., 3] == 0)
                img[dith] = (*hexc(AURA[1]), 150)
        frames.append(img)
    return frames


# ---------------------------------------------------------------- земля и искры
def wave_radius(f):
    t = np.clip((f - 10) / 6, 0, 1)
    return 0.35 + (1.62 - 0.35) * ease_out(t) + 0.13 * max(0, f - 16)


def ground_and_air(seed=13):
    rng = np.random.default_rng(seed)
    noise = burn.value_noise(FH, FW, 5, rng)
    jag = fw.CHEB + 0.06 * (noise - 0.5)
    yy, xx = np.mgrid[0:FH, 0:FW]
    sparks = burn.Particles()
    unders, overs = [], []
    pillars_done = False
    for f in range(FRAMES):
        under = np.zeros((FH, FW, 4), np.uint8)
        over = np.zeros((FH, FW, 4), np.uint8)
        # свет на клетке под слоном, пока он светится
        k = glow_level(f)
        if k > 0.1:
            ex, ey = (xx + 0.5 - CX) / 19.0, (yy + 0.5 - CY) / 14.25
            disk = np.hypot(ex, ey) <= 1.0
            under[disk & ((xx + yy + f) % 2 == 0)] = (*hexc(LIGHT[1]), int(200 * k))
            under[disk & (np.hypot(ex, ey) <= 0.55)] = (*hexc(LIGHT[1]), int(230 * k))
        # кольцо света по соседним клеткам
        if f >= 10:
            dc = wave_radius(f) - jag
            zone = fw.AREA & (fw.CHEB > 0.42)
            band = zone & (dc >= 0) & (dc < 0.34)
            for lo, hi, col in ((0.0, 0.07, LIGHT[0]), (0.07, 0.15, LIGHT[1]), (0.15, 0.24, LIGHT[2]), (0.24, 0.34, LIGHT[3])):
                under[band & (dc >= lo) & (dc < hi)] = (*hexc(col), 255)
            trail = zone & (dc >= 0.34) & (dc < 0.75) & ((xx + yy) % 2 == 0)
            under[trail] = (*hexc(LIGHT[1]), int(140 * max(0.0, 1 - (f - 12) / 8)))
            # столбы света над соседними клетками, когда кромка проходит их середину
            if not pillars_done and wave_radius(f) >= 1.0:
                pillars_done = f
        if pillars_done and f - pillars_done < 3:
            age = f - pillars_done
            for di in (-1, 0, 1):
                for dj in (-1, 0, 1):
                    if di == dj == 0:
                        continue
                    px, py = art.iso(di, dj)
                    bx, by = CX + px, CY + py
                    top = 26 - 6 * age
                    for y in range(by - top, by + 1):
                        kk = (by - y) / max(top, 1)
                        half = max(0, int(round(2.5 * (1 - kk))))
                        if (y + age) % 2 == 0 or kk < 0.5:
                            over[y, bx - half : bx + half + 1] = (*hexc(LIGHT[1]), 210 - 50 * age)
                            over[y, bx] = (*hexc(LIGHT[0]), 230 - 50 * age)
        # искры: при свечении собираются к слону, с волной — взлетают от кольца
        if 2 <= f <= 9:
            a = rng.uniform(0, 2 * math.pi)
            sparks.items.append(dict(x=CX + math.cos(a) * 26, y=CY - 20 + math.sin(a) * 18,
                                     vx=-math.cos(a) * 1.6, vy=-math.sin(a) * 1.2, drag=0.9,
                                     life=int(rng.integers(5, 8)), age=0, star=False))
        if 10 <= f <= 19:
            ys, xs = np.nonzero(fw.AREA & (fw.CHEB > 0.5) & (np.abs(wave_radius(f) - fw.CHEB) < 0.1))
            for k2 in rng.choice(len(ys), size=min(4, len(ys)), replace=False) if len(ys) else []:
                sparks.items.append(dict(x=xs[k2] + 0.5, y=ys[k2] - 1.0, vx=rng.uniform(-0.3, 0.3),
                                         vy=rng.uniform(-1.8, -0.8), drag=0.96, life=int(rng.integers(6, 11)),
                                         age=0, star=rng.random() < 0.35, wob=0.3, ph=rng.uniform(0, 6)))
        for p in sparks.items:
            x, y = int(p["x"]), int(p["y"])
            life = p["age"] / p["life"]
            col = hexc(SPARK[min(int(life * 4), 3)])
            if p.get("star") and life < 0.7 and 1 <= x < FW - 1 and 1 <= y < FH - 1:
                over[y, x] = (*hexc(SPARK[0]), 255)
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    over[y + dy, x + dx] = (*col, 255)
            elif 0 <= x < FW and 0 <= y < FH:
                over[y, x] = (*col, 255)
        sparks.step()
        unders.append(under)
        overs.append(over)
    return unders, overs


# ---------------------------------------------------------------- превью
def save_all(out, scale=2):
    os.makedirs(out, exist_ok=True)
    unders, overs = ground_and_air()
    figs = {c: figure_frames(c) for c in "wb"}
    Image.fromarray(np.concatenate(unders, axis=1), "RGBA").save(os.path.join(out, "bless_under.png"))
    Image.fromarray(np.concatenate(overs, axis=1), "RGBA").save(os.path.join(out, "bless_over.png"))
    for c, frames in figs.items():
        Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(out, f"bless_figure_{c}.png"))

    def scene(color, center, position):
        tiles, _, _, _ = fw.board_scene({}, center)
        sprites = fw.load_sprites()
        size = tiles.size
        ox = size[0] // 2 - art.TW // 2
        oy = size[1] // 2 - art.TH // 2 + 10

        def tile_center(sq):
            i, j = sq[0] - center[0], (7 - sq[1]) - (7 - center[1])
            x, y = art.iso(i, j)
            return ox + x + art.TW // 2, oy + y + art.TH // 2

        frames = []
        cx0, cy0 = tile_center(center)
        for k in range(-4, FRAMES + 5):
            f = min(max(k, 0), FRAMES - 1)
            live = 0 <= k < FRAMES
            im = tiles.copy()
            allp = {**position, center: color + "B"}
            order = sorted(allp.items(), key=lambda kv: kv[0][0] + 7 - kv[0][1])
            for sq, code in order:
                x, y = tile_center(sq)
                im.alpha_composite(sprites[code][1], (x - art.AX, y - art.AY))
            if live:
                im.alpha_composite(Image.fromarray(unders[f], "RGBA"), (cx0 - CX, cy0 - CY))
            for sq, code in order:
                x, y = tile_center(sq)
                if sq == center and live:
                    im.alpha_composite(Image.fromarray(figs[color][f], "RGBA"), (x - FAX, y - FAY))
                else:
                    im.alpha_composite(sprites[code][0], (x - art.AX, y - art.AY))
            if live:
                im.alpha_composite(Image.fromarray(overs[f], "RGBA"), (cx0 - CX, cy0 - CY))
            frames.append(im)
        return frames

    # клетки перед слоном (e3, f3, f4) почти пустые — его ничто не закрывает
    neighbours = {(3, 4): "bP", (4, 4): "bN", (3, 3): "wP", (5, 4): "wR", (3, 2): "wN", (4, 2): "wP", (2, 5): "bP"}
    white = scene("w", (4, 3), neighbours)
    black = scene("b", (4, 3), neighbours)
    big = [im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST) for im in white]
    allpix = Image.new("RGB", (big[0].width, big[0].height * len(big)))
    for k, p in enumerate(big):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in big]
    gif[0].save(os.path.join(out, "preview_bless.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    # лист: каждый третий кадр, белый слон сверху, чёрный снизу
    picks = list(range(0, FRAMES, 3))
    w, h = white[0].size
    box = (w // 2 - 95, 0, w // 2 + 95, h - 20)
    bw, bh = box[2] - box[0], box[3] - box[1]
    sheet = Image.new("RGB", (len(picks) * (bw + 4) + 4, 2 * (bh + 4) + 4), (27, 30, 39))
    for row, frames in enumerate((white, black)):
        for n, f in enumerate(picks):
            sheet.paste(frames[f + 4].crop(box).convert("RGB"), (4 + n * (bw + 4), 4 + row * (bh + 4)))
    sheet.save(os.path.join(out, "preview_bless_frames.png"))


# ---------------------------------------------------------------- арт карты
# Белый слон по пикселям (палитра белых фигур, голубая прорезь на митре); крылья — того же
# стиля, что в анимации (`spread_wing`), с пропорциями под окно карты (`WING_CARD`).
BISHOP_ICON = [
    "....h....",
    "...hlm...",
    "..hllmd..",
    "..hlcmd..",
    "..hlmcd..",
    "..hlmmd..",
    "...lmd...",
    "..tttTT..",
    "...lmd...",
    "...lmd...",
    "..llmmd..",
    ".ttttTTT.",
    "hllllmmmd",
    "tttttTTTT",
]
def icon(rows, colors):
    """RGBA значка с контуром в пиксель из строк-символов (цвета по символам)."""
    grid = np.array([list(r) for r in rows])
    h, w = grid.shape
    img = np.zeros((h + 2, w + 2, 4), np.uint8)
    body = np.pad(grid != ".", 1)
    img[bc.outline4(body)] = (*hexc(OUTLINE), 255)
    for ch, col in colors.items():
        img[1:-1, 1:-1][grid == ch] = (*hexc(col), 255)
    return img


def card_art(seed=21):
    """Окно карты 38×25: белый слон с крыльями и нимбом, лучи и кольцо света."""
    w, h = fw.ART_SIZE
    rng = np.random.default_rng(seed)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    horizon = 12
    for y0, y1, col in ((0, 4, "#11141f"), (4, 8, "#1a1824"), (8, horizon, "#2e2820")):
        img[(yy >= y0) & (yy < y1)] = (*hexc(col), 255)
    img[(yy == 7) & ((xx + yy) % 2 == 0)] = (*hexc("#2e2820"), 255)
    # лучи от головы слона: светлые клинья через один
    ang = np.degrees(np.arctan2(yy + 0.5 - 8, xx + 0.5 - 19)) % 360
    rays = (yy < horizon) & ((ang // 18) % 2 == 0) & (np.hypot(xx - 19, (yy - 8) * 1.3) < 20)
    img[rays & (yy < 4)] = (*hexc("#1a1c28"), 255)
    img[rays & (yy >= 4) & (yy < 8)] = (*hexc("#262230"), 255)
    img[rays & (yy >= 8)] = (*hexc("#3d3424"), 255)
    # земля-доска и кольцо света вокруг слона
    ground = yy >= horizon
    gu = (xx + 0.5 - w / 2) / 8 + (yy + 0.5 - 18) / 4
    gv = (yy + 0.5 - 18) / 4 - (xx + 0.5 - w / 2) / 8
    checker = (np.floor(gu).astype(int) + np.floor(gv).astype(int)) % 2 == 0
    img[ground & checker] = (*hexc("#2b2320"), 255)
    img[ground & ~checker] = (*hexc("#221c1b"), 255)
    img[yy == horizon] = (*hexc("#4a3c24"), 255)
    d = np.abs(xx + 0.5 - 19) / 15.0 + np.abs(yy + 0.5 - 19.5) / 3.6
    img[ground & (d < 0.8) & ((xx + yy) % 2 == 0)] = (*hexc("#4a3c24"), 255)
    img[ground & (d >= 0.8) & (d <= 1.0)] = (*hexc(LIGHT[2]), 255)
    img[ground & (d >= 0.88) & (d <= 1.0)] = (*hexc(LIGHT[1]), 255)

    def put_icon(spr, x0, y0):
        m = spr[..., 3] > 0
        img[y0 : y0 + spr.shape[0], x0 : x0 + spr.shape[1]][m] = spr[m]

    # крылья за слоном, потом слон, нимб над митрой
    bishop = icon(BISHOP_ICON, fw.PAWN_COLORS)
    bx, by = 19 - bishop.shape[1] // 2, 21 - bishop.shape[0] + 1
    for right, sx in ((False, bx + 4), (True, bx + bishop.shape[1] - 4)):
        spr, (ax, ay) = spread_wing(right, WING_CARD)
        x0, y0 = sx - ax, by + 9 - ay  # плечи — у воротника слона
        ys, xs = np.nonzero(spr[..., 3])
        for y, x in zip(ys, xs):
            if 0 <= y0 + y < h and 0 <= x0 + x < w:
                img[y0 + y, x0 + x] = spr[y, x]
    put_icon(bishop, bx, by)
    halo = np.abs(np.hypot((xx + 0.5 - 19.5) / 3.6, (yy + 0.5 - 4.5) / 1.3) - 1.0) < 0.28
    img[halo] = (*hexc(LIGHT[2]), 255)
    img[halo & (yy + 0.5 < 4.5)] = (*hexc(LIGHT[1]), 255)
    for _ in range(7):  # искры
        x, y = int(rng.integers(1, w - 1)), int(rng.integers(1, horizon + 6))
        if img[y, x, :3].sum() < 200:
            img[y, x] = (*hexc(SPARK[int(rng.integers(0, 3))]), 255)
    return img


def save_card_art(art_img):
    art_dir = os.path.join(art.ROOT, "assets", "breach", "cards", "art")
    os.makedirs(art_dir, exist_ok=True)
    Image.fromarray(art_img, "RGBA").save(os.path.join(art_dir, "bless.png"))
    faces = []
    for kind in ("w", "b", "victory", "sacrifice"):
        for sel in (False, True):
            face = bc.card_front(kind, selected=sel).copy()
            x0, y0 = fw.ART_AT
            face[y0 : y0 + fw.ART_SIZE[1], x0 : x0 + fw.ART_SIZE[0]] = art_img
            faces.append(face)
    gap = 10
    sheet = Image.new("RGBA", (4 * (bc.CW + gap) + gap, 2 * (bc.CH + gap) + gap), hexc("#1b1e27") + (255,))
    for n, face in enumerate(faces):
        col, row = n // 2, n % 2
        x, y = gap + col * (bc.CW + gap), gap + row * (bc.CH + gap)
        sheet.alpha_composite(Image.fromarray(bc.shadow_of(face), "RGBA"), (x, y))
        sheet.alpha_composite(Image.fromarray(face, "RGBA"), (x, y))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(art_dir, "preview_bless_card.png"))


if __name__ == "__main__":
    save_all(OUT)
    save_card_art(card_art())
    print("done", OUT)
