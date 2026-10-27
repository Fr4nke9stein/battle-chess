//! Интерфейс стола из арт-проекта: кнопка «Конец хода» (`tools/breach_ui.py`), стрелка
//! прицеливания картой со звеньями и метками цели (`tools/breach_arrow.py`), огненная
//! волна (`tools/breach_firewave.py`), портал обмена (`tools/breach_portal.py`) и
//! благословение слона для исцеления (`tools/breach_bless.py`).

use std::cell::RefCell;
use std::collections::HashMap;

use eframe::egui::{self, TextureHandle, TextureOptions, Vec2, vec2};

use super::Pixels;
use super::cards::{Sprite, rotsprite, sprite};
use crate::game::{Card, Chest, Color, Crest, Kind, Piece, Power};

/// Кнопка «Конец хода» без тени; тень лежит в картинке справа и снизу.
pub const BUTTON: Vec2 = vec2(53.0, 35.0);
/// Монета из арт-проекта (`tools/breach_coin.py`): большая — счётчик золота rogue-like,
/// маленькая — цены в магазине.
pub const COIN: Vec2 = vec2(32.0, 32.0);
pub const COIN_SMALL: Vec2 = vec2(16.0, 16.0);
/// Гербы rogue-like из арт-проекта (`tools/breach_crest.py`), без тени.
pub const CREST: Vec2 = vec2(49.0, 45.0);
/// Сундуки магазина rogue-like (`tools/breach_chest.py`, с 0.110), без тени.
pub const CHEST: Vec2 = vec2(46.0, 40.0);
/// Большие сундуки (с 0.113).
pub const BIG_CHEST: Vec2 = vec2(55.0, 46.0);
/// Карты сил rogue-like (`tools/breach_power_cards.py`, с 0.111), без тени.
pub const POWER_CARD: Vec2 = vec2(51.0, 70.0);
/// Кадр огненной волны и центр верхней грани клетки фигуры в нём.
pub const WAVE_FRAME: Vec2 = vec2(184.0, 168.0);
pub const WAVE_CENTER: Vec2 = vec2(92.0, 98.0);
pub const WAVE_FRAMES: usize = 20;
pub const WAVE_FRAME_TIME: f64 = 0.06;
/// Большая огненная волна крестом (`tools/breach_firewave_plus.py`, с 0.87): кадр 304×248, центр
/// верхней грани клетки фигуры — `BIG_WAVE_CENTER`, кадры — по `WAVE_FRAME_TIME`.
pub const BIG_WAVE_FRAME: Vec2 = vec2(304.0, 248.0);
pub const BIG_WAVE_CENTER: Vec2 = vec2(152.0, 132.0);
pub const BIG_WAVE_FRAMES: usize = 26;
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
/// Электричество фигуры (с 0.116 — замедленное `electric_slow_*` из `tools/breach_spark_slow.py`:
/// движение в `ELECTRIC_SLOW` раз медленнее, чем у `electric_hold`, кадров во столько же раз
/// больше): кадр больше спрайта, точка опоры — центр клетки, по 60 мс. Вступление со вспышкой
/// `ELECTRIC_FLASH`, потом без конца повторяются кадры `ELECTRIC_LOOP`.
pub const ELECTRIC_SLOW: usize = 3;
pub const ELECTRIC_FRAMES: usize = 24 * ELECTRIC_SLOW;
pub const ELECTRIC_FRAME_TIME: f64 = 0.06;
pub const ELECTRIC_FIGURE: Vec2 = vec2(120.0, 120.0);
pub const ELECTRIC_ANCHOR: Vec2 = vec2(60.0, 92.0);
pub const ELECTRIC_FLASH: usize = 5 * ELECTRIC_SLOW;
pub const ELECTRIC_LOOP: [usize; 2] = [8 * ELECTRIC_SLOW, 24 * ELECTRIC_SLOW];
/// Вихрь воздушной фигуры (`tools/breach_wind.py`, с 0.67): кадр 120×120 с той же точкой
/// опоры, что у электричества, 60 мс; вступление [0, 8), повтор [8, 40).
pub const WIND_FRAMES: usize = 40;
pub const WIND_LOOP: [usize; 2] = [8, 40];
/// Ядовитая фигура (`tools/breach_poison.py`, с 0.79): лужа и капли — кадр 120×120 с той же
/// опорой, 60 мс; вступление [0, 8), повтор [8, 40). След шага — `POISON_TRAIL` 96×80, середина
/// между центрами клеток — `POISON_TRAIL_CENTER`, `POISON_TRAIL_FRAMES` кадров по 60 мс.
pub const POISON_FRAMES: usize = 40;
pub const POISON_LOOP: [usize; 2] = [8, 40];
pub const POISON_TRAIL: Vec2 = vec2(96.0, 80.0);
pub const POISON_TRAIL_CENTER: Vec2 = vec2(48.0, 40.0);
pub const POISON_TRAIL_FRAMES: usize = 30;
/// Метеорит мага (`tools/breach_meteor.py`, с 0.83): кадр `METEOR` с центром клетки
/// `METEOR_CENTER`, `METEOR_FRAMES` кадров по `METEOR_FRAME_TIME`; касается клетки в кадре
/// `METEOR_IMPACT`. С 0.84 (просьба пользователя) — крупный камень с кратерами в широком хвосте
/// пламени, кадров вдвое больше и они короче — плавнее.
pub const METEOR: Vec2 = vec2(200.0, 280.0);
pub const METEOR_CENTER: Vec2 = vec2(72.0, 250.0);
pub const METEOR_FRAMES: usize = 44;
pub const METEOR_IMPACT: usize = 16;
pub const METEOR_FRAME_TIME: f64 = 0.03;
/// Горящий кратер (`tools/breach_crater.py`, с 0.94): кадр 80×112, центр клетки (40, 84),
/// повтор из `CRATER_FRAMES` кадров по `CRATER_FRAME_TIME`, слои под фигурами и над ними.
pub const CRATER: Vec2 = vec2(80.0, 112.0);
pub const CRATER_CENTER: Vec2 = vec2(40.0, 84.0);
pub const CRATER_FRAMES: usize = 12;
pub const CRATER_FRAME_TIME: f64 = 0.08;
/// Волна метеорита крестом (`tools/breach_meteor_wave.py`): кадр как у огненной волны,
/// `METEOR_WAVE_FRAMES` кадров по `WAVE_FRAME_TIME`.
pub const METEOR_WAVE_FRAMES: usize = 18;
/// Теневая фигура (с 0.70, арт-проект): тёмная аура `dark_fire_*` (`tools/breach_darkfire.py`,
/// 40 кадров, вступление [0, 8), повтор [8, 40)) и теневой шаг (`tools/breach_shadowstep.py`):
/// исчезновение на исходной клетке `STEP_OUT` кадров, появление на целевой `STEP_IN` кадров
/// с кадра исчезновения `STEP_IN_AT`. Кадр и опора — как у электричества.
pub const DARK_FRAMES: usize = 40;
pub const DARK_LOOP: [usize; 2] = [8, 40];
pub const STEP_OUT: usize = 18;
pub const STEP_IN: usize = 13;
pub const STEP_IN_AT: usize = 7;
/// Кадр теневого шага (с 0.71, просьба пользователя: вдвое быстрее — 30 мс вместо 60).
pub const STEP_FRAME_TIME: f64 = 0.03;

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
/// Сундуки в порядке `Chest::ALL`.
const CHESTS: [&[u8]; 6] = breach!(
    "ui/chest_pieces.png",
    "ui/chest_crests.png",
    "ui/chest_cards.png",
    "ui/chest_pieces_big.png",
    "ui/chest_crests_big.png",
    "ui/chest_cards_big.png"
);
/// Карты rogue-like в порядке `Card::ALL`: силы, потом маска, герб, две монеты, монета, зеркало,
/// ранг, перо.
const POWER_CARDS: [&[u8]; 15] = breach!(
    "ui/card_fire.png",
    "ui/card_spark.png",
    "ui/card_wind.png",
    "ui/card_gold.png",
    "ui/card_shade.png",
    "ui/card_ice.png",
    "ui/card_barbarian.png",
    "ui/card_wizard.png",
    "ui/card_mask.png",
    "ui/card_crest.png",
    "ui/card_double.png",
    "ui/card_coin.png",
    "ui/card_mirror.png",
    "ui/card_rank.png",
    "ui/card_feather.png"
);
/// Гербы в порядке `Crest::ALL`.
const CRESTS: [&[u8]; 43] = breach!(
    "ui/crest_heart.png",
    "ui/crest_snow2.png",
    "ui/crest_crystal.png",
    "ui/crest_axe_sword.png",
    "ui/crest_axe_fangs.png",
    "ui/crest_whirl.png",
    "ui/crest_fire_axe.png",
    "ui/crest_flames.png",
    "ui/crest_flame_skull.png",
    "ui/crest_flame_sword.png",
    "ui/crest_bomb.png",
    "ui/crest_dark_skull.png",
    "ui/crest_dark_flames.png",
    "ui/crest_dark_portal.png",
    "ui/crest_dark_eye.png",
    "ui/crest_winds.png",
    "ui/crest_wind_sword.png",
    "ui/crest_bolt_plus.png",
    "ui/crest_bolts.png",
    "ui/crest_red_bolt.png",
    "ui/crest_bolt_one.png",
    "ui/crest_reroll_coin.png",
    "ui/crest_hats.png",
    "ui/crest_meteor.png",
    "ui/crest_hat_meteor.png",
    "ui/crest_hat_plus.png",
    "ui/crest_gold_king.png",
    "ui/crest_king_queen.png",
    "ui/crest_king_skull.png",
    "ui/crest_piggy.png",
    "ui/crest_mult.png",
    "ui/crest_king_mult.png",
    "ui/crest_few.png",
    "ui/crest_gold_mult.png",
    "ui/crest_pawn_mult.png",
    "ui/crest_knight_mult.png",
    "ui/crest_sword_mult.png",
    "ui/crest_first.png",
    "ui/crest_queen_mult.png",
    "ui/crest_flame_mult.png",
    "ui/crest_snow_mult.png",
    "ui/crest_bolt_mult.png",
    "ui/crest_wind_mult.png",
);
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
const BIG_WAVE: [&[u8]; 2] = breach!(
    "effects/fire_wave_plus_under.png",
    "effects/fire_wave_plus_over.png"
);
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
/// Анимации объединённых фигур — те же полосы на спрайте варвара (`tools/breach_barbarian_combo.py`,
/// с 0.96): идут в массивах следом за обычными, `+ BARBARIAN` к номеру.
const BARBARIAN: usize = 12;
/// Золотые объединённые (`tools/breach_gold_combo.py`, с 0.97) — следом за варварами, `+ GOLD`.
const GOLD: usize = 24;
/// Маги-объединённые (`tools/breach_wizard_combo.py`, с 0.98) — следом за золотыми, `+ WIZARD`.
const WIZARD: usize = 36;
/// Ледяные объединённые (`tools/breach_ice_combo.py`, с 0.99) — следом за магами, `+ ICE`.
const ICE: usize = 48;
/// Огненные объединённые (`tools/breach_fire_combo.py`, с 0.101): кадры силы на теле с трещинами
/// ожидания взрыва — следом за ледяными, `+ FIRE`; своей лавы у набора нет (взрыв — обычная).
const FIRE: usize = 60;
/// Электрические объединённые (`tools/breach_spark_combo.py`, с 0.102): вихрь и тёмная аура вместе с
/// молниями (40 кадров, как у вихря и ауры) — следом за огненными, `+ SPARK`; своего набора
/// электричества нет.
const SPARK: usize = 72;
/// Воздушные теневые (`tools/breach_wind_combo.py`, с 0.103): тёмная аура с вихрем и теневой шаг —
/// следом за электрическими, `+ WIND`; только у тени.
const WIND: usize = 84;
const WIND_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/wind/dark_fire_wP.png",
    "effects/wind/dark_fire_wN.png",
    "effects/wind/dark_fire_wB.png",
    "effects/wind/dark_fire_wR.png",
    "effects/wind/dark_fire_wQ.png",
    "effects/wind/dark_fire_wK.png",
    "effects/wind/dark_fire_bP.png",
    "effects/wind/dark_fire_bN.png",
    "effects/wind/dark_fire_bB.png",
    "effects/wind/dark_fire_bR.png",
    "effects/wind/dark_fire_bQ.png",
    "effects/wind/dark_fire_bK.png",
);
const WIND_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/wind/shadowstep_out_wP.png",
    "effects/wind/shadowstep_out_wN.png",
    "effects/wind/shadowstep_out_wB.png",
    "effects/wind/shadowstep_out_wR.png",
    "effects/wind/shadowstep_out_wQ.png",
    "effects/wind/shadowstep_out_wK.png",
    "effects/wind/shadowstep_out_bP.png",
    "effects/wind/shadowstep_out_bN.png",
    "effects/wind/shadowstep_out_bB.png",
    "effects/wind/shadowstep_out_bR.png",
    "effects/wind/shadowstep_out_bQ.png",
    "effects/wind/shadowstep_out_bK.png",
);
const WIND_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/wind/shadowstep_in_wP.png",
    "effects/wind/shadowstep_in_wN.png",
    "effects/wind/shadowstep_in_wB.png",
    "effects/wind/shadowstep_in_wR.png",
    "effects/wind/shadowstep_in_wQ.png",
    "effects/wind/shadowstep_in_wK.png",
    "effects/wind/shadowstep_in_bP.png",
    "effects/wind/shadowstep_in_bN.png",
    "effects/wind/shadowstep_in_bB.png",
    "effects/wind/shadowstep_in_bR.png",
    "effects/wind/shadowstep_in_bQ.png",
    "effects/wind/shadowstep_in_bK.png",
);
const SPARK_WIND_PNG: [&[u8]; 12] = breach!(
    "effects/spark/wind_wP.png",
    "effects/spark/wind_wN.png",
    "effects/spark/wind_wB.png",
    "effects/spark/wind_wR.png",
    "effects/spark/wind_wQ.png",
    "effects/spark/wind_wK.png",
    "effects/spark/wind_bP.png",
    "effects/spark/wind_bN.png",
    "effects/spark/wind_bB.png",
    "effects/spark/wind_bR.png",
    "effects/spark/wind_bQ.png",
    "effects/spark/wind_bK.png",
);
const SPARK_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/spark/dark_fire_wP.png",
    "effects/spark/dark_fire_wN.png",
    "effects/spark/dark_fire_wB.png",
    "effects/spark/dark_fire_wR.png",
    "effects/spark/dark_fire_wQ.png",
    "effects/spark/dark_fire_wK.png",
    "effects/spark/dark_fire_bP.png",
    "effects/spark/dark_fire_bN.png",
    "effects/spark/dark_fire_bB.png",
    "effects/spark/dark_fire_bR.png",
    "effects/spark/dark_fire_bQ.png",
    "effects/spark/dark_fire_bK.png",
);
const SPARK_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/spark/shadowstep_out_wP.png",
    "effects/spark/shadowstep_out_wN.png",
    "effects/spark/shadowstep_out_wB.png",
    "effects/spark/shadowstep_out_wR.png",
    "effects/spark/shadowstep_out_wQ.png",
    "effects/spark/shadowstep_out_wK.png",
    "effects/spark/shadowstep_out_bP.png",
    "effects/spark/shadowstep_out_bN.png",
    "effects/spark/shadowstep_out_bB.png",
    "effects/spark/shadowstep_out_bR.png",
    "effects/spark/shadowstep_out_bQ.png",
    "effects/spark/shadowstep_out_bK.png",
);
const SPARK_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/spark/shadowstep_in_wP.png",
    "effects/spark/shadowstep_in_wN.png",
    "effects/spark/shadowstep_in_wB.png",
    "effects/spark/shadowstep_in_wR.png",
    "effects/spark/shadowstep_in_wQ.png",
    "effects/spark/shadowstep_in_wK.png",
    "effects/spark/shadowstep_in_bP.png",
    "effects/spark/shadowstep_in_bN.png",
    "effects/spark/shadowstep_in_bB.png",
    "effects/spark/shadowstep_in_bR.png",
    "effects/spark/shadowstep_in_bQ.png",
    "effects/spark/shadowstep_in_bK.png",
);
const FIRE_ELECTRIC_PNG: [&[u8]; 12] = breach!(
    "effects/fire/electric_slow_wP.png",
    "effects/fire/electric_slow_wN.png",
    "effects/fire/electric_slow_wB.png",
    "effects/fire/electric_slow_wR.png",
    "effects/fire/electric_slow_wQ.png",
    "effects/fire/electric_slow_wK.png",
    "effects/fire/electric_slow_bP.png",
    "effects/fire/electric_slow_bN.png",
    "effects/fire/electric_slow_bB.png",
    "effects/fire/electric_slow_bR.png",
    "effects/fire/electric_slow_bQ.png",
    "effects/fire/electric_slow_bK.png",
);
const FIRE_WIND_PNG: [&[u8]; 12] = breach!(
    "effects/fire/wind_wP.png",
    "effects/fire/wind_wN.png",
    "effects/fire/wind_wB.png",
    "effects/fire/wind_wR.png",
    "effects/fire/wind_wQ.png",
    "effects/fire/wind_wK.png",
    "effects/fire/wind_bP.png",
    "effects/fire/wind_bN.png",
    "effects/fire/wind_bB.png",
    "effects/fire/wind_bR.png",
    "effects/fire/wind_bQ.png",
    "effects/fire/wind_bK.png",
);
const FIRE_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/fire/dark_fire_wP.png",
    "effects/fire/dark_fire_wN.png",
    "effects/fire/dark_fire_wB.png",
    "effects/fire/dark_fire_wR.png",
    "effects/fire/dark_fire_wQ.png",
    "effects/fire/dark_fire_wK.png",
    "effects/fire/dark_fire_bP.png",
    "effects/fire/dark_fire_bN.png",
    "effects/fire/dark_fire_bB.png",
    "effects/fire/dark_fire_bR.png",
    "effects/fire/dark_fire_bQ.png",
    "effects/fire/dark_fire_bK.png",
);
const FIRE_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/fire/shadowstep_out_wP.png",
    "effects/fire/shadowstep_out_wN.png",
    "effects/fire/shadowstep_out_wB.png",
    "effects/fire/shadowstep_out_wR.png",
    "effects/fire/shadowstep_out_wQ.png",
    "effects/fire/shadowstep_out_wK.png",
    "effects/fire/shadowstep_out_bP.png",
    "effects/fire/shadowstep_out_bN.png",
    "effects/fire/shadowstep_out_bB.png",
    "effects/fire/shadowstep_out_bR.png",
    "effects/fire/shadowstep_out_bQ.png",
    "effects/fire/shadowstep_out_bK.png",
);
const FIRE_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/fire/shadowstep_in_wP.png",
    "effects/fire/shadowstep_in_wN.png",
    "effects/fire/shadowstep_in_wB.png",
    "effects/fire/shadowstep_in_wR.png",
    "effects/fire/shadowstep_in_wQ.png",
    "effects/fire/shadowstep_in_wK.png",
    "effects/fire/shadowstep_in_bP.png",
    "effects/fire/shadowstep_in_bN.png",
    "effects/fire/shadowstep_in_bB.png",
    "effects/fire/shadowstep_in_bR.png",
    "effects/fire/shadowstep_in_bQ.png",
    "effects/fire/shadowstep_in_bK.png",
);
const ICE_LAVA_PNG: [&[u8]; 12] = breach!(
    "effects/ice/lava_crack_wP.png",
    "effects/ice/lava_crack_wN.png",
    "effects/ice/lava_crack_wB.png",
    "effects/ice/lava_crack_wR.png",
    "effects/ice/lava_crack_wQ.png",
    "effects/ice/lava_crack_wK.png",
    "effects/ice/lava_crack_bP.png",
    "effects/ice/lava_crack_bN.png",
    "effects/ice/lava_crack_bB.png",
    "effects/ice/lava_crack_bR.png",
    "effects/ice/lava_crack_bQ.png",
    "effects/ice/lava_crack_bK.png",
);
const ICE_ELECTRIC_PNG: [&[u8]; 12] = breach!(
    "effects/ice/electric_slow_wP.png",
    "effects/ice/electric_slow_wN.png",
    "effects/ice/electric_slow_wB.png",
    "effects/ice/electric_slow_wR.png",
    "effects/ice/electric_slow_wQ.png",
    "effects/ice/electric_slow_wK.png",
    "effects/ice/electric_slow_bP.png",
    "effects/ice/electric_slow_bN.png",
    "effects/ice/electric_slow_bB.png",
    "effects/ice/electric_slow_bR.png",
    "effects/ice/electric_slow_bQ.png",
    "effects/ice/electric_slow_bK.png",
);
const ICE_WIND_PNG: [&[u8]; 12] = breach!(
    "effects/ice/wind_wP.png",
    "effects/ice/wind_wN.png",
    "effects/ice/wind_wB.png",
    "effects/ice/wind_wR.png",
    "effects/ice/wind_wQ.png",
    "effects/ice/wind_wK.png",
    "effects/ice/wind_bP.png",
    "effects/ice/wind_bN.png",
    "effects/ice/wind_bB.png",
    "effects/ice/wind_bR.png",
    "effects/ice/wind_bQ.png",
    "effects/ice/wind_bK.png",
);
const ICE_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/ice/dark_fire_wP.png",
    "effects/ice/dark_fire_wN.png",
    "effects/ice/dark_fire_wB.png",
    "effects/ice/dark_fire_wR.png",
    "effects/ice/dark_fire_wQ.png",
    "effects/ice/dark_fire_wK.png",
    "effects/ice/dark_fire_bP.png",
    "effects/ice/dark_fire_bN.png",
    "effects/ice/dark_fire_bB.png",
    "effects/ice/dark_fire_bR.png",
    "effects/ice/dark_fire_bQ.png",
    "effects/ice/dark_fire_bK.png",
);
const ICE_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/ice/shadowstep_out_wP.png",
    "effects/ice/shadowstep_out_wN.png",
    "effects/ice/shadowstep_out_wB.png",
    "effects/ice/shadowstep_out_wR.png",
    "effects/ice/shadowstep_out_wQ.png",
    "effects/ice/shadowstep_out_wK.png",
    "effects/ice/shadowstep_out_bP.png",
    "effects/ice/shadowstep_out_bN.png",
    "effects/ice/shadowstep_out_bB.png",
    "effects/ice/shadowstep_out_bR.png",
    "effects/ice/shadowstep_out_bQ.png",
    "effects/ice/shadowstep_out_bK.png",
);
const ICE_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/ice/shadowstep_in_wP.png",
    "effects/ice/shadowstep_in_wN.png",
    "effects/ice/shadowstep_in_wB.png",
    "effects/ice/shadowstep_in_wR.png",
    "effects/ice/shadowstep_in_wQ.png",
    "effects/ice/shadowstep_in_wK.png",
    "effects/ice/shadowstep_in_bP.png",
    "effects/ice/shadowstep_in_bN.png",
    "effects/ice/shadowstep_in_bB.png",
    "effects/ice/shadowstep_in_bR.png",
    "effects/ice/shadowstep_in_bQ.png",
    "effects/ice/shadowstep_in_bK.png",
);
const WIZARD_LAVA_PNG: [&[u8]; 12] = breach!(
    "effects/wizard/lava_crack_wP.png",
    "effects/wizard/lava_crack_wN.png",
    "effects/wizard/lava_crack_wB.png",
    "effects/wizard/lava_crack_wR.png",
    "effects/wizard/lava_crack_wQ.png",
    "effects/wizard/lava_crack_wK.png",
    "effects/wizard/lava_crack_bP.png",
    "effects/wizard/lava_crack_bN.png",
    "effects/wizard/lava_crack_bB.png",
    "effects/wizard/lava_crack_bR.png",
    "effects/wizard/lava_crack_bQ.png",
    "effects/wizard/lava_crack_bK.png",
);
const WIZARD_ELECTRIC_PNG: [&[u8]; 12] = breach!(
    "effects/wizard/electric_slow_wP.png",
    "effects/wizard/electric_slow_wN.png",
    "effects/wizard/electric_slow_wB.png",
    "effects/wizard/electric_slow_wR.png",
    "effects/wizard/electric_slow_wQ.png",
    "effects/wizard/electric_slow_wK.png",
    "effects/wizard/electric_slow_bP.png",
    "effects/wizard/electric_slow_bN.png",
    "effects/wizard/electric_slow_bB.png",
    "effects/wizard/electric_slow_bR.png",
    "effects/wizard/electric_slow_bQ.png",
    "effects/wizard/electric_slow_bK.png",
);
const WIZARD_WIND_PNG: [&[u8]; 12] = breach!(
    "effects/wizard/wind_wP.png",
    "effects/wizard/wind_wN.png",
    "effects/wizard/wind_wB.png",
    "effects/wizard/wind_wR.png",
    "effects/wizard/wind_wQ.png",
    "effects/wizard/wind_wK.png",
    "effects/wizard/wind_bP.png",
    "effects/wizard/wind_bN.png",
    "effects/wizard/wind_bB.png",
    "effects/wizard/wind_bR.png",
    "effects/wizard/wind_bQ.png",
    "effects/wizard/wind_bK.png",
);
const WIZARD_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/wizard/dark_fire_wP.png",
    "effects/wizard/dark_fire_wN.png",
    "effects/wizard/dark_fire_wB.png",
    "effects/wizard/dark_fire_wR.png",
    "effects/wizard/dark_fire_wQ.png",
    "effects/wizard/dark_fire_wK.png",
    "effects/wizard/dark_fire_bP.png",
    "effects/wizard/dark_fire_bN.png",
    "effects/wizard/dark_fire_bB.png",
    "effects/wizard/dark_fire_bR.png",
    "effects/wizard/dark_fire_bQ.png",
    "effects/wizard/dark_fire_bK.png",
);
const WIZARD_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/wizard/shadowstep_out_wP.png",
    "effects/wizard/shadowstep_out_wN.png",
    "effects/wizard/shadowstep_out_wB.png",
    "effects/wizard/shadowstep_out_wR.png",
    "effects/wizard/shadowstep_out_wQ.png",
    "effects/wizard/shadowstep_out_wK.png",
    "effects/wizard/shadowstep_out_bP.png",
    "effects/wizard/shadowstep_out_bN.png",
    "effects/wizard/shadowstep_out_bB.png",
    "effects/wizard/shadowstep_out_bR.png",
    "effects/wizard/shadowstep_out_bQ.png",
    "effects/wizard/shadowstep_out_bK.png",
);
const WIZARD_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/wizard/shadowstep_in_wP.png",
    "effects/wizard/shadowstep_in_wN.png",
    "effects/wizard/shadowstep_in_wB.png",
    "effects/wizard/shadowstep_in_wR.png",
    "effects/wizard/shadowstep_in_wQ.png",
    "effects/wizard/shadowstep_in_wK.png",
    "effects/wizard/shadowstep_in_bP.png",
    "effects/wizard/shadowstep_in_bN.png",
    "effects/wizard/shadowstep_in_bB.png",
    "effects/wizard/shadowstep_in_bR.png",
    "effects/wizard/shadowstep_in_bQ.png",
    "effects/wizard/shadowstep_in_bK.png",
);
const GOLD_LAVA_PNG: [&[u8]; 12] = breach!(
    "effects/gold/lava_crack_wP.png",
    "effects/gold/lava_crack_wN.png",
    "effects/gold/lava_crack_wB.png",
    "effects/gold/lava_crack_wR.png",
    "effects/gold/lava_crack_wQ.png",
    "effects/gold/lava_crack_wK.png",
    "effects/gold/lava_crack_bP.png",
    "effects/gold/lava_crack_bN.png",
    "effects/gold/lava_crack_bB.png",
    "effects/gold/lava_crack_bR.png",
    "effects/gold/lava_crack_bQ.png",
    "effects/gold/lava_crack_bK.png",
);
const GOLD_ELECTRIC_PNG: [&[u8]; 12] = breach!(
    "effects/gold/electric_slow_wP.png",
    "effects/gold/electric_slow_wN.png",
    "effects/gold/electric_slow_wB.png",
    "effects/gold/electric_slow_wR.png",
    "effects/gold/electric_slow_wQ.png",
    "effects/gold/electric_slow_wK.png",
    "effects/gold/electric_slow_bP.png",
    "effects/gold/electric_slow_bN.png",
    "effects/gold/electric_slow_bB.png",
    "effects/gold/electric_slow_bR.png",
    "effects/gold/electric_slow_bQ.png",
    "effects/gold/electric_slow_bK.png",
);
const GOLD_WIND_PNG: [&[u8]; 12] = breach!(
    "effects/gold/wind_wP.png",
    "effects/gold/wind_wN.png",
    "effects/gold/wind_wB.png",
    "effects/gold/wind_wR.png",
    "effects/gold/wind_wQ.png",
    "effects/gold/wind_wK.png",
    "effects/gold/wind_bP.png",
    "effects/gold/wind_bN.png",
    "effects/gold/wind_bB.png",
    "effects/gold/wind_bR.png",
    "effects/gold/wind_bQ.png",
    "effects/gold/wind_bK.png",
);
const GOLD_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/gold/dark_fire_wP.png",
    "effects/gold/dark_fire_wN.png",
    "effects/gold/dark_fire_wB.png",
    "effects/gold/dark_fire_wR.png",
    "effects/gold/dark_fire_wQ.png",
    "effects/gold/dark_fire_wK.png",
    "effects/gold/dark_fire_bP.png",
    "effects/gold/dark_fire_bN.png",
    "effects/gold/dark_fire_bB.png",
    "effects/gold/dark_fire_bR.png",
    "effects/gold/dark_fire_bQ.png",
    "effects/gold/dark_fire_bK.png",
);
const GOLD_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/gold/shadowstep_out_wP.png",
    "effects/gold/shadowstep_out_wN.png",
    "effects/gold/shadowstep_out_wB.png",
    "effects/gold/shadowstep_out_wR.png",
    "effects/gold/shadowstep_out_wQ.png",
    "effects/gold/shadowstep_out_wK.png",
    "effects/gold/shadowstep_out_bP.png",
    "effects/gold/shadowstep_out_bN.png",
    "effects/gold/shadowstep_out_bB.png",
    "effects/gold/shadowstep_out_bR.png",
    "effects/gold/shadowstep_out_bQ.png",
    "effects/gold/shadowstep_out_bK.png",
);
const GOLD_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/gold/shadowstep_in_wP.png",
    "effects/gold/shadowstep_in_wN.png",
    "effects/gold/shadowstep_in_wB.png",
    "effects/gold/shadowstep_in_wR.png",
    "effects/gold/shadowstep_in_wQ.png",
    "effects/gold/shadowstep_in_wK.png",
    "effects/gold/shadowstep_in_bP.png",
    "effects/gold/shadowstep_in_bN.png",
    "effects/gold/shadowstep_in_bB.png",
    "effects/gold/shadowstep_in_bR.png",
    "effects/gold/shadowstep_in_bQ.png",
    "effects/gold/shadowstep_in_bK.png",
);
const BARBARIAN_LAVA_PNG: [&[u8]; 12] = breach!(
    "effects/barbarian/lava_crack_wP.png",
    "effects/barbarian/lava_crack_wN.png",
    "effects/barbarian/lava_crack_wB.png",
    "effects/barbarian/lava_crack_wR.png",
    "effects/barbarian/lava_crack_wQ.png",
    "effects/barbarian/lava_crack_wK.png",
    "effects/barbarian/lava_crack_bP.png",
    "effects/barbarian/lava_crack_bN.png",
    "effects/barbarian/lava_crack_bB.png",
    "effects/barbarian/lava_crack_bR.png",
    "effects/barbarian/lava_crack_bQ.png",
    "effects/barbarian/lava_crack_bK.png",
);
const BARBARIAN_ELECTRIC_PNG: [&[u8]; 12] = breach!(
    "effects/barbarian/electric_slow_wP.png",
    "effects/barbarian/electric_slow_wN.png",
    "effects/barbarian/electric_slow_wB.png",
    "effects/barbarian/electric_slow_wR.png",
    "effects/barbarian/electric_slow_wQ.png",
    "effects/barbarian/electric_slow_wK.png",
    "effects/barbarian/electric_slow_bP.png",
    "effects/barbarian/electric_slow_bN.png",
    "effects/barbarian/electric_slow_bB.png",
    "effects/barbarian/electric_slow_bR.png",
    "effects/barbarian/electric_slow_bQ.png",
    "effects/barbarian/electric_slow_bK.png",
);
const BARBARIAN_WIND_PNG: [&[u8]; 12] = breach!(
    "effects/barbarian/wind_wP.png",
    "effects/barbarian/wind_wN.png",
    "effects/barbarian/wind_wB.png",
    "effects/barbarian/wind_wR.png",
    "effects/barbarian/wind_wQ.png",
    "effects/barbarian/wind_wK.png",
    "effects/barbarian/wind_bP.png",
    "effects/barbarian/wind_bN.png",
    "effects/barbarian/wind_bB.png",
    "effects/barbarian/wind_bR.png",
    "effects/barbarian/wind_bQ.png",
    "effects/barbarian/wind_bK.png",
);
const BARBARIAN_DARK_PNG: [&[u8]; 12] = breach!(
    "effects/barbarian/dark_fire_wP.png",
    "effects/barbarian/dark_fire_wN.png",
    "effects/barbarian/dark_fire_wB.png",
    "effects/barbarian/dark_fire_wR.png",
    "effects/barbarian/dark_fire_wQ.png",
    "effects/barbarian/dark_fire_wK.png",
    "effects/barbarian/dark_fire_bP.png",
    "effects/barbarian/dark_fire_bN.png",
    "effects/barbarian/dark_fire_bB.png",
    "effects/barbarian/dark_fire_bR.png",
    "effects/barbarian/dark_fire_bQ.png",
    "effects/barbarian/dark_fire_bK.png",
);
const BARBARIAN_STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/barbarian/shadowstep_out_wP.png",
    "effects/barbarian/shadowstep_out_wN.png",
    "effects/barbarian/shadowstep_out_wB.png",
    "effects/barbarian/shadowstep_out_wR.png",
    "effects/barbarian/shadowstep_out_wQ.png",
    "effects/barbarian/shadowstep_out_wK.png",
    "effects/barbarian/shadowstep_out_bP.png",
    "effects/barbarian/shadowstep_out_bN.png",
    "effects/barbarian/shadowstep_out_bB.png",
    "effects/barbarian/shadowstep_out_bR.png",
    "effects/barbarian/shadowstep_out_bQ.png",
    "effects/barbarian/shadowstep_out_bK.png",
);
const BARBARIAN_STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/barbarian/shadowstep_in_wP.png",
    "effects/barbarian/shadowstep_in_wN.png",
    "effects/barbarian/shadowstep_in_wB.png",
    "effects/barbarian/shadowstep_in_wR.png",
    "effects/barbarian/shadowstep_in_wQ.png",
    "effects/barbarian/shadowstep_in_wK.png",
    "effects/barbarian/shadowstep_in_bP.png",
    "effects/barbarian/shadowstep_in_bN.png",
    "effects/barbarian/shadowstep_in_bB.png",
    "effects/barbarian/shadowstep_in_bR.png",
    "effects/barbarian/shadowstep_in_bQ.png",
    "effects/barbarian/shadowstep_in_bK.png",
);
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
const DARK_PNG: [&[u8]; 12] = breach!(
    "effects/dark_fire_wP.png",
    "effects/dark_fire_wN.png",
    "effects/dark_fire_wB.png",
    "effects/dark_fire_wR.png",
    "effects/dark_fire_wQ.png",
    "effects/dark_fire_wK.png",
    "effects/dark_fire_bP.png",
    "effects/dark_fire_bN.png",
    "effects/dark_fire_bB.png",
    "effects/dark_fire_bR.png",
    "effects/dark_fire_bQ.png",
    "effects/dark_fire_bK.png",
);
const STEP_OUT_PNG: [&[u8]; 12] = breach!(
    "effects/shadowstep_out_wP.png",
    "effects/shadowstep_out_wN.png",
    "effects/shadowstep_out_wB.png",
    "effects/shadowstep_out_wR.png",
    "effects/shadowstep_out_wQ.png",
    "effects/shadowstep_out_wK.png",
    "effects/shadowstep_out_bP.png",
    "effects/shadowstep_out_bN.png",
    "effects/shadowstep_out_bB.png",
    "effects/shadowstep_out_bR.png",
    "effects/shadowstep_out_bQ.png",
    "effects/shadowstep_out_bK.png",
);
const STEP_IN_PNG: [&[u8]; 12] = breach!(
    "effects/shadowstep_in_wP.png",
    "effects/shadowstep_in_wN.png",
    "effects/shadowstep_in_wB.png",
    "effects/shadowstep_in_wR.png",
    "effects/shadowstep_in_wQ.png",
    "effects/shadowstep_in_wK.png",
    "effects/shadowstep_in_bP.png",
    "effects/shadowstep_in_bN.png",
    "effects/shadowstep_in_bB.png",
    "effects/shadowstep_in_bR.png",
    "effects/shadowstep_in_bQ.png",
    "effects/shadowstep_in_bK.png",
);
const METEOR_PNG: [&[u8]; 1] = breach!("effects/meteor.png");
const CRATER_PNG: [&[u8]; 2] = breach!("effects/crater_under.png", "effects/crater_over.png");
const METEOR_WAVE_PNG: [&[u8]; 2] = breach!(
    "effects/meteor_wave_under.png",
    "effects/meteor_wave_over.png"
);
const POISON_PNG: [&[u8]; 12] = breach!(
    "effects/poison_wP.png",
    "effects/poison_wN.png",
    "effects/poison_wB.png",
    "effects/poison_wR.png",
    "effects/poison_wQ.png",
    "effects/poison_wK.png",
    "effects/poison_bP.png",
    "effects/poison_bN.png",
    "effects/poison_bB.png",
    "effects/poison_bR.png",
    "effects/poison_bQ.png",
    "effects/poison_bK.png",
);
/// След ядовитой фигуры по направлениям `Trail` (на экране): вверх-вправо, вниз-влево,
/// вверх-влево, вниз-вправо.
const POISON_TRAIL_PNG: [&[u8]; 4] = breach!(
    "effects/poison_trail_ne.png",
    "effects/poison_trail_sw.png",
    "effects/poison_trail_nw.png",
    "effects/poison_trail_se.png",
);
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
    "effects/electric_slow_wP.png",
    "effects/electric_slow_wN.png",
    "effects/electric_slow_wB.png",
    "effects/electric_slow_wR.png",
    "effects/electric_slow_wQ.png",
    "effects/electric_slow_wK.png",
    "effects/electric_slow_bP.png",
    "effects/electric_slow_bN.png",
    "effects/electric_slow_bB.png",
    "effects/electric_slow_bR.png",
    "effects/electric_slow_bQ.png",
    "effects/electric_slow_bK.png",
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
    chests: Vec<Pixels>,
    power_cards: Vec<Pixels>,
    crests: Vec<Pixels>,
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
    poison: Vec<Strip>,
    poison_trail: Vec<Strip>,
    big_wave: Vec<Strip>,
    crater: Vec<Strip>,
    meteor_wave: Vec<Strip>,
    meteor: Strip,
    /// Аура теневой фигуры, исчезновение и появление теневого шага.
    dark: [Vec<Strip>; 3],
}

/// Номер полосы анимации силы `of` у фигуры: у варвара — его набор (`BARBARIAN` следом за
/// обычными), у золотой — свой (`GOLD`), у мага — `WIZARD`, у ледяной — `ICE`; без них — набор
/// другой силы фигуры, если в нём есть эта анимация: огня (`FIRE`: всё, кроме лавы), молнии
/// (`SPARK`: вихрь и тень), ветра (`WIND`: тень).
fn strip_index(piece: Piece, of: Power) -> usize {
    let set = match base_set(piece) {
        0 if piece.fire && of != Power::Fire => FIRE,
        0 if piece.spark && !matches!(of, Power::Fire | Power::Spark) => SPARK,
        0 if piece.wind && of == Power::Shade => WIND,
        set => set,
    };
    super::index(piece.color, piece.kind) + set
}

/// Набор полос по основе фигуры: варвар, золотая, маг, ледяная; иначе — обычные (0).
fn base_set(piece: Piece) -> usize {
    if piece.barbarian {
        BARBARIAN
    } else if piece.gold {
        GOLD
    } else if piece.wizard {
        WIZARD
    } else if piece.icy {
        ICE
    } else {
        0
    }
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
            chests: decode(&CHESTS),
            power_cards: decode(&POWER_CARDS),
            crests: decode(&CRESTS),
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
                .chain(&BARBARIAN_LAVA_PNG)
                .chain(&GOLD_LAVA_PNG)
                .chain(&WIZARD_LAVA_PNG)
                .chain(&ICE_LAVA_PNG)
                .map(|png| Strip::new(png, super::SPRITE, LAVA_FRAMES))
                .collect(),
            dark: [
                (
                    &DARK_PNG,
                    [
                        &BARBARIAN_DARK_PNG,
                        &GOLD_DARK_PNG,
                        &WIZARD_DARK_PNG,
                        &ICE_DARK_PNG,
                        &FIRE_DARK_PNG,
                        &SPARK_DARK_PNG,
                        &WIND_DARK_PNG,
                    ],
                    DARK_FRAMES,
                ),
                (
                    &STEP_OUT_PNG,
                    [
                        &BARBARIAN_STEP_OUT_PNG,
                        &GOLD_STEP_OUT_PNG,
                        &WIZARD_STEP_OUT_PNG,
                        &ICE_STEP_OUT_PNG,
                        &FIRE_STEP_OUT_PNG,
                        &SPARK_STEP_OUT_PNG,
                        &WIND_STEP_OUT_PNG,
                    ],
                    STEP_OUT,
                ),
                (
                    &STEP_IN_PNG,
                    [
                        &BARBARIAN_STEP_IN_PNG,
                        &GOLD_STEP_IN_PNG,
                        &WIZARD_STEP_IN_PNG,
                        &ICE_STEP_IN_PNG,
                        &FIRE_STEP_IN_PNG,
                        &SPARK_STEP_IN_PNG,
                        &WIND_STEP_IN_PNG,
                    ],
                    STEP_IN,
                ),
            ]
            .map(|(pngs, combos, frames)| {
                pngs.iter()
                    .chain(combos.into_iter().flatten())
                    .map(|png| Strip::new(png, ELECTRIC_FIGURE, frames))
                    .collect()
            }),
            wind: WIND_PNG
                .iter()
                .chain(&BARBARIAN_WIND_PNG)
                .chain(&GOLD_WIND_PNG)
                .chain(&WIZARD_WIND_PNG)
                .chain(&ICE_WIND_PNG)
                .chain(&FIRE_WIND_PNG)
                .chain(&SPARK_WIND_PNG)
                .map(|png| Strip::new(png, ELECTRIC_FIGURE, WIND_FRAMES))
                .collect(),
            poison: POISON_PNG
                .iter()
                .map(|png| Strip::new(png, ELECTRIC_FIGURE, POISON_FRAMES))
                .collect(),
            poison_trail: POISON_TRAIL_PNG
                .iter()
                .map(|png| Strip::new(png, POISON_TRAIL, POISON_TRAIL_FRAMES))
                .collect(),
            meteor: Strip::new(METEOR_PNG[0], METEOR, METEOR_FRAMES),
            big_wave: BIG_WAVE
                .iter()
                .map(|png| Strip::new(png, BIG_WAVE_FRAME, BIG_WAVE_FRAMES))
                .collect(),
            crater: CRATER_PNG
                .iter()
                .map(|png| Strip::new(png, CRATER, CRATER_FRAMES))
                .collect(),
            meteor_wave: METEOR_WAVE_PNG
                .iter()
                .map(|png| Strip::new(png, WAVE_FRAME, METEOR_WAVE_FRAMES))
                .collect(),
            electric: ELECTRIC_PNG
                .iter()
                .chain(&BARBARIAN_ELECTRIC_PNG)
                .chain(&GOLD_ELECTRIC_PNG)
                .chain(&WIZARD_ELECTRIC_PNG)
                .chain(&ICE_ELECTRIC_PNG)
                .chain(&FIRE_ELECTRIC_PNG)
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
            chests: all("chest", &self.chests),
            power_cards: all("power-card", &self.power_cards),
            crests: all("crest", &self.crests),
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
            dark: [0, 1, 2].map(|n| {
                self.dark[n]
                    .iter()
                    .enumerate()
                    .map(|(i, strip)| strip.textures(ctx, &format!("dark-{n}-{i}"), k))
                    .collect()
            }),
            wind: self
                .wind
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("wind-{i}"), k))
                .collect(),
            poison: self
                .poison
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("poison-{i}"), k))
                .collect(),
            poison_trail: self
                .poison_trail
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("poison-trail-{i}"), k))
                .collect(),
            meteor: self.meteor.textures(ctx, "meteor", k),
            big_wave: self
                .big_wave
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("big-wave-{i}"), k))
                .collect(),
            crater: self
                .crater
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("crater-{i}"), k))
                .collect(),
            meteor_wave: self
                .meteor_wave
                .iter()
                .enumerate()
                .map(|(i, strip)| strip.textures(ctx, &format!("meteor-wave-{i}"), k))
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
    chests: Vec<TextureHandle>,
    power_cards: Vec<TextureHandle>,
    crests: Vec<TextureHandle>,
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
    poison: Vec<StripTextures>,
    poison_trail: Vec<StripTextures>,
    big_wave: Vec<StripTextures>,
    crater: Vec<StripTextures>,
    meteor_wave: Vec<StripTextures>,
    meteor: StripTextures,
    dark: [Vec<StripTextures>; 3],
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

    /// Сундук магазина.
    pub fn chest(&self, chest: Chest, big: bool) -> &TextureHandle {
        &self.chests[chest as usize + if big { Chest::ALL.len() } else { 0 }]
    }

    /// Карта rogue-like.
    pub fn rogue_card(&self, card: Card) -> &TextureHandle {
        let i = Card::ALL.iter().position(|&c| c == card).unwrap_or(0);
        &self.power_cards[i]
    }

    /// Герб (`CREST`).
    pub fn crest(&self, crest: Crest) -> &TextureHandle {
        &self.crests[crest as usize]
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

    /// Фигура `piece` с лавой в кадре `frame` — на холсте фигуры (варвар — на своём спрайте).
    pub fn lava(&self, piece: Piece, frame: usize) -> Option<&FrameSprite> {
        self.lava[strip_index(piece, Power::Fire)].frame(frame)
    }

    /// Фигура `piece` с электричеством в кадре `frame` — вокруг `ELECTRIC_ANCHOR`.
    pub fn electric(&self, piece: Piece, frame: usize) -> Option<&FrameSprite> {
        self.electric[strip_index(piece, Power::Spark)].frame(frame)
    }

    /// Теневая фигура: `phase` 0 — аура, 1 — исчезновение, 2 — появление; кадр `frame`
    /// вокруг `ELECTRIC_ANCHOR`.
    pub fn dark(&self, phase: usize, piece: Piece, frame: usize) -> Option<&FrameSprite> {
        self.dark[phase][strip_index(piece, Power::Shade)].frame(frame)
    }

    /// Воздушная фигура `piece` в вихре, кадр `frame` — вокруг `ELECTRIC_ANCHOR`.
    pub fn wind(&self, piece: Piece, frame: usize) -> Option<&FrameSprite> {
        self.wind[strip_index(piece, Power::Wind)].frame(frame)
    }

    /// Кадр ядовитой фигуры (лужа и капли) — вокруг точки опоры `ELECTRIC_ANCHOR`.
    pub fn poison(&self, color: Color, kind: Kind, frame: usize) -> Option<&FrameSprite> {
        self.poison[super::index(color, kind)].frame(frame)
    }

    /// Кадр следа ядовитой фигуры: `direction` — 0 вверх-вправо, 1 вниз-влево, 2 вверх-влево,
    /// 3 вниз-вправо (на экране).
    pub fn poison_trail(&self, direction: usize, frame: usize) -> Option<&FrameSprite> {
        self.poison_trail[direction].frame(frame)
    }

    /// Кадр горящего кратера: `over` — слой над фигурами.
    pub fn crater(&self, over: bool, frame: usize) -> Option<&FrameSprite> {
        self.crater[over as usize].frame(frame % CRATER_FRAMES)
    }

    /// Кадр волны метеорита крестом: `over` — слой над фигурами.
    pub fn meteor_wave(&self, over: bool, frame: usize) -> Option<&FrameSprite> {
        self.meteor_wave[over as usize].frame(frame)
    }

    /// Кадр большой огненной волны крестом: `over` — слой над фигурами.
    pub fn big_wave(&self, over: bool, frame: usize) -> Option<&FrameSprite> {
        self.big_wave[over as usize].frame(frame)
    }

    /// Кадр метеорита мага — вокруг центра клетки `METEOR_CENTER`.
    pub fn meteor(&self, frame: usize) -> Option<&FrameSprite> {
        self.meteor.frame(frame)
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
        assert_eq!(art.crests.len(), Crest::ALL.len());
        for image in &art.crests {
            assert_eq!(
                [image.width, image.height],
                [CREST.x as usize, CREST.y as usize]
            );
        }
        assert_eq!(art.power_cards.len(), Card::ALL.len());
        for image in &art.power_cards {
            assert_eq!(
                [image.width, image.height],
                [POWER_CARD.x as usize, POWER_CARD.y as usize]
            );
        }
        assert_eq!(art.chests.len(), 2 * Chest::ALL.len());
        for (i, image) in art.chests.iter().enumerate() {
            let size = if i < Chest::ALL.len() {
                CHEST
            } else {
                BIG_CHEST
            };
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
    fn every_special_piece_finds_its_animation_strips() {
        use crate::game::{COMBOS, Stock};
        let art = UiArt::new();
        let singles = [
            Power::Plain,
            Power::Fire,
            Power::Spark,
            Power::Wind,
            Power::Gold,
            Power::Shade,
            Power::Ice,
            Power::Barbarian,
            Power::Wizard,
        ]
        .map(|p| (p, Power::Plain));
        for (base, with) in COMBOS.into_iter().chain(singles) {
            for kind in Kind::ALL {
                for color in [Color::White, Color::Black] {
                    let piece = Stock::combo(kind, base, with).piece(color);
                    let index = super::super::index(color, kind);
                    for (of, len) in [
                        (Power::Fire, art.lava.len()),
                        (Power::Spark, art.electric.len()),
                        (Power::Wind, art.wind.len()),
                        (Power::Shade, art.dark[0].len()),
                    ] {
                        let i = strip_index(piece, of);
                        assert!(i < len, "{} — {of:?}", piece.title());
                        assert_eq!(i % 12, index, "{} — {of:?}", piece.title());
                    }
                }
            }
        }
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
