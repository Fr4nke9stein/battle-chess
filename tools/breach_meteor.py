"""Метеорит мага в стиле Into the Breach: падает сверху-справа и взрывается на одной клетке.

С 2026-10-21 (просьба пользователя, по образцу-картинке): метеорит крупный — бугристый камень с
кратерами (`ROCK`), спереди раскалён (кромка светится оранжевым и жёлтым), камень медленно
вращается; его окутывает огонь, позади — длинный хвост пламени из языков, бегущих назад (слои от
бледно-жёлтого у камня к тёмно-красному на конце), за хвостом — искры. Кадров вдвое больше и
они короче (FRAME_MS) — движение плавнее; шум огня и дыма течёт непрерывно (`value_noise`).

Кадр FW×FH, центр верхней грани клетки — (CX, CY). Кадры [0, FALL) — метеорит летит (первые
кадры проявляется — хвост целиком в кадре, без обрезанного края), в кадре IMPACT = FALL он
касается клетки: вспышка, огненный шар поднимается и тускнеет, по клетке — кольцо огня и
подпалина, во все стороны — раскалённые осколки, в конце — клуб дыма. Взрыв не выходит за ромб
клетки (шар и дым — над ней). Палитры огня, дыма и контура — из `breach_burn.py`.

`py tools/breach_meteor.py [папка]` — по умолчанию assets/breach/effects: meteor.png (кадры
в ряд, FRAME_MS), preview_meteor.gif, preview_meteor_frames.png.
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
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

FIRE, OUTLINE, SMOKE, CHAR = burn.FIRE, burn.OUTLINE, burn.SMOKE, burn.CHAR
SCORCH = hexc("#4a2414")
ROCK = [hexc(c) for c in ["#24141a", "#442a32", "#6a464a", "#8e6862", "#b48a78"]]  # камень, от тёмного
FW, FH = 200, 280
CX, CY = 72, 250  # центр верхней грани клетки
HTW, HTH = art.TW / 2, art.TH / 2
FALL = 16
IMPACT = FALL
BOOM = 28
FRAMES = FALL + BOOM
FRAME_MS = 30
START = (CX + 88, 80)  # где голова метеорита в первом кадре
END = (CX, CY - 8)
ROCK_R = 12.0  # радиус камня
TAIL = (60.0, 84.0)  # длина хвоста в первом и последнем кадре полёта
SEED = 11

YY, XX = np.mgrid[0:FH, 0:FW]
DX, DY = XX + 0.5 - CX, YY + 0.5 - CY
DIAMOND = np.abs(DX) / HTW + np.abs(DY) / HTH <= 1.0


def value_noise(x, y, seed):
    """Гладкий шум 0..1 в любой точке (x, y) — по решётке с шагом 1; для течения огня и дыма."""
    x0, y0 = np.floor(x).astype(np.int64), np.floor(y).astype(np.int64)
    fx, fy = x - x0, y - y0
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)

    def h(ix, iy):
        v = (ix * 374761393 + iy * 668265263 + seed * 2246822519) & 0xFFFFFFFF
        v = ((v ^ (v >> 13)) * 1274126177) & 0xFFFFFFFF
        return ((v ^ (v >> 16)) & 0xFFFF) / 65535.0

    a, b = h(x0, y0), h(x0 + 1, y0)
    c, d = h(x0, y0 + 1), h(x0 + 1, y0 + 1)
    return (a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + d * fx) * fy


class Frame:
    def __init__(self):
        self.rgb = np.zeros((FH, FW, 3), float)
        self.alpha = np.zeros((FH, FW), float)
        self.fire = np.zeros((FH, FW), bool)  # пиксели, которым нужен контур

    def put(self, mask, color, alpha=255, outline=True):
        self.rgb[mask] = color
        self.alpha[mask] = alpha
        if outline:
            self.fire |= mask

    def dot(self, x, y, color, alpha=255):
        x, y = int(round(x)), int(round(y))
        if 0 <= x < FW and 0 <= y < FH:
            self.rgb[y, x] = color
            self.alpha[y, x] = alpha

    def image(self, fade=1.0):
        """Тёмный контур в пиксель вокруг огня, как у всего в игре; fade — проявление."""
        f = self.fire
        around = np.zeros_like(f)
        around[1:] |= f[:-1]
        around[:-1] |= f[1:]
        around[:, 1:] |= f[:, :-1]
        around[:, :-1] |= f[:, 1:]
        edge = around & ~f & (self.alpha < 128)
        rgb, alpha = self.rgb.copy(), self.alpha.copy()
        rgb[edge] = OUTLINE
        alpha[edge] = 255
        alpha *= fade
        out = np.dstack([rgb, alpha]).clip(0, 255).astype(np.uint8)
        return Image.fromarray(out, "RGBA")


def fire_ramp(t):
    """0 — белое ядро, 1 — тёмно-красная кромка."""
    i = min(len(FIRE) - 1, max(0, int(t * len(FIRE))))
    return FIRE[i]


def head_at(f):
    """Голова метеорита в кадре полёта f (к кадру FALL − 1 касается клетки), направление."""
    k = ((f + 1) / FALL) ** 1.3
    x = START[0] + (END[0] - START[0]) * k
    y = START[1] + (END[1] - START[1]) * k
    dx, dy = END[0] - START[0], END[1] - START[1]
    n = math.hypot(dx, dy)
    return x, y, dx / n, dy / n


def meteor(frame, f):
    x, y, ux, uy = head_at(f)
    t = f / (FALL - 1)
    rx, ry = XX + 0.5 - x, YY + 0.5 - y
    fwd = rx * ux + ry * uy  # вперёд по полёту
    s = -fwd  # назад, вдоль хвоста
    p = -rx * uy + ry * ux  # поперёк
    dist = np.hypot(rx, ry)
    # ---- огонь: оболочка вокруг камня и хвост из языков, узор бежит назад
    length = TAIL[0] + (TAIL[1] - TAIL[0]) * t
    halo = ROCK_R + 3.2
    flow = f * 3.2  # пикселей назад за кадр
    n1 = value_noise((s + flow) / 7.0, p / 3.2, SEED)
    n2 = value_noise((s + flow * 1.4) / 3.5, p / 2.0, SEED + 1)
    n = 0.65 * n1 + 0.35 * n2
    u = np.clip(s / length, 0, 1)
    width = halo * (1 + 0.45 * np.sin(np.clip(u / 0.22, 0, 1) * math.pi / 2)) * (1 - u) ** 0.85  # за камнем пламя шире
    width = np.maximum(width, np.where(u < 0.12, halo, 0))
    tongues = 0.55 + 0.45 * np.cos(p / np.maximum(width, 1) * 3.0 * math.pi + n1 * 5)  # языки к концу хвоста
    keep = np.where(u > 0.35, tongues > (u - 0.35) * 1.1, True)
    tail = (s >= 0) & (s <= length) & (np.abs(p) < width * (0.78 + 0.5 * (n - 0.5))) & keep
    shell = dist < halo + 1.8 * (n - 0.5) - np.clip(fwd, 0, None) * 0.18  # спереди огонь тоньше
    flame = tail | shell
    heat = np.abs(p) / np.maximum(width, 1) * 0.55 + u * 0.8 + (n - 0.5) * 0.4
    heat = np.where(s < 0, np.clip(dist - ROCK_R, 0, None) / 3.4 * 0.35, heat)
    # у камня — бледно-жёлтое, дальше оранжевое, к краям и концу — красное
    for lim, col in ((0.14, FIRE[1]), (0.26, FIRE[2]), (0.42, FIRE[3]), (0.58, FIRE[5]), (0.74, FIRE[6]), (9, FIRE[7])):
        layer = flame & (heat < lim) & (frame.alpha == 0)
        frame.put(layer, col)
    # ---- камень: бугристый, вращается; спереди раскалён, позади тёмный, кратеры
    spin = f * 0.09
    th = np.arctan2(ry, rx) - spin
    edge_r = ROCK_R * (1 + 0.07 * np.sin(3 * th + 0.8) + 0.045 * np.sin(5 * th + 2.1) + 0.03 * np.sin(8 * th))
    body = dist <= edge_r
    frame.put(body, ROCK[2])
    frame.put(body & (fwd < -ROCK_R * 0.25), ROCK[1])
    frame.put(body & (fwd < -ROCK_R * 0.7), ROCK[0])
    frame.put(body & (fwd > ROCK_R * 0.15) & (p < 0), ROCK[3])  # свет огня сбоку-спереди
    craters = [(0.3, 0.45, 2.6), (2.1, 0.5, 2.2), (3.7, 0.38, 3.0), (4.9, 0.62, 1.7), (1.2, 0.72, 1.5), (5.8, 0.15, 1.6)]
    for ang, dd, cr in craters:
        a = ang + spin
        cx_, cy_ = x + math.cos(a) * dd * ROCK_R, y + math.sin(a) * dd * ROCK_R
        cd = np.hypot(XX + 0.5 - cx_, YY + 0.5 - cy_)
        pit = body & (cd < cr)
        lit = pit & (np.hypot(XX + 0.5 - cx_ + ux * 0.9, YY + 0.5 - cy_ + uy * 0.9) >= cr)  # стенка к огню
        frame.put(pit, ROCK[0])
        frame.put(lit, ROCK[3])
        rim = body & (cd >= cr) & (cd < cr + 0.9) & (((XX + 0.5 - cx_) * ux + (YY + 0.5 - cy_) * uy) < 0)
        frame.put(rim, ROCK[4])
    glow = body & (dist > edge_r - 2.0) & (fwd > ROCK_R * 0.1)  # раскалённая кромка спереди
    frame.put(glow, FIRE[3])
    frame.put(glow & (fwd > ROCK_R * 0.55), FIRE[2])
    frame.put(body & (dist > edge_r - 1.0) & (fwd > ROCK_R * 0.75), FIRE[1])
    # ---- искры: летят назад вдоль хвоста и гаснут
    rng = np.random.default_rng(SEED + 7)
    for k in range(10):
        life = (rng.random() + f * 0.09) % 1.0
        back = halo + life * length * 1.15
        side = (rng.random() - 0.5) * halo * 1.8 * (0.5 + life)
        sx, sy = x - ux * back - uy * side, y - uy * back + ux * side
        frame.dot(sx, sy, FIRE[1 if life < 0.4 else 3 if life < 0.75 else 5])


def explosion(frame, k):
    """Кадр взрыва k (0 — касание); время kk приведено к прежним 15 кадрам — формы те же, кадров больше."""
    kk = k * 14 / (BOOM - 1)
    u = kk / 14
    rise = kk * 2.2
    n0 = value_noise(XX / 3.0, (YY + rise * 1.3) / 3.0, SEED * 7)
    n1 = value_noise(XX / 4.0, (YY + rise) / 4.0, SEED * 7 + 1)
    n2 = value_noise(XX / 5.0, (YY + rise * 0.8) / 5.0, SEED * 7 + 2)
    # подпалина на клетке — держится, к концу тает
    scorch_r = min(1.0, 0.35 + kk * 0.18)
    scorch = DIAMOND & ((DX / (HTW * 0.8 * scorch_r)) ** 2 + (DY / (HTH * 0.8 * scorch_r)) ** 2 <= 1)
    if kk >= 1:
        fade = 1.0 if kk < 10 else max(0.0, (14 - kk) / 4)
        frame.put(scorch, SCORCH, 200 * fade, outline=False)
        embers = scorch & (value_noise(XX / 3.0, YY / 3.0, SEED * 9) > 0.78)
        if kk < 11:
            frame.put(embers, FIRE[6 + int(kk * 1.5) % 2], 230 * fade, outline=False)
    # кольцо огня по клетке
    if kk < 6:
        ring = 0.35 + kk * 0.13
        d = np.sqrt((DX / HTW) ** 2 + (DY / HTH) ** 2)
        band = DIAMOND & (np.abs(d - ring) < 0.09 + 0.02 * kk)
        band &= n1 > 0.25 + 0.08 * kk
        frame.put(band, fire_ramp(0.2 + kk * 0.12))
    # огненный шар: растёт, поднимается, остывает и уходит в дым
    if kk < 13.5:
        radius = min(6 + 5.0 * kk, 22) * (1 if kk < 9 else max(0.4, 1 - (kk - 9) * 0.12))
        lift = 3 + rise
        cx, cy = CX, CY - lift
        dist = np.sqrt((XX + 0.5 - cx) ** 2 + ((YY + 0.5 - cy) * 1.1) ** 2)
        wobble = (n2 - 0.5) * 6
        ball = dist <= radius + wobble
        ball &= (YY + 0.5) <= CY + HTH * 0.5  # не ниже переднего края клетки
        hot = max(0.0, 1 - kk / 8)
        if kk >= 7:
            # дым — клубы: сверху светлее, снизу темнее, кромка по шуму
            puff = (YY + 0.5 - cy) / max(radius, 1) + (n0 - 0.5) * 0.9
            alpha = 210 * (1 - u * 0.55)
            frame.put(ball & (puff < -0.35), SMOKE[0], alpha, outline=False)
            frame.put(ball & (puff >= -0.35) & (puff < 0.25), SMOKE[1], alpha, outline=False)
            frame.put(ball & (puff >= 0.25), SMOKE[2], alpha, outline=False)
            # внутри ещё тлеет — рваные пятна, гаснут
            glow = ball & (dist <= radius * 0.6) & (n1 > 0.45 + 0.07 * (kk - 7))
            if kk < 11:
                frame.put(glow, FIRE[5 + min(2, int((kk - 7) // 2))], 255, outline=False)
        else:
            rel = dist / max(radius, 1) + (n1 - 0.5) * 0.35
            for i, edge in enumerate([0.35, 0.55, 0.75, 1.01]):
                layer = ball & (rel <= edge + wobble * 0.02) & (frame.alpha < 255)
                shade = fire_ramp(min(0.99, (i * 0.25 + (1 - hot) * 0.35)))
                frame.put(layer, shade)
    # вспышка в миг удара
    if kk < 2:
        flash = (DX / (17 - 5 * kk)) ** 2 + ((DY + 7) / (14 - 4 * kk)) ** 2 <= 1
        frame.put(flash, FIRE[0])
    # раскалённые осколки
    rng = np.random.default_rng(SEED)
    for _ in range(12):
        vx, vy, size = rng.uniform(-2.8, 2.8), rng.uniform(-4.4, -1.6), int(rng.integers(1, 3))
        px = CX + vx * kk
        py = CY - 6 + vy * kk + 0.45 * kk * kk
        if py > CY + HTH * 0.6 or kk > 12:
            continue
        color = FIRE[2] if kk < 4 else FIRE[4] if kk < 8 else CHAR[2]
        frame.dot(px, py, color)
        if size > 1:
            frame.dot(px + 1, py, color)
            frame.dot(px, py - 1, OUTLINE)


def render():
    frames = []
    for f in range(FRAMES):
        frame = Frame()
        if f < FALL:
            meteor(frame, f)
            frames.append(frame.image(min(1.0, (f + 1) / 3)))  # проявляется за первые кадры
        else:
            explosion(frame, f - FALL)
            frames.append(frame.image())
    return frames


def save(frames):
    os.makedirs(OUT, exist_ok=True)
    strip = Image.new("RGBA", (FW * len(frames), FH))
    for i, im in enumerate(frames):
        strip.paste(im, (i * FW, 0))
    strip.save(os.path.join(OUT, "meteor.png"))
    # превью: на клетке доски, ×2
    tile = art.block("dark", 11)
    shots = []
    for im in frames:
        bg = Image.new("RGBA", (FW, FH), (38, 42, 58, 255))
        bg.alpha_composite(tile, (int(CX - HTW), int(CY - HTH)))
        bg.alpha_composite(im)
        shots.append(bg.resize((FW * 2, FH * 2), Image.NEAREST))
    shots[0].save(
        os.path.join(OUT, "preview_meteor.gif"),
        save_all=True,
        append_images=shots[1:] + [shots[-1]] * 10,
        duration=FRAME_MS,
        loop=0,
        disposal=2,
    )
    cols = 11
    sheet = Image.new("RGBA", (FW * cols, FH * ((len(frames) + cols - 1) // cols)), (38, 42, 58, 255))
    for i, im in enumerate(shots):
        sheet.alpha_composite(im.resize((FW, FH), Image.NEAREST), ((i % cols) * FW, (i // cols) * FH))
    sheet.save(os.path.join(OUT, "preview_meteor_frames.png"))


if __name__ == "__main__":
    save(render())
