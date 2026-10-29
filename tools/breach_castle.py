"""Готический замок на доске — в стиле Into the Breach.

Стоит прямо на восьми клетках a3–a6 и b3–b6. История: собор Святого Вита (пользователю показался пластмассовым,
игрушечным), замок со стенами и донжоном, дворец буквой П, дворец по образцу Букингемского; сейчас — по рисунку-плану
пользователя: один длинный корпус под высокой готической кровлей, ворота посередине фасада к доске, круглые
башни на двух передних углах; цвет и фактура — как у замка до Букингемского дворца.

Корпус — три этажа стрельчатых окон, между ними контрфорсы с пинаклями; крутая черепичная кровля со стрельчатыми
слуховыми окнами (острый щипец, золотое навершие), гребнем по коньку и шпилем-фонарём с флажком; западный щипец с
большим окном с переплётом и пинаклями; надвратный ризалит — ворота с полуопущенной решёткой и створками, герб,
окна, крутой щипец с золотым навершием, две угловые башенки с конусами; передние башни — откос у основания,
машикули, зубчатый парапет на выносе, высокий шиферный конус с флажком; две закопчённые трубы на заднем скате.

Камень — бежевый песчаник с почти чёрными тенями (`STONE`). Чтобы не выглядело пластмассой: кладка — блоками
разного тона, у земли копоть, по стенам чёрные потёки (`masonry`), черепица и шифер рядами (`tiles`);
затенение в углах и у оснований (`ambient`, по расстоянию до тел вдоль нормали); тени от башен и контрфорсов на
стены (луч к свету `LIGHT_DIR`); несколько тёплых горящих окон.

Модель объёмная, рисуется лучами, как фигуры `breach_art.py` (масштаб `K` = 0.86 пикселя на единицу, сторона
клетки `L` = 46 единиц, свет `art.L_SHADE`, пять тонов на материал, контур #020201, швы на скачках глубины,
блик по левому и верхнему краю силуэта). Свои оси: u — с запада на восток (вдоль доски от 3-й горизонтали к
6-й, мир −z), v — с севера на юг (от вертикали a к b, мир +x), y — вверх; начало — общая вершина клеток a4,
a5, b4, b5. Тела: `box`, `prism` (кровля, щипцы), `frustum` (пирамиды пинаклей и наверший), `lathe` (башни,
конусы, шпиль — тела вращения; `solid=False` — замкнутый профиль-кольцо), `battlement`/`ring_battlement` (зубцы
и гребень — вычитанием периодических прорезей). Каждая деталь — `Part` с ограничивающим ящиком, `Scene`
проверяет деталь только рядом. Материал детали — `deco(основа, указания…)`: указания по точке и нормали меняют
материал (окна, бойницы, ворота, герб, переплёт) и сдвиг тона (фактура).

`py tools/breach_castle.py [--quick]` — в assets/breach/landmarks: castle.png и castle_shadow.png (тень на доске
тем же размером, как у фигур — чёрная с альфой 77); точка начала осей на спрайте печатается при запуске и
ставится на нижний угол ромба клетки a5, как опора фигуры — на центр клетки; превью preview_castle.png (пустая
доска). `--quick` — без сглаживания.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_ice_pieces as ip  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
OUT = os.path.join(ROOT, "landmarks")

K = 0.86  # пикселей на единицу — как у фигур
L = (art.TW / 2) / (K / math.sqrt(2))  # сторона клетки в единицах (46.05)


def local(w):
    """Мировой вектор (x, y, z) → свои оси (u, y, v): u = −z, v = x."""
    return np.array([-w[2], w[1], w[0]], float)


R_L, U_L, D_L = local(art.R), local(art.U), local(art.D)
LIGHT_DIR = local(art.L_SHADE)
LIGHT_DIR = LIGHT_DIR / np.linalg.norm(LIGHT_DIR)
SHADOW_DIR = local(art.norm(-0.5 * art.R + 2.4 * art.Y - 0.55 * art.T))  # тень на доску — круче, чем у фигур


def project(q):
    """Точки (N, 3) в своих осях → смещение на экране от точки начала (x вправо, y вниз)."""
    return np.stack([K * (q @ R_L), -K * (q @ U_L)], 1)


# ---------------------------------------------------------------- материалы и палитры
(STONE, TILE, SLATE, WOOD, IRON, WINDOW, LIT, DOOR, FLAG, GOLD, HERALD) = range(11)
PALETTE = {
    STONE: ["#0d0b09", "#3a3228", "#7a6a52", "#ab9878", "#d3c19e"],  # бежевый песчаник, тени — почти чёрные
    TILE: ["#1c0e0b", "#361a14", "#51261d", "#6c3427", "#874634"],  # старая черепица
    SLATE: ["#111318", "#1f232a", "#2f343d", "#434a55", "#5c6470"],  # шифер башен
    WOOD: ["#17100b", "#2a1d14", "#40301f", "#57422b", "#715739"],
    IRON: ["#0b0b0d", "#16161a", "#222228", "#313138", "#44444c"],
    WINDOW: ["#060608", "#0d0e12", "#15171d", "#1f222a", "#2b2f3a"],
    LIT: ["#3a1a06", "#7a3a0a", "#c06a16", "#e89a30", "#ffcf72"],  # окна со свечами
    DOOR: ["#0a0807", "#14100c", "#1e1812", "#2a2119", "#372b20"],
    FLAG: ["#2a0606", "#550c0c", "#821616", "#a82222", "#c83a2a"],
    GOLD: ["#4a2e08", "#7e5414", "#b88428", "#dcb04a", "#f4d88a"],
    HERALD: ["#0a1430", "#14275a", "#203e86", "#3358ac", "#4f78cc"],  # поле герба
}
FLAT = {IRON, WINDOW, LIT, DOOR, GOLD}  # без блика по краю силуэта
TEXTURED = {STONE}


def tone_of(l):
    """Свет → тон 0…4: верх — 4, западные стены — 3, южные (к доске) — 2."""
    return np.select([l > 0.72, l > 0.42, l > -0.2, l > -0.55], [4, 3, 2, 1], 0)


def hash2(a, b, seed):
    a, b = a.astype(np.int64), b.astype(np.int64)
    h = (a * 73856093) ^ (b * 19349663) ^ (seed * 83492791)
    h = (h ^ (h >> 13)) * 1274126177
    return ((h ^ (h >> 16)) & 0xFFFF) / 65535.0


# ---------------------------------------------------------------- тела
class Part:
    def __init__(self, f, lo, hi, mat):
        self.f, self.mat = f, mat
        self.lo, self.hi = np.asarray(lo, float), np.asarray(hi, float)


def box(u0, u1, y0, y1, v0, v1, mat, r=0.25):
    lo, hi = np.array([u0, y0, v0], float), np.array([u1, y1, v1], float)
    c, h = (lo + hi) / 2, (hi - lo) / 2 - r

    def f(p):
        q = np.abs(p - c) - h
        return np.linalg.norm(np.maximum(q, 0), axis=1) + np.minimum(q.max(axis=1), 0) - r

    return Part(f, lo, hi, mat)


def prism(poly, axis, b0, b1, mat):
    """Многоугольник в плоскости (v, y) вдоль u (axis = 'u') или в плоскости (u, y) вдоль v (axis = 'v')."""
    poly = np.asarray(poly, float)
    bc, hb = (b0 + b1) / 2, (b1 - b0) / 2
    a_i, b_i = (2, 0) if axis == "u" else (0, 2)

    def f(p):
        d2 = art.sd_polygon(p[:, a_i], p[:, 1], poly)
        w = np.stack([d2, np.abs(p[:, b_i] - bc) - hb], 1)
        return np.minimum(w.max(1), 0) + np.linalg.norm(np.maximum(w, 0), axis=1)

    if axis == "u":
        lo, hi = [b0, poly[:, 1].min(), poly[:, 0].min()], [b1, poly[:, 1].max(), poly[:, 0].max()]
    else:
        lo, hi = [poly[:, 0].min(), poly[:, 1].min(), b0], [poly[:, 0].max(), poly[:, 1].max(), b1]
    return Part(f, lo, hi, mat)


def lathe(cu, cv, profile, mat, solid=True):
    """Тело вращения: профиль (радиус, высота) снизу вверх; solid=False — профиль уже замкнут (кольцо)."""
    pts = [(0.0, profile[0][1])] + list(profile) + [(0.0, profile[-1][1])] if solid else list(profile)

    def f(p):
        return art.sd_polygon(np.hypot(p[:, 0] - cu, p[:, 2] - cv), p[:, 1], pts)

    rmax = max(r for r, _ in profile)
    ys = [y for _, y in profile]
    return Part(f, [cu - rmax, min(ys), cv - rmax], [cu + rmax, max(ys), cv + rmax], mat)


def ball(c, r, mat):
    c = np.asarray(c, float)
    return Part(lambda p: np.linalg.norm(p - c, axis=1) - r, c - r, c + r, mat)


# ---------------------------------------------------------------- узоры и фактура
def on(p, n, face):
    """Точки на грани и координата вдоль неё. face: '+v'/'-u'/… или (cu, cv, угол°) — круглая башня."""
    if isinstance(face, str):
        axis = 0 if face[1] == "u" else 2
        sign = 1 if face[0] == "+" else -1
        return n[:, axis] * sign > 0.7, (p[:, 2] if axis == 0 else p[:, 0])
    cu, cv, deg = face
    a = math.radians(deg)
    du, dv = p[:, 0] - cu, p[:, 2] - cv
    ang = np.arctan2(dv, du)
    d = ((ang - a + np.pi) % (2 * np.pi)) - np.pi
    return (np.abs(d) < 0.9) & (np.abs(n[:, 1]) < 0.6), d * np.hypot(du, dv)


def arch_mask(s, y, s0, w, y0, y1, pointed=True):
    """Арочный проём ширины w: прямоугольник до пяты, сверху полукруг или две дуги (стрельчатая)."""
    h = w / 2
    if pointed:
        spring = y1 - 0.866 * w
        top = (np.hypot(s - (s0 - h), y - spring) <= w) & (np.hypot(s - (s0 + h), y - spring) <= w)
    else:
        spring = y1 - h
        top = np.hypot(s - s0, y - spring) <= h
    inside = (np.abs(s - s0) <= h) & (y >= y0)
    return inside & ((y < spring) | top)


def herald(face, s0, yc, size):
    """Герб над воротами: синий щит с золотым поясом и обводом."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        ds, dy = (s - s0) / size, (p[:, 1] - yc) / size
        shield = (np.abs(ds) <= 1.0) & (dy <= 1.0) & (dy >= -1.4 + 0.8 * ds ** 2)
        inner = (np.abs(ds) <= 0.72) & (dy <= 0.78) & (dy >= -1.12 + 0.8 * ds ** 2)
        out[m & shield] = GOLD
        out[m & inner] = HERALD
        out[m & inner & (np.abs(dy - 0.05) < 0.22)] = GOLD
    return apply


def masonry(course=4.2, block=6.6, seed=1, grime=14.0):
    """Кладка: блоки рядами со сдвигом, часть темнее, часть светлее; у земли копоть, по стенам чёрные потёки."""
    def apply(p, n, out, off):
        vert = np.abs(n[:, 1]) < 0.55
        hl = np.hypot(n[:, 0], n[:, 2]) + 1e-6
        s = (-p[:, 0] * n[:, 2] + p[:, 2] * n[:, 0]) / hl
        row = np.floor(p[:, 1] / course)
        col = np.floor((s + (row % 2) * block / 2) / block)
        h = hash2(row, col, seed)
        o = np.where(h < 0.2, -1, np.where(h > 0.88, 1, 0))
        low = (p[:, 1] < grime) & (hash2(row, col, seed + 7) < 0.45)
        o = np.where(low, -1, o)
        strip = np.floor(s / 1.5)  # потёки копоти: узкие полосы разной длины сверху вниз
        top = 12 + 70 * hash2(strip, strip * 0 + 1, seed + 11)
        length = 6 + 26 * hash2(strip, strip * 0 + 2, seed + 12)
        soot = (hash2(strip, strip * 0 + 3, seed + 13) < 0.16) & (p[:, 1] < top) & (p[:, 1] > top - length)
        o = o - soot.astype(int)
        sel = vert & np.isin(out, list(TEXTURED))
        off[sel] += o[sel]
    return apply


def flagstones(seed=2):
    """Верх стен и площадки: крупные плиты разного тона."""
    def apply(p, n, out, off):
        top = n[:, 1] > 0.7
        h = hash2(np.floor(p[:, 0] / 3.4), np.floor(p[:, 2] / 3.4), seed)
        sel = top & np.isin(out, list(TEXTURED))
        off[sel] += np.where(h < 0.3, -1, 0)[sel]
    return apply


def tiles(row_h=2.7, tile_w=3.0, seed=3):
    """Черепица / шифер: ряды по высоте со сдвигом, тень под каждым рядом, часть плиток светлее и темнее."""
    def apply(p, n, out, off):
        hl = np.hypot(n[:, 0], n[:, 2]) + 1e-6
        s = (-p[:, 0] * n[:, 2] + p[:, 2] * n[:, 0]) / hl
        row = np.floor(p[:, 1] / row_h)
        col = np.floor((s + (row % 2) * tile_w / 2) / tile_w)
        h = hash2(row, col, seed)
        o = np.where(h < 0.16, -1, np.where(h > 0.87, 1, 0))
        o = np.where((p[:, 1] % row_h) < 0.55, -1, o)
        off += o
    return apply


def deco(base, *specs):
    def mat(p, n):
        out = np.full(len(p), base)
        off = np.zeros(len(p), int)
        for apply in specs:
            apply(p, n, out, off)
        return out, off
    return mat


# ---------------------------------------------------------------- замок
def frustum(cu, cv, y0, y1, r0, r1, mat, n=4):
    """Многогранник: апофема r0 внизу, r1 вверху (r1 = 0 — пирамида)."""
    h_ = y1 - y0
    s = math.hypot(h_, r1 - r0)
    ang = 2 * np.pi * np.arange(n) / n
    nrm = np.stack([np.cos(ang), np.sin(ang)], 1)

    def f(p):
        h = np.stack([p[:, 0] - cu, p[:, 2] - cv], 1) @ nrm.T
        d = (h * h_ - r0 * h_ - (r1 - r0) * (p[:, 1:2] - y0)) / s
        return np.maximum.reduce([d.max(1), y0 - p[:, 1], p[:, 1] - y1])

    big = max(r0, r1) / math.cos(math.pi / n)
    return Part(f, [cu - big, y0, cv - big], [cu + big, y1, cv + big], mat)


def battlement(u0, u1, y0, y1, v0, v1, axis, period, gap, mat):
    """Полоса с зубцами: ящик, из верхней половины вычтены прорези через `period` вдоль оси."""
    b = box(u0, u1, y0, y1, v0, v1, mat, r=0.2)
    cut = y0 + (y1 - y0) * 0.45
    i = 0 if axis == "u" else 2
    start = u0 if axis == "u" else v0

    def f(p):
        w = ((p[:, i] - start - period / 2) % period) - period / 2
        g = np.maximum(np.abs(w) - gap / 2, cut - p[:, 1])
        return np.maximum(b.f(p), -g)

    return Part(f, b.lo, b.hi, mat)


def ring_battlement(cu, cv, r_in, r_out, y0, y1, period, gap, mat):
    """Круглый парапет башни с зубцами (на выносе — над машикулями)."""
    n = max(4, int(round(2 * math.pi * r_out / period)))
    step = 2 * math.pi / n
    ring = lathe(cu, cv, [(r_in, y0), (r_out, y0), (r_out, y1), (r_in, y1)], mat, solid=False)
    cut = y0 + (y1 - y0) * 0.45

    def f(p):
        a = np.arctan2(p[:, 2] - cv, p[:, 0] - cu)
        w = (((a + step / 2) % step) - step / 2) * r_out
        g = np.maximum(np.abs(w) - gap / 2, cut - p[:, 1])
        return np.maximum(ring.f(p), -g)

    return Part(f, ring.lo, ring.hi, mat)


def windows(face, centres, w, y0, y1, mat=WINDOW, pointed=True, sill=None):
    """Окна: стрельчатые (или полукруглые) проёмы; mat — один материал или список по окнам."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        for k, s0 in enumerate(centres):
            mk = mat[k] if isinstance(mat, (list, tuple)) else mat
            out[m & arch_mask(s, p[:, 1], s0, w, y0, y1, pointed)] = mk
            if sill is not None:
                out[m & (np.abs(s - s0) <= w / 2 + 0.6) & (p[:, 1] >= y0 - 1.0) & (p[:, 1] < y0)] = sill
    return apply


def tracery(face, s0, w, y0, y1):
    """Большое стрельчатое окно с переплётом: средники и круг в навершии."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        inner = m & arch_mask(s, y, s0, w, y0, y1)
        out[inner] = WINDOW
        spring = y1 - 0.866 * w
        mull = (np.abs(np.abs(s - s0) - w / 6) < 0.55) & (y < spring + 1)
        ring = np.abs(np.hypot(s - s0, y - (spring + w * 0.33)) - w * 0.2) < 0.55
        out[inner & (mull | ring)] = STONE
        off[inner & (mull | ring)] += 1
    return apply


def slits(face, centres, y0, y1):
    """Бойницы — узкие тёмные щели с поперечиной."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        for s0 in centres:
            out[m & (np.abs(s - s0) < 0.75) & (y >= y0) & (y < y1)] = WINDOW
            out[m & (np.abs(s - s0) < 1.6) & (np.abs(y - (y0 + y1) / 2) < 0.7)] = WINDOW
    return apply


def ring_corbels(cu, cv, r, y0, y1, period):
    """Машикули башни: тёмные проёмы между консолями под выносом парапета."""
    def apply(p, n, out, off):
        a = np.arctan2(p[:, 2] - cv, p[:, 0] - cu)
        y = p[:, 1]
        side = np.abs(n[:, 1]) < 0.6
        out[side & (y >= y0) & (y < y1) & (((a * r) / period) % 1.0 < 0.45)] = DOOR
    return apply


def gate(face, s0, w, y0, y1):
    """Ворота: стрельчатый проём, полуопущенная решётка, за ней дубовые створки; светлый обвод."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        frame = m & arch_mask(s, y, s0, w + 2.4, y0, y1 + 1.2)
        off[frame] += 1
        hole = m & arch_mask(s, y, s0, w, y0, y1)
        out[hole] = DOOR
        planks = hole & (y < y0 + (y1 - y0) * 0.55) & (((s - s0) / 1.4) % 1.0 < 0.8)
        out[planks] = WOOD
        bars = hole & (y > y0 + (y1 - y0) * 0.3) & ((((s - s0) / 1.7) % 1.0 < 0.38) | (((y - y0) / 2.2) % 1.0 < 0.3))
        out[bars] = IRON
    return apply


def round_tower(cu, cv, r, top, roof, seed=0, faces=()):
    """Круглая башня: откос у основания, машикули, зубчатый парапет на выносе, высокий конус с флажком."""
    wall = deco(STONE, *faces, ring_corbels(cu, cv, r + 1.4, top - 4, top - 1, 2.6), masonry(seed=seed))
    tip = top + roof
    return [
        lathe(cu, cv, [(r + 2.6, 0), (r + 0.4, 14), (r, 18), (r, top)], wall),
        lathe(cu, cv, [(r - 0.5, top - 4.5), (r + 1.6, top - 1), (r + 1.6, top + 0.5), (r - 0.5, top + 0.5)],
              deco(STONE, ring_corbels(cu, cv, r + 1.6, top - 4.5, top - 1, 2.6), masonry(seed=seed + 1))),
        ring_battlement(cu, cv, r - 0.6, r + 1.6, top + 0.5, top + 6.5, 4.6, 2.0,
                        deco(STONE, masonry(seed=seed + 2), flagstones(seed + 3))),
        lathe(cu, cv, [(r + 0.2, top + 2.5), (r - 0.4, top + 4.0), (0.0, tip)], deco(SLATE, tiles(2.4, 2.6, seed + 4))),
        box(cu - 0.35, cu + 0.35, tip - 1, tip + 9, cv - 0.35, cv + 0.35, IRON, r=0.1),
        ball((cu, tip + 0.2, cv), 0.9, GOLD),
        box(cu - 0.3, cu + 0.3, tip + 5.2, tip + 8.8, cv + 0.3, cv + 5.8, FLAG, r=0.1),
    ]


def pinnacle(cu, cv, y0, half=1.8, height=10.0, mat=STONE, seed=0):
    """Пинакль: короткий столбик и острая четырёхгранная пирамидка."""
    return [box(cu - half, cu + half, y0, y0 + 3, cv - half, cv + half, deco(mat, masonry(1.6, 2.0, seed=seed))),
            frustum(cu, cv, y0 + 3, y0 + 3 + height, half + 0.3, 0.0, deco(mat, masonry(2.0, 2.0, seed=seed + 1)))]


def lighten(k=1):
    def apply(p, n, out, off):
        off += k
    return apply


EAVES = 52.0  # верх стен корпуса
FRONT_V = 24.0  # плоскость фасада к доске
BACK_V = -40.0
RIDGE_V, RIDGE_Y = -8.0, 103.0  # конёк готической кровли


def roof_y(v):
    """Высота переднего ската кровли над плоскостью v (для слуховых окон)."""
    return EAVES + (FRONT_V + 2 - v) * (RIDGE_Y - EAVES) / (FRONT_V + 2 - RIDGE_V)


def gothic_dormer(uc, seed):
    """Слуховое окно на переднем скате: стрельчатое окно, крутой щипец, шпиль-навершие."""
    front = FRONT_V - 6.0
    y0 = roof_y(front) - 4.0
    return [
        box(uc - 3.6, uc + 3.6, y0, y0 + 12, front - 9, front, deco(STONE, windows("+v", [uc], 2.8, y0 + 1.5, y0 + 10.5,
                                                                                [LIT if seed % 3 == 0 else WINDOW]),
                                                                    masonry(2.4, 2.6, seed=seed))),
        prism([(uc - 4.4, y0 + 11.5), (uc + 4.4, y0 + 11.5), (uc, y0 + 21)], "v", front - 9.6, front + 0.6,
              deco(TILE, tiles(2.0, 2.2, seed + 1))),
        frustum(uc, front + 0.2, y0 + 19, y0 + 26, 0.7, 0.0, GOLD),
    ]


def build():
    groups = []

    # длинный корпус: три этажа стрельчатых окон, контрфорсы с пинаклями между ними
    bays = [-58, -42, -26, 26, 42, 58]
    lit1, lit2 = [WINDOW, LIT, WINDOW, WINDOW, WINDOW, LIT], [WINDOW, WINDOW, LIT, WINDOW, WINDOW, WINDOW]
    hall = deco(STONE,
                windows("+v", bays, 3.0, 6, 14),
                windows("+v", bays, 4.6, 20, 37, lit1, sill=STONE),
                windows("+v", bays, 3.6, 41, 49, lit2),
                windows("-u", [-32, -20, -8, 3], 3.0, 6, 14),
                windows("-u", [-32, -20, -8, 3], 4.4, 20, 37, [WINDOW, WINDOW, LIT, WINDOW], sill=STONE),
                windows("-u", [-32, -20, -8, 3], 3.4, 41, 49),
                masonry(seed=111), flagstones(112))
    groups.append([
        box(-78, 78, 0, EAVES, BACK_V, FRONT_V, hall),
        prism([(BACK_V - 2, EAVES), (FRONT_V + 2, EAVES), (RIDGE_V, RIDGE_Y)], "u", -78.5, 78.5,
              deco(TILE, tiles(2.7, 3.0, 113))),
        battlement(-76, 76, RIDGE_Y - 1.5, RIDGE_Y + 3.0, RIDGE_V - 0.6, RIDGE_V + 0.6, "u", 3.4, 1.6,
                   deco(TILE, lighten(-1))),
    ])
    for k, u in enumerate((-66, -50, -34, -18, 18, 34, 50, 66)):
        groups.append([box(u - 1.6, u + 1.6, 0, EAVES - 4, FRONT_V, FRONT_V + 3.4, deco(STONE, masonry(2.4, 2.6, seed=120 + k)))]
                      + pinnacle(u, FRONT_V + 1.7, EAVES - 4, 1.5, 6.5, seed=130 + k))
    for k, uc in enumerate((-58, -42, -26, 26, 42, 58)):
        groups.append(gothic_dormer(uc, 140 + 3 * k))

    # западный щипец: стена над карнизом с большим окном с переплётом, пинакли по углам и на вершине
    gable = deco(STONE, tracery("-u", RIDGE_V, 14, 58, 92), masonry(seed=151))
    groups.append([prism([(BACK_V, EAVES), (FRONT_V, EAVES), (RIDGE_V, RIDGE_Y - 1)], "u", -79.4, -77.6, gable)]
                  + pinnacle(-78.5, BACK_V + 1.5, EAVES, 1.8, 10, seed=152)
                  + pinnacle(-78.5, RIDGE_V, RIDGE_Y - 2, 1.6, 12, seed=154))

    # шпиль-фонарь на коньке над воротами
    groups.append([
        lathe(0, RIDGE_V, [(3.4, RIDGE_Y - 6), (3.4, RIDGE_Y + 14), (4.2, RIDGE_Y + 15), (4.2, RIDGE_Y + 16.5),
                           (3.2, RIDGE_Y + 17), (0.0, RIDGE_Y + 44)],
              deco(SLATE, tiles(2.2, 2.2, 161), lantern_slots(0, RIDGE_V, RIDGE_Y + 4, RIDGE_Y + 12, 0.8))),
        ball((0, RIDGE_Y + 44.3, RIDGE_V), 0.9, GOLD),
        box(-0.35, 0.35, RIDGE_Y + 43, RIDGE_Y + 53, RIDGE_V - 0.35, RIDGE_V + 0.35, IRON, r=0.1),
        box(-0.3, 0.3, RIDGE_Y + 48.5, RIDGE_Y + 52.5, RIDGE_V + 0.35, RIDGE_V + 6.5, FLAG, r=0.1),
    ])

    # надвратный ризалит: ворота с решёткой, герб, окна, крутой щипец, угловые башенки
    gh = deco(STONE, gate("+v", 0, 11, 0.5, 30), herald("+v", 0, 37.5, 3.2),
              windows("+v", [-5, 5], 3.0, 44, 54, [LIT, WINDOW]), masonry(seed=171), flagstones(172))
    gv0, gv1 = FRONT_V - 4, FRONT_V + 7
    g = [
        box(-12, 12, 0, 62, gv0, gv1, gh),
        prism([(-13, 61.5), (13, 61.5), (0, 84)], "v", gv0 - 1, gv1 + 0.6, deco(TILE, tiles(2.4, 2.6, 173))),
        prism([(-12, 62), (12, 62), (0, 82.5)], "v", gv1 - 0.6, gv1, deco(STONE, masonry(seed=174),
                                                                      windows("+v", [0], 3.0, 66, 76))),
        frustum(0, gv1 - 0.3, 81.5, 89, 0.8, 0.0, GOLD),
    ]
    for cu, sd in ((-12, 175), (12, 177)):
        g += [lathe(cu, gv1 - 0.5, [(0.4, 42), (2.9, 46), (2.9, 64)], deco(STONE, masonry(1.8, 2.2, seed=sd))),
              lathe(cu, gv1 - 0.5, [(3.4, 63.5), (3.0, 65), (0.0, 78)], deco(SLATE, tiles(2.0, 2.0, sd + 1))),
              ball((cu, 78.3, gv1 - 0.5), 0.6, GOLD)]
    groups.append(g)

    # круглые башни на передних углах
    groups.append(round_tower(-78, FRONT_V, 11.0, 76, 42, seed=21,
                              faces=(slits((-78, FRONT_V, 180), [0], 26, 33), slits((-78, FRONT_V, 90), [0], 26, 33),
                                     slits((-78, FRONT_V, 135), [0], 48, 55))))
    groups.append(round_tower(78, FRONT_V, 11.0, 76, 42, seed=41,
                              faces=(slits((78, FRONT_V, 90), [0], 26, 33), slits((78, FRONT_V, 45), [0], 48, 55))))

    # трубы на заднем скате
    groups.append([box(cu - 1.8, cu + 1.8, 76, 92, -26, -22, deco(STONE, masonry(2.2, 2.4, seed=180 + k), lighten(-1)))
                   for k, cu in enumerate((-44, 44))])
    return groups


def lantern_slots(cu, cv, y0, y1, half):
    """Проёмы фонаря: тёмная полоса посередине каждой из восьми сторон."""
    def apply(p, n, out, off):
        du, dv = p[:, 0] - cu, p[:, 2] - cv
        a = np.arctan2(dv, du)
        k = np.round(a / (np.pi / 4)) * (np.pi / 4)
        s = -du * np.sin(k) + dv * np.cos(k)
        y = p[:, 1]
        side = np.abs(n[:, 1]) < 0.5
        out[side & (y >= y0) & (y < y1) & (np.abs(s) < half)] = DOOR
    return apply


# ---------------------------------------------------------------- лучи
def aabb_dist(p, lo, hi):
    q = np.maximum(lo - p, p - hi)
    return np.linalg.norm(np.maximum(q, 0), axis=1) + np.minimum(q.max(axis=1), 0)


class Scene:
    def __init__(self, groups):
        self.parts = [part for g in groups for part in g]
        self.groups = []
        k = 0
        for g in groups:
            lo = np.min([part.lo for part in g], axis=0) - 0.5
            hi = np.max([part.hi for part in g], axis=0) + 0.5
            self.groups.append((lo, hi, g, k))
            k += len(g)
        self.lo = np.min([g[0] for g in self.groups], axis=0)
        self.hi = np.max([g[1] for g in self.groups], axis=0)

    def query(self, p, want_index=False):
        d = np.full(len(p), 1e9)
        idx = np.full(len(p), -1) if want_index else None
        for lo, hi, parts, base in self.groups:
            sel = np.nonzero(aabb_dist(p, lo, hi) < d)[0]
            if not len(sel):
                continue
            ps, ds = p[sel], d[sel]
            ids = idx[sel] if want_index else None
            for k, part in enumerate(parts):
                s2 = np.nonzero(aabb_dist(ps, part.lo - 0.3, part.hi + 0.3) < ds)[0]
                if not len(s2):
                    continue
                dv = part.f(ps[s2])
                better = dv < ds[s2]
                ds[s2[better]] = dv[better]
                if want_index:
                    ids[s2[better]] = base + k
            d[sel] = ds
            if want_index:
                idx[sel] = ids
        return (d, idx) if want_index else d

    def march(self, o, d, t0, t1, steps=320, eps=0.02):
        t = t0.copy()
        hit = np.zeros(len(o), bool)
        alive = t < t1
        for _ in range(steps):
            idx = np.nonzero(alive)[0]
            if not len(idx):
                break
            p = o[idx] + d * t[idx, None]
            dist = self.query(p)
            h = dist < eps
            hit[idx[h]] = True
            alive[idx[h]] = False
            t[idx] += np.maximum(dist, 0.01)
            gone = t[idx] > t1[idx]
            alive[idx[gone]] = False
        return hit, t

    def normals(self, p):
        e = 0.05
        n = np.zeros_like(p)
        for k in range(3):
            dv = np.zeros(3)
            dv[k] = e
            n[:, k] = self.query(p + dv) - self.query(p - dv)
        return n / (np.linalg.norm(n, axis=1, keepdims=True) + 1e-9)

    def materials(self, p, n):
        _, idx = self.query(p, want_index=True)
        mat = np.zeros(len(p), int)
        off = np.zeros(len(p), int)
        for k in np.unique(idx):
            sel = idx == k
            m = self.parts[k].mat
            if callable(m):
                mat[sel], off[sel] = m(p[sel], n[sel])
            else:
                mat[sel] = m
        return mat, off

    def ambient(self, p, n):
        """Затенение: насколько близко другие тела вдоль нормали (углы, основания, под выносами)."""
        ao = np.zeros(len(p))
        for k, w in ((1, 0.4), (2, 0.3), (3, 0.2), (4, 0.1)):
            h = 1.6 * k
            ao += w * np.clip((h - self.query(p + n * h)) / h, 0, 1)
        return ao


def slab(o, d, lo, hi):
    """Вход и выход луча в ящик (направление без нулевых составляющих)."""
    ta, tb = (lo - o) / d, (hi - o) / d
    return np.minimum(ta, tb).max(1), np.maximum(ta, tb).min(1)


def shadowed(scene, start, direction, t_max=400.0):
    t0, t1 = slab(start, direction, scene.lo - 1, scene.hi + 1)
    t1 = np.minimum(t1, t_max)
    miss = t0 > t1
    t0 = np.where(miss, 1.0, np.maximum(t0, 0.0))
    t1 = np.where(miss, 0.0, t1)
    hit, _ = scene.march(start, direction, t0, t1, steps=200, eps=0.05)
    return hit


def canvas_frame(scene):
    lo, hi = scene.lo, scene.hi
    corners = np.array([[x, y, z] for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])])
    ground = corners - SHADOW_DIR * (corners[:, 1:2] / SHADOW_DIR[1])
    xy = project(np.concatenate([corners, ground]))
    ax = int(math.ceil(-xy[:, 0].min())) + 4
    ay = int(math.ceil(-xy[:, 1].min())) + 4
    return int(math.ceil(xy[:, 0].max())) + ax + 4, int(math.ceil(xy[:, 1].max())) + ay + 4, ax, ay


def rays(w, h, ax, ay, ss):
    xs = (np.arange(w * ss) + 0.5) / ss - ax
    ys = (np.arange(h * ss) + 0.5) / ss - ay
    gx, gy = np.meshgrid(xs, ys)
    gx, gy = gx.ravel() / K, gy.ravel() / K
    return gx[:, None] * R_L - gy[:, None] * U_L + D_L * 800.0


def render(scene, ss=3):
    w, h, ax, ay = canvas_frame(scene)
    o = rays(w, h, ax, ay, ss)
    d = -D_L
    t0, t1 = slab(o, d, scene.lo - 1, scene.hi + 1)
    miss = t0 > t1
    t0 = np.where(miss, 1.0, np.maximum(t0, 0.0))
    t1 = np.where(miss, 0.0, t1)
    hit, t = scene.march(o, d, t0, t1)
    print("hits", int(hit.sum()), "of", len(hit), flush=True)
    hp = o[hit] + d * t[hit, None]
    nrm = scene.normals(hp)
    mat_h, off_h = scene.materials(hp, nrm)
    l = nrm @ LIGHT_DIR
    tn = tone_of(l)
    lit_side = l > 0.05
    shade = np.zeros(len(hp), bool)
    shade[lit_side] = shadowed(scene, hp[lit_side] + nrm[lit_side] * 0.8, LIGHT_DIR)
    tn = np.where(shade, np.maximum(tn - 2, 1), tn)
    ao = scene.ambient(hp, nrm)
    tn = tn - (ao > 0.32).astype(int) - (ao > 0.62).astype(int)
    tn = np.clip(tn + off_h, 0, 4)
    tn = np.where(mat_h == GOLD, np.clip(tn + 1, 2, 4), tn)
    tn = np.where(mat_h == LIT, np.where(l > -0.3, 4, 3), tn)
    tn = np.where(np.isin(mat_h, [WINDOW, IRON]), np.minimum(tn, 2), tn)
    tn = np.where(mat_h == DOOR, np.minimum(tn, 1), tn)
    tone = np.zeros(len(o), int)
    mat = np.zeros(len(o), int)
    tone[hit], mat[hit] = tn, mat_h

    n_s = ss * ss
    hit_s = hit.reshape(h, ss, w, ss).transpose(0, 2, 1, 3).reshape(h, w, n_s)
    tone_s = tone.reshape(h, ss, w, ss).transpose(0, 2, 1, 3).reshape(h, w, n_s)
    mat_s = mat.reshape(h, ss, w, ss).transpose(0, 2, 1, 3).reshape(h, w, n_s)
    t_s = t.reshape(h, ss, w, ss).transpose(0, 2, 1, 3).reshape(h, w, n_s)
    solid = hit_s.sum(2) >= (n_s + 1) // 2
    centre = n_s // 2
    tone_px = np.zeros((h, w), int)
    mat_px = np.zeros((h, w), int)
    depth = np.full((h, w), np.inf)
    for y, x in zip(*np.nonzero(solid)):
        hs = hit_s[y, x]
        if hs[centre]:
            m = mat_s[y, x, centre]
            tone_px[y, x] = tone_s[y, x, centre]
        else:
            vals, cnt = np.unique(mat_s[y, x][hs], return_counts=True)
            m = vals[cnt.argmax()]
            tone_px[y, x] = int(round(np.median(tone_s[y, x][hs & (mat_s[y, x] == m)])))
        mat_px[y, x] = m
        depth[y, x] = np.median(t_s[y, x][hs]) * K
    left_open = solid & ~np.roll(solid, 1, 1)
    top_open = solid & ~np.roll(solid, 1, 0)
    rim = (left_open | top_open) & ~np.isin(mat_px, list(FLAT)) & (tone_px >= 2)
    tone_px[rim] = np.minimum(tone_px[rim] + 1, 4)
    pal = {k: [art.hexc(c) for c in v] for k, v in PALETTE.items()}
    img = np.zeros((h, w, 4), np.uint8)
    for y, x in zip(*np.nonzero(solid)):
        img[y, x, :3] = pal[mat_px[y, x]][tone_px[y, x]]
        img[y, x, 3] = 255
    seam = np.zeros((h, w), bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        nb = np.roll(np.roll(depth, dy, 0), dx, 1)
        with np.errstate(invalid="ignore"):
            seam |= solid & np.isfinite(nb) & (depth - nb > 3.2)
    img[seam, :3] = art.OUTLINE
    out = np.zeros((h, w), bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        out |= np.roll(np.roll(solid, dy, 0), dx, 1)
    out &= ~solid
    img[out, :3] = art.OUTLINE
    img[out, 3] = 255

    # тень на доску (по центрам пикселей)
    og = rays(w, h, ax, ay, 1)
    tg = og[:, 1] / D_L[1]
    ground = og - D_L * tg[:, None] + np.array([0, 0.05, 0])
    sh = shadowed(scene, ground, SHADOW_DIR / np.linalg.norm(SHADOW_DIR))
    shadow = np.zeros((h, w, 4), np.uint8)
    shadow[sh.reshape(h, w)] = art.SHADOW
    return Image.fromarray(img, "RGBA"), Image.fromarray(shadow, "RGBA"), (ax, ay)


# ---------------------------------------------------------------- превью на доске
MARGINS = dict(margin_x=40, margin_top=70, margin_bottom=60)


def preview(body, shadow, anchor):
    """Пустая доска `art.board_preview` и замок: начало осей — нижний угол ромба a5."""
    empty = Image.new("RGBA", (art.W, art.H), (0, 0, 0, 0))
    canvas = art.board_preview({c + k: (empty, empty) for c in "wb" for k in "PNBRQK"}, scale=1, demo=False,
                               **MARGINS)
    ox = MARGINS["margin_x"] + 8 * art.TW // 2 - art.TW // 2
    oy = MARGINS["margin_top"]
    at = (ox - 56 - anchor[0], oy + 105 - anchor[1])
    canvas.alpha_composite(shadow, at)
    canvas.alpha_composite(body, at)
    return canvas.resize((canvas.width * 2, canvas.height * 2), Image.NEAREST)


def main():
    quick = "--quick" in sys.argv
    os.makedirs(OUT, exist_ok=True)
    scene = Scene(build())
    body, shadow, anchor = render(scene, ss=1 if quick else 3)
    suffix = "_quick" if quick else ""
    body.save(os.path.join(OUT, f"castle{suffix}.png"))
    shadow.save(os.path.join(OUT, f"castle_shadow{suffix}.png"))
    print("anchor", anchor, "size", body.size, flush=True)
    preview(body, shadow, anchor).save(os.path.join(OUT, f"preview_castle{suffix}.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
