"""Картинки окна для карт без картинок — HD, 76×50 (окно лица карты 2x в точке (12, 34)).

Те же правила, что у картинок `breach_cards_hd.py`: ночное небо полосами с шахматным
переходом, тёмная доска ромбами, фигуры — пиксель-арт игры: свои спрайты (`pieces/*.png`)
или тот же рендер `breach_art.render_piece` мельче, если фигура не влезает в окно. Лёд —
палитра `breach_freeze.py` (иней, полупрозрачные глыбы, снег), огонь — `breach_firewave.py`,
свет — `breach_bless.py`. Контур #020201, свет сверху-слева.

Карты (имя файла — по `CardKind`): обычные — буря, лавина, ледяной барьер, они множатся,
ледяная стена, массовое жертвоприношение, абсолютный ноль, ледяной снаряд, ледяной взрыв;
жертвы — жертва, окружены и сломлены, кровь богу крови, хладнокровный правитель, холодная
зима; победы — один в поле воин, оборонительная позиция, барьер, истории побед, воин из
льда, северный воин.

`py tools/breach_card_arts.py [папка]` — по умолчанию assets/breach/cards/hd: картинки в
art/, сжигание в burn/ (`burn_strip_<имя>_<лицо>`), превью preview_card_arts.png.
`--no-burn` — без сжигания (быстро).
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_bless as bl  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_cards as bc  # noqa: E402
import breach_cards_hd as hd  # noqa: E402
import breach_firewave as fw  # noqa: E402
import breach_freeze as fz  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
ARGS = [a for a in sys.argv[1:] if not a.startswith("--")]
OUT = ARGS[0] if ARGS else os.path.join(art.ROOT, "assets", "breach", "cards", "hd")
CACHE = os.environ.get("BC_PIECE_CACHE")  # папка для готовых рендеров фигур (ускоряет правки)

W, H = hd.ART_SIZE  # 76×50
YY, XX = np.mgrid[0:H, 0:W]
PX, PY = XX + 0.5, YY + 0.5
OUTLINE = "#020201"
ICE, ICE_LINE, SNOW = fz.ICE, fz.ICE_LINE, fz.SNOW
BLOOD = ["#ffb0a8", "#ff4a4a", "#c0242a", "#7a1418", "#3a0a0c"]  # блик … тень (акцент карт жертвы)
GOLD = ["#fff1b4", "#ffc35a", "#ff8a2e", "#d0501b", "#6e2408"]
CYAN = ["#e9fdff", "#a6f2ff", "#57d9ff", "#2aa9d6", "#16627f"]  # огни белых
COLD_SKY = ["#0e1220", "#141a2a", "#1f2a3d"]
BLOOD_SKY = ["#0e0809", "#170b0e", "#2a0f13"]


# ---------------------------------------------------------------- основа
def new():
    return np.zeros((H, W, 4), np.uint8)


def fill(img, mask, color):
    img[mask] = (*hexc(color), 255)


def tint(img, mask, color, a):
    """Цвет поверх с прозрачностью a (0…1) — для свечения, льда, тумана."""
    c = np.array(hexc(color), float)
    img[mask, :3] = (img[mask, :3] * (1 - a) + c * a).astype(np.uint8)
    img[mask, 3] = 255


def darken(img, mask, k=0.62):
    img[mask, :3] = (img[mask, :3] * k).astype(np.uint8)


def sky(img, horizon, colors):
    """Небо тремя полосами до горизонта, на стыках — шахматный переход в пиксель."""
    bounds = [0, horizon // 3, 2 * horizon // 3, horizon]
    for k, col in enumerate(colors):
        fill(img, (YY >= bounds[k]) & (YY < bounds[k + 1]), col)
        if k:
            fill(img, (YY == bounds[k]) & ((XX + YY) % 2 == 0), colors[k - 1])
            fill(img, (YY == bounds[k] - 1) & ((XX + YY) % 2 == 1), col)


def board(img, horizon, cy, edge, tones=("#2b2320", "#221c1b", "#33292a")):
    """Тёмная доска ромбами 32×16 от горизонта вниз, как `breach_cards_hd.dark_board`."""
    ground = YY >= horizon
    gu = (PX - W / 2) / 16 + (PY - cy) / 8
    gv = (PY - cy) / 8 - (PX - W / 2) / 16
    checker = (np.floor(gu).astype(int) + np.floor(gv).astype(int)) % 2 == 0
    fill(img, ground & checker, tones[0])
    fill(img, ground & ~checker, tones[1])
    du = 1 / 16
    seam = ground & ((np.abs(gu - np.round(gu)) < du) | (np.abs(gv - np.round(gv)) < du))
    fill(img, seam, tones[2])
    fill(img, YY == horizon, edge)
    return ground


def stars(img, rng, n, ymax, colors=("#a6f2ff", "#57d9ff")):
    for _ in range(n):
        x, y = int(rng.integers(1, W - 1)), int(rng.integers(0, ymax))
        fill(img, (XX == x) & (YY == y), colors[int(rng.random() < 0.4)])


def snow_cover(img, mask, rng, amount=0.6, cell=6):
    """Снег пятнами по маске земли: сугробы светлее к середине, кромки — тень."""
    n = burn.value_noise(H, W, cell, rng) * 0.7 + burn.value_noise(H, W, 2, rng) * 0.3
    snow = mask & (n > 1 - amount)
    fill(img, snow, SNOW[2])
    fill(img, snow & (n > 1 - amount * 0.55), SNOW[1])
    fill(img, snow & (n > 1 - amount * 0.25), SNOW[0])
    below = np.vstack([snow[1:], np.zeros((1, W), bool)])
    fill(img, snow & ~below, SNOW[3])  # нижняя кромка сугроба
    fill(img, snow & (rng.random((H, W)) < 0.03), "#ffffff")
    return snow


def diamond(cx, cy, rx, ry):
    """Расстояние в «ромбовой» норме: 1 — край ромба."""
    return np.abs(PX - cx) / rx + np.abs(PY - cy) / ry


def ellipse(cx, cy, rx, ry):
    return np.hypot((PX - cx) / rx, (PY - cy) / ry)


def poly(points):
    return art.sd_polygon(PX, PY, points) <= 0


def seg(a, b, w=0.6):
    return art.sd_segment2(PX, PY, a, b) < w


def outline(img, mask, color=OUTLINE):
    fill(img, bc.outline4(mask), color)


def spark(img, x, y, color, r=1):
    """Звёздочка-крестик: центр и лучи длиной r."""
    for dx, dy in [(0, 0)] + [(d * s, 0) for d in range(1, r + 1) for s in (1, -1)] + \
            [(0, d * s) for d in range(1, r + 1) for s in (1, -1)]:
        if 0 <= x + dx < W and 0 <= y + dy < H:
            img[y + dy, x + dx] = (*hexc(color), 255)


def put(img, spr, x0, y0, alpha=None):
    """Спрайт на холст; alpha — общая прозрачность (0…1) или маска пикселей, которые класть."""
    h, w = spr.shape[:2]
    sx0, sy0 = max(0, -x0), max(0, -y0)
    sx1, sy1 = min(w, W - x0), min(h, H - y0)
    if sx0 >= sx1 or sy0 >= sy1:
        return
    part = spr[sy0:sy1, sx0:sx1]
    dst = img[y0 + sy0 : y0 + sy1, x0 + sx0 : x0 + sx1]
    m = part[..., 3] > 0
    if alpha is None:
        dst[m] = part[m]
        return
    a = (part[..., 3:4] / 255.0) * alpha
    mix = dst[..., :3] * (1 - a) + part[..., :3] * a
    dst[..., :3] = np.where(m[..., None], mix, dst[..., :3]).astype(np.uint8)
    dst[m, 3] = 255


# ---------------------------------------------------------------- фигуры
_PIECES = {}


def piece(code, scale=None):
    """Спрайт фигуры: (картинка, точка опоры x, y) — обрезан по непрозрачному. Без scale —
    спрайт игры, со scale — тот же рендер `render_piece` в другом масштабе (игра — 0.86)."""
    key = (code, scale)
    if key in _PIECES:
        return _PIECES[key]
    path = os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")
    cached = CACHE and os.path.join(CACHE, f"piece_{code}_{scale}.png")
    if scale is None:
        body = np.array(Image.open(path).convert("RGBA"))
    elif cached and os.path.exists(cached):
        body = np.array(Image.open(cached).convert("RGBA"))
    else:
        im, _ = art.render_piece(art.PIECES[code[1]](), code[0], scale=scale)
        body = np.array(im)
        if cached:
            os.makedirs(CACHE, exist_ok=True)
            im.save(cached)
    ys, xs = np.nonzero(body[..., 3])
    y0, x0 = ys.min(), xs.min()
    out = (body[y0 : ys.max() + 1, x0 : xs.max() + 1].copy(), art.AX - x0, art.AY - y0)
    _PIECES[key] = out
    return out


def place(img, p, fx, fy, spr=None, alpha=None, shadow=True):
    """Фигура `p` (из `piece`) точкой опоры на (fx, fy); тень — овал вправо-вниз."""
    body, ax, ay = p
    if shadow:
        w = body.shape[1]
        m = ellipse(fx + 2, fy + 1, w * 0.42, w * 0.17) <= 1
        darken(img, m, 0.6)
    put(img, body if spr is None else spr, int(fx - ax), int(fy - ay), alpha)


def frosted(p):
    """Фигура в инее (`breach_freeze.frozen_sprite`)."""
    return (fz.frozen_sprite(p[0]), p[1], p[2])


def ice_body(p):
    """Фигура изо льда: тона по яркости — палитра льда, огни — белые, контур тёмно-синий."""
    body = p[0]
    solid = body[..., 3] > 0
    rgb = body[..., :3].astype(int)
    lum = rgb.mean(axis=2)
    glow = (rgb[..., 2] - rgb[..., 0] > 60) & (lum > 80)
    out = body.copy()
    lo, hi = np.percentile(lum[solid & (lum > 40)], [10, 90])
    k = np.clip((lum - lo) / max(hi - lo, 1), 0, 1)
    tones = np.select([k > 0.88, k > 0.62, k > 0.36, k > 0.14], [0, 1, 2, 3], 4)
    for t in range(5):
        out[solid & (tones == t), :3] = hexc(ICE[t])
    out[solid & (lum < 40), :3] = hexc(ICE_LINE)
    out[solid & glow, :3] = hexc("#ffffff")
    # прожилки-грани: косые светлые штрихи по телу
    yy, xx = np.mgrid[0 : body.shape[0], 0 : body.shape[1]]
    vein = solid & (lum >= 40) & ~glow & (np.abs(((xx - yy * 0.7) % 9) - 4.5) < 0.5)
    out[vein, :3] = hexc(ICE[0])
    return (out, p[1], p[2])


def toppled(p, right=True):
    """Поверженная фигура — лежит на боку (поворот на 90°, спрайт без размытия)."""
    body = np.rot90(p[0], -1 if right else 1).copy()
    h, w = body.shape[:2]
    return (body, w // 2, h - 3)


# ---------------------------------------------------------------- лёд
def prism(base, heights, apex=None):
    """Призма над многоугольником (точки на земле по часовой): левые и правые грани, верх,
    рёбра; apex — остриё (смещение x, высота) над центром основания вместо верха."""
    tops = [(x, y - h) for (x, y), h in zip(base, heights)]
    if apex is not None:
        cx = sum(p[0] for p in base) / len(base)
        cy = sum(p[1] for p in base) / len(base)
        tip = (cx + apex[0], cy - apex[1])
    left = np.zeros((H, W), bool)
    right = np.zeros((H, W), bool)
    edges = np.zeros((H, W), bool)
    n = len(base)
    for i in range(n):
        a, b = base[i], base[(i + 1) % n]
        if b[0] - a[0] >= 0:  # грань смотрит от зрителя
            continue
        face = [a, b, tip] if apex is not None else [a, b, tops[(i + 1) % n], tops[i]]
        m = poly(face)
        if b[1] - a[1] < 0:
            left |= m
        else:
            right |= m
        edges |= seg(a, tip if apex is not None else tops[i])
        edges |= seg(b, tip if apex is not None else tops[(i + 1) % n])
    cap = np.zeros((H, W), bool)
    if apex is None:
        cap = poly(tops)
        for i in range(n):
            edges |= seg(tops[i], tops[(i + 1) % n])
    return left, right, cap, edges


def iso_base(cx, cy, r, k=4, rot=0.0):
    """Основание на земле: k вершин на изометрическом эллипсе (r × 0.75r), по часовой."""
    return [(cx + r * math.cos(rot + 2 * math.pi * i / k), cy + 0.75 * r * math.sin(rot + 2 * math.pi * i / k))
            for i in range(k)]


def block_h(p, r):
    """Высота глыбы радиуса r: верх крышки чуть выше макушки фигуры `p`."""
    return p[2] + 2 - 0.4 * r


def ice_block(img, fx, fy, r, h, variant=0, cracks=None):
    """Полупрозрачная глыба льда вокруг фигуры (как в `breach_freeze`), рисуется после неё.
    variant: 0 — брусок, 1 — восьмигранник, 2 — узкая с осколками, 3 — скошенная."""
    if variant == 0:
        parts = [(iso_base(fx, fy, r), [h - 1, h + 1, h - 2, h], None)]
    elif variant == 1:
        parts = [(iso_base(fx, fy, r * 1.05, 8, math.pi / 8), [h + 2, h + 1, h + 3, h + 2, h + 1, h + 2, h + 3, h + 1], None)]
    elif variant == 2:
        parts = [
            (iso_base(fx - r * 0.75, fy - 2, r * 0.28), [0] * 4, (-r * 0.3, h * 0.55)),
            (iso_base(fx, fy, r * 0.8), [h, h + 1, h - 1, h], None),
            (iso_base(fx + r * 0.75, fy + 2, r * 0.28), [0] * 4, (r * 0.3, h * 0.45)),
        ]
    else:
        parts = [(iso_base(fx, fy, r), [h - 3, h + 4, h + 6, h - 1], None),
                 (iso_base(fx + r * 0.85, fy + 2, r * 0.25), [0] * 4, (r * 0.25, h * 0.5))]
    body = np.zeros((H, W), bool)
    for base, heights, apex in parts:
        left, right, cap, edges = prism(base, heights, apex)
        part = left | right | cap
        tint(img, left, ICE[2], 0.45)
        tint(img, right, ICE[3], 0.55)
        tint(img, cap, ICE[1], 0.55)
        fill(img, left & (np.abs(((XX - YY * 0.6) % 11) - 5.5) < 0.5), ICE[0])
        fill(img, right & (np.abs(((XX + YY * 0.6) % 13) - 6.5) < 0.5), ICE[1])
        fill(img, edges & part, ICE[0])
        fill(img, bc.outline4(part) & ~body, ICE_LINE)
        body |= part
    outline(img, body, ICE_LINE)
    if cracks is not None:  # трещины: ломаные от точки удара
        rng = np.random.default_rng(cracks)
        ys, xs = np.nonzero(body)
        cx, cy = xs.mean(), ys.mean() - 2
        for _ in range(4):
            ang = rng.uniform(0, 2 * math.pi)
            x, y = cx, cy
            for _ in range(3):
                nx, ny = x + math.cos(ang) * 4, y + math.sin(ang) * 4
                m = seg((x, y), (nx, ny), 0.55) & body
                fill(img, m, "#ffffff")
                x, y, ang = nx, ny, ang + rng.uniform(-0.7, 0.7)
    return body


def shard(img, fx, fy, r, h, lean=0.0, colors=None):
    """Острый кристалл на земле: четырёхгранное остриё, светлая левая грань."""
    c = colors or ICE
    left, right, _, edges = prism(iso_base(fx, fy, r), [0] * 4, (lean, h))
    m = left | right
    fill(img, left, c[1])
    fill(img, right, c[3])
    fill(img, left & (np.abs(PX - (fx - r * 0.35 + lean * 0.3)) < 0.6), c[0])
    fill(img, edges & m, c[0] if colors is None else c[0])
    outline(img, m, ICE_LINE if colors is None else OUTLINE)
    return m


def snowflake(img, cx, cy, r, colors=None, glow=True):
    """Шестилучевая снежинка с веточками; свет — сверху-слева."""
    c = colors or [ICE[0], ICE[1], ICE[2]]
    m = np.zeros((H, W), bool)
    for k in range(6):
        a = math.radians(90 + 60 * k)
        tip = (cx + r * math.cos(a), cy - r * math.sin(a))
        m |= seg((cx, cy), tip, 0.95)
        for t, ln in ((0.5, 0.36), (0.78, 0.22)):
            bx, by = cx + r * t * math.cos(a), cy - r * t * math.sin(a)
            for s in (1, -1):
                b = a + s * math.radians(55)
                m |= seg((bx, by), (bx + r * ln * math.cos(b), by - r * ln * math.sin(b)), 0.7)
    m |= ellipse(cx, cy, r * 0.22, r * 0.22) <= 1
    if glow:
        halo = (ellipse(cx, cy, r * 1.35, r * 1.35) <= 1) & ~m
        tint(img, halo & ((XX + YY) % 2 == 0), ICE[3], 0.5)
    fill(img, m, c[0])
    fill(img, m & (PX - cx + (PY - cy) > r * 0.15), c[1])
    outline(img, m, ICE_LINE)
    return m


def flakes(img, rng, n, ymin=0, ymax=H, big=0.35):
    for _ in range(n):
        x, y = int(rng.integers(1, W - 1)), int(rng.integers(ymin, ymax))
        if rng.random() < big:
            spark(img, x, y, ICE[1])
            fill(img, (XX == x) & (YY == y), "#ffffff")
        else:
            fill(img, (XX == x) & (YY == y), SNOW[int(rng.integers(0, 2))])

# ---------------------------------------------------------------- обычные карты
def wind_band(img, rng, yc, n, spread=3):
    """Полоса ветра со снегом: штрихи слева направо, голова — белая, хвост — пунктир."""
    for _ in range(n):
        x0 = int(rng.integers(-8, W - 4))
        ln = int(rng.integers(6, 17))
        y0 = yc + int(rng.integers(-spread, spread + 1))
        ph = rng.uniform(0, 6)
        for k in range(ln):
            x = x0 + k
            y = y0 + int(round(1.1 * math.sin(k / 5.0 + ph)))
            if not (0 <= x < W and 0 <= y < H):
                continue
            if k >= ln - 2:
                col = "#ffffff"
            elif k > ln // 3:
                col = ICE[1]
            elif k % 2 == 0:
                col = ICE[2]
            else:
                continue
            img[y, x] = (*hexc(col), 255)
    for _ in range(n):  # хлопья в полосе — короткие чёрточки
        x, y = int(rng.integers(0, W - 1)), yc + int(rng.integers(-spread - 1, spread + 2))
        if 0 <= y < H:
            img[y, x : x + 2] = (*hexc(SNOW[0]), 255)


def drift(img, cx, cy, rx, ry):
    """Сугробик у основания фигуры: свет сверху, тень снизу, контур голубоватый."""
    m = (ellipse(cx, cy, rx, ry) <= 1) & (PY <= cy + 1)
    fill(img, m, SNOW[1])
    fill(img, m & (PY < cy - ry * 0.35), SNOW[0])
    fill(img, m & (PY > cy - 0.5), SNOW[3])
    outline(img, m, "#2f3d52")
    return m


def windswept(img, mask, rng, amount=0.5):
    """Снег, вытянутый ветром по горизонтали: полосы-заносы."""
    n = burn.value_noise(H, W // 4 + 1, 2, rng).repeat(4, axis=1)[:, :W]
    n = 0.75 * n + 0.25 * burn.value_noise(H, W, 2, rng)
    snow = mask & (n > 1 - amount)
    fill(img, snow, SNOW[2])
    fill(img, snow & (n > 1 - amount * 0.5), SNOW[1])
    below = np.vstack([snow[1:], np.zeros((1, W), bool)])
    fill(img, snow & ~below, SNOW[3])
    return snow


def snowed(p, side=-1, depth=2):
    """Снег, налипший на фигуру с наветренной стороны (side −1 — слева): крайние пиксели
    силуэта на `depth` от кромки белеют, кроме контура."""
    body = p[0].copy()
    solid = body[..., 3] > 0
    inner = solid & (body[..., :3].sum(axis=2) >= 60)
    edge = np.zeros_like(solid)
    for d in range(1, depth + 2):
        edge |= solid & ~np.roll(solid, -side * d, 1)
    rows = np.arange(body.shape[0])[:, None] % 5 != 0  # налипло не сплошь
    m = inner & edge & rows
    body[m, :3] = hexc(SNOW[1])
    lit = m & ~np.roll(m, 1, 0)
    body[lit, :3] = hexc(SNOW[0])
    return (body, p[1], p[2])


def art_storm(seed=31):
    """Буря: три полосы ветра со снегом поперёк поля, фигуры обеих сторон в инее."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 24
    sky(img, horizon, COLD_SKY)
    stars(img, rng, 6, 8)
    ground = board(img, horizon, 36, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    windswept(img, ground, rng, 0.3)
    for code, fx, fy in (("wP", 22, 39), ("bP", 54, 46)):
        place(img, snowed(frost_lower(piece(code), 0.3)), fx, fy)
    for yc, n in ((6, 8), (17, 8), (29, 9)):
        wind_band(img, rng, yc, n)
    return img


def billow(img, cx, cy, r, under):
    """Клуб снега: шар с тенью вправо-вниз; по снегу позади — кромка-разделитель."""
    m = ellipse(cx, cy, r, r * 0.85) <= 1
    k = (-(PX - cx) * 0.6 - (PY - cy) * 0.8) / r
    fill(img, bc.outline4(m) & under, SNOW[4])
    fill(img, m, SNOW[4])
    fill(img, m & (k > -0.7), SNOW[3])
    fill(img, m & (k > -0.3), SNOW[2])
    fill(img, m & (k > 0.15), SNOW[1])
    fill(img, m & (k > 0.55), SNOW[0])
    return m


def art_avalanche(seed=32):
    """Лавина: клубы снега катятся с горы на поле, фигура в инее под снежной пылью."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 30
    sky(img, horizon, COLD_SKY)
    stars(img, rng, 8, 8)
    peaks = [(-2, 31), (10, 14), (22, 24), (38, 4), (56, 22), (66, 12), (78, 26), (78, 31)]
    mount = poly(peaks)
    fill(img, mount, "#1a2232")
    for (x0, y0), (x1, y1) in zip(peaks, peaks[1:]):  # левые склоны светлее
        if y1 < y0:
            fill(img, mount & poly([(x0, y0), (x1, y1), (x1 + 3, 31), (x0, 31)]) & (PX < x1), "#253046")
    cap = mount & (PY < 31 - (31 - 4) * 0.62 + 3 * np.sin(PX * 0.9))
    fill(img, cap, SNOW[2])
    fill(img, cap & (PX < 38 - (PY - 4) * 0.2), SNOW[1])
    outline(img, mount, "#0b0f18")
    ground = board(img, horizon, 40, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    snow_cover(img, ground, rng, 0.5)
    p = frosted(piece("wP"))
    place(img, p, 17, 46)
    snow = np.zeros((H, W), bool)
    puffs = []
    for _ in range(30):  # клубы вразброс по клину: узкий у вершины, широкий фронт внизу
        t = rng.uniform(0, 1) ** 0.8
        cx, cy = 60 - 24 * t, 6 + 34 * t
        r = 2.0 + 5.0 * t * rng.uniform(0.6, 1.25)
        puffs.append((cy + rng.uniform(-2, 2), cx + rng.uniform(-1, 1) * (2 + 12 * t), r))
    for cy, cx, r in sorted(puffs):  # от дальних к ближним
        snow |= billow(img, cx, cy, r, snow)
    fill(img, bc.outline4(snow), "#2a3850")
    for _ in range(6):  # снежные струи, сорвавшиеся вниз по склону
        x, y = int(rng.integers(30, 62)), int(rng.integers(8, 34))
        fill(img, (XX == x) & (YY > y) & (YY < y + int(rng.integers(3, 7))) & ~snow & ((YY % 2) == 0), SNOW[1])
    for _ in range(26):  # снежная пыль впереди лавины
        x, y = int(rng.integers(8, 40)), int(rng.integers(26, 49))
        fill(img, (XX == x) & (YY == y) & ~snow, SNOW[int(rng.integers(0, 2))])
    for _ in range(5):  # камни в снегу
        x, y = int(rng.integers(36, 62)), int(rng.integers(12, 40))
        if snow[y, x]:
            img[y, x : x + 2] = (*hexc("#3a3430"), 255)
    return img


def art_ice_barrier(seed=33):
    """Ледяной барьер: зона 3×3 под снегом с ледяной кромкой и шипами, внутри — фигура в глыбе."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 16
    sky(img, horizon, COLD_SKY)
    stars(img, rng, 7, 10)
    board(img, horizon, 30, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    cx, cy, rx, ry = 38, 37, 35, 12.5
    d = diamond(cx, cy, rx, ry)
    zone = d <= 1.0
    lumps = burn.value_noise(H, W, 4, rng)
    fill(img, zone, SNOW[2])
    fill(img, zone & (lumps > 0.6), SNOW[1])
    fill(img, zone & (lumps > 0.75), SNOW[0])
    fill(img, zone & (lumps < 0.3), SNOW[3])
    u = (PX - cx) / rx + (PY - cy) / ry
    v = (PY - cy) / ry - (PX - cx) / rx
    grid = zone & ((np.abs(np.abs(u) - 1 / 3) < 0.03) | (np.abs(np.abs(v) - 1 / 3) < 0.03))
    fill(img, grid & ((XX + YY) % 2 == 0), SNOW[3])
    halo = (d > 1.0) & (d <= 1.18)
    tint(img, halo & ((XX + YY) % 2 == 0), ICE[2], 0.6)
    fill(img, (d > 0.9) & zone, ICE[1])
    fill(img, (d > 0.96) & zone, ICE[0])
    back = [(cx - rx, cy), (cx, cy - ry), (cx + rx, cy)]
    front = [(cx - rx, cy), (cx, cy + ry), (cx + rx, cy)]

    def along(pts, step):
        out = []
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            n = int(math.hypot(x1 - x0, y1 - y0) / step)
            out += [(x0 + (x1 - x0) * t / n, y0 + (y1 - y0) * t / n) for t in range(n)]
        return out + [pts[-1]]

    for x, y in along(back, 6.5):
        shard(img, x, y, rng.uniform(2.2, 3.2), rng.uniform(5, 11), rng.uniform(-1.5, 1.5))
    p = piece("bP", 0.62)
    place(img, p, cx, cy + 3, shadow=False)
    ice_block(img, cx, cy + 3, 12, block_h(p, 12), 1)
    for x, y in along(front, 8)[1:-1]:
        shard(img, x, y + 1, rng.uniform(1.8, 2.6), rng.uniform(3, 6), rng.uniform(-1, 1))
    for _ in range(9):
        spark(img, int(rng.integers(4, W - 4)), int(rng.integers(18, 46)), "#ffffff", int(rng.integers(0, 2)))
    return img


def art_multiply(seed=34):
    """Они множатся: фигура делится — рядом проявляется её копия, между ними след-призраки."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 22
    sky(img, horizon, ["#0f1220", "#141a28", "#123038"])
    stars(img, rng, 9, 12)
    board(img, horizon, 36, "#1d4450", ("#232a36", "#1c222c", "#2b3442"))
    p = piece("wP")
    body, ax, ay = p
    solid = body[..., 3] > 0
    a, b = (23, 42), (54, 42)
    for (fx, fy), strong in ((a, True), (b, False)):
        d = diamond(fx, fy, 15, 6.5)
        tint(img, d <= 1, CYAN[4], 0.45 if strong else 0.3)
        fill(img, (d > 0.86) & (d <= 1) & (strong | ((XX + YY) % 2 == 0)), CYAN[2])
    place(img, p, *a)
    rim = solid & ~(np.roll(solid, 1, 0) & np.roll(solid, -1, 0) & np.roll(solid, 1, 1) & np.roll(solid, -1, 1))
    yy, xx = np.mgrid[0 : body.shape[0], 0 : body.shape[1]]
    for k, fx in enumerate((33, 43)):  # призраки: контур в пунктир, гаснет к копии
        ghost = np.zeros(body.shape, np.uint8)
        ghost[rim & ((xx + yy + k) % 2 == 0)] = (*hexc(CYAN[2 + k]), 255)
        put(img, ghost, fx - ax, a[1] - ay)
    copy = body.astype(float)
    copy[..., :3] = copy[..., :3] * 0.4 + np.array(hexc(CYAN[1]), float) * 0.6
    copy = copy.astype(np.uint8)
    rows = (yy % 3 == 0) & solid
    copy[rows, :3] = (copy[rows, :3] * 0.75).astype(np.uint8)  # строки голограммы
    copy[rim, :3] = hexc(CYAN[2])
    put(img, copy, b[0] - ax, b[1] - ay, alpha=0.85)
    for _ in range(14):  # искры деления
        x = int(rng.integers(30, 70))
        y = int(rng.integers(6, 40))
        if img[y, x, :3].sum() < 260:
            spark(img, x, y, CYAN[int(rng.integers(0, 3))], int(rng.random() < 0.3))
    return img


def art_ice_wall(seed=35):
    """Ледяная стена: ряд ледяных шипов вдоль горизонтали поля, в стене — замёрзшая фигура."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 18
    sky(img, horizon, COLD_SKY)
    stars(img, rng, 10, 12)
    ground = board(img, horizon, 34, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    line = lambda x: 30 + x * 0.17  # noqa: E731
    near = ground & (np.abs(PY - line(PX)) < 4.5 + 2 * burn.value_noise(H, W, 5, rng))
    snow_cover(img, near, rng, 0.95, 3)
    glow = (PY < line(PX)) & (PY > line(PX) - 16)
    tint(img, glow & ((XX + YY) % 2 == 0), ICE[3], 0.35)
    xs = []
    x = -3.0
    while x < W + 3:
        xs.append(x)
        x += rng.uniform(5.0, 7.0)
    hero = 40
    p = frosted(piece("bP"))
    drawn = False
    for x in xs:
        if abs(x - hero) < 9:
            continue
        if x > hero and not drawn:
            place(img, p, hero, line(hero) + 1, shadow=False)
            drawn = True
        mid = 1 - abs(x - W / 2) / (W / 2)
        shard(img, x, line(x), rng.uniform(3.4, 4.8), 12 + 12 * mid + rng.uniform(-3, 3), rng.uniform(-2, 2))
    for dx, h in ((-6, 9), (6, 8)):
        shard(img, hero + dx, line(hero + dx) + 4, 3.0, h, -dx * 0.2)
    for _ in range(10):
        spark(img, int(rng.integers(2, W - 2)), int(rng.integers(4, 40)), "#ffffff", int(rng.random() < 0.3))
    return img


DROP = [".a.", "abb", "bbc", ".c."]


def drop(img, x, y, colors=None):
    """Капля крови 3×4 с контуром."""
    c = colors or {"a": BLOOD[1], "b": BLOOD[2], "c": BLOOD[3]}
    m = np.zeros((H, W), bool)
    for dy, row in enumerate(DROP):
        for dx, ch in enumerate(row):
            if ch != "." and 0 <= y + dy < H and 0 <= x + dx < W:
                m[y + dy, x + dx] = True
    outline(img, m)
    for dy, row in enumerate(DROP):
        for dx, ch in enumerate(row):
            if ch != "." and 0 <= y + dy < H and 0 <= x + dx < W:
                img[y + dy, x + dx] = (*hexc(c[ch]), 255)


def bloodied(p, frac=0.45):
    """Фигура, отдающая здоровье: нижняя часть залита красным."""
    body = p[0].copy()
    solid = body[..., 3] > 0
    rows = np.arange(body.shape[0])[:, None] >= body.shape[0] * (1 - frac)
    m = solid & rows & (body[..., :3].sum(axis=2) > 60)
    body[m, :3] = (body[m, :3] * 0.5 + np.array(hexc(BLOOD[2])) * 0.5).astype(np.uint8)
    return (body, p[1], p[2])


def art_mass_sacrifice(seed=36):
    """Массовое жертвоприношение: свои фигуры в кровавом квадрате 3×3 отдают кровь вверх."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 16
    sky(img, horizon, ["#0f0a0d", "#170c10", "#2c1016"])
    board(img, horizon, 30, "#4a1820", ("#2a1c1e", "#211617", "#33232a"))
    cx, cy, rx, ry = 38, 37, 35, 12.5
    d = diamond(cx, cy, rx, ry)
    zone = d <= 1
    tint(img, zone, BLOOD[4], 0.45)
    u = (PX - cx) / rx + (PY - cy) / ry
    v = (PY - cy) / ry - (PX - cx) / rx
    grid = zone & ((np.abs(np.abs(u) - 1 / 3) < 0.035) | (np.abs(np.abs(v) - 1 / 3) < 0.035))
    fill(img, grid, BLOOD[3])
    tint(img, (d > 1) & (d <= 1.2) & ((XX + YY) % 2 == 0), BLOOD[2], 0.5)
    fill(img, (d > 0.9) & zone, BLOOD[2])
    fill(img, (d > 0.96) & zone, BLOOD[1])
    p = bloodied(piece("wP", 0.58))
    spots = [(38, 30), (21, 37), (55, 37), (38, 45)]
    for fx, fy in spots:
        place(img, p, fx, fy)
    for fx, fy in spots:  # капли поднимаются над головами
        top = fy - p[2]
        for k in range(3):
            drop(img, fx - 1 + int(rng.integers(-4, 5)), top - 5 - k * 6 + int(rng.integers(-1, 2)))
    for _ in range(12):
        x, y = int(rng.integers(4, W - 4)), int(rng.integers(1, 30))
        fill(img, (XX == x) & (YY == y), BLOOD[int(rng.integers(1, 3))])
    return img


def frost_lower(p, frac):
    """Иней снизу на доле высоты фигуры, кромка — белая (как в кадре заморозки)."""
    body = p[0]
    frozen = fz.frozen_sprite(body)
    line = body.shape[0] * (1 - frac)
    rows = np.arange(body.shape[0])[:, None] >= line
    out = body.copy()
    m = rows & (body[..., 3] > 0)
    out[m] = frozen[m]
    above = np.vstack([np.zeros((1, m.shape[1]), bool), m[:-1]])
    out[m & ~above & (body[..., :3].sum(axis=2) >= 60)] = (*hexc(ICE[0]), 255)
    return (out, p[1], p[2])


def art_absolute_zero(seed=37):
    """Абсолютный ноль: огромная снежинка над полем, фигуры соперника в глыбах навсегда."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 27
    sky(img, horizon, ["#0a0f1e", "#0f1830", "#16264a"])
    ang = np.degrees(np.arctan2(PY - 13, PX - 38)) % 360
    rays = (YY < horizon) & ((ang // 15) % 2 == 0) & (ellipse(38, 13, 40, 30) <= 1)
    tint(img, rays, "#2a4a78", 0.35)
    ground = board(img, horizon, 38, "#4a6a90", ("#232a36", "#1c222c", "#2b3442"))
    snow_cover(img, ground, rng, 0.8, 5)
    snowflake(img, 38, 12, 10.5)
    for code, fx, fy, v in (("bP", 13, 43, 0), ("bN", 38, 49, 1), ("bB", 63, 43, 3)):
        p = piece(code, 0.48)
        place(img, p, fx, fy, shadow=False)
        r = 8.5 if code == "bP" else 9.5
        ice_block(img, fx, fy, r, block_h(p, r), v)
    flakes(img, rng, 18, 0, 30)
    return img


def art_ice_shard(seed=38):
    """Ледяной снаряд: граненая сосулька летит в фигуру соперника, удар — осколки и иней."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 22
    sky(img, horizon, COLD_SKY)
    stars(img, rng, 10, 14)
    board(img, horizon, 36, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    p = frost_lower(piece("bP"), 0.5)
    place(img, p, 57, 44)
    tail, tip = np.array([3.0, 5.0]), np.array([45.0, 25.0])
    ax = (tip - tail) / np.linalg.norm(tip - tail)
    nrm = np.array([ax[1], -ax[0]])  # вверх от оси
    mid = tip - ax * 14
    up, down = mid + nrm * 5.2, mid - nrm * 4.6
    back = tail + ax * 6
    for k in range(9):  # след: снежная пыль и штрихи вдоль полёта
        s = rng.uniform(0, 1)
        pt = tail + (back - tail) * s - ax * rng.uniform(0, 4) + nrm * rng.uniform(-4, 4)
        x, y = int(pt[0]), int(pt[1])
        if 0 <= x < W and 0 <= y < H:
            fill(img, (XX == x) & (YY == y), SNOW[int(rng.integers(0, 3))])
    for off in (-3.0, 0.0, 3.0):
        a = tail + nrm * off - ax * 6
        b = back + nrm * off * 0.6
        m = seg(tuple(a), tuple(b), 0.5) & ((XX + YY) % 2 == 0)
        fill(img, m, ICE[2])
    top_face = poly([tuple(tip), tuple(up), tuple(back)])
    low_face = poly([tuple(tip), tuple(back), tuple(down)])
    body = top_face | low_face
    fill(img, low_face, ICE[3])
    fill(img, top_face, ICE[1])
    fill(img, seg(tuple(tip), tuple(back), 0.55) & body, ICE[0])
    fill(img, seg(tuple(tip), tuple(up), 0.55) & body, "#ffffff")
    outline(img, body, ICE_LINE)
    ix, iy = int(tip[0]) + 2, int(tip[1]) + 1  # удар: лучи-осколки
    for k in range(7):
        a = math.radians(-60 + k * 30 + rng.uniform(-8, 8))
        ln = rng.uniform(4, 7)
        m = seg((ix + math.cos(a) * 2, iy + math.sin(a) * 2), (ix + math.cos(a) * ln, iy + math.sin(a) * ln), 0.6)
        fill(img, bc.outline4(m), ICE_LINE)
        fill(img, m, ICE[0] if k % 2 else ICE[1])
    spark(img, ix, iy, "#ffffff", 3)
    flakes(img, rng, 10, 0, 40)
    return img


def art_ice_blast(seed=39):
    """Ледяной взрыв: глыба с фигурой соперника разлетается осколками, по земле — кольцо инея."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 20
    sky(img, horizon, COLD_SKY)
    board(img, horizon, 34, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    cx, cy = 38, 40
    ang = np.degrees(np.arctan2(PY - 26, PX - cx)) % 360
    rays = ((ang // 14) % 2 == 0) & (ellipse(cx, 26, 44, 30) <= 1)
    tint(img, rays, ICE[3], 0.3)
    e = ellipse(cx, cy, 35, 11)
    tint(img, (e < 1) & ((XX + YY) % 2 == 0), ICE[3], 0.35)
    fill(img, np.abs(e - 1) < 0.05, ICE[1])
    fill(img, np.abs(e - 0.55) < 0.06, ICE[2])
    spikes = [(cx + 35 * math.cos(t), cy + 11 * math.sin(t)) for t in np.linspace(0, 2 * math.pi, 14, endpoint=False)]
    for x, y in sorted(spikes, key=lambda s: s[1]):
        if y < cy:
            shard(img, x, y, 2.2, rng.uniform(4, 7), (x - cx) * 0.06)
    p = piece("bP", 0.62)
    place(img, p, cx, cy, shadow=False)
    ice_block(img, cx, cy, 13, block_h(p, 13), 2, cracks=seed)
    for x, y in sorted(spikes, key=lambda s: s[1]):
        if y >= cy:
            shard(img, x, y, 2.4, rng.uniform(4, 7), (x - cx) * 0.06)
    for k in range(10):  # осколки летят наружу
        a = math.radians(k * 36 + rng.uniform(-10, 10))
        dist = rng.uniform(17, 25)
        sx, sy = cx + math.cos(a) * dist, 24 + math.sin(a) * dist * 0.7
        d = np.array([math.cos(a), math.sin(a) * 0.7])
        d /= np.linalg.norm(d)
        n = np.array([d[1], -d[0]])
        c = np.array([sx, sy])
        pts = [c + d * 3.5, c + n * 1.6, c - d * 2.5, c - n * 1.6]
        m = poly([tuple(q) for q in pts])
        fill(img, m, ICE[1])
        fill(img, m & poly([tuple(pts[0]), tuple(pts[1]), tuple(pts[2])]), ICE[0])
        outline(img, m, ICE_LINE)
    spark(img, cx, 22, "#ffffff", 3)
    return img

# ---------------------------------------------------------------- карты жертвы
BLOOD_BOARD = ("#241819", "#1c1314", "#2e2022")
BLACK_PAWN = {"h": "#8a80b6", "l": "#544c78", "m": "#342e4c", "d": "#1f1a30",
              "t": "#94613f", "T": "#643f29", "c": "#ff8a2e", "C": "#ffc35a"}


def candle(img, x, y, h=8):
    """Свеча: восковой столбик 3×h с тенью справа, огонёк и тёплый отсвет."""
    tint(img, (ellipse(x + 1, y - h - 3, 7, 6) <= 1) & ((XX + YY) % 2 == 0), GOLD[2], 0.25)
    body = (XX >= x) & (XX <= x + 2) & (YY > y - h) & (YY <= y)
    fill(img, body, "#e8dcc0")
    fill(img, body & (XX == x + 2), "#b8a888")
    fill(img, body & (YY == y - h + 1), "#fff6e0")
    outline(img, body)
    fill(img, (XX == x + 1) & (YY == y - h), "#2a1a12")  # фитиль
    flame = ((XX == x + 1) & (YY >= y - h - 4) & (YY <= y - h - 1)) | ((XX >= x) & (XX <= x + 2) & (YY >= y - h - 2) & (YY <= y - h - 1))
    fill(img, bc.outline4(flame) & ~body, OUTLINE)
    fill(img, flame, GOLD[1])
    fill(img, flame & (YY >= y - h - 2) & (XX == x + 1), GOLD[0])
    fill(img, flame & (YY == y - h - 4), GOLD[2])


def slab(img, cx, cy, rx, ry, h, top, left, right, edge):
    """Каменная плита в изометрии: верх-ромб и две видимые боковые грани."""
    L, T, R, B = (cx - rx, cy - h), (cx, cy - h - ry), (cx + rx, cy - h), (cx, cy - h + ry)
    lf = poly([L, B, (B[0], B[1] + h), (L[0], L[1] + h)])
    rf = poly([B, R, (R[0], R[1] + h), (B[0], B[1] + h)])
    tp = poly([L, T, R, B])
    fill(img, lf, left)
    fill(img, rf, right)
    fill(img, tp, top)
    fill(img, (seg(L, B, 0.55) | seg(B, R, 0.55)) & (lf | rf | tp), edge)
    outline(img, lf | rf | tp)
    return tp, lf, rf, (L, T, R, B)


def dissolving(p, rng, frac=0.3):
    """Фигура уходит кровавыми искрами: верх рассыпается, частицы — над ней."""
    body = p[0].copy()
    hgt = body.shape[0]
    solid = body[..., 3] > 0
    line = hgt * frac + 2 * np.sin(np.arange(body.shape[1]) * 1.3)[None, :]
    yy = np.arange(hgt)[:, None]
    gone = solid & (yy < line) & (rng.random(body.shape[:2]) < 0.85 - 1.6 * yy / hgt)
    body[gone] = 0
    edge = solid & ~gone & (yy < line + 3)
    body[edge & (body[..., :3].sum(axis=2) >= 60), :3] = hexc(BLOOD[1])
    return (body, p[1], p[2])


def art_offering(seed=41):
    """Жертва: своя фигура на каменном алтаре рассыпается кровавыми искрами, по бокам свечи."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 24
    sky(img, horizon, BLOOD_SKY)
    ang = np.degrees(np.arctan2(PY - 30, PX - 38)) % 360
    rays = (YY < horizon) & ((ang // 16) % 2 == 0) & (ellipse(38, 30, 40, 34) <= 1)
    tint(img, rays, BLOOD[3], 0.35)
    board(img, horizon, 38, "#4a1820", BLOOD_BOARD)
    tp, lf, rf, (L, T, R, B) = slab(img, 38, 44, 20, 8, 7, "#5a5354", "#3e3839", "#2a2526", "#7a7172")
    stone = (lf | rf | tp) & (rng.random((H, W)) < 0.08)
    darken(img, stone, 0.8)
    fill(img, seg((38, 37), B, 0.6) & tp, BLOOD[2])  # кровосток
    for x in (31, 36, 41, 46):  # подтёки по граням
        y0 = 37 + 8 - abs(x - 38) * 0.4
        ln = int(rng.integers(2, 6))
        fill(img, (XX == x) & (YY >= y0) & (YY < y0 + ln) & (lf | rf), BLOOD[2])
        fill(img, (XX == x) & (YY == int(y0 + ln)) & (lf | rf), BLOOD[3])
    p = dissolving(piece("wP", 0.72), rng)
    place(img, p, 38, 37, shadow=False)
    top = 37 - p[2]
    for _ in range(16):  # кровавые искры уходят вверх
        x = int(38 + rng.normal(0, 4.5))
        y = int(top + rng.uniform(-12, 4))
        if 0 <= x < W and 0 <= y < H:
            spark(img, x, y, BLOOD[int(rng.integers(0, 3))], int(rng.random() < 0.25))
    candle(img, 10, 42, 9)
    candle(img, 64, 42, 7)
    return img


def crack(img, sx, sy, rng, steps=4, step=4.0, ang=math.pi / 2, spread=0.8, mask=None):
    """Трещина-ломаная: тёмная линия с красным свечением по краям."""
    pts = [(sx, sy)]
    for _ in range(steps):
        ang += rng.uniform(-spread, spread)
        x, y = pts[-1]
        pts.append((x + math.cos(ang) * step, y + math.sin(ang) * step))
    m = np.zeros((H, W), bool)
    for a, b in zip(pts, pts[1:]):
        m |= seg(a, b, 0.55)
    if mask is not None:
        m &= mask
    glow = bc.outline4(m) & (mask if mask is not None else True)
    fill(img, glow, BLOOD[1])
    fill(img, m, OUTLINE)
    return m


def art_encircle(seed=42):
    """Окружены и сломлены: чужой король в кольце из ваших фигур, по нему идут трещины."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 16
    sky(img, horizon, BLOOD_SKY)
    stars(img, rng, 5, 10, (BLOOD[1], BLOOD[2]))
    board(img, horizon, 32, "#4a1820", BLOOD_BOARD)
    e = ellipse(38, 39, 33, 11)
    tint(img, (e <= 1) & ((XX + YY) % 2 == 0), BLOOD[4], 0.5)
    fill(img, np.abs(e - 1) < 0.055, BLOOD[2])
    fill(img, (np.abs(e - 1) < 0.055) & (PY < 39), BLOOD[1])
    king = piece("bK", 0.56)
    pawn = piece("wP", 0.55)
    figs = [(pawn, 18, 31), (pawn, 58, 31), (king, 38, 41), (pawn, 11, 46), (pawn, 65, 46)]
    for p, fx, fy in sorted(figs, key=lambda f: f[2]):
        place(img, p, fx, fy)
        if p is king:
            body, ax, ay = p
            x0, y0 = int(fx - ax), int(fy - ay)
            kmask = np.zeros((H, W), bool)
            hh, ww = body.shape[:2]
            kmask[y0 : y0 + hh, x0 : x0 + ww] = (body[..., 3] > 0) & (body[..., :3].sum(axis=2) >= 40)
            crack(img, fx, y0 + 6, rng, 6, 4.5, math.pi / 2, 0.7, kmask)
            crack(img, fx - 2, y0 + 18, rng, 2, 4.0, math.pi * 0.85, 0.4, kmask)
            crack(img, fx + 2, y0 + 24, rng, 2, 4.0, math.pi * 0.15, 0.4, kmask)
            for dx, dy in ((-11, -6), (10, -2), (12, -12)):  # отколотые куски
                chip = (np.abs(PX - (fx + dx)) + np.abs(PY - (y0 + 20 + dy)) <= 1.6)
                fill(img, bc.outline4(chip), OUTLINE)
                fill(img, chip, "#544c78")
    return img


BONE_RED = ["#f0dccc", "#cfae9c", "#9a7466", "#5c3a34"]


def art_blood_god(seed=43):
    """Кровь Богу Крови: рогатый череп-идол плачет кровью в чашу на алтаре."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 34
    sky(img, horizon, ["#0c0607", "#18080a", "#320b10"])
    ang = np.degrees(np.arctan2(PY - 17, PX - 38)) % 360
    rays = (YY < horizon) & ((ang // 12) % 2 == 0)
    tint(img, rays, BLOOD[3], 0.4)
    board(img, horizon, 42, "#5a1820", BLOOD_BOARD)
    # рога: толстые дуги, закручены наружу и вниз
    for s in (-1, 1):
        cx, cy = 38 + s * 17, 12
        horn = np.zeros((H, W), bool)
        ridge = np.zeros((H, W), bool)
        n = 26
        for i in range(n):
            t = i / (n - 1)
            a = math.radians(200 - 300 * t) if s < 0 else math.radians(-20 + 300 * t)
            r = 8.5 - 4.5 * t
            x, y = cx + s * 0 + r * math.cos(a), cy - r * math.sin(a)
            w = 3.6 - 2.4 * t
            m = ellipse(x, y, w, w) <= 1
            horn |= m
            if i % 3 == 0:
                ridge |= seg((x - w * 0.6, y), (x + w * 0.6, y), 0.5) & m
        fill(img, bc.outline4(horn), OUTLINE)
        fill(img, horn, "#4a3628")
        light, dark = bc.shade_mask(horn)
        fill(img, light, "#7a5c40")
        fill(img, dark, "#2c1e16")
        fill(img, ridge & ~light, "#2c1e16")
    # череп
    cran = ellipse(38, 16, 11.5, 10) <= 1
    jaw = poly([(29.5, 21), (46.5, 21), (45, 30), (31, 30)])
    skull = cran | jaw
    fill(img, bc.outline4(skull), OUTLINE)
    k = (-(PX - 38) * 0.55 - (PY - 14) * 0.85) / 11
    fill(img, skull, BONE_RED[2])
    fill(img, skull & (k > -0.35), BONE_RED[1])
    fill(img, skull & (k > 0.35), BONE_RED[0])
    eyes = (ellipse(33.5, 18, 3.2, 3.4) <= 1) | (ellipse(42.5, 18, 3.2, 3.4) <= 1)
    fill(img, eyes, "#140305")
    for ex in (33, 42):
        fill(img, (XX >= ex) & (XX <= ex + 1) & (YY >= 18) & (YY <= 19), BLOOD[1])
        fill(img, (XX == ex) & (YY == 18), BLOOD[0])
    nose = poly([(36.4, 22.2), (39.6, 22.2), (38, 25.2)])
    fill(img, nose, "#140305")
    teeth = (YY >= 26) & (YY <= 28) & (PX > 32) & (PX < 44)
    fill(img, teeth & ((XX % 2) == 0), BONE_RED[3])
    fill(img, (YY == 27) & (PX > 32) & (PX < 44), "#140305")
    # кровавые слёзы и поток в чашу
    for ex in (33, 43):
        ln = int(rng.integers(5, 9))
        m = (XX == ex) & (YY >= 21) & (YY < 21 + ln) & skull
        fill(img, m, BLOOD[2])
        fill(img, (XX == ex) & (YY == 21 + ln), BLOOD[1])
    stream = (np.abs(PX - 38) <= 1.0) & (YY >= 30) & (YY <= 41)
    fill(img, stream, BLOOD[2])
    fill(img, stream & (XX == 37), BLOOD[1])
    # чаша на алтаре
    slab(img, 38, 48, 15, 5, 4, "#4a3a3c", "#342829", "#241b1c", "#6a5658")
    bowl_out = ellipse(38, 41, 10, 3.6) <= 1
    bowl_side = poly([(28, 41), (48, 41), (45, 45), (31, 45)])
    fill(img, bc.outline4(bowl_out | bowl_side), OUTLINE)
    fill(img, bowl_side, "#5a4a3c")
    fill(img, bowl_side & (PX > 41), "#3a2e26")
    fill(img, bowl_out, "#6a5a4a")
    fill(img, ellipse(38, 41, 8.5, 2.6) <= 1, BLOOD[2])
    fill(img, (ellipse(36, 40.6, 3, 1) <= 1), BLOOD[1])
    for _ in range(12):  # брызги
        x, y = int(rng.integers(20, 56)), int(rng.integers(35, 49))
        if not (bowl_out | bowl_side)[y, x]:
            fill(img, (XX == x) & (YY == y), BLOOD[int(rng.integers(1, 4))])
    return img


def gem(img, cx, cy, hw=3.0, hh=4.0):
    """Ромб маны (как на обычных картах), голубой, с бликом."""
    d = diamond(cx, cy, hw, hh)
    m = d <= 1
    fill(img, bc.outline4(m), ICE_LINE)
    fill(img, m, CYAN[3])
    fill(img, m & (d <= 0.6), CYAN[2])
    fill(img, m & (d <= 0.6) & (PY < cy), CYAN[1])
    fill(img, (XX == int(cx - 1)) & (YY == int(cy - 2)), CYAN[0])


def art_cold_ruler(seed=44):
    """Хладнокровный правитель: ледяная корона на бархатной подушке, вокруг всплывают ромбы маны."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 30
    sky(img, horizon, ["#0b0a10", "#100f18", "#1a1a2a"])
    stars(img, rng, 9, 18)
    board(img, horizon, 40, "#2a2a40", ("#1e1b22", "#18161c", "#26232b"))
    # подушка
    tint(img, (ellipse(38, 44, 24, 6) <= 1) & ((XX + YY) % 2 == 0), ICE[3], 0.3)
    side = (ellipse(38, 41, 20, 6.5) <= 1) & (PY >= 37)
    side |= poly([(18, 37), (58, 37), (58, 41), (18, 41)]) & (ellipse(38, 41, 20.5, 7) <= 1.05)
    topm = ellipse(38, 37, 20, 6.5) <= 1
    fill(img, bc.outline4(side | topm), OUTLINE)
    fill(img, side, "#3a0a0c")
    fill(img, topm, "#7a1418")
    fill(img, topm & (ellipse(34, 36, 12, 3.6) <= 1), "#a01c24")
    fill(img, topm & (ellipse(32, 35.4, 5, 1.4) <= 1), "#c4343e")
    for tx in (17, 57):  # кисточки
        t = (XX >= tx) & (XX <= tx + 1) & (YY >= 39) & (YY <= 43)
        fill(img, bc.outline4(t), OUTLINE)
        fill(img, t, GOLD[1])
        fill(img, t & (YY == 43), GOLD[2])
    # корона: задняя половина обода, зубцы, передняя половина
    cy, rx, ry, band = 32, 13, 4.0, 4
    back = (ellipse(38, cy - band, rx, ry) <= 1) & (PY < cy - band)
    fill(img, back, ICE[3])
    crown = np.zeros((H, W), bool)
    spikes = [(26.5, 8), (32, 12), (38, 15), (44, 12), (49.5, 8)]
    for x, h in spikes:
        yb = cy - band - ry * math.sqrt(max(0.0, 1 - ((x - 38) / rx) ** 2)) * -1 + 1
        tri_l = poly([(x - 3.2, yb), (x, yb - h), (x, yb)])
        tri_r = poly([(x, yb), (x, yb - h), (x + 3.2, yb)])
        fill(img, tri_l, ICE[1])
        fill(img, tri_r, ICE[3])
        fill(img, seg((x - 3.2, yb), (x, yb - h), 0.5) & tri_l, ICE[0])
        crown |= tri_l | tri_r
        spark(img, int(x), int(yb - h - 1), "#ffffff", 0)
    front = ((ellipse(38, cy, rx, ry) <= 1) | (np.abs(PX - 38) <= rx) & (PY >= cy - band) & (PY <= cy)) & \
        ((PY >= cy - band + ry * np.sqrt(np.clip(1 - ((PX - 38) / rx) ** 2, 0, 1)) - ry) | (PY >= cy - band))
    front &= np.abs(PX - 38) <= rx
    front &= PY <= cy + ry * np.sqrt(np.clip(1 - ((PX - 38) / rx) ** 2, 0, 1))
    front &= PY >= cy - band + ry * np.sqrt(np.clip(1 - ((PX - 38) / rx) ** 2, 0, 1)) - 0.5
    fill(img, front, ICE[2])
    fill(img, front & (PX < 34), ICE[1])
    fill(img, front & (PX > 44), ICE[3])
    upper = front & ~np.roll(front, 1, 0)
    fill(img, upper, ICE[0])
    crown |= front | back
    for gx in (31, 38, 45):  # камни обода
        gy = int(cy - band / 2 + ry * math.sqrt(max(0.0, 1 - ((gx - 38) / rx) ** 2)))
        fill(img, (np.abs(XX - gx) + np.abs(YY - gy) <= 1) & front, CYAN[2])
        fill(img, (XX == gx) & (YY == gy - 1), CYAN[0])
    outline(img, crown, ICE_LINE)
    for gx, gy in ((12, 16), (64, 12), (59, 26), (17, 28)):
        for k in range(3):  # след всплывающего ромба
            fill(img, (XX == gx) & (YY == gy + 6 + k * 2), CYAN[3 if k else 2])
        gem(img, gx, gy)
    flakes(img, rng, 8, 0, 28, big=0.2)
    return img


def art_hard_winter(seed=45):
    """Холодная зима: метель и сосульки, чужие фигуры в глыбах трескаются от мороза."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 26
    sky(img, horizon, ["#0b0d15", "#10131d", "#181d2b"])
    ground = board(img, horizon, 38, "#4a6a90", ("#232a36", "#1c222c", "#2b3442"))
    snow_cover(img, ground, rng, 0.5, 5)
    for code, fx, fy, r, v in (("bP", 22, 41, 11, 0), ("bR", 54, 46, 12, 3)):
        p = piece(code, 0.6)
        place(img, p, fx, fy, shadow=False)
        body = ice_block(img, fx, fy, r, block_h(p, r), v, cracks=seed + fx)
        ys, xs = np.nonzero(body)
        for _ in range(5):  # красные сколы — урон
            a = rng.uniform(-math.pi, 0)
            x = int(xs.mean() + math.cos(a) * rng.uniform(r * 0.8, r * 1.3))
            y = int(ys.min() + 6 + math.sin(a) * rng.uniform(3, 8))
            if 0 <= x < W - 1 and 0 <= y < H - 1:
                chip = (XX >= x) & (XX <= x + 1) & (YY == y) | ((XX == x) & (YY == y + 1))
                fill(img, bc.outline4(chip), OUTLINE)
                fill(img, chip, BLOOD[1])
    # снег на кромке и сосульки сверху
    lump = burn.value_noise(H, W, 3, rng)
    eave = (YY <= 1) | ((YY == 2) & (lump > 0.5))
    x = 0
    icicles = np.zeros((H, W), bool)
    while x < W:
        w = int(rng.integers(3, 5))
        ln = int(rng.integers(4, 13))
        m = poly([(x, 1), (x + w, 1), (x + w / 2, 1 + ln)])
        fill(img, m, ICE[1])
        fill(img, m & (PX > x + w / 2), ICE[3])
        fill(img, m & (np.abs(PX - (x + w * 0.3)) < 0.6), ICE[0])
        icicles |= m
        x += w + int(rng.integers(1, 4))
    outline(img, icicles, ICE_LINE)
    fill(img, eave, SNOW[1])
    fill(img, (YY == 0), SNOW[0])
    fill(img, bc.outline4(eave) & (YY > 0) & ~icicles, SNOW[3])
    for _ in range(40):  # метель — косые хлопья
        x, y = int(rng.integers(0, W - 1)), int(rng.integers(3, H - 1))
        if rng.random() < 0.3:
            spark(img, x, y, SNOW[1])
            fill(img, (XX == x) & (YY == y), "#ffffff")
        else:
            fill(img, ((XX == x) & (YY == y)) | ((XX == x + 1) & (YY == y + 1)), SNOW[int(rng.integers(0, 2))])
    return img


# ---------------------------------------------------------------- карты победы
WARM_SKY = ["#121220", "#1a1824", "#30261a"]
WARM_BOARD = ("#2b2320", "#221c1b", "#33292a")


def aura(img, p, fx, fy, colors):
    """Сияние по силуэту фигуры: кольцо в пиксель и пунктир снаружи."""
    body, ax, ay = p
    m = np.zeros((H, W), bool)
    x0, y0 = int(fx - ax), int(fy - ay)
    h, w = body.shape[:2]
    for y, x in zip(*np.nonzero(body[..., 3] > 0)):
        if 0 <= y0 + y < H and 0 <= x0 + x < W:
            m[y0 + y, x0 + x] = True
    r1 = bc.outline4(m)
    r2 = bc.outline4(m | r1)
    fill(img, r1 & (PY < fy), colors[0])
    fill(img, r2 & (PY < fy - 2) & ((XX + YY) % 2 == 0), colors[1])
    return m


def art_champion(seed=51):
    """Один в поле воин: белый конь в столбе золотого света, вокруг — поверженные чужие пешки."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 26
    sky(img, horizon, WARM_SKY)
    ang = np.degrees(np.arctan2(PY - 18, PX - 38)) % 360
    rays = (YY < horizon + 6) & ((ang // 12) % 2 == 0) & (ellipse(38, 18, 42, 30) <= 1)
    tint(img, rays, "#5a4424", 0.45)
    ground = board(img, horizon, 40, "#5a4424", WARM_BOARD)
    d = diamond(38, 44, 19, 7)
    tint(img, (d <= 1) & ground & ((XX + YY) % 2 == 0), bl.LIGHT[3], 0.4)
    fill(img, (d > 0.88) & (d <= 1), bl.LIGHT[2])
    fill(img, (d > 0.95) & (d <= 1), bl.LIGHT[1])
    pillar = (np.abs(PX - 38) < 9 - (PY / H) * 2) & (PY < 44)
    tint(img, pillar & ((XX + YY) % 2 == 0), bl.LIGHT[1], 0.22)
    # враги кольцом вдали — тёмные силуэты, светятся только огни
    body, ax, ay = piece("bP", 0.4)
    dark = body.copy()
    rgb = dark[..., :3].astype(int)
    lit = (rgb[..., 0] > 180) & (rgb[..., 2] < 120)
    dark[~lit & (dark[..., 3] > 0), :3] = (rgb[~lit & (dark[..., 3] > 0)] * 0.45).astype(np.uint8)
    for fx, fy in ((8, 31), (20, 28), (56, 28), (68, 31), (4, 44), (72, 45)):
        place(img, (dark, ax, ay), fx, fy)
    knight = piece("wN", 0.64)
    place(img, knight, 38, 44)
    aura(img, knight, 38, 44, (bl.LIGHT[1], bl.LIGHT[2]))
    for _ in range(14):
        x, y = int(rng.integers(20, 58)), int(rng.integers(2, 40))
        if img[y, x, :3].sum() < 300:
            spark(img, x, y, bl.SPARK[int(rng.integers(0, 3))], int(rng.random() < 0.3))
    return img


def art_bastion(seed=52):
    """Оборонительная позиция: ладья и соседи под куполом щита, удары гаснут на нём."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 22
    sky(img, horizon, ["#10121c", "#151a26", "#222a38"])
    stars(img, rng, 8, 12)
    board(img, horizon, 36, "#33445c", WARM_BOARD)
    cx, cy, rx, ry = 38, 43, 34, 31
    foot = ellipse(cx, cy, rx, 9.5)
    tint(img, foot <= 1, CYAN[4], 0.35)
    fill(img, (np.abs(foot - 1) < 0.07) & (PY < cy), CYAN[3])
    rook, pawn = piece("wR", 0.6), piece("wP", 0.5)
    for p, fx, fy in sorted([(pawn, 18, 41), (pawn, 58, 41), (rook, 38, 44)], key=lambda f: f[2]):
        place(img, p, fx, fy)
    dome = ((ellipse(cx, cy, rx, ry) <= 1) & (PY <= cy)) | (foot <= 1)
    tint(img, dome, CYAN[2], 0.14)
    # соты: шестиугольная сетка по куполу
    q = (PX - cx) / 6.0
    r = (PY - cy) / 5.2
    hexl = (np.abs(((q + (np.floor(r) % 2) * 0.5) % 1) - 0.5) < 0.09) | (np.abs((r % 1) - 0.5) < 0.1)
    tint(img, dome & hexl & ((XX + YY) % 2 == 0), CYAN[1], 0.5)
    shell = (np.abs(ellipse(cx, cy, rx, ry) - 1) < 0.035) & (PY <= cy)
    fill(img, shell, CYAN[2])
    fill(img, shell & (PX < cx) & (PY < cy - 10), CYAN[0])
    fill(img, (np.abs(foot - 1) < 0.07) & (PY >= cy), CYAN[1])
    for (sx, sy), (ix, iy) in (((2, 2), (14, 21)), ((74, 4), (61, 17))):
        for k in range(3):  # летящий удар: штрих и вспышка на щите
            a, b = (sx + (ix - sx) * k / 3, sy + (iy - sy) * k / 3), (sx + (ix - sx) * (k + 0.6) / 3, sy + (iy - sy) * (k + 0.6) / 3)
            fill(img, seg(a, b, 0.5), GOLD[2] if k < 2 else GOLD[1])
        hit = ellipse(ix, iy, 4.5, 3.5) <= 1
        tint(img, hit & dome, CYAN[0], 0.55)
        spark(img, ix, iy, "#ffffff", 2)
        for dx, dy in ((-3, 2), (3, -2), (2, 3)):
            fill(img, (XX == ix + dx) & (YY == iy + dy), GOLD[1])
    return img


def art_barrier(seed=53):
    """Барьер: золотые стены квадрата 3×3 — своя фигура внутри, чужая разбивается о стену."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 18
    sky(img, horizon, ["#14121a", "#1a1722", "#2a2418"])
    stars(img, rng, 6, 10, (GOLD[0], GOLD[1]))
    board(img, horizon, 32, "#5a4424", WARM_BOARD)
    cx, cy, rx, ry, h = 37, 37, 25, 11, 17
    L, T, R, B = (cx - rx, cy), (cx, cy - ry), (cx + rx, cy), (cx, cy + ry)
    up = lambda p: (p[0], p[1] - h)  # noqa: E731
    floor = poly([L, T, R, B])
    tint(img, floor & ((XX + YY) % 2 == 0), GOLD[1], 0.25)

    def wall(a, b, k):
        m = poly([a, b, up(b), up(a)])
        tint(img, m, GOLD[1], k)
        band = m & (((YY - (PX - a[0]) * (b[1] - a[1]) / (b[0] - a[0])) % 4) < 1)
        tint(img, band & ((XX + YY) % 2 == 0), GOLD[0], 0.55)
        fill(img, seg(up(a), up(b), 0.55), GOLD[0])
        fill(img, seg(a, b, 0.55), GOLD[2])
        return m

    wall(L, T, 0.22)
    wall(T, R, 0.3)
    fill(img, seg(T, up(T), 0.55), GOLD[1])
    place(img, piece("wP", 0.62), cx, cy + 1)
    enemy = piece("bP", 0.62)
    place(img, enemy, 69, 48)
    wall(L, B, 0.26)
    wall(B, R, 0.34)
    for p in (L, B, R):
        fill(img, seg(p, up(p), 0.6), GOLD[0])
    ix, iy = 57, 38  # удар о стену
    spark(img, ix, iy, "#ffffff", 3)
    for k in range(6):
        a = math.radians(150 + k * 22 + rng.uniform(-8, 8))
        ln = rng.uniform(3, 6)
        fill(img, seg((ix + math.cos(a) * 2, iy + math.sin(a) * 2), (ix + math.cos(a) * ln, iy + math.sin(a) * ln), 0.5), GOLD[int(k % 2)])
    for dx, dy in ((5, -3), (7, 1), (4, 4)):
        fill(img, (XX == ix + dx) & (YY == iy + dy), BLOOD[1])
    for _ in range(10):
        x, y = int(rng.integers(cx - 18, cx + 18)), int(rng.integers(12, 36))
        if poly([up(L), up(T), up(R), R, B, L])[y, x]:
            fill(img, (XX == x) & (YY == y), GOLD[0])
    return img


def art_victory_tales(seed=54):
    """Истории побед: раскрытая книга с венком и записями о павших врагах, из неё всплывает мана."""
    img, rng = new(), np.random.default_rng(seed)
    sky(img, 30, ["#120d0e", "#1a1214", "#261a16"])
    table = YY >= 30
    fill(img, table, "#3a2418")
    fill(img, table & (YY % 4 == 1), "#2e1c13")
    fill(img, table & (YY % 4 == 3) & ((XX * 7 + YY * 3) % 11 < 4), "#4a2e1e")
    fill(img, YY == 30, "#5a3a26")
    tint(img, (ellipse(64, 22, 16, 14) <= 1) & ((XX + YY) % 2 == 0), GOLD[2], 0.18)
    # обложка, страницы, корешок
    cover = poly([(5, 27), (38, 31), (71, 27), (71, 45), (38, 48), (5, 45)])
    fill(img, bc.outline4(cover), OUTLINE)
    fill(img, cover, "#5c0c18")
    fill(img, cover & (PY > 44), "#3a0710")
    lp = poly([(7, 26), (37, 30), (37, 45), (7, 42)])
    rp = poly([(39, 30), (69, 26), (69, 42), (39, 45)])
    for pg, shade_right in ((lp, True), (rp, False)):
        fill(img, pg, "#efe6c6")
        fill(img, pg & ((PX > 33) if shade_right else (PX < 43)), "#d8cba4")
    fill(img, bc.outline4(lp | rp) & cover, "#a8956a")
    for k in range(1, 3):  # срез страниц снизу
        fill(img, seg((8, 42 + k), (37, 45 + k), 0.5) & cover, "#e1d6b3" if k == 1 else "#a8956a")
        fill(img, seg((39, 45 + k), (68, 42 + k), 0.5) & cover, "#e1d6b3" if k == 1 else "#a8956a")
    fill(img, seg((38, 30), (38, 46), 0.6), "#8a1426")
    # левая страница — венок победы, правая — строки и перечёркнутая чужая пешка
    small = np.zeros((bc.CH, bc.CW, 4), np.uint8)
    bc.emblem_wreath(small, bc.CARD_TYPES["victory"], bc.CARD_TYPES["victory"]["frame"], 0, 0)
    ys, xs = np.nonzero(small[..., 3])
    wreath = small[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
    put(img, wreath, 22 - wreath.shape[1] // 2, 34 - wreath.shape[0] // 2)
    for k, y in enumerate((33, 36, 39)):
        x0, x1 = 42, 64 - k * 3
        fill(img, (YY == y - (PX - 39) * 0.13 + 0.5).astype(bool) & (PX > x0) & (PX < x1) & (XX % 3 != 0), "#7a6a52")
    for k, y in enumerate((31, 34, 37)):
        fill(img, (np.abs(PY - (y + 1 - (PX - 7) * -0.13)) < 0.5) & (PX > 9) & (PX < 14) & (XX % 3 != 0), "#7a6a52")
    icon = bl.icon(fw.PAWN_ICON, BLACK_PAWN)
    put(img, icon, 57, 30)
    ih, iw = icon.shape[:2]
    for a, b in (((56, 30), (57 + iw, 30 + ih)), ((57 + iw, 30), (56, 30 + ih))):
        fill(img, seg(a, b, 0.55), "#a81a30")
    for _ in range(4):
        gem(img, int(rng.integers(14, 62)), int(rng.integers(5, 20)), 2.0, 2.8)
    for _ in range(16):  # искры маны над страницами
        x, y = int(rng.integers(10, 66)), int(rng.integers(2, 28))
        fill(img, (XX == x) & (YY == y), CYAN[int(rng.integers(0, 3))])
    candle(img, 66, 30, 8)
    return img


def frost_cracks(img, cx, cy, rng, n=7, ln=10):
    """Трещины инея по земле от основания: голубые ломаные, сплющенные по вертикали."""
    for k in range(n):
        a = 2 * math.pi * k / n + rng.uniform(-0.3, 0.3)
        x, y = cx, cy
        for _ in range(3):
            nx, ny = x + math.cos(a) * ln / 3, y + math.sin(a) * ln / 3 * 0.45
            fill(img, seg((x, y), (nx, ny), 0.5), ICE[1])
            x, y, a = nx, ny, a + rng.uniform(-0.5, 0.5)


def art_ice_warrior(seed=55):
    """Воин из льда: фигура изо льда, под ней трещины инея и шипы, вокруг вьётся снег."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 26
    sky(img, horizon, COLD_SKY)
    stars(img, rng, 10, 16)
    ground = board(img, horizon, 40, "#33445c", ("#232a36", "#1c222c", "#2b3442"))
    snow_cover(img, ground & ((PX < 16) | (PX > 60)), rng, 0.35)
    tint(img, ground & (ellipse(38, 44, 30, 8) <= 1) & ((XX + YY) % 2 == 0), ICE[3], 0.4)
    frost_cracks(img, 38, 44, rng, 8, 30)
    p = ice_body(piece("wB", 0.66))
    for x, y, r, hh, lean in ((25, 41, 2.6, 8, -2), (52, 41, 2.6, 9, 2)):
        shard(img, x, y, r, hh, lean)
    place(img, p, 38, 44)
    for x, y, r, hh, lean in ((29, 47, 2.4, 6, -1.5), (48, 48, 2.6, 7, 1.5), (19, 46, 2.0, 5, -1)):
        shard(img, x, y, r, hh, lean)
    aura(img, p, 38, 44, (ICE[1], ICE[2]))
    for k in range(18):  # снежный вихрь вокруг
        a = k * 0.62
        rr = 9 + k * 0.85
        x, y = int(38 + math.cos(a) * rr), int(26 + math.sin(a) * rr * 0.55)
        if 0 <= x < W and 0 <= y < H:
            fill(img, (XX == x) & (YY == y), SNOW[0] if k % 3 else ICE[1])
    return img


def aurora(img, yc, colors, amp=4.0, ln=(6, 12), phase=0.0):
    """Полярное сияние: занавес вдоль волнистой линии — яркая кромка сверху, книзу полосы
    тускнеют и редеют пунктиром; длина складок меняется плавно."""
    for x in range(W):
        y0 = yc + amp * math.sin(x * 0.11 + phase) + 1.8 * math.sin(x * 0.29 + phase * 2)
        n = int(ln[0] + (ln[1] - ln[0]) * (0.5 + 0.5 * math.sin(x * 0.23 + phase * 3)))
        for k in range(n):
            y = int(y0) + k
            if not 0 <= y < H:
                continue
            t = k / n
            if t > 0.5 and (x + y) % 2:
                continue
            if t > 0.8 and (x + y) % 4:
                continue
            img[y, x] = (*hexc(colors[min(int(t * len(colors)), len(colors) - 1)]), 255)


def art_north_warrior(seed=56):
    """Северный воин: белый ферзь под полярным сиянием, сила течёт к нему от замёрзших врагов."""
    img, rng = new(), np.random.default_rng(seed)
    horizon = 30
    sky(img, horizon, ["#070b16", "#0b1222", "#101a2e"])
    stars(img, rng, 12, 20)
    aurora(img, 14, ["#6a4cd0", "#4a3498", "#2e2470"], amp=3, ln=(3, 7), phase=2.0)
    aurora(img, 6, ["#c8ffd8", "#3ee08a", "#2ab8a8", "#1f6a78", "#174a5e"])
    hills = poly([(-2, 31), (8, 25), (20, 28), (30, 24), (44, 27), (56, 23), (68, 27), (78, 25), (78, 31)])
    fill(img, hills, "#16203a")
    fill(img, hills & ~np.roll(hills, 2, 0), SNOW[3])
    ground = board(img, horizon, 40, "#4a6a90", ("#232a36", "#1c222c", "#2b3442"))
    snow_cover(img, ground, rng, 0.6, 5)
    queen = piece("wQ", 0.6)
    chest = (38, 44 - queen[2] * 0.55)
    for fx, fy in ((12, 41), (64, 41)):
        p = piece("bP", 0.42)
        place(img, p, fx, fy, shadow=False)
        ice_block(img, fx, fy, 7, block_h(p, 7), 1 if fx < 38 else 3)
        sx, sy = fx, fy - p[2] - 2
        for k in range(1, 9):  # поток силы — пунктирная дуга к ферзю
            t = k / 9
            x = sx + (chest[0] - sx) * t
            y = sy + (chest[1] - sy) * t - math.sin(t * math.pi) * 8
            if k % 2:
                fill(img, (XX == int(x)) & (YY == int(y)), "#3ee08a" if k < 5 else CYAN[1])
    place(img, queen, 38, 45)
    aura(img, queen, 38, 45, (CYAN[1], CYAN[2]))
    for x, y in ((27, 14), (50, 12), (47, 22)):  # «плюсы» силы
        spark(img, x, y, CYAN[0], 1)
    flakes(img, rng, 10, 0, 30, big=0.2)
    return img


# ---------------------------------------------------------------- сборка
NORMAL = {
    "storm": art_storm, "avalanche": art_avalanche, "ice_barrier": art_ice_barrier,
    "multiply": art_multiply, "ice_wall": art_ice_wall, "mass_sacrifice": art_mass_sacrifice,
    "absolute_zero": art_absolute_zero, "ice_shard": art_ice_shard, "ice_blast": art_ice_blast,
}
SACRIFICE = {
    "offering": art_offering, "encircle": art_encircle, "blood_god": art_blood_god,
    "cold_ruler": art_cold_ruler, "hard_winter": art_hard_winter,
}
VICTORY = {
    "champion": art_champion, "bastion": art_bastion, "barrier": art_barrier,
    "victory_tales": art_victory_tales, "ice_warrior": art_ice_warrior, "north_warrior": art_north_warrior,
}
TITLES = {
    "storm": "Буря", "avalanche": "Лавина", "ice_barrier": "Ледяной барьер", "multiply": "Они множатся",
    "ice_wall": "Ледяная стена", "mass_sacrifice": "Массовое Жертвоприношение", "absolute_zero": "Абсолютный Ноль",
    "ice_shard": "Ледяной снаряд", "ice_blast": "Ледяной взрыв", "offering": "Жертва",
    "encircle": "Окружены и сломлены", "blood_god": "Кровь Богу Крови", "cold_ruler": "Хладнокровный правитель",
    "hard_winter": "Холодная зима", "champion": "Один в Поле Воин", "bastion": "Оборонительная позиция",
    "barrier": "Барьер", "victory_tales": "Истории Побед", "ice_warrior": "Воин из Льда",
    "north_warrior": "Северный воин",
}


def faces_of(name):
    if name in NORMAL:
        return ["w", "b"]
    return ["sacrifice"] if name in SACRIFICE else ["victory"]


def preview(arts, path, scale=2, cols=5):
    """Лист превью: каждая картинка на лице своего типа (обычные — на белом), подпись снизу."""
    gap, label = 8, 18
    names = list(arts)
    rows = (len(names) + cols - 1) // cols
    cw, ch = hd.CW, hd.CH
    sheet = Image.new("RGBA", ((cw + gap) * cols + gap, (ch + label + gap) * rows + gap), hexc("#1b1e27") + (255,))
    faces = {}
    for n, name in enumerate(names):
        kind = faces_of(name)[0]
        if kind not in faces:
            faces[kind] = hd.card_front(kind)
        x, y = gap + (n % cols) * (cw + gap), gap + (n // cols) * (ch + label + gap)
        sheet.alpha_composite(Image.fromarray(hd.with_art(faces[kind], arts[name]), "RGBA"), (x, y))
    big = sheet.resize((sheet.width * scale, sheet.height * scale), Image.NEAREST)
    draw = ImageDraw.Draw(big)
    font = ImageFont.truetype("segoeui.ttf", 15)
    for n, name in enumerate(names):
        x, y = gap + (n % cols) * (cw + gap), gap + (n // cols) * (ch + label + gap) + ch
        draw.text(((x + cw / 2) * scale, (y + 3) * scale), TITLES[name], fill=(220, 226, 236), font=font, anchor="mt")
    big.save(path)


def save(img, *path):
    full = os.path.join(OUT, *path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    Image.fromarray(img, "RGBA").save(full)


def main():
    only = os.environ.get("BC_ONLY")
    draw = {**NORMAL, **SACRIFICE, **VICTORY}
    if only:
        draw = {k: v for k, v in draw.items() if k in only.split(",")}
    arts = {name: f() for name, f in draw.items()}
    preview_path = os.environ.get("BC_PREVIEW") or os.path.join(OUT, "preview_card_arts.png")
    preview(arts, preview_path)
    if os.environ.get("BC_PREVIEW"):  # только превью — ничего не сохранять
        print("preview", preview_path)
        return
    for name, img in arts.items():
        save(img, "art", f"{name}.png")
    if "--no-burn" not in sys.argv:
        faces = {k: hd.card_front(k) for k in bc.CARD_KINDS}
        for name, img in arts.items():
            for kind in faces_of(name):
                strip = np.concatenate(hd.render_burn(bc.card_front(kind), hd.with_art(faces[kind], img)), axis=1)
                save(strip, "burn", f"burn_strip_{name}_{kind}.png")
                print("burn", name, kind)
    print("done", OUT)


if __name__ == "__main__":
    main()
