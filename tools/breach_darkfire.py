"""Тёмная аура: фигура в чёрно-фиолетовом пламени — в стиле Into the Breach.

Устроена как аура электрической фигуры (`breach_arcs.py`): пламя вокруг силуэта —
по бокам колышется, вверх уходит длинными языками, узор непрерывно бежит вверх. Только
гуще (шире и плотнее) и в чёрно-фиолетовых тонах: нутро почти чёрное, у самой фигуры —
тёмно-фиолетовая кайма (фигура не сливается с аурой), снаружи — яркий фиолетовый контур,
кончики языков — сиреневые блики. Фигура чуть темнеет, по её краю мерцает фиолетовый
отсвет; в ауре — фиолетовые искры.

Кадры — замена спрайта на холсте FW×FH, точка опоры (центр клетки) — (FAX, FAY), как у
`electric_*`; тень — обычная. Вступление (аура разгорается) — [0, LOOP_START), повтор —
[LOOP_START, FRAMES), после последнего кадра снова LOOP_START.

`py tools/breach_darkfire.py [папка]` — по умолчанию assets/breach/effects:
dark_fire_<цвет><фигура>.png (кадры в ряд, все 12 фигур), preview_dark_fire.gif,
preview_dark_fire_frames.png.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_arcs as ba  # noqa: E402
import breach_burn as burn  # noqa: E402
import breach_firewave as fw  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

FILL = hexc("#0c0812")  # нутро ауры — почти чёрное
DEEP = hexc("#170d26")  # ближе к фигуре — чуть фиолетовее
LINER = hexc("#3a1c66")  # кайма вплотную к фигуре: отделяет её чёрный контур от ауры
GLOW = hexc("#4f249a")  # полоса под контуром
CONTOUR = hexc("#8644f2")  # контур снаружи — яркий фиолетовый
GLINT = [hexc(c) for c in ["#e2c8ff", "#b583ff", "#7a3fe0"]]  # блики и искры, от светлого
TINT = np.array(hexc("#160d24"), float)  # чем темнеет фигура
RIM = np.array(hexc("#a066ff"), float)  # отсвет по краю фигуры
FW, FH = ba.FW, ba.FH  # 120×120
FAX, FAY = ba.FAX, ba.FAY  # центр клетки
SX, SY = FAX - art.AX, FAY - art.AY
FRAME_MS = 60
INTRO = 8
LOOP = 32  # 1,9 с
LOOP_START, FRAMES = INTRO, INTRO + LOOP
RISE = 3  # пикселей вверх за кадр; период шума = RISE × LOOP — шов не виден
SIDE = (4.5, 5.5)  # толщина ауры по бокам: основа и размах колыхания (у электрической 3.5 и 4.5)
UP = (7.0, 25.0)  # высота языков вверх: основа и размах (у электрической 5 и 22)


def render(code, seed=5, bodies=None):
    """Кадры тёмной ауры; bodies(f) — своё тело фигуры в кадре f (холст FW×FH, аура — по обычному силуэту)."""
    rng = np.random.default_rng(seed)
    sprite = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA"))
    canvas = np.zeros((FH, FW, 4), np.uint8)
    canvas[SY : SY + sprite.shape[0], SX : SX + sprite.shape[1]] = sprite
    sil = canvas[..., 3] > 0
    inner = sil & ~burn.boundary(sil)
    rim = burn.boundary(inner)
    base = np.nonzero(sil)[0].max()
    d = ba.distance(sil, 16)
    period = RISE * LOOP
    noise = ba.loop_noise(4, period, rng)
    tongues = ba.loop_noise(8, period, rng, cell_x=3)
    # высота над силуэтом (расширенным на 2 пикселя) по столбцу — для языков вверх
    wide = ba.dilate(ba.dilate(sil))
    gap = np.full((FH, FW), np.inf)
    run = np.full(FW, np.inf)
    for y in range(FH - 1, -1, -1):
        run = np.where(wide[y], 0, run + 1)
        gap[y] = run
    yy = np.mgrid[0:FH, 0:FW][0]
    frames = []
    for f in range(FRAMES):
        lf = f - LOOP_START
        grow = min(1.0, (f + 1) / INTRO) if f < INTRO else 1.0
        shift = RISE * (lf % LOOP) if lf >= 0 else RISE * f
        n, nt = noise(shift), tongues(shift)
        # ---- аура, как у электрической, но гуще
        side = (SIDE[0] + SIDE[1] * n) * grow
        side = np.where(yy > base - 3, side * 0.4, side)
        up = (UP[0] + UP[1] * nt ** 2) * grow
        aura = ((d <= side) | (gap <= up)) & ~sil
        aura &= yy < base + 2
        img = np.zeros((FH, FW, 4), np.uint8)
        if aura.any():
            outer = burn.boundary(aura) & ~burn.neighbours_any(sil)
            under = burn.boundary(aura & ~outer) & ~burn.neighbours_any(sil)
            img[aura] = (*FILL, 215)
            img[aura & ((d <= 3) | (gap <= up * 0.3))] = (*DEEP, 230)
            img[aura & (d <= 1)] = (*LINER, 240)
            img[under] = (*GLOW, 235)
            img[outer] = (*CONTOUR, 250)
            # кончики языков — сиреневые блики
            for x in range(FW):
                col = np.nonzero(aura[:, x])[0]
                if len(col) > 6 and col[0] < base - 10:
                    img[col[0], x] = (*GLINT[0], 255)
                    img[col[1], x] = (*GLINT[1], 255)
            # искорки в ауре: короткие чёрточки
            pts = np.argwhere(aura & ~outer & (d > 1))
            for y, x in pts[rng.choice(len(pts), size=min(6, len(pts)), replace=False)]:
                img[max(0, y - 1) : y + 1, x] = (*GLINT[1], 255)
        # ---- фигура: чуть темнее, по краю — фиолетовый отсвет
        body = (canvas if bodies is None else bodies(f)).copy()
        rgb = body[..., :3].astype(float)
        dark = 0.18 * grow
        rgb[sil] = rgb[sil] * (1 - dark) + TINT * dark
        glow = (0.3 + 0.3 * n[rim]) * grow
        rgb[rim] = rgb[rim] * (1 - glow[:, None]) + RIM * glow[:, None]
        body[..., :3] = rgb.astype(np.uint8)
        img[sil] = body[sil]
        loose = (body[..., 3] > 0) & ~sil  # то, что своё тело выносит за силуэт (искры лавы)
        img[loose] = body[loose]
        frames.append(img)
    return frames


# ---------------------------------------------------------------- превью на доске
def scene_frames(frames, code, center, position, loops=3):
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
    """Все фигуры: верхний ряд — белые, нижний — чёрные."""
    crop = (70, 0, 230, 200)
    rows = []
    for color in "wb":
        enemy = "b" if color == "w" else "w"
        rows.append([[im.crop(crop) for im in scene_frames(all_frames[color + k], color + k, (4, 3),
                                                          {(4, 3): color + k, (5, 4): enemy + "P"})]
                     for k in "PNBRQK"])
    cw, ch = crop[2] - crop[0], crop[3] - crop[1]
    frames = []
    for n in range(len(rows[0][0])):
        im = Image.new("RGB", (6 * cw + 5 * 4, 2 * ch + 4), (27, 30, 39))
        for r, row in enumerate(rows):
            for c, scene in enumerate(row):
                im.paste(scene[n], (c * (cw + 4), r * (ch + 4)))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 4))
    for i, k in enumerate((10, 20, 30, 40)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_dark_fire.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    cols = 10
    rows_n = (FRAMES + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (FW + 2), 2 * rows_n * (FH + 2)), (40, 44, 56, 255))
    for r, code in enumerate(("wN", "bQ")):
        for k, fr in enumerate(all_frames[code]):
            sheet.alpha_composite(Image.fromarray(fr, "RGBA"),
                                  ((k % cols) * (FW + 2), (r * rows_n + k // cols) * (FH + 2)))
    sheet.save(os.path.join(out, "preview_dark_fire_frames.png"))


def main():
    os.makedirs(OUT, exist_ok=True)
    all_frames = {}
    for code in (c + k for c in "wb" for k in "PNBRQK"):
        frames = render(code, seed=5 + "PNBRQK".index(code[1]))
        all_frames[code] = frames
        Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(OUT, f"dark_fire_{code}.png"))
        print(code, flush=True)
    save_preview(all_frames, OUT)
    print("done", OUT)


if __name__ == "__main__":
    main()
