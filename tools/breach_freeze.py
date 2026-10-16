"""Заморозка клетки и фигуры в стиле Into the Breach — повторяемая по клеткам.

Эффект собран из одинаковых кусков на одну клетку, поэтому годится и для одной фигуры,
и для половины поля: на каждой клетке играются те же кадры, а начало сдвигается на
`DELAY_PER_TILE` кадров на каждую клетку расстояния от точки заморозки — получается
расходящаяся волна. Лёд в конце заполняет весь ромб клетки, так что соседние замёрзшие
клетки сливаются в сплошной лёд. Последний кадр — состояние «заморожено», его держат.

Куски (кадр клетки FW×FH, центр верхней грани — (CX, CY), FRAMES кадров по FRAME_MS):
- `freeze_tile_<v>.png` — снег на клетке: наметает от середины, сугробики, искорки,
  спереди бортик слоя — до фигур;
- `freeze_block_<P|N|B|R|Q|K>_<v>.png` — полупрозрачная глыба льда вокруг фигуры своей
  высоты (сквозь неё видно фигуру) — после фигуры; на пустой клетке не рисуется;
- v — один из `VARIANTS` = 4 вариантов (рисунок снега; глыба: брусок, восьмигранник,
  узкая с осколками, скошенная с осколком); клетка берёт `variant_of(file, rank)`;
- `freeze_over.png` — снежинки, искры, холодный пар — после фигур;
- `pieces/frozen/<цвет><фигура>.png` — фигуры, покрытые инеем (как обычные спрайты
  64×84, точка опоры 32,66); иней поднимается от основания: в кадре f видна замёрзшая
  часть ниже линии `FROST[f]` (доля высоты фигуры `PIECE_TOP`), выше — обычная фигура.

`py tools/breach_freeze.py [папка]` — по умолчанию assets/breach/effects, превью:
preview_freeze_one.gif (одна фигура), preview_freeze_half.gif (половина поля).
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
PIECES_DIR = os.path.join(art.ROOT, "assets", "breach", "pieces")

TW, TH = art.TW, art.TH
FW, FH = 80, 116
CX, CY = 40, 88
FRAMES = 18
FRAME_MS = 60
DELAY_PER_TILE = 2  # кадров задержки на клетку расстояния — для волны по многим клеткам

ICE = ["#f2fbff", "#c6e8f6", "#8fcbe6", "#5a98bc", "#2f5f80"]  # блик … тень (глыбы, иней)
ICE_LINE = "#0b1622"
FROST_TINT = np.array(hexc("#cdeefa"), float)

# Ход эффекта по кадрам
TILE_R = [0.0, 0.18, 0.36, 0.54, 0.72, 0.88, 1.0] + [1.06] * 11  # лёд на клетке
FROST = [0.0] * 2 + [min(1.0, (f - 1) / 8) for f in range(2, 10)] + [1.0] * 8  # иней на фигуре
BLOCK = [0.0] * 8 + [0.15, 0.35, 0.6, 0.82, 0.95, 1.0] + [1.0] * 4  # высота глыбы
CLINK = 14  # кадр вспышки на углу глыбы


def piece_tops():
    """Высота каждой фигуры над точкой опоры (по непрозрачным пикселям спрайта)."""
    tops = {}
    for k in "PNBRQK":
        a = np.array(Image.open(os.path.join(PIECES_DIR, f"w{k}.png")))[..., 3]
        tops[k] = art.AY - int(np.nonzero(a.any(axis=1))[0].min())
    return tops


PIECE_TOP = piece_tops()


# ---------------------------------------------------------------- фигуры в инее
def frozen_sprite(body):
    """Фигура под инеем: цвета к ледяному, контур — тёмно-синий, блики сверху-слева."""
    img = body.astype(float)
    solid = body[..., 3] > 0
    dark = body[..., :3].sum(axis=2) < 60
    img[..., :3] = img[..., :3] * 0.45 + FROST_TINT * 0.55
    out = img.astype(np.uint8)
    out[solid & dark, :3] = hexc(ICE_LINE)
    lit, _ = bc.shade_mask(solid & ~dark)
    out[lit, :3] = hexc(ICE[0])
    return out


def frost_reveal(body, frozen, f, kind):
    """Спрайт в кадре f: ниже линии инея — замёрзшая фигура, выше — обычная."""
    line = art.AY - FROST[f] * (PIECE_TOP[kind] + 2)
    out = body.copy()
    rows = np.arange(body.shape[0])[:, None] >= line
    m = rows & (body[..., 3] > 0)
    out[m] = frozen[m]
    if FROST[f] < 1:  # кромка инея — белая полоса, ползущая вверх
        above = np.vstack([np.zeros((1, m.shape[1]), bool), m[:-1]])
        out[m & ~above & (body[..., :3].sum(axis=2) >= 60)] = (*hexc(ICE[0]), 255)
    return out


# ---------------------------------------------------------------- снег на клетке
YY, XX = np.mgrid[0:FH, 0:FW]
DX, DY = XX + 0.5 - CX, YY + 0.5 - CY
DMD = np.abs(DX) / (TW / 2) + np.abs(DY) / (TH / 2)  # 1 — край ромба клетки
SNOW = ["#ffffff", "#eef5fa", "#d8e6f0", "#b9cddd", "#8ea9bf"]  # блик … глубокая тень
VARIANTS = 4  # рисунков снега и форм глыб; клетка берёт свой — `variant_of`


def variant_of(file, rank):
    """Вариант снега и глыбы для клетки — соседние клетки получают разные."""
    return (file * 3 + rank * 5) % VARIANTS


def tile_frames(variant=0):
    """Снег на клетке: наметает от середины, мягкие сугробики пятнами, искорки; спереди —
    бортик снежного слоя (виден на краю замёрзшей области)."""
    rng = np.random.default_rng(40 + variant)
    grow = burn.value_noise(FH, FW, 4, rng)
    lumps = 0.65 * burn.value_noise(FH, FW, 4, rng) + 0.35 * burn.value_noise(FH, FW, 2, rng)
    jag = DMD + 0.10 * (grow - 0.5)
    glints = rng.random((FH, FW)) < 0.03
    lip = (DY > 0) & (DMD > 1.0) & (DMD <= 1.0 + 2.2 / (TH / 2))  # толщина слоя у передних рёбер
    frames = []
    for f in range(FRAMES):
        img = np.zeros((FH, FW, 4), np.uint8)
        R = TILE_R[f]
        snow = (DMD <= 1.0) & (jag <= R)
        img[snow] = (*hexc(SNOW[2]), 255)
        img[snow & (lumps > 0.58)] = (*hexc(SNOW[1]), 255)
        img[snow & (lumps > 0.70)] = (*hexc(SNOW[0]), 255)
        img[snow & (lumps < 0.38)] = (*hexc(SNOW[3]), 255)
        img[snow & glints] = (*hexc(SNOW[0]), 255)
        img[snow & (DY < 0) & (DMD > 0.94)] = (*hexc(SNOW[3]), 255)  # задние кромки — сетка
        if R >= 1.0:
            img[lip] = (*hexc(SNOW[3]), 255)
            img[lip & (DMD > 1.0 + 1.2 / (TH / 2))] = (*hexc(SNOW[4]), 255)
        else:
            img[snow & (jag > R - 0.08)] = (*hexc(SNOW[0]), 255)  # пушистая кромка наметания
        frames.append(img)
    return frames


# ---------------------------------------------------------------- глыбы льда
def mask(points):
    return art.sd_polygon(XX + 0.5, YY + 0.5, points) <= 0


def segment(a, b, w=0.6):
    return art.sd_segment2(XX + 0.5, YY + 0.5, a, b) < w


def prism(base, heights, g, apex=None):
    """Призма над многоугольником `base` (по часовой на экране, точки на земле): видимые
    боковые грани (левые и правые), верх и рёбра. apex — острый осколок: вместо верха
    грани сходятся к точке (смещение над центром основания)."""
    tops = [(x, y - h * g) for (x, y), h in zip(base, heights)]
    if apex is not None:
        cx = sum(p[0] for p in base) / len(base)
        cy = sum(p[1] for p in base) / len(base)
        tip = (cx + apex[0] * g, cy - apex[1] * g)
    left = np.zeros((FH, FW), bool)
    right = np.zeros((FH, FW), bool)
    edges = np.zeros((FH, FW), bool)
    n = len(base)
    for i in range(n):
        a, b = base[i], base[(i + 1) % n]
        if b[0] - a[0] >= 0:  # грань смотрит от зрителя
            continue
        face = [a, b, tip] if apex is not None else [a, b, tops[(i + 1) % n], tops[i]]
        m = mask(face)
        if (b[1] - a[1]) < 0:  # нормаль влево — грань светлее
            left |= m
        else:
            right |= m
        edges |= segment(a, tip if apex is not None else tops[i])
        edges |= segment(b, tip if apex is not None else tops[(i + 1) % n])
    cap = mask(tops) if apex is None else np.zeros((FH, FW), bool)
    if apex is None:
        for i in range(n):
            edges |= segment(tops[i], tops[(i + 1) % n])
    return left, right, cap, edges


def iso_poly(rx, ry, k, rot=0.0, dx=0.0, dy=0.0):
    """Многоугольник на земле: k вершин на изометрическом эллипсе, по часовой."""
    return [(CX + dx + rx * math.cos(rot + 2 * math.pi * i / k), CY + dy + ry * math.sin(rot + 2 * math.pi * i / k))
            for i in range(k)]


def block_shapes(kind, variant):
    """Части глыбы варианта: (основание, высоты вершин, остриё или None, с какой доли
    роста появляется) — от задних к передним."""
    h = PIECE_TOP[kind] + 5
    diamond = [(CX - 19, CY), (CX, CY - 14.25), (CX + 19, CY), (CX, CY + 14.25)]
    if variant == 0:  # глыба-брусок
        return [(diamond, [h - 1, h + 1, h - 2, h], None, 0.0)]
    if variant == 1:  # восьмигранная призма, верх чуть неровный
        octa = iso_poly(20, 15, 8, rot=math.pi / 8)
        return [(octa, [h + 2, h + 1, h + 3, h + 2, h + 1, h + 2, h + 3, h + 1], None, 0.0)]
    if variant == 2:  # узкая глыба и острые осколки по бокам
        core = [(CX - 15, CY), (CX, CY - 11.25), (CX + 15, CY), (CX, CY + 11.25)]
        shard = lambda cx, cy, s: [(cx - s, cy), (cx, cy - s * 0.75), (cx + s, cy), (cx, cy + s * 0.75)]  # noqa: E731
        return [
            (shard(CX - 14, CY - 3, 5), [0] * 4, (-6, h * 0.55), 0.5),
            (core, [h, h + 1, h - 1, h], None, 0.0),
            (shard(CX + 14, CY + 4, 5), [0] * 4, (6, h * 0.45), 0.55),
            (shard(CX - 9, CY + 10, 3.5), [0] * 4, (-4, h * 0.3), 0.65),
        ]
    # скошенный скол сверху и осколок справа
    return [
        (diamond, [h + 5, h + 7, h - 3, h - 1], None, 0.0),
        (iso_poly(5, 3.75, 4, dx=16, dy=3), [0] * 4, (5, h * 0.5), 0.55),
    ]


def block_frames(kind, variant=0, seed=9):
    """Полупрозрачная глыба льда варианта `variant` вокруг фигуры `kind`, растёт снизу."""
    rng = np.random.default_rng(seed + "PNBRQK".index(kind) * 7 + variant)
    parts = block_shapes(kind, variant)
    frames = []
    for f in range(FRAMES):
        img = np.zeros((FH, FW, 4), np.uint8)
        g = BLOCK[f]
        if g <= 0:
            frames.append(img)
            continue
        body = np.zeros((FH, FW), bool)
        for base, heights, apex, start in parts:
            gg = 1.0 if g >= 1 else max(0.0, (g - start) / (1 - start)) if start else g
            if gg <= 0:
                continue
            left, right, cap, edges = prism(base, heights, gg, apex)
            part = left | right | cap
            if g < 0.95 and apex is None:  # растущая кромка неровная, как намерзающий лёд
                top_y = min(y for _, y in base) - max(heights) * gg
                jag = rng.integers(0, 3, size=FW)
                cut = YY < (top_y + jag[None, :])
                part &= ~cut
                left, right, cap, edges = left & ~cut, right & ~cut, cap & ~cut, edges & ~cut
            # части рисуются от задних к передним: передняя перекрывает заднюю
            img[left] = (*hexc(ICE[2]), 110)
            img[right] = (*hexc(ICE[3]), 140)
            img[cap] = (*hexc(ICE[1]), 130)
            u = XX - YY * 0.6
            img[left & (np.abs((u % 14) - 7) < 0.6)] = (*hexc(ICE[0]), 170)
            img[right & (np.abs(((XX + YY * 0.6) % 16) - 8) < 0.6)] = (*hexc(ICE[1]), 170)
            img[edges & part] = (*hexc(ICE[0]), 230)
            ring = bc.outline4(part) & ~body
            img[ring] = (*hexc(ICE_LINE), 255)
            body |= part
        img[bc.outline4(body)] = (*hexc(ICE_LINE), 255)
        # иней у земли вдоль передних нижних рёбер основной глыбы
        main = parts[0][0] if parts[0][2] is None else parts[1][0]
        for i in range(len(main)):
            a, b = main[i], main[(i + 1) % len(main)]
            if b[0] - a[0] < 0:
                img[segment(a, b, 1.3)] = (*hexc(ICE[0]), 255)
        # вспышка-«звяк» на верхнем левом углу, когда глыба встала
        if f in (CLINK, CLINK + 1):
            ys, xs = np.nonzero(body)
            k = np.argmin(xs + ys * 0.5)
            tx, ty = xs[k] + 1, ys[k] + 1
            r = 3 if f == CLINK else 2
            for d in range(-r, r + 1):
                for x, y in ((tx + d, ty), (tx, ty + d)):
                    if 0 <= x < FW and 0 <= y < FH:
                        img[y, x] = (*hexc("#ffffff"), 255)
        frames.append(img)
    return frames


# ---------------------------------------------------------------- снег и пар
def over_frames(seed=12):
    rng = np.random.default_rng(seed)
    flakes = burn.Particles()
    frames = []
    for f in range(FRAMES):
        img = np.zeros((FH, FW, 4), np.uint8)
        if f < 13:  # снежинки падают на клетку, покачиваясь
            for _ in range(2):
                flakes.items.append(dict(x=CX + rng.uniform(-24, 24), y=CY - rng.uniform(40, 70),
                                         vx=0, vy=rng.uniform(1.6, 2.6), life=int(rng.integers(8, 14)),
                                         age=0, wob=0.5, ph=rng.uniform(0, 6), big=rng.random() < 0.4))
        for p in flakes.items:
            x, y = int(p["x"]), int(p["y"])
            if 1 <= x < FW - 1 and 1 <= y < FH - 1:
                img[y, x] = (*hexc(ICE[0]), 255)
                if p["big"]:
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        img[y + dy, x + dx] = (*hexc(ICE[1]), 220)
        flakes.step()
        # холодный пар у основания, пока растёт глыба
        if 8 <= f <= 15:
            k = (f - 8) / 7
            for sx in (-14, 12):
                cx, cy = CX + sx, CY + 6 - 4 * k
                r = 2.5 + 2.0 * math.sin(k * math.pi)
                puff = np.hypot(XX + 0.5 - cx, (YY + 0.5 - cy) * 1.4) <= r
                img[puff & ((XX + YY + f) % 2 == 0)] = (*hexc(ICE[0]), 150)
        # искорки на льду после заморозки
        if f >= 6:
            for _ in range(3):
                x = int(CX + rng.uniform(-20, 20))
                y = int(CY + rng.uniform(-12, 12))
                if DMD[y, x] < 0.85:
                    img[y, x] = (*hexc("#ffffff"), 255)
        frames.append(img)
    return frames


# ---------------------------------------------------------------- превью
def load_sprites():
    sprites = {}
    for color in "wb":
        for k in "PNBRQK":
            body = np.array(Image.open(os.path.join(PIECES_DIR, f"{color}{k}.png")).convert("RGBA"))
            shadow = Image.open(os.path.join(PIECES_DIR, f"{color}{k}_shadow.png")).convert("RGBA")
            sprites[color + k] = (body, shadow, frozen_sprite(body))
    return sprites


def board(position, layers, sprites, frame_of):
    """Вся доска: клетки, лёд, тени, фигуры (с инеем), глыбы, снег. frame_of(sq) — кадр
    эффекта на клетке или None."""
    margin_x, top = 40, 70
    w, h = 8 * TW + 2 * margin_x, 8 * TH + art.SIDE + top + 50
    ox = margin_x + 4 * TW - TW // 2
    canvas = Image.new("RGBA", (w, h), hexc("#151821") + (255,))

    def tile_xy(f, r):
        i, j = f, 7 - r
        return ox + (i - j) * (TW // 2), top + (i + j) * (TH // 2)

    cells = sorted(((f, r) for f in range(8) for r in range(8)), key=lambda c: c[0] + 7 - c[1])
    for f, r in cells:
        kind = "dark" if (f + r) % 2 == 0 else "light"
        x, y = tile_xy(f, r)
        canvas.alpha_composite(BLOCKS[(kind, (f * 7 + r * 3) % 4)], (x, y))
    for sq in cells:
        fr = frame_of(sq)
        if fr is not None:
            x, y = tile_xy(*sq)
            tile = layers["tile"][variant_of(*sq)][fr]
            canvas.alpha_composite(Image.fromarray(tile, "RGBA"), (x + TW // 2 - CX, y + TH // 2 - CY))
    order = sorted(position.items(), key=lambda kv: kv[0][0] + 7 - kv[0][1])
    for sq, code in order:
        x, y = tile_xy(*sq)
        canvas.alpha_composite(sprites[code][1], (x + TW // 2 - art.AX, y + TH // 2 - art.AY))
    for sq, code in order:
        x, y = tile_xy(*sq)
        body, _, frozen = sprites[code]
        fr = frame_of(sq)
        spr = body if fr is None else frost_reveal(body, frozen, fr, code[1])
        canvas.alpha_composite(Image.fromarray(spr, "RGBA"), (x + TW // 2 - art.AX, y + TH // 2 - art.AY))
        if fr is not None:
            block = layers["block"][code[1]][variant_of(*sq)][fr]
            canvas.alpha_composite(Image.fromarray(block, "RGBA"), (x + TW // 2 - CX, y + TH // 2 - CY))
    for sq in cells:
        fr = frame_of(sq)
        if fr is not None:
            x, y = tile_xy(*sq)
            canvas.alpha_composite(Image.fromarray(layers["over"][fr], "RGBA"), (x + TW // 2 - CX, y + TH // 2 - CY))
    return canvas


BLOCKS = {(kind, v): art.block(kind, 11 * v + (0 if kind == "dark" else 5)) for kind in ("dark", "light") for v in range(4)}


def save_gif(frames, path, scale=2, crop=None):
    big = []
    for im in frames:
        if crop:
            im = im.crop(crop)
        big.append(im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (big[0].width, big[0].height * len(big)))
    for k, p in enumerate(big):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in big]
    gif[0].save(path, save_all=True, append_images=gif[1:], duration=FRAME_MS, loop=0, disposal=1)


def start_position():
    pos = {}
    back = "RNBQKBNR"
    for f in range(8):
        pos[(f, 0)], pos[(f, 1)] = "w" + back[f], "wP"
        pos[(f, 6)], pos[(f, 7)] = "bP", "b" + back[f]
    return pos


def save_all(out):
    os.makedirs(out, exist_ok=True)
    frozen_dir = os.path.join(PIECES_DIR, "frozen")
    os.makedirs(frozen_dir, exist_ok=True)
    sprites = load_sprites()
    for code, (_, _, frozen) in sprites.items():
        Image.fromarray(frozen, "RGBA").save(os.path.join(frozen_dir, f"{code}.png"))
    layers = {
        "tile": [tile_frames(v) for v in range(VARIANTS)],
        "over": over_frames(),
        "block": {k: [block_frames(k, v) for v in range(VARIANTS)] for k in "PNBRQK"},
    }
    for v in range(VARIANTS):
        Image.fromarray(np.concatenate(layers["tile"][v], axis=1), "RGBA").save(os.path.join(out, f"freeze_tile_{v}.png"))
        for k in "PNBRQK":
            strip = np.concatenate(layers["block"][k][v], axis=1)
            Image.fromarray(strip, "RGBA").save(os.path.join(out, f"freeze_block_{k}_{v}.png"))
    Image.fromarray(np.concatenate(layers["over"], axis=1), "RGBA").save(os.path.join(out, "freeze_over.png"))
    for stale in ["freeze_tile.png"] + [f"freeze_block_{k}.png" for k in "PNBRQK"]:  # прежние имена
        if os.path.exists(os.path.join(out, stale)):
            os.remove(os.path.join(out, stale))

    pos = start_position()
    pos[(4, 3)] = pos.pop((4, 1))  # e4
    pos[(2, 4)] = pos.pop((2, 6))  # c5
    # одна фигура: белая пешка e4
    one = []
    for k in range(-4, FRAMES + 8):
        one.append(board(pos, layers, sprites,
                         lambda sq, k=k: None if sq != (4, 3) or k < 0 else min(k, FRAMES - 1)))
    save_gif(one, os.path.join(out, "preview_freeze_one.gif"), crop=(120, 120, 410, 400))
    # половина поля: чёрная половина замерзает волной от e5
    origin = (4, 4)
    half = []
    for k in range(-4, FRAMES + 8 * DELAY_PER_TILE + 8):
        def frame_of(sq, k=k):
            if sq[1] < 4:
                return None
            d = max(abs(sq[0] - origin[0]), abs(sq[1] - origin[1]))
            t = k - d * DELAY_PER_TILE
            return None if t < 0 else min(t, FRAMES - 1)
        half.append(board(pos, layers, sprites, frame_of))
    save_gif(half, os.path.join(out, "preview_freeze_half.gif"))
    # контрольный лист: одна клетка по кадрам и итог половины поля
    picks = [0, 3, 6, 9, 11, 13, 17]
    cells = [one[p + 4].crop((180, 160, 330, 330)) for p in picks]
    last = half[-1].convert("RGB")
    sheet = Image.new("RGB", (max(len(cells) * 152 + 2, last.width + 4), 174 + last.height), (27, 30, 39))
    for n, c in enumerate(cells):
        sheet.paste(c.convert("RGB"), (2 + n * 152, 2))
    sheet.paste(last, (2, 172))
    sheet.save(os.path.join(out, "preview_freeze_frames.png"))


if __name__ == "__main__":
    save_all(OUT)
    print("done", OUT)
