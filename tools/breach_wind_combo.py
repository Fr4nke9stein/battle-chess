"""Воздушные фигуры вместе с теневыми — в стиле Into the Breach, как `breach_fire_combo.py`.

Тёмная аура `breach_darkfire.py` (фигура чуть темнее, фиолетовый отсвет по краю) и вокруг низа фигуры —
вихрь `breach_wind.py`, у которого вторая спираль струй фиолетовая (`SHADE`: голова сиреневая, середина и
хвост фиолетовые, как блики и контур тёмной ауры; у земли — струи `k` = 0 и π/2, по клетке — половина
колец). Сборка — `wind.render(code, bodies=кадры dark.render, hot=SHADE)`: задние струи, тёмная аура
поверх них по альфе, фигура, дымка вихря, передние струи. 40 кадров, разметка как у `dark_fire`/`wind`
(вступление [0, 8), повтор [8, 40)). Теневой шаг — `breach_shadowstep.py` с этими кадрами вместо ауры
(`aura`): вихрь втягивается вместе с аурой и при появлении набирается заново.

Воздух с остальными уже есть: на ледяных, золотых, варварах и магах — `effects/<ice|gold|barbarian|
wizard>/wind_*`, на огненной — `effects/fire/wind_*` (оранжевая спираль), на электрической —
`effects/spark/wind_*`; в превью они стоят рядами.

`py tools/breach_wind_combo.py` — по умолчанию в assets/breach:
effects/wind/<dark_fire|shadowstep_out|shadowstep_in>_<код>.png; превью pieces/preview_wind_combo.png (ряды:
воздушные, воздушные ледяные, золотые, варвары, маги, огненные, электрические, теневые) и
effects/preview_wind_combo.gif.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_barbarian_combo as bc  # noqa: E402
import breach_darkfire as dark  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "wind")
CODES = bc.CODES
SHADE = [hexc(c) for c in ["#e2c8ff", "#b583ff", "#8644f2", "#5a2aa8", "#1e0f33"]]  # фиолетовые струи: как `wind.AIR`


def strip_frames(path, size=120):
    strip = np.array(Image.open(path).convert("RGBA"))
    return [strip[:, i * size : (i + 1) * size] for i in range(strip.shape[1] // size)]


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    shade = {}
    for code in CODES:
        k = code[1]
        aura = dark.render(code, seed=5 + "PNBRQK".index(k))
        shade[code] = wind.render(code, bodies=lambda f: aura[f], hot=SHADE)
        out_f, in_f, _ = step.render(code, seed=31 + "PNBRQK".index(k), aura=shade[code])
        for name, frames in (("dark_fire", shade[code]), ("shadowstep_out", out_f), ("shadowstep_in", in_f)):
            Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(EFFECTS, f"{name}_{code}.png"))
        print("effects", code, flush=True)

    effects = os.path.join(ROOT, "effects")
    load = lambda folder: {c: strip_frames(os.path.join(effects, folder, f"wind_{c}.png")) for c in CODES}  # noqa: E731
    plain, fire, spark = load(""), load("fire"), load("spark")
    anchor = (wind.FAX, wind.FAY)
    pick = lambda strips, i: {c: (Image.fromarray(strips[c][i], "RGBA"), anchor) for c in CODES}  # noqa: E731
    rows = [pick(plain, 20)] + [pick(load(folder), 20) for folder in ("ice", "gold", "barbarian", "wizard")]
    rows += [pick(fire, 20), pick(spark, 20), pick(shade, 20)]
    bc.sheet(rows, os.path.join(PIECES, "preview_wind_combo.png"))
    loop = (wind.LOOP_START, wind.FRAMES)
    bc.anim_gif([(plain, anchor, loop), (fire, anchor, loop), (spark, anchor, loop), (shade, anchor, loop)],
                os.path.join(effects, "preview_wind_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
