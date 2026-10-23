"""Электрические дуги вокруг слона «в духе Супер Саяна 2» — в стиле Into the Breach.

Слон заряжается: по телу пробегают искорки, от основания поднимается золотая
аура-пламя, вспышка — и от слона во все стороны расходятся молнии. Потом держится
состояние: аура колышется, вокруг тела
трещат голубые дуги, изредка молния бьёт наружу или в землю. В конце аура опадает,
последние искры гаснут.

Кадры — замена спрайта слона на большем холсте FW×FH, точка опоры (центр клетки) —
(FAX, FAY), как у `bless_figure` из `breach_bless.py`; тень — обычная тень слона.
Части: вступление — кадры [0, LOOP_START), повтор — [LOOP_START, LOOP_END) (можно
крутить сколько нужно), концовка — [LOOP_END, FRAMES).

`py tools/breach_arcs.py [папка]` — по умолчанию assets/breach/effects:
electric_<цвет><фигура>.png (кадры в ряд, все 12 фигур), electric_hold_<цвет><фигура>.png (не гаснет: вступление и
повтор, без концовки — после кадра LOOP_END − 1 идёт снова LOOP_START), preview_electric.gif
(вступление, повтор дважды, концовка, слоны на доске), preview_electric_hold.gif
(вступление и повтор 10 раз), preview_electric_frames.png, preview_electric_all.gif
(остальные фигуры).
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
import breach_firewave as fw  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

CYAN = [hexc(c) for c in ["#ffffff", "#e9fdff", "#a6f2ff", "#57d9ff", "#2aa9d6", "#16627f"]]
GOLD = [hexc(c) for c in ["#fff1b4", "#ffd970", "#ffc35a", "#ff8a2e"]]
FW, FH = 120, 120
FAX, FAY = 60, 92  # центр клетки слона
SX, SY = FAX - art.AX, FAY - art.AY  # где на холсте лежит спрайт 64×84
FRAME_MS = 60
FLASH = 5  # кадр вспышки: молнии расходятся во все стороны
LOOP_START, LOOP_END = 8, 24
FRAMES = 30


def dilate(m):
    out = m.copy()
    out[1:] |= m[:-1]
    out[:-1] |= m[1:]
    out[:, 1:] |= m[:, :-1]
    out[:, :-1] |= m[:, 1:]
    return out


def distance(sil, steps=14):
    """Расстояние до силуэта в шагах (ромбом); дальше steps — бесконечность."""
    d = np.where(sil, 0.0, np.inf)
    cur = sil
    for i in range(1, steps + 1):
        nxt = dilate(cur)
        if i % 2 == 0:  # через шаг — и по диагонали: круглее
            nxt |= np.roll(np.roll(cur, 1, 0), 1, 1) | np.roll(np.roll(cur, -1, 0), -1, 1)
            nxt |= np.roll(np.roll(cur, 1, 0), -1, 1) | np.roll(np.roll(cur, -1, 0), 1, 1)
        d[nxt & ~cur] = i
        cur = nxt
    return d


def loop_noise(cell, period, rng, cell_x=None):
    """Шум, бегущий вверх и повторяющийся через `period` пикселей сдвига; с cell_x уже
    cell — вытянут вверх, как языки пламени."""
    cx = cell_x or cell
    gh, gw = period // cell, FW // cx + 2
    g = rng.random((gh, gw))
    yy, xx = np.mgrid[0:FH, 0:FW]

    def sample(shift):
        y, x = (yy + shift) / cell, xx / cx
        y0, x0 = np.floor(y).astype(int), x.astype(int)
        fy, fx = y - y0, x - x0
        fy, fx = fy * fy * (3 - 2 * fy), fx * fx * (3 - 2 * fx)
        a = g[y0 % gh, x0] * (1 - fx) + g[y0 % gh, x0 + 1] * fx
        b = g[(y0 + 1) % gh, x0] * (1 - fx) + g[(y0 + 1) % gh, x0 + 1] * fx
        return a * (1 - fy) + b * fy

    return sample


def bolt_points(p0, p1, rng, rough=0.32, bow=0.0):
    """Ломаная молнии: деление пополам со смещением; bow — изгиб дугой наружу."""
    (x0, y0), (x1, y1) = p0, p1
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2
    dx, dy = x1 - x0, y1 - y0
    ln = math.hypot(dx, dy) + 1e-6
    nx, ny = -dy / ln, dx / ln
    mid = (mx + nx * bow * ln, my + ny * bow * ln)
    pts = [p0, mid, p1]
    for _ in range(4):
        new = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            sl = math.hypot(b[0] - a[0], b[1] - a[1])
            if sl > 2.5:
                off = rng.normal(0, rough * sl)
                ux, uy = -(b[1] - a[1]) / sl, (b[0] - a[0]) / sl
                new.append(((a[0] + b[0]) / 2 + ux * off, (a[1] + b[1]) / 2 + uy * off))
            new.append(b)
        pts = new
    return pts


def raster(pts):
    """Пиксели ломаной, связные по сторонам: в пиксель-арте линия без дыр."""
    m = np.zeros((FH, FW), bool)
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        n = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        px, py = None, None
        for t in np.linspace(0, 1, n + 1):
            x, y = int(x0 + (x1 - x0) * t), int(y0 + (y1 - y0) * t)
            if px is not None and x != px and y != py and 0 <= px < FW and 0 <= y < FH:
                m[y, px] = True  # угол: ступенька
            if 0 <= x < FW and 0 <= y < FH:
                m[y, x] = True
            px, py = x, y
    return m


class Bolt:
    def __init__(self, pts, front, life=2, width=1):
        self.mask = raster(pts)
        if width > 1:
            self.mask |= np.roll(self.mask, 1, 1)
        self.front, self.life, self.age = front, life, 0
        self.ends = [pts[0], pts[-1]]


def paint_bolts(layer, bolts):
    """Молния: белое ядро, голубая кайма, тёмно-синий край; старая — тусклее."""
    for b in sorted(bolts, key=lambda b: -b.age):
        core = b.mask
        edge = burn.neighbours_any(core) & ~core
        rim = burn.neighbours_any(core | edge) & ~core & ~edge
        if b.age == 0:
            layer[rim & (layer[..., 3] == 0)] = (*CYAN[5], 150)
            layer[edge] = (*CYAN[3], 255)
            layer[core] = (*CYAN[0], 255)
        else:
            layer[edge & (layer[..., 3] == 0)] = (*CYAN[5], 170)
            layer[core] = (*CYAN[3], 255)


def edge_point(sil, cx, cy, ang):
    """Последний пиксель силуэта по лучу из центра."""
    best = (cx, cy)
    for r in np.arange(0, 50, 0.5):
        x, y = cx + math.cos(ang) * r, cy + math.sin(ang) * r
        if 0 <= int(x) < FW and 0 <= int(y) < FH and sil[int(y), int(x)]:
            best = (x, y)
    return best


def ground_point(r, ang):
    """Точка на верхней грани клетки на «радиусе» r (ромб 2:1 — эллипс)."""
    return FAX + math.cos(ang) * r, FAY + math.sin(ang) * r * 0.5


def render(code, seed=7, bodies=None, under=None, length=None):
    """Кадры электричества. bodies(f) — своё тело фигуры в кадре f (холст FW×FH, аура — по обычному
    силуэту; что тело выносит за силуэт, ложится поверх ауры и задних молний); under(f) — свой слой
    вместо золотой ауры (тогда и контур фигуры ею не подсвечивается); length — столько кадров без
    концовки: после LOOP_END повтор продолжается."""
    rng = np.random.default_rng(seed)
    sprite = np.array(Image.open(os.path.join(fw.art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA"))
    canvas = np.zeros((FH, FW, 4), np.uint8)
    canvas[SY : SY + sprite.shape[0], SX : SX + sprite.shape[1]] = sprite
    sil = canvas[..., 3] > 0
    ys, xs = np.nonzero(sil)
    cx, cy = xs.mean(), (ys.min() + ys.max()) / 2
    top, base = ys.min(), ys.max()
    d = distance(sil)
    loop_len = LOOP_END - LOOP_START
    noise = loop_noise(4, 3 * loop_len, rng)  # 3 пикселя вверх за кадр, повтор через loop_len
    tongues = loop_noise(8, 3 * loop_len, rng, cell_x=3)
    # высота над силуэтом (расширенным на 2 пикселя) по столбцу — для языков ауры вверх
    wide = dilate(dilate(sil))
    gap = np.full((FH, FW), np.inf)
    run = np.full(FW, np.inf)
    for y in range(FH - 1, -1, -1):
        run = np.where(wide[y], 0, run + 1)
        gap[y] = run
    ring_edge = burn.boundary(sil)
    rim = burn.boundary(sil & ~ring_edge)  # пиксели под контуром — подсветка аурой
    yy, xx = np.mgrid[0:FH, 0:FW]
    bolts = []
    sparks = burn.Particles()
    frames = []
    for f in range(FRAMES if length is None else length):
        looping = f < LOOP_END or length is not None
        # сила ауры: растёт во вступлении, держится, опадает в концовке
        if f < LOOP_START:
            grow = np.clip((f - 1) / (FLASH - 1), 0, 1)
        elif looping:
            grow = 1.0
        else:
            grow = max(0.0, 1 - (f - LOOP_END + 1) / (FRAMES - LOOP_END))
        lf = (f - LOOP_START) % loop_len  # фаза повтора — чтобы шов совпал
        shift = 3 * lf if f >= LOOP_START else 3 * f
        n, nt = noise(shift), tongues(shift)
        # ---- новые молнии этого кадра
        for b in bolts:
            b.age += 1
        bolts = [b for b in bolts if b.age < b.life]
        if f < FLASH:  # искорки по телу
            for _ in range(1 + f // 2):
                a = rng.uniform(0, 2 * math.pi)
                p = edge_point(sil, cx, cy + rng.uniform(-10, 14), a)
                q = (p[0] + rng.uniform(-6, 6), p[1] + rng.uniform(-6, 4))
                bolts.append(Bolt(bolt_points(p, q, rng, 0.4), front=True, life=1))
        elif f == FLASH:  # вспышка: молнии расходятся во все стороны, с развилками
            for k in range(8):
                a = 2 * math.pi * k / 8 + rng.uniform(-0.2, 0.2)
                p = edge_point(sil, cx, cy, a)
                r = rng.uniform(20, 30)
                # вниз — по земле, сплюснуто, как клетка в изометрии
                q = (p[0] + math.cos(a) * r, p[1] + math.sin(a) * r * (0.85 if math.sin(a) < 0 else 0.4))
                pts = bolt_points(p, q, rng, 0.3)
                bolts.append(Bolt(pts, front=math.sin(a) > -0.2, life=3, width=2 if k % 2 == 0 else 1))
                m = pts[len(pts) // 2]
                fq = (m[0] + math.cos(a + 0.7) * r * 0.45, m[1] + math.sin(a + 0.7) * r * 0.4)
                bolts.append(Bolt(bolt_points(m, fq, rng, 0.35), front=math.sin(a) > -0.2, life=3))
        elif grow > 0.3 or f < LOOP_START:
            # дуги вдоль тела — главный рисунок «саяна»: короткие, изогнутые наружу
            for _ in range(3 if looping else 1):
                a = rng.uniform(0, 2 * math.pi)
                span = rng.uniform(0.6, 1.2)
                oy = rng.uniform(-14, 14)
                p = edge_point(sil, cx, cy + oy, a)
                q = edge_point(sil, cx, cy + oy + rng.uniform(-8, 8), a + span)
                front = rng.random() < 0.55
                bolts.append(Bolt(bolt_points(p, q, rng, 0.3, bow=-rng.uniform(0.3, 0.6)), front, life=2))
            # иногда — молния наружу или в землю
            if rng.random() < 0.35 and looping:
                if rng.random() < 0.5:
                    a = rng.uniform(math.pi * 0.9, math.pi * 2.1)  # вверх и в стороны
                    p = edge_point(sil, cx, cy, a)
                    r = rng.uniform(12, 20)
                    q = (p[0] + math.cos(a) * r, p[1] + math.sin(a) * r)
                else:
                    a = rng.uniform(0, 2 * math.pi)
                    p = edge_point(sil, cx, base - 6, a)
                    q = ground_point(rng.uniform(18, 26), a)
                    for _ in range(3):
                        sparks.items.append(dict(x=q[0], y=q[1], vx=rng.uniform(-1.2, 1.2), vy=rng.uniform(-1.6, -0.4),
                                                 drag=0.85, life=int(rng.integers(2, 5)), age=0))
                bolts.append(Bolt(bolt_points(p, q, rng, 0.3), front=q[1] > cy + 4, life=2))
        # ---- аура: золотое пламя вокруг силуэта, вверх — длинными языками, бегущими вверх
        side = (3.5 + 4.5 * n) * grow
        side = np.where(yy > base - 3, side * 0.35, side)
        up = (5 + 22 * nt ** 2) * grow
        aura = ((d <= side) | (gap <= up)) & ~sil & (grow > 0)
        aura &= yy < base + 2
        img = np.zeros((FH, FW, 4), np.uint8) if under is None else under(f).copy()
        if under is None and aura.any():
            outer = burn.boundary(aura) & ~burn.neighbours_any(sil)
            img[aura] = (*GOLD[2], 150)
            img[aura & ((d <= 2) | (gap <= up * 0.35))] = (*GOLD[1], 185)
            img[aura & ((d <= 1) | (gap <= up * 0.15))] = (*GOLD[0], 215)
            img[outer] = (*GOLD[3], 235)
            # искорки ауры: короткие черточки, бегут вверх
            pts = np.argwhere(aura & ~outer)
            for y, x in pts[rng.choice(len(pts), size=min(5, len(pts)), replace=False)]:
                img[max(0, y - 1) : y + 1, x] = (*GOLD[0], 255)
        # ---- задние молнии (кольца по земле на вспышке нет с 2026-10-18 — просьба пользователя)
        back = np.zeros((FH, FW, 4), np.uint8)
        front_l = np.zeros((FH, FW, 4), np.uint8)
        paint_bolts(back, [b for b in bolts if not b.front])
        m = back[..., 3] > 0
        img[m] = back[m]
        # ---- слон: подсветка контура аурой, на вспышке — бело-голубой
        body = (canvas if bodies is None else bodies(f)).copy()
        rgb = body[..., :3].astype(float)
        if grow > 0 and under is None:
            k = 0.35 * grow
            rgb[rim] = rgb[rim] * (1 - k) + np.array(GOLD[0], float) * k
        if f in (FLASH, FLASH + 1):
            k = 0.55 if f == FLASH else 0.25
            inner = sil & ~ring_edge
            rgb[inner] = rgb[inner] * (1 - k) + np.array(CYAN[1], float) * k
        body[..., :3] = rgb.astype(np.uint8)
        img[sil] = body[sil]
        loose = (body[..., 3] > 0) & ~sil  # то, что своё тело выносит за силуэт (искры лавы, вихрь)
        if loose.any():
            sa = body[..., 3:4].astype(float) / 255
            da = img[..., 3:4].astype(float) / 255
            oa = sa + da * (1 - sa)
            rgb = (body[..., :3] * sa + img[..., :3] * da * (1 - sa)) / np.maximum(oa, 1e-6)
            img[loose, :3] = np.round(rgb[loose]).astype(np.uint8)
            img[loose, 3] = np.round(oa[loose, 0] * 255).astype(np.uint8)
        # ---- передние молнии, искры
        paint_bolts(front_l, [b for b in bolts if b.front])
        m = front_l[..., 3] > 0
        img[m] = front_l[m]
        for b in bolts:  # вспышка на концах молний
            if b.age == 0:
                for x, y in b.ends:
                    x, y = int(x), int(y)
                    if 1 <= x < FW - 1 and 1 <= y < FH - 1 and not sil[y, x]:
                        img[y, x - 1 : x + 2] = (*CYAN[1], 255)
                        img[y - 1 : y + 2, x] = (*CYAN[1], 255)
                        img[y, x] = (*CYAN[0], 255)
        for p in sparks.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                img[y, x] = (*CYAN[[0, 2, 4][min(p["age"], 2)]], 255)
        sparks.step()
        frames.append(img)
    return frames


# ---------------------------------------------------------------- превью на доске
def scene_frames(frames, code, center, position, hold=False):
    tiles, shadows, _, at = fw.board_scene(position, center)
    rest = {sq: c for sq, c in position.items() if sq != center}
    back = {sq: c for sq, c in rest.items() if sq[0] + 7 - sq[1] < center[0] + 7 - center[1]}
    front = {sq: c for sq, c in rest.items() if sq not in back}
    back_layer = fw.board_scene(back, center)[2]
    front_layer = fw.board_scene(front, center)[2]
    cell = (at[0] + fw.CX, at[1] + fw.CY)  # центр верхней грани клетки
    fat = (cell[0] - FAX, cell[1] - FAY)
    still = Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA")
    if hold:  # не гаснет: вступление и повтор без конца (в превью — 10 раз)
        order = [None] * 4 + list(range(LOOP_START)) + list(range(LOOP_START, LOOP_END)) * 10
    else:
        order = ([None] * 4 + list(range(LOOP_START)) + list(range(LOOP_START, LOOP_END)) * 2
                 + list(range(LOOP_END, FRAMES)) + [None] * 6)
    out = []
    for f in order:
        im = tiles.copy()
        im.alpha_composite(shadows)
        im.alpha_composite(back_layer)
        if f is None:
            im.alpha_composite(still, (cell[0] - art.AX, cell[1] - art.AY))
        else:
            im.alpha_composite(Image.fromarray(frames[f], "RGBA"), fat)
        im.alpha_composite(front_layer)
        out.append(im.convert("RGB"))
    return out


def save_gif(all_frames, path, hold, scale=2):
    scenes = [scene_frames(all_frames["wB"], "wB", (4, 3), {(4, 3): "wB", (3, 4): "bP", (5, 4): "bN", (2, 3): "wP"}, hold),
              scene_frames(all_frames["bB"], "bB", (4, 3), {(4, 3): "bB", (3, 2): "wP", (5, 3): "wR", (6, 4): "bP"}, hold)]
    w, h = scenes[0][0].size
    frames = []
    for a, b in zip(*scenes):
        im = Image.new("RGB", (2 * w + 4, h), (27, 30, 39))
        im.paste(a, (0, 0))
        im.paste(b, (w + 4, 0))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 8))
    for n in range(8):
        allpix.paste(frames[4 + n * 5], (0, n * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(path, save_all=True, append_images=gif[1:], duration=FRAME_MS, loop=0, disposal=1)


def save_preview(all_frames, out):
    save_gif(all_frames, os.path.join(out, "preview_electric.gif"), hold=False)
    save_gif(all_frames, os.path.join(out, "preview_electric_hold.gif"), hold=True)
    # лист всех кадров обоих слонов крупно
    cols = 10
    sheet = Image.new("RGBA", (cols * (FW + 2), 2 * 3 * (FH + 2)), (40, 44, 56, 255))
    for row, code in enumerate(("wB", "bB")):
        for k, fr in enumerate(all_frames[code]):
            sheet.alpha_composite(Image.fromarray(fr, "RGBA"),
                                  ((k % cols) * (FW + 2), (row * 3 + k // cols) * (FH + 2)))
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(os.path.join(out, "preview_electric_frames.png"))


# ---------------------------------------------------------------- спокойный вариант
# Повтор нынешней анимации мерцает каждый кадр и всё время двигает большую ауру — отвлекает.
# Спокойный (просьба пользователя): вступление и вспышка те же, потом аура опадает (кадры
# концовки) и идёт долгий тихий повтор: аура — кайма в 1–2 пикселя по контуру, медленно
# «дышит»; изредка короткая искорка; раз за повтор — разряд из нескольких дуг (как сейчас:
# белое ядро, одна молния в землю), кайма на миг ярче.
CALM_SETTLE = list(range(LOOP_END, FRAMES))  # кадры концовки — аура опадает до каймы
CALM_LOOP_START = LOOP_START + len(CALM_SETTLE)  # 14
CALM_LOOP = 48  # 2,9 с
CALM_FRAMES = CALM_LOOP_START + CALM_LOOP  # 62
CALM_SPARKS = (8, 19)  # кадры повтора с искоркой
CALM_DISCHARGE = 30  # кадр повтора с разрядом


def render_calm(code, seed, base):
    """Кадры спокойного варианта: base — кадры обычной анимации этой фигуры."""
    rng = np.random.default_rng(seed + 100)
    sprite = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA"))
    canvas = np.zeros((FH, FW, 4), np.uint8)
    canvas[SY : SY + sprite.shape[0], SX : SX + sprite.shape[1]] = sprite
    sil = canvas[..., 3] > 0
    ys, xs = np.nonzero(sil)
    cx, cy = xs.mean(), (ys.min() + ys.max()) / 2
    base_y = ys.max()
    d = distance(sil, 3)
    rim = burn.boundary(sil & ~burn.boundary(sil))

    def halo(img, f):
        """Кайма ауры вокруг фигуры: f — кадр повтора (дыхание и разряд)."""
        breath = 0.5 + 0.5 * math.sin(2 * math.pi * f / CALM_LOOP)
        burst = max(0.0, 1 - abs(f - CALM_DISCHARGE - 0.5) / 2.5)  # ярче на разряде
        a1 = int(110 + 70 * breath + 70 * burst)
        a2 = int(40 + 35 * breath + 90 * burst)
        ring2 = (d == 2) & (img[..., 3] == 0)
        ring1 = (d == 1) & (img[..., 3] == 0)
        img[ring2] = (*GOLD[2], min(a2, 255))
        img[ring1] = (*GOLD[1], min(a1, 255))
        if burst > 0.6:
            img[(d == 3) & (img[..., 3] == 0)] = (*GOLD[2], 60)
        return breath, burst

    def body(img, breath, burst):
        b = canvas.copy()
        rgb = b[..., :3].astype(float)
        k = 0.18 + 0.12 * breath
        rgb[rim] = rgb[rim] * (1 - k) + np.array(GOLD[0], float) * k
        if burst > 0.6:
            inner = sil & ~burn.boundary(sil)
            rgb[inner] = rgb[inner] * 0.8 + np.array(CYAN[1], float) * 0.2
        b[..., :3] = rgb.astype(np.uint8)
        img[sil] = b[sil]

    def hug(length):
        a = rng.uniform(0, 2 * math.pi)
        oy = rng.uniform(-14, 14)
        p = edge_point(sil, cx, cy + oy, a)
        q = edge_point(sil, cx, cy + oy + rng.uniform(-6, 6), a + length)
        return Bolt(bolt_points(p, q, rng, 0.3, bow=-rng.uniform(0.3, 0.6)), rng.random() < 0.55, life=2)

    frames = [f.copy() for f in base[:LOOP_START]]
    # опадание: кадры концовки, под ними уже проступает кайма
    for f in CALM_SETTLE:
        img = base[f].copy()
        under = np.zeros((FH, FW, 4), np.uint8)
        halo(under, 0)
        m = img[..., 3] == 0
        img[m] = under[m]
        frames.append(img)
    bolts, sparks = [], burn.Particles()
    for f in range(CALM_LOOP):
        for b in bolts:
            b.age += 1
        bolts = [b for b in bolts if b.age < b.life]
        if f in CALM_SPARKS:
            bolts.append(hug(rng.uniform(0.35, 0.6)))
        if f == CALM_DISCHARGE:
            for _ in range(3):
                bolts.append(hug(rng.uniform(0.7, 1.2)))
            a = rng.uniform(0, 2 * math.pi)
            p = edge_point(sil, cx, base_y - 6, a)
            q = ground_point(rng.uniform(18, 24), a)
            for _ in range(3):
                sparks.items.append(dict(x=q[0], y=q[1], vx=rng.uniform(-1.2, 1.2), vy=rng.uniform(-1.6, -0.4),
                                         drag=0.85, life=int(rng.integers(2, 5)), age=0))
            bolts.append(Bolt(bolt_points(p, q, rng, 0.3), front=q[1] > cy + 4, life=2))
        if f == CALM_DISCHARGE + 1:
            bolts.append(hug(rng.uniform(0.5, 0.9)))
        img = np.zeros((FH, FW, 4), np.uint8)
        back = np.zeros((FH, FW, 4), np.uint8)
        paint_bolts(back, [b for b in bolts if not b.front])
        m = back[..., 3] > 0
        img[m] = back[m]
        breath, burst = halo(img, f)
        body(img, breath, burst)
        front = np.zeros((FH, FW, 4), np.uint8)
        paint_bolts(front, [b for b in bolts if b.front])
        m = front[..., 3] > 0
        img[m] = front[m]
        for p in sparks.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                img[y, x] = (*CYAN[[0, 2, 4][min(p["age"], 2)]], 255)
        sparks.step()
        frames.append(img)
    return frames


def save_calm_preview(all_frames, calm_frames, out, scale=2):
    """Сравнение: слева нынешний повтор, справа спокойный; белый и чёрный слоны."""
    from PIL import ImageDraw, ImageFont
    crop = (55, 0, 245, 200)
    n = 4 + CALM_LOOP_START + CALM_LOOP * 3

    def calm_at(f):
        return f if f < CALM_LOOP_START else CALM_LOOP_START + (f - CALM_LOOP_START) % CALM_LOOP

    def now_at(f):
        return f if f < LOOP_START else LOOP_START + (f - LOOP_START) % (LOOP_END - LOOP_START)

    cols = []
    for frames_of, at_frame in ((all_frames, now_at), (calm_frames, calm_at)):
        col = []
        for code, pos in (("wB", {(4, 3): "wB", (5, 4): "bP", (3, 4): "bN"}),
                          ("bB", {(4, 3): "bB", (5, 4): "wP", (3, 4): "wN"})):
            tiles, shadows, _, at = fw.board_scene(pos, (4, 3))
            rest = {sq: c for sq, c in pos.items() if sq != (4, 3)}
            back = fw.board_scene({sq: c for sq, c in rest.items() if sq[0] + 7 - sq[1] < 8}, (4, 3))[2]
            front = fw.board_scene({sq: c for sq, c in rest.items() if sq[0] + 7 - sq[1] >= 8}, (4, 3))[2]
            cell = (at[0] + fw.CX, at[1] + fw.CY)
            still = Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA")
            seq = []
            for k in range(n):
                im = tiles.copy()
                im.alpha_composite(shadows)
                im.alpha_composite(back)
                if k < 4:
                    im.alpha_composite(still, (cell[0] - art.AX, cell[1] - art.AY))
                else:
                    fr = frames_of[code][at_frame(k - 4)]
                    im.alpha_composite(Image.fromarray(fr, "RGBA"), (cell[0] - FAX, cell[1] - FAY))
                im.alpha_composite(front)
                seq.append(im.convert("RGB").crop(crop))
            col.append(seq)
        cols.append(col)
    cw, ch = crop[2] - crop[0], crop[3] - crop[1]
    head = 14
    try:
        font = ImageFont.truetype("segoeuib.ttf", 11)
    except OSError:
        font = ImageFont.load_default()
    frames = []
    for k in range(n):
        im = Image.new("RGB", (2 * cw + 4, head + 2 * ch + 4), (27, 30, 39))
        dr = ImageDraw.Draw(im)
        for c, title in enumerate(("сейчас", "спокойно")):
            dr.text((c * (cw + 4) + cw // 2, 2), title, font=font, fill=(230, 230, 230), anchor="mt")
            for r in range(2):
                im.paste(cols[c][r][k], (c * (cw + 4), head + r * (ch + 4)))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 6))
    for i, k in enumerate((8, 12, 20, 40, 48, 50)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [q.quantize(palette=pal, dither=Image.Dither.NONE) for q in frames]
    gif[0].save(os.path.join(out, "preview_electric_calm.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)


# зерно по виду фигуры (у белых и чёрных одинаковые молнии); у слона — прежнее 7, его кадры уже в игре
SEEDS = {"B": 7, "P": 11, "N": 13, "R": 17, "Q": 19, "K": 23}


def save_all_preview(all_frames, out, hold, scale=2):
    """Остальные фигуры, каждая на своём куске доски: верхний ряд — белые, нижний — чёрные."""
    crop = (55, 0, 245, 200)
    rows = []
    for color in "wb":
        enemy = "b" if color == "w" else "w"
        row = []
        for k in "PNRQK":
            code = color + k
            pos = {(4, 3): code, (5, 4): enemy + "P"}
            row.append([im.crop(crop) for im in scene_frames(all_frames[code], code, (4, 3), pos, hold)])
        rows.append(row)
    cw, ch = crop[2] - crop[0], crop[3] - crop[1]
    frames = []
    for n in range(len(rows[0][0])):
        im = Image.new("RGB", (5 * cw + 4 * 4, 2 * ch + 4), (27, 30, 39))
        for r, row in enumerate(rows):
            for c, scene in enumerate(row):
                im.paste(scene[n], (c * (cw + 4), r * (ch + 4)))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 6))
    for i, k in enumerate((6, 9, 12, 16, 20, 26)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    name = "preview_electric_all_hold.gif" if hold else "preview_electric_all.gif"
    gif[0].save(os.path.join(out, name), save_all=True, append_images=gif[1:], duration=FRAME_MS, loop=0, disposal=1)


def main():
    os.makedirs(OUT, exist_ok=True)
    all_frames = {}
    for code in (c + k for c in "wb" for k in "PNBRQK"):
        frames = render(code, SEEDS[code[1]])
        all_frames[code] = frames
        Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(OUT, f"electric_{code}.png"))
        # версия, которая не гаснет: вступление и повтор, концовки нет
        Image.fromarray(np.concatenate(frames[:LOOP_END], axis=1), "RGBA").save(
            os.path.join(OUT, f"electric_hold_{code}.png"))
    calm_frames = {}
    for code, frames in all_frames.items():
        calm_frames[code] = render_calm(code, SEEDS[code[1]], frames)
        Image.fromarray(np.concatenate(calm_frames[code], axis=1), "RGBA").save(
            os.path.join(OUT, f"electric_calm_{code}.png"))
    save_preview(all_frames, OUT)
    save_all_preview(all_frames, OUT, hold=False)
    save_calm_preview(all_frames, calm_frames, OUT)
    print("done", OUT)


if __name__ == "__main__":
    main()
