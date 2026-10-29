"""Значки зданий города для колонки купленных (с 0.133): те же здания `breach_city.py`, отрисованные тем же
лучевым движком `breach_castle.py`, только мельче — масштаб подбирается, чтобы здание вписалось в `FIT` пикселей
(не крупнее `K_MAX`). Без тени, обрезаны по непрозрачному.

`py tools/breach_city_icons.py [имя …]` — в assets/breach/city: `icon_<имя>.png`, превью `preview_icons.png`.
"""
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_castle as bc  # noqa: E402
import breach_city as city  # noqa: E402

sys.argv = _argv
FIT = 42
K_MAX = 0.6


def icon(name):
    k = K_MAX
    for _ in range(2):
        bc.K = k
        body, _, _ = bc.render(bc.Scene(city.BUILDINGS[name]()), ss=3)
        box = body.getbbox()
        size = max(box[2] - box[0], box[3] - box[1])
        if size <= FIT:
            break
        k *= FIT / size * 0.98
    return body.crop(body.getbbox())


def main():
    names = sys.argv[1:] or list(city.BUILDINGS)
    images = []
    for name in names:
        img = icon(name)
        img.save(os.path.join(city.OUT, f"icon_{name}.png"))
        print(name, img.size, flush=True)
        images.append(img)
    w = sum(i.width for i in images) + 4 * (len(images) + 1)
    h = max(i.height for i in images) + 8
    sheet = Image.new("RGBA", (w, h), (24, 28, 41, 255))
    x = 4
    for img in images:
        sheet.alpha_composite(img, (x, h - 4 - img.height))
        x += img.width + 4
    sheet.resize((w * 4, h * 4), Image.NEAREST).save(os.path.join(city.OUT, "preview_icons.png"))


if __name__ == "__main__":
    main()
