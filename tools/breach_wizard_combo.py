"""Маги вместе с остальными особыми силами — в стиле Into the Breach, как `breach_barbarian_combo.py`.

Основа — фигуры-маги из `breach_wizard.py` (остроконечная шляпа: у белых синяя, у чёрных багровая;
`pieces/wizard/`). С ними:

- анимации — те же генераторы, что у обычных особых фигур, на спрайте мага (подмена источника:
  путь `pieces/<код>.png` ведёт в `pieces/wizard/`): огненный — лавовые трещины, электрический — аура
  с дугами без концовки (`electric_hold`), воздушный — вихрь, теневой — тёмная аура и теневой шаг.
  Разметка кадров и опоры — как у обычных полос;
- спрайт — ледяной маг, объёмный заново (`ice`: корпус иссиня-голубой с наледью и инеем по
  `breach_ice_pieces.iced`, сосульки свисают с полей шляпы и из-под поясов, кристаллы у основания;
  шляпа — в цвете стороны; материалы льда — 11 и 12, у мага 5–7 заняты шляпой, лентой и звёздами).
  Золотой маг и маг с рогами уже есть — `pieces/gold_wizard/` (`breach_gold_combo.py`) и
  `pieces/wizard_horns/` (`breach_wizard_horns.py`), в превью они стоят рядами.

`py tools/breach_wizard_combo.py` — по умолчанию в assets/breach:
effects/wizard/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png,
pieces/wizard_ice/<код>.png и _shadow.png; превью pieces/preview_wizard_combo.png (ряды: маги, ледяные,
золотые, с рогами и по кадру огненного, электрического, воздушного, теневого) и
effects/preview_wizard_combo.gif.
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
import breach_ice_pieces as ip  # noqa: E402
import breach_lava as lava  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402
import breach_wizard as wz  # noqa: E402
from breach_art import MAIN, R, T, TRIM, Y, norm  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "wizard")
CODES = bc.CODES
ICE_M, RIME_M = 11, 12

_open = Image.open


def from_wizard(fp, *args, **kwargs):
    """Image.open, у которого pieces/<код>.png — спрайт мага (генераторы эффектов берут обычный)."""
    if isinstance(fp, str):
        folder, name = os.path.split(os.path.abspath(fp))
        if os.path.normcase(folder) == os.path.normcase(os.path.abspath(PIECES)) and re.fullmatch(r"[wb][PNBRQK]\.png", name):
            fp = os.path.join(PIECES, "wizard", name)
    return _open(fp, *args, **kwargs)


def iced_wizard(kind, color):
    """Маг в наледи: части — через `ip.iced`, сосульки с полей шляпы и из-под поясов, кристаллы у основания."""
    piece = wz.wizard(kind)
    seed = 11 + "PNBRQK".index(kind) * 5 + (0 if color == "w" else 2)
    rng = np.random.default_rng(seed)
    parts = [ip.iced(p, seed + k, 1.8) for k, p in enumerate(piece.parts)]
    extra = []
    a = math.radians(18)
    fwd = norm(-R * math.cos(a) + T * math.sin(a))
    if kind == "N":
        brim = (fwd * 1.2, 46.0 - 0.6, 9.5 - 0.7)  # поля шляпы коня: центр по горизонтали, высота, радиус
    else:
        _, brim_y, brim_r, _, _ = wz.HATS[kind]
        brim = (np.zeros(3), brim_y + 0.0, brim_r - 0.7)
    centre, by, br = brim
    for az in (ip.FRONT - 60, ip.FRONT - 30, ip.FRONT, ip.FRONT + 30, ip.FRONT + 60, ip.FRONT + 90):  # с полей шляпы
        if rng.random() < 0.85:
            extra += ip.icicle(centre + np.asarray(ip.at(br, by, az + rng.uniform(-8, 8))), rng.uniform(2.6, 4.6))
    for y, r in ip.ICICLES[kind]:  # из-под поясов
        if kind in ("Q", "K") and y > by - 2:
            continue  # у ферзя и короля верхний пояс — под шляпой
        for az in (ip.FRONT - 50, ip.FRONT - 18, ip.FRONT + 14, ip.FRONT + 46, ip.FRONT + 80):
            if rng.random() < 0.7:
                extra += ip.icicle(ip.at(r, y, az + rng.uniform(-6, 6)), rng.uniform(2.5, 4.4))
    r0 = ip.BASE_R[kind]
    for az, ln in ((ip.FRONT - 55, 4.5), (ip.FRONT + 8, 5.5), (ip.FRONT + 62, 4.0)):
        extra += ip.shard(ip.at(r0 - 1.0, 1.5, az), ip.at(1, 0, az), ln, 1.1)
    return art.Piece(parts + extra, stretch=piece.stretch)


def render_ice(kind, color):
    key = f"wizard_ice_{color}"
    pal = dict(art.PALETTES[color])
    pal.update({MAIN: ip.PALETTES[color][MAIN], TRIM: ip.PALETTES[color][TRIM], wz.HAT: wz.HAT_COLORS[color],
                wz.GOLD: wz.GOLD_RAMP, wz.STAR: wz.STAR_RAMP, ICE_M: ip.ICE_RAMP, RIME_M: ip.RIME_RAMP})
    art.PALETTES[key] = pal
    body, shadow = art.render_piece(iced_wizard(kind, color), key, 0.86)
    img = np.array(body)  # искры на самом светлом льду, как у ледяных фигур
    top = np.array(art.hexc(ip.ICE_RAMP[3]))
    bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
    ys, xs = np.nonzero(bright)
    for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:3]:
        img[y, x, :3] = (255, 255, 255)
    return Image.fromarray(img, "RGBA"), shadow


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    ip.ICE, ip.RIME = ICE_M, RIME_M  # `iced`/`icicle`/`shard` читают их при сборке и отрисовке
    load = lambda folder: {c: (Image.open(os.path.join(PIECES, folder, f"{c}.png")).convert("RGBA"),  # noqa: E731
                               (art.AX, art.AY)) for c in CODES}
    folder = os.path.join(PIECES, "wizard_ice")
    os.makedirs(folder, exist_ok=True)
    ice_row = {}
    for code in CODES:
        body, shadow = render_ice(code[1], code[0])
        body.save(os.path.join(folder, f"{code}.png"))
        shadow.save(os.path.join(folder, f"{code}_shadow.png"))
        ice_row[code] = (body, (art.AX, art.AY))
        print("ice", code, flush=True)
    rows = [load("wizard"), ice_row, load("gold_wizard"), load("wizard_horns")]

    Image.open = from_wizard  # дальше генераторы эффектов видят спрайт мага
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
    bc.sheet(rows, os.path.join(PIECES, "preview_wizard_combo.png"))
    bc.anim_gif([(fire, (art.AX, art.AY), (0, lava.FRAMES)), (spark, (arcs.FAX, arcs.FAY), (arcs.LOOP_START, arcs.LOOP_END)),
                 (air, (wind.FAX, wind.FAY), (wind.LOOP_START, wind.FRAMES)),
                 (shade, (dark.FAX, dark.FAY), (dark.LOOP_START, dark.FRAMES))],
                os.path.join(ROOT, "effects", "preview_wizard_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
