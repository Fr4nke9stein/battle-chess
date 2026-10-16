"""Знаки типов карт отдельно — для счётчиков очков в игре: венок победы, череп жертвы и
ромб маны (кристалл обычной карты белых и чёрных), те же, что в левом верхнем углу лиц
карт (`emblem_wreath`, `emblem_skull`, `emblem_gem` из breach_cards.py), с тёмным контуром
на прозрачном фоне. Игра выводит их крупнее и пишет над ними число очков.

`py tools/breach_emblems.py` — assets/breach/cards/emblem_wreath.png, emblem_skull.png,
emblem_gem_w.png, emblem_gem_b.png.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_cards as bc  # noqa: E402

sys.argv = _argv
OUT = os.path.join(art.ROOT, "assets", "breach", "cards")


def emblem(kind):
    """Знак карты типа `kind`, обрезанный по контуру."""
    t = bc.CARD_TYPES[kind]
    img = np.zeros((bc.CH, bc.CW, 4), np.uint8)
    bc.EMBLEMS[t["emblem"]](img, t, t["frame"])
    ys, xs = np.nonzero(img[..., 3])
    print(kind, "на лице карты: x", xs.min(), "…", xs.max(), ", y", ys.min(), "…", ys.max())
    return img[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]


if __name__ == "__main__":
    for name, kind in (("wreath", "victory"), ("skull", "sacrifice"), ("gem_w", "w"), ("gem_b", "b")):
        Image.fromarray(emblem(kind), "RGBA").save(os.path.join(OUT, f"emblem_{name}.png"))
    print("done", OUT)
