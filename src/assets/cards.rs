//! Карты в стиле Into the Breach, вдвое детальнее доски (`tools/breach_cards_hd.py`): лицо
//! 100×140 пикселей карты на 50×70 пикселей арта (`CARD_DENSITY` = 2). Лица трёх типов —
//! обычные карты сторон, карты жертвы и карты победы — у всех одна раскладка: шапка, окно
//! под картинку (у карты без картинки — пустое) и поле описания. Рубашки сторон, стопка
//! колоды, повороты для веера в руке и кадры сжигания.
//! Повороты — RotSprite, как в генераторе: картинка трижды увеличивается scale2x и
//! выбирается по пикселям, поэтому не размывается.

use std::cell::RefCell;
use std::collections::HashMap;

use eframe::egui::{self, TextureHandle, TextureOptions, Vec2, vec2};

use super::{Pixels, glyph, rgb};
use crate::balance::balance;
use crate::game::{CardKind, Color, Family, Points};

/// Карта, пиксели арта.
pub const CARD: Vec2 = vec2(50.0, 70.0);
/// Пикселей карты на пиксель арта: карты нарисованы вдвое детальнее доски.
pub const CARD_DENSITY: usize = 2;
const D: usize = CARD_DENSITY;
/// Кадр сжигания: карта лежит в нём с отступом `BURN_CARD_AT`, сверху место для огня и дыма
/// (пиксели арта; в картинке кадр вдвое больше).
pub const BURN_FRAME: Vec2 = vec2(74.0, 110.0);
pub const BURN_CARD_AT: Vec2 = vec2(12.0, 36.0);
pub const BURN_FRAMES: usize = 32;
/// Кадр сжигания длится 70 мс, как в превью арт-проекта.
pub const BURN_FRAME_TIME: f64 = 0.07;
/// Сколько торцов карт видно под верхней картой колоды.
pub const DECK_EDGES: u8 = 12;
/// Тень карты лежит со сдвигом на 2 вправо и 3 вниз (пиксели арта).
pub const SHADOW_SHIFT: Vec2 = vec2(2.0, 3.0);
/// Знаки счётчиков очков выводятся втрое крупнее, чем на картах 1x: число поверх (вдвое
/// крупнее цифр на карте) знак не закрывает.
pub const EMBLEM_ZOOM: usize = 3;

const CHAMFER: usize = 2 * D;
const DECK_PAD: usize = 2 * D;

/// Лица: обычные белых, чёрных, жертвы, победы — у каждого обычное и выбранное.
const FACES: [&[u8]; 8] = breach!(
    "cards/hd/front_w.png",
    "cards/hd/front_w_selected.png",
    "cards/hd/front_b.png",
    "cards/hd/front_b_selected.png",
    "cards/hd/front_sacrifice.png",
    "cards/hd/front_sacrifice_selected.png",
    "cards/hd/front_victory.png",
    "cards/hd/front_victory_selected.png",
);
const BACKS: [&[u8]; 2] = breach!("cards/hd/back_w.png", "cards/hd/back_b.png");
/// Сжигание: лица без картинки белых, чёрных, жертвы, победы, затем лица с картинкой —
/// огненная волна белых и чёрных, обмен, дополнительное финансирование, исцеление.
const BURNS: [&[u8]; 9] = breach!(
    "cards/hd/burn/burn_strip_plain_w.png",
    "cards/hd/burn/burn_strip_plain_b.png",
    "cards/hd/burn/burn_strip_plain_sacrifice.png",
    "cards/hd/burn/burn_strip_plain_victory.png",
    "cards/hd/burn/burn_strip_fire_wave_w.png",
    "cards/hd/burn/burn_strip_fire_wave_b.png",
    "cards/hd/burn/burn_strip_swap_victory.png",
    "cards/hd/burn/burn_strip_funding_sacrifice.png",
    "cards/hd/burn/burn_strip_heal_victory.png",
);
/// Картинки в окне лица 76×50 в точке (12, 34): огненная волна, обмен, договор в крови
/// (дополнительное финансирование), слон в крыльях (исцеление).
const ART: [&[u8]; 4] = breach!(
    "cards/hd/art/fire_wave.png",
    "cards/hd/art/swap.png",
    "cards/hd/art/contract_blood.png",
    "cards/hd/art/bless.png"
);
const ART_AT: (i32, i32) = (12, 34);
/// Все виды карт по порядку лиц (`kind_index`): у каждого четыре лица — белых и чёрных,
/// обычное и выбранное.
const KINDS: [CardKind; 27] = [
    CardKind::Normal,
    CardKind::FireWave,
    CardKind::Sacrifice,
    CardKind::Victory,
    CardKind::Swap,
    CardKind::Funding,
    CardKind::Heal,
    CardKind::Storm,
    CardKind::Avalanche,
    CardKind::IceBarrier,
    CardKind::Offering,
    CardKind::Encircle,
    CardKind::Multiply,
    CardKind::Champion,
    CardKind::Bastion,
    CardKind::IceWall,
    CardKind::MassSacrifice,
    CardKind::BloodGod,
    CardKind::AbsoluteZero,
    CardKind::IceShard,
    CardKind::Barrier,
    CardKind::ColdRuler,
    CardKind::IceBlast,
    CardKind::VictoryTales,
    CardKind::HardWinter,
    CardKind::IceWarrior,
    CardKind::NorthWarrior,
];

/// Место вида карты в `KINDS`; новая карта без места здесь не соберётся.
fn kind_index(kind: CardKind) -> usize {
    match kind {
        CardKind::Normal => 0,
        CardKind::FireWave => 1,
        CardKind::Sacrifice => 2,
        CardKind::Victory => 3,
        CardKind::Swap => 4,
        CardKind::Funding => 5,
        CardKind::Heal => 6,
        CardKind::Storm => 7,
        CardKind::Avalanche => 8,
        CardKind::IceBarrier => 9,
        CardKind::Offering => 10,
        CardKind::Encircle => 11,
        CardKind::Multiply => 12,
        CardKind::Champion => 13,
        CardKind::Bastion => 14,
        CardKind::IceWall => 15,
        CardKind::MassSacrifice => 16,
        CardKind::BloodGod => 17,
        CardKind::AbsoluteZero => 18,
        CardKind::IceShard => 19,
        CardKind::Barrier => 20,
        CardKind::ColdRuler => 21,
        CardKind::IceBlast => 22,
        CardKind::VictoryTales => 23,
        CardKind::HardWinter => 24,
        CardKind::IceWarrior => 25,
        CardKind::NorthWarrior => 26,
    }
}

/// Картинка в окне лица карты `kind` (номер в `ART`); у остальных карт окно пустое.
fn card_art(kind: CardKind) -> Option<usize> {
    match kind {
        CardKind::FireWave => Some(0),
        CardKind::Swap => Some(1),
        CardKind::Funding => Some(2),
        CardKind::Heal => Some(3),
        _ => None,
    }
}

/// Венок победы, череп жертвы и ромб маны белых и чёрных отдельно
/// (`tools/breach_emblems.py`) — для счётчиков очков.
const EMBLEMS: [&[u8]; 4] = breach!(
    "cards/emblem_wreath.png",
    "cards/emblem_skull.png",
    "cards/emblem_gem_w.png",
    "cards/emblem_gem_b.png"
);
/// Цена карты пишется на её знаке цифрой пиксельного шрифта, как мана в Hearthstone:
/// левый верхний угол цифры (без контура) на лице — посередине венка, черепа или ромба
/// (пиксели арта; на лице цифра вдвое крупнее в пикселях карты).
pub const COST_AT: [Vec2; 3] = [vec2(7.0, 4.0), vec2(4.0, 4.0), vec2(5.0, 4.0)];
/// Цвет чисел счётчиков очков: победы — белые, жертвы — красные, как глаза черепа, мана —
/// светло-голубые.
pub const POINTS_TINT: [u32; 3] = [0xffffff, 0xff4a4a, 0xa6f2ff];
/// Цвет цифр цены на карте (светлые пиксели и тень нижнего края, как у шрифта): на ромбе
/// маны — белые, голубые сливались бы с кристаллом.
pub const COST_TINT: [u32; 3] = [0xffffff, 0xff4a4a, 0xffffff];
const COST_SHADE: [u32; 3] = [0xb4b4b4, 0xb43434, 0xb4b4b4];
/// Кромка рубашки и стопки: контур, тёмная, средняя, светлая, блик (`FACTIONS` в генераторе).
const FRAME: [[u32; 5]; 2] = [
    [0x0d0f17, 0x34425a, 0x495c79, 0x6b80a3, 0x9fb3d1],
    [0x0d0a12, 0x3a2418, 0x643f29, 0x94613f, 0xc8975a],
];
/// Контур карт жертвы и победы и выбранной карты (`CARD_TYPES`, `SELECT` в генераторе).
const SACRIFICE_OUTLINE: u32 = 0x080405;
const VICTORY_OUTLINE: u32 = 0x24060c;
const SELECTED_OUTLINE: u32 = 0x0d0f17;

/// Какая картинка карты лежит на столе.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardPic {
    /// Лицо карты стороны; `selected` — бледно-зелёная кромка карты, которую можно сыграть.
    Front {
        kind: CardKind,
        color: Color,
        selected: bool,
    },
    Back(Color),
}

impl CardPic {
    fn index(self) -> usize {
        match self {
            CardPic::Front {
                kind,
                color,
                selected,
            } => kind_index(kind) * 4 + color as usize * 2 + selected as usize,
            CardPic::Back(color) => KINDS.len() * 4 + color as usize,
        }
    }

    fn outline(self) -> u32 {
        match self {
            CardPic::Front { selected: true, .. } => SELECTED_OUTLINE,
            CardPic::Front { kind, color, .. } => match kind.family() {
                Family::Sacrifice => SACRIFICE_OUTLINE,
                Family::Victory => VICTORY_OUTLINE,
                Family::Normal => FRAME[color as usize][0],
            },
            CardPic::Back(color) => FRAME[color as usize][0],
        }
    }
}

/// Картинка на столе: тело, тень и размер тела в пикселях арта.
#[derive(Clone)]
pub struct Sprite {
    pub body: TextureHandle,
    pub shadow: TextureHandle,
    pub size: Vec2,
}

/// Колоды на выбор в меню: полная стопка белых, перекрашенная поворотом оттенка (градусы) —
/// синяя, багровая, изумрудная, золотая, аметистовая.
pub const DECK_SKINS: usize = 5;
const SKIN_HUES: [f32; DECK_SKINS] = [0.0, 145.0, -90.0, -170.0, 65.0];

pub struct CardArt {
    /// Лица всех видов карт (`CardPic::index`) и рубашки — в пикселях карты.
    faces: Vec<Pixels>,
    /// Колоды на выбор в меню.
    skins: Vec<Pixels>,
    /// Стопки колод: [сторона][число торцов − 1].
    decks: Vec<Pixels>,
    /// Кадры сжигания: [картинка из `BURNS`][кадр].
    burns: Vec<Pixels>,
    /// Венок победы и череп жертвы для счётчиков очков (1x).
    emblems: Vec<Pixels>,
    /// Цены карт, впечатанные в лица (`CardKind as usize`): изменились — лица нужно перерисовать.
    costs: [u8; CardKind::COUNT],
}

/// Лица всех видов карт (по четыре на вид: стороны и выбранная) с картинкой в окне и ценой на
/// знаке; цена — по действующему балансу.
fn card_faces() -> Vec<Pixels> {
    let bases: Vec<Pixels> = FACES.iter().map(|png| Pixels::decode(png)).collect();
    let arts: Vec<Pixels> = ART.iter().map(|png| Pixels::decode(png)).collect();
    let mut faces = Vec::with_capacity(KINDS.len() * 4 + 2);
    for kind in KINDS {
        for color in [Color::White, Color::Black] {
            for selected in [false, true] {
                let slot = match kind.family() {
                    Family::Normal => color as usize,
                    Family::Sacrifice => 2,
                    Family::Victory => 3,
                };
                let mut face = bases[slot * 2 + usize::from(selected)].clone();
                if let Some(art) = card_art(kind) {
                    face.paste(&arts[art], ART_AT.0, ART_AT.1);
                }
                if let Some((points, n)) = kind.cost() {
                    paste_cost(&mut face, points, n);
                }
                faces.push(face);
            }
        }
    }
    faces
}

impl CardArt {
    pub fn new() -> CardArt {
        let mut faces = card_faces();
        let backs: Vec<Pixels> = BACKS.iter().map(|png| Pixels::decode(png)).collect();
        let decks: Vec<Pixels> = [Color::White, Color::Black]
            .iter()
            .flat_map(|&color| {
                let back = &backs[color as usize];
                (1..=DECK_EDGES).map(move |count| deck(back, color, count as usize))
            })
            .collect();
        faces.extend(backs);
        let full: &Pixels = &decks[DECK_EDGES as usize - 1];
        let skins = SKIN_HUES.iter().map(|&hue| turn_hue(full, hue)).collect();
        let (w, h) = (BURN_FRAME.x as usize * D, BURN_FRAME.y as usize * D);
        let burns = BURNS
            .iter()
            .flat_map(|png| {
                let strip = Pixels::decode(png);
                (0..BURN_FRAMES).map(move |f| strip.crop([f * w, 0], [w, h]))
            })
            .collect();
        CardArt {
            faces,
            skins,
            decks,
            burns,
            emblems: EMBLEMS.iter().map(|png| Pixels::decode(png)).collect(),
            costs: *balance().costs(),
        }
    }

    /// Цены в балансе изменились — перерисовывает лица карт с новыми ценами; `true`, если так.
    pub fn sync_costs(&mut self) -> bool {
        let costs = *balance().costs();
        if costs == self.costs {
            return false;
        }
        let backs = self.faces.split_off(KINDS.len() * 4);
        self.faces = card_faces();
        self.faces.extend(backs);
        self.costs = costs;
        true
    }

    pub fn textures(&self, ctx: &egui::Context, k: usize) -> CardTextures {
        let burn_scale = dense_scale(k, 1);
        CardTextures {
            k,
            faces: self
                .faces
                .iter()
                .enumerate()
                .map(|(i, image)| card_sprite(ctx, &format!("card-{i}"), image, k, 1))
                .collect(),
            zoomed: RefCell::new(HashMap::new()),
            decks: self
                .decks
                .iter()
                .enumerate()
                .map(|(i, image)| card_sprite(ctx, &format!("deck-{i}"), image, k, 1))
                .collect(),
            burns: self
                .burns
                .iter()
                .enumerate()
                .map(|(i, image)| {
                    ctx.load_texture(
                        format!("burn-{i}-x{k}"),
                        image.upscale(burn_scale),
                        TextureOptions::LINEAR,
                    )
                })
                .collect(),
            emblems: self
                .emblems
                .iter()
                .enumerate()
                .map(|(i, image)| sprite(ctx, &format!("emblem-{i}"), image, EMBLEM_ZOOM * k))
                .collect(),
            skins: self
                .skins
                .iter()
                .enumerate()
                .map(|(i, image)| card_sprite(ctx, &format!("skin-{i}"), image, k, 1))
                .collect(),
            turned: RefCell::new(HashMap::new()),
        }
    }

    /// Карта, повёрнутая на `degrees` по часовой стрелке; повороты запоминаются.
    pub fn turned(
        &self,
        ctx: &egui::Context,
        textures: &CardTextures,
        pic: CardPic,
        degrees: f32,
    ) -> Sprite {
        let index = pic.index();
        let key = (index, (degrees * 100.0).round() as i32);
        if key.1 == 0 {
            return textures.faces[index].clone();
        }
        let mut cache = textures.turned.borrow_mut();
        cache
            .entry(key)
            .or_insert_with(|| {
                let image = rotsprite(&self.faces[index], degrees, rgb(pic.outline()));
                card_sprite(
                    ctx,
                    &format!("card-{index}-{}", key.1),
                    &image,
                    textures.k,
                    1,
                )
            })
            .clone()
    }

    /// Лицо, показанное в `zoom` раз крупнее (карта под курсором): текстура увеличена во
    /// столько же раз сильнее, и пиксели остаются резкими. Делается при первом показе.
    pub fn zoomed(
        &self,
        ctx: &egui::Context,
        textures: &CardTextures,
        pic: CardPic,
        zoom: usize,
    ) -> Sprite {
        let index = pic.index();
        let mut cache = textures.zoomed.borrow_mut();
        cache
            .entry((index, zoom))
            .or_insert_with(|| {
                let name = format!("card-zoom{zoom}-{index}");
                card_sprite(ctx, &name, &self.faces[index], textures.k, zoom)
            })
            .clone()
    }
}

/// Текстуры карт при одном увеличении.
pub struct CardTextures {
    k: usize,
    faces: Vec<Sprite>,
    /// Лица, увеличенные сильнее — для крупной карты под курсором (по требованию).
    zoomed: RefCell<HashMap<(usize, usize), Sprite>>,
    decks: Vec<Sprite>,
    burns: Vec<TextureHandle>,
    /// Знаки счётчиков очков, увеличенные в `EMBLEM_ZOOM` раз сильнее карт 1x.
    emblems: Vec<Sprite>,
    /// Колоды на выбор в меню.
    skins: Vec<Sprite>,
    turned: RefCell<HashMap<(usize, i32), Sprite>>,
}

impl CardTextures {
    /// Знак счётчика очков стороны `color`: венок победы, череп жертвы или ромб маны в
    /// цвет её обычных карт, в `EMBLEM_ZOOM` раз крупнее, чем на карте.
    pub fn emblem(&self, points: Points, color: Color) -> &Sprite {
        let index = match points {
            Points::Mana => 2 + color as usize,
            other => other as usize,
        };
        &self.emblems[index]
    }

    /// Колода `i` из меню выбора колод.
    pub fn skin(&self, i: usize) -> &Sprite {
        &self.skins[i % self.skins.len()]
    }

    /// Колода из `cards` карт: видно не больше `DECK_EDGES` торцов; пустой колоды нет.
    pub fn deck(&self, color: Color, cards: u8) -> Option<&Sprite> {
        let edges = cards.min(DECK_EDGES) as usize;
        (edges > 0).then(|| &self.decks[color as usize * DECK_EDGES as usize + edges - 1])
    }

    /// Кадр сжигания карты `kind` стороны `color`: лица с картинкой горят своими кадрами,
    /// остальные — лицом своего типа с пустым окном.
    pub fn burn(&self, kind: CardKind, color: Color, frame: usize) -> &TextureHandle {
        let strip = match kind {
            CardKind::FireWave => 4 + color as usize,
            CardKind::Swap => 6,
            CardKind::Funding => 7,
            CardKind::Heal => 8,
            kind => match kind.family() {
                Family::Normal => color as usize,
                Family::Sacrifice => 2,
                Family::Victory => 3,
            },
        };
        &self.burns[strip * BURN_FRAMES + frame.min(BURN_FRAMES - 1)]
    }
}

/// Во сколько раз увеличить картинку карты (`D` пикселей на пиксель арта), чтобы при
/// показе в `zoom` раз крупнее на пиксель арта приходилось `k` пикселей экрана: целое, не
/// меньше 1 (в мелком окне текстура лишь чуть уменьшается).
fn dense_scale(k: usize, zoom: usize) -> usize {
    (k * zoom).div_ceil(D).max(1)
}

/// Картинка карты (пиксели карты) с тенью: размер — в пикселях арта.
fn card_sprite(ctx: &egui::Context, name: &str, image: &Pixels, k: usize, zoom: usize) -> Sprite {
    let options = TextureOptions::LINEAR;
    let scale = dense_scale(k, zoom);
    Sprite {
        body: ctx.load_texture(format!("{name}-x{k}"), image.upscale(scale), options),
        shadow: ctx.load_texture(
            format!("{name}-shadow-x{k}"),
            shadow_of(image, D).upscale(scale),
            options,
        ),
        size: vec2(image.width as f32, image.height as f32) / D as f32,
    }
}

/// Картинка 1x (пиксель картинки — пиксель арта) с тенью.
pub(super) fn sprite(ctx: &egui::Context, name: &str, image: &Pixels, k: usize) -> Sprite {
    let options = TextureOptions::LINEAR;
    Sprite {
        body: ctx.load_texture(format!("{name}-x{k}"), image.upscale(k), options),
        shadow: ctx.load_texture(
            format!("{name}-shadow-x{k}"),
            shadow_of(image, 1).upscale(k),
            options,
        ),
        size: vec2(image.width as f32, image.height as f32),
    }
}

/// Левый верхний угол числа `n` (первой цифры без контура) на лице карты, пиксели арта:
/// одна цифра — в `COST_AT`, у двузначного числа середина там же, где у одной цифры.
pub fn cost_corner(points: Points, n: u8) -> Vec2 {
    let digits = n.to_string().len() as f32;
    // Цифра 6 пикселей и пиксель между цифрами.
    COST_AT[points as usize] - vec2(((digits - 1.0) * 7.0 / 2.0).ceil(), 0.0)
}

/// Цена `n` в очках `points` на знаке карты: цифры пиксельного шрифта (пиксель шрифта —
/// пиксель арта, то есть 2×2 пикселя карты) поверх венка, черепа или ромба маны.
fn paste_cost(face: &mut Pixels, points: Points, n: u8) {
    let at = cost_corner(points, n) * D as f32;
    let (light, shade) = (COST_TINT[points as usize], COST_SHADE[points as usize]);
    let mut x = at.x as i32;
    for digit in n.to_string().bytes() {
        let image = enlarge(&glyph(digit, light, shade), D);
        // Картинка знака на пиксель шрифта шире цифры с каждой стороны и на пиксель выше.
        face.paste(&image, x - D as i32, at.y as i32 - D as i32);
        x += image.width as i32 - D as i32;
    }
}

/// Увеличение в `k` раз без сглаживания.
fn enlarge(image: &Pixels, k: usize) -> Pixels {
    let mut out = Pixels::new(image.width * k, image.height * k);
    for y in 0..out.height {
        for x in 0..out.width {
            out.rgba[y * out.width + x] = image.rgba[(y / k) * image.width + x / k];
        }
    }
    out
}

/// Картинка с оттенком, повёрнутым на `degrees`: насыщенность и яркость прежние, серые
/// пиксели остаются серыми.
fn turn_hue(image: &Pixels, degrees: f32) -> Pixels {
    let mut out = image.clone();
    for px in &mut out.rgba {
        let [r, g, b] = [px[0], px[1], px[2]].map(|c| c as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        if delta <= 0.0 {
            continue;
        }
        let hue = if max == r {
            60.0 * ((g - b) / delta).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        let h = (hue + degrees).rem_euclid(360.0) / 60.0;
        let x = delta * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
        let (r1, g1, b1) = match h as u32 {
            0 => (delta, x, 0.0),
            1 => (x, delta, 0.0),
            2 => (0.0, delta, x),
            3 => (0.0, x, delta),
            4 => (x, 0.0, delta),
            _ => (delta, 0.0, x),
        };
        let to = |c: f32| ((c + min) * 255.0).round().clamp(0.0, 255.0) as u8;
        px[0] = to(r1);
        px[1] = to(g1);
        px[2] = to(b1);
    }
    out
}

/// Тень: силуэт 30% чёрного со сдвигом `SHADOW_SHIFT`; `density` — пикселей картинки на
/// пиксель арта.
fn shadow_of(image: &Pixels, density: usize) -> Pixels {
    let (dx, dy) = (
        SHADOW_SHIFT.x as usize * density,
        SHADOW_SHIFT.y as usize * density,
    );
    let mut shadow = Pixels::new(image.width + dx, image.height + dy);
    for y in 0..image.height {
        for x in 0..image.width {
            if image.alpha(x, y) > 0 {
                shadow.set(x + dx, y + dy, [0, 0, 0], 77);
            }
        }
    }
    shadow
}

/// Колода рубашкой вверх: под верхней картой видны торцы нижних с лёгким разбросом. Всё в
/// пикселях карты: торец — `D` строк, разброс — на `D` пикселей.
fn deck(back: &Pixels, color: Color, count: usize) -> Pixels {
    let frame = FRAME[color as usize];
    let (cw, ch) = (back.width, back.height);
    let mut image = Pixels::new(cw + 2 * DECK_PAD, ch + count * D + DECK_PAD + D);
    // Сдвиг торцов: −1, 0 или +1 пиксель арта, всегда одинаковый у одной стопки.
    let shift = |k: usize| [0i32, 0, 1, 0, -1, 0, 0, 1, 0, -1, 0, 0][k % 12] * D as i32;
    for k in (1..count).rev() {
        let dx = (DECK_PAD as i32 + shift(k)) as usize;
        let (x0, x1) = (dx + CHAMFER, dx + cw - 1 - CHAMFER);
        let color = if k % 2 == 1 { frame[2] } else { frame[1] };
        for row in ch - D + k * D..ch + k * D {
            // Верхняя строка торца светлее — торцы читаются по одному.
            let line = if row == ch - D + k * D {
                frame[3]
            } else {
                color
            };
            for x in x0..=x1 {
                image.set(x, row, rgb(line), 255);
            }
            image.set(x0 - 1, row, rgb(frame[0]), 255);
            image.set(x1 + 1, row, rgb(frame[0]), 255);
        }
    }
    for row in ch + (count - 1) * D..ch + count * D {
        for x in DECK_PAD + CHAMFER..DECK_PAD + cw - CHAMFER {
            image.set(x, row, rgb(frame[0]), 255);
        }
    }
    image.paste(back, DECK_PAD as i32, 0);
    image
}

/// Увеличение вдвое по правилам scale2x: диагональные края остаются ступеньками.
fn scale2x(image: &Pixels) -> Pixels {
    let (w, h) = (image.width as i32, image.height as i32);
    let at = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= w || y >= h {
            [0; 4]
        } else {
            image.rgba[(y * w + x) as usize]
        }
    };
    let mut out = Pixels::new(image.width * 2, image.height * 2);
    for y in 0..h {
        for x in 0..w {
            let (b, d, e, f, hh) = (
                at(x, y - 1),
                at(x - 1, y),
                at(x, y),
                at(x + 1, y),
                at(x, y + 1),
            );
            let edge = b != hh && d != f;
            let pick = |same: bool, other: [u8; 4]| if edge && same { other } else { e };
            let (ox, oy) = (2 * x as usize, 2 * y as usize);
            let ow = out.width;
            out.rgba[oy * ow + ox] = pick(d == b, d);
            out.rgba[oy * ow + ox + 1] = pick(b == f, f);
            out.rgba[(oy + 1) * ow + ox] = pick(d == hh, d);
            out.rgba[(oy + 1) * ow + ox + 1] = pick(hh == f, f);
        }
    }
    out
}

/// Поворот пиксель-арта без размытия: центр картинки остаётся центром, край силуэта
/// обводится цветом контура.
pub(super) fn rotsprite(image: &Pixels, degrees: f32, outline: [u8; 3]) -> Pixels {
    let big = scale2x(&scale2x(&scale2x(image)));
    let (w, h) = (image.width as f32, image.height as f32);
    let (sa, ca) = degrees.to_radians().sin_cos();
    let rw = (w * ca.abs() + h * sa.abs()).ceil() as usize + 4;
    let rh = (w * sa.abs() + h * ca.abs()).ceil() as usize + 4;
    let mut out = Pixels::new(rw, rh);
    for yy in 0..rh {
        for xx in 0..rw {
            let dx = xx as f32 + 0.5 - rw as f32 / 2.0;
            let dy = yy as f32 + 0.5 - rh as f32 / 2.0;
            let lx = ca * dx + sa * dy + w / 2.0;
            let ly = -sa * dx + ca * dy + h / 2.0;
            let (sx, sy) = ((lx * 8.0).floor(), (ly * 8.0).floor());
            if sx >= 0.0 && sy >= 0.0 && sx < w * 8.0 && sy < h * 8.0 {
                out.rgba[yy * rw + xx] = big.rgba[sy as usize * big.width + sx as usize];
            }
        }
    }
    let solid = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && (x as usize) < rw
            && (y as usize) < rh
            && out.alpha(x as usize, y as usize) > 0
    };
    let mut edges = Vec::new();
    for y in 0..rh as i32 {
        for x in 0..rw as i32 {
            if solid(x, y)
                && !(solid(x - 1, y) && solid(x + 1, y) && solid(x, y - 1) && solid(x, y + 1))
            {
                edges.push((x as usize, y as usize));
            }
        }
    }
    for (x, y) in edges {
        out.set(x, y, outline, 255);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_pictures_match_the_generator() {
        let art = CardArt::new();
        assert_eq!(art.faces.len(), KINDS.len() * 4 + 2);
        for face in &art.faces {
            assert_eq!(
                [face.width, face.height],
                [CARD.x as usize * D, CARD.y as usize * D]
            );
        }
        for (i, &kind) in KINDS.iter().enumerate() {
            assert_eq!(kind_index(kind), i, "{kind:?}");
        }
        for frame in &art.burns {
            assert_eq!(
                [frame.width, frame.height],
                [BURN_FRAME.x as usize * D, BURN_FRAME.y as usize * D]
            );
        }
        assert_eq!(art.burns.len(), BURNS.len() * BURN_FRAMES);
        // В первом кадре карта целая и стоит на своём месте.
        let first = &art.burns[0];
        let at = BURN_CARD_AT * D as f32;
        assert!(first.alpha(at.x as usize + 50, at.y as usize + 70) > 0);
        assert_eq!(first.alpha(0, 0), 0);
        // Последний кадр почти пуст: от карты остался дым.
        let last = &art.burns[BURN_FRAMES - 1];
        let left = last.rgba.iter().filter(|p| p[3] == 255).count();
        assert!(left < 160, "в последнем кадре {left} непрозрачных пикселей");
        // Полная колода — 54×85 пикселей арта, как раньше.
        let full = &art.decks[DECK_EDGES as usize - 1];
        assert_eq!([full.width, full.height], [54 * D, 85 * D]);
    }

    #[test]
    fn turned_card_keeps_its_size_and_centre() {
        let art = CardArt::new();
        let card = &art.faces[0];
        let flat = rotsprite(card, 0.0, [0, 0, 0]);
        assert_eq!([flat.width, flat.height], [card.width + 4, card.height + 4]);
        let turned = rotsprite(card, 17.0, [0, 0, 0]);
        assert!(turned.width > flat.width && turned.height < flat.height + 60);
        // Центр повёрнутой карты непрозрачен, углы холста — пустые.
        assert!(turned.alpha(turned.width / 2, turned.height / 2) > 0);
        assert_eq!(turned.alpha(0, 0), 0);
    }
}
