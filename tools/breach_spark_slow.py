"""Замедленное электричество фигур (0.116, просьба пользователя: молния втрое медленнее, чем в
`breach_arcs.py`, но так же плавно) — `breach_arcs.render(..., slow=SLOW)`: кадр прежний (60 мс),
кадров втрое больше. Вступление — [0, LOOP_START·SLOW) (вспышка — кадр FLASH·SLOW), повтор —
[LOOP_START·SLOW, LOOP_END·SLOW), концовки нет (как у `electric_hold`).

Полосы — для всех основ, как `electric_hold` (те же зёрна и тела фигур):
`effects/electric_slow_<код>.png` (обычные) и `effects/<barbarian|gold|wizard|ice|fire>/electric_slow_<код>.png`
(спрайт варвара, золотой, мага, ледяной — подмена `Image.open` из генераторов объединённых фигур; огненная —
тело из кадров трещин ожидания, `breach_fire_combo.lava_bodies`). Превью — `effects/preview_electric_slow.gif`
(обычные белые фигуры, вступление и повтор дважды).

    py tools/breach_spark_slow.py
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_arcs as arcs  # noqa: E402
import breach_barbarian_combo as barbarian  # noqa: E402
import breach_fire_combo as fire  # noqa: E402
import breach_gold_combo as gold  # noqa: E402
import breach_ice_combo as ice  # noqa: E402
import breach_wizard_combo as wizard  # noqa: E402

sys.argv = _argv
from PIL import Image as _Image  # noqa: E402

EFFECTS = os.path.join(arcs.art.ROOT, "assets", "breach", "effects")
SLOW = 3
LENGTH = arcs.LOOP_END * SLOW
CODES = [c + k for c in "wb" for k in "PNBRQK"]
# основа → подмена Image.open (None — обычный спрайт)
SETS = [
    ("", None),
    ("barbarian", barbarian.from_barbarian),
    ("gold", gold.from_gold),
    ("wizard", wizard.from_wizard),
    ("ice", ice.from_ice),
    ("fire", None),
]


def render(folder, opener, code):
    seed = arcs.SEEDS[code[1]]
    if folder == "fire":
        _, bodies = fire.lava_bodies(code)
        return arcs.render(code, seed, bodies=bodies, length=LENGTH, slow=SLOW)
    original = _Image.open
    if opener is not None:
        _Image.open = opener
    try:
        return arcs.render(code, seed, length=LENGTH, slow=SLOW)
    finally:
        _Image.open = original


def save_preview(frames, path, scale=2):
    """Обычные белые фигуры в ряд на тёмном: вступление и повтор дважды."""
    codes = [c for c in CODES if c[0] == "w"]
    order = list(range(LENGTH)) + list(range(arcs.LOOP_START * SLOW, LENGTH))
    pics = []
    for f in order:
        sheet = Image.new("RGBA", (arcs.FW * len(codes), arcs.FH), (24, 28, 41, 255))
        for i, code in enumerate(codes):
            sheet.alpha_composite(Image.fromarray(frames[code][f], "RGBA"), (i * arcs.FW, 0))
        pics.append(sheet.resize((sheet.width * scale, sheet.height * scale), Image.NEAREST).convert("P"))
    pics[0].save(path, save_all=True, append_images=pics[1:], duration=arcs.FRAME_MS, loop=0)


def main():
    base = {}
    for folder, opener in SETS:
        out = os.path.join(EFFECTS, folder)
        os.makedirs(out, exist_ok=True)
        for code in CODES:
            frames = render(folder, opener, code)
            if not folder:
                base[code] = frames
            Image.fromarray(np.concatenate(frames, axis=1), "RGBA").save(
                os.path.join(out, f"electric_slow_{code}.png"))
        print("done", folder or "base")
    save_preview(base, os.path.join(EFFECTS, "preview_electric_slow.gif"))


if __name__ == "__main__":
    main()
