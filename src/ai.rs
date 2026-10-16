//! Компьютерный соперник: перебор вариантов с альфа-бета отсечением.
//!
//! Оценка позиции учитывает ценность фигур (и короля — без него не сыграть карт) с
//! поправкой на оставшееся здоровье и простые позиционные бонусы. Ход в переборе — ход
//! фигурой и конец хода с ударами фигур (`Position::make_turn`). Выигрыш — когда у
//! соперника не осталось фигур. Карт компьютер пока не играет (с 0.32, просьба
//! пользователя).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::game::{Color, Kind, Move, Piece, Position, Square, file_of, rank_of};

/// Сила компьютера — одна, бывший уровень 2 «любитель» (с 0.31 других нет, просьба
/// пользователя): глубина перебора в полуходах, сколько можно думать над ходом и случайная
/// добавка к оценкам ходов (в сотых пешки) — компьютер иногда ошибается.
const DEPTH: u32 = 2;
const THINK_TIME: Duration = Duration::from_millis(800);
const NOISE: i32 = 40;

const MATE: i32 = 100_000;
const INFINITY: i32 = 1_000_000;
const MAX_PLY: i32 = 24;
/// Сколько стоит король: боец чуть слабее ладьи, и только с ним играют карты.
const KING_WORTH: i32 = 850;

/// Выбирает ход для стороны, которая ходит в позиции `pos`; `None`, если ходов нет.
/// Поиск обрывается по времени или когда выставлен флаг `cancel`.
pub fn choose_move(pos: &Position, cancel: &AtomicBool) -> Option<Move> {
    let mut moves = pos.legal_moves();
    if moves.is_empty() {
        return None;
    }

    let mut search = Search {
        deadline: Instant::now() + THINK_TIME,
        cancel,
        nodes: 0,
        aborted: false,
    };
    let mut rng = Rng::from_clock();
    let mut best = moves[0];

    for depth in 1..=DEPTH {
        let mut scored = Vec::with_capacity(moves.len());
        let mut alpha = -INFINITY;
        for &mv in &moves {
            let mut next = pos.clone();
            next.make_turn(mv);
            // Для шума нужны точные оценки всех ходов, без него хватает отсечения.
            let floor = if NOISE > 0 { -INFINITY } else { alpha };
            let score = -search.negamax(&next, depth - 1, -INFINITY, -floor, 1);
            if search.aborted {
                break;
            }
            alpha = alpha.max(score);
            scored.push((score, mv));
        }
        if search.aborted {
            // Недосчитанную глубину не используем: хватит результата предыдущей.
            break;
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        moves = scored.iter().map(|&(_, mv)| mv).collect();
        best = pick(&scored, NOISE, &mut rng);
        if scored[0].0.abs() >= MATE - MAX_PLY {
            break;
        }
    }
    Some(best)
}

fn pick(scored: &[(i32, Move)], noise: i32, rng: &mut Rng) -> Move {
    scored
        .iter()
        .map(|&(score, mv)| {
            // Выигрыш (у соперника не осталось фигур) шумом не размываем: его компьютер
            // видит всегда.
            let jitter = if score.abs() >= MATE - MAX_PLY {
                0
            } else {
                rng.range(noise)
            };
            (score + jitter, mv)
        })
        .max_by_key(|&(score, _)| score)
        .map(|(_, mv)| mv)
        .expect("есть хотя бы один ход")
}

struct Search<'a> {
    deadline: Instant,
    cancel: &'a AtomicBool,
    nodes: u64,
    aborted: bool,
}

impl Search<'_> {
    fn out_of_time(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes % 1024 == 0
            && (Instant::now() >= self.deadline || self.cancel.load(Ordering::Relaxed))
        {
            self.aborted = true;
        }
        self.aborted
    }

    fn negamax(&mut self, pos: &Position, depth: u32, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        if self.out_of_time() {
            return 0;
        }
        if let Some(score) = decided(pos, ply) {
            return score;
        }
        if depth == 0 {
            return evaluate(pos);
        }
        let mut moves = Vec::with_capacity(48);
        pos.pseudo_moves(&mut moves);

        if moves.is_empty() {
            // Ходить нечем (фигуры заморожены или заперты) — ход пропускается, фигуры бьют.
            let mut next = pos.clone();
            next.end_turn();
            return -self.negamax(&next, depth - 1, -beta, -alpha, ply + 1);
        }
        for mv in moves {
            let mut next = pos.clone();
            next.make_turn(mv);
            let score = -self.negamax(&next, depth - 1, -beta, -alpha, ply + 1);
            if self.aborted {
                return 0;
            }
            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        alpha
    }
}

/// Партия решена: у одной из сторон не осталось фигур. Оценка — для стороны, которая
/// ходит; чем раньше выигрыш, тем он дороже.
fn decided(pos: &Position, ply: i32) -> Option<i32> {
    let us = pos.side();
    if !pos.has_pieces(us) || pos.king_lost(us) {
        Some(-MATE + ply)
    } else if !pos.has_pieces(us.opponent()) || pos.king_lost(us.opponent()) {
        Some(MATE - ply)
    } else {
        None
    }
}

/// Оценка позиции с точки зрения стороны, которая ходит (в сотых долях пешки).
pub fn evaluate(pos: &Position) -> i32 {
    let material: i32 = pos.pieces().map(|(_, p)| value(p.kind)).sum();
    let endgame = material <= 2600;
    let mut score = 0;
    for (sq, piece) in pos.pieces() {
        let worth = worth(piece) + placement(piece, sq, endgame);
        score += if piece.color == Color::White {
            worth
        } else {
            -worth
        };
    }
    // Запас стоит половину фигуры на доске: его ещё надо выставить.
    for color in [Color::White, Color::Black] {
        let stock: i32 = pos
            .reserve(color)
            .iter()
            .map(|&(kind, ready, waiting)| value(kind) * i32::from(ready + waiting) / 2)
            .sum();
        score += if color == Color::White { stock } else { -stock };
    }
    let score = if pos.side() == Color::White {
        score
    } else {
        -score
    };
    score + 10 // право хода
}

fn value(kind: Kind) -> i32 {
    match kind {
        Kind::Pawn => 100,
        Kind::Knight => 310,
        Kind::Bishop => 320,
        Kind::Rook => 500,
        Kind::Queen => 900,
        Kind::King => 0,
    }
}

/// Ценность фигуры с учётом здоровья: раненую проще добить, она стоит меньше.
fn worth(piece: Piece) -> i32 {
    let hp = piece.hp;
    let max = piece.max_hp();
    // Король в `value` не считается: по сумме `value` определяется эндшпиль.
    let full = match piece.kind {
        Kind::King => KING_WORTH,
        kind => value(kind),
    };
    full * (max + 2 * hp) / (3 * max)
}

fn placement(piece: Piece, sq: Square, endgame: bool) -> i32 {
    let file = file_of(sq) as i32;
    let rank = match piece.color {
        Color::White => rank_of(sq) as i32,
        Color::Black => 7 - rank_of(sq) as i32,
    };
    // 0 в углу, 12 в центре доски.
    let centre = 14 - (2 * file - 7).abs() - (2 * rank - 7).abs();
    match piece.kind {
        Kind::Pawn => rank * 6 + if (2..=5).contains(&file) { rank * 3 } else { 0 },
        Kind::Knight => centre * 4 - 16,
        Kind::Bishop => centre * 3 - 8,
        Kind::Rook => centre + if rank == 6 { 16 } else { 0 },
        Kind::Queen => centre * 2,
        Kind::King if endgame => centre * 4,
        Kind::King => -centre * 3 + if rank == 0 { 12 } else { 0 },
    }
}

/// Простой xorshift: случайная добавка к оценкам ходов.
struct Rng(u64);

impl Rng {
    fn from_clock() -> Rng {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        Rng(nanos | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Случайное число от −n до n.
    fn range(&mut self, n: i32) -> i32 {
        if n <= 0 {
            return 0;
        }
        (self.next() % (2 * n as u64 + 1)) as i32 - n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Status, parse_square};

    #[test]
    fn finishes_off_the_last_piece() {
        // У белых остался раненый король h1: ладья a1 встаёт рядом на g1 и в конце хода
        // добивает его — фигур у белых нет.
        let mut pos = Position::from_fen("k7/1r6/8/8/8/8/8/r6K b - - 0 1").unwrap();
        pos.set_hp(parse_square("h1").unwrap(), 150);
        let mv = choose_move(&pos, &AtomicBool::new(false)).unwrap();
        assert_eq!(mv.to, parse_square("g1").unwrap());
        let mut game = Game::from_position(pos);
        game.play(mv).unwrap();
        game.end_turn();
        assert_eq!(
            game.status(),
            Status::Victory {
                winner: Color::Black
            }
        );
    }

    #[test]
    fn plays_a_legal_move_from_the_start() {
        let pos = Position::start();
        let mv = choose_move(&pos, &AtomicBool::new(false)).unwrap();
        assert!(pos.legal_moves().contains(&mv));
    }

    #[test]
    fn walks_up_to_a_free_kill() {
        // Белый ферзь подходит к раненой ладье d5 и в конце хода добивает её.
        let mut pos = Position::from_fen("4k3/8/8/3r4/8/8/3Q4/4K3 w - - 0 1").unwrap();
        pos.set_hp(parse_square("d5").unwrap(), 100);
        let mv = choose_move(&pos, &AtomicBool::new(false)).unwrap();
        pos.make_turn(mv);
        assert!(pos.piece(parse_square("d5").unwrap()).is_none());
    }

    #[test]
    fn plays_a_game_against_itself() {
        let mut game = Game::new([0, 1]);
        let cancel = AtomicBool::new(false);
        for _ in 0..60 {
            let Some(mv) = choose_move(game.position(), &cancel) else {
                game.end_turn();
                continue;
            };
            game.play(mv).unwrap();
            game.end_turn();
            if game.is_over() {
                break;
            }
        }
    }

    #[test]
    fn passes_when_every_piece_is_frozen() {
        let mut pos = Position::from_fen("4k3/8/8/8/8/8/PP6/KN6 w - - 0 1").unwrap();
        for name in ["a2", "b2", "b1"] {
            pos.set_frozen(parse_square(name).unwrap(), 1);
        }
        assert_eq!(choose_move(&pos, &AtomicBool::new(false)), None);
    }
}
