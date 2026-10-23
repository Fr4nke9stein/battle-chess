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
    /// Огненная фигура (rogue-like, из магазина, с 0.63): как пешка с лавой у усиленного
    /// компьютера взрывается в конце каждого своего хода, но огонь не уходит.
    pub fire: bool,
    /// Воздушная фигура (rogue-like, с 0.67): ходит дважды за один ход, тратя один.
    pub wind: bool,
    /// Золотая фигура (rogue-like, с 0.68): победа ею даёт сверху `Rule::RogueGoldBonus`.
    pub gold: bool,
    /// Теневая фигура (rogue-like, с 0.70): ходит сквозь фигуры (`shade_moves`).
    pub shade: bool,
    /// Ледяная фигура (rogue-like, с 0.72): ударившая её фигура замерзает на ход; сама она
    /// своим ударом не замораживает.
    pub icy: bool,
    /// Король-толстосум (rogue-like, с 0.73): босс последней волны первого уровня — своё
    /// здоровье и атака (`Rule::RogueBoss*`), бьёт 16 клеток (`BOSS_STRIKES`).
    pub boss: bool,
    /// Ядовитая фигура волны (rogue-like, с 0.79): её удар отравляет выжившую цель.
    pub venom: bool,
    /// Сколько ударов по теневой фигуре прошло с прошлого уклонения (герб двух теневых огней).
    pub dodge: u8,
    /// Заряд электрической фигуры — сколько она убила (герб красной молнии, с 0.91).
    pub charge: u8,
    /// Бронированная фигура волны (rogue-like, с 0.89): здоровья больше на `Rule::RogueArmorHp`.
    pub armored: bool,
    /// Фигура-вампир волны (rogue-like, с 0.89): здоровья больше на `Rule::RogueVampireHp`,
    /// каждым ударом по чужой фигуре восстанавливает `Rule::RogueVampireHeal`.
    pub vampire: bool,
    /// Фигура-зомби волны (rogue-like, с 0.105): здоровья больше на `Rule::RogueZombieHp`; погибнув,
    /// оставляет могилу (`Position::graves`) и через ход восстаёт.
    pub zombie: bool,
    /// Зомби уже восставал — второй раз не восстанет.
    pub risen: bool,
    /// Варвар (rogue-like, с 0.80): убив кого-то за удар, бьёт ещё раз (`strike_all`).
    pub barbarian: bool,
    /// Маг (rogue-like, с 0.83): в конце своего хода — метеорит по случайной чужой фигуре.
    pub wizard: bool,
    /// Отравлена: через столько концов ходов своей стороны получит `Rule::RoguePoisonDamage`
    /// (0 — не отравлена; яд не копится).
    pub poison: u8,
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
            fire: false,
            wind: false,
            gold: false,
            shade: false,
            icy: false,
            boss: false,
            venom: false,
            dodge: 0,
            charge: 0,
            armored: false,
            vampire: false,
            zombie: false,
            risen: false,
            barbarian: false,
            wizard: false,
            poison: 0,
            reborn: 0,
        }
    }

    pub fn is_frozen(self) -> bool {
        self.frozen > 0
    }

    /// Новая фигура (полное здоровье, без льда) с возвращениями и огнём фигуры `old`.
    fn with(self, old: Piece) -> Piece {
        Piece {
            reborn: old.reborn,
            fire: old.fire,
            spark: old.spark,
            wind: old.wind,
            gold: old.gold,
            shade: old.shade,
            icy: old.icy,
            barbarian: old.barbarian,
            charge: old.charge,
            wizard: old.wizard,
            ..self
        }
    }

    pub fn max_hp(self) -> i32 {
        if self.boss {
            return balance().rule(Rule::RogueBossHp).max(1) * crate::balance::UNIT;
        }
        let base = self.kind.stats().max_hp;
        let base = if self.champion {
            base * (100 + balance().param(Param::ChampionPercent)) / 100
        } else {
            base
        };
        base + self.bonus_hp as i32
    }

    pub fn attack(self) -> i32 {
        if self.boss {
            return balance().rule(Rule::RogueBossAttack).max(0) * crate::balance::UNIT;
        }
        let step = balance().param(Param::ChampionPercent);
        self.kind.stats().attack * (100 + step * self.boost as i32) / 100 + self.bonus_attack as i32
    }

    /// Есть ли у фигуры сила `power` (флаг).
    pub fn has_power(self, power: Power) -> bool {
        match power {
            Power::Plain => false,
            Power::Fire => self.fire,
            Power::Spark => self.spark,
            Power::Wind => self.wind,
            Power::Gold => self.gold,
            Power::Shade => self.shade,
            Power::Ice => self.icy,
            Power::Barbarian => self.barbarian,
            Power::Wizard => self.wizard,
        }
    }

    /// «Белый конь», «Чёрная ладья».
    pub fn title(self) -> String {
        let adjective = match (self.color, self.kind.is_feminine()) {
            (Color::White, false) => "Белый",
            (Color::White, true) => "Белая",
            (Color::Black, false) => "Чёрный",
            (Color::Black, true) => "Чёрная",
        };
        if self.boss {
            return format!("{adjective} король-толстосум");
        }
        // Силы — прилагательными по порядку (объединённая — обе, основа — ближе к имени: «огненный
        // золотой», «золотой варварский»), маг — «-маг» в конце.
        let powers: String = [
            Power::Fire,
            Power::Spark,
            Power::Wind,
            Power::Shade,
            Power::Ice,
            Power::Gold,
            Power::Barbarian,
        ]
        .into_iter()
        .filter(|&p| self.has_power(p))
        .map(|p| p.adjective(self.kind.is_feminine()))
        .collect();
        if self.wizard {
            return format!("{adjective} {powers}{}-маг", self.kind.name());
        }
        if self.vampire && self.armored {
            let armored = if self.kind.is_feminine() {
                "бронированная"
            } else {
                "бронированный"
            };
            return format!("{adjective} {armored} {}-вампир", self.kind.name());
        }
        if self.vampire {
            return format!("{adjective} {}-вампир", self.kind.name());
        }
        if self.zombie && self.venom {
            let venom = if self.kind.is_feminine() {
                "ядовитая"
            } else {
                "ядовитый"
            };
            return format!("{adjective} {venom} {}-зомби", self.kind.name());
        }
        if self.zombie {
            return format!("{adjective} {}-зомби", self.kind.name());
        }
        if self.armored {
            let armored = if self.kind.is_feminine() {
                "бронированная"
            } else {
                "бронированный"
            };
            return format!("{adjective} {armored} {}", self.kind.name());
        }
        if self.venom {
            let venom = if self.kind.is_feminine() {
                "ядовитая"
            } else {
                "ядовитый"
            };
            return format!("{adjective} {venom} {}", self.kind.name());
        }
        format!("{adjective} {powers}{}", self.kind.name())
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
    /// Rogue-like (с 0.63): выставление огненной фигуры из запаса.
    FireDrop(Kind),
    /// Rogue-like (с 0.64): выставление электрической фигуры из запаса.
    SparkDrop(Kind),
    /// Rogue-like (с 0.67): выставление воздушной фигуры из запаса.
    WindDrop(Kind),
    /// Rogue-like (с 0.68): выставление золотой фигуры из запаса.
    GoldDrop(Kind),
    /// Rogue-like (с 0.70): выставление теневой фигуры из запаса.
    ShadeDrop(Kind),
    /// Rogue-like (с 0.72): выставление ледяной фигуры из запаса.
    IceDrop(Kind),
    /// Выставление фигуры-варвара из запаса (rogue-like, с 0.80).
    BarbarianDrop(Kind),
    /// Выставление фигуры-мага из запаса (rogue-like, с 0.83).
    WizardDrop(Kind),
    /// Выставление объединённой фигуры из запаса (rogue-like, с 0.96): вид и номер в `COMBOS`.
    ComboDrop(Kind, u8),
}

/// Сила фигуры rogue-like из магазина: обычная, огненная (с 0.63: `Piece::fire`) или
/// электрическая (с 0.64: `Piece::spark`, как слоны усиленного компьютера до 0.41).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Power {
    Plain,
    Fire,
    Spark,
    /// Воздушная (с 0.67, `Piece::wind`): ходит дважды за один ход.
    Wind,
    /// Золотая (с 0.68, `Piece::gold`): победа ею даёт больше золота.
    Gold,
    /// Теневая (с 0.70, `Piece::shade`): ходит сквозь фигуры.
    Shade,
    /// Ледяная (с 0.72, `Piece::icy`): ударивший её замерзает.
    Ice,
    /// Варвар (с 0.80, `Piece::barbarian`): убив за удар, бьёт ещё раз.
    Barbarian,
    /// Маг (с 0.83, `Piece::wizard`; не пешка и не король): в конце своего хода шлёт метеорит
    /// на случайную чужую фигуру.
    Wizard,
}

impl Power {
    /// «огненный », «электрическая »; у обычной — пусто.
    pub fn adjective(self, feminine: bool) -> &'static str {
        match (self, feminine) {
            (Power::Plain, _) => "",
            (Power::Fire, false) => "огненный ",
            (Power::Fire, true) => "огненная ",
            (Power::Spark, false) => "электрический ",
            (Power::Spark, true) => "электрическая ",
            (Power::Wind, false) => "воздушный ",
            (Power::Wind, true) => "воздушная ",
            (Power::Gold, false) => "золотой ",
            (Power::Gold, true) => "золотая ",
            (Power::Shade, false) => "теневой ",
            (Power::Shade, true) => "теневая ",
            (Power::Ice, false) => "ледяной ",
            (Power::Ice, true) => "ледяная ",
            (Power::Barbarian, false) => "варварский ",
            (Power::Barbarian, true) => "варварская ",
            (Power::Wizard, false) => "магический ",
            (Power::Wizard, true) => "магическая ",
        }
    }
}

/// Фигура в запасе: вид и сила (rogue-like, с 0.63); у объединённой (с 0.96) — ещё вторая сила
/// `with` (иначе `Power::Plain`): пара из `COMBOS`, `power` — основа (варвар, золотая, маг,
/// ледяная, огненная, электрическая или воздушная).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stock {
    pub kind: Kind,
    pub power: Power,
    pub with: Power,
}

/// Объединённые фигуры (rogue-like): (основа, вторая сила) — варвары (с 0.96), золотые (с 0.97),
/// маги (с 0.98), ледяные (с 0.99), огненные (с 0.101), электрические (с 0.102), воздушные (с
/// 0.103) — все пары особых сил; по порядку `combo_reserve` и кодов выставления
/// `MoveKind::ComboDrop` (новые — только в конец).
pub const COMBOS: [(Power, Power); 28] = [
    (Power::Barbarian, Power::Fire),
    (Power::Barbarian, Power::Spark),
    (Power::Barbarian, Power::Wind),
    (Power::Barbarian, Power::Gold),
    (Power::Barbarian, Power::Shade),
    (Power::Barbarian, Power::Ice),
    (Power::Barbarian, Power::Wizard),
    (Power::Gold, Power::Fire),
    (Power::Gold, Power::Spark),
    (Power::Gold, Power::Wind),
    (Power::Gold, Power::Shade),
    (Power::Gold, Power::Ice),
    (Power::Gold, Power::Wizard),
    (Power::Wizard, Power::Fire),
    (Power::Wizard, Power::Spark),
    (Power::Wizard, Power::Wind),
    (Power::Wizard, Power::Shade),
    (Power::Wizard, Power::Ice),
    (Power::Ice, Power::Fire),
    (Power::Ice, Power::Spark),
    (Power::Ice, Power::Wind),
    (Power::Ice, Power::Shade),
    (Power::Fire, Power::Spark),
    (Power::Fire, Power::Wind),
    (Power::Fire, Power::Shade),
    (Power::Spark, Power::Wind),
    (Power::Spark, Power::Shade),
    (Power::Wind, Power::Shade),
];

impl Stock {
    pub fn new(kind: Kind, power: Power) -> Stock {
        Stock {
            kind,
            power,
            with: Power::Plain,
        }
    }

    /// Объединённая фигура: основа `base` с силой `with` — пара из `COMBOS`.
    pub fn combo(kind: Kind, base: Power, with: Power) -> Stock {
        Stock {
            kind,
            power: base,
            with,
        }
    }

    /// Номер объединённой фигуры в `COMBOS`; у одиночной — `None`.
    pub fn combo_index(self) -> Option<usize> {
        COMBOS
            .iter()
            .position(|&pair| pair == (self.power, self.with))
    }

    /// Есть ли у фигуры сила `power` (первая или вторая).
    pub fn has(self, power: Power) -> bool {
        self.power == power || self.with == power
    }

    /// Как фигура называется без цвета: «огненный варварский конь», «золотой конь-маг».
    pub fn name(self) -> String {
        let title = self.piece(Color::White).title();
        title
            .split_once(' ')
            .map_or(title.clone(), |(_, rest)| rest.to_owned())
    }

    /// Какой фигурой запаса станет `piece`.
    pub fn of(piece: Piece) -> Stock {
        if let Some(&(base, with)) = COMBOS
            .iter()
            .find(|&&(base, with)| piece.has_power(base) && piece.has_power(with))
        {
            return Stock::combo(piece.kind, base, with);
        }
        let power = if piece.fire {
            Power::Fire
        } else if piece.spark {
            Power::Spark
        } else if piece.wind {
            Power::Wind
        } else if piece.gold {
            Power::Gold
        } else if piece.shade {
            Power::Shade
        } else if piece.icy {
            Power::Ice
        } else if piece.barbarian {
            Power::Barbarian
        } else if piece.wizard {
            Power::Wizard
        } else {
            Power::Plain
        };
        Stock::new(piece.kind, power)
    }

    /// Новая фигура этого вида и силы (сил) стороны `color`.
    pub fn piece(self, color: Color) -> Piece {
        Piece {
            fire: self.has(Power::Fire),
            spark: self.has(Power::Spark),
            wind: self.has(Power::Wind),
            gold: self.has(Power::Gold),
            shade: self.has(Power::Shade),
            icy: self.has(Power::Ice),
            barbarian: self.has(Power::Barbarian),
            wizard: self.has(Power::Wizard),
            ..Piece::new(self.kind, color)
        }
    }

    /// Ход, которым её выставляют.
    pub fn drop_kind(self) -> MoveKind {
        if let Some(i) = self.combo_index() {
            return MoveKind::ComboDrop(self.kind, i as u8);
        }
        match self.power {
            Power::Plain => MoveKind::Drop(self.kind),
            Power::Fire => MoveKind::FireDrop(self.kind),
            Power::Spark => MoveKind::SparkDrop(self.kind),
            Power::Wind => MoveKind::WindDrop(self.kind),
            Power::Gold => MoveKind::GoldDrop(self.kind),
            Power::Shade => MoveKind::ShadeDrop(self.kind),
            Power::Ice => MoveKind::IceDrop(self.kind),
            Power::Barbarian => MoveKind::BarbarianDrop(self.kind),
            Power::Wizard => MoveKind::WizardDrop(self.kind),
        }
    }
}

impl MoveKind {
    /// Какую фигуру запаса выставляет этот ход.
    pub fn stock(self) -> Option<Stock> {
        match self {
            MoveKind::Drop(kind) => Some(Stock::new(kind, Power::Plain)),
            MoveKind::FireDrop(kind) => Some(Stock::new(kind, Power::Fire)),
            MoveKind::SparkDrop(kind) => Some(Stock::new(kind, Power::Spark)),
            MoveKind::WindDrop(kind) => Some(Stock::new(kind, Power::Wind)),
            MoveKind::GoldDrop(kind) => Some(Stock::new(kind, Power::Gold)),
            MoveKind::ShadeDrop(kind) => Some(Stock::new(kind, Power::Shade)),
            MoveKind::IceDrop(kind) => Some(Stock::new(kind, Power::Ice)),
            MoveKind::BarbarianDrop(kind) => Some(Stock::new(kind, Power::Barbarian)),
            MoveKind::WizardDrop(kind) => Some(Stock::new(kind, Power::Wizard)),
            MoveKind::ComboDrop(kind, i) => {
                let (base, with) = COMBOS[i as usize % COMBOS.len()];
                Some(Stock::combo(kind, base, with))
            }
            _ => None,
        }
    }
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
        self.kind.stock().is_some()
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

/// Прыжок молнии электрической фигуры: с клетки `from` (цель удара или задетая раньше) на
/// задетую `hit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Jump {
    pub from: Square,
    pub hit: Hit,
}

/// Удар фигуры в конце хода её стороны: ответа нет; убив цель, атакующий встаёт на её клетку.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Электрическая фигура: прыжки молнии от цели по чужим фигурам вокруг (по порядку; с
    /// гербом двух молний — цепью по всем связанным).
    pub splash: Vec<Jump>,
    /// Убив цель вблизи, атакующий встал на её клетку (партия «с перемещением»).
    pub advanced: bool,
    /// Цель — ледяная фигура (rogue-like, с 0.72): атакующий замёрз и больше в этот ход не
    /// бьёт.
    pub rebound: bool,
    /// Герб снежинки и шипов (с 0.77): ледяная цель вернула урон атакующему (его клетка —
    /// `from`).
    pub thorns: Option<Hit>,
    /// Ядовитый атакующий отравил цель (rogue-like, с 0.79).
    pub poisoned: bool,
    /// Повторный удар варвара после убийства (rogue-like, с 0.80).
    pub again: bool,
    /// Повторный удар — круговой (герб топора и красного круга, с 0.81).
    pub whirl: bool,
    /// Теневая цель не получила этот удар (герб двух теневых огней, с 0.88).
    pub dodged: bool,
    /// Теневая цель ушла порталом (герб теневого огня и портала): её новая клетка и она сама;
    /// удар принимает обычная фигура (`hit.target`), вставшая на её место.
    pub swap: Option<(Square, Piece)>,
}

impl Strike {
    /// Все задетые ударом: цель, потом задетые молнией.
    pub fn hits(&self) -> impl Iterator<Item = &Hit> {
        std::iter::once(&self.hit).chain(self.splash.iter().map(|j| &j.hit))
    }

    /// Все, кому удар снял здоровье: задетые (`hits`) и атакующий, получивший урон шипов.
    pub fn wounded(&self) -> impl Iterator<Item = &Hit> {
        self.hits().chain(self.thorns.iter())
    }
}

/// Взрыв пешки, насыщенной лавой, в конце хода её стороны: огненная волна только по чужим
/// фигурам вокруг; сама пешка цела.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blast {
    pub center: Square,
    pub hits: Vec<WaveHit>,
    /// Большой взрыв крестом (герб двух огоньков, с 0.87) — своя анимация.
    pub wide: bool,
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
    /// Rogue-like: после расстановки начался этот уровень (его первая волна уже вышла).
    pub level_up: Option<u8>,
    /// Rogue-like: шаги фигур волн вперёд (откуда, куда) — до ударов.
    pub marched: Vec<(Square, Square)>,
    /// Rogue-like (с 0.59): все волны уровня отбиты — расстановка перед этим уровнем.
    pub planning: Option<u8>,
    /// Rogue-like: фигуры игрока, вернувшиеся к расстановке на клетки начала уровня (откуда,
    /// куда).
    pub returned: Vec<(Square, Square)>,
    /// Rogue-like (с 0.60): золото, полученное за побеждённые в конце этого хода фигуры.
    pub gold: u16,
    /// Rogue-like (с 0.79): урон яда отравленным фигурам стороны — после ударов и взрывов.
    pub poison: Vec<Hit>,
    /// Rogue-like (с 0.83): метеориты магов стороны — после взрывов, до яда.
    pub meteors: Vec<Meteor>,
    /// Rogue-like (с 0.94): урон фигурам, шагнувшим в горящий кратер (герб шляпы и метеорита).
    pub burned: Vec<Hit>,
    /// Rogue-like (с 0.95): золото свиньи-копилки на конец уровня.
    pub interest: u16,
    /// Rogue-like (с 0.105): восставшие зомби — клетка и вид.
    pub risen: Vec<(Square, Kind)>,
}

/// Метеорит мага (rogue-like, с 0.83; с 0.94 — отдельной записью).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meteor {
    /// Клетка мага.
    pub from: Square,
    /// Куда упал.
    pub at: Square,
    /// Задетые: клетка падения, с гербом шляпы и плюса — и соседние по прямым.
    pub hits: Vec<Hit>,
    /// Бьёт крестом (герб шляпы и плюса) — своя волна.
    pub wide: bool,
    /// Оставил горящий кратер (герб шляпы и метеорита).
    pub crater: bool,
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
    /// Сколько волн уже вышло и сколько всего на этом уровне.
    pub wave: u8,
    pub waves: u8,
    /// Уровень (с 1) и сколько их всего (с 0.54): отбил все волны уровня — следующий, на
    /// одну волну длиннее, пешек в волнах — реже.
    pub level: u8,
    pub levels: u8,
    /// Расстановка перед следующим уровнем (с 0.59, просьба пользователя): фигуры игрока
    /// вернулись на клетки начала уровня и вылечились; он двигает и меняет их местами,
    /// выставляет запас, а кнопка «Начать раунд» начинает уровень.
    pub planning: bool,
    /// Откуда начинала уровень фигура игрока, стоящая на клетке (`NO_ORIGIN` — выставлена из
    /// запаса посреди уровня); `relocate` переносит метку вместе с фигурой.
    origin: [u8; 64],
    /// Золото игрока (с 0.60, просьба пользователя): за каждую побеждённую фигуру волн —
    /// `rogue_price` её вида.
    pub gold: u16,
    /// Открыт магазин (после уровня, до расстановки): на прилавке `offers`, «Уйти» закрывает.
    pub shop: bool,
    /// Фигуры на прилавке (`None` — куплена); их `Rule::RogueShopSlots`.
    offers: [Option<Stock>; SHOP_MAX],
    /// Сколько раз прилавок заполнялся — случайность магазина.
    rolls: u16,
    /// Гербы игрока (с 0.77) — в порядке покупки; действуют до конца забега.
    crests: [Option<Crest>; CRESTS],
    /// Гербы на прилавке (`None` — куплен или не осталось); их `Rule::RogueCrestSlots`.
    crest_offers: [Option<Crest>; SHOP_MAX],
    /// Сколько раз гербы на прилавке выкладывались заново — их случайность.
    crest_rolls: u16,
    /// Случайность волн этой партии.
    seed: u64,
}

/// Герб rogue-like (с 0.77, просьба пользователя): покупается в магазине за
/// `Rule::RogueCrestPrice` и усиливает фигуры игрока до конца забега. Каждый — один раз.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Crest {
    /// Снежинка и сердце: ледяные фигуры — с `Rule::RogueCrestHeartHp` здоровья сверху.
    Heart,
    /// Снежинка и монета: ледяные фигуры выходят на прилавок чаще (`Rule::RogueCrestCoinPercent`).
    Coin,
    /// Двойная снежинка: ледяные фигуры замораживают ударившего на
    /// `Rule::RogueCrestFreezeTurns` хода.
    Snow2,
    /// Снежинка и пешка: ледяная пешка в магазине стоит как обычная.
    Pawn,
    /// Снежинка и шипы: ударивший ледяную фигуру получает её урон
    /// (`Rule::RogueCrestSpikesPercent`).
    Spikes,
    /// Замороженная снежинка: ледяная фигура, у которой остаётся `Rule::RogueCrestCrystalHp`
    /// здоровья, вмерзает в лёд и, пока во льду, не получает урона.
    Crystal,
    /// Топор и меч (с 0.81): варвары бьют сильнее (`Rule::RogueCrestAxeAttack`), здоровья у них
    /// меньше (`Rule::RogueCrestAxeHp`).
    AxeSword,
    /// Топор и клыки: варвар, убив, раз за ход лечится (`Rule::RogueCrestFangsHeal`).
    AxeFangs,
    /// Топор и ферзь: ферзь-варвар в магазине дешевле (`Rule::RogueCrestQueenDiscount`).
    AxeQueen,
    /// Два топора: варвары выходят на прилавок чаще (`Rule::RogueCrestAxesPercent`).
    Axes,
    /// Топор и красный круг: повторный удар варвара — круговой, по всем чужим в радиусе
    /// `Rule::RogueCrestWhirlRadius`.
    Whirl,
    /// Огненный топор: варвар с `Rule::RogueCrestFireAxeHp` здоровья и меньше бьёт после убийства
    /// ещё `Rule::RogueCrestFireAxeRepeats` раз сверху.
    FireAxe,
    /// Огонь и пешка (с 0.86): огненная пешка в магазине дешевле
    /// (`Rule::RogueCrestFlamePawnDiscount`).
    FlamePawn,
    /// Два огонька: взрыв огненной фигуры ещё и крестом на `Rule::RogueCrestFlamesReach` клетки.
    Flames,
    /// Огонь и череп: погибшая огненная фигура взрывается.
    FlameSkull,
    /// Огонь и меч: взрыв огненной фигуры — `Rule::RogueCrestFlameSwordDamage` урона.
    FlameSword,
    /// Огонь и монета: огненные фигуры выходят на прилавок чаще
    /// (`Rule::RogueCrestFlameCoinPercent`).
    FlameCoin,
    /// Падающая бомба: взрыв огненной фигуры сильнее на `Rule::RogueCrestBombDamage`, но задевает
    /// и своих.
    Bomb,
    /// Теневой огонь и ладья (с 0.88): теневая ладья в магазине дешевле
    /// (`Rule::RogueCrestDarkRookDiscount`).
    DarkRook,
    /// Теневой огонь и череп: теневая фигура бьёт своей атакой чужих, через которых прошла.
    DarkSkull,
    /// Два теневых огня: теневая фигура не получает каждый `Rule::RogueCrestDarkDodgeEvery`-й
    /// удар по ней.
    DarkFlames,
    /// Теневой огонь и портал: теневая фигура с `Rule::RogueCrestDarkPortalHp` здоровья и
    /// меньше от смертельного удара уходит порталом — меняется местами с ближайшей своей
    /// обычной фигурой (не королём), удар и дальнейшие удары принимает та.
    DarkPortal,
    /// Теневой огонь и монета: теневые фигуры выходят на прилавок чаще
    /// (`Rule::RogueCrestDarkCoinPercent`).
    DarkCoin,
    /// Теневой огонь и глаз: теневая фигура бьёт сквозь фигуры — всех в досягаемости по линии.
    DarkEye,
    /// Два ветра (с 0.90): воздушная фигура ходит трижды за один ход (второй и третий —
    /// бесплатно).
    Winds,
    /// Ветер и монета: воздушные фигуры выходят на прилавок чаще
    /// (`Rule::RogueCrestWindCoinPercent`).
    WindCoin,
    /// Ветер и конь: воздушный конь в магазине дешевле (`Rule::RogueCrestWindKnightDiscount`).
    WindKnight,
    /// Ветер и меч: воздушная фигура бьёт первую из своих целей дважды.
    WindSword,
    /// Молния и слон (с 0.91): электрический слон в магазине дешевле
    /// (`Rule::RogueCrestBoltBishopDiscount`).
    BoltBishop,
    /// Молния и монета: электрические фигуры выходят на прилавок чаще
    /// (`Rule::RogueCrestBoltCoinPercent`).
    BoltCoin,
    /// Молния и плюс: молния расходится от цели не только по диагоналям, но и по прямым.
    BoltPlus,
    /// Две молнии (с 0.92): молния бьёт цепью — от каждой задетой дальше на связанные с ней
    /// чужие фигуры (по тем же направлениям и на ту же дальность), пока связаны.
    Bolts,
    /// Красная молния: убийства электрической фигуры копят заряд (`Piece::charge`); за каждые
    /// `Rule::RogueCrestRedBoltStep` молния летит на клетку дальше (через пустые), не больше чем
    /// на `Rule::RogueCrestRedBoltMax`.
    RedBolt,
    /// Молния и единица: электрические фигуры бьют первыми.
    BoltOne,
    /// Две пешки (с 0.93): пешки выходят на прилавок чаще (`Rule::RogueCrestPairPercent`).
    Pawns,
    /// Два слона: слоны на прилавке чаще.
    Bishops,
    /// Два коня: кони на прилавке чаще.
    Knights,
    /// Две ладьи: ладьи на прилавке чаще.
    Rooks,
    /// Два ферзя: ферзи на прилавке чаще.
    Queens,
    /// Монета и обновление: обновить фигуры в магазине — `Rule::RogueCrestRerollCoinCost`.
    RerollCoin,
    /// Шляпа и монета (с 0.94): маги выходят на прилавок чаще (`Rule::RogueCrestHatCoinPercent`).
    HatCoin,
    /// Две шляпы: маг шлёт два метеорита вместо одного.
    Hats,
    /// Шляпа и слон: слон-маг в магазине дешевле (`Rule::RogueCrestHatBishopDiscount`).
    HatBishop,
    /// Метеорит: урон метеорита — `Rule::RogueCrestMeteorDamage`.
    Meteor,
    /// Шляпа и метеорит: на месте падения — горящий кратер до вашего следующего хода; вошедшая в
    /// него фигура получает `Rule::RogueCrestCraterDamage`.
    HatMeteor,
    /// Шляпа и плюс: метеорит бьёт крестом — клетку и четыре соседние по прямым.
    HatPlus,
    /// Король (с 0.95): победа золотой фигурой — ещё `Rule::RogueCrestGoldKingBonus` золота.
    GoldKing,
    /// Король и монета: фигуры запаса продаются за полную цену в магазине.
    KingCoin,
    /// Два короля: золотые фигуры выходят на прилавок чаще (`Rule::RogueCrestKings2Percent`).
    Kings2,
    /// Король и ферзь: победа золотым ферзём — ещё `Rule::RogueCrestKingQueenBonus` золота.
    KingQueen,
    /// Король и череп: погибшая золотая фигура возвращает свою цену в магазине золотом.
    KingSkull,
    /// Свинья-копилка: в конце уровня за каждые `Rule::RogueCrestPiggyStep` золота каждая золотая
    /// фигура (на доске и в запасе) приносит монету.
    Piggy,
    /// Топор и огонёк (с 0.96): в магазине появляются огненные варвары (`unlocks`).
    AxeFlame,
    /// Топор и молния: электрические варвары.
    AxeBolt,
    /// Топор и смерч: воздушные варвары.
    AxeWind,
    /// Топор и король: золотые варвары.
    AxeKing,
    /// Топор и тёмный огонёк: теневые варвары.
    AxeDark,
    /// Топор и снежинка: ледяные варвары.
    AxeSnow,
    /// Топор и шляпа: варвары-маги.
    AxeHat,
    /// Король и огонёк (с 0.97): в магазине появляются огненные золотые фигуры (`unlocks`).
    KingFlame,
    /// Король и молния: электрические золотые.
    KingBolt,
    /// Король и смерч: воздушные золотые.
    KingWind,
    /// Король и тёмный огонёк: теневые золотые.
    KingDark,
    /// Король и снежинка: ледяные золотые.
    KingSnow,
    /// Король и шляпа: золотые маги.
    KingHat,
    /// Шляпа и огонёк (с 0.98): в магазине появляются огненные маги (`unlocks`).
    HatFlame,
    /// Шляпа и молния: электрические маги.
    HatBolt,
    /// Шляпа и смерч: воздушные маги.
    HatWind,
    /// Шляпа и тёмный огонёк: теневые маги.
    HatDark,
    /// Шляпа и снежинка: ледяные маги.
    HatSnow,
    /// Огонёк и снежинка (с 0.99): в магазине появляются огненные ледяные фигуры (`unlocks`).
    FlameSnow,
    /// Снежинка и молния: электрические ледяные.
    SnowBolt,
    /// Снежинка и смерч: воздушные ледяные.
    SnowWind,
    /// Снежинка и тёмный огонёк: теневые ледяные.
    SnowDark,
    /// Огонёк и молния (с 0.101): в магазине появляются огненные электрические фигуры
    /// (`unlocks`).
    FlameBolt,
    /// Огонёк и смерч: огненные воздушные.
    FlameWind,
    /// Огонёк и тёмный огонёк: огненные теневые.
    FlameDark,
    /// Молния и смерч (с 0.102): в магазине появляются электрические воздушные фигуры
    /// (`unlocks`).
    BoltWind,
    /// Молния и тёмный огонёк: электрические теневые.
    BoltDark,
    /// Смерч и тёмный огонёк (с 0.103): в магазине появляются воздушные теневые фигуры
    /// (`unlocks`).
    WindDark,
}

/// Сколько всего гербов (длина `Rogue::crests`).
pub const CRESTS: usize = 80;

impl Crest {
    pub const ALL: [Crest; CRESTS] = [
        Crest::Heart,
        Crest::Coin,
        Crest::Snow2,
        Crest::Pawn,
        Crest::Spikes,
        Crest::Crystal,
        Crest::AxeSword,
        Crest::AxeFangs,
        Crest::AxeQueen,
        Crest::Axes,
        Crest::Whirl,
        Crest::FireAxe,
        Crest::FlamePawn,
        Crest::Flames,
        Crest::FlameSkull,
        Crest::FlameSword,
        Crest::FlameCoin,
        Crest::Bomb,
        Crest::DarkRook,
        Crest::DarkSkull,
        Crest::DarkFlames,
        Crest::DarkPortal,
        Crest::DarkCoin,
        Crest::DarkEye,
        Crest::Winds,
        Crest::WindCoin,
        Crest::WindKnight,
        Crest::WindSword,
        Crest::BoltBishop,
        Crest::BoltCoin,
        Crest::BoltPlus,
        Crest::Bolts,
        Crest::RedBolt,
        Crest::BoltOne,
        Crest::Pawns,
        Crest::Bishops,
        Crest::Knights,
        Crest::Rooks,
        Crest::Queens,
        Crest::RerollCoin,
        Crest::HatCoin,
        Crest::Hats,
        Crest::HatBishop,
        Crest::Meteor,
        Crest::HatMeteor,
        Crest::HatPlus,
        Crest::GoldKing,
        Crest::KingCoin,
        Crest::Kings2,
        Crest::KingQueen,
        Crest::KingSkull,
        Crest::Piggy,
        Crest::AxeFlame,
        Crest::AxeBolt,
        Crest::AxeWind,
        Crest::AxeKing,
        Crest::AxeDark,
        Crest::AxeSnow,
        Crest::AxeHat,
        Crest::KingFlame,
        Crest::KingBolt,
        Crest::KingWind,
        Crest::KingDark,
        Crest::KingSnow,
        Crest::KingHat,
        Crest::HatFlame,
        Crest::HatBolt,
        Crest::HatWind,
        Crest::HatDark,
        Crest::HatSnow,
        Crest::FlameSnow,
        Crest::SnowBolt,
        Crest::SnowWind,
        Crest::SnowDark,
        Crest::FlameBolt,
        Crest::FlameWind,
        Crest::FlameDark,
        Crest::BoltWind,
        Crest::BoltDark,
        Crest::WindDark,
    ];

    /// Какую объединённую фигуру открывает герб: (основа, вторая сила) из `COMBOS`.
    pub fn unlocks(self) -> Option<(Power, Power)> {
        let (base, with) = match self {
            Crest::AxeFlame => (Power::Barbarian, Power::Fire),
            Crest::AxeBolt => (Power::Barbarian, Power::Spark),
            Crest::AxeWind => (Power::Barbarian, Power::Wind),
            Crest::AxeKing => (Power::Barbarian, Power::Gold),
            Crest::AxeDark => (Power::Barbarian, Power::Shade),
            Crest::AxeSnow => (Power::Barbarian, Power::Ice),
            Crest::AxeHat => (Power::Barbarian, Power::Wizard),
            Crest::KingFlame => (Power::Gold, Power::Fire),
            Crest::KingBolt => (Power::Gold, Power::Spark),
            Crest::KingWind => (Power::Gold, Power::Wind),
            Crest::KingDark => (Power::Gold, Power::Shade),
            Crest::KingSnow => (Power::Gold, Power::Ice),
            Crest::KingHat => (Power::Gold, Power::Wizard),
            Crest::HatFlame => (Power::Wizard, Power::Fire),
            Crest::HatBolt => (Power::Wizard, Power::Spark),
            Crest::HatWind => (Power::Wizard, Power::Wind),
            Crest::HatDark => (Power::Wizard, Power::Shade),
            Crest::HatSnow => (Power::Wizard, Power::Ice),
            Crest::FlameSnow => (Power::Ice, Power::Fire),
            Crest::SnowBolt => (Power::Ice, Power::Spark),
            Crest::SnowWind => (Power::Ice, Power::Wind),
            Crest::SnowDark => (Power::Ice, Power::Shade),
            Crest::FlameBolt => (Power::Fire, Power::Spark),
            Crest::FlameWind => (Power::Fire, Power::Wind),
            Crest::FlameDark => (Power::Fire, Power::Shade),
            Crest::BoltWind => (Power::Spark, Power::Wind),
            Crest::BoltDark => (Power::Spark, Power::Shade),
            Crest::WindDark => (Power::Wind, Power::Shade),
            _ => return None,
        };
        Some((base, with))
    }

    pub fn title(self) -> &'static str {
        match self {
            Crest::Heart => "Герб снежинки и сердца",
            Crest::Coin => "Герб снежинки и монеты",
            Crest::Snow2 => "Герб двойной снежинки",
            Crest::Pawn => "Герб снежинки и пешки",
            Crest::Spikes => "Герб снежинки и шипов",
            Crest::Crystal => "Герб замороженной снежинки",
            Crest::AxeSword => "Герб топора и меча",
            Crest::AxeFangs => "Герб топора и клыков",
            Crest::AxeQueen => "Герб топора и ферзя",
            Crest::Axes => "Герб двух топоров",
            Crest::Whirl => "Герб топора и красного круга",
            Crest::FireAxe => "Герб огненного топора",
            Crest::FlamePawn => "Герб огня и пешки",
            Crest::Flames => "Герб двух огоньков",
            Crest::FlameSkull => "Герб огня и черепа",
            Crest::FlameSword => "Герб огня и меча",
            Crest::FlameCoin => "Герб огня и монеты",
            Crest::Bomb => "Герб падающей бомбы",
            Crest::DarkRook => "Герб теневого огня и ладьи",
            Crest::DarkSkull => "Герб теневого огня и черепа",
            Crest::DarkFlames => "Герб двух теневых огней",
            Crest::DarkPortal => "Герб теневого огня и портала",
            Crest::DarkCoin => "Герб теневого огня и монеты",
            Crest::DarkEye => "Герб теневого огня и глаза",
            Crest::Winds => "Герб двух ветров",
            Crest::WindCoin => "Герб ветра и монеты",
            Crest::WindKnight => "Герб ветра и коня",
            Crest::WindSword => "Герб ветра и меча",
            Crest::BoltBishop => "Герб молнии и слона",
            Crest::BoltCoin => "Герб молнии и монеты",
            Crest::BoltPlus => "Герб молнии и плюса",
            Crest::Bolts => "Герб двух молний",
            Crest::RedBolt => "Герб красной молнии",
            Crest::BoltOne => "Герб молнии и единицы",
            Crest::Pawns => "Герб двух пешек",
            Crest::Bishops => "Герб двух слонов",
            Crest::Knights => "Герб двух коней",
            Crest::Rooks => "Герб двух ладей",
            Crest::Queens => "Герб двух ферзей",
            Crest::RerollCoin => "Герб монеты и обновления",
            Crest::HatCoin => "Герб шляпы и монеты",
            Crest::Hats => "Герб двух шляп",
            Crest::HatBishop => "Герб шляпы и слона",
            Crest::Meteor => "Герб метеорита",
            Crest::HatMeteor => "Герб шляпы и метеорита",
            Crest::HatPlus => "Герб шляпы и плюса",
            Crest::GoldKing => "Герб короля",
            Crest::KingCoin => "Герб короля и монеты",
            Crest::Kings2 => "Герб двух королей",
            Crest::KingQueen => "Герб короля и ферзя",
            Crest::KingSkull => "Герб короля и черепа",
            Crest::Piggy => "Герб свиньи-копилки",
            Crest::AxeFlame => "Герб топора и огонька",
            Crest::AxeBolt => "Герб топора и молнии",
            Crest::AxeWind => "Герб топора и смерча",
            Crest::AxeKing => "Герб топора и короля",
            Crest::AxeDark => "Герб топора и тёмного огонька",
            Crest::AxeSnow => "Герб топора и снежинки",
            Crest::AxeHat => "Герб топора и шляпы",
            Crest::KingFlame => "Герб короля и огонька",
            Crest::KingBolt => "Герб короля и молнии",
            Crest::KingWind => "Герб короля и смерча",
            Crest::KingDark => "Герб короля и тёмного огонька",
            Crest::KingSnow => "Герб короля и снежинки",
            Crest::KingHat => "Герб короля и шляпы",
            Crest::HatFlame => "Герб шляпы и огонька",
            Crest::HatBolt => "Герб шляпы и молнии",
            Crest::HatWind => "Герб шляпы и смерча",
            Crest::HatDark => "Герб шляпы и тёмного огонька",
            Crest::HatSnow => "Герб шляпы и снежинки",
            Crest::FlameSnow => "Герб огонька и снежинки",
            Crest::SnowBolt => "Герб снежинки и молнии",
            Crest::SnowWind => "Герб снежинки и смерча",
            Crest::SnowDark => "Герб снежинки и тёмного огонька",
            Crest::FlameBolt => "Герб огонька и молнии",
            Crest::FlameWind => "Герб огонька и смерча",
            Crest::FlameDark => "Герб огонька и тёмного огонька",
            Crest::BoltWind => "Герб молнии и смерча",
            Crest::BoltDark => "Герб молнии и тёмного огонька",
            Crest::WindDark => "Герб смерча и тёмного огонька",
        }
    }

    /// Что даёт герб — числа из баланса.
    pub fn description(self) -> String {
        let rule = |rule: Rule| balance().rule(rule);
        match self {
            Crest::Heart => format!(
                "Ледяные фигуры получают +{} к здоровью.",
                rule(Rule::RogueCrestHeartHp)
            ),
            Crest::Coin => match rule(Rule::RogueCrestCoinPercent) {
                200 => "Ледяные фигуры появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс ледяных фигур в магазине — {p}% от обычного."),
            },
            Crest::Snow2 => {
                let n = rule(Rule::RogueCrestFreezeTurns);
                let word = match (n % 10, n % 100) {
                    (1, r) if r != 11 => "ход",
                    (2..=4, r) if !(12..=14).contains(&r) => "хода",
                    _ => "ходов",
                };
                format!("Ледяные фигуры замораживают ударившего на {n} {word}.")
            }
            Crest::Pawn => "Ледяные пешки стоят в магазине как обычные.".to_owned(),
            Crest::Spikes => match rule(Rule::RogueCrestSpikesPercent) {
                100 => "Ледяные фигуры возвращают ударившему полученный урон.".to_owned(),
                p => format!("Ледяные фигуры возвращают ударившему {p}% полученного урона."),
            },
            Crest::Crystal => format!(
                "Ледяная фигура, у которой осталось {} здоровья, вмерзает в лёд до вашего \
                 хода: пока во льду, урона не получает.",
                rule(Rule::RogueCrestCrystalHp)
            ),
            Crest::AxeSword => format!(
                "Варвары наносят на {} урона больше, но у них на {} здоровья меньше.",
                rule(Rule::RogueCrestAxeAttack),
                rule(Rule::RogueCrestAxeHp)
            ),
            Crest::AxeFangs => format!(
                "Варвар, убив фигуру, восстанавливает {} здоровья — раз за ход.",
                rule(Rule::RogueCrestFangsHeal)
            ),
            Crest::AxeQueen => format!(
                "Ферзь-варвар стоит в магазине на {} дешевле.",
                rule(Rule::RogueCrestQueenDiscount)
            ),
            Crest::Axes => match rule(Rule::RogueCrestAxesPercent) {
                200 => "Варвары появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс варваров в магазине — {p}% от обычного."),
            },
            Crest::Whirl => format!(
                "Вместо повторной атаки варвар бьёт круговым ударом: все враги в радиусе {} \
                 клеток получают {} урона.",
                rule(Rule::RogueCrestWhirlRadius),
                rule(Rule::RogueCrestWhirlDamage)
            ),
            Crest::FireAxe => format!(
                "Варвар, у которого осталось {} здоровья, после убийства может ударить ещё раз \
                 сверху.",
                rule(Rule::RogueCrestFireAxeHp)
            ),
            Crest::FlamePawn => format!(
                "Огненная пешка стоит в магазине на {} дешевле.",
                rule(Rule::RogueCrestFlamePawnDiscount)
            ),
            Crest::Flames => format!(
                "Взрыв огненной фигуры достаёт ещё и крестом — на {} клетки по прямым.",
                rule(Rule::RogueCrestFlamesReach)
            ),
            Crest::FlameSkull => "Огненная фигура, погибнув, взрывается.".to_owned(),
            Crest::FlameSword => format!(
                "Взрыв огненной фигуры наносит {} урона.",
                rule(Rule::RogueCrestFlameSwordDamage)
            ),
            Crest::FlameCoin => match rule(Rule::RogueCrestFlameCoinPercent) {
                200 => "Огненные фигуры появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс огненных фигур в магазине — {p}% от обычного."),
            },
            Crest::Bomb => format!(
                "Взрыв огненной фигуры наносит на {} урона больше, но задевает и ваши фигуры.",
                rule(Rule::RogueCrestBombDamage)
            ),
            Crest::DarkRook => format!(
                "Теневая ладья стоит в магазине на {} дешевле.",
                rule(Rule::RogueCrestDarkRookDiscount)
            ),
            Crest::DarkSkull => {
                "Теневая фигура бьёт своей атакой врагов, через которых прошла.".to_owned()
            }
            Crest::DarkFlames => match rule(Rule::RogueCrestDarkDodgeEvery) {
                3 => "Теневая фигура не получает каждый третий удар по ней.".to_owned(),
                n => format!("Теневая фигура не получает каждый {n}-й удар по ней."),
            },
            Crest::DarkPortal => format!(
                "Теневая фигура с {} здоровья перед смертельным ударом уходит порталом: меняется \
                 местами с ближайшей вашей обычной фигурой (не королём), удар принимает та.",
                rule(Rule::RogueCrestDarkPortalHp)
            ),
            Crest::DarkCoin => match rule(Rule::RogueCrestDarkCoinPercent) {
                200 => "Теневые фигуры появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс теневых фигур в магазине — {p}% от обычного."),
            },
            Crest::DarkEye => {
                "Теневая фигура бьёт сквозь фигуры — всех врагов в досягаемости.".to_owned()
            }
            Crest::Winds => {
                "Воздушная фигура ходит трижды за ход: второй и третий ход — бесплатно.".to_owned()
            }
            Crest::WindCoin => match rule(Rule::RogueCrestWindCoinPercent) {
                200 => "Воздушные фигуры появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс воздушных фигур в магазине — {p}% от обычного."),
            },
            Crest::WindKnight => format!(
                "Воздушный конь стоит в магазине на {} дешевле.",
                rule(Rule::RogueCrestWindKnightDiscount)
            ),
            Crest::WindSword => "Воздушная фигура бьёт одну из своих целей дважды.".to_owned(),
            Crest::BoltBishop => format!(
                "Электрический слон стоит в магазине на {} дешевле.",
                rule(Rule::RogueCrestBoltBishopDiscount)
            ),
            Crest::BoltCoin => match rule(Rule::RogueCrestBoltCoinPercent) {
                200 => "Электрические фигуры появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс электрических фигур в магазине — {p}% от обычного."),
            },
            Crest::BoltPlus => {
                "Молния электрической фигуры расходится от цели не только по диагоналям, но и \
                 по прямым."
                    .to_owned()
            }
            Crest::Bolts => {
                "Молния бьёт цепью: от задетой фигуры перескакивает на все связанные с ней \
                 вражеские фигуры, пока они идут подряд."
                    .to_owned()
            }
            Crest::RedBolt => format!(
                "Электрическая фигура копит заряд за убийства; каждые {} — молния летит на \
                 клетку дальше через пустые клетки (до +{}).",
                rule(Rule::RogueCrestRedBoltStep),
                rule(Rule::RogueCrestRedBoltMax)
            ),
            Crest::BoltOne => "Электрические фигуры бьют первыми.".to_owned(),
            Crest::Pawns | Crest::Bishops | Crest::Knights | Crest::Rooks | Crest::Queens => {
                let kinds = match self {
                    Crest::Pawns => "Пешки",
                    Crest::Bishops => "Слоны",
                    Crest::Knights => "Кони",
                    Crest::Rooks => "Ладьи",
                    _ => "Ферзи",
                };
                match rule(Rule::RogueCrestPairPercent) {
                    200 => format!("{kinds} появляются в магазине вдвое чаще."),
                    p => format!(
                        "Шанс, что в магазине появятся {}, — {p}% от обычного.",
                        kinds.to_lowercase()
                    ),
                }
            }
            Crest::HatCoin => match rule(Rule::RogueCrestHatCoinPercent) {
                200 => "Маги появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс магов в магазине — {p}% от обычного."),
            },
            Crest::Hats => "Маг выпускает два метеорита вместо одного.".to_owned(),
            Crest::HatBishop => format!(
                "Слон-маг стоит в магазине на {} дешевле.",
                rule(Rule::RogueCrestHatBishopDiscount)
            ),
            Crest::Meteor => format!(
                "Метеорит мага наносит {} урона.",
                rule(Rule::RogueCrestMeteorDamage)
            ),
            Crest::HatMeteor => format!(
                "Метеорит оставляет горящий кратер до вашего следующего хода: вошедшая в него \
                 фигура получает {} урона.",
                rule(Rule::RogueCrestCraterDamage)
            ),
            Crest::HatPlus => {
                "Метеорит бьёт крестом: клетку падения и четыре соседние по прямым.".to_owned()
            }
            Crest::GoldKing => format!(
                "Золотые фигуры приносят ещё {} за каждую победу.",
                coins(
                    rule(Rule::RogueCrestGoldKingBonus).clamp(0, 999) as u16,
                    true
                )
            ),
            Crest::KingCoin => "Фигуры из запаса продаются за полную цену в магазине.".to_owned(),
            Crest::Kings2 => match rule(Rule::RogueCrestKings2Percent) {
                200 => "Золотые фигуры появляются в магазине вдвое чаще.".to_owned(),
                p => format!("Шанс золотых фигур в магазине — {p}% от обычного."),
            },
            Crest::KingQueen => format!(
                "Золотой ферзь приносит ещё {} за каждую победу.",
                coins(
                    rule(Rule::RogueCrestKingQueenBonus).clamp(0, 999) as u16,
                    true
                )
            ),
            Crest::KingSkull => {
                "Погибшая золотая фигура возвращает золотом свою цену в магазине.".to_owned()
            }
            Crest::Piggy => format!(
                "В конце уровня за каждые {} золота каждая ваша золотая фигура — на доске и в \
                 запасе — приносит ещё монету.",
                rule(Rule::RogueCrestPiggyStep)
            ),
            Crest::AxeFlame
            | Crest::AxeBolt
            | Crest::AxeWind
            | Crest::AxeKing
            | Crest::AxeDark
            | Crest::AxeSnow
            | Crest::AxeHat
            | Crest::KingFlame
            | Crest::KingBolt
            | Crest::KingWind
            | Crest::KingDark
            | Crest::KingSnow
            | Crest::KingHat
            | Crest::HatFlame
            | Crest::HatBolt
            | Crest::HatWind
            | Crest::HatDark
            | Crest::HatSnow
            | Crest::FlameSnow
            | Crest::SnowBolt
            | Crest::SnowWind
            | Crest::SnowDark
            | Crest::FlameBolt
            | Crest::FlameWind
            | Crest::FlameDark
            | Crest::BoltWind
            | Crest::BoltDark
            | Crest::WindDark => {
                let (base, with) = self.unlocks().expect("герб открывает объединённую");
                let adjective = |power: Power| match power {
                    Power::Fire => "огненные",
                    Power::Spark => "электрические",
                    Power::Wind => "воздушные",
                    Power::Gold => "золотые",
                    Power::Shade => "теневые",
                    _ => "ледяные",
                };
                // «огненные ледяные фигуры», «золотые маги», «варвары-маги».
                let whose = |power: Power| match power {
                    Power::Fire => "огненных",
                    Power::Spark => "электрических",
                    Power::Wind => "воздушных",
                    Power::Gold => "золотых",
                    Power::Shade => "теневых",
                    Power::Ice => "ледяных",
                    Power::Barbarian => "варваров",
                    _ => "магов",
                };
                let plural = match (base, with) {
                    (Power::Barbarian, Power::Wizard) => "варвары-маги".to_owned(),
                    (Power::Barbarian, with) => format!("{} варвары", adjective(with)),
                    (Power::Gold, Power::Wizard) => "золотые маги".to_owned(),
                    (Power::Gold, with) => format!("{} золотые фигуры", adjective(with)),
                    (Power::Ice, with) => format!("{} ледяные фигуры", adjective(with)),
                    (Power::Fire, with) => format!("огненные {} фигуры", adjective(with)),
                    (Power::Spark, with) => format!("электрические {} фигуры", adjective(with)),
                    (Power::Wind, with) => format!("воздушные {} фигуры", adjective(with)),
                    (_, with) => format!("{} маги", adjective(with)),
                };
                format!(
                    "В магазине появляются {plural} — фигуры с силами обеих: на них действуют \
                     гербы и {}, и {}.",
                    whose(base),
                    whose(with)
                )
            }
            Crest::RerollCoin => format!(
                "Обновить фигуры в магазине стоит {} вместо {}.",
                coins(
                    rule(Rule::RogueCrestRerollCoinCost).clamp(0, 999) as u16,
                    false
                ),
                rule(Rule::RogueRerollCost)
            ),
        }
    }
}

/// Фигура игрока начала уровень в запасе.
const NO_ORIGIN: u8 = 64;

/// Варвар бьёт за конец хода не больше стольких раз подряд (каждый следующий — после убийства).
const BARBARIAN_ROUNDS: u8 = 16;

/// Наибольшее число мест на прилавке магазина (длина массива `Rogue::offers`).
pub const SHOP_MAX: usize = 5;

/// Что продаёт магазин (короля нет) и вес — тот же, что у фигур волн на первом уровне
/// (`Rule::RogueWeight*`, с 0.63).
fn shop_weights() -> [(Kind, u64); 5] {
    let rule = |rule: Rule| balance().rule(rule).max(0) as u64;
    [
        (Kind::Pawn, rule(Rule::RogueWeightPawn)),
        (Kind::Knight, rule(Rule::RogueWeightKnight)),
        (Kind::Bishop, rule(Rule::RogueWeightBishop)),
        (Kind::Rook, rule(Rule::RogueWeightRook)),
        (Kind::Queen, rule(Rule::RogueWeightQueen)),
    ]
}

/// Огненная и электрическая фигуры выходят на прилавок в столько раз реже обычной того же
/// вида.
const FIRE_RARITY: u64 = 6;

/// Rogue-like: цена фигуры в магазине — `Rule::RogueShopPercent` (с 0.61 — 200%) от золота
/// за её победу; огненная — ещё `Rule::RogueFirePercent` от этого, электрическая —
/// `Rule::RogueSparkPercent`.
pub fn shop_price(stock: Stock) -> u16 {
    let percent = |rule: Rule| balance().rule(rule).clamp(0, 1000) as u32;
    let mut price = u32::from(rogue_price(stock.kind)) * percent(Rule::RogueShopPercent) / 100;
    // Объединённая — своя наценка (`Rule::RogueComboPercent`).
    if stock.combo_index().is_some() {
        price = price * percent(Rule::RogueComboPercent) / 100;
        return price.min(u32::from(u16::MAX)) as u16;
    }
    match stock.power {
        Power::Plain => {}
        Power::Fire => price = price * percent(Rule::RogueFirePercent) / 100,
        Power::Spark => price = price * percent(Rule::RogueSparkPercent) / 100,
        Power::Wind => price = price * percent(Rule::RogueWindPercent) / 100,
        Power::Gold => price = price * percent(Rule::RogueGoldPiecePercent) / 100,
        Power::Shade => price = price * percent(Rule::RogueShadePercent) / 100,
        Power::Ice => price = price * percent(Rule::RogueIcePercent) / 100,
        Power::Barbarian => price = price * percent(Rule::RogueBarbarianPercent) / 100,
        Power::Wizard => price = price * percent(Rule::RogueWizardPercent) / 100,
    }
    price.min(u32::from(u16::MAX)) as u16
}

/// Rogue-like: полное здоровье коня (`Rule::RogueKnightHp`, с 0.78) — у обеих сторон.
pub fn rogue_knight_hp() -> i32 {
    balance().rule(Rule::RogueKnightHp).clamp(1, 99) * crate::balance::UNIT
}

/// Rogue-like: цена герба в магазине.
pub fn crest_price() -> u16 {
    balance().rule(Rule::RogueCrestPrice).clamp(0, 999) as u16
}

/// Rogue-like: сколько стоит обновить гербы.
pub fn crest_reroll_cost() -> u16 {
    balance().rule(Rule::RogueCrestRerollCost).clamp(0, 999) as u16
}

/// Rogue-like: за сколько фигуру продают из запаса (с 0.69): за цену обычной такой же в
/// магазине, золотую — за `Rule::RogueGoldSellPercent` (200%) от неё; огненная,
/// электрическая и воздушная — как обычная.
pub fn sell_price(stock: Stock) -> u16 {
    let plain = u32::from(shop_price(Stock::new(stock.kind, Power::Plain)));
    let price = if stock.has(Power::Gold) {
        plain * balance().rule(Rule::RogueGoldSellPercent).clamp(0, 1000) as u32 / 100
    } else {
        plain
    };
    price.min(u32::from(u16::MAX)) as u16
}

/// Rogue-like: золото за побеждённую фигуру волн: король-толстосум — `Rule::RogueBossGold`.
pub fn kill_gold(piece: Piece) -> u16 {
    if piece.boss {
        balance().rule(Rule::RogueBossGold).clamp(0, 999) as u16
    } else {
        rogue_price(piece.kind)
    }
}

/// Rogue-like: столько золота дают за побеждённую фигуру вида `kind`.
pub fn rogue_price(kind: Kind) -> u16 {
    let rule = match kind {
        Kind::Pawn => Rule::RogueGoldPawn,
        Kind::Knight => Rule::RogueGoldKnight,
        Kind::Bishop => Rule::RogueGoldBishop,
        Kind::Rook => Rule::RogueGoldRook,
        Kind::Queen => Rule::RogueGoldQueen,
        Kind::King => return 0,
    };
    balance().rule(rule).clamp(0, 999) as u16
}

/// «1 монета», «2 монеты», «5 монет»; `accusative` — «за 1 монету».
fn coins(n: u16, accusative: bool) -> String {
    let word = match (n % 10, n % 100) {
        (1, r) if r != 11 => {
            if accusative {
                "монету"
            } else {
                "монета"
            }
        }
        (2..=4, r) if !(12..=14).contains(&r) => "монеты",
        _ => "монет",
    };
    format!("{n} {word}")
}

/// Действие rogue-like вне доски.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RogueAction {
    /// Кнопка «Выиграть раунд»: фигуры волн на доске побеждены разом (за них — золото), волн
    /// уровня больше нет.
    WinRound { gold: u16 },
    /// Покупка фигуры в магазине — в запас.
    Buy { stock: Stock, price: u16 },
    /// Новые фигуры на прилавке.
    Reroll { price: u16 },
    /// Продажа фигуры из запаса (с 0.65) — за `sell_price`.
    Sell { stock: Stock, price: u16 },
    /// «Уйти» из магазина — к расстановке.
    Leave,
    /// Покупка герба (с 0.77).
    BuyCrest { crest: Crest, price: u16 },
    /// Новые гербы на прилавке.
    RerollCrests { price: u16 },
}

impl Rogue {
    /// Волн на уровне `level`: на первом — `Rule::RogueWaves`, дальше — на одну больше каждый.
    fn waves_at(level: u8) -> u8 {
        (balance().rule(Rule::RogueWaves) + i32::from(level) - 1).clamp(1, 255) as u8
    }
}

impl TurnEnd {
    /// Никого не задело: в списке ходов такой конец хода не пишется.
    pub fn is_quiet(&self) -> bool {
        self.strikes.is_empty()
            && self.poison.is_empty()
            && self.meteors.is_empty()
            && self.burned.is_empty()
            && self.blasts.iter().all(|b| b.hits.is_empty())
            && self.spawned.is_empty()
            && self.risen.is_empty()
            && self.marched.is_empty()
            && self.level_up.is_none()
            && self.planning.is_none()
    }
}

/// Rogue-like (с 0.105): могила погибшего зомби — какой фигурой он восстанет и через сколько
/// концов хода своей стороны.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Grave {
    pub kind: Option<Kind>,
    wait: u8,
    /// Ядовитый зомби (с 0.107) — восстанет ядовитым.
    venom: bool,
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
    /// Теневая фигура с гербом теневого огня и черепа ударила чужих, через которых прошла.
    pub passed: Vec<WaveHit>,
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
/// Удары короля-толстосума (rogue-like): 8 клеток вокруг и ещё по одной через клетку по
/// диагоналям, горизонтали и вертикали — 16, без преград.
const BOSS_STRIKES: [(i8, i8); 16] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
    (-2, -2),
    (2, -2),
    (-2, 2),
    (2, 2),
    (0, -2),
    (0, 2),
    (-2, 0),
    (2, 0),
];

fn strike_steps(piece: Piece) -> &'static [(i8, i8)] {
    if piece.boss {
        return &BOSS_STRIKES;
    }
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
    /// Огненные фигуры в запасе (rogue-like, с 0.63): [сторона][`Kind`], всегда готовы.
    fire_reserve: [[u8; 6]; 2],
    /// Электрические фигуры в запасе (с 0.64): [сторона][`Kind`], всегда готовы.
    spark_reserve: [[u8; 6]; 2],
    /// Воздушные фигуры в запасе (с 0.67): [сторона][`Kind`], всегда готовы.
    wind_reserve: [[u8; 6]; 2],
    /// Золотые фигуры в запасе (с 0.68): [сторона][`Kind`], всегда готовы.
    gold_reserve: [[u8; 6]; 2],
    /// Теневые фигуры в запасе (с 0.70): [сторона][`Kind`], всегда готовы.
    shade_reserve: [[u8; 6]; 2],
    /// Ледяные фигуры в запасе (с 0.72): [сторона][`Kind`], всегда готовы.
    ice_reserve: [[u8; 6]; 2],
    /// Варвары в запасе (с 0.80): [сторона][`Kind`], всегда готовы.
    barbarian_reserve: [[u8; 6]; 2],
    /// Rogue-like (с 0.105): могилы погибших зомби по клеткам.
    graves: [Grave; 64],
    /// Rogue-like (с 0.106): клетки вампиров в броне, уже лечившихся ударом в этот ход.
    fed: u64,
    /// Объединённые фигуры в запасе (с 0.96): [номер в `COMBOS`][сторона][`Kind`], готовы.
    combo_reserve: [[[u8; 6]; 2]; COMBOS.len()],
    /// Маги в запасе (с 0.83): [сторона][`Kind`], всегда готовы.
    wizard_reserve: [[u8; 6]; 2],
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
    /// Воздушные фигуры, сходившие в этом ходу один раз (rogue-like, с 0.67): второй ход —
    /// бесплатный, даже если ходы кончились.
    airborne: u64,
    /// Воздушные фигуры, уже сделавшие бесплатный ход (герб двух ветров, с 0.90): третий ход —
    /// тоже бесплатный, потом всё.
    twice: u64,
    /// Горящие кратеры метеоритов (герб шляпы и метеорита, с 0.94) — до хода игрока.
    craters: u64,
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
            fire_reserve: [[0; 6]; 2],
            spark_reserve: [[0; 6]; 2],
            wind_reserve: [[0; 6]; 2],
            gold_reserve: [[0; 6]; 2],
            shade_reserve: [[0; 6]; 2],
            ice_reserve: [[0; 6]; 2],
            barbarian_reserve: [[0; 6]; 2],
            graves: [Grave::default(); 64],
            fed: 0,
            combo_reserve: [[[0; 6]; 2]; COMBOS.len()],
            wizard_reserve: [[0; 6]; 2],
            king_lost: [false; 2],
            advance: false,
            rogue: None,
            moves_made: 0,
            used: 0,
            airborne: 0,
            twice: 0,
            craters: 0,
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
        // Rogue-like: за побеждённую фигуру волны игроку — золото.
        if let Some(r) = &mut self.rogue
            && killer == r.player
            && victim.color != r.player
        {
            r.gold = r.gold.saturating_add(kill_gold(victim));
        }
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

    /// Фигура на `sq` может ещё сходить в этом ходу: не ходила, и ходы остались — или она
    /// воздушная и сходила только раз.
    pub fn can_move(&self, sq: Square) -> bool {
        !self.used(sq) && (self.moves_left() > 0 || self.airborne & 1 << sq != 0)
    }

    /// Rogue-like: волны соперника; `None` — обычная партия.
    pub fn rogue(&self) -> Option<Rogue> {
        self.rogue
    }

    /// Проиграла ли сторона `color`: нет фигур или погиб король. В rogue-like соперник
    /// проигрывает, только когда вышли все волны и фигур не осталось.
    pub fn defeated(&self, color: Color) -> bool {
        match self.rogue {
            Some(r) if r.player != color => {
                r.level >= r.levels && r.wave >= r.waves && !self.waves_left(color)
            }
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
        pos.reserve[player as usize][Kind::Pawn as usize][0][0] = 2;
        pos.give_stock(Position::starting_special(seed).piece(player));
        pos.rogue = Some(Rogue {
            player,
            wave: 0,
            waves: Rogue::waves_at(1),
            level: 1,
            levels: balance().rule(Rule::RogueLevels).clamp(1, 255) as u8,
            planning: false,
            origin: pos.origins(player),
            gold: 0,
            shop: false,
            offers: [None; SHOP_MAX],
            rolls: 0,
            crests: [None; CRESTS],
            crest_offers: [None; SHOP_MAX],
            crest_rolls: 0,
            seed: seed | 1,
        });
        for sq in 0..64 {
            if let Some(piece) = pos.board[sq] {
                pos.board[sq] = Some(pos.crested(piece));
            }
        }
        pos.spawn_wave();
        pos
    }

    /// Rogue-like (с 0.104): особая фигура в запасе игрока в начале забега — вместо прежних слона и
    /// коня. Вид — по весам волн (`shop_weights`, без пешки: ферзь реже всех), сила — любая из
    /// восьми поровну; случайность — зерно партии.
    fn starting_special(seed: u64) -> Stock {
        let mut state = (seed ^ 0x5EED_0F5E_C1A1_F16E).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut next = move |n: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % n.max(1)
        };
        let kinds: Vec<(Kind, u64)> = shop_weights()
            .into_iter()
            .filter(|&(kind, w)| kind != Kind::Pawn && w > 0)
            .collect();
        let total: u64 = kinds.iter().map(|&(_, w)| w).sum();
        let mut roll = next(total);
        let kind = kinds
            .iter()
            .find(|&&(_, w)| {
                let hit = roll < w;
                roll = roll.saturating_sub(w);
                hit
            })
            .map_or(Kind::Knight, |&(kind, _)| kind);
        let powers = [
            Power::Fire,
            Power::Spark,
            Power::Wind,
            Power::Gold,
            Power::Shade,
            Power::Ice,
            Power::Barbarian,
            Power::Wizard,
        ];
        Stock::new(kind, powers[next(powers.len() as u64) as usize])
    }

    /// Следующая волна соперника выходит на его крайнюю горизонталь: от `Rule::RogueWaveMin`
    /// до `Rule::RogueWaveMax` фигур (сколько поместится на свободные клетки), вид — по весам
    /// `Rule::RogueWeight*` (дорогие — реже). Крайняя горизонталь занята — фигура выходит на
    /// ближайшую со свободной клеткой (с 0.55), до второй горизонтали игрока. Не поместилась
    /// ни одна — волна ждёт следующего хода и не считается. Случайность — от зерна партии и ключа позиции: при отмене и в
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
            // С каждым уровнем пешки выходят реже.
            (
                Kind::Pawn,
                rule(Rule::RogueWeightPawn)
                    .saturating_sub(rule(Rule::RoguePawnDrop) * u64::from(r.level - 1)),
            ),
            (Kind::Knight, rule(Rule::RogueWeightKnight)),
            (Kind::Bishop, rule(Rule::RogueWeightBishop)),
            (Kind::Rook, rule(Rule::RogueWeightRook)),
            (Kind::Queen, rule(Rule::RogueWeightQueen)),
        ];
        let total: u64 = weights.iter().map(|w| w.1).sum();
        let wall = self.wall(enemy);
        let back = enemy.home_rank();
        let ranks: Vec<i8> = (0..7).map(|k| back + k * enemy.forward()).collect();
        let mut spawned = Vec::new();
        // Последняя волна первого уровня ведёт короля-толстосума (с 0.73) — ближе к середине
        // крайней горизонтали (занята — ближе к игроку); выходит первым.
        if r.level == 1 && r.wave + 1 == r.waves {
            let files = [4, 3, 5, 2, 6, 1, 7, 0];
            let spot = ranks.iter().find_map(|&rank| {
                files
                    .iter()
                    .filter_map(|&file| square(file, rank))
                    .find(|&sq| self.open(sq, wall))
            });
            if let Some(sq) = spot {
                let mut boss = Piece {
                    boss: true,
                    ..Piece::new(Kind::King, enemy)
                };
                boss.hp = boss.max_hp();
                self.board[sq as usize] = Some(boss);
                spawned.push((sq, Kind::King));
            }
        }
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
            // Крайняя занята (фигуры игрока не дают выйти) — на ближайшую горизонталь со
            // свободной клеткой: вторую с края, третью и так далее, но не на первую игрока.
            let Some(free) = ranks.iter().find_map(|&rank| {
                let free: Vec<Square> = (0..8)
                    .filter_map(|file| square(file, rank))
                    .filter(|&sq| self.open(sq, wall))
                    .collect();
                (!free.is_empty()).then_some(free)
            }) else {
                break;
            };
            let sq = free[next(free.len() as u64) as usize];
            // На уровне `Rule::RoguePoisonLevel` фигура выходит ядовитой с шансом
            // `Rule::RoguePoisonPercent` (с 0.79).
            let mut venom = i32::from(r.level) == balance().rule(Rule::RoguePoisonLevel)
                && (next(100) as i32) < balance().rule(Rule::RoguePoisonPercent);
            // У каждого уровня — свой вид усиленных (с 0.105.1, просьба пользователя: половина
            // волны обычные, половина — те, что заданы для уровня, других нет): на уровне
            // `Rule::RogueArmorLevel` (с волны `RogueArmorWave`) — бронированные, на
            // `RogueVampireLevel` — вампиры, на `RogueZombieLevel` — зомби; шанс — свой `…Percent`.
            let (mut armored, mut vampire, mut zombie) = (false, false, false);
            let mut on = |level: Rule, wave: Rule, percent: Rule| {
                i32::from(r.level) == balance().rule(level)
                    && i32::from(r.wave) + 1 >= balance().rule(wave)
                    && (next(100) as i32) < balance().rule(percent).clamp(0, 100)
            };
            if !venom {
                armored = on(
                    Rule::RogueArmorLevel,
                    Rule::RogueArmorWave,
                    Rule::RogueArmorPercent,
                );
                vampire = !armored
                    && on(
                        Rule::RogueVampireLevel,
                        Rule::RogueVampireWave,
                        Rule::RogueVampirePercent,
                    );
                zombie = !armored
                    && !vampire
                    && on(
                        Rule::RogueZombieLevel,
                        Rule::RogueZombieWave,
                        Rule::RogueZombiePercent,
                    );
                // Вампир в броне (с 0.106) — объединённая фигура: и бронированная, и вампир.
                if !armored
                    && !vampire
                    && !zombie
                    && on(
                        Rule::RogueArmoredVampireLevel,
                        Rule::RogueArmoredVampireWave,
                        Rule::RogueArmoredVampirePercent,
                    )
                {
                    (armored, vampire) = (true, true);
                }
                // Ядовитый зомби (с 0.107): яд и зомби вместе — восстаёт тоже ядовитым.
                if !armored
                    && !vampire
                    && !zombie
                    && on(
                        Rule::RogueVenomZombieLevel,
                        Rule::RogueVenomZombieWave,
                        Rule::RogueVenomZombiePercent,
                    )
                {
                    (venom, zombie) = (true, true);
                }
            }
            self.board[sq as usize] = Some(self.crested(Piece {
                venom,
                armored,
                vampire,
                zombie,
                ..Piece::new(kind, enemy)
            }));
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
        // Rogue-like: кнопка «Начать раунд» кончает расстановку.
        if self.planning() {
            return self.start_round();
        }
        let us = self.side;
        let gold_before = self.rogue.map_or(0, |r| r.gold);
        let strikes = self.strike_all();
        let mut blasts = self.lava_blasts();
        let meteors = self.meteors();
        let poison = self.poison_tick();
        // Герб огня и черепа: погибшие огненные фигуры взрываются (после ударов, взрывов и яда).
        let dead: Vec<(Square, Piece)> = strikes
            .iter()
            .flat_map(|s| s.wounded().copied())
            .chain(poison.iter().copied())
            .filter(|h| h.killed)
            .map(|h| (h.square, h.target))
            .chain(
                blasts
                    .iter()
                    .flat_map(|b| b.hits.iter())
                    .filter(|h| h.killed)
                    .map(|h| (h.square, h.target)),
            )
            .collect();
        let deaths = self.death_blasts(dead);
        blasts.extend(deaths);
        // Rogue-like: фигуры волн сами не ходят — в конце хода сначала бьют, потом шагают
        // вперёд; новая волна выходит после шага и в этот ход не бьёт. Шагнувшие в горящий
        // кратер — горят.
        let marched = self.march();
        let burned = self.crater_burns(&marched);
        // Rogue-like: зомби восстают из могил после шага волн (до новой волны).
        let risen = if self.rogue.is_some_and(|r| r.player != us) {
            self.rise_zombies()
        } else {
            Vec::new()
        };
        let gold = self.rogue.map_or(0, |r| r.gold) - gold_before;
        // Rogue-like: в конце хода соперника на его крайнюю горизонталь выходит новая волна.
        // Игрок отбил все волны уровня — расстановка перед следующим: ход остаётся у него.
        let (mut planning, mut returned, mut interest) = (None, Vec::new(), 0);
        let spawned = match self.rogue {
            Some(r) if r.player != us => self.spawn_wave(),
            Some(r)
                if r.level < r.levels && r.wave >= r.waves && !self.waves_left(us.opponent()) =>
            {
                // Свинья-копилка — по золоту и золотым фигурам на конец уровня.
                interest = self.piggy_bank();
                if let Some(r) = &mut self.rogue {
                    r.gold = r.gold.saturating_add(interest);
                }
                returned = self.begin_planning();
                planning = Some(r.level + 1);
                Vec::new()
            }
            _ => Vec::new(),
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
        self.side = if planning.is_some() {
            us
        } else {
            us.opponent()
        };
        // Горящие кратеры гаснут к ходу игрока (продержались ход соперника).
        if self.rogue.is_some_and(|r| r.player == self.side) {
            self.craters = 0;
        }
        self.thaw(self.side);

        self.tick_reserve(self.side);
        // Зоны новой стороны продержались ход соперника; ледяная стена — ещё на ход.
        self.ice[self.side as usize] = std::mem::take(&mut self.ice_later[self.side as usize]);
        self.barrier[self.side as usize] = 0;
        self.moved = false;
        self.moves_made = 0;
        self.used = 0;
        self.airborne = 0;
        self.twice = 0;
        self.fed = 0;
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
            level_up: None,
            planning,
            returned,
            gold,
            poison,
            meteors,
            burned,
            interest,
            risen,
        }
    }

    /// Rogue-like: у соперника ещё есть фигуры или могилы зомби, которые восстанут.
    fn waves_left(&self, color: Color) -> bool {
        self.has_pieces(color) || self.graves.iter().any(|g| g.kind.is_some())
    }

    /// Rogue-like: могила зомби на клетке `sq` — каким видом он восстанет.
    pub fn grave(&self, sq: Square) -> Option<Kind> {
        self.graves[sq as usize].kind
    }

    /// Rogue-like: зомби, чья очередь подошла, восстают с `Rule::RogueZombieRiseHp` здоровья — на
    /// своей клетке, а занята — на ближайшей свободной рядом (нет такой — ждут ещё ход). Второй
    /// раз восставший не восстанет.
    fn rise_zombies(&mut self) -> Vec<(Square, Kind)> {
        let Some(r) = self.rogue else {
            return Vec::new();
        };
        let enemy = r.player.opponent();
        let wall = self.wall(enemy);
        let hp = balance().rule(Rule::RogueZombieRiseHp).clamp(1, 99) * crate::balance::UNIT;
        let mut risen = Vec::new();
        for sq in 0..64u8 {
            let grave = self.graves[sq as usize];
            let Some(kind) = grave.kind else {
                continue;
            };
            if grave.wait > 1 {
                self.graves[sq as usize].wait -= 1;
                continue;
            }
            let (file, rank) = (file_of(sq), rank_of(sq));
            let spot = std::iter::once(sq)
                .chain(
                    KING_STEPS
                        .iter()
                        .filter_map(|&(df, dr)| square(file + df, rank + dr)),
                )
                .find(|&to| {
                    self.open(to, wall) && self.graves[to as usize].kind.is_none_or(|_| to == sq)
                });
            let Some(to) = spot else {
                continue;
            };
            self.graves[sq as usize] = Grave::default();
            let piece = self.crested(Piece {
                zombie: true,
                risen: true,
                fresh: true,
                venom: grave.venom,
                ..Piece::new(kind, enemy)
            });
            self.board[to as usize] = Some(Piece {
                hp: hp.min(piece.max_hp()),
                ..piece
            });
            risen.push((to, kind));
        }
        risen
    }

    /// Конец хода: каждый свободный (не замёрзший, не только что выставленный) маг стороны
    /// шлёт метеорит (с гербом двух шляп — два) на случайную клетку с чужой фигурой —
    /// `Rule::RogueWizardDamage` урона (с гербом метеорита — `Rule::RogueCrestMeteorDamage`); с
    /// гербом шляпы и плюса — ещё по соседним по прямым чужим, с гербом шляпы и метеорита —
    /// горящий кратер на клетке падения. Случайность — зерно партии, ключ позиции, клетка мага.
    fn meteors(&mut self) -> Vec<Meteor> {
        let us = self.side;
        let Some(r) = self.rogue else {
            return Vec::new();
        };
        let damage_rule = if self.crest_for(us, Crest::Meteor) {
            Rule::RogueCrestMeteorDamage
        } else {
            Rule::RogueWizardDamage
        };
        let raw = balance().rule(damage_rule).clamp(0, 99) * crate::balance::UNIT;
        let count = if self.crest_for(us, Crest::Hats) {
            2
        } else {
            1
        };
        let wide = self.crest_for(us, Crest::HatPlus);
        let crater = self.crest_for(us, Crest::HatMeteor);
        let wall = self.wall(us);
        let mut meteors = Vec::new();
        'wizards: for from in strike_queue() {
            if self.board[from as usize]
                .is_none_or(|p| p.color != us || !p.wizard || p.is_frozen() || p.fresh)
            {
                continue;
            }
            for n in 0..count {
                let targets: Vec<Square> = (0..64u8)
                    .filter(|&sq| self.board[sq as usize].is_some_and(|p| p.color != us))
                    .collect();
                if targets.is_empty() {
                    break 'wizards;
                }
                let mut state = (r.seed
                    ^ self.key()
                    ^ (u64::from(from) + 1 + 64 * n).wrapping_mul(0xA24B_AED4_963E_E407))
                    | 1;
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let at = targets[(state % targets.len() as u64) as usize];
                let mut squares = vec![at];
                if wide {
                    squares.extend(
                        ROOK_RAYS
                            .iter()
                            .filter_map(|&(df, dr)| square(file_of(at) + df, rank_of(at) + dr))
                            .filter(|&s| wall & 1 << s == 0),
                    );
                }
                let mut hits = Vec::new();
                for sq in squares {
                    let Some(target) = self.board[sq as usize].filter(|p| p.color != us) else {
                        continue;
                    };
                    let damage = self.damage_at(sq, raw);
                    let (_, killed) = self.wound(sq, damage);
                    if killed {
                        self.score_kill(target, us);
                    }
                    hits.push(Hit {
                        square: sq,
                        target,
                        damage,
                        killed,
                    });
                }
                if crater {
                    self.craters |= 1 << at;
                }
                meteors.push(Meteor {
                    from,
                    at,
                    hits,
                    wide,
                    crater,
                });
            }
        }
        meteors
    }

    /// Горящие кратеры метеоритов (герб шляпы и метеорита): маска клеток.
    pub fn craters(&self) -> u64 {
        self.craters
    }

    /// Фигуры, шагнувшие (`marched`) в горящий кратер, получают `Rule::RogueCrestCraterDamage`;
    /// за погибших — золото игроку.
    fn crater_burns(&mut self, marched: &[(Square, Square)]) -> Vec<Hit> {
        let Some(r) = self.rogue else {
            return Vec::new();
        };
        let raw = balance().rule(Rule::RogueCrestCraterDamage).clamp(0, 99) * crate::balance::UNIT;
        let mut burned = Vec::new();
        for &(_, to) in marched {
            if self.craters & 1 << to == 0 {
                continue;
            }
            let Some(target) = self.board[to as usize] else {
                continue;
            };
            let damage = self.damage_at(to, raw);
            let (_, killed) = self.wound(to, damage);
            if killed {
                self.score_kill(target, r.player);
            }
            burned.push(Hit {
                square: to,
                target,
                damage,
                killed,
            });
        }
        burned
    }

    /// Конец хода: отравленные фигуры стороны на шаг ближе к урону яда; дошли — получают
    /// `Rule::RoguePoisonDamage` (могут погибнуть), яд уходит.
    fn poison_tick(&mut self) -> Vec<Hit> {
        let us = self.side;
        let raw = balance().rule(Rule::RoguePoisonDamage).clamp(0, 99) * crate::balance::UNIT;
        let mut hits = Vec::new();
        for sq in 0..64u8 {
            let Some(piece) = self.board[sq as usize].filter(|p| p.color == us && p.poison > 0)
            else {
                continue;
            };
            let poison = piece.poison - 1;
            self.board[sq as usize] = Some(Piece { poison, ..piece });
            if poison > 0 {
                continue;
            }
            let target = piece;
            let (damage, killed) = self.wound(sq, raw);
            if killed {
                self.score_kill(target, us.opponent());
            }
            hits.push(Hit {
                square: sq,
                target,
                damage,
                killed,
            });
        }
        hits
    }

    /// Rogue-like: открыт магазин.
    pub fn shop_open(&self) -> bool {
        self.rogue.is_some_and(|r| r.shop)
    }

    /// Rogue-like: прилавок магазина — `Rule::RogueShopSlots` мест (`None` — куплено).
    pub fn offers(&self) -> Vec<Option<Stock>> {
        let n = (balance()
            .rule(Rule::RogueShopSlots)
            .clamp(1, SHOP_MAX as i32)) as usize;
        self.rogue.map_or_else(Vec::new, |r| r.offers[..n].to_vec())
    }

    /// Rogue-like: прилавок заполняется заново — по весам фигур волн первого уровня
    /// (`shop_weights`), огненная фигура — в `FIRE_RARITY` раз реже обычной того же вида.
    /// Случайность — зерно партии и счёт заполнений: при отмене прилавок тот же.
    fn roll_offers(&mut self) {
        let Some(mut r) = self.rogue else {
            return;
        };
        r.rolls = r.rolls.wrapping_add(1);
        let mut state = (r.seed
            ^ (u64::from(r.rolls) << 8 | u64::from(r.level)).wrapping_mul(0xD1B5_4A32_D192_ED03))
            | 1;
        let mut next = move |n: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % n.max(1)
        };
        // Гербы пар (две пешки, два слона…): этот вид — чаще, на все силы сразу.
        let pair = balance().rule(Rule::RogueCrestPairPercent).clamp(0, 1000) as u64;
        let paired = |kind: Kind| {
            let crest = match kind {
                Kind::Pawn => Crest::Pawns,
                Kind::Bishop => Crest::Bishops,
                Kind::Knight => Crest::Knights,
                Kind::Rook => Crest::Rooks,
                Kind::Queen => Crest::Queens,
                Kind::King => return 100,
            };
            if self.has_crest(crest) { pair } else { 100 }
        };
        // Объединённые, открытые гербами, — с весом особой (`Rule::RogueComboWeightPercent`) и
        // множителями обеих сил.
        let combo = balance().rule(Rule::RogueComboWeightPercent).clamp(0, 1000) as u64;
        let combos: Vec<(Power, Power)> =
            COMBOS.into_iter().filter(|&c| self.combo_open(c)).collect();
        let weights: Vec<(Stock, u64)> = shop_weights()
            .into_iter()
            .map(|(kind, w)| (kind, w * paired(kind)))
            .flat_map(|(kind, w)| {
                let mut row = vec![(Stock::new(kind, Power::Plain), w * FIRE_RARITY * 100)];
                for power in [
                    Power::Fire,
                    Power::Spark,
                    Power::Wind,
                    Power::Gold,
                    Power::Shade,
                    Power::Ice,
                    Power::Barbarian,
                    Power::Wizard,
                ] {
                    // Магом пешка не бывает.
                    let w = if power == Power::Wizard && kind == Kind::Pawn {
                        0
                    } else {
                        w * self.power_percent(power)
                    };
                    row.push((Stock::new(kind, power), w));
                }
                for &(base, with) in &combos {
                    let w = if (base == Power::Wizard || with == Power::Wizard)
                        && kind == Kind::Pawn
                    {
                        0
                    } else {
                        w * combo * self.power_percent(base) / 100 * self.power_percent(with) / 100
                    };
                    row.push((Stock::combo(kind, base, with), w));
                }
                row
            })
            .collect();
        let total: u64 = weights.iter().map(|w| w.1).sum();
        for offer in &mut r.offers {
            let mut roll = next(total);
            *offer = weights
                .iter()
                .find(|&&(_, w)| {
                    let hit = roll < w;
                    roll = roll.saturating_sub(w);
                    hit
                })
                .map(|&(stock, _)| stock)
                .or(Some(Stock::new(Kind::Pawn, Power::Plain)));
        }
        self.rogue = Some(r);
    }

    /// Rogue-like: покупка фигуры с места `slot` прилавка — в запас игрока, готовой.
    fn buy(&mut self, slot: usize) -> Option<RogueAction> {
        let mut r = self.rogue.filter(|r| r.shop)?;
        let stock = (*self.offers().get(slot)?)?;
        let price = self.shop_price(stock);
        if r.gold < price {
            return None;
        }
        r.gold -= price;
        r.offers[slot] = None;
        self.rogue = Some(r);
        self.give_stock(stock.piece(r.player));
        Some(RogueAction::Buy { stock, price })
    }

    /// Rogue-like: новые фигуры на прилавке за `Rule::RogueRerollCost` золота.
    fn reroll(&mut self) -> Option<RogueAction> {
        let mut r = self.rogue.filter(|r| r.shop)?;
        let price = self.reroll_price();
        if r.gold < price {
            return None;
        }
        r.gold -= price;
        self.rogue = Some(r);
        self.roll_offers();
        Some(RogueAction::Reroll { price })
    }

    /// Rogue-like: продажа готовой фигуры `stock` из запаса игрока — за цену в магазине.
    fn sell(&mut self, stock: Stock) -> Option<RogueAction> {
        let r = self.rogue.filter(|r| r.shop)?;
        if !self.stock_ready(r.player, stock) {
            return None;
        }
        self.take_stock(r.player, stock);
        let price = self.sell_price(stock);
        let mut r = self.rogue?;
        r.gold = r.gold.saturating_add(price);
        self.rogue = Some(r);
        Some(RogueAction::Sell { stock, price })
    }

    /// Rogue-like: цена фигуры `stock` в магазине с гербами игрока (ледяная пешка с гербом
    /// снежинки и пешки — как обычная).
    pub fn shop_price(&self, stock: Stock) -> u16 {
        if stock.kind == Kind::Pawn && stock.has(Power::Ice) && self.has_crest(Crest::Pawn) {
            // Как такая же без льда: обычная — у ледяной, другая сила — у объединённой (ледяной
            // варвар — как варвар, огненная ледяная — как огненная).
            let other = if stock.power == Power::Ice {
                stock.with
            } else {
                stock.power
            };
            return shop_price(Stock::new(Kind::Pawn, other));
        }
        if stock.kind == Kind::Pawn && stock.has(Power::Fire) && self.has_crest(Crest::FlamePawn) {
            let discount = balance()
                .rule(Rule::RogueCrestFlamePawnDiscount)
                .clamp(0, 999) as u16;
            return shop_price(stock).saturating_sub(discount);
        }
        if stock.kind == Kind::Bishop
            && stock.has(Power::Wizard)
            && self.has_crest(Crest::HatBishop)
        {
            let discount = balance()
                .rule(Rule::RogueCrestHatBishopDiscount)
                .clamp(0, 999) as u16;
            return shop_price(stock).saturating_sub(discount);
        }
        if stock.kind == Kind::Bishop
            && stock.has(Power::Spark)
            && self.has_crest(Crest::BoltBishop)
        {
            let discount = balance()
                .rule(Rule::RogueCrestBoltBishopDiscount)
                .clamp(0, 999) as u16;
            return shop_price(stock).saturating_sub(discount);
        }
        if stock.kind == Kind::Knight && stock.has(Power::Wind) && self.has_crest(Crest::WindKnight)
        {
            let discount = balance()
                .rule(Rule::RogueCrestWindKnightDiscount)
                .clamp(0, 999) as u16;
            return shop_price(stock).saturating_sub(discount);
        }
        if stock.kind == Kind::Rook && stock.has(Power::Shade) && self.has_crest(Crest::DarkRook) {
            let discount = balance()
                .rule(Rule::RogueCrestDarkRookDiscount)
                .clamp(0, 999) as u16;
            return shop_price(stock).saturating_sub(discount);
        }
        if stock.kind == Kind::Queen
            && stock.has(Power::Barbarian)
            && self.has_crest(Crest::AxeQueen)
        {
            let discount = balance().rule(Rule::RogueCrestQueenDiscount).clamp(0, 999) as u16;
            return shop_price(stock).saturating_sub(discount);
        }
        shop_price(stock)
    }

    /// Rogue-like: сколько стоит обновить фигуры в магазине — `Rule::RogueRerollCost`, с гербом
    /// монеты и обновления — `Rule::RogueCrestRerollCoinCost`.
    pub fn reroll_price(&self) -> u16 {
        let rule = if self.has_crest(Crest::RerollCoin) {
            Rule::RogueCrestRerollCoinCost
        } else {
            Rule::RogueRerollCost
        };
        balance().rule(rule).clamp(0, 999) as u16
    }

    /// Rogue-like: за сколько продаётся фигура `stock` из запаса: `sell_price`, с гербом короля и
    /// монеты — не меньше полной цены в магазине.
    pub fn sell_price(&self, stock: Stock) -> u16 {
        let price = sell_price(stock);
        if self.has_crest(Crest::KingCoin) {
            price.max(self.shop_price(stock))
        } else {
            price
        }
    }

    /// Rogue-like: золото свиньи-копилки к концу уровня — за каждые `Rule::RogueCrestPiggyStep`
    /// золота по монете за каждую золотую фигуру игрока на доске и в запасе.
    fn piggy_bank(&self) -> u16 {
        let Some(r) = self.rogue.filter(|_| self.has_crest(Crest::Piggy)) else {
            return 0;
        };
        let step = balance().rule(Rule::RogueCrestPiggyStep).clamp(1, 999) as u16;
        let on_board = self
            .board
            .iter()
            .flatten()
            .filter(|p| p.color == r.player && p.gold)
            .count();
        // В запасе — золотые и золотые варвары.
        let stored: usize = self
            .reserve(r.player)
            .iter()
            .filter(|(stock, ..)| stock.has(Power::Gold))
            .map(|&(_, ready, _)| ready as usize)
            .sum();
        (r.gold / step).saturating_mul((on_board + stored) as u16)
    }

    /// Rogue-like: шанс фигур силы `power` в магазине, % — с гербами «… и монеты» (и двух
    /// топоров, двух королей).
    fn power_percent(&self, power: Power) -> u64 {
        let (crest, rule) = match power {
            Power::Ice => (Crest::Coin, Rule::RogueCrestCoinPercent),
            Power::Fire => (Crest::FlameCoin, Rule::RogueCrestFlameCoinPercent),
            Power::Gold => (Crest::Kings2, Rule::RogueCrestKings2Percent),
            Power::Wizard => (Crest::HatCoin, Rule::RogueCrestHatCoinPercent),
            Power::Spark => (Crest::BoltCoin, Rule::RogueCrestBoltCoinPercent),
            Power::Wind => (Crest::WindCoin, Rule::RogueCrestWindCoinPercent),
            Power::Shade => (Crest::DarkCoin, Rule::RogueCrestDarkCoinPercent),
            Power::Barbarian => (Crest::Axes, Rule::RogueCrestAxesPercent),
            Power::Plain => return 100,
        };
        if self.has_crest(crest) {
            balance().rule(rule).clamp(0, 1000) as u64
        } else {
            100
        }
    }

    /// Rogue-like: открыта ли объединённая фигура `(основа, сила)` — куплен её герб (топор, король,
    /// шляпа, снежинка, огонёк, молния или смерч с этой силой).
    pub fn combo_open(&self, combo: (Power, Power)) -> bool {
        Crest::ALL
            .iter()
            .any(|&c| c.unlocks() == Some(combo) && self.has_crest(c))
    }

    /// Rogue-like: гербы игрока в порядке покупки.
    pub fn crests(&self) -> Vec<Crest> {
        self.rogue
            .map_or_else(Vec::new, |r| r.crests.iter().flatten().copied().collect())
    }

    /// Rogue-like: у игрока есть герб `crest`.
    pub fn has_crest(&self, crest: Crest) -> bool {
        self.rogue.is_some_and(|r| r.crests.contains(&Some(crest)))
    }

    /// Герб `crest` действует на фигуры стороны `color` — она игрок rogue-like и герб куплен.
    fn crest_for(&self, color: Color, crest: Crest) -> bool {
        self.rogue.is_some_and(|r| r.player == color) && self.has_crest(crest)
    }

    /// Rogue-like: гербы на прилавке — `Rule::RogueCrestSlots` мест (`None` — куплен или
    /// гербов не осталось).
    pub fn crest_offers(&self) -> Vec<Option<Crest>> {
        let n = (balance()
            .rule(Rule::RogueCrestSlots)
            .clamp(1, SHOP_MAX as i32)) as usize;
        self.rogue
            .map_or_else(Vec::new, |r| r.crest_offers[..n].to_vec())
    }

    /// Rogue-like: гербы на прилавке выкладываются заново — разные, из ещё не купленных.
    /// Случайность — зерно партии и счёт выкладываний: при отмене прилавок тот же.
    fn roll_crests(&mut self) {
        let Some(mut r) = self.rogue else {
            return;
        };
        r.crest_rolls = r.crest_rolls.wrapping_add(1);
        let mut state = (r.seed
            ^ (u64::from(r.crest_rolls) << 8 | u64::from(r.level))
                .wrapping_mul(0x9E37_79B9_7F4A_7C15))
            | 1;
        let mut next = move |n: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % n.max(1) as u64) as usize
        };
        let mut left: Vec<Crest> = Crest::ALL
            .into_iter()
            .filter(|c| !r.crests.contains(&Some(*c)))
            .collect();
        for offer in &mut r.crest_offers {
            *offer = (!left.is_empty()).then(|| left.remove(next(left.len())));
        }
        self.rogue = Some(r);
    }

    /// Rogue-like: покупка герба с места `slot` прилавка за `Rule::RogueCrestPrice`.
    fn buy_crest(&mut self, slot: usize) -> Option<RogueAction> {
        let mut r = self.rogue.filter(|r| r.shop)?;
        let crest = (*self.crest_offers().get(slot)?)?;
        let price = crest_price();
        if r.gold < price {
            return None;
        }
        r.gold -= price;
        r.crest_offers[slot] = None;
        *r.crests.iter_mut().find(|c| c.is_none())? = Some(crest);
        self.rogue = Some(r);
        // Герб снежинки и сердца — и ледяным фигурам, что уже стоят на доске.
        for sq in 0..64 {
            if let Some(piece) = self.board[sq] {
                self.board[sq] = Some(self.crested(piece));
            }
        }
        Some(RogueAction::BuyCrest { crest, price })
    }

    /// Rogue-like: новые гербы на прилавке за `Rule::RogueCrestRerollCost`.
    fn reroll_crests(&mut self) -> Option<RogueAction> {
        let mut r = self.rogue.filter(|r| r.shop)?;
        let price = crest_reroll_cost();
        if r.gold < price {
            return None;
        }
        r.gold -= price;
        self.rogue = Some(r);
        self.roll_crests();
        Some(RogueAction::RerollCrests { price })
    }

    /// Фигура `piece` по правилам rogue-like (прибавка к здоровью — `bonus_hp`, здоровье
    /// меняется на ту же разницу): конь — со здоровьем `rogue_knight_hp` (с 0.78), ледяная с
    /// гербом снежинки и сердца — ещё с `Rule::RogueCrestHeartHp`. Не rogue-like — как есть.
    fn crested(&self, mut piece: Piece) -> Piece {
        if self.rogue.is_none() || piece.boss {
            return piece;
        }
        let mut bonus = 0;
        if piece.kind == Kind::Knight {
            bonus += rogue_knight_hp() - piece.kind.stats().max_hp;
        }
        let unit = crate::balance::UNIT;
        if piece.armored && piece.vampire {
            bonus += balance().rule(Rule::RogueArmoredVampireHp).clamp(0, 99) * unit;
        } else if piece.armored {
            bonus += balance().rule(Rule::RogueArmorHp).clamp(0, 99) * unit;
        } else if piece.vampire {
            bonus += balance().rule(Rule::RogueVampireHp).clamp(0, 99) * unit;
        }
        if piece.zombie {
            bonus += balance().rule(Rule::RogueZombieHp).clamp(0, 99) * unit;
        }
        if piece.icy && self.crest_for(piece.color, Crest::Heart) {
            bonus += balance().rule(Rule::RogueCrestHeartHp).clamp(0, 99) * crate::balance::UNIT;
        }
        // Герб топора и меча: варвар бьёт сильнее, здоровья меньше (не меньше одной единицы).
        let mut attack = 0;
        if piece.barbarian && self.crest_for(piece.color, Crest::AxeSword) {
            let unit = crate::balance::UNIT;
            attack = balance().rule(Rule::RogueCrestAxeAttack).clamp(0, 99) * unit;
            bonus -= balance().rule(Rule::RogueCrestAxeHp).clamp(0, 99) * unit;
            bonus = bonus.max(unit - piece.kind.stats().max_hp);
        }
        let attack = attack.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
        if piece.bonus_attack != attack {
            piece.bonus_attack = attack;
        }
        let bonus = bonus.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
        if piece.bonus_hp != bonus {
            piece.hp = (piece.hp + i32::from(bonus) - i32::from(piece.bonus_hp)).max(1);
            piece.bonus_hp = bonus;
        }
        piece
    }

    /// Полное здоровье фигуры вида `kind` в этой партии (rogue-like — конь со своим).
    pub fn kind_max_hp(&self, kind: Kind) -> i32 {
        if self.rogue.is_some() && kind == Kind::Knight {
            rogue_knight_hp()
        } else {
            kind.stats().max_hp
        }
    }

    /// Rogue-like: «Уйти» из магазина — начинается расстановка.
    fn leave_shop(&mut self) -> Option<RogueAction> {
        let mut r = self.rogue.filter(|r| r.shop)?;
        r.shop = false;
        self.rogue = Some(r);
        Some(RogueAction::Leave)
    }

    /// Rogue-like: можно нажать «Выиграть раунд» — идёт бой, ходит игрок.
    pub fn can_win_round(&self) -> bool {
        self.rogue
            .is_some_and(|r| !r.planning && r.player == self.side)
    }

    /// Rogue-like: «Выиграть раунд» — фигуры волн на доске побеждены (за них — золото), волн
    /// на этом уровне больше не будет.
    fn win_round(&mut self) -> Option<RogueAction> {
        let mut r = self.rogue.filter(|_| self.can_win_round())?;
        let mut gold = 0u16;
        for sq in 0..64 {
            if let Some(p) = self.board[sq].filter(|p| p.color != r.player) {
                self.board[sq] = None;
                gold = gold.saturating_add(kill_gold(p));
            }
        }
        r.gold = r.gold.saturating_add(gold);
        r.wave = r.waves;
        self.rogue = Some(r);
        self.graves = [Grave::default(); 64];
        Some(RogueAction::WinRound { gold })
    }

    /// Rogue-like: идёт расстановка перед следующим уровнем.
    pub fn planning(&self) -> bool {
        self.rogue.is_some_and(|r| r.planning)
    }

    /// Метки начала уровня: каждая фигура стороны `player` начинает его на своей клетке.
    fn origins(&self, player: Color) -> [u8; 64] {
        std::array::from_fn(|sq| match self.board[sq] {
            Some(p) if p.color == player => sq as u8,
            _ => NO_ORIGIN,
        })
    }

    /// Rogue-like: уровень пройден — начинается расстановка. Выжившие фигуры игрока встают на
    /// клетки, где начинали уровень, с полным здоровьем; выставленные из запаса посреди уровня
    /// возвращаются в запас; погибшие не возвращаются. Возвращает (откуда, куда) для анимации.
    fn begin_planning(&mut self) -> Vec<(Square, Square)> {
        let Some(mut r) = self.rogue else {
            return Vec::new();
        };
        let player = r.player;
        let mut home = Vec::new();
        for sq in 0..64u8 {
            let Some(piece) = self.board[sq as usize].filter(|p| p.color == player) else {
                continue;
            };
            self.board[sq as usize] = None;
            match r.origin[sq as usize] {
                NO_ORIGIN => self.give_stock(Piece::new(piece.kind, player).with(piece)),
                origin => home.push((sq, origin, piece)),
            }
        }
        let mut returned = Vec::new();
        for (from, to, piece) in home {
            self.board[to as usize] =
                Some(self.crested(Piece::new(piece.kind, player).with(piece)));
            if piece.kind == Kind::King {
                self.kings[player as usize] = Some(to);
            }
            if from != to {
                returned.push((from, to));
            }
        }
        r.planning = true;
        r.shop = true;
        r.origin = self.origins(player);
        self.rogue = Some(r);
        self.roll_offers();
        self.roll_crests();
        returned
    }

    /// Rogue-like: «Начать раунд» — расстановка запоминается как начало уровня, уровень
    /// начинается, его первая волна выходит сразу; ходит игрок.
    fn start_round(&mut self) -> TurnEnd {
        let Some(mut r) = self.rogue else {
            return TurnEnd::default();
        };
        r.planning = false;
        r.level += 1;
        r.wave = 0;
        self.graves = [Grave::default(); 64];
        r.waves = Rogue::waves_at(r.level);
        r.origin = self.origins(r.player);
        self.rogue = Some(r);
        // Рокировка снова возможна, если король и ладья стоят на своих местах.
        let (short, long) = r.player.castling_bits();
        self.castling |= short | long;
        let spawned = self.spawn_wave();
        self.moved = false;
        self.moves_made = 0;
        self.used = 0;
        self.airborne = 0;
        self.twice = 0;
        TurnEnd {
            spawned,
            level_up: Some(r.level),
            ..TurnEnd::default()
        }
    }

    /// Куда при расстановке встаёт фигура вида `kind` стороны `us`: на две первые свои
    /// горизонтали (с 0.61), пешка — только на вторую.
    fn arrange_fits(us: Color, kind: Kind, sq: Square) -> bool {
        let row = (rank_of(sq) - us.home_rank()).abs();
        if kind == Kind::Pawn {
            row == 1
        } else {
            row < 2
        }
    }

    /// Действия расстановки: фигура — на пустую клетку или на место своей фигуры (обмен, если
    /// и та может встать на её клетку); фигура из запаса — на пустую клетку.
    fn arrange_moves(&self, out: &mut Vec<Move>) {
        let us = self.side;
        for from in 0..64u8 {
            let Some(piece) = self.board[from as usize].filter(|p| p.color == us) else {
                continue;
            };
            for to in (0..64u8).filter(|&to| to != from && Self::arrange_fits(us, piece.kind, to)) {
                let free = match self.board[to as usize] {
                    None => true,
                    Some(other) => other.color == us && Self::arrange_fits(us, other.kind, from),
                };
                if free {
                    out.push(Move::new(from, to, MoveKind::Quiet));
                }
            }
        }
        for stock in self.reserve(us).into_iter().map(|s| s.0) {
            if self.stock_ready(us, stock) {
                out.extend(
                    (0..64u8)
                        .filter(|&sq| {
                            self.board[sq as usize].is_none()
                                && Self::arrange_fits(us, stock.kind, sq)
                        })
                        .map(|sq| Move::new(sq, sq, stock.drop_kind())),
                );
            }
        }
    }

    /// Действие расстановки: ходов не тратит, никто не перехватывает. Обмен — фигура с клетки
    /// `to` встаёт на `from` (в `Outcome::rook`, анимация — как у ладьи рокировки).
    fn arrange(&mut self, mv: Move) -> Outcome {
        let us = self.side;
        let mut outcome = Outcome {
            mv,
            piece: Piece::new(Kind::Pawn, us),
            rook: None,
            promoted: None,
            frozen: false,
            reactions: Vec::new(),
            passed: Vec::new(),
        };
        if let Some(stock) = mv.kind.stock() {
            let piece = self.take_stock(us, stock);
            self.board[mv.to as usize] = Some(piece);
            outcome.piece = piece;
            return outcome;
        }
        outcome.piece = self.board[mv.from as usize].expect("расстановка с пустой клетки");
        if self.board[mv.to as usize].is_some() {
            self.board.swap(mv.from as usize, mv.to as usize);
            outcome.rook = Some((mv.to, mv.from));
        } else {
            self.relocate(mv.from, mv.to);
        }
        for sq in [mv.from, mv.to] {
            if self.board[sq as usize].is_some_and(|p| p.kind == Kind::King && p.color == us) {
                self.kings[us as usize] = Some(sq);
            }
        }
        outcome
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
                // Король-толстосум волн — не король стороны: шаха и гибели короля у неё нет.
                if piece.kind == Kind::King && !piece.boss {
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
            let Some(pawn) =
                self.board[center as usize].filter(|p| p.color == us && (p.lava || p.fire))
            else {
                continue;
            };
            if let Some(piece) = &mut self.board[center as usize] {
                piece.lava = false;
            }
            // Огненная фигура, только что выставленная, взорвётся в конце следующего хода.
            if pawn.is_frozen() || pawn.fire && !pawn.lava && pawn.fresh {
                continue;
            }
            // Огненная фигура (с гербами своей стороны) взрывается, только если волна заденет
            // чужую фигуру.
            let crested = pawn.fire && !pawn.lava;
            if crested && !self.blast_reaches(center, us, wall) {
                continue;
            }
            let blast = self.explode(center, us, if crested { None } else { Some(raw) });
            blasts.push(blast);
        }
        blasts
    }

    /// Огненная волна стороны `us` из клетки `center`: урон `raw` (`None` — урон огненной фигуры
    /// с гербами `fire_damage`) чужим фигурам на клетках `blast_area`, с гербом падающей бомбы —
    /// и своим (кроме самой `center`). Барьер соперника — недосягаем.
    fn explode(&mut self, center: Square, us: Color, raw: Option<i32>) -> Blast {
        let crested = raw.is_none();
        let raw = raw.unwrap_or_else(|| self.fire_damage(us));
        let wall = self.wall(us);
        let friendly = crested && self.crest_for(us, Crest::Bomb);
        let mut hits = Vec::new();
        for sq in self.blast_area(center, us, crested) {
            let Some(target) =
                self.board[sq as usize].filter(|p| p.color != us || friendly && sq != center)
            else {
                continue;
            };
            if target.color != us && wall & 1 << sq != 0 {
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
        Blast {
            center,
            hits,
            wide: self.wide_blast(us, crested),
        }
    }

    /// Клетки взрыва из `center`: восемь вокруг; у огненной фигуры с гербом двух огоньков —
    /// толстый крест (с 0.87, просьба пользователя): 3×3 вокруг и ещё по три клетки с каждой
    /// стороны — до `Rule::RogueCrestFlamesReach` клеток от фигуры, полосой в три клетки.
    fn blast_area(&self, center: Square, us: Color, crested: bool) -> Vec<Square> {
        let mut steps: Vec<(i8, i8)> = KING_STEPS.to_vec();
        if self.wide_blast(us, crested) {
            let reach = balance().rule(Rule::RogueCrestFlamesReach).clamp(1, 7) as i8;
            steps = (-reach..=reach)
                .flat_map(|df| (-reach..=reach).map(move |dr| (df, dr)))
                .filter(|&(df, dr)| (df, dr) != (0, 0) && (df.abs() <= 1 || dr.abs() <= 1))
                .collect();
        }
        steps
            .into_iter()
            .filter_map(|(df, dr)| square(file_of(center) + df, rank_of(center) + dr))
            .collect()
    }

    /// Взрыв огненной фигуры стороны `us` — большой крест (герб двух огоньков).
    fn wide_blast(&self, us: Color, crested: bool) -> bool {
        crested && self.crest_for(us, Crest::Flames)
    }

    /// Урон взрыва огненной фигуры стороны `us`: `Rule::LavaDamage`, с гербом огня и меча —
    /// `Rule::RogueCrestFlameSwordDamage`, с гербом падающей бомбы — ещё
    /// `Rule::RogueCrestBombDamage` сверху.
    fn fire_damage(&self, us: Color) -> i32 {
        let unit = crate::balance::UNIT;
        let mut raw = if self.crest_for(us, Crest::FlameSword) {
            balance()
                .rule(Rule::RogueCrestFlameSwordDamage)
                .clamp(0, 99)
                * unit
        } else {
            balance().rule(Rule::LavaDamage)
        };
        if self.crest_for(us, Crest::Bomb) {
            raw += balance().rule(Rule::RogueCrestBombDamage).clamp(0, 99) * unit;
        }
        raw
    }

    /// Есть ли чужая фигура (вне барьера `wall`) на клетках взрыва огненной фигуры `us` из
    /// `center`.
    fn blast_reaches(&self, center: Square, us: Color, wall: u64) -> bool {
        self.blast_area(center, us, true)
            .into_iter()
            .any(|sq| wall & 1 << sq == 0 && self.board[sq as usize].is_some_and(|p| p.color != us))
    }

    /// Герб огня и черепа: погибшие огненные фигуры игрока (`dead` — клетка и фигура до
    /// гибели) взрываются там, где погибли; погибшие от этих взрывов свои огненные — тоже.
    fn death_blasts(&mut self, dead: Vec<(Square, Piece)>) -> Vec<Blast> {
        let mut blasts = Vec::new();
        let mut queue = dead;
        let mut done = 0u64;
        while let Some((sq, piece)) = queue.pop() {
            if !piece.fire
                || done & 1 << sq != 0
                || self.board[sq as usize].is_some()
                || !self.crest_for(piece.color, Crest::FlameSkull)
            {
                continue;
            }
            done |= 1 << sq;
            let blast = self.explode(sq, piece.color, None);
            queue.extend(
                blast
                    .hits
                    .iter()
                    .filter(|h| h.killed && h.target.color == piece.color)
                    .map(|h| (h.square, h.target)),
            );
            blasts.push(blast);
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
        // Герб молнии и единицы: электрические фигуры бьют первыми.
        let mut queue: Vec<Square> = strike_queue().collect();
        if self.crest_for(us, Crest::BoltOne) {
            queue.sort_by_key(|&sq| !self.board[sq as usize].is_some_and(|p| p.spark));
        }
        for from in queue {
            if done & 1 << from != 0 {
                continue;
            }
            if self.board[from as usize].is_none_or(|p| p.color != us || p.is_frozen() || p.fresh) {
                continue;
            }
            // Варвар (rogue-like, с 0.80): убил кого-то за удар — бьёт ещё раз всех, кто
            // остался в досягаемости (с 0.81 — `Rule::RogueBarbarianRepeats` раз подряд, гербы
            // огненного топора и красного круга — ниже).
            let mut round = 0;
            // Герб топора и клыков: лечение за убийство — раз за ход.
            let mut fed = false;
            // Клетки, куда от этого атакующего ушли порталом теневые фигуры.
            let mut skip = 0u64;
            'rounds: loop {
                let Some(mut attacker) = self.board[from as usize] else {
                    break;
                };
                // Герб топора и красного круга: повторный удар варвара — по всем чужим в радиусе.
                let whirl = round > 0 && self.crest_for(us, Crest::Whirl);
                let mut targets: Vec<(Square, Piece)> = if whirl {
                    let radius = balance().rule(Rule::RogueCrestWhirlRadius).clamp(1, 7) as i8;
                    (0..64u8)
                        .filter(|&sq| {
                            let (df, dr) =
                                (file_of(sq) - file_of(from), rank_of(sq) - rank_of(from));
                            sq != from && df.abs() <= radius && dr.abs() <= radius
                        })
                        .filter(|&sq| wall & 1 << sq == 0)
                        .filter_map(|sq| {
                            Some((sq, self.board[sq as usize].filter(|p| p.color != us)?))
                        })
                        .collect()
                } else {
                    strike_steps(attacker)
                        .iter()
                        // Герб теневого огня и глаза: теневая бьёт сквозь фигуры.
                        .filter(|&&step| {
                            attacker.shade && self.crest_for(us, Crest::DarkEye)
                                || self.line_clear(from, attacker, step, wall)
                        })
                        .filter_map(|&(df, dr)| square(file_of(from) + df, rank_of(from) + dr))
                        .filter(|&sq| wall & 1 << sq == 0)
                        .filter_map(|sq| {
                            Some((sq, self.board[sq as usize].filter(|p| p.color != us)?))
                        })
                        .collect()
                };
                targets.sort_by_key(|&(sq, p)| {
                    (std::cmp::Reverse(p.kind.points()), p.hp, strike_order(sq))
                });
                let mut killed = false;
                let order: Vec<Square> = targets.iter().map(|&(sq, _)| sq).collect();
                for (sq, _) in targets {
                    // Ушедшую порталом теневую этот же атакующий больше не бьёт.
                    if skip & 1 << sq != 0 {
                        continue;
                    }
                    // Цель могла погибнуть от молнии предыдущего удара.
                    let Some(target) = self.board[sq as usize].filter(|p| p.color != us) else {
                        continue;
                    };
                    if let Some(me) = self.board[from as usize] {
                        attacker = me;
                    }
                    // Круговой удар — свой урон `Rule::RogueCrestWhirlDamage` каждой фигуре.
                    let raw = if whirl {
                        balance().rule(Rule::RogueCrestWhirlDamage).clamp(0, 99)
                            * crate::balance::UNIT
                    } else {
                        attacker.attack()
                    };
                    let mut strike = self.strike_raw(from, attacker, sq, target, wall, raw);
                    if let Some((other, _)) = strike.swap {
                        skip |= 1 << other;
                    }
                    strike.again = round > 0;
                    strike.whirl = whirl;
                    let kill = strike.hits().any(|h| h.killed);
                    killed |= kill;
                    if kill
                        && !fed
                        && attacker.barbarian
                        && self.crest_for(us, Crest::AxeFangs)
                        && let Some(me) = &mut self.board[from as usize]
                    {
                        fed = true;
                        let heal = (balance().rule(Rule::RogueCrestFangsHeal).clamp(0, 99)
                            * crate::balance::UNIT)
                            .min(me.max_hp() - me.hp)
                            .max(0);
                        me.hp += heal;
                        strike.feast += heal;
                    }
                    let advance = self.advance
                        && strike.hit.killed
                        && close_strike(attacker.kind, from, sq)
                        && self.board[sq as usize].is_none();
                    if advance {
                        self.advance_to(from, &mut strike);
                        done |= 1 << sq;
                    }
                    let stop = advance || strike.rebound || self.board[from as usize].is_none();
                    strikes.push(strike);
                    if stop {
                        break 'rounds;
                    }
                }
                // Герб ветра и меча: воздушная бьёт первую ещё живую из своих целей второй раз.
                if round == 0
                    && attacker.wind
                    && self.crest_for(us, Crest::WindSword)
                    && let Some(me) = self.board[from as usize]
                    && let Some(sq) = order.iter().copied().find(|&sq| {
                        skip & 1 << sq == 0
                            && self.board[sq as usize].is_some_and(|p| p.color != us)
                    })
                {
                    let target = self.board[sq as usize].expect("чужая фигура");
                    let mut strike = self.strike(from, me, sq, target, wall);
                    strike.again = true;
                    killed |= strike.hits().any(|h| h.killed);
                    let stop = strike.rebound || self.board[from as usize].is_none();
                    strikes.push(strike);
                    if stop {
                        break 'rounds;
                    }
                }
                round += 1;
                if !(attacker.barbarian && killed) || round > self.barbarian_repeats(from) {
                    break;
                }
            }
        }
        if !strikes.is_empty() {
            self.halfmove_clock = 0;
        }
        strikes
    }

    /// Сколько раз подряд варвар на `from` может ударить ещё раз после убийства:
    /// `Rule::RogueBarbarianRepeats`, с гербом огненного топора при малом здоровье — больше; не
    /// больше `BARBARIAN_ROUNDS`.
    fn barbarian_repeats(&self, from: Square) -> u8 {
        let Some(piece) = self.board[from as usize] else {
            return 0;
        };
        let rule = |rule: Rule| balance().rule(rule).clamp(0, 15) as u8;
        let mut repeats = rule(Rule::RogueBarbarianRepeats);
        let low = balance().rule(Rule::RogueCrestFireAxeHp).clamp(1, 99) * crate::balance::UNIT;
        if self.crest_for(piece.color, Crest::FireAxe) && piece.hp <= low {
            repeats += rule(Rule::RogueCrestFireAxeRepeats);
        }
        repeats.min(BARBARIAN_ROUNDS)
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
        self.strike_raw(from, attacker, sq, target, wall, attacker.attack())
    }

    /// Удар уроном `raw` вместо атаки (круговой удар варвара — свой урон).
    fn strike_raw(
        &mut self,
        from: Square,
        attacker: Piece,
        sq: Square,
        target: Piece,
        wall: u64,
        raw: i32,
    ) -> Strike {
        let us = attacker.color;
        let promoted = None;
        let mut feast = 0;
        let mut chilled = false;
        let mut target = target;
        // Гербы теневых фигур игрока: каждый N-й удар по ней не проходит (два теневых огня), а
        // от смертельного удара при малом здоровье она уходит порталом — удар принимает
        // ближайшая своя обычная фигура (теневой огонь и портал). Уклонение — раньше портала.
        let mut dodged = false;
        let mut swap = None;
        if target.shade && self.crest_for(target.color, Crest::DarkFlames) {
            let every = balance().rule(Rule::RogueCrestDarkDodgeEvery).clamp(2, 20) as u8;
            if let Some(piece) = &mut self.board[sq as usize] {
                piece.dodge += 1;
                dodged = piece.dodge >= every;
                if dodged {
                    piece.dodge = 0;
                }
                target.dodge = piece.dodge;
            }
        }
        let low = balance().rule(Rule::RogueCrestDarkPortalHp).clamp(1, 99) * crate::balance::UNIT;
        if !dodged
            && target.shade
            && target.hp <= low
            && self.damage_at(sq, raw) >= target.hp
            && self.crest_for(target.color, Crest::DarkPortal)
            && let Some(other) = self.portal_partner(sq, target.color)
        {
            self.board.swap(sq as usize, other as usize);
            if let Some(r) = &mut self.rogue {
                r.origin.swap(sq as usize, other as usize);
            }
            swap = Some((other, target));
            target = self.board[sq as usize].expect("обычная фигура");
        }
        let mut damage = if dodged { 0 } else { self.damage_at(sq, raw) };
        // Герб замороженной снежинки: ледяная фигура во льду урона не получает, а удар, после
        // которого у неё осталось бы `Rule::RogueCrestCrystalHp` здоровья или меньше, оставляет
        // ровно столько — и она вмерзает в лёд до конца этого хода соперника: оттаивает к
        // своему ходу и может ходить (с 0.78, просьба пользователя).
        let mut encased = false;
        if target.icy && self.crest_for(target.color, Crest::Crystal) {
            let floor =
                balance().rule(Rule::RogueCrestCrystalHp).clamp(1, 99) * crate::balance::UNIT;
            if target.is_frozen() {
                damage = 0;
            } else if target.hp > floor && target.hp - damage <= floor {
                damage = target.hp - floor;
                encased = true;
            }
        }
        let (_, killed) = self.wound(sq, damage);
        // Ядовитая фигура отравляет выжившую цель (уже отравленную — нет): урон яда — в конце
        // следующего хода цели (бьют в ход соперника — в конце её ближайшего хода, в её ход —
        // в конце следующего).
        let mut poisoned = false;
        if attacker.venom
            && damage > 0
            && let Some(piece) = &mut self.board[sq as usize]
            && piece.poison == 0
        {
            piece.poison = if self.side == piece.color { 2 } else { 1 };
            poisoned = true;
        }
        if encased && let Some(piece) = &mut self.board[sq as usize] {
            piece.frozen = piece.frozen.max(1);
            chilled = true;
        }
        // Вампир восстанавливает здоровье за каждый удар по чужой фигуре, вампир в броне — только
        // за первый за ход (`Position::fed`).
        let fed = attacker.armored && self.fed & 1 << from != 0;
        if attacker.vampire && attacker.armored && damage > 0 {
            self.fed |= 1 << from;
        }
        if attacker.vampire
            && damage > 0
            && !fed
            && let Some(me) = &mut self.board[from as usize]
        {
            let heal = (balance().rule(Rule::RogueVampireHeal).clamp(0, 99) * crate::balance::UNIT)
                .min(me.max_hp() - me.hp)
                .max(0);
            me.hp += heal;
            feast += heal;
        }
        // Герб снежинки и шипов: ударивший ледяную фигуру получает её урон.
        let mut thorns = None;
        if target.icy && damage > 0 && self.crest_for(target.color, Crest::Spikes) {
            let back = damage * balance().rule(Rule::RogueCrestSpikesPercent).clamp(0, 1000) / 100;
            if let Some(me) = self.board[from as usize]
                && back > 0
            {
                let (_, dead) = self.wound(from, back);
                if dead {
                    self.score_kill(me, target.color);
                }
                thorns = Some(Hit {
                    square: from,
                    target: me,
                    damage: back,
                    killed: dead,
                });
            }
        }
        // Ледяная цель замораживает ударившего (даже если она погибла) — на весь его следующий
        // ход: границы ходов его стороны — конец этого, начало и конец следующего; с гербом
        // двойной снежинки — на `Rule::RogueCrestFreezeTurns` его ходов.
        let turns = if self.crest_for(target.color, Crest::Snow2) {
            balance().rule(Rule::RogueCrestFreezeTurns).clamp(1, 9) as u8
        } else {
            1
        };
        let rebound = target.icy
            && match &mut self.board[from as usize] {
                Some(piece) if piece.kind != Kind::King => {
                    piece.frozen = piece.frozen.max(1 + 2 * turns);
                    true
                }
                _ => false,
            };
        if killed {
            self.score_kill(target, us);
            // Rogue-like: победа золотой фигурой игрока — золото сверху (с гербами короля и
            // короля и ферзя — ещё).
            if attacker.gold && self.rogue.is_some_and(|r| r.player == us) {
                let rule = |rule: Rule| balance().rule(rule).clamp(0, 999) as u16;
                let mut bonus = rule(Rule::RogueGoldBonus);
                if self.has_crest(Crest::GoldKing) {
                    bonus = bonus.saturating_add(rule(Rule::RogueCrestGoldKingBonus));
                }
                if attacker.kind == Kind::Queen && self.has_crest(Crest::KingQueen) {
                    bonus = bonus.saturating_add(rule(Rule::RogueCrestKingQueenBonus));
                }
                if let Some(r) = &mut self.rogue {
                    r.gold = r.gold.saturating_add(bonus);
                }
            }
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
        // Электрическая фигура: удар расходится от цели на чужие фигуры — по диагоналям (с
        // гербом молнии и плюса — и по прямым), на `spark_reach` клеток через пустые, до первой
        // фигуры на каждом направлении. С гербом двух молний (с 0.92) — цепью: от каждой
        // задетой дальше так же, пока задетые связаны; каждую — один раз.
        let mut splash = Vec::new();
        if attacker.spark {
            let spread = attacker.attack() * balance().rule(Rule::SparkPercent) / 100;
            let rays: &[(i8, i8)] = if self.crest_for(us, Crest::BoltPlus) {
                &KING_STEPS
            } else {
                &BISHOP_RAYS
            };
            let reach = self.spark_reach(attacker);
            let chain = self.crest_for(us, Crest::Bolts);
            // Задетые клетки (и цель): молния через них не возвращается и бьёт каждую раз.
            let mut seen = 1u64 << sq | 1u64 << from;
            let mut sources = vec![sq];
            let mut next = 0;
            while next < sources.len() {
                let at = sources[next];
                next += 1;
                for &(df, dr) in rays {
                    let mut found = None;
                    for k in 1..=reach {
                        let Some(s) = square(file_of(at) + df * k, rank_of(at) + dr * k) else {
                            break;
                        };
                        if wall & 1 << s != 0 {
                            break;
                        }
                        if seen & 1 << s != 0 {
                            break;
                        }
                        match self.board[s as usize] {
                            None => continue,
                            Some(p) if p.color != us => found = Some((s, p)),
                            Some(_) => {}
                        }
                        break;
                    }
                    let Some((s, victim)) = found else {
                        continue;
                    };
                    seen |= 1 << s;
                    let damage = self.damage_at(s, spread);
                    let (_, dead) = self.wound(s, damage);
                    if dead {
                        self.score_kill(victim, us);
                    }
                    splash.push(Jump {
                        from: at,
                        hit: Hit {
                            square: s,
                            target: victim,
                            damage,
                            killed: dead,
                        },
                    });
                    if chain {
                        sources.push(s);
                    }
                }
            }
        }
        // Герб красной молнии: убийства электрической фигуры копят ей заряд.
        if attacker.spark && self.crest_for(us, Crest::RedBolt) {
            let kills = u8::from(killed) + splash.iter().filter(|j| j.hit.killed).count() as u8;
            if let Some(me) = &mut self.board[from as usize] {
                me.charge = me.charge.saturating_add(kills);
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
            rebound,
            thorns,
            poisoned,
            again: false,
            whirl: false,
            dodged,
            swap,
        }
    }

    /// На сколько клеток летит молния электрической фигуры `piece` (от цели и в цепи — от
    /// задетой к следующей): на одну; с гербом красной молнии — ещё на клетку за каждые
    /// `Rule::RogueCrestRedBoltStep` её убийств (не больше `Rule::RogueCrestRedBoltMax`).
    pub fn spark_reach(&self, piece: Piece) -> i8 {
        if self.crest_for(piece.color, Crest::RedBolt) {
            let step = balance().rule(Rule::RogueCrestRedBoltStep).clamp(1, 999);
            let max = balance().rule(Rule::RogueCrestRedBoltMax).clamp(0, 7);
            return 1 + (i32::from(piece.charge) / step).min(max) as i8;
        }
        1
    }

    /// Куда уходит порталом теневая фигура стороны `color` с клетки `sq`: ближайшая своя
    /// обычная фигура (без особой силы, не король и не босс).
    fn portal_partner(&self, sq: Square, color: Color) -> Option<Square> {
        (0..64u8)
            .filter(|&s| {
                s != sq
                    && self.board[s as usize].is_some_and(|p| {
                        p.color == color
                            && p.kind != Kind::King
                            && !p.boss
                            && Stock::of(p).power == Power::Plain
                    })
            })
            .min_by_key(|&s| {
                let d = (file_of(s) - file_of(sq))
                    .abs()
                    .max((rank_of(s) - rank_of(sq)).abs());
                (d, strike_order(s))
            })
    }

    /// Клетки, через которые прошла теневая фигура с `from` на `to`: по прямой — все между
    /// ними, ход «конём» на две и одну — две клетки рядом с обеими.
    fn shade_passed(from: Square, to: Square) -> Vec<Square> {
        let (df, dr) = (file_of(to) - file_of(from), rank_of(to) - rank_of(from));
        if df == 0 || dr == 0 || df.abs() == dr.abs() {
            let n = df.abs().max(dr.abs());
            let (sf, sr) = (df.signum(), dr.signum());
            return (1..n)
                .filter_map(|k| square(file_of(from) + sf * k, rank_of(from) + sr * k))
                .collect();
        }
        let mut out = Vec::new();
        for f in -1..=1i8 {
            for r in -1..=1i8 {
                let (x, y) = (file_of(from) + f, rank_of(from) + r);
                if (f, r) != (0, 0)
                    && (x - file_of(to)).abs() <= 1
                    && (y - rank_of(to)).abs() <= 1
                    && let Some(sq) = square(x, y)
                {
                    out.push(sq);
                }
            }
        }
        out
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
        // Ходы кончились — сходить может только воздушная фигура вторым ходом.
        let spent = self.moves_made >= self.moves_per_turn();
        if spent && self.airborne == 0 {
            return;
        }
        // Rogue-like: фигуры волн сами не ходят — они шагают вперёд в конце хода (`march`).
        if self.rogue.is_some_and(|r| r.player != self.side) {
            return;
        }
        if self.planning() {
            // Пока открыт магазин, доска ждёт.
            if !self.shop_open() {
                self.arrange_moves(out);
            }
            return;
        }
        let us = self.side;
        for from in 0..64u8 {
            let Some(piece) = self.board[from as usize] else {
                continue;
            };
            if piece.color != us
                || piece.is_frozen()
                || self.used(from)
                || spent && self.airborne & 1 << from == 0
            {
                continue;
            }
            if piece.shade && piece.kind != Kind::King {
                self.shade_moves(from, piece, out);
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
        for stock in self.reserve(us).into_iter().map(|s| s.0).filter(|_| !spent) {
            if self.stock_ready(us, stock) {
                out.extend(
                    self.drop_squares(us, stock.kind)
                        .map(|sq| Move::new(sq, sq, stock.drop_kind())),
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
    pub fn reserve(&self, color: Color) -> Vec<(Stock, u8, u8)> {
        let kinds = [
            Kind::Queen,
            Kind::Rook,
            Kind::Bishop,
            Kind::Knight,
            Kind::Pawn,
        ];
        // Объединённые — самыми первыми, потом особые (огненные, электрические, …).
        let combos = COMBOS
            .into_iter()
            .enumerate()
            .flat_map(|(i, (base, with))| {
                kinds.into_iter().filter_map(move |kind| {
                    let n = self.combo_reserve[i][color as usize][kind as usize];
                    (n > 0).then_some((Stock::combo(kind, base, with), n, 0))
                })
            });
        let charged = [
            Power::Fire,
            Power::Spark,
            Power::Wind,
            Power::Gold,
            Power::Shade,
            Power::Ice,
            Power::Barbarian,
            Power::Wizard,
        ]
        .into_iter()
        .flat_map(|power| {
            kinds.into_iter().filter_map(move |kind| {
                let n = self.charged(power, color)?[kind as usize];
                (n > 0).then_some((Stock::new(kind, power), n, 0))
            })
        });
        let plain = kinds.into_iter().filter_map(|kind| {
            let [ready, w1, w2] =
                self.reserve[color as usize][kind as usize].map(|r| r.iter().sum::<u8>());
            (ready + w1 + w2 > 0).then_some((Stock::new(kind, Power::Plain), ready, w1 + w2))
        });
        combos.chain(charged).chain(plain).collect()
    }

    /// Запас фигур `stock` стороны `color` по видам — у объединённых и особых (готовы всегда);
    /// у обычных — `None`.
    fn charged_stock(&self, stock: Stock, color: Color) -> Option<&[u8; 6]> {
        match stock.combo_index() {
            Some(i) => Some(&self.combo_reserve[i][color as usize]),
            None => self.charged(stock.power, color),
        }
    }

    fn charged_stock_mut(&mut self, stock: Stock, color: Color) -> Option<&mut [u8; 6]> {
        match stock.combo_index() {
            Some(i) => Some(&mut self.combo_reserve[i][color as usize]),
            None => self.charged_mut(stock.power, color),
        }
    }

    /// Запас огненных или электрических фигур стороны `color` по видам; у обычных — `None`.
    fn charged(&self, power: Power, color: Color) -> Option<&[u8; 6]> {
        match power {
            Power::Plain => None,
            Power::Fire => Some(&self.fire_reserve[color as usize]),
            Power::Spark => Some(&self.spark_reserve[color as usize]),
            Power::Wind => Some(&self.wind_reserve[color as usize]),
            Power::Gold => Some(&self.gold_reserve[color as usize]),
            Power::Shade => Some(&self.shade_reserve[color as usize]),
            Power::Ice => Some(&self.ice_reserve[color as usize]),
            Power::Barbarian => Some(&self.barbarian_reserve[color as usize]),
            Power::Wizard => Some(&self.wizard_reserve[color as usize]),
        }
    }

    fn charged_mut(&mut self, power: Power, color: Color) -> Option<&mut [u8; 6]> {
        match power {
            Power::Plain => None,
            Power::Fire => Some(&mut self.fire_reserve[color as usize]),
            Power::Spark => Some(&mut self.spark_reserve[color as usize]),
            Power::Wind => Some(&mut self.wind_reserve[color as usize]),
            Power::Gold => Some(&mut self.gold_reserve[color as usize]),
            Power::Shade => Some(&mut self.shade_reserve[color as usize]),
            Power::Ice => Some(&mut self.ice_reserve[color as usize]),
            Power::Barbarian => Some(&mut self.barbarian_reserve[color as usize]),
            Power::Wizard => Some(&mut self.wizard_reserve[color as usize]),
        }
    }

    /// Сколько готовых фигур `stock` в запасе стороны `color`.
    fn stock_ready(&self, color: Color, stock: Stock) -> bool {
        match self.charged_stock(stock, color) {
            Some(charged) => charged[stock.kind as usize] > 0,
            None => self.reserve[color as usize][stock.kind as usize][0]
                .iter()
                .any(|&n| n > 0),
        }
    }

    /// Берёт из запаса фигуру `stock` для выставления: какой она выйдет (кроме `fresh`).
    fn take_stock(&mut self, color: Color, stock: Stock) -> Piece {
        if let Some(charged) = self.charged_stock_mut(stock, color) {
            let n = &mut charged[stock.kind as usize];
            *n = n.saturating_sub(1);
            return self.crested(stock.piece(color));
        }
        // Первой выходит фигура, что возвращалась реже.
        let ready = &mut self.reserve[color as usize][stock.kind as usize][0];
        let r = ready.iter().position(|&n| n > 0).expect("фигура в запасе");
        ready[r] -= 1;
        self.crested(Piece {
            reborn: r as u8 + 1,
            ..Piece::new(stock.kind, color)
        })
    }

    /// Фигура `piece` уходит в запас готовой (rogue-like: расстановка, покупка).
    fn give_stock(&mut self, piece: Piece) {
        if let Some(charged) = self.charged_stock_mut(Stock::of(piece), piece.color) {
            let n = &mut charged[piece.kind as usize];
            *n = n.saturating_add(1);
            return;
        }
        let color = piece.color as usize;
        let slot = (piece.reborn.max(1) - 1).min(MAX_REBIRTHS - 1) as usize;
        let ready = &mut self.reserve[color][piece.kind as usize][0];
        ready[slot] = ready[slot].saturating_add(1);
    }

    /// Огненные фигуры в запасе стороны `color` по видам (тесты).
    #[cfg(test)]
    pub fn fire_reserve(&self, color: Color) -> [u8; 6] {
        self.fire_reserve[color as usize]
    }

    /// Электрические фигуры в запасе стороны `color` по видам (тесты).
    #[cfg(test)]
    pub fn spark_reserve(&self, color: Color) -> [u8; 6] {
        self.spark_reserve[color as usize]
    }

    /// Воздушные фигуры в запасе стороны `color` по видам (тесты).
    #[cfg(test)]
    pub fn wind_reserve(&self, color: Color) -> [u8; 6] {
        self.wind_reserve[color as usize]
    }

    /// Золотые фигуры в запасе стороны `color` по видам (тесты).
    #[cfg(test)]
    pub fn gold_reserve(&self, color: Color) -> [u8; 6] {
        self.gold_reserve[color as usize]
    }

    /// Теневые фигуры в запасе стороны `color` по видам (тесты).
    #[cfg(test)]
    pub fn shade_reserve(&self, color: Color) -> [u8; 6] {
        self.shade_reserve[color as usize]
    }

    /// Ледяные фигуры в запасе стороны `color` по видам (тесты).
    #[cfg(test)]
    pub fn ice_reserve(&self, color: Color) -> [u8; 6] {
        self.ice_reserve[color as usize]
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
    /// Ходы теневой фигуры (rogue-like, с 0.70) — сквозь фигуры, на пустые клетки: слон,
    /// ладья и ферзь — по всей своей линии; конь — на любую клетку через одну (16 клеток на
    /// расстоянии 2); пешка — на клетку вперёд и через одну фигуру перед собой (с начальной
    /// горизонтали — на две, как обычно).
    fn shade_moves(&self, from: Square, piece: Piece, out: &mut Vec<Move>) {
        let us = piece.color;
        let wall = self.wall(us);
        let (file, rank) = (file_of(from), rank_of(from));
        let land = |to: Square, out: &mut Vec<Move>| {
            let mv = Move::new(from, to, MoveKind::Quiet);
            if piece.kind == Kind::Pawn && rank_of(to) == us.last_rank() {
                out.extend(Kind::PROMOTIONS.map(|kind| Move {
                    promotion: Some(kind),
                    ..mv
                }));
            } else {
                out.push(mv);
            }
        };
        let rays: &[(i8, i8)] = match piece.kind {
            Kind::Bishop => &BISHOP_RAYS,
            Kind::Rook => &ROOK_RAYS,
            _ => &[],
        };
        let queen = [ROOK_RAYS, BISHOP_RAYS].concat();
        let rays = if piece.kind == Kind::Queen {
            &queen[..]
        } else {
            rays
        };
        match piece.kind {
            Kind::Pawn => {
                let dir = us.forward();
                let one = square(file, rank + dir);
                if let Some(to) = one.filter(|&to| self.open(to, wall)) {
                    land(to, out);
                }
                let blocked = one.is_some_and(|to| !self.open(to, wall));
                if (blocked || rank == us.home_rank() + dir)
                    && let Some(to) = square(file, rank + 2 * dir).filter(|&to| self.open(to, wall))
                {
                    land(to, out);
                }
            }
            Kind::Knight => {
                for df in -2..=2i8 {
                    for dr in -2..=2i8 {
                        if df.abs().max(dr.abs()) != 2 {
                            continue;
                        }
                        if let Some(to) =
                            square(file + df, rank + dr).filter(|&to| self.open(to, wall))
                        {
                            land(to, out);
                        }
                    }
                }
            }
            _ => {
                for &(df, dr) in rays {
                    let (mut x, mut y) = (file + df, rank + dr);
                    while let Some(to) = square(x, y) {
                        if self.open(to, wall) {
                            land(to, out);
                        }
                        x += df;
                        y += dr;
                    }
                }
            }
        }
    }

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
        // Rogue-like: погибший зомби волны (ещё не восстававший) ложится в могилу — восстанет в
        // конце следующего хода своей стороны (погиб в её же ход — того, что после).
        if piece.zombie && !piece.risen && self.rogue.is_some_and(|r| r.player != piece.color) {
            let wait = if self.side == piece.color { 2 } else { 1 };
            self.graves[sq as usize] = Grave {
                kind: Some(piece.kind),
                wait,
                venom: piece.venom,
            };
        }
        // Герб короля и черепа: погибшая золотая фигура игрока возвращает свою цену.
        if piece.gold && self.crest_for(piece.color, Crest::KingSkull) {
            let refund = self.shop_price(Stock::of(piece));
            if let Some(r) = &mut self.rogue {
                r.gold = r.gold.saturating_add(refund);
            }
        }
        self.castling &= !rights_lost_at(sq);
        if piece.kind == Kind::King && !piece.boss {
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
            if let Some(r) = &mut self.rogue {
                r.origin[to as usize] = std::mem::replace(&mut r.origin[from as usize], NO_ORIGIN);
            }
        }
    }

    /// Делает ход (законность не проверяется) и сообщает, что произошло.
    pub fn make_move(&mut self, mv: Move) -> Outcome {
        if self.planning() {
            return self.arrange(mv);
        }
        let us = self.side;
        let piece = match mv.kind.stock() {
            Some(stock) => Piece::new(stock.kind, us),
            None => self.board[mv.from as usize].expect("ход с пустой клетки"),
        };
        let mut outcome = Outcome {
            mv,
            piece,
            rook: None,
            promoted: None,
            frozen: false,

            reactions: Vec::new(),
            passed: Vec::new(),
        };
        if let Some(stock) = mv.kind.stock() {
            let fresh = Piece {
                fresh: true,
                ..self.take_stock(us, stock)
            };
            self.board[mv.to as usize] = Some(fresh);
            if let Some(r) = &mut self.rogue {
                r.origin[mv.to as usize] = NO_ORIGIN;
            }
            outcome.piece = fresh;
            outcome.frozen = self.on_ice(mv.to) && self.freeze_at(mv.to);
            self.halfmove_clock = 0;
            self.moved = true;
            self.moves_made += 1;
            self.used |= 1 << mv.to;
            self.airborne &= !(1 << mv.to);
            outcome.reactions = self.intercept(mv.to, None);
            return outcome;
        }

        self.relocate(mv.from, mv.to);
        // Герб теневого огня и черепа: теневая бьёт своей атакой чужих, через которых прошла.
        if piece.shade && self.crest_for(us, Crest::DarkSkull) {
            for sq in Self::shade_passed(mv.from, mv.to) {
                let Some(target) = self.board[sq as usize].filter(|p| p.color != us) else {
                    continue;
                };
                let damage = self.damage_at(sq, piece.attack());
                let (_, killed) = self.wound(sq, damage);
                if killed {
                    self.score_kill(target, us);
                }
                outcome.passed.push(WaveHit {
                    square: sq,
                    target,
                    damage,
                    killed,
                });
            }
        }
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
            self.board[mv.to as usize] = Some(self.crested(Piece::new(kind, us)));
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
        // Воздушная фигура (rogue-like): первый ход — как обычный, второй — бесплатный.
        // Между её ходами могут ходить другие фигуры.
        // С гербом двух ветров после второго хода — ещё третий, тоже бесплатный (`twice`).
        let second = self.airborne & 1 << mv.from != 0;
        let twice = self.twice & 1 << mv.from != 0;
        self.airborne &= !(1 << mv.from | 1 << mv.to);
        self.twice &= !(1 << mv.from | 1 << mv.to);
        if !second {
            self.moves_made += 1;
        }
        if piece.wind && !second {
            self.airborne |= 1 << mv.to;
        } else if piece.wind && !twice && self.crest_for(us, Crest::Winds) {
            self.airborne |= 1 << mv.to;
            self.twice |= 1 << mv.to;
        } else {
            self.used |= 1 << mv.to;
        }
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
    /// `passed` в их досягаемости, — по её новой клетке. В барьер соперника не бьют. В
    /// rogue-like (с 0.55, просьба пользователя) кони не перехватывают — только пешки.
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
        let rogue = self.rogue.is_some();
        let mut guards: Vec<Square> = [Kind::Knight, Kind::Pawn]
            .into_iter()
            .filter(|&kind| !(rogue && kind == Kind::Knight))
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
    /// Rogue-like: магазин и кнопка «Выиграть раунд».
    Rogue(RogueAction),
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
            // Расстановка rogue-like: «Кg1→d4», «Кg1⇄Сf1», «К@c3».
            Event::Move(o) if self.before.planning() => {
                let letter = o.piece.kind.letter();
                let (from, to) = (square_name(o.mv.from), square_name(o.mv.to));
                let text = match (o.mv.kind, o.rook) {
                    (
                        MoveKind::Drop(_)
                        | MoveKind::FireDrop(_)
                        | MoveKind::SparkDrop(_)
                        | MoveKind::WindDrop(_)
                        | MoveKind::GoldDrop(_)
                        | MoveKind::ShadeDrop(_)
                        | MoveKind::IceDrop(_)
                        | MoveKind::BarbarianDrop(_)
                        | MoveKind::WizardDrop(_)
                        | MoveKind::ComboDrop(..),
                        _,
                    ) => {
                        format!("{letter}@{to}")
                    }
                    (_, Some(_)) => {
                        let other = self.before.piece(o.mv.to).map_or("", |p| p.kind.letter());
                        format!("{letter}{from}⇄{other}{to}")
                    }
                    _ => format!("{letter}{from}→{to}"),
                };
                (text, 0)
            }
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
                    MoveKind::Drop(_)
                    | MoveKind::FireDrop(_)
                    | MoveKind::SparkDrop(_)
                    | MoveKind::WindDrop(_)
                    | MoveKind::GoldDrop(_)
                    | MoveKind::ShadeDrop(_)
                    | MoveKind::IceDrop(_)
                    | MoveKind::BarbarianDrop(_)
                    | MoveKind::WizardDrop(_)
                    | MoveKind::ComboDrop(..) => {
                        format!("{letter}@{}", square_name(o.mv.to))
                    }
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
            Event::Rogue(action) => {
                let text = match *action {
                    RogueAction::WinRound { .. } => "Раунд выигран".to_owned(),
                    RogueAction::Buy { stock, price } => {
                        let symbol = |power: Power| match power {
                            Power::Plain => "",
                            Power::Fire => "🔥",
                            Power::Spark => "⚡",
                            Power::Wind => "≋",
                            Power::Gold => "★",
                            Power::Shade => "◐",
                            Power::Ice => "❄",
                            Power::Barbarian => "⚔",
                            Power::Wizard => "✦",
                        };
                        let fire = format!("{}{}", symbol(stock.power), symbol(stock.with));
                        format!("Магазин: {fire}{} −{price}", stock.kind.letter())
                    }
                    RogueAction::Reroll { price } => format!("Обновить −{price}"),
                    RogueAction::Sell { stock, price } => {
                        format!("Продажа: {} +{price}", stock.kind.letter())
                    }
                    RogueAction::Leave => "Из магазина".to_owned(),
                    RogueAction::BuyCrest { crest, price } => {
                        format!("{} −{price}", crest.title())
                    }
                    RogueAction::RerollCrests { price } => format!("Обновить гербы −{price}"),
                };
                (text, 0)
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
                        let spark = if !s.splash.is_empty() { "⚡" } else { "" };
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
                if turn.planning.is_some() {
                    text.push("Расстановка".to_owned());
                }
                if let Some(level) = turn.level_up {
                    text.push(format!("Уровень {level}"));
                }
                if !turn.spawned.is_empty() && turn.level_up.is_none() {
                    let wave = self.before.rogue().map_or(0, |r| r.wave) + 1;
                    text.push(format!("Волна {wave}"));
                }
                if !turn.risen.is_empty() {
                    text.push(format!("Зомби ↑{}", turn.risen.len()));
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
            Event::Move(o) if self.before.planning() => {
                let name = o.piece.title();
                let (from, to) = (square_name(o.mv.from), square_name(o.mv.to));
                return match (o.mv.kind, o.rook) {
                    (
                        MoveKind::Drop(_)
                        | MoveKind::FireDrop(_)
                        | MoveKind::SparkDrop(_)
                        | MoveKind::WindDrop(_)
                        | MoveKind::GoldDrop(_)
                        | MoveKind::ShadeDrop(_)
                        | MoveKind::IceDrop(_)
                        | MoveKind::BarbarianDrop(_)
                        | MoveKind::WizardDrop(_)
                        | MoveKind::ComboDrop(..),
                        _,
                    ) => {
                        format!("Расстановка: {name} из запаса — на {to}.")
                    }
                    (_, Some(_)) => {
                        let other = self
                            .before
                            .piece(o.mv.to)
                            .map_or("фигура", |p| p.kind.name());
                        format!("Расстановка: {name} {from} и {other} {to} поменялись местами.")
                    }
                    _ => format!("Расстановка: {name} {from} → {to}."),
                };
            }
            Event::Move(outcome) => outcome,
            Event::Card(play) => return card_description(play, &self.before) + &ending,
            Event::Rogue(action) => {
                let text = match *action {
                    RogueAction::WinRound { gold } => format!(
                        "Кнопка «Выиграть раунд»: волны уровня побеждены, +{}.",
                        coins(gold, false)
                    ),
                    RogueAction::Buy { stock, price } => format!(
                        "Магазин: {} за {} — в запасе.",
                        stock.name(),
                        coins(price, true)
                    ),
                    RogueAction::Reroll { price } => {
                        format!("Магазин обновлён за {}.", coins(price, true))
                    }
                    RogueAction::Sell { stock, price } => format!(
                        "Продан{} {} из запаса за {}.",
                        if stock.kind.is_feminine() { "а" } else { "" },
                        stock.name(),
                        coins(price, true)
                    ),
                    RogueAction::Leave => "Ушли из магазина — расстановка.".to_owned(),
                    RogueAction::BuyCrest { crest, price } => format!(
                        "Куплен {} за {}: {}",
                        crest.title().to_lowercase(),
                        coins(price, true),
                        crest.description()
                    ),
                    RogueAction::RerollCrests { price } => {
                        format!("Гербы обновлены за {}.", coins(price, true))
                    }
                };
                return text + &ending;
            }
            Event::EndTurn(color, turn) => {
                let mut text = match turn.level_up {
                    Some(level) => format!("{} начали уровень {level}.", color.name()),
                    None => format!("{} закончили ход.", color.name()),
                };
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
                    let (name, verb) = self.before.piece(blast.center).map_or(
                        ("Пешка", "взорвалась"),
                        |p| {
                            let verb = if p.kind.is_feminine() {
                                "взорвалась"
                            } else {
                                "взорвался"
                            };
                            (p.kind.title(), verb)
                        },
                    );
                    text += &format!(" {name} {} {verb} лавой", square_name(blast.center));
                    text += &if blast.hits.is_empty() {
                        ", никого не задев.".to_owned()
                    } else {
                        format!(": {}.", hits_text(&blast.hits))
                    };
                }
                let waves = |hits: &[Hit]| -> Vec<WaveHit> {
                    hits.iter()
                        .map(|h| WaveHit {
                            square: h.square,
                            target: h.target,
                            damage: h.damage,
                            killed: h.killed,
                        })
                        .collect()
                };
                for meteor in &turn.meteors {
                    text += &format!(
                        " Метеорит мага с {} на {}: {}.",
                        square_name(meteor.from),
                        square_name(meteor.at),
                        hits_text(&waves(&meteor.hits))
                    );
                    if meteor.crater {
                        text += " Остался горящий кратер.";
                    }
                }
                if !turn.burned.is_empty() {
                    text += &format!(" Горящий кратер: {}.", hits_text(&waves(&turn.burned)));
                }
                if !turn.poison.is_empty() {
                    let hits: Vec<WaveHit> = turn
                        .poison
                        .iter()
                        .map(|h| WaveHit {
                            square: h.square,
                            target: h.target,
                            damage: h.damage,
                            killed: h.killed,
                        })
                        .collect();
                    text += &format!(" Яд: {}.", hits_text(&hits));
                }
                if !turn.charged.is_empty() {
                    let squares: Vec<String> =
                        turn.charged.iter().map(|&sq| square_name(sq)).collect();
                    text += &format!(" Лавой насытились пешки {}.", squares.join(", "));
                }
                if turn.gold > 0 {
                    let total = self.before.rogue().map_or(0, |r| r.gold) + turn.gold;
                    text += &format!(" Золото: +{} (всего {total}).", turn.gold);
                }
                if turn.interest > 0 {
                    text += &format!(" Свинья-копилка: +{}.", coins(turn.interest, false));
                }
                if let Some(level) = turn.planning {
                    text += &format!(
                        " Все волны уровня отбиты! Расстановка перед уровнем {level}: фигуры \
                         вернулись на места начала уровня и вылечились, выставленные из запаса — \
                         снова в запасе. Открылся магазин."
                    );
                }
                if !turn.risen.is_empty() {
                    let pieces: Vec<String> = turn
                        .risen
                        .iter()
                        .map(|&(sq, kind)| format!("{} {}", kind.name(), square_name(sq)))
                        .collect();
                    text += &format!(" Восстали зомби: {}.", pieces.join(", "));
                }
                if !turn.spawned.is_empty() {
                    let rogue = self.before.rogue();
                    let (wave, waves) = match (turn.level_up, rogue) {
                        (Some(level), _) => (1, Rogue::waves_at(level)),
                        (None, Some(r)) => (r.wave + 1, r.waves),
                        (None, None) => (0, 0),
                    };
                    let pieces: Vec<String> = turn
                        .spawned
                        .iter()
                        .map(|&(sq, kind)| {
                            // Король в волнах — только король-толстосум.
                            let name = if kind == Kind::King {
                                "король-толстосум"
                            } else {
                                kind.name()
                            };
                            format!("{name} {}", square_name(sq))
                        })
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
            MoveKind::Drop(_)
            | MoveKind::FireDrop(_)
            | MoveKind::SparkDrop(_)
            | MoveKind::WindDrop(_)
            | MoveKind::GoldDrop(_)
            | MoveKind::ShadeDrop(_)
            | MoveKind::IceDrop(_)
            | MoveKind::BarbarianDrop(_)
            | MoveKind::WizardDrop(_)
            | MoveKind::ComboDrop(..) => format!(
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
        if !o.passed.is_empty() {
            text += &format!(" Тень бьёт насквозь: {}.", hits_text(&o.passed));
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
    if strike.again {
        text.insert_str(
            0,
            if strike.whirl {
                " Круговой удар —"
            } else {
                " Снова —"
            },
        );
    }
    if strike.feast > 0 {
        let healer = if strike.attacker.champion {
            "Воин".to_owned()
        } else {
            attacker.title().to_owned()
        };
        text += &format!(
            " {healer} восстановил{verb_end} {} HP.",
            shown(strike.feast)
        );
    }
    if strike.promoted.is_some() {
        text += " Пешка стала ферзём.";
    }
    let splash: Vec<WaveHit> = strike
        .splash
        .iter()
        .map(|j| j.hit)
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
    if let Some(thorns) = strike.thorns {
        text += &format!(" Шипы льда: {} −{}", attacker.title(), shown(thorns.damage));
        text += &if thorns.killed {
            format!(", погиб{}.", if attacker.is_feminine() { "ла" } else { "" })
        } else {
            format!(
                ", осталось {} из {}.",
                shown(thorns.hp_after()),
                shown(thorns.target.max_hp())
            )
        };
    }
    if strike.dodged {
        text += &format!(
            " {} не получил{} удар.",
            hit.target.title(),
            if hit.target.kind.is_feminine() {
                "а"
            } else {
                ""
            }
        );
    }
    if let Some((other, shade)) = strike.swap {
        text += &format!(
            " {} ушл{} порталом на {}, удар принял{} {}.",
            shade.title(),
            if shade.kind.is_feminine() { "а" } else { "" },
            square_name(other),
            if hit.target.kind.is_feminine() {
                "а"
            } else {
                ""
            },
            hit.target.title().to_lowercase()
        );
    }
    if strike.poisoned {
        text += &format!(
            " {} отравлен{}.",
            hit.target.kind.title(),
            if hit.target.kind.is_feminine() {
                "а"
            } else {
                ""
            }
        );
    }
    if strike.rebound {
        text += &format!(
            " {} замёрз{} о лёд.",
            attacker.title(),
            if attacker.is_feminine() { "ла" } else { "" }
        );
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
        !self.is_over()
            && !self.position.shop_open()
            && (self.legal.is_empty() || self.position.moved() || self.position.planning())
    }

    /// Rogue-like: действие магазина или «Выиграть раунд» — в историю, как ход.
    fn rogue_action(&mut self, act: impl FnOnce(&mut Position) -> Option<RogueAction>) -> bool {
        if self.is_over() {
            return false;
        }
        let before = self.position.clone();
        let Some(action) = act(&mut self.position) else {
            return false;
        };
        self.history.push(Record {
            before,
            event: Event::Rogue(action),
            check: false,
            loser: None,
        });
        self.refresh();
        true
    }

    /// Rogue-like: купить фигуру с места `slot` прилавка.
    pub fn buy(&mut self, slot: usize) -> bool {
        self.rogue_action(|pos| pos.buy(slot))
    }

    /// Rogue-like: обновить магазин.
    pub fn reroll(&mut self) -> bool {
        self.rogue_action(Position::reroll)
    }

    /// Rogue-like: продать фигуру `stock` из запаса (пока открыт магазин).
    pub fn sell(&mut self, stock: Stock) -> bool {
        self.rogue_action(|pos| pos.sell(stock))
    }

    /// Rogue-like: купить герб с места `slot` прилавка гербов.
    pub fn buy_crest(&mut self, slot: usize) -> bool {
        self.rogue_action(|pos| pos.buy_crest(slot))
    }

    /// Rogue-like: обновить гербы на прилавке.
    pub fn reroll_crests(&mut self) -> bool {
        self.rogue_action(Position::reroll_crests)
    }

    /// Rogue-like: уйти из магазина к расстановке.
    pub fn leave_shop(&mut self) -> bool {
        self.rogue_action(Position::leave_shop)
    }

    /// Rogue-like: кнопка «Выиграть раунд» — волны уровня побеждены, ход кончается (на
    /// последнем уровне — победа).
    pub fn win_round(&mut self) -> bool {
        if !self.rogue_action(Position::win_round) {
            return false;
        }
        if !self.note_victory() {
            self.finish_turn();
        }
        true
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
        // Ходы в этом ходу ещё есть (rogue-like) — ход ждёт кнопку; расстановку кончает только
        // кнопка «Начать раунд».
        if !self.note_victory()
            && !self.position.any_card_playable()
            && self.legal.is_empty()
            && !self.position.planning()
        {
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
                Event::Card(_) | Event::EndTurn(..) | Event::Rogue(_) => {}
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
