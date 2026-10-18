"""Воздушный вихрь вокруг низа коня — как у воздушного элементаля или Windrun, в стиле
Into the Breach.

Подставку и низ туловища плотно обвивает столб из струй воздуха: струи идут друг над
другом через RING_STEP пикселей, радиус каждой — по ширине силуэта на её высоте (вихрь
облегает фигуру), фаза закручена по высоте — получается спираль. Струи быстро крутятся и
непрерывно ползут вверх: сверху тают, снизу появляются новые. У струи белая голова, хвост
светлеет в голубоватый и тает, снизу — тёмная подводка (видно и на светлых клетках);
передняя половина эллипса — перед фигурой, задняя — за ней. По клетке — бледный след.
Фигура стоит на месте.

Кадры — замена спрайта на холсте FW×FH, точка опоры (центр клетки) — (FAX, FAY), как у
`electric_*`; тень — обычная. Вступление (вихрь набирается) — [0, LOOP_START), повтор —
[LOOP_START, FRAMES), после последнего кадра снова LOOP_START.

`py tools/breach_wind.py [папка]` — по умолчанию assets/breach/effects: wind_<цвет><фигура>.png
(кадры в ряд, все 12 фигур), preview_wind.gif (кони на доске), preview_wind_all.gif (остальные),
preview_wind_frames.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_firewave as fw  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

AIR = [hexc(c) for c in ["#ffffff", "#e6f4f8", "#bfdbe6", "#8fb5c7", "#4f7489"]]
FW, FH = 120, 120
FAX, FAY = 60, 92  # центр клетки
SX, SY = FAX - art.AX, FAY - art.AY
FRAME_MS = 60
INTRO = 8
LOOP = 32  # 1,9 с
LOOP_START, FRAMES = INTRO, INTRO + LOOP
SPIN = 2 * math.pi * 3 / LOOP  # у струи две дуги напротив — шов незаметен при целых полуоборотах
RING_STEP = 2.6  # шаг струй по высоте
WRAP = 0.5  # до какой доли высоты фигуры поднимается вихрь
CLIMB = 4  # на сколько шагов струи поднимаются за повтор
TWIST = 0.32  # закрутка фазы по высоте (рад на пиксель) — спираль
GAP = 2.5  # насколько струя отстоит от силуэта
FLARE, FLARE_H = 9.0, 7.0  # у самой земли вихрь расширяется раструбом: на сколько и до какой высоты
THIN = 0.7  # насколько вихрь редеет к верху столба (у середины фигуры): 0 — ровный, 1 — исчезает
DENSE_H = 6.0  # ниже этой высоты — третья и четвёртая струя и толстые головы
GROUND = 3  # колец ветра, расходящихся по клетке от основания
GROUND_PASSES = 2  # сколько раз каждое кольцо проходит путь за повтор (целое — шов не виден)
GROUND_MAX = 26.0  # до какого радиуса расходятся (край верхней грани клетки)


def arc(layer, cx, h, rx, theta, length, alpha, front, thick=False):
    """Струя: дуга изометрического эллипса (ry = rx / 2) с центром (cx, FAY − h); голова в
    theta, хвост тянется назад на length. front — передняя (True) или задняя половина;
    thick — голова и середина в два пикселя (у основания вихрь плотнее)."""
    if alpha <= 0.02 or rx < 2:
        return
    cy = FAY - h
    steps = int(length * rx * 1.6) + 2
    pts = {}
    for i in range(steps):
        t = i / (steps - 1)  # 0 — голова, 1 — конец хвоста
        a = theta - length * t
        if (math.sin(a) > 0) != front:
            continue
        x, y = int(round(cx + math.cos(a) * rx)), int(round(cy + math.sin(a) * rx * 0.5))
        if 0 <= x < FW and 0 <= y < FH and (x, y) not in pts:
            pts[(x, y)] = t
    for (x, y), t in pts.items():
        if t < 0.18:
            col, a = AIR[0], 255
        elif t < 0.45:
            col, a = AIR[1], 240
        elif t < 0.75:
            col, a = AIR[2], 200
        else:
            col, a = AIR[3], 130
        a = int(a * alpha)
        if y + 1 < FH and t < 0.6 and layer[y + 1, x, 3] < 100:  # подводка снизу
            layer[y + 1, x] = (*AIR[4], int(150 * alpha))
        if thick and t < 0.5 and y - 1 >= 0 and layer[y - 1, x, 3] < a:
            layer[y - 1, x] = (*AIR[1], int(a * 0.9))
        if layer[y, x, 3] <= a:
            layer[y, x] = (*col, a)


def render(code):
    sprite = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA"))
    canvas = np.zeros((FH, FW, 4), np.uint8)
    canvas[SY : SY + sprite.shape[0], SX : SX + sprite.shape[1]] = sprite
    sil = canvas[..., 3] > 0
    rows = np.nonzero(sil.any(axis=1))[0]
    top = FAY - rows.min()  # высота фигуры над центром клетки
    # ширина и середина силуэта по строкам: вихрь облегает фигуру (берём шире из соседних строк)
    half, mid = np.zeros(FH), np.full(FH, float(FAX))
    for y in rows:
        xs = np.nonzero(sil[y])[0]
        half[y], mid[y] = (xs.max() - xs.min() + 1) / 2, (xs.max() + xs.min() + 1) / 2
    wide = np.array([half[max(0, y - 3) : y + 4].max() for y in range(FH)])
    span = top * WRAP  # высота столба
    n_rings = int(round(span / RING_STEP))
    span = n_rings * RING_STEP  # кратно шагу — при подъёме набор струй повторяется
    def rx_at(h, grow):
        """Радиус вихря на высоте h: по ширине силуэта, у земли — раструб."""
        y = min(max(int(round(FAY - h * grow)), 0), FH - 1)
        flare = FLARE * max(0.0, 1 - h / FLARE_H) ** 2
        return (wide[y] + GAP) * (0.8 + 0.2 * grow) + 1.5 * h / span + flare * grow

    frames = []
    for f in range(FRAMES):
        grow = min(1.0, (f + 1) / INTRO) if f < INTRO else 1.0
        theta = SPIN * f
        climb = (RING_STEP * CLIMB / LOOP) * f
        back = np.zeros((FH, FW, 4), np.uint8)
        front = np.zeros((FH, FW, 4), np.uint8)
        # по клетке от основания расходятся кольца ветра и тают к её краю
        r0 = rx_at(0, grow)
        for i in range(GROUND):
            u = (GROUND_PASSES * f / LOOP + i / GROUND) % 1.0
            r = r0 + (GROUND_MAX - r0) * u
            a = grow * min(1.0, u / 0.12) * (1 - u) ** 1.3
            for k in (0.0, math.pi):
                for layer, is_front in ((back, False), (front, True)):
                    arc(layer, FAX, -0.5, r, theta * 0.8 + 2.0 * i + k, 2.2 - 0.8 * u, a, is_front)
        for i in range(n_rings):
            h = (i * RING_STEP + climb) % span
            p = h / span  # 0 — у земли, 1 — верх столба
            hh = h * grow
            fade = min(1.0, h / (RING_STEP * 1.2)) * min(1.0, (1 - p) / 0.3)  # снизу проявляется, сверху тает
            y = int(round(FAY - hh))
            rx = rx_at(h, grow)
            cx = mid[min(max(y, 0), FH - 1)] if half[min(max(y, 0), FH - 1)] else FAX
            ph = theta + TWIST * h
            ln = 2.3 - 0.6 * p
            dense = h < DENSE_H
            strength = fade * grow * (1 - THIN * p)
            for k in ((0.0, math.pi / 2, math.pi, 1.5 * math.pi) if dense else (0.0, math.pi)):  # струи по кругу
                for layer, is_front in ((back, False), (front, True)):
                    arc(layer, cx, hh, rx, ph + k, ln, strength, is_front, thick=dense)
        img = back.copy()
        img[sil] = canvas[sil]
        # дымка столба: белёсая завеса по объёму вихря, к краям гуще
        yy, xx = np.mgrid[0:FH, 0:FW]
        mist = np.zeros((FH, FW))
        for h in np.arange(0, span * grow, 1.0):
            y = int(round(FAY - h))
            rx = rx_at(h, grow)
            cx = mid[y] if half[y] else FAX
            e = np.hypot((xx + 0.5 - cx) / rx, (yy + 0.5 - (FAY - h)) / (rx * 0.5))
            inside = e <= 1
            k = (0.12 + 0.17 * e) * (1 - 0.85 * h / span) * grow
            mist = np.where(inside, np.maximum(mist, k), mist)
        mm = mist > 0.02
        a = mist[..., None]
        img[mm, :3] = (img[..., :3] * (1 - a) + np.array(AIR[1]) * a)[mm].astype(np.uint8)
        img[mm & (img[..., 3] == 0), 3] = (mist[mm & (img[..., 3] == 0)] * 255).astype(np.uint8)
        m = front[..., 3] > 0
        fa = front[..., 3:4].astype(float) / 255
        mix = img[..., :3] * (1 - fa) + front[..., :3] * fa
        img[m, :3] = mix[m].astype(np.uint8)
        img[m, 3] = np.maximum(img[m, 3], front[m, 3])
        frames.append(img)
    return frames


# ---------------------------------------------------------------- превью на доске
def scene_frames(frames, code, center, position, loops=4):
    tiles, shadows, _, at = fw.board_scene(position, center)
    rest = {sq: c for sq, c in position.items() if sq != center}
    back = fw.board_scene({sq: c for sq, c in rest.items() if sq[0] + 7 - sq[1] < center[0] + 7 - center[1]}, center)[2]
    front = fw.board_scene({sq: c for sq, c in rest.items() if sq[0] + 7 - sq[1] >= center[0] + 7 - center[1]}, center)[2]
    cell = (at[0] + fw.CX, at[1] + fw.CY)
    still = Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA")
    order = [None] * 4 + list(range(LOOP_START)) + list(range(LOOP_START, FRAMES)) * loops
    out = []
    for f in order:
        im = tiles.copy()
        im.alpha_composite(shadows)
        im.alpha_composite(back)
        if f is None:
            im.alpha_composite(still, (cell[0] - art.AX, cell[1] - art.AY))
        else:
            im.alpha_composite(Image.fromarray(frames[f], "RGBA"), (cell[0] - FAX, cell[1] - FAY))
        im.alpha_composite(front)
        out.append(im.convert("RGB"))
    return out


def save_preview(all_frames, out, scale=2):
    crop = (55, 10, 245, 200)
    scenes = [scene_frames(all_frames["wN"], "wN", (4, 3), {(4, 3): "wN", (5, 4): "bP", (2, 3): "wP"}),
              scene_frames(all_frames["bN"], "bN", (4, 3), {(4, 3): "bN", (3, 2): "wP", (6, 4): "bP"})]
    frames = []
    for a, b in zip(*scenes):
        a, b = a.crop(crop), b.crop(crop)
        im = Image.new("RGB", (2 * a.width + 4, a.height), (27, 30, 39))
        im.paste(a, (0, 0))
        im.paste(b, (a.width + 4, 0))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 4))
    for i, k in enumerate((10, 20, 30, 40)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_wind.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    cols = 10
    rows = (FRAMES + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (FW + 2), 2 * rows * (FH + 2)), (40, 44, 56, 255))
    for r, code in enumerate(("wN", "bN")):
        for k, fr in enumerate(all_frames[code]):
            sheet.alpha_composite(Image.fromarray(fr, "RGBA"),
                                  ((k % cols) * (FW + 2), (r * rows + k // cols) * (FH + 2)))
    sheet.save(os.path.join(out, "preview_wind_frames.png"))


def save_all_preview(all_frames, out, scale=2):
    """Остальные фигуры, каждая на своём куске доски: верхний ряд — белые, нижний — чёрные."""
    crop = (55, 0, 245, 200)
    rows = []
    for color in "wb":
        enemy = "b" if color == "w" else "w"
        rows.append([[im.crop(crop) for im in scene_frames(all_frames[color + k], color + k, (4, 3),
                                                          {(4, 3): color + k, (5, 4): enemy + "P"}, loops=3)]
                     for k in "PBRQK"])
    cw, ch = crop[2] - crop[0], crop[3] - crop[1]
    frames = []
    for n in range(len(rows[0][0])):
        im = Image.new("RGB", (5 * cw + 4 * 4, 2 * ch + 4), (27, 30, 39))
        for r, row in enumerate(rows):
            for c, scene in enumerate(row):
                im.paste(scene[n], (c * (cw + 4), r * (ch + 4)))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 4))
    for i, k in enumerate((10, 20, 30, 40)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_wind_all.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)


def main():
    os.makedirs(OUT, exist_ok=True)
    all_frames = {}
    for code in (c + k for c in "wb" for k in "PNBRQK"):
        frames = render(code)
        all_frames[code] = frames
        Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(OUT, f"wind_{code}.png"))
    save_preview(all_frames, OUT)
    save_all_preview(all_frames, OUT)
    print("done", OUT)


if __name__ == "__main__":
    main()
