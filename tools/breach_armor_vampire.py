"""Бронированные вампиры — латы `breach_armor.py` и «Дракула» `breach_vampire.py` вместе, в стиле Into the Breach.

От лат: стальной корпус и шлем (`PLATE`, у белых светлая полированная сталь, у чёрных воронёная), полосы лат
с нависающими кромками (`armor.lames`), ступенчатые наплечники. От «Дракулы»: стоячий воротник плаща за
головой (`vp.collar`: снаружи чёрный, изнутри алый), брошь на груди, кроваво-красные огни (`vp.BLOOD_GLOW`)
и немного крови сверху (`vp.bloody`: тонкая кайма на макушке шлема и четыре узких потёка). Отделка —
кайма лат, пояса, шары короны, крест, грива коня — не латунь, а багровый металл (`CRIMSON`). У коня
воротника нет (как у «Дракулы»).

Номера материалов у лат и вампиров совпадают (5, 6, …), поэтому детали каждого генератора перекрашиваются
в свои номера (`recolor`): сталь — 5, багровое — 13, плащ — 14, подкладка — 15, кровь — 16.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_armor_vampire.py [папка]` — по умолчанию assets/breach/pieces: armor_vampire/<цвет><фигура>.png и
_shadow.png, превью preview_armor_vampire.png (ряды: в латах, «Дракула», вместе) и
preview_armor_vampire_board.png.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_armor as armor  # noqa: E402
import breach_art as art  # noqa: E402
import breach_poison as bp  # noqa: E402
import breach_vampire as vp  # noqa: E402
from breach_art import GLOW, MAIN, TRIM, Y, sphere  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

PLATE, CRIMSON, CAPE, LINING, BLOOD = armor.PLATE, 13, 14, 15, 16


def recolor(part, table):
    """Деталь с перенумерованными материалами."""
    f, m = part

    def mat(p):
        base = m(p) if callable(m) else np.full(len(p), m)
        out = base.copy()
        for old, new in table.items():
            out[base == old] = new
        return out

    return f, mat


ARMOR_IDS = {armor.BRASS: CRIMSON}  # латунь лат — багровый металл
VAMPIRE_IDS = {vp.CAPE: CAPE, vp.LINING: LINING, vp.CRIMSON: CRIMSON, vp.BLOOD: BLOOD}


def armored_vampire(kind, color):
    seed = 81 + "PNBRQK".index(kind) * 7 + (0 if color == "w" else 3)  # как у «Дракулы»
    rng = np.random.default_rng(seed)
    top = bp.top_of(art.PIECES[kind]())
    drips = [(a + rng.uniform(-10, 10), rng.uniform(0.1, 0.32) * top, rng.uniform(0.7, 1.1))
             for a in np.linspace(vp.FRONT - 70, vp.FRONT + 70, 4)]
    piece = armor.armour(kind)
    bodies = 2 if kind == "N" else 1  # корпус (у коня — и голова): кровь, потом сталь
    parts = []
    for k, part in enumerate(piece.parts):
        if k < bodies:
            plain = art.PIECES[kind]().parts[k]
            f, m = vp.bloody(plain, seed + k, top, drips)  # кровь — по исходным материалам корпуса
            part = armor.steel((f, m))
            part = recolor(part, {**ARMOR_IDS, vp.BLOOD: BLOOD})
        else:
            part = recolor(part, ARMOR_IDS)
        parts.append(part)
    if kind != "N":
        parts.append(recolor(vp.collar(*vp.COLLARS[kind]), VAMPIRE_IDS))
        y, r = vp.BROOCH[kind]
        parts.append((sphere(art.T * r + Y * y, 1.15), GLOW))
    return art.Piece(parts, stretch=piece.stretch)


def render(kind, color):
    key = f"armor_vampire_{color}"
    pal = dict(art.PALETTES[color])
    pal.update(armor.ARMOR[color])
    pal[GLOW] = vp.BLOOD_GLOW
    pal.update({CRIMSON: vp.EXTRA[vp.CRIMSON], CAPE: vp.EXTRA[vp.CAPE], LINING: vp.EXTRA[vp.LINING],
                BLOOD: vp.EXTRA[vp.BLOOD]})
    art.PALETTES[key] = pal
    return art.render_piece(armored_vampire(kind, color), key, 0.86)


def main():
    folder = os.path.join(OUT, "armor_vampire")
    os.makedirs(folder, exist_ok=True)
    load = lambda sub: {c + k: Image.open(os.path.join(OUT, sub, f"{c}{k}.png")).convert("RGBA")  # noqa: E731
                        for c in "wb" for k in "PNBRQK"}
    sprites, shadows = {}, {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(folder, f"{color}{kind}.png"))
            shadow.save(os.path.join(folder, f"{color}{kind}_shadow.png"))
            sprites[color + kind], shadows[color + kind] = body, shadow
            print("armor_vampire", color + kind, flush=True)
    vp.preview([load("armor"), load("vampire_dracula"), sprites], os.path.join(OUT, "preview_armor_vampire.png"))
    board = {code: (sprites[code], shadows[code]) for code in sprites}
    art.board_preview(board, scale=2, demo=False).save(os.path.join(OUT, "preview_armor_vampire_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
