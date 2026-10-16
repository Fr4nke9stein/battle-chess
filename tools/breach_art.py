"""Пиксель-арт доски и фигур в духе Into the Breach (свои спрайты, не из игры).

Фигуры — маленькие 3D-модели (SDF), которые рендерятся лучами в низком разрешении и
квантуются в палитру из 5 тонов: контур 1px, свет сверху-слева, тень 30% вправо-вниз.
Белые — светлые мехи с голубыми огнями, чёрные — веки: фиолетовый панцирь и хитин.
Клетки — изометрические блоки 56×42 (как в ITB) со «слоистыми» боками.

Нужны Pillow и numpy: `py -m pip install pillow numpy`.
`py tools/breach_art.py [папка]` — по умолчанию пишет в assets/breach.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "assets", "breach")


def hexc(h):
    h = h.lstrip("#")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4))


# ---------------------------------------------------------------- камера
SIN_E = 0.75  # ромб клетки 56×42
COS_E = math.sqrt(1 - SIN_E**2)
A = COS_E / math.sqrt(2)
D = np.array([A, SIN_E, A])  # от сцены к камере
R = np.array([1.0, 0.0, -1.0]) / math.sqrt(2)  # вправо по экрану
U = np.cross(D, R)  # вверх по экрану
T = np.array([1.0, 0.0, 1.0]) / math.sqrt(2)  # к зрителю по горизонтали
Y = np.array([0.0, 1.0, 0.0])


def norm(v):
    v = np.asarray(v, float)
    return v / np.linalg.norm(v)


L_SHADE = norm(-0.62 * R + 0.95 * Y + 0.42 * T)
L_SHADOW = norm(-0.5 * R + 1.0 * Y - 0.55 * T)


# ---------------------------------------------------------------- SDF
def sd_polygon(px, py, verts):
    v = np.asarray(verts, float)
    keep = np.ones(len(v), bool)
    keep[1:] = np.any(np.abs(np.diff(v, axis=0)) > 1e-9, axis=1)
    if np.allclose(v[0], v[-1]):
        keep[-1] = False
    v = v[keep]
    d = (px - v[0, 0]) ** 2 + (py - v[0, 1]) ** 2
    s = np.ones_like(px)
    n = len(v)
    j = n - 1
    for i in range(n):
        ex, ey = v[j, 0] - v[i, 0], v[j, 1] - v[i, 1]
        wx, wy = px - v[i, 0], py - v[i, 1]
        t = np.clip((wx * ex + wy * ey) / (ex * ex + ey * ey), 0.0, 1.0)
        bx, by = wx - ex * t, wy - ey * t
        d = np.minimum(d, bx * bx + by * by)
        c1 = py >= v[i, 1]
        c2 = py < v[j, 1]
        c3 = ex * wy > ey * wx
        flip = (c1 & c2 & c3) | (~c1 & ~c2 & ~c3)
        s = np.where(flip, -s, s)
        j = i
    return s * np.sqrt(d)


def sd_segment2(px, py, a, b):
    ax, ay = a
    bx, by = b
    ex, ey = bx - ax, by - ay
    t = np.clip(((px - ax) * ex + (py - ay) * ey) / (ex * ex + ey * ey), 0, 1)
    return np.hypot(px - ax - ex * t, py - ay - ey * t)


def arc(cx, cy, r, a0, a1, n=10):
    return [
        (cx + r * math.cos(math.radians(a)), cy + r * math.sin(math.radians(a)))
        for a in np.linspace(a0, a1, n)
    ]


def lathe(profile):
    pts = [(0.0, profile[0][1])] + list(profile) + [(0.0, profile[-1][1])]

    def f(p):
        r = np.hypot(p[:, 0], p[:, 2])
        return sd_polygon(r, p[:, 1], pts)

    return f


def sphere(c, r):
    c = np.asarray(c, float)

    def f(p):
        return np.linalg.norm(p - c, axis=1) - r

    return f


def rbox(c, half, axes, rad=0.4):
    """Скруглённый параллелепипед: центр, полуразмеры по осям `axes` (3 вектора)."""
    c = np.asarray(c, float)
    half = np.asarray(half, float) - rad
    ax = np.asarray(axes, float)

    def f(p):
        q = np.abs((p - c) @ ax.T) - half
        return np.linalg.norm(np.maximum(q, 0), axis=1) + np.minimum(q.max(axis=1), 0) - rad

    return f


def extrude(poly, fwd, side, half_w, rad=1.2):
    fwd, side = norm(fwd), norm(side)

    def f(p):
        u = p @ fwd
        y = p[:, 1]
        v = p @ side
        d2 = sd_polygon(u, y, poly) + rad
        w = np.stack([d2, np.abs(v) - (half_w - rad)], axis=1)
        return np.minimum(w.max(axis=1), 0) + np.linalg.norm(np.maximum(w, 0), axis=1) - rad

    return f


def cut(f, g):
    return lambda p: np.maximum(f(p), -g(p))


MAIN, TRIM, GLOW, SEAM, DEEP = 0, 1, 2, 3, 4
# Фигуры чуть вытянуты вверх: сверху под углом ITB классические силуэты сплющиваются.
STRETCH = np.array([1.0, 1.32, 1.0])


class Piece:
    def __init__(self, parts, stretch=None):
        # parts: [(sdf, material(p) -> массив id)]
        self.parts = parts
        self.stretch = STRETCH if stretch is None else np.asarray(stretch, float)

    def dist(self, p):
        p = p / self.stretch
        return np.min(np.stack([f(p) for f, _ in self.parts]), axis=0)

    def material(self, p):
        p = p / self.stretch
        ds = np.stack([f(p) for f, _ in self.parts])
        which = ds.argmin(axis=0)
        mat = np.zeros(len(p), int)
        for k, (_, m) in enumerate(self.parts):
            sel = which == k
            if sel.any():
                mat[sel] = m(p[sel]) if callable(m) else m
        return mat


def bands(*ranges, inside=TRIM, outside=MAIN):
    def m(p):
        y = p[:, 1]
        out = np.full(len(p), outside)
        for lo, hi in ranges:
            out[(y >= lo) & (y < hi)] = inside
        return out

    return m


# ---------------------------------------------------------------- фигуры
def base(r0):
    """Ступенчатое основание, общий для всех фигур профиль."""
    return [
        (r0, 0.0),
        (r0, 2.0),
        (r0 - 0.8, 3.0),
        (r0 - 2.2, 3.0),
        (r0 - 2.2, 5.6),
        (r0 - 3.2, 6.6),
    ]


BASE_BANDS = (3.0, 6.7)


def front_mask(p, cos_min):
    h = p[:, [0, 2]]
    r = np.linalg.norm(h, axis=1) + 1e-6
    return (h @ T[[0, 2]]) / r > cos_min


def pawn():
    prof = base(13.5) + [
        (8.6, 7.6),
        (6.6, 10.0),
        (5.6, 13.4),
        (8.6, 13.9),
        (8.6, 15.8),
        (6.0, 16.6),
    ] + arc(0, 23.0, 7.4, -62, 90)

    def mat(p):
        m = bands(BASE_BANDS, (13.0, 16.7))(p)
        visor = (np.abs(p[:, 1] - 23.4) < 1.25) & front_mask(p, 0.55)
        m[visor] = GLOW
        return m

    return Piece([(lathe(prof), mat)])


def rook():
    prof = base(14.0) + [
        (9.8, 8.0),
        (9.4, 10.0),
        (8.8, 26.5),
        (11.4, 27.6),
        (12.0, 29.0),
        (12.0, 37.5),
    ]
    body = lathe(prof)
    hollow = lambda p: np.maximum(np.hypot(p[:, 0], p[:, 2]) - 8.6, 34.0 - p[:, 1])  # noqa: E731
    notches = lambda p: np.maximum(  # noqa: E731
        np.minimum(np.abs(p @ R), np.abs(p @ T)) - 2.3, 33.2 - p[:, 1]
    )
    f = cut(cut(body, hollow), notches)

    def mat(p):
        m = bands(BASE_BANDS, (26.0, 29.6))(p)
        r = np.hypot(p[:, 0], p[:, 2])
        slit = (p[:, 1] > 13) & (p[:, 1] < 21) & front_mask(p, 0.975) & (r > 7.5)
        m[slit] = GLOW
        m[(p[:, 1] > 33.5) & (r < 8.9)] = DEEP  # внутренность башни
        return m

    return Piece([(f, mat)])


def bishop():
    prof = base(13.8) + [
        (7.8, 8.6),
        (6.2, 12.0),
        (5.2, 21.0),
        (9.2, 21.6),
        (9.2, 23.6),
        (6.4, 24.6),
        (7.8, 27.0),
        (8.5, 30.5),
        (8.3, 34.0),
        (7.1, 38.0),
        (5.0, 41.3),
        (2.2, 43.6),
        (1.2, 44.6),
    ] + arc(0, 47.0, 2.7, -64, 90, 8)
    body = lathe(prof)
    c = np.array([0.0, 34.5, 0.0])
    line = norm(R * math.cos(math.radians(48)) - Y * math.sin(math.radians(48)))
    n = norm(np.cross(T, line))
    slit = lambda p: np.maximum(np.abs((p - c) @ n) - 0.95, 1.2 - (p @ T))  # noqa: E731
    f = cut(body, slit)

    def mat(p):
        m = bands(BASE_BANDS, (20.8, 24.7))(p)
        near = (np.abs((p - c) @ n) < 1.9) & ((p @ T) > 0.6) & (p[:, 1] > 25)
        m[near] = GLOW
        m[p[:, 1] > 44.0] = GLOW
        return m

    return Piece([(f, mat)])


def queen():
    prof = base(14.2) + [
        (8.0, 8.8),
        (6.4, 12.0),
        (5.0, 25.0),
        (9.4, 25.6),
        (9.4, 27.8),
        (6.2, 28.8),
        (6.6, 31.0),
        (8.6, 37.0),
        (10.8, 42.4),
        (11.1, 43.8),
        (8.4, 44.4),
    ] + arc(0, 44.4, 8.4, 0, 90, 10)
    parts = [(lathe(prof), bands(BASE_BANDS, (25.0, 28.9), (41.2, 44.0)))]
    for k in range(8):
        a = math.radians(22.5 + 45 * k)
        parts.append((sphere([10.6 * math.cos(a), 45.0, 10.6 * math.sin(a)], 2.3), GLOW))
    parts.append((sphere([0, 55.2, 0], 2.8), GLOW))
    return Piece(parts)


def king():
    prof = base(14.6) + [
        (8.4, 9.0),
        (6.8, 12.0),
        (5.6, 27.6),
        (9.8, 28.2),
        (9.8, 30.6),
        (6.6, 31.6),
        (7.2, 34.0),
        (9.2, 41.0),
        (10.4, 46.2),
        (11.0, 46.8),
        (11.0, 48.8),
        (8.2, 49.6),
    ] + arc(0, 49.6, 8.2, 0, 90, 10)
    axes = [R, Y, T]
    cross_v = rbox([0, 63.6, 0], [2.1, 7.4, 2.1], axes, 0.5)
    cross_h = rbox([0, 65.4, 0], [6.4, 2.1, 2.1], axes, 0.5)

    def cross_mat(p):
        m = np.full(len(p), TRIM)
        gem = (np.abs(p @ R) < 1.5) & (np.abs(p[:, 1] - 65.4) < 1.5) & ((p @ T) > 1.0)
        m[gem] = GLOW
        return m

    return Piece(
        [
            (lathe(prof), bands(BASE_BANDS, (27.4, 31.7), (46.4, 49.0))),
            (cross_v, cross_mat),
            (cross_h, cross_mat),
        ]
    )


def knight():
    prof = base(14.0) + [(9.8, 7.6), (9.4, 9.6), (0.0, 9.6)]
    back = [
        (-10.0, 9.0),
        (-11.5, 16.0),
        (-11.6, 23.0),
        (-10.6, 30.0),
        (-8.6, 36.5),
        (-6.0, 41.5),
        (-4.5, 45.5),
    ]
    head = back + [
        (-3.6, 52.5),  # ухо
        (-1.2, 47.8),
        (0.2, 46.8),
        (3.5, 46.0),  # лоб
        (7.5, 43.2),
        (11.5, 39.0),
        (14.5, 35.6),
        (16.6, 33.0),
        (17.4, 30.6),  # нос
        (17.0, 28.2),
        (15.4, 26.8),
        (12.6, 26.6),  # рот
        (13.6, 25.4),
        (12.0, 23.8),
        (8.6, 23.6),
        (4.8, 25.4),  # угол челюсти
        (2.6, 23.0),
        (3.4, 18.5),
        (6.4, 14.5),
        (8.8, 11.0),
        (9.2, 9.0),
    ]
    # грива: зубчатый гребень вдоль спины
    outer, inner = [], []
    for k in range(len(back) - 1):
        (x0, y0), (x1, y1) = back[k], back[k + 1]
        nx, ny = -(y1 - y0), x1 - x0
        nl = math.hypot(nx, ny)
        nx, ny = nx / nl, ny / nl
        for s, off in ((0.15, 0.5), (0.6, 2.3)):
            outer.append((x0 + (x1 - x0) * s + nx * off, y0 + (y1 - y0) * s + ny * off))
        inner.append((x0 - nx * 2.6, y0 - ny * 2.6))
    outer.append(back[-1])
    mane_poly = outer + inner[::-1]

    a = math.radians(18)
    fwd = norm(-R * math.cos(a) + T * math.sin(a))
    side = norm(np.cross(Y, fwd))

    def head_sdf(p):
        u, y, v = p @ fwd, p[:, 1], p @ side
        hw = 4.9 - 0.11 * np.maximum(u - 6.0, 0.0)
        rad = 1.4
        d2 = sd_polygon(u, y, head) + rad
        w = np.stack([d2, np.abs(v) - (hw - rad)], axis=1)
        return (np.minimum(w.max(axis=1), 0) + np.linalg.norm(np.maximum(w, 0), axis=1) - rad) / 1.01

    def head_mat(p):
        u, y, v = p @ fwd, p[:, 1], p @ side
        m = np.full(len(p), MAIN)
        strap = (np.abs(u - (1.2 + (y - 24.0) * 0.16)) < 0.9) & (y > 23.5) & (y < 46.5)
        m[strap] = TRIM
        eye = (np.hypot(u - 7.6, y - 39.4) < 1.7) & (np.abs(v) > 2.6)
        m[eye] = GLOW
        nostril = (np.hypot(u - 15.4, y - 30.6) < 1.0) & (np.abs(v) > 2.2)
        m[nostril] = SEAM
        return m

    return Piece(
        [
            (lathe(prof), bands(BASE_BANDS, (6.7, 9.7))),
            (head_sdf, head_mat),
            (extrude(mane_poly, fwd, side, 2.6, 0.6), TRIM),
        ],
        stretch=[1.0, 1.12, 1.0],
    )

# ---------------------------------------------------------------- палитры
OUTLINE = hexc("#020201")
SHADOW = (0, 0, 0, 77)

PALETTES = {
    # Белые — светлые мехи с голубой подсветкой.
    "w": {
        MAIN: ["#3c3a30", "#6e6a55", "#a49e80", "#d3cdb1", "#f4f1df"],
        TRIM: ["#1d2220", "#333a36", "#535d57", "#7d8881", "#a9b3ac"],
        GLOW: ["#16627f", "#2aa9d6", "#57d9ff", "#a6f2ff", "#e9fdff"],
        SEAM: ["#020201"] * 5,
    },
    # Чёрные — веки: фиолетовый панцирь, хитин и оранжевые огни.
    "b": {
        MAIN: ["#100d1a", "#1f1a30", "#342e4c", "#544c78", "#8a80b6"],
        TRIM: ["#1a110d", "#3a2418", "#643f29", "#94613f", "#c8975a"],
        GLOW: ["#6e2408", "#d0501b", "#ff8a2e", "#ffc35a", "#fff1b4"],
        SEAM: ["#020201"] * 5,
    },
}


def tone_of(l):
    return np.select([l > 0.83, l > 0.5, l > 0.16, l > -0.22], [4, 3, 2, 1], 0)


# ---------------------------------------------------------------- рендер фигуры
W, H = 64, 84
AX, AY = 32, 66  # где на холсте центр клетки
SS = 3


def march(piece, o, d, tmax, steps=120, eps=0.03):
    t = np.zeros(len(o))
    hit = np.zeros(len(o), bool)
    alive = np.ones(len(o), bool)
    for _ in range(steps):
        idx = np.nonzero(alive)[0]
        if len(idx) == 0:
            break
        p = o[idx] + d[idx] * t[idx, None] if d.ndim == 2 else o[idx] + d * t[idx, None]
        dist = piece.dist(p)
        h = dist < eps
        hit[idx[h]] = True
        alive[idx[h]] = False
        t[idx] += np.maximum(dist, 0.02)
        gone = t[idx] > tmax
        alive[idx[gone]] = False
    return hit, t


def normals(piece, p):
    e = 0.05
    n = np.zeros_like(p)
    for k in range(3):
        dv = np.zeros(3)
        dv[k] = e
        n[:, k] = piece.dist(p + dv) - piece.dist(p - dv)
    return n / (np.linalg.norm(n, axis=1, keepdims=True) + 1e-9)


def render_piece(piece, color, scale=0.86):
    """RGBA-тело и отдельный слой тени."""
    pal = {k: [hexc(c) for c in v] for k, v in PALETTES[color].items()}
    m = pal[MAIN]
    pal[DEEP] = [OUTLINE, m[0], m[0], m[1], m[1]]
    # лучи по субпикселям
    xs = (np.arange(W * SS) + 0.5) / SS - AX
    ys = (np.arange(H * SS) + 0.5) / SS - AY
    gx, gy = np.meshgrid(xs, ys)
    gx, gy = gx.ravel() / scale, gy.ravel() / scale
    o = gx[:, None] * R - gy[:, None] * U + D * 200.0
    hit, t = march(piece, o, -D, 420.0)
    p = o + (-D) * t[:, None]
    tone = np.zeros(len(o), int)
    mat = np.zeros(len(o), int)
    hp = p[hit]
    n = normals(piece, hp)
    l = n @ L_SHADE
    tn = tone_of(l)
    # самозатенение
    sh_hit, _ = march(piece, hp + n * 0.35, L_SHADOW, 90.0, steps=70, eps=0.05)
    tn = np.where(sh_hit, np.minimum(tn, 1), tn)
    mt = piece.material(hp)
    # светящееся почти не зависит от освещения
    tn = np.where(mt == GLOW, np.clip(tn + 2, 2, 4), tn)
    tone[hit] = tn
    mat[hit] = mt

    # сводим субпиксели в пиксели
    hit_g = hit.reshape(H, SS, W, SS)
    cover = hit_g.sum(axis=(1, 3))
    solid = cover >= (SS * SS + 1) // 2
    tone_g = tone.reshape(H, SS, W, SS).transpose(0, 2, 1, 3).reshape(H, W, SS * SS)
    mat_g = mat.reshape(H, SS, W, SS).transpose(0, 2, 1, 3).reshape(H, W, SS * SS)
    hit_s = hit_g.transpose(0, 2, 1, 3).reshape(H, W, SS * SS)
    t_g = t.reshape(H, SS, W, SS).transpose(0, 2, 1, 3).reshape(H, W, SS * SS)
    img = np.zeros((H, W, 4), np.uint8)
    depth = np.full((H, W), np.inf)
    tone_px = np.zeros((H, W), int)
    mat_px = np.zeros((H, W), int)
    centre = (SS * SS) // 2
    for y in range(H):
        for x in range(W):
            if not solid[y, x]:
                continue
            hs = hit_s[y, x]
            # материал — по центральному субпикселю, если он попал, иначе большинством
            if hs[centre]:
                m = mat_g[y, x, centre]
            else:
                vals, cnt = np.unique(mat_g[y, x][hs], return_counts=True)
                m = vals[cnt.argmax()]
            sel = hs & (mat_g[y, x] == m)
            tone_px[y, x] = int(round(np.median(tone_g[y, x][sel])))
            mat_px[y, x] = m
            depth[y, x] = np.median(t_g[y, x][hs]) * scale
    # блик по левому и верхнему краю силуэта, как у юнитов ITB
    left_open = solid & ~np.roll(solid, 1, 1)
    top_open = solid & ~np.roll(solid, 1, 0)
    rim = (left_open | top_open) & (mat_px != GLOW) & (tone_px >= 2)
    tone_px[rim] = np.minimum(tone_px[rim] + 1, 4)
    for y, x in zip(*np.nonzero(solid)):
        img[y, x, :3] = pal[mat_px[y, x]][tone_px[y, x]]
        img[y, x, 3] = 255
    # внутренние швы на скачках глубины
    seam = np.zeros((H, W), bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        nb = np.roll(np.roll(depth, dy, 0), dx, 1)
        with np.errstate(invalid="ignore"):
            seam |= solid & np.isfinite(nb) & (depth - nb > 3.2)
    img[seam, :3] = OUTLINE
    # внешний контур
    out = np.zeros((H, W), bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        out |= np.roll(np.roll(solid, dy, 0), dx, 1)
    out &= ~solid
    img[out, :3] = OUTLINE
    img[out, 3] = 255

    # тень на плоскость земли (по центрам пикселей)
    cx = (np.arange(W) + 0.5 - AX) / scale
    cy = (np.arange(H) + 0.5 - AY) / scale
    gx, gy = np.meshgrid(cx, cy)
    o = gx.ravel()[:, None] * R - gy.ravel()[:, None] * U + D * 200.0
    tg = o[:, 1] / D[1]
    ground = o - D * tg[:, None]
    sh, _ = march(piece, ground + Y * 0.05, L_SHADOW, 120.0, steps=80, eps=0.05)
    shadow = np.zeros((H, W, 4), np.uint8)
    shadow[sh.reshape(H, W)] = SHADOW
    return Image.fromarray(img, "RGBA"), Image.fromarray(shadow, "RGBA")


PIECES = {"P": pawn, "N": knight, "B": bishop, "R": rook, "Q": queen, "K": king}


# ---------------------------------------------------------------- клетки
TW, TH, SIDE = 56, 42, 31
BLOCK_H = TH + SIDE + 1

TOPS = {
    # тёмные клетки — трава
    "dark": {
        "base": "#46513d",
        "rim": "#55704b",
        "rim2": "#4d5f44",
        "spots": ["#3e4736", "#4f6045", "#5b7a50"],
        "edge": "#33392e",
        "lip": "#36432f",
    },
    # светлые клетки — песок
    "light": {
        "base": "#c3935d",
        "rim": "#dbad70",
        "rim2": "#cfa066",
        "spots": ["#ae824f", "#d6a86b", "#e5bd82"],
        "edge": "#8c6943",
        "lip": "#94683f",
    },
}
LEFT_STRATA = ["#6a5c47", "#5c5040", "#4f4537", "#433a30", "#383029", "#2d2723", "#24201d"]
RIGHT_STRATA = ["#43535b", "#39474f", "#303c42", "#29333a", "#232b30", "#1e2428", "#1a1e21"]
BLOCK_EDGE = hexc("#18191a")


def block(kind, seed):
    rng = np.random.default_rng(seed)
    st = TOPS[kind]
    img = np.zeros((BLOCK_H, TW, 4), np.uint8)
    top = np.zeros((BLOCK_H, TW), bool)
    for y in range(BLOCK_H):
        for x in range(TW):
            dx = (x + 0.5 - TW / 2) / (TW / 2)
            dy = (y + 0.5 - TH / 2) / (TH / 2)
            k = abs(dx) + abs(dy)
            if k <= 1.0:
                top[y, x] = True
                c = st["base"]
                if k > 0.93:
                    c = st["rim"]
                elif k > 0.86:
                    c = st["rim2"]
                img[y, x, :3] = hexc(c)
                img[y, x, 3] = 255
                continue
            # бока
            if x < TW / 2:
                ye = TH / 2 + (x + 0.5) * TH / TW
                strata, along = LEFT_STRATA, x
            else:
                ye = TH / 2 + (TW - x - 0.5) * TH / TW
                strata, along = RIGHT_STRATA, TW - 1 - x
            yy = y + 0.5 - ye
            if 0 < yy <= SIDE:
                if yy <= 2.5:
                    c = st["lip"]
                else:
                    wob = 1.2 if (along // 6 + seed) % 3 == 0 else 0.0
                    band = int((yy - 2.5 + wob) / (SIDE - 2.5) * len(strata))
                    c = strata[min(max(band, 0), len(strata) - 1)]
                img[y, x, :3] = hexc(c)
                img[y, x, 3] = 255
    # пятна текстуры на верхней грани: кластеры 1–3 пикселя
    ys, xs = np.nonzero(top)
    for _ in range(26):
        k = rng.integers(len(ys))
        y0, x0 = ys[k], xs[k]
        c = hexc(st["spots"][rng.integers(len(st["spots"]))])
        for dy, dx in [(0, 0), (0, 1), (0, -1), (1, 0)][: rng.integers(1, 4)]:
            y, x = y0 + dy, x0 + dx
            if 0 <= y < BLOCK_H and 0 <= x < TW and top[y, x]:
                dxn = abs((x + 0.5 - TW / 2) / (TW / 2)) + abs((y + 0.5 - TH / 2) / (TH / 2))
                if dxn < 0.84:
                    img[y, x, :3] = c
    # камешки на боках
    for _ in range(14):
        x = rng.integers(TW)
        y = rng.integers(TH // 2, BLOCK_H)
        if img[y, x, 3] and not top[y, x]:
            img[y, x, :3] = (np.array(img[y, x, :3], int) * 0.8).astype(np.uint8)
    # линии: задние кромки ромба (сетка), передние кромки, рёбра блока
    for y in range(BLOCK_H):
        for x in range(TW):
            if not img[y, x, 3]:
                continue
            if top[y, x]:
                up = y == 0 or not top[y - 1, x]
                if up and y < TH / 2:
                    img[y, x, :3] = hexc(st["edge"])
                down = y + 1 >= BLOCK_H or not top[y + 1, x]
                if down and y >= TH / 2 - 1:
                    img[y, x, :3] = BLOCK_EDGE
            else:
                if x in (0, TW - 1, TW // 2 - 1, TW // 2) and x not in ():
                    if x in (0, TW - 1) or (TW // 2 - 1 <= x <= TW // 2 and y > TH - 2):
                        img[y, x, :3] = BLOCK_EDGE
                below = y + 1 >= BLOCK_H or not img[y + 1, x, 3]
                if below:
                    img[y, x, :3] = BLOCK_EDGE
    return Image.fromarray(img, "RGBA")


def diamond_mask(inset=0):
    m = np.zeros((TH, TW), bool)
    for y in range(TH):
        for x in range(TW):
            dx = abs(x + 0.5 - TW / 2) / (TW / 2 - inset * 4 / 3)
            dy = abs(y + 0.5 - TH / 2) / (TH / 2 - inset)
            m[y, x] = dx + dy <= 1.0
    return m


def overlay(fill, line, fill_a=110, inset=2, width=2):
    """Подсветка клетки: полупрозрачная заливка и пиксельная рамка."""
    outer = diamond_mask(inset)
    inner = diamond_mask(inset + width)
    img = np.zeros((TH, TW, 4), np.uint8)
    img[outer] = (*hexc(fill), fill_a)
    ring = outer & ~inner
    img[ring] = (*hexc(line), 255)
    return Image.fromarray(img, "RGBA")


# ---------------------------------------------------------------- шрифт координат
GLYPHS = {
    "A": [".####.", "##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##"],
    "B": ["#####.", "##..##", "##..##", "#####.", "##..##", "##..##", "##..##", "#####."],
    "C": [".####.", "##..##", "##....", "##....", "##....", "##....", "##..##", ".####."],
    "D": ["#####.", "##..##", "##..##", "##..##", "##..##", "##..##", "##..##", "#####."],
    "E": ["######", "##....", "##....", "#####.", "##....", "##....", "##....", "######"],
    "F": ["######", "##....", "##....", "#####.", "##....", "##....", "##....", "##...."],
    "G": [".####.", "##..##", "##....", "##....", "##.###", "##..##", "##..##", ".####."],
    "H": ["##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##", "##..##"],
    "1": ["..##..", ".###..", "####..", "..##..", "..##..", "..##..", "..##..", "######"],
    "2": [".####.", "##..##", "....##", "...##.", "..##..", ".##...", "##....", "######"],
    "3": [".####.", "##..##", "....##", "..###.", "....##", "....##", "##..##", ".####."],
    "4": ["...##.", "..###.", ".####.", "##.##.", "######", "...##.", "...##.", "...##."],
    "5": ["######", "##....", "##....", "#####.", "....##", "....##", "##..##", ".####."],
    "6": [".####.", "##....", "##....", "#####.", "##..##", "##..##", "##..##", ".####."],
    "7": ["######", "....##", "...##.", "...##.", "..##..", "..##..", ".##...", ".##..."],
    "8": [".####.", "##..##", "##..##", ".####.", "##..##", "##..##", "##..##", ".####."],
}


def glyph(ch, color="#e9ecf2"):
    rows = GLYPHS[ch]
    h, w = len(rows), len(rows[0])
    img = np.zeros((h + 3, w + 2, 4), np.uint8)
    body = np.zeros((h + 3, w + 2), bool)
    for y, row in enumerate(rows):
        for x, c in enumerate(row):
            if c == "#":
                body[y + 1, x + 1] = True
    out = np.zeros_like(body)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (1, -1), (2, 0)):
        out |= np.roll(np.roll(body, dy, 0), dx, 1)
    img[out & ~body] = (*hexc("#08090d"), 255)
    img[body] = (*hexc(color), 255)
    # нижняя строка каждой линии чуть темнее — объём
    shade = body & ~np.roll(body, -1, 0)
    img[shade] = (*hexc("#a9b0bf"), 255)
    return Image.fromarray(img, "RGBA")


# ---------------------------------------------------------------- полоска здоровья
def health_bar(frac, segments=5, w=30):
    h = 7
    img = np.zeros((h, w, 4), np.uint8)
    img[:, :] = (*hexc("#08090d"), 255)
    img[1:-1, 1:-1] = (*hexc("#e6e9ef"), 255)
    img[2:-2, 2:-2] = (*hexc("#0d0f14"), 255)
    inner = w - 4
    fill = int(round(inner * frac))
    green = hexc("#53d04a") if frac > 0.5 else hexc("#f2c53d") if frac > 0.25 else hexc("#ef4b3c")
    img[2:-2, 2 : 2 + fill] = (*green, 255)
    img[2, 2 : 2 + fill] = (*[min(255, c + 60) for c in green], 255)
    for k in range(1, segments):
        x = 2 + round(inner * k / segments)
        img[2:-2, x] = (*hexc("#0d0f14"), 255)
    return Image.fromarray(img, "RGBA")


# ---------------------------------------------------------------- сборка доски
def iso(i, j):
    return (i - j) * (TW // 2), (i + j) * (TH // 2)


def board_preview(
    sprites, scale=2, demo=True, margin_x=40, margin_top=70, margin_bottom=60, margin_right=None
):
    bw = 8 * TW
    bh = 8 * TH + SIDE
    right = margin_x if margin_right is None else margin_right
    cw, ch = bw + margin_x + right, bh + margin_top + margin_bottom
    ox = margin_x + bw // 2 - TW // 2
    oy = margin_top

    # фон: тёмно-синий с виньеткой
    yy, xx = np.mgrid[0:ch, 0:cw]
    top_c, bot_c = np.array(hexc("#20242f"), float), np.array(hexc("#0f1117"), float)
    g = (yy / ch)[..., None]
    bg = top_c * (1 - g) + bot_c * g
    r = np.hypot((xx - cw / 2) / (cw * 0.62), (yy - ch * 0.45) / (ch * 0.7))
    bg *= np.clip(1.15 - r * 0.45, 0.6, 1.1)[..., None]
    # мелкая пиксельная «пыль»
    rng = np.random.default_rng(3)
    for _ in range(160):
        x, y = rng.integers(cw), rng.integers(ch)
        bg[y, x] = bg[y, x] * 1.35 + 6
    canvas = Image.fromarray(np.clip(bg, 0, 255).astype(np.uint8), "RGB").convert("RGBA")

    blocks = {
        (kind, v): block(kind, 11 * v + (0 if kind == "dark" else 5))
        for kind in ("dark", "light")
        for v in range(4)
    }

    def sq(file, rank):  # клетка шахмат -> (i, j)
        return file, 7 - rank

    def cell_xy(file, rank):
        i, j = sq(file, rank)
        x, y = iso(i, j)
        return ox + x, oy + y

    order = sorted(((f, r) for f in range(8) for r in range(8)), key=lambda c: sum(sq(*c)))
    for f, r in order:
        kind = "dark" if (f + r) % 2 == 0 else "light"
        x, y = cell_xy(f, r)
        canvas.alpha_composite(blocks[(kind, (f * 7 + r * 3) % 4)], (x, y))

    # подписи: буквы вдоль нижнего левого края, цифры вдоль нижнего правого
    for f in range(8):
        gl = glyph("ABCDEFGH"[f])
        x, y = cell_xy(f, 0)
        canvas.alpha_composite(gl, (x + 9, y + 61))
    for r in range(8):
        gl = glyph("12345678"[r])
        x, y = cell_xy(7, r)
        canvas.alpha_composite(gl, (x + 39, y + 61))

    position = {}
    back = "RNBQKBNR"
    for f in range(8):
        position[(f, 0)] = "w" + back[f]
        position[(f, 1)] = "wP"
        position[(f, 6)] = "bP"
        position[(f, 7)] = "b" + back[f]
    hp = {}
    if demo:
        # немного «партии»: e4, e5, Nf3, Nc6, выбран белый конь f3
        del position[(4, 1)]
        position[(4, 3)] = "wP"
        del position[(4, 6)]
        position[(4, 4)] = "bP"
        del position[(6, 0)]
        position[(5, 2)] = "wN"
        del position[(1, 7)]
        position[(2, 5)] = "bN"
        hp = {(4, 3): 0.6, (4, 4): 0.8, (5, 2): 1.0, (2, 5): 0.4}
        marks = {
            (5, 2): overlay("#ffe27a", "#fff3b8", 70, 1, 2),
            (6, 0): overlay("#ffe27a", "#ffe27a", 45, 3, 1),
            (7, 3): overlay("#2f8fe0", "#7fd4ff", 95),
            (6, 4): overlay("#2f8fe0", "#7fd4ff", 95),
            (3, 3): overlay("#2f8fe0", "#7fd4ff", 95),
            (4, 4): overlay("#e0452f", "#ff9a6b", 110),
        }
        for (f, r), ov in marks.items():
            x, y = cell_xy(f, r)
            canvas.alpha_composite(ov, (x, y))

    # тени, затем фигуры от дальних к ближним
    items = sorted(position.items(), key=lambda kv: sum(sq(*kv[0])))
    for (f, r), code in items:
        _, shadow = sprites[code]
        x, y = cell_xy(f, r)
        canvas.alpha_composite(shadow, (x + TW // 2 - AX, y + TH // 2 - AY))
    for (f, r), code in items:
        body, _ = sprites[code]
        x, y = cell_xy(f, r)
        canvas.alpha_composite(body, (x + TW // 2 - AX, y + TH // 2 - AY))
        if (f, r) in hp:
            bb = np.array(body)[..., 3].nonzero()[0]
            top = bb.min() if len(bb) else 0
            bar = health_bar(hp[(f, r)])
            canvas.alpha_composite(bar, (x + TW // 2 - bar.width // 2, y + TH // 2 - AY + top - 9))

    canvas = canvas.resize((cw * scale, ch * scale), Image.NEAREST)
    return canvas


def sprite_sheet(sprites, scale=4):
    codes = [c + k for c in "wb" for k in "PNBRQK"]
    sheet = Image.new("RGBA", (W * 6, H * 2), hexc("#1b1e27") + (255,))
    for n, code in enumerate(codes):
        body, shadow = sprites[code]
        x, y = (n % 6) * W, (n // 6) * H
        cell = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        cell.alpha_composite(shadow)
        cell.alpha_composite(body)
        sheet.alpha_composite(cell, (x, y))
    return sheet.resize((sheet.width * scale, sheet.height * scale), Image.NEAREST)


if __name__ == "__main__":
    for sub in ("pieces", "tiles"):
        os.makedirs(os.path.join(OUT, sub), exist_ok=True)
    sprites = {}
    for color in "wb":
        for k, make in PIECES.items():
            body, shadow = render_piece(make(), color)
            sprites[color + k] = (body, shadow)
            body.save(os.path.join(OUT, "pieces", f"{color}{k}.png"))
            shadow.save(os.path.join(OUT, "pieces", f"{color}{k}_shadow.png"))
            print("piece", color + k, flush=True)
    for kind in ("dark", "light"):
        for v in range(4):
            block(kind, 11 * v + (0 if kind == "dark" else 5)).save(
                os.path.join(OUT, "tiles", f"{kind}_{v}.png")
            )
    sprite_sheet(sprites).save(os.path.join(OUT, "preview_pieces.png"))
    board_preview(sprites).save(os.path.join(OUT, "preview_board.png"))
    print("done", OUT)
