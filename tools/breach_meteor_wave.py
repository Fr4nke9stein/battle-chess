"""Волна урона метеорита крестом (герб шляпы и плюса) — огненная волна по образцу
`breach_firewave_plus.py`, только узким крестом: клетка удара и по одной клетке с каждой стороны
(без диагоналей). Кромка огня расходится от клетки удара по четырём рукам креста, за ней —
подпалины, догорающие огоньки, искры и дым.

Кадр FW×FH (как у обычной огненной волны), центр верхней грани клетки удара — (CX, CY), FRAMES
кадров по FRAME_MS; слои `under` (до фигур) и `over` (после).

`py tools/breach_meteor_wave.py [папка]` — по умолчанию assets/breach/effects:
meteor_wave_under.png, meteor_wave_over.png (кадры в ряд), preview_meteor_wave.gif,
preview_meteor_wave_frames.png.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_firewave as fw  # noqa: E402
import breach_firewave_plus as fp  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

FW, FH = 184, 168
CX, CY = 92, 98
FRAMES = 18
FRAME_MS = 60

# Холст и крест — в обоих модулях: функции пламени берут их оттуда.
for module in (fw, fp):
    module.FW, module.FH, module.CX, module.CY = FW, FH, CX, CY
U, V = fw.grid_coords()
CHEB = np.maximum(np.abs(U), np.abs(V))
AREA = (CHEB <= 1.5) & (np.minimum(np.abs(U), np.abs(V)) <= 0.5)
BACK = (np.round(U) + np.round(V)) < 0
for module in (fw, fp):
    module.U, module.V, module.CHEB, module.AREA, module.BACK = U, V, CHEB, AREA, BACK
fp.FRAMES, fp.WAVE_START, fp.WAVE_END, fp.EDGE = FRAMES, 1, 8, 1.62
fw.front_radius = fp.front_radius


def save(unders, overs, out, scale=2):
    os.makedirs(out, exist_ok=True)
    Image.fromarray(np.concatenate(unders, axis=1), "RGBA").save(os.path.join(out, "meteor_wave_under.png"))
    Image.fromarray(np.concatenate(overs, axis=1), "RGBA").save(os.path.join(out, "meteor_wave_over.png"))
    position = {(4, 3): "bN", (3, 3): "bP", (5, 3): "bP", (4, 4): "bP", (4, 2): "bR", (5, 4): "bP"}
    tiles, shadows, pieces, at = fw.board_scene(position, (4, 3))
    frames = []
    for k in range(-3, FRAMES + 5):
        f = min(max(k, 0), FRAMES - 1)
        im = tiles.copy()
        im.alpha_composite(shadows)
        if 0 <= k < FRAMES:
            im.alpha_composite(Image.fromarray(unders[f], "RGBA"), at)
        im.alpha_composite(pieces)
        if 0 <= k < FRAMES:
            im.alpha_composite(Image.fromarray(overs[f], "RGBA"), at)
        frames.append(im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST))
    frames[0].save(os.path.join(out, "preview_meteor_wave.gif"), save_all=True, append_images=frames[1:],
                   duration=FRAME_MS, loop=0)
    w, h = frames[0].width // scale, frames[0].height // scale
    picks = list(range(0, FRAMES, 2))
    sheet = Image.new("RGB", (5 * (w + 4) + 4, 2 * (h + 4) + 4), (27, 30, 39))
    for n, f in enumerate(picks):
        sheet.paste(frames[f + 3].resize((w, h), Image.NEAREST), (4 + (n % 5) * (w + 4), 4 + (n // 5) * (h + 4)))
    sheet.save(os.path.join(out, "preview_meteor_wave_frames.png"))


if __name__ == "__main__":
    save(*fp.render(), OUT)
