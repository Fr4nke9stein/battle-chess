"""Маги с варварскими рогами — в стиле Into the Breach.

Фигуры-маги из `breach_wizard.py` (остроконечная шляпа: белые — синяя, чёрные — багровая; золотая
лента, звёздочки), из боков тульи у самых полей растут рога варвара (`breach_barbarian.horn`: кость,
к концу тоньше и темнее) — наружу и вверх. Номера материалов рогов — свои (`HORN_M`, `TIP_M`), у мага
5–7 заняты шляпой, лентой и звёздами.

Спрайты — как обычные (64×84, точка опоры art.AX, art.AY, отдельная тень):
`py tools/breach_wizard_horns.py [папка]` — по умолчанию assets/breach/pieces:
wizard_horns/<цвет><фигура>.png и _shadow.png, превью preview_wizard_horns.png (маги и маги с рогами) и
preview_wizard_horns_board.png.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_barbarian as bb  # noqa: E402
import breach_wizard as wz  # noqa: E402
from breach_art import R, T, Y, norm  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "pieces")

HORN_M, TIP_M = 13, 14
# рога: вынос наружу и вверх, толщина — в долях радиуса тульи у полей
HORN_SHAPE = (1.05, 1.3, 0.34)
HORN_SCALE = 1.3  # рога крупнее тульи: при размере с тулью их наполовину прятали поля


def horns(base_centre, r0):
    """Пара рогов из боков тульи: по бокам экрана — видно в силуэте."""
    out_k, up_k, thick = HORN_SHAPE
    parts = []
    for s in (R, -R):
        for f, m in bb.horn(base_centre + s * (r0 * 0.7), s, r0 * HORN_SCALE, 0.0, out_k, up_k, thick):
            parts.append((f, TIP_M if m == bb.TIP else HORN_M))
    return parts


def horned_wizard(kind):
    piece = wz.wizard(kind)
    if kind == "N":
        a = math.radians(18)
        fwd = norm(-R * math.cos(a) + T * math.sin(a))
        base, r0 = fwd * 1.2 + Y * 49.5, 5.6
    else:
        _, brim_y, _, r0, _ = wz.HATS[kind]
        base = Y * (brim_y + 3.6)  # над полями — иначе их закрывают поля
    return art.Piece(list(piece.parts) + horns(base, r0), stretch=piece.stretch)


def render(kind, color):
    key = f"wizard_horns_{color}"
    pal = dict(art.PALETTES[color])
    pal.update({wz.HAT: wz.HAT_COLORS[color], wz.GOLD: wz.GOLD_RAMP, wz.STAR: wz.STAR_RAMP,
                HORN_M: bb.EXTRA[bb.HORN], TIP_M: bb.EXTRA[bb.TIP]})
    art.PALETTES[key] = pal
    return art.render_piece(horned_wizard(kind), key, 0.86)


def preview(rows, path):
    cw, ch = 48, 84
    sheet = Image.new("RGBA", (12 * cw + 8, len(rows) * ch + 8), art.hexc("#2c3a2c") + (255,))
    for r in range(1, len(rows), 2):
        sheet.alpha_composite(Image.new("RGBA", (12 * cw + 8, ch), art.hexc("#3d4a3a") + (255,)), (0, 4 + r * ch))
    for r, sprites in enumerate(rows):
        for c, code in enumerate(color + k for color in "wb" for k in "PNBRQK"):
            sheet.alpha_composite(sprites[code].crop((8, 0, 56, 84)), (4 + c * cw, 4 + r * ch))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(path)


def main():
    folder = os.path.join(OUT, "wizard_horns")
    os.makedirs(folder, exist_ok=True)
    plain = {c + k: Image.open(os.path.join(OUT, "wizard", f"{c}{k}.png")).convert("RGBA") for c in "wb" for k in "PNBRQK"}
    sprites, board = {}, {}
    for color in "wb":
        for kind in "PNBRQK":
            body, shadow = render(kind, color)
            body.save(os.path.join(folder, f"{color}{kind}.png"))
            shadow.save(os.path.join(folder, f"{color}{kind}_shadow.png"))
            sprites[color + kind] = body
            board[color + kind] = (body, shadow)
            print(color + kind, flush=True)
    preview([plain, sprites], os.path.join(OUT, "preview_wizard_horns.png"))
    art.board_preview(board, scale=2, demo=False).save(os.path.join(OUT, "preview_wizard_horns_board.png"))
    print("done", OUT)


if __name__ == "__main__":
    main()
