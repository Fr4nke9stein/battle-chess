"""Ледяные фигуры вместе с остальными особыми силами — в стиле Into the Breach, как `breach_barbarian_combo.py`.

Основа — ледяные фигуры игры из `breach_ice_pieces.py` (иссиня-голубые, наледь, иней, сосульки,
`pieces/ice/`). Анимации — те же генераторы, что у обычных особых фигур, на ледяном спрайте (подмена
источника: путь `pieces/<код>.png` ведёт в `pieces/ice/`): огненный — лавовые трещины, электрический —
аура с дугами без концовки (`electric_hold`), воздушный — вихрь, теневой — тёмная аура и теневой шаг.
Разметка кадров и опоры — как у обычных полос.

Ледяные спрайты с другими силами уже есть: золотые — `pieces/gold_ice/` (`breach_gold_combo.py`),
варвары — `pieces/barbarian_ice/` (`breach_barbarian_combo.py`), маги — `pieces/wizard_ice/`
(`breach_wizard_combo.py`); в превью они стоят рядами.

`py tools/breach_ice_combo.py` — по умолчанию в assets/breach:
effects/ice/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png; превью
pieces/preview_ice_combo.png (ряды: ледяные, золотые, варвары, маги и по кадру огненного, электрического,
воздушного, теневого) и effects/preview_ice_combo.gif.
"""
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
import breach_lava as lava  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "ice")
CODES = bc.CODES

_open = Image.open


def from_ice(fp, *args, **kwargs):
    """Image.open, у которого pieces/<код>.png — ледяной спрайт (генераторы эффектов берут обычный)."""
    if isinstance(fp, str):
        folder, name = os.path.split(os.path.abspath(fp))
        if os.path.normcase(folder) == os.path.normcase(os.path.abspath(PIECES)) and re.fullmatch(r"[wb][PNBRQK]\.png", name):
            fp = os.path.join(PIECES, "ice", name)
    return _open(fp, *args, **kwargs)


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    load = lambda folder: {c: (Image.open(os.path.join(PIECES, folder, f"{c}.png")).convert("RGBA"),  # noqa: E731
                               (art.AX, art.AY)) for c in CODES}
    rows = [load("ice"), load("gold_ice"), load("barbarian_ice"), load("wizard_ice")]

    Image.open = from_ice  # дальше генераторы эффектов видят ледяной спрайт
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
    bc.sheet(rows, os.path.join(PIECES, "preview_ice_combo.png"))
    bc.anim_gif([(fire, (art.AX, art.AY), (0, lava.FRAMES)), (spark, (arcs.FAX, arcs.FAY), (arcs.LOOP_START, arcs.LOOP_END)),
                 (air, (wind.FAX, wind.FAY), (wind.LOOP_START, wind.FRAMES)),
                 (shade, (dark.FAX, dark.FAY), (dark.LOOP_START, dark.FRAMES))],
                os.path.join(ROOT, "effects", "preview_ice_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
