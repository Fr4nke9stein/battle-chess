"""Золотые фигуры вместе с остальными особыми силами — в стиле Into the Breach.

Основа — золотые фигуры из `breach_gilded.py` (вариант `gold`: белые — жёлтое золото, белая эмаль,
сапфиры; чёрные — красное золото, чёрная эмаль, рубины; жемчуг и камни на поясах и короне — те же, что
в игре, `pieces/gold/`). С ними:

- анимации — те же генераторы, что у обычных особых фигур, на золотом спрайте (подмена источника:
  путь `pieces/<код>.png` ведёт в `pieces/gold/`, как у `breach_barbarian_combo.py`): огненный —
  лавовые трещины, электрический — аура с дугами без концовки (`electric_hold`), воздушный — вихрь,
  теневой — тёмная аура и теневой шаг. Разметка кадров и опоры — как у обычных полос;
- спрайты — объёмные заново: ледяной (`ice`: золото с наледью и инеем по `breach_ice_pieces.iced`,
  сосульки из-под поясов, кристаллы у основания; материалы льда — 11 и 12, у золотых 5–7 заняты
  узором, камнями и жемчугом) и маг (`wizard`: фигура-маг из `breach_wizard.py` с золотым корпусом и
  эмалью; шляпа — в цвете стороны, лента и звёзды — золото). Золотой варвар уже есть —
  `pieces/barbarian_gold/` из `breach_barbarian_combo.py`, в превью он стоит третьим рядом.

На самом светлом золоте спрайтов — белый блеск, как у золотых фигур.

`py tools/breach_gold_combo.py` — по умолчанию в assets/breach:
effects/gold/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png,
pieces/gold_<ice|wizard>/<код>.png и _shadow.png; превью pieces/preview_gold_combo.png (ряды: золотые,
ледяные, варвары, маги и по кадру огненного, электрического, воздушного, теневого) и
effects/preview_gold_combo.gif.
"""
import math
import os
import re
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_arcs as arcs  # noqa: E402
import breach_barbarian_combo as bc  # noqa: E402
import breach_darkfire as dark  # noqa: E402
import breach_gilded as gilded  # noqa: E402
import breach_ice_pieces as ip  # noqa: E402
import breach_lava as lava  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402
import breach_wizard as wz  # noqa: E402
from breach_art import MAIN, R, T, TRIM, Y, norm  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "gold")
CODES = bc.CODES
ICE_M, RIME_M = 11, 12

_open = Image.open


def from_gold(fp, *args, **kwargs):
    """Image.open, у которого pieces/<код>.png — золотой спрайт (генераторы эффектов берут обычный)."""
    if isinstance(fp, str):
        folder, name = os.path.split(os.path.abspath(fp))
        if os.path.normcase(folder) == os.path.normcase(os.path.abspath(PIECES)) and re.fullmatch(r"[wb][PNBRQK]\.png", name):
            fp = os.path.join(PIECES, "gold", name)
    return _open(fp, *args, **kwargs)


def iced_gold(kind, color):
    """Золотая фигура в наледи: части с украшениями — через `ip.iced`, сосульки и кристаллы — как у ледяных."""
    piece = gilded.adorned(kind)
    seed = 11 + "PNBRQK".index(kind) * 5 + (0 if color == "w" else 2)
    rng = np.random.default_rng(seed)
    parts = [ip.iced(p, seed + k, 1.8) for k, p in enumerate(piece.parts)]
    extra = []
    for y, r in ip.ICICLES[kind]:
        for az in (ip.FRONT - 50, ip.FRONT - 18, ip.FRONT + 14, ip.FRONT + 46, ip.FRONT + 80):
            if rng.random() < 0.8:
                extra += ip.icicle(ip.at(r, y, az + rng.uniform(-6, 6)), rng.uniform(2.5, 4.8))
    if kind == "N":  # сосульки из-под челюсти коня
        a = math.radians(18)
        fwd = norm(-R * math.cos(a) + T * math.sin(a))
        side = norm(np.cross(Y, fwd))
        if side @ art.D < 0:
            side = -side
        for u, ln in ((9.4, 3.6), (11.8, 2.8)):
            extra += ip.icicle(fwd * u + Y * 23.4 + side * 2.6, ln, 0.85)
    r0 = ip.BASE_R[kind]
    for az, ln in ((ip.FRONT - 55, 4.5), (ip.FRONT + 8, 5.5), (ip.FRONT + 62, 4.0)):
        extra += ip.shard(ip.at(r0 - 1.0, 1.5, az), ip.at(1, 0, az), ln, 1.1)
    return art.Piece(parts + extra, stretch=piece.stretch)


def shine(body, ramp, count=2):
    """Белый блеск на самом светлом золоте (открытом сверху-слева), как у золотых фигур."""
    img = np.array(body)
    top = np.array(art.hexc(ramp[4]))
    bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:count]:
        img[y, x, :3] = (255, 255, 255)
    return Image.fromarray(img, "RGBA")


def render_static(variant, kind, color):
    gold = gilded.PALETTES["gold"][color]
    key = f"gold_{variant}_{color}"
    if variant == "ice":
        pal = dict(gold)
        pal[ICE_M] = ip.ICE_RAMP
        pal[RIME_M] = ip.RIME_RAMP
        piece = iced_gold(kind, color)
    else:
        pal = dict(art.PALETTES[color])
        pal.update({MAIN: gold[MAIN], TRIM: gold[TRIM], wz.HAT: wz.HAT_COLORS[color], wz.GOLD: wz.GOLD_RAMP,
                    wz.STAR: wz.STAR_RAMP})
        piece = wz.wizard(kind)
    art.PALETTES[key] = pal
    body, shadow = art.render_piece(piece, key, 0.86)
    return shine(body, gold[MAIN]), shadow


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    ip.ICE, ip.RIME = ICE_M, RIME_M  # `iced`/`icicle`/`shard` читают их при сборке и отрисовке
    load = lambda folder: {c: (Image.open(os.path.join(PIECES, folder, f"{c}.png")).convert("RGBA"),  # noqa: E731
                               (art.AX, art.AY)) for c in CODES}
    rows = [load("gold")]
    statics = {}
    for variant in ("ice", "wizard"):
        folder = os.path.join(PIECES, f"gold_{variant}")
        os.makedirs(folder, exist_ok=True)
        row = {}
        for code in CODES:
            body, shadow = render_static(variant, code[1], code[0])
            body.save(os.path.join(folder, f"{code}.png"))
            shadow.save(os.path.join(folder, f"{code}_shadow.png"))
            row[code] = (body, (art.AX, art.AY))
            print(variant, code, flush=True)
        statics[variant] = row
    rows += [statics["ice"], load("barbarian_gold"), statics["wizard"]]

    Image.open = from_gold  # дальше генераторы эффектов видят золотой спрайт
    try:
        fire, spark, air, shade = {}, {}, {}, {}
        for code in CODES:
            k = code[1]
            fire[code] = lava.render(code, seed=40 + "PNBRQK".index(k))
            spark[code] = arcs.render(code, arcs.SEEDS[k])[: arcs.LOOP_END]
            air[code] = wind.render(code)
            out_f, in_f, _ = step.render(code, seed=31 + "PNBRQK".index(k))
            shade[code] = dark.render(code, seed=5 + "PNBRQK".index(k))
            for name, frames in (("lava_crack", fire[code]), ("electric_hold", spark[code]), ("wind", air[code]),
                                 ("dark_fire", shade[code]), ("shadowstep_out", out_f), ("shadowstep_in", in_f)):
                Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(EFFECTS, f"{name}_{code}.png"))
            print("effects", code, flush=True)
    finally:
        Image.open = _open

    pick = lambda strips, i, anchor: {c: (Image.fromarray(strips[c][i], "RGBA"), anchor) for c in CODES}  # noqa: E731
    rows.append(pick(fire, 20, (art.AX, art.AY)))
    rows.append(pick(spark, 14, (arcs.FAX, arcs.FAY)))
    rows.append(pick(air, 20, (wind.FAX, wind.FAY)))
    rows.append(pick(shade, 20, (dark.FAX, dark.FAY)))
    bc.sheet(rows, os.path.join(PIECES, "preview_gold_combo.png"))
    bc.anim_gif([(fire, (art.AX, art.AY), (0, lava.FRAMES)), (spark, (arcs.FAX, arcs.FAY), (arcs.LOOP_START, arcs.LOOP_END)),
                 (air, (wind.FAX, wind.FAY), (wind.LOOP_START, wind.FRAMES)),
                 (shade, (dark.FAX, dark.FAY), (dark.LOOP_START, dark.FRAMES))],
                os.path.join(ROOT, "effects", "preview_gold_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
