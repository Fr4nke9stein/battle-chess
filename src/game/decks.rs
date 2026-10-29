//! Колоды игры — пять, как в меню «Выбрать колоду» (порядок тот же, что у `DECKS` в
//! `app/menu.rs`). Здесь — состав (собран пользователем в странице-конструкторе, 2026-10-15):
//! вид карты и число копий; правила берут его через `crate::balance`. Порядок в списке не
//! важен — колода перемешивается.

use super::CardKind::{self, *};
use crate::balance::balance;

/// Состав колоды: вид карты и сколько таких карт в ней.
pub type Recipe = [(CardKind, u8)];

/// Сколько колод на выбор.
pub const DECK_COUNT: usize = 5;

/// Названия колод в порядке меню (рубашки — `SKIN_HUES` в `assets/cards.rs`).
pub const DECK_NAMES: [&str; DECK_COUNT] = [
    "Лазурная",
    "Багровая",
    "Изумрудная",
    "Золотая",
    "Аметистовая",
];

/// Больше карт в колоде быть не может (длина массива в `Cards`).
pub const MAX_DECK: usize = 64;

/// «Лазурная» — ледяная: больше заморозки, меньше простых карт.
const AZURE: &Recipe = &[
    (FireWave, 2),
    (IceShard, 2),
    (IceBlast, 2),
    (IceWall, 2),
    (MassSacrifice, 1),
    (IceBarrier, 1),
    (Avalanche, 1),
    (Storm, 1),
    (AbsoluteZero, 1),
    (Offering, 1),
    (Funding, 2),
    (ColdRuler, 2),
    (HardWinter, 1),
    (Swap, 1),
    (Heal, 2),
    (Barrier, 1),
    (Bastion, 1),
    (NorthWarrior, 2),
    (IceWarrior, 2),
    (VictoryTales, 2),
];

/// Колода, с которой игра началась до выбора колод: 15 обычных карт, 9 жертвы и 10 победы.
const CLASSIC: &Recipe = &[
    (Normal, 2),
    (FireWave, 4),
    (Multiply, 1),
    (IceShard, 1),
    (IceBlast, 1),
    (IceWall, 1),
    (MassSacrifice, 1),
    (IceBarrier, 1),
    (Avalanche, 1),
    (Storm, 1),
    (AbsoluteZero, 1),
    (Sacrifice, 2),
    (Offering, 1),
    (Funding, 2),
    (ColdRuler, 1),
    (BloodGod, 1),
    (HardWinter, 1),
    (Encircle, 1),
    (Swap, 2),
    (Heal, 2),
    (Barrier, 1),
    (Bastion, 1),
    (NorthWarrior, 1),
    (IceWarrior, 1),
    (VictoryTales, 1),
    (Champion, 1),
];

/// Стандартные составы колод по номерам из меню (названия — `DECK_NAMES`).
pub const RECIPES: [&Recipe; DECK_COUNT] = [AZURE, CLASSIC, CLASSIC, CLASSIC, CLASSIC];

/// Карты колоды `index` (по кругу) в порядке списка, не перемешанные: по составу из баланса.
#[allow(dead_code)] // карты убраны из интерфейса (0.40)
pub fn deck_cards(index: usize) -> Vec<CardKind> {
    balance()
        .deck(index)
        .iter()
        .flat_map(|&(kind, copies)| std::iter::repeat_n(kind, copies as usize))
        .collect()
}

/// Колода позиций, собранных из FEN без перемешивания (тесты и отладка): состав классической
/// колоды, а сверху — простые карты, чтобы руки были предсказуемы и без действий.
pub const UNSHUFFLED: [CardKind; 34] = [
    IceShard,
    Barrier,
    ColdRuler,
    IceBlast,
    VictoryTales,
    HardWinter,
    IceWarrior,
    NorthWarrior,
    AbsoluteZero,
    Offering,
    Encircle,
    BloodGod,
    Champion,
    Bastion,
    Storm,
    FireWave,
    Funding,
    Avalanche,
    Swap,
    IceBarrier,
    FireWave,
    Funding,
    Multiply,
    Swap,
    IceWall,
    FireWave,
    Heal,
    MassSacrifice,
    Heal,
    FireWave,
    Sacrifice,
    Normal,
    Sacrifice,
    Normal,
];

// ---------- Код колоды ----------
//
// Колоду можно передать короткой строкой и вставить в игру (Ctrl+V на экране «Выбрать колоду»):
// `BC1-` и base64url (без «=») от байтов: [сколько пар] + пары [номер вида карты, копий] по
// возрастанию номера + контрольный байт. Номер вида — `CardKind as usize` (порядок `CardKind::ALL`:
// новые виды карт только в конец, иначе старые коды сломаются — порядок закреплён тестом). Тот же
// формат читала и страница-конструктор (до 0.133).

const CODE_PREFIX: &str = "BC1-";
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Контрольный байт кода: ловит обрезанные и испорченные при копировании коды.
fn checksum(bytes: &[u8]) -> u8 {
    bytes
        .iter()
        .fold(0xA5u8, |sum, &b| sum.wrapping_mul(31).wrapping_add(b))
}

/// Код колоды: карты с нулём копий пропускаются, порядок в составе не важен.
pub fn encode_deck_code(recipe: &Recipe) -> String {
    let mut pairs: Vec<(u8, u8)> = recipe
        .iter()
        .filter(|&&(_, copies)| copies > 0)
        .map(|&(kind, copies)| (kind as u8, copies))
        .collect();
    pairs.sort_unstable();
    let mut bytes = vec![pairs.len() as u8];
    for (kind, copies) in pairs {
        bytes.extend([kind, copies]);
    }
    bytes.push(checksum(&bytes));
    let mut code = String::from(CODE_PREFIX);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        // Три байта — четыре знака, неполная группа — на один знак больше числа байтов.
        for i in 0..=chunk.len() {
            code.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    code
}

#[cfg(test)]
/// Состав колоды из кода (пробелы и переводы строк в нём не мешают) — или понятная ошибка.
pub fn decode_deck_code(code: &str) -> Result<Vec<(CardKind, u8)>, String> {
    let code: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    let Some(body) = code.strip_prefix(CODE_PREFIX) else {
        return Err(format!(
            "это не код колоды — он начинается с «{CODE_PREFIX}»"
        ));
    };
    let mut bytes = Vec::new();
    let (mut bits, mut have) = (0u32, 0);
    for c in body.trim_end_matches('=').bytes() {
        let value = ALPHABET
            .iter()
            .position(|&a| a == c)
            .ok_or_else(|| format!("в коде лишний знак «{}»", c as char))?;
        bits = bits << 6 | value as u32;
        have += 6;
        if have >= 8 {
            have -= 8;
            bytes.push((bits >> have) as u8);
            bits &= (1 << have) - 1;
        }
    }
    let damaged = "код повреждён — скопируйте его целиком";
    let (&sum, data) = bytes.split_last().ok_or(damaged)?;
    let pairs = *data.first().ok_or(damaged)? as usize;
    if data.len() != 1 + 2 * pairs || checksum(data) != sum {
        return Err(damaged.to_owned());
    }
    let mut recipe = Vec::with_capacity(pairs);
    for pair in data[1..].chunks(2) {
        let kind = *CardKind::ALL
            .get(pair[0] as usize)
            .ok_or("в коде есть неизвестная карта — возможно, он от новой версии игры")?;
        if pair[1] == 0 || recipe.iter().any(|&(k, _)| k == kind) {
            return Err(damaged.to_owned());
        }
        recipe.push((kind, pair[1]));
    }
    let total: usize = recipe.iter().map(|&(_, n)| n as usize).sum();
    if total == 0 || total > MAX_DECK {
        return Err(format!("в колоде {total} карт — нужно от 1 до {MAX_DECK}"));
    }
    recipe.sort_unstable_by_key(|&(kind, _)| kind as usize);
    Ok(recipe)
}
