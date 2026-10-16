"""Портал под фигурой (исчезновение и появление) и арт карты обмена в стиле Into the Breach.

Под фигурой раскрывается изометрический портал: тёмно-фиолетовый вихрь с вращающимися
рукавами, голубое кольцо с бегущими бликами, мерцающее свечение на клетке. Фигура
проваливается в него — её срезает передняя кромка кольца, срез светится, — портал
закрывается со вспышкой-столбом. Появление — наоборот: портал раскрывается, фигура
поднимается, кольцо схлопывается и по клетке расходится волна. Вокруг — искры.

Кадр FW×FH, центр верхней грани клетки — (CX, CY). Слои: `under` (вихрь, задняя половина
кольца, свечение — до фигур) и `over` (передняя половина кольца, искры, вспышка — после).
Фигура в кадре f сдвинута вниз на SINK[вид][f] пикселей арта и срезана по `front_rim`:
видна часть выше передней кромки кольца (масштаб кольца — SCALE[f]).

`py tools/breach_portal.py [папка]` — по умолчанию assets/breach/effects:
portal_<vanish|appear>_<under|over>.png (кадры в ряд), preview_portal.gif,
preview_portal_frames.png; арт карты — assets/breach/cards/art/swap.png и
preview_swap_card.png.
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
ART_DIR = os.path.join(art.ROOT, "assets", "breach", "cards", "art")

TW, TH = art.TW, art.TH
FW, FH = 80, 116
CX, CY = 40, 88  # центр верхней грани клетки
RX, RY = 17.0, 12.75  # портал — круг на земле в изометрии (0.75)
FRAMES = 16
FRAME_MS = 60
DEPTH = 72  # на столько фигура уходит вниз — ниже любой фигуры

VORTEX = ["#0c0818", "#1e1240", "#33206e", "#4a2fa0", "#6a44d6"]
RIM = {"inner": "#2aa9d6", "outer": "#57d9ff", "spark": "#e9fdff", "line": "#0b1622"}
GLOW = (*hexc("#57d9ff"), 110)
MOTES = ["#e9fdff", "#a6f2ff", "#57d9ff", "#6a44d6"]
CUT = "#a6f2ff"  # светящийся срез фигуры

# Масштаб кольца по кадрам: раскрытие с перелётом, удержание, схлопывание.
SCALE = [0.15, 0.45, 0.8, 1.06] + [1.0] * 8 + [0.85, 0.55, 0.25, 0.0]


def ease_in(u):
    return u * u


SINK = {
    "vanish": [0.0] * 4 + [DEPTH * ease_in((f - 3) / 8) for f in range(4, 12)] + [float(DEPTH)] * 4,
    "appear": [float(DEPTH)] * 4 + [DEPTH * (1 - (f - 3) / 8) ** 2 for f in range(4, 12)] + [0.0] * 4,
}


def ellipse_coords(scale):
    yy, xx = np.mgrid[0:FH, 0:FW]
    ex = (xx + 0.5 - CX) / (RX * max(scale, 1e-3))
    ey = (yy + 0.5 - CY) / (RY * max(scale, 1e-3))
    return ex, ey, np.hypot(ex, ey), np.arctan2(ey, ex)


def front_rim(scale):
    """Для каждого столбца — y передней кромки кольца; вне портала — −1 (всё срезано)."""
    xs = np.arange(FW) + 0.5 - CX
    rx, ry = RX * scale, RY * scale
    inside = np.abs(xs) < rx
    rim = np.full(FW, -1.0)
    rim[inside] = CY + ry * np.sqrt(1 - (xs[inside] / rx) ** 2)
    return rim


def portal_layers(f, kind, rng, motes):
    """Слои портала в кадре f: under и over (RGBA FH×FW)."""
    under = np.zeros((FH, FW, 4), np.uint8)
    over = np.zeros((FH, FW, 4), np.uint8)
    s = SCALE[f]
    yy, xx = np.mgrid[0:FH, 0:FW]
    if s > 0.05:
        ex, ey, r, th = ellipse_coords(s)
        rot = f * 0.11
        disk = r <= 1.0
        # свечение на клетке: мерцающий шахматный узор вокруг кольца
        glow = (r > 1.0) & (r <= 1.28) & ((xx + yy + f) % 2 == 0)
        under[glow] = GLOW
        # вихрь: тёмный центр, светлее к краю, рукава закручиваются
        tone = np.select([r < 0.35, r < 0.6], [0, 1], 2)
        arm = ((th / (2 * math.pi)) * 3 + r * 1.3 + rot) % 1 < 0.28
        tone = np.where(arm & (r > 0.2), tone + 2, tone)
        inner = disk & (r < 0.84)
        for k, col in enumerate(VORTEX):
            under[inner & (tone == k)] = (*hexc(col), 255)
        # кольцо: голубое, снаружи светлее; блики бегут по кругу
        ring = disk & (r >= 0.84)
        sparks = np.zeros_like(ring)
        for k in range(5):
            a = rot * 2.2 + k * 2 * math.pi / 5
            sparks |= np.abs(np.angle(np.exp(1j * (th - a)))) < 0.22
        line = bc.outline4(disk)
        for layer, half in ((under, ey < 0), (over, ey >= 0)):
            layer[ring & half] = (*hexc(RIM["inner"]), 255)
            layer[ring & half & (r > 0.93)] = (*hexc(RIM["outer"]), 255)
            layer[ring & half & sparks] = (*hexc(RIM["spark"]), 255)
            layer[line & half] = (*hexc(RIM["line"]), 255)
    elif kind == "vanish" or f == 0:
        # искра: точка с крестиком — портал только появился или схлопнулся
        for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
            over[CY + dy, CX + dx] = (*hexc(RIM["spark"]), 255)

    # вспышка-столб: фигура ушла (исчезновение) или только показалась (появление)
    if (kind == "vanish" and f == 12) or (kind == "appear" and f == 4):
        for y in range(CY - 62, CY + 1):
            k = (CY - y) / 62
            half = max(1, int(round(4 * (1 - k * 0.7))))
            if (y + f) % 2 == 0 or k < 0.6:
                over[y, CX - half : CX + half + 1] = (*hexc(RIM["outer"]), 200)
                over[y, CX - 1 : CX + 2] = (*hexc(RIM["spark"]), 230)
    # волна по клетке при появлении: тонкое кольцо расходится и тает
    if kind == "appear" and 12 <= f <= 14:
        ex, ey, r, th = ellipse_coords(1.0)
        rr = 1.05 + 0.22 * (f - 12)
        wave = (np.abs(r - rr) < 0.06) & ((xx + yy) % (1 + f - 12) == 0)
        under[wave] = (*hexc(RIM["outer"]), 220)

    # искры: поднимаются от кольца, по пути синеют
    if 1 <= f <= 13 and s > 0.3:
        for _ in range(3):
            a = rng.uniform(0, 2 * math.pi)
            motes.items.append(dict(x=CX + math.cos(a) * RX * s, y=CY + math.sin(a) * RY * s,
                                    vx=rng.uniform(-0.3, 0.3), vy=rng.uniform(-2.0, -0.9), drag=0.97,
                                    life=int(rng.integers(6, 12)), age=0, wob=0.4, ph=rng.uniform(0, 6)))
    for p in motes.items:
        x, y = int(p["x"]), int(p["y"])
        if 0 <= x < FW and 0 <= y < FH:
            life = p["age"] / p["life"]
            over[y, x] = (*hexc(MOTES[min(int(life * 4), 3)]), 255)
            if p["age"] < 2 and y + 1 < FH:
                over[y + 1, x] = (*hexc(MOTES[2]), 255)
    motes.step()
    return under, over


def render(kind, seed=3):
    rng = np.random.default_rng(seed)
    motes = burn.Particles()
    unders, overs = [], []
    for f in range(FRAMES):
        u, o = portal_layers(f, kind, rng, motes)
        unders.append(u)
        overs.append(o)
    return unders, overs


def sunk_piece(body, f, kind):
    """Фигура в кадре f в координатах кадра эффекта: сдвинута вниз и срезана кромкой."""
    out = np.zeros((FH, FW, 4), np.uint8)
    spr = np.array(body)
    off = int(round(SINK[kind][f]))
    x0, y0 = CX - art.AX, CY - art.AY + off
    h, w = spr.shape[:2]
    ys, xs = slice(max(0, y0), min(FH, y0 + h)), slice(max(0, x0), min(FW, x0 + w))
    out[ys, xs] = spr[ys.start - y0 : ys.stop - y0, xs.start - x0 : xs.stop - x0]
    if off > 0:
        rim = front_rim(max(SCALE[f], 0.0))
        yy = np.mgrid[0:FH, 0:FW][0]
        cut = (yy + 0.5) >= rim[None, :]
        solid = out[..., 3] > 0
        edge = solid & ~cut & np.vstack([cut[1:], np.ones((1, FW), bool)])
        out[cut] = 0
        out[edge] = (*hexc(CUT), 255)
    return out


# ---------------------------------------------------------------- превью на доске
def save_effect(out, scale=2):
    os.makedirs(out, exist_ok=True)
    layers = {k: render(k, seed=3 if k == "vanish" else 7) for k in ("vanish", "appear")}
    for kind, (unders, overs) in layers.items():
        Image.fromarray(np.concatenate(unders, axis=1), "RGBA").save(os.path.join(out, f"portal_{kind}_under.png"))
        Image.fromarray(np.concatenate(overs, axis=1), "RGBA").save(os.path.join(out, f"portal_{kind}_over.png"))

    # белый конь c3 и чёрный слон f5 меняются местами
    a, b = (2, 2), (5, 4)
    swap = {a: "wN", b: "bB"}
    static = {(3, 1): "wP", (4, 2): "wP", (1, 1): "wP", (4, 5): "bP", (6, 5): "bP", (3, 4): "bN"}
    size, center = (320, 250), (4, 3)
    tiles, _, _, _ = fw.board_scene({}, center, size=size)
    sprites = fw.load_sprites()
    ox = size[0] // 2 - TW // 2
    oy = size[1] // 2 - TH // 2 + 10

    def tile_center(sq):
        i, j = sq[0] - center[0], (7 - sq[1]) - (7 - center[1])
        x, y = art.iso(i, j)
        return ox + x + TW // 2, oy + y + TH // 2

    def frame(kind, f, where):
        """kind None — без эффекта; where — какая фигура на какой клетке."""
        im = tiles.copy()
        allp = {**static, **where}
        order = sorted(allp.items(), key=lambda kv: kv[0][0] + 7 - kv[0][1])
        for sq, code in order:  # тени — только у фигур, что стоят на месте
            if kind is None or sq not in swap or SINK[kind][f] == 0:
                cx, cy = tile_center(sq)
                im.alpha_composite(sprites[code][1], (cx - art.AX, cy - art.AY))
        if kind is not None:
            for sq in swap:
                cx, cy = tile_center(sq)
                im.alpha_composite(Image.fromarray(layers[kind][0][f], "RGBA"), (cx - CX, cy - CY))
        for sq, code in order:
            cx, cy = tile_center(sq)
            if kind is not None and sq in swap:
                piece = sunk_piece(sprites[code][0], f, kind)
                im.alpha_composite(Image.fromarray(piece, "RGBA"), (cx - CX, cy - CY))
            else:
                im.alpha_composite(sprites[code][0], (cx - art.AX, cy - art.AY))
        if kind is not None:
            for sq in swap:
                cx, cy = tile_center(sq)
                im.alpha_composite(Image.fromarray(layers[kind][1][f], "RGBA"), (cx - CX, cy - CY))
        return im

    swapped = {a: swap[b], b: swap[a]}
    seq = [frame(None, 0, swap)] * 5
    seq += [frame("vanish", f, swap) for f in range(FRAMES)]
    seq += [frame("appear", f, swapped) for f in range(FRAMES)]
    seq += [frame(None, 0, swapped)] * 8
    big = [im.convert("RGB").resize((size[0] * scale, size[1] * scale), Image.NEAREST) for im in seq]
    allpix = Image.new("RGB", (big[0].width, big[0].height * len(big)))
    for k, p in enumerate(big):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in big]
    gif[0].save(os.path.join(out, "preview_portal.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    # лист: каждый второй кадр исчезновения и появления крупно у одной клетки
    cx, cy = tile_center(a)
    box = (cx - 44, cy - 92, cx + 44, cy + 24)
    picks = [("vanish", f) for f in range(0, FRAMES, 2)] + [("appear", f) for f in range(0, FRAMES, 2)]
    w, h = box[2] - box[0], box[3] - box[1]
    sheet = Image.new("RGB", (8 * (w + 4) + 4, 2 * (h + 4) + 4), (27, 30, 39))
    for n, (kind, f) in enumerate(picks):
        im = frame(kind, f, swap if kind == "vanish" else swapped).crop(box)
        sheet.paste(im.convert("RGB"), (4 + (n % 8) * (w + 4), 4 + (n // 8) * (h + 4)))
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(
        os.path.join(out, "preview_portal_frames.png")
    )


# ---------------------------------------------------------------- арт карты обмена
def thick_arc(p0, c, p1, width=1.0, n=200):
    """Маска толстой квадратичной дуги на холсте ART_SIZE и касательная в конце."""
    w, h = fw.ART_SIZE
    t = np.linspace(0, 1, n)
    x = (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * c[0] + t**2 * p1[0]
    y = (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * c[1] + t**2 * p1[1]
    yy, xx = np.mgrid[0:h, 0:w]
    d = np.min(np.hypot(xx[..., None] + 0.5 - x, yy[..., None] + 0.5 - y), axis=2)
    ang = math.atan2(y[-1] - y[-8], x[-1] - x[-8])
    return d <= width, ang


def arrowhead(tip, ang, length=4.5, half=3.2):
    w, h = fw.ART_SIZE
    yy, xx = np.mgrid[0:h, 0:w]
    ux, uy = math.cos(ang), math.sin(ang)
    base = (tip[0] - ux * length, tip[1] - uy * length)
    poly = [tip, (base[0] - uy * half, base[1] + ux * half), (base[0] + uy * half, base[1] - ux * half)]
    return art.sd_polygon(xx + 0.5, yy + 0.5, poly) <= 0.2


def card_art(seed=9):
    """Окно карты 38×25: две белые пешки на порталах и знак обмена ⇄ между ними."""
    w, h = fw.ART_SIZE
    rng = np.random.default_rng(seed)
    img = np.zeros((h, w, 4), np.uint8)
    yy, xx = np.mgrid[0:h, 0:w]
    horizon = 12
    # фон как у огненной волны: ночное небо полосами, тёмная доска; отсвет — фиолетовый
    for y0, y1, col in ((0, 4, "#11141f"), (4, 8, "#161a27"), (8, horizon, "#1e1636")):
        img[(yy >= y0) & (yy < y1)] = (*hexc(col), 255)
    img[(yy == 7) & ((xx + yy) % 2 == 0)] = (*hexc("#1e1636"), 255)
    ground = yy >= horizon
    gu = (xx + 0.5 - w / 2) / 8 + (yy + 0.5 - 18) / 4
    gv = (yy + 0.5 - 18) / 4 - (xx + 0.5 - w / 2) / 8
    checker = (np.floor(gu).astype(int) + np.floor(gv).astype(int)) % 2 == 0
    img[ground & checker] = (*hexc("#2b2320"), 255)
    img[ground & ~checker] = (*hexc("#221c1b"), 255)
    img[yy == horizon] = (*hexc("#2e2244"), 255)
    for _ in range(6):  # звёзды — только в верхней части неба, над пешками и стрелками
        img[rng.integers(0, 5), rng.integers(1, w - 1)] = (*hexc("#a6f2ff"), 255)
    # порталы под пешками
    feet = [(7.5, 19.5), (30.5, 19.5)]
    for fx, fy in feet:
        ex, ey = (xx + 0.5 - fx) / 6.5, (yy + 0.5 - fy) / 2.6
        r = np.hypot(ex, ey)
        img[r <= 1.25] = (*hexc("#0b1622"), 255)
        img[r <= 1.0] = (*hexc(RIM["outer"]), 255)
        img[r <= 0.8] = (*hexc(VORTEX[2]), 255)
        img[r <= 0.5] = (*hexc(VORTEX[0]), 255)
        img[(r > 0.8) & (r <= 1.0) & (ey < 0)] = (*hexc(RIM["inner"]), 255)
    # две белые пешки стоят в порталах
    pawn = fw.pawn_icon()
    ph, pw = pawn.shape[:2]
    m = pawn[..., 3] > 0
    for fx, fy in feet:
        x0, y0 = int(round(fx - pw / 2)), int(round(fy + 1 - ph))
        img[y0 : y0 + ph, x0 : x0 + pw][m] = pawn[m]
    # знак обмена ⇄ в просвете между пешками: голубые стрелки, как огни белых
    arrow = np.array([[ch == "#" for ch in row] for row in SWAP_ARROW])
    for flip, y0 in ((False, 8), (True, 14)):
        col, light = "#57d9ff", "#a6f2ff"
        shape = np.zeros((h, w), bool)
        a = arrow[:, ::-1] if flip else arrow
        x0 = (w - a.shape[1]) // 2
        shape[y0 : y0 + a.shape[0], x0 : x0 + a.shape[1]] = a
        img[bc.outline4(shape)] = (*hexc("#08090d"), 255)
        img[shape] = (*hexc(col), 255)
        img[shape & ~np.vstack([np.zeros((1, w), bool), shape[:-1]])] = (*hexc(light), 255)
    return img


# Стрелка знака обмена остриём вправо, по пикселям: древко в пиксель, наконечник 3×5.
SWAP_ARROW = [
    ".......#..",
    ".......##.",
    "##########",
    ".......##.",
    ".......#..",
]


def save_card_art(art_img):
    os.makedirs(ART_DIR, exist_ok=True)
    Image.fromarray(art_img, "RGBA").save(os.path.join(ART_DIR, "swap.png"))
    faces = []
    for kind in ("w", "b"):
        for sel in (False, True):
            face = bc.card_front(kind, selected=sel).copy()
            x0, y0 = fw.ART_AT
            face[y0 : y0 + fw.ART_SIZE[1], x0 : x0 + fw.ART_SIZE[0]] = art_img
            faces.append(face)
    gap = 10
    sheet = Image.new("RGBA", (4 * (bc.CW + gap) + gap, bc.CH + 2 * gap), hexc("#1b1e27") + (255,))
    for n, face in enumerate(faces):
        x = gap + n * (bc.CW + gap)
        sheet.alpha_composite(Image.fromarray(bc.shadow_of(face), "RGBA"), (x, gap))
        sheet.alpha_composite(Image.fromarray(face, "RGBA"), (x, gap))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(ART_DIR, "preview_swap_card.png"))


if __name__ == "__main__":
    save_effect(OUT)
    save_card_art(card_art())
    print("done", OUT, ART_DIR)
