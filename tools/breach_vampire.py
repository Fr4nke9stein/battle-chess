"""Вампирские фигуры — три варианта на выбор, в стиле Into the Breach.

Те же 3D-фигуры, что в `breach_art.py`; у всех вариантов огоньки (глаза, прорези) — кроваво-красные
(`BLOOD_GLOW`).

- `count` («Граф», Дракула): за головой высокий стоячий воротник плаща — оболочка вокруг оси
  фигуры только сзади, к верху расходится (`collar`, `COLLARS`); снаружи чёрный (`CAPE`), изнутри —
  алый (`LINING`), его и видно спереди. Отделка — багровая (`CRIMSON`), на груди — светящаяся
  брошь. У коня воротника нет (плащ на шее выходил чёрным бруском) — багровая грива и отделка.
- `blood` («Кровь»): бледные фигуры (белые — цвета кости, чёрные — почти чёрно-винные, `PALE`),
  сверху облиты кровью, по бокам потёки и капли на концах — форма из `breach_poison.py`
  (`poisoned`), жидкость — `BLOOD`.
- `dracula` («Дракула», выбран пользователем — «Граф» и «Кровь» вместе): воротник, багровая отделка и
  брошь «Графа», корпус бледный у белых и почти чёрный с винным отливом у чёрных (`DRACULA_SKIN`),
  крови немного (`bloody`: тонкая кайма `LITTLE_COAT` на макушке и четыре узких коротких потёка, без
  лужиц и капель).
- `bat` («Нетопырь»): по бокам головы — перепончатые крылья летучей мыши (`wing`: плоский
  силуэт `WING` — верхняя кромка к кончику, нижняя фестонами между «пальцами», чуть заведён
  назад), перепонка — тёмная (`MEMBRANE`), пальцы — костяные (`RIB`). Корпус — в цветах сторон.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_vampire.py [папка]` — по умолчанию assets/breach/pieces:
vampire_<count|blood|bat|dracula>/<цвет><фигура>.png и _shadow.png, превью preview_vampire_pieces.png
(ряды: обычные, «Граф», «Кровь», «Нетопырь», «Дракула»), preview_vampire_dracula.png (обычные и «Дракула»)
и preview_vampire_dracula_board.png. `py tools/breach_vampire.py <папка> dracula` — только «Дракула».
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_poison as bp  # noqa: E402
from breach_art import GLOW, MAIN, R, T, TRIM, Y, norm, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

CAPE, LINING, CRIMSON, BLOOD, MEMBRANE, RIB = 5, 6, 7, 8, 9, 10
BLOOD_GLOW = ["#5a0010", "#b0102a", "#ff3050", "#ff8a9a", "#ffe0e6"]
EXTRA = {
    CAPE: ["#050407", "#0e0b12", "#1a1520", "#2a2232", "#40364a"],
    LINING: ["#3a0410", "#6e0a1c", "#a8142c", "#d82c42", "#ff6a78"],
    CRIMSON: ["#2a0408", "#5a0a14", "#8e1424", "#c42838", "#ee6070"],
    BLOOD: ["#2a0006", "#5e000e", "#9a0a1a", "#d0202e", "#ff6a70"],
    MEMBRANE: ["#0c070e", "#1e1020", "#341c34", "#4e2c4c", "#704468"],
    RIB: ["#1a1012", "#3a2628", "#5c4040", "#806060", "#a88a86"],
}
PALE = {
    "w": ["#4a4448", "#8a8288", "#c4bcc0", "#e8e0e2", "#fffafa"],  # кость
    "b": ["#120408", "#24080e", "#3c0e18", "#5a1828", "#842a3c"],  # чёрно-винный
}
# корпус «Дракулы»: белые — кость, чёрные — почти чёрный с винным отливом (чёрно-винный «Крови» сливался с кровью)
DRACULA_SKIN = {"w": PALE["w"], "b": ["#06030a", "#120810", "#20101c", "#32182c", "#4a2640"]}
# воротник «Графа»: низ (шея), верх, радиус у низа, насколько расходится к верху
COLLARS = {
    "P": (16.4, 31.0, 6.6, 0.5),
    "B": (24.4, 39.0, 7.4, 0.5),
    "R": (29.6, 44.0, 12.8, 0.32),
    "Q": (28.6, 47.0, 7.4, 0.48),
    "K": (31.4, 50.0, 7.8, 0.48),
}
BROOCH = {"P": (15.0, 8.75), "B": (22.6, 9.35), "R": (28.2, 12.1), "Q": (26.8, 9.55), "K": (29.5, 9.95)}
# крылья «Нетопыря»: центр головы по высоте, радиус головы
HEADS = {"P": (23.5, 6.6), "B": (31.0, 7.6), "R": (31.5, 11.0), "Q": (40.0, 8.0), "K": (44.0, 8.2)}
# силуэт крыла (a — наружу, b — вверх от точки крепления): верх к кончику, низ фестонами
WING = [(0.0, 1.2), (2.6, 4.2), (5.6, 6.8), (9.6, 6.4), (9.2, 3.2), (7.6, 3.6), (6.4, 1.2), (5.0, 2.4),
        (3.6, 0.0), (2.2, 1.0), (0.0, -1.2)]
FINGERS = [(9.6, 6.4), (6.4, 1.2), (3.6, 0.0)]


def collar(y0, y1, r0, flare):
    """Стоячий воротник: оболочка вокруг оси фигуры только сзади (от зрителя), к верху шире."""
    def f(p):
        rad = np.hypot(p[:, 0], p[:, 2])
        rc = r0 + np.clip(p[:, 1] - y0, 0, None) * flare
        shell = np.abs(rad - rc) - 0.5
        back = p @ T - 0.3 * rad  # остаётся та часть, что сзади
        az = np.arctan2(p @ R, -(p @ T))
        top = y1 - 2.2 * np.abs(np.sin(az * 1.5))  # верх зубцами
        return np.maximum.reduce([shell, back, y0 - p[:, 1], p[:, 1] - top])

    def mat(p):
        rad = np.hypot(p[:, 0], p[:, 2])
        rc = r0 + np.clip(p[:, 1] - y0, 0, None) * flare
        return np.where(rad < rc, LINING, CAPE)

    return f, mat


def neck_cape(head_f, fwd):
    """Плащ на шее коня: оболочка вокруг головы-шеи сзади."""
    def f(p):
        u = p @ fwd
        return np.maximum.reduce([np.abs(head_f(p) - 1.0) - 0.45, u + 1.5, 20.0 - p[:, 1], p[:, 1] - 42.0])

    def mat(p):
        return np.where(head_f(p) < 1.0, LINING, CAPE)

    return f, mat


def count(kind):
    piece = art.PIECES[kind]()
    if kind == "N":
        base_part, head, (mane_f, _) = piece.parts
        a = math.radians(18)
        fwd = norm(-R * math.cos(a) + T * math.sin(a))
        parts = [crimson(base_part), crimson(head), (mane_f, CRIMSON)]  # плащ на шее выходил чёрным бруском
        return art.Piece(parts, stretch=piece.stretch)
    parts = [crimson(piece.parts[0])] + list(piece.parts[1:])
    parts.append(collar(*COLLARS[kind]))
    y, r = BROOCH[kind]
    parts.append((sphere(T * r + Y * y, 1.15), GLOW))
    return art.Piece(parts, stretch=piece.stretch)


def crimson(part):
    """Отделка — багровая."""
    f, m = part
    return f, (lambda p: np.where((m(p) if callable(m) else np.full(len(p), m)) == TRIM, CRIMSON,
                                  m(p) if callable(m) else np.full(len(p), m)))


def wing(anchor, out, size):
    """Крыло летучей мыши: плоский силуэт WING, обращён к зрителю, кончик заведён назад."""
    poly = [(a * size, b * size) for a, b in WING]
    fingers = [(a * size, b * size) for a, b in FINGERS]
    c = np.asarray(anchor, float)

    def local(p):
        d = p - c
        a = d @ out
        return a, d[:, 1], d @ T + 0.22 * a

    def f(p):
        a, b, w = local(p)
        return np.maximum(art.sd_polygon(a, b, poly), np.abs(w) - 0.45)

    def mat(p):
        a, b, _ = local(p)
        rib = np.min(np.stack([art.sd_segment2(a, b, (0.0, 0.0), tip) for tip in fingers]), axis=0) < 0.55
        edge = art.sd_polygon(a, b, poly) > -0.55
        out_m = np.full(len(p), MEMBRANE)
        out_m[rib | (edge & (b > a * 0.55))] = RIB  # пальцы и верхняя кромка — кость
        return out_m

    return f, mat


def bat(kind):
    piece = art.PIECES[kind]()
    parts = list(piece.parts)
    if kind == "N":
        a = math.radians(18)
        fwd = norm(-R * math.cos(a) + T * math.sin(a))
        anchor, size = fwd * 0.5 + Y * 34.0, 1.45
        for s in (R, -R):
            parts.append(wing(anchor + s * 3.0, s, size))
        return art.Piece(parts, stretch=piece.stretch)
    y, r = HEADS[kind]
    size = 1.55 if kind != "P" else 1.3
    for s in (R, -R):
        parts.append(wing(Y * (y - 1.5) + s * (r * 0.75), s, size))
    return art.Piece(parts, stretch=piece.stretch)


LITTLE_COAT = 0.07  # у «Дракулы» крови меньше: тонкая кайма на макушке
FRONT = 45.0  # азимут к зрителю


def bloody(part, seed, top, drips):
    """Часть фигуры в крови, но немного: тонкая неровная кайма сверху и несколько узких потёков, без лужиц."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        out = base.copy()
        free = (base == MAIN) | (base == TRIM) | (base == CRIMSON)  # огни светят сквозь кровь
        az = np.degrees(np.arctan2(p[:, 2], p[:, 0]))
        r = np.maximum(np.hypot(p[:, 0], p[:, 2]), 2.0)
        cap = LITTLE_COAT * top
        depth = cap + 1.2 * (bp.ip.noise3(p, 3.0, seed) - 0.5)
        for a, length, width in drips:
            d = (((az - a + 180) % 360) - 180) * math.pi / 180 * r
            k = np.clip(1 - (d / width) ** 2, 0, 1)
            depth = np.maximum(depth, cap + length * np.sqrt(k))
        out[free & (p[:, 1] > top - depth)] = BLOOD
        return out

    return f, mat


def dracula(kind, color):
    """«Дракула» — «Граф» (воротник, багровая отделка, брошь) с бледным корпусом и немногими потёками крови."""
    piece = count(kind)
    seed = 81 + "PNBRQK".index(kind) * 7 + (0 if color == "w" else 3)
    rng = np.random.default_rng(seed)
    top = bp.top_of(art.PIECES[kind]())
    drips = [(a + rng.uniform(-10, 10), rng.uniform(0.1, 0.32) * top, rng.uniform(0.7, 1.1))
             for a in np.linspace(FRONT - 70, FRONT + 70, 4)]
    bodies = 2 if kind == "N" else 1  # у коня кровь и на голове
    parts = [bloody(p, seed + k, top, drips) if k < bodies else p for k, p in enumerate(piece.parts)]
    return art.Piece(parts, stretch=piece.stretch)


def render(variant, kind, color):
    key = f"vampire_{variant}_{color}"
    pal = dict(art.PALETTES[color])
    pal[GLOW] = BLOOD_GLOW
    pal.update(EXTRA)
    if variant == "blood":
        pal[MAIN] = PALE[color]
        pal[bp.SLIME] = EXTRA[BLOOD]
        piece = bp.poisoned(kind, color)
    elif variant == "dracula":
        pal[MAIN] = DRACULA_SKIN[color]
        piece = dracula(kind, color)
    else:
        piece = count(kind) if variant == "count" else bat(kind)
    art.PALETTES[key] = pal
    return art.render_piece(piece, key, 0.86)


def preview(rows, path):
    cw, ch = 48, 80
    sheet = Image.new("RGBA", (12 * cw + 8, len(rows) * ch + 8), art.hexc("#2c3a2c") + (255,))
    for r in range(1, len(rows), 2):
        sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), art.hexc("#3d4a3a") + (255,)), (0, 4 + r * ch))
    for r, sprites in enumerate(rows):
        for c, code in enumerate(color + k for color in "wb" for k in "PNBRQK"):
            sheet.alpha_composite(sprites[code].crop((8, 0, 56, 80)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def main():
    only = sys.argv[2:] or ["count", "blood", "bat", "dracula"]  # можно перерисовать не все варианты
    normal = {c + k: Image.open(os.path.join(OUT, f"{c}{k}.png")).convert("RGBA") for c in "wb" for k in "PNBRQK"}
    rows = [normal]
    for variant in only:
        folder = os.path.join(OUT, f"vampire_{variant}")
        os.makedirs(folder, exist_ok=True)
        sprites = {}
        for color in "wb":
            for kind in "PNBRQK":
                body, shadow = render(variant, kind, color)
                body.save(os.path.join(folder, f"{color}{kind}.png"))
                shadow.save(os.path.join(folder, f"{color}{kind}_shadow.png"))
                sprites[color + kind] = body
                print(variant, color + kind, flush=True)
        rows.append(sprites)
        if variant == "dracula":  # выбранный вариант: рядом с обычными и на доске
            preview([normal, sprites], os.path.join(OUT, "preview_vampire_dracula.png"))
            shadows = {code: Image.open(os.path.join(folder, f"{code}_shadow.png")).convert("RGBA") for code in sprites}
            board = {code: (sprites[code], shadows[code]) for code in sprites}
            art.board_preview(board, scale=2, demo=False).save(os.path.join(OUT, "preview_vampire_dracula_board.png"))
    if len(rows) == 5:
        preview(rows, os.path.join(OUT, "preview_vampire_pieces.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
