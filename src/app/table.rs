//! Стол, как в арт-проекте (`tools/breach_cards.py`, `table_preview`, «вариант 12»): своя
//! колода в левом нижнем углу, рука веером под правым краем доски; колода и рука соперника
//! — то же, повёрнутое на 180° вокруг центра доски; кнопка «Конец хода» — справа по
//! центру. Каждый ход сторона берёт карту из колоды и может разыграть до трёх карт.
//!
//! Карту берут мышью, как в Hearthstone: она примагничивается серединой к курсору и
//! тянется за ним, покачиваясь от скорости; отпущенная над рукой — возвращается, в другом
//! месте — разыгрывается и сгорает. Карта с выбором фигуры (огненная волна, обмен) после
//! этого встаёт над рукой, и от короля к курсору тянется стрелка, как в Slay the Spire
//! (`tools/breach_arrow.py`); щелчок по своей фигуре применяет карту, обмену нужны две.
//! Карту на поле (буря, лавина, ледяной барьер) направляют на любую клетку: подсвечены
//! клетки, на которые ляжет лёд (`tools/breach_freeze.py`).

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, Stroke,
    StrokeKind, Vec2, epaint::TextShape, vec2,
};

use super::{
    BattleChessApp, BoardGeom, Effect, Figure, Mark, PIXEL_WHITE, STATS_TIME, Sink, UV,
    ease_in_out, ease_out, ease_out_back,
};
use crate::assets::{
    ANCHOR, ARROW_HEAD, ARROW_SPACING, Aim, ArrowPart, BLESS_FRAME_TIME, BLESS_FRAMES,
    BURN_CARD_AT, BURN_FRAME, BURN_FRAME_TIME, BURN_FRAMES, BUTTON, Button, CARD, COST_TINT,
    CardPic, EMBLEM_ZOOM, FREEZE_CENTER, FREEZE_CLINK, FREEZE_DELAY, FREEZE_FRAME_TIME,
    FREEZE_FRAMES, FrameSprite, GLYPH_HEIGHT, Glyph, POINTS_TINT, PORTAL_CENTER, PORTAL_FRAME,
    PORTAL_FRAME_TIME, PORTAL_FRAMES, PORTAL_SCALE, PORTAL_SINK, SHADOW_SHIFT, SPRITE, Sprite,
    TARGET_PAD, TILE, Textures, WAVE_CENTER, WAVE_FRAME, WAVE_FRAME_TIME, WAVE_FRAMES, cost_corner,
};
use crate::game::{
    Area, CardKind, CardPlay, Color, Event, Family, Heal, Kind, MoveKind, Piece, Points, Square,
    WaveHit, file_of, rank_of,
};
use crate::sound::Sound;
use crate::theme;

/// Центр верхней грани доски, её левый и правый углы (правый — низ бока), пиксели арта.
const MID: Vec2 = vec2(224.0, 168.0);
const LEFT: Vec2 = vec2(0.0, 168.0);
const CORNER: Vec2 = vec2(448.0, 199.0);
/// Своя колода — левый верхний угол верхней карты, в левом нижнем углу стола; рука —
/// середина средней карты.
const DECK_AT: Vec2 = vec2(LEFT.x - 76.0, LEFT.y + 152.0);
const HAND_AT: Vec2 = vec2(CORNER.x - 24.0, CORNER.y + 135.0);
/// Кнопка «Конец хода» — левый верхний угол; справа по центру стола, как в арт-проекте.
const BUTTON_AT: Vec2 = vec2(472.0, 161.0);
/// Полная стопка (12 торцов): по ней колода соперника отражается через центр доски.
const DECK_FULL: Vec2 = vec2(54.0, 85.0);
/// Сколько торцов видно под верхней картой полной стопки.
const DECK_EDGES: u8 = 12;
/// Веер: карты на дуге радиуса `RADIUS`, через `STEP` градусов; при большой руке
/// шаг уменьшается, чтобы веер не разъезжался.
const RADIUS: f32 = 220.0;
const STEP: f32 = 8.5;
const SPREAD: f32 = 42.5;
/// На сколько приподнимается карта под курсором — та, что можно сыграть, и остальные.
const LIFT: f32 = 32.0;
const LIFT_IDLE: f32 = 20.0;
/// За столько секунд карта под курсором выезжает вверх.
const RISE_TIME: f64 = 0.2;
/// Карта под курсором крупнее втрое — как в Hearthstone, чтобы читался текст (карты
/// нарисованы вдвое детальнее, и крупная карта остаётся резкой).
const ZOOM: f32 = 3.0;
/// Карта из колоды долетает до руки.
const DRAW_TIME: f64 = 0.35;
/// Добор ждёт, пока доедет походившая фигура.
pub const DRAW_DELAY: f64 = 0.2;
/// Несколько карт подряд вылетают из колоды с таким промежутком.
const DRAW_GAP: f64 = 0.15;
/// Самый крупный кегль описания на лице карты, пиксели карты.
const DESCRIPTION_FONT: f32 = 5.0;
/// Свет исцеления доходит до соседних клеток на 12-м кадре (столбы света в breach_bless.py).
const HEAL_IMPACT: f64 = 12.0 * BLESS_FRAME_TIME;
/// Счётчики очков у колоды: отступ от колоды и между венком и черепом, пиксели арта.
const POINTS_GAP: f32 = 8.0;
const POINTS_SPACING: f32 = 6.0;
/// Число очков над знаком — вдвое крупнее цифры цены на карте, с отступом от знака.
const POINTS_FONT: f32 = 2.0;
const POINTS_RAISE: f32 = 4.0;
/// Взятая карта догоняет курсор: за 1/`FOLLOW` секунды проходит две трети пути —
/// середина карты за доли секунды «прилипает» к курсору и дальше идёт за ним с
/// лёгкой задержкой.
const FOLLOW: f32 = 28.0;
/// Покачивание взятой карты: наклон по часовой при движении вправо — градусов на
/// пиксель арта в секунду, предел и плавность.
const SWAY: f32 = 0.06;
const SWAY_MAX: f32 = 16.0;
const SWAY_EASE: f32 = 12.0;
/// Взятая карта поднята над столом — тень отходит дальше.
const HELD_SHADOW: Vec2 = vec2(6.0, 8.0);
/// Отпущенная над рукой карта возвращается на место.
const RETURN_TIME: f64 = 0.22;
/// Зона руки шире самих карт на столько пикселей арта.
const HAND_ZONE_MARGIN: f32 = 10.0;
/// Кромка огненной волны доходит до соседних клеток — тогда цифры урона.
const WAVE_IMPACT: f64 = 0.27;
/// Снег без замёрзших фигур (после бури и лавины, растаявшей зоны) и лёд оттаявших фигур
/// тают за столько секунд.
const MELT_TIME: f64 = 0.4;
/// Наконечник стрелки смотрит в фигуру чуть выше центра клетки.
const AIM_RISE: f32 = 14.0;
/// Стрелка начинается чуть выше макушки короля (или первой фигуры обмена), пиксели арта.
const KING_CROWN: f32 = 2.0;
/// За столько секунд гаснет текст сгорающей карты.
const BURN_TEXT_TIME: f64 = 0.5;
/// Табличка с числом карт: отступ от колоды и откуда она выезжает, пиксели шрифта.
const DECK_PLATE_GAP: f32 = 3.0;
const DECK_PLATE_SLIDE: f32 = 4.0;
/// Значок карты на табличке — цвета огней белых и чёрных фигур (`GLOW` в breach_art.py).
const DECK_ICON: [Color32; 2] = [
    Color32::from_rgb(0x57, 0xd9, 0xff),
    Color32::from_rgb(0xff, 0x8a, 0x2e),
];

/// Поле стола вокруг картинки доски: с ним вместе доска вписывается в окно. Слева —
/// веер руки соперника (до −91 с тенью), справа — своя рука, сверху — колода соперника
/// (−69), снизу — своя колода с тенью (408). 648×480: в развёрнутом окне 1920×1080 с обеими
/// панелями — ровно вдвое (1296×960).
pub const TABLE_MIN: Vec2 = vec2(-100.0, -70.0);
pub const TABLE_MAX: Vec2 = vec2(548.0, 410.0);

/// Место за столом: своё — внизу справа, соперника — вверху слева.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    Near,
    Far,
}

/// Карта в руке: середина и поворот по часовой стрелке, в градусах.
#[derive(Clone, Copy)]
struct Slot {
    center: Vec2,
    degrees: f32,
}

/// Карта, которую держат мышью.
pub struct Held {
    seat: Seat,
    /// Место карты в руке — туда она вернётся.
    index: usize,
    kind: CardKind,
    /// Середина карты и наклон; курсор — в пикселях арта.
    at: Vec2,
    tilt: f32,
    pointer: Vec2,
    last: f64,
}

/// Карта с выбором фигуры ждёт цель: стоит над рукой, к курсору тянется стрелка.
pub struct Aiming {
    seat: Seat,
    index: usize,
    kind: CardKind,
    /// Первая выбранная фигура обмена: вторую выбирают из тех, с кем её можно поменять.
    first: Option<Square>,
    /// Свои фигуры, которые можно выбрать сейчас: пока карта ждёт, позиция не меняется.
    targets: Vec<Square>,
}

/// Карта, летящая обратно в руку.
pub struct Returning {
    seat: Seat,
    index: usize,
    kind: CardKind,
    from: Vec2,
    tilt: f32,
    start: f64,
}

/// Сгорающая карта.
pub struct Burn {
    kind: CardKind,
    color: Color,
    /// Середина карты, пиксели арта.
    center: Vec2,
    start: f64,
}

/// Карта, летящая из колоды в руку.
pub struct Draw {
    color: Color,
    start: f64,
}

/// Огненная волна вокруг фигуры.
pub struct Wave {
    center: Square,
    hits: Vec<WaveHit>,
    start: f64,
    impacted: bool,
}

impl Wave {
    /// Огненная волна вокруг `center`, начинается в `start` (может быть впереди).
    pub fn new(center: Square, hits: Vec<WaveHit>, start: f64) -> Wave {
        Wave {
            center,
            hits,
            start,
            impacted: false,
        }
    }
}

/// Исцеление (`tools/breach_bless.py`): слон светится и раскрывает крылья, по соседним
/// клеткам бежит кольцо света — тогда над вылеченными фигурами всплывают зелёные цифры.
pub struct Bless {
    center: Square,
    color: Color,
    heals: Vec<Heal>,
    start: f64,
    impacted: bool,
}

/// Обмен (`tools/breach_portal.py`): под обеими фигурами раскрываются порталы, фигуры
/// проваливаются, потом поднимаются каждая на клетке другой.
pub struct Portal {
    squares: [Square; 2],
    start: f64,
    sounded: bool,
}

impl Portal {
    /// Что сейчас: `appear` — фигуры поднимаются (иначе проваливаются) — и кадр;
    /// `None` — обмен кончился.
    fn phase(&self, now: f64) -> Option<(bool, usize)> {
        let frame = ((now - self.start).max(0.0) / PORTAL_FRAME_TIME) as usize;
        match frame {
            f if f < PORTAL_FRAMES => Some((false, f)),
            f if f < 2 * PORTAL_FRAMES => Some((true, f - PORTAL_FRAMES)),
            _ => None,
        }
    }
}

/// Заморозка (`tools/breach_freeze.py`): от клетки прицела по клеткам карты расходится
/// лёд — наметает снег, на замёрзших фигурах поднимается иней и вырастает глыба. Фигура,
/// вставшая в ледяную зону, замерзает так же, только снег под ней уже лежит.
pub struct Freeze {
    /// Клетки, где наметает снег, и задержка каждой, кадров.
    tiles: Vec<(Square, usize)>,
    /// Клетки замёрзших фигур и задержка каждой.
    pieces: Vec<(Square, usize)>,
    start: f64,
    sounded: bool,
}

impl Freeze {
    /// Лёд расходится от `origin`: на клетке дальше на одну — на `FREEZE_DELAY` кадров позже.
    pub(super) fn new(origin: Square, tiles: &[Square], pieces: &[Square], start: f64) -> Freeze {
        let delay = |sq: Square| {
            let (df, dr) = (file_of(sq) - file_of(origin), rank_of(sq) - rank_of(origin));
            df.unsigned_abs().max(dr.unsigned_abs()) as usize * FREEZE_DELAY
        };
        Freeze {
            tiles: tiles.iter().map(|&sq| (sq, delay(sq))).collect(),
            pieces: pieces.iter().map(|&sq| (sq, delay(sq))).collect(),
            start,
            sounded: false,
        }
    }

    /// Кадр на клетке `sq` из списка `list`: `Some(None)` — лёд ещё не дошёл, `None` —
    /// клетки в списке нет.
    fn frame(&self, list: &[(Square, usize)], sq: Square, now: f64) -> Option<Option<usize>> {
        let &(_, delay) = list.iter().find(|&&(s, _)| s == sq)?;
        let f = ((now - self.start) / FREEZE_FRAME_TIME).floor() - delay as f64;
        Some((f >= 0.0).then_some(f as usize))
    }

    /// Когда на клетке с задержкой `delay` лёд встал целиком.
    fn settled(&self, delay: usize) -> f64 {
        self.start + (delay + FREEZE_FRAMES) as f64 * FREEZE_FRAME_TIME
    }

    /// Когда лёд встал на всех клетках.
    fn done(&self) -> f64 {
        let last = self.tiles.iter().chain(&self.pieces).map(|&(_, d)| d).max();
        self.settled(last.unwrap_or(0))
    }
}

impl Seat {
    fn sign(self) -> f32 {
        match self {
            Seat::Near => 1.0,
            Seat::Far => -1.0,
        }
    }

    fn hand_at(self) -> Vec2 {
        match self {
            Seat::Near => HAND_AT,
            Seat::Far => MID * 2.0 - HAND_AT,
        }
    }

    fn deck_at(self) -> Vec2 {
        match self {
            Seat::Near => DECK_AT,
            Seat::Far => MID * 2.0 - (DECK_AT + DECK_FULL),
        }
    }

    /// Карта `k` из `n` в веере; приподнятая карта стоит прямо.
    fn slot(self, k: usize, n: usize, lift: f32) -> Slot {
        let sign = self.sign();
        let step = if n > 1 {
            STEP.min(SPREAD / (n - 1) as f32)
        } else {
            STEP
        };
        let angle = (k as f32 - (n as f32 - 1.0) / 2.0) * step;
        let (sin, cos) = angle.to_radians().sin_cos();
        let base = self.hand_at();
        let pivot = base.y + sign * RADIUS;
        Slot {
            center: vec2(base.x + sin * RADIUS, pivot - sign * (cos * RADIUS + lift)),
            degrees: if lift > 0.0 { 0.0 } else { sign * angle },
        }
    }

    /// Зона руки из `n` карт: над ней отпущенная карта возвращается в руку. От веера
    /// с приподнятыми картами — до края стола за рукой.
    fn hand_zone(self, n: usize) -> Rect {
        let half = CARD / 2.0 + Vec2::splat(HAND_ZONE_MARGIN);
        let mut zone = Rect::NOTHING;
        for k in 0..n.max(1) {
            for lift in [0.0, LIFT] {
                let c = self.slot(k, n.max(1), lift).center;
                zone = zone.union(Rect::from_center_size(c.to_pos2(), half * 2.0));
            }
        }
        match self {
            Seat::Near => zone.with_max_y(f32::INFINITY),
            Seat::Far => zone.with_min_y(f32::NEG_INFINITY),
        }
    }
}

/// Кнопка «Конец хода» на столе, пиксели арта.
fn button_rect() -> Rect {
    Rect::from_min_size(BUTTON_AT.to_pos2(), BUTTON)
}

/// Запас взятых фигур («шведка», с 0.43): своя полоса — внизу слева, соперника —
/// отражение через центр доски; ячейка на вид фигуры, пиксели арта.
const RESERVE_AT: Vec2 = vec2(-90.0, 338.0);
const RESERVE_SLOT: Vec2 = vec2(46.0, 72.0);
const RESERVE_GAP: f32 = 3.0;
/// Фигура в ячейке — на четверть мельче, чем на доске; основание — на столько ниже верха
/// ячейки.
const RESERVE_SCALE: f32 = 0.75;
const RESERVE_GROUND: f32 = 50.0;

/// С 0.40 карты убраны из интерфейса: колоды, руки и очки не рисуются.
const SHOW_CARDS: bool = false;

impl BattleChessApp {
    /// Чья рука на месте `seat`: своя сторона — та, что внизу доски.
    pub(super) fn seat_color(&self, seat: Seat) -> Color {
        let near = if self.flipped {
            Color::Black
        } else {
            Color::White
        };
        match seat {
            Seat::Near => near,
            Seat::Far => near.opponent(),
        }
    }

    fn seat_of(&self, color: Color) -> Seat {
        if self.seat_color(Seat::Near) == color {
            Seat::Near
        } else {
            Seat::Far
        }
    }

    fn hand(&self, seat: Seat) -> &[CardKind] {
        self.game.position().cards(self.seat_color(seat)).hand()
    }

    /// Карты руки в веере (номера в руке): без летящих из колоды и без взятой мышью.
    fn hand_layout(&self, seat: Seat) -> Vec<usize> {
        let color = self.seat_color(seat);
        let flying = self.draws.iter().filter(|d| d.color == color).count();
        let held = self
            .held
            .as_ref()
            .filter(|h| h.seat == seat)
            .map(|h| h.index);
        (0..self.hand(seat).len().saturating_sub(flying))
            .filter(|&i| Some(i) != held)
            .collect()
    }

    /// Место карты `index` в веере.
    fn slot_of(&self, seat: Seat, index: usize, lift: f32) -> Slot {
        let layout = self.hand_layout(seat);
        let k = layout
            .iter()
            .position(|&i| i == index)
            .unwrap_or(layout.len());
        seat.slot(k, layout.len().max(k + 1), lift)
    }

    /// Середина крупной карты под курсором: нижним краем она на месте карты, поднятой на
    /// `lift` (у соперника — верхним).
    fn zoomed_center(&self, seat: Seat, index: usize, lift: f32) -> Vec2 {
        let slot = self.slot_of(seat, index, lift);
        slot.center - vec2(0.0, seat.sign() * CARD.y * (ZOOM - 1.0) / 2.0)
    }

    /// Насколько сейчас поднята карта под курсором: она выезжает вверх за `RISE_TIME`.
    fn hover_lift(&self, seat: Seat, now: f64) -> f32 {
        let u = ((now - self.card_hover_since) / RISE_TIME).clamp(0.0, 1.0) as f32;
        self.lift(seat) * ease_out(u)
    }

    /// Карта под курсором ещё выезжает — кадры нужны непрерывно.
    pub(super) fn card_rising(&self, now: f64) -> bool {
        self.card_hover.is_some() && now - self.card_hover_since < RISE_TIME
    }

    /// Картинка карты: свою руку видно всегда, руку соперника — только если за ним
    /// тоже человек.
    fn pic(&self, seat: Seat, kind: CardKind, selected: bool) -> CardPic {
        let color = self.seat_color(seat);
        if seat == Seat::Far && self.computer.is_some() {
            CardPic::Back(color)
        } else {
            CardPic::Front {
                kind,
                color,
                selected,
            }
        }
    }

    /// Может ли человек сейчас сыграть карту из руки на месте `seat`.
    fn can_play_from(&self, seat: Seat) -> bool {
        let pos = self.game.position();
        self.human_to_move()
            && self.promotion.is_none()
            && self.aiming.is_none()
            && pos.side() == self.seat_color(seat)
            && pos.can_play_card()
    }

    fn lift(&self, seat: Seat) -> f32 {
        if self.can_play_from(seat) {
            LIFT
        } else {
            LIFT_IDLE
        }
    }

    /// Карта под курсором — только в самом веере, по узким полосам (`fan_slot_at`).
    /// Крупная карта над веером курсор не держит: увёл курсор с руки — она опускается.
    pub(super) fn card_under(&self, board: &BoardGeom, pointer: Pos2) -> Option<(Seat, usize)> {
        let at = board.to_art(pointer);
        [Seat::Near, Seat::Far]
            .into_iter()
            // Рука соперника-компьютера не отзывается на курсор.
            .filter(|&seat| seat == Seat::Near || self.computer.is_none())
            .find_map(|seat| {
                let layout = self.hand_layout(seat);
                let away = |k: usize| {
                    self.returning_card(seat, layout[k]) || self.aimed_card(seat, layout[k])
                };
                fan_slot_at(seat, layout.len(), at, away).map(|k| (seat, layout[k]))
            })
    }

    /// Сколько карт в колоде на месте `seat` — вместе с теми, что ещё летят в руку: столько
    /// и нарисовано в стопке.
    fn deck_count(&self, seat: Seat) -> u8 {
        let color = self.seat_color(seat);
        let flying = self.draws.iter().filter(|d| d.color == color).count() as u8;
        self.game.position().cards(color).deck_len() + flying
    }

    /// Стопка колоды на месте `seat`: верхняя карта и торцы под ней, как в `deck` из
    /// assets/cards.rs; пустой колоды нет.
    fn deck_rect(&self, seat: Seat) -> Option<Rect> {
        let cards = self.deck_count(seat);
        let edges = cards.min(DECK_EDGES) as f32;
        let size = vec2(DECK_FULL.x, CARD.y + edges + 3.0);
        (cards > 0).then(|| Rect::from_min_size(seat.deck_at().to_pos2(), size))
    }

    /// Колода под курсором.
    pub(super) fn deck_under(&self, board: &BoardGeom, pointer: Pos2) -> Option<Seat> {
        let at = board.to_art(pointer).to_pos2();
        [Seat::Near, Seat::Far]
            .into_iter()
            .find(|&seat| self.deck_rect(seat).is_some_and(|rect| rect.contains(at)))
    }

    /// Курсор над колодой или рукой соперника-компьютера: эти карты не берут.
    pub(super) fn over_closed_cards(&self, board: &BoardGeom, pointer: Pos2) -> bool {
        let at = board.to_art(pointer).to_pos2();
        if self.deck_under(board, pointer).is_some() {
            return true;
        }
        if self.computer.is_none() {
            return false;
        }
        let layout = self.hand_layout(Seat::Far);
        (0..layout.len()).any(|k| covers(Seat::Far.slot(k, layout.len(), 0.0), at.to_vec2()))
    }

    fn returning_card(&self, seat: Seat, index: usize) -> bool {
        self.returning
            .iter()
            .any(|r| r.seat == seat && r.index == index)
    }

    fn aimed_card(&self, seat: Seat, index: usize) -> bool {
        self.aiming
            .as_ref()
            .is_some_and(|a| a.seat == seat && a.index == index)
    }

    /// Курсор над картой, которую можно взять.
    pub(super) fn card_playable(&self) -> bool {
        self.card_hover
            .is_some_and(|(seat, index)| self.usable(seat, index))
    }

    pub(super) fn holding_card(&self) -> bool {
        self.held.is_some()
    }

    pub(super) fn aiming_card(&self) -> bool {
        self.aiming.is_some()
    }

    /// Фигуры, на которые можно направить карту, ждущую цель.
    pub(super) fn aim_targets(&self) -> &[Square] {
        self.aiming.as_ref().map_or(&[], |a| &a.targets)
    }

    /// Первая выбранная фигура обмена.
    pub(super) fn aim_first(&self) -> Option<Square> {
        self.aiming.as_ref().and_then(|a| a.first)
    }

    /// Можно ли щёлкнуть по фигуре на `sq`, пока карта ждёт цель: выбрать её или снять
    /// выбор первой фигуры обмена.
    pub(super) fn aim_pickable(&self, sq: Square) -> bool {
        self.aim_targets().contains(&sq) || self.aim_first() == Some(sq)
    }

    /// Карта на поле, ждущая клетку: как она ляжет вокруг клетки под курсором.
    pub(super) fn aim_area(&self) -> Option<Area> {
        self.aiming.as_ref().and_then(|a| a.kind.area())
    }

    /// Подсказка в строке хода, пока карта ждёт цель.
    pub(super) fn aim_hint(&self) -> &'static str {
        match self.aiming.as_ref().map(|a| (a.kind, a.first.is_some())) {
            Some((CardKind::Swap, false)) => "Обмен: выберите первую свою фигуру (Esc — отмена)",
            Some((CardKind::Heal, _)) => "Исцеление: выберите своего слона (Esc — отмена)",
            Some((CardKind::Swap, true)) => {
                "Обмен: выберите вторую фигуру — они поменяются местами (Esc — отмена)"
            }
            Some((CardKind::Storm, _)) => {
                "Буря: выберите клетку — замёрзнут три горизонтали вокруг неё (Esc — отмена)"
            }
            Some((CardKind::Avalanche, _)) => {
                "Лавина: выберите клетку — замёрзнут три вертикали вокруг неё (Esc — отмена)"
            }
            Some((CardKind::IceBarrier, _)) => {
                "Ледяной барьер: выберите клетку — лёд ляжет квадратом 3×3 вокруг неё (Esc — отмена)"
            }
            Some((CardKind::IceWall, _)) => {
                "Ледяная стена: выберите горизонталь — она обледенеет на два хода (Esc — отмена)"
            }
            Some((CardKind::MassSacrifice, _)) => {
                "Массовое жертвоприношение: выберите квадрат 3×3 со своими фигурами (Esc — отмена)"
            }
            Some((CardKind::Offering, _)) => {
                "Жертва: выберите свою фигуру — она уйдёт с доски (Esc — отмена)"
            }
            Some((CardKind::Multiply, _)) => {
                "Они множатся: выберите свою фигуру, кроме короля и ферзя (Esc — отмена)"
            }
            Some((CardKind::Champion, _)) => {
                "Один в Поле Воин: выберите свою фигуру, кроме короля (Esc — отмена)"
            }
            Some((CardKind::Bastion, _)) => {
                "Оборонительная позиция: выберите свою ладью (Esc — отмена)"
            }
            Some((CardKind::IceShard, _)) => {
                "Ледяной снаряд: выберите фигуру соперника — 500 урона и заморозка (Esc — отмена)"
            }
            Some((CardKind::IceBlast, _)) => {
                "Ледяной взрыв: выберите замёрзшую фигуру соперника (Esc — отмена)"
            }
            Some((CardKind::Barrier, _)) => {
                "Барьер: выберите клетку — соперник не войдёт в квадрат 3×3 вокруг неё (Esc — отмена)"
            }
            Some((CardKind::IceWarrior, _)) => {
                "Воин из Льда: выберите свою фигуру — её удары будут замораживать (Esc — отмена)"
            }
            Some((CardKind::NorthWarrior, _)) => {
                "Северный воин: выберите свою фигуру (Esc — отмена)"
            }
            _ => "Выберите свою фигуру — волна ударит вокруг неё (Esc — отмена)",
        }
    }

    /// Нажатие на карту: карта, которую можно сыграть, отрывается от руки и тянется к курсору.
    pub(super) fn grab_card(&mut self, seat: Seat, index: usize, pointer: Vec2, now: f64) {
        if !self.usable(seat, index) || self.returning_card(seat, index) {
            return;
        }
        let Some(&kind) = self.hand(seat).get(index) else {
            return;
        };
        let start = self.slot_of(seat, index, LIFT).center;
        self.held = Some(Held {
            seat,
            index,
            kind,
            at: start,
            tilt: 0.0,
            pointer,
            last: now,
        });
        self.card_hover = None;
    }

    /// Взятая карта догоняет курсор и покачивается от скорости.
    pub(super) fn carry_card(&mut self, pointer: Option<Vec2>, now: f64) {
        let Some(held) = &mut self.held else {
            return;
        };
        if let Some(pointer) = pointer {
            held.pointer = pointer;
        }
        let dt = (now - held.last).clamp(0.0, 0.1) as f32;
        held.last = now;
        if dt <= 0.0 {
            return;
        }
        let before = held.at;
        held.at += (held.pointer - held.at) * (1.0 - (-dt * FOLLOW).exp());
        let speed = (held.at.x - before.x) / dt;
        let target = (speed * SWAY).clamp(-SWAY_MAX, SWAY_MAX);
        held.tilt += (target - held.tilt) * (1.0 - (-dt * SWAY_EASE).exp());
    }

    /// Кнопку отпустили: над рукой карта возвращается, в другом месте — разыгрывается.
    /// Карта с выбором фигуры или клетки встаёт над рукой и ждёт цель — где бы её ни
    /// отпустили, даже над целью: от короля к курсору тянется стрелка, цель выбирают
    /// щелчком (у карты на поле подсвечены клетки, на которые она ляжет).
    pub(super) fn drop_card(&mut self, pointer: Option<Vec2>, now: f64) {
        let Some(held) = self.held.take() else {
            return;
        };
        let at = pointer.unwrap_or(held.pointer);
        let n = self.hand_layout(held.seat).len() + 1;
        let over_hand = held.seat.hand_zone(n).contains(at.to_pos2());
        if over_hand || !self.can_play_from(held.seat) {
            self.return_card(held, now);
            return;
        }
        if held.kind.targeted() {
            self.aiming = Some(Aiming {
                seat: held.seat,
                index: held.index,
                kind: held.kind,
                first: None,
                targets: self.game.position().card_targets(held.index),
            });
            return;
        }
        self.play_card(held.seat, held.index, &[], held.at, now);
    }

    /// Щелчок с картой, ждущей цель, по своей фигуре: волна бьёт вокруг неё; обмену
    /// первая фигура запоминается, вторая — меняется с ней местами. Повторный щелчок по
    /// первой фигуре снимает выбор.
    pub(super) fn aim_at(&mut self, target: Square, now: f64) {
        let Some(mut aim) = self.aiming.take() else {
            return;
        };
        let pos = self.game.position();
        if aim.first == Some(target) {
            aim.first = None;
            aim.targets = pos.card_targets(aim.index);
        } else if !aim.targets.contains(&target) {
            // Сюда карту не сыграть — ждём дальше.
        } else if aim.kind.picks() == 2 && aim.first.is_none() {
            aim.first = Some(target);
            aim.targets = pos.swap_partners(target);
        } else {
            let targets: Vec<Square> = aim.first.into_iter().chain([target]).collect();
            let center = self.slot_of(aim.seat, aim.index, LIFT).center;
            self.play_card(aim.seat, aim.index, &targets, center, now);
            return;
        }
        self.aiming = Some(aim);
    }

    /// Карта, ждущая цель, возвращается в руку (Esc, правая кнопка, щелчок мимо цели).
    pub(super) fn cancel_aim(&mut self) {
        self.aiming = None;
    }

    /// Правая кнопка или Esc: взятая карта возвращается в руку.
    pub(super) fn cancel_card(&mut self, now: f64) {
        if let Some(held) = self.held.take() {
            self.return_card(held, now);
        }
    }

    /// Карту можно взять: карту с выбором — только если есть свои фигуры для неё.
    fn usable(&self, seat: Seat, index: usize) -> bool {
        self.can_play_from(seat) && self.game.position().card_playable(index)
    }

    /// Карта `index` сгорает там, где она была (`center`); огненная волна расходится
    /// от выбранной фигуры, обмен открывает порталы под двумя фигурами.
    fn play_card(&mut self, seat: Seat, index: usize, targets: &[Square], center: Vec2, now: f64) {
        let color = self.seat_color(seat);
        let next = color.opponent();
        let hand = self.game.position().cards(next).hand().len();
        let Some(play) = self.game.play_card(index, targets) else {
            return;
        };
        self.after_card(play, center, now);
        self.card_hover = None;
        self.aiming = None;
        // Карта после хода фигурой заканчивает ход — соперник берёт карту.
        self.note_draw(next, hand, now + DRAW_DELAY);
    }

    fn after_card(&mut self, play: CardPlay, center: Vec2, now: f64) {
        self.burns.push(Burn {
            kind: play.kind,
            color: play.color,
            center,
            start: now,
        });
        // Взятые картой карты (дополнительное финансирование) летят из колоды одна за другой.
        for k in 0..play.drawn {
            self.draws.push(Draw {
                color: play.color,
                start: now + DRAW_DELAY + f64::from(k) * DRAW_GAP,
            });
        }
        match (play.kind, play.targets.as_slice()) {
            (CardKind::FireWave, &[center]) => self.waves.push(Wave {
                center,
                hits: play.hits,
                start: now,
                impacted: false,
            }),
            (CardKind::Heal, &[center]) => self.blesses.push(Bless {
                center,
                color: play.color,
                heals: play.heals,
                start: now,
                impacted: false,
            }),
            (CardKind::Swap, &[a, b]) => {
                self.portals.push(Portal {
                    squares: [a, b],
                    start: now,
                    sounded: false,
                });
                // Фигура, вставшая в ледяную зону, замерзает, поднявшись из портала.
                if !play.frozen.is_empty() {
                    let up = now + 2.0 * PORTAL_FRAMES as f64 * PORTAL_FRAME_TIME;
                    for &sq in &play.frozen {
                        self.freezes.push(Freeze::new(sq, &[], &[sq], up));
                    }
                }
            }
            (CardKind::Offering, &[sq]) => {
                // Принесённая в жертву фигура растворяется, как погибшая.
                let before = self
                    .game
                    .history()
                    .iter()
                    .rev()
                    .find_map(|r| match &r.event {
                        Event::Card(p) if p.kind == CardKind::Offering => r.before.piece(sq),
                        _ => None,
                    });
                if let Some(piece) = before {
                    self.effects.push(Effect {
                        square: sq,
                        victim: Some(piece),
                        amount: piece.hp,
                        killed: true,
                        mark: Mark::Hit,
                        start: now,
                        stack: 0,
                    });
                }
            }
            (CardKind::AbsoluteZero, _) => {
                // Лёд нарастает на замёрзших навсегда фигурах, расходясь от первой.
                if let Some(&origin) = play.frozen.first() {
                    self.freezes
                        .push(Freeze::new(origin, &play.frozen, &play.frozen, now));
                }
            }
            (
                CardKind::MassSacrifice
                | CardKind::IceShard
                | CardKind::IceBlast
                | CardKind::HardWinter,
                _,
            ) => {
                // Урон картой — цифры и вспышка на задетых, погибшие растворяются.
                for hit in &play.hits {
                    self.effects.push(Effect {
                        square: hit.square,
                        victim: Some(hit.target),
                        amount: hit.damage,
                        killed: hit.killed,
                        mark: Mark::Hit,
                        start: now,
                        stack: 0,
                    });
                }
                // Выжившую цель ледяного снаряда сковывает лёд.
                for &sq in &play.frozen {
                    self.freezes.push(Freeze::new(sq, &[sq], &[sq], now));
                }
            }
            (kind, &[aim]) if kind.freezes() => {
                // Буря, лавина, ледяной барьер, ледяная стена: лёд расходится от прицела.
                if let Some(area) = kind.area() {
                    let tiles = area.squares(aim);
                    self.freezes
                        .push(Freeze::new(aim, &tiles, &play.frozen, now));
                }
            }
            _ => {}
        }
    }

    fn return_card(&mut self, held: Held, now: f64) {
        self.returning.push(Returning {
            seat: held.seat,
            index: held.index,
            kind: held.kind,
            from: held.at,
            tilt: held.tilt,
            start: now,
        });
    }

    /// После хода сторона, которой ходить, берёт карту — она летит из колоды в руку.
    pub(super) fn note_draw(&mut self, color: Color, hand_before: usize, start: f64) {
        if self.game.position().cards(color).hand().len() > hand_before {
            self.draws.push(Draw { color, start });
        }
    }

    /// Огненная волна доходит до соседей — цифры урона и звук; закончившиеся
    /// анимации карт убираются.
    pub(super) fn tick_cards(&mut self, now: f64) {
        let burn_time = BURN_FRAMES as f64 * BURN_FRAME_TIME;
        self.burns.retain(|b| now - b.start < burn_time);
        self.draws.retain(|d| now - d.start < DRAW_TIME);
        self.returning.retain(|r| now - r.start < RETURN_TIME);
        let mut effects = Vec::new();
        let mut sounds = Vec::new();
        for wave in &mut self.waves {
            if !wave.impacted && now - wave.start >= WAVE_IMPACT {
                wave.impacted = true;
                for hit in &wave.hits {
                    effects.push(Effect {
                        square: hit.square,
                        victim: Some(hit.target),
                        amount: hit.damage,
                        killed: hit.killed,
                        mark: Mark::Hit,
                        start: now,
                        stack: 0,
                    });
                }
                if !wave.hits.is_empty() {
                    let killed = wave.hits.iter().any(|h| h.killed);
                    sounds.push(if killed { Sound::Kill } else { Sound::Hit });
                }
            }
        }
        let wave_time = WAVE_FRAMES as f64 * WAVE_FRAME_TIME;
        self.waves.retain(|w| now - w.start < wave_time);
        // Фигуры обмена показываются из порталов со вспышкой — тогда и звук.
        for portal in &mut self.portals {
            if !portal.sounded && portal.phase(now).is_none_or(|(appear, f)| appear && f >= 4) {
                portal.sounded = true;
                sounds.push(Sound::Move);
            }
        }
        self.portals.retain(|p| p.phase(now).is_some());
        // Свет исцеления доходит до соседних клеток — цифры лечения.
        for bless in &mut self.blesses {
            if !bless.impacted && now - bless.start >= HEAL_IMPACT {
                bless.impacted = true;
                for heal in &bless.heals {
                    effects.push(Effect::note(heal.square, Mark::Heal, heal.amount, now));
                }
                if !bless.heals.is_empty() {
                    sounds.push(Sound::Move);
                }
            }
        }
        let bless_time = BLESS_FRAMES as f64 * BLESS_FRAME_TIME;
        self.blesses.retain(|b| now - b.start < bless_time);
        // Глыба льда встала и звякнула.
        for freeze in &mut self.freezes {
            let clinked = freeze.pieces.iter().any(|&(_, delay)| {
                now >= freeze.start + (delay + FREEZE_CLINK) as f64 * FREEZE_FRAME_TIME
            });
            if !freeze.sounded && clinked {
                freeze.sounded = true;
                sounds.push(Sound::Move);
            }
        }
        self.freezes.retain(|f| now < f.done() + MELT_TIME);
        self.tick_ice(now);
        self.effects.extend(effects);
        for sound in sounds {
            self.sfx.play(sound);
        }
    }

    /// Следит за льдом: оттаявшие фигуры, растаявшие зоны и погибшие замёрзшие фигуры
    /// оставляют таять лёд и снег на своих клетках.
    fn tick_ice(&mut self, now: f64) {
        let (frozen, snowy) = self.ice_masks();
        let (was_frozen, was_snowy) = self.ice_seen;
        let melted = (was_snowy & !snowy) | (was_frozen & !frozen);
        for sq in 0..64u8 {
            if melted & 1 << sq != 0 {
                self.thaws.retain(|&(s, _)| s != sq);
                self.thaws.push((sq, now));
            }
        }
        self.ice_seen = (frozen, snowy);
        self.thaws.retain(|&(_, start)| now - start < MELT_TIME);
    }

    /// Клетки замёрзших фигур и клетки под снегом (замёрзшие фигуры и ледяные зоны).
    pub(super) fn ice_masks(&self) -> (u64, u64) {
        let pos = self.game.position();
        let frozen = pos
            .pieces()
            .filter(|(_, p)| p.is_frozen())
            .fold(0u64, |mask, (sq, _)| mask | 1 << sq);
        (frozen, frozen | pos.ice_zones())
    }

    /// Позиция сменилась без ходов (отмена, новая партия): лёд — как в ней, без таяния.
    pub(super) fn reset_ice(&mut self) {
        self.freezes.clear();
        self.thaws.clear();
        self.ice_seen = self.ice_masks();
    }

    /// Ход фигурой: фигура, вставшая в ледяную зону (и ладья рокировки), замерзает, когда
    /// доедет — в `arrive` секунд.
    pub(super) fn freeze_arrivals(&mut self, squares: &[Square], arrive: f64) {
        for &sq in squares {
            self.freezes.push(Freeze::new(sq, &[], &[sq], arrive));
        }
    }

    /// Снег на клетке `sq`: кадр заморозки и непрозрачность; `None` — снега нет.
    pub(super) fn snow_at(&self, sq: Square, now: f64) -> Option<(usize, f32)> {
        let last = FREEZE_FRAMES - 1;
        let pos = self.game.position();
        let steady = pos.piece(sq).is_some_and(|p| p.is_frozen()) || pos.ice_zones() & 1 << sq != 0;
        for freeze in &self.freezes {
            let Some(frame) = freeze.frame(&freeze.tiles, sq, now) else {
                continue;
            };
            // Лёд ещё не дошёл — клетка пока прежняя.
            let frame = frame?;
            if frame < last || steady {
                return Some((frame.min(last), 1.0));
            }
            // Лёд встал, но держать его некому (буря, лавина) — снег тает.
            let delay = freeze
                .tiles
                .iter()
                .find(|&&(s, _)| s == sq)
                .map_or(0, |t| t.1);
            let melt = ((now - freeze.settled(delay)) / MELT_TIME) as f32;
            return (melt < 1.0).then_some((last, 1.0 - melt.max(0.0)));
        }
        if steady {
            return Some((last, 1.0));
        }
        self.melting(sq, now).map(|alpha| (last, alpha))
    }

    /// Лёд на фигуре `piece` на клетке `sq`: кадр заморозки и непрозрачность.
    pub(super) fn ice_on(&self, sq: Square, piece: Piece, now: f64) -> Option<(usize, f32)> {
        let last = FREEZE_FRAMES - 1;
        for freeze in &self.freezes {
            if let Some(frame) = freeze.frame(&freeze.pieces, sq, now) {
                return frame.map(|f| (f.min(last), 1.0));
            }
        }
        if piece.is_frozen() {
            return Some((last, 1.0));
        }
        self.melting(sq, now).map(|alpha| (last, alpha))
    }

    /// Тает ли лёд на клетке `sq`: сколько его осталось.
    fn melting(&self, sq: Square, now: f64) -> Option<f32> {
        let &(_, start) = self.thaws.iter().find(|&&(s, _)| s == sq)?;
        let u = ((now - start) / MELT_TIME) as f32;
        (u < 1.0).then_some(1.0 - u.max(0.0))
    }

    /// Снег на клетках — до подсветок и фигур, от дальних клеток к ближним (передний
    /// бортик снега ложится на дальние клетки).
    pub(super) fn paint_snow(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        now: f64,
    ) {
        let mut squares: Vec<(Square, usize, f32)> = (0..64u8)
            .filter_map(|sq| self.snow_at(sq, now).map(|(f, a)| (sq, f, a)))
            .collect();
        squares.sort_by(|a, b| {
            board
                .depth(super::center(a.0))
                .total_cmp(&board.depth(super::center(b.0)))
        });
        for (sq, frame, alpha) in squares {
            if let Some(tile) = tex.ui.freeze_tile(ice_variant(sq), frame) {
                put_frame(painter, board, tile, sq, alpha);
            }
        }
    }

    /// Снежинки, пар и искры заморозки — над фигурами.
    pub(super) fn paint_freeze_over(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        now: f64,
    ) {
        for freeze in &self.freezes {
            let mut squares: Vec<Square> = freeze.tiles.iter().map(|t| t.0).collect();
            for &(sq, _) in &freeze.pieces {
                if !squares.contains(&sq) {
                    squares.push(sq);
                }
            }
            for sq in squares {
                let list = if freeze.tiles.iter().any(|t| t.0 == sq) {
                    &freeze.tiles
                } else {
                    &freeze.pieces
                };
                if let Some(Some(frame)) = freeze.frame(list, sq, now)
                    && frame < FREEZE_FRAMES
                    && let Some(over) = tex.ui.freeze_over(frame)
                {
                    put_frame(painter, board, over, sq, 1.0);
                }
            }
        }
    }

    pub(super) fn cards_moving(&self) -> bool {
        !self.burns.is_empty()
            || !self.draws.is_empty()
            || !self.returning.is_empty()
            || !self.waves.is_empty()
            || !self.portals.is_empty()
            || !self.blesses.is_empty()
            || !self.freezes.is_empty()
            || !self.thaws.is_empty()
            || self.held.is_some()
    }

    /// Карта ещё действует на доске (огненная волна, обмен, исцеление, заморозка) —
    /// компьютер ждёт.
    pub(super) fn card_effect_running(&self, now: f64) -> bool {
        !self.waves.is_empty()
            || !self.portals.is_empty()
            || !self.blesses.is_empty()
            || self.freezes.iter().any(|f| now < f.done())
    }

    /// Кадр слона с крыльями, если на `sq` сейчас исцеляет слон стороны `color`.
    pub(super) fn bless_frame(&self, sq: Square, color: Color, now: f64) -> Option<usize> {
        self.blesses
            .iter()
            .find(|b| b.center == sq && b.color == color)
            .map(|b| ((now - b.start).max(0.0) / BLESS_FRAME_TIME) as usize)
    }

    /// Кадр исцеления: `over` — слой над фигурами (столбы света, искры), иначе — свет на
    /// земле под слоном и кольцо по соседним клеткам.
    pub(super) fn paint_blesses(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        over: bool,
        now: f64,
    ) {
        for bless in &self.blesses {
            let frame = ((now - bless.start).max(0.0) / BLESS_FRAME_TIME) as usize;
            let center = board.to_art(board.ground(bless.center));
            let min = board.art_to_screen(center - WAVE_CENTER);
            let rect = Rect::from_min_size(min, WAVE_FRAME * board.scale);
            painter.image(tex.ui.bless(over, frame).id(), rect, UV, Color32::WHITE);
        }
    }

    /// Ход фигурой во время обмена: обмен доигрывается сразу.
    pub(super) fn finish_portals(&mut self) {
        self.portals.clear();
    }

    /// Клетки, фигуры на которых рисует обмен.
    pub(super) fn portal_hides(&self, sq: Square) -> bool {
        self.portals.iter().any(|p| p.squares.contains(&sq))
    }

    /// Фигуры обмена: пока они проваливаются, каждая стоит на своей прежней клетке, потом
    /// поднимается на клетке другой; ушедшая в портал фигура срезана его кромкой.
    pub(super) fn portal_figures(&self, now: f64) -> Vec<Figure> {
        let pos = self.game.position();
        let mut figures = Vec::new();
        for portal in &self.portals {
            let Some((appear, frame)) = portal.phase(now) else {
                continue;
            };
            let sink = Sink {
                depth: PORTAL_SINK[appear as usize][frame],
                scale: PORTAL_SCALE[frame],
            };
            for (k, &sq) in portal.squares.iter().enumerate() {
                // Позиция уже после обмена: прежняя фигура клетки стоит на другой.
                let from = if appear { sq } else { portal.squares[1 - k] };
                if let Some(piece) = pos.piece(from) {
                    figures.push(Figure {
                        sink: Some(sink),
                        shadow: sink.depth == 0.0,
                        ..Figure::new(super::center(sq), piece)
                    });
                }
            }
        }
        figures
    }

    /// Кадр порталов обмена: `over` — слой над фигурами, иначе — под ними.
    pub(super) fn paint_portals(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        over: bool,
        now: f64,
    ) {
        for portal in &self.portals {
            let Some((appear, frame)) = portal.phase(now) else {
                continue;
            };
            let texture = tex.ui.portal(appear, over, frame).id();
            for sq in portal.squares {
                let ground = board.to_art(board.ground(sq));
                let min = board.art_to_screen(ground - PORTAL_CENTER);
                let rect = Rect::from_min_size(min, PORTAL_FRAME * board.scale);
                painter.image(texture, rect, UV, Color32::WHITE);
            }
        }
    }

    /// Пока огненная волна не дошла, задетые фигуры стоят как до неё: живые — с прежним
    /// здоровьем, погибшие — ещё на месте.
    pub(super) fn wave_victims(&self) -> Vec<WaveHit> {
        self.waves
            .iter()
            .filter(|w| !w.impacted)
            .flat_map(|w| w.hits.iter().copied())
            .collect()
    }

    /// Кадр огненной волны: `over` — слой над фигурами, иначе — под ними.
    pub(super) fn paint_waves(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        over: bool,
        now: f64,
    ) {
        for wave in self.waves.iter().filter(|w| now >= w.start) {
            let frame = ((now - wave.start) / WAVE_FRAME_TIME) as usize;
            let center = board.to_art(board.ground(wave.center));
            let min = board.art_to_screen(center - WAVE_CENTER);
            let rect = Rect::from_min_size(min, WAVE_FRAME * board.scale);
            painter.image(tex.ui.wave(over, frame).id(), rect, UV, Color32::WHITE);
        }
    }

    /// Над колодой под курсором выезжает табличка с числом карт в ней — как атака и
    /// здоровье у фигур: значок карты в цвет огней стороны и цифры пиксельным шрифтом.
    /// Своя колода внизу — табличка над ней, колода соперника вверху — под ней.
    pub(super) fn paint_deck_count(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        now: f64,
    ) {
        let Some((seat, since)) = self.deck_hover else {
            return;
        };
        let Some(deck) = self.deck_rect(seat) else {
            return;
        };
        let t = ((now - since) / STATS_TIME).clamp(0.0, 1.0) as f32;
        let out = ease_out_back(t);
        let alpha = (t * 2.0).min(1.0);
        let icon = match self.seat_color(seat) {
            Color::White => DECK_ICON[0],
            Color::Black => DECK_ICON[1],
        };
        let digits = Glyph::digits(self.deck_count(seat).into());
        let text: Vec<(Glyph, Color32)> = std::iter::once((Glyph::Card, icon))
            .chain(digits.into_iter().map(|g| (g, PIXEL_WHITE)))
            .collect();
        // В маленьком окне табличка не мельче, чем нужно для чтения.
        let s = board.scale.max(1.75);
        let size = self.plate_size(&text) * s;
        let slide = DECK_PLATE_SLIDE * (1.0 - out) * s;
        let x = deck.min.x + CARD.x / 2.0;
        let min = match seat {
            Seat::Near => {
                let anchor = board.art_to_screen(vec2(x, deck.min.y - DECK_PLATE_GAP));
                anchor - vec2(size.x / 2.0, size.y - slide)
            }
            Seat::Far => {
                let anchor = board.art_to_screen(vec2(x, deck.max.y + DECK_PLATE_GAP));
                anchor - vec2(size.x / 2.0, slide)
            }
        };
        self.paint_plate(painter, tex, min, s, &text, alpha);
    }

    /// Очки стороны на месте `seat` у её колоды: большие венок победы, череп жертвы и ромб
    /// маны, над ними — число очков пиксельным шрифтом. Свои — справа от колоды внизу слева, очки
    /// соперника — отражение через центр доски.
    fn paint_points(&self, painter: &Painter, board: &BoardGeom, tex: &Textures, seat: Seat) {
        let color = self.seat_color(seat);
        let zoom = EMBLEM_ZOOM as f32;
        let kinds = [Points::Victory, Points::Sacrifice, Points::Mana];
        // Знаки разной высоты стоят вровень низом, числа над ними — на одной линии над
        // самым высоким.
        let tallest = kinds
            .iter()
            .map(|&p| tex.cards.emblem(p, color).size.y * zoom)
            .fold(0.0, f32::max);
        let bottom = DECK_AT.y + DECK_FULL.y;
        let mut x = DECK_AT.x + DECK_FULL.x + POINTS_GAP;
        for points in kinds {
            let sprite = tex.cards.emblem(points, color);
            let size = sprite.size * zoom;
            // Нижним краем знаки — вровень с низом полной колоды.
            let near = vec2(x, bottom - size.y);
            x += size.x + POINTS_SPACING;
            let min = match seat {
                Seat::Near => near,
                Seat::Far => MID * 2.0 - (near + size),
            };
            let center = min + size / 2.0;
            put(painter, board, sprite, center, Vec2::ZERO, zoom);
            let tint = POINTS_TINT[points as usize];
            let tint = Color32::from_rgb((tint >> 16) as u8, (tint >> 8) as u8, tint as u8);
            let n = self.game.position().points(color, points);
            let text: Vec<(Glyph, Color32)> = Glyph::digits(n.into())
                .into_iter()
                .map(|g| (g, tint))
                .collect();
            let px = board.scale * POINTS_FONT;
            let width = self.text_width(&text) * px;
            // Число — над знаком; у соперника стол повёрнут, и число под знаком.
            let height = GLYPH_HEIGHT * POINTS_FONT;
            let top = match seat {
                Seat::Near => bottom - tallest - POINTS_RAISE - height,
                Seat::Far => MID.y * 2.0 - bottom + tallest + POINTS_RAISE,
            };
            let corner = board.art_to_screen(vec2(center.x, top)) - vec2(width / 2.0, 0.0);
            self.pixel_text(painter, tex, corner, px, &text);
        }
    }

    // ----- Запас взятых фигур -----

    /// Ячейки запаса стороны на месте `seat`: вид, сколько готово, сколько ждёт, место
    /// (пиксели арта).
    fn reserve_slots(&self, seat: Seat) -> Vec<(Kind, u8, u8, Rect)> {
        let reserve = self.game.position().reserve(self.seat_color(seat));
        reserve
            .into_iter()
            .enumerate()
            .map(|(i, (kind, ready, waiting))| {
                let min = RESERVE_AT + vec2(i as f32 * (RESERVE_SLOT.x + RESERVE_GAP), 0.0);
                let min = match seat {
                    Seat::Near => min,
                    Seat::Far => MID * 2.0 - min - RESERVE_SLOT,
                };
                (
                    kind,
                    ready,
                    waiting,
                    Rect::from_min_size(min.to_pos2(), RESERVE_SLOT),
                )
            })
            .collect()
    }

    /// Ячейка запаса под курсором.
    pub(super) fn reserve_under(&self, board: &BoardGeom, pointer: Pos2) -> Option<(Seat, Kind)> {
        let at = board.to_art(pointer).to_pos2();
        [Seat::Near, Seat::Far].into_iter().find_map(|seat| {
            self.reserve_slots(seat)
                .into_iter()
                .find(|s| s.3.contains(at))
                .map(|s| (seat, s.0))
        })
    }

    /// Можно ли выставить фигуру `kind` из запаса на месте `seat` прямо сейчас.
    pub(super) fn reserve_usable(&self, seat: Seat, kind: Kind) -> bool {
        let pos = self.game.position();
        self.human_to_move()
            && self.seat_color(seat) == pos.side()
            && self.aiming.is_none()
            && pos
                .legal_moves()
                .iter()
                .any(|m| m.kind == MoveKind::Drop(kind))
    }

    /// Запас обеих сторон: фигура, под ней — сколько готово (×N) и сколько ждёт (+N).
    fn paint_reserve(&self, painter: &Painter, board: &BoardGeom, tex: &Textures) {
        let s = board.scale;
        for seat in [Seat::Near, Seat::Far] {
            let color = self.seat_color(seat);
            for (kind, ready, waiting, slot) in self.reserve_slots(seat) {
                let rect = Rect::from_min_max(
                    board.art_to_screen(slot.min.to_vec2()),
                    board.art_to_screen(slot.max.to_vec2()),
                );
                let usable = self.reserve_usable(seat, kind);
                let picked = usable && self.drop_pick == Some(kind);
                let hovered = usable && self.reserve_hover == Some((seat, kind));
                let shadow = rect.translate(vec2(2.0, 3.0) * s);
                painter.rect_filled(shadow, CornerRadius::ZERO, Color32::from_black_alpha(77));
                painter.rect_filled(
                    rect,
                    CornerRadius::ZERO,
                    Color32::from_rgb(0x0d, 0x0f, 0x17),
                );
                let inner = rect.shrink(s);
                let fill = if picked {
                    Color32::from_rgb(0x2a, 0x45, 0x5d)
                } else if hovered {
                    Color32::from_rgb(0x2a, 0x35, 0x4d)
                } else {
                    Color32::from_rgb(0x18, 0x1c, 0x29)
                };
                painter.rect_filled(inner, CornerRadius::ZERO, fill);
                let edge = if picked || hovered {
                    Color32::from_rgb(0x9f, 0xb3, 0xd1)
                } else if usable {
                    Color32::from_rgb(0x5f, 0xc8, 0xe0)
                } else {
                    Color32::from_rgb(0x2a, 0x33, 0x45)
                };
                painter.rect_stroke(
                    inner,
                    CornerRadius::ZERO,
                    Stroke::new(s, edge),
                    StrokeKind::Inside,
                );
                let ground = rect.min + vec2(RESERVE_SLOT.x / 2.0, RESERVE_GROUND) * s;
                let sprite = Rect::from_min_size(
                    board.snap(ground - ANCHOR * s * RESERVE_SCALE),
                    SPRITE * s * RESERVE_SCALE,
                );
                let tint = if ready > 0 {
                    Color32::WHITE
                } else {
                    Color32::from_white_alpha(110)
                };
                painter.image(tex.body(color, kind), sprite, UV, tint);
                let mut label = String::new();
                if ready > 0 {
                    label += &format!("×{ready}");
                }
                if waiting > 0 {
                    if !label.is_empty() {
                        label.push(' ');
                    }
                    label += &format!("+{waiting}");
                }
                let text = if ready > 0 {
                    theme::TEXT_BRIGHT
                } else {
                    Color32::from_rgb(0x6c, 0x76, 0x8a)
                };
                painter.text(
                    rect.min + vec2(RESERVE_SLOT.x / 2.0, RESERVE_SLOT.y - 9.0) * s,
                    Align2::CENTER_CENTER,
                    label,
                    FontId::new(10.0 * s, theme::bold()),
                    text,
                );
            }
        }
    }

    // ----- Кнопка «Конец хода» -----

    /// Курсор над кнопкой «Конец хода».
    pub(super) fn over_button(&self, board: &BoardGeom, pointer: Pos2) -> bool {
        button_rect().contains(board.to_art(pointer).to_pos2())
    }

    /// Кнопка работает, когда человек сходил фигурой и может закончить ход.
    pub(super) fn button_active(&self) -> bool {
        self.human_to_move() && self.game.can_end_turn() && self.aiming.is_none()
    }

    fn button_look(&self) -> Button {
        if !self.human_to_move() {
            Button::Disabled
        } else if !self.button_active() {
            Button::Normal
        } else if self.button_down && self.button_hover {
            Button::Pressed
        } else if self.button_hover {
            Button::Hover
        } else {
            // Фигурой уже сходили: осталась только карта — пора заканчивать ход.
            Button::Ready
        }
    }

    // ----- Рисование -----

    /// Колоды, кнопка, руки, летящие, возвращающиеся и горящие карты.
    pub(super) fn paint_table(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        now: f64,
    ) {
        let pos = self.game.position();
        let card = |pic: CardPic, center: Vec2, degrees: f32| {
            self.paint_card(painter, board, tex, pic, center, degrees, false);
        };

        let look = self.button_look();
        let size = if look == Button::Pressed {
            BUTTON + vec2(1.0, 1.0)
        } else {
            BUTTON + SHADOW_SHIFT
        };
        let rect = Rect::from_min_size(board.art_to_screen(BUTTON_AT), size * board.scale);
        painter.image(tex.ui.button(look).id(), rect, UV, Color32::WHITE);
        self.paint_reserve(painter, board, tex);

        for seat in [Seat::Near, Seat::Far].into_iter().filter(|_| SHOW_CARDS) {
            let color = self.seat_color(seat);
            let flying = self.draws.iter().filter(|d| d.color == color).count() as u8;
            if let Some(deck) = tex.cards.deck(color, pos.cards(color).deck_len() + flying) {
                put(
                    painter,
                    board,
                    deck,
                    seat.deck_at() + deck.size / 2.0,
                    Vec2::ZERO,
                    1.0,
                );
            }
            self.paint_points(painter, board, tex, seat);
            let layout = self.hand_layout(seat);
            let n = layout.len();
            let hand = self.hand(seat);
            let hovered = self
                .card_hover
                .filter(|&(s, i)| s == seat && layout.contains(&i))
                .map(|(_, i)| i);
            for k in draw_order(seat, n) {
                let index = layout[k];
                if Some(index) == hovered
                    || self.returning_card(seat, index)
                    || self.aimed_card(seat, index)
                {
                    continue;
                }
                let slot = seat.slot(k, n, 0.0);
                card(
                    self.pic(seat, hand[index], false),
                    slot.center,
                    slot.degrees,
                );
            }
            // Карта, ждущая цель, стоит над рукой со светлой кромкой.
            if let Some(aim) = self.aiming.as_ref().filter(|a| a.seat == seat) {
                let slot = self.slot_of(seat, aim.index, LIFT);
                card(self.pic(seat, aim.kind, true), slot.center, 0.0);
            }
            if let Some(index) = hovered {
                // Крупная карта — на верхнем слое: у края руки она заходит на правую панель,
                // и та её не обрезает.
                let painter = &painter.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("board-top"),
                ));
                // Светлая кромка — только у карты, которую можно сыграть (и на неё хватает очков).
                let pic = self.pic(seat, hand[index], self.usable(seat, index));
                let center = self.zoomed_center(seat, index, self.hover_lift(seat, now));
                self.paint_card(painter, board, tex, pic, center, 0.0, true);
                if let CardPic::Front { kind, .. } = pic
                    && kind.description().is_empty()
                {
                    // Карты без текста подписаны над картой.
                    let edge = center - vec2(0.0, seat.sign() * (CARD.y * ZOOM / 2.0 + 3.0));
                    let anchor = if seat == Seat::Near {
                        Align2::CENTER_BOTTOM
                    } else {
                        Align2::CENTER_TOP
                    };
                    paint_title(painter, board.art_to_screen(edge), anchor, kind.title());
                }
            }
        }

        // Карта из колоды летит на крайнее место в руке. Летящие карты — последние в руке.
        for (color, flying) in [Color::White, Color::Black].map(|c| {
            (
                c,
                self.draws
                    .iter()
                    .filter(|d| d.color == c)
                    .collect::<Vec<_>>(),
            )
        }) {
            let seat = self.seat_of(color);
            let hand = self.hand(seat);
            let n = self.hand_layout(seat).len() + 1;
            for (j, draw) in flying.iter().enumerate() {
                let u = ((now - draw.start) / DRAW_TIME).clamp(0.0, 1.0) as f32;
                if u <= 0.0 {
                    continue;
                }
                let from = seat.deck_at() + CARD / 2.0 + vec2(2.0, 0.0);
                let k = draw_order(seat, n).last().unwrap_or(0);
                let to = seat.slot(k, n, 0.0).center;
                let index = hand.len() - flying.len() + j;
                let kind = hand.get(index).copied().unwrap_or(CardKind::Normal);
                card(
                    self.pic(seat, kind, false),
                    from + (to - from) * ease_out(u),
                    0.0,
                );
            }
        }

        // Отпущенная над рукой карта возвращается на своё место и ложится в веер.
        for back in &self.returning {
            let u = ((now - back.start) / RETURN_TIME).clamp(0.0, 1.0) as f32;
            let slot = self.slot_of(back.seat, back.index, 0.0);
            let e = ease_in_out(u);
            let at = back.from + (slot.center - back.from) * e;
            let degrees = (back.tilt + (slot.degrees - back.tilt) * e).round();
            card(self.pic(back.seat, back.kind, false), at, degrees);
        }

        for burn in &self.burns {
            let frame = ((now - burn.start) / BURN_FRAME_TIME) as usize;
            let min = burn.center - CARD / 2.0 - BURN_CARD_AT;
            let rect = Rect::from_min_size(
                board.snap(board.art_to_screen(min)),
                BURN_FRAME * board.scale,
            );
            let texture = tex.cards.burn(burn.kind, burn.color, frame);
            painter.image(texture.id(), rect, UV, Color32::WHITE);
            // Текст и цена гаснут, пока огонь не добрался до середины карты.
            let fade = 1.0 - ((now - burn.start) / BURN_TEXT_TIME) as f32;
            let at = board.art_to_screen(burn.center);
            paint_card_text(painter, at, board.scale, 0.0, burn.kind, fade);
            if let Some((points, n)) = burn.kind.cost()
                && fade > 0.0
            {
                let tint = COST_TINT[points as usize];
                let tint = Color32::from_rgb((tint >> 16) as u8, (tint >> 8) as u8, tint as u8)
                    .gamma_multiply(fade);
                let text: Vec<(Glyph, Color32)> = Glyph::digits(n.into())
                    .into_iter()
                    .map(|g| (g, tint))
                    .collect();
                let corner = burn.center - CARD / 2.0 + cost_corner(points, n);
                self.pixel_text(
                    painter,
                    tex,
                    board.art_to_screen(corner),
                    board.scale,
                    &text,
                );
            }
        }
    }

    /// Карта с тенью и текстом; `zoomed` — в `ZOOM` раз крупнее, для карты под курсором.
    #[allow(clippy::too_many_arguments)]
    fn paint_card(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        pic: CardPic,
        center: Vec2,
        degrees: f32,
        zoomed: bool,
    ) {
        if zoomed {
            let sprite = self.art.card_zoomed(painter.ctx(), tex, pic, ZOOM as usize);
            put(painter, board, &sprite, center, SHADOW_SHIFT, ZOOM);
        } else {
            let sprite = self.art.card(painter.ctx(), tex, pic, degrees);
            put(painter, board, &sprite, center, Vec2::ZERO, 1.0);
        }
        if let CardPic::Front { kind, .. } = pic {
            let zoom = if zoomed { ZOOM } else { 1.0 };
            paint_card_text(
                painter,
                board.art_to_screen(center),
                board.scale * zoom,
                degrees,
                kind,
                1.0,
            );
        }
    }

    /// Взятая карта — поверх всего: доски, фигур и панелей.
    pub(super) fn paint_held(&self, painter: &Painter, board: &BoardGeom, tex: &Textures) {
        let Some(held) = &self.held else {
            return;
        };
        let pic = self.pic(held.seat, held.kind, true);
        let degrees = held.tilt.round();
        let sprite = self.art.card(painter.ctx(), tex, pic, degrees);
        put(
            painter,
            board,
            &sprite,
            held.at,
            HELD_SHADOW - SHADOW_SHIFT,
            1.0,
        );
        paint_card_text(
            painter,
            board.art_to_screen(held.at),
            board.scale,
            degrees,
            held.kind,
            1.0,
        );
    }

    /// Стрелка карты, ждущей цель, — от своего короля к курсору, как в Hearthstone от
    /// героя; над своей фигурой она голубая и вокруг клетки встают метки цели. У обмена,
    /// когда первая фигура выбрана, метки остаются на ней, а стрелка идёт от неё ко
    /// второй — к той, с кем она поменяется.
    pub(super) fn paint_aim(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        pointer: Option<Pos2>,
    ) {
        let Some(aim) = &self.aiming else {
            return;
        };
        let pos = self.game.position();
        let color = self.seat_color(aim.seat);
        // Метки цели — четыре шеврона вокруг ромба клетки.
        let mark = |sq: Square, look: Aim| {
            let ground = board.to_art(board.ground(sq));
            let min = ground - TILE / 2.0 - Vec2::splat(TARGET_PAD);
            let size = TILE + Vec2::splat(2.0 * TARGET_PAD);
            let rect = Rect::from_min_size(board.art_to_screen(min), size * board.scale);
            painter.image(tex.ui.target(look).id(), rect, UV, Color32::WHITE);
            ground - vec2(0.0, AIM_RISE)
        };
        if let Some(first) = aim.first {
            mark(first, Aim::Ally);
        }
        // Стрелка выходит из макушки короля или первой фигуры обмена (без короля карты не
        // играют — и целиться нечем).
        let Some(origin) = aim.first.or(pos.king_square(color)) else {
            return;
        };
        let height = pos
            .piece(origin)
            .map_or(0.0, |p| self.art.height(p.color, p.kind));
        let start = board.to_art(board.ground(origin)) - vec2(0.0, height + KING_CROWN);
        let locked = self.hovered.filter(|sq| self.aim_targets().contains(sq));
        let (end, look) = match (locked, pointer) {
            (Some(target), _) => {
                // Карта по чужой фигуре — красная стрелка и метка.
                let look = if pos.piece(target).is_some_and(|p| p.color != color) {
                    Aim::Enemy
                } else {
                    Aim::Ally
                };
                (mark(target, look), look)
            }
            (None, Some(pointer)) => (board.to_art(pointer), Aim::Idle),
            (None, None) => return,
        };
        // Цель — сама фигура, от которой идёт стрелка: хватает меток цели.
        if locked != Some(origin) {
            self.paint_arrow(painter, board, tex, start, end, look);
        }
    }

    /// Стрелка из звеньев по квадратичной кривой: выходит вверх и подходит к цели
    /// сверху-сбоку; у начала звенья мелкие, к цели крупнее, на конце наконечник.
    fn paint_arrow(
        &self,
        painter: &Painter,
        board: &BoardGeom,
        tex: &Textures,
        start: Vec2,
        end: Vec2,
        look: Aim,
    ) {
        const SAMPLES: usize = 160;
        let control = vec2(start.x, start.y.min(end.y) - 0.28 * (end - start).length());
        let points: Vec<Vec2> = (0..=SAMPLES)
            .map(|i| {
                let t = i as f32 / SAMPLES as f32;
                start * (1.0 - t) * (1.0 - t) + control * 2.0 * (1.0 - t) * t + end * t * t
            })
            .collect();
        let mut lengths = vec![0.0f32];
        for pair in points.windows(2) {
            lengths.push(lengths.last().copied().unwrap_or(0.0) + (pair[1] - pair[0]).length());
        }
        let total = lengths.last().copied().unwrap_or(0.0);
        // Точка и направление касательной на расстоянии `d` вдоль кривой.
        let point_at = |d: f32| {
            let i = lengths.partition_point(|&s| s < d).clamp(1, SAMPLES);
            let dir = points[i] - points[i - 1];
            (points[i], dir.y.atan2(dir.x).to_degrees())
        };
        let head = ARROW_HEAD * 0.8;
        let mut parts = Vec::new();
        let mut d = 6.0;
        while d < total - head {
            let frac = d / total.max(1.0);
            let (part, step) = if frac < 0.3 {
                (ArrowPart::Small, ARROW_SPACING[0])
            } else if frac < 0.62 {
                (ArrowPart::Medium, ARROW_SPACING[1])
            } else {
                (ArrowPart::Large, ARROW_SPACING[2])
            };
            parts.push((part, point_at(d)));
            d += step;
        }
        parts.push((ArrowPart::Head, point_at((total - head / 2.0).max(0.0))));
        let sprites: Vec<(Vec2, Sprite)> = parts
            .into_iter()
            .map(|(part, (at, degrees))| {
                let degrees = (degrees / 5.0).round() as i32 * 5;
                (at, self.art.arrow(painter.ctx(), tex, part, look, degrees))
            })
            .collect();
        // Сначала все тени, потом звенья от карты к наконечнику.
        for (at, sprite) in &sprites {
            let min = board.snap(board.art_to_screen(*at - sprite.size / 2.0));
            let rect = Rect::from_min_size(min, (sprite.size + SHADOW_SHIFT) * board.scale);
            painter.image(sprite.shadow.id(), rect, UV, Color32::WHITE);
        }
        for (at, sprite) in &sprites {
            let min = board.snap(board.art_to_screen(*at - sprite.size / 2.0));
            let rect = Rect::from_min_size(min, sprite.size * board.scale);
            painter.image(sprite.body.id(), rect, UV, Color32::WHITE);
        }
    }
}

/// Название и описание на лице карты: название — в шапке справа от кристалла,
/// описание — в нижнем поле. Текст подбирается по размеру поля и поворачивается с картой;
/// `px` — точек экрана на пиксель карты.
pub(super) fn paint_card_text(
    painter: &Painter,
    center: Pos2,
    px: f32,
    degrees: f32,
    kind: CardKind,
    alpha: f32,
) {
    let description = kind.description();
    if description.is_empty() || alpha <= 0.0 {
        return;
    }
    let angle = degrees.to_radians();
    let (sin, cos) = angle.sin_cos();
    let turn = |v: Vec2| vec2(cos * v.x - sin * v.y, sin * v.x + cos * v.y);
    // На тёмных лицах сторон — белый текст; на светлом лице карты победы — багровое
    // название и почти чёрное описание (цвета рамки из `CARD_TYPES` в breach_cards.py).
    let (title_color, text_color) = match kind.family() {
        Family::Victory => (0x8a1426, 0x24060c),
        _ => (0xffffff, 0xffffff),
    };
    // Поля на лице карты 50×70 (пиксели арта) — от её середины: шапка x 17…45, y 4…12,
    // нижнее поле x 6…43, y 45…61 (по рамкам из breach_cards_hd.py; у всех карт одни).
    // Короткое описание не раздувается: на всех картах оно примерно одного размера.
    let boxes = [
        (
            kind.title(),
            vec2(6.0, -27.0),
            vec2(27.0, 8.0),
            f32::INFINITY,
            theme::bold(),
            title_color,
        ),
        (
            &description,
            vec2(-0.5, 18.0),
            vec2(36.0, 15.0),
            DESCRIPTION_FONT,
            theme::semibold(),
            text_color,
        ),
    ];
    for (text, middle, size, cap, family, color) in boxes {
        let largest = (size.y * 0.9).min(size.x * 0.25).min(cap) * px;
        let Some(galley) = fit_text(painter, text, size * px, largest, family) else {
            continue;
        };
        let color = Color32::from_rgb((color >> 16) as u8, (color >> 8) as u8, color as u8)
            .gamma_multiply(alpha);
        let corner = turn(middle * px - galley.size() / 2.0);
        painter.add(TextShape::new(center + corner, galley, color).with_angle(angle));
    }
}

/// Самый крупный шрифт не больше `largest`, которым текст влезает в поле `size` (с
/// переносом по ширине).
fn fit_text(
    painter: &Painter,
    text: &str,
    size: Vec2,
    largest: f32,
    family: FontFamily,
) -> Option<std::sync::Arc<egui::Galley>> {
    let mut font = largest;
    while font >= 4.0 {
        // Цвет не запекается в разметку: его задаёт `TextShape` (и гасит при сгорании).
        let galley = painter.layout(
            text.to_owned(),
            FontId::new(font, family.clone()),
            Color32::PLACEHOLDER,
            size.x,
        );
        if galley.size().x <= size.x + 0.5 && galley.size().y <= size.y + 0.5 {
            return Some(galley);
        }
        font *= 0.92;
    }
    None
}

/// Подпись над картой: светлый текст на тёмно-синей плашке с кромкой, как панели ITB.
pub(super) fn paint_title(painter: &Painter, at: Pos2, anchor: Align2, text: &str) {
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        FontId::new(13.0, theme::bold()),
        theme::TEXT_BRIGHT,
    );
    let rect = anchor
        .anchor_size(at, galley.size())
        .expand2(vec2(6.0, 2.0));
    painter.rect_filled(
        rect,
        CornerRadius::ZERO,
        Color32::from_rgb(0x18, 0x1c, 0x29),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0_f32, Color32::from_rgb(0x49, 0x5c, 0x79)),
        StrokeKind::Inside,
    );
    painter.galley(rect.min + vec2(6.0, 2.0), galley, theme::TEXT_BRIGHT);
}

/// Вариант снега и глыбы на клетке — у соседних клеток разные (`variant_of` в
/// breach_freeze.py).
pub(super) fn ice_variant(sq: Square) -> usize {
    ((file_of(sq) * 3 + rank_of(sq) * 5) % 4) as usize
}

/// Кадр заморозки на клетке `sq`: в кадре центр верхней грани клетки — `FREEZE_CENTER`.
fn put_frame(painter: &Painter, board: &BoardGeom, sprite: &FrameSprite, sq: Square, alpha: f32) {
    let ground = board.to_art(board.ground(sq));
    let min = board.art_to_screen(ground - FREEZE_CENTER + sprite.at);
    let rect = Rect::from_min_size(min, sprite.size * board.scale);
    let tint = Color32::from_white_alpha((alpha.clamp(0.0, 1.0) * 255.0) as u8);
    painter.image(sprite.texture, rect, UV, tint);
}

/// Картинка с тенью, увеличенная в `zoom` раз; `lift` отодвигает тень дальше обычного —
/// карта поднята над столом.
fn put(painter: &Painter, board: &BoardGeom, sprite: &Sprite, center: Vec2, lift: Vec2, zoom: f32) {
    let size = sprite.size * zoom;
    // Левый верхний угол — на физическом пикселе: карта в веере и в полёте не размывается.
    let min = board.snap(board.art_to_screen(center - size / 2.0));
    let shadow = Rect::from_min_size(
        min + lift * board.scale,
        (size + SHADOW_SHIFT * zoom) * board.scale,
    );
    painter.image(sprite.shadow.id(), shadow, UV, Color32::WHITE);
    let body = Rect::from_min_size(min, size * board.scale);
    painter.image(sprite.body.id(), body, UV, Color32::WHITE);
}

/// В каком порядке рисовать карты руки: правые своей руки поверх левых, у соперника
/// наоборот (рука повёрнута на 180°). Последней в порядке лежит верхняя карта.
fn draw_order(seat: Seat, n: usize) -> Box<dyn DoubleEndedIterator<Item = usize>> {
    match seat {
        Seat::Near => Box::new(0..n),
        Seat::Far => Box::new((0..n).rev()),
    }
}

/// Точка (в пикселях арта) в системе координат карты на месте `slot`: x — поперёк карты
/// от её оси, y — вдоль.
fn card_local(slot: Slot, at: Vec2) -> Vec2 {
    let (sin, cos) = slot.degrees.to_radians().sin_cos();
    let d = at - slot.center;
    // Поворот на тот же угол обратно — в систему координат карты.
    vec2(cos * d.x + sin * d.y, -sin * d.x + cos * d.y)
}

/// Попадает ли точка (в пикселях арта) в карту на месте `slot`.
fn covers(slot: Slot, at: Vec2) -> bool {
    let local = card_local(slot, at);
    local.x.abs() <= CARD.x / 2.0 && local.y.abs() <= CARD.y / 2.0
}

/// Место в веере из `n` карт под точкой `at` (`away` — карты, которых в веере сейчас
/// нет). Из карт, которые накрывают точку, берётся та, к чьей оси точка ближе: полоса
/// каждой карты — от середины до середины между ней и соседями, уже самой карты, и
/// курсор, ведомый вдоль руки, перебирает карты ровно по одной, без перескоков.
fn fan_slot_at(seat: Seat, n: usize, at: Vec2, away: impl Fn(usize) -> bool) -> Option<usize> {
    (0..n)
        .filter(|&k| !away(k))
        .map(|k| (k, seat.slot(k, n, 0.0)))
        .filter(|&(_, slot)| covers(slot, at))
        .min_by(|a, b| {
            let off = |slot: Slot| card_local(slot, at).x.abs();
            off(a.1).total_cmp(&off(b.1))
        })
        .map(|(k, _)| k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_matches_the_art_layout() {
        for seat in [Seat::Near, Seat::Far] {
            for n in 1..=7 {
                for k in 0..n {
                    for lift in [0.0, LIFT] {
                        let c = seat.slot(k, n, lift).center;
                        // Повёрнутая карта занимает не больше квадрата со стороной диагонали.
                        let half = CARD.length() / 2.0;
                        assert!(
                            c.x - half >= TABLE_MIN.x - 12.0 && c.x + half <= TABLE_MAX.x + 12.0
                        );
                        assert!(
                            c.y - half >= TABLE_MIN.y - 12.0 && c.y + half <= TABLE_MAX.y + 12.0
                        );
                    }
                }
            }
            let deck = seat.deck_at();
            assert!(deck.x >= TABLE_MIN.x && deck.x + DECK_FULL.x <= TABLE_MAX.x);
            assert!(deck.y >= TABLE_MIN.y && deck.y + DECK_FULL.y <= TABLE_MAX.y);
        }
        // Как в table_preview: своя колода в (24, 390) холста, чужая — в (570, 1), кнопка —
        // в (572, 231); холст сдвинут от картинки доски на (100, 70).
        let canvas = vec2(100.0, 70.0);
        assert_eq!(Seat::Near.deck_at() + canvas, vec2(24.0, 390.0));
        assert_eq!(Seat::Far.deck_at() + canvas, vec2(570.0, 1.0));
        assert_eq!(BUTTON_AT + canvas, vec2(572.0, 231.0));
        assert!(button_rect().max.x <= TABLE_MAX.x);
    }

    #[test]
    fn far_hand_mirrors_the_near_one() {
        for k in 0..5 {
            let near = Seat::Near.slot(k, 5, 0.0);
            let far = Seat::Far.slot(4 - k, 5, 0.0);
            // Через центр доски: карта k своей руки переходит в карту n−1−k соперника.
            assert!((near.center + far.center - MID * 2.0).length() < 1e-3);
            assert!((near.degrees - far.degrees).abs() < 1e-3);
        }
    }

    #[test]
    fn card_hit_test_follows_the_rotation() {
        let slot = Slot {
            center: vec2(100.0, 100.0),
            degrees: 20.0,
        };
        assert!(covers(slot, slot.center));
        // Верхний край повёрнутой по часовой карты сдвинут вправо.
        let top = vec2(20f32.to_radians().sin(), -20f32.to_radians().cos()) * 33.0;
        assert!(covers(slot, slot.center + top));
        assert!(!covers(slot, slot.center + vec2(0.0, -40.0)));
    }

    #[test]
    fn scanning_the_hand_visits_every_card_in_turn() {
        for seat in [Seat::Near, Seat::Far] {
            let sign = seat.sign();
            let base = seat.hand_at();
            for n in 1..=7 {
                let step = if n > 1 {
                    STEP.min(SPREAD / (n - 1) as f32)
                } else {
                    STEP
                };
                let angle = |k: usize| (k as f32 - (n as f32 - 1.0) / 2.0) * step;
                // Точка на дуге веера, по которой стоят середины карт.
                let arc = |degrees: f32| {
                    let (sin, cos) = degrees.to_radians().sin_cos();
                    vec2(base.x + sin * RADIUS, base.y + sign * RADIUS * (1.0 - cos))
                };
                assert!((arc(angle(0)) - seat.slot(0, n, 0.0).center).length() < 1e-3);
                // Курсор идёт вдоль руки: каждая карта — один раз и по порядку.
                let mut seen: Vec<usize> = Vec::new();
                let mut degrees = angle(0) - 12.0;
                while degrees < angle(n - 1) + 12.0 {
                    if let Some(k) = fan_slot_at(seat, n, arc(degrees), |_| false)
                        && seen.last() != Some(&k)
                    {
                        seen.push(k);
                    }
                    degrees += 0.1;
                }
                assert_eq!(seen, (0..n).collect::<Vec<_>>(), "{seat:?}, {n} карт");
                // Граница между соседями — ровно посередине между их осями.
                for k in 0..n - 1 {
                    let middle = (angle(k) + angle(k + 1)) / 2.0;
                    assert_eq!(fan_slot_at(seat, n, arc(middle - 0.2), |_| false), Some(k));
                    assert_eq!(
                        fan_slot_at(seat, n, arc(middle + 0.2), |_| false),
                        Some(k + 1)
                    );
                }
            }
        }
        // Карты, которой нет в веере, курсор не находит: под ним — соседняя.
        let middle = Seat::Near.slot(2, 5, 0.0).center;
        let left = middle - vec2(10.0, 0.0);
        assert_eq!(fan_slot_at(Seat::Near, 5, left, |_| false), Some(2));
        assert_eq!(fan_slot_at(Seat::Near, 5, left, |k| k == 2), Some(1));
        // Над веером (там, где крупная карта) — ни одной.
        let above = middle - vec2(0.0, CARD.y / 2.0 + 8.0);
        assert_eq!(fan_slot_at(Seat::Near, 5, above, |_| false), None);
    }

    #[test]
    fn hand_zone_covers_the_fan_but_not_the_board() {
        let zone = Seat::Near.hand_zone(5);
        for k in 0..5 {
            assert!(zone.contains(Seat::Near.slot(k, 5, 0.0).center.to_pos2()));
        }
        // Ниже руки — тоже рука, над доской — нет.
        assert!(zone.contains(HAND_AT.to_pos2() + vec2(0.0, 200.0)));
        assert!(!zone.contains(MID.to_pos2()));
        assert!(!zone.contains((HAND_AT + vec2(0.0, -90.0)).to_pos2()));
        let far = Seat::Far.hand_zone(5);
        assert!(far.contains((MID * 2.0 - HAND_AT).to_pos2()));
        assert!(!far.contains(MID.to_pos2()));
    }
}
