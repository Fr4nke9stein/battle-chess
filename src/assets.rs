//! Графика игры. Доска и фигуры — пиксель-арт в духе Into the Breach из `assets/breach`
//! (его рисует `tools/breach_art.py`): изометрические клетки-блоки с земляными боками,
//! белые — светлые мехи, чёрные — веки, у каждой фигуры отдельный слой тени. Здесь
//! картинка доски собирается из клеток вместе с буквами координат, рисуются подсветки
//! клеток, и всё увеличивается до текстур нужного размера. Значок окна — конь cburnett.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::{
    self, ColorImage, IconData, TextureFilter, TextureHandle, TextureId, TextureOptions, Vec2, vec2,
};
use resvg::{tiny_skia, usvg};

use crate::game::{Color, Kind};

/// Ромб верхней грани клетки, пиксели арта.
pub const TILE: Vec2 = vec2(56.0, 42.0);
/// Картинка доски: восемь ромбов по ширине, по высоте — до низа передней клетки.
pub const BOARD: Vec2 = vec2(448.0, 368.0);
/// Холст фигуры и точка на нём, которая ставится в центр клетки.
pub const SPRITE: Vec2 = vec2(64.0, 84.0);
pub const ANCHOR: Vec2 = vec2(32.0, 66.0);

macro_rules! breach {
    ($($name:literal),* $(,)?) => {
        [$(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/breach/", $name)) as &[u8]),*]
    };
}

/// Фигуры: белые пешка, конь, слон, ладья, ферзь, король, затем так же чёрные.
const BODIES: [&[u8]; 12] = breach!(
    "pieces/wP.png",
    "pieces/wN.png",
    "pieces/wB.png",
    "pieces/wR.png",
    "pieces/wQ.png",
    "pieces/wK.png",
    "pieces/bP.png",
    "pieces/bN.png",
    "pieces/bB.png",
    "pieces/bR.png",
    "pieces/bQ.png",
    "pieces/bK.png",
);
/// Фигуры в инее (`tools/breach_freeze.py`) — в том же порядке и на тех же холстах.
const FROZEN: [&[u8]; 12] = breach!(
    "pieces/frozen/wP.png",
    "pieces/frozen/wN.png",
    "pieces/frozen/wB.png",
    "pieces/frozen/wR.png",
    "pieces/frozen/wQ.png",
    "pieces/frozen/wK.png",
    "pieces/frozen/bP.png",
    "pieces/frozen/bN.png",
    "pieces/frozen/bB.png",
    "pieces/frozen/bR.png",
    "pieces/frozen/bQ.png",
    "pieces/frozen/bK.png",
);
/// Кромка инея (`ICE[0]` генератора): белая полоса, ползущая вверх по фигуре.
const FROST_EDGE: [u8; 3] = [0xf2, 0xfb, 0xff];
const SHADOWS: [&[u8]; 12] = breach!(
    "pieces/wP_shadow.png",
    "pieces/wN_shadow.png",
    "pieces/wB_shadow.png",
    "pieces/wR_shadow.png",
    "pieces/wQ_shadow.png",
    "pieces/wK_shadow.png",
    "pieces/bP_shadow.png",
    "pieces/bN_shadow.png",
    "pieces/bB_shadow.png",
    "pieces/bR_shadow.png",
    "pieces/bQ_shadow.png",
    "pieces/bK_shadow.png",
);
/// Клетки: по четыре варианта тёмных (трава) и светлых (песок).
const TILES: [&[u8]; 8] = breach!(
    "tiles/dark_0.png",
    "tiles/dark_1.png",
    "tiles/dark_2.png",
    "tiles/dark_3.png",
    "tiles/light_0.png",
    "tiles/light_1.png",
    "tiles/light_2.png",
    "tiles/light_3.png",
);
/// Курсоры в порядке `Pointer` (рисует `tools/breach_cursor.py`).
/// Курсор удара (`cursor_attack.png`) с 0.32 не нужен: ходом не бьют.
const CURSORS: [&[u8]; 5] = breach!(
    "cursor/cursor_default.png",
    "cursor/cursor_hover.png",
    "cursor/cursor_move.png",
    "cursor/cursor_card.png",
    "cursor/cursor_off.png",
);
/// Холст курсора — стрелка-дротик с тенью внутри; точка нажатия — кончик стрелки.
pub const CURSOR: Vec2 = vec2(22.0, 24.0);
pub const CURSOR_HOTSPOT: Vec2 = vec2(1.0, 1.0);

/// Вид курсора в игре.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointer {
    /// Обычная стрелка.
    Default,
    /// Над своей фигурой, которую можно выбрать: кромка цвета выделения.
    Hover,
    /// Сюда можно сходить: голубой ромб-клетка.
    Move,

    /// Карту можно взять и перетащить: мини-карта.
    Card,
    /// Нельзя: серый ромб, перечёркнутый красным.
    Off,
}

mod cards;
mod ui;

pub use cards::{
    BURN_CARD_AT, BURN_FRAME, BURN_FRAME_TIME, BURN_FRAMES, CARD, COST_TINT, CardPic, DECK_SKINS,
    EMBLEM_ZOOM, POINTS_TINT, SHADOW_SHIFT, Sprite, cost_corner,
};
pub use ui::{
    ARROW_HEAD, ARROW_SPACING, Aim, ArrowPart, BLESS_ANCHOR, BLESS_FIGURE, BLESS_FRAME_TIME,
    BLESS_FRAMES, BUTTON, Button, ELECTRIC_ANCHOR, ELECTRIC_FLASH, ELECTRIC_FRAME_TIME,
    ELECTRIC_LOOP, FREEZE_CENTER, FREEZE_CLINK, FREEZE_DELAY, FREEZE_FRAME_TIME, FREEZE_FRAMES,
    FREEZE_FROST, FrameSprite, LAVA_FRAME_TIME, LAVA_FRAMES, LAVA_HOLD, LAVA_IGNITE, LAVA_SPREAD,
    LAVA_WAVE_AT, PORTAL_CENTER, PORTAL_CUT, PORTAL_FRAME, PORTAL_FRAME_TIME, PORTAL_FRAMES,
    PORTAL_RADII, PORTAL_SCALE, PORTAL_SINK, TARGET_PAD, WAVE_CENTER, WAVE_FRAME, WAVE_FRAME_TIME,
    WAVE_FRAMES,
};

/// Значки в панелях увеличены вдвое: так они остаются резкими и в крупной карточке фигуры.
const ICON_SCALE: usize = 2;

fn index(color: Color, kind: Kind) -> usize {
    color as usize * 6 + kind as usize
}

/// Подсветка клетки: полупрозрачный ромб с пиксельной рамкой, как в Into the Breach.
#[derive(Clone, Copy)]
pub enum Marker {
    /// Выбранная фигура.
    Selected,
    /// Откуда и куда пошла фигура последним ходом.
    LastMove,
    /// Ход на пустую клетку.
    Move,
    /// Удар.
    Attack,
    /// Клетка под курсором, куда можно пойти или ударить (поверх хода или удара).
    Target,
    /// Шах: короля могут убить.
    Check,
    /// Угроза королю.
    Threat,
    /// Сплошной белый ромб — заливка клетки любым цветом.
    Fill,
    /// Клетка, на которую ляжет лёд карты (буря, лавина, ледяной барьер).
    Ice,
}

impl Marker {
    const ALL: [Marker; 9] = [
        Marker::Selected,
        Marker::LastMove,
        Marker::Move,
        Marker::Attack,
        Marker::Target,
        Marker::Check,
        Marker::Threat,
        Marker::Fill,
        Marker::Ice,
    ];

    fn image(self) -> Pixels {
        // Заливка, рамка, непрозрачность заливки, отступ от края ромба и толщина рамки.
        let (fill, line, alpha, inset, width) = match self {
            Marker::Selected => (0xffe27a, 0xfff3b8, 70, 1.0, 2.0),
            Marker::LastMove => (0xffe27a, 0xffe27a, 45, 3.0, 1.0),
            // Плотнее, чем на превью генератора: там ходы были только по траве, а на
            // песке синий при 95 выходит серым.
            Marker::Move => (0x2f8fe0, 0x7fd4ff, 125, 2.0, 2.0),
            Marker::Attack => (0xe0452f, 0xff9a6b, 110, 2.0, 2.0),
            Marker::Target => (0xffffff, 0xffffff, 0, 1.0, 1.0),
            Marker::Check => (0xff2a1a, 0xffb08a, 140, 1.0, 2.0),
            Marker::Threat => (0xe68a2e, 0xffc35a, 100, 2.0, 2.0),
            Marker::Fill => (0xffffff, 0xffffff, 255, 0.0, 0.0),
            // Цвета льда из breach_freeze.py: заливка `ICE[2]`, кромка `ICE[0]`.
            Marker::Ice => (0x8fcbe6, 0xf2fbff, 120, 1.0, 1.0),
        };
        overlay(rgb(fill), rgb(line), alpha, inset, width)
    }
}

const fn rgb(hex: u32) -> [u8; 3] {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8]
}

/// Картинка с обычной (не умноженной на прозрачность) альфой.
#[derive(Clone)]
struct Pixels {
    width: usize,
    height: usize,
    rgba: Vec<[u8; 4]>,
}

impl Pixels {
    fn new(width: usize, height: usize) -> Pixels {
        Pixels {
            width,
            height,
            rgba: vec![[0; 4]; width * height],
        }
    }

    fn decode(png: &[u8]) -> Pixels {
        let pixmap = tiny_skia::Pixmap::decode_png(png).expect("встроенная картинка не читается");
        let rgba = pixmap
            .pixels()
            .iter()
            .map(|pixel| {
                let c = pixel.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        Pixels {
            width: pixmap.width() as usize,
            height: pixmap.height() as usize,
            rgba,
        }
    }

    fn alpha(&self, x: usize, y: usize) -> u8 {
        self.rgba[y * self.width + x][3]
    }

    fn set(&mut self, x: usize, y: usize, rgb: [u8; 3], alpha: u8) {
        self.rgba[y * self.width + x] = [rgb[0], rgb[1], rgb[2], alpha];
    }

    /// Кладёт `top` левым верхним углом в точку (`x0`, `y0`), смешивая по прозрачности.
    fn paste(&mut self, top: &Pixels, x0: i32, y0: i32) {
        for y in 0..top.height {
            for x in 0..top.width {
                let (tx, ty) = (x0 + x as i32, y0 + y as i32);
                if tx < 0 || ty < 0 || tx as usize >= self.width || ty as usize >= self.height {
                    continue;
                }
                let src = top.rgba[y * top.width + x];
                let dst = &mut self.rgba[ty as usize * self.width + tx as usize];
                let (sa, da) = (src[3] as f32 / 255.0, dst[3] as f32 / 255.0);
                let out = sa + da * (1.0 - sa);
                if out <= 0.0 {
                    continue;
                }
                for c in 0..3 {
                    let mixed = (src[c] as f32 * sa + dst[c] as f32 * da * (1.0 - sa)) / out;
                    dst[c] = mixed.round() as u8;
                }
                dst[3] = (out * 255.0).round() as u8;
            }
        }
    }

    /// Границы непрозрачной части: левый верхний угол и размер.
    fn bounds(&self) -> ([usize; 2], [usize; 2]) {
        let (mut min, mut max) = ([self.width, self.height], [0, 0]);
        for y in 0..self.height {
            for x in 0..self.width {
                if self.alpha(x, y) > 0 {
                    min = [min[0].min(x), min[1].min(y)];
                    max = [max[0].max(x + 1), max[1].max(y + 1)];
                }
            }
        }
        if max[0] <= min[0] {
            return ([0, 0], [self.width, self.height]);
        }
        (min, [max[0] - min[0], max[1] - min[1]])
    }

    fn crop(&self, min: [usize; 2], size: [usize; 2]) -> Pixels {
        let mut out = Pixels::new(size[0], size[1]);
        for y in 0..size[1] {
            for x in 0..size[0] {
                out.rgba[y * size[0] + x] = self.rgba[(min[1] + y) * self.width + min[0] + x];
            }
        }
        out
    }

    /// Увеличение в `k` раз без сглаживания: пиксели арта остаются квадратными, а
    /// текстура при выводе лишь немного уменьшается — края пикселей чёткие.
    fn upscale(&self, k: usize) -> ColorImage {
        // Цвет умножается на прозрачность прямо в sRGB, как смешивает и генератор арта:
        // `from_rgba_unmultiplied` делает это в линейном пространстве, и полупрозрачные
        // подсветки выходят заметно светлее.
        let premultiplied: Vec<[u8; 4]> = self
            .rgba
            .iter()
            .map(|&[r, g, b, a]| {
                let mul = |c: u8| ((c as u32 * a as u32 + 127) / 255) as u8;
                [mul(r), mul(g), mul(b), a]
            })
            .collect();
        let (width, height) = (self.width * k, self.height * k);
        let mut bytes = Vec::with_capacity(width * height * 4);
        for y in 0..height {
            let row = &premultiplied[(y / k) * self.width..][..self.width];
            for x in 0..width {
                bytes.extend_from_slice(&row[x / k]);
            }
        }
        ColorImage::from_rgba_premultiplied([width, height], &bytes)
    }
}

/// Кромка инея на фигуре `body` цветом `FROST_EDGE`: её светлые (не контурные) пиксели, а с
/// `crest` — только те из них, над которыми прозрачно (верхние края силуэта).
fn frost_edge(body: &Pixels, crest: bool) -> Pixels {
    let mut edge = Pixels::new(body.width, body.height);
    for y in 0..body.height {
        for x in 0..body.width {
            let [r, g, b, a] = body.rgba[y * body.width + x];
            let light = a > 0 && r as u32 + g as u32 + b as u32 >= 60;
            let open = y == 0 || body.alpha(x, y - 1) == 0;
            if light && (!crest || open) {
                edge.set(x, y, FROST_EDGE, 255);
            }
        }
    }
    edge
}

/// Ромб размером с верхнюю грань клетки, ужатый на `inset` пикселей.
fn diamond_mask(inset: f32) -> Vec<bool> {
    let (w, h) = (TILE.x, TILE.y);
    let mut mask = Vec::with_capacity((w * h) as usize);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as f32 + 0.5 - w / 2.0).abs() / (w / 2.0 - inset * 4.0 / 3.0);
            let dy = (y as f32 + 0.5 - h / 2.0).abs() / (h / 2.0 - inset);
            mask.push(dx + dy <= 1.0);
        }
    }
    mask
}

fn overlay(fill: [u8; 3], line: [u8; 3], alpha: u8, inset: f32, width: f32) -> Pixels {
    let outer = diamond_mask(inset);
    let inner = diamond_mask(inset + width);
    let mut image = Pixels::new(TILE.x as usize, TILE.y as usize);
    for (i, (&o, &n)) in outer.iter().zip(&inner).enumerate() {
        let (x, y) = (i % image.width, i / image.width);
        if o && !n {
            image.set(x, y, line, 255);
        } else if o {
            image.set(x, y, fill, alpha);
        }
    }
    image
}

/// Пиксельный шрифт: буквы и цифры 6×8 с толстыми линиями, как подписи доски, и
/// значки для чисел над фигурами. `x` — череп, `<` — стрелка ответного удара, `!` — меч,
/// `h` — сердце, `c` — карта с ромбом, как на рубашке колоды.
const GLYPHS: [(u8, [&str; 8]); 25] = [
    (
        b'A',
        [
            ".####.", "##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##",
        ],
    ),
    (
        b'B',
        [
            "#####.", "##..##", "##..##", "#####.", "##..##", "##..##", "##..##", "#####.",
        ],
    ),
    (
        b'C',
        [
            ".####.", "##..##", "##....", "##....", "##....", "##....", "##..##", ".####.",
        ],
    ),
    (
        b'D',
        [
            "#####.", "##..##", "##..##", "##..##", "##..##", "##..##", "##..##", "#####.",
        ],
    ),
    (
        b'E',
        [
            "######", "##....", "##....", "#####.", "##....", "##....", "##....", "######",
        ],
    ),
    (
        b'F',
        [
            "######", "##....", "##....", "#####.", "##....", "##....", "##....", "##....",
        ],
    ),
    (
        b'G',
        [
            ".####.", "##..##", "##....", "##....", "##.###", "##..##", "##..##", ".####.",
        ],
    ),
    (
        b'H',
        [
            "##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##", "##..##",
        ],
    ),
    (
        b'0',
        [
            ".####.", "##..##", "##..##", "##..##", "##..##", "##..##", "##..##", ".####.",
        ],
    ),
    (
        b'1',
        [
            "..##..", ".###..", "####..", "..##..", "..##..", "..##..", "..##..", "######",
        ],
    ),
    (
        b'2',
        [
            ".####.", "##..##", "....##", "...##.", "..##..", ".##...", "##....", "######",
        ],
    ),
    (
        b'3',
        [
            ".####.", "##..##", "....##", "..###.", "....##", "....##", "##..##", ".####.",
        ],
    ),
    (
        b'4',
        [
            "...##.", "..###.", ".####.", "##.##.", "######", "...##.", "...##.", "...##.",
        ],
    ),
    (
        b'5',
        [
            "######", "##....", "##....", "#####.", "....##", "....##", "##..##", ".####.",
        ],
    ),
    (
        b'6',
        [
            ".####.", "##....", "##....", "#####.", "##..##", "##..##", "##..##", ".####.",
        ],
    ),
    (
        b'7',
        [
            "######", "....##", "...##.", "...##.", "..##..", "..##..", ".##...", ".##...",
        ],
    ),
    (
        b'8',
        [
            ".####.", "##..##", "##..##", ".####.", "##..##", "##..##", "##..##", ".####.",
        ],
    ),
    (
        b'9',
        [
            ".####.", "##..##", "##..##", "##..##", ".#####", "....##", "....##", ".####.",
        ],
    ),
    (
        b'-',
        [
            ".....", ".....", ".....", "#####", "#####", ".....", ".....", ".....",
        ],
    ),
    (
        b'+',
        [
            "......", "..##..", "..##..", "######", "######", "..##..", "..##..", "......",
        ],
    ),
    (
        b'x',
        [
            ".#####.", "#######", "#..#..#", "#..#..#", "#######", ".#####.", ".#.#.#.", ".......",
        ],
    ),
    (
        b'<',
        [
            "...#...", "..##...", ".######", "#######", ".######", "..##...", "...#...", ".......",
        ],
    ),
    (
        b'!',
        [
            "......##", ".....###", "....###.", ".#.###..", "..###...", "..##....", ".#..#...",
            "#.......",
        ],
    ),
    (
        b'h',
        [
            ".##.##.", "#######", "#######", "#######", ".#####.", "..###..", "...#...", ".......",
        ],
    ),
    (
        b'c',
        [
            "#######", "###.###", "##...##", "#.....#", "##...##", "###.###", "#######", "#######",
        ],
    ),
];

/// Знак с тёмным контуром и тенью снизу; нижний пиксель каждой линии темнее.
/// Картинка на пиксель шире знака с каждой стороны, сверху — на пиксель, снизу — на два.
fn glyph(ch: u8, light: u32, shade: u32) -> Pixels {
    let rows = GLYPHS
        .iter()
        .find(|(c, _)| *c == ch)
        .map(|(_, rows)| rows)
        .expect("знак есть в шрифте");
    let (w, h) = (rows[0].len() + 2, 11);
    let mut body = vec![false; w * h];
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.bytes().enumerate() {
            body[(y + 1) * w + x + 1] = c == b'#';
        }
    }
    let at = |x: i32, y: i32| {
        x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && {
            body[y as usize * w + x as usize]
        }
    };
    let mut image = Pixels::new(w, h);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let color = if at(x, y) {
                if at(x, y + 1) { light } else { shade }
            } else if [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (1, -1), (2, 0)]
                .iter()
                .any(|&(dy, dx)| at(x - dx, y - dy))
            {
                0x08090d
            } else {
                continue;
            };
            image.set(x as usize, y as usize, rgb(color), 255);
        }
    }
    image
}

/// Знак пиксельного шрифта для чисел над фигурами.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Digit(u8),
    Minus,
    Plus,
    Skull,

    /// Атака.
    Sword,
    /// Здоровье.
    Heart,
    /// Карты в колоде.
    Card,
}

/// Знаки шрифта в порядке текстур.
const FONT: &[u8] = b"0123456789-+x<!hc";

impl Glyph {
    fn char(self) -> u8 {
        match self {
            Glyph::Digit(d) => b'0' + d.min(9),
            Glyph::Minus => b'-',
            Glyph::Plus => b'+',
            Glyph::Skull => b'x',

            Glyph::Sword => b'!',
            Glyph::Heart => b'h',
            Glyph::Card => b'c',
        }
    }

    fn index(self) -> usize {
        FONT.iter()
            .position(|&c| c == self.char())
            .expect("знак есть в шрифте")
    }

    /// Цифры неотрицательного числа.
    pub fn digits(n: i32) -> Vec<Glyph> {
        n.max(0)
            .to_string()
            .bytes()
            .map(|b| Glyph::Digit(b - b'0'))
            .collect()
    }
}

/// Высота знака шрифта (без контура), пиксели.
pub const GLYPH_HEIGHT: f32 = 8.0;
/// Знаки шрифта увеличены ещё вдвое против доски: всплывая, числа подрастают.
const FONT_SCALE: usize = 2;

/// Левый верхний угол блока клетки (`i`, `j`) на картинке доски: `i` растёт вправо-вниз,
/// `j` — влево-вниз, клетка (0, 0) — верхняя.
fn cell_origin(i: usize, j: usize) -> (i32, i32) {
    let half = (TILE.x as i32 / 2, TILE.y as i32 / 2);
    (
        (i as i32 - j as i32) * half.0 + (BOARD.x as i32 - TILE.x as i32) / 2,
        (i + j) as i32 * half.1,
    )
}

/// Какая клетка шахмат (вертикаль, горизонталь) стоит на месте (`i`, `j`). Белые — внизу
/// слева, а у перевёрнутой доски — чёрные.
fn square_of(i: usize, j: usize, flipped: bool) -> (usize, usize) {
    if flipped { (7 - i, j) } else { (i, 7 - j) }
}

/// Доска целиком: клетки от дальних к ближним (передние закрывают бока задних), буквы
/// под левым передним краем, цифры — под правым.
fn board_image(tiles: &[Pixels], flipped: bool) -> Pixels {
    let mut image = Pixels::new(BOARD.x as usize, BOARD.y as usize);
    let mut cells: Vec<(usize, usize)> = (0..8).flat_map(|i| (0..8).map(move |j| (i, j))).collect();
    cells.sort_by_key(|&(i, j)| i + j);
    for (i, j) in cells {
        let (file, rank) = square_of(i, j, flipped);
        let light = (file + rank) % 2 == 1;
        let variant = (file * 7 + rank * 3) % 4;
        let (x, y) = cell_origin(i, j);
        image.paste(&tiles[light as usize * 4 + variant], x, y);
    }
    for k in 0..8 {
        let (file, _) = square_of(k, 7, flipped);
        let (x, y) = cell_origin(k, 7);
        image.paste(&glyph(b"ABCDEFGH"[file], 0xe9ecf2, 0xa9b0bf), x + 9, y + 61);
        let (_, rank) = square_of(7, k, flipped);
        let (x, y) = cell_origin(7, k);
        image.paste(
            &glyph(b"12345678"[rank], 0xe9ecf2, 0xa9b0bf),
            x + 39,
            y + 61,
        );
    }
    image
}

/// Текстуры доски, фигур и подсветок, увеличенные в `scale` раз.
pub struct Textures {
    scale: u32,
    boards: [TextureHandle; 2],
    bodies: Vec<TextureHandle>,
    frozen: Vec<TextureHandle>,
    frost_rims: Vec<TextureHandle>,
    frost_crests: Vec<TextureHandle>,
    shadows: Vec<TextureHandle>,
    markers: Vec<TextureHandle>,
    font: Vec<TextureHandle>,
    pub cards: cards::CardTextures,
    pub ui: ui::UiTextures,
}

impl Textures {
    pub fn board(&self, flipped: bool) -> TextureId {
        self.boards[flipped as usize].id()
    }

    pub fn body(&self, color: Color, kind: Kind) -> TextureId {
        self.bodies[index(color, kind)].id()
    }

    pub fn shadow(&self, color: Color, kind: Kind) -> TextureId {
        self.shadows[index(color, kind)].id()
    }

    /// Фигура в инее — на том же холсте, что и обычная.
    pub fn frozen(&self, color: Color, kind: Kind) -> TextureId {
        self.frozen[index(color, kind)].id()
    }

    /// Кромка инея, пока он поднимается по фигуре: `crest` — верхние края силуэта (белые
    /// пиксели под прозрачными), иначе — все светлые пиксели фигуры (для строки самой кромки).
    pub fn frost_edge(&self, color: Color, kind: Kind, crest: bool) -> TextureId {
        let edges = if crest {
            &self.frost_crests
        } else {
            &self.frost_rims
        };
        edges[index(color, kind)].id()
    }

    pub fn marker(&self, marker: Marker) -> TextureId {
        self.markers[marker as usize].id()
    }

    /// Знак шрифта: белый с тёмным контуром, красится оттенком при выводе.
    pub fn glyph(&self, glyph: Glyph) -> TextureId {
        self.font[glyph.index()].id()
    }
}

pub struct Art {
    bodies: Vec<Pixels>,
    frozen: Vec<Pixels>,
    /// Кромка инея: светлые пиксели фигуры и верхние края её силуэта (`frost_reveal` в
    /// breach_freeze.py).
    frost_rims: Vec<Pixels>,
    frost_crests: Vec<Pixels>,
    shadows: Vec<Pixels>,
    boards: [Pixels; 2],
    markers: Vec<Pixels>,
    font: Vec<Pixels>,
    cards: cards::CardArt,
    ui: ui::UiArt,
    cursors: Vec<Pixels>,
    /// Курсоры при одном увеличении: оно своё, целое, чтобы стрелка была резкой.
    cursor_textures: RefCell<Option<(u32, Vec<TextureHandle>)>>,
    /// Первая непрозрачная строка каждой фигуры — над ней ставятся числа.
    tops: Vec<usize>,
    textures: RefCell<Option<Rc<Textures>>>,
    icons: RefCell<Vec<(TextureHandle, Vec2)>>,
}

impl Art {
    pub fn new() -> Art {
        let bodies: Vec<Pixels> = BODIES.iter().map(|png| Pixels::decode(png)).collect();
        let tiles: Vec<Pixels> = TILES.iter().map(|png| Pixels::decode(png)).collect();
        Art {
            tops: bodies.iter().map(|body| body.bounds().0[1]).collect(),
            frozen: FROZEN.iter().map(|png| Pixels::decode(png)).collect(),
            frost_rims: bodies.iter().map(|body| frost_edge(body, false)).collect(),
            frost_crests: bodies.iter().map(|body| frost_edge(body, true)).collect(),
            shadows: SHADOWS.iter().map(|png| Pixels::decode(png)).collect(),
            boards: [board_image(&tiles, false), board_image(&tiles, true)],
            markers: Marker::ALL.iter().map(|m| m.image()).collect(),
            font: FONT.iter().map(|&c| glyph(c, 0xffffff, 0xb4b4b4)).collect(),
            cards: cards::CardArt::new(),
            ui: ui::UiArt::new(),
            cursors: CURSORS.iter().map(|png| Pixels::decode(png)).collect(),
            cursor_textures: RefCell::new(None),
            bodies,
            textures: RefCell::new(None),
            icons: RefCell::new(Vec::new()),
        }
    }

    /// Текстуры, где пиксель арта — `scale`×`scale` пикселей.
    pub fn textures(&self, ctx: &egui::Context, scale: u32) -> Rc<Textures> {
        let scale = scale.clamp(1, 8);
        let mut cache = self.textures.borrow_mut();
        if let Some(textures) = cache.as_ref().filter(|t| t.scale == scale) {
            return Rc::clone(textures);
        }
        let k = scale as usize;
        let load = |name: String, image: &Pixels| {
            ctx.load_texture(name, image.upscale(k), TextureOptions::LINEAR)
        };
        let all = |name: &str, images: &[Pixels]| -> Vec<TextureHandle> {
            images
                .iter()
                .enumerate()
                .map(|(i, image)| load(format!("{name}-{i}-x{scale}"), image))
                .collect()
        };
        let textures = Rc::new(Textures {
            scale,
            boards: [
                load(format!("board-x{scale}"), &self.boards[0]),
                load(format!("board-flipped-x{scale}"), &self.boards[1]),
            ],
            bodies: all("piece", &self.bodies),
            frozen: all("frozen-piece", &self.frozen),
            frost_rims: all("frost-rim", &self.frost_rims),
            frost_crests: all("frost-crest", &self.frost_crests),
            shadows: all("shadow", &self.shadows),
            markers: all("marker", &self.markers),
            font: self
                .font
                .iter()
                .enumerate()
                .map(|(i, image)| {
                    let image = image.upscale(k * FONT_SCALE);
                    ctx.load_texture(format!("glyph-{i}-x{scale}"), image, TextureOptions::LINEAR)
                })
                .collect(),
            cards: self.cards.textures(ctx, k),
            ui: self.ui.textures(ctx, k),
        });
        *cache = Some(Rc::clone(&textures));
        textures
    }

    /// Высота макушки фигуры над точкой, стоящей в центре клетки, в пикселях арта.
    pub fn height(&self, color: Color, kind: Kind) -> f32 {
        ANCHOR.y - self.tops[index(color, kind)] as f32
    }

    /// Цены карт в балансе изменились: перерисовывает лица карт (и сбрасывает текстуры).
    pub fn sync_costs(&mut self) {
        if self.cards.sync_costs() {
            *self.textures.borrow_mut() = None;
        }
    }

    /// Картинка карты, повёрнутая на `degrees` по часовой стрелке.
    pub fn card(
        &self,
        ctx: &egui::Context,
        textures: &Textures,
        pic: CardPic,
        degrees: f32,
    ) -> Sprite {
        self.cards.turned(ctx, &textures.cards, pic, degrees)
    }

    /// Лицо карты для показа в `zoom` раз крупнее — карта под курсором.
    pub fn card_zoomed(
        &self,
        ctx: &egui::Context,
        textures: &Textures,
        pic: CardPic,
        zoom: usize,
    ) -> Sprite {
        self.cards.zoomed(ctx, &textures.cards, pic, zoom)
    }

    /// Ширина знака шрифта без контура, пиксели.
    pub fn glyph_width(&self, glyph: Glyph) -> f32 {
        (self.font[glyph.index()].width - 2) as f32
    }

    /// Закрывает ли фигура точку `at` своего холста (в пикселях арта).
    pub fn covers(&self, color: Color, kind: Kind, at: Vec2) -> bool {
        if at.x < 0.0 || at.y < 0.0 || at.x >= SPRITE.x || at.y >= SPRITE.y {
            return false;
        }
        self.bodies[index(color, kind)].alpha(at.x as usize, at.y as usize) > 0
    }

    /// Часть стрелки прицеливания, повёрнутая на `degrees` (кратно 5) по часовой стрелке.
    pub fn arrow(
        &self,
        ctx: &egui::Context,
        textures: &Textures,
        part: ArrowPart,
        aim: Aim,
        degrees: i32,
    ) -> Sprite {
        self.ui.arrow(ctx, &textures.ui, part, aim, degrees)
    }

    /// Курсор, увеличенный ровно в `scale` раз и выводимый без сглаживания.
    pub fn cursor(&self, ctx: &egui::Context, pointer: Pointer, scale: u32) -> TextureId {
        let scale = scale.clamp(1, 8);
        let mut cache = self.cursor_textures.borrow_mut();
        if cache.as_ref().is_none_or(|(s, _)| *s != scale) {
            let textures = self
                .cursors
                .iter()
                .enumerate()
                .map(|(i, image)| {
                    let image = image.upscale(scale as usize);
                    ctx.load_texture(
                        format!("cursor-{i}-x{scale}"),
                        image,
                        TextureOptions::NEAREST,
                    )
                })
                .collect();
            *cache = Some((scale, textures));
        }
        let (_, textures) = cache.as_ref().expect("курсоры только что загружены");
        textures[pointer as usize].id()
    }

    /// Значок фигуры для панелей: текстура обрезанной по краям фигуры и её размер в
    /// пикселях арта.
    pub fn icon(&self, ctx: &egui::Context, color: Color, kind: Kind) -> (TextureId, Vec2) {
        let mut icons = self.icons.borrow_mut();
        if icons.is_empty() {
            let options = TextureOptions::LINEAR.with_mipmap_mode(Some(TextureFilter::Linear));
            *icons = self
                .bodies
                .iter()
                .enumerate()
                .map(|(i, body)| {
                    let (min, size) = body.bounds();
                    let image = body.crop(min, size).upscale(ICON_SCALE);
                    let texture = ctx.load_texture(format!("icon-{i}"), image, options);
                    (texture, vec2(size[0] as f32, size[1] as f32))
                })
                .collect();
        }
        let (texture, size) = &icons[index(color, kind)];
        (texture.id(), *size)
    }
}

/// Значок окна — белый конь cburnett с lichess.
pub fn window_icon() -> IconData {
    const SIZE: u32 = 64;
    let svg = include_bytes!("../assets/piece/cburnett/wN.svg");
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default())
        .expect("встроенный SVG фигуры не читается");
    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE).expect("размер значка не нулевой");
    let scale = SIZE as f32 / tree.size().width().max(tree.size().height());
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let c = pixel.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursors_match_the_generator() {
        for png in CURSORS {
            let image = Pixels::decode(png);
            assert_eq!(
                [image.width, image.height],
                [CURSOR.x as usize, CURSOR.y as usize]
            );
            // Кончик стрелки непрозрачен, правый верхний угол пуст.
            let (x, y) = (CURSOR_HOTSPOT.x as usize, CURSOR_HOTSPOT.y as usize);
            assert_eq!(image.alpha(x, y), 255);
            assert_eq!(image.alpha(image.width - 1, 0), 0);
        }
    }

    #[test]
    fn sprites_fit_the_board_layout() {
        for png in BODIES.iter().chain(&SHADOWS) {
            let image = Pixels::decode(png);
            assert_eq!(
                [image.width, image.height],
                [SPRITE.x as usize, SPRITE.y as usize]
            );
        }
        for png in TILES {
            let image = Pixels::decode(png);
            assert_eq!(image.width, TILE.x as usize);
            // Нижняя клетка доски доходит ровно до низа картинки доски.
            let (_, y) = cell_origin(7, 7);
            assert_eq!(y as usize + image.height, BOARD.y as usize);
        }
    }

    #[test]
    fn board_is_lit_where_the_cells_are() {
        let art = Art::new();
        for board in &art.boards {
            // Центр каждой клетки непрозрачен, углы картинки — пустые.
            for i in 0..8 {
                for j in 0..8 {
                    let (x, y) = cell_origin(i, j);
                    let (cx, cy) = (x as usize + 28, y as usize + 21);
                    assert_eq!(board.alpha(cx, cy), 255);
                }
            }
            assert_eq!(board.alpha(0, 0), 0);
            assert_eq!(board.alpha(BOARD.x as usize - 1, 0), 0);
        }
        // Внизу слева у белых — a1, тёмная клетка; у перевёрнутой доски там h8, тоже тёмная.
        assert_eq!(square_of(0, 7, false), (0, 0));
        assert_eq!(square_of(0, 7, true), (7, 7));
    }
}
