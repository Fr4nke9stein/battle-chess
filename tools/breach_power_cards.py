"""Карты сил rogue-like (0.111, просьба пользователя: «пусть на них будет нарисован знак типа этих
фигур») в стиле игры: тёмное поле в цвет силы, рамка тёмного металла с золотыми уголками и
ромбами, посередине — знак силы из генератора гербов (`breach_crest.crest(знак, bare=True)`):
огонёк, молния, смерч, золотой король, чёрно-фиолетовый огонёк, снежинка, топор, шляпа мага.

Карта 51×70 с контуром, тени нет: `ui/card_<сила>.png` (fire, spark, wind, gold, shade, ice,
barbarian, wizard — порядок `Power` в игре). С 0.112 ещё семь карт со своими знаками: `mask` (маска),
`crest` (маленький герб со звездой), `double` (две монеты), `coin` (монета), `mirror` (ручное
зеркало), `rank` (стрелка вверх), `feather` (перо). Превью — `ui/preview_power_cards.png` (×4).

    py tools/breach_power_cards.py [папка assets/breach/ui]
"""

import os
import sys

import numpy as np
from PIL import Image

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "assets", "breach", "ui")
sys.argv = sys.argv[:1] + [OUT]  # гербам — та же папка (монета для знаков с монетой)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import breach_crest as bc  # noqa: E402

W, H = 51, 70
POWERS = [
    ("fire", "flame", "#5a1c12"),
    ("spark", "bolt", "#16284a"),
    ("wind", "wind", "#123e40"),
    ("gold", "gold_king", "#4a3410"),
    ("shade", "dark_flame", "#2e1648"),
    ("ice", "snow", "#12344e"),
    ("barbarian", "axe", "#45291a"),
    ("wizard", "wizard_hat", "#221e56"),
]


def tones(base):
    r, g, b = bc.hexc(base)
    scale = (1.35, 1.0, 0.72, 0.45)
    return [tuple(min(255, int(c * k)) for c in (r, g, b)) for k in scale]


def card(emblem, base):
    img = np.zeros((H, W, 4), np.uint8)
    yy, xx = np.mgrid[0:H, 0:W]

    def put(mask, col):
        img[mask] = (*col, 255)

    # силуэт: прямоугольник со срезанными уголками
    shape = np.ones((H, W), bool)
    for x, y in ((0, 0), (W - 1, 0), (0, H - 1), (W - 1, H - 1)):
        shape[y, x] = False
    put(shape, bc.OUTLINE)
    inside = (xx >= 1) & (xx <= W - 2) & (yy >= 1) & (yy <= H - 2)
    field = (xx >= 3) & (xx <= W - 4) & (yy >= 3) & (yy <= H - 4)
    frame = inside & ~field
    # рамка: тёмный металл, свет сверху слева
    put(frame, bc.IRON[2])
    put(frame & ((xx <= 2) | (yy <= 2)), bc.IRON[1])
    put(frame & ((xx == 1) | (yy == 1)) & (xx < W - 2) & (yy < H - 2), bc.IRON[0])
    put(frame & ((xx >= W - 3) | (yy >= H - 3)) & (xx > 2) & (yy > 2), bc.IRON[3])
    edge = frame & ((xx == 2) | (xx == W - 3) | (yy == 2) | (yy == H - 3)) & (xx >= 2) & (yy >= 2) & (
        xx <= W - 3) & (yy <= H - 3)
    put(edge, bc.IRON[4])
    # поле — в цвет силы, к свету светлее, у рамки тень
    t = tones(base)
    g = (xx - 3) / (W - 6) + (yy - 3) / (H - 6)
    put(field, t[1])
    put(field & (g < 0.55), t[0])
    put(field & (g > 1.35), t[2])
    put(field & ((xx == 3) | (yy == 3)), t[3])
    # знак силы посередине
    mark = emblem() if callable(emblem) else bc.crest(emblem, bare=True)
    mh, mw = mark.shape[:2]
    ox, oy = (W - mw) // 2, (H - mh) // 2
    for y in range(mh):
        for x in range(mw):
            if mark[y, x, 3] and field[oy + y, ox + x]:
                img[oy + y, ox + x] = mark[y, x]
    # золотые уголки и ромбы на рамке сверху и снизу
    for cx, cy, sx, sy in ((1, 1, 1, 1), (W - 2, 1, -1, 1), (1, H - 2, 1, -1), (W - 2, H - 2, -1, -1)):
        for d in range(4):
            for k in range(4 - d):
                x, y = cx + sx * k, cy + sy * d
                img[y, x] = (*(bc.GOLD[1] if d + k < 2 else bc.GOLD[3]), 255)
    for cy in (2, H - 3):
        mid = W // 2
        for dx, dy, col in ((0, -1, bc.GOLD[1]), (-1, 0, bc.GOLD[2]), (0, 0, bc.GOLD[0]), (1, 0, bc.GOLD[3]),
                            (0, 1, bc.GOLD[3]), (-2, 0, bc.GOLD[4]), (2, 0, bc.GOLD[4])):
            img[cy + dy, mid + dx] = (*col, 255)
    return img


# ----- Знаки остальных карт (0.112): холст знака 47×43, как у `bc.crest(..., bare=True)` -----

SW, SH = 47, 43
CX = 23.5
SHADE = bc.hexc("#120306")  # кайма и тень знака — как у гербов (`WINE[4]`)


def ramp(*colors):
    return [bc.hexc(c) for c in colors]


PURPLE = ramp("#e0b8ff", "#b07ae8", "#7e48c0", "#542a8a", "#2e1450")
GREEN = ramp("#d8ffb0", "#8ee060", "#4fb03a", "#2e7a28", "#174a18")
GLASS = ramp("#ffffff", "#d8f4ff", "#9cdcf4", "#5aa8cc", "#2e6688")
QUILL = ramp("#ffffff", "#ecebe4", "#c9c6ba", "#97938a", "#5e5a54")
BLUE_INK = ramp("#a8d0ff", "#5a8ee0", "#2e5cae")


def grid():
    yy, xx = np.mgrid[0:SH, 0:SW]
    return xx + 0.5, yy + 0.5


def ring_of(mask):
    sh = bc.shifted
    return (sh(mask, 1, 0) | sh(mask, -1, 0) | sh(mask, 0, 1) | sh(mask, 0, -1)) & ~mask


def edge_of(mask):
    sh = bc.shifted
    return mask & ~(sh(mask, 1, 0) & sh(mask, -1, 0) & sh(mask, 0, 1) & sh(mask, 0, -1))


def shaded(img, mask, tones_):
    """Тело знака: середина, тень справа-снизу, свет слева-сверху, кайма и тень на поле."""
    sh = bc.shifted
    drop = (sh(mask, 1, 1) | sh(mask, 2, 1)) & ~mask
    img[ring_of(mask) | drop] = (*SHADE, 255)
    img[mask] = (*tones_[2], 255)
    img[mask & ~(sh(mask, -1, 0) & sh(mask, 0, -1))] = (*tones_[3], 255)
    img[mask & ~(sh(mask, 1, 0) & sh(mask, 0, 1))] = (*tones_[1], 255)


def blank():
    return np.zeros((SH, SW, 4), np.uint8)


def sym_mask():
    """Карнавальная маска: полоса с рожками вверх по краям, прорези глаз, вырез под нос."""
    x, y = grid()
    img = blank()
    cy = 22.0
    dx, dy = np.abs(x - CX), y - cy
    body = ((dx / 15.0) ** 2 + (dy / 7.5) ** 2 <= 1) | (
        (dx > 9) & (dx < 16.5) & (dy < 1) & (dy > -6 - (dx - 9) * 0.9))
    eyes = (((dx - 6.0) / 3.8) ** 2 + ((dy + 0.5) / 2.3) ** 2) <= 1
    nose = ((dx / 3.0) ** 2 + ((dy - 7.5) / 3.4) ** 2) <= 1
    body &= ~eyes & ~nose
    shaded(img, body, PURPLE)
    edge = edge_of(body)
    img[edge] = (*bc.GOLD[3], 255)
    img[edge & (y < cy)] = (*bc.GOLD[1], 255)
    img[eyes & (np.abs(dy + 0.5) < 3)] = (*SHADE, 255)
    for sx, sy in ((15, 18), (32, 18), (23, 17)):
        img[sy, sx] = (*bc.GOLD[0], 255)
    return img


def sym_crest():
    """Маленький герб: щит игры поменьше, железный обод, бордовое поле, золотая звезда."""
    x, y = grid()
    img = blank()
    k = 1.55

    def scaled(inset):
        return bc.coverage(lambda px, py: bc.shield(CX + (px - CX) * k, 21.5 + (py - 21.5) * k, inset), SW, SH)

    body, field = scaled(0.0), scaled(2.6)
    shaded(img, body, bc.IRON)
    img[field] = (*bc.WINE[1], 255)
    img[field & (x + y > 50)] = (*bc.WINE[2], 255)
    ang = np.arctan2(y - 20.5, x - CX)
    r = np.hypot(x - CX, y - 20.5)
    star = (r < 2.6 + 3.8 * (0.5 + 0.5 * np.cos(5 * (ang + np.pi / 2))) ** 2) & field
    img[star] = (*bc.GOLD[2], 255)
    img[star & (y < 20.5)] = (*bc.GOLD[1], 255)
    img[19, 23] = (*bc.GOLD[0], 255)
    return img


def paste_coin(img, name, left, top):
    coin = np.array(Image.open(os.path.join(OUT, name)).convert("RGBA"))
    h, w = coin.shape[:2]
    mask = np.zeros((SH, SW), bool)
    mask[top:top + h, left:left + w] = coin[:, :, 3] > 0
    sh = bc.shifted
    img[(sh(mask, 1, 1) | sh(mask, 2, 1)) & ~mask] = (*SHADE, 255)
    sub = img[top:top + h, left:left + w]
    sub[coin[:, :, 3] > 0] = coin[coin[:, :, 3] > 0]
    return img


def sym_double():
    """Две монеты: задняя повыше слева, передняя ниже справа."""
    img = blank()
    paste_coin(img, "coin_24.png", 7, 5)
    paste_coin(img, "coin_24.png", 16, 13)
    return img


def sym_coin():
    return paste_coin(blank(), "coin_32.png", 8, 5)


def sym_mirror():
    """Ручное зеркало: золотая овальная рама, голубое стекло с бликами, ручка с навершием."""
    x, y = grid()
    img = blank()
    cy = 15.5
    frame = (((x - CX) / 10.0) ** 2 + ((y - cy) / 12.0) ** 2) <= 1
    glass = (((x - CX) / 7.6) ** 2 + ((y - cy) / 9.6) ** 2) <= 1
    handle = (np.abs(x - CX) < 1.6) & (y > cy + 11) & (y < cy + 21.5)
    knob = np.hypot(x - CX, y - (cy + 22.5)) < 2.4
    neck = (np.abs(x - CX) < 3.0) & (y > cy + 10.5) & (y < cy + 13.5)
    shaded(img, frame | handle | knob | neck, bc.GOLD)
    img[glass] = (*GLASS[2], 255)
    img[glass & (x + y < CX + cy - 1)] = (*GLASS[1], 255)
    img[glass & (x + y > CX + cy + 7)] = (*GLASS[3], 255)
    shine = glass & (np.abs((x - CX) + (y - cy) + 4) < 1.0) & (y < cy + 2)
    shine |= glass & (np.abs((x - CX) + (y - cy) + 8.5) < 0.7) & (y < cy - 1)
    img[shine] = (*GLASS[0], 255)
    return img


def sym_rank():
    """Стрелка вверх: широкий наконечник, толстое древко, золотая кайма."""
    x, y = grid()
    img = blank()
    tip, base = 5.0, 21.0
    head = (y >= tip) & (y <= base) & (np.abs(x - CX) <= (y - tip) * 0.85)
    shaft = (np.abs(x - CX) < 4.5) & (y > base - 1) & (y < 38)
    arrow = head | shaft
    shaded(img, arrow, GREEN)
    edge = edge_of(arrow)
    img[edge] = (*bc.GOLD[3], 255)
    img[edge & (x < CX)] = (*bc.GOLD[1], 255)
    img[(np.abs(x - CX) < 0.6) & (y > 9) & (y < 35)] = (*GREEN[1], 255)
    return img


def sym_feather():
    """Перо: опахало вдоль изогнутого стержня снизу слева вверх вправо, синий кончик, золотой очин."""
    x, y = grid()
    img = blank()
    ax, ay, bx, by = 12.0, 38.0, 35.0, 4.0
    vx, vy = bx - ax, by - ay
    length = np.hypot(vx, vy)
    ux, uy = vx / length, vy / length
    s = ((x - ax) * ux + (y - ay) * uy) / length  # вдоль стержня 0..1
    d = (x - ax) * -uy + (y - ay) * ux - 2.2 * np.sin(np.pi * np.clip(s, 0, 1))  # поперёк
    width = 6.0 * np.clip((s - 0.18) / 0.25, 0, 1) * np.clip((1.02 - s) / 0.35, 0, 1) ** 0.6
    vane = (s > 0.18) & (s < 1.0) & (np.abs(d) < width)
    notch = (np.abs(d) > width * 0.7) & (np.mod(s * 9 + (d > 0) * 0.5, 1) < 0.14)
    vane &= ~notch
    stem = (s > 0) & (s < 1.0) & (np.abs(d) < 0.8)
    shaded(img, vane | stem, QUILL)
    img[vane & (s > 0.8)] = (*BLUE_INK[1], 255)
    img[vane & (s > 0.9)] = (*BLUE_INK[2], 255)
    img[stem & (s > 0.15)] = (*QUILL[3], 255)
    img[stem & (s <= 0.18)] = (*bc.GOLD[3], 255)
    return img


OTHERS = [
    ("mask", sym_mask, "#2a1440"),
    ("crest", sym_crest, "#3a1018"),
    ("double", sym_double, "#3e3010"),
    ("coin", sym_coin, "#3a2a0e"),
    ("mirror", sym_mirror, "#163848"),
    ("rank", sym_rank, "#183c1a"),
    ("feather", sym_feather, "#2a2c34"),
]


def main():
    os.makedirs(OUT, exist_ok=True)
    cards = []
    for name, emblem, base in POWERS + OTHERS:
        img = card(emblem, base)
        Image.fromarray(img, "RGBA").save(os.path.join(OUT, f"card_{name}.png"))
        cards.append(img)
    sheet = Image.new("RGBA", (len(cards) * (W + 4) + 4, H + 8), (24, 28, 41, 255))
    for k, img in enumerate(cards):
        sheet.alpha_composite(Image.fromarray(img, "RGBA"), (4 + k * (W + 4), 4))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(
        os.path.join(OUT, "preview_power_cards.png"))


if __name__ == "__main__":
    main()
