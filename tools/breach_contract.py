"""Арт карты «Договор» в стиле Into the Breach: свиток с печатью, несколько вариантов.

Картинка — окно лица карты 38×25 (как у огненной волны и обмена). Свиток трёх видов:
`scroll` — пергамент с валиками сверху и снизу, рваные края, красная лента, восковая
печать (по образцу пользователя); `sheet` — плоский рваный лист под наклоном; `dark` —
тёмный «кровавый договор»: старый пергамент, красные письмена, чёрная печать. Фоны:
пусто, стол (дерево, перо с чернильницей или свеча), кровь (брызги, потёки, отпечаток).

`py tools/breach_contract.py` — assets/breach/cards/art/contract_<вариант>.png и
preview_contract.png (варианты на лицах всех типов карт).
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_cards as bc  # noqa: E402
import breach_firewave as fw  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
ART_DIR = os.path.join(art.ROOT, "assets", "breach", "cards", "art")
W, H = fw.ART_SIZE
YY, XX = np.mgrid[0:H, 0:W]
PX, PY = XX + 0.5, YY + 0.5

OUT = "#140c06"  # контур — тёмно-коричневый почти чёрный
PAPER = ["#fbf0cf", "#efdcab", "#dcbf86", "#b8935a", "#8a6436"]  # блик … тень
PAPER_OLD = ["#e2c690", "#c8a46a", "#a8824c", "#82602e", "#5a3e1c"]
INK, SIGN = "#8a6a44", "#3a2414"
RIBBON = ["#ff5a52", "#c8242c", "#7a1018"]
SEAL = ["#c0503e", "#8a2a1e", "#561410"]
SEAL_BLACK = ["#4a4450", "#24202a", "#0e0c12"]
BLOOD = ["#d8323c", "#9a121c", "#5a080e"]
WOOD = ["#8e6440", "#7e5634", "#6e4a2c", "#5a3a22", "#2e1c10"]  # свет … шов


def paint(img, mask, color, alpha=255):
    img[mask] = (*hexc(color), alpha)


def ellipse(cx, cy, rx, ry):
    return ((PX - cx) / rx) ** 2 + ((PY - cy) / ry) ** 2 <= 1.0


def polygon(points):
    return art.sd_polygon(PX, PY, points) <= 0


def stroke(points, width):
    """Маска ломаной толщиной width."""
    d = np.full((H, W), np.inf)
    for a, b in zip(points, points[1:]):
        d = np.minimum(d, art.sd_segment2(PX, PY, a, b))
    return d <= width / 2


def outline(img, mask, color=OUT):
    paint(img, bc.outline4(mask), color)


# ---------------------------------------------------------------- фоны
def bg_plain(img, rng):
    """Пусто: тёмная синева с мягкой виньеткой, как окно обычной карты."""
    paint(img, YY >= 0, "#161a27")
    r = np.hypot((PX - W / 2) / W, (PY - H / 2) / H)
    paint(img, r > 0.42, "#11141f")
    paint(img, (r > 0.36) & (r <= 0.42) & ((XX + YY) % 2 == 0), "#11141f")
    paint(img, r < 0.2, "#1c2130")


def bg_table(img, rng, glow=None):
    """Стол: доски поперёк, швы, прожилки; glow — тёплый отсвет от свечи (x, y)."""
    plank = (YY // 9) % 3
    for k, col in enumerate((WOOD[2], WOOD[3], WOOD[1])):
        paint(img, plank == k, col)
    paint(img, YY % 9 == 8, WOOD[4])  # швы между досками
    for _ in range(14):  # прожилки
        y = int(rng.integers(0, H))
        if y % 9 == 8:
            continue
        x0 = int(rng.integers(0, W - 6))
        ln = int(rng.integers(3, 9))
        img[y, x0 : x0 + ln] = (*hexc(WOOD[0] if rng.random() < 0.5 else WOOD[3]), 255)
    if glow is not None:
        r = np.hypot(PX - glow[0], (PY - glow[1]) * 1.3)
        warm = np.array(hexc("#ffcf7a"), float)
        for rr, k in ((16, 0.12), (10, 0.2), (5, 0.3)):
            m = (r < rr) & ((XX + YY) % 2 == 0 if rr == 16 else r < rr)
            img[m, :3] = (img[m, :3] * (1 - k) + warm * k).astype(np.uint8)
    # тень под краем окна сверху — стол уходит в темноту
    img[0:2, :, :3] = (img[0:2, :, :3] * 0.7).astype(np.uint8)


def bg_blood(img, rng):
    """Кровь: тёмный фон с багровым отсветом снизу."""
    paint(img, YY >= 0, "#120c10")
    paint(img, YY >= 15, "#1c0e12")
    paint(img, (YY == 14) & ((XX + YY) % 2 == 0), "#1c0e12")
    paint(img, (YY >= 21), "#26101a")


# ---------------------------------------------------------------- свитки
def torn(rng, n, lo, hi):
    """Неровный край: смещение 0/1 пиксель по строкам."""
    return rng.integers(lo, hi + 1, size=n)


def scroll(img, rng, paper=PAPER, text=INK, seal=SEAL, x0=10, x1=27, y0=2, y1=22):
    """Пергамент с валиками сверху и снизу, лентой и печатью (по образцу)."""
    p = paper
    sheet = np.zeros((H, W), bool)
    left, right = torn(rng, H, x0 - 1, x0), torn(rng, H, x1, x1 + 1)
    for y in range(y0 + 3, y1 - 2):
        sheet[y, left[y] : right[y] + 1] = True
    rolls = np.zeros((H, W), bool)
    for ry in (y0, y1 - 3):
        rolls[ry : ry + 4, x0 - 2 : x1 + 3] = True
        for dy in (0, 2):  # загнутые рваные концы валиков
            rolls[ry + dy, x0 - 3] = True
            rolls[ry + dy + 1, x1 + 3] = True
    body = sheet | rolls
    outline(img, body)
    # лист: светлый, тень под верхним валиком и у правого края, строчки и подпись
    paint(img, sheet, p[1])
    paint(img, sheet & (YY == y0 + 4), p[3])
    paint(img, sheet & (YY == y0 + 5), p[2])
    paint(img, sheet & (XX >= right[YY] - 1), p[2])
    paint(img, sheet & (XX <= left[YY]), p[0])
    for ty in range(y0 + 7, y1 - 6, 2):
        for x in range(x0 + 2, x1 - 1):
            if (x * 7 + ty * 3) % 5 != 0 and sheet[ty, x]:
                img[ty, x] = (*hexc(text), 255)
    for k, x in enumerate(range(x0 + 2, x0 + 9)):  # подпись-росчерк
        img[y1 - 5 - (k % 3 == 1), x] = (*hexc(SIGN), 255)
    # валики: блик сверху, тень снизу, торцы темнее, на торцах завиток
    for ry in (y0, y1 - 3):
        band = rolls & (YY >= ry) & (YY < ry + 4)
        paint(img, band, p[1])
        paint(img, band & (YY == ry), p[0])
        paint(img, band & (YY == ry + 2), p[2])
        paint(img, band & (YY == ry + 3), p[3])
        paint(img, band & ((XX <= x0 - 2) | (XX >= x1 + 2)), p[3])
        img[ry + 1, x0 - 2] = (*hexc(p[4]), 255)
        img[ry + 2, x1 + 2] = (*hexc(p[4]), 255)
    # лента на верхнем валике и хвост влево-вниз
    band = (XX >= x1 - 6) & (XX <= x1 - 5) & (YY >= y0) & (YY <= y0 + 3)
    tail = stroke([(x0 + 2.5, y0 + 2.5), (x0 - 1.5, y0 + 6.5), (x0 - 3.5, y0 + 8.5)], 1.6)
    rib = band | tail
    outline(img, rib)
    paint(img, rib, RIBBON[1])
    paint(img, rib & ~np.roll(rib, 1, 0), RIBBON[0])
    paint(img, rib & ~np.roll(rib, -1, 0), RIBBON[2])
    # печать с хвостами ленты внизу справа
    cx, cy = x1 - 2.5, y1 - 3.0
    tails = stroke([(cx - 1, cy + 2), (cx - 2, cy + 5)], 1.6) | stroke([(cx + 1, cy + 2), (cx + 2.5, cy + 5)], 1.6)
    outline(img, tails)
    paint(img, tails, RIBBON[1])
    paint(img, tails & (XX < cx), RIBBON[0])
    wax_seal(img, cx, cy, seal)


def wax_seal(img, cx, cy, seal, r=3.3):
    """Восковая печать: зубчатый край, тиснёное кольцо, блик сверху-слева."""
    ang = np.arctan2(PY - cy, PX - cx)
    rr = np.hypot(PX - cx, PY - cy)
    disk = rr <= r + 0.45 * np.cos(ang * 8)
    outline(img, disk)
    paint(img, disk, seal[1])
    paint(img, disk & (rr > r - 0.6), seal[2])
    paint(img, disk & (np.abs(rr - 1.6) < 0.5), seal[0])
    paint(img, disk & (PX < cx) & (PY < cy) & (rr > r - 1.2) & (rr <= r), seal[0])
    img[int(cy), int(cx)] = (*hexc(seal[2]), 255)


def sheet(img, rng, tilt=-9, paper=PAPER):
    """Плоский рваный лист под наклоном, загнутый уголок, печать и лента внизу."""
    p = paper
    cx, cy, hw, hh = 19.0, 12.5, 9.0, 10.5
    a = math.radians(tilt)
    corners = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
    pts = []
    for (ax, ay), (bx, by) in zip(corners, corners[1:] + corners[:1]):
        for k in range(6):  # рваные края: точки с дрожью
            t = k / 6
            x, y = ax + (bx - ax) * t, ay + (by - ay) * t
            j = rng.uniform(-0.6, 0.6)
            x, y = x + (j if ay == by else 0), y + (j if ax == bx else 0)
            pts.append((cx + x * math.cos(a) - y * math.sin(a), cy + x * math.sin(a) + y * math.cos(a)))
    body = polygon(pts)
    outline(img, body)
    paint(img, body, p[1])
    lit, shaded = bc.shade_mask(body)
    paint(img, shaded, p[3])
    paint(img, lit, p[0])
    # строчки, наклонённые вместе с листом
    u = (PX - cx) * math.cos(a) + (PY - cy) * math.sin(a)
    v = -(PX - cx) * math.sin(a) + (PY - cy) * math.cos(a)
    for k in range(5):
        line = body & (np.abs(v - (-7 + 2.6 * k)) < 0.5) & (np.abs(u) < 6.5) & ((XX * 5 + YY) % 6 != 0)
        paint(img, line, INK)
    paint(img, body & (np.abs(v - 6.2) < 0.5) & (u > -6) & (u < -1), SIGN)
    # загнутый уголок справа сверху
    tip = (cx + hw * math.cos(a) + hh * math.sin(a), cy + hw * math.sin(a) - hh * math.cos(a))
    curl = polygon([(tip[0] - 3.5, tip[1] + 0.3), (tip[0] + 0.2, tip[1] + 0.2), (tip[0] - 0.3, tip[1] + 3.6)])
    paint(img, curl & body, p[3])
    paint(img, curl & body & (PX + PY < tip[0] + tip[1] + 0.5), p[4])
    # печать с лентой внизу посередине
    sx, sy = cx - 1.0, cy + 7.5
    tails = stroke([(sx - 1, sy + 2), (sx - 2.5, sy + 5)], 1.6) | stroke([(sx + 1, sy + 2), (sx + 2.5, sy + 5)], 1.6)
    outline(img, tails)
    paint(img, tails, RIBBON[1])
    paint(img, tails & (XX < sx), RIBBON[0])
    wax_seal(img, sx, sy, SEAL)


def dark_scroll(img, rng):
    """«Кровавый договор»: старый пергамент, красные письмена, чёрная печать."""
    scroll(img, rng, paper=PAPER_OLD, text="#a01820", seal=SEAL_BLACK)


# ---------------------------------------------------------------- предметы и кровь
def quill(img):
    """Перо: белое опахало наискось, тёмный кончик в чернильнице справа."""
    # опахало — вытянутый лист вдоль стержня, с косыми бородками
    ax, ay, bx, by = 35.8, 1.5, 31.2, 15.5
    dx, dy = bx - ax, by - ay
    ln = math.hypot(dx, dy)
    ux, uy = dx / ln, dy / ln
    t = (PX - ax) * ux + (PY - ay) * uy
    s = -(PX - ax) * uy + (PY - ay) * ux
    width = 2.6 * np.sin(np.clip(t / (ln * 0.78), 0, 1) * math.pi) ** 0.8
    vane = (t > 0) & (t < ln * 0.78) & (np.abs(s) <= width)
    shaft = stroke([(ax, ay), (bx, by)], 1.0)
    outline(img, vane | shaft)
    paint(img, vane, "#e6e4d8")
    paint(img, vane & (s > 0.3), "#b4b2a4")  # правая половина в тени
    paint(img, vane & (np.abs((t + np.abs(s) * 1.4) % 2.2 - 1.1) < 0.35), "#9c9a8c")  # бородки
    paint(img, shaft, "#fbfaf2")
    paint(img, shaft & (t > ln * 0.78), "#3a3a3a")  # кончик в чернилах
    jar = ellipse(31.5, 19.0, 3.2, 3.6) | ((np.abs(PX - 31.5) <= 1.6) & (PY > 14.5) & (PY < 17))
    outline(img, jar)
    paint(img, jar, "#1c2430")
    paint(img, jar & (PX < 30.5) & (PY < 20), "#4a6a88")
    paint(img, jar & (PY > 21), "#0a0a14")


def candle(img):
    """Свеча слева: столбик воска с потёком, пламя."""
    wax = (np.abs(PX - 4.5) <= 1.6) & (PY > 9) & (PY < 24)
    outline(img, wax)
    paint(img, wax, "#efe6cf")
    paint(img, wax & (PX > 5.0), "#c9bd9c")
    img[10, 3] = (*hexc("#fbf6e6"), 255)
    flame = ellipse(4.5, 6.5, 1.3, 2.6)
    outline(img, flame, "#3a1a06")
    paint(img, flame, "#ffbc03")
    paint(img, flame & (PY > 6.5), "#fff3c4")
    paint(img, flame & (PY < 5.2), "#e47a1b")


def blood(img, rng, amount=1.0, on_paper=True, pool=False):
    """Брызги, капли и потёки; на пергаменте — темнее (впиталось)."""
    paper = (img[..., :3].astype(int).sum(axis=-1) > 450)
    marks = np.zeros((H, W), bool)
    for _ in range(int(5 * amount)):  # брызги — пятна с каплями вокруг
        cx, cy = rng.uniform(6, 32), rng.uniform(4, 21)
        r = rng.uniform(1.2, 2.4)
        marks |= ellipse(cx, cy, r, r * 0.85)
        for _ in range(4):
            a = rng.uniform(0, 2 * math.pi)
            d = r + rng.uniform(1.0, 3.0)
            marks |= ellipse(cx + math.cos(a) * d, cy + math.sin(a) * d, 0.6, 0.6)
    for _ in range(int(3 * amount)):  # потёки вниз
        x = int(rng.integers(10, 28))
        y0 = int(rng.integers(5, 12))
        ln = int(rng.integers(3, 8))
        marks[y0 : y0 + ln, x] = True
        marks[min(H - 1, y0 + ln), x] = True
        if x + 1 < W:
            marks[y0 + ln - 1 : y0 + ln + 1, x + 1] = True
    if pool:  # лужа на столе внизу
        marks |= ellipse(10, 23.5, 7, 2.2) | ellipse(15, 24.2, 4, 1.5)
    soaked = marks & paper if on_paper else np.zeros_like(marks)
    paint(img, marks & ~paper, BLOOD[1])
    paint(img, marks & ~paper & ~np.roll(marks, 1, 0), BLOOD[0])
    paint(img, soaked, "#7a1018")
    paint(img, soaked & ~np.roll(marks, -1, 0), BLOOD[2])


def fingerprint(img, cx, cy):
    """Кровавый отпечаток пальца — подпись на договоре."""
    rr = np.hypot((PX - cx) / 2.2, (PY - cy) / 2.8)
    paint(img, rr <= 1.0, "#9a121c")
    paint(img, (rr <= 1.0) & (np.abs((rr * 3.2) % 1 - 0.5) < 0.22), "#5a080e")


# ---------------------------------------------------------------- варианты
def v_empty(img, rng):
    bg_plain(img, rng)
    scroll(img, rng)


def v_table(img, rng):
    bg_table(img, rng)
    scroll(img, rng)
    quill(img)


def v_blood(img, rng):
    bg_blood(img, rng)
    scroll(img, rng)
    blood(img, rng, amount=0.9)


def v_candle(img, rng):
    bg_table(img, rng, glow=(4.5, 6.5))
    sheet(img, rng)
    candle(img)


def v_pact(img, rng):
    bg_blood(img, rng)
    dark_scroll(img, rng)
    fingerprint(img, 13.5, 17.0)
    blood(img, rng, amount=0.6, on_paper=False)


def v_table_blood(img, rng):
    bg_table(img, rng)
    scroll(img, rng)
    blood(img, rng, amount=0.9, pool=True)


VARIANTS = [
    ("empty", "пусто", v_empty),
    ("table", "на столе", v_table),
    ("blood", "в крови", v_blood),
    ("candle", "лист и свеча", v_candle),
    ("pact", "кровавый договор", v_pact),
    ("table_blood", "стол и кровь", v_table_blood),
]


def render(fn, seed=7):
    rng = np.random.default_rng(seed)
    img = np.zeros((H, W, 4), np.uint8)
    fn(img, rng)
    img[..., 3] = 255
    return img


def save_all():
    os.makedirs(ART_DIR, exist_ok=True)
    arts = []
    for name, _, fn in VARIANTS:
        a = render(fn)
        Image.fromarray(a, "RGBA").save(os.path.join(ART_DIR, f"contract_{name}.png"))
        arts.append(a)
    kinds = ("w", "b", "victory", "sacrifice")
    gap = 10
    sheet_img = Image.new("RGBA", (len(kinds) * (bc.CW + gap) + gap + 2 * W + gap,
                                   len(arts) * (bc.CH + gap) + gap), hexc("#1b1e27") + (255,))
    for r, a in enumerate(arts):
        y = gap + r * (bc.CH + gap)
        big = Image.fromarray(a, "RGBA").resize((W * 2, H * 2), Image.NEAREST)
        sheet_img.alpha_composite(big, (gap, y + (bc.CH - H * 2) // 2))
        for k, kind in enumerate(kinds):
            face = bc.card_front(kind).copy()
            x0, y0 = fw.ART_AT
            face[y0 : y0 + H, x0 : x0 + W] = a
            x = gap + W * 2 + gap + k * (bc.CW + gap)
            sheet_img.alpha_composite(Image.fromarray(bc.shadow_of(face), "RGBA"), (x, y))
            sheet_img.alpha_composite(Image.fromarray(face, "RGBA"), (x, y))
    sheet_img.resize((sheet_img.width * 3, sheet_img.height * 3), Image.NEAREST).save(
        os.path.join(ART_DIR, "preview_contract.png")
    )


if __name__ == "__main__":
    save_all()
    print("done", ART_DIR)
