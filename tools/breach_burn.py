"""Анимация сжигания карты в стиле Into the Breach.

Карта прогорает неровной кромкой от точки поджога внизу: перед кромкой — подпалина,
на кромке — бело-жёлтый жар с оранжевой и тёмно-красной каймой, за ней — уголь с
тлеющими искрами, который осыпается пеплом. Над кромкой — языки пламени с чёрным
контуром (как взрывы ITB), вверх летят искры и клубы дыма.

`py tools/breach_burn.py [папка]` — по умолчанию assets/breach/cards/burn:
burn_strip.png (кадры обычной карты белых в ряд, прозрачный фон), burn_strip_b.png,
burn_strip_sacrifice.png, burn_strip_victory.png (то же для обычной карты чёрных, карты
жертвы и карты победы), burn_strip_fire_wave_w/b.png и burn_strip_swap_w/b.png (огненная
волна и обмен — обычная карта с картинкой; так же burn_strip_bless_w/b.png — благословение
слона), burn_strip_swap_victory.png (обмен в игре — карта победы с той же картинкой),
burn_strip_funding_sacrifice.png («Дополнительное финансирование» — карта жертвы с
договором в крови), burn_strip_heal_victory.png («Исцеление» — карта победы со слоном в
крыльях), preview_burn.gif, preview_burn_frames.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # breach_art читает папку вывода из argv
import breach_art as art  # noqa: E402
import breach_cards as bc  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "cards", "burn")

CW, CH = bc.CW, bc.CH
PAD_X, PAD_TOP, PAD_BOTTOM = 12, 36, 4
FW, FH = CW + 2 * PAD_X, CH + PAD_TOP + PAD_BOTTOM  # кадр 74×110
FRAMES = 32
IGNITE = 3  # кадр, с которого карта горит
BURN_FRAMES = 20  # за столько кадров кромка проходит всю карту
FPS_MS = 70

# от самого горячего к остывающему — цвета огня ITB
FIRE = [hexc(c) for c in ["#fffbf0", "#fff8ce", "#ffeba1", "#ffbc03", "#f1a610", "#e47a1b", "#d3762d", "#8f1602"]]
CHAR = [hexc(c) for c in ["#1c1210", "#2a1a17", "#3d2520"]]
SCORCH = np.array(hexc("#4a2414"), float)
SMOKE = [hexc(c) for c in ["#eaeee9", "#bfc1be", "#949494"]]
ASH = [hexc(c) for c in ["#504842", "#2a242a"]]
OUTLINE = hexc("#020201")
SMOKE_ALPHA = 179

SCORCH_T, GLOW_T, CHAR_T = 0.06, 0.055, 0.11


def value_noise(h, w, cell, rng):
    gh, gw = h // cell + 2, w // cell + 2
    g = rng.random((gh, gw))
    y, x = np.mgrid[0:h, 0:w] / cell
    y0, x0 = y.astype(int), x.astype(int)
    fy, fx = y - y0, x - x0
    fy, fx = fy * fy * (3 - 2 * fy), fx * fx * (3 - 2 * fx)
    a = g[y0, x0] * (1 - fx) + g[y0, x0 + 1] * fx
    b = g[y0 + 1, x0] * (1 - fx) + g[y0 + 1, x0 + 1] * fx
    return a * (1 - fy) + b * fy


def burn_times(seed=7):
    """Момент прогорания каждого пикселя карты, 0…1."""
    rng = np.random.default_rng(seed)
    y, x = np.mgrid[0:CH, 0:CW]
    ix, iy = CW * 0.36, CH - 5
    radial = np.hypot((x - ix) / CW, (y - iy) / CH * 0.85)
    upward = (CH - 1 - y) / (CH - 1)
    noise = 0.6 * value_noise(CH, CW, 6, rng) + 0.4 * value_noise(CH, CW, 3, rng)
    tb = 0.5 * radial / radial.max() + 0.5 * upward + 0.24 * (noise - 0.5)
    return (tb - tb.min()) / (tb.max() - tb.min()), (int(ix), int(iy))


def _shifts(mask):
    p = np.pad(mask, 1)
    return p[:-2, 1:-1], p[2:, 1:-1], p[1:-1, :-2], p[1:-1, 2:]


def neighbours_any(mask):
    up, down, left, right = _shifts(mask)
    return up | down | left | right


def boundary(mask):
    """Пиксели маски, у которых есть сосед вне маски (край массива — тоже «вне»)."""
    up, down, left, right = _shifts(mask)
    return mask & ~(up & down & left & right)


class Particles:
    def __init__(self):
        self.items = []

    def step(self):
        alive = []
        for p in self.items:
            p["age"] += 1
            p["x"] += p["vx"] + p.get("wob", 0) * math.sin(p["age"] * 0.9 + p.get("ph", 0))
            p["y"] += p["vy"]
            p["vy"] *= p.get("drag", 1.0)
            if p["age"] < p["life"]:
                alive.append(p)
        self.items = alive


def card_layer(card, tb, t, rng):
    """Карта в момент t: цвета и маска непрозрачного, плюс маска горящей кромки."""
    img = card.astype(float).copy()
    solid = card[..., 3] > 0
    dt = t - tb
    scorch = solid & (dt >= -SCORCH_T) & (dt < 0)
    burning = solid & (dt >= 0) & (dt < GLOW_T)
    char = solid & (dt >= GLOW_T) & (dt < GLOW_T + CHAR_T)
    gone = solid & (dt >= GLOW_T + CHAR_T)
    # подпалина: две ступени потемнения к кромке
    k = np.where(dt > -SCORCH_T / 2, 0.72, 0.4)
    img[scorch, :3] = img[scorch, :3] * (1 - k[scorch, None]) + SCORCH * k[scorch, None]
    # жар: у самой кромки светлее, к углю — темнее
    u = (dt / GLOW_T).clip(0, 0.999)
    idx = np.array([1, 2, 3, 5, 7])[(u * 5).astype(int)]
    for i in np.unique(idx[burning]):
        img[burning & (idx == i), :3] = FIRE[i]
    # уголь с тлеющими точками, тлеет всё реже
    c = ((dt - GLOW_T) / CHAR_T).clip(0, 0.999)
    cidx = 2 - (c * 3).astype(int)
    for i in range(3):
        img[char & (cidx == i), :3] = CHAR[i]
    sparks = char & (rng.random(char.shape) < 0.22 * (1 - c))
    img[sparks, :3] = FIRE[5]
    img[sparks & (rng.random(char.shape) < 0.3), :3] = FIRE[3]
    alive = solid & ~gone
    img[~alive, 3] = 0
    img[boundary(alive), :3] = OUTLINE
    return img.astype(np.uint8), alive, burning, gone


def flame_layer(burning, f, rng):
    """Языки пламени над горящей кромкой: маска и цвета в координатах кадра."""
    h_img = np.zeros((FH, FW, 4), np.uint8)
    fl = np.zeros((FH, FW), bool)
    cols = np.nonzero(burning.any(axis=0))[0]
    if len(cols) == 0:
        return h_img
    base = np.full(FW, -1)
    count = np.zeros(FW)
    for x in cols:
        ys = np.nonzero(burning[:, x])[0]
        base[x + PAD_X] = ys.min() + PAD_TOP
        count[x + PAD_X] = len(ys)
    height = np.where(base >= 0, np.clip(count * 0.9, 1, 4), 0)
    # языки: треугольные профили со случайными высотами, не ближе 4 px друг к другу
    order = rng.permutation(cols + PAD_X)
    taken = []
    for cx in order:
        if any(abs(cx - t) < 4 for t in taken):
            continue
        taken.append(cx)
        ht = rng.uniform(5, 12) * min(1.0, count[cx] / 3 + 0.4)
        wt = rng.uniform(2.5, 4.5)
        for x in range(int(cx - wt), int(cx + wt) + 1):
            if 0 <= x < FW and base[x] >= 0:
                height[x] = max(height[x], ht * (1 - abs(x - cx) / (wt + 0.5)))
    for x in range(FW):
        if base[x] < 0 or height[x] <= 0:
            continue
        top = int(round(base[x] - height[x]))
        fl[max(top, 0) : base[x] + 1, x] = True
    # слегка «прожечь» вниз, чтобы огонь закрывал кромку
    for x in range(FW):
        if base[x] >= 0:
            fl[base[x] : min(base[x] + 2, FH), x] = True
    edge = boundary(fl)
    inner_edge = boundary(fl & ~edge)
    h_img[fl] = (*FIRE[1], 255)
    h_img[inner_edge] = (*FIRE[3], 255)
    h_img[edge] = (*FIRE[5], 255)
    # кончики языков темнеют
    for x in range(FW):
        col = np.nonzero(fl[:, x])[0]
        if len(col) > 4:
            h_img[col[0], x, :3] = FIRE[7]
            h_img[col[1], x, :3] = FIRE[6]
    ring = neighbours_any(fl) & ~fl
    h_img[ring] = (*OUTLINE, 255)
    return h_img


def draw_puff(img, p):
    """Комковатый клуб дыма ITB: три круга, светлый верх-лево, тёмный низ-право, контур."""
    r = p["r"]
    if r < 0.8:
        return
    cx, cy = p["x"], p["y"]
    y, x = np.mgrid[0:FH, 0:FW]
    px, py = x + 0.5, (y + 0.5)
    body = np.zeros((FH, FW), bool)
    for ox, oy, k in ((-0.45, 0.1, 0.72), (0.45, -0.05, 0.68), (0.0, -0.45, 0.62)):
        body |= np.hypot(px - (cx + ox * r), (py - (cy + oy * r)) * 1.15) <= r * k
    if not body.any():
        return
    lit = body & (np.hypot(px - (cx - r * 0.35), (py - (cy - r * 0.45)) * 1.15) <= r * 0.5)
    shade = body & ~np.roll(np.roll(body, -2, 0), -1, 1)
    layer = np.zeros_like(img)
    layer[body] = (*SMOKE[1], SMOKE_ALPHA)
    layer[shade] = (*SMOKE[2], SMOKE_ALPHA)
    layer[lit & ~shade] = (*SMOKE[0], SMOKE_ALPHA)
    ring = neighbours_any(body) & ~body
    layer[ring] = (*OUTLINE, SMOKE_ALPHA)
    a = layer[..., 3:4] / 255.0
    img[..., :3] = (img[..., :3] * (1 - a) + layer[..., :3] * a).astype(np.uint8)
    img[..., 3] = np.maximum(img[..., 3], layer[..., 3])

def render(face, seed=7):
    rng = np.random.default_rng(seed)
    tb, (ix, iy) = burn_times(seed)
    total_t = 1.0 + GLOW_T + CHAR_T + SCORCH_T
    embers, smoke, ash = Particles(), Particles(), Particles()
    frames = []
    for f in range(FRAMES):
        frame = np.zeros((FH, FW, 4), np.uint8)
        t = (f - IGNITE) / BURN_FRAMES * total_t - SCORCH_T if f >= IGNITE else -1.0
        card, alive, burning, gone = card_layer(face, tb, t, rng)
        frame[PAD_TOP : PAD_TOP + CH, PAD_X : PAD_X + CW] = card

        # искра поджога
        if f in (IGNITE - 1, IGNITE):
            r = 1 if f == IGNITE - 1 else 3
            y, x = np.mgrid[0:FH, 0:FW]
            d = np.abs(x - (ix + PAD_X)) + np.abs(y - (iy + PAD_TOP))
            star = d <= r
            if f == IGNITE:
                star |= ((x == ix + PAD_X) & (np.abs(y - iy - PAD_TOP) <= 5)) | (
                    (y == iy + PAD_TOP) & (np.abs(x - ix - PAD_X) <= 5)
                )
            frame[neighbours_any(star) & ~star] = (*OUTLINE, 255)
            frame[star] = (*FIRE[0], 255)

        # частицы рождаются на кромке и там, где уголь осыпается
        bys, bxs = np.nonzero(burning)
        if len(bys):
            for _ in range(min(4, len(bys) // 6 + 1)):
                k = rng.integers(len(bys))
                embers.items.append(
                    dict(x=bxs[k] + PAD_X + 0.5, y=bys[k] + PAD_TOP - 1.0, vx=rng.uniform(-0.5, 0.5),
                         vy=rng.uniform(-5.5, -3.6), drag=0.88, life=int(rng.integers(6, 11)), age=0,
                         wob=0.35, ph=rng.uniform(0, 6))
                )
            if f % 2 == 0:
                # дым — от верха кромки, быстрее огня, потом замедляется
                top = bys.min()
                xs_top = bxs[bys <= top + 3]
                smoke.items.append(
                    dict(x=rng.choice(xs_top) + PAD_X + rng.uniform(-2, 2), y=top + PAD_TOP - 9.0,
                         vx=rng.uniform(-0.4, 0.4), vy=rng.uniform(-4.2, -3.2), drag=0.86,
                         life=int(rng.integers(10, 15)), age=0, r0=rng.uniform(2.0, 3.0), wob=0.3,
                         ph=rng.uniform(0, 6))
                )
        if f >= IGNITE:
            prev_t = (f - 1 - IGNITE) / BURN_FRAMES * total_t - SCORCH_T
            fresh = (face[..., 3] > 0) & (t - tb >= GLOW_T + CHAR_T) & (prev_t - tb < GLOW_T + CHAR_T)
            fys, fxs = np.nonzero(fresh)
            for k in range(len(fys)):
                if rng.random() < 0.008:
                    # пепел осыпается вниз, покачиваясь
                    ash.items.append(
                        dict(x=fxs[k] + PAD_X + 0.5, y=fys[k] + PAD_TOP + 0.5, vx=rng.uniform(-0.3, 0.3),
                             vy=rng.uniform(0.4, 1.0), life=int(rng.integers(6, 11)), age=0, wob=0.6,
                             ph=rng.uniform(0, 6), big=rng.random() < 0.6)
                    )

        # дым позади огня
        for p in smoke.items:
            life = p["age"] / p["life"]
            grow = p["r0"] + 4.2 * min(life / 0.6, 1.0)
            p["r"] = grow * (1.0 if life < 0.65 else max(0.0, 1 - (life - 0.65) / 0.35))
            draw_puff(frame, p)
        # огонь
        flames = flame_layer(burning, f, rng)
        m = flames[..., 3] > 0
        frame[m] = flames[m]
        # пепел и искры поверх
        for p in ash.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW - 1 and 0 <= y < FH:
                col = ASH[0] if p["age"] < p["life"] * 0.6 else ASH[1]
                frame[y, x] = (*col, 255)
                if p["big"]:
                    frame[y, x + 1] = (*ASH[1], 255)
        for p in embers.items:
            x, y = int(p["x"]), int(p["y"])
            if 0 <= x < FW and 0 <= y < FH:
                life = p["age"] / p["life"]
                frame[y, x] = (*FIRE[[1, 3, 5, 7][min(int(life * 4), 3)]], 255)
                if p["age"] < 2 and y + 1 < FH:
                    frame[y + 1, x] = (*FIRE[5], 255)
        frames.append(frame)
        for ps in (embers, smoke, ash):
            ps.step()
    return frames


def save_all(frames, out, bg="#1b1e27", scale=3):
    os.makedirs(out, exist_ok=True)
    strip = np.concatenate(frames, axis=1)
    Image.fromarray(strip, "RGBA").save(os.path.join(out, "burn_strip.png"))
    # лист кадров
    cols = 8
    rows = (len(frames) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (FW + 4) + 4, rows * (FH + 4) + 4), hexc(bg) + (255,))
    for k, fr in enumerate(frames):
        sheet.alpha_composite(Image.fromarray(fr, "RGBA"), (4 + (k % cols) * (FW + 4), 4 + (k // cols) * (FH + 4)))
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(os.path.join(out, "preview_burn_frames.png"))
    # GIF на тёмном фоне, с паузами в начале и в конце; общая палитра без дизеринга
    pics = []
    for fr in frames:
        im = Image.new("RGBA", (FW, FH), hexc(bg) + (255,))
        im.alpha_composite(Image.fromarray(fr, "RGBA"))
        pics.append(im.convert("RGB").resize((FW * scale, FH * scale), Image.NEAREST))
    allpix = Image.new("RGB", (pics[0].width, pics[0].height * len(pics)))
    for k, p in enumerate(pics):
        allpix.paste(p, (0, k * p.height))
    pal = allpix.quantize(colors=255, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    gif = [p.quantize(palette=pal, dither=Image.Dither.NONE) for p in pics]
    durations = [FPS_MS] * len(gif)
    durations[0] = 600
    durations[-1] = 500
    gif[0].save(os.path.join(out, "preview_burn.gif"), save_all=True, append_images=gif[1:],
                duration=durations, loop=0, disposal=1)


def save_on_face(out, art_name, kind, strip_name):
    """Карта с картинкой `art_name` в окне лица типа `kind` — как её собирает
    `src/assets/cards.rs`."""
    path = os.path.join(art.ROOT, "assets", "breach", "cards", "art", f"{art_name}.png")
    card_art = np.array(Image.open(path).convert("RGBA"))
    face = bc.card_front(kind).copy()
    face[17 : 17 + card_art.shape[0], 6 : 6 + card_art.shape[1]] = card_art
    strip = np.concatenate(render(face), axis=1)
    Image.fromarray(strip, "RGBA").save(os.path.join(out, f"{strip_name}.png"))


def save_swap_victory(out):
    """Обмен в игре — карта победы (просьба пользователя): картинка обмена на лице победы."""
    save_on_face(out, "swap", "victory", "burn_strip_swap_victory")


def save_heal_victory(out):
    """«Исцеление» в игре — карта победы со слоном в крыльях (картинка благословения из
    breach_bless.py)."""
    save_on_face(out, "bless", "victory", "burn_strip_heal_victory")


def save_funding_sacrifice(out):
    """«Дополнительное финансирование» в игре — карта жертвы с договором в крови
    (вариант 3 из breach_contract.py, выбор пользователя)."""
    save_on_face(out, "contract_blood", "sacrifice", "burn_strip_funding_sacrifice")


if __name__ == "__main__":
    save_all(render(bc.card_front("w")), OUT)
    # Остальные карты горят так же; игре нужны только кадры.
    for kind, name in (("b", "burn_strip_b"), ("sacrifice", "burn_strip_sacrifice"), ("victory", "burn_strip_victory")):
        strip = np.concatenate(render(bc.card_front(kind)), axis=1)
        Image.fromarray(strip, "RGBA").save(os.path.join(OUT, f"{name}.png"))
    # Карты с картинкой (огненная волна, обмен, благословение) — обычная карта стороны с картинкой в окне
    # (как в breach_firewave.py и breach_portal.py).
    for name in ("fire_wave", "swap", "bless"):
        card_art = np.array(Image.open(os.path.join(art.ROOT, "assets", "breach", "cards", "art", f"{name}.png")).convert("RGBA"))
        for f in "wb":
            face = bc.card_front(f).copy()
            face[17 : 17 + card_art.shape[0], 6 : 6 + card_art.shape[1]] = card_art
            strip = np.concatenate(render(face), axis=1)
            Image.fromarray(strip, "RGBA").save(os.path.join(OUT, f"burn_strip_{name}_{f}.png"))
    save_swap_victory(OUT)
    save_funding_sacrifice(OUT)
    save_heal_victory(OUT)
    print("done", OUT)
