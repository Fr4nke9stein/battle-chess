"""Элементы интерфейса в стиле Into the Breach: кнопка «Конец хода».

Кнопка — панель как рамки карт (контур, кромка 2px с фаской, тёмная заливка), надпись в
две строки пиксельным шрифтом игры (6×8, толстые линии, тёмный контур и тень снизу);
кириллица дорисована в том же стиле. Состояния: normal, hover (кромка цвета выделения),
pressed (фаска наоборот, надпись ниже на пиксель), disabled (ход соперника), ready
(янтарная кромка — ходить больше нечем, пора закончить ход).

`py tools/breach_ui.py [папка]` — по умолчанию assets/breach/ui: end_turn_<состояние>.png
(1x, тень уже в картинке), preview_ui.png.
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
_argv, sys.argv = sys.argv, sys.argv[:1]  # генераторы читают папку вывода из argv
import breach_art as art  # noqa: E402
import breach_cards as bc  # noqa: E402
from breach_art import hexc  # noqa: E402

sys.argv = _argv
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(art.ROOT, "assets", "breach", "ui")

# Шрифт как GLYPHS в src/assets.rs: 6×8, линии в 2 пикселя; у Ц и Д — выносной элемент
# девятой строкой ниже строки.
FONT = {
    "А": [".####.", "##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##"],
    "Д": ["..####.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", "##..##.", "#######",
          "##...##"],
    "Е": ["######", "##....", "##....", "#####.", "##....", "##....", "##....", "######"],
    "К": ["##..##", "##.##.", "####..", "###...", "####..", "##.##.", "##..##", "##..##"],
    "Н": ["##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##", "##..##"],
    "О": [".####.", "##..##", "##..##", "##..##", "##..##", "##..##", "##..##", ".####."],
    "Х": ["##..##", "##..##", ".####.", "..##..", "..##..", ".####.", "##..##", "##..##"],
    "Ц": ["##..##.", "##..##.", "##..##.", "##..##.", "##..##.", "##..##.", "##..##.", "#######",
          ".....##"],
    " ": ["...", "...", "...", "...", "...", "...", "...", "..."],
}
LINE_STEP = 11  # от верха строки до верха следующей: 8 строк знака, выносной, зазор
OUTLINE = "#08090d"


def text_mask(lines):
    """Маска надписи (строки по центру), с запасом в пиксель под контур и два под тень."""
    widths = [sum(len(FONT[ch][0]) + 1 for ch in line) - 1 for line in lines]
    w = max(widths) + 2
    h = LINE_STEP * (len(lines) - 1) + 9 + 3
    body = np.zeros((h, w), bool)
    for n, line in enumerate(lines):
        x = 1 + (max(widths) - widths[n]) // 2
        y = 1 + n * LINE_STEP
        for ch in line:
            rows = FONT[ch]
            for dy, row in enumerate(rows):
                for dx, c in enumerate(row):
                    if c == "#":
                        body[y + dy, x + dx] = True
            x += len(rows[0]) + 1
    return body


def text_image(lines, light, shade):
    """Надпись как знаки игры: светлое тело, нижний пиксель линий темнее, контур и тень."""
    body = text_mask(lines)
    img = np.zeros(body.shape + (4,), np.uint8)
    below2 = np.zeros_like(body)
    below2[2:] = body[:-2]
    dark = bc.outline4(body) | (below2 & ~body)
    img[dark] = (*hexc(OUTLINE), 255)
    img[body] = (*hexc(light), 255)
    bottom = body & ~np.vstack([body[1:], np.zeros((1, body.shape[1]), bool)])
    img[bottom] = (*hexc(shade), 255)
    return img


STATES = {
    # кромка (контур, тёмная, средняя, светлая, блик), заливка, надпись (свет, тень)
    "normal": (bc.FACTIONS["w"]["frame"], "#22222a", ("#e9ecf2", "#a9b0bf")),
    "hover": (bc.SELECT, "#2a2f3d", ("#ffffff", "#c9d2c0")),
    "pressed": (bc.FACTIONS["w"]["frame"], "#181c29", ("#d3d8e2", "#959cab")),
    "disabled": (["#0d0f17", "#1f2533", "#2a3140", "#353e50", "#465066"], "#1b1e27", ("#6b7280", "#4b5160")),
    "ready": (["#1a0d02", "#8a4a08", "#e47a1b", "#ffbc03", "#ffeba1"], "#2a2418", ("#fff8ce", "#e0b860")),
}
LABEL = ["КОНЕЦ", "ХОДА"]
PAD_X, PAD_Y = 4, 2


def end_turn(state="normal"):
    """Кнопка «Конец хода» без тени."""
    frame, fill, (light, shade) = STATES[state]
    text = text_image(LABEL, light, shade)
    th, tw = text.shape[:2]
    w, h = tw + 2 * PAD_X + 8, th + 2 * PAD_Y + 8
    img = np.zeros((h, w, 4), np.uint8)
    d, side = bc.rings(w, h)
    bc.paint(img, d >= 4, fill)
    pressed = state == "pressed"
    bc.bevel_frame(img, d, -side if pressed else side, frame)
    # верхняя строка заливки чуть светлее — панель выпуклая, у нажатой — темнее
    yy = np.mgrid[0:h, 0:w][0]
    bc.paint(img, (d == 4) & (yy == 4), "#2c3242" if not pressed else "#11141f")
    x0, y0 = 4 + PAD_X, 4 + PAD_Y + (1 if pressed else 0)
    m = text[..., 3] > 0
    region = img[y0 : y0 + th, x0 : x0 + tw]
    region[m[: region.shape[0]]] = text[: region.shape[0]][m[: region.shape[0]]]
    return img


def with_shadow(img, pressed=False):
    """Кнопка с жёсткой тенью 30% вправо-вниз (у нажатой тень короче)."""
    dx, dy = (1, 1) if pressed else (2, 3)
    out = bc.shadow_of(img, dx, dy)
    m = img[..., 3] > 0
    out[: img.shape[0], : img.shape[1]][m] = img[m]
    return out


def button_png(state):
    return with_shadow(end_turn(state), pressed=state == "pressed")


def preview(scale=4):
    pics = [button_png(s) for s in STATES]
    gap = 10
    w = sum(p.shape[1] for p in pics) + gap * (len(pics) + 1)
    h = max(p.shape[0] for p in pics) + 2 * gap
    canvas = Image.new("RGBA", (w, h), hexc("#1b1e27") + (255,))
    x = gap
    for p in pics:
        canvas.alpha_composite(Image.fromarray(p, "RGBA"), (x, gap))
        x += p.shape[1] + gap
    return canvas.resize((w * scale, h * scale), Image.NEAREST)


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for state in STATES:
        Image.fromarray(button_png(state), "RGBA").save(os.path.join(OUT, f"end_turn_{state}.png"))
    preview().save(os.path.join(OUT, "preview_ui.png"))
    print("done", OUT, end_turn().shape[1::-1])
