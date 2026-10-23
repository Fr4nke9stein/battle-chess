"""Фигуры-зомби — в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`, но мёртвые и полуразложившиеся:

- кожа — трупная (`SKIN`: у белых бледная серо-зелёная, у чёрных тёмная болотная), по ней пятна
  гнили (`ROT`: у белых синюшные, у чёрных тёмные синяки; крупный 3D-шум `ip.noise3`);
- отделка — рваное тряпьё (`RAG`): пояса с лохмотьями вниз (полосы разной длины по азимуту) и дырами,
  сквозь которые видна кожа; короны — позеленевшая бронза (`RUST`);
- вырванные куски (`chunks`: шар вычтен из тела): на теле видно мясо (`FLESH`) с рёбрами (`BONE`),
  на голове — кость черепа, у ладьи обломаны зубцы;
- зашитые шрамы (`scar`: тёмный шов со стежками поперёк);
- основание — в кладбищенской земле (`EARTH`) с клочками мха (`MOSS`), ступня — камень цвета отделки;
- глаза и огни (`GLOW`): у белых гнилостно-зелёные, у чёрных — янтарные, чтобы стороны различались. Пешка —
  с лицом: один глаз светится, второй — пустая глазница, рот с зубами. Ферзь — костяные шишки короны (двух
  нет), король — кривой костяной крест; конь — рваное ухо, вырванный бок шеи с рёбрами, зубы, свалявшаяся
  грива (`HAIR`) с вырванными клоками.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_zombie.py [папка]` — по умолчанию assets/breach/pieces: zombie/<цвет><фигура>.png и
_shadow.png, превью preview_zombie_pieces.png (обычные / зомби) и preview_zombie_board.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_ice_pieces as ip  # noqa: E402
import breach_poison as bp  # noqa: E402
import breach_vampire as vp  # noqa: E402
from breach_art import GLOW, MAIN, R, SEAM, T, TRIM, Y, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

ROT, RAG, FLESH, BONE, EARTH, MOSS, RUST, HAIR = 5, 6, 7, 8, 9, 10, 11, 12
SKIN = {
    "w": ["#26301f", "#4e5f3e", "#7f9466", "#adc18e", "#d9e6bc"],  # бледная серо-зелёная
    "b": ["#0b100b", "#18231a", "#2a3a2b", "#425a40", "#66805c"],  # тёмная болотная
}
GLOWS = {
    "w": ["#1d5a12", "#3fa020", "#7ce83a", "#c4ff8a", "#f0ffd8"],  # гнилостно-зелёные
    "b": ["#5a3a06", "#a06a10", "#e0a830", "#ffe07a", "#fff6d0"],  # янтарные
}
SIDE = {
    "w": {ROT: ["#1e1820", "#3c303a", "#5c4a56", "#7a6670", "#988690"],  # синяки
          RAG: ["#241e16", "#463c2e", "#6e624a", "#968a6e", "#bcb094"],  # грязный лён
          HAIR: ["#14120e", "#26221a", "#3a3428", "#524a3a", "#6e6450"]},
    "b": {ROT: ["#0e080c", "#1e1018", "#321c28", "#482838", "#643a4e"],  # тёмные синяки (светлая плесень — как камуфляж)
          RAG: ["#120c0a", "#261a14", "#3e2c22", "#5a4232", "#7c5e48"],  # истлевшая ткань
          HAIR: ["#0a0806", "#18120e", "#281e16", "#3a2c20", "#503e2e"]},
}
COMMON = {
    FLESH: ["#1e0408", "#4a0a12", "#7a1420", "#a8283a", "#d05060"],
    BONE: ["#3a3428", "#6e6650", "#a49c80", "#d2ccb0", "#f2eedc"],
    EARTH: ["#120c07", "#281c11", "#42301c", "#5e462a", "#7e6040"],
    MOSS: ["#0e1a08", "#1e3410", "#34521a", "#4e7226", "#72983a"],
    RUST: ["#0e1c18", "#1c362e", "#2c5446", "#467a62", "#70a68a"],
}
FRONT = 45.0  # азимут к зрителю
BASE_TOP = 6.7  # выше — пояса фигуры, ниже — основание
# пояса: нижние кромки тряпья (от них вниз — лохмотья) и короны (бронза)
RAGS = {"P": [13.0], "B": [20.8], "R": [26.0], "Q": [25.0], "K": [27.4], "N": [6.7]}
CROWNS = {"Q": [(41.2, 44.0)], "K": [(46.4, 49.0)]}
# вырванные куски: высота, азимут, радиус шара, что внутри
CHUNKS = {
    "P": [(9.0, FRONT + 36, 2.9, "flesh"), (27.4, FRONT - 50, 2.6, "bone")],
    "B": [(15.0, FRONT + 40, 2.8, "flesh"), (37.0, FRONT - 46, 2.7, "bone")],
    "R": [(18.0, FRONT + 34, 3.6, "flesh"), (36.6, FRONT - 38, 2.8, "rot"), (36.6, FRONT + 60, 2.4, "rot")],
    "Q": [(17.0, FRONT + 36, 2.8, "flesh"), (34.0, FRONT - 52, 2.6, "flesh")],
    "K": [(18.5, FRONT - 38, 2.9, "flesh"), (40.0, FRONT + 48, 2.7, "flesh")],
}
# шрамы: (азимут, высота) начала и конца
SCARS = {
    "B": ((FRONT - 32, 33.0), (FRONT + 26, 29.0)),
    "R": ((FRONT + 16, 24.5), (FRONT + 52, 11.5)),
    "Q": ((FRONT - 30, 39.0), (FRONT + 24, 33.5)),
    "K": ((FRONT - 28, 42.0), (FRONT + 26, 37.0)),
}


def azimuth(p):
    return np.degrees(np.arctan2(p[:, 2], p[:, 0]))


def adiff(a, b):
    return ((a - b + 180) % 360) - 180


def chunk_spheres(piece, kind):
    """Шары вырванных кусков: центр чуть снаружи поверхности фигуры."""
    out = []
    rs = np.arange(20, 0, -0.1)
    for y, az, r, inner in CHUNKS.get(kind, []):
        d = bp.part_dist(piece, np.stack([ip.at(q, y, az) for q in rs]))
        sr = rs[np.nonzero(d < 0)[0][0]]  # снаружи внутрь: у ладьи ось наверху — внутри башни
        out.append((ip.at(sr + 0.55 * r, y, az), r, inner))
    return out


def bitten(f, spheres):
    for c, r, _ in spheres:
        f = art.cut(f, sphere(c, r))
    return f


def wounds(p, out, spheres):
    """Поверхность вырванных кусков: мясо с рёбрами, кость или гниль."""
    for c, r, inner in spheres:
        on = np.linalg.norm(p - c, axis=1) - r < 0.3
        if inner == "bone":
            out[on] = BONE
        elif inner == "rot":
            out[on] = ROT
        else:
            out[on] = FLESH
            ribs = on & ((p[:, 1] - c[1]) % 1.9 < 0.75) & (np.linalg.norm(p - c, axis=1) < r + 0.1)
            out[ribs] = BONE
    return out


def scar_mask(p, kind):
    if kind not in SCARS:
        return np.zeros(len(p), bool)
    (a0, y0), (a1, y1) = SCARS[kind]
    rad = np.hypot(p[:, 0], p[:, 2])
    s = rad * np.radians(adiff(azimuth(p), FRONT))  # дуга по поверхности от фаса
    rr = rad.mean() if len(p) else 1.0
    a = (rr * math.radians(a0 - FRONT), y0)
    b = (rr * math.radians(a1 - FRONT), y1)
    line = art.sd_segment2(s, p[:, 1], a, b) < 0.34
    dx, dy = b[0] - a[0], b[1] - a[1]
    ln = math.hypot(dx, dy)
    nx, ny = -dy / ln, dx / ln
    ticks = np.zeros(len(p), bool)
    for t in np.arange(0.8, ln - 0.5, 1.5):  # стежки поперёк шва
        cx, cy = a[0] + dx * t / ln, a[1] + dy * t / ln
        ticks |= art.sd_segment2(s, p[:, 1], (cx - nx * 0.75, cy - ny * 0.75), (cx + nx * 0.75, cy + ny * 0.75)) < 0.24
    front = np.abs(adiff(azimuth(p), FRONT)) < 80
    return (line | ticks) & front


def rot_body(kind, seed, spheres, face=None):
    """Материал корпуса: гниль, тряпьё с лохмотьями и дырами, бронза корон, земля основания, раны, шрам."""
    crowns = CROWNS.get(kind, [])

    def wrap(m):
        def mat(p):
            base = m(p) if callable(m) else np.full(len(p), m)
            out = base.copy()
            y = p[:, 1]
            az = azimuth(p)
            skin = base == MAIN
            out[skin & (ip.noise3(p, 3.4, seed) > 0.68)] = ROT
            trim = base == TRIM
            crown = np.zeros(len(p), bool)
            for lo, hi in crowns:
                crown |= (y >= lo - 0.2) & (y < hi + 0.2)
            out[trim & crown] = RUST
            rag = trim & ~crown & (y >= BASE_TOP)
            strip = np.floor((az + 180) / 11).astype(np.int64)
            for k, lo in enumerate(RAGS.get(kind, [])):
                length = 0.3 + 4.6 * ip.hash3(strip, np.full(len(p), k), np.zeros(len(p), np.int64), seed) ** 1.3
                rag |= skin & (y < lo) & (y > lo - length)
            holes = ip.noise3(p, 1.7, seed + 5) > 0.72
            out[rag] = RAG
            out[rag & holes] = MAIN
            band = trim & (y < BASE_TOP)
            out[band] = EARTH
            out[band & (ip.noise3(p, 1.4, seed + 9) > 0.66)] = MOSS
            out[skin & (y < 3.0)] = TRIM  # ступня — камень цвета отделки
            out[(base == GLOW)] = GLOW
            if face is not None:
                face(p, out)
            out = wounds(p, out, spheres)
            out[scar_mask(p, kind) & ((out == MAIN) | (out == ROT))] = SEAM
            return out

        return mat

    return wrap


def pawn_face(p, out):
    """Пешка: вместо забрала — светящийся глаз, пустая глазница и рот с зубами."""
    y, az = p[:, 1], azimuth(p)
    head = (y > 17.0) & (np.abs(adiff(az, FRONT)) < 70)
    out[head & (out == GLOW)] = MAIN
    rad = np.hypot(p[:, 0], p[:, 2])
    s = rad * np.radians(adiff(az, FRONT))
    eye = head & (np.hypot(s + 2.6, y - 24.4) < 1.65)
    socket = head & (np.hypot(s - 2.8, y - 24.8) < 1.35)
    mouth = head & (np.abs(y - 20.4) < 0.6) & (np.abs(s + 0.2) < 3.2)
    teeth = mouth & (y > 20.4) & ((np.floor((s + 4.0) / 1.0) % 2) == 0)
    out[eye] = GLOW
    out[socket | mouth] = SEAM
    out[teeth] = BONE


def zombie(kind, color):
    seed = 31 + "PNBRQK".index(kind) * 9 + (0 if color == "w" else 4)
    if kind == "N":
        return zombie_knight(color, seed)
    piece = art.PIECES[kind]()
    spheres = chunk_spheres(piece, kind)
    wrap = rot_body(kind, seed, spheres, pawn_face if kind == "P" else None)
    f, m = piece.parts[0]
    parts = [(bitten(f, spheres), wrap(m))]
    if kind == "Q":  # шишки короны — кость, двух нет; верхний камень светится
        for k, (sf, sm) in enumerate(piece.parts[1:9]):
            if k not in (2, 5):
                parts.append((sf, BONE))
        parts.append(piece.parts[9])
    elif kind == "K":
        parts += crooked_cross()
    else:
        parts += list(piece.parts[1:])
    return art.Piece(parts, stretch=piece.stretch)


def crooked_cross(tilt=14.0):
    """Крест короля — кость, покосился."""
    a = math.radians(tilt)
    rx = R * math.cos(a) + Y * math.sin(a)
    ry = -R * math.sin(a) + Y * math.cos(a)
    pivot = np.array([0.0, 56.4, 0.0])
    axes = [rx, ry, T]

    def at(c):
        d = np.asarray(c, float) - pivot
        return pivot + rx * (d @ R) + ry * (d @ Y) + T * (d @ T)

    cross_v = art.rbox(at([0, 63.6, 0]), [2.1, 7.4, 2.1], axes, 0.5)
    cross_h = art.rbox(at([0, 65.4, 0]), [6.4, 2.1, 2.1], axes, 0.5)
    gem_c = at([0, 65.4, 0])

    def mat(p):
        out = np.full(len(p), BONE)
        d = p - gem_c
        gem = (np.abs(d @ rx) < 1.5) & (np.abs(d @ ry) < 1.5) & ((p @ T) > 1.0)
        out[gem] = GLOW
        return out

    return [(cross_v, mat), (cross_h, mat)]


def zombie_knight(color, seed):
    piece = art.PIECES["N"]()
    (base_f, base_m), (head_f, head_m), (mane_f, _) = piece.parts
    a = math.radians(18)
    fwd = norm(-R * math.cos(a) + T * math.sin(a))
    side = norm(np.cross(Y, fwd))  # к зрителю
    neck = (fwd * -1.6 + Y * 31.0 + side * 4.4, 2.9, "flesh")  # вырванный бок шеи — рёбра
    jaw = (fwd * 9.5 + Y * 22.6 + side * 3.6, 1.9, "flesh")
    ear = (fwd * -2.6 + Y * 52.6, 1.9, "rot")  # рваное ухо
    spheres = [neck, jaw, ear]
    head = bitten(head_f, spheres)
    wrap = rot_body("N", seed, [])

    def head_mat(p):
        base = head_m(p)
        out = wrap(lambda q: np.where(base == TRIM, TRIM, base))(p)
        out[(base == TRIM)] = RAG  # уздечка — тряпка
        u, y = p @ fwd, p[:, 1]
        teeth = (u > 9.0) & (u < 15.2) & (np.abs(y - 26.5) < 0.55) & (np.floor(u / 0.9) % 2 == 0)
        out[teeth] = BONE
        return wounds(p, out, spheres)

    tufts = [(fwd * -9.6 + Y * 22.0, 1.8), (fwd * -6.8 + Y * 37.5, 1.6)]
    mane = mane_f
    for c, r in tufts:
        mane = art.cut(mane, sphere(c, r))
    hair = lambda p: np.where(ip.noise3(p, 2.0, seed + 2) > 0.7, ROT, HAIR)  # noqa: E731
    base_wrap = wrap(base_m)
    return art.Piece([(base_f, base_wrap), (head, head_mat), (mane, hair)], stretch=piece.stretch)


def render(kind, color):
    key = f"zombie_{color}"
    pal = dict(art.PALETTES[color])
    pal[MAIN] = SKIN[color]
    pal[GLOW] = GLOWS[color]
    pal.update(COMMON)
    pal.update(SIDE[color])
    art.PALETTES[key] = pal
    return art.render_piece(zombie(kind, color), key, 0.86)


def main():
    folder = os.path.join(OUT, "zombie")
    os.makedirs(folder, exist_ok=True)
    normal = {c + k: Image.open(os.path.join(OUT, f"{c}{k}.png")).convert("RGBA") for c in "wb" for k in "PNBRQK"}
    sprites, shadows = {}, {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(folder, f"{color}{kind}.png"))
            shadow.save(os.path.join(folder, f"{color}{kind}_shadow.png"))
            sprites[color + kind], shadows[color + kind] = body, shadow
            print("zombie", color + kind, flush=True)
    vp.preview([normal, sprites], os.path.join(OUT, "preview_zombie_pieces.png"))
    board = {code: (sprites[code], shadows[code]) for code in sprites}
    art.board_preview(board, scale=2, demo=False).save(os.path.join(OUT, "preview_zombie_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
