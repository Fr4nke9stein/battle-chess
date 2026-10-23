"""Электрические фигуры вместе с остальными особыми силами — в стиле Into the Breach, как `breach_fire_combo.py`.

Основа — электрическая фигура игры: аура с дугами без концовки (`electric_hold` из `breach_arcs.py`). С ней:

- воздушная — вихрь `breach_wind.py` вокруг обычной фигуры, сверху электричество: золотая аура позади,
  вихрь (`bodies` у `arcs.render`: что выходит за силуэт, кладётся поверх ауры и задних молний), передние
  молнии поверх вихря. 40 кадров, разметка как у `wind`;
- теневая — золотой ауры нет, вместо неё тёмная фиолетовая аура `breach_darkfire.py` (`under` у
  `arcs.render`; контур фигуры подсвечивает она, а не золото), голубые молнии те же. 40 кадров, разметка
  как у `dark_fire`; теневой шаг (`breach_shadowstep.py`, `aura` — эти кадры): молнии гаснут вместе с
  аурой при втягивании, при появлении — вступление электричества со вспышкой.

Электричество в обоих — `arcs.render(..., length=40)`: повтор идёт и после `LOOP_END`, без концовки (первые
24 кадра — как у `electric_hold`). Электричество с остальными уже есть: на ледяных, золотых, варварах и
магах — `effects/<ice|gold|barbarian|wizard>/electric_hold_*`, на огненной — `effects/fire/electric_hold_*`
(`breach_fire_combo.py`); в превью они стоят рядами.

`py tools/breach_spark_combo.py` — по умолчанию в assets/breach:
effects/spark/<wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png; превью pieces/preview_spark_combo.png
(ряды: электрические, электрические ледяные, золотые, варвары, маги, огненные, воздушные, теневые) и
effects/preview_spark_combo.gif.
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
import breach_darkfire as dark  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "spark")
CODES = bc.CODES


def silhouette(code):
    """Силуэт обычной фигуры на холсте сил 120×120 (опора (60, 92))."""
    sprite = np.array(Image.open(os.path.join(PIECES, f"{code}.png")).convert("RGBA"))
    sil = np.zeros((arcs.FH, arcs.FW), bool)
    sx, sy = arcs.FAX - art.AX, arcs.FAY - art.AY
    sil[sy : sy + sprite.shape[0], sx : sx + sprite.shape[1]] = sprite[..., 3] > 0
    return sil


def strip_frames(path, size=120):
    strip = np.array(Image.open(path).convert("RGBA"))
    return [strip[:, i * size : (i + 1) * size] for i in range(strip.shape[1] // size)]


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    air, shade = {}, {}
    for code in CODES:
        k = code[1]
        seed = arcs.SEEDS[k]
        vortex = wind.render(code)
        air[code] = arcs.render(code, seed, bodies=lambda f: vortex[f], length=wind.FRAMES)
        sil = silhouette(code)[..., None]
        aura = dark.render(code, seed=5 + "PNBRQK".index(k))
        shade[code] = arcs.render(code, seed, length=dark.FRAMES,
                                  under=lambda f: np.where(sil, 0, aura[f]).astype(np.uint8),
                                  bodies=lambda f: np.where(sil, aura[f], 0).astype(np.uint8))
        out_f, in_f, _ = step.render(code, seed=31 + "PNBRQK".index(k), aura=shade[code])
        for name, frames in (("wind", air[code]), ("dark_fire", shade[code]),
                             ("shadowstep_out", out_f), ("shadowstep_in", in_f)):
            Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(EFFECTS, f"{name}_{code}.png"))
        print("effects", code, flush=True)

    effects = os.path.join(ROOT, "effects")
    plain = {c: strip_frames(os.path.join(effects, f"electric_hold_{c}.png")) for c in CODES}
    fire = {c: strip_frames(os.path.join(effects, "fire", f"electric_hold_{c}.png")) for c in CODES}
    anchor = (arcs.FAX, arcs.FAY)
    pick = lambda strips, i: {c: (Image.fromarray(strips[c][i], "RGBA"), anchor) for c in CODES}  # noqa: E731
    rows = [pick(plain, 14)]
    for folder in ("ice", "gold", "barbarian", "wizard"):
        rows.append(pick({c: strip_frames(os.path.join(effects, folder, f"electric_hold_{c}.png")) for c in CODES}, 14))
    rows += [pick(fire, 14), pick(air, 20), pick(shade, 20)]
    bc.sheet(rows, os.path.join(PIECES, "preview_spark_combo.png"))
    hold = (arcs.LOOP_START, arcs.LOOP_END)
    bc.anim_gif([(plain, anchor, hold), (fire, anchor, hold), (air, anchor, (wind.LOOP_START, wind.FRAMES)),
                 (shade, anchor, (dark.LOOP_START, dark.FRAMES))],
                os.path.join(effects, "preview_spark_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
