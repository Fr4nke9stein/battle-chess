"""Горящий кратер метеорита (герб шляпы и метеорита) — клетка горит по краям, в стиле игры.

Одна клетка: посередине — тёмная выжженная воронка с тлеющими трещинами (огня в центре нет), к
краям — подпалина, по самому краю ромба — полоса раскалённых углей и низкое пламя языками
(огонь, контур и раскраска — функции `breach_firewave.py`: `flame_wall`, `paint_flames`).
Пламя на задних кромках (верхняя половина ромба) — слой `under` (до фигур), на передних —
`over` (после фигур), чтобы фигура в кратере стояла в огне. Кадры повторяются по кругу: языки
и угли каждый кадр свои — огонь мерцает.

Кадр FW×FH, центр верхней грани клетки — (CX, CY), FRAMES кадров по FRAME_MS.

`py tools/breach_crater.py [папка]` — по умолчанию assets/breach/effects: crater_under.png,
crater_over.png (кадры в ряд), preview_crater.gif, preview_crater_frames.png.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_firewave as fw  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

FIRE, OUTLINE, CHAR = burn.FIRE, burn.OUTLINE, burn.CHAR
FW, FH = 80, 112
CX, CY = 40, 84
FRAMES = 12
FRAME_MS = 80
SEED = 23

fw.FW, fw.FH, fw.CX, fw.CY = FW, FH, CX, CY
U, V = fw.grid_coords()
CHEB = np.maximum(np.abs(U), np.abs(V))  # 0.5 — край клетки
YY, XX = np.mgrid[0:FH, 0:FW]
fw.BACK = YY < CY  # верхняя половина ромба — за фигурой
TILE = CHEB <= 0.5


def render():
    rng = np.random.default_rng(SEED)
    noise = burn.value_noise(FH, FW, 4, rng)
    cracks = burn.value_noise(FH, FW, 2, rng)
    unders, overs = [], []
    for f in range(FRAMES):
        under = np.zeros((FH, FW, 4), np.uint8)
        over = np.zeros((FH, FW, 4), np.uint8)
        flicker = burn.value_noise(FH, FW, 3, np.random.default_rng(SEED * 31 + f))
        # подпалина по всей клетке, к середине темнее — выжженная воронка
        d = CHEB + 0.06 * (noise - 0.5)
        under[TILE & (d < 0.47)] = (*CHAR[2], 150)
        under[TILE & (d < 0.33)] = (*CHAR[1], 210)
        under[TILE & (d < 0.2)] = (*CHAR[0], 235)
        # тлеющие трещины в воронке — тусклые, мерцают
        crack = TILE & (d < 0.3) & (np.abs(cracks - 0.5) < 0.035)
        under[crack] = (*FIRE[7], 255)
        under[crack & (flicker > 0.6)] = (*FIRE[6], 255)
        # раскалённые угли по краю ромба
        band = TILE & (d >= 0.38)
        glow = band & (flicker > 0.35)
        under[band] = (*FIRE[7], 255)
        under[glow] = (*FIRE[5], 255)
        under[band & (flicker > 0.72)] = (*FIRE[3], 255)
        edge = burn.neighbours_any(TILE) & ~TILE
        under[edge & (YY > CY - 25)] = (*OUTLINE, 160)
        # низкое пламя языками по кромке
        lead = TILE & (d >= 0.44)
        fb, ff = fw.flame_wall(lead, rng, base_h=2.5, tongue=(5.0, 10.0), spacing=5)
        fw.paint_flames(under, over, fb, ff)
        # редкие искры над краем
        for _ in range(3):
            ys, xs = np.nonzero(lead)
            k = rng.integers(len(ys))
            y = ys[k] - int(rng.integers(8, 18))
            if 0 <= y < FH:
                over[y, xs[k]] = (*FIRE[int(rng.integers(1, 4))], 255)
        unders.append(under)
        overs.append(over)
    return unders, overs


def save(unders, overs, out, scale=3):
    os.makedirs(out, exist_ok=True)
    Image.fromarray(np.concatenate(unders, axis=1), "RGBA").save(os.path.join(out, "crater_under.png"))
    Image.fromarray(np.concatenate(overs, axis=1), "RGBA").save(os.path.join(out, "crater_over.png"))
    position = {(4, 3): "bP"}
    tiles, shadows, pieces, at = fw.board_scene(position, (4, 3), size=(200, 170))
    frames = []
    for k in range(FRAMES * 2):
        f = k % FRAMES
        im = tiles.copy()
        im.alpha_composite(shadows)
        im.alpha_composite(Image.fromarray(unders[f], "RGBA"), at)
        im.alpha_composite(pieces)
        im.alpha_composite(Image.fromarray(overs[f], "RGBA"), at)
        frames.append(im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST))
    frames[0].save(os.path.join(out, "preview_crater.gif"), save_all=True, append_images=frames[1:],
                   duration=FRAME_MS, loop=0)
    w, h = frames[0].size
    sheet = Image.new("RGB", (w * 4, h * 3), (27, 30, 39))
    for i in range(FRAMES):
        sheet.paste(frames[i], ((i % 4) * w, (i // 4) * h))
    sheet.save(os.path.join(out, "preview_crater_frames.png"))


if __name__ == "__main__":
    save(*render(), OUT)
