"""Теневой прыжок фигур с тёмной аурой: варп-разлом + «теневой шаг» — в стиле Into the Breach.

Фигура перескакивает с клетки на клетку, не проходя путь (фигуры между ними не мешают):
нечто среднее между варп-прыжком Warhammer 40 000 и теневым шагом из аниме.

Исчезновение (`shadowstep_out`, на исходной клетке): аура стягивается, фигура чернеет до
силуэта с фиолетовым контуром, по полу к ней сходится кольцо энергии; за фигурой
раскрывается варп-разлом — рваная вертикальная трещина, внутри чёрная с фиолетовыми
завихрениями, с сиреневой кромкой и фиолетовым контуром; фигуру за пару кадров втягивает
в разлом (сжимается по ширине), в стороны бьют теневые полосы рывка, осколки летят в
разлом, он схлопывается. На месте фигуры остаётся её едва заметный силуэт (тёмно-
фиолетовый, с контуром) — пару мгновений мерцает и тает построчно.

Появление (`shadowstep_in`, на целевой клетке): раскрывается такой же разлом, из него
рывком выходит тёмный силуэт фигуры, разлом закрывается, по полу расходится кольцо,
фигура возвращает цвета, и разгорается тёмная аура: последние кадры — кадры вступления
`dark_fire_*` (`breach_darkfire.py`), так что после появления идёт её повтор с кадра
`darkfire.LOOP_START`.

Кадры — на холсте FW×FH с точкой опоры (центр клетки) (FAX, FAY), как у `dark_fire_*`;
тень фигуры — обычная, пока фигура видна (исчезновение — до кадра VANISH, появление — с
кадра EMERGE). Появление начинается в кадре исчезновения IN_AT.

`py tools/breach_shadowstep.py [папка]` — по умолчанию assets/breach/effects:
shadowstep_out_<цвет><фигура>.png и shadowstep_in_<цвет><фигура>.png (кадры в ряд, все 12),
preview_shadowstep.gif, preview_shadowstep_frames.png.
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
import breach_darkfire as dark  # noqa: E402
import breach_firewave as fw  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")

FW, FH, FAX, FAY = dark.FW, dark.FH, dark.FAX, dark.FAY
SX, SY = FAX - art.AX, FAY - art.AY
FRAME_MS = 60
VOID = hexc("#05030a")  # нутро разлома
SWIRL = [hexc(c) for c in ["#3a1c66", "#5a2aa8"]]  # завихрения в разломе
LIP = hexc("#c9a2ff")  # кромка разлома — сиреневая
CONTOUR = dark.CONTOUR  # фиолетовый контур
HALO = dark.GLOW  # свечение вокруг разлома
SIL = np.array(hexc("#120a1e"), float)  # тёмный силуэт фигуры
GHOST = hexc("#2a1446")  # оставленный силуэт
GLINT = dark.GLINT
DASH_BODY, DASH_EDGE = hexc("#3a1c66"), LIP  # силуэт в рывке: светлее нутра разлома, иначе в нём не виден

OUT_LEN = 18
VANISH = 7  # кадр исчезновения, с которого фигуры (и её тени) на исходной клетке нет
IN_AT = 7  # кадр исчезновения, в котором начинается появление
IN_LEN = 13
EMERGE = 2  # кадр появления, с которого фигура (и тень) видна на целевой клетке
# ширина разлома по кадрам
RIFT_OUT = {4: 14, 5: 24, 6: 24, 7: 9}  # раскрывается за 2 кадра, схлопывается за 1
RIFT_IN = {0: 14, 1: 24, 2: 24, 3: 24, 4: 9}
SQUASH_OUT = {5: 0.55, 6: 0.2}  # ширина фигуры при втягивании (до 5 — 1, с VANISH — нет)
SQUASH_IN = {2: 0.2, 3: 0.6}  # ширина при выходе (с 4 — 1)


def piece_canvas(code):
    sprite = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "pieces", f"{code}.png")).convert("RGBA"))
    canvas = np.zeros((FH, FW, 4), np.uint8)
    canvas[SY : SY + sprite.shape[0], SX : SX + sprite.shape[1]] = sprite
    return canvas


def darken(img, sil, s, body_col=SIL, edge_col=CONTOUR):
    """Фигура темнеет до силуэта: тело — к тёмно-фиолетовому, контур — к фиолетовому."""
    if s <= 0:
        return img
    out = img.copy()
    rgb = out[..., :3].astype(float)
    edge = burn.boundary(sil)
    body = sil & ~edge
    rgb[body] = rgb[body] * (1 - s) + np.asarray(body_col, float) * s
    rgb[edge] = rgb[edge] * (1 - s) + np.asarray(edge_col, float) * s
    out[..., :3] = rgb.astype(np.uint8)
    return out


def squash(img, cx, base, sx):
    """Фигура сжата по ширине к центру (sx) и чуть вытянута вверх; основание на месте."""
    if sx >= 0.999:
        return img
    sy = 1 + 0.3 * (1 - sx)
    yy, xx = np.mgrid[0:FH, 0:FW]
    src_x = np.round(cx + (xx + 0.5 - cx) / max(sx, 1e-3) - 0.5).astype(int)
    src_y = np.round(base - (base - yy - 0.5) / sy - 0.5).astype(int)
    ok = (src_x >= 0) & (src_x < FW) & (src_y >= 0) & (src_y < FH)
    out = np.zeros_like(img)
    out[ok] = img[src_y[ok], src_x[ok]]
    return out


def rift(layer, cx, cy, h, w, rng, phase):
    """Варп-разлом: рваная вертикальная линза шириной w и высотой h."""
    if w <= 0:
        return
    yy, xx = np.mgrid[0:FH, 0:FW]
    t = (yy + 0.5 - cy) / (h / 2)
    jag = np.array([rng.uniform(-1.0, 1.0) for _ in range(FH)])[:, None]  # рваный край по строкам
    half = (w / 2) * np.clip(1 - t ** 2, 0, 1) ** 0.6 + jag * (w > 3)
    inside = (np.abs(xx + 0.5 - cx) <= half) & (np.abs(t) < 1)
    if not inside.any():
        return
    lip = burn.boundary(inside)
    ring = burn.neighbours_any(inside) & ~inside
    halo = burn.neighbours_any(ring) & ~inside & ~ring
    halo2 = burn.neighbours_any(halo | ring) & ~inside & ~ring & ~halo
    layer[halo2 & (layer[..., 3] == 0)] = (*HALO, 80)
    layer[halo & (layer[..., 3] == 0)] = (*HALO, 160)
    layer[ring] = (*CONTOUR, 255)
    layer[inside] = (*VOID, 255)
    # завихрения: косые полосы, бегущие по кругу
    sw = np.sin((xx - cx) * 0.9 + (yy - cy) * 0.35 + phase * 2.4) > 0.75
    layer[inside & sw & ~lip] = (*SWIRL[0], 255)
    layer[inside & sw & ~lip & (np.abs(xx + 0.5 - cx) < half * 0.4)] = (*SWIRL[1], 255)
    layer[lip] = (*LIP, 255)


def floor_ring(layer, r, alpha):
    """Кольцо энергии на клетке (эллипс 2:1)."""
    if alpha <= 0.02 or r < 2:
        return
    yy, xx = np.mgrid[0:FH, 0:FW]
    e = np.hypot((xx + 0.5 - FAX) / r, (yy + 0.5 - FAY) / (r * 0.5))
    ring = np.abs(e - 1) < 1.1 / r * 1.4
    inner = (np.abs(e - 1 + 1.6 / r) < 0.9 / r) & ~ring
    layer[inner & (layer[..., 3] == 0)] = (*HALO, int(150 * alpha))
    layer[ring] = (*CONTOUR, int(255 * alpha))


def speed_lines(layer, cx, cy, h, rng, out_dir, k, n=7):
    """Теневые полосы рывка: горизонтальные чёрточки по обе стороны от фигуры."""
    for _ in range(n):
        y = int(cy + rng.uniform(-0.45, 0.45) * h)
        side = rng.choice([-1, 1])
        start = rng.uniform(3, 8) + (6 * k if out_dir else 14 - 5 * k)
        ln = int(rng.uniform(5, 13))
        x0 = int(cx + side * start)
        for i in range(ln):
            x = x0 + side * i
            if 0 <= x < FW and 0 <= y < FH:
                col = GLINT[0] if i < 2 else (GLINT[1] if i < ln * 0.5 else CONTOUR)
                layer[y, x] = (*col, 255 if i < ln * 0.6 else 160)


def ghost(layer, sil, alpha, rng):
    """Оставленный силуэт: тёмно-фиолетовый с контуром, тает построчно и мерцает."""
    if alpha <= 0.02:
        return
    rows = rng.random(FH) < 0.25 + 0.75 * alpha  # часть строк уже растаяла
    edge = burn.boundary(sil)
    body = sil & ~edge & rows[:, None]
    e = edge & rows[:, None]
    layer[body] = (*GHOST, int(150 * alpha))
    layer[e] = (*CONTOUR, int(210 * alpha))


def over(dst, src):
    m = src[..., 3] > 0
    dst[m] = src[m]


def render(code, seed, bodies=None, aura=None):
    """Исчезновение, появление и кадры ауры; bodies(f) — своё тело фигуры в кадре f ауры (см. `dark.render`),
    aura — свои кадры ауры с фигурой (разметка как у `dark_fire`) вместо `dark.render`."""
    rng = np.random.default_rng(seed)
    canvas = piece_canvas(code)
    sil = canvas[..., 3] > 0
    ys, xs = np.nonzero(sil)
    cx = xs.mean() + 0.5
    base = ys.max() + 1
    top = ys.min()
    cy = (top + base) / 2 - 6
    fig_h = base - top
    h = 0.5 * fig_h + 12  # разлом ниже фигуры — компактная трещина посередине
    if aura is None:
        aura = dark.render(code, seed=5 + "PNBRQK".index(code[1]), bodies=bodies)  # кадры тёмной ауры этой фигуры
    shards = burn.Particles()
    out_frames = []
    for f in range(OUT_LEN):
        img = np.zeros((FH, FW, 4), np.uint8)
        # кольцо энергии сходится к фигуре, пока она заряжается
        if f < 6:
            floor_ring(img, 26 - 3.6 * f, min(1.0, (f + 1) / 2))
        # оставленный силуэт — с момента втягивания, тает за ~10 кадров
        if f >= 5:
            a = max(0.0, 1 - (f - 5) / 11) * (0.85 if f % 2 else 1.0)
            ghost(img, sil, a, rng)
        rift(img, cx, cy, h, RIFT_OUT.get(f, 0), rng, f)
        if f < 5:
            base_img = aura[dark.LOOP_START + f]
            fig = darken(base_img, sil, min(1.0, f / 3.5))
            if f >= 3:  # аура стягивается в фигуру: остаётся только сама фигура
                fig = np.where(sil[..., None], fig, 0).astype(np.uint8)
            over(img, fig)
        elif f < VANISH:
            over(img, squash(darken(canvas, sil, 1.0, DASH_BODY, DASH_EDGE), cx, base, SQUASH_OUT[f]))
        if 5 <= f <= 8:
            speed_lines(img, cx, cy, fig_h, rng, True, f - 5)
        # осколки летят в разлом
        if 2 <= f <= 7:
            for _ in range(5):
                a = rng.uniform(0, 2 * math.pi)
                r = rng.uniform(14, 24)
                x, y = cx + math.cos(a) * r, cy + math.sin(a) * r * 0.8
                shards.items.append(dict(x=x, y=y, vx=(cx - x) * 0.3, vy=(cy - y) * 0.3, drag=1.0,
                                         life=3, age=0))
        for p in shards.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                img[y, x] = (*GLINT[min(p["age"], 2)], 255)
        shards.step()
        # вспышка, когда разлом схлопывается
        if f == 8:
            for dx in range(-3, 4):
                x, y = int(cx) + dx, int(cy)
                if 0 <= x < FW:
                    img[y, x] = (*(GLINT[0] if abs(dx) < 2 else GLINT[1]), 255)
            for dy in range(-6, 7):
                y = int(cy) + dy
                if 0 <= y < FH:
                    img[y, int(cx)] = (*(GLINT[0] if abs(dy) < 3 else GLINT[1]), 255)
        out_frames.append(img)

    sparks = burn.Particles()
    in_frames = []
    for f in range(IN_LEN):
        img = np.zeros((FH, FW, 4), np.uint8)
        if f >= 4:  # кольцо расходится по клетке
            k = f - 4
            floor_ring(img, 8 + 3 * k, max(0.0, 1 - k / 7))
        rift(img, cx, cy, h, RIFT_IN.get(f, 0), rng, f + 3)
        if EMERGE <= f < 5:
            sx = SQUASH_IN.get(f, 1.0)
            over(img, squash(darken(canvas, sil, 1.0, DASH_BODY, DASH_EDGE), cx, base, sx))
        elif f >= 5:
            k = f - 5  # кадры вступления ауры: фигура светлеет обратно, аура разгорается
            base_img = aura[min(k, dark.LOOP_START - 1)]
            over(img, darken(base_img, sil, max(0.0, 1 - k / 4)))
        if 2 <= f <= 4:
            speed_lines(img, cx, cy, fig_h, rng, False, f - 2)
        if 1 <= f <= 4:  # искры из разлома
            for _ in range(4):
                sparks.items.append(dict(x=cx + rng.uniform(-2, 2), y=cy + rng.uniform(-h / 3, h / 3),
                                         vx=rng.uniform(-2.2, 2.2), vy=rng.uniform(-1.2, 0.6), drag=0.85,
                                         life=int(rng.integers(3, 6)), age=0))
        for p in sparks.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                img[y, x] = (*GLINT[min(p["age"], 2)], 255)
        sparks.step()
        in_frames.append(img)
    # последний кадр появления — ровно кадр вступления ауры, с которого продолжится её повтор
    in_frames[-1] = aura[IN_LEN - 1 - 5].copy() if IN_LEN - 1 - 5 < dark.LOOP_START else in_frames[-1]
    return out_frames, in_frames, aura


# ---------------------------------------------------------------- превью на доске
def cell_xy(sq, center, size):
    ox = size[0] // 2 - art.TW // 2
    oy = size[1] // 2 - art.TH // 2 + 10
    i, j = sq[0] - center[0], (7 - sq[1]) - (7 - center[1])
    x, y = art.iso(i, j)
    return ox + x + art.TW // 2, oy + y + art.TH // 2  # центр верхней грани клетки


def scene(code, a, b, others, frames, size=(300, 236)):
    """Кадры: фигура в ауре на a, прыжок, появление на b, снова аура."""
    out_f, in_f, aura = frames
    center = ((a[0] + b[0]) // 2, (a[1] + b[1]) // 2)
    tiles, _, _, _ = fw.board_scene({}, center, size)
    sprites = fw.load_sprites()
    loop = list(range(dark.LOOP_START, dark.FRAMES))
    timeline = []  # (что на a, что на b): ("aura", k) / ("out", k) / ("in", k) / None
    for k in range(14):
        timeline.append((("aura", loop[k % len(loop)]), None))
    for k in range(max(OUT_LEN, IN_AT + IN_LEN)):
        ea = ("out", k) if k < OUT_LEN else None
        eb = ("in", k - IN_AT) if 0 <= k - IN_AT < IN_LEN else None
        if eb is None and k - IN_AT >= IN_LEN:
            eb = ("aura", loop[(k - IN_AT - IN_LEN) % len(loop)])
        timeline.append((ea, eb))
    for k in range(20):
        timeline.append((None, ("aura", loop[(k + 6) % len(loop)])))
    _, shadow = sprites[code]
    result = []
    for ea, eb in timeline:
        im = tiles.copy()
        items = []
        for sq, c in others.items():
            items.append((sq[0] + 7 - sq[1], "piece", sq, c))
        for sq, e in ((a, ea), (b, eb)):
            if e is None:
                continue
            kind, k = e
            visible = kind == "aura" or (kind == "out" and k < VANISH) or (kind == "in" and k >= EMERGE)
            if visible:
                x, y = cell_xy(sq, center, size)
                im.alpha_composite(shadow, (x - art.AX, y - art.AY))
            img = aura[k] if kind == "aura" else (out_f[k] if kind == "out" else in_f[k])
            items.append((sq[0] + 7 - sq[1], "fx", sq, img))
        for sq, c in others.items():
            x, y = cell_xy(sq, center, size)
            im.alpha_composite(sprites[c][1], (x - art.AX, y - art.AY))
        for _, kind, sq, obj in sorted(items, key=lambda t: t[0]):
            x, y = cell_xy(sq, center, size)
            if kind == "piece":
                im.alpha_composite(sprites[obj][0], (x - art.AX, y - art.AY))
            else:
                im.alpha_composite(Image.fromarray(obj, "RGBA"), (x - FAX, y - FAY))
        result.append(im.convert("RGB"))
    return result


def save_preview(all_frames, out, scale=2):
    scenes = [scene("wN", (3, 2), (5, 4), {(4, 3): "bP", (6, 5): "bR", (2, 3): "wP"}, all_frames["wN"]),
              scene("bQ", (5, 5), (3, 3), {(4, 4): "wP", (2, 2): "wN", (6, 4): "bP"}, all_frames["bQ"])]
    frames = []
    for s1, s2 in zip(*scenes):
        s1, s2 = s1.crop((20, 0, 280, 236)), s2.crop((20, 0, 280, 236))
        im = Image.new("RGB", (2 * s1.width + 4, s1.height), (27, 30, 39))
        im.paste(s1, (0, 0))
        im.paste(s2, (s1.width + 4, 0))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 5))
    for i, k in enumerate((5, 18, 20, 23, 28)):
        allpix.paste(frames[k], (0, i * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_shadowstep.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    # лист кадров коня: исчезновение и появление
    out_f, in_f, _ = all_frames["wN"]
    cols = max(OUT_LEN, IN_LEN)
    sheet = Image.new("RGBA", (cols * (FW + 2), 2 * (FH + 2)), (40, 44, 56, 255))
    for r, seq in enumerate((out_f, in_f)):
        for k, fr in enumerate(seq):
            sheet.alpha_composite(Image.fromarray(fr, "RGBA"), (k * (FW + 2), r * (FH + 2)))
    sheet.save(os.path.join(out, "preview_shadowstep_frames.png"))


def main():
    os.makedirs(OUT, exist_ok=True)
    all_frames = {}
    for code in (c + k for c in "wb" for k in "PNBRQK"):
        out_f, in_f, aura = render(code, seed=31 + "PNBRQK".index(code[1]))
        all_frames[code] = (out_f, in_f, aura)
        Image.fromarray(np.concatenate(out_f, axis=1), "RGBA").save(os.path.join(OUT, f"shadowstep_out_{code}.png"))
        Image.fromarray(np.concatenate(in_f, axis=1), "RGBA").save(os.path.join(OUT, f"shadowstep_in_{code}.png"))
        print(code, flush=True)
    save_preview(all_frames, OUT)
    print("done", OUT)


if __name__ == "__main__":
    main()
