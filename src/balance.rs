//! Баланс игры — числа, которые пользователь правит сам, без программиста: характеристики
//! фигур, цены и числа карт, правила хода и составы пяти колод. Лежат в текстовом файле
//! `%APPDATA%\BattleChess\balance.ini` (формат как у `settings.ini`, но с разделами).
//!
//! Если файла нет, игра создаёт его со стандартными значениями и комментариями; если в нём
//! не хватает строк (после обновления игры) — дописывает их, сохранив чужие значения.
//! Читается при запуске и при каждой новой партии: правки подхватываются без перезапуска.
//! Неверная строка (не число, вне допустимых пределов, неизвестное имя) не ломает игру: для
//! неё берётся стандартное значение, а предупреждение показывает главное меню.
//!
//! Правила читают числа через [`balance`]; в тестах файл не читается — там стандартные.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use crate::game::{CardKind, DECK_COUNT, DECK_NAMES, DEFAULT_RECIPES, Family, Kind, MAX_DECK};

/// Числа одной фигуры.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PieceNumbers {
    pub attack: i32,
    pub hp: i32,
    /// Очки победы убийце и очки жертвы хозяину.
    pub points: u8,
}

/// Числа правил хода.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    ManaPerTurn,
    CardsPerTurn,
    StartHand,
    MaxHand,
    LavaPawns,
    LavaDamage,
    SparkPercent,
    RogueWaves,
    RogueWaveMin,
    RogueWaveMax,
    RogueMoves,
    RogueWeightPawn,
    RogueWeightKnight,
    RogueWeightBishop,
    RogueWeightRook,
    RogueWeightQueen,
}

pub struct RuleDef {
    pub key: &'static str,
    pub default: i32,
    pub min: i32,
    pub max: i32,
    pub note: &'static str,
}

/// Строки `[rules]` прежних версий, которых больше нет: в старом файле они не предупреждение.
/// `rogue_wave_turns` — с 0.52 волна rogue-like выходит каждый ход.
const RETIRED_RULES: [&str; 1] = ["rogue_wave_turns"];

/// В порядке `Rule`.
pub const RULES: [RuleDef; 16] = [
    RuleDef {
        key: "mana_per_turn",
        default: 2,
        min: 0,
        max: 20,
        note: "сколько маны сторона получает в начале каждого своего хода (мана копится)",
    },
    RuleDef {
        key: "cards_per_turn",
        default: 3,
        min: 0,
        max: 10,
        note: "сколько карт можно сыграть за ход — до хода фигурой и после него вместе",
    },
    RuleDef {
        key: "start_hand",
        default: 3,
        min: 0,
        max: 10,
        note: "сколько карт сдаётся в руку перед первым ходом",
    },
    RuleDef {
        key: "max_hand",
        default: 7,
        min: 1,
        max: 10,
        note: "больше карт в руке не бывает: с полной рукой карта остаётся в колоде",
    },
    RuleDef {
        key: "lava_pawns",
        default: 3,
        min: 0,
        max: 16,
        note: "усиленный компьютер: сколько его случайных пешек в начале вашего хода насыщается \
               лавой (в конце хода компьютера они взрываются огненной волной)",
    },
    RuleDef {
        key: "lava_damage",
        default: 300,
        min: 0,
        max: 9999,
        note: "усиленный компьютер: урон взрыва лавы каждой вашей фигуре вокруг пешки \
               (свои фигуры взрыв не задевает), в тех же единицах, что атака",
    },
    RuleDef {
        key: "spark_percent",
        default: 100,
        min: 0,
        max: 1000,
        note: "усиленный компьютер: сколько процентов атаки слона с электричеством достаётся \
               каждой вашей фигуре по диагонали вокруг его цели",
    },
    RuleDef {
        key: "rogue_waves",
        default: 10,
        min: 1,
        max: 99,
        note: "rogue-like: сколько волн фигур соперника выйдет за партию; отбил все — победа",
    },
    RuleDef {
        key: "rogue_wave_min",
        default: 2,
        min: 1,
        max: 16,
        note: "rogue-like: меньше всего фигур в волне",
    },
    RuleDef {
        key: "rogue_wave_max",
        default: 4,
        min: 1,
        max: 16,
        note: "rogue-like: больше всего фигур в волне (если меньше наименьшего — берётся наименьшее)",
    },
    RuleDef {
        key: "rogue_moves",
        default: 4,
        min: 1,
        max: 16,
        note: "rogue-like: сколько ходов подряд делает игрок за свой ход — каждый другой фигурой",
    },
    RuleDef {
        key: "rogue_weight_pawn",
        default: 10,
        min: 0,
        max: 100,
        note: "rogue-like: вес пешки при выборе фигур волны — чем больше, тем чаще",
    },
    RuleDef {
        key: "rogue_weight_knight",
        default: 5,
        min: 0,
        max: 100,
        note: "rogue-like: вес коня в волне",
    },
    RuleDef {
        key: "rogue_weight_bishop",
        default: 5,
        min: 0,
        max: 100,
        note: "rogue-like: вес слона в волне",
    },
    RuleDef {
        key: "rogue_weight_rook",
        default: 3,
        min: 0,
        max: 100,
        note: "rogue-like: вес ладьи в волне",
    },
    RuleDef {
        key: "rogue_weight_queen",
        default: 1,
        min: 0,
        max: 100,
        note: "rogue-like: вес ферзя в волне",
    },
];

/// Числа внутри карт.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    WaveDamage,
    HealAmount,
    FundingDraw,
    IceShardDamage,
    HardWinterDamage,
    IceBlastRadius,
    NorthAttack,
    NorthHp,
    ChampionPercent,
    ColdRulerMana,
    VictoryTalesMana,
    MassSacrificePercent,
    OfferingMultiplier,
    MultiplyPercent,
    BastionPercent,
    AbsoluteZeroPercent,
}

pub struct ParamDef {
    pub param: Param,
    pub card: CardKind,
    pub key: &'static str,
    pub default: i32,
    pub min: i32,
    pub max: i32,
    pub note: &'static str,
}

const fn def(
    param: Param,
    card: CardKind,
    key: &'static str,
    default: i32,
    (min, max): (i32, i32),
    note: &'static str,
) -> ParamDef {
    ParamDef {
        param,
        card,
        key,
        default,
        min,
        max,
        note,
    }
}

/// В порядке `Param`.
pub const PARAMS: [ParamDef; 16] = [
    def(
        Param::WaveDamage,
        CardKind::FireWave,
        "damage",
        300,
        (0, 9999),
        "урон каждой фигуре вокруг выбранной (и чужой, и своей)",
    ),
    def(
        Param::HealAmount,
        CardKind::Heal,
        "amount",
        600,
        (0, 9999),
        "сколько здоровья лечит каждой своей фигуре рядом со слоном",
    ),
    def(
        Param::FundingDraw,
        CardKind::Funding,
        "draw",
        2,
        (0, 10),
        "сколько карт берёт из колоды",
    ),
    def(
        Param::IceShardDamage,
        CardKind::IceShard,
        "damage",
        500,
        (0, 9999),
        "урон выбранной фигуре соперника (выжившая замерзает)",
    ),
    def(
        Param::HardWinterDamage,
        CardKind::HardWinter,
        "damage",
        500,
        (0, 9999),
        "урон каждой замёрзшей фигуре соперника",
    ),
    def(
        Param::IceBlastRadius,
        CardKind::IceBlast,
        "radius",
        2,
        (1, 7),
        "радиус взрыва в клетках; бьёт атакой замёрзшей фигуры",
    ),
    def(
        Param::NorthAttack,
        CardKind::NorthWarrior,
        "attack",
        50,
        (0, 999),
        "прибавка к атаке за каждую замёрзшую фигуру соперника",
    ),
    def(
        Param::NorthHp,
        CardKind::NorthWarrior,
        "hp",
        100,
        (0, 999),
        "прибавка к здоровью за каждую замёрзшую фигуру соперника",
    ),
    def(
        Param::ChampionPercent,
        CardKind::Champion,
        "percent",
        25,
        (0, 300),
        "на сколько процентов растут атака и здоровье; столько же атаки добавляет каждое убийство",
    ),
    def(
        Param::ColdRulerMana,
        CardKind::ColdRuler,
        "mana",
        2,
        (0, 30),
        "маны за каждую замёрзшую фигуру соперника",
    ),
    def(
        Param::VictoryTalesMana,
        CardKind::VictoryTales,
        "mana",
        2,
        (0, 30),
        "маны за каждую фигуру соперника, убитую за партию",
    ),
    def(
        Param::MassSacrificePercent,
        CardKind::MassSacrifice,
        "percent",
        50,
        (0, 100),
        "сколько процентов здоровья теряет каждая своя фигура в квадрате",
    ),
    def(
        Param::OfferingMultiplier,
        CardKind::Offering,
        "multiplier",
        2,
        (1, 9),
        "во сколько раз больше очков жертвы даёт принесённая в жертву фигура",
    ),
    def(
        Param::MultiplyPercent,
        CardKind::Multiply,
        "percent",
        50,
        (1, 100),
        "сколько процентов здоровья остаётся у каждой из двух фигур (но не меньше 1)",
    ),
    def(
        Param::BastionPercent,
        CardKind::Bastion,
        "percent",
        50,
        (0, 100),
        "сколько процентов урона получают союзники рядом с ладьёй (50 — вдвое меньше)",
    ),
    def(
        Param::AbsoluteZeroPercent,
        CardKind::AbsoluteZero,
        "percent",
        50,
        (1, 100),
        "какая доля свободных фигур соперника (в процентах, с округлением вверх) замерзает навсегда",
    ),
];

/// Единица здоровья и урона (с 0.44, просьба пользователя): правила считают в прежних числах
/// (так не теряются доли — оборона, «воин», проценты карт), а игрок видит их делёнными на
/// `UNIT` — на экране и в `balance.ini`.
pub const UNIT: i32 = 300;

/// Здоровье или урон так, как их видит игрок: делённые на `UNIT`, доля — вверх (раненая фигура
/// с 100 из 900 — «1 из 3»).
pub fn shown(value: i32) -> i32 {
    (value.max(0) + UNIT - 1) / UNIT
}

/// Атака, здоровье и урон лавы в `balance.ini` — в единицах `UNIT`.
pub fn to_file(value: i32) -> i32 {
    if value % UNIT == 0 {
        value / UNIT
    } else {
        value
    }
}

/// Число из `balance.ini` во внутреннее: до 0.44 там стояли прежние числа (300, 900…) — такие
/// (от 100) читаются как есть, чтобы старый файл не стал в 300 раз сильнее.
pub fn from_file(n: i32) -> i32 {
    if n < 100 { n * UNIT } else { n }
}

/// Правило в единицах `UNIT` (урон).
pub fn rule_in_units(key: &str) -> bool {
    key == "lava_damage"
}

/// Стандартные характеристики фигур по `Kind as usize`: пешка, конь, слон, ладья, ферзь, король.
const DEFAULT_PIECES: [PieceNumbers; 6] = [
    PieceNumbers {
        attack: 300,
        hp: 900,
        points: 1,
    },
    PieceNumbers {
        attack: 300,
        hp: 900,
        points: 3,
    },
    PieceNumbers {
        attack: 300,
        hp: 900,
        points: 3,
    },
    PieceNumbers {
        attack: 300,
        hp: 1200,
        points: 5,
    },
    PieceNumbers {
        attack: 300,
        hp: 1200,
        points: 7,
    },
    PieceNumbers {
        attack: 300,
        hp: 1500,
        points: 9,
    },
];

/// Цена карты по умолчанию; 0 — бесплатна. Чем платят — решает тип карты.
fn default_cost(kind: CardKind) -> u8 {
    use CardKind::*;
    match kind {
        FireWave | Multiply | Heal | IceShard | Barrier => 2,
        Storm | Avalanche | IceBarrier | Encircle | Champion => 10,
        Swap | Funding => 1,
        Bastion | VictoryTales | IceWarrior | NorthWarrior => 3,
        IceWall => 6,
        MassSacrifice => 8,
        BloodGod | HardWinter => 5,
        AbsoluteZero => 30,
        ColdRuler | IceBlast => 4,
        Normal | Sacrifice | Victory | Offering => 0,
    }
}

const MAX_COST: i32 = 99;
const MAX_COPIES: i32 = 99;

/// Все числа баланса.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    rules: [i32; RULES.len()],
    pieces: [PieceNumbers; 6],
    costs: [u8; CardKind::COUNT],
    params: [i32; PARAMS.len()],
    decks: [Vec<(CardKind, u8)>; DECK_COUNT],
}

impl Default for Balance {
    fn default() -> Balance {
        Balance {
            rules: RULES.map(|r| r.default),
            pieces: DEFAULT_PIECES,
            costs: CardKind::ALL.map(default_cost),
            params: PARAMS.map(|p| p.default),
            decks: std::array::from_fn(|i| standard_deck(i)),
        }
    }
}

/// Стандартный состав колоды `index` (карты по порядку `CardKind`).
fn standard_deck(index: usize) -> Vec<(CardKind, u8)> {
    let mut deck = DEFAULT_RECIPES[index].to_vec();
    deck.sort_by_key(|&(kind, _)| kind as usize);
    deck
}

/// Что нашлось при разборе файла.
#[derive(Debug, Default)]
pub struct Report {
    /// Строки, которые не удалось прочесть; для них взяты стандартные значения.
    pub warnings: Vec<String>,
    /// В файле нет каких-то известных строк (новая версия игры).
    pub missing: bool,
}

impl Balance {
    pub fn rule(&self, rule: Rule) -> i32 {
        self.rules[rule as usize]
    }

    pub fn mana_per_turn(&self) -> u8 {
        self.rule(Rule::ManaPerTurn) as u8
    }

    pub fn cards_per_turn(&self) -> u8 {
        self.rule(Rule::CardsPerTurn) as u8
    }

    pub fn start_hand(&self) -> u8 {
        self.rule(Rule::StartHand) as u8
    }

    pub fn max_hand(&self) -> u8 {
        self.rule(Rule::MaxHand) as u8
    }

    pub fn piece(&self, kind: Kind) -> PieceNumbers {
        self.pieces[kind as usize]
    }

    /// Цена карты в очках её типа; 0 — бесплатна.
    pub fn cost(&self, kind: CardKind) -> u8 {
        self.costs[kind as usize]
    }

    pub fn param(&self, param: Param) -> i32 {
        self.params[param as usize]
    }

    /// Состав колоды `index` (по кругу): вид карты и число копий.
    pub fn deck(&self, index: usize) -> &[(CardKind, u8)] {
        &self.decks[index % DECK_COUNT]
    }

    /// Цены всех карт по `CardKind as usize` — по ним понятно, надо ли перерисовать лица карт.
    pub fn costs(&self) -> &[u8; CardKind::COUNT] {
        &self.costs
    }

    /// Разбирает текст файла: незнакомое и испорченное пропускается (с предупреждением).
    pub fn parse(text: &str) -> (Balance, Report) {
        let mut balance = Balance::default();
        let mut report = Report::default();
        let mut seen: HashSet<String> = HashSet::new();
        let mut deck_started = [false; DECK_COUNT];
        let mut section = String::new();
        for (index, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
            let number = index + 1;
            let line = raw.split([';', '#']).next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[') {
                section = name.trim_end_matches(']').trim().to_owned();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                report
                    .warnings
                    .push(format!("строка {number}: нет знака «=»: {line}"));
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            let Ok(n) = value.parse::<i32>() else {
                report.warnings.push(format!(
                    "строка {number}: «{key} = {value}» — нужно целое число"
                ));
                continue;
            };
            let spot = Spot {
                number,
                section: &section,
                key,
            };
            balance.apply(&spot, n, &mut seen, &mut deck_started, &mut report.warnings);
        }
        balance.check(&deck_started, &mut report.warnings);
        report.missing = expected_keys().any(|key| !seen.contains(&key));
        (balance, report)
    }

    fn apply(
        &mut self,
        spot: &Spot,
        n: i32,
        seen: &mut HashSet<String>,
        deck_started: &mut [bool; DECK_COUNT],
        warnings: &mut Vec<String>,
    ) {
        let (group, name) = spot.section.split_once('.').unwrap_or((spot.section, ""));
        let key = spot.key;
        let fits = |min: i32, max: i32, warnings: &mut Vec<String>| {
            let ok = (min..=max).contains(&n);
            if !ok {
                warnings.push(format!(
                    "строка {}: «{key} = {n}» — допустимо от {min} до {max}",
                    spot.number
                ));
            }
            ok
        };
        match group {
            "rules" => match RULES.iter().position(|r| r.key == key) {
                Some(i) if fits(RULES[i].min, RULES[i].max, warnings) => {
                    self.rules[i] = if rule_in_units(key) { from_file(n) } else { n };
                    seen.insert(format!("rules.{key}"));
                }
                Some(_) => {}
                // Числа прежних версий, которых больше нет, — молча пропускаются.
                None if RETIRED_RULES.contains(&key) => {}
                None => spot.unknown(warnings),
            },
            "piece" => {
                let Some(kind) = Kind::ALL.into_iter().find(|k| format!("{k:?}") == name) else {
                    return spot.unknown(warnings);
                };
                let piece = &mut self.pieces[kind as usize];
                let ok = match key {
                    "attack" if fits(1, 9999, warnings) => {
                        piece.attack = from_file(n);
                        true
                    }
                    "hp" if fits(1, 99999, warnings) => {
                        piece.hp = from_file(n);
                        true
                    }
                    "points" if fits(0, 99, warnings) => {
                        piece.points = n as u8;
                        true
                    }
                    "attack" | "hp" | "points" => return,
                    _ => return spot.unknown(warnings),
                };
                if ok {
                    seen.insert(format!("piece.{name}.{key}"));
                }
            }
            "card" => {
                let Some(kind) = CardKind::ALL.into_iter().find(|k| format!("{k:?}") == name)
                else {
                    return spot.unknown(warnings);
                };
                if key == "cost" {
                    if fits(0, MAX_COST, warnings) {
                        self.costs[kind as usize] = n as u8;
                        seen.insert(format!("card.{name}.cost"));
                    }
                } else if let Some(def) = PARAMS.iter().find(|p| p.card == kind && p.key == key) {
                    if fits(def.min, def.max, warnings) {
                        self.params[def.param as usize] = n;
                        seen.insert(format!("card.{name}.{key}"));
                    }
                } else {
                    spot.unknown(warnings);
                }
            }
            "deck" => {
                let Some(deck) = DECK_NAMES.iter().position(|d| *d == name) else {
                    return spot.unknown(warnings);
                };
                let Some(kind) = CardKind::ALL.into_iter().find(|k| format!("{k:?}") == key) else {
                    return spot.unknown(warnings);
                };
                if !fits(0, MAX_COPIES, warnings) {
                    return;
                }
                // Первая строка раздела вытесняет стандартный состав колоды.
                if !std::mem::replace(&mut deck_started[deck], true) {
                    self.decks[deck].clear();
                }
                self.decks[deck].retain(|&(k, _)| k != kind);
                if n > 0 {
                    self.decks[deck].push((kind, n as u8));
                }
                seen.insert(format!("deck.{name}"));
            }
            _ => spot.unknown(warnings),
        }
    }

    /// Проверки, которые видны только по всему файлу: колоды и руки.
    fn check(&mut self, deck_started: &[bool; DECK_COUNT], warnings: &mut Vec<String>) {
        for (i, started) in deck_started.iter().enumerate() {
            let total: usize = self.decks[i].iter().map(|&(_, n)| n as usize).sum();
            if *started && (total == 0 || total > MAX_DECK) {
                warnings.push(format!(
                    "колода «{}»: {total} карт — нужно от 1 до {MAX_DECK}; взят стандартный состав",
                    DECK_NAMES[i]
                ));
                self.decks[i] = standard_deck(i);
            }
            // Порядок карт в составе ни на что не влияет — приводим к одному виду.
            self.decks[i].sort_by_key(|&(kind, _)| kind as usize);
        }
        if self.start_hand() > self.max_hand() {
            warnings.push(format!(
                "start_hand ({}) больше max_hand ({}); взяты стандартные значения",
                self.start_hand(),
                self.max_hand()
            ));
            self.rules[Rule::StartHand as usize] = RULES[Rule::StartHand as usize].default;
            self.rules[Rule::MaxHand as usize] = RULES[Rule::MaxHand as usize].default;
        }
    }

    /// Текст файла: все числа с комментариями.
    pub fn to_ini(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "; Баланс «Боевых шахмат»: правьте числа и сохраняйте файл — новые значения\n\
             ; подхватятся при следующем нажатии «Начать играть» (перезапускать игру не нужно).\n\
             ; Всё после «;» — комментарий. Неверная строка (не число, вне пределов) заменяется\n\
             ; стандартным значением, а главное меню покажет предупреждение.\n\
             ; Удобнее править всё кнопками: меню игры → «Править баланс» (откроется страница в браузере).\n\n",
        );
        out.push_str("[rules]\n");
        for (rule, value) in RULES.iter().zip(self.rules) {
            let value = if rule_in_units(rule.key) {
                to_file(value)
            } else {
                value
            };
            let _ = writeln!(out, "{} = {value}   ; {}", rule.key, rule.note);
        }
        for kind in Kind::ALL {
            let piece = self.piece(kind);
            let _ = write!(
                out,
                "\n[piece.{kind:?}]   ; {}\nattack = {}   ; урон одного удара\nhp = {}   ; здоровье\n\
                 points = {}   ; очков победы убийце и очков жертвы хозяину\n",
                kind.title(),
                to_file(piece.attack),
                to_file(piece.hp),
                piece.points
            );
        }
        for kind in ordered_kinds() {
            let _ = write!(
                out,
                "\n[card.{kind:?}]   ; {} — {}\ncost = {}   ; цена: {}; 0 — бесплатна\n",
                kind.title(),
                family_name(kind.family()),
                self.cost(kind),
                payment(kind.family())
            );
            for def in PARAMS.iter().filter(|p| p.card == kind) {
                let _ = writeln!(
                    out,
                    "{} = {}   ; {}",
                    def.key,
                    self.param(def.param),
                    def.note
                );
            }
        }
        for (i, name) in DECK_NAMES.iter().enumerate() {
            let _ = write!(
                out,
                "\n[deck.{name}]   ; сколько копий каждой карты в колоде (0 — нет в колоде)\n"
            );
            for kind in ordered_kinds() {
                let copies = self.decks[i]
                    .iter()
                    .find(|&&(k, _)| k == kind)
                    .map_or(0, |&(_, n)| n);
                let _ = writeln!(out, "{kind:?} = {copies}   ; {}", kind.title());
            }
        }
        out
    }
}

/// Где в файле строка — для предупреждений.
struct Spot<'a> {
    number: usize,
    section: &'a str,
    key: &'a str,
}

impl Spot<'_> {
    fn unknown(&self, warnings: &mut Vec<String>) {
        warnings.push(format!(
            "строка {}: неизвестный параметр «{}» в разделе [{}]",
            self.number, self.key, self.section
        ));
    }
}

/// Все строки, которые должны быть в файле, — чтобы заметить недостающие.
fn expected_keys() -> impl Iterator<Item = String> {
    let rules = RULES.iter().map(|r| format!("rules.{}", r.key));
    let pieces = Kind::ALL
        .into_iter()
        .flat_map(|k| ["attack", "hp", "points"].map(|key| format!("piece.{k:?}.{key}")));
    let cards = CardKind::ALL.into_iter().flat_map(|kind| {
        std::iter::once(format!("card.{kind:?}.cost")).chain(
            PARAMS
                .iter()
                .filter(move |p| p.card == kind)
                .map(move |p| format!("card.{kind:?}.{}", p.key)),
        )
    });
    let decks = DECK_NAMES.iter().map(|name| format!("deck.{name}"));
    rules.chain(pieces).chain(cards).chain(decks)
}

/// Карты в порядке файла и конструктора: по типу (обычные, жертвы, победы), от дешёвых к
/// дорогим по стандартным ценам.
pub fn ordered_kinds() -> Vec<CardKind> {
    let mut kinds = CardKind::ALL.to_vec();
    kinds.sort_by_key(|&k| (k.family() as u8, default_cost(k)));
    kinds
}

fn family_name(family: Family) -> &'static str {
    match family {
        Family::Normal => "обычная карта",
        Family::Sacrifice => "карта жертвы",
        Family::Victory => "карта победы",
    }
}

fn payment(family: Family) -> &'static str {
    match family {
        Family::Normal => "мана",
        Family::Sacrifice => "очки жертвы",
        Family::Victory => "очки победы",
    }
}

// ---------- Общий доступ ----------

static CURRENT: RwLock<Option<&'static Balance>> = RwLock::new(None);
static DEFAULT: OnceLock<Balance> = OnceLock::new();

/// Действующий баланс: из последнего чтения файла, а до него (и в тестах) — стандартный.
pub fn balance() -> &'static Balance {
    let current = *CURRENT.read().unwrap_or_else(|e| e.into_inner());
    current.unwrap_or_else(|| DEFAULT.get_or_init(Balance::default))
}

/// Делает `new` действующим балансом. Старый остаётся в памяти (несколько килобайт на каждую
/// перемену) — зато правила держат на него ссылку без блокировок.
fn install(new: Balance) {
    if *balance() == new {
        return;
    }
    let leaked: &'static Balance = Box::leak(Box::new(new));
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = Some(leaked);
}

/// Стандартный баланс — действующий (сетевая партия: у обоих игроков одни числа).
pub fn use_default() {
    install(Balance::default());
}

/// `%APPDATA%\BattleChess\balance.ini` (на macOS — в `~/Library/Application Support`).
pub fn path() -> Option<PathBuf> {
    crate::settings::data_dir().map(|dir| dir.join("balance.ini"))
}

/// Читает файл баланса и делает его действующим; возвращает предупреждения. Нет файла — его
/// создают со стандартными значениями; не хватает строк — файл переписывается с ними (значения
/// пользователя сохраняются), но только если в нём нет ошибок: правки с опечатками не трогаем.
pub fn reload() -> Vec<String> {
    let Some(path) = path() else {
        return Vec::new();
    };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let (balance, report) = Balance::parse(&String::from_utf8_lossy(&bytes));
            if report.missing && report.warnings.is_empty() {
                write_file(&path, &balance);
            }
            install(balance);
            report.warnings
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let balance = Balance::default();
            write_file(&path, &balance);
            install(balance);
            Vec::new()
        }
        Err(e) => {
            install(Balance::default());
            vec![format!("не удалось прочитать balance.ini: {e}")]
        }
    }
}

fn write_file(path: &std::path::Path, balance: &Balance) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, balance.to_ini());
}

/// Заменяет в тексте файла раздел `[deck.<name>]` строками `body` (остальной текст не трогает);
/// нет такого раздела — дописывает его в конец.
fn replace_deck_section(text: &str, name: &str, body: &[String]) -> String {
    let header = format!("[deck.{name}]");
    let lines: Vec<&str> = text.lines().collect();
    let is_header = |line: &str| line.trim_start_matches('\u{feff}').trim().starts_with('[');
    let start = lines.iter().position(|line| {
        let line = line.trim_start_matches('\u{feff}');
        line.split(';').next().unwrap_or("").trim() == header
    });
    let mut out: Vec<String> = Vec::new();
    match start {
        Some(start) => {
            let end = lines[start + 1..]
                .iter()
                .position(|line| is_header(line))
                .map_or(lines.len(), |n| start + 1 + n);
            out.extend(lines[..=start].iter().map(|l| (*l).to_owned()));
            out.extend(body.iter().cloned());
            // Пустая строка между разделами — как в файле, который пишет игра.
            if end < lines.len() {
                out.push(String::new());
            }
            out.extend(lines[end..].iter().map(|l| (*l).to_owned()));
        }
        None => {
            out.extend(lines.iter().map(|l| (*l).to_owned()));
            out.push(String::new());
            out.push(format!(
                "{header}   ; сколько копий каждой карты в колоде (0 — нет в колоде)"
            ));
            out.extend(body.iter().cloned());
        }
    }
    out.join("\n") + "\n"
}

/// Строки раздела колоды: все виды карт по порядку файла, с числом копий (0 — нет).
fn deck_section_body(recipe: &[(CardKind, u8)]) -> Vec<String> {
    ordered_kinds()
        .into_iter()
        .map(|kind| {
            let copies = recipe
                .iter()
                .find(|&&(k, _)| k == kind)
                .map_or(0, |&(_, n)| n);
            format!("{kind:?} = {copies}   ; {}", kind.title())
        })
        .collect()
}

/// Записывает состав в раздел колоды `index` файла `balance.ini`: прежний файл остаётся рядом
/// как `balance.ini.bak`, остальные строки и комментарии не меняются. Файл должен быть без
/// ошибок в разделе колод — иначе игра на следующем чтении взяла бы стандартный состав.
pub fn set_deck_in_file(index: usize, recipe: &[(CardKind, u8)]) -> Result<(), String> {
    let path = path().ok_or("не найдена папка настроек (APPDATA)")?;
    // Нет файла — создаём стандартный, чтобы было что править.
    if !path.exists() {
        reload();
    }
    set_deck_in(&path, index, recipe)
}

fn set_deck_in(
    path: &std::path::Path,
    index: usize,
    recipe: &[(CardKind, u8)],
) -> Result<(), String> {
    let old = std::fs::read(path).map_err(|e| format!("не удалось прочитать balance.ini: {e}"))?;
    let text = String::from_utf8_lossy(&old);
    let name = DECK_NAMES[index % DECK_COUNT];
    let new = replace_deck_section(&text, name, &deck_section_body(recipe));
    let (check, _) = Balance::parse(&new);
    let mut wanted = recipe.to_vec();
    wanted.sort_by_key(|&(kind, _)| kind as usize);
    if check.deck(index) != wanted.as_slice() {
        return Err(
            "файл balance.ini нельзя править автоматически — исправьте в нём ошибки".to_owned(),
        );
    }
    let _ = std::fs::write(path.with_extension("ini.bak"), &old);
    std::fs::write(path, new).map_err(|e| format!("не удалось записать balance.ini: {e}"))
}

/// Текст файла баланса (нет файла — сначала создаётся стандартный): для редактора в браузере.
pub fn file_text() -> Result<String, String> {
    let path = path().ok_or("не найдена папка настроек (APPDATA)")?;
    if !path.exists() {
        reload();
    }
    std::fs::read(&path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|e| format!("не удалось прочитать balance.ini: {e}"))
}

/// Записывает присланный редактором текст в `balance.ini` (прежний файл — `balance.ini.bak`);
/// возвращает предупреждения разбора — их же покажет меню.
pub fn save_text(text: &str) -> Result<Vec<String>, String> {
    let path = path().ok_or("не найдена папка настроек (APPDATA)")?;
    save_text_to(&path, text)
}

fn save_text_to(path: &std::path::Path, text: &str) -> Result<Vec<String>, String> {
    let (_, report) = Balance::parse(text);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(old) = std::fs::read(path)
        && old != text.as_bytes()
    {
        let _ = std::fs::write(path.with_extension("ini.bak"), old);
    }
    // Через временный файл: игра никогда не прочтёт файл записанным наполовину.
    let temp = path.with_extension("ini.tmp");
    std::fs::write(&temp, text)
        .and_then(|()| std::fs::rename(&temp, path))
        .map_err(|e| format!("не удалось записать balance.ini: {e}"))?;
    Ok(report.warnings)
}

/// Открывает `balance.ini` в Блокноте (создав файл, если его ещё нет) — если редактор в браузере
/// не запустился.
pub fn open_in_editor() {
    reload();
    if let Some(path) = path() {
        let _ = if cfg!(windows) {
            std::process::Command::new("notepad.exe").arg(path).spawn()
        } else if cfg!(target_os = "macos") {
            std::process::Command::new("open")
                .arg("-t")
                .arg(path)
                .spawn()
        } else {
            std::process::Command::new("xdg-open").arg(path).spawn()
        };
    }
}

/// Слово по числу: `plural(2, ["карта", "карты", "карт"])` — «карты».
pub fn plural(n: i32, [one, few, many]: [&'static str; 3]) -> &'static str {
    match (n % 100, n % 10) {
        (11..=14, _) => many,
        (_, 1) => one,
        (_, 2..=4) => few,
        _ => many,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_follow_the_enums() {
        for (i, rule) in [
            Rule::ManaPerTurn,
            Rule::CardsPerTurn,
            Rule::StartHand,
            Rule::MaxHand,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(rule as usize, i);
        }
        for (i, def) in PARAMS.iter().enumerate() {
            assert_eq!(def.param as usize, i, "{}", def.key);
            assert!((def.min..=def.max).contains(&def.default), "{}", def.key);
        }
        for (i, kind) in CardKind::ALL.into_iter().enumerate() {
            assert_eq!(kind as usize, i);
        }
        for (i, kind) in [
            Kind::Pawn,
            Kind::Knight,
            Kind::Bishop,
            Kind::Rook,
            Kind::Queen,
            Kind::King,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(kind as usize, i);
        }
        assert_eq!(MAX_DECK, 64);
    }

    #[test]
    fn attack_and_health_are_written_in_units_and_old_files_still_read() {
        let text = Balance::default().to_ini();
        assert!(text.contains("[piece.Knight]   ; Конь\nattack = 1   ;"));
        assert!(text.contains("hp = 3   ; здоровье"));
        assert!(text.contains("lava_damage = 1   ;"));
        // Новый файл — в единицах, старый (до 0.44) — прежними числами: оба дают 900.
        for hp in ["3", "900"] {
            let (balance, _) = Balance::parse(&format!("[piece.Knight]\nhp = {hp}\n"));
            assert_eq!(balance.piece(Kind::Knight).hp, 900, "hp = {hp}");
        }
    }

    #[test]
    fn the_file_written_by_the_game_reads_back_as_the_same_balance() {
        let balance = Balance::default();
        let (read, report) = Balance::parse(&balance.to_ini());
        assert_eq!(read, balance);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert!(!report.missing);
        // Файл со своими числами тоже переживает запись и чтение.
        let (edited, _) =
            Balance::parse("[card.FireWave]\ncost = 5\ndamage = 450\n[rules]\nmax_hand = 9");
        let (again, report) = Balance::parse(&edited.to_ini());
        assert_eq!(again, edited);
        assert!(report.warnings.is_empty() && !report.missing);
    }

    #[test]
    fn edited_numbers_replace_the_defaults_and_the_rest_stays() {
        let text = "\u{feff}; свои правки\r\n[card.FireWave]\r\ncost = 5   ; дороже\r\ndamage = 450\r\n\
                    [piece.Pawn]\r\nattack=500\r\nhp = 1000\r\n[rules]\r\nmana_per_turn = 4\r\n";
        let (balance, report) = Balance::parse(text);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert!(report.missing, "остальных строк в файле нет");
        assert_eq!(balance.cost(CardKind::FireWave), 5);
        assert_eq!(balance.param(Param::WaveDamage), 450);
        assert_eq!(balance.piece(Kind::Pawn).attack, 500);
        assert_eq!(balance.piece(Kind::Pawn).hp, 1000);
        assert_eq!(balance.piece(Kind::Pawn).points, 1);
        assert_eq!(balance.mana_per_turn(), 4);
        assert_eq!(balance.cost(CardKind::Storm), 10);
        assert_eq!(balance.piece(Kind::King).hp, 1500);
        assert_eq!(balance.deck(0), Balance::default().deck(0));
    }

    #[test]
    fn broken_lines_keep_the_defaults_and_warn() {
        let text = "[card.FireWave]\ncost = много\ndamage = 100000\nweight = 3\n\
                    [card.Nope]\ncost = 1\n[piece.Pawn]\nhp = 0\nмусор\n[magic]\nx = 1\n\
                    [rules]\nmana_per_turn = -1\n";
        let (balance, report) = Balance::parse(text);
        assert_eq!(report.warnings.len(), 8, "{:?}", report.warnings);
        let defaults = Balance::default();
        assert_eq!(
            balance.cost(CardKind::FireWave),
            defaults.cost(CardKind::FireWave)
        );
        assert_eq!(balance.param(Param::WaveDamage), 300);
        assert_eq!(balance.piece(Kind::Pawn), defaults.piece(Kind::Pawn));
        assert_eq!(balance.mana_per_turn(), 2);
        assert!(report.warnings.iter().any(|w| w.contains("строка 2")));
    }

    #[test]
    fn decks_come_from_their_sections() {
        let text = "[deck.Лазурная]\nFireWave = 3\nStorm = 1\nNormal = 0\n[deck.Золотая]\nFireWave = 0\n\
                    [deck.Неизвестная]\nStorm = 1\n";
        let (balance, report) = Balance::parse(text);
        assert_eq!(
            balance.deck(0),
            [(CardKind::FireWave, 3), (CardKind::Storm, 1)]
        );
        // Пустая колода — стандартная, и об этом предупреждают.
        assert_eq!(balance.deck(3), Balance::default().deck(3));
        assert_eq!(report.warnings.len(), 2, "{:?}", report.warnings);
        // Слишком большая колода тоже заменяется.
        let (big, report) = Balance::parse("[deck.Багровая]\nFireWave = 99\nStorm = 99\n");
        assert_eq!(big.deck(1), Balance::default().deck(1));
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn a_deck_section_is_replaced_in_place_and_the_rest_stays() {
        let text = "[rules]\nmax_hand = 9   ; рука\n\n[deck.Лазурная]   ; моя\nFireWave = 2\nStorm = 1\n\n\
                    [deck.Багровая]\nFireWave = 4\n";
        let body = deck_section_body(&[(CardKind::Storm, 3)]);
        let new = replace_deck_section(text, "Лазурная", &body);
        assert!(new.starts_with("[rules]\nmax_hand = 9   ; рука\n\n[deck.Лазурная]   ; моя\n"));
        assert!(new.contains("Storm = 3   ; Буря\n") && new.contains("FireWave = 0   ; "));
        assert!(
            new.ends_with("\n\n[deck.Багровая]\nFireWave = 4\n"),
            "{new}"
        );
        let (read, report) = Balance::parse(&new);
        assert_eq!(read.deck(0), [(CardKind::Storm, 3)]);
        assert_eq!(read.deck(1), [(CardKind::FireWave, 4)]);
        assert_eq!(read.max_hand(), 9);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        // Повторная запись того же состава ничего не меняет.
        assert_eq!(replace_deck_section(&new, "Лазурная", &body), new);
        // Нет раздела — он дописывается в конец.
        let appended = replace_deck_section("[rules]\nmax_hand = 9\n", "Золотая", &body);
        assert!(appended.contains("\n\n[deck.Золотая]   ; "));
        assert_eq!(Balance::parse(&appended).0.deck(3), [(CardKind::Storm, 3)]);
    }

    #[test]
    fn a_deck_goes_into_the_file_and_the_old_file_is_kept() {
        let dir = std::env::temp_dir().join(format!("battle-chess-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("balance.ini");
        let old = Balance::default()
            .to_ini()
            .replace("max_hand = 7", "max_hand = 9");
        std::fs::write(&path, &old).unwrap();
        let recipe = [(CardKind::Storm, 3), (CardKind::FireWave, 5)];
        set_deck_in(&path, 2, &recipe).unwrap();
        let new = std::fs::read_to_string(&path).unwrap();
        let (read, report) = Balance::parse(&new);
        assert_eq!(
            read.deck(2),
            [(CardKind::FireWave, 5), (CardKind::Storm, 3)]
        );
        assert_eq!(
            read.deck(1),
            Balance::default().deck(1),
            "другие колоды не тронуты"
        );
        assert_eq!(read.max_hand(), 9, "чужие правки сохранились");
        assert!(report.warnings.is_empty() && !report.missing);
        assert_eq!(
            std::fs::read_to_string(dir.join("balance.ini.bak")).unwrap(),
            old
        );
        // Нет файла — нечего править.
        assert!(set_deck_in(&dir.join("нет.ini"), 0, &recipe).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_editor_saves_the_whole_file_and_keeps_the_old_one() {
        let dir = std::env::temp_dir().join(format!("battle-chess-save-{}", std::process::id()));
        let path = dir.join("BattleChess").join("balance.ini");
        let first = Balance::default().to_ini();
        assert!(
            save_text_to(&path, &first).unwrap().is_empty(),
            "папка создаётся сама"
        );
        let second = first.replace("max_hand = 7", "max_hand = 9");
        assert!(save_text_to(&path, &second).unwrap().is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), second);
        assert_eq!(
            std::fs::read_to_string(path.with_extension("ini.bak")).unwrap(),
            first
        );
        assert!(!path.with_extension("ini.tmp").exists());
        let broken = second.replace("max_hand = 9", "max_hand = много");
        assert_eq!(
            save_text_to(&path, &broken).unwrap().len(),
            1,
            "ошибки возвращаются"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_starting_hand_must_fit_into_the_hand() {
        let (balance, report) = Balance::parse("[rules]\nstart_hand = 9\nmax_hand = 5\n");
        assert_eq!((balance.start_hand(), balance.max_hand()), (3, 7));
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn plurals_in_russian() {
        let cards = ["карта", "карты", "карт"];
        let got: Vec<&str> = [1, 2, 5, 11, 12, 21, 22, 25, 30]
            .iter()
            .map(|&n| plural(n, cards))
            .collect();
        assert_eq!(
            got,
            [
                "карта",
                "карты",
                "карт",
                "карт",
                "карт",
                "карта",
                "карты",
                "карт",
                "карт"
            ]
        );
    }
}
