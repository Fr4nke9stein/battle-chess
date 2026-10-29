"""Средневековый университет на четырёх клетках a1, a2, b1, b2 — слева от готического замка `breach_castle.py`, в его
стиле (бежевый песчаник с почти чёрной копотью, черепица, шифер, тёплые окна). Здесь был базар — пользователь
попросил заменить его зданием университета.

Главный корпус — библиотека: высокие стрельчатые окна с переплётом (`tracery`), контрфорсы с пинаклями, крутая
черепичная кровля со слуховыми окнами и трубами. Часовая башня выступает к доске (к вертикали c): стрельчатый
портал с дубовыми дверями (`door`) и крыльцом, над ним герб — синий щит с раскрытой книгой (`book_crest`), золотые
часы на двух видимых гранях (`clock`), зубчатый парапет с угловыми пинаклями, восьмигранный фонарь с проёмами,
шиферный купол-луковица, золотое навершие с флюгером. Западное крыло — щипец к доске с большим окном, на западной
стене эркер, как у пражского Каролинума: гранёный выступ на консоли (`frustum` с n = 8 — призма, перевёрнутая
пирамида снизу, шатёр сверху) со стрельчатыми окнами на гранях.

Тот же лучевой рендер, тела и фактура, что у замка (`breach_castle` — `Scene`, `render`, `box`, `prism`, `lathe`,
`frustum`, `battlement`, `pinnacle`, `windows`, `tracery`, `herald`-подобные узоры, `masonry`, `tiles`); палитра
циферблата добавлена в `bc.PALETTE` под номером 20. Свои оси — как у замка (u — от 1-й горизонтали к 8-й, мир −z;
v — от вертикали a к b, мир +x), начало — общая вершина клеток a1, a2, b1, b2 (нижний угол ромба a2); от начала
осей замка — на 3 клетки к 1-й горизонтали.

`py tools/breach_university.py [--quick]` — в assets/breach/landmarks: university.png и university_shadow.png
(точка начала осей печатается при запуске, ставится на нижний угол ромба a2); превью preview_university.png —
пустая доска с замком и университетом (замок — из castle.png, его точку начала осей считает `bc.canvas_frame`).
`--quick` — без сглаживания.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_castle as bc  # noqa: E402
from breach_castle import (DOOR, FLAG, GOLD, HERALD, IRON, LIT, SLATE, STONE, TILE, WINDOW, WOOD, ball,  # noqa: E402
                           battlement, box, deco, flagstones, frustum, lathe, lantern_slots, lighten, masonry, on,
                           pinnacle, prism, tiles, tracery, windows, arch_mask)

sys.argv = _argv
OUT = bc.OUT
CLOCK = 20
bc.PALETTE[CLOCK] = ["#05060c", "#0b0f20", "#121a34", "#1c2848", "#283860"]


# ---------------------------------------------------------------- узоры
def clock(face, s0, yc, radius):
    """Циферблат: золотой обод, тёмное поле, золотые стрелки."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        ds, dy = s - s0, p[:, 1] - yc
        r = np.hypot(ds, dy)
        inside = m & (r <= radius)
        out[inside] = CLOCK
        hands = (art.sd_segment2(ds, dy, (0, 0), (0, radius * 0.75)) < 0.55) | \
                (art.sd_segment2(ds, dy, (0, 0), (radius * 0.5, -radius * 0.2)) < 0.55)
        out[inside & ((r > radius - 1.1) | hands | (r < 0.8))] = GOLD
    return apply


def book_crest(face, s0, yc, size):
    """Герб университета: синий щит в золотой кайме, на нём раскрытая книга."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        ds, dy = (s - s0) / size, (p[:, 1] - yc) / size
        shield = (np.abs(ds) <= 1.0) & (dy <= 1.0) & (dy >= -1.4 + 0.8 * ds ** 2)
        inner = (np.abs(ds) <= 0.74) & (dy <= 0.78) & (dy >= -1.12 + 0.8 * ds ** 2)
        out[m & shield] = GOLD
        out[m & inner] = HERALD
        book = inner & (np.abs(ds) <= 0.58) & (np.abs(dy - 0.05 - 0.12 * np.abs(ds)) <= 0.3)
        out[m & book] = STONE
        off[m & book] += 2
        out[m & book & (np.abs(ds) < 0.07)] = HERALD  # корешок
    return apply


def door(face, s0, w, y0, y1):
    """Стрельчатый портал: светлый обвод, тёмный проём, дубовые двери с щелями досок."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        off[m & arch_mask(s, y, s0, w + 2.4, y0, y1 + 1.2)] += 1
        hole = m & arch_mask(s, y, s0, w, y0, y1)
        out[hole] = WOOD
        out[hole & ((((s - s0) / 1.5) % 1.0) < 0.22)] = DOOR
        out[hole & (np.abs(s - s0) < 0.35)] = DOOR
        out[hole & (np.abs(y - (y0 + (y1 - y0) * 0.45)) < 0.5)] = IRON
    return apply


# ---------------------------------------------------------------- университет
def build():
    groups = []
    eaves, front = 46.0, 6.0  # верх стен и южная плоскость главного корпуса

    # главный корпус — библиотека
    hall = deco(STONE,
                *[tracery("+v", u, 6.4, 14, 38) for u in (-14.5, 15.5, 27.5)],
                windows("+v", [-14.5, 15.5, 27.5], 2.6, 4, 10),
                windows("-u", [-28, -16, -4], 3.0, 6, 14),
                windows("-u", [-28, -16, -4], 4.0, 20, 38, [WINDOW, LIT, WINDOW], sill=STONE),
                masonry(seed=211), flagstones(212))
    groups.append([
        box(-36, 40, 0, eaves, -36, front, hall),
        prism([(-38, eaves), (front + 2, eaves), (-15, 74)], "u", -37.5, 41.5, deco(TILE, tiles(2.7, 3.0, 213))),
        box(-28, -24.4, 62, 76, -26, -22.4, deco(STONE, masonry(2.2, 2.4, seed=214), lighten(-1))),
        box(24, 27.6, 60, 74, -27, -23.4, deco(STONE, masonry(2.2, 2.4, seed=215), lighten(-1))),
    ])
    for k, u in enumerate((-20, 10, 21.5, 33.5)):
        groups.append([box(u - 1.5, u + 1.5, 0, eaves - 4, front, front + 3.2, deco(STONE, masonry(2.4, 2.6, seed=220 + k)))]
                      + pinnacle(u, front + 1.6, eaves - 4, 1.4, 6.0, seed=230 + k))
    for k, uc in enumerate((21.5, 33.5)):
        y0 = 51.0
        groups.append([
            box(uc - 3.2, uc + 3.2, y0, y0 + 10, -4, 2, deco(STONE, windows("+v", [uc], 2.6, y0 + 1.5, y0 + 8.5,
                                                                            [LIT if k else WINDOW]),
                                                              masonry(2.4, 2.6, seed=240 + k))),
            prism([(uc - 4.0, y0 + 9.5), (uc + 4.0, y0 + 9.5), (uc, y0 + 17)], "v", -4.6, 2.6,
                  deco(TILE, tiles(2.0, 2.2, 242 + k))),
        ])

    # часовая башня с порталом, гербом, часами, зубцами, фонарём и куполом
    tu0, tu1, tv0, tv1, top = -6.5, 9.5, 0.0, 17.0, 92.0
    tc_u, tc_v = (tu0 + tu1) / 2, (tv0 + tv1) / 2
    tower = deco(STONE,
                 door("+v", tc_u, 7.0, 0.5, 18), book_crest("+v", tc_u, 25.5, 3.0),
                 windows("+v", [tc_u], 3.2, 33, 44), windows("+v", [tc_u - 3, tc_u + 3], 2.4, 52, 62),
                 clock("+v", tc_u, 74, 5.0), windows("+v", [tc_u - 3, tc_u + 3], 2.2, 82, 89, DOOR),
                 windows("-u", [tc_v], 3.0, 22, 33), windows("-u", [tc_v], 3.0, 40, 50, [LIT]),
                 windows("-u", [tc_v - 3, tc_v + 3], 2.4, 56, 64), clock("-u", tc_v, 74, 5.0),
                 windows("-u", [tc_v - 3, tc_v + 3], 2.2, 82, 89, DOOR),
                 masonry(seed=251), flagstones(252))
    g = [
        box(tu0, tu1, 0, top, tv0, tv1, tower),
        battlement(tu0 - 0.8, tu1 + 0.8, top, top + 5, tv1 - 1.2, tv1 + 0.8, "u", 3.4, 1.5, deco(STONE, masonry(seed=253))),
        battlement(tu0 - 0.8, tu0 + 1.2, top, top + 5, tv0 - 0.8, tv1 + 0.8, "v", 3.4, 1.5, deco(STONE, masonry(seed=254))),
        battlement(tu0 - 0.8, tu1 + 0.8, top, top + 5, tv0 - 0.8, tv0 + 1.2, "u", 3.4, 1.5, deco(STONE, masonry(seed=255))),
        battlement(tu1 - 1.2, tu1 + 0.8, top, top + 5, tv0 - 0.8, tv1 + 0.8, "v", 3.4, 1.5, deco(STONE, masonry(seed=256))),
        lathe(tc_u, tc_v, [(4.6, top - 2), (4.6, top + 13), (5.6, top + 14), (5.6, top + 15.5), (5.0, top + 16),
                           (5.3, top + 18), (4.4, top + 21), (2.4, top + 24), (1.0, top + 26.5), (0.6, top + 27.5)],
              deco(SLATE, tiles(2.0, 2.0, 257), lantern_slots(tc_u, tc_v, top + 3, top + 11, 0.9))),
        ball((tc_u, top + 28, tc_v), 0.9, GOLD),
        box(tc_u - 0.3, tc_u + 0.3, top + 27, top + 35, tc_v - 0.3, tc_v + 0.3, IRON, r=0.1),
        box(tc_u - 0.25, tc_u + 0.25, top + 31.5, top + 34.2, tc_v + 0.3, tc_v + 4.6, FLAG, r=0.1),
        box(tc_u - 6.5, tc_u + 6.5, 0, 1.2, tv1, tv1 + 4.5, deco(STONE, flagstones(258))),  # крыльцо
        box(tc_u - 5.5, tc_u + 5.5, 1.2, 2.4, tv1, tv1 + 2.4, deco(STONE, flagstones(259))),
    ]
    for cu in (tu0 - 0.2, tu1 + 0.2):
        for cv in (tv0 - 0.2, tv1 + 0.2):
            g += pinnacle(cu, cv, top + 4, 1.1, 5.5, seed=260)
    groups.append(g)

    # западное крыло: щипец к доске с большим окном, эркер на западной стене
    wu0, wu1, wv0, wv1, weaves = -38.0, -20.0, front, 34.0, 40.0
    wum = (wu0 + wu1) / 2
    wing = deco(STONE,
                windows("+v", [wum - 4, wum + 4], 2.6, 5, 12),
                windows("+v", [wum - 4, wum + 4], 3.2, 18, 33, [LIT, WINDOW], sill=STONE),
                windows("-u", [12, 29], 2.8, 5, 12), windows("-u", [12, 29], 3.2, 18, 33, [WINDOW, WINDOW], sill=STONE),
                masonry(seed=271), flagstones(272))
    groups.append([
        box(wu0, wu1, 0, weaves, wv0, wv1, wing),
        prism([(wu0 - 1.5, weaves), (wu1 + 1.5, weaves), (wum, 68)], "v", wv0 - 1, wv1 + 1.5, deco(TILE, tiles(2.7, 3.0, 273))),
        prism([(wu0, weaves + 0.5), (wu1, weaves + 0.5), (wum, 66.5)], "v", wv1 - 0.6, wv1,
              deco(STONE, tracery("+v", wum, 6.0, 44, 62), masonry(seed=274))),
    ] + pinnacle(wum, wv1 - 0.3, 65.5, 1.2, 6.0, seed=275))
    ou, ov = wu0, 20.0  # эркер: гранёный выступ на консоли
    oriel = deco(STONE, *[windows((ou, ov, a), [0], 2.2, 19, 31, [LIT if a == 180 else WINDOW]) for a in (135, 180, 225)],
                 masonry(2.4, 2.4, seed=281))
    groups.append([
        frustum(ou, ov, 17, 34, 5.0, 5.0, oriel, n=8),
        frustum(ou, ov, 11, 17, 0.6, 5.0, deco(STONE, masonry(2.0, 2.0, seed=282)), n=8),
        frustum(ou, ov, 33.6, 44, 5.8, 0.0, deco(SLATE, tiles(1.8, 1.8, 283)), n=8),
        ball((ou, 44.3, ov), 0.7, GOLD),
    ])
    return groups


# ---------------------------------------------------------------- превью
def preview(body, shadow, anchor):
    """Пустая доска, замок (castle.png) и университет: замок — на нижнем углу ромба a5, университет — a2."""
    empty = Image.new("RGBA", (art.W, art.H), (0, 0, 0, 0))
    canvas = art.board_preview({c + k: (empty, empty) for c in "wb" for k in "PNBRQK"}, scale=1, demo=False,
                               **bc.MARGINS)
    ox = bc.MARGINS["margin_x"] + 8 * art.TW // 2 - art.TW // 2
    oy = bc.MARGINS["margin_top"]
    castle = Image.open(os.path.join(OUT, "castle.png")).convert("RGBA")
    castle_shadow = Image.open(os.path.join(OUT, "castle_shadow.png")).convert("RGBA")
    _, _, cax, cay = bc.canvas_frame(bc.Scene(bc.build()))
    at_castle = (ox - 56 - cax, oy + 105 - cay)
    at_uni = (ox - 140 - anchor[0], oy + 168 - anchor[1])
    canvas.alpha_composite(castle_shadow, at_castle)
    canvas.alpha_composite(shadow, at_uni)
    canvas.alpha_composite(castle, at_castle)
    canvas.alpha_composite(body, at_uni)
    return canvas.resize((canvas.width * 2, canvas.height * 2), Image.NEAREST)


def main():
    quick = "--quick" in sys.argv
    os.makedirs(OUT, exist_ok=True)
    scene = bc.Scene(build())
    body, shadow, anchor = bc.render(scene, ss=1 if quick else 3)
    suffix = "_quick" if quick else ""
    body.save(os.path.join(OUT, f"university{suffix}.png"))
    shadow.save(os.path.join(OUT, f"university_shadow{suffix}.png"))
    print("anchor", anchor, "size", body.size, flush=True)
    preview(body, shadow, anchor).save(os.path.join(OUT, f"preview_university{suffix}.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
