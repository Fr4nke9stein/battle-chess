"""Здания города игрока на доске — в стиле замка `breach_castle.py` (бежевый песчаник с копотью, черепица, шифер).

Правила пользователя: здание занимает свои клетки целиком, фигур на них не бывает; главное здание, набор
новобранцев, здания короля, ферзя и ладьи — 2×2, конь — три клетки буквой «Г», слон — три клетки в линию; цвета по
зерну забега не меняются, анимаций особых сил на зданиях нет.

- `main` — главное здание (2×2): донжон с шиферной пирамидой и большим флагом, вокруг низкие стены с зубцами и
  четыре круглые башни; ворота с гербом к 1-й горизонтали.
- `shop` — магазин (2×2, с 0.134): лавка мага — фиолетовые витрины, фахверк, полосатый навес, вывеска с
  монетой, башенка со светящимся шаром за домом, бочки и сундук у входа.
- `recruit` — набор новобранцев (2×2): казарма с фахверком сзади, спереди утоптанный двор за частоколом — соломенные
  чучела, стойка с копьями, бочки, знамя на шесте.
- `king` — здание короля (2×2): дворец с портиком на колоннах, угловыми павильонами, барабаном и шиферным куполом с
  золотыми рёбрами, наверху золотая корона с крестом.
- `queen` — здание ферзя (2×2): дворец под вальмовой кровлей, четыре круглые башенки с конусами и золотыми шарами,
  посередине восьмигранная башня с высоким шпилем, кольцом золотых шаров и шаром на вершине; щипец с розой.
- `rook` — здание ладьи (2×2): массивная круглая башня, как сама ладья, — откос, машикули, крупные зубцы; вокруг
  низкая стена с зубцами и воротами.
- `knight` — конюшня (буква «Г»: клетки (0,0), (1,0), (1,1) — угол сзади, внутренний угол к зрителю): два крыла с
  фахверком, ворота денников внутрь угла, сеновальные двери в щипцах, на перекрестье кровель — башенка с флюгером.
- `bishop` — церковь (три клетки в линию вдоль горизонталей): неф с контрфорсами и стрельчатыми окнами, фасад с
  порталом и розой к 1-й горизонтали, сзади колокольня с восьмигранным шпилем и золотым крестом.
Временные размеры (пользователь их ещё не назвал; выбрал «дорисовать остальные»): 1×1 — `pawn` (дом ополчения с
фахверком, каменная пешка у входа), `wind` (мельница: каменная башня, шатёр, четыре крыла с решёткой и парусиной —
`sail`), `spark` (башня с медными поясами, катушкой, медным шаром и громоотводом, голубые окна), `wizard` (башня мага с
балконом и синей шляпой-кровлей в звёздах, спереди башенка); 2×1 вдоль вертикалей — `forge` (кузня: дом с трубой,
открытый навес с горном, наковальней, бочкой, углём), `ice` (ледяной чертог из голубых блоков, ледяная черепица со
снегом на коньке, сосульки, круглая башня с хрустальным шпилем, кристаллы у основания), `treasury` (казна: портик с
монетой на фронтоне, решётки на окнах, золотой гребень, сундуки и столбики монет), `crypt` (склеп тёмного камня с
фиолетовыми окнами, шпиль с фиолетовым шаром, обелиски), `longhouse` (длинный дом варваров под соломой, скрещённые
доски-рога на коньке, щиты на щипце, крыльцо с рогатым черепом, столб с черепом). Светящиеся палитры (`VIOLET`,
`ARC`, `MAGIC`) светятся во всех тонах — без правок рендера; анимаций нет.

Любое здание можно отразить через вертикаль экрана (`mirror`: (u, v) → (Wv − v, Wu − u), имя с `_m`): фасады −u и
+v меняются местами, 2×1 и линия встают вдоль горизонталей.

Оси — как у замка: u — от 1-й горизонтали к 8-й (мир −z), v — от вертикали a к h (мир +x), y — вверх; начало — угол
клетки (0, 0) здания у вертикали a и 1-й горизонтали (левый угол ромба на экране); клетка (i, j) — u от i·L до
(i+1)·L, v от j·L до (j+1)·L. Видны грани −u (к 1-й горизонтали, светлее) и +v (к вертикали h).

`py tools/breach_city.py [имя | имя_m …] [--quick] [--preview]` — в assets/breach/city: `<имя>.png` и
`<имя>_shadow.png`, точка начала осей — `anchor_<имя>.txt`; превью preview_city.png (`LAYOUT` — семь зданий) и
preview_town.png (`TOWN` — поле, застроенное всеми 16 зданиями), preview_full.png (`FULL` — все 64 клетки, с
повторами). `--preview` — только собрать превью из готовых
файлов. `--quick` — без сглаживания.
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
from breach_castle import (DOOR, FLAG, GOLD, HERALD, IRON, LIT, SLATE, STONE, TILE, WINDOW, WOOD, arch_mask,  # noqa: E402
                           ball, battlement, box, deco, flagstones, frustum, gate, herald, lathe, lantern_slots,
                           lighten, masonry, on, pinnacle, prism, ring_battlement, ring_corbels, round_tower, slits,
                           tiles, tracery, windows)

sys.argv = _argv
OUT = os.path.join(bc.ROOT, "city")
L = bc.L
C = L  # середина 2×2

STRAW, PLASTER, EARTH = 21, 22, 23
bc.PALETTE[STRAW] = ["#2a1d08", "#5a4212", "#8f6c22", "#c49a3a", "#e6c46a"]
bc.PALETTE[PLASTER] = ["#1a1712", "#48402f", "#857a60", "#b8aa8a", "#dacfb0"]
bc.PALETTE[EARTH] = ["#1a120a", "#33241a", "#4e3a28", "#66503a", "#7e6649"]


# ---------------------------------------------------------------- узоры
def door(face, s0, w, y0, y1, pointed=True):
    """Портал: светлый обвод, дубовые двери с щелями досок и железной полосой."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        off[m & arch_mask(s, y, s0, w + 2.0, y0, y1 + 1.0, pointed)] += 1
        hole = m & arch_mask(s, y, s0, w, y0, y1, pointed)
        out[hole] = WOOD
        out[hole & ((((s - s0) / 1.5) % 1.0) < 0.22)] = DOOR
        out[hole & (np.abs(s - s0) < 0.35)] = DOOR
        out[hole & (np.abs(y - (y0 + (y1 - y0) * 0.45)) < 0.5)] = IRON
    return apply


def stalls(face, centres, w, y0, y1):
    """Ворота денников: доски, верхняя половина отдельно, косая перекладина буквой Z."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        for s0 in centres:
            hole = m & (np.abs(s - s0) <= w / 2) & (y >= y0) & (y < y1)
            off[m & (np.abs(s - s0) <= w / 2 + 0.9) & (y >= y0) & (y < y1 + 0.9) & ~hole] -= 1
            out[hole] = WOOD
            out[hole & ((((s - s0) / 1.3) % 1.0) < 0.2)] = DOOR
            mid = y0 + (y1 - y0) * 0.55
            out[hole & (np.abs(y - mid) < 0.45)] = DOOR
            diag = np.abs((s - s0) / (w / 2) - (2 * (y - y0) / (mid - y0) - 1)) < 0.18
            out[hole & (y < mid) & diag] = IRON
    return apply


def timber(y0, y1, post=5.5, seed=0):
    """Фахверк: выше y0 — светлая штукатурка, тёмные балки — пояса, стойки и раскосы в каждом втором пролёте."""
    def apply(p, n, out, off):
        vert = np.abs(n[:, 1]) < 0.55
        hl = np.hypot(n[:, 0], n[:, 2]) + 1e-6
        s = (-p[:, 0] * n[:, 2] + p[:, 2] * n[:, 0]) / hl
        y = p[:, 1]
        sel = vert & (out == STONE) & (y >= y0)
        out[sel] = PLASTER
        k = np.floor(s / post)
        f = (s / post) % 1.0
        t = np.clip((y - y0) / max(y1 - y0, 1e-6), 0, 1)
        beam = (y < y0 + 0.9) | (y > y1 - 0.9) | (np.abs(y - (y0 + y1) / 2) < 0.4) | (f < 0.16)
        brace = (np.abs(f - np.where(k % 2 == 0, t, 1 - t)) < 0.11) & (k % 3 != 1)
        out[sel & (beam | brace)] = WOOD
        h = bc.hash2(np.floor(s / 2.0), np.floor(y / 2.0), seed)
        off[sel & ~(beam | brace) & (h < 0.15)] -= 1
    return apply


def rose(face, s0, yc, r):
    """Роза: круглое окно с переплётом — обод, спицы, середина."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        ds, dy = s - s0, p[:, 1] - yc
        rr = np.hypot(ds, dy)
        a = np.arctan2(dy, ds)
        inside = m & (rr <= r)
        out[inside] = WINDOW
        spokes = (np.abs(((a * 8 / (2 * np.pi)) % 1.0) - 0.5) > 0.38) & (rr > r * 0.25)
        stone = inside & ((rr > r - 0.9) | spokes | (np.abs(rr - r * 0.25) < 0.5))
        out[stone] = STONE
        off[stone] += 1
        out[inside & (rr < r * 0.18)] = LIT
    return apply


def ribs(cu, cv, count, width, mat=GOLD):
    """Рёбра купола: узкие полосы по меридианам."""
    def apply(p, n, out, off):
        a = np.arctan2(p[:, 2] - cv, p[:, 0] - cu)
        f = (a * count / (2 * np.pi)) % 1.0
        out[(f < width) | (f > 1 - width)] = mat
    return apply


def bands(y_list, half, mat=IRON):
    def apply(p, n, out, off):
        for y in y_list:
            out[np.abs(p[:, 1] - y) < half] = mat
    return apply


def crenels(u0, u1, v0, v1, y0, h=5.0, t=2.0, mat=None, seed=0):
    """Зубчатый парапет по четырём сторонам прямоугольника."""
    mat = mat or deco(STONE, masonry(seed=seed), flagstones(seed + 1))
    return [
        battlement(u0, u1, y0, y0 + h, v1 - t, v1, "u", 3.6, 1.6, mat),
        battlement(u0, u0 + t, y0, y0 + h, v0, v1, "v", 3.6, 1.6, mat),
        battlement(u0, u1, y0, y0 + h, v0, v0 + t, "u", 3.6, 1.6, mat),
        battlement(u1 - t, u1, y0, y0 + h, v0, v1, "v", 3.6, 1.6, mat),
    ]


def flagpole(cu, cv, y0, height, flag_len=8.0, flag_h=5.0, mat=FLAG):
    tip = y0 + height
    return [box(cu - 0.4, cu + 0.4, y0, tip, cv - 0.4, cv + 0.4, IRON, r=0.1),
            ball((cu, tip + 0.4, cv), 0.8, GOLD),
            box(cu - 0.3, cu + 0.3, tip - flag_h - 0.5, tip - 0.5, cv + 0.4, cv + 0.4 + flag_len, mat, r=0.1)]


# ---------------------------------------------------------------- главное здание
def build_main():
    groups = []
    w0, w1, top = C - 32, C + 32, 24.0  # стены
    wall = lambda face_specs, seed: deco(STONE, *face_specs, masonry(seed=seed), flagstones(seed + 1))  # noqa: E731
    groups.append([
        box(w0, w0 + 4, 0, top, w0, w1, wall([gate("-u", C, 10, 0.5, 17), herald("-u", C, 20.5, 2.4)], 311)),
        box(w0, w1, 0, top, w1 - 4, w1, wall([slits("+v", [C - 14, C, C + 14], 9, 16)], 313)),
        box(w0, w1, 0, top, w0, w0 + 4, wall([], 315)),
        box(w1 - 4, w1, 0, top, w0, w1, wall([], 317)),
    ] + crenels(w0 - 0.6, w1 + 0.6, w0 - 0.6, w1 + 0.6, top, 4.5, 2.2, seed=319))

    k0, k1, ktop = C - 15, C + 15, 54.0  # донжон
    keep = deco(STONE,
                windows("-u", [C - 6, C + 6], 3.0, 28, 37, [LIT, WINDOW], sill=STONE),
                windows("-u", [C], 3.2, 42, 50),
                windows("+v", [C - 6, C + 6], 3.0, 28, 37, [WINDOW, WINDOW], sill=STONE),
                windows("+v", [C], 3.2, 42, 50, [LIT]),
                masonry(seed=321), flagstones(322))
    groups.append([
        box(k0, k1, 0, ktop, k0, k1, keep),
        frustum(C, C, ktop, ktop + 16, 13.5, 0.0, deco(SLATE, tiles(2.4, 2.6, 323)), n=4),
    ] + crenels(k0 - 0.8, k1 + 0.8, k0 - 0.8, k1 + 0.8, ktop, 5.0, 2.2, seed=324)
      + flagpole(C, C, ktop + 14, 14, 11, 7))

    for k, (cu, cv) in enumerate(((w0 + 2, w0 + 2), (w0 + 2, w1 - 2), (w1 - 2, w0 + 2), (w1 - 2, w1 - 2))):
        tower = round_tower(cu, cv, 7.5, 36, 18, seed=330 + 10 * k,
                            faces=(slits((cu, cv, 180), [0], 20, 26), slits((cu, cv, 90), [0], 20, 26)))
        groups.append(tower[:5])  # без флажков: флаг один — на донжоне
    return groups


# ---------------------------------------------------------------- набор новобранцев
def dummy(cu, cv, seed):
    return [box(cu - 0.5, cu + 0.5, 0, 12, cv - 0.5, cv + 0.5, WOOD, r=0.1),
            box(cu - 0.4, cu + 0.4, 8.6, 9.4, cv - 4, cv + 4, WOOD, r=0.1),
            lathe(cu, cv, [(1.6, 4), (2.4, 6), (2.5, 9), (1.8, 11)], deco(STRAW, tiles(1.0, 1.4, seed))),
            ball((cu, 12.6, cv), 1.9, deco(STRAW, tiles(0.9, 1.2, seed + 1))),
            ball((cu, 1.0, cv), 1.6, deco(STRAW, lighten(-1)))]


def barrel(cu, cv, seed):
    return [lathe(cu, cv, [(2.4, 0), (2.9, 2.6), (2.4, 5.2)], deco(WOOD, bands([0.9, 4.3], 0.4)))]


def build_recruit():
    groups = []
    edge = 2 * L
    groups.append([box(2, edge - 2, 0, 0.8, 2, edge - 2, deco(EARTH, flagstones(401)))])  # утоптанный двор

    h0, h1, eaves = 50.0, 87.0, 22.0  # казарма сзади
    hall = deco(STONE, timber(8, eaves, seed=402),
                door("-u", 30, 6, 0.8, 13), door("-u", 62, 6, 0.8, 13),
                windows("-u", [14, 46, 78], 3.0, 4, 8.5, [WINDOW, LIT, WINDOW], pointed=False),
                windows("-u", [14, 22, 38, 54, 70, 78], 2.4, 12.5, 18.5, [WINDOW, LIT, WINDOW, WINDOW, LIT, WINDOW],
                        pointed=False),
                windows("+v", [62, 76], 2.6, 12.5, 18.5, [WINDOW, LIT], pointed=False),
                masonry(seed=403))
    groups.append([
        box(h0, h1, 0.8, eaves, 5, 87, hall),
        prism([(h0 - 2.2, eaves), (h1 + 2.2, eaves), ((h0 + h1) / 2, 40)], "v", 3.5, 88.5, deco(TILE, tiles(2.4, 2.8, 404))),
        prism([(h0, eaves), (h1, eaves), ((h0 + h1) / 2, 38.5)], "v", 86.4, 87,
              deco(STONE, timber(eaves, 38, seed=405), windows("+v", [(h0 + h1) / 2], 2.6, 27, 32, [WINDOW], pointed=False))),
        box(58, 61.5, 34, 45, 20, 23.5, deco(STONE, masonry(2.2, 2.4, seed=406), lighten(-1))),
    ])

    fence = deco(WOOD, tiles(3.0, 1.1, 407))
    groups.append([
        battlement(3, 5, 0.8, 9.5, 4, 32, "v", 1.9, 0.5, fence),
        battlement(3, 5, 0.8, 9.5, 52, 88, "v", 1.9, 0.5, fence),
        battlement(3, 50, 0.8, 9.5, 86, 88, "u", 1.9, 0.5, fence),
        box(3, 5.2, 0.8, 13, 30.5, 32.7, WOOD, r=0.2), box(3, 5.2, 0.8, 13, 51.3, 53.5, WOOD, r=0.2),
        box(3, 5.2, 12, 13.6, 30.5, 53.5, WOOD, r=0.2),
    ])
    groups.append(dummy(22, 66, 410) + dummy(34, 76, 412) + dummy(30, 56, 414))
    rack = [box(44.6, 45.6, 0.8, 11, 30, 31, WOOD, r=0.1), box(44.6, 45.6, 0.8, 11, 44, 45, WOOD, r=0.1),
            box(44.4, 45.8, 7.6, 8.6, 29.5, 45.5, WOOD, r=0.1)]
    for k, v in enumerate((32.5, 35.5, 38.5, 41.5)):
        rack += [box(45.4, 46.1, 0.8, 15, v - 0.35, v + 0.35, WOOD, r=0.1),
                 frustum(45.75, v, 15, 18, 0.9, 0.0, IRON)]
    groups.append(rack)
    groups.append(barrel(42, 80, 420) + barrel(46, 75, 421) + barrel(43.5, 77.5, 422))
    groups.append(flagpole(12, 14, 0.8, 30, 10, 6))
    return groups


# ---------------------------------------------------------------- король
def build_king():
    groups = []
    u0, u1, v0, v1, eaves = 16.0, 84.0, 8.0, 84.0, 30.0
    block = deco(STONE,
                 door("-u", C, 7, 0.5, 15, pointed=False),
                 windows("-u", [C - 22, C + 22], 3.0, 4, 11, pointed=False),
                 windows("-u", [C - 22, C + 22], 3.4, 16, 25, [LIT, WINDOW], pointed=False, sill=STONE),
                 windows("+v", [30, 42, 54, 66], 3.0, 4, 11, pointed=False),
                 windows("+v", [30, 42, 54, 66], 3.4, 16, 25, [WINDOW, LIT, WINDOW, WINDOW], pointed=False, sill=STONE),
                 masonry(seed=501), flagstones(502))
    groups.append([box(u0, u1, 0, eaves, v0, v1, block)]
                  + crenels(u0 - 0.6, u1 + 0.6, v0 - 0.6, v1 + 0.6, eaves, 3.2, 1.6, seed=503))

    # угловые павильоны с пирамидальными кровлями
    for k, (cu, cv) in enumerate(((u0 + 7, v0 + 7), (u0 + 7, v1 - 7), (u1 - 7, v0 + 7), (u1 - 7, v1 - 7))):
        pav = deco(STONE, windows("-u", [cv], 2.6, 20, 30, [LIT if k == 1 else WINDOW]),
                   windows("+v", [cu], 2.6, 20, 30), masonry(seed=510 + k))
        groups.append([box(cu - 8, cu + 8, 0, 36, cv - 8, cv + 8, pav),
                       frustum(cu, cv, 36, 50, 9.2, 0.0, deco(SLATE, tiles(2.2, 2.4, 515 + k)), n=4),
                       ball((cu, 50.4, cv), 0.9, GOLD)])

    # портик к 1-й горизонтали: колонны, антаблемент, фронтон, ступени
    port = [box(3, 16, 0, 1.2, C - 16, C + 16, deco(STONE, flagstones(520))),
            box(5, 16, 1.2, 2.4, C - 15, C + 15, deco(STONE, flagstones(521))),
            box(5, 16.5, 21, 24.5, C - 15, C + 15, deco(STONE, masonry(1.8, 3.0, seed=522), lighten(1))),
            prism([(C - 16, 24.5), (C + 16, 24.5), (C, 33)], "u", 4.4, 16.5,
                  deco(TILE, tiles(2.2, 2.6, 523))),
            prism([(C - 14.5, 24.5), (C + 14.5, 24.5), (C, 31.5)], "u", 4.6, 5.3,
                  deco(STONE, herald("-u", C, 27.6, 1.8), lighten(1)))]
    for v in (C - 12, C - 4, C + 4, C + 12):
        port.append(lathe(7.5, v, [(2.0, 2.4), (2.0, 3.4), (1.4, 4.2), (1.4, 20), (2.0, 20.6), (2.0, 21.2)],
                          deco(STONE, lighten(1))))
    groups.append(port)

    # барабан, купол с золотыми рёбрами, корона с крестом
    cu, cv = (u0 + u1) / 2, (v0 + v1) / 2
    drum = deco(STONE, windows((cu, cv, 180), [0], 2.4, 36, 43, [LIT]), windows((cu, cv, 135), [0], 2.4, 36, 43),
                windows((cu, cv, 90), [0], 2.4, 36, 43), masonry(2.2, 3.0, seed=530))
    dome_r, dome_y = 15.0, 46.0
    profile = [(dome_r * math.cos(t), dome_y + dome_r * math.sin(t)) for t in np.linspace(0, math.pi / 2, 12)]
    crown_y = dome_y + dome_r - 0.8
    groups.append([
        lathe(cu, cv, [(14.2, eaves - 1), (14.2, dome_y - 1.5), (15.6, dome_y - 0.5), (15.6, dome_y + 0.3)], drum),
        lathe(cu, cv, profile, deco(SLATE, tiles(2.0, 2.2, 531), ribs(cu, cv, 12, 0.07))),
        lathe(cu, cv, [(3.6, crown_y), (5.0, crown_y), (5.0, crown_y + 2.2), (3.6, crown_y + 2.2)], GOLD, solid=False),
        ring_battlement(cu, cv, 3.8, 5.0, crown_y + 2.2, crown_y + 6.2, 4.6, 2.6, GOLD),
        lathe(cu, cv, [(3.7, crown_y + 0.5), (3.7, crown_y + 4.5), (2.2, crown_y + 7.0), (0.0, crown_y + 7.6)],
              deco(FLAG, lighten(1))),
        ball((cu, crown_y + 8.0, cv), 1.1, GOLD),
        box(cu - 0.45, cu + 0.45, crown_y + 8.5, crown_y + 14.5, cv - 0.45, cv + 0.45, GOLD, r=0.1),
        box(cu - 0.45, cu + 0.45, crown_y + 11.6, crown_y + 12.6, cv - 2.2, cv + 2.2, GOLD, r=0.1),
    ])
    return groups


# ---------------------------------------------------------------- ферзь
def build_queen():
    groups = []
    u0, u1, v0, v1, eaves = 12.0, 80.0, 12.0, 80.0, 28.0
    block = deco(STONE,
                 door("-u", C, 6.5, 0.5, 14),
                 windows("-u", [C - 18, C - 9, C + 9, C + 18], 2.8, 16, 24, [WINDOW, LIT, WINDOW, WINDOW], sill=STONE),
                 windows("-u", [C - 18, C + 18], 2.6, 4, 11),
                 windows("+v", [26, 36, 56, 66], 2.8, 16, 24, [WINDOW, WINDOW, LIT, WINDOW], sill=STONE),
                 windows("+v", [26, 46, 66], 2.6, 4, 11),
                 masonry(seed=601), flagstones(602))
    groups.append([
        box(u0, u1, 0, eaves, v0, v1, block),
        frustum(C, C, eaves, eaves + 15, (u1 - u0) / 2 + 2, 20, deco(TILE, tiles(2.4, 2.6, 603)), n=4),
    ])
    # щипец с розой над входом
    groups.append([
        box(u0 - 1, u0 + 8, eaves - 2, eaves + 6, C - 9, C + 9, deco(STONE, rose("-u", C, eaves + 5.5, 4.4), masonry(seed=604))),
        prism([(u0 - 1.6, eaves + 5.5), (u0 + 9, eaves + 5.5), (u0 + 9, eaves + 16)], "v", C - 9.6, C + 9.6,
              deco(TILE, tiles(2.0, 2.4, 605))),
        prism([(C - 9, eaves + 6), (C + 9, eaves + 6), (C, eaves + 18)], "u", u0 - 1, u0 - 0.2,
              deco(STONE, rose("-u", C, eaves + 5.5, 4.4), masonry(seed=606))),
        frustum(u0 - 0.6, C, eaves + 17, eaves + 23, 0.8, 0.0, GOLD),
    ])
    # круглые башенки по углам
    for k, (cu, cv) in enumerate(((u0, v0), (u0, v1), (u1, v0), (u1, v1))):
        tw = deco(STONE, windows((cu, cv, 180), [0], 2.2, 22, 29, [LIT if k == 1 else WINDOW]),
                  windows((cu, cv, 90), [0], 2.2, 22, 29), masonry(1.9, 2.6, seed=610 + k))
        groups.append([lathe(cu, cv, [(7.0, 0), (6.0, 6), (6.0, 34)], tw),
                       lathe(cu, cv, [(7.2, 33.5), (6.6, 35), (0.0, 52)], deco(SLATE, tiles(2.0, 2.0, 615 + k))),
                       ball((cu, 52.6, cv), 1.1, GOLD)])
    # восьмигранная башня со шпилем, кольцо шаров и шар на вершине
    tower = deco(STONE, *[windows((C, C, a), [0], 2.4, 48, 58, [LIT if a == 180 else WINDOW]) for a in (90, 135, 180)],
                 masonry(2.2, 2.8, seed=620))
    top = 62.0
    g = [frustum(C, C, eaves + 10, top, 9.0, 9.0, tower, n=8),
         frustum(C, C, top - 0.5, top + 1.5, 10.4, 10.4, deco(STONE, lighten(1)), n=8),
         frustum(C, C, top + 1.5, top + 32, 9.6, 0.0, deco(SLATE, tiles(2.0, 2.0, 621)), n=8),
         ball((C, top + 33.6, C), 2.0, GOLD)]
    for a in np.linspace(0, 2 * np.pi, 8, endpoint=False):
        g.append(ball((C + 10.0 * math.cos(a + np.pi / 8), top + 2.4, C + 10.0 * math.sin(a + np.pi / 8)), 1.2, GOLD))
    groups.append(g)
    return groups


# ---------------------------------------------------------------- ладья
def build_rook():
    groups = []
    e0, e1, top = 3.0, 2 * L - 3.0, 14.0  # низкая стена по краю
    wall = lambda specs, seed: deco(STONE, *specs, masonry(seed=seed), flagstones(seed + 1))  # noqa: E731
    groups.append([
        box(e0, e0 + 3.5, 0, top, e0, e1, wall([gate("-u", C, 8, 0.5, 11.5)], 701)),
        box(e0, e1, 0, top, e1 - 3.5, e1, wall([slits("+v", [24, 68], 5, 10)], 703)),
        box(e0, e1, 0, top, e0, e0 + 3.5, wall([], 705)),
        box(e1 - 3.5, e1, 0, top, e0, e1, wall([], 707)),
    ] + crenels(e0 - 0.5, e1 + 0.5, e0 - 0.5, e1 + 0.5, top, 4.0, 2.0, seed=709))
    groups.append([box(e0 + 3.5, e1 - 3.5, 0, 0.8, e0 + 3.5, e1 - 3.5, deco(STONE, flagstones(711)))])

    r, h = 20.0, 50.0  # башня-ладья: откос, ствол, машикули, вынос, крупные зубцы
    shaft = deco(STONE, slits((C, C, 180), [0], 18, 26), slits((C, C, 90), [0], 18, 26),
                 slits((C, C, 135), [0], 32, 40), slits((C, C, 180), [0], 38, 44),
                 windows((C, C, 135), [0], 3.2, 0.5, 9, DOOR), masonry(seed=712))
    groups.append([
        lathe(C, C, [(r + 4.5, 0), (r + 0.8, 12), (r, 14), (r, h)], shaft),
        lathe(C, C, [(r - 0.5, h - 6), (r + 3.6, h - 1), (r + 3.6, h + 0.5), (r - 0.5, h + 0.5)],
              deco(STONE, ring_corbels(C, C, r + 3.6, h - 6, h - 1, 3.2), masonry(seed=713))),
        ring_battlement(C, C, r - 1.2, r + 3.6, h + 0.5, h + 10, 8.5, 3.6, deco(STONE, masonry(seed=714), flagstones(715))),
        lathe(C, C, [(r - 1, h + 0.5), (r - 1, h + 1.5)], deco(STONE, flagstones(716))),
    ] + flagpole(C, C, h + 1.5, 16, 10, 6))
    return groups


# ---------------------------------------------------------------- конь (конюшня буквой «Г»)
def build_knight():
    groups = []
    eaves, ridge = 18.0, 36.0
    a_v0, a_v1 = 7.0, 40.0  # крыло A вдоль u: клетки (0,0), (1,0)
    b_u0, b_u1 = 53.0, 86.0  # крыло B вдоль v: клетки (1,0), (1,1)
    wing_a = deco(STONE, timber(7, eaves, seed=801),
                  stalls("+v", [12, 22, 32, 42], 6.0, 0.5, 11),
                  windows("+v", [17, 27, 37], 1.8, 12.5, 15.5, [WINDOW, LIT, WINDOW], pointed=False),
                  windows("-u", [(a_v0 + a_v1) / 2], 3.0, 3, 9, [WINDOW], pointed=False),
                  masonry(seed=802))
    wing_b = deco(STONE, timber(7, eaves, seed=803),
                  stalls("-u", [50, 60, 70, 80], 6.0, 0.5, 11),
                  windows("-u", [55, 65, 75], 1.8, 12.5, 15.5, [LIT, WINDOW, WINDOW], pointed=False),
                  windows("+v", [(b_u0 + b_u1) / 2], 3.0, 3, 9, [WINDOW], pointed=False),
                  masonry(seed=804))
    am, bm = (a_v0 + a_v1) / 2, (b_u0 + b_u1) / 2
    groups.append([
        box(5, b_u1, 0, eaves, a_v0, a_v1, wing_a),
        prism([(a_v0 - 2.2, eaves), (a_v1 + 2.2, eaves), (am, ridge)], "u", 3.6, b_u1 + 1.4, deco(TILE, tiles(2.4, 2.8, 805))),
        prism([(a_v0, eaves), (a_v1, eaves), (am, ridge - 1.5)], "u", 4.4, 5.0,
              deco(STONE, timber(eaves, ridge, seed=806), stalls("-u", [am], 5.0, eaves + 1.5, eaves + 8))),
    ])
    groups.append([
        box(b_u0, b_u1, 0, eaves, a_v0, 87, wing_b),
        prism([(b_u0 - 2.2, eaves), (b_u1 + 2.2, eaves), (bm, ridge)], "v", a_v0 - 1.4, 88.4, deco(TILE, tiles(2.4, 2.8, 807))),
        prism([(b_u0, eaves), (b_u1, eaves), (bm, ridge - 1.5)], "v", 86.4, 87,
              deco(STONE, timber(eaves, ridge, seed=808), stalls("+v", [bm], 5.0, eaves + 1.5, eaves + 8))),
    ])
    # башенка с флюгером на перекрестье кровель
    cu, cv = bm, am
    groups.append([
        box(cu - 3.5, cu + 3.5, ridge - 4, ridge + 5, cv - 3.5, cv + 3.5,
            deco(WOOD, windows("-u", [cv], 2.6, ridge + 0.5, ridge + 4, DOOR, pointed=False),
                 windows("+v", [cu], 2.6, ridge + 0.5, ridge + 4, DOOR, pointed=False))),
        frustum(cu, cv, ridge + 5, ridge + 12, 4.6, 0.0, deco(SLATE, tiles(1.6, 1.8, 809)), n=4),
        box(cu - 0.3, cu + 0.3, ridge + 11, ridge + 17, cv - 0.3, cv + 0.3, IRON, r=0.1),
        box(cu - 0.3, cu + 0.3, ridge + 14.6, ridge + 15.6, cv - 3.4, cv + 3.4, GOLD, r=0.1),
        frustum(cu, cv + 3.4, ridge + 14.3, ridge + 15.9, 1.0, 1.0, GOLD, n=4),
    ])
    # сено у щипцов
    groups.append([ball((6.5, 2.2, 13), 2.6, deco(STRAW, tiles(1.0, 1.3, 810))),
                   ball((6.0, 2.0, 34.5), 2.3, deco(STRAW, tiles(1.0, 1.3, 811))),
                   ball((80, 2.2, 88.5), 2.4, deco(STRAW, tiles(1.0, 1.3, 812)))])
    return groups


# ---------------------------------------------------------------- слон (церковь в линию)
def build_bishop():
    groups = []
    n0, n1, v0, v1, eaves, ridge = 9.0, 104.0, 9.0, 37.0, 28.0, 50.0
    vm = (v0 + v1) / 2
    bays = [24, 38, 52, 66, 80, 94]
    nave = deco(STONE, door("-u", vm, 6.5, 0.5, 15), rose("-u", vm, 21.5, 4.6),
                windows("+v", bays, 3.0, 8, 23, [WINDOW, LIT, WINDOW, WINDOW, LIT, WINDOW], sill=STONE),
                masonry(seed=901), flagstones(902))
    groups.append([
        box(n0, n1, 0, eaves, v0, v1, nave),
        prism([(v0 - 2.2, eaves), (v1 + 2.2, eaves), (vm, ridge)], "u", n0 + 0.6, n1 + 1, deco(TILE, tiles(2.6, 2.8, 903))),
        prism([(v0, eaves), (v1, eaves), (vm, ridge - 1)], "u", n0, n0 + 0.6,
              deco(STONE, windows("-u", [vm], 2.6, 33, 42), masonry(seed=904))),
        box(n0 - 3, n0, 0, 1.0, vm - 7, vm + 7, deco(STONE, flagstones(905))),
    ] + pinnacle(n0 + 0.3, vm, ridge - 2, 1.4, 7.5, seed=906)
      + pinnacle(n0 + 0.3, v0 + 0.5, eaves - 2, 1.4, 6, seed=908)
      + pinnacle(n0 + 0.3, v1 - 0.5, eaves - 2, 1.4, 6, seed=910))
    for k, u in enumerate((31, 45, 59, 73, 87)):
        groups.append([box(u - 1.5, u + 1.5, 0, eaves - 3, v1, v1 + 3.2, deco(STONE, masonry(2.4, 2.6, seed=920 + k)))]
                      + pinnacle(u, v1 + 1.6, eaves - 3, 1.3, 5.5, seed=930 + k))

    t0, t1, tv0, tv1, top = 104.0, 128.0, 10.0, 36.0, 64.0  # колокольня сзади
    tu, tv = (t0 + t1) / 2, (tv0 + tv1) / 2
    tower = deco(STONE, windows("-u", [tv - 4.5, tv + 4.5], 3.0, 48, 59, DOOR),
                 windows("+v", [tu - 4.5, tu + 4.5], 3.0, 48, 59, DOOR),
                 windows("+v", [tu], 2.6, 30, 38, [LIT]), windows("+v", [tu], 2.4, 10, 17),
                 masonry(seed=941), flagstones(942))
    groups.append([
        box(t0, t1, 0, top, tv0, tv1, tower),
        frustum(tu, tv, top - 1, top + 34, 11.5, 0.0, deco(SLATE, tiles(2.2, 2.2, 943)), n=8),
        box(tu - 0.5, tu + 0.5, top + 33, top + 42, tv - 0.5, tv + 0.5, GOLD, r=0.1),
        box(tu - 0.5, tu + 0.5, top + 38, top + 39, tv - 2.6, tv + 2.6, GOLD, r=0.1),
    ] + crenels(t0 - 0.8, t1 + 0.8, tv0 - 0.8, tv1 + 0.8, top, 4.0, 1.8, seed=944)
      + sum([pinnacle(cu, cv, top + 3, 1.2, 6, seed=950) for cu in (t0 - 0.2, t1 + 0.2) for cv in (tv0 - 0.2, tv1 + 0.2)], []))
    return groups


ICE, SNOW, DARK, VIOLET, COPPER, ARC, HAT, BONE, MAGIC, THATCH = range(24, 34)
bc.PALETTE[ICE] = ["#0e2236", "#1f4a6e", "#3c7fae", "#77b6dc", "#c6ecff"]  # голубой лёд
bc.PALETTE[SNOW] = ["#55667a", "#8496ad", "#b6c6d8", "#dce8f3", "#ffffff"]
bc.PALETTE[DARK] = ["#07060a", "#16131d", "#272231", "#3a3346", "#50475e"]  # тёмный камень склепа
bc.PALETTE[VIOLET] = ["#4a2290", "#6a34c0", "#8c52ea", "#b583ff", "#e2c8ff"]  # фиолетовый свет (все тоны светятся)
bc.PALETTE[COPPER] = ["#2a1208", "#5a2a12", "#8f4a1e", "#c2743a", "#e8a86a"]
bc.PALETTE[ARC] = ["#1a6a8a", "#2a9ac0", "#46c6e6", "#8fe6ff", "#e0faff"]  # голубой свет молний
bc.PALETTE[HAT] = ["#0a0f2a", "#141f52", "#22318a", "#3449b8", "#5a72e0"]  # синяя шляпа мага
bc.PALETTE[BONE] = ["#2a2418", "#5a5038", "#958a68", "#c8bd98", "#ece4c8"]
bc.PALETTE[MAGIC] = ["#3a2c8a", "#5446c0", "#7a6ae8", "#a898ff", "#e4dcff"]  # окна мага
bc.PALETTE[THATCH] = ["#1a1209", "#382714", "#5a4224", "#7a5e36", "#98794a"]  # старая солома кровли
bc.FLAT.update({VIOLET, ARC, MAGIC})
bc.TEXTURED.update({ICE, DARK})


def sdbox(q):
    return np.linalg.norm(np.maximum(q, 0), axis=1) + np.minimum(q.max(axis=1), 0)


def rbox(c, half, axis, ang, mat):
    """Повёрнутый ящик: ось 'u' — поворот в плоскости (v, y), 'v' — в плоскости (u, y); half — (вдоль оси, вдоль
    направления ang, поперёк)."""
    cu, cy, cv = c
    ca, sa = math.cos(ang), math.sin(ang)

    def f(p):
        dy = p[:, 1] - cy
        if axis == "u":
            x, d = p[:, 0] - cu, p[:, 2] - cv
        else:
            x, d = p[:, 2] - cv, p[:, 0] - cu
        a, b = d * ca + dy * sa, -d * sa + dy * ca
        return sdbox(np.stack([np.abs(x) - half[0], np.abs(a) - half[1], np.abs(b) - half[2]], 1))

    r = math.sqrt(sum(h * h for h in half))
    return bc.Part(f, [cu - r, cy - r, cv - r], [cu + r, cy + r, cv + r], mat)


def planks(width=1.6, seed=0):
    """Доски: вертикальные полосы с тёмными щелями и разным тоном."""
    def apply(p, n, out, off):
        vert = np.abs(n[:, 1]) < 0.55
        hl = np.hypot(n[:, 0], n[:, 2]) + 1e-6
        s = (-p[:, 0] * n[:, 2] + p[:, 2] * n[:, 0]) / hl
        k = np.floor(s / width)
        sel = vert & (out == WOOD)
        off[sel & (((s / width) % 1.0) < 0.2)] -= 1
        off[sel & (bc.hash2(k, k * 0, seed) > 0.8)] += 1
    return apply


def grille(face, centres, w, y0, y1):
    """Решётки на окнах казны."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        y = p[:, 1]
        for s0 in centres:
            inside = m & (np.abs(s - s0) <= w / 2) & (y >= y0) & (y < y1)
            out[inside & ((((s - s0) / 1.1) % 1.0) < 0.35)] = IRON
    return apply


def coin(face, s0, yc, r):
    """Золотая монета на фронтоне: диск, кант, выпуклая середина."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        rr = np.hypot(s - s0, p[:, 1] - yc)
        out[m & (rr <= r)] = GOLD
        off[m & (np.abs(rr - r * 0.7) < 0.35)] -= 1
    return apply


def shields(face, centres, yc, r, colours):
    """Круглые щиты на стене: поле в две краски по четвертям, железный обод, золотой умбон."""
    def apply(p, n, out, off):
        m, s = on(p, n, face)
        for k, s0 in enumerate(centres):
            ds, dy = s - s0, p[:, 1] - yc
            rr = np.hypot(ds, dy)
            disc = m & (rr <= r)
            a, b = colours[k % len(colours)]
            out[disc] = np.where((ds * dy) >= 0, a, b)[disc]
            out[disc & (rr > r - 0.6)] = IRON
            out[disc & (rr < 0.8)] = GOLD
    return apply


def stars(cu, cv, cell=2.6):
    """Золотые звёздочки на шляпе мага — редкие точки по сетке на поверхности."""
    def apply(p, n, out, off):
        a = np.arctan2(p[:, 2] - cv, p[:, 0] - cu)
        s = a * 9.0
        i, j = np.floor(s / cell), np.floor(p[:, 1] / (cell * 1.3))
        h = bc.hash2(i, j, 77)
        fs, fy = (s / cell) % 1.0, (p[:, 1] / (cell * 1.3)) % 1.0
        dot = (np.abs(fs - 0.5) < 0.22) & (np.abs(fy - 0.5) < 0.2)
        out[(h < 0.2) & dot & (out == HAT)] = GOLD
    return apply


def snowcap(y0):
    """Снег выше y0 на верхних поверхностях."""
    def apply(p, n, out, off):
        out[(p[:, 1] > y0) & (n[:, 1] > 0.3)] = SNOW
    return apply


def top_face(mat):
    def apply(p, n, out, off):
        out[n[:, 1] > 0.7] = mat
    return apply


def icicle(cu, cv, y1, length):
    return frustum(cu, cv, y1 - length, y1, 0.0, 0.8, deco(ICE, lighten(1)), n=4)


def crystal(cu, cv, h, r):
    return frustum(cu, cv, 0, h, r, 0.0, deco(ICE, lighten(1)), n=6)


def skull(c, size=1.5, along="v"):
    """Рогатый череп: кость и рога цепочкой шаров наружу и вверх."""
    cu, cy, cv = c
    parts = [ball((cu, cy, cv), size, BONE)]
    for side in (-1, 1):
        for k in range(6):
            t = k / 5
            d, up = side * (size * 0.8 + 2.4 * t), 0.2 + 2.6 * t ** 1.6
            pos = (cu, cy + up, cv + d) if along == "v" else (cu + d, cy + up, cv)
            parts.append(ball(pos, size * 0.55 * (1 - 0.55 * t), deco(BONE, lighten(-1 if t > 0.6 else 0))))
    return parts


def portico(u_front, u_back, vm, y_top, columns, half):
    """Колоннада с антаблементом и фронтоном, ступени (для дворца и казны)."""
    parts = [box(u_front - 3, u_back, 0, 1.2, vm - half - 1, vm + half + 1, deco(STONE, flagstones(1))),
             box(u_front - 1, u_back, 1.2, 2.4, vm - half, vm + half, deco(STONE, flagstones(2))),
             box(u_front - 0.5, u_back, y_top - 3.5, y_top, vm - half, vm + half, deco(STONE, lighten(1)))]
    for v in columns:
        parts.append(lathe(u_front + 2, v, [(2.0, 2.4), (2.0, 3.4), (1.4, 4.2), (1.4, y_top - 4.4), (2.0, y_top - 3.9),
                                            (2.0, y_top - 3.5)], deco(STONE, lighten(1))))
    return parts


# ---------------------------------------------------------------- пешка (1×1)
def build_pawn():
    groups = []
    u0, u1, v0, v1, eaves, ridge = 9.0, 38.0, 9.0, 37.0, 17.0, 32.0
    vm = (v0 + v1) / 2
    house = deco(STONE, timber(7, eaves, post=5.0, seed=1001),
                 door("-u", vm, 5.0, 0.5, 10, pointed=False),
                 windows("+v", [17, 30], 2.6, 9.5, 14, [LIT, WINDOW], pointed=False),
                 masonry(seed=1002))
    groups.append([
        box(u0, u1, 0, eaves, v0, v1, house),
        prism([(v0 - 2, eaves), (v1 + 2, eaves), (vm, ridge)], "u", u0 - 1.6, u1 + 1, deco(TILE, tiles(2.2, 2.6, 1003))),
        prism([(v0, eaves), (v1, eaves), (vm, ridge - 1.5)], "u", u0 - 0.6, u0,
              deco(STONE, timber(eaves, ridge - 1.5, post=4.5, seed=1004),
                   windows("-u", [vm], 2.4, eaves + 3, eaves + 7.5, [LIT], pointed=False))),
        ball((u0 - 1.4, ridge + 0.6, vm), 1.3, GOLD),
        box(30, 33.5, 22, 36, 30, 33.5, deco(STONE, masonry(2.0, 2.2, seed=1005), lighten(-1))),
    ])
    statue = deco(STONE, lighten(1))  # каменная пешка у входа
    groups.append([lathe(4.6, 5.8, [(2.4, 0), (2.4, 1.2), (1.6, 1.8), (1.0, 5.5), (1.8, 6.2), (1.8, 6.8), (0.9, 7.2)], statue),
                   ball((4.6, 8.6, 5.8), 1.7, statue)])
    groups.append(barrel(5.0, 40.0, 1006))
    return groups


# ---------------------------------------------------------------- огонь: кузня (2×1)
def build_forge():
    groups = []
    u0, u1, v0, v1, eaves, ridge = 10.0, 40.0, 5.0, 54.0, 20.0, 36.0
    um = (u0 + u1) / 2
    smithy = deco(STONE, door("-u", 17, 5.5, 0.5, 12),
                  windows("-u", [31, 43], 3.0, 6, 13, [LIT, LIT], pointed=False),
                  windows("+v", [um], 9.0, 0.6, 11, LIT),
                  masonry(seed=1101), flagstones(1102))
    groups.append([
        box(u0, u1, 0, eaves, v0, v1, smithy),
        prism([(u0 - 2, eaves), (u1 + 2, eaves), (um, ridge)], "v", v0 - 1.4, v1 + 0.6, deco(TILE, tiles(2.4, 2.8, 1103))),
        prism([(u0, eaves), (u1, eaves), (um, ridge - 1.5)], "v", v1 - 0.6, v1, deco(STONE, masonry(seed=1104))),
        box(26, 34, 0, 50, 40, 48, deco(STONE, masonry(2.2, 2.6, seed=1105), lighten(-1))),
        box(27, 33, 49.6, 50.6, 41, 47, LIT),
    ])
    shed = [box(u0, u1, 0, 0.6, v1, 86, deco(STONE, flagstones(1106))),
            prism([(u0 - 1.5, 16.5), (u1 + 1.5, 16.5), (um, 28)], "v", v1, 87.5, deco(TILE, tiles(2.2, 2.6, 1107)))]
    for cu, cv in ((u0 + 1, 85), (u1 - 1, 85), (u0 + 1, 70)):
        shed.append(box(cu - 1, cu + 1, 0.6, 17, cv - 1, cv + 1, deco(WOOD, planks(1.0, 1108))))
    shed.append(box(u0, u1, 15.6, 17, 84.3, 85.7, WOOD, r=0.2))
    groups.append(shed)
    groups.append([  # горн с углями, колпак, наковальня, бочка, уголь
        box(14, 24, 0.6, 7, 57, 67, deco(STONE, masonry(1.6, 2.0, seed=1109), top_face(LIT))),
        frustum(19, 62, 11, 16.8, 6.0, 2.6, deco(STONE, masonry(1.6, 2.0, seed=1110), lighten(-1)), n=4),
        box(29.5, 32.5, 0.6, 4, 71.5, 74.5, IRON), box(28, 34, 4, 6, 72, 74, IRON), box(34, 36, 4.6, 5.6, 72.5, 73.5, IRON),
        lathe(34, 62, [(2.4, 0.6), (2.9, 3.0), (2.5, 5.4)], deco(WOOD, bands([1.5, 4.4], 0.4), top_face(WINDOW))),
        ball((16, 1.4, 79), 2.3, IRON), ball((18.6, 1.1, 81.6), 1.8, IRON), ball((15, 1.0, 83.2), 1.6, IRON),
    ])
    return groups


# ---------------------------------------------------------------- лёд: ледяной чертог (2×1)
def build_ice():
    groups = []
    u0, u1, v0, v1, eaves, ridge = 9.0, 37.0, 4.0, 62.0, 22.0, 42.0
    um = (u0 + u1) / 2
    hall = deco(ICE, windows("-u", [16, 28, 40, 52], 2.8, 8, 17, ARC, sill=SNOW), masonry(3.2, 5.0, seed=1201),
                flagstones(1202))
    groups.append([
        box(u0, u1, 0, eaves, v0, v1, hall),
        prism([(u0 - 2, eaves), (u1 + 2, eaves), (um, ridge)], "v", v0 - 1.2, v1 + 0.8,
              deco(ICE, tiles(2.4, 3.0, 1203), lighten(-1), snowcap(ridge - 5))),
    ] + [icicle(u0 - 1.4, v, eaves, 3.0 + 3.5 * ((k * 7) % 5) / 4) for k, v in enumerate(np.arange(6, 62, 4.6))])
    tc = 75.0
    tower = deco(ICE, door((um, tc, 90), 0, 5.5, 0.5, 11),
                 windows((um, tc, 180), [0], 2.6, 20, 28, ARC), windows((um, tc, 135), [0], 2.6, 30, 37, ARC),
                 windows((um, tc, 90), [0], 2.6, 22, 30, ARC), masonry(3.0, 4.0, seed=1204))
    groups.append([
        lathe(um, tc, [(12.5, 0), (11, 6), (10, 40), (11.5, 44)], tower),
        lathe(um, tc, [(12.4, 43.5), (11.5, 46), (0.0, 68)], deco(ICE, tiles(2.2, 2.4, 1205), snowcap(60))),
        frustum(um, tc, 66, 80, 2.2, 0.0, deco(ICE, lighten(1)), n=6),
    ] + [icicle(um + 11.6 * math.cos(a), tc + 11.6 * math.sin(a), 44, 3.5) for a in np.radians([110, 150, 190, 230])])
    groups.append([crystal(u, v, h, r) for u, v, h, r in
                   ((4, 8, 13, 2.2), (6, 4.5, 8, 1.6), (3.5, 13.5, 7, 1.4), (42, 58, 11, 2.0), (41.5, 64, 7, 1.4),
                    (5.5, 67, 9, 1.8), (4, 86, 7, 1.5))])
    return groups


# ---------------------------------------------------------------- ветер: мельница (1×1)
def sail(cu, cy, cv, ang, length=20.0, width=3.6):
    """Крыло мельницы в плоскости (v, y): маховая балка и решётка с парусиной."""
    ca, sa = math.cos(ang), math.sin(ang)

    def local(p):
        dv, dy = p[:, 2] - cv, p[:, 1] - cy
        return p[:, 0] - cu, dv * ca + dy * sa, -dv * sa + dy * ca

    def f(p):
        x, a, b = local(p)
        spar = sdbox(np.stack([np.abs(x) - 0.5, np.abs(a - length / 2) - length / 2, np.abs(b) - 0.45], 1))
        board = sdbox(np.stack([np.abs(x + 0.15) - 0.3, np.abs(a - (length + 4) / 2) - (length - 4) / 2,
                                np.abs(b - width / 2 - 0.3) - width / 2], 1))
        return np.minimum(spar, board)

    def mat(p, n):
        x, a, b = local(p)
        out = np.full(len(p), PLASTER)
        frame = (np.abs(b) < 0.6) | (np.abs(b - width - 0.3) < 0.5) | ((((a - 4) / 2.6) % 1.0) < 0.22) | (a > length - 0.6)
        out[frame] = WOOD
        return out, np.zeros(len(p), int)

    r = math.hypot(length, width + 1)
    return bc.Part(f, [cu - 1, cy - r, cv - r], [cu + 1, cy + r, cv + r], mat)


def build_wind():
    c = L / 2
    tower = deco(STONE, door((c, c, 180), 0, 5, 0.5, 10), windows((c, c, 180), [0], 2.4, 18, 24),
                 windows((c, c, 90), [0], 2.4, 13, 19, [LIT]), windows((c, c, 135), [0], 2.2, 26, 31),
                 masonry(seed=1301))
    hub_u, hub_y = c - 14.5, 40.0
    groups = [[lathe(c, c, [(14.5, 0), (13.5, 3), (11, 34), (11.6, 35)], tower),
               lathe(c, c, [(12.4, 34.5), (12, 37), (9, 44), (4, 49), (0.0, 50.5)], deco(TILE, tiles(2.0, 2.2, 1302))),
               box(hub_u, c - 9, hub_y - 1, hub_y + 1, c - 1, c + 1, WOOD, r=0.2),
               ball((hub_u - 0.6, hub_y, c), 1.7, WOOD)]]
    groups.append([sail(hub_u, hub_y, c, math.radians(a)) for a in (45, 135, 225, 315)])
    groups.append([ball((6, 1.8, 37), 2.1, deco(PLASTER, lighten(-1))), ball((8.5, 1.6, 40.5), 1.9, PLASTER),
                   ball((5.5, 1.5, 41.5), 1.7, deco(PLASTER, lighten(-1)))])
    return groups


# ---------------------------------------------------------------- золото: казна (2×1)
def build_treasury():
    groups = []
    u0, u1, v0, v1, eaves = 14.0, 40.0, 6.0, 86.0, 24.0
    um, vm = (u0 + u1) / 2, (v0 + v1) / 2
    wins = [16, 28, 64, 76]
    block = deco(STONE, door("-u", vm, 6, 2.4, 15, pointed=False),
                 windows("-u", wins, 2.8, 8, 16, pointed=False, sill=STONE), grille("-u", wins, 2.8, 8, 16),
                 windows("+v", [um], 2.8, 8, 16, pointed=False), grille("+v", [um], 2.8, 8, 16),
                 masonry(seed=1401), flagstones(1402))
    groups.append([
        box(u0, u1, 0, eaves, v0, v1, block),
        box(u0 - 1, u1 + 1, eaves, eaves + 2, v0 - 1, v1 + 1, deco(STONE, lighten(1))),
        prism([(u0 - 1.5, eaves + 2), (u1 + 1.5, eaves + 2), (um, eaves + 12)], "v", v0 - 1.5, v1 + 1.5,
              deco(SLATE, tiles(2.2, 2.6, 1403))),
        battlement(um - 0.5, um + 0.5, eaves + 11.5, eaves + 15, v0 + 2, v1 - 2, "v", 3.0, 1.4, GOLD),
        ball((um, eaves + 12.8, v0 - 1.2), 1.2, GOLD), ball((um, eaves + 12.8, v1 + 1.2), 1.2, GOLD),
    ])
    groups.append(portico(5, u0, vm, 22.5, [vm - 12, vm - 4, vm + 4, vm + 12], 15) + [
        prism([(vm - 16, 22.5), (vm + 16, 22.5), (vm, 30)], "u", 4, u0, deco(SLATE, tiles(2.0, 2.4, 1404))),
        prism([(vm - 14.5, 22.5), (vm + 14.5, 22.5), (vm, 29)], "u", 4.2, 4.8,
              deco(STONE, coin("-u", vm, 25.3, 2.3), lighten(1))),
    ])
    chests = []
    for cu, cv in ((6.5, 13), (6.5, 79)):
        chests += [box(cu - 2.6, cu + 2.6, 0, 3.6, cv - 3.6, cv + 3.6,
                       deco(WOOD, planks(1.2, 1405), bands([0.5, 3.0], 0.35, GOLD))),
                   box(cu - 2.8, cu + 2.8, 3.6, 4.6, cv - 3.8, cv + 3.8, deco(WOOD, top_face(GOLD)))]
    for cu, cv, h in ((3.5, 21, 1.8), (4.2, 71, 2.4), (8.8, 72.5, 1.2)):
        chests.append(lathe(cu, cv, [(1.4, 0), (1.4, h)], deco(GOLD, bands([h * 0.5], 0.15, GOLD))))
    groups.append(chests)
    return groups


# ---------------------------------------------------------------- тень: склеп (2×1)
def build_crypt():
    groups = []
    u0, u1, v0, v1, eaves, ridge = 11.0, 35.0, 10.0, 82.0, 24.0, 44.0
    um, vm = (u0 + u1) / 2, (v0 + v1) / 2
    hall = deco(DARK, windows("-u", [20, 32, 60, 72], 2.4, 7, 19, VIOLET),
                windows("-u", [vm], 6.6, 2.5, 17, VIOLET), windows("-u", [vm], 4.8, 2.5, 15.3, DOOR),
                windows("+v", [um], 3.0, 8, 20, VIOLET), masonry(seed=1501), flagstones(1502))
    groups.append([
        box(6, 40, 0, 2.5, 4, 88, deco(DARK, flagstones(1503), masonry(1.2, 4.0, seed=1504))),
        box(u0, u1, 2.5, eaves, v0, v1, hall),
        prism([(u0 - 2, eaves), (u1 + 2, eaves), (um, ridge)], "v", v0 - 1.5, v1 + 1.5,
              deco(SLATE, tiles(2.4, 2.6, 1505), lighten(-1))),
        frustum(um, vm, ridge - 3, ridge + 20, 3.2, 0.0, deco(DARK, masonry(2.0, 2.0, seed=1506)), n=4),
        ball((um, ridge + 20.8, vm), 1.3, VIOLET),
    ])
    for k, v in enumerate((26, 66)):
        groups.append([box(u0 - 3, u0, 2.5, eaves - 4, v - 1.4, v + 1.4, deco(DARK, masonry(2.2, 2.4, seed=1510 + k)))]
                      + pinnacle(u0 - 1.5, v, eaves - 4, 1.3, 5.5, mat=DARK, seed=1512 + k))
    for k, (cu, cv) in enumerate(((6.5, 6.5), (6.5, 85.5))):
        groups.append([frustum(cu, cv, 0, 22, 2.2, 1.3, deco(DARK, masonry(2.4, 3.0, seed=1520 + k)), n=4),
                       frustum(cu, cv, 22, 26, 1.5, 0.0, VIOLET, n=4)])
    groups.append([box(7.2, 8.6, 2.5, 6.5, v - 1.5, v + 1.5, deco(DARK, lighten(1)), r=0.5) for v in (39, 53)])
    return groups


# ---------------------------------------------------------------- молния: башня с громоотводом (1×1)
def build_spark():
    c = L / 2
    base = deco(STONE, door("-u", c, 5, 0.5, 9, pointed=False), windows("+v", [c], 2.4, 3, 8, ARC, pointed=False),
                masonry(seed=1601), flagstones(1602))
    shaft = deco(STONE, windows((c, c, 180), [0], 2.2, 22, 29, ARC), windows((c, c, 90), [0], 2.2, 30, 37, ARC),
                 windows((c, c, 135), [0], 2.2, 40, 45, ARC), bands([18, 33, 47], 0.8, COPPER), masonry(seed=1603))
    coil = [(3.0, 54.5)] + [(4.2 if k % 2 == 0 else 5.2, 55 + 1.1 * k) for k in range(9)] + [(3.0, 64.5)]
    g = [lathe(c, c, [(10.5, 11), (9, 15), (8, 50), (10.5, 53)], shaft),
         lathe(c, c, [(11.5, 52.5), (11.5, 54.5)], deco(COPPER, lighten(-1))),
         lathe(c, c, coil, COPPER, solid=False),
         lathe(c, c, [(2.6, 54), (2.6, 65)], COPPER),
         ball((c, 70, c), 5.5, COPPER),
         box(c - 0.4, c + 0.4, 74, 84, c - 0.4, c + 0.4, IRON, r=0.1),
         ball((c, 84.6, c), 1.2, ARC)]
    for du, dv in ((-8, -8), (-8, 8), (8, -8), (8, 8)):
        g += [box(c + du - 0.35, c + du + 0.35, 54.5, 61, c + dv - 0.35, c + dv + 0.35, IRON, r=0.1),
              ball((c + du, 61.5, c + dv), 0.8, ARC)]
    return [[box(7, 39, 0, 11, 7, 39, base)] + crenels(6.4, 39.6, 6.4, 39.6, 11, 3.0, 1.6, seed=1604), g]


# ---------------------------------------------------------------- варвары: длинный дом (2×1)
def build_longhouse():
    groups = []
    u0, u1, v0, v1 = 11.0, 35.0, 7.0, 85.0
    um, vm, apex = (u0 + u1) / 2, (v0 + v1) / 2, 36.0
    wood = deco(WOOD, planks(1.6, 1701))
    groups.append([
        box(9.5, 36.5, 0, 2, 5.5, 86.5, deco(STONE, flagstones(1702), masonry(1.0, 3.0, seed=1703))),
        box(u0, u1, 2, 11, v0, v1, wood),
        prism([(5.5, 7.5), (40.5, 7.5), (um, apex)], "v", 4.5, 87.5, deco(THATCH, tiles(1.6, 1.1, 1704))),
        box(um - 0.9, um + 0.9, apex - 1, apex + 1, 4.5, 87.5, WOOD, r=0.3),
        prism([(u0, 2), (u1, 2), (u1, 15), (um, 33.5), (u0, 15)], "v", v1 - 0.6, v1,
              deco(WOOD, planks(1.6, 1705), shields("+v", [16.5, 29.5], 8.5, 2.7, [(FLAG, HERALD), (HERALD, GOLD)]),
                   windows("+v", [um], 2.2, 23, 27, DOOR, pointed=False))),
    ])
    for vend in (4.8, 87.2):  # скрещённые доски-рога на коньке
        for ang in (55, 125):
            a = math.radians(ang)
            d = (math.cos(a), math.sin(a))
            groups.append([rbox((um + 2 * d[0], apex + 2 * d[1], vend), (0.5, 6.0, 0.6), "v", a, deco(WOOD, lighten(1))),
                           ball((um + 8 * d[0], apex + 8 * d[1], vend), 0.9, BONE)])
    groups.append([
        box(5, 11.5, 2, 12, vm - 5.5, vm + 5.5, deco(WOOD, planks(1.4, 1706), door("-u", vm, 4.4, 2, 10, pointed=False))),
        prism([(vm - 6.5, 11.5), (vm + 6.5, 11.5), (vm, 19)], "u", 4, 13, deco(THATCH, tiles(1.4, 1.1, 1707))),
    ] + skull((3.8, 14.8, vm), 1.4))
    groups.append([box(4, 6, 0, 15, 9, 11, deco(WOOD, planks(1.0, 1708)))] + skull((5, 16.6, 10), 1.5))
    return groups


# ---------------------------------------------------------------- маг: башня (1×1)
def build_wizard():
    c = L / 2 + 1.5
    tw = deco(STONE, door((c, c, 180), 0, 4.6, 0.5, 9.5),
              windows((c, c, 180), [0], 2.2, 20, 27, MAGIC), windows((c, c, 135), [0], 2.2, 31, 38, MAGIC),
              windows((c, c, 90), [0], 2.2, 14, 21, [LIT]), windows((c, c, 90), [0], 2.2, 49, 56, MAGIC),
              windows((c, c, 180), [0], 2.2, 50, 57, MAGIC), masonry(seed=1801))
    hat = deco(HAT, tiles(2.0, 2.2, 1802), bands([64.4], 0.9, GOLD), stars(c, c))
    groups = [[
        lathe(c, c, [(12.5, 0), (11, 5), (9, 12), (9, 58), (10.5, 61)], tw),
        lathe(c, c, [(8.8, 42.5), (12, 44), (12, 45.5), (8.8, 45.5)], deco(STONE, lighten(1)), solid=False),
        ring_battlement(c, c, 11.0, 12.0, 45.5, 48.5, 2.8, 1.6, deco(STONE, masonry(1.6, 2.0, seed=1803))),
        lathe(c, c, [(12.8, 60.5), (12.8, 61.8), (9.5, 63), (7.2, 70), (4.8, 79), (2.6, 88), (1.0, 95), (0.0, 98)], hat),
        ball((c, 98.6, c), 1.0, GOLD),
    ]]
    tu, tv = 10.0, 37.5  # башенка спереди
    groups.append([
        lathe(tu, tv, [(5.6, 0), (4.6, 4), (4.6, 28), (5.4, 30)],
              deco(STONE, windows((tu, tv, 180), [0], 1.8, 16, 21, MAGIC), windows((tu, tv, 90), [0], 1.8, 8, 13, [LIT]),
                   masonry(1.8, 2.4, seed=1804))),
        lathe(tu, tv, [(5.9, 29.5), (5.9, 30.6), (4.4, 31.4), (2.6, 38), (1.0, 44), (0.0, 46)],
              deco(HAT, tiles(1.8, 2.0, 1805), bands([32.2], 0.7, GOLD), stars(tu, tv, 2.0))),
        ball((tu, 46.5, tv), 0.8, GOLD),
    ])
    return groups



# ---------------------------------------------------------------- магазин (2×2, с 0.134)
AWNING, CANVAS = 34, 35
bc.PALETTE[AWNING] = ["#2a070c", "#5a1420", "#8a2430", "#b8403e", "#de6a5a"]  # красная полоса навеса
bc.PALETTE[CANVAS] = ["#3a3424", "#6c6248", "#a49a78", "#d4cba6", "#f2ead0"]  # светлая полоса навеса


def stripes(width=4.0):
    """Навес в полоску: красные и светлые полосы поперёк вертикалей (по v)."""
    def apply(p, n, out, off):
        sel = out == AWNING
        out[sel & (((p[:, 2] / width) % 2.0) < 1.0)] = CANVAS
    return apply


def build_shop():
    """Лавка мага: каменный низ с фиолетовыми витринами и дверью, фахверк второго этажа, черепичная кровля со
    щипцом, полосатый навес над витринами, вывеска с золотой монетой на кронштейне; за домом — круглая башенка
    с шиферным конусом и светящимся шаром; у входа бочки и сундук."""
    groups = []
    edge = 2 * L
    groups.append([box(2, edge - 2, 0, 0.8, 2, edge - 2, deco(STONE, flagstones(2101)))])  # мощёная площадка

    h0, h1, v0, v1 = 30.0, 82.0, 10.0, 82.0
    floor, eaves, ridge = 16.0, 31.0, 56.0
    vm = (v0 + v1) / 2
    hall = deco(STONE, timber(floor, eaves, seed=2102),
                door("-u", vm, 6.6, 0.8, 13.5),
                windows("-u", [20, 32, 60, 72], 4.4, 3.5, 12.5, MAGIC, pointed=False, sill=STONE),
                windows("-u", [22, 38, 54, 70], 2.6, 20, 27, [LIT, WINDOW, LIT, WINDOW], pointed=False),
                windows("+v", [44, 68], 3.4, 4, 12, MAGIC, pointed=False),
                windows("+v", [56], 2.4, 20, 27, [LIT], pointed=False),
                masonry(seed=2103))
    groups.append([
        box(h0, h1, 0.8, eaves, v0, v1, hall),
        # кровля щипцом к фасаду (к 1-й горизонтали)
        prism([(v0 - 2.6, eaves), (v1 + 2.6, eaves), (vm, ridge)], "u", h0 + 0.4, h1 + 2.2,
              deco(TILE, tiles(2.4, 2.8, 2104))),
        prism([(v0 - 2.6, eaves - 0.6), (v1 + 2.6, eaves - 0.6), (vm, ridge + 0.6)], "u", h0 - 1.8, h0 + 0.4,
              deco(TILE, tiles(2.4, 2.8, 2114))),
        prism([(v0 + 1.2, eaves), (v1 - 1.2, eaves), (vm, ridge - 2.6)], "u", h0 - 2.0, h0 - 1.4,
              deco(STONE, timber(eaves, ridge - 2.5, seed=2105),
                   windows("-u", [vm], 5.0, 34, 47, MAGIC))),
        box(h1 - 14, h1 - 9, 40, 60, v1 - 16, v1 - 11, deco(STONE, masonry(2.2, 2.4, seed=2106), lighten(-1))),
    ])
    # навес над витринами: наклонная полоса на столбах
    awning = deco(AWNING, stripes(4.0))
    groups.append([
        prism([(h0, 16.2), (h0, 15.0), (h0 - 10, 11.2), (h0 - 10, 12.4)], "v", v0 + 1, v1 - 14, awning),
        box(h0 - 10, h0 - 9, 0.8, 11.6, v0 + 1.5, v0 + 2.5, WOOD, r=0.15),
        box(h0 - 10, h0 - 9, 0.8, 11.6, v1 - 15.5, v1 - 14.5, WOOD, r=0.15),
    ])
    # вывеска с монетой на кронштейне у правого угла фасада, лицом к вертикали h
    sv = v1 - 6.0
    groups.append([
        box(h0 - 8, h0, 24.6, 25.4, sv - 0.4, sv + 0.4, IRON, r=0.1),
        box(h0 - 7.4, h0 - 1.0, 16.0, 23.6, sv - 0.6, sv + 0.6,
            deco(WOOD, planks(1.0, 2107), coin("+v", h0 - 4.2, 19.8, 2.2))),
        box(h0 - 6.6, h0 - 6.2, 23.6, 24.6, sv - 0.2, sv + 0.2, IRON, r=0.05),
        box(h0 - 2.2, h0 - 1.8, 23.6, 24.6, sv - 0.2, sv + 0.2, IRON, r=0.05),
    ])
    # башенка мага за домом, над кровлей
    tu, tv = h1 - 3.0, v0 + 6.0
    groups.append([
        lathe(tu, tv, [(7.0, 0), (6.0, 4), (6.0, 60), (7.0, 62)],
              deco(STONE, windows((tu, tv, 180), [0], 2.0, 50, 56, MAGIC),
                   windows((tu, tv, 90), [0], 2.0, 44, 50, MAGIC), masonry(1.8, 2.4, seed=2108))),
        lathe(tu, tv, [(7.4, 61.5), (7.4, 62.8), (5.0, 64), (2.6, 71), (0.8, 76), (0.0, 77)],
              deco(SLATE, tiles(1.8, 2.0, 2109))),
        ball((tu, 79.8, tv), 3.0, MAGIC),
    ])
    # у входа: бочки и сундук с золотым кантом
    props = barrel(8, 14, 2110) + barrel(11.5, 19, 2111) + barrel(7, 20.5, 2112)
    props += [box(5, 11, 0.8, 4.6, 70, 79, deco(WOOD, planks(1.2, 2113), bands([1.4, 4.0], 0.35, GOLD))),
              box(4.8, 11.2, 4.6, 5.6, 69.8, 79.2, deco(WOOD, top_face(GOLD)))]
    groups.append(props)
    return groups

# ---------------------------------------------------------------- зеркало: фасады −u и +v меняются местами
def mirror(groups, footprint):
    """Отражение через вертикаль экрана: (u, v) → (Wv − v, Wu − u). Видимые грани остаются видимыми."""
    wu = (max(i for i, _ in footprint) + 1) * L
    wv = (max(j for _, j in footprint) + 1) * L

    def wrap(part):
        f, m = part.f, part.mat

        def T(p):
            return np.stack([wu - p[:, 2], p[:, 1], wv - p[:, 0]], 1)

        def g(p):
            return f(T(p))

        mat = m
        if callable(m):
            def mat(p, n):
                return m(T(p), np.stack([-n[:, 2], n[:, 1], -n[:, 0]], 1))
        lo = [wv - part.hi[2], part.lo[1], wu - part.hi[0]]
        hi = [wv - part.lo[2], part.hi[1], wu - part.lo[0]]
        return bc.Part(g, lo, hi, mat)

    return [[wrap(part) for part in grp] for grp in groups]


def mirror_cells(footprint):
    a = max(i for i, _ in footprint) + 1
    b = max(j for _, j in footprint) + 1
    return [(b - 1 - j, a - 1 - i) for i, j in footprint]


BUILDINGS = {
    "main": build_main, "recruit": build_recruit, "king": build_king, "queen": build_queen,
    "rook": build_rook, "knight": build_knight, "bishop": build_bishop,
    "pawn": build_pawn, "forge": build_forge, "ice": build_ice, "wind": build_wind, "treasury": build_treasury,
    "crypt": build_crypt, "spark": build_spark, "longhouse": build_longhouse, "wizard": build_wizard,
    "shop": build_shop,
}
SQUARE = [(0, 0), (0, 1), (1, 0), (1, 1)]
ONE, TWO = [(0, 0)], [(0, 0), (0, 1)]  # 2×1 — вдоль вертикалей; вдоль горизонталей — зеркальное
FOOTPRINT = {  # клетки (i — к 8-й горизонтали, j — к вертикали h)
    "main": SQUARE, "recruit": SQUARE, "shop": SQUARE, "king": SQUARE, "queen": SQUARE, "rook": SQUARE,
    "knight": [(0, 0), (1, 0), (1, 1)], "bishop": [(0, 0), (1, 0), (2, 0)],
    "pawn": ONE, "wind": ONE, "spark": ONE, "wizard": ONE,
    "forge": TWO, "ice": TWO, "treasury": TWO, "crypt": TWO, "longhouse": TWO,
}
# раскладки превью: (здание, зеркально, вертикаль, горизонталь клетки (0, 0)), от a1 = (0, 0)
LAYOUT = [("recruit", False, 0, 0), ("main", False, 3, 0), ("rook", False, 6, 0), ("bishop", False, 2, 2),
          ("knight", False, 4, 3), ("king", False, 0, 5), ("queen", False, 5, 6)]
TOWN = [
    ("queen", True, 0, 6), ("bishop", True, 2, 7), ("king", False, 6, 6),
    ("ice", False, 0, 5), ("knight", False, 3, 4), ("wind", False, 0, 4), ("wizard", False, 1, 4),
    ("crypt", False, 5, 4), ("spark", False, 7, 4),
    ("rook", True, 0, 2), ("treasury", True, 7, 2), ("main", False, 3, 1),
    ("longhouse", True, 0, 0), ("pawn", False, 1, 0), ("recruit", False, 5, 0), ("forge", True, 7, 0),
]

FULL = TOWN + [  # все 64 клетки: свободные клетки TOWN заняты повторами тех же зданий
    ("wizard", False, 5, 7), ("crypt", False, 2, 6), ("treasury", False, 4, 6), ("forge", True, 2, 4),
    ("longhouse", False, 5, 5), ("wind", False, 7, 5), ("spark", False, 4, 4), ("ice", True, 2, 2),
    ("treasury", False, 3, 3), ("rook", False, 5, 2), ("forge", False, 1, 1), ("crypt", False, 2, 0),
    ("pawn", False, 4, 0),
]


def key(name, mirrored):
    return name + ("_m" if mirrored else "")


def cells_of(name, mirrored, fe, re):
    fp = mirror_cells(FOOTPRINT[name]) if mirrored else FOOTPRINT[name]
    return [(fe + j, re + i) for i, j in fp]  # (вертикаль, горизонталь)


def check(layout):
    seen = {}
    for name, m, fe, re in layout:
        for f, r in cells_of(name, m, fe, re):
            assert 0 <= f < 8 and 0 <= r < 8, (name, f, r)
            assert (f, r) not in seen, (name, seen.get((f, r)), f, r)
            seen[(f, r)] = name
    return seen


def draw_order(layout):
    """Порядок рисования: здание A раньше B, если A целиком дальше (выше по горизонталям при общих вертикалях или
    левее по вертикалям при общих горизонталях); среди свободных — по передней вершине."""
    info = []
    for item in layout:
        cs = cells_of(*item)
        fs, rs = [f for f, _ in cs], [r for _, r in cs]
        front = max(vertex(f + 1, r)[1] for f, r in cs)
        info.append((item, min(fs), max(fs), min(rs), max(rs), front))

    def behind(a, b):
        _, af0, af1, ar0, ar1, _ = a
        _, bf0, bf1, br0, br1, _ = b
        files = af0 <= bf1 and bf0 <= af1
        ranks = ar0 <= br1 and br0 <= ar1
        return (ar0 > br1 and (files or af1 < bf0)) or (af1 < bf0 and ranks)

    order, left = [], list(info)
    while left:
        free = [a for a in left if not any(behind(b, a) for b in left if b is not a)]
        pick = min(free or left, key=lambda a: a[5])
        order.append(pick[0])
        left.remove(pick)
    return order


# ---------------------------------------------------------------- превью
def vertex(fe, re):
    """Экранная точка вершины доски: fe — граница вертикалей (0 — край у a), re — граница горизонталей."""
    ox = bc.MARGINS["margin_x"] + 8 * art.TW // 2 - art.TW // 2
    oy = bc.MARGINS["margin_top"]
    return ox - 56 + 28 * (fe - 1) + 28 * (re - 4), oy + 105 + 21 * (fe - 1) - 21 * (re - 4)


def read_anchor(k, suffix):
    with open(os.path.join(OUT, f"anchor_{k}{suffix}.txt"), encoding="utf-8") as f:
        _, ax, ay = f.read().split()
    return int(ax), int(ay)


def preview(layout, suffix=""):
    check(layout)
    empty = Image.new("RGBA", (art.W, art.H), (0, 0, 0, 0))
    canvas = art.board_preview({c + k: (empty, empty) for c in "wb" for k in "PNBRQK"}, scale=1, demo=False,
                               **bc.MARGINS)
    placed = []
    for name, m, fe, re in draw_order(layout):
        k = key(name, m)
        x, y = vertex(fe, re)
        ax, ay = read_anchor(k, suffix)
        body = Image.open(os.path.join(OUT, f"{k}{suffix}.png")).convert("RGBA")
        shadow = Image.open(os.path.join(OUT, f"{k}_shadow{suffix}.png")).convert("RGBA")
        placed.append(((x - ax, y - ay), body, shadow))
    for at, _, shadow in placed:
        canvas.alpha_composite(shadow, at)
    for at, body, _ in placed:
        canvas.alpha_composite(body, at)
    return canvas.resize((canvas.width * 2, canvas.height * 2), Image.NEAREST)


def render_one(k, quick):
    name, m = (k[:-2], True) if k.endswith("_m") else (k, False)
    groups = BUILDINGS[name]()
    if m:
        groups = mirror(groups, FOOTPRINT[name])
    suffix = "_quick" if quick else ""
    body, shadow, anchor = bc.render(bc.Scene(groups), ss=1 if quick else 3)
    body.save(os.path.join(OUT, f"{k}{suffix}.png"))
    shadow.save(os.path.join(OUT, f"{k}_shadow{suffix}.png"))
    with open(os.path.join(OUT, f"anchor_{k}{suffix}.txt"), "w", encoding="utf-8") as f:
        f.write(f"{k} {anchor[0]} {anchor[1]}\n")
    print(k, "anchor", anchor, "size", body.size, flush=True)


def main():
    """`py tools/breach_city.py [имя | имя_m …] [--quick] [--preview]`: без имён — все здания обоих превью."""
    quick = "--quick" in sys.argv
    suffix = "_quick" if quick else ""
    wanted = sorted({key(n, m) for n, m, _, _ in LAYOUT + TOWN + FULL})
    names = [a for a in sys.argv[1:] if not a.startswith("--")] or wanted
    os.makedirs(OUT, exist_ok=True)
    if "--preview" not in sys.argv:
        for k in names:
            render_one(k, quick)
    for layout, out in ((LAYOUT, "preview_city"), (TOWN, "preview_town"), (FULL, "preview_full")):
        need = {key(n, m) for n, m, _, _ in layout}
        if all(os.path.exists(os.path.join(OUT, f"anchor_{k}{suffix}.txt")) for k in need):
            preview(layout, suffix).save(os.path.join(OUT, f"{out}{suffix}.png"))
            print(out, OUT)


if __name__ == "__main__":
    main()
