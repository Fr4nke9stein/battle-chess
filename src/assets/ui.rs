//! Интерфейс стола из арт-проекта: кнопка «Конец хода» (`tools/breach_ui.py`), стрелка
//! прицеливания картой со звеньями и метками цели (`tools/breach_arrow.py`), огненная
//! волна (`tools/breach_firewave.py`), портал обмена (`tools/breach_portal.py`) и
//! благословение слона для исцеления (`tools/breach_bless.py`).

use std::cell::RefCell;
use std::collections::HashMap;

use eframe::egui::{self, TextureHandle, TextureOptions, Vec2, vec2};

use super::Pixels;
use super::cards::{Sprite, rotsprite, sprite};
use crate::game::{Color, Kind};

/// Кнопка «Конец хода» без тени; тень лежит в картинке справа и снизу.
pub const BUTTON: Vec2 = vec2(53.0, 35.0);
/// Монета из арт-проекта (`tools/breach_coin.py`): большая — счётчик золота rogue-like,
/// маленькая — цены в магазине.
pub const COIN: Vec2 = vec2(32.0, 32.0);
pub const COIN_SMALL: Vec2 = vec2(16.0, 16.0);
/// Кадр огненной волны и центр верхней грани клетки фигуры в нём.
pub const WAVE_FRAME: Vec2 = vec2(184.0, 168.0);
pub const WAVE_CENTER: Vec2 = vec2(92.0, 98.0);
pub const WAVE_FRAMES: usize = 20;
pub const WAVE_FRAME_TIME: f64 = 0.06;
/// Метка цели: ромб клетки и поля вокруг него (`marker_pad()` в генераторе).
pub const TARGET_PAD: f32 = 15.0;
/// Звенья стрелки — через столько пикселей вдоль кривой, у карты мелкие, к цели крупнее.
pub const ARROW_SPACING: [f32; 3] = [8.5, 10.5, 12.5];
/// Длина наконечника (без контура) — по ней наконечник отступает от цели.
pub const ARROW_HEAD: f32 = 28.0;
/// Портал под фигурой: кадр и центр верхней грани клетки в нём; 16 кадров по 60 мс —
/// фигура проваливается, потом столько же — поднимается на другой клетке.
pub const PORTAL_FRAME: Vec2 = vec2(80.0, 116.0);
pub const PORTAL_CENTER: Vec2 = vec2(40.0, 88.0);
pub const PORTAL_FRAMES: usize = 16;
pub const PORTAL_FRAME_TIME: f64 = 0.06;
/// Кольцо портала на земле — полуоси при полном раскрытии и масштаб по кадрам
/// (`RX`, `RY`, `SCALE` в генераторе).
pub const PORTAL_RADII: Vec2 = vec2(17.0, 12.75);
pub const PORTAL_SCALE: [f32; PORTAL_FRAMES] = [
    0.15, 0.45, 0.8, 1.06, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.85, 0.55, 0.25, 0.0,
];
/// Насколько фигура ушла вниз в каждом кадре — при исчезновении и при появлении
/// (`SINK` в генераторе, округлено так же).
pub const PORTAL_SINK: [[f32; PORTAL_FRAMES]; 2] = [
    [
        0.0, 0.0, 0.0, 0.0, 1.0, 4.0, 10.0, 18.0, 28.0, 40.0, 55.0, 72.0, 72.0, 72.0, 72.0, 72.0,
    ],
    [
        72.0, 72.0, 72.0, 72.0, 55.0, 40.0, 28.0, 18.0, 10.0, 4.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
];
/// Светящийся срез фигуры по передней кромке кольца.
pub const PORTAL_CUT: [u8; 3] = [0xa6, 0xf2, 0xff];
/// Благословение слона: 24 кадра по 60 мс; свет на земле и столбы света — в кадрах как у
/// огненной волны (`WAVE_FRAME`, `WAVE_CENTER`), слон с крыльями — в своём кадре с точкой
/// опоры (центром клетки) `BLESS_ANCHOR`.
pub const BLESS_FRAMES: usize = 24;
pub const BLESS_FRAME_TIME: f64 = 0.06;
pub const BLESS_FIGURE: Vec2 = vec2(120.0, 112.0);
pub const BLESS_ANCHOR: Vec2 = vec2(60.0, 92.0);
/// Заморозка (`tools/breach_freeze.py`): кадр клетки и центр её верхней грани в нём, 18
/// кадров по 60 мс; на соседних клетках лёд начинается позже на `FREEZE_DELAY` кадров за
/// клетку от точки заморозки. Последний кадр — «заморожено», его держат.
pub const FREEZE_FRAME: Vec2 = vec2(80.0, 116.0);
pub const FREEZE_CENTER: Vec2 = vec2(40.0, 88.0);
pub const FREEZE_FRAMES: usize = 18;
pub const FREEZE_FRAME_TIME: f64 = 0.06;
pub const FREEZE_DELAY: usize = 2;
/// Иней на фигуре по кадрам — доля её высоты (`FROST` в генераторе).
pub const FREEZE_FROST: [f32; FREEZE_FRAMES] = [
    0.0, 0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0,
];
/// Кадр, на котором глыба встала и звякнула вспышкой (`CLINK` в генераторе).
pub const FREEZE_CLINK: usize = 14;
/// Лава в пешке усиленного компьютера (`tools/breach_lava.py`): кадры вместо спрайта пешки
/// (холст фигуры, та же точка опоры) по 60 мс. Трещины растут до кадра `LAVA_SPREAD`, пока
/// пешка ждёт взрыва — повторяются кадры `LAVA_HOLD`, с `LAVA_IGNITE` трещины разгораются,
/// с `LAVA_WAVE_AT` начинается огненная волна, к последнему кадру всё гаснет.
pub const LAVA_FRAMES: usize = 28;
pub const LAVA_FRAME_TIME: f64 = 0.06;
pub const LAVA_SPREAD: usize = 13;
pub const LAVA_HOLD: [usize; 2] = [13, 17];
pub const LAVA_IGNITE: usize = 17;
pub const LAVA_WAVE_AT: usize = 21;
/// Электричество в слоне усиленного компьютера (`tools/breach_arcs.py`, вариант «не гаснет»):
/// кадр больше спрайта, точка опоры — центр клетки, по 60 мс. Вступление со вспышкой
/// `ELECTRIC_FLASH`, потом без конца повторяются кадры `ELECTRIC_LOOP`.
pub const ELECTRIC_FRAMES: usize = 24;
pub const ELECTRIC_FRAME_TIME: f64 = 0.06;
pub const ELECTRIC_FIGURE: Vec2 = vec2(120.0, 120.0);
pub const ELECTRIC_ANCHOR: Vec2 = vec2(60.0, 92.0);
pub const ELECTRIC_FLASH: usize = 5;
pub const ELECTRIC_LOOP: [usize; 2] = [8, 24];
/// Вихрь воздушной фигуры (`tools/breach_wind.py`, с 0.67): кадр 120×120 с той же точкой
/// опоры, что у электричества, 60 мс; вступление [0, 8), повтор [8, 40).
pub const WIND_FRAMES: usize = 40;
pub const WIND_LOOP: [usize; 2] = [8, 40];

/// Состояния кнопки в порядке `Button`: «Конец хода», потом «Начать раунд» (rogue-like,
/// расстановка перед уровнем, с 0.59).
const BUTTONS: [&[u8]; 10] = breach!(
    "ui/end_turn_normal.png",
    "ui/end_turn_hover.png",
    "ui/end_turn_pressed.png",
    "ui/end_turn_disabled.png",
    "ui/end_turn_ready.png",
    "ui/start_round_normal.png",
    "ui/start_round_hover.png",
    "ui/start_round_pressed.png",
    "ui/start_round_disabled.png",
    "ui/start_round_ready.png",
);
const BUTTON_STATES: usize = 5;
/// Монеты: большая и маленькая.
const COINS: [&[u8]; 2] = breach!("ui/coin_32.png", "ui/coin_16.png");
/// Звенья s, m, l и наконечник — по состояниям `Aim` (idle, enemy, ally).
const ARROW: [&[u8]; 12] = breach!(
    "ui/arrow_seg_s_idle.png",
    "ui/arrow_seg_m_idle.png",
    "ui/arrow_seg_l_idle.png",
    "ui/arrow_head_idle.png",
    "ui/arrow_seg_s_enemy.png",
    "ui/arrow_seg_m_enemy.png",
    "ui/arrow_seg_l_enemy.png",
    "ui/arrow_head_enemy.png",
    "ui/arrow_seg_s_ally.png",
    "ui/arrow_seg_m_ally.png",
    "ui/arrow_seg_l_ally.png",
    "ui/arrow_head_ally.png",
);
const TARGETS: [&[u8]; 3] = breach!(
    "ui/target_idle.png",
    "ui/target_enemy.png",
    "ui/target_ally.png"
);
/// Огненная волна: слой под фигурами и слой над ними, кадры в ряд.
const WAVE: [&[u8]; 2] = breach!("effects/fire_wave_under.png", "effects/fire_wave_over.png");
/// Благословение: слой под фигурами и над ними; слон с крыльями белых и чёрных.
const BLESS: [&[u8]; 2] = breach!("effects/bless_under.png", "effects/bless_over.png");
const BLESS_FIGURE_PNG: [&[u8]; 2] =
    breach!("effects/bless_figure_w.png", "effects/bless_figure_b.png");
/// Портал: исчезновение и появление, у каждого слой под фигурами и над ними.
const PORTAL: [&[u8]; 4] = breach!(
    "effects/portal_vanish_under.png",
    "effects/portal_vanish_over.png",
    "effects/portal_appear_under.png",
    "effects/portal_appear_over.png",
);
/// Заморозка: снег на клетке по вариантам, снежинки и пар над фигурами, глыбы льда по
/// фигурам (пешка, конь, слон, ладья, ферзь, король) и вариантам.
const FREEZE_TILES: [&[u8]; 4] = breach!(
    "effects/freeze_tile_0.png",
    "effects/freeze_tile_1.png",
    "effects/freeze_tile_2.png",
    "effects/freeze_tile_3.png",
);
const FREEZE_OVER: [&[u8]; 1] = breach!("effects/freeze_over.png");
const FREEZE_BLOCKS: [&[u8]; 24] = breach!(
    "effects/freeze_block_P_0.png",
    "effects/freeze_block_P_1.png",
    "effects/freeze_block_P_2.png",
    "effects/freeze_block_P_3.png",
    "effects/freeze_block_N_0.png",
    "effects/freeze_block_N_1.png",
    "effects/freeze_block_N_2.png",
    "effects/freeze_block_N_3.png",
    "effects/freeze_block_B_0.png",
    "effects/freeze_block_B_1.png",
    "effects/freeze_block_B_2.png",
    "effects/freeze_block_B_3.png",
    "effects/freeze_block_R_0.png",
    "effects/freeze_block_R_1.png",
    "effects/freeze_block_R_2.png",
    "effects/freeze_block_R_3.png",
    "effects/freeze_block_Q_0.png",
    "effects/freeze_block_Q_1.png",
    "effects/freeze_block_Q_2.png",
    "effects/freeze_block_Q_3.png",
    "effects/freeze_block_K_0.png",
    "effects/freeze_block_K_1.png",
    "effects/freeze_block_K_2.png",
    "effects/freeze_block_K_3.png",
);
/// Лава в фигурах (с 0.63 — всех видов, из арт-проекта: огненные фигуры rogue-like) в
/// порядке `super::index`, электричество в слоне — белых и чёрных.
const LAVA_PNG: [&[u8]; 12] = breach!(
    "effects/lava_crack_wP.png",
    "effects/lava_crack_wN.png",
    "effects/lava_crack_wB.png",
    "effects/lava_crack_wR.png",
    "effects/lava_crack_wQ.png",
    "effects/lava_crack_wK.png",
    "effects/lava_crack_bP.png",
    "effects/lava_crack_bN.png",
    "effects/lava_crack_bB.png",
    "effects/lava_crack_bR.png",
    "effects/lava_crack_bQ.png",
    "effects/lava_crack_bK.png",
);
/// Электричество — с 0.64 у всех видов (из арт-проекта: электрические фигуры rogue-like), с
/// 0.66 — последняя версия `electric_hold_*` без кольца по земле во вспышке (просьба
/// пользователя; спокойный `electric_calm_*` из 0.65 не подошёл), в порядке `super::index`.
const WIND_PNG: [&[u8]; 12] = breach!(
    "effects/wind_wP.png",
    "effects/wind_wN.png",
    "effects/wind_wB.png",
    "effects/wind_wR.png",
    "effects/wind_wQ.png",
    "effects/wind_wK.png",
    "effects/wind_bP.png",
    "effects/wind_bN.png",
    "effects/wind_bB.png",
    "effects/wind_bR.png",
    "effects/wind_bQ.png",
    "effects/wind_bK.png",
);
const ELECTRIC_PNG: [&[u8]; 12] = breach!(
    "effects/electric_hold_wP.png",
    "effects/electric_hold_wN.png",
    "effects/electric_hold_wB.png",
    "effects/electric_hold_wR.png",
    "effects/electric_hold_wQ.png",
    "effects/electric_hold_wK.png",
    "effects/electric_hold_bP.png",
    "effects/electric_hold_bN.png",
    "effects/electric_hold_bB.png",
    "effects/electric_hold_bR.png",
    "effects/electric_hold_bQ.png",
    "effects/electric_hold_bK.png",
);
/// Контур звеньев по состояниям — им обводится повёрнутое звено.
const ARROW_OUTLINE: [[u8; 3]; 3] = [[0x0d, 0x0f, 0x17], [0x1a, 0x05, 0x05], [0x06, 0x14, 0x1c]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Normal,
    Hover,
    Pressed,
    /// Ход соперника.
    Disabled,
    /// Делать больше нечего — пора закончить ход.
    Ready,
}

/// Цвет стрелки: цели нет, чужая фигура (ледяной снаряд и взрыв), своя фигура.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    Idle,
    Enemy,
    Ally,
}

/// Часть стрелки: звенья трёх размеров и наконечник.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrowPart {
    Small,
    Medium,
    Large,
    Head,
}

pub struct UiArt {
    buttons: Vec<Pixels>,
    coins: Vec<Pixels>,
    arrow: Vec<Pixels>,
    targets: Vec<Pixels>,
    /// Кадры волны: [слой][кадр].
    wave: Vec<Pixels>,
    /// Кадры портала: [исчезновение или появление, слой][кадр].
    portal: Vec<Pixels>,
    /// Кадры благословения: [слой][кадр] и [сторона][кадр] слона с крыльями.
    bless: Vec<Pixels>,
    bless_figure: Vec<Pixels>,
    /// Заморозка: снег по вариантам, снежинки и глыбы [фигура][вариант].
    freeze_tiles: Vec<Strip>,
    freeze_over: Strip,
    freeze_blocks: Vec<Strip>,
    /// Лава в пешке и электричество в слоне по сторонам.
    lava: Vec<Strip>,
    electric: Vec<Strip>,
    wind: Vec<Strip>,
}

/// Кадры из полос `strips`, где кадр `size` и их `count` в ряд.
fn frames(strips: &[&[u8]], size: Vec2, count: usize) -> Vec<Pixels> {
    strips
        .iter()
        .flat_map(|png| {
            let strip = Pixels::decode(png);
            let (w, h) = (size.x as usize, size.y as usize);
            (0..count).map(move |f| strip.crop([f * w, 0], [w, h]))
        })
        .collect()
}

/// Кадры полосы, обрезанные по непрозрачному: у заморозки их много (глыба на каждую
/// фигуру и вариант), а лёд в каждом кадре — малая часть клетки. Пустые кадры не хранятся,
/// повторяющийся кадр — один раз.
struct Strip {
    /// Картинки и где их левый верхний угол в кадре.
    images: Vec<(Pixels, Vec2)>,
    /// Кадр → картинка; `None` — кадр пуст.
    frames: Vec<Option<usize>>,
}

impl Strip {
    fn new(png: &[u8], size: Vec2, count: usize) -> Strip {
        let strip = Pixels::decode(png);
        let (w, h) = (size.x as usize, size.y as usize);
        let mut images = Vec::new();
        let mut frames = Vec::new();
        let mut previous: Option<(Pixels, Option<usize>)> = None;
        for f in 0..count {
            let frame = strip.crop([f * w, 0], [w, h]);
            if let Some((last, image)) = &previous
                && last.rgba == frame.rgba
            {
                frames.push(*image);
                continue;
            }
            let image = frame.rgba.iter().any(|p| p[3] > 0).then(|| {
                let (min, size) = frame.bounds();
                let at = vec2(min[0] as f32, min[1] as f32);
                images.push((frame.crop(min, size), at));
                images.len() - 1
            });
            frames.push(image);
            previous = Some((frame, image));
        }
        Strip { images, frames }
    }

    fn textures(&self, ctx: &egui::Context, name: &str, k: usize) -> StripTextures {
        StripTextures {
            images: self
                .images
                .iter()
                .enumerate()
                .map(|(i, (image, at))| {
                    let texture = ctx.load_texture(
                        format!("{name}-{i}-x{k}"),
                        image.upscale(k),
                        TextureOptions::LINEAR,
                    );
                    let size = vec2(image.width as f32, image.height as f32);
                    FrameSprite {
                        texture: texture.id(),
                        at: *at,
                        size,
                        _keep: texture,
                    }
                })
                .collect(),
            frames: self.frames.clone(),
        }
    }
}

/// Кадр эффекта: текстура, её левый верхний угол в кадре и размер, пиксели арта.
#[derive(Clone)]
pub struct FrameSprite {
    pub texture: egui::TextureId,
    pub at: Vec2,
    pub size: Vec2,
    _keep: TextureHandle,
}

struct StripTextures {
    images: Vec<FrameSprite>,
    frames: Vec<Option<usize>>,
}

impl StripTextures {
    /// Кадр `frame` (последний — если дальше); `None` — кадр пуст.
    fn frame(&self, frame: usize) -> Option<&FrameSprite> {
        let frame = frame.min(self.frames.len() - 1);
        self.frames[frame].map(|i| &self.images[i])
    }
}

impl UiArt {
    pub fn new() -> UiArt {
        let decode = |pngs: &[&[u8]]| pngs.iter().map(|png| Pixels::decode(png)).collect();
        UiArt {
            buttons: decode(&BUTTONS),
            coins: decode(&COINS),
            arrow: decode(&ARROW),
            targets: decode(&TARGETS),
            wave: frames(&WAVE, WAVE_FRAME, WAVE_FRAMES),
            portal: frames(&PORTAL, PORTAL_FRAME, PORTAL_FRAMES),
            bless: frames(&BLESS, WAVE_FRAME, BLESS_FRAMES),
            bless_figure: frames(&BLESS_FIGURE_PNG, BLESS_FIGURE, BLESS_FRAMES),
            freeze_tiles: FREEZE_TILES
                .iter()
                .map(|png| Strip::new(png, FREEZE_FRAME, FREEZE_FRAMES))
                .collect(),
            freeze_over: Strip::new(FREEZE_OVER[0], FREEZE_FRAME, FREEZE_FRAMES),
            freeze_blocks: FREEZE_BLOCKS
                .iter()
                .map(|png| Strip::new(png, FREEZE_FRAME, FREEZE_FRAMES))
                .collect(),
            lava: LAVA_PNG
                .iter()
                .map(|png| Strip::new(png, super::SPRITE, LAVA_FRAMES))
                .collect(),
            wind: WIND_PNG
                .iter()
                .map(|png| Strip::new(png, ELECTRIC_FIGURE, WIND_FRAMES))
                .collect(),
            electric: ELECTRIC_PNG
                .iter()
                .map(|png| Strip::new(png, ELECTRIC_FIGURE, ELECTRIC_FRAMES))
                .collect(),
        }
    }

    pub fn textures(&self, ctx: &egui::Context, k: usize) -> UiTextures {
        let load = |name: String, image: &Pixels| {
            ctx.load_texture(name, image.upscale(k), TextureOptions::LINEAR)
        };
        let all = |name: &str, images: &[Pixels]| -> Vec<TextureHandle> {
            images
                .iter()
                .enumerate()
                .map(|(i, image)| load(format!("{name}-{i}-x{k}"), image))
                .collect()
        };
        UiTextures {
            k,
            buttons: all("end-turn", &self.buttons),
            coins: all("coin", &self.coins),
            targets: all("target", &self.targets),
            wave: all("fire-wave", &self.wave),
            portal: all("portal", &self.portal),
            bless: all("bless", &self.bless),
            bless_figure: all("bless-figure", &self.bless_figure),
            freeze_tiles: self
                .freeze_tiles
                .iter()
                .enumerate()
                .map(|(v, strip)| strip.textures(ctx, &format!("freeze-tile-{v}"), k))
                .collect(),
            freeze_over: self.freeze_over.textures(ctx, "freeze-over", k),
            freeze_blocks: self
                .freeze_blocks
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("freeze-block-{i}"), k))
                .collect(),
            lava: self
                .lava
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("lava-{i}"), k))
                .collect(),
            wind: self
                .wind
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("wind-{i}"), k))
                .collect(),
            electric: self
                .electric
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("electric-{i}"), k))
                .collect(),
            turned: RefCell::new(HashMap::new()),
        }
    }

    /// Часть стрелки, повёрнутая по касательной: угол — в целых градусах, кратных 5,
    /// как в генераторе; повороты запоминаются.
    pub fn arrow(
        &self,
        ctx: &egui::Context,
        textures: &UiTextures,
        part: ArrowPart,
        aim: Aim,
        degrees: i32,
    ) -> Sprite {
        let index = aim as usize * 4 + part as usize;
        let degrees = degrees.rem_euclid(360);
        let mut cache = textures.turned.borrow_mut();
        cache
            .entry((index, degrees))
            .or_insert_with(|| {
                let image = rotsprite(
                    &self.arrow[index],
                    degrees as f32,
                    ARROW_OUTLINE[aim as usize],
                );
                sprite(ctx, &format!("arrow-{index}-{degrees}"), &image, textures.k)
            })
            .clone()
    }
}

/// Текстуры интерфейса при одном увеличении.
pub struct UiTextures {
    k: usize,
    buttons: Vec<TextureHandle>,
    coins: Vec<TextureHandle>,
    targets: Vec<TextureHandle>,
    wave: Vec<TextureHandle>,
    portal: Vec<TextureHandle>,
    bless: Vec<TextureHandle>,
    bless_figure: Vec<TextureHandle>,
    freeze_tiles: Vec<StripTextures>,
    freeze_over: StripTextures,
    freeze_blocks: Vec<StripTextures>,
    lava: Vec<StripTextures>,
    electric: Vec<StripTextures>,
    wind: Vec<StripTextures>,
    turned: RefCell<HashMap<(usize, i32), Sprite>>,
}

impl UiTextures {
    /// Кнопка в состоянии `button`; у нажатой картинка меньше — тень короче. `start` — надпись
    /// «Начать раунд» вместо «Конец хода».
    pub fn button(&self, button: Button, start: bool) -> &TextureHandle {
        &self.buttons[usize::from(start) * BUTTON_STATES + button as usize]
    }

    /// Монета: большая (`COIN`) или маленькая (`COIN_SMALL`).
    pub fn coin(&self, big: bool) -> &TextureHandle {
        &self.coins[usize::from(!big)]
    }

    pub fn target(&self, aim: Aim) -> &TextureHandle {
        &self.targets[aim as usize]
    }

    /// Кадр волны: `over` — слой над фигурами.
    pub fn wave(&self, over: bool, frame: usize) -> &TextureHandle {
        &self.wave[over as usize * WAVE_FRAMES + frame.min(WAVE_FRAMES - 1)]
    }

    /// Кадр портала: `appear` — фигура поднимается, иначе проваливается; `over` — слой
    /// над фигурами.
    pub fn portal(&self, appear: bool, over: bool, frame: usize) -> &TextureHandle {
        let layer = appear as usize * 2 + over as usize;
        &self.portal[layer * PORTAL_FRAMES + frame.min(PORTAL_FRAMES - 1)]
    }

    /// Кадр благословения: `over` — слой над фигурами.
    pub fn bless(&self, over: bool, frame: usize) -> &TextureHandle {
        &self.bless[over as usize * BLESS_FRAMES + frame.min(BLESS_FRAMES - 1)]
    }

    /// Слон стороны `color` с крыльями и ореолом — вместо обычного спрайта слона.
    pub fn bless_figure(&self, color: Color, frame: usize) -> &TextureHandle {
        &self.bless_figure[color as usize * BLESS_FRAMES + frame.min(BLESS_FRAMES - 1)]
    }

    /// Снег на клетке варианта `variant` в кадре заморозки `frame`.
    pub fn freeze_tile(&self, variant: usize, frame: usize) -> Option<&FrameSprite> {
        self.freeze_tiles[variant % self.freeze_tiles.len()].frame(frame)
    }

    /// Снежинки, пар и искры над клеткой в кадре заморозки `frame`.
    pub fn freeze_over(&self, frame: usize) -> Option<&FrameSprite> {
        self.freeze_over.frame(frame)
    }

    /// Глыба льда вокруг фигуры `kind` варианта `variant` в кадре заморозки `frame`.
    pub fn freeze_block(&self, kind: Kind, variant: usize, frame: usize) -> Option<&FrameSprite> {
        self.freeze_blocks[kind as usize * 4 + variant % 4].frame(frame)
    }

    /// Фигура `kind` стороны `color` с лавой в кадре `frame` — на холсте фигуры.
    pub fn lava(&self, color: Color, kind: Kind, frame: usize) -> Option<&FrameSprite> {
        self.lava[super::index(color, kind)].frame(frame)
    }

    /// Слон стороны `color` с электричеством в кадре `frame` — вокруг `ELECTRIC_ANCHOR`.
    pub fn electric(&self, color: Color, kind: Kind, frame: usize) -> Option<&FrameSprite> {
        self.electric[super::index(color, kind)].frame(frame)
    }

    /// Воздушная фигура `kind` стороны `color` в вихре, кадр `frame` — вокруг `ELECTRIC_ANCHOR`.
    pub fn wind(&self, color: Color, kind: Kind, frame: usize) -> Option<&FrameSprite> {
        self.wind[super::index(color, kind)].frame(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_pictures_match_the_generators() {
        let art = UiArt::new();
        for (button, image) in art.buttons.iter().enumerate() {
            // Тень кнопки — 2×3 пикселя, у нажатой — 1×1.
            let shadow = if button % BUTTON_STATES == Button::Pressed as usize {
                vec2(1.0, 1.0)
            } else {
                vec2(2.0, 3.0)
            };
            let size = BUTTON + shadow;
            assert_eq!(
                [image.width, image.height],
                [size.x as usize, size.y as usize]
            );
        }
        for (image, size) in art.coins.iter().zip([COIN, COIN_SMALL]) {
            assert_eq!(
                [image.width, image.height],
                [size.x as usize, size.y as usize]
            );
        }
        for image in &art.targets {
            let size = super::super::TILE + Vec2::splat(2.0 * TARGET_PAD);
            assert_eq!(
                [image.width, image.height],
                [size.x as usize, size.y as usize]
            );
        }
        assert_eq!(art.wave.len(), 2 * WAVE_FRAMES);
        assert_eq!([art.wave[0].width, art.wave[0].height], [184, 168]);
        assert_eq!(art.bless.len(), 2 * BLESS_FRAMES);
        assert_eq!(art.bless_figure.len(), 2 * BLESS_FRAMES);
        let figure = &art.bless_figure[10];
        assert_eq!([figure.width, figure.height], [120, 112]);
        // Слон стоит основанием на точке опоры.
        assert!(figure.alpha(BLESS_ANCHOR.x as usize, BLESS_ANCHOR.y as usize - 3) > 0);
        assert_eq!(art.portal.len(), 4 * PORTAL_FRAMES);
        assert_eq!([art.portal[0].width, art.portal[0].height], [80, 116]);
        // Раскрытый портал (кадр 6 исчезновения): вихрь — на нижнем слое вокруг центра
        // клетки, передняя половина кольца — на верхнем.
        let (under, over) = (&art.portal[6], &art.portal[PORTAL_FRAMES + 6]);
        let (cx, cy) = (PORTAL_CENTER.x as usize, PORTAL_CENTER.y as usize);
        assert!(under.alpha(cx, cy) > 0 && under.alpha(cx + 5, cy) > 0);
        assert!(under.alpha(cx + 24, cy) == 0);
        assert!(over.alpha(cx, cy + 12) > 0, "передняя кромка кольца");
        assert!(over.alpha(cx, cy - 12) == 0, "задняя кромка — под фигурой");
        // Последний кадр исчезновения пуст: портал схлопнулся.
        let shut = &art.portal[PORTAL_FRAMES - 1];
        assert!(shut.rgba.iter().all(|p| p[3] == 0));
        // Наконечник смотрит вправо: правый край у середины по высоте непрозрачен.
        let head = &art.arrow[ArrowPart::Head as usize];
        assert!(head.alpha(head.width - 2, head.height / 2) > 0);
    }

    #[test]
    fn freeze_frames_match_the_generator() {
        let art = UiArt::new();
        assert_eq!(art.freeze_tiles.len(), 4);
        assert_eq!(art.freeze_blocks.len(), 24);
        let center = |strip: &Strip, frame: usize| {
            let (image, at) = &strip.images[strip.frames[frame].expect("кадр не пуст")];
            let (x, y) = (FREEZE_CENTER.x - at.x, FREEZE_CENTER.y - at.y);
            image.alpha(x as usize, y as usize)
        };
        for strip in art.freeze_tiles.iter().chain(&art.freeze_blocks) {
            assert_eq!(strip.frames.len(), FREEZE_FRAMES);
        }
        // Снег в конце покрывает середину клетки, в первом кадре его нет.
        let tile = &art.freeze_tiles[0];
        assert!(tile.frames[0].is_none());
        assert!(center(tile, FREEZE_FRAMES - 1) > 0);
        // Глыба растёт с 8-го кадра и стоит вокруг клетки фигуры.
        let block = &art.freeze_blocks[0];
        assert!(block.frames[7].is_none());
        assert!(center(block, FREEZE_FRAMES - 1) > 0);
        // Одинаковые кадры (снег уже лёг) хранятся один раз.
        assert!(tile.images.len() < FREEZE_FRAMES / 2);
    }
}
