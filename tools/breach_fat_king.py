"""Король-толстосум: золотой король с большим пузом и свисающими украшениями — в стиле ITB.

Основа — золотой король из `breach_gilded.py` (те же палитры: белые — жёлтое золото с
белой эмалью и сапфирами, чёрные — красное золото с чёрной эмалью и рубинами). Вместо
тонкой ножки — широкое круглое брюхо от самой подставки, вперёд выпирает живот (эллипсоид,
плавно слитый с корпусом), под ним пояс эмали с крупной пряжкой-камнем. Украшения
свисают: две нитки ожерелий провисают дугами по пузу (крупный жемчуг; камни через
жемчужину с большим медальоном-камнем в оправе), с боков короны — нитки жемчуга с
каплями-камнями (как у византийских корон), с пояса — короткие подвески с камешками.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_fat_king.py [папка]` — по умолчанию assets/breach/pieces:
fat_king/<w|b>K.png и <w|b>K_shadow.png, превью preview_fat_king.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_gilded as gd  # noqa: E402
from breach_art import MAIN, TRIM, GLOW, T, Y, R, hexc, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

FILI, GEM, PEARL = gd.FILI, gd.GEM, gd.PEARL
FRONT = gd.FRONT

# профиль: подставка, брюхо от самой подставки, воротник, голова и корона — как у короля
PROFILE = art.base(15.2) + [
    (9.6, 7.6),  # снизу живот подворачивается к подставке
    (11.8, 9.0),
    (14.6, 11.6),
    (16.3, 15.0),
    (16.7, 18.2),
    (15.7, 21.9),
    (13.3, 25.1),
    (10.8, 27.3),
    (10.2, 28.2),
    (10.2, 30.6),
    (7.0, 31.6),
    (7.6, 34.0),
    (9.4, 41.0),
    (10.6, 46.2),
    (11.2, 46.8),
    (11.2, 48.8),
    (8.4, 49.6),
] + art.arc(0, 49.6, 8.4, 0, 90, 10)
BELLY_C = T * 4.6 + Y * 17.0  # живот выпирает вперёд — к зрителю
BELLY_R = np.array([11.8, 8.4, 11.8])
BELT = (9.4, 11.0)  # пояс под пузом


def smin(a, b, k):
    """Плавное объединение расстояний: живот сливается с корпусом без складки."""
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0, 1)
    return b * (1 - h) + a * h - k * h * (1 - h)


def ellipsoid(c, radii, axes=None):
    c = np.asarray(c, float)
    radii = np.asarray(radii, float)
    ax = np.eye(3) if axes is None else np.asarray(axes, float)

    def f(p):
        q = (p - c) @ ax.T / radii
        return (np.linalg.norm(q, axis=1) - 1) * radii.min()

    return f


def body_sdf():
    lathe = art.lathe(PROFILE)
    belly = ellipsoid(BELLY_C, BELLY_R)
    return lambda p: smin(lathe(p), belly(p), 3.0)


BODY = body_sdf()


def surface_r(y, az, r_max=24.0):
    """Радиус поверхности тела на высоте y и азимуте az (по лучу от оси)."""
    a = math.radians(az)
    d = np.array([math.cos(a), 0.0, math.sin(a)])
    rs = np.linspace(0, r_max, 241)
    pts = np.outer(rs, d) + np.array([0, y, 0])
    inside = BODY(pts) < 0
    idx = np.nonzero(inside)[0]
    return rs[idx.max()] if len(idx) else 0.0


def on_surface(y, az, lift):
    a = math.radians(az)
    r = surface_r(y, az) + lift
    return np.array([r * math.cos(a), y, r * math.sin(a)])


def body_mat(p):
    """Корпус — золото; эмаль — воротник, корона и пояс; узор — кольцо на чаше."""
    y = p[:, 1]
    out = np.full(len(p), MAIN)
    for lo, hi in (art.BASE_BANDS, (27.4, 31.7), (46.4, 49.0), BELT):
        out[(y >= lo) & (y < hi)] = TRIM
    out[np.abs(y - 41.0) < 0.5] = FILI
    return out


def necklace(y_top, sag, span, bead, mats, step, lift=0.75, pendant=None):
    """Нитка бус, провисающая дугой по пузу: от азимутов FRONT ± span на высоте y_top вниз
    на sag в середине. mats — материалы бусин по кругу; pendant — медальон в нижней точке."""
    parts = []
    n = max(3, int(2 * span / step))
    for i in range(n + 1):
        t = -1 + 2 * i / n
        az = FRONT + span * t
        y = y_top - sag * (1 - t * t)
        parts.append((sphere(on_surface(y, az, lift + bead * 0.5), bead), mats[i % len(mats)]))
    if pendant:
        size = pendant
        y = y_top - sag - size * 1.3
        c = on_surface(y, FRONT, 1.1)
        out = norm(np.array([c[0], 0, c[2]]))
        side = norm(np.cross(Y, out))
        axes = [side, Y, out]
        parts.append((ellipsoid(c, [size * 1.25, size * 1.45, 0.9], axes), MAIN))  # оправа
        parts.append((ellipsoid(c + out * 0.6, [size * 0.85, size, 0.8], axes), GEM))  # камень
        parts.append((sphere(c + Y * (size * 1.45 + 0.4), 0.7), MAIN))  # ушко
    return parts


def dangle(top, length, bead, mats, drop):
    """Подвеска: бусины вниз от точки, на конце капля-камень."""
    parts = []
    k = max(2, int(length / (bead * 2.1)))
    for i in range(k):
        parts.append((sphere(top - Y * (i * bead * 2.1), bead), mats[i % len(mats)]))
    end = top - Y * (k * bead * 2.1 + drop * 0.6)
    parts.append((ellipsoid(end, [drop * 0.8, drop * 1.2, drop * 0.8]), GEM))
    return parts


def fat_king():
    axes = [R, Y, T]
    cross_v = art.rbox([0, 63.6, 0], [2.1, 7.4, 2.1], axes, 0.5)
    cross_h = art.rbox([0, 65.4, 0], [6.4, 2.1, 2.1], axes, 0.5)

    def cross_mat(p):
        m = np.full(len(p), TRIM)  # крест — эмаль стороны, как у золотого короля
        gem = (np.abs(p @ R) < 1.5) & (np.abs(p[:, 1] - 65.4) < 1.5) & ((p @ T) > 1.0)
        m[gem] = GEM
        return m

    parts = [(BODY, body_mat), (cross_v, cross_mat), (cross_h, cross_mat)]
    parts += gd.pearls("K")
    parts += gd.ring(29.4, 10.5, 5, 1.85, GEM)  # камни на воротнике
    parts += gd.ring(47.7, 11.45, 8, 1.5, GEM)  # камни на короне
    # пряжка пояса — крупный камень в оправе
    c = on_surface(10.4, FRONT, 0.6)
    out = norm(np.array([c[0], 0, c[2]]))
    side = norm(np.cross(Y, out))
    parts += [(ellipsoid(c, [2.6, 2.0, 1.0], [side, Y, out]), MAIN),
              (ellipsoid(c + out * 0.6, [1.7, 1.3, 0.8], [side, Y, out]), GEM)]
    # ожерелья по пузу: короткое жемчужное, среднее жемчуг через камень, длинное золотое с медальоном
    # крупные бусины — мелкие на пузе рассыпаются крошкой
    parts += necklace(26.8, 4.0, 50, 0.95, [PEARL], 12)
    parts += necklace(26.2, 5.6, 62, 1.05, [GEM, PEARL], 13, pendant=2.9)
    # подвески с короны (по бокам) и с пояса
    for az in (FRONT - 70, FRONT + 70, FRONT - 140, FRONT + 140):
        a = math.radians(az)
        top = np.array([11.6 * math.cos(a), 46.2, 11.6 * math.sin(a)])
        parts += dangle(top, 7.0, 0.8, [PEARL], 1.5)
    for az in (FRONT - 38, FRONT + 38):
        top = on_surface(9.6, az, 0.9)
        parts += dangle(top, 1.8, 0.85, [PEARL], 1.3)
    return art.Piece(parts)


def render(color):
    key = f"fat_{color}"
    art.PALETTES[key] = gd.PALETTES["gold"][color]
    body, shadow = art.render_piece(fat_king(), key, 0.86)
    img = np.array(body)
    top = np.array(hexc(gd.PALETTES["gold"][color][MAIN][4]))
    bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:3]:
        img[y, x, :3] = (255, 255, 255)
    return Image.fromarray(img, "RGBA"), shadow


def preview(sprites, path):
    """Слева направо: золотой король и толстосум — белые, затем чёрные; ниже — на доске."""
    cells = []
    for color in "wb":
        cells.append(Image.open(os.path.join(OUT, "gold", f"{color}K.png")).convert("RGBA"))
        cells.append(sprites[color][0])
    cw = 48
    sheet = Image.new("RGBA", (4 * cw + 8, 84), hexc("#3d4a3a") + (255,))
    for i, im in enumerate(cells):
        sheet.alpha_composite(im.crop((8, 0, 56, 84)), (4 + i * cw, 0))
    sheet = sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST)
    # на доске: оба толстосума среди золотых фигур
    gold = {}
    for color in "wb":
        for k in "PNBRQK":
            gold[color + k] = (Image.open(os.path.join(OUT, "gold", f"{color}{k}.png")).convert("RGBA"),
                               Image.open(os.path.join(OUT, "gold", f"{color}{k}_shadow.png")).convert("RGBA"))
        gold[color + "K"] = sprites[color]
    board = art.board_preview(gold, scale=2, demo=False)
    w = max(sheet.width, board.width)
    out = Image.new("RGBA", (w, sheet.height + board.height), hexc("#15171f") + (255,))
    out.alpha_composite(sheet, ((w - sheet.width) // 2, 0))
    out.alpha_composite(board, ((w - board.width) // 2, sheet.height))
    out.save(path)


def main():
    os.makedirs(os.path.join(OUT, "fat_king"), exist_ok=True)
    sprites = {}
    for color in "wb":
        body, shadow = render(color)
        body.save(os.path.join(OUT, "fat_king", f"{color}K.png"))
        shadow.save(os.path.join(OUT, "fat_king", f"{color}K_shadow.png"))
        sprites[color] = (body, shadow)
        print(color + "K", flush=True)
    preview(sprites, os.path.join(OUT, "preview_fat_king.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
