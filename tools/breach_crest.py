"""Герб с молнией — по эскизу пользователя, в стиле Into the Breach.

Щит симметричный: верх вогнут к середине, углы торчат вверх рожками, бока плавно сходятся к
острию снизу. Обод — тёмный металл с фаской (свет сверху-слева), на рожках — золотые
уголки-треугольники с искрой. Поле — очень тёмное бордовое, к свету чуть светлее, под
ободом — тень. Посередине — золотая молния-зигзаг: светлая кромка сверху-слева, тёмная
снизу-справа, тень на поле. Контур #020201. Размер — на 40% больше монеты `coin_32`
(просьба пользователя): 45 пикселей в высоту.

Второй вариант — тот же щит со снежинкой вместо молнии (шесть лучей с веточками, лёд —
бело-голубой).
Третий — молния и снежинка вместе, четвёртый — сердце и снежинка, пятый — монета и снежинка,
шестой — две снежинки,
седьмой — пешка и снежинка, восьмой — шипы и снежинка, девятый — снежинка в ледяном кристалле
(crest_both.png, crest_heart.png, crest_coin.png, crest_snow2.png, crest_pawn.png, crest_spikes.png,
crest_crystal.png). Топоры (просьба пользователя): топор и меч, топор и клыки, топор и ферзь, два топора, топор в
красном вихре, горящий топор (crest_axe_sword.png, crest_axe_fangs.png, crest_axe_queen.png, crest_axes.png,
crest_whirl.png, crest_fire_axe.png; все шесть рядом — preview_crest_axes.png).
Огонёк — пламя слоями посередине щита с отсветом на поле (crest_flame.png); огонёк и пешка, двойной огонёк,
огонёк и череп, огонёк и меч, огонёк и монета, падающая авиабомба (crest_flame_pawn.png, crest_flames.png,
crest_flame_skull.png, crest_flame_sword.png, crest_flame_coin.png, crest_bomb.png; рядом — preview_crest_fires.png);
чёрно-фиолетовый огонёк в цветах теневых фигур (crest_dark_flame.png) и с ним: ладья, череп, два огонька,
фиолетовый портал, монета, демонический глаз (crest_dark_<rook|skull|flames|portal|coin|eye>.png; рядом —
preview_crest_darks.png); воздушный смерч (crest_wind.png) и с ним: два смерча, монета, конь, меч
(crest_<winds|wind_coin|wind_knight|wind_sword>.png; рядом — preview_crest_winds.png); молния и слон, молния и
монета, молния и плюс, две молнии, красная молния (crest_<bolt_bishop|bolt_coin|bolt_plus|bolts|red_bolt>.png;
рядом — preview_crest_bolts.png); молния и цифра 1 (crest_bolt_one.png); пары
фигур — две пешки, два слона, два коня (мордами друг к другу), две ладьи, два ферзя — и знак обновления с монетой
(crest_<pawns2|bishops2|knights2|rooks2|queens2|reroll_coin>.png; рядом — preview_crest_pairs.png); магическая
шляпа (crest_wizard_hat.png) и с ней: монета, две шляпы, слон, метеорит, плюс; отдельно метеорит
(crest_<hat_coin|hats|hat_bishop|hat_meteor|hat_plus|meteor>.png; рядом — preview_crest_hats.png); золотой король
и с ним монета, второй король, ферзь, череп; свинка-копилка (crest_<gold_king|king_coin|kings2|king_queen|king_skull|
piggy>.png; рядом — preview_crest_kings.png); топор с огоньком, снежинкой, молнией, шляпой, фиолетовым
пламенем, смерчем, золотым королём (crest_axe_<flame|snow|bolt|hat|dark|wind|king>.png; рядом —
preview_crest_axe_mix.png); золотой король с огоньком, молнией, снежинкой, смерчем, фиолетовым пламенем,
шляпой (crest_king_<flame|bolt|snow|wind|dark|hat>.png; рядом — preview_crest_king_mix.png; король с топором —
crest_axe_king.png); шляпа мага с огоньком, снежинкой, молнией, смерчем, фиолетовым пламенем
(crest_hat_<flame|snow|bolt|wind|dark>.png; рядом — preview_crest_hat_mix.png; шляпа с топором и королём —
crest_axe_hat.png, crest_king_hat.png); огонёк со снежинкой, молнией, смерчем, фиолетовым пламенем
(crest_flame_<snow|bolt|wind|dark>.png; рядом — preview_crest_flame_mix.png; огонёк с топором, королём, шляпой —
crest_axe_flame.png, crest_king_flame.png, crest_hat_flame.png); снежинка с молнией, смерчем, фиолетовым пламенем
(crest_snow_<bolt|wind|dark>.png; рядом — preview_crest_snow_mix.png; молния слева и снежинка — crest_both.png).

`py tools/breach_crest.py [папка]` — по умолчанию assets/breach/ui: crest.png (молния) и
crest_snow.png (снежинка), W×H; превью preview_crest.png и preview_crest_snow.png (крупно и
рядом с монетой в настоящем размере ×2).
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "ui")

W, H = 49, 45  # с контуром
OUTLINE = hexc("#020201")
GOLD = [hexc(c) for c in ["#fffbe0", "#ffe27a", "#f5b638", "#d07d1c", "#8a4510"]]
IRON = [hexc(c) for c in ["#8a8c96", "#5a5c66", "#3a3b44", "#24252c", "#141519"]]
WINE = [hexc(c) for c in ["#4a121a", "#3a0d14", "#2c0910", "#1e060b", "#120306"]]  # очень тёмное бордо
SS = 4
TOP_DIP = 0.17  # насколько верх прогибается к середине (доля высоты)
WAIST = 0.55  # до этой высоты бока идут почти прямо, дальше дугой сходятся к острию
# молния: (x — доля полуширины от середины, y — доля высоты)
PAWN = [  # 9×14, середина — столбец 16 щита (как у снежинки справа, 30)
    "...###...",
    "..#####..",
    "..#####..",
    "..#####..",
    "...###...",
    ".#######.",
    "..#####..",
    "...###...",
    "...###...",
    "..#####..",
    "..#####..",
    ".#######.",
    "#########",
    "#########",
]
SPIKES = [  # 11×12, стальные шипы: высокий посередине, два ниже по бокам, общее основание
    ".....#.....",
    ".....#.....",
    "....###....",
    "....###....",
    ".#..###..#.",
    ".#..###..#.",
    ".##.###.##.",
    ".##.###.##.",
    "###.###.###",
    "###.###.###",
    "###########",
    "###########",
]
STEEL = [hexc(c) for c in ["#ffffff", "#dfe3ec", "#a9b0c0", "#6c7286", "#3a3e4c"]]
# кристалл: от самого светлого (блик) до самого тёмного (кайма снежинки)
CRYSTAL = [hexc(c) for c in ["#f2fdff", "#bfeaf8", "#7cc4e4", "#3f86b4", "#1f4a72", "#122c48"]]
BOLT = [(0.14, 0.27), (-0.26, 0.58), (-0.02, 0.58), (-0.22, 0.88), (0.30, 0.49), (0.06, 0.49), (0.34, 0.27)]


def shield(px, py, inset=0.0):
    """Маска щита в координатах холста без контура (ширина W−2, высота H−2)."""
    w, h = W - 2, H - 2
    cx = w / 2
    u = (px - cx) / (w / 2 - inset)
    v = (py - inset * 0.9) / (h - inset * 2.2)
    top = TOP_DIP * (1 - u ** 2) + 0.004  # вогнутый верх, рожки по углам
    hw0 = 1.0 - 0.16 * np.clip(v / WAIST, 0, 1)  # бока чуть сужаются, почти прямые
    t = np.clip((v - WAIST) / (1 - WAIST), 0, 1)
    hw = np.where(v < WAIST, hw0, (1 - 0.16) * (1 - t ** 1.6) ** 0.75)  # к острию — плавной дугой, остриё острое
    return (v >= top) & (v <= 1) & (np.abs(u) <= hw)


def in_poly(x, y, poly):
    inside = np.zeros(x.shape, bool)
    for i in range(len(poly)):
        x0, y0 = poly[i]
        x1, y1 = poly[(i + 1) % len(poly)]
        inside ^= ((y0 > y) != (y1 > y)) & (x < (x1 - x0) * (y - y0) / (y1 - y0 + 1e-9) + x0)
    return inside


def coverage(fn, w, h):
    yy, xx = np.mgrid[0 : h * SS, 0 : w * SS]
    m = fn((xx + 0.5) / SS, (yy + 0.5) / SS)
    return m.reshape(h, SS, w, SS).mean(axis=(1, 3)) >= 0.5


def shifted(m, dy, dx):
    out = np.zeros_like(m)
    h, w = m.shape
    out[max(dy, 0) : h + min(dy, 0), max(dx, 0) : w + min(dx, 0)] = m[max(-dy, 0) : h - max(dy, 0),
                                                                      max(-dx, 0) : w - max(dx, 0)]
    return out


ICE = [hexc(c) for c in ["#ffffff", "#e2f6ff", "#a6dcf2", "#62a8d0", "#2f5f80"]]
RED = [hexc(c) for c in ["#ffd6cc", "#ff6a5c", "#e0303a", "#9a1620", "#4a0a10"]]  # сердце — ярче поля


WOOD = [hexc(c) for c in ["#e8b878", "#b87a40", "#8a5428", "#5e3418", "#341c0c"]]  # топорище
LEATHER = [hexc(c) for c in ["#a07850", "#6e4a2c", "#4e321c", "#341f10", "#1e1108"]]  # рукоять меча
BONE = [hexc(c) for c in ["#fffaf0", "#f0e4c8", "#cdb890", "#94805c", "#5a4a32"]]  # клыки
WHIRL = [hexc(c) for c in ["#ffd0c0", "#ff5040", "#d81e26", "#8c0e16", "#3e050a"]]  # вихрь — ярко-красный
FIRE = [hexc(c) for c in ["#fff6c0", "#ffd040", "#ff8a20", "#d0401a", "#6e1808"]]
KING_BIG = [  # 13×20, большой золотой король: крест, корона с тремя зубцами (в обруче — рубины), шея, воротник, подставка
    "......#......",
    ".....###.....",
    "......#......",
    "..#...#...#..",
    "..##.###.##..",
    "..#########..",
    "..#########..",
    "...#######...",
    "....#####....",
    "....#####....",
    "...#######...",
    "....#####....",
    "....#####....",
    "....#####....",
    "....#####....",
    "...#######...",
    "..#########..",
    ".###########.",
    "#############",
    "#############",
]
KING_SMALL = [  # 9×16, малый золотой король
    "....#....",
    "...###...",
    "....#....",
    ".#..#..#.",
    ".##.#.##.",
    ".#######.",
    "..#####..",
    "...###...",
    "..#####..",
    "...###...",
    "...###...",
    "...###...",
    "..#####..",
    ".#######.",
    "#########",
    "#########",
]
QUEEN_SMALL = [  # 9×14, малый ферзь — ниже малого короля (король в шахматах выше ферзя)
    "#...#...#",
    "##.###.##",
    "#########",
    ".#######.",
    "..#####..",
    "...###...",
    "...###...",
    "..#####..",
    "...###...",
    "...###...",
    "..#####..",
    ".#######.",
    "#########",
    "#########",
]
PIG_LEG = [hexc(c) for c in ["#ffb0c8", "#f080a4", "#c85c84", "#8a3458", "#4a1430"]]  # ножки — тоном темнее тела
RUBY = [hexc(c) for c in ["#ffd0d4", "#ff4050", "#a01020"]]  # камни в короне
PIG = [hexc(c) for c in ["#ffe0ea", "#ffb0c8", "#f080a4", "#b84c74", "#6a2440"]]  # свинка-копилка
BISHOP = [  # 9×15, золотой слон: митра с прорезью, воротник, ножка, подставка
    "....#....",
    "...###...",
    "..#####..",
    "..###.#..",
    "..##.##..",
    "..#####..",
    "...###...",
    "..#####..",
    "...###...",
    "...###...",
    "...###...",
    "..#####..",
    ".#######.",
    "#########",
    "#########",
]
ONE = [  # 7×13, цифра 1: носик, стойка, подставка
    "...###.",
    "..####.",
    ".#####.",
    "##.###.",
    "...###.",
    "...###.",
    "...###.",
    "...###.",
    "...###.",
    "...###.",
    "...###.",
    "#######",
    "#######",
]
PLUS = [  # 11×11, толстый плюс
    "....###....",
    "....###....",
    "....###....",
    "....###....",
    "###########",
    "###########",
    "###########",
    "....###....",
    "....###....",
    "....###....",
    "....###....",
]
CYAN = [hexc(c) for c in ["#ffffff", "#c8f6ff", "#6adcf8", "#2a9cd0", "#14507a"]]  # плюс — электрический голубой
KNIGHT = [  # 12×15, золотой конь мордой влево: ухо, глаз, морда, нижняя челюсть, шея, подставка
    "......#.....",
    ".....###....",
    "....#####...",
    "...#######..",
    "..##.######.",
    ".##########.",
    "###########.",
    "###...######",
    "......######",
    ".....#######",
    ".....######.",
    "....#######.",
    "...#########",
    "..##########",
    "############",
]
QUEEN = [  # 11×17, золотой ферзь: корона с тремя зубцами, шея, воротник, подставка
    "#....#....#",
    "##..###..##",
    "###.###.###",
    "###########",
    ".#########.",
    "..#######..",
    "...#####...",
    "...#####...",
    "....###....",
    "....###....",
    "...#####...",
    "....###....",
    "...#####...",
    "..#######..",
    ".#########.",
    "###########",
    "###########",
]


def grid_mask(rows, x0, y0, w, h):
    m = np.zeros((h, w), bool)
    for r, row in enumerate(rows):
        for c, ch in enumerate(row):
            m[y0 + r, x0 + c] = ch == "#"
    return m


def axe(w, h, hx, yc, size=1.0, side=-1, haft_len=17.0):
    """Топор: топорище шириной 2·size по x = hx, лезвие в сторону side (−1 — влево): от топорища
    узкая шейка расходится веером, кромка выпуклая (≈ 7.5·size от оси, 13·size в высоту); на стыке
    и внизу — золотые обмотка и навершие. Возвращает маски (топорище, золото, лезвие)."""
    def blade(x, y):
        d = (x - hx) * side / size  # от оси топорища в сторону лезвия
        dy = (y - yc) / size
        edge = d <= 7.6 - 0.04 * dy ** 2
        neck = np.abs(dy) < 1.5 + 1.05 * (d - 1)
        return (d >= 1) & edge & neck

    def haft(x, y):
        return (np.abs(x - hx) < 1.0 * size) & (y > yc - 5 * size) & (y < yc + haft_len * size)

    def gold(x, y):
        band = (np.abs(x - hx) < 1.0 * size) & (np.abs(y - yc) < 2.4 * size)
        knob = (np.abs(x - hx) < 1.6 * size) & (y > yc + (haft_len - 1.6) * size) & (y < yc + (haft_len + 0.6) * size)
        return band | knob

    return coverage(haft, w, h), coverage(gold, w, h), coverage(blade, w, h)


def fang(x, y, cx, top, bot, bend):
    """Клык: широкий у корня, к концу острый, загнут вбок на bend пикселей."""
    t = np.clip((y - top) / (bot - top), 0, 1)
    mid = cx + bend * t ** 2
    half = 2.3 * (1 - t) ** 0.8 + 0.15
    return (y >= top) & (y <= bot) & (np.abs(x - mid) < half)


def whirl(x, y, cx, cy, r):
    """Вихрь: три дуги-струи по кругу, у головы толстые, хвост тонкий (вращение против часовой)."""
    th = np.degrees(np.arctan2(-(y - cy), x - cx)) % 360
    rr = np.hypot(x - cx, y - cy)
    m = np.zeros(x.shape, bool)
    for k in range(3):
        s = ((th - (30 + 120 * k)) % 360) / 118.0  # 0 — хвост, 1 — голова; хвост доходит до следующей головы
        inside = s <= 1
        rc = r - 1.2 * (1 - s)  # хвост чуть ближе к середине — спираль
        half = 0.6 + 1.6 * s ** 0.8
        m |= inside & (np.abs(rr - rc) < half)
    return m


# языки пламени горящего топора: (x середины у основания, y основания, y кончика, полуширина, изгиб)
FLAMES = [(21.0, 21.0, 8.5, 3.6, 1.6), (25.0, 20.0, 10.5, 3.0, -1.4), (17.5, 26.0, 13.5, 2.6, 1.2),
          (28.5, 19.5, 13.0, 2.0, -1.0)]


# огонёк герба: слои от внешнего к сердцевине — каждый язык (x, основание, кончик, полуширина, изгиб)
FLAME_LAYERS = [
    [(23.5, 29.0, 8.5, 7.4, 1.8), (17.0, 31.0, 17.0, 3.4, -1.4), (30.0, 31.0, 18.0, 3.2, 1.4)],  # красный
    [(23.5, 29.5, 12.0, 5.4, 1.4), (18.5, 31.0, 21.0, 2.2, -1.0), (28.5, 31.0, 21.5, 2.0, 1.0)],  # оранжевый
    [(23.5, 30.0, 16.5, 3.6, 1.0)],  # жёлтый
    [(23.5, 30.5, 22.5, 1.9, 0.5)],  # бледное ядро
]


def small_flame(cx, base, k=0.62):
    """Огонёк поменьше — слои FLAME_LAYERS, ужатые в k раз к точке (23.5, 30) и сдвинутые в (cx, base)."""
    return [[(cx + (x - 23.5) * k, base + (b - 30) * k, base + (t - 30) * k, wd * k, bd * k)
             for x, b, t, wd, bd in layer] for layer in FLAME_LAYERS]


SKULL = [  # 11×11, череп: глазницы и нос — дыры, зубы
    "...#####...",
    ".#########.",
    "###########",
    "###########",
    "##...#...##",
    "##...#...##",
    "###########",
    ".####.####.",
    "..#######..",
    "..#.#.#.#..",
    "..#######..",
]
BOMB = [hexc(c) for c in ["#dfe2c4", "#a2a67e", "#6c7050", "#454832", "#24261a"]]  # авиабомба — военный оливковый
FINS = [hexc(c) for c in ["#f0f2f8", "#c4c8d4", "#9096a6", "#5c6070", "#30323c"]]  # оперение — светлая сталь
HAZARD = [hexc(c) for c in ["#fff6c0", "#ffd84a", "#e8a818", "#a86a08", "#5a3604"]]  # жёлтая полоса


# чёрно-фиолетовое пламя — цвета ауры теневых фигур (`breach_darkfire.py`): слои снаружи внутрь —
# фиолетовая кромка, тёмно-фиолетовый, почти чёрный, чёрное нутро; кончики — сиреневые, отсвет — фиолетовый
DARK_FIRE = [hexc(c) for c in ["#8644f2", "#4f249a", "#1e0f34", "#0c0812"]]
DARK_TIP = hexc("#b583ff")
DARK_SPARK = hexc("#e2c8ff")
DARK_GLOW = (hexc("#2c0f3c"), hexc("#3c1458"))


ROOK = [  # 11×13, золотая ладья: зубцы, бойница, башня, подставка
    "##..###..##",
    "##..###..##",
    "###########",
    ".#########.",
    "..#######..",
    "..###.###..",
    "..###.###..",
    "..#######..",
    "..#######..",
    "..#######..",
    ".#########.",
    "###########",
    "###########",
]
VOID = hexc("#0c0812")  # нутро портала и зрачок


# воздушный смерч — цвета вихря воздушных фигур (`breach_wind.py`): голова струи белая, хвост голубой
AIR = [hexc(c) for c in ["#ffffff", "#dff4ff", "#a8d8f0", "#6aa8d0", "#3a6e98"]]
DUST = [hexc(c) for c in ["#e8e2d4", "#b8b0a0", "#86806e"]]


def tornado_strokes(w, h, mid=23.5, top=14.0, bottom=33.0, width=10.0, n=13, taper=1.25):
    """Смерч из витков: точки (x, y, тон, спереди ли) — широкая воронка сверху, к низу уже и
    чуть изогнута; каждый виток — дуга эллипса, голова струи белая, хвост голубой, задняя
    половина витка темнее."""
    pts = []
    k_w = width / 10.0
    for i in range(n):
        t = i / (n - 1)  # 0 — верх, 1 — низ
        cy = top + (bottom - top) * t
        rx = width * (1 - t) ** taper + 1.6 * k_w ** 0.5  # taper < 1 — ровный конус, > 1 — чаша
        ry = 0.3 * rx + 0.7
        cx = mid + 1.9 * k_w * math.sin(t * 3.1)
        start = math.radians(i * 41)
        span = math.radians(250)
        for k in range(90):
            u = k / 89  # 0 — хвост, 1 — голова
            th = start + span * u
            x, y = cx + rx * math.cos(th), cy + ry * math.sin(th)
            front = math.sin(th) > -0.15
            tone = 0 if u > 0.8 else 1 if u > 0.55 else 2 if u > 0.25 else 3
            if not front:
                tone = min(4, tone + 2)
            pts.append((x, y, tone, front))
    return pts


def reroll_arrow(x, y, cx, cy, r=5.4, half=1.3):
    """Знак обновления: кольцо почти по кругу (разрыв справа сверху) и наконечник на конце —
    стрелка идёт против часовой."""
    th = np.degrees(np.arctan2(-(y - cy), x - cx)) % 360
    rr = np.hypot(x - cx, y - cy)
    ring = (np.abs(rr - r) < half) & ~((th > 8) & (th < 72))
    a = math.radians(8)  # конец кольца у разрыва
    ex, ey = cx + r * math.cos(a), cy - r * math.sin(a)
    tx, ty = -math.sin(a), -math.cos(a)  # касательная против часовой (на экране вверх)
    nx, ny = math.cos(a), -math.sin(a)  # наружу
    head = [(ex + nx * 3.8, ey + ny * 3.8), (ex - nx * 3.6, ey - ny * 3.6), (ex + tx * 4.6, ey + ty * 4.6)]
    return ring | in_poly(x, y, head)


ROCK = [hexc(c) for c in ["#24141a", "#442a32", "#6a464a", "#8e6862", "#b48a78"]]  # камень метеорита, как в игре
HAT = [hexc(c) for c in ["#9db8ff", "#5a7ee0", "#2f4fb8", "#1a2e78", "#0c1640"]]  # шляпа мага — синяя, как у магов
STARLIGHT = [hexc(c) for c in ["#ffffff", "#fff4b0", "#ffe070"]]  # звёздочки и искорки


def wizard_hat(x, y, cx=23.5, by=31.0, k=1.0):
    """Шляпа мага: конус (ось изогнута, кончик заломлен влево) и поля — эллипс. Возвращает (конус, поля).
    cx, by — середина полей, k — масштаб (большая шляпа — 23.5, 31, 1)."""
    x = 23.5 + (x - cx) / k
    y = 31.0 + (y - by) / k
    t = np.clip((30.0 - y) / 21.0, 0, 1)  # 0 — у полей, 1 — кончик
    mid = 23.5 - 4.8 * t ** 4.5  # ровный конус, гнётся только кончик (раньше край выходил отвесным — парус)
    half = 6.8 * (1 - t) ** 0.9 + 0.5
    cone = (y <= 30.5) & (y >= 9.0) & (np.abs(x - mid) < half)
    brim = np.hypot((x - 23.5) / 11.8, (y - 31.0) / 3.3) < 1
    return cone, brim


def flames(x, y, tongues):
    """Языки пламени: от круглого основания вверх к острому кончику, изогнуты по синусу."""
    m = np.zeros(x.shape, bool)
    for cx, base, tip, wid, bend in tongues:
        t = np.clip((base - y) / (base - tip), 0, 1)
        mid = cx + bend * np.sin(t * math.pi)
        half = wid * (1 - t) ** 0.85
        m |= (y <= base) & (y >= tip) & (np.abs(x - mid) < half)
        m |= np.hypot(x - cx, y - base) < wid
    return m


def heart(x, y, cx, cy, s):
    """Сердце (x² + y² − 1)³ ≤ x²y³: ширина ≈ 2,3·s, высота ≈ 2,2·s, (cx, cy) — середина по высоте."""
    u = (x - cx) / s
    v = -(y - cy) / s + 0.12
    return (u * u + v * v - 1) ** 3 - u * u * v ** 3 <= 0


def seg_dist(x, y, a, b):
    ax, ay = a
    bx, by = b
    dx, dy = bx - ax, by - ay
    t = np.clip(((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy), 0, 1)
    return np.hypot(x - (ax + t * dx), y - (ay + t * dy))


def snowflake(x, y, cx, cy, r, twigs=((0.5, 3.0), (0.78, 2.0))):
    """Шестилучевая снежинка: лучи через 60° (вертикальный и по 30° к горизонтали — в
    пикселях это чистые ступеньки 2:1), на каждом луче две пары веточек под 60°."""
    m = np.hypot(x - cx, y - cy) < 1.9  # серединка
    for k in range(6):
        a = math.radians(90 + 60 * k)
        d = (math.cos(a), -math.sin(a))
        tip = (cx + d[0] * r, cy + d[1] * r)
        m |= seg_dist(x, y, (cx, cy), tip) < 0.62
        for at, ln in twigs:
            q = (cx + d[0] * r * at, cy + d[1] * r * at)
            for side in (-1, 1):
                b = a + side * math.radians(60)
                e = (q[0] + math.cos(b) * ln, q[1] - math.sin(b) * ln)
                m |= seg_dist(x, y, q, e) < 0.5
    return m


def crest(emblem="bolt"):
    w, h = W - 2, H - 2
    body = coverage(lambda x, y: shield(x, y), w, h)
    field = coverage(lambda x, y: shield(x, y, 2.6), w, h)
    rim = body & ~field
    yy, xx = np.mgrid[0:h, 0:w]
    img = np.zeros((h, w, 4), np.uint8)

    def put(mask, col):
        img[mask] = (*col, 255)

    left = xx < w / 2
    # обод: тёмный металл, фаска к свету
    put(rim, IRON[2])
    put(rim & left, IRON[1])
    put(rim & ~left & (yy > h * 0.25), IRON[3])
    outer = rim & ~(shifted(body, 1, 0) & shifted(body, 0, 1) & shifted(body, -1, 0) & shifted(body, 0, -1))
    put(outer & left & (yy < h * 0.75), IRON[0])
    inner = rim & (shifted(field, 1, 0) | shifted(field, 0, 1) | shifted(field, -1, 0) | shifted(field, 0, -1))
    put(inner, IRON[4])
    # поле: очень тёмное бордо, к свету чуть светлее, у обода тень
    g = xx / w + yy / h
    put(field, WINE[2])
    put(field & (g < 0.62), WINE[1])
    put(field & (g < 0.42), WINE[0])
    put(field & (g > 1.2), WINE[3])
    shade = field & ~(shifted(field, 2, 0) & shifted(field, 0, 2) & shifted(field, 2, 2))
    put(shade, WINE[4])
    # золотые уголки на рожках
    for sgn in (-1, 1):
        tip_x = w / 2 + sgn * (w / 2)
        cap = body & ((np.abs(xx + 0.5 - tip_x) + (yy + 0.5) * 1.15) < 10.5)
        edge = cap & ~(shifted(cap, 1, 0) & shifted(cap, -1, 0) & shifted(cap, 0, 1) & shifted(cap, 0, -1))
        put(cap, GOLD[2])
        put(cap & (yy < 3), GOLD[1])
        put(edge & body & ~outer, GOLD[4])  # кромка уголка на щите — тень
        put(cap & outer & ((xx < w / 2) | (yy < 2)), GOLD[0] if sgn < 0 else GOLD[1])
    marks = []  # (маска, тона)
    def bolt_mask(k, ox, oy):
        poly = [(w / 2 + (x * k + ox) * (w / 2), (0.55 + (y - 0.55) * k + oy) * h) for x, y in BOLT]
        return coverage(lambda x, y: in_poly(x, y, poly), w, h) & field

    def draw_hat(cx, by, k):
        """Малая шляпа: конус, поля, лента, звёздочка — знаки в marks, звёзды — в hat_stars."""
        cone = coverage(lambda x, y: wizard_hat(x, y, cx, by, k)[0], w, h) & field
        brim = coverage(lambda x, y: wizard_hat(x, y, cx, by, k)[1], w, h) & field
        band = cone & ~brim & (yy + 0.5 > by - 5.5 * k)
        marks.extend([(cone, HAT), (band, GOLD), (brim, HAT)])
        hat_parts.append((cone, brim))
        hat_stars.append((round(cx - 1.5 * k), round(by - 11 * k)))

    def draw_meteor(cx, cy, r, length, glow=True):
        """Метеорит: бугристый камень с кратерами, спереди (вниз-влево) раскалённая кромка; хвост пламени вверх-вправо."""
        ux, uy = math.cos(math.radians(45)), -math.sin(math.radians(45))  # хвост — вверх-вправо
        rx, ry = xx + 0.5 - cx, yy + 0.5 - cy
        s_ = rx * ux + ry * uy
        p_ = -rx * uy + ry * ux
        wobble = 0.6 * np.sin(s_ * 1.3 + p_ * 0.8)
        width = (r + 1.8) * np.clip(1 - s_ / length, 0, 1) ** 0.7
        tail = (s_ >= 0) & (np.abs(p_) < width + wobble) & field
        th = np.arctan2(ry, rx)
        edge_r = r * (1 + 0.08 * np.sin(3 * th + 0.7) + 0.05 * np.sin(5 * th + 2.0))
        rock = (np.hypot(rx, ry) <= edge_r) & field
        halo = (np.hypot(rx, ry) <= edge_r + 1.6) & field & ~rock
        fire = (tail | halo) & ~rock
        shape = fire | rock
        put((shifted(shape, 1, 0) | shifted(shape, -1, 0) | shifted(shape, 0, 1) | shifted(shape, 0, -1)
             | shifted(shape, 1, 1)) & field & ~shape, WINE[4])
        heat = np.abs(p_) / np.maximum(width, 1) * 0.5 + np.clip(s_ / length, 0, 1) * 0.9
        put(fire, FIRE[4])  # слои: у камня бледно-жёлтое, к краям и концу хвоста — красное
        for lim, col in ((0.8, FIRE[3]), (0.6, FIRE[2]), (0.4, FIRE[1]), (0.2, FIRE[0])):
            put(fire & (heat < lim), col)
        fwd = -(rx * ux + ry * uy)  # вперёд по полёту — вниз-влево
        put(rock, ROCK[2])
        put(rock & (fwd < -r * 0.3), ROCK[1])
        put(rock & (rx + ry < -r * 0.4), ROCK[3])  # свет сверху-слева
        for ang, dd, cr in ((0.5, 0.45, 0.32), (2.4, 0.5, 0.26), (4.1, 0.35, 0.3)):
            ccx, ccy = cx + math.cos(ang) * dd * r, cy + math.sin(ang) * dd * r
            cd = np.hypot(xx + 0.5 - ccx, yy + 0.5 - ccy)
            put(rock & (cd < cr * r), ROCK[0])
            put(rock & (cd >= cr * r) & (cd < cr * r + 0.9) & ((xx + 0.5 - ccx) + (yy + 0.5 - ccy) > 0), ROCK[4])
        glow_rim = rock & (np.hypot(rx, ry) > edge_r - 1.6) & (fwd > r * 0.15)
        put(glow_rim, FIRE[2])
        put(glow_rim & (fwd > r * 0.6), FIRE[1])

    hat_parts, hat_stars = [], []
    if emblem in ("hat_coin", "hat_bishop", "hat_meteor", "hat_plus"):  # малая шляпа слева
        draw_hat(16.0, 29.5, 0.62)
    if emblem == "hats":  # две малые шляпы
        for cx in (15.0, 32.0):
            draw_hat(cx, 29.5, 0.6)
    if emblem == "hat_coin":  # монета справа
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem == "hat_bishop":  # золотой слон справа
        marks.append((grid_mask(BISHOP, 27, 15, w, h) & field, GOLD))
    if emblem == "hat_plus":  # голубой плюс справа
        marks.append((grid_mask(PLUS, 26, 17, w, h) & field, CYAN))
    if emblem == "hat_meteor":  # малый метеорит справа
        draw_meteor(29.5, 26.5, 4.2, 13.0)
    if emblem == "meteor":  # метеорит посередине: камень снизу-слева, хвост через щит вверх-вправо
        draw_meteor(19.5, 27.0, 7.0, 25.0)
        for sx, sy in ((30, 25), (33, 21), (25, 13)):  # искры
            if field[sy, sx]:
                put((xx == sx) & (yy == sy), FIRE[1])
    gems = []  # (x, y) рубины в коронах — поверх
    if emblem == "gold_king":  # большой золотой король посередине
        marks.append((grid_mask(KING_BIG, 17, 12, w, h) & field, GOLD))
        gems += [(23, 18), (20, 18), (26, 18)]
    if emblem in ("king_coin", "king_queen", "king_skull", "kings2"):  # малый король слева
        marks.append((grid_mask(KING_SMALL, 11, 14, w, h) & field, GOLD))
        gems.append((15, 19))
    if emblem == "kings2":  # и справа — второй
        marks.append((grid_mask(KING_SMALL, 27, 14, w, h) & field, GOLD))
        gems.append((31, 19))
    if emblem == "king_queen":  # золотой ферзь справа — малый, ниже короля; основания на одной линии
        marks.append((grid_mask(QUEEN_SMALL, 27, 16, w, h) & field, GOLD))
    if emblem == "king_skull":  # костяной череп справа
        marks.append((grid_mask(SKULL, 26, 17, w, h) & field, BONE))
    if emblem == "king_coin":  # монета справа
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem == "piggy":  # свинка-копилка мордой влево, сверху в прорезь падает монетка
        def pig(x, y):  # пухлое тело, голова, ухо
            body = np.hypot((x - 24.5) / 10.6, (y - 24.0) / 7.6) < 1
            head = np.hypot((x - 15.0) / 5.2, (y - 23.5) / 5.2) < 1
            ear = in_poly(x, y, [(14.4, 19.6), (16.6, 15.0), (19.2, 19.2)])
            return body | head | ear

        pig_m = coverage(pig, w, h) & field
        # ножки — только две (спереди и сзади), короткие и толстые: четыре тонких под брюхом походили на вымя
        legs = coverage(lambda x, y: ((np.abs(x - 18.0) < 2.1) | (np.abs(x - 30.0) < 2.1)) & (y > 28.0) & (y < 33.4),
                        w, h) & field & ~pig_m
        snout = coverage(lambda x, y: np.hypot((x - 10.4) / 2.1, (y - 24.6) / 2.9) < 1, w, h) & field
        tail = coverage(lambda x, y: (np.abs(np.hypot(x - 36.4, y - 21.6) - 1.6) < 0.55) & (x > 35.0), w, h) & field
        coin_m = coverage(lambda x, y: np.hypot((x - 24.5) / 2.8, (y - 13.0) / 2.8) < 1, w, h) & field  # выше — режет верх щита
        marks += [(legs, PIG_LEG), (pig_m, PIG), (snout, PIG), (tail, PIG), (coin_m, GOLD)]
    if emblem == "wizard_hat":  # магическая шляпа посередине: конус, поля, золотая лента, звёзды
        cone = coverage(lambda x, y: wizard_hat(x, y)[0], w, h) & field
        brim = coverage(lambda x, y: wizard_hat(x, y)[1], w, h) & field
        band = cone & ~brim & (yy + 0.5 > 25.5)
        marks += [(cone, HAT), (band, GOLD), (brim, HAT)]
    pairs = {"pawns2": (PAWN, 11, 27, 16), "bishops2": (BISHOP, 11, 27, 15), "rooks2": (ROOK, 10, 26, 16),
             "queens2": (QUEEN, 10, 26, 14)}
    if emblem in pairs:  # две одинаковые золотые фигуры, симметрично середине щита
        grid, xl, xr, y0 = pairs[emblem]
        marks.append((grid_mask(grid, xl, y0, w, h) & field, GOLD))
        marks.append((grid_mask(grid, xr, y0, w, h) & field, GOLD))
    if emblem == "knights2":  # два коня мордами друг к другу: левый отражён
        marks.append((grid_mask([row[::-1] for row in KNIGHT], 10, 15, w, h) & field, GOLD))
        marks.append((grid_mask(KNIGHT, 25, 15, w, h) & field, GOLD))
    if emblem == "reroll_coin":  # знак обновления слева, монета справа
        marks.append((coverage(lambda x, y: reroll_arrow(x, y, 16.0, 22.5), w, h) & field, CYAN))
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem == "bolt_one":  # молния слева, справа цифра 1 светлой сталью
        marks.append((bolt_mask(0.72, -0.31, -0.07), GOLD))
        marks.append((grid_mask(ONE, 27, 16, w, h) & field, STEEL))
    if emblem in ("bolt_bishop", "bolt_coin", "bolt_plus"):  # молния слева — как на гербе с молнией и снежинкой
        marks.append((bolt_mask(0.72, -0.31, -0.07), GOLD))
    if emblem == "bolts":  # две молнии рядом, поменьше
        for ox in (-0.365, 0.315):
            marks.append((bolt_mask(0.62, ox, -0.05), GOLD))
    if emblem == "red_bolt":  # большая красная молния, как на первом гербе
        marks.append((bolt_mask(1.0, 0.0, 0.0), RED))
    if emblem == "bolt_bishop":  # золотой слон справа
        marks.append((grid_mask(BISHOP, 27, 15, w, h) & field, GOLD))
    if emblem == "bolt_plus":  # голубой плюс справа
        marks.append((grid_mask(PLUS, 26, 17, w, h) & field, CYAN))
    if emblem == "bolt_coin":  # монета справа
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem in ("bolt", "both"):  # молния; на гербе с обоими — меньше и слева
        k, ox, oy = (1.0, 0.0, 0.0) if emblem == "bolt" else (0.72, -0.31, -0.07)
        poly = [(w / 2 + (x * k + ox) * (w / 2), (0.55 + (y - 0.55) * k + oy) * h) for x, y in BOLT]
        marks.append((coverage(lambda x, y: in_poly(x, y, poly), w, h) & field, GOLD))
    if emblem == "heart":  # сердце слева, на высоте снежинки, на пиксель дальше от неё
        marks.append((coverage(lambda x, y: heart(x, y, w - 30.5, 22.5, 4.8), w, h) & field, RED))
    if emblem == "coin":  # монета игры coin_16 слева — со своим контуром, тень на поле
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(w - 31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem in ("snow", "both", "heart", "coin"):  # снежинка — ледяная; на гербе с двумя знаками — меньше и справа
        if emblem == "snow":
            fn = lambda x, y: snowflake(x, y, w / 2, h * 0.52, 11.5)  # noqa: E731
        else:
            fn = lambda x, y: snowflake(x, y, 29.5, 22.5, 7.6, ((0.6, 2.2),))  # центр ровно в пикселе — симметрично  # noqa: E731
        marks.append((coverage(fn, w, h) & field, ICE))
    if emblem == "pawn":  # золотая пешка слева — по пикселям: голова, шейка, воротник, тело, подставка
        mark = np.zeros((h, w), bool)
        for r, row in enumerate(PAWN):
            for c, ch in enumerate(row):
                mark[16 + r, 12 + c] = ch == "#"
        marks.append((mark & field, GOLD))
    if emblem == "spikes":  # стальные шипы слева — по пикселям
        mark = np.zeros((h, w), bool)
        for r, row in enumerate(SPIKES):
            for c, ch in enumerate(row):
                mark[17 + r, 10 + c] = ch == "#"
        marks.append((mark & field, STEEL))
    if emblem in ("pawn", "spikes"):
        marks.append((coverage(lambda x, y: snowflake(x, y, 30.5, 22.5, 7.6, ((0.6, 2.2),)), w, h) & field, ICE))
    if emblem == "snow2":  # две снежинки рядом, симметрично середине щита (23.5)
        for cx in (15.5, 31.5):
            fn = lambda x, y, cx=cx: snowflake(x, y, cx, 22.5, 7.6, ((0.6, 2.2),))  # noqa: E731
            marks.append((coverage(fn, w, h) & field, ICE))
    under = []  # (маска, тона) — под знаками: вихрь, со своей каймой и тенью
    if emblem in ("axe_sword", "axe_fangs", "axe_queen", "axes"):  # топор слева, лезвием наружу
        haft, gold, blade = axe(w, h, 20.0, 16.5)
        marks += [(haft & field, WOOD), (gold & field, GOLD), (blade & field, STEEL)]
    if emblem == "axes":  # второй — зеркально справа
        haft, gold, blade = axe(w, h, 27.0, 16.5, side=1)
        marks += [(haft & field, WOOD), (gold & field, GOLD), (blade & field, STEEL)]
    if emblem == "axe_sword":  # меч справа: клинок, гарда, рукоять, навершие
        sword_blade = grid_mask([".#.", ".#."] + ["###"] * 15, 29, 9, w, h)
        guard = grid_mask(["#########", "#########"], 26, 26, w, h) | grid_mask(["###", "###"], 29, 32, w, h)
        grip = grid_mask(["#"] * 4, 30, 28, w, h)
        marks += [(sword_blade & field, STEEL), (grip & field, LEATHER), (guard & field, GOLD)]
    if emblem == "axe_fangs":  # два клыка справа, концы загнуты друг к другу
        for cx, bend in ((27.5, 1.6), (33.5, -1.6)):
            fn = lambda x, y, cx=cx, bend=bend: fang(x, y, cx, 13.0, 31.0, bend)  # noqa: E731
            marks.append((coverage(fn, w, h) & field, BONE))
    if emblem == "axe_queen":  # золотой ферзь справа
        marks.append((grid_mask(QUEEN, 25, 14, w, h) & field, GOLD))
    if emblem == "whirl":  # топор посередине в красном вихре
        under.append((coverage(lambda x, y: whirl(x, y, 23.5, 23.5, 12.5), w, h) & field, WHIRL))
        haft, gold, blade = axe(w, h, 26.5, 20.0, 0.9, haft_len=14.0)
        marks += [(haft & field, WOOD), (gold & field, GOLD), (blade & field, STEEL)]
    if emblem == "fire_axe":  # горящий топор посередине: из-за лезвия рвутся языки пламени
        haft, gold, blade = axe(w, h, 26.5, 23.5, 1.0, haft_len=13.0)
        fire = coverage(lambda x, y: flames(x, y, FLAMES), w, h)
        fire |= blade | shifted(blade, 1, 0) | shifted(blade, -1, 0) | shifted(blade, 0, 1) | shifted(blade, 0, -1)
        fire &= field
        put((shifted(fire, 1, 0) | shifted(fire, -1, 0) | shifted(fire, 0, 1) | shifted(fire, 0, -1)) & field & ~fire,
            WINE[4])
        layer = fire
        for tone in (FIRE[3], FIRE[2], FIRE[1], FIRE[0]):  # снаружи красное, к середине — жёлтое
            put(layer, tone)
            layer = layer & shifted(layer, 1, 0) & shifted(layer, -1, 0) & shifted(layer, 0, 1) & shifted(layer, 0, -1)
        for sx, sy in ((15, 12), (29, 10), (18, 7)):  # искры
            if field[sy, sx] and not fire[sy, sx]:
                put((xx == sx) & (yy == sy), FIRE[1])
        marks += [(haft & field, WOOD), (gold & field, GOLD), (blade & field, STEEL)]
    def draw_flame(layer_set, glow_at=None, glow_r=0.0, tones=None, tip=None, glow_tones=None):
        """Пламя слоями (красный, оранжевый, жёлтый, ядро) с тёмной каймой и тенью; glow_at — отсвет на поле.
        tones/tip/glow_tones — свои цвета слоёв, кончиков и отсвета (чёрно-фиолетовый огонь)."""
        tones = tones or (FIRE[3], FIRE[2], FIRE[1], FIRE[0])
        tip = tip or FIRE[2]
        glow_tones = glow_tones or (hexc("#5c1a12"), hexc("#74220e"))
        layers = [coverage(lambda x, y, t=t: flames(x, y, t), w, h) & field for t in layer_set]
        outer = layers[0]
        if glow_at is not None:
            gx, gy = glow_at
            glow = field & ~outer & (np.hypot(xx + 0.5 - gx, (yy + 0.5 - gy) * 0.8) < glow_r)
            put(glow, glow_tones[0])  # отсвет огня на тёмном поле
            put(glow & (np.hypot(xx + 0.5 - gx, (yy + 0.5 - gy - 1.0) * 0.8) < glow_r * 0.8), glow_tones[1])
        near = (shifted(outer, 1, 0) | shifted(outer, -1, 0) | shifted(outer, 0, 1) | shifted(outer, 0, -1)) & field & ~outer
        put((shifted(outer, 1, 1) | shifted(outer, 2, 1)) & field & ~outer, WINE[4])
        put(near, WINE[4])
        for tone, layer in zip(tones, layers):
            put(layer & outer, tone)
        tips = outer & ~shifted(outer, 1, 0)  # кончики языков — светлее
        put(tips & ~layers[1], tip)
        return outer

    if emblem == "flame":  # огонёк посередине: слои пламени, вокруг — отсвет на поле, сверху искры
        outer = draw_flame(FLAME_LAYERS, (23.5, 23.0), 15.5)
        for sx, sy in ((14, 12), (32, 10), (19, 6), (29, 15)):  # искры
            if field[sy, sx] and not outer[sy, sx]:
                put((xx == sx) & (yy == sy), FIRE[1])
    if emblem == "dark_flame":  # чёрно-фиолетовый огонёк — как аура теневых фигур
        outer = draw_flame(FLAME_LAYERS, (23.5, 23.0), 15.5, DARK_FIRE, DARK_TIP, DARK_GLOW)
        for sx, sy in ((14, 12), (32, 10), (19, 6), (29, 15)):  # искры
            if field[sy, sx] and not outer[sy, sx]:
                put((xx == sx) & (yy == sy), DARK_SPARK)
    dark_pairs = ("dark_rook", "dark_skull", "dark_portal", "dark_coin", "dark_eye")
    if emblem in dark_pairs:  # чёрно-фиолетовый огонёк слева
        draw_flame(small_flame(16.0, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "dark_flames":  # два чёрно-фиолетовых огонька
        for cx in (15.5, 31.5):
            draw_flame(small_flame(cx, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "dark_rook":  # золотая ладья справа
        marks.append((grid_mask(ROOK, 26, 16, w, h) & field, GOLD))
    if emblem == "dark_skull":  # костяной череп справа
        marks.append((grid_mask(SKULL, 26, 17, w, h) & field, BONE))
    if emblem == "dark_coin":  # монета справа
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem == "dark_portal":  # фиолетовый портал справа: овальное кольцо, внутри тёмная воронка со струями
        pcx, pcy = 31.5, 22.5
        rr = np.hypot((xx + 0.5 - pcx) / 6.2, (yy + 0.5 - pcy) / 9.2)
        disk = (rr < 1.0) & field
        put((shifted(disk, 1, 1) | shifted(disk, 2, 1) | shifted(disk, 1, 0) | shifted(disk, -1, 0)
             | shifted(disk, 0, 1) | shifted(disk, 0, -1)) & field & ~disk, WINE[4])
        put(disk, DARK_FIRE[1])
        put(disk & (rr < 0.86), DARK_FIRE[2])
        put(disk & (rr < 0.66), VOID)
        ang = np.arctan2((yy + 0.5 - pcy) / 9.2, (xx + 0.5 - pcx) / 6.2)
        swirl = disk & (rr < 0.82) & (rr > 0.12) & (np.cos(2 * ang + rr * 7.5) > 0.55)  # закрученные струи
        put(swirl, DARK_FIRE[1])
        put(swirl & (rr > 0.5), DARK_FIRE[0])
        lit = disk & (rr > 0.86) & ((xx + 0.5 - pcx) + (yy + 0.5 - pcy) * 0.6 < 0)  # кромка к свету светлее
        put(lit, DARK_FIRE[0])
        put(disk & (rr > 0.86) & (yy + 0.5 < pcy - 6.0) & (xx + 0.5 < pcx), DARK_TIP)
        put((xx == 31) & (yy == 22), DARK_SPARK)  # искра в глубине
    if emblem == "dark_eye":  # демонический глаз справа: миндаль, фиолетовая радужка, зрачок-щель, шипы над веком
        # (первый вариант — пользователю он понравился больше нарисованного по пикселям)
        ecx, ecy = 31.0, 23.0
        lens = coverage(lambda x, y: np.abs(y - ecy) < 4.2 * np.clip(1 - ((x - ecx) / 6.6) ** 2, 0, 1) ** 0.75, w, h) & field
        spikes = coverage(lambda x, y: np.minimum.reduce([
            np.where((y < ecy - 2.0) & (np.abs(x - sx) < 1.1 * (y - (ecy - 7.0 + abs(sx - ecx) * 0.35)) / 3.0), -1.0, 1.0)
            for sx in (27.5, 31.0, 34.5)]) < 0, w, h) & field & ~lens
        shape = lens | spikes
        put((shifted(shape, 1, 1) | shifted(shape, 2, 1) | shifted(shape, 1, 0) | shifted(shape, -1, 0)
             | shifted(shape, 0, 1) | shifted(shape, 0, -1)) & field & ~shape, WINE[4])
        put(spikes, DARK_FIRE[1])
        put(spikes & ~shifted(spikes, 1, 0), DARK_TIP)  # кончики шипов
        put(lens, DARK_FIRE[2])  # тёмное «белко»
        rim = lens & ~(shifted(lens, 1, 0) & shifted(lens, -1, 0) & shifted(lens, 0, 1) & shifted(lens, 0, -1))
        put(rim, DARK_FIRE[0])  # светящийся край века
        ir = np.hypot(xx + 0.5 - ecx, yy + 0.5 - ecy)
        iris = lens & (ir < 3.3)
        put(iris, DARK_FIRE[0])
        put(iris & (ir < 2.3), DARK_TIP)
        put(iris & (ir < 1.2), DARK_SPARK)
        put(lens & (np.abs(xx + 0.5 - ecx) < 0.6) & (np.abs(yy + 0.5 - ecy) < 3.4), VOID)  # зрачок-щель
        put((xx == 29) & (yy == 21), hexc("#ffffff"))  # блик
    def draw_tornado(mid, top, bottom, width, n, base_y, taper=1.25):
        """Смерч с облачком пыли у основания (base_y), тёмная кайма вокруг."""
        k = width / 10.0
        pts = tornado_strokes(w, h, mid, top, bottom, width, n, taper)
        mask = np.zeros((h, w), bool)
        cells = {}
        for x, y, tone, front in sorted(pts, key=lambda q: q[3]):  # сначала задние половины витков
            ix, iy = int(math.floor(x)), int(math.floor(y))
            if 0 <= ix < w and 0 <= iy < h and field[iy, ix]:
                mask[iy, ix] = True
                if front or (iy, ix) not in cells:
                    cells[(iy, ix)] = min(tone, cells.get((iy, ix), 9)) if front else tone
        puffs = [(-4.0, 0.8, 2.4), (0.0, 0.0, 2.9), (4.3, 0.8, 2.3), (-6.9, 1.6, 1.5), (7.1, 1.6, 1.5)]
        dust = coverage(lambda x, y: np.minimum.reduce([  # облачко пыли у основания — клубы
            np.hypot(x - (mid + dx * k), (y - (base_y + dy * k)) * 1.3) - r * k ** 0.8 for dx, dy, r in puffs]) < 0,
            w, h) & field
        dust &= ~mask
        shape = mask | dust
        put((shifted(shape, 1, 0) | shifted(shape, -1, 0) | shifted(shape, 0, 1) | shifted(shape, 0, -1)
             | shifted(shape, 1, 1)) & field & ~shape, WINE[4])
        put(dust, DUST[1])
        put(dust & ~shifted(dust, -1, 0), DUST[2])
        put(dust & ~shifted(dust, 1, 0), DUST[0])
        for (iy, ix), tone in cells.items():
            put((xx == ix) & (yy == iy), AIR[tone])

    if emblem == "wind":  # воздушный смерч посередине: витки воронки, пыль у основания
        draw_tornado(23.5, 14.0, 33.0, 10.0, 13, 34.2)
    if emblem in ("wind_coin", "wind_knight", "wind_sword"):  # малый смерч слева
        draw_tornado(15.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)  # малый — ровный конус, иначе похож на бокал
    if emblem == "winds":  # два малых смерча, симметрично середине щита
        for mid in (15.0, 32.0):
            draw_tornado(mid, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    if emblem == "wind_coin":  # монета справа
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem == "wind_knight":  # золотой конь справа
        marks.append((grid_mask(KNIGHT, 26, 15, w, h) & field, GOLD))
    if emblem == "wind_sword":  # меч справа — как у огонька с мечом
        sword_blade = grid_mask([".#.", ".#."] + ["###"] * 15, 30, 9, w, h)
        guard = grid_mask(["#########", "#########"], 27, 26, w, h) | grid_mask(["###", "###"], 30, 32, w, h)
        grip = grid_mask(["#"] * 4, 31, 28, w, h)
        marks += [(sword_blade & field, STEEL), (grip & field, LEATHER), (guard & field, GOLD)]
    if emblem in ("flame_pawn", "flame_skull", "flame_sword", "flame_coin"):  # огонёк слева
        draw_flame(small_flame(16.0, 27.0))
    if emblem == "flames":  # два огонька, симметрично середине щита
        for cx in (15.5, 31.5):
            draw_flame(small_flame(cx, 27.0))
    if emblem == "flame_pawn":  # золотая пешка справа
        marks.append((grid_mask(PAWN, 27, 16, w, h) & field, GOLD))
    if emblem == "flame_skull":  # костяной череп справа
        marks.append((grid_mask(SKULL, 26, 17, w, h) & field, BONE))
    if emblem == "flame_sword":  # меч справа — как у топора с мечом
        sword_blade = grid_mask([".#.", ".#."] + ["###"] * 15, 30, 9, w, h)
        guard = grid_mask(["#########", "#########"], 27, 26, w, h) | grid_mask(["###", "###"], 30, 32, w, h)
        grip = grid_mask(["#"] * 4, 31, 28, w, h)
        marks += [(sword_blade & field, STEEL), (grip & field, LEATHER), (guard & field, GOLD)]
    if emblem == "flame_coin":  # монета игры справа — со своим контуром, тень на поле
        coin = np.array(Image.open(os.path.join(OUT, "coin_16.png")).convert("RGBA"))
        ys, xs = np.nonzero(coin[..., 3])
        coin = coin[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]
        ch, cw = coin.shape[:2]
        x0, y0 = round(31.5 - cw / 2), round(22.5 - ch / 2)
        solid = np.zeros((h, w), bool)
        solid[y0 : y0 + ch, x0 : x0 + cw] = coin[..., 3] > 0
        put((shifted(solid, 1, 1) | shifted(solid, 2, 1)) & field & ~solid, WINE[4])
        img[solid] = coin[coin[..., 3] > 0]
    if emblem == "bomb":  # авиабомба падает носом вниз: овальный корпус, хвост с оперением в рамке, полосы падения
        for lx, top, ln in ((13, 16, 8), (34, 16, 8), (17, 24, 6), (30, 24, 6)):  # к бомбе светлее
            for k in range(ln):
                col = STEEL[3] if k < ln // 3 else STEEL[2] if k < 2 * ln // 3 else STEEL[1]
                if field[top + k, lx]:
                    put((xx == lx) & (yy == top + k), col)

        def hull(x, y):  # корпус: овал, к хвосту сужается конусом
            egg = ((x - 23.5) / 5.4) ** 2 + ((y - 26.5) / 7.6) ** 2 < 1
            t = np.clip((y - 13.5) / (20.5 - 13.5), 0, 1)  # 0 — у оперения, 1 — где начинается овал
            cone = (y >= 13.5) & (y <= 21.0) & (np.abs(x - 23.5) < 1.4 + 3.2 * t)
            return egg | cone

        def fins(x, y):  # стабилизаторы по бокам, ребро посередине и рамка поверх
            left = in_poly(x, y, [(21.6, 20.0), (15.6, 15.4), (15.6, 11.0), (21.6, 12.6)])
            right = in_poly(x, y, [(25.4, 20.0), (31.4, 15.4), (31.4, 11.0), (25.4, 12.6)])
            middle = (np.abs(x - 23.5) < 0.6) & (y > 10.6) & (y < 15.0)
            frame = (y > 10.6) & (y < 12.6) & (np.abs(x - 23.5) < 7.9)
            return left | right | middle | frame

        body_m = coverage(hull, w, h) & field
        fins_m = coverage(fins, w, h) & field & ~body_m
        band = body_m & (np.abs(yy + 0.5 - 22.5) < 1.5)
        marks += [(fins_m, FINS), (body_m, BOMB), (band, HAZARD)]
        spark, shine = [], [(21, 21), (21, 22), (20, 24), (20, 25)]  # блик вдоль корпуса
    axe_mix = ("axe_flame", "axe_snow", "axe_bolt", "axe_hat", "axe_dark", "axe_wind", "axe_king")
    if emblem in axe_mix:  # топор слева, лезвием наружу — как у топора с мечом; справа — знак другой силы
        haft, gold, blade = axe(w, h, 20.0, 16.5)
        marks += [(haft & field, WOOD), (gold & field, GOLD), (blade & field, STEEL)]
    if emblem == "axe_flame":
        draw_flame(small_flame(31.5, 27.0))
    if emblem == "axe_dark":
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "axe_snow":
        marks.append((coverage(lambda x, y: snowflake(x, y, 30.5, 22.5, 7.6, ((0.6, 2.2),)), w, h) & field, ICE))
    if emblem == "axe_bolt":
        marks.append((bolt_mask(0.72, 0.29, -0.07), GOLD))
    if emblem == "axe_hat":
        draw_hat(31.0, 29.5, 0.62)
    if emblem == "axe_wind":
        draw_tornado(31.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    if emblem == "axe_king":
        marks.append((grid_mask(KING_SMALL, 27, 14, w, h) & field, GOLD))
        gems.append((31, 19))
    king_mix = ("king_flame", "king_bolt", "king_snow", "king_wind", "king_dark", "king_hat")
    if emblem in king_mix:  # малый золотой король слева, справа — знак другой силы
        marks.append((grid_mask(KING_SMALL, 11, 14, w, h) & field, GOLD))
        gems.append((15, 19))
    if emblem == "king_flame":
        draw_flame(small_flame(31.5, 27.0))
    if emblem == "king_dark":
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "king_bolt":
        marks.append((bolt_mask(0.72, 0.29, -0.07), GOLD))
    if emblem == "king_snow":
        marks.append((coverage(lambda x, y: snowflake(x, y, 30.5, 22.5, 7.6, ((0.6, 2.2),)), w, h) & field, ICE))
    if emblem == "king_wind":
        draw_tornado(31.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    if emblem == "king_hat":
        draw_hat(31.0, 29.5, 0.62)
    hat_mix = ("hat_flame", "hat_snow", "hat_bolt", "hat_wind", "hat_dark")
    if emblem in hat_mix:  # малая шляпа мага слева, справа — знак другой силы
        draw_hat(16.0, 29.5, 0.62)
    if emblem == "hat_flame":
        draw_flame(small_flame(31.5, 27.0))
    if emblem == "hat_dark":
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "hat_bolt":
        marks.append((bolt_mask(0.72, 0.29, -0.07), GOLD))
    if emblem == "hat_snow":
        marks.append((coverage(lambda x, y: snowflake(x, y, 30.5, 22.5, 7.6, ((0.6, 2.2),)), w, h) & field, ICE))
    if emblem == "hat_wind":
        draw_tornado(31.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    flame_mix = ("flame_snow", "flame_bolt", "flame_wind", "flame_dark")
    if emblem in flame_mix:  # малый огонёк слева, справа — знак другой силы
        draw_flame(small_flame(16.0, 27.0))
    if emblem == "flame_dark":
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "flame_bolt":
        marks.append((bolt_mask(0.72, 0.29, -0.07), GOLD))
    if emblem == "flame_snow":
        marks.append((coverage(lambda x, y: snowflake(x, y, 30.5, 22.5, 7.6, ((0.6, 2.2),)), w, h) & field, ICE))
    if emblem == "flame_wind":
        draw_tornado(31.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    snow_mix = ("snow_bolt", "snow_wind", "snow_dark")
    if emblem in snow_mix:  # снежинка слева (центр ровно в пикселе — симметрична), справа — знак другой силы
        marks.append((coverage(lambda x, y: snowflake(x, y, 16.5, 22.5, 7.6, ((0.6, 2.2),)), w, h) & field, ICE))
    if emblem == "snow_bolt":
        marks.append((bolt_mask(0.72, 0.29, -0.07), GOLD))
    if emblem == "snow_dark":
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "snow_wind":
        draw_tornado(31.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    if emblem in ("bolt_wind", "bolt_dark"):  # малая молния слева (как в `both`), справа — знак другой силы
        marks.append((bolt_mask(0.72, -0.31, -0.07), GOLD))
    if emblem == "bolt_wind":
        draw_tornado(31.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
    if emblem == "bolt_dark":
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    if emblem == "wind_dark":  # малый смерч слева (как у смерча с монетой), справа — чёрно-фиолетовый огонёк
        draw_tornado(15.5, 12.5, 30.0, 5.2, 12, 31.0, 0.8)
        draw_flame(small_flame(31.5, 27.0), tones=DARK_FIRE, tip=DARK_TIP)
    for mark, tones in under:
        put((shifted(mark, 1, 1) | shifted(mark, 2, 1) | shifted(mark, 1, 0) | shifted(mark, -1, 0)
             | shifted(mark, 0, 1) | shifted(mark, 0, -1)) & field & ~mark, WINE[4])
        put(mark, tones[2])
        put(mark & ~(shifted(mark, -1, 0) & shifted(mark, 0, -1)), tones[3])
        lit = mark & ~(shifted(mark, 1, 0) & shifted(mark, 0, 1))
        put(lit, tones[1])
        put(lit & (yy < h * 0.5), tones[0])
    for mark, tones in marks:
        drop = (shifted(mark, 1, 1) | shifted(mark, 2, 1)) & field & ~mark
        ring = (shifted(mark, 1, 0) | shifted(mark, -1, 0) | shifted(mark, 0, 1) | shifted(mark, 0, -1)) & field & ~mark
        put(ring, WINE[4])
        put(drop, WINE[4])
    for mark, tones in marks:
        put(mark, tones[2])
        put(mark & ~(shifted(mark, -1, 0) & shifted(mark, 0, -1)), tones[3])
        lit = mark & ~(shifted(mark, 1, 0) & shifted(mark, 0, 1))
        put(lit, tones[1])
        put(lit & (yy < h * 0.5), tones[0])
    if emblem == "crystal":  # снежинка внутри ледяного кристалла — шестигранник посередине щита
        C = CRYSTAL
        cx, top, sh, low, bot, hw = 23.5, 10.5, 16.5, 30.0, 37.0, 9.5
        poly = [(cx, top), (cx + hw, sh), (cx + hw, low), (cx, bot), (cx - hw, low), (cx - hw, sh)]
        gem = coverage(lambda x, y: in_poly(x, y, poly), w, h) & field
        put((shifted(gem, 1, 1) | shifted(gem, 2, 1)) & field & ~gem, WINE[4])
        put((shifted(gem, 1, 0) | shifted(gem, -1, 0) | shifted(gem, 0, 1) | shifted(gem, 0, -1)) & field & ~gem, WINE[4])
        xc, yc = xx + 0.5, yy + 0.5
        lft = xc < cx
        # грани: верхние скаты светлее, нижние темнее, левая сторона к свету
        put(gem & lft, C[3])  # грани глубже кромок — белая снежинка на них видна
        put(gem & ~lft, C[4])
        put(gem & lft & (yc < sh), C[2])
        put(gem & ~lft & (yc < sh), C[3])
        put(gem & lft & (yc > low), C[4])
        put(gem & ~lft & (yc > low), C[5])
        put(gem & (np.abs(xc - (cx - 7)) < 0.6) & (yc > sh) & (yc < low), C[2])  # отсвет на левой грани
        put(gem & ~(shifted(gem, -1, 0) & shifted(gem, 0, -1)), C[4])
        lit = gem & ~(shifted(gem, 1, 0) & shifted(gem, 0, 1))
        put(lit, C[1])
        put(lit & (yy < h * 0.5), C[0])
        # снежинка во льду: почти белая, тень на лёд вправо-вниз (каймы нет — дробит мелкие веточки)
        sf = coverage(lambda x, y: snowflake(x, y, cx, 23.5, 7.6, ((0.6, 2.2),)), w, h) & gem
        put(shifted(sf, 1, 1) & gem & ~sf, C[5])
        put(sf, ICE[1])
        put(sf & ~(shifted(sf, -1, 0) & shifted(sf, 0, -1)), ICE[2])
        put(sf & ~(shifted(sf, 1, 0) & shifted(sf, 0, 1)), ICE[0])
        for gx, gy in ((17, 14), (17, 13), (16, 14), (18, 14), (17, 15)):  # блик-звёздочка на верхнем скате
            put((xx == gx) & (yy == gy) & gem, C[0])
    for gx, gy in gems:  # рубины в обруче короны
        put((xx == gx) & (yy == gy), RUBY[1])
    if emblem == "piggy":  # пятачок — светлый диск с обводкой, ноздри; копытца; глаз, ухо, прорезь, монетка
        rim_s = snout & ~(shifted(snout, 1, 0) & shifted(snout, -1, 0) & shifted(snout, 0, 1) & shifted(snout, 0, -1))
        put(snout, PIG[1])
        put(rim_s, PIG[3])
        put(((xx == 9) | (xx == 11)) & (yy == 24), PIG[4])
        put(legs & ~shifted(legs, -1, 0), PIG_LEG[4])  # копытца — нижний ряд ножек
        put((xx == 16) & (yy == 18), PIG[3])  # внутри уха
        put((xx == 14) & (yy == 21), PIG[4])  # глаз
        put((yy == 17) & (xx >= 22) & (xx <= 27), PIG[4])  # прорезь для монет
        put((xx == 23) & (yy == 12), GOLD[0])  # блик на монетке (следа падения нет — с ним монетка походила на кнопку)
    for cone_, brim_ in hat_parts:  # малые шляпы: тень полей на тулье и звёздочка
        put(shifted(brim_, -1, 0) & cone_ & ~brim_, HAT[4])
    for sx, sy in hat_stars:
        for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
            put((xx == sx + dx) & (yy == sy + dy), STARLIGHT[1] if (dx, dy) != (0, 0) else STARLIGHT[0])
    if emblem == "wizard_hat":
        put(shifted(brim, -1, 0) & cone & ~brim, HAT[4])  # тень полей на тулье — поля отделены от конуса
        stars = [(22, 20), (24, 24), (21, 14), (19, 22)]
        for i, (sx, sy) in enumerate(stars):  # звёздочки на тулье: крестик из пяти пикселей, мелкие — точка
            if i < 2:
                for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
                    put((xx == sx + dx) & (yy == sy + dy), STARLIGHT[1] if (dx, dy) != (0, 0) else STARLIGHT[0])
            else:
                put((xx == sx) & (yy == sy), STARLIGHT[1])
        for sx, sy in ((32, 13), (12, 21), (35, 22)):  # волшебные искорки вокруг
            for dx, dy, col in ((0, 0, STARLIGHT[0]), (1, 0, STARLIGHT[2]), (-1, 0, STARLIGHT[2]), (0, 1, STARLIGHT[2]),
                                (0, -1, STARLIGHT[2])):
                if field[sy + dy, sx + dx]:
                    put((xx == sx + dx) & (yy == sy + dy), col)
    if emblem == "bomb":
        for sx, sy, col in spark:
            if field[sy, sx]:
                put((xx == sx) & (yy == sy), col)
        for sx, sy in shine:
            put((xx == sx) & (yy == sy), BOMB[0])
    # контур
    out = np.zeros((H, W, 4), np.uint8)
    out[1:-1, 1:-1] = img
    big = np.zeros((H, W), bool)
    big[1:-1, 1:-1] = body
    halo = (shifted(big, 1, 0) | shifted(big, -1, 0) | shifted(big, 0, 1) | shifted(big, 0, -1)) & ~big
    out[halo] = (*OUTLINE, 255)
    # искра на левом уголке
    out[3, 4] = (*GOLD[0], 255)
    out[2, 4] = (*GOLD[1], 255)
    out[3, 5] = (*GOLD[1], 255)
    return out


def preview(icon, path):
    zoom = 8
    h, w = icon.shape[:2]
    sheet = Image.new("RGBA", (w * 2 + 18, h + 10), hexc("#151821") + (255,))
    sheet.alpha_composite(Image.new("RGBA", (w + 8, h + 10), hexc("#3d4a3a") + (255,)), (w + 10, 0))
    im = Image.fromarray(icon, "RGBA")
    sheet.alpha_composite(im, (5, 5))
    sheet.alpha_composite(im, (w + 14, 5))
    big = sheet.resize((sheet.width * zoom, sheet.height * zoom), Image.NEAREST)
    # рядом в размере игры (×2): герб и монета coin_32
    coin = Image.open(os.path.join(OUT, "coin_32.png")).convert("RGBA")
    small = Image.new("RGBA", (w + coin.width + 18, h + 10), hexc("#151821") + (255,))
    small.alpha_composite(im, (5, 5))
    small.alpha_composite(coin, (w + 12, 5 + h - coin.height))
    small = small.resize((small.width * 2, small.height * 2), Image.NEAREST)
    out = Image.new("RGBA", (big.width + small.width + 16, big.height), hexc("#151821") + (255,))
    out.alpha_composite(big, (0, 0))
    out.alpha_composite(small, (big.width + 8, 8))
    out.save(path)


def main():
    os.makedirs(OUT, exist_ok=True)
    img = crest("bolt")
    Image.fromarray(img, "RGBA").save(os.path.join(OUT, "crest.png"))
    preview(img, os.path.join(OUT, "preview_crest.png"))
    snow = crest("snow")
    Image.fromarray(snow, "RGBA").save(os.path.join(OUT, "crest_snow.png"))
    preview(snow, os.path.join(OUT, "preview_crest_snow.png"))
    both = crest("both")
    Image.fromarray(both, "RGBA").save(os.path.join(OUT, "crest_both.png"))
    preview(both, os.path.join(OUT, "preview_crest_both.png"))
    love = crest("heart")
    Image.fromarray(love, "RGBA").save(os.path.join(OUT, "crest_heart.png"))
    preview(love, os.path.join(OUT, "preview_crest_heart.png"))
    rich = crest("coin")
    Image.fromarray(rich, "RGBA").save(os.path.join(OUT, "crest_coin.png"))
    preview(rich, os.path.join(OUT, "preview_crest_coin.png"))
    frost = crest("snow2")
    Image.fromarray(frost, "RGBA").save(os.path.join(OUT, "crest_snow2.png"))
    preview(frost, os.path.join(OUT, "preview_crest_snow2.png"))
    pawn = crest("pawn")
    Image.fromarray(pawn, "RGBA").save(os.path.join(OUT, "crest_pawn.png"))
    preview(pawn, os.path.join(OUT, "preview_crest_pawn.png"))
    for name in ("spikes", "crystal", "axe_sword", "axe_fangs", "axe_queen", "axes", "whirl", "fire_axe", "flame",
                 "flame_pawn", "flames", "flame_skull", "flame_sword", "flame_coin", "bomb", "dark_flame",
                 "dark_rook", "dark_skull", "dark_flames", "dark_portal", "dark_coin", "dark_eye", "wind",
                 "winds", "wind_coin", "wind_knight", "wind_sword",
                 "bolt_bishop", "bolt_coin", "bolt_plus", "bolts", "red_bolt", "bolt_one",
                 "pawns2", "bishops2", "knights2", "rooks2", "queens2", "reroll_coin", "wizard_hat",
                 "hat_coin", "hats", "hat_bishop", "meteor", "hat_meteor", "hat_plus",
                 "gold_king", "king_coin", "kings2", "king_queen", "king_skull", "piggy",
                 "axe_flame", "axe_snow", "axe_bolt", "axe_hat", "axe_dark", "axe_wind", "axe_king",
                 "king_flame", "king_bolt", "king_snow", "king_wind", "king_dark", "king_hat",
                 "hat_flame", "hat_snow", "hat_bolt", "hat_wind", "hat_dark",
                 "flame_snow", "flame_bolt", "flame_wind", "flame_dark",
                 "snow_bolt", "snow_wind", "snow_dark", "bolt_wind", "bolt_dark", "wind_dark"):
        icon = crest(name)
        Image.fromarray(icon, "RGBA").save(os.path.join(OUT, f"crest_{name}.png"))
        preview(icon, os.path.join(OUT, f"preview_crest_{name}.png"))
    axes = ("axe_sword", "axe_fangs", "axe_queen", "axes", "whirl", "fire_axe")
    sheet = Image.new("RGBA", (len(axes) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(axes):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_axes.png"))
    fires = ("flame_pawn", "flames", "flame_skull", "flame_sword", "flame_coin", "bomb")
    sheet = Image.new("RGBA", (len(fires) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(fires):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_fires.png"))
    darks = ("dark_rook", "dark_skull", "dark_flames", "dark_portal", "dark_coin", "dark_eye")
    sheet = Image.new("RGBA", (len(darks) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(darks):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_darks.png"))
    airs = ("winds", "wind_coin", "wind_knight", "wind_sword")
    sheet = Image.new("RGBA", (len(airs) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(airs):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_winds.png"))
    sparks = ("bolt_bishop", "bolt_coin", "bolt_plus", "bolts", "red_bolt")
    sheet = Image.new("RGBA", (len(sparks) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(sparks):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_bolts.png"))
    duos = ("pawns2", "bishops2", "knights2", "rooks2", "queens2", "reroll_coin")
    sheet = Image.new("RGBA", (len(duos) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(duos):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_pairs.png"))
    hats = ("hat_coin", "hats", "hat_bishop", "meteor", "hat_meteor", "hat_plus")
    sheet = Image.new("RGBA", (len(hats) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(hats):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_hats.png"))
    kings = ("gold_king", "king_coin", "kings2", "king_queen", "king_skull", "piggy")
    sheet = Image.new("RGBA", (len(kings) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(kings):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_kings.png"))
    mixes = ("axe_flame", "axe_snow", "axe_bolt", "axe_hat", "axe_dark", "axe_wind", "axe_king")
    sheet = Image.new("RGBA", (len(mixes) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(mixes):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_axe_mix.png"))
    kmix = ("king_flame", "king_bolt", "king_snow", "king_wind", "king_dark", "king_hat")
    sheet = Image.new("RGBA", (len(kmix) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(kmix):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_king_mix.png"))
    hmix = ("hat_flame", "hat_snow", "hat_bolt", "hat_wind", "hat_dark")
    sheet = Image.new("RGBA", (len(hmix) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(hmix):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_hat_mix.png"))
    fmix = ("flame_snow", "flame_bolt", "flame_wind", "flame_dark")
    sheet = Image.new("RGBA", (len(fmix) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(fmix):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_flame_mix.png"))
    smix = ("snow_bolt", "snow_wind", "snow_dark")
    sheet = Image.new("RGBA", (len(smix) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(smix):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_snow_mix.png"))
    bmix = ("bolt_wind", "bolt_dark")
    sheet = Image.new("RGBA", (len(bmix) * (W + 8) + 8, H + 16), hexc("#151821") + (255,))
    for i, name in enumerate(bmix):
        sheet.alpha_composite(Image.open(os.path.join(OUT, f"crest_{name}.png")), (8 + i * (W + 8), 8))
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.NEAREST).save(os.path.join(OUT, "preview_crest_bolt_mix.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
