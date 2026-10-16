"""Лавовые трещины на фигуре перед огненной волной — в стиле Into the Breach.

Из точки в теле фигуры по спрайту расходятся трещины. В остриё трещины —
бело-жёлтое, дальше оранжевая лава, по бокам — тёмная корка. Потом трещины
разгораются: лава светлеет, из неё бьют язычки пламени и искры, фигура
раскаляется. Затем всё гаснет, и фигура становится прежней. Огненная волна
(`breach_firewave.py`) начинается в кадре WAVE_AT, пока трещины гаснут.

Кадры — замена спрайта фигуры: холст 64×84 с той же точкой опоры (art.AX, art.AY),
тень — обычная тень фигуры. Пламя и искры не выходят за холст.

`py tools/breach_lava.py [папка]` — по умолчанию assets/breach/effects:
lava_crack_<цвет><фигура>.png (кадры в ряд, все 12 фигур), preview_lava_crack.gif
(пешки на доске, потом волна), preview_lava_crack_frames.png.
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
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "effects")
PIECES = os.path.join(art.ROOT, "assets", "breach", "pieces")

FIRE, OUTLINE = burn.FIRE, burn.OUTLINE
CRUST = np.array(hexc("#2a0d08"), float)
FRAME_MS = fw.FRAME_MS  # 60 мс, как у волны
SPREAD = 13  # к этому кадру трещины доросли до краёв
IGNITE, PEAK = 17, 20  # трещины разгораются с IGNITE, ярче всего в PEAK
FADE_END = 27  # к этому кадру трещины погасли
FRAMES = FADE_END + 1
WAVE_AT = 21  # кадр трещин, в котором начинается огненная волна (её кадр 0)


def load(code):
    return np.array(Image.open(os.path.join(PIECES, f"{code}.png")).convert("RGBA"))


def crack_times(sprite, rng):
    """Дерево трещин по телу фигуры: для каждого пикселя трещины — кадр, в котором до
    него дошла трещина, и её колено (0 — ствол, 1+ — ветви)."""
    alpha = sprite[..., 3] > 0
    inner = alpha & ~burn.boundary(alpha)
    inner &= ~burn.boundary(inner)  # трещина не доходит до контура на пиксель
    ys, xs = np.nonzero(inner)
    h, w = inner.shape
    # начало — чуть ниже середины тела: там у всех фигур самая широкая часть
    ox = xs.mean()
    oy = ys.min() + 0.62 * (ys.max() - ys.min())
    cand = np.argmin(np.hypot(xs - ox, ys - oy))
    ox, oy = xs[cand] + 0.5, ys[cand] + 0.5
    reach = math.hypot(xs.max() - xs.min(), ys.max() - ys.min())
    speed = reach * 0.62 / SPREAD  # пикселей за кадр
    times = np.full((h, w), np.inf)
    gens = np.zeros((h, w), int)

    def grow(x, y, ang, length, gen, t):
        since_branch = 0
        for _ in range(int(length)):
            ang += rng.normal(0, 0.12)
            if rng.random() < 0.16:  # излом
                ang += rng.choice([-1, 1]) * rng.uniform(0.35, 0.7)
            x += math.cos(ang)
            y += math.sin(ang) * 0.9
            t += 1 / speed
            ix, iy = int(x), int(y)
            if not (0 <= ix < w and 0 <= iy < h and inner[iy, ix]):
                return
            if t < times[iy, ix]:
                times[iy, ix], gens[iy, ix] = t, gen
            since_branch += 1
            if gen < 1 and since_branch > 4 and rng.random() < 0.2:
                since_branch = 0
                side = rng.choice([-1, 1])
                grow(x, y, ang + side * rng.uniform(0.6, 1.1), length * 0.5, gen + 1, t)

    # стволы: вверх, вниз-влево, вниз-вправо и короткий в сторону
    starts = [(-math.pi / 2, 1.0), (math.pi * 0.8, 0.8), (math.pi * 0.2, 0.8), (rng.choice([0.0, math.pi]), 0.5)]
    for ang, k in starts:
        grow(ox, oy, ang + rng.normal(0, 0.25), reach * k, 0, 0.0)
    # очаг: 2×2 пикселя, с которых всё начинается
    for dy in (0, 1):
        for dx in (-1, 0):
            iy, ix = int(oy) - 1 + dy, int(ox) + dx
            if inner[iy, ix]:
                times[iy, ix], gens[iy, ix] = 0.0, 0
    return times, gens, inner


def heat(f):
    """Насколько раскалена фигура: 0 — холодная, 1 — пик."""
    if f < IGNITE:
        return 0.0
    if f <= PEAK:
        return (f - IGNITE + 1) / (PEAK - IGNITE + 1)
    return max(0.0, 1 - (f - PEAK) / (FADE_END - PEAK))


def flame_tongues(points, rng, height, size):
    """Язычки пламени из трещин: широкие у основания, кончик гнётся и дрожит."""
    h, w = size
    fl = np.zeros((h, w), bool)
    for bx, by, k in points:
        ht = height * k * rng.uniform(0.8, 1.15)
        wt = 1.6 + 1.2 * k
        lean = rng.uniform(-2.0, 2.0)
        for dy in range(int(ht) + 1):
            p = dy / max(ht, 1)
            half = wt * (1 - p) ** 0.8
            cx = bx + lean * p * p
            y = int(by) - dy
            if 0 <= y < h and half > 0.3:
                fl[y, max(0, int(round(cx - half))) : min(w, int(round(cx + half)) + 1)] = True
    return fl


def paint_fire(img, fl):
    """Раскраска пламени как у волны: светлое нутро, оранжевая и тёмная кайма, контур."""
    if not fl.any():
        return
    edge = burn.boundary(fl)
    inner_edge = burn.boundary(fl & ~edge)
    c = np.zeros(img.shape, np.uint8)
    c[fl] = (*FIRE[1], 255)
    c[inner_edge] = (*FIRE[3], 255)
    c[edge] = (*FIRE[5], 255)
    for x in range(fl.shape[1]):
        col = np.nonzero(fl[:, x])[0]
        if len(col) > 3:
            c[col[0], x, :3] = FIRE[7]
    ring = burn.neighbours_any(fl) & ~fl
    img[ring] = (*OUTLINE, 255)
    img[fl] = c[fl]


def render(code, seed):
    rng = np.random.default_rng(seed)
    sprite = load(code)
    times, gens, inner = crack_times(sprite, rng)
    h, w = inner.shape
    alpha = sprite[..., 3] > 0
    shimmer = burn.value_noise(h, w, 3, rng)
    cracks = np.isfinite(times)
    cpts = np.argwhere(cracks)
    # язычки пламени — из верхних точек трещин: слева, посередине и справа
    top = cpts[cpts[:, 0] <= np.percentile(cpts[:, 0], 45)]
    tongue_pts = []
    for part, k in zip(np.array_split(top[np.argsort(top[:, 1])], 3), (0.75, 1.0, 0.7)):
        if len(part):
            y, x = part[np.argmin(part[:, 0])]
            tongue_pts.append((x + 0.5, y + 1, k))
    embers = burn.Particles()
    body = alpha & ~burn.boundary(alpha)
    frames = []
    for f in range(FRAMES):
        img = sprite.copy()
        rgb = img[..., :3].astype(float)
        hot = heat(f)
        # гаснут: 0 — ещё горят, 1 — погасли; последние кадры растворяются в фигуре
        cool = 0.0 if f <= PEAK else (f - PEAK) / (FADE_END - PEAK)
        shown = cracks & (times <= f)
        if f >= FADE_END:
            shown[:] = False
        # корка вокруг трещин, отсвет лавы рядом, на пике — раскалённое тело
        lip = burn.neighbours_any(shown) & inner & ~shown
        glow = burn.neighbours_any(lip | shown) & alpha & ~lip & ~shown
        crust_k = 0.62 * (1 - cool) ** 1.5
        rgb[lip] = rgb[lip] * (1 - crust_k) + CRUST * crust_k
        glow_k = 0.16 + 0.3 * hot if shown.any() else 0.0
        rgb[glow] = rgb[glow] * (1 - glow_k) + np.array(FIRE[4], float) * glow_k
        if hot > 0:
            rgb[body] = rgb[body] * (1 - 0.16 * hot) + np.array(FIRE[5], float) * 0.16 * hot
        img[..., :3] = np.clip(rgb, 0, 255).astype(np.uint8)
        # сама лава: остриё светлое, старые участки переливаются оранжевым
        age = f - times
        for y, x in np.argwhere(shown):
            a = age[y, x]
            if cool > 0:
                stage = int(cool * 5)
                if stage >= 3:  # остывшая — тёмный шов, растворяется в фигуре
                    k = 0.7 if stage == 3 else 0.35
                    img[y, x, :3] = (sprite[y, x, :3] * (1 - k) + CRUST * k).astype(np.uint8)
                    continue
                col = FIRE[[2, 4, 6][stage]]
            elif hot > 0:
                col = FIRE[[3, 2, 1, 0][min(int(hot * 4), 3)]]
            elif a < 1:
                col = FIRE[0]
            elif a < 2.5:
                col = FIRE[2]
            else:
                pulse = shimmer[y, x] + 0.25 * math.sin(f * 0.9 + x * 0.7 + y * 0.4)
                col = FIRE[3] if pulse > 0.62 or gens[y, x] == 0 and pulse > 0.4 else FIRE[4]
            img[y, x, :3] = col
        # разгораясь, трещины расширяются: лава переливается через корку
        if hot > 0 and cool == 0:
            img[lip, :3] = FIRE[[6, 5, 5, 3][min(int(hot * 4), 3)]]
        elif 0 < cool < 0.3:
            img[lip, :3] = FIRE[7]
        # язычки пламени из трещин
        if IGNITE + 1 <= f <= FADE_END - 2:
            k = hot if f <= PEAK else hot ** 0.7
            fl = flame_tongues(tongue_pts, rng, 3.0 + 8.0 * k, (h, w)) if k > 0.15 else np.zeros((h, w), bool)
            paint_fire(img, fl)
        # искры: немного, пока трещины растут, и сноп на пике
        n_new = 1 if (3 <= f < IGNITE and f % 2 == 0) else (4 if IGNITE <= f <= PEAK + 1 else 0)
        live = np.argwhere(shown)
        for _ in range(n_new if len(live) else 0):
            y, x = live[rng.integers(len(live))]
            embers.items.append(dict(x=x + 0.5, y=y - 1.0, vx=rng.uniform(-0.9, 0.9), vy=rng.uniform(-2.6, -1.2),
                                     drag=0.9, life=int(rng.integers(4, 8)), age=0, wob=0.25, ph=rng.uniform(0, 6)))
        for p in embers.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < w and 0 <= y < h:
                life = p["age"] / p["life"]
                img[y, x] = (*FIRE[[1, 3, 5, 7][min(int(life * 4), 3)]], 255)
        embers.step()
        frames.append(img)
    return frames


# ---------------------------------------------------------------- превью на доске
def scene_frames(lava, code, center, position, under, over):
    """Кадры превью: трещины на фигуре в клетке center, с WAVE_AT — огненная волна."""
    tiles, shadows, _, at = fw.board_scene(position, center)
    sprites = fw.load_sprites()
    rest = {sq: c for sq, c in position.items() if sq != center}
    # фигура в клетке center стоит в общем порядке рисования: задние фигуры — до неё,
    # передние — после
    back = {sq: c for sq, c in rest.items() if sq[0] + 7 - sq[1] < center[0] + 7 - center[1]}
    front = {sq: c for sq, c in rest.items() if sq not in back}
    back_layer = fw.board_scene(back, center)[2]
    front_layer = fw.board_scene(front, center)[2]
    cx, cy = at[0] + fw.CX - art.TW // 2, at[1] + fw.CY - art.TH // 2  # угол клетки
    piece_at = (cx + art.TW // 2 - art.AX, cy + art.TH // 2 - art.AY)
    n_wave = under.shape[1] // fw.FW
    total = max(FRAMES, WAVE_AT + n_wave)
    out = []
    for k in range(-5, total + 6):
        f = min(max(k, 0), total - 1)
        im = tiles.copy()
        im.alpha_composite(shadows)
        wf = k - WAVE_AT
        if 0 <= wf < n_wave:
            im.alpha_composite(Image.fromarray(under[:, wf * fw.FW : (wf + 1) * fw.FW]), at)
        im.alpha_composite(back_layer)
        body = Image.fromarray(lava[f], "RGBA") if 0 <= k < FRAMES else sprites[code][0]
        im.alpha_composite(body, piece_at)
        im.alpha_composite(front_layer)
        if 0 <= wf < n_wave:
            im.alpha_composite(Image.fromarray(over[:, wf * fw.FW : (wf + 1) * fw.FW]), at)
        out.append(im.convert("RGB"))
    return out


def save_preview(all_frames, out, scale=2):
    under = np.array(Image.open(os.path.join(out, "fire_wave_under.png")).convert("RGBA"))
    over = np.array(Image.open(os.path.join(out, "fire_wave_over.png")).convert("RGBA"))
    scenes = []
    for code, pos in (("wP", {(4, 3): "wP", (3, 4): "bP", (5, 4): "bN", (5, 3): "bP", (3, 2): "wP", (6, 5): "bR"}),
                      ("bP", {(4, 3): "bP", (3, 2): "wP", (4, 2): "wB", (5, 3): "wP", (3, 4): "bP", (2, 1): "wR"})):
        scenes.append(scene_frames(all_frames[code], code, (4, 3), pos, under, over))
    w, h = scenes[0][0].size
    frames = []
    for a, b in zip(*scenes):
        im = Image.new("RGB", (2 * w + 4, h), (27, 30, 39))
        im.paste(a, (0, 0))
        im.paste(b, (w + 4, 0))
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    allpix = Image.new("RGB", (frames[0].width, frames[0].height * 8))
    for n, k in enumerate(range(0, len(frames), max(1, len(frames) // 8))):
        if n < 8:
            allpix.paste(frames[k], (0, n * frames[0].height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in frames]
    gif[0].save(os.path.join(out, "preview_lava_crack.gif"), save_all=True, append_images=gif[1:],
                duration=FRAME_MS, loop=0, disposal=1)
    # лист кадров белой пешки крупно: каждый второй кадр трещин и начало волны
    crop = (56, 50, 244, 200)
    picks = list(range(0, len(scenes[0]) - 6, 3))[:20]
    cw, chh = crop[2] - crop[0], crop[3] - crop[1]
    sheet = Image.new("RGB", (5 * (cw + 4) + 4, 4 * (chh + 4) + 4), (27, 30, 39))
    for n, k in enumerate(picks):
        sheet.paste(scenes[0][k].crop(crop), (4 + (n % 5) * (cw + 4), 4 + (n // 5) * (chh + 4)))
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(
        os.path.join(out, "preview_lava_crack_frames.png"))


def main():
    os.makedirs(OUT, exist_ok=True)
    all_frames = {}
    for n, code in enumerate(c + k for c in "wb" for k in "PNBRQK"):
        frames = render(code, seed=40 + "PNBRQK".index(code[1]))  # у белых и чёрных одинаковые трещины
        all_frames[code] = frames
        Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(os.path.join(OUT, f"lava_crack_{code}.png"))
    save_preview(all_frames, OUT)
    print("done", OUT)


if __name__ == "__main__":
    main()
