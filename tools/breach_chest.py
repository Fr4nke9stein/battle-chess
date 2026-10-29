"""Сундуки магазина rogue-like (0.110, просьба пользователя) в стиле игры: пиксель-арт Into the
Breach, свет сверху слева, тёмный контур, как у монеты и гербов.

- `ui/chest_pieces.png` — сундук фигур: деревянный, железные полосы, золотой замок, на крышке —
  золотая пешка;
- `ui/chest_crests.png` — сундук гербов: крашеный в тёмное бордо, золотые полосы, на крышке — щит
  в золотой кайме;
- `ui/chest_cards.png` (0.111) — сундук карт: синий, серебряные полосы, на крышке — карта в золотой
  рамке с красным ромбом;
- `ui/chest_medals.png` (0.126) — сундук медалей: тёмно-зелёный, золотые полосы, на крышке — золотая
  медаль на красной ленте.

Картинка 46×40 (сундук 44×38 и контур), тени нет. Большие (0.113) — `ui/chest_<вид>_big.png`, 55×46:
шире и выше, золотые уголки, красные камни на полосах. Превью — `ui/preview_chests.png` (×6).

    py tools/breach_chest.py [папка assets/breach/ui]
"""

import os
import sys

import numpy as np
from PIL import Image


def hexc(s):
    s = s.lstrip("#")
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "assets", "breach", "ui")
OUTLINE = hexc("#020201")
GOLD = [hexc(c) for c in ["#fffbe0", "#ffe27a", "#f5b638", "#d07d1c", "#8a4510"]]
IRON = [hexc(c) for c in ["#8a8c96", "#5a5c66", "#3a3b44", "#24252c", "#141519"]]
WOOD = [hexc(c) for c in ["#c88c4c", "#a06634", "#7c4a24", "#5a3318", "#3a200e"]]
PAINT = [hexc(c) for c in ["#8e2a3c", "#6c1c2c", "#521422", "#3a0c18", "#22060c"]]
NAVY = [hexc(c) for c in ["#3a5c96", "#2a4474", "#1e3258", "#14223e", "#0a1224"]]
SILVER = [hexc(c) for c in ["#eef2f8", "#bcc4d2", "#8c95a8", "#5e6678", "#363c4a"]]
CREAM = [hexc(c) for c in ["#fff8e6", "#e8dcc0", "#c2b494"]]
RED = [hexc(c) for c in ["#ff8a7a", "#e8402e", "#b0201a", "#701010"]]
FOREST = [hexc(c) for c in ["#4f8a52", "#3a6c3e", "#2a522e", "#1c3820", "#0e2012"]]
STONE = [hexc(c) for c in ["#b8ab8a", "#8f8468", "#6c624c", "#4a4334", "#2a261c"]]  # сундук зданий (0.133)

W, H = 44, 38
LID = 15  # нижний край крышки (строка обода)
PAWN = [
    "...###...",
    "..#####..",
    "..#####..",
    "...###...",
    "..#####..",
    "...###...",
    "...###...",
    "..#####..",
    ".#######.",
    "#########",
]
CARD = [
    "########",
    "#......#",
    "#......#",
    "#..oo..#",
    "#.oooo.#",
    "#..oo..#",
    "#......#",
    "#......#",
    "########",
]
MEDAL = [  # медаль на ленте: r — лента, # — обод, o — лицо
    ".rr...rr.",
    "..rr.rr..",
    "...rrr...",
    "..#####..",
    ".##ooo##.",
    ".#ooooo#.",
    ".#ooooo#.",
    ".##ooo##.",
    "..#####..",
]
HOUSE = [  # домик: крыша, стены, дверь (сундук зданий, 0.133)
    "....#....",
    "...###...",
    "..#####..",
    ".#######.",
    "#########",
    ".#ooooo#.",
    ".#o#o#o#.",
    ".#o#o#o#.",
    ".#######.",
]
SHIELD = [
    "#########",
    "#.......#",
    "#.......#",
    "#.......#",
    "#.......#",
    ".#.....#.",
    "..#...#..",
    "...#.#...",
    "....#....",
]


def shape():
    """Силуэт: крышка со скруглёнными верхними углами, ящик со скошенными нижними."""
    mask = np.zeros((H, W), bool)
    r = 7.0
    for y in range(H):
        for x in range(W):
            cx, cy = x + 0.5, y + 0.5
            if cy < r:
                left = cx < r and (cx - r) ** 2 + (cy - r) ** 2 > r * r
                right = cx > W - r and (cx - (W - r)) ** 2 + (cy - r) ** 2 > r * r
                if left or right:
                    continue
            if y >= H - 1 and (x < 1 or x >= W - 1):
                continue
            mask[y, x] = True
    return mask


def chest(kind, big=False):
    """Сундук `kind`; `big` (0.113) — большой: шире и выше, золотые уголки, камни на полосах."""
    global W, H, LID
    W, H, LID = (53, 44, 18) if big else (44, 38, 15)
    wood, metal = {"pieces": (WOOD, IRON), "crests": (PAINT, GOLD), "cards": (NAVY, SILVER),
                   "medals": (FOREST, GOLD), "buildings": (STONE, GOLD)}[kind]
    mask = shape()
    img = np.zeros((H, W, 4), np.uint8)

    def put(x, y, col):
        if 0 <= x < W and 0 <= y < H and mask[y, x]:
            img[y, x] = (*col, 255)

    # дерево: крышка светлее ящика, доски через строку-шов
    for y in range(H):
        for x in range(W):
            put(x, y, wood[1] if y < LID else wood[2])
    for y in [LID // 3, 2 * LID // 3] + list(range(LID + 7, H - 2, 6)):
        for x in range(W):
            put(x, y, wood[3])
    # свет слева сверху, тень справа
    for y in range(H):
        row = [x for x in range(W) if mask[y, x]]
        if not row:
            continue
        for x in row[:2]:
            put(x, y, wood[0] if y < LID else wood[1])
        for x in row[-2:]:
            put(x, y, wood[3] if y < LID else wood[4])
    for x in range(W):
        top = next((y for y in range(H) if mask[y, x]), None)
        if top is not None:
            put(x, top, wood[0])
            put(x, top + 1, wood[0] if x < W * 0.6 else wood[1])
    # обод крышки и низ ящика — металл
    for x in range(W):
        put(x, LID - 1, metal[1])
        put(x, LID, metal[2])
        put(x, LID + 1, metal[3])
        put(x, H - 2, metal[2])
        put(x, H - 1, metal[3])
    # вертикальные полосы с заклёпками
    for bx in (4, W - 8):
        for y in range(H):
            put(bx, y, metal[1])
            put(bx + 1, y, metal[2])
            put(bx + 2, y, metal[2])
            put(bx + 3, y, metal[3])
        for y in (3, LID - 6, LID + 5, H - 7):
            put(bx + 1, y, metal[0])
            put(bx + 2, y, metal[1])
            put(bx + 2, y + 1, metal[4])
    if big:
        # золотые уголки: сверху на крышке и снизу на ящике
        for cx, cy, sx, sy in ((0, 2, 1, 1), (W - 1, 2, -1, 1), (0, H - 1, 1, -1), (W - 1, H - 1, -1, -1)):
            for d in range(5):
                for k in range(5 - d):
                    put(cx + sx * k, cy + sy * d, GOLD[1] if d + k < 3 else GOLD[3])
        # камни на полосах — на крышке и на ящике
        for bx in (4, W - 8):
            for gy, gem in ((LID // 2, RED), (LID + (H - LID) // 2, RED)):
                for dx, dy, col in ((1, 0, gem[0]), (2, 0, gem[1]), (1, 1, gem[1]), (2, 1, gem[2]),
                                    (0, 0, GOLD[2]), (3, 0, GOLD[3]), (0, 1, GOLD[2]), (3, 1, GOLD[3]),
                                    (1, -1, GOLD[1]), (2, -1, GOLD[2]), (1, 2, GOLD[3]), (2, 2, GOLD[4])):
                    put(bx + dx, gy + dy, col)
    # замок на шве
    lx, ly = W // 2 - 4, LID - 3
    for y in range(ly, ly + 11):
        for x in range(lx, lx + 8):
            edge = x in (lx, lx + 7) or y in (ly, ly + 10)
            col = GOLD[4] if edge else GOLD[2]
            if not edge and (x - lx) + (y - ly) < 5:
                col = GOLD[1]
            if not edge and (x - lx) + (y - ly) > 11:
                col = GOLD[3]
            put(x, y, col)
    for y in (ly + 4, ly + 5, ly + 6, ly + 7):
        put(lx + 3, y, OUTLINE)
        put(lx + 4, y, OUTLINE)
    put(lx + 2, ly + 5, OUTLINE)
    put(lx + 5, ly + 5, OUTLINE)
    put(lx + 1, ly + 1, GOLD[0])
    # знак на крышке: пешка или щит
    pattern = {"pieces": PAWN, "crests": SHIELD, "cards": CARD, "medals": MEDAL, "buildings": HOUSE}[kind]
    ph, pw = len(pattern), len(pattern[0])
    ox, oy = W // 2 - pw // 2, max(1, (LID - 3 - ph) // 2)
    for j, line in enumerate(pattern):
        for i, ch in enumerate(line):
            x, y = ox + i, oy + j
            if kind == "pieces":
                if ch == "#":
                    put(x, y, GOLD[3] if i >= pw - 2 or j == ph - 1 else GOLD[1])
            elif kind == "medals":  # медаль: красная лента, золотой обод, светлое лицо
                if ch == "r":
                    put(x, y, RED[1] if i < pw // 2 else RED[2])
                elif ch == "#":
                    put(x, y, GOLD[2] if i < pw // 2 and j < 6 else GOLD[3])
                elif ch == "o":
                    put(x, y, GOLD[0] if i + j < 9 else GOLD[1])
            elif kind == "buildings":  # домик: золотая крыша и стены, красная кладка между ними
                if ch == "#":
                    put(x, y, GOLD[1] if j < 5 and i < pw // 2 else GOLD[2] if j < 5 else GOLD[3])
                elif ch == "o":
                    put(x, y, RED[1] if i < pw // 2 else RED[2])
            elif kind == "cards":  # карта: золотая рамка, светлое поле, красный ромб
                if ch == "#":
                    put(x, y, GOLD[1] if i == 0 or j == 0 else GOLD[3])
                elif ch == "o":
                    put(x, y, RED[1] if i < pw // 2 and j < 4 else RED[2])
                else:
                    put(x, y, CREAM[0] if i + j < 6 else CREAM[1])
            else:
                inside = "#" in line[:i] and "#" in line[i + 1:]
                if ch == "#":
                    put(x, y, GOLD[2] if i < pw // 2 or j == 0 else GOLD[3])
                elif inside:
                    put(x, y, RED[1] if i < pw // 2 + 1 and j < 4 else RED[2])
    if kind == "pieces":
        put(ox + 3, oy, GOLD[0])
        put(ox + 2, oy + 1, GOLD[0])
        put(ox + 2, oy + 2, GOLD[0])
    elif kind == "crests":
        put(ox + 2, oy + 2, RED[0])
    # тень знака на дереве — вправо вниз, где нет самого знака
    cells = set()
    for j, line in enumerate(pattern):
        for i in range(pw):
            if line[i] in "#r" or ("#" in line[:i] and "#" in line[i + 1:]):
                cells.add((i, j))
    for i, j in cells:
        if (i + 1, j + 1) not in cells:
            put(ox + i + 1, oy + j + 1, wood[4])
    # контур
    out = np.zeros((H + 2, W + 2, 4), np.uint8)
    out[1:-1, 1:-1] = img
    alpha = out[:, :, 3] > 0
    grown = alpha.copy()
    grown[1:, :] |= alpha[:-1, :]
    grown[:-1, :] |= alpha[1:, :]
    grown[:, 1:] |= alpha[:, :-1]
    grown[:, :-1] |= alpha[:, 1:]
    out[grown & ~alpha] = (*OUTLINE, 255)
    return out


def main():
    os.makedirs(OUT, exist_ok=True)
    images = []
    for big in (False, True):
        for kind in ("pieces", "crests", "cards", "medals", "buildings"):
            img = chest(kind, big)
            name = f"chest_{kind}_big.png" if big else f"chest_{kind}.png"
            Image.fromarray(img, "RGBA").save(os.path.join(OUT, name))
            images.append(img)
    h = max(i.shape[0] for i in images)
    w = max(i.shape[1] for i in images)
    n = len(images) // 2
    sheet = Image.new("RGBA", (n * (w + 4) + 4, 2 * (h + 4) + 4), (24, 28, 41, 255))
    for k, img in enumerate(images):
        part = Image.fromarray(img, "RGBA")
        sheet.alpha_composite(part, (4 + (k % n) * (w + 4), 4 + (k // n) * (h + 4) + h - img.shape[0]))
    sheet.resize((sheet.width * 6, sheet.height * 6), Image.NEAREST).save(
        os.path.join(OUT, "preview_chests.png"))


if __name__ == "__main__":
    main()
