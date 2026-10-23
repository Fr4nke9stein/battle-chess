"""Большой взрыв огненной фигуры крестом (герб двух огоньков) — по образцу `breach_firewave.py`.

Огненная волна расходится от клетки фигуры толстым крестом: 3×3 клетки вокруг и ещё по три клетки
с каждой стороны (`AREA`: в квадрате 5×5 без четырёх угловых клеток). Кромка огня бежит
квадратными кольцами, как у обычной волны, только дальше (до `REACH` + 0.5 клетки) и дольше; за
ней — подпалины, огоньки догорают по всему кресту, искры и дым. Слои `under` (до фигур) и `over`
(после) — как у `fire_wave_*`; пламя, огоньки и дым рисуют функции `breach_firewave.py`.

Кадр FW×FH, центр верхней грани клетки фигуры — (CX, CY), FRAMES кадров по FRAME_MS.

`py tools/breach_firewave_plus.py [папка]` — по умолчанию assets/breach/effects:
fire_wave_plus_under.png, fire_wave_plus_over.png (кадры в ряд), preview_fire_wave_plus.gif,
preview_fire_wave_plus_frames.png.
"""
import math
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

FIRE, OUTLINE = burn.FIRE, burn.OUTLINE
TW, TH = art.TW, art.TH
REACH = 2  # крест — на столько клеток от фигуры
FW, FH = 304, 248
CX, CY = 152, 132  # центр верхней грани клетки фигуры
FRAMES = 26
FRAME_MS = 60
WAVE_START, WAVE_END = 2, 14

# Холст, клетки и области — те же формулы, что у обычной волны, на своём холсте.
fw.FW, fw.FH, fw.CX, fw.CY = FW, FH, CX, CY
U, V = fw.grid_coords()
CHEB = np.maximum(np.abs(U), np.abs(V))
AREA = (CHEB <= REACH + 0.5) & (np.minimum(np.abs(U), np.abs(V)) <= 1.5)
BACK = (np.round(U) + np.round(V)) < 0  # клетки позади фигуры — под фигурами
fw.U, fw.V, fw.CHEB, fw.AREA, fw.BACK = U, V, CHEB, AREA, BACK
EDGE = REACH + 0.62


def front_radius(f):
    """Радиус кромки в клетках: быстро вначале, медленнее к краю креста."""
    t = np.clip((f - WAVE_START) / (WAVE_END - WAVE_START), 0, 1)
    r = 0.35 + (EDGE - 0.35) * (1 - (1 - t) ** 2)
    return r + 0.13 * max(0, f - WAVE_END)


def render(seed=17):
    rng = np.random.default_rng(seed)
    noise = burn.value_noise(FH, FW, 5, rng)
    residue = burn.value_noise(FH, FW, 3, rng)
    jag = CHEB + 0.11 * (noise - 0.5)  # неровная кромка
    embers, smoke = burn.Particles(), burn.Particles()
    unders, overs = [], []
    ring_zone = AREA & (CHEB > 0.42)
    for f in range(FRAMES):
        under = np.zeros((FH, FW, 4), np.uint8)
        over = np.zeros((FH, FW, 4), np.uint8)
        dc = front_radius(f) - jag

        # вспышка по краю своей клетки перед волной
        if f < WAVE_START + 1:
            flash = (CHEB > 0.40) & (CHEB <= 0.47 + 0.03 * f)
            under[burn.neighbours_any(flash) & ~flash & AREA] = (*OUTLINE, 255)
            under[flash] = (*FIRE[0 if f == 1 else 2], 255)

        # подпалины позади кромки, к концу гаснут
        fade = np.clip((FRAMES - 1 - f) / 8, 0, 1)
        scorch = ring_zone & (dc >= 0.36)
        under[scorch] = (0, 0, 0, int(75 * fade))
        glow = scorch & (residue > 0.66 + 0.3 * (1 - fade)) & (rng.random((FH, FW)) < 0.3)
        under[glow] = (*FIRE[7], 255)
        under[glow & (residue > 0.8)] = (*FIRE[5], 255)

        # кромка огня по земле — только внутри креста
        band = ring_zone & (dc >= 0) & (dc < 0.36)
        for lo, hi, col in ((0.0, 0.07, FIRE[1]), (0.07, 0.15, FIRE[3]), (0.15, 0.25, FIRE[5]), (0.25, 0.36, FIRE[7])):
            under[band & (dc >= lo) & (dc < hi)] = (*col, 255)
        ahead = burn.neighbours_any(band) & ~band & ring_zone & (dc < 0)
        under[ahead] = (*OUTLINE, 255)

        # огненная стена на кромке; за ней по всему кресту догорают огоньки
        lead = band & (dc < 0.15)
        ys, xs = np.nonzero(lead & (dc < 0.07))
        fb, ff = fw.flame_wall(lead, rng) if lead.any() else (np.zeros((FH, FW), bool),) * 2
        behind = AREA & (CHEB > 0.62) & (dc > 0.5)
        if behind.any() and f >= 5:
            left = max(0, 14 - max(0, f - WAVE_END) * 3)
            zone = np.nonzero(behind)
            if left:
                pick = rng.choice(len(zone[0]), size=min(left, len(zone[0])), replace=False)
                pts = [(zone[1][k], zone[0][k], 1.0) for k in pick]
                sb, sf = fw.small_fires(pts, rng)
                fb, ff = fb | sb, ff | sf
        fw.paint_flames(under, over, fb, ff)

        # искры наружу и вверх, дым от догорающего огня
        if len(ys):
            for k in rng.choice(len(ys), size=min(10, len(ys)), replace=False):
                dx, dy = xs[k] - CX, ys[k] - CY
                dl = math.hypot(dx, dy) + 1e-6
                embers.items.append(dict(x=xs[k] + 0.5, y=ys[k] - 2.0, vx=1.4 * dx / dl + rng.uniform(-0.4, 0.4),
                                         vy=rng.uniform(-3.4, -1.6), drag=0.88, life=int(rng.integers(5, 9)),
                                         age=0, wob=0.3, ph=rng.uniform(0, 6)))
        if 6 <= f <= FRAMES - 6 and f % 2 == 0:
            zone = np.nonzero(AREA & (CHEB > 0.7) & (dc > 0.4))
            if len(zone[0]):
                for _ in range(2):
                    k = rng.integers(len(zone[0]))
                    smoke.items.append(dict(x=zone[1][k] + 0.5, y=zone[0][k] - 8.0, vx=rng.uniform(-0.3, 0.3),
                                            vy=rng.uniform(-1.8, -1.2), drag=0.94, life=int(rng.integers(7, 11)),
                                            age=0, r0=rng.uniform(2.0, 2.8), wob=0.3, ph=rng.uniform(0, 6)))
        for p in smoke.items:
            life = p["age"] / p["life"]
            grow = p["r0"] + 3.0 * min(life / 0.6, 1.0)
            p["r"] = grow * (1.0 if life < 0.65 else max(0.0, 1 - (life - 0.65) / 0.35))
            fw.puff(over, p)
        for p in embers.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                life = p["age"] / p["life"]
                over[y, x] = (*FIRE[[1, 3, 5, 7][min(int(life * 4), 3)]], 255)
                if p["age"] < 2 and y + 1 < FH:
                    over[y + 1, x] = (*FIRE[5], 255)
        embers.step()
        smoke.step()
        unders.append(under)
        overs.append(over)
    return unders, overs


def save(unders, overs, out, scale=2):
    os.makedirs(out, exist_ok=True)
    Image.fromarray(np.concatenate(unders, axis=1), "RGBA").save(os.path.join(out, "fire_wave_plus_under.png"))
    Image.fromarray(np.concatenate(overs, axis=1), "RGBA").save(os.path.join(out, "fire_wave_plus_over.png"))
    position = {(4, 3): "wN", (3, 4): "bP", (4, 5): "bP", (6, 3): "bN", (5, 2): "bP", (2, 2): "bB",
                (4, 1): "bR", (2, 4): "wP", (1, 3): "bQ", (6, 5): "bP"}
    tiles, shadows, pieces, at = fw.board_scene(position, (4, 3), size=(420, 330))
    frames = []
    for k in range(-4, FRAMES + 6):
        f = min(max(k, 0), FRAMES - 1)
        im = tiles.copy()
        im.alpha_composite(shadows)
        if 0 <= k < FRAMES:
            im.alpha_composite(Image.fromarray(unders[f], "RGBA"), at)
        im.alpha_composite(pieces)
        if 0 <= k < FRAMES:
            im.alpha_composite(Image.fromarray(overs[f], "RGBA"), at)
        frames.append(im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * len(frames)))
    for k, p in enumerate(frames):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_fire_wave_plus.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    picks = list(range(0, FRAMES, 2))
    w, h = frames[0].width // scale, frames[0].height // scale
    cols = 5
    rows = (len(picks) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * (w + 4) + 4, rows * (h + 4) + 4), (27, 30, 39))
    for n, f in enumerate(picks):
        sheet.paste(frames[f + 4].resize((w, h), Image.NEAREST), (4 + (n % cols) * (w + 4), 4 + (n // cols) * (h + 4)))
    sheet.save(os.path.join(out, "preview_fire_wave_plus_frames.png"))


if __name__ == "__main__":
    fw.front_radius = front_radius
    unders, overs = render()
    save(unders, overs, OUT)
