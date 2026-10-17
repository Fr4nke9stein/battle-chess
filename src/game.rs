//! Правила «боевых шахмат».
//!
//! Это шахматы, только у каждой фигуры есть атака и здоровье, и бьют фигуры не ходом, а
//! сами: ход — только на пустую клетку, а в конце хода стороны каждая её свободная фигура
//! бьёт одну чужую в досягаемости — на одну клетку по правилам своего хода (пешка — по
//! диагонали вперёд, конь — буквой «Г»; см. [`Position::end_turn`]). Удар снимает с цели
//! ровно столько здоровья, сколько у атакующего атаки, — без случайностей и без ответа.
//! Убив цель, атакующий встаёт на её клетку, ранив — остаётся на месте. Все числа в игре целые.
//!
//! Король тоже получает урон и гибнет, как любая фигура: партия на этом не
//! кончается, но без короля сторона больше не играет карты (новый король из пешки
//! не появится). Проигрывает сторона, у которой не осталось ни одной фигуры. Шах —
//! положение, в котором короля может убить удар соперника в конце его хода: это
//! предупреждение, а не запрет.
//!
//! Буря, лавина и ледяной барьер замораживают фигуры, кроме королей: замороженная фигура
//! пропускает ближайший ход своей стороны — не ходит, не бьёт и не отвечает на удары.
//! Кому ходить нечем, тот пропускает ход фигурой.

mod decks;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub use decks::{
    DECK_COUNT, DECK_NAMES, MAX_DECK, RECIPES as DEFAULT_RECIPES, decode_deck_code,
    encode_deck_code,
};
use decks::{UNSHUFFLED, deck_cards};

use crate::balance::{Param, Rule, balance, plural, shown};

/// Длина массива руки: больше `max_hand` из баланса быть не может.
pub const HAND_CAP: usize = 10;
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opponent(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// «Белые», «Чёрные».
    pub fn name(self) -> &'static str {
        match self {
            Color::White => "Белые",
            Color::Black => "Чёрные",
        }
    }

    /// «белых», «чёрных».
    pub fn genitive(self) -> &'static str {
        match self {
            Color::White => "белых",
            Color::Black => "чёрных",
        }
    }

    /// Направление хода пешек по вертикали.
    pub fn forward(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    fn home_rank(self) -> i8 {
        match self {
            Color::White => 0,
            Color::Black => 7,
        }
    }

    fn last_rank(self) -> i8 {
        self.opponent().home_rank()
    }

    /// Биты прав на короткую и длинную рокировку.
    fn castling_bits(self) -> (u8, u8) {
        match self {
            Color::White => (WHITE_SHORT, WHITE_LONG),
            Color::Black => (BLACK_SHORT, BLACK_LONG),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Боевые характеристики фигуры.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub attack: i32,
    pub max_hp: i32,
}

impl Kind {
    /// От короля к пешке — в таком порядке фигуры показаны в справке.
    pub const ALL: [Kind; 6] = [
        Kind::King,
        Kind::Queen,
        Kind::Rook,
        Kind::Bishop,
        Kind::Knight,
        Kind::Pawn,
    ];

    /// Во что может превратиться пешка (в порядке lichess).
    pub const PROMOTIONS: [Kind; 4] = [Kind::Queen, Kind::Knight, Kind::Rook, Kind::Bishop];

    /// Атака и здоровье из баланса (`balance.ini`); стандартные подобраны так, чтобы ни один
    /// удар не убивал целую фигуру, а на большинство целей уходило 2–6 ударов.
    pub fn stats(self) -> Stats {
        let piece = balance().piece(self);
        Stats {
            attack: piece.attack,
            max_hp: piece.hp,
        }
    }

    /// Сколько очков победы получает убийца фигуры и очков жертвы — её хозяин.
    pub fn points(self) -> u8 {
        balance().piece(self).points
    }

    /// «Пешка», «Конь», …
    pub fn title(self) -> &'static str {
        match self {
            Kind::Pawn => "Пешка",
            Kind::Knight => "Конь",
            Kind::Bishop => "Слон",
            Kind::Rook => "Ладья",
            Kind::Queen => "Ферзь",
            Kind::King => "Король",
        }
    }

    /// «пешка», «конь», …
    pub fn name(self) -> &'static str {
        match self {
            Kind::Pawn => "пешка",
            Kind::Knight => "конь",
            Kind::Bishop => "слон",
            Kind::Rook => "ладья",
            Kind::Queen => "ферзь",
            Kind::King => "король",
        }
    }

    /// «пешку», «коня», …
    pub fn accusative(self) -> &'static str {
        match self {
            Kind::Pawn => "пешку",
            Kind::Knight => "коня",
            Kind::Bishop => "слона",
            Kind::Rook => "ладью",
            Kind::Queen => "ферзя",
            Kind::King => "короля",
        }
    }

    pub fn is_feminine(self) -> bool {
        matches!(self, Kind::Pawn | Kind::Rook)
    }

    /// Буква в русской нотации; у пешки буквы нет.
    pub fn letter(self) -> &'static str {
        match self {
            Kind::Pawn => "",
            Kind::Knight => "К",
            Kind::Bishop => "С",
            Kind::Rook => "Л",
            Kind::Queen => "Ф",
            Kind::King => "Кр",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub kind: Kind,
    pub color: Color,
    pub hp: i32,
    /// Заморожена: не ходит и не бьёт, пока не пройдёт столько границ ходов её стороны —
    /// начал и концов (0 — не заморожена, `FOREVER` — навсегда).
    pub frozen: u8,
    /// «Один в поле воин»: здоровье на 25% больше, каждое убийство лечит целиком.
    pub champion: bool,
    /// Сколько раз атака выросла на 25% от обычной («Один в поле воин»).
    pub boost: u8,
    /// Ладья «Оборонительной позиции»: фигуры вокруг неё получают вдвое меньше урона.
    pub bastion: bool,
    /// «Воин из Льда»: удар фигуры замораживает выжившую цель.
    pub frost_touch: bool,
    /// Прибавка к атаке и к полному здоровью («Северный воин»).
    pub bonus_attack: i16,
    pub bonus_hp: i16,
    /// Пешка усиленного компьютера насыщена лавой: в конце хода своей стороны взорвётся
    /// огненной волной по чужим фигурам вокруг.
    pub lava: bool,
    /// Слон усиленного компьютера насыщен электричеством: урон его удара расходится на чужие
    /// фигуры по диагонали вокруг цели.
    pub spark: bool,
    /// Только что выставлена из запаса: в конце этого хода не бьёт.
    pub fresh: bool,
    /// Сколько раз фигура уже возвращалась из запаса (не больше `MAX_REBIRTHS`).
    pub reborn: u8,
}

impl Piece {
    pub fn new(kind: Kind, color: Color) -> Piece {
        Piece {
            kind,
            color,
            hp: kind.stats().max_hp,
            frozen: 0,
            champion: false,
            boost: 0,
            bastion: false,
            frost_touch: false,
            bonus_attack: 0,
            bonus_hp: 0,
            lava: false,
            spark: false,
            fresh: false,
            reborn: 0,
        }
    }

    pub fn is_frozen(self) -> bool {
        self.frozen > 0
    }

    pub fn max_hp(self) -> i32 {
        let base = self.kind.stats().max_hp;
        let base = if self.champion {
            base * (100 + balance().param(Param::ChampionPercent)) / 100
        } else {
            base
        };
        base + self.bonus_hp as i32
    }

    pub fn attack(self) -> i32 {
        let step = balance().param(Param::ChampionPercent);
        self.kind.stats().attack * (100 + step * self.boost as i32) / 100 + self.bonus_attack as i32
    }

    /// «Белый конь», «Чёрная ладья».
    pub fn title(self) -> String {
        let adjective = match (self.color, self.kind.is_feminine()) {
            (Color::White, false) => "Белый",
            (Color::White, true) => "Белая",
            (Color::Black, false) => "Чёрный",
            (Color::Black, true) => "Чёрная",
        };
        format!("{adjective} {}", self.kind.name())
    }
}

/// Клетка доски: 0 — a1, 7 — h1, 63 — h8.
pub type Square = u8;

pub fn square(file: i8, rank: i8) -> Option<Square> {
    if (0..8).contains(&file) && (0..8).contains(&rank) {
        Some((rank * 8 + file) as Square)
    } else {
        None
    }
}

pub fn file_of(sq: Square) -> i8 {
    (sq % 8) as i8
}

pub fn rank_of(sq: Square) -> i8 {
    (sq / 8) as i8
}

/// «e4».
pub fn square_name(sq: Square) -> String {
    format!("{}{}", (b'a' + sq % 8) as char, sq / 8 + 1)
}

#[cfg(test)]
pub fn parse_square(name: &str) -> Option<Square> {
    let mut chars = name.chars();
    let file = (chars.next()? as u32).checked_sub('a' as u32)?;
    let rank = (chars.next()? as u32).checked_sub('1' as u32)?;
    if chars.next().is_some() || file > 7 || rank > 7 {
        return None;
    }
    square(file as i8, rank as i8)
}

/// Ход фигурой — только на пустую клетку: бить ходом нельзя, фигуры бьют сами в конце
/// хода своей стороны (`Strike`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoveKind {
    Quiet,
    CastleShort,
    CastleLong,
    /// Выставление фигуры из запаса на пустую клетку своей половины (`from` = `to`).
    Drop(Kind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub kind: MoveKind,
    /// Во что превратится пешка, дошедшая до последней горизонтали.
    pub promotion: Option<Kind>,
}

impl Move {
    pub fn is_drop(&self) -> bool {
        matches!(self.kind, MoveKind::Drop(_))
    }

    fn new(from: Square, to: Square, kind: MoveKind) -> Move {
        Move {
            from,
            to,
            kind,
            promotion: None,
        }
    }
}

/// Результат удара.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Где стояла цель.
    pub square: Square,
    /// Цель до удара.
    pub target: Piece,
    pub damage: i32,
    pub killed: bool,
}

impl Hit {
    pub fn hp_after(&self) -> i32 {
        (self.target.hp - self.damage).max(0)
    }
}

/// Удар фигуры в конце хода её стороны: ответа нет; убив цель, атакующий встаёт на её клетку.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strike {
    /// Кто бил и откуда.
    pub from: Square,
    pub attacker: Piece,
    pub hit: Hit,
    /// Сколько здоровья восстановил «воин», убив цель.
    pub feast: i32,
    /// Выжившая цель замёрзла («Воин из Льда»).
    pub chilled: bool,
    /// Пешка, убив цель на последней горизонтали, стала ферзём.
    pub promoted: Option<Kind>,
    /// Слон с электричеством: тот же удар по чужим фигурам по диагонали вокруг цели (их не
    /// больше трёх — четвёртая диагональ ведёт к самому слону).
    pub splash: [Option<Hit>; 3],
    /// Убив цель вблизи, атакующий встал на её клетку (партия «с перемещением»).
    pub advanced: bool,
}

impl Strike {
    /// Все задетые ударом: цель, потом задетые молнией.
    pub fn hits(&self) -> impl Iterator<Item = &Hit> {
        std::iter::once(&self.hit).chain(self.splash.iter().flatten())
    }
}

/// Взрыв пешки, насыщенной лавой, в конце хода её стороны: огненная волна только по чужим
/// фигурам вокруг; сама пешка цела.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blast {
    pub center: Square,
    pub hits: Vec<WaveHit>,
}

/// Что случилось в конце хода стороны.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TurnEnd {
    /// Удары фигур — по очереди.
    pub strikes: Vec<Strike>,
    /// Взрывы пешек, насыщенных лавой, — после ударов.
    pub blasts: Vec<Blast>,
    /// Пешки усиленной стороны, насытившиеся лавой к ходу соперника.
    pub charged: Vec<Square>,
    /// Rogue-like: фигуры новой волны соперника, вышедшие к следующему ходу.
    pub spawned: Vec<(Square, Kind)>,
    /// Rogue-like: шаги фигур волн вперёд (откуда, куда) — до ударов.
    pub marched: Vec<(Square, Square)>,
}

/// Rogue-like режим (с 0.50, просьба пользователя): у соперника нет своих фигур — они
/// волнами выходят на его крайнюю горизонталь в конце каждого его хода («туман войны» — на
/// будущее) и шагают вперёд (`march`); игрок за свой ход
/// делает до `Rule::RogueMoves` ходов разными фигурами; «шведки» нет, зато в запасе с начала
/// две пешки, слон и конь. Отбил все волны — победа.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rogue {
    /// Сторона игрока; волны — у соперника.
    pub player: Color,
    /// Сколько волн уже вышло и сколько всего.
    pub wave: u8,
    pub waves: u8,
    /// Случайность волн этой партии.
    seed: u64,
}

impl TurnEnd {
    /// Никого не задело: в списке ходов такой конец хода не пишется.
    pub fn is_quiet(&self) -> bool {
        self.strikes.is_empty()
            && self.blasts.iter().all(|b| b.hits.is_empty())
            && self.spawned.is_empty()
            && self.marched.is_empty()
    }
}

/// Что произошло в результате хода.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub mv: Move,
    /// Походившая фигура (до хода).
    pub piece: Piece,
    /// Ход ладьи при рокировке.
    pub rook: Option<(Square, Square)>,
    pub promoted: Option<Kind>,
    /// Фигура встала в ледяную зону и замёрзла.
    pub frozen: bool,

    /// Перехват (с 0.44): чужие кони, до которых дошла походившая фигура, сразу бьют её;
    /// пешку, прошедшую двойным ходом мимо чужой пешки, та бьёт на проходе.
    pub reactions: Vec<Strike>,
}

const WHITE_SHORT: u8 = 1;
const WHITE_LONG: u8 = 2;
const BLACK_SHORT: u8 = 4;
const BLACK_LONG: u8 = 8;

const KNIGHT_JUMPS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const KING_STEPS: [(i8, i8); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];
const ROOK_RAYS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_RAYS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
/// Слон бьёт по диагонали на две клетки (с 0.40).
const BISHOP_STRIKES: [(i8, i8); 8] = [
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
    (2, 2),
    (2, -2),
    (-2, 2),
    (-2, -2),
];
/// Ладья бьёт по прямой на две клетки (с 0.43).
const ROOK_STRIKES: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (2, 0),
    (-2, 0),
    (0, 2),
    (0, -2),
];
/// Ферзь бьёт на две клетки во все стороны — по прямой и по диагонали (с 0.43).
const QUEEN_STRIKES: [(i8, i8); 16] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (2, 0),
    (2, 2),
    (0, 2),
    (-2, 2),
    (-2, 0),
    (-2, -2),
    (0, -2),
    (2, -2),
];
/// Пешки бьют по диагонали вперёд.
const WHITE_PAWN_STRIKES: [(i8, i8); 2] = [(-1, 1), (1, 1)];
const BLACK_PAWN_STRIKES: [(i8, i8); 2] = [(-1, -1), (1, -1)];

/// Куда достаёт удар фигуры — на одну клетку по правилам её хода: пешка — по диагонали
/// вперёд, слон — по диагонали на две клетки, ладья — по прямой на две, ферзь — во все стороны
/// на две, король — на одну во все стороны, конь — буквой «Г».
fn strike_steps(piece: Piece) -> &'static [(i8, i8)] {
    match (piece.kind, piece.color) {
        (Kind::Pawn, Color::White) => &WHITE_PAWN_STRIKES,
        (Kind::Pawn, Color::Black) => &BLACK_PAWN_STRIKES,
        (Kind::Knight, _) => &KNIGHT_JUMPS,
        (Kind::Bishop, _) => &BISHOP_STRIKES,
        (Kind::Rook, _) => &ROOK_STRIKES,
        (Kind::Queen, _) => &QUEEN_STRIKES,
        (Kind::King, _) => &KING_STEPS,
    }
}

/// Удар вблизи — после него, убив, фигура встаёт на клетку цели: у пешки, коня и короля
/// любой, у слона, ладьи и ферзя — только по соседней клетке.
fn close_strike(kind: Kind, from: Square, to: Square) -> bool {
    matches!(kind, Kind::Pawn | Kind::Knight | Kind::King)
        || (file_of(from) - file_of(to))
            .abs()
            .max((rank_of(from) - rank_of(to)).abs())
            == 1
}

/// Место клетки в очереди ударов: a8, a7, …, a1, b8, …, h1.
#[allow(dead_code)] // карты убраны из интерфейса (0.40)
pub fn strike_order(sq: Square) -> u8 {
    (file_of(sq) * 8 + 7 - rank_of(sq)) as u8
}

/// Клетки в очереди ударов.
fn strike_queue() -> impl Iterator<Item = Square> {
    (0..8).flat_map(|file| (0..8).rev().map(move |rank| (rank * 8 + file) as Square))
}

/// Какие права на рокировку пропадают, если с клетки ушла или на ней погибла фигура.
fn rights_lost_at(sq: Square) -> u8 {
    match sq {
        0 => WHITE_LONG,
        4 => WHITE_SHORT | WHITE_LONG,
        7 => WHITE_SHORT,
        56 => BLACK_LONG,
        60 => BLACK_SHORT | BLACK_LONG,
        63 => BLACK_SHORT,
        _ => 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    board: [Option<Piece>; 64],
    /// Где стоят короли; `None` — короля нет (погиб).
    kings: [Option<Square>; 2],
    side: Color,
    castling: u8,
    /// Полуходы без ударов и ходов пешек — для правила 50 ходов.
    halfmove_clock: u16,
    fullmove: u16,
    /// Колоды и руки сторон.
    cards: [Cards; 2],
    /// Сколько карт сторона, которая ходит, сыграла в этом ходу (не больше `CARDS_PER_TURN`).
    cards_played: u8,
    /// Сторона, которая ходит, уже сходила фигурой: дальше — карта или конец хода.
    moved: bool,
    /// Очки сторон: [сторона][победы, жертвы, мана].
    points: [[u8; 3]; 2],
    /// Ледяные зоны (клетки битовой маской) по сторонам, поставившим их: зона держится до
    /// начала следующего хода своей стороны и замораживает каждую вставшую в неё фигуру,
    /// кроме короля.
    ice: [u64; 2],
    /// Зоны, которые продержатся ещё ход своей стороны (ледяная стена): в начале её хода
    /// они переходят в `ice`.
    ice_later: [u64; 2],
    /// «Кровь Богу Крови»: вместо очков жертвы сторона получает очки победы.
    blood_god: [bool; 2],
    /// Сколько карт «Они множатся» вернётся в руку стороне в конце её хода.
    returning: [u8; 2],
    /// Сторона окружила соперника («Окружены и сломлены») и победила.
    encircled_by: Option<Color>,
    /// Барьеры (клетки маской) по сторонам, поставившим их: соперник в них не входит; барьер
    /// держится до начала следующего хода своей стороны.
    barrier: [u64; 2],
    /// Сколько фигур соперника убила сторона («Истории Побед»).
    kills: [u8; 2],
    /// Сторона с усиленными фигурами (компьютер): лава в пешках, электричество в слонах.
    boosted: Option<Color>,

    /// Запас убитых фигур соперника («шведка», с 0.43): [сторона][`Kind`][ходов ждать 0..=2]
    /// [сколько раз уже возвращалась: 0..`MAX_REBIRTHS`] — сколько таких фигур.
    reserve: [[[[u8; MAX_REBIRTHS as usize]; 3]; 6]; 2],
    /// Король стороны погиб — она проиграла (с 0.44).
    king_lost: [bool; 2],
    /// Партия «с перемещением при взятии» (с 0.46): ближники встают на клетку убитой.
    advance: bool,
    /// Rogue-like режим: волны фигур соперника.
    rogue: Option<Rogue>,
    /// Сколько ходов фигурой сторона сделала в этом ходу и клетки фигур, которые уже ходили
    /// (в rogue-like игрок ходит несколько раз — каждый раз другой фигурой).
    moves_made: u8,
    used: u64,
}

/// Сколько начал своих ходов ждать, прежде чем выставить взятую фигуру.
pub const RESERVE_WAIT: usize = 2;
/// Сколько раз фигура может вернуться из запаса (с 0.44): погибшая в последний раз в запас
/// не уходит.
pub const MAX_REBIRTHS: u8 = 2;

/// Очки, которыми платят за карты: очко победы — за убитую фигуру соперника (им платят
/// за карты победы), очко жертвы — за потерянную свою фигуру (за карты жертвы), мана —
/// `mana_per_turn` из баланса в начале каждого своего хода, копится (за обычные карты).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Points {
    Victory,
    Sacrifice,
    Mana,
}

/// `Piece::frozen` фигуры, замёрзшей навсегда («Абсолютный Ноль»): она не оттаивает.
pub const FOREVER: u8 = u8::MAX;

impl Points {
    /// «1 очко победы», «2 очка жертвы».
    pub fn amount(self, n: u8) -> String {
        if self == Points::Mana {
            return format!(
                "{n} {}",
                if n % 10 == 1 && n % 100 != 11 {
                    "ману"
                } else {
                    "маны"
                }
            );
        }
        let word = match n % 100 {
            11..=14 => "очков",
            _ => match n % 10 {
                1 => "очко",
                2..=4 => "очка",
                _ => "очков",
            },
        };
        let kind = match self {
            Points::Victory => "победы",
            Points::Sacrifice => "жертвы",
            Points::Mana => unreachable!("мана считается выше"),
        };
        format!("{n} {word} {kind}")
    }
}

/// Тип карты — как в арт-проекте.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardKind {
    /// Обычная карта — в цветах своей стороны, без действия.
    Normal,
    /// Огненная волна — обычная карта с действием: бьёт фигуры вокруг выбранной своей.
    FireWave,
    /// Карта жертвы: чёрная с красным, с черепом.
    Sacrifice,
    /// Карта победы без действия: белая с багровым, с венком. С 0.22 в колоде все карты
    /// победы с действием (обмен, исцеление); тип и картинки остаются.
    #[allow(dead_code)]
    Victory,
    /// Обмен — карта победы с действием: две выбранные свои фигуры меняются местами.
    Swap,
    /// Дополнительное финансирование — карта жертвы с действием: сторона берёт две карты.
    Funding,
    /// Исцеление — карта победы с действием: свой слон лечит своих соседей.
    Heal,
    /// Буря — обычная карта: замораживает три горизонтали.
    Storm,
    /// Лавина — обычная карта: замораживает три вертикали.
    Avalanche,
    /// Ледяной барьер — обычная карта: зона 3×3 на ход замораживает фигуры в ней и вошедших.
    IceBarrier,
    /// Жертва — карта жертвы: своя фигура уходит с доски за двойные очки жертвы.
    Offering,
    /// Окружены и сломлены — карта жертвы: свои фигуры на всех четырёх краях доски — победа.
    Encircle,
    /// Они множатся — обычная карта: своя фигура делится на две, карта возвращается в руку.
    Multiply,
    /// Один в поле воин — карта победы: фигура сильнее и лечится каждым убийством.
    Champion,
    /// Оборонительная позиция — карта победы: вокруг своей ладьи урон вдвое меньше.
    Bastion,
    /// Ледяная стена — обычная карта: горизонталь на два хода замораживает вставших на неё.
    IceWall,
    /// Массовое жертвоприношение — обычная карта: свои фигуры в квадрате 3×3 отдают
    /// половину здоровья за очки жертвы.
    MassSacrifice,
    /// Кровь Богу Крови — карта жертвы: вместо очков жертвы сторона получает очки победы.
    BloodGod,
    /// Абсолютный ноль — обычная карта: половина фигур соперника замерзает навсегда.
    AbsoluteZero,
    /// Ледяной снаряд — обычная карта: фигура соперника получает 500 урона и замерзает.
    IceShard,
    /// Барьер — карта победы: квадрат 3×3, в который соперник не может войти.
    Barrier,
    /// Хладнокровный правитель — карта жертвы: мана за замёрзшие фигуры соперника.
    ColdRuler,
    /// Ледяной взрыв — обычная карта: замёрзшая фигура соперника бьёт своей атакой всех
    /// вокруг на две клетки.
    IceBlast,
    /// Истории Побед — карта победы: мана за убитые фигуры соперника.
    VictoryTales,
    /// Холодная зима — карта жертвы: 500 урона всем замёрзшим фигурам соперника.
    HardWinter,
    /// Воин из Льда — карта победы: удары своей фигуры замораживают.
    IceWarrior,
    /// Северный воин — карта победы: своя фигура сильнее за каждую замёрзшую фигуру
    /// соперника.
    NorthWarrior,
}

/// Тип лица карты: обычная (в цветах стороны, за ману), жертвы, победы.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Normal,
    Sacrifice,
    Victory,
}

/// Как карта ложится на поле, если её направляют на клетку: три горизонтали, три
/// вертикали, квадрат 3×3 вокруг клетки или одна горизонталь.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    Ranks,
    Files,
    Square,
    Rank,
}

impl Area {
    /// Клетки, которые накроет карта, нацеленная на `aim`: у края доски полоса или квадрат
    /// сдвигаются внутрь, чтобы лечь на доску целиком.
    pub fn squares(self, aim: Square) -> Vec<Square> {
        let (file, rank) = (file_of(aim).clamp(1, 6), rank_of(aim).clamp(1, 6));
        (0..64)
            .filter(|&sq| {
                let near_file = (file_of(sq) - file).abs() <= 1;
                let near_rank = (rank_of(sq) - rank).abs() <= 1;
                match self {
                    Area::Ranks => near_rank,
                    Area::Files => near_file,
                    Area::Square => near_file && near_rank,
                    Area::Rank => rank_of(sq) == rank_of(aim),
                }
            })
            .collect()
    }

    /// Те же клетки битовой маской.
    fn mask(self, aim: Square) -> u64 {
        self.squares(aim).iter().fold(0, |mask, &sq| mask | 1 << sq)
    }
}

impl CardKind {
    /// Сколько видов карт.
    pub const COUNT: usize = 27;

    /// Все виды карт в порядке объявления (`kind as usize` — номер в этом списке).
    pub const ALL: [CardKind; CardKind::COUNT] = [
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

    pub fn title(self) -> &'static str {
        match self {
            CardKind::Normal => "Обычная карта",
            CardKind::FireWave => "Огненная Волна",
            CardKind::Sacrifice => "Карта жертвы",
            CardKind::Victory => "Карта победы",
            CardKind::Swap => "Обмен",
            CardKind::Funding => "Дополнительное финансирование",
            CardKind::Heal => "Исцеление",
            CardKind::Storm => "Буря",
            CardKind::Avalanche => "Лавина",
            CardKind::IceBarrier => "Ледяной барьер",
            CardKind::Offering => "Жертва",
            CardKind::Encircle => "Окружены и сломлены",
            CardKind::Multiply => "Они множатся",
            CardKind::Champion => "Один в Поле Воин",
            CardKind::Bastion => "Оборонительная позиция",
            CardKind::IceWall => "Ледяная стена",
            CardKind::MassSacrifice => "Массовое Жертвоприношение",
            CardKind::BloodGod => "Кровь Богу Крови",
            CardKind::AbsoluteZero => "Абсолютный Ноль",
            CardKind::IceShard => "Ледяной снаряд",
            CardKind::Barrier => "Барьер",
            CardKind::ColdRuler => "Хладнокровный правитель",
            CardKind::IceBlast => "Ледяной взрыв",
            CardKind::VictoryTales => "Истории Побед",
            CardKind::HardWinter => "Холодная зима",
            CardKind::IceWarrior => "Воин из Льда",
            CardKind::NorthWarrior => "Северный воин",
        }
    }

    /// Текст на лице карты; пусто — у карты нет действия. Числа берутся из баланса
    /// (`balance.ini`), поэтому после правки файла текст сам остаётся верным.
    pub fn description(self) -> String {
        let p = |param| balance().param(param);
        // «Половина», «вдвое меньше» — при стандартных 50%, иначе проценты.
        let percent = |param, half: &str, other: &str| {
            let n = p(param);
            if n == 50 {
                half.to_owned()
            } else {
                other.replace("{}", &n.to_string())
            }
        };
        match self {
            CardKind::FireWave => format!(
                "Выберите вашу фигуру\nНаносит {} урона всем фигурам в радиусе одной клетки",
                p(Param::WaveDamage)
            ),
            CardKind::Swap => "Выберите две ваши фигуры\nОни меняются местами".to_owned(),
            CardKind::Funding => {
                let n = p(Param::FundingDraw);
                format!("Вы берёте {n} {}", plural(n, ["карту", "карты", "карт"]))
            }
            CardKind::Heal => format!(
                "Применяется на Слона\nЛечит союзные существа на {} здоровья в радиусе одной клетки",
                p(Param::HealAmount)
            ),
            CardKind::Storm => {
                "Замораживает 3 ряда фигур по горизонтали, кроме короля, на один ход".to_owned()
            }
            CardKind::Avalanche => {
                "Замораживает 3 ряда фигур по вертикали, кроме короля, на один ход".to_owned()
            }
            CardKind::IceBarrier => {
                "Замораживает зону 3 на 3 на 1 ход: все фигуры, кроме короля, которые оказались в \
                 этой зоне или вошли в неё до её окончания, замораживаются на один ход"
                    .to_owned()
            }
            CardKind::Offering => {
                let times = p(Param::OfferingMultiplier);
                let how = if times == 2 {
                    "в два раза".to_owned()
                } else {
                    format!("в {times} {}", plural(times, ["раз", "раза", "раз"]))
                };
                format!("Убирает вашу фигуру, даёт {how} больше очков жертвы за неё")
            }
            CardKind::Encircle => {
                "Если ваши фигуры стоят по 4 краям поля, вы побеждаете".to_owned()
            }
            CardKind::Multiply => format!(
                "Выберите фигуру, кроме короля и ферзя: она разделится на две, вторая появляется \
                 рядом с первой на ближайшей клетке к королю. Обе фигуры имеют то же количество \
                 атаки, но {}, но не меньше одного (на фигуру с одним \
                 здоровьем карту применить можно, она размножится на две с одним здоровьем). \
                 После конца хода эта карта возвращается к вам в руку",
                percent(
                    Param::MultiplyPercent,
                    "вдвое меньше здоровья",
                    "{}% здоровья"
                )
            ),
            CardKind::Champion => format!(
                "Выберите фигуру, кроме короля: её атака и здоровье увеличиваются на {pct}%, после \
                 каждого поедания фигуры этой фигурой она восстанавливает всё здоровье и \
                 увеличивает свой урон ещё на {pct}%",
                pct = p(Param::ChampionPercent)
            ),
            CardKind::Bastion => format!(
                "Выберите ладью: до конца матча все союзные фигуры в радиусе одной клетки вокруг \
                 неё получают {}",
                percent(
                    Param::BastionPercent,
                    "в два раза меньше урона",
                    "{}% урона"
                )
            ),
            CardKind::IceWall => {
                "Выберите линию по горизонтали: в течение двух ходов любая наступившая на эту \
                 линию фигура будет заморожена на один ход"
                    .to_owned()
            }
            CardKind::MassSacrifice => format!(
                "Выберите квадрат 3 на 3: все ваши фигуры в этом квадрате теряют {}, \
                 вы получаете очки жертвы за эти фигуры",
                percent(
                    Param::MassSacrificePercent,
                    "половину здоровья",
                    "{}% здоровья"
                )
            ),
            CardKind::AbsoluteZero => format!(
                "{} фигур противника навсегда замораживается; если съесть все \
                 незамороженные фигуры, противник проигрывает",
                percent(Param::AbsoluteZeroPercent, "Половина", "{}%")
            ),
            CardKind::IceShard => format!(
                "Замораживает фигуру и наносит ей {} урона",
                p(Param::IceShardDamage)
            ),
            CardKind::Barrier => {
                "Поставьте барьер 3 на 3: враги не могут войти в этот барьер".to_owned()
            }
            CardKind::ColdRuler => {
                let n = p(Param::ColdRulerMana);
                format!(
                    "Вы получаете {n} {} маны за каждую замороженную фигуру противника",
                    plural(n, ["очко", "очка", "очков"])
                )
            }
            CardKind::IceBlast => {
                let radius = [
                    "одной клетки",
                    "двух клеток",
                    "трёх клеток",
                    "четырёх клеток",
                    "пяти клеток",
                    "шести клеток",
                    "семи клеток",
                ][(p(Param::IceBlastRadius) as usize).clamp(1, 7) - 1];
                format!(
                    "Выберите замороженную фигуру противника: нанесите урон, равный её атаке, всем \
                     фигурам в радиусе {radius}"
                )
            }
            CardKind::VictoryTales => {
                let n = p(Param::VictoryTalesMana);
                format!(
                    "Получите {n} {} за каждую побеждённую фигуру противника",
                    plural(n, ["ману", "маны", "маны"])
                )
            }
            CardKind::HardWinter => format!(
                "Нанесите всем замороженным фигурам противника {} урона",
                p(Param::HardWinterDamage)
            ),
            CardKind::IceWarrior => {
                "Зачаровывает вашу фигуру: теперь её атаки замораживают фигуры противника"
                    .to_owned()
            }
            CardKind::NorthWarrior => format!(
                "Зачаровывает вашу фигуру: она получает {} атаки и {} здоровья за каждую \
                 замороженную фигуру противника",
                p(Param::NorthAttack),
                p(Param::NorthHp)
            ),
            CardKind::BloodGod => {
                "Вместо очков жертвы вы теперь получаете очки победы, но вы больше не можете \
                 получать очки жертвы"
                    .to_owned()
            }
            _ => String::new(),
        }
    }

    /// Тип лица: им же определяется, чем платят за карту.
    pub fn family(self) -> Family {
        match self {
            CardKind::Sacrifice
            | CardKind::Funding
            | CardKind::Offering
            | CardKind::Encircle
            | CardKind::BloodGod
            | CardKind::ColdRuler
            | CardKind::HardWinter => Family::Sacrifice,
            CardKind::Victory
            | CardKind::Swap
            | CardKind::Heal
            | CardKind::Champion
            | CardKind::Bastion
            | CardKind::Barrier
            | CardKind::VictoryTales
            | CardKind::IceWarrior
            | CardKind::NorthWarrior => Family::Victory,
            _ => Family::Normal,
        }
    }

    /// Цена карты из баланса (`balance.ini`): обычные платят маной, карты жертвы — очками
    /// жертвы, победы — очками победы; у бесплатных карт (цена 0) цены нет.
    pub fn cost(self) -> Option<(Points, u8)> {
        let n = balance().cost(self);
        if n == 0 {
            return None;
        }
        let points = match self.family() {
            Family::Normal => Points::Mana,
            Family::Sacrifice => Points::Sacrifice,
            Family::Victory => Points::Victory,
        };
        Some((points, n))
    }

    /// Карта разыгрывается на выбранные клетки: свои фигуры или клетку поля.
    pub fn targeted(self) -> bool {
        self.picks() > 0
    }

    /// Сколько клеток выбрать для карты: волне, исцелению, жертве, размножению, воину и
    /// обороне — свою фигуру, обмену — две, картам на поле (`area`) — любую клетку.
    pub fn picks(self) -> usize {
        match self {
            CardKind::FireWave
            | CardKind::Heal
            | CardKind::Offering
            | CardKind::Multiply
            | CardKind::Champion
            | CardKind::Bastion
            | CardKind::IceShard
            | CardKind::IceBlast
            | CardKind::IceWarrior
            | CardKind::NorthWarrior => 1,
            CardKind::Swap => 2,
            kind if kind.area().is_some() => 1,
            _ => 0,
        }
    }

    /// Карта на поле: от короля к выбранной клетке тянется стрелка, и карта ложится на
    /// клетки вокруг неё.
    pub fn area(self) -> Option<Area> {
        match self {
            CardKind::Storm => Some(Area::Ranks),
            CardKind::Avalanche => Some(Area::Files),
            CardKind::IceBarrier | CardKind::MassSacrifice | CardKind::Barrier => {
                Some(Area::Square)
            }
            CardKind::IceWall => Some(Area::Rank),
            _ => None,
        }
    }

    /// Карта со льдом: буря, лавина, ледяной барьер, ледяная стена.
    pub fn freezes(self) -> bool {
        matches!(
            self,
            CardKind::Storm | CardKind::Avalanche | CardKind::IceBarrier | CardKind::IceWall
        )
    }
}

/// Все виды карт, которые есть в колодах игры, — для коллекции в меню: обычные, жертвы,
/// победы, в каждом типе от дешёвых к дорогим (при равной цене — по порядку в колодах).
/// Карта, которую пользователь убрал из всех колод в `balance.ini`, из коллекции пропадает.
pub fn collection() -> Vec<CardKind> {
    let mut kinds: Vec<CardKind> = Vec::new();
    for index in 0..DECK_COUNT {
        for &(kind, _) in balance().deck(index).iter().rev() {
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
    }
    kinds.sort_by_key(|k| (k.family() as u8, k.cost().map_or(0, |(_, n)| n)));
    kinds
}
/// «Северный воин»: прибавка к атаке и к здоровью за `frozen` замёрзших фигур соперника.
fn north_bonus(frozen: u8) -> (i16, i16) {
    let scale = |param| (balance().param(param) * i32::from(frozen)).min(i16::MAX.into()) as i16;
    (scale(Param::NorthAttack), scale(Param::NorthHp))
}

/// Сколько маны даёт «Хладнокровный правитель» за каждую замёрзшую фигуру.
fn cold_ruler_mana() -> u8 {
    balance().param(Param::ColdRulerMana) as u8
}

/// Сколько маны дают «Истории Побед» за каждую убитую фигуру.
fn victory_tales_mana() -> u8 {
    balance().param(Param::VictoryTalesMana) as u8
}

/// Лечение исцелением своей фигуры рядом со слоном.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heal {
    pub square: Square,
    /// Фигура до лечения.
    pub target: Piece,
    /// Сколько здоровья вернулось: не больше лечения карты и не выше полного.
    pub amount: i32,
}

/// Удар огненной волны по фигуре рядом с выбранной.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveHit {
    pub square: Square,
    /// Фигура до удара.
    pub target: Piece,
    pub damage: i32,
    pub killed: bool,
}

/// Разыгранная карта.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardPlay {
    pub color: Color,
    pub kind: CardKind,
    /// Где карта лежала в руке.
    pub index: usize,
    /// Выбранные фигуры: центр огненной волны, две фигуры обмена.
    pub targets: Vec<Square>,
    /// Кого задела огненная волна.
    pub hits: Vec<WaveHit>,
    /// Кого вылечило исцеление.
    pub heals: Vec<Heal>,
    /// Сколько карт сторона взяла из колоды (дополнительное финансирование): меньше
    /// карт финансирования, если колода кончилась или рука полна.
    pub drawn: u8,
    /// Кого заморозила карта: буря, лавина и ледяной барьер — фигуры на своих клетках,
    /// обмен — фигуру, вставшую в ледяную зону.
    pub frozen: Vec<Square>,
    /// Где встала вторая фигура «Они множатся».
    pub spawned: Option<Square>,
}

/// Колода и рука стороны. Массивы без кучи: компьютер копирует позиции тысячами.
/// За длиной массива лежат обычные карты, чтобы одинаковые руки были равны.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cards {
    /// Верх колоды — в конце.
    deck: [CardKind; MAX_DECK],
    deck_len: u8,
    hand: [CardKind; HAND_CAP],
    hand_len: u8,
}

impl Cards {
    fn new(cards: &[CardKind]) -> Cards {
        let mut deck = [CardKind::Normal; MAX_DECK];
        deck[..cards.len()].copy_from_slice(cards);
        Cards {
            deck,
            deck_len: cards.len() as u8,
            hand: [CardKind::Normal; HAND_CAP],
            hand_len: 0,
        }
    }

    /// Карты в руке слева направо; взятая из колоды встаёт справа.
    pub fn hand(&self) -> &[CardKind] {
        &self.hand[..self.hand_len as usize]
    }

    /// Сколько карт осталось в колоде.
    pub fn deck_len(&self) -> u8 {
        self.deck_len
    }

    /// Берёт верхнюю карту колоды, если колода не пуста и в руке есть место; alse —
    /// не взяла.
    fn draw(&mut self) -> bool {
        if self.deck_len == 0 || self.hand_len >= balance().max_hand() {
            return false;
        }
        self.deck_len -= 1;
        let top = std::mem::replace(&mut self.deck[self.deck_len as usize], CardKind::Normal);
        self.hand[self.hand_len as usize] = top;
        self.hand_len += 1;
        true
    }

    /// Возвращает карту в руку справа, а в полную руку — наверх колоды.
    fn give_back(&mut self, kind: CardKind) {
        if self.hand_len < balance().max_hand() {
            self.hand[self.hand_len as usize] = kind;
            self.hand_len += 1;
        } else if (self.deck_len as usize) < MAX_DECK {
            self.deck[self.deck_len as usize] = kind;
            self.deck_len += 1;
        }
    }

    /// Достаёт из руки карту `index`; остальные сдвигаются влево.
    fn take(&mut self, index: usize) -> Option<CardKind> {
        let len = self.hand_len as usize;
        if index >= len {
            return None;
        }
        let card = self.hand[index];
        self.hand.copy_within(index + 1..len, index);
        self.hand[len - 1] = CardKind::Normal;
        self.hand_len -= 1;
        Some(card)
    }
}

impl Position {
    pub fn start() -> Position {
        Position::from_fen(START_FEN).expect("начальная позиция корректна")
    }

    /// Позиция из обычной записи FEN; все фигуры получают полное здоровье.
    pub fn from_fen(fen: &str) -> Result<Position, String> {
        let mut fields = fen.split_whitespace();
        let placement = fields.next().ok_or("пустая строка FEN")?;
        let mut board = [None; 64];
        let (mut file, mut rank) = (0i8, 7i8);
        for c in placement.chars() {
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => file += c.to_digit(10).unwrap_or(0) as i8,
                _ => {
                    let color = if c.is_ascii_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let kind = match c.to_ascii_lowercase() {
                        'p' => Kind::Pawn,
                        'n' => Kind::Knight,
                        'b' => Kind::Bishop,
                        'r' => Kind::Rook,
                        'q' => Kind::Queen,
                        'k' => Kind::King,
                        _ => return Err(format!("неизвестная фигура «{c}»")),
                    };
                    let sq = square(file, rank).ok_or("фигура за пределами доски")?;
                    board[sq as usize] = Some(Piece::new(kind, color));
                    file += 1;
                }
            }
        }
        let side = match fields.next().unwrap_or("w") {
            "w" => Color::White,
            "b" => Color::Black,
            other => return Err(format!("неизвестная сторона «{other}»")),
        };
        let mut castling = 0;
        for c in fields.next().unwrap_or("-").chars() {
            castling |= match c {
                'K' => WHITE_SHORT,
                'Q' => WHITE_LONG,
                'k' => BLACK_SHORT,
                'q' => BLACK_LONG,
                '-' => 0,
                _ => return Err(format!("неверные права на рокировку «{c}»")),
            };
        }
        // Битое поле пропускаем: пешки ходом не бьют, взятия на проходе нет.
        fields.next();
        let halfmove_clock = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let fullmove = fields.next().and_then(|s| s.parse().ok()).unwrap_or(1);

        // Короля может и не быть — погиб.
        let mut kings = [None; 2];
        for color in [Color::White, Color::Black] {
            let mut found = (0..64u8).filter(|&s| {
                matches!(board[s as usize], Some(p) if p.kind == Kind::King && p.color == color)
            });
            kings[color as usize] = found.next();
            if found.next().is_some() {
                return Err(format!("у {} больше одного короля", color.genitive()));
            }
        }
        let mut position = Position {
            board,
            kings,
            side,
            castling,
            halfmove_clock,
            fullmove,
            cards: [Cards::new(&UNSHUFFLED); 2],
            cards_played: 0,
            moved: false,
            points: [[0; 3]; 2],
            ice: [0; 2],
            ice_later: [0; 2],
            blood_god: [false; 2],
            returning: [0; 2],
            encircled_by: None,
            barrier: [0; 2],
            kills: [0; 2],
            boosted: None,

            reserve: [[[[0; MAX_REBIRTHS as usize]; 3]; 6]; 2],
            king_lost: [false; 2],
            advance: false,
            rogue: None,
            moves_made: 0,
            used: 0,
        };
        position.deal();
        Ok(position)
    }

    /// Та же позиция, но колоды сторон (номера из меню; белые, чёрные) перемешаны заново и
    /// карты сданы из них.
    #[allow(dead_code)] // карты убраны из интерфейса (0.40)
    pub fn with_shuffled_cards(mut self, seed: u64, decks: [usize; 2]) -> Position {
        let mut state = seed | 1;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for color in [Color::White, Color::Black] {
            let mut deck = deck_cards(decks[color as usize]);
            for i in (1..deck.len()).rev() {
                deck.swap(i, (next() % (i as u64 + 1)) as usize);
            }
            self.cards[color as usize] = Cards::new(&deck);
        }
        self.deal();
        self
    }

    /// Та же позиция, но с перемещением при взятии или без (правило партии).
    pub fn with_advance(mut self, on: bool) -> Position {
        self.advance = on;
        self
    }

    /// Ближники встают на клетку убитой.
    pub fn advance(&self) -> bool {
        self.advance
    }

    /// Та же позиция без карт: колоды и руки пусты (с 0.40 карты убраны из интерфейса).
    pub fn without_cards(mut self) -> Position {
        self.cards = [Cards::new(&[]); 2];
        self.deal();
        self
    }

    /// Сдаёт каждой стороне по `start_hand` карт; ход начинается с карты из колоды и маны.
    /// Очки — с нуля: сдают и начальной позиции, и заново после перемешивания колод.
    fn deal(&mut self) {
        for color in [Color::White, Color::Black] {
            for _ in 0..balance().start_hand() {
                self.draw_card(color);
            }
        }
        self.cards_played = 0;
        self.moved = false;
        self.points = [[0; 3]; 2];
        self.draw_card(self.side);
        self.gain_mana();
    }

    pub fn side(&self) -> Color {
        self.side
    }

    pub fn cards(&self, color: Color) -> &Cards {
        &self.cards[color as usize]
    }

    /// Очки стороны `color`: победы или жертвы.
    pub fn points(&self, color: Color, kind: Points) -> u8 {
        self.points[color as usize][kind as usize]
    }

    /// Фигура `victim` погибла от стороны `killer`: её хозяин получает очки жертвы, убийца-
    /// соперник — очки победы, сколько стоит фигура (`Kind::points`); своя фигура, убитая
    /// своей волной, очков победы не даёт.
    fn score_kill(&mut self, victim: Piece, killer: Color) {
        let n = victim.kind.points();
        self.give_sacrifice(victim.color, n);
        if killer != victim.color {
            self.add_points(killer, Points::Victory, n);
            self.kills[killer as usize] = self.kills[killer as usize].saturating_add(1);
            // Убитая фигура (кроме короля) уходит в запас убийце — если ещё не исчерпала
            // возвращения.
            if victim.kind != Kind::King && victim.reborn < MAX_REBIRTHS && self.rogue.is_none() {
                let slot = &mut self.reserve[killer as usize][victim.kind as usize][RESERVE_WAIT]
                    [victim.reborn as usize];
                *slot = slot.saturating_add(1);
            }
        }
    }

    /// Очки жертвы стороне `color` — или очки победы, если она отдала кровь Богу Крови.
    fn give_sacrifice(&mut self, color: Color, n: u8) {
        let kind = if self.blood_god[color as usize] {
            Points::Victory
        } else {
            Points::Sacrifice
        };
        self.add_points(color, kind, n);
    }

    fn add_points(&mut self, color: Color, kind: Points, n: u8) {
        let points = &mut self.points[color as usize][kind as usize];
        *points = points.saturating_add(n);
    }

    /// Сыграла ли сторона «Кровь Богу Крови».
    #[allow(dead_code)] // карты убраны из интерфейса (0.40)
    pub fn blood_god(&self, color: Color) -> bool {
        self.blood_god[color as usize]
    }

    /// Свои фигуры на всех четырёх краях доски: «Окружены и сломлены».
    pub fn encircles(&self, color: Color) -> bool {
        let own: Vec<Square> = self
            .pieces()
            .filter(|&(_, p)| p.color == color)
            .map(|(sq, _)| sq)
            .collect();
        let edge = |test: &dyn Fn(Square) -> bool| own.iter().any(|&sq| test(sq));
        edge(&|sq| rank_of(sq) == 0)
            && edge(&|sq| rank_of(sq) == 7)
            && edge(&|sq| file_of(sq) == 0)
            && edge(&|sq| file_of(sq) == 7)
    }

    /// Кто победил, окружив соперника.
    pub fn encircled_by(&self) -> Option<Color> {
        self.encircled_by
    }

    /// Начало хода: стороне, которая ходит, — `mana_per_turn` маны (мана копится).
    fn gain_mana(&mut self) {
        let mana = &mut self.points[self.side as usize][Points::Mana as usize];
        *mana = mana.saturating_add(balance().mana_per_turn());
    }

    /// Хватает ли стороне, которая ходит, очков на карту `kind`.
    fn can_afford(&self, kind: CardKind) -> bool {
        kind.cost()
            .is_none_or(|(points, n)| self.points(self.side, points) >= n)
    }

    /// Сколько карт сторона, которая ходит, сыграла в этом ходу.
    pub fn cards_played(&self) -> u8 {
        self.cards_played
    }

    /// Сходила ли сторона фигурой в этом ходу: тогда фигурой больше не ходят — можно
    /// сыграть карту или закончить ход.
    pub fn moved(&self) -> bool {
        self.moved
    }

    /// Сколько ходов фигурой сторона, которая ходит, делает за свой ход: игрок в rogue-like —
    /// `Rule::RogueMoves`, остальные — один.
    pub fn moves_per_turn(&self) -> u8 {
        match self.rogue {
            Some(r) if r.player == self.side => balance().rule(Rule::RogueMoves).clamp(1, 64) as u8,
            _ => 1,
        }
    }

    /// Сколько ходов фигурой у стороны, которая ходит, ещё осталось в этом ходу.
    pub fn moves_left(&self) -> u8 {
        self.moves_per_turn().saturating_sub(self.moves_made)
    }

    /// Фигура на `sq` уже ходила в этом ходу (rogue-like: каждый ход — другой фигурой).
    pub fn used(&self, sq: Square) -> bool {
        self.used & 1 << sq != 0
    }

    /// Rogue-like: волны соперника; `None` — обычная партия.
    pub fn rogue(&self) -> Option<Rogue> {
        self.rogue
    }

    /// Проиграла ли сторона `color`: нет фигур или погиб король. В rogue-like соперник
    /// проигрывает, только когда вышли все волны и фигур не осталось.
    pub fn defeated(&self, color: Color) -> bool {
        match self.rogue {
            Some(r) if r.player != color => r.wave >= r.waves && !self.has_pieces(color),
            _ => !self.has_pieces(color) || self.king_lost(color) || self.breached(),
        }
    }

    /// Rogue-like: фигура волны дошла до первой горизонтали игрока — он проиграл.
    pub fn breached(&self) -> bool {
        let Some(r) = self.rogue else {
            return false;
        };
        let home = r.player.home_rank();
        (0..8).any(|file| {
            square(file, home)
                .and_then(|sq| self.board[sq as usize])
                .is_some_and(|p| p.color != r.player)
        })
    }

    /// Начало rogue-like партии: у игрока (белые) обычная армия и в запасе две пешки, слон и
    /// конь; у соперника фигур нет — сразу выходит первая волна. `seed` — случайность волн.
    pub fn rogue_start(seed: u64) -> Position {
        let mut pos = Position::from_fen("8/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1")
            .expect("начало rogue-like")
            .without_cards();
        let player = Color::White;
        let ready = &mut pos.reserve[player as usize];
        ready[Kind::Pawn as usize][0][0] = 2;
        ready[Kind::Bishop as usize][0][0] = 1;
        ready[Kind::Knight as usize][0][0] = 1;
        pos.rogue = Some(Rogue {
            player,
            wave: 0,
            waves: balance().rule(Rule::RogueWaves).clamp(1, 255) as u8,
            seed: seed | 1,
        });
        pos.spawn_wave();
        pos
    }

    /// Следующая волна соперника выходит на его крайнюю горизонталь: от `Rule::RogueWaveMin`
    /// до `Rule::RogueWaveMax` фигур (сколько поместится на свободные клетки), вид — по весам
    /// `Rule::RogueWeight*` (дорогие — реже). Не поместилась ни одна — волна ждёт следующего
    /// хода и не считается. Случайность — от зерна партии и ключа позиции: при отмене и в
    /// переборе волна та же.
    fn spawn_wave(&mut self) -> Vec<(Square, Kind)> {
        let Some(mut r) = self.rogue.filter(|r| r.wave < r.waves) else {
            return Vec::new();
        };
        let enemy = r.player.opponent();
        let mut state =
            r.seed ^ self.key() ^ (u64::from(r.wave) + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut next = move |n: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % n.max(1)
        };
        let rule = |rule: Rule| balance().rule(rule).max(0) as u64;
        let min = rule(Rule::RogueWaveMin).max(1);
        let max = rule(Rule::RogueWaveMax).max(min);
        let count = min + next(max - min + 1);
        let weights = [
            (Kind::Pawn, rule(Rule::RogueWeightPawn)),
            (Kind::Knight, rule(Rule::RogueWeightKnight)),
            (Kind::Bishop, rule(Rule::RogueWeightBishop)),
            (Kind::Rook, rule(Rule::RogueWeightRook)),
            (Kind::Queen, rule(Rule::RogueWeightQueen)),
        ];
        let total: u64 = weights.iter().map(|w| w.1).sum();
        let wall = self.wall(enemy);
        let back = enemy.home_rank();
        let mut spawned = Vec::new();
        for _ in 0..count {
            let kind = if total == 0 {
                Kind::Pawn
            } else {
                let mut roll = next(total);
                weights
                    .iter()
                    .find(|&&(_, w)| {
                        let hit = roll < w;
                        roll = roll.saturating_sub(w);
                        hit
                    })
                    .map_or(Kind::Pawn, |&(kind, _)| kind)
            };
            let free: Vec<Square> = (0..8)
                .filter_map(|file| square(file, back))
                .filter(|&sq| self.open(sq, wall))
                .collect();
            if free.is_empty() {
                break;
            }
            let sq = free[next(free.len() as u64) as usize];
            self.board[sq as usize] = Some(Piece::new(kind, enemy));
            spawned.push((sq, kind));
        }
        if spawned.is_empty() {
            return spawned;
        }
        r.wave += 1;
        self.rogue = Some(r);
        spawned.sort_unstable_by_key(|&(sq, _)| sq);
        spawned
    }

    /// Может ли сторона, которая ходит, сыграть карту: до `cards_per_turn` карт за ход и
    /// только пока жив её король.
    pub fn can_play_card(&self) -> bool {
        self.kings[self.side as usize].is_some()
            && self.cards_played < balance().cards_per_turn()
            && self.cards[self.side as usize].hand_len > 0
    }

    /// Есть ли что сыграть: карта без выбора фигуры или карта, для которой есть фигуры.
    pub fn any_card_playable(&self) -> bool {
        (0..self.cards[self.side as usize].hand().len()).any(|i| self.card_playable(i))
    }

    /// Можно ли сейчас сыграть карту `index`: до трёх карт за ход, на цену карты хватает
    /// очков, а карте с выбором нужна хотя бы одна подходящая своя фигура (для обмена —
    /// пара; карте на поле годится любая клетка).
    pub fn card_playable(&self, index: usize) -> bool {
        let Some(&kind) = self.cards[self.side as usize].hand().get(index) else {
            return false;
        };
        if !self.can_play_card() || !self.can_afford(kind) {
            return false;
        }
        match kind {
            CardKind::Swap => self.swap_pairs().next().is_some(),
            CardKind::Encircle => self.encircles(self.side),
            CardKind::BloodGod => !self.blood_god[self.side as usize],
            CardKind::AbsoluteZero => !self.zero_targets().is_empty(),
            CardKind::ColdRuler | CardKind::HardWinter => self.frozen_enemies() > 0,
            CardKind::VictoryTales => self.kills[self.side as usize] > 0,
            kind if kind.area().is_some() => true,
            kind if kind.picks() == 1 => (0..64).any(|sq| self.piece_target(kind, sq)),
            _ => true,
        }
    }

    /// Годится ли фигура на `sq` для карты `kind`, которую играют на одну фигуру. Чужая —
    /// ледяному снаряду любая, ледяному взрыву — замёрзшая. Своя — волне и жертве любая,
    /// исцелению — слон, размножению — не король и не ферзь (и рядом есть куда встать
    /// второй), воину — не король, обороне — ладья (по разу), «Воину из Льда» — ещё не
    /// зачарованная, «Северному воину» — любая, если у соперника есть замёрзшие.
    fn piece_target(&self, kind: CardKind, sq: Square) -> bool {
        let Some(piece) = self.piece(sq) else {
            return false;
        };
        if piece.color != self.side {
            return match kind {
                CardKind::IceShard => true,
                CardKind::IceBlast => piece.is_frozen(),
                _ => false,
            };
        }
        match kind {
            CardKind::FireWave | CardKind::Offering => true,
            CardKind::Heal => piece.kind == Kind::Bishop,
            CardKind::Multiply => {
                !matches!(piece.kind, Kind::King | Kind::Queen) && self.spawn_square(sq).is_some()
            }
            CardKind::Champion => piece.kind != Kind::King && !piece.champion,
            CardKind::Bastion => piece.kind == Kind::Rook && !piece.bastion,
            CardKind::IceWarrior => !piece.frost_touch,
            CardKind::NorthWarrior => self.frozen_enemies() > 0,
            _ => false,
        }
    }

    /// Куда встанет вторая фигура размножения с `sq`: пустая соседняя клетка, ближайшая к
    /// своему королю (пешка — не на последнюю горизонталь).
    fn spawn_square(&self, sq: Square) -> Option<Square> {
        let piece = self.piece(sq)?;
        let king = self.king_square(piece.color)?;
        let distance = |s: Square| {
            let (df, dr) = (file_of(s) - file_of(king), rank_of(s) - rank_of(king));
            (df.abs().max(dr.abs()), df * df + dr * dr, s)
        };
        KING_STEPS
            .iter()
            .filter_map(|&(df, dr)| square(file_of(sq) + df, rank_of(sq) + dr))
            .filter(|&s| self.board[s as usize].is_none())
            .filter(|&s| self.barrier[piece.color.opponent() as usize] & 1 << s == 0)
            .filter(|&s| piece.kind != Kind::Pawn || rank_of(s) != piece.color.last_rank())
            .min_by_key(|&s| distance(s))
    }

    /// Клетки, которые можно выбрать первыми для карты `index`: центр огненной волны,
    /// первая фигура обмена, подходящая своя фигура (`piece_target`), любая клетка для
    /// карты на поле; пусто — карта без выбора или сыграть её нельзя.
    pub fn card_targets(&self, index: usize) -> Vec<Square> {
        let hand = self.cards[self.side as usize].hand();
        if !self.can_play_card() || !hand.get(index).is_some_and(|&k| self.can_afford(k)) {
            return Vec::new();
        }
        match hand.get(index) {
            Some(CardKind::Swap) => {
                let mut squares: Vec<Square> =
                    self.swap_pairs().flat_map(|(a, b)| [a, b]).collect();
                squares.sort_unstable();
                squares.dedup();
                squares
            }
            Some(kind) if kind.area().is_some() => (0..64).collect(),
            Some(&kind) if kind.picks() == 1 => {
                (0..64).filter(|&sq| self.piece_target(kind, sq)).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Кого и насколько вылечит исцеление слоном на `center`: свои фигуры на 8 соседних
    /// клетках — на лечение карты, но не выше полного здоровья; целые не в счёт.
    pub fn heal_targets(&self, center: Square) -> Vec<Heal> {
        let Some(bishop) = self.piece(center) else {
            return Vec::new();
        };
        KING_STEPS
            .iter()
            .filter_map(|&(df, dr)| square(file_of(center) + df, rank_of(center) + dr))
            .filter_map(|sq| {
                let target = self.board[sq as usize].filter(|p| p.color == bishop.color)?;
                let amount = balance()
                    .param(Param::HealAmount)
                    .min(target.max_hp() - target.hp);
                (amount > 0).then_some(Heal {
                    square: sq,
                    target,
                    amount,
                })
            })
            .collect()
    }

    fn heal_around(&mut self, center: Square) -> Vec<Heal> {
        let heals = self.heal_targets(center);
        for heal in &heals {
            if let Some(piece) = &mut self.board[heal.square as usize] {
                piece.hp += heal.amount;
            }
        }
        heals
    }

    /// С какими своими фигурами можно обменять фигуру на `first`.
    pub fn swap_partners(&self, first: Square) -> Vec<Square> {
        self.own_squares()
            .filter(|&sq| self.swap_is_safe(first, sq))
            .collect()
    }

    /// Клетки своих фигур — стороны, которая ходит.
    fn own_squares(&self) -> impl Iterator<Item = Square> + '_ {
        self.pieces()
            .filter(move |&(_, p)| p.color == self.side)
            .map(|(sq, _)| sq)
    }

    /// Пары своих фигур, которые можно поменять местами.
    fn swap_pairs(&self) -> impl Iterator<Item = (Square, Square)> + '_ {
        let own: Vec<Square> = self.own_squares().collect();
        let pairs: Vec<(Square, Square)> = own
            .iter()
            .enumerate()
            .flat_map(|(i, &a)| own[i + 1..].iter().map(move |&b| (a, b)))
            .collect();
        // Проверка ленивая: чтобы узнать, есть ли пара, хватает первой подходящей.
        pairs
            .into_iter()
            .filter(move |&(a, b)| self.swap_is_safe(a, b))
    }

    /// Можно ли поменять местами свои фигуры на `a` и `b`. Пешку не ставят на последнюю
    /// горизонталь: там она не ходит и не превращается.
    fn swap_is_safe(&self, a: Square, b: Square) -> bool {
        let (Some(first), Some(second)) = (self.piece(a), self.piece(b)) else {
            return false;
        };
        if a == b || first.color != self.side || second.color != self.side {
            return false;
        }
        let stuck = |piece: Piece, sq: Square| {
            piece.kind == Kind::Pawn && rank_of(sq) == piece.color.last_rank()
        };
        // В барьер соперника фигуры не входят — и обменом тоже.
        let wall = self.barrier[self.side.opponent() as usize];
        !stuck(first, b) && !stuck(second, a) && wall & (1 << a | 1 << b) == 0
    }

    /// Фигуры на `a` и `b` меняются местами вместе со своим здоровьем (и заморозкой);
    /// ушедшие со своих клеток король и ладьи теряют право на рокировку, вставшая в ледяную
    /// зону фигура замерзает. Возвращает замёрзшие клетки.
    fn swap_pieces(&mut self, a: Square, b: Square) -> Vec<Square> {
        self.board.swap(a as usize, b as usize);
        for sq in [a, b] {
            if let Some(piece) = self.board[sq as usize]
                && piece.kind == Kind::King
            {
                self.kings[piece.color as usize] = Some(sq);
            }
        }
        self.castling &= !(rights_lost_at(a) | rights_lost_at(b));
        [a, b]
            .into_iter()
            .filter(|&sq| self.on_ice(sq) && self.freeze_at(sq))
            .collect()
    }

    /// Лежит ли клетка в чьей-нибудь ледяной зоне.
    fn on_ice(&self, sq: Square) -> bool {
        self.ice_zones() & 1 << sq != 0
    }

    /// Ледяные зоны — клетки, где сейчас лежит лёд барьера и стены (обеих сторон).
    pub fn ice_zones(&self) -> u64 {
        self.ice[0] | self.ice[1] | self.ice_later[0] | self.ice_later[1]
    }

    /// Урон `raw` по фигуре на `sq`: рядом со своей ладьёй «Оборонительной позиции» — вдвое
    /// меньше (чужих ладья не прикрывает).
    pub fn damage_at(&self, sq: Square, raw: i32) -> i32 {
        let Some(target) = self.board[sq as usize] else {
            return raw;
        };
        let shielded = KING_STEPS.iter().any(|&(df, dr)| {
            square(file_of(sq) + df, rank_of(sq) + dr)
                .and_then(|s| self.board[s as usize])
                .is_some_and(|p| p.bastion && p.color == target.color)
        });
        if shielded {
            raw * balance().param(Param::BastionPercent) / 100
        } else {
            raw
        }
    }

    /// «Они множатся»: фигура на `sq` делится — у обеих доля здоровья (не меньше 1),
    /// вторая встаёт на `spawn_square`. Возвращает клетку второй.
    fn multiply(&mut self, sq: Square) -> Option<Square> {
        let spawn = self.spawn_square(sq)?;
        let piece = self.board[sq as usize].as_mut()?;
        piece.hp = (piece.hp * balance().param(Param::MultiplyPercent) / 100).max(1);
        let twin = *piece;
        self.board[spawn as usize] = Some(twin);
        if self.on_ice(spawn) {
            self.freeze_at(spawn);
        }
        Some(spawn)
    }

    /// «Массовое жертвоприношение»: свои фигуры в квадрате теряют долю здоровья, за
    /// каждую — очки жертвы по её цене.
    fn mass_sacrifice(&mut self, aim: Square) -> Vec<WaveHit> {
        let side = self.side;
        let mut hits = Vec::new();
        for sq in Area::Square.squares(aim) {
            let Some(target) = self.board[sq as usize].filter(|p| p.color == side) else {
                continue;
            };
            let damage = target.hp * balance().param(Param::MassSacrificePercent) / 100;
            if let Some(piece) = &mut self.board[sq as usize] {
                piece.hp -= damage;
            }
            self.give_sacrifice(side, target.kind.points());
            hits.push(WaveHit {
                square: sq,
                target,
                damage,
                killed: false,
            });
        }
        hits
    }

    /// Свободные (не замёрзшие навсегда) фигуры соперника, кроме короля, — из них
    /// «Абсолютный Ноль» замораживает половину.
    fn zero_targets(&self) -> Vec<Square> {
        let them = self.side.opponent();
        self.pieces()
            .filter(|&(_, p)| p.color == them && p.kind != Kind::King && p.frozen != FOREVER)
            .map(|(sq, _)| sq)
            .collect()
    }

    /// «Абсолютный Ноль»: половина свободных фигур соперника (с округлением вверх, короли не
    /// замерзают) замерзает навсегда. Какие — решает перемешивание по ключу позиции: так оно
    /// одно и то же при отмене и в переборе компьютера. Возвращает замёрзшие клетки.
    fn absolute_zero(&mut self) -> Vec<Square> {
        let them = self.side.opponent();
        let free = self
            .pieces()
            .filter(|&(_, p)| p.color == them && p.frozen != FOREVER)
            .count();
        let mut targets = self.zero_targets();
        let mut state = self.key() | 1;
        for i in (1..targets.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            targets.swap(i, (state % (i as u64 + 1)) as usize);
        }
        let share = balance().param(Param::AbsoluteZeroPercent) as usize;
        targets.truncate((free * share).div_ceil(100));
        targets.sort_unstable();
        for &sq in &targets {
            if let Some(piece) = &mut self.board[sq as usize] {
                piece.frozen = FOREVER;
            }
        }
        targets
    }

    /// Замораживает фигуру на `sq`, кроме короля, на две границы ходов её стороны (начало и
    /// конец хода — `end_turn`): чужая пропустит свой ближайший ход и оттает в его конце, своя
    /// (заморожена в свой ход) стоит до конца хода и оттает в начале следующего своего хода.
    /// `false` — замораживать некого.
    fn freeze_at(&mut self, sq: Square) -> bool {
        match &mut self.board[sq as usize] {
            Some(piece) if piece.kind != Kind::King => {
                piece.frozen = piece.frozen.max(2);
                true
            }
            _ => false,
        }
    }

    /// Буря, лавина или ледяной барьер, нацеленные на `aim`: фигуры на клетках карты
    /// замерзают (кроме королей); барьер ещё и оставляет на них ледяную зону до начала
    /// следующего хода своей стороны. Возвращает замёрзшие клетки.
    fn freeze_area(&mut self, area: Area, aim: Square, zone: bool) -> Vec<Square> {
        let squares = area.squares(aim);
        if zone {
            self.ice[self.side as usize] |= area.mask(aim);
        }
        squares
            .into_iter()
            .filter(|&sq| self.freeze_at(sq))
            .collect()
    }

    /// Ледяная стена: горизонталь станет ледяной зоной на два хода — этот ход соперника и
    /// следующий (до начала хода своей стороны после следующего). Стоящие на ней не мёрзнут.
    fn ice_wall(&mut self, aim: Square) {
        self.ice_later[self.side as usize] |= Area::Rank.mask(aim);
    }

    /// Кого и насколько заденет огненная волна вокруг клетки `center` — всех соседей,
    /// королей тоже.
    pub fn wave_hits(&self, center: Square) -> Vec<WaveHit> {
        KING_STEPS
            .iter()
            .filter_map(|&(df, dr)| square(file_of(center) + df, rank_of(center) + dr))
            .filter_map(|sq| {
                let target = self.board[sq as usize]?;
                let damage = self.damage_at(sq, balance().param(Param::WaveDamage));
                Some(WaveHit {
                    square: sq,
                    target,
                    damage,
                    killed: damage >= target.hp,
                })
            })
            .collect()
    }

    /// Сколько фигур соперника стороны, которая ходит, сейчас заморожено.
    pub fn frozen_enemies(&self) -> u8 {
        let them = self.side.opponent();
        self.pieces()
            .filter(|&(_, p)| p.color == them && p.is_frozen())
            .count() as u8
    }

    /// Сколько фигур соперника убила сторона `color` за партию («Истории Побед»).
    pub fn kills(&self, color: Color) -> u8 {
        self.kills[color as usize]
    }

    /// Барьер стороны `color` (клетки маской); пусто — барьера нет.
    pub fn barrier(&self, color: Color) -> u64 {
        self.barrier[color as usize]
    }

    /// Урон `raw` картой фигуре на `sq` (с обороной ладьи); за погибшую — очки, как за
    /// удар стороны, которая ходит.
    fn blast(&mut self, sq: Square, raw: i32) -> Option<WaveHit> {
        let target = self.board[sq as usize]?;
        let damage = self.damage_at(sq, raw);
        let (_, killed) = self.wound(sq, damage);
        if killed {
            self.score_kill(target, self.side);
        }
        self.halfmove_clock = 0;
        Some(WaveHit {
            square: sq,
            target,
            damage,
            killed,
        })
    }

    /// Клетки на расстоянии от 1 до радиуса взрыва от `center` (по Чебышёву).
    pub fn blast_squares(center: Square) -> Vec<Square> {
        let r = balance().param(Param::IceBlastRadius) as i8;
        (-r..=r)
            .flat_map(|dr| (-r..=r).map(move |df| (df, dr)))
            .filter(|&d| d != (0, 0))
            .filter_map(|(df, dr)| square(file_of(center) + df, rank_of(center) + dr))
            .collect()
    }

    /// «Ледяной взрыв»: замёрзшая фигура соперника на `center` бьёт своей атакой все фигуры
    /// вокруг на две клетки — и чужие, и свои, королей тоже; сама не страдает.
    fn ice_blast(&mut self, center: Square) -> Vec<WaveHit> {
        let Some(source) = self.piece(center) else {
            return Vec::new();
        };
        let hits = Self::blast_squares(center)
            .into_iter()
            .filter_map(|sq| self.blast(sq, source.attack()))
            .collect();
        hits
    }

    fn burn_wave(&mut self, center: Square) -> Vec<WaveHit> {
        let hits = self.wave_hits(center);
        for hit in &hits {
            self.wound(hit.square, hit.damage);
            if hit.killed {
                self.score_kill(hit.target, self.side);
            }
        }
        if !hits.is_empty() {
            self.halfmove_clock = 0;
        }
        hits
    }

    /// Сторона, которая ходит, разыгрывает карту `index` из руки — карта сгорает. Карте
    /// с выбором нужны клетки (`picks`): центр огненной волны из `card_targets`, две
    /// фигуры обмена (вторая — из `swap_partners` первой), клетка для карты на поле. Ход
    /// при этом не кончается.
    pub fn play_card(&mut self, index: usize, targets: &[Square]) -> Option<CardPlay> {
        let kind = *self.cards[self.side as usize].hand().get(index)?;
        if !self.can_play_card() || !self.can_afford(kind) || targets.len() != kind.picks() {
            return None;
        }
        let allowed = match (kind, targets) {
            (CardKind::Swap, &[a, b]) => self.swap_is_safe(a, b),
            (_, &[aim]) if kind.area().is_some() => aim < 64,
            (_, &[sq]) => self.piece_target(kind, sq),
            (CardKind::Encircle, []) => self.encircles(self.side),
            (CardKind::BloodGod, []) => !self.blood_god[self.side as usize],
            (CardKind::AbsoluteZero, []) => !self.zero_targets().is_empty(),
            (CardKind::ColdRuler | CardKind::HardWinter, []) => self.frozen_enemies() > 0,
            (CardKind::VictoryTales, []) => self.kills[self.side as usize] > 0,
            (_, []) => true,
            _ => false,
        };
        if !allowed {
            return None;
        }
        self.cards[self.side as usize].take(index);
        self.cards_played += 1;
        if let Some((points, n)) = kind.cost() {
            self.points[self.side as usize][points as usize] -= n;
        }
        let mut drawn = 0;
        let mut heals = Vec::new();
        let mut frozen = Vec::new();
        let mut spawned = None;
        let hits = match (kind, targets) {
            (CardKind::FireWave, &[center]) => self.burn_wave(center),
            (CardKind::Swap, &[a, b]) => {
                frozen = self.swap_pieces(a, b);
                Vec::new()
            }
            (CardKind::Heal, &[center]) => {
                heals = self.heal_around(center);
                Vec::new()
            }
            (CardKind::Funding, []) => {
                for _ in 0..balance().param(Param::FundingDraw) {
                    drawn += u8::from(self.cards[self.side as usize].draw());
                }
                Vec::new()
            }
            (CardKind::Offering, &[sq]) => {
                // Своя фигура уходит с доски — за неё в несколько раз больше очков жертвы.
                if let Some(piece) = self.piece(sq) {
                    self.fall(sq, piece);
                    let times = balance().param(Param::OfferingMultiplier) as u8;
                    self.give_sacrifice(piece.color, times.saturating_mul(piece.kind.points()));
                }
                Vec::new()
            }
            (CardKind::Multiply, &[sq]) => {
                if let Some(spawn) = self.multiply(sq) {
                    spawned = Some(spawn);
                }
                self.returning[self.side as usize] += 1;
                Vec::new()
            }
            (CardKind::Champion, &[sq]) => {
                if let Some(piece) = &mut self.board[sq as usize] {
                    piece.champion = true;
                    piece.boost += 1;
                    let grown = piece.hp * (100 + balance().param(Param::ChampionPercent)) / 100;
                    piece.hp = grown.min(piece.max_hp());
                }
                Vec::new()
            }
            (CardKind::Bastion, &[sq]) => {
                if let Some(piece) = &mut self.board[sq as usize] {
                    piece.bastion = true;
                }
                Vec::new()
            }
            (CardKind::MassSacrifice, &[aim]) => self.mass_sacrifice(aim),
            (CardKind::IceWall, &[aim]) => {
                self.ice_wall(aim);
                Vec::new()
            }
            (CardKind::Barrier, &[aim]) => {
                self.barrier[self.side as usize] |= Area::Square.mask(aim);
                Vec::new()
            }
            (CardKind::IceShard, &[sq]) => {
                // Снаряд бьёт, выжившая цель замерзает (король — нет).
                let hit = self.blast(sq, balance().param(Param::IceShardDamage));
                if hit.is_some_and(|h| !h.killed) && self.freeze_at(sq) {
                    frozen.push(sq);
                }
                hit.into_iter().collect()
            }
            (CardKind::IceBlast, &[center]) => self.ice_blast(center),
            (CardKind::IceWarrior, &[sq]) => {
                if let Some(piece) = &mut self.board[sq as usize] {
                    piece.frost_touch = true;
                }
                Vec::new()
            }
            (CardKind::NorthWarrior, &[sq]) => {
                let (attack, hp) = north_bonus(self.frozen_enemies());
                if let Some(piece) = &mut self.board[sq as usize] {
                    piece.bonus_attack = piece.bonus_attack.saturating_add(attack);
                    piece.bonus_hp = piece.bonus_hp.saturating_add(hp);
                    piece.hp += i32::from(hp);
                }
                Vec::new()
            }
            (_, &[aim]) => {
                if let Some(area) = kind.area() {
                    frozen = self.freeze_area(area, aim, kind == CardKind::IceBarrier);
                }
                Vec::new()
            }
            (CardKind::Encircle, []) => {
                self.encircled_by = Some(self.side);
                Vec::new()
            }
            (CardKind::AbsoluteZero, []) => {
                frozen = self.absolute_zero();
                Vec::new()
            }
            (CardKind::BloodGod, []) => {
                self.blood_god[self.side as usize] = true;
                Vec::new()
            }
            (CardKind::ColdRuler, []) => {
                let mana = self.frozen_enemies().saturating_mul(cold_ruler_mana());
                self.add_points(self.side, Points::Mana, mana);
                Vec::new()
            }
            (CardKind::VictoryTales, []) => {
                let mana = self.kills[self.side as usize].saturating_mul(victory_tales_mana());
                self.add_points(self.side, Points::Mana, mana);
                Vec::new()
            }
            (CardKind::HardWinter, []) => {
                let them = self.side.opponent();
                let targets: Vec<Square> = self
                    .pieces()
                    .filter(|&(_, p)| p.color == them && p.is_frozen())
                    .map(|(sq, _)| sq)
                    .collect();
                let hits = targets
                    .into_iter()
                    .filter_map(|sq| self.blast(sq, balance().param(Param::HardWinterDamage)))
                    .collect();
                hits
            }
            _ => Vec::new(),
        };
        Some(CardPlay {
            color: self.side,
            kind,
            index,
            targets: targets.to_vec(),
            hits,
            heals,
            drawn,
            frozen,
            spawned,
        })
    }

    /// Сторона заканчивает ход: её свободные фигуры бьют (`strike_all`), насыщенные лавой
    /// пешки взрываются (`lava_blasts`), замороженные отсчитывают пропущенный ход, ходит
    /// соперник — его ледяные зоны тают (продержались ход), и начинает он с карты из колоды;
    /// у усиленной стороны к ходу соперника насыщаются лавой новые пешки.
    pub fn end_turn(&mut self) -> TurnEnd {
        let us = self.side;
        let strikes = self.strike_all();
        let blasts = self.lava_blasts();
        // Rogue-like: фигуры волн сами не ходят — в конце хода сначала бьют, потом шагают
        // вперёд; новая волна выходит после шага и в этот ход не бьёт.
        let marched = self.march();
        // Rogue-like: в конце хода соперника на его крайнюю горизонталь выходит новая волна.
        let spawned = if self.rogue.is_some_and(|r| r.player != us) {
            self.spawn_wave()
        } else {
            Vec::new()
        };
        self.thaw(us);
        // Сыгранные «Они множатся» возвращаются в руку (в полной руке — наверх колоды).
        for _ in 0..std::mem::take(&mut self.returning[us as usize]) {
            self.cards[us as usize].give_back(CardKind::Multiply);
        }
        if us == Color::Black {
            self.fullmove += 1;
        }
        for piece in self.board.iter_mut().flatten() {
            if piece.color == us {
                piece.fresh = false;
            }
        }
        self.side = us.opponent();
        self.thaw(self.side);

        self.tick_reserve(self.side);
        // Зоны новой стороны продержались ход соперника; ледяная стена — ещё на ход.
        self.ice[self.side as usize] = std::mem::take(&mut self.ice_later[self.side as usize]);
        self.barrier[self.side as usize] = 0;
        self.moved = false;
        self.moves_made = 0;
        self.used = 0;
        self.cards_played = 0;
        self.draw_card(self.side);
        self.gain_mana();
        let charged = if self.boosted == Some(us) {
            self.charge_lava(us)
        } else {
            Vec::new()
        };
        TurnEnd {
            strikes,
            blasts,
            charged,
            spawned,
            marched,
        }
    }

    /// Rogue-like: все свободные (не замёрзшие) фигуры волн стороны, которая ходит, шагают
    /// на клетку вперёд — к игроку, освобождая дальние горизонтали под следующую волну.
    /// Первыми шагают передние, за ними задние встают на освободившиеся клетки; занятая
    /// клетка, барьер или край доски — фигура стоит. Вставшая в ледяную зону замерзает.
    fn march(&mut self) -> Vec<(Square, Square)> {
        let Some(r) = self.rogue else {
            return Vec::new();
        };
        let us = self.side;
        if us == r.player {
            return Vec::new();
        }
        let wall = self.wall(us);
        let dir = us.forward();
        // Передние — ближе к игроку: у чёрных — с младших горизонталей.
        let ranks: Vec<i8> = match us {
            Color::Black => (0..8).collect(),
            Color::White => (0..8).rev().collect(),
        };
        let mut marched = Vec::new();
        for rank in ranks {
            for file in 0..8 {
                let Some(from) = square(file, rank) else {
                    continue;
                };
                let Some(piece) = self.board[from as usize].filter(|p| p.color == us) else {
                    continue;
                };
                if piece.is_frozen() {
                    continue;
                }
                let Some(to) = square(file, rank + dir).filter(|&to| self.open(to, wall)) else {
                    continue;
                };
                self.relocate(from, to);
                if piece.kind == Kind::King {
                    self.kings[us as usize] = Some(to);
                }
                if self.on_ice(to) {
                    self.freeze_at(to);
                }
                marched.push((from, to));
            }
        }
        marched
    }

    /// Усиливает фигуры стороны `color` (компьютер): пешки насыщаются лавой в начале каждого
    /// хода соперника (если ходит соперник — сразу). Электричества слонов с 0.41 нет (просьба
    /// пользователя: слишком сильно) — `spark` остался для тестов.
    pub fn boost(&mut self, color: Color) {
        self.boosted = Some(color);
        if self.side != color {
            self.charge_lava(color);
        }
    }

    /// Сторона с усиленными фигурами.
    pub fn boosted(&self) -> Option<Color> {
        self.boosted
    }

    /// Случайные пешки стороны `color` (их `Rule::LavaPawns`) насыщаются лавой. Выбор —
    /// перемешивание по ключу позиции: так он один и тот же при отмене и в переборе.
    fn charge_lava(&mut self, color: Color) -> Vec<Square> {
        let mut pawns: Vec<Square> = self
            .pieces()
            .filter(|&(_, p)| p.color == color && p.kind == Kind::Pawn)
            .map(|(sq, _)| sq)
            .collect();
        let mut state = self.key() | 1;
        for i in (1..pawns.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            pawns.swap(i, (state % (i as u64 + 1)) as usize);
        }
        pawns.truncate(balance().rule(Rule::LavaPawns).max(0) as usize);
        pawns.sort_unstable();
        for &sq in &pawns {
            if let Some(piece) = &mut self.board[sq as usize] {
                piece.lava = true;
            }
        }
        pawns
    }

    /// Насыщенные лавой пешки стороны, которая ходит, взрываются по очереди ударов: огненная
    /// волна (`Rule::LavaDamage`) по чужим фигурам на 8 клетках вокруг, кроме барьера
    /// соперника; сама пешка цела. Замёрзшая не взрывается; лава уходит у всех.
    fn lava_blasts(&mut self) -> Vec<Blast> {
        let us = self.side;
        let wall = self.wall(us);
        let raw = balance().rule(Rule::LavaDamage);
        let mut blasts = Vec::new();
        for center in strike_queue() {
            let Some(pawn) = self.board[center as usize].filter(|p| p.color == us && p.lava) else {
                continue;
            };
            if let Some(piece) = &mut self.board[center as usize] {
                piece.lava = false;
            }
            if pawn.is_frozen() {
                continue;
            }
            let mut hits = Vec::new();
            for &(df, dr) in &KING_STEPS {
                let Some(sq) = square(file_of(center) + df, rank_of(center) + dr) else {
                    continue;
                };
                let Some(target) = self.board[sq as usize].filter(|p| p.color != us) else {
                    continue;
                };
                if wall & 1 << sq != 0 {
                    continue;
                }
                let damage = self.damage_at(sq, raw);
                let (_, killed) = self.wound(sq, damage);
                if killed {
                    self.score_kill(target, us);
                }
                hits.push(WaveHit {
                    square: sq,
                    target,
                    damage,
                    killed,
                });
            }
            if !hits.is_empty() {
                self.halfmove_clock = 0;
            }
            blasts.push(Blast { center, hits });
        }
        blasts
    }

    /// Граница хода стороны `color` (конец или начало): её замёрзшие фигуры на шаг ближе к
    /// оттепели; замёрзшие навсегда — нет.
    fn thaw(&mut self, color: Color) {
        for piece in self.board.iter_mut().flatten() {
            if piece.color == color && piece.frozen > 0 && piece.frozen != FOREVER {
                piece.frozen -= 1;
            }
        }
    }

    /// Удары в конце хода (с 0.40): каждая свободная (не замёрзшая) фигура стороны, которая
    /// ходит, по очереди a8, a7, …, a1, b8, …, h1 бьёт всех чужих в досягаемости
    /// (`strike_steps`) — сначала самую дорогую (`Kind::points`), из равных — с меньшим
    /// здоровьем, потом первую по очереди — и остаётся на месте. Фигуры в барьере соперника
    /// недосягаемы. Убитая цель даёт очки, «воин» после убийства лечится целиком и бьёт
    /// сильнее, удар «Воина из Льда» замораживает выжившую цель.
    ///
    /// В партии «с перемещением» (с 0.46): убив цель вблизи (`close_strike`), фигура встаёт на
    /// её клетку и остальных в этот ход не бьёт (`advance_to`); второй раз она не бьёт и с
    /// новой клетки.
    fn strike_all(&mut self) -> Vec<Strike> {
        let us = self.side;
        let wall = self.wall(us);
        let mut strikes = Vec::new();
        // Клетки, куда встали убийцы: второй раз они в этот ход не бьют.
        let mut done = 0u64;
        for from in strike_queue() {
            if done & 1 << from != 0 {
                continue;
            }
            let Some(mut attacker) =
                self.board[from as usize].filter(|p| p.color == us && !p.is_frozen() && !p.fresh)
            else {
                continue;
            };
            let mut targets: Vec<(Square, Piece)> = strike_steps(attacker)
                .iter()
                .filter(|&&step| self.line_clear(from, attacker, step, wall))
                .filter_map(|&(df, dr)| square(file_of(from) + df, rank_of(from) + dr))
                .filter(|&sq| wall & 1 << sq == 0)
                .filter_map(|sq| Some((sq, self.board[sq as usize].filter(|p| p.color != us)?)))
                .collect();
            targets.sort_by_key(|&(sq, p)| {
                (std::cmp::Reverse(p.kind.points()), p.hp, strike_order(sq))
            });
            for (sq, _) in targets {
                // Цель могла погибнуть от молнии предыдущего удара.
                let Some(target) = self.board[sq as usize].filter(|p| p.color != us) else {
                    continue;
                };
                if let Some(me) = self.board[from as usize] {
                    attacker = me;
                }
                let mut strike = self.strike(from, attacker, sq, target, wall);
                let advance = self.advance
                    && strike.hit.killed
                    && close_strike(attacker.kind, from, sq)
                    && self.board[sq as usize].is_none();
                if advance {
                    self.advance_to(from, &mut strike);
                    done |= 1 << sq;
                }
                strikes.push(strike);
                if advance {
                    break;
                }
            }
        }
        if !strikes.is_empty() {
            self.halfmove_clock = 0;
        }
        strikes
    }

    /// Перемещение при взятии: убийца встаёт на клетку цели; пешка на последней горизонтали
    /// становится ферзём, в ледяной зоне фигура замерзает.
    fn advance_to(&mut self, from: Square, strike: &mut Strike) {
        let Some(piece) = self.board[from as usize] else {
            return;
        };
        let landing = strike.hit.square;
        let us = piece.color;
        self.relocate(from, landing);
        self.castling &= !rights_lost_at(from);
        if piece.kind == Kind::King {
            self.kings[us as usize] = Some(landing);
        }
        if piece.kind == Kind::Pawn && rank_of(landing) == us.last_rank() {
            self.board[landing as usize] = Some(Piece::new(Kind::Queen, us));
            strike.promoted = Some(Kind::Queen);
        }
        if self.on_ice(landing) {
            self.freeze_at(landing);
        }
        strike.advanced = true;
    }

    /// Один удар фигуры `attacker` с клетки `from` по `target` на `sq`.
    fn strike(
        &mut self,
        from: Square,
        attacker: Piece,
        sq: Square,
        target: Piece,
        wall: u64,
    ) -> Strike {
        let us = attacker.color;
        let promoted = None;
        let mut feast = 0;
        let mut chilled = false;
        let damage = self.damage_at(sq, attacker.attack());
        let (_, killed) = self.wound(sq, damage);
        if killed {
            self.score_kill(target, us);
            if let Some(winner) = &mut self.board[from as usize]
                && winner.champion
            {
                feast = winner.max_hp() - winner.hp;
                winner.hp = winner.max_hp();
                winner.boost = winner.boost.saturating_add(1);
            }
        } else if attacker.frost_touch {
            chilled = self.freeze_at(sq);
        }
        // Слон с электричеством: удар расходится по диагонали от цели на чужие фигуры.
        let mut splash = [None; 3];
        if attacker.spark {
            let spread = attacker.attack() * balance().rule(Rule::SparkPercent) / 100;
            let around: Vec<Square> = BISHOP_RAYS
                .iter()
                .filter_map(|&(df, dr)| square(file_of(sq) + df, rank_of(sq) + dr))
                .filter(|&s| wall & 1 << s == 0)
                .filter(|&s| self.board[s as usize].is_some_and(|p| p.color != us))
                .collect();
            for (slot, s) in splash.iter_mut().zip(around) {
                let victim = self.board[s as usize].expect("чужая фигура");
                let damage = self.damage_at(s, spread);
                let (_, dead) = self.wound(s, damage);
                if dead {
                    self.score_kill(victim, us);
                }
                *slot = Some(Hit {
                    square: s,
                    target: victim,
                    damage,
                    killed: dead,
                });
            }
        }
        Strike {
            from,
            attacker,
            promoted,
            splash,
            hit: Hit {
                square: sq,
                target,
                damage,
                killed,
            },
            feast,
            chilled,
            advanced: false,
        }
    }

    /// Сторона берёт карту из колоды, если колода не пуста и в руке есть место.
    fn draw_card(&mut self, color: Color) {
        self.cards[color as usize].draw();
    }

    pub fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    pub fn piece(&self, sq: Square) -> Option<Piece> {
        self.board[sq as usize]
    }

    /// Где стоит король стороны `color`; `None` — король погиб.
    pub fn king_square(&self, color: Color) -> Option<Square> {
        self.kings[color as usize]
    }

    /// Остались ли у стороны `color` свободные фигуры: без них (фигур нет или все замёрзли
    /// навсегда — «Абсолютный Ноль») сторона проиграла.
    pub fn has_pieces(&self, color: Color) -> bool {
        self.board
            .iter()
            .flatten()
            .any(|p| p.color == color && p.frozen != FOREVER)
    }

    pub fn pieces(&self) -> impl Iterator<Item = (Square, Piece)> + '_ {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(sq, piece)| piece.map(|p| (sq as Square, p)))
    }

    #[cfg(test)]
    pub(crate) fn set_hp(&mut self, sq: Square, hp: i32) {
        if let Some(piece) = &mut self.board[sq as usize] {
            piece.hp = hp;
        }
    }

    /// Насыщает слона на `sq` электричеством (для тестов).
    #[cfg(test)]
    pub(crate) fn set_spark(&mut self, sq: Square) {
        if let Some(piece) = &mut self.board[sq as usize] {
            piece.spark = true;
        }
    }

    /// Дальний удар слона, ладьи и ферзя (с 0.42) бьёт только первую фигуру на линии: клетки
    /// между `from` и целью (`step`) должны быть пусты и не в барьере. У остальных фигур удар
    /// на одну клетку — линия всегда свободна.
    fn line_clear(&self, from: Square, piece: Piece, step: (i8, i8), wall: u64) -> bool {
        if !matches!(piece.kind, Kind::Bishop | Kind::Rook | Kind::Queen) {
            return true;
        }
        let k = step.0.abs().max(step.1.abs());
        let unit = (step.0 / k, step.1 / k);
        (1..k).all(|i| {
            let Some(sq) = square(file_of(from) + unit.0 * i, rank_of(from) + unit.1 * i) else {
                return false;
            };
            wall & 1 << sq == 0 && self.board[sq as usize].is_none()
        })
    }

    /// Насыщает пешку на `sq` лавой (для тестов).
    #[cfg(test)]
    pub(crate) fn set_lava(&mut self, sq: Square) {
        if let Some(piece) = &mut self.board[sq as usize] {
            piece.lava = true;
        }
    }

    /// Замораживает фигуру на `sq` на `turns` ходов её стороны (для тестов).
    #[cfg(test)]
    pub(crate) fn set_frozen(&mut self, sq: Square, turns: u8) {
        if let Some(piece) = &mut self.board[sq as usize] {
            piece.frozen = turns;
        }
    }

    /// Даёт стороне очки (для тестов).
    #[cfg(test)]
    pub(crate) fn give_points(&mut self, color: Color, kind: Points, n: u8) {
        self.points[color as usize][kind as usize] += n;
    }

    /// Кладёт карту в руку стороны, если есть место (для тестов).
    #[cfg(test)]
    pub(crate) fn give_card(&mut self, color: Color, kind: CardKind) {
        let cards = &mut self.cards[color as usize];
        if cards.hand_len < balance().max_hand() {
            cards.hand[cards.hand_len as usize] = kind;
            cards.hand_len += 1;
        }
    }

    /// Ключ позиции для правила троекратного повторения (здоровье фигур тоже учитывается).
    pub fn key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.board.hash(&mut hasher);
        self.side.hash(&mut hasher);
        self.castling.hash(&mut hasher);
        self.ice.hash(&mut hasher);
        self.ice_later.hash(&mut hasher);
        self.barrier.hash(&mut hasher);

        self.reserve.hash(&mut hasher);
        self.rogue.hash(&mut hasher);
        self.used.hash(&mut hasher);
        hasher.finish()
    }

    /// Шах: в конце следующего хода соперника его фигуры могут убить короля стороны
    /// `color`, если тот останется на месте. Без короля шаха нет.
    pub fn in_check(&self, color: Color) -> bool {
        self.king_square(color).is_some_and(|sq| {
            self.board[sq as usize].is_some_and(|king| self.can_kill(sq, king, color.opponent()))
        })
    }

    /// Сколько урона король стороны `color` может получить от одного удара соперника (0 —
    /// его никто не достаёт или короля нет).
    pub fn king_danger(&self, color: Color) -> i32 {
        self.king_square(color)
            .map_or(0, |sq| self.max_damage(sq, color.opponent()))
    }

    /// Может ли фигура стороны `by` одним ударом убить фигуру `victim`, стоящую на `sq`.
    pub fn can_kill(&self, sq: Square, victim: Piece, by: Color) -> bool {
        self.any_attacker(sq, by, |_, attacker| {
            self.damage_at(sq, attacker.attack()) >= victim.hp
        })
    }

    /// Наибольший урон, который одна фигура стороны `by` может нанести фигуре на `sq`.
    pub fn max_damage(&self, sq: Square, by: Color) -> i32 {
        let mut best = 0;
        self.any_attacker(sq, by, |_, attacker| {
            best = best.max(self.damage_at(sq, attacker.attack()));
            false
        });
        best
    }

    /// Перебирает свободные фигуры стороны `by`, которые достают ударом до клетки `sq`
    /// (`strike_steps`), пока `test` не вернёт `true`. Клетки барьера соперника стороны `by`
    /// недосягаемы.
    fn any_attacker(
        &self,
        sq: Square,
        by: Color,
        mut test: impl FnMut(Square, Piece) -> bool,
    ) -> bool {
        if self.wall(by) & 1 << sq != 0 {
            return false;
        }
        let (file, rank) = (file_of(sq), rank_of(sq));
        // Удар с клетки `s` шагом `step` попадает в `sq`, значит `s` = `sq` − `step`.

        // Все удары — в пределах двух клеток: ферзь покрывает прямые и диагонали, конь — «Г».
        QUEEN_STRIKES.iter().chain(&KNIGHT_JUMPS).any(|&step| {
            let Some(s) = square(file - step.0, rank - step.1) else {
                return false;
            };
            match self.board[s as usize] {
                Some(p)
                    if p.color == by
                        && !p.is_frozen()
                        && strike_steps(p).contains(&step)
                        && self.line_clear(s, p, step, self.wall(by)) =>
                {
                    test(s, p)
                }
                _ => false,
            }
        })
    }

    /// Ходы фигурой в `out` — те же, что `legal_moves`, но без своего вектора (для
    /// перебора). Сходившая сторона фигурой больше не ходит, замороженные фигуры — тоже.
    pub fn pseudo_moves(&self, out: &mut Vec<Move>) {
        if self.moves_made >= self.moves_per_turn() {
            return;
        }
        // Rogue-like: фигуры волн сами не ходят — они шагают вперёд в конце хода (`march`).
        if self.rogue.is_some_and(|r| r.player != self.side) {
            return;
        }
        let us = self.side;
        for from in 0..64u8 {
            let Some(piece) = self.board[from as usize] else {
                continue;
            };
            if piece.color != us || piece.is_frozen() || self.used(from) {
                continue;
            }
            match piece.kind {
                Kind::Pawn => self.pawn_moves(from, us, out),
                Kind::Knight => self.step_moves(from, us, &KNIGHT_JUMPS, out),
                Kind::Bishop => self.slide_moves(from, us, &BISHOP_RAYS, out),
                Kind::Rook => self.slide_moves(from, us, &ROOK_RAYS, out),
                Kind::Queen => {
                    self.slide_moves(from, us, &ROOK_RAYS, out);
                    self.slide_moves(from, us, &BISHOP_RAYS, out);
                }
                Kind::King => {
                    self.step_moves(from, us, &KING_STEPS, out);
                    self.castling_moves(from, out);
                }
            }
        }
        // Выставление готовой фигуры из запаса — вместо хода.
        for kind in [
            Kind::Pawn,
            Kind::Knight,
            Kind::Bishop,
            Kind::Rook,
            Kind::Queen,
        ] {
            if self.reserve[us as usize][kind as usize][0]
                .iter()
                .any(|&n| n > 0)
            {
                out.extend(
                    self.drop_squares(us, kind)
                        .map(|sq| Move::new(sq, sq, MoveKind::Drop(kind))),
                );
            }
        }
    }

    /// Ходы фигурой — только на пустые клетки. Своего короля можно оставить под ударом:
    /// он гибнет, как любая фигура.
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        self.pseudo_moves(&mut moves);
        moves
    }

    /// Клетки барьера соперника стороны `us`: туда её фигуры не входят, через них не ходят
    /// и туда не бьют.
    fn wall(&self, us: Color) -> u64 {
        self.barrier[us.opponent() as usize]
    }

    /// Запас стороны `color`: по видам — (готовы выставить, ждут).
    pub fn reserve(&self, color: Color) -> Vec<(Kind, u8, u8)> {
        [
            Kind::Queen,
            Kind::Rook,
            Kind::Bishop,
            Kind::Knight,
            Kind::Pawn,
        ]
        .into_iter()
        .filter_map(|kind| {
            let [ready, w1, w2] =
                self.reserve[color as usize][kind as usize].map(|r| r.iter().sum::<u8>());
            (ready + w1 + w2 > 0).then_some((kind, ready, w1 + w2))
        })
        .collect()
    }

    /// Начало хода стороны `color`: взятые фигуры на ход ближе к выставлению.
    fn tick_reserve(&mut self, color: Color) {
        for counts in &mut self.reserve[color as usize] {
            for r in 0..MAX_REBIRTHS as usize {
                counts[0][r] += counts[1][r];
                counts[1][r] = counts[2][r];
                counts[2][r] = 0;
            }
        }
    }

    /// Клетки, куда сторона `us` может выставить фигуру вида `kind`: пустые на своей половине
    /// вне барьера соперника; пешку — не на первую горизонталь.
    fn drop_squares(&self, us: Color, kind: Kind) -> impl Iterator<Item = Square> + '_ {
        let wall = self.wall(us);
        (0..64u8).filter(move |&sq| {
            let rank = rank_of(sq);
            let own_half = match us {
                Color::White => rank < 4,
                Color::Black => rank >= 4,
            };
            own_half && self.open(sq, wall) && !(kind == Kind::Pawn && rank == us.home_rank())
        })
    }

    /// Можно ли встать на клетку `to` (или пройти через неё): она пуста и не в барьере.
    fn open(&self, to: Square, wall: u64) -> bool {
        self.board[to as usize].is_none() && wall & 1 << to == 0
    }

    fn step_moves(&self, from: Square, us: Color, steps: &[(i8, i8)], out: &mut Vec<Move>) {
        let wall = self.wall(us);
        for &(df, dr) in steps {
            if let Some(to) = square(file_of(from) + df, rank_of(from) + dr)
                && self.open(to, wall)
            {
                out.push(Move::new(from, to, MoveKind::Quiet));
            }
        }
    }

    /// Ходы по лучам до первой фигуры или барьера.
    fn slide_moves(&self, from: Square, us: Color, rays: &[(i8, i8)], out: &mut Vec<Move>) {
        let wall = self.wall(us);
        for &(df, dr) in rays {
            let (mut x, mut y) = (file_of(from) + df, rank_of(from) + dr);
            while let Some(to) = square(x, y)
                && self.open(to, wall)
            {
                out.push(Move::new(from, to, MoveKind::Quiet));
                x += df;
                y += dr;
            }
        }
    }

    /// Пешка ходит вперёд на одну клетку (с начальной — и на две); дойдя до последней
    /// горизонтали, превращается.
    fn pawn_moves(&self, from: Square, us: Color, out: &mut Vec<Move>) {
        let (file, rank) = (file_of(from), rank_of(from));
        let dir = us.forward();
        let wall = self.wall(us);
        let Some(to) = square(file, rank + dir).filter(|&to| self.open(to, wall)) else {
            return;
        };
        let mv = Move::new(from, to, MoveKind::Quiet);
        if rank + dir == us.last_rank() {
            out.extend(Kind::PROMOTIONS.map(|kind| Move {
                promotion: Some(kind),
                ..mv
            }));
        } else {
            out.push(mv);
        }
        if rank == us.home_rank() + dir
            && let Some(to2) = square(file, rank + 2 * dir)
            && self.open(to2, wall)
        {
            out.push(Move::new(from, to2, MoveKind::Quiet));
        }
    }

    /// Рокировка — как в шахматах: король не под шахом и не проходит через поле, где его
    /// убьют, и не встаёт на такое поле.
    fn castling_moves(&self, from: Square, out: &mut Vec<Move>) {
        let us = self.side;
        let rank = us.home_rank();
        if square(4, rank) != Some(from) || self.in_check(us) {
            return;
        }
        let Some(king) = self.board[from as usize] else {
            return;
        };
        let at = |file: i8| self.board[(rank * 8 + file) as usize];
        // Замёрзшая ладья в рокировке не участвует.
        let own_rook = |file: i8| matches!(at(file), Some(p) if p.kind == Kind::Rook && p.color == us && !p.is_frozen());
        let (short, long) = us.castling_bits();
        // Король и ладья проходят только по пустым клеткам вне барьера соперника.
        let wall = self.wall(us);
        let open = |files: std::ops::RangeInclusive<i8>| {
            files
                .into_iter()
                .all(|f| self.open((rank * 8 + f) as Square, wall))
        };
        if self.castling & short != 0
            && open(5..=6)
            && own_rook(7)
            && !self.deadly_for_king(from, from + 1, king)
            && !self.deadly_for_king(from, from + 2, king)
        {
            out.push(Move::new(from, from + 2, MoveKind::CastleShort));
        }
        if self.castling & long != 0
            && open(1..=3)
            && own_rook(0)
            && !self.deadly_for_king(from, from - 1, king)
            && !self.deadly_for_king(from, from - 2, king)
        {
            out.push(Move::new(from, from - 2, MoveKind::CastleLong));
        }
    }

    /// Убьют ли короля, если он перейдёт с `from` на `to` (проверка поля, через которое идёт рокировка).
    fn deadly_for_king(&self, from: Square, to: Square, king: Piece) -> bool {
        let mut probe = self.clone();
        probe.relocate(from, to);
        probe.can_kill(to, king, king.color.opponent())
    }

    /// Наносит урон фигуре на `sq`: возвращает, сколько урона прошло, и погибла ли она.
    fn wound(&mut self, sq: Square, dealt: i32) -> (i32, bool) {
        let Some(piece) = self.board[sq as usize] else {
            return (0, false);
        };
        let hp = piece.hp - dealt;
        let killed = hp <= 0;
        if killed {
            self.fall(sq, piece);
        } else {
            self.board[sq as usize] = Some(Piece { hp, ..piece });
        }
        (dealt, killed)
    }

    /// Фигура `piece` на `sq` погибла: клетка пустеет, права на рокировку с неё
    /// пропадают, погибшего короля у стороны больше нет.
    fn fall(&mut self, sq: Square, piece: Piece) {
        self.board[sq as usize] = None;
        self.castling &= !rights_lost_at(sq);
        if piece.kind == Kind::King {
            self.kings[piece.color as usize] = None;
            self.king_lost[piece.color as usize] = true;
        }
    }

    /// Король стороны `color` погиб в этой партии — она проиграла.
    pub fn king_lost(&self, color: Color) -> bool {
        self.king_lost[color as usize]
    }

    fn relocate(&mut self, from: Square, to: Square) {
        if from != to {
            self.board[to as usize] = self.board[from as usize].take();
        }
    }

    /// Делает ход (законность не проверяется) и сообщает, что произошло.
    pub fn make_move(&mut self, mv: Move) -> Outcome {
        let us = self.side;
        let piece = match mv.kind {
            MoveKind::Drop(kind) => Piece::new(kind, us),
            _ => self.board[mv.from as usize].expect("ход с пустой клетки"),
        };
        let mut outcome = Outcome {
            mv,
            piece,
            rook: None,
            promoted: None,
            frozen: false,

            reactions: Vec::new(),
        };
        if let MoveKind::Drop(kind) = mv.kind {
            // Первой выходит фигура, что возвращалась реже.
            let ready = &mut self.reserve[us as usize][kind as usize][0];
            let r = ready.iter().position(|&n| n > 0).expect("фигура в запасе");
            ready[r] -= 1;
            let fresh = Piece {
                fresh: true,
                reborn: r as u8 + 1,
                ..Piece::new(kind, us)
            };
            self.board[mv.to as usize] = Some(fresh);
            outcome.piece = fresh;
            outcome.frozen = self.on_ice(mv.to) && self.freeze_at(mv.to);
            self.halfmove_clock = 0;
            self.moved = true;
            self.moves_made += 1;
            self.used |= 1 << mv.to;
            outcome.reactions = self.intercept(mv.to, None);
            return outcome;
        }

        self.relocate(mv.from, mv.to);
        if matches!(mv.kind, MoveKind::CastleShort | MoveKind::CastleLong) {
            let (rook_from, rook_to) = if mv.kind == MoveKind::CastleShort {
                (mv.from + 3, mv.from + 1)
            } else {
                (mv.from - 4, mv.from - 1)
            };
            self.relocate(rook_from, rook_to);
            outcome.rook = Some((rook_from, rook_to));
        }
        // В короля пешка не превращается: его нет среди `PROMOTIONS`.
        if let Some(kind) = mv.promotion
            && Kind::PROMOTIONS.contains(&kind)
            && piece.kind == Kind::Pawn
            && rank_of(mv.to) == us.last_rank()
        {
            // Новая фигура приходит с полным здоровьем.
            self.board[mv.to as usize] = Some(Piece::new(kind, us));
            outcome.promoted = Some(kind);
        }
        if piece.kind == Kind::King {
            self.kings[us as usize] = Some(mv.to);
        }
        // Вставшая в ледяную зону фигура замерзает (король — нет), ладья рокировки тоже.
        outcome.frozen = self.on_ice(mv.to) && self.freeze_at(mv.to);
        if let Some((_, rook)) = outcome.rook
            && self.on_ice(rook)
        {
            self.freeze_at(rook);
        }
        self.castling &= !rights_lost_at(mv.from);
        self.halfmove_clock = if piece.kind == Kind::Pawn {
            0
        } else {
            self.halfmove_clock + 1
        };
        self.moved = true;
        self.moves_made += 1;
        self.used |= 1 << mv.to;
        if let Some((_, rook)) = outcome.rook {
            self.used |= 1 << rook;
        }
        // Двойной ход пешки проходит через клетку — там её бьют на проходе.
        let passed = (piece.kind == Kind::Pawn && rank_of(mv.from).abs_diff(rank_of(mv.to)) == 2)
            .then(|| (mv.from + mv.to) / 2);
        outcome.reactions = self.intercept(mv.to, passed);
        if !outcome.reactions.is_empty() {
            self.halfmove_clock = 0;
        }
        outcome
    }

    /// Перехват: свободные (не замёрзшие и не только что выставленные) чужие кони, до которых
    /// ударом достаёт клетка `sq`, по очереди бьют вставшую на неё фигуру, пока она жива. Пешки
    /// бьют только на проходе (с 0.45): чужую пешку, прошедшую двойным ходом через клетку
    /// `passed` в их досягаемости, — по её новой клетке. В барьер соперника не бьют.
    fn intercept(&mut self, sq: Square, passed: Option<Square>) -> Vec<Strike> {
        let mut strikes = Vec::new();
        let Some(mover) = self.board[sq as usize] else {
            return strikes;
        };

        let them = mover.color.opponent();
        let wall = self.wall(them);
        if wall & 1 << sq != 0 {
            return strikes;
        }
        let mut guards: Vec<Square> = [Kind::Knight, Kind::Pawn]
            .into_iter()
            .flat_map(|kind| {
                strike_steps(Piece::new(kind, them))
                    .iter()
                    .map(move |&step| (kind, step))
            })
            .filter_map(|(kind, (df, dr))| {
                // Конь бьёт по клетке, куда встала фигура; пешка — по пройденной.
                let at = match kind {
                    Kind::Pawn => passed?,
                    _ => sq,
                };
                let s = square(file_of(at) - df, rank_of(at) - dr)?;
                self.board[s as usize]
                    .is_some_and(|p| {
                        p.color == them && p.kind == kind && !p.is_frozen() && !p.fresh
                    })
                    .then_some(s)
            })
            .collect();
        guards.sort_by_key(|&s| strike_order(s));
        for from in guards {
            let (Some(knight), Some(target)) = (self.board[from as usize], self.board[sq as usize])
            else {
                break;
            };
            if target.color == them {
                break;
            }
            strikes.push(self.strike(from, knight, sq, target, wall));
        }
        strikes
    }

    /// Ход фигурой и конец хода — так ходит компьютер в переборе.
    pub fn make_turn(&mut self, mv: Move) -> Outcome {
        let outcome = self.make_move(mv);
        self.end_turn();
        outcome
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ongoing,
    /// У соперника `winner` не осталось ни одной фигуры.
    Victory {
        winner: Color,
    },
    /// `winner` окружил соперника: «Окружены и сломлены».
    Encircled {
        winner: Color,
    },
    FiftyMoves,
    Repetition,
}

/// Что произошло за одно действие стороны.
#[derive(Clone, Debug)]
pub enum Event {
    /// Ход фигурой.
    Move(Outcome),
    /// Сыгранная карта.
    Card(CardPlay),
    /// Сторона закончила ход: удары её фигур, взрывы лавы, новая лава.
    EndTurn(Color, TurnEnd),
}

/// Действие стороны вместе с позицией до него.
#[derive(Clone, Debug)]
pub struct Record {
    pub before: Position,
    pub event: Event,
    /// Король соперника под шахом после хода (уточняется в конце хода).
    pub check: bool,
    /// Сторона, у которой после этого действия не осталось фигур, — партия кончилась.
    pub loser: Option<Color>,
}

/// Запись хода для списка ходов, по частям.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notation {
    /// «e2–e4», «Кb1–c3», «Огненная Волна d4», удары в конце хода — «Кc3⚔d5, Сc4×e6».
    pub text: String,
    /// Урон картой или ударами.
    pub damage: Option<i32>,
    /// Знак шаха или конца партии («#» — у соперника не осталось фигур).
    pub suffix: &'static str,
}

impl Record {
    /// Ход фигурой, если это он.
    pub fn outcome(&self) -> Option<&Outcome> {
        match &self.event {
            Event::Move(outcome) => Some(outcome),
            _ => None,
        }
    }

    /// Удары в конце хода, если это конец хода.
    pub fn strikes(&self) -> &[Strike] {
        match &self.event {
            Event::Move(o) => &o.reactions,
            _ => self.turn_end().map_or(&[], |turn| &turn.strikes),
        }
    }

    /// Конец хода — удары, взрывы лавы, новая лава, — если это он.
    pub fn turn_end(&self) -> Option<&TurnEnd> {
        match &self.event {
            Event::EndTurn(_, turn) => Some(turn),
            _ => None,
        }
    }

    /// Чьё это действие.
    pub fn color(&self) -> Color {
        self.before.side()
    }

    pub fn notation(&self) -> Notation {
        let suffix = if self.loser.is_some() {
            "#"
        } else if self.check {
            "+"
        } else {
            ""
        };
        let (text, dealt) = match &self.event {
            Event::Move(o) => {
                let letter = o.piece.kind.letter();
                let mut text = match o.mv.kind {
                    MoveKind::CastleShort => "0-0".to_owned(),
                    MoveKind::CastleLong => "0-0-0".to_owned(),
                    MoveKind::Quiet => format!(
                        "{letter}{}–{}",
                        square_name(o.mv.from),
                        square_name(o.mv.to)
                    ),
                    MoveKind::Drop(_) => format!("{letter}@{}", square_name(o.mv.to)),
                };
                if let Some(kind) = o.promoted {
                    text.push_str(kind.letter());
                }
                // Перехват конём или пешкой на проходе: «, Кf3×e5».
                for s in &o.reactions {
                    let mark = if s.hit.killed { "×" } else { "⚔" };
                    text += &format!(
                        ", {}{}{mark}{}",
                        s.attacker.kind.letter(),
                        square_name(s.from),
                        square_name(s.hit.square)
                    );
                }
                (text, o.reactions.iter().map(|s| s.hit.damage).sum())
            }
            Event::Card(play) => {
                // «Огненная Волна d4», «Обмен b1↔c1».
                let squares: Vec<String> = play.targets.iter().map(|&sq| square_name(sq)).collect();
                let text = if squares.is_empty() {
                    play.kind.title().to_owned()
                } else {
                    format!("{} {}", play.kind.title(), squares.join("↔"))
                };
                (text, play.hits.iter().map(|h| h.damage).sum())
            }
            Event::EndTurn(_, turn) => {
                // «Кc3⚔d5» — ранил, «Сc4×e6» — убил, «⚡» — молния слона задела ещё фигуры,
                // «Лава e7» — взрыв пешки.
                let mut text: Vec<String> = turn
                    .strikes
                    .iter()
                    .map(|s| {
                        let mark = if s.hit.killed { "×" } else { "⚔" };
                        let spark = if s.splash.iter().any(Option::is_some) {
                            "⚡"
                        } else {
                            ""
                        };
                        format!(
                            "{}{}{mark}{}{}{spark}",
                            s.attacker.kind.letter(),
                            square_name(s.from),
                            square_name(s.hit.square),
                            s.promoted.map_or("", Kind::letter)
                        )
                    })
                    .collect();
                text.extend(
                    turn.blasts
                        .iter()
                        .filter(|b| !b.hits.is_empty())
                        .map(|b| format!("Лава {}", square_name(b.center))),
                );
                if !turn.marched.is_empty() {
                    text.insert(0, format!("Волна ↓{}", turn.marched.len()));
                }
                if !turn.spawned.is_empty() {
                    let wave = self.before.rogue().map_or(0, |r| r.wave) + 1;
                    text.push(format!("Волна {wave}"));
                }
                let dealt = turn.strikes.iter().flat_map(Strike::hits).map(|h| h.damage);
                let burnt = turn.blasts.iter().flat_map(|b| &b.hits).map(|h| h.damage);
                (text.join(", "), dealt.chain(burnt).sum())
            }
        };
        Notation {
            text,
            damage: (dealt > 0).then_some(dealt),
            suffix,
        }
    }

    /// Подробное описание действия.
    pub fn description(&self) -> String {
        let encircled = matches!(&self.event, Event::Card(play) if play.kind == CardKind::Encircle);
        // Rogue-like: волна прорвалась к первой горизонтали игрока.
        let breach = matches!(&self.event, Event::EndTurn(_, turn)
            if self.before.rogue().is_some_and(|r| turn.marched.iter().any(|&(_, to)| rank_of(to) == r.player.home_rank())));
        let ending = self.loser.map_or_else(String::new, |loser| {
            if breach {
                format!(
                    " Волна дошла до первой горизонтали {} — поражение!",
                    loser.genitive()
                )
            } else if encircled {
                format!(" {} окружены и сломлены — победа!", loser.name())
            } else {
                format!(" У {} не осталось фигур!", loser.genitive())
            }
        });
        let o = match &self.event {
            Event::Move(outcome) => outcome,
            Event::Card(play) => return card_description(play, &self.before) + &ending,
            Event::EndTurn(color, turn) => {
                let mut text = format!("{} закончили ход.", color.name());
                if !turn.marched.is_empty() {
                    let steps: Vec<String> = turn
                        .marched
                        .iter()
                        .map(|&(from, to)| {
                            let name =
                                self.before.piece(from).map_or("фигура", |p| p.kind.name());
                            format!("{name} {}–{}", square_name(from), square_name(to))
                        })
                        .collect();
                    text += &format!(" Волны шагнули вперёд: {}.", steps.join(", "));
                }
                for strike in &turn.strikes {
                    text += &strike_description(strike);
                }
                for blast in &turn.blasts {
                    text += &format!(" Пешка {} взорвалась лавой", square_name(blast.center));
                    text += &if blast.hits.is_empty() {
                        ", никого не задев.".to_owned()
                    } else {
                        format!(": {}.", hits_text(&blast.hits))
                    };
                }
                if !turn.charged.is_empty() {
                    let squares: Vec<String> =
                        turn.charged.iter().map(|&sq| square_name(sq)).collect();
                    text += &format!(" Лавой насытились пешки {}.", squares.join(", "));
                }
                if !turn.spawned.is_empty() {
                    let rogue = self.before.rogue();
                    let wave = rogue.map_or(0, |r| r.wave) + 1;
                    let waves = rogue.map_or(0, |r| r.waves);
                    let pieces: Vec<String> = turn
                        .spawned
                        .iter()
                        .map(|&(sq, kind)| format!("{} {}", kind.name(), square_name(sq)))
                        .collect();
                    text += &format!(" Вышла волна {wave} из {waves}: {}.", pieces.join(", "));
                }
                return text + &ending;
            }
        };
        let piece = o.piece;
        let mut text = match o.mv.kind {
            MoveKind::CastleShort => format!("{}: короткая рокировка.", piece.color.name()),
            MoveKind::CastleLong => format!("{}: длинная рокировка.", piece.color.name()),
            MoveKind::Quiet => format!(
                "{} {}–{}.",
                piece.title(),
                square_name(o.mv.from),
                square_name(o.mv.to)
            ),
            MoveKind::Drop(_) => format!(
                "{} из запаса выставлен{} на {}.",
                piece.title(),
                if piece.kind.is_feminine() { "а" } else { "" },
                square_name(o.mv.to)
            ),
        };
        if let Some(kind) = o.promoted {
            text += &format!(" Пешка превратилась в {}.", kind.accusative());
        }
        if o.frozen {
            let kind = o.promoted.unwrap_or(piece.kind);
            let verb = if kind.is_feminine() {
                "вмёрзла"
            } else {
                "вмёрз"
            };
            text += &format!(" {} {verb} в ледяной барьер.", kind.title());
        }
        for strike in &o.reactions {
            text += " Перехват:";
            text += &strike_description(strike);
        }
        if self.loser.is_some() {
            text += &ending;
        } else if self.check {
            text += " Шах!";
        }
        text
    }
}

/// «конь d5 — −300 HP, слон f5 — погиб».
fn hits_text(hits: &[WaveHit]) -> String {
    let parts: Vec<String> = hits
        .iter()
        .map(|h| {
            let fate = if h.killed {
                "погиб".to_owned()
            } else {
                format!("−{} HP", shown(h.damage))
            };
            format!(
                "{} {} — {fate}",
                h.target.kind.name(),
                square_name(h.square)
            )
        })
        .collect();
    parts.join(", ")
}

/// Удар в описании конца хода: « Конь c3 ударил пешку d5: −500 HP, осталось 400/900.»,
/// « Слон c4 убил пешку e6 (−500 HP).»
fn strike_description(strike: &Strike) -> String {
    let attacker = strike.attacker.kind;
    let hit = strike.hit;
    let verb_end = if attacker.is_feminine() { "а" } else { "" };
    let mut text = if hit.killed {
        format!(
            " {} {} убил{verb_end} {} на {} (−{} HP){}.",
            attacker.title(),
            square_name(strike.from),
            hit.target.kind.accusative(),
            square_name(hit.square),
            shown(hit.damage),
            if strike.advanced {
                format!(" и встал{verb_end} на её место")
            } else {
                String::new()
            }
        )
    } else {
        format!(
            " {} {} ударил{verb_end} {} на {}: −{} HP, осталось {}/{}.",
            attacker.title(),
            square_name(strike.from),
            hit.target.kind.accusative(),
            square_name(hit.square),
            shown(hit.damage),
            shown(hit.hp_after()),
            shown(hit.target.max_hp())
        )
    };
    if strike.feast > 0 {
        text += &format!(" Воин восстановил {} HP.", shown(strike.feast));
    }
    if strike.promoted.is_some() {
        text += " Пешка стала ферзём.";
    }
    let splash: Vec<WaveHit> = strike
        .splash
        .iter()
        .flatten()
        .map(|h| WaveHit {
            square: h.square,
            target: h.target,
            damage: h.damage,
            killed: h.killed,
        })
        .collect();
    if !splash.is_empty() {
        text += &format!(" Молния задела: {}.", hits_text(&splash));
    }
    if strike.chilled {
        let verb = if hit.target.kind.is_feminine() {
            "замёрзла"
        } else {
            "замёрз"
        };
        text += &format!(" {} {verb}.", hit.target.kind.title());
    }
    text
}

/// «Белые сыграли карту «Огненная Волна» на d4: …», «… «Обмен»: конь b1 и слон c1
/// поменялись местами.»; `before` — позиция до карты.
fn card_description(play: &CardPlay, before: &Position) -> String {
    let mut text = format!(
        "{} сыграли карту «{}»",
        play.color.name(),
        play.kind.title()
    );
    if let Some((points, n)) = play.kind.cost() {
        text += &format!(" за {}", points.amount(n));
    }
    let name = |sq: Square| {
        before
            .piece(sq)
            .map_or_else(String::new, |p| format!("{} ", p.kind.name()))
    };
    if let &[a, b] = play.targets.as_slice() {
        let mut text = format!(
            "{text}: {}{} и {}{} поменялись местами.",
            name(a),
            square_name(a),
            name(b),
            square_name(b)
        );
        // Фигура, вставшая в ледяную зону, замерзает; на `sq` теперь фигура с другой клетки.
        for &sq in &play.frozen {
            let from = if sq == a { b } else { a };
            text += &format!(" Лёд сковал {}на {}.", name(from), square_name(sq));
        }
        return text;
    }
    if play.kind == CardKind::Funding {
        let drawn = match play.drawn {
            0 => "карт не взяли — колода пуста или рука полна".to_owned(),
            1 => "взяли 1 карту".to_owned(),
            n => format!("взяли {n} карты"),
        };
        return format!("{text}: {drawn}.");
    }
    if let &[sq] = play.targets.as_slice() {
        text += &format!(" на {}", square_name(sq));
    }
    if play.kind == CardKind::Heal {
        if play.heals.is_empty() {
            return format!("{text}: лечить было некого.");
        }
        let heals: Vec<String> = play
            .heals
            .iter()
            .map(|h| {
                format!(
                    "{} {} — +{} HP",
                    h.target.kind.name(),
                    square_name(h.square),
                    shown(h.amount)
                )
            })
            .collect();
        return format!("{text}: {}.", heals.join(", "));
    }
    let target = play
        .targets
        .first()
        .map_or_else(String::new, |&sq| name(sq));
    match play.kind {
        CardKind::Offering => return format!("{text}: {target}уходит с доски."),
        CardKind::Multiply => {
            return match play.spawned {
                Some(sq) => format!(
                    "{text}: {target}размножился — второй на {}.",
                    square_name(sq)
                ),
                None => format!("{text}: размножаться некуда."),
            };
        }
        CardKind::Champion => return format!("{text}: {target}— один в поле воин."),
        CardKind::Bastion => return format!("{text}: ладья заняла оборону."),
        CardKind::IceWall => return format!("{text}: горизонталь покрылась льдом на два хода."),
        CardKind::MassSacrifice => {
            let lost: Vec<String> = play
                .hits
                .iter()
                .map(|h| {
                    format!(
                        "{} {} — −{} HP",
                        h.target.kind.name(),
                        square_name(h.square),
                        shown(h.damage)
                    )
                })
                .collect();
            return if lost.is_empty() {
                format!("{text}: своих фигур в квадрате нет.")
            } else {
                format!("{text}: {}.", lost.join(", "))
            };
        }
        CardKind::Encircle => return format!("{text}."),
        CardKind::BloodGod => return format!("{text}: теперь вместо очков жертвы — очки победы."),
        CardKind::AbsoluteZero => {
            let frozen: Vec<String> = play
                .frozen
                .iter()
                .map(|&sq| format!("{}{}", name(sq), square_name(sq)))
                .collect();
            return format!("{text}: навсегда замёрзли {}.", frozen.join(", "));
        }
        CardKind::Barrier => {
            let squares = Area::Square.squares(play.targets[0]);
            return format!(
                "{text}: барьер {}–{} — соперник не войдёт в него до начала следующего хода {}.",
                square_name(squares[0]),
                square_name(squares[squares.len() - 1]),
                play.color.genitive()
            );
        }
        CardKind::IceWarrior => {
            return format!("{text}: {target}— воин изо льда, его удары замораживают.");
        }
        CardKind::NorthWarrior => {
            let (attack, hp) = north_bonus(before.frozen_enemies());
            return format!("{text}: {target}получает +{attack} атаки и +{hp} здоровья.");
        }
        CardKind::ColdRuler => {
            let n = before.frozen_enemies();
            let mana = u32::from(n) * u32::from(cold_ruler_mana());
            return format!("{text}: +{mana} маны за замёрзших ({n}).");
        }
        CardKind::VictoryTales => {
            let n = before.kills(play.color);
            let mana = u32::from(n) * u32::from(victory_tales_mana());
            return format!("{text}: +{mana} маны за побеждённых ({n}).");
        }
        _ => {}
    }
    if let (Some(area), &[aim], true) = (
        play.kind.area(),
        play.targets.as_slice(),
        play.kind.freezes(),
    ) {
        let squares = area.squares(aim);
        let (first, last) = (squares[0], squares[squares.len() - 1]);
        let files =
            |a: Square, b: Square| format!("{}–{}", (b'a' + a % 8) as char, (b'a' + b % 8) as char);
        let ranks = |a: Square, b: Square| format!("{}–{}", a / 8 + 1, b / 8 + 1);
        let place = match area {
            Area::Ranks => format!("горизонтали {} замёрзли", ranks(first, last)),
            Area::Files => format!("вертикали {} замёрзли", files(first, last)),
            Area::Rank => format!("горизонталь {} замёрзла", first / 8 + 1),
            Area::Square => format!(
                "ледяная зона {}–{} до начала следующего хода {}",
                square_name(first),
                square_name(last),
                play.color.genitive()
            ),
        };
        let frozen: Vec<String> = play
            .frozen
            .iter()
            .map(|&sq| format!("{}{}", name(sq), square_name(sq)))
            .collect();
        return if frozen.is_empty() {
            format!("{text}: {place}, но замораживать было некого.")
        } else {
            format!("{text}: {place} — заморожены {}.", frozen.join(", "))
        };
    }
    if play.hits.is_empty() {
        return match play.kind {
            CardKind::IceBlast => format!("{text}: задеть было некого."),
            _ => text + ".",
        };
    }
    let hits: Vec<String> = play
        .hits
        .iter()
        .map(|h| {
            let fate = if h.killed {
                "погиб".to_owned()
            } else {
                format!("−{} HP", shown(h.damage))
            };
            format!(
                "{} {} — {fate}",
                h.target.kind.name(),
                square_name(h.square)
            )
        })
        .collect();
    let chill = if play.kind == CardKind::IceShard && !play.frozen.is_empty() {
        " и замёрз"
    } else {
        ""
    };
    format!("{text}: {}{chill}.", hits.join(", "))
}

/// Партия: текущая позиция, история и итог. Ход стороны — ход фигурой и, по желанию,
/// до `CARDS_PER_TURN` карт (до и после него); ход кончается кнопкой «Конец хода» или
/// сам, когда делать больше нечего.
#[derive(Clone, Debug)]
pub struct Game {
    position: Position,
    history: Vec<Record>,
    /// Ключи позиций в начале каждого хода, включая начальную, — для троекратного
    /// повторения.
    keys: Vec<u64>,
    legal: Vec<Move>,
    status: Status,
}

impl Game {
    /// Новая партия с перемешанными колодами сторон (номера колод из меню; белые, чёрные).
    #[allow(dead_code)] // карты убраны из интерфейса (0.40)
    pub fn new(decks: [usize; 2]) -> Game {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x9E37_79B9_7F4A_7C15, |d| d.as_nanos() as u64);
        Game::from_position(Position::start().with_shuffled_cards(seed, decks))
    }

    pub fn from_position(position: Position) -> Game {
        let mut game = Game {
            keys: vec![position.key()],
            position,
            history: Vec::new(),
            legal: Vec::new(),
            status: Status::Ongoing,
        };
        game.refresh();
        game
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn history(&self) -> &[Record] {
        &self.history
    }

    /// Ходы фигурой; после хода фигурой в этом ходу их больше нет.
    pub fn legal_moves(&self) -> &[Move] {
        &self.legal
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn is_over(&self) -> bool {
        self.status != Status::Ongoing
    }

    /// Можно ли закончить ход: фигурой уже сходили или ходить нечем (фигуры заморожены
    /// или заперты) — тогда ход фигурой пропускается.
    pub fn can_end_turn(&self) -> bool {
        !self.is_over() && (self.legal.is_empty() || self.position.moved())
    }

    /// Ходит фигурой, если ход законен. Если карту в этом ходу уже не сыграть, ход
    /// кончается сам; если у одной из сторон не осталось фигур — кончается партия.
    pub fn play(&mut self, mv: Move) -> Option<Outcome> {
        if self.is_over() || !self.legal.contains(&mv) {
            return None;
        }
        let before = self.position.clone();
        let outcome = self.position.make_move(mv);
        let check = self.position.in_check(before.side().opponent());
        self.history.push(Record {
            before,
            event: Event::Move(outcome.clone()),
            check,
            loser: None,
        });
        self.refresh();
        // Ходы в этом ходу ещё есть (rogue-like) — ход ждёт кнопку.
        if !self.note_victory() && !self.position.any_card_playable() && self.legal.is_empty() {
            self.finish_turn();
        }
        Some(outcome)
    }

    /// Если последнее действие оставило сторону без фигур, отмечает это в нём; `true` —
    /// партия кончилась.
    fn note_victory(&mut self) -> bool {
        let (Status::Victory { winner } | Status::Encircled { winner }) = self.status else {
            return false;
        };
        if let Some(record) = self.history.last_mut() {
            record.loser = Some(winner.opponent());
        }
        true
    }

    /// Сторона, которая ходит, разыгрывает карту `index` — до `CARDS_PER_TURN` карт за
    /// ход; карте с выбором нужны клетки `targets`. Если фигурой уже сходили (или ходить
    /// нечем), а сыграть больше нечего, ход кончается.
    pub fn play_card(&mut self, index: usize, targets: &[Square]) -> Option<CardPlay> {
        if self.is_over() {
            return None;
        }
        let before = self.position.clone();
        let play = self.position.play_card(index, targets)?;
        let check = self.position.in_check(before.side().opponent());
        self.history.push(Record {
            before,
            event: Event::Card(play.clone()),
            check,
            loser: None,
        });
        self.refresh();
        if !self.note_victory() && self.legal.is_empty() && !self.position.any_card_playable() {
            self.finish_turn();
        }
        Some(play)
    }

    /// Усиливает фигуры стороны `color` (компьютер) — в начале партии.
    pub fn boost(&mut self, color: Color) {
        self.position.boost(color);
        self.keys = vec![self.position.key()];
        self.refresh();
    }

    /// Кнопка «Конец хода»: работает, если фигурой уже сходили или ходить нечем.
    pub fn end_turn(&mut self) -> bool {
        if !self.can_end_turn() {
            return false;
        }
        self.finish_turn();
        true
    }

    /// Конец хода: фигуры сходившей стороны бьют; если у соперника не осталось фигур —
    /// партия кончилась.
    fn finish_turn(&mut self) {
        let before = self.position.clone();
        let mover = before.side();
        let turn = self.position.end_turn();
        self.keys.push(self.position.key());
        self.history.push(Record {
            before,
            event: Event::EndTurn(mover, turn),
            check: false,
            loser: None,
        });
        self.refresh();
        if self.note_victory() {
            return;
        }
        // Шах — у последнего хода фигурой: после карты шах мог появиться или уйти.
        let check = self.position.in_check(self.position.side());
        if let Some(record) = self
            .history
            .iter_mut()
            .rev()
            .take_while(|r| r.color() == mover)
            .find(|r| r.outcome().is_some())
        {
            record.check = check;
        }
    }

    /// Отменяет последнее действие: ход фигурой, карту или конец хода.
    pub fn undo(&mut self) -> bool {
        let Some(record) = self.history.pop() else {
            return false;
        };
        if matches!(record.event, Event::EndTurn(..)) {
            self.keys.pop();
        }
        self.position = record.before;
        self.refresh();
        true
    }

    /// Фигуры, которых убила сторона `color` — ударами в конце хода или картой.
    pub fn kills_by(&self, color: Color) -> Vec<Piece> {
        let mut kills = Vec::new();
        for record in &self.history {
            match &record.event {
                Event::Move(o) => {
                    kills.extend(
                        o.reactions
                            .iter()
                            .filter(|s| s.attacker.color == color && s.hit.killed)
                            .map(|s| s.hit.target),
                    );
                }
                Event::EndTurn(mover, turn) if *mover == color => {
                    let struck = turn.strikes.iter().flat_map(Strike::hits).copied();
                    let burnt = turn.blasts.iter().flat_map(|b| &b.hits).map(|h| Hit {
                        square: h.square,
                        target: h.target,
                        damage: h.damage,
                        killed: h.killed,
                    });
                    kills.extend(struck.chain(burnt).filter(|h| h.killed).map(|h| h.target));
                }
                Event::Card(play) if play.color == color => {
                    kills.extend(
                        play.hits
                            .iter()
                            .filter(|h| h.killed && h.target.color != color)
                            .map(|h| h.target),
                    );
                }
                Event::Card(_) | Event::EndTurn(..) => {}
            }
        }
        kills
    }

    fn refresh(&mut self) {
        self.legal = self.position.legal_moves();
        let side = self.position.side();
        let key = self.position.key();
        // Сторона без фигур проигрывает сразу, даже посреди хода; если карта (волна, ледяной
        // взрыв) оставила без фигур обе стороны — проигрывает сыгравший её, он и ходит. Ничьи
        // решаются в начале хода: посреди хода фигурой уже сходили. Пата нет: кому ходить
        // нечем, тот пропускает ход фигурой.
        let wiped = [side, side.opponent()]
            .into_iter()
            .find(|&color| self.position.defeated(color));
        self.status = if let Some(loser) = wiped {
            Status::Victory {
                winner: loser.opponent(),
            }
        } else if let Some(winner) = self.position.encircled_by() {
            Status::Encircled { winner }
        } else if self.position.moved() || self.position.rogue().is_some() {
            // Посреди хода ничьих нет; в rogue-like их нет вовсе — волны идут.
            Status::Ongoing
        } else if self.position.halfmove_clock() >= 100 {
            Status::FiftyMoves
        } else if self.keys.iter().filter(|&&k| k == key).count() >= 3 {
            Status::Repetition
        } else {
            Status::Ongoing
        };
        if self.is_over() {
            self.legal.clear();
        }
    }
}

#[cfg(test)]
mod tests;
