"""Варвар вместе с остальными особыми силами — в стиле Into the Breach.

Основа — варвар из `breach_barbarian.py` (рогатый шлем, мех, корпус в цветах сторон). С ним:

- анимации — те же генераторы, что у обычных особых фигур, но на спрайте варвара (подмена
  источника в `from_barbarian`: путь `pieces/<код>.png` ведёт в `pieces/barbarian/`):
  огненный — лавовые трещины (`breach_lava.py`, 28 кадров 64×84), электрический — аура с дугами
  без концовки (`breach_arcs.py`, `electric_hold`, 24 кадра 120×120), воздушный — вихрь
  (`breach_wind.py`, 40 кадров), теневой — тёмная аура (`breach_darkfire.py`, 40 кадров) и теневой
  шаг (`breach_shadowstep.py`, исчезновение 18 и появление 13 кадров). Разметка кадров и опоры —
  как у обычных полос этих эффектов;
- спрайты — объёмные заново: золотой (`gold`: корпус и шлем — золото, у чёрных красное, обод —
  эмаль, рога — кость, мех — как был) и ледяной (`ice`: корпус иссиня-голубой с наледью и инеем
  по `breach_ice_pieces.iced`, шлем — заиндевелое железо, сосульки из-под поясов, кристаллы у
  основания; материалы льда — 11 и 12, чтобы не спорить со шлемом и ободом варвара).

Маг с варварскими рогами — отдельный набор спрайтов `breach_wizard_horns.py`.

`py tools/breach_barbarian_combo.py` — по умолчанию в assets/breach:
effects/barbarian/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png,
pieces/barbarian_<gold|ice>/<код>.png и _shadow.png; превью pieces/preview_barbarian_combo.png
(ряды: варвар, золотой, ледяной и по кадру огненного, электрического, воздушного, теневого) и
effects/preview_barbarian_combo.gif (анимации на клетках).
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
import breach_barbarian as bb  # noqa: E402
import breach_darkfire as dark  # noqa: E402
import breach_firewave as fw  # noqa: E402
import breach_gilded as gilded  # noqa: E402
import breach_ice_pieces as ip  # noqa: E402
import breach_lava as lava  # noqa: E402
import breach_shadowstep as step  # noqa: E402
import breach_wind as wind  # noqa: E402
from breach_art import GLOW, MAIN, R, T, TRIM, Y, norm  # noqa: E402

sys.argv = _argv
ROOT = os.path.join(art.ROOT, "assets", "breach")
PIECES = os.path.join(ROOT, "pieces")
EFFECTS = os.path.join(ROOT, "effects", "barbarian")
CODES = [c + k for c in "wb" for k in "PNBRQK"]

# ---------------------------------------------------------------- спрайт варвара вместо обычного
_open = Image.open


def from_barbarian(fp, *args, **kwargs):
    """Image.open, у которого pieces/<код>.png — спрайт варвара (генераторы эффектов берут обычный)."""
    if isinstance(fp, str):
        folder, name = os.path.split(os.path.abspath(fp))
        if os.path.normcase(folder) == os.path.normcase(os.path.abspath(PIECES)) and re.fullmatch(r"[wb][PNBRQK]\.png", name):
            fp = os.path.join(PIECES, "barbarian", name)
    return _open(fp, *args, **kwargs)


# ---------------------------------------------------------------- золотой и ледяной варвар
ICE_M, RIME_M = 11, 12  # лёд и иней — свои номера (5 и 6 у варвара — шлем и обод)
FROST_IRON = ["#121c28", "#283c52", "#466684", "#7aa0c0", "#bcd8ee"]  # заиндевелый шлем


def gold_palette(color):
    g = gilded.PALETTES["gold"][color]
    pal = dict(art.PALETTES[color])
    pal.update(bb.EXTRA)
    pal[MAIN] = g[MAIN]
    pal[TRIM] = g[TRIM]
    pal[bb.HELM] = g[MAIN]  # шлем — тоже золото
    pal[bb.BRONZE] = g[TRIM]  # обод и заклёпки — эмаль
    return pal


def ice_palette(color):
    pal = dict(art.PALETTES[color])
    pal.update(bb.EXTRA)
    pal[MAIN] = ip.PALETTES[color][MAIN]
    pal[TRIM] = ip.PALETTES[color][TRIM]
    pal[bb.HELM] = FROST_IRON
    pal[ICE_M] = ip.ICE_RAMP
    pal[RIME_M] = ip.RIME_RAMP
    return pal


def iced_barbarian(kind, color):
    """Варвар в наледи: части — через `ip.iced` (номера льда подменены), сосульки и кристаллы — как у ледяных."""
    piece = bb.barbarian(kind, color)
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


def render_static(variant, kind, color):
    key = f"barbarian_{variant}_{color}"
    if variant == "gold":
        art.PALETTES[key] = gold_palette(color)
        piece = bb.barbarian(kind, color)
    else:
        art.PALETTES[key] = ice_palette(color)
        piece = iced_barbarian(kind, color)
    body, shadow = art.render_piece(piece, key, 0.86)
    if variant == "ice":  # искры на самом светлом льду, как у ледяных фигур
        img = np.array(body)
        top = np.array(art.hexc(ip.ICE_RAMP[3]))
        bright = (img[..., :3] == top).all(axis=-1) & (img[..., 3] > 0)
        ys, xs = np.nonzero(bright)
        for y, x in sorted(zip(ys, xs), key=lambda q: q[0] + q[1])[:3]:
            img[y, x, :3] = (255, 255, 255)
        body = Image.fromarray(img, "RGBA")
    return body, shadow


# ---------------------------------------------------------------- превью
def sheet(rows, path):
    cw, ch = 60, 104
    w = 12 * cw + 8
    im = Image.new("RGBA", (w, len(rows) * ch + 8), art.hexc("#2c3a2c") + (255,))
    for r in range(1, len(rows), 2):
        im.alpha_composite(Image.new("RGBA", (w, ch), art.hexc("#3d4a3a") + (255,)), (0, 4 + r * ch))
    for r, row in enumerate(rows):
        for c, code in enumerate(CODES):
            frame, anchor = row[code]
            ax, ay = anchor
            im.alpha_composite(frame.crop((ax - cw // 2, ay - 86, ax + cw // 2, ay + 18)), (4 + c * cw, 4 + r * ch))
    im.resize((im.width * 2, im.height * 2), Image.NEAREST).save(path)


def anim_gif(anims, path, codes=("wN", "bQ", "wK"), length=48, scale=2):
    """Анимации на клетке: строки — фигуры, столбцы — огненная, электрическая, воздушная, теневая."""
    tile = art.block("dark", 11)
    cell_w, cell_h = 120, 120
    frames = []
    for f in range(length):
        im = Image.new("RGBA", (len(anims) * cell_w, len(codes) * cell_h), (27, 30, 39, 255))
        for c, (strips, anchor, loop) in enumerate(anims):
            for r, code in enumerate(codes):
                seq = strips[code]
                i = f if f < len(seq) else loop[0] + (f - loop[0]) % (loop[1] - loop[0])
                ax, ay = anchor
                x0, y0 = c * cell_w + 60 - ax, r * cell_h + 96 - ay
                im.alpha_composite(tile, (c * cell_w + 60 - art.TW // 2, r * cell_h + 96 - art.TH // 2))
                im.alpha_composite(Image.fromarray(seq[i], "RGBA"), (x0, y0))
        frames.append(im.convert("RGB").resize((im.width * scale, im.height * scale), Image.NEAREST))
    pal = frames[length // 2].quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(path, save_all=True, append_images=gif[1:], duration=60, loop=0, disposal=1)


def main():
    os.makedirs(EFFECTS, exist_ok=True)
    ip.ICE, ip.RIME = ICE_M, RIME_M  # `iced`/`icicle`/`shard` читают их при сборке и отрисовке
    rows = []
    base = {code: (Image.open(os.path.join(PIECES, "barbarian", f"{code}.png")).convert("RGBA"), (art.AX, art.AY))
            for code in CODES}
    rows.append(base)
    for variant in ("gold", "ice"):
        folder = os.path.join(PIECES, f"barbarian_{variant}")
        os.makedirs(folder, exist_ok=True)
        row = {}
        for code in CODES:
            body, shadow = render_static(variant, code[1], code[0])
            body.save(os.path.join(folder, f"{code}.png"))
            shadow.save(os.path.join(folder, f"{code}_shadow.png"))
            row[code] = (body, (art.AX, art.AY))
            print(variant, code, flush=True)
        rows.append(row)

    Image.open = from_barbarian  # дальше генераторы эффектов видят спрайт варвара
    try:
        fire, spark, air, shade = {}, {}, {}, {}
        for code in CODES:
            k = code[1]
            fire[code] = lava.render(code, seed=40 + "PNBRQK".index(k))
            spark[code] = arcs.render(code, arcs.SEEDS[k])[: arcs.LOOP_END]
            air[code] = wind.render(code)
            out_f, in_f, aura = step.render(code, seed=31 + "PNBRQK".index(k))
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
    sheet(rows, os.path.join(PIECES, "preview_barbarian_combo.png"))
    anim_gif([(fire, (art.AX, art.AY), (0, lava.FRAMES)), (spark, (arcs.FAX, arcs.FAY), (arcs.LOOP_START, arcs.LOOP_END)),
              (air, (wind.FAX, wind.FAY), (wind.LOOP_START, wind.FRAMES)),
              (shade, (dark.FAX, dark.FAY), (dark.LOOP_START, dark.FRAMES))],
             os.path.join(ROOT, "effects", "preview_barbarian_combo.gif"))
    print("done", ROOT)


if __name__ == "__main__":
    main()
