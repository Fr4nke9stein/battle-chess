"""Огненные фигуры вместе с остальными особыми силами — в стиле Into the Breach, как `breach_ice_combo.py`.

Основа — огненная фигура игры: обычный спрайт с лавовыми трещинами из `breach_lava.py` в состоянии
ожидания взрыва (кадры `HOLD` = 13–16 полосы `lava_crack`, в игре — `LAVA_HOLD`). Анимации других сил —
те же генераторы, но тело фигуры в каждом кадре — кадр трещин (`bodies` у `render`: аура строится по
обычному силуэту, тело — кадр `HOLD[0] + f % 4`; повторы сил кратны 4 — шов не виден; искры лавы за
силуэтом тоже рисуются): электрический — аура с дугами без концовки (`electric_hold`), воздушный — вихрь,
у которого вторая спираль струй огненная (`HOT`: голова бледно-жёлтая, середина янтарная и оранжевая,
хвост тёмно-красный; у земли — струи `k` = 0 и π/2, по клетке — половина колец), теневой — тёмная аура
и теневой шаг. Разметка кадров и опоры — как у обычных полос (`electric_hold`, `wind`, `dark_fire`,
`shadowstep_out`/`_in`); набухание и взрыв — обычная полоса `lava_crack`.

Огонь с остальными уже есть — полосы `lava_crack` на спрайтах ледяных, золотых, варваров и магов
(`effects/<ice|gold|barbarian|wizard>/`); в превью они стоят рядами.

`py tools/breach_fire_combo.py` — по умолчанию в assets/breach:
effects/fire/<electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png; превью
pieces/preview_fire_combo.png (ряды: огненные, огненные ледяные, золотые, варвары, маги и по кадру
электрического, воздушного, теневого) и effects/preview_fire_combo.gif.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_arcs as arcs  # noqa: E402
import breach_barbarian_combo as bc  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_darkfire as dark  # noqa: E402
import breach_lava as lava  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "fire")
CODES = bc.CODES
HOLD = (13, 17)  # кадры трещин, пока огненная фигура ждёт взрыва (`LAVA_HOLD` в игре)
FIRE = burn.FIRE
HOT = [hexc(c) for c in ["#ffe08a", "#ff9a2e", "#f0641e", "#a8260c", "#3d0a04"]]  # огненные струи вихря: как `wind.AIR`


def lava_bodies(code):
    """Кадры трещин ожидания на холсте 120×120 сил (опора (60, 92)); тело кадра f — `HOLD[0] + f % 4`."""
    strip = lava.render(code, seed=40 + "PNBRQK".index(code[1]))
    sx, sy = arcs.FAX - art.AX, arcs.FAY - art.AY
    hold = []
    for i in range(*HOLD):
        c = np.zeros((arcs.FH, arcs.FW, 4), np.uint8)
        c[sy : sy + art.H, sx : sx + art.W] = strip[i]
        hold.append(c)
    return strip, lambda f: hold[f % len(hold)]


def lava_frame(folder, code, i):
    """Кадр i готовой полосы `lava_crack` (огонь на другой основе)."""
    strip = Image.open(os.path.join(ROOT, "effects", folder, f"lava_crack_{code}.png")).convert("RGBA")
    return strip.crop((i * art.W, 0, (i + 1) * art.W, art.H))


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    fire, spark, air, shade = {}, {}, {}, {}
    for code in CODES:
        k = code[1]
        strip, bodies = lava_bodies(code)
        fire[code] = strip[HOLD[0] : HOLD[1]]
        spark[code] = arcs.render(code, arcs.SEEDS[k], bodies=bodies)[: arcs.LOOP_END]
        air[code] = wind.render(code, bodies=bodies, hot=HOT)
        out_f, in_f, _ = step.render(code, seed=31 + "PNBRQK".index(k), bodies=bodies)
        shade[code] = dark.render(code, seed=5 + "PNBRQK".index(k), bodies=bodies)
        for name, frames in (("electric_hold", spark[code]), ("wind", air[code]), ("dark_fire", shade[code]),
                             ("shadowstep_out", out_f), ("shadowstep_in", in_f)):
            Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(EFFECTS, f"{name}_{code}.png"))
        print("effects", code, flush=True)

    rows = [{c: (Image.fromarray(fire[c][0], "RGBA"), (art.AX, art.AY)) for c in CODES}]
    for folder in ("ice", "gold", "barbarian", "wizard"):
        rows.append({c: (lava_frame(folder, c, HOLD[0]), (art.AX, art.AY)) for c in CODES})
    pick = lambda strips, i, anchor: {c: (Image.fromarray(strips[c][i], "RGBA"), anchor) for c in CODES}  # noqa: E731
    rows.append(pick(spark, 14, (arcs.FAX, arcs.FAY)))
    rows.append(pick(air, 20, (wind.FAX, wind.FAY)))
    rows.append(pick(shade, 20, (dark.FAX, dark.FAY)))
    bc.sheet(rows, os.path.join(PIECES, "preview_fire_combo.png"))
    bc.anim_gif([(fire, (art.AX, art.AY), (0, len(fire[CODES[0]]))),
                 (spark, (arcs.FAX, arcs.FAY), (arcs.LOOP_START, arcs.LOOP_END)),
                 (air, (wind.FAX, wind.FAY), (wind.LOOP_START, wind.FRAMES)),
                 (shade, (dark.FAX, dark.FAY), (dark.LOOP_START, dark.FRAMES))],
                os.path.join(ROOT, "effects", "preview_fire_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
