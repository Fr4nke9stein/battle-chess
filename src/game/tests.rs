use super::*;

// Тесты идут на стандартном балансе: `balance.ini` в них не читается.
const START_HAND: u8 = 3;
const MAX_HAND: u8 = 7;
const CARDS_PER_TURN: u8 = 3;
const WAVE_DAMAGE: i32 = 300;
const FUNDING_DRAW: u8 = 2;

#[test]
fn tests_run_on_the_default_balance() {
    let b = balance();
    assert_eq!(
        (
            b.start_hand(),
            b.max_hand(),
            b.cards_per_turn(),
            b.mana_per_turn()
        ),
        (START_HAND, MAX_HAND, CARDS_PER_TURN, 2)
    );
    assert_eq!(b.param(Param::WaveDamage), WAVE_DAMAGE);
    assert_eq!(b.param(Param::FundingDraw) as u8, FUNDING_DRAW);
    assert_eq!(*b, crate::balance::Balance::default());
}

fn position(fen: &str) -> Position {
    Position::from_fen(fen).expect("FEN")
}

fn sq(name: &str) -> Square {
    parse_square(name).expect("клетка")
}

fn find(pos: &Position, from: &str, to: &str) -> Move {
    *pos.legal_moves()
        .iter()
        .find(|m| m.from == sq(from) && m.to == sq(to))
        .unwrap_or_else(|| panic!("нет хода {from}-{to}"))
}

fn has_move(pos: &Position, from: &str, to: &str) -> bool {
    pos.legal_moves()
        .iter()
        .any(|m| m.from == sq(from) && m.to == sq(to))
}

fn hp(pos: &Position, name: &str) -> i32 {
    pos.piece(sq(name)).expect("фигура").hp
}

/// Удары в конце хода стороны, которая ходит: «b1-c3 500», убийство — «b1-c3 500†».
fn strikes(pos: &mut Position) -> Vec<String> {
    pos.end_turn()
        .strikes
        .iter()
        .map(|s| {
            let killed = if s.hit.killed { "†" } else { "" };
            format!(
                "{}-{} {}{killed}",
                square_name(s.from),
                square_name(s.hit.square),
                s.hit.damage
            )
        })
        .collect()
}

/// Позиция из списка фигур (буквы FEN) на клетках; ход стороны `side` ('w' или 'b').
fn board(pieces: &[(&str, char)], side: char) -> Position {
    let mut rows = Vec::new();
    for rank in (0..8).rev() {
        let mut row = String::new();
        let mut empty = 0;
        for file in 0..8 {
            let here = pieces
                .iter()
                .find(|(name, _)| sq(name) == (rank * 8 + file) as Square);
            match here {
                Some(&(_, letter)) => {
                    if empty > 0 {
                        row += &empty.to_string();
                        empty = 0;
                    }
                    row.push(letter);
                }
                None => empty += 1,
            }
        }
        if empty > 0 {
            row += &empty.to_string();
        }
        rows.push(row);
    }
    position(&format!("{} {side} - - 0 1", rows.join("/")))
}

#[test]
fn start_position_has_twenty_moves() {
    assert_eq!(Position::start().legal_moves().len(), 20);
}

#[test]
fn nobody_dies_from_one_hit_at_full_health() {
    for attacker in Kind::ALL {
        for defender in Kind::ALL {
            assert!(
                attacker.stats().attack < defender.stats().max_hp,
                "{attacker:?} убивает {defender:?} с одного удара"
            );
        }
    }
}

#[test]
fn moves_go_only_to_empty_squares() {
    // Ладья a1 упирается в коня a5, пешка e2 не бьёт коня d3 по диагонали.
    let pos = position("4k3/8/8/n7/8/3n4/4P3/R3K3 w - - 0 1");
    assert!(has_move(&pos, "a1", "a4"));
    assert!(!has_move(&pos, "a1", "a5") && !has_move(&pos, "a1", "a6"));
    assert!(has_move(&pos, "e2", "e3") && has_move(&pos, "e2", "e4"));
    assert!(!has_move(&pos, "e2", "d3"));
    assert!(pos.legal_moves().iter().all(|m| pos.piece(m.to).is_none()));
}

#[test]
fn pieces_strike_at_the_end_of_their_turn_without_an_answer() {
    // Конь b1 бьёт пешку c3 на 300; пешка не отвечает, конь остаётся на месте.
    let mut pos = position("4k3/8/8/8/8/2p5/8/1N2K3 w - - 0 1");
    assert_eq!(strikes(&mut pos), ["b1-c3 300"]);
    assert_eq!(hp(&pos, "c3"), 600);
    assert_eq!(hp(&pos, "b1"), 900);
    assert_eq!(pos.side(), Color::Black);
    // Бьёт только сторона, закончившая ход: пешка c3 бьёт вниз по диагонали — там пусто.
    assert!(strikes(&mut pos).is_empty());
}

#[test]
fn a_killer_stays_on_its_square() {
    let mut pos = position("4k3/8/8/8/8/2p5/8/1N2K3 w - - 0 1");
    pos.set_hp(sq("c3"), 300);
    assert_eq!(strikes(&mut pos), ["b1-c3 300†"]);
    assert_eq!(pos.piece(sq("b1")).map(|p| p.kind), Some(Kind::Knight));
    assert!(pos.piece(sq("c3")).is_none());
    assert_eq!(pos.points(Color::White, Points::Victory), 1);
    assert_eq!(pos.points(Color::Black, Points::Sacrifice), 1);
}

#[test]
fn a_piece_strikes_every_enemy_in_reach() {
    // Ферзь d4 бьёт и слона e5, и ладью d5, и пешку c5.
    let mut pos = position("4k3/8/8/2prb3/3Q4/8/8/4K3 w - - 0 1");
    pos.set_hp(sq("e5"), 300);
    // Сначала дороже: ладья, потом слон, потом пешка.
    assert_eq!(strikes(&mut pos), ["d4-d5 300", "d4-e5 300†", "d4-c5 300"]);
    assert_eq!(pos.piece(sq("d4")).map(|p| p.kind), Some(Kind::Queen));
}

#[test]
fn a_bishop_strikes_two_squares_diagonally() {
    for (target, reached) in [("f6", true), ("b2", true), ("g7", false), ("d6", false)] {
        let mut pos = board(&[("d4", 'B'), (target, 'p')], 'w');
        assert_eq!(!strikes(&mut pos).is_empty(), reached, "d4 → {target}");
    }
}

#[test]
fn rooks_and_queens_strike_two_squares() {
    for (letter, target, reached) in [
        ('R', "d6", true),
        ('R', "b4", true),
        ('R', "d7", false),
        ('R', "f6", false),
        ('Q', "f6", true),
        ('Q', "d2", true),
        ('Q', "b4", true),
        ('Q', "g7", false),
        ('Q', "a4", false),
        ('Q', "f5", false),
    ] {
        let mut pos = board(&[("d4", letter), (target, 'p')], 'w');
        assert_eq!(
            !strikes(&mut pos).is_empty(),
            reached,
            "{letter} d4 → {target}"
        );
    }
}

#[test]
fn long_strikes_hit_only_the_first_piece_on_a_line() {
    // Ладья d4: пешки d5 и d6 на одной линии — бьёт только d5; своя пешка e4 закрывает f4.
    let mut pos = board(
        &[
            ("d4", 'R'),
            ("d5", 'p'),
            ("d6", 'p'),
            ("e4", 'P'),
            ("f4", 'p'),
        ],
        'w',
    );
    let hits = strikes(&mut pos);
    assert!(hits.iter().any(|h| h.starts_with("d4-d5")));
    assert!(
        !hits
            .iter()
            .any(|h| h.starts_with("d4-d6") || h.starts_with("d4-f4"))
    );
    // Ферзь и слон — так же по диагонали.
    for letter in ['Q', 'B'] {
        let mut pos = board(&[("d4", letter), ("e5", 'p'), ("f6", 'p')], 'w');
        assert_eq!(strikes(&mut pos).len(), 1, "{letter}");
    }
}

#[test]
fn killed_pieces_go_to_the_killers_reserve_and_drop_after_two_turns() {
    // Конь b1 добивает пешку c3 — пешка в запасе белых, ждёт два начала их хода.
    let mut pos = position("4k3/8/8/8/8/2p5/8/1N2K3 w - - 0 1");
    pos.set_hp(sq("c3"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    pos.end_turn();
    assert_eq!(pos.reserve(Color::White), [(Kind::Pawn, 0, 1)]);
    let drops = |pos: &Position| pos.legal_moves().into_iter().filter(Move::is_drop).count();
    pos.make_move(find(&pos, "e8", "d8"));
    pos.end_turn();
    assert_eq!(drops(&pos), 0, "ход белых 1 — ещё ждёт");
    pos.make_move(find(&pos, "f1", "e1"));
    pos.end_turn();
    pos.make_move(find(&pos, "d8", "e8"));
    pos.end_turn();
    assert_eq!(pos.reserve(Color::White), [(Kind::Pawn, 1, 0)]);
    let moves = pos.legal_moves();
    let targets: Vec<Square> = moves.iter().filter(|m| m.is_drop()).map(|m| m.to).collect();
    assert!(
        targets.iter().all(|&t| (8..32).contains(&t)),
        "своя половина, не 1-я"
    );
    assert!(!targets.contains(&sq("b1")) && targets.contains(&sq("d4")));
    // Выставленная пешка d4 в этот ход не бьёт, со следующего — бьёт.
    let mut pos2 = pos.clone();
    pos2.set_hp(sq("e8"), 1500);
    let mut board_pos = position("4k3/8/8/2p5/8/8/8/4K3 w - - 0 1");
    board_pos.reserve = pos.reserve;
    let drop = Move::new(sq("d4"), sq("d4"), MoveKind::Drop(Kind::Pawn));
    assert!(board_pos.legal_moves().contains(&drop));
    board_pos.make_move(drop);
    assert!(board_pos.reserve(Color::White).is_empty());
    assert!(
        board_pos.end_turn().strikes.is_empty(),
        "в ход выставления не бьёт"
    );
    board_pos.make_move(find(&board_pos, "e8", "e7"));
    board_pos.end_turn();
    board_pos.make_move(find(&board_pos, "e1", "e2"));
    let turn = board_pos.end_turn();
    assert!(
        turn.strikes
            .iter()
            .any(|s| s.from == sq("d4") && s.hit.square == sq("c5"))
    );
}

#[test]
fn with_advance_melee_killers_take_the_square_and_ranged_stay() {
    // Конь b1 бьёт сначала ладью a3 (дороже), убивает, встаёт на a3 и пешку c3 не бьёт.
    let fen = "4k3/8/8/8/8/r1p5/8/1N2K3 w - - 0 1";
    let mut pos = position(fen).with_advance(true);
    pos.set_hp(sq("c3"), 300);
    pos.set_hp(sq("a3"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    let turn = pos.end_turn();
    assert!(pos.piece(sq("b1")).is_none());
    assert_eq!(pos.piece(sq("a3")).map(|p| p.kind), Some(Kind::Knight));
    assert_eq!(hp(&pos, "c3"), 300, "после взятия больше не бьёт");
    assert_eq!(turn.strikes.len(), 1);
    assert!(turn.strikes[0].advanced);
    // Из равных — сначала с меньшим здоровьем: раненая d2 раньше полной c3.
    let mut pos = position("4k3/8/8/8/8/2p5/3p4/1N2K3 w - - 0 1").with_advance(true);
    pos.set_hp(sq("d2"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    pos.end_turn();
    assert_eq!(
        pos.piece(sq("d2")).map(|p| (p.kind, p.color)),
        Some((Kind::Knight, Color::White))
    );
    assert_eq!(hp(&pos, "c3"), 900, "полную пешку c3 не бил");
    // Дальнобойная фигура, убив вплотную, тоже встаёт: ладья c2 добивает пешку c3.
    let mut pos = position("4k3/8/8/8/8/2p5/2R5/4K3 w - - 0 1").with_advance(true);
    pos.set_hp(sq("c3"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    pos.end_turn();
    assert_eq!(pos.piece(sq("c3")).map(|p| p.kind), Some(Kind::Rook));
    assert!(pos.piece(sq("c2")).is_none());
    // Без перемещения — конь стоит.
    let mut pos = position(fen);
    pos.set_hp(sq("c3"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    pos.end_turn();
    assert_eq!(pos.piece(sq("b1")).map(|p| p.kind), Some(Kind::Knight));
    // Ладья бьёт издалека и стоит на месте.
    let mut pos = position("4k3/8/8/8/2p5/8/2R5/4K3 w - - 0 1").with_advance(true);
    pos.set_hp(sq("c4"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    pos.end_turn();
    assert_eq!(pos.piece(sq("c2")).map(|p| p.kind), Some(Kind::Rook));
    assert!(pos.piece(sq("c4")).is_none());
    // Пешка, вставшая на последнюю горизонталь, — ферзь; второй раз в ход она не бьёт.
    let mut pos = position("1n2k3/P7/8/8/8/8/8/4K3 w - - 0 1").with_advance(true);
    pos.set_hp(sq("b8"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    let turn = pos.end_turn();
    assert_eq!(turn.strikes.len(), 1);
    assert_eq!(turn.strikes[0].promoted, Some(Kind::Queen));
    assert_eq!(pos.piece(sq("b8")).map(|p| p.kind), Some(Kind::Queen));
    assert!(pos.piece(sq("a7")).is_none());
}

#[test]
fn a_knight_intercepts_a_piece_moving_into_its_reach() {
    // Чёрный конь d5 сразу бьёт белую ладью, вставшую на c3; дальше ладья не перехватывается.
    let mut pos = position("4k3/8/8/3n4/8/8/8/R3K3 w - - 0 1");
    let outcome = pos.make_move(find(&pos, "a1", "a3"));
    assert!(outcome.reactions.is_empty(), "a3 конь не достаёт");
    let mut pos = position("4k3/8/8/3n4/8/8/8/2R1K3 w - - 0 1");
    let outcome = pos.make_move(find(&pos, "c1", "c3"));
    assert_eq!(outcome.reactions.len(), 1);
    let strike = outcome.reactions[0];
    assert_eq!((strike.from, strike.hit.square), (sq("d5"), sq("c3")));
    assert_eq!(hp(&pos, "c3"), 1200 - 300);
    // Перехват убил — фигура уходит в запас коня.
    let mut pos = position("4k3/8/8/3n4/8/8/8/2R1K3 w - - 0 1");
    pos.set_hp(sq("c1"), 300);
    let outcome = pos.make_move(find(&pos, "c1", "c3"));
    assert!(outcome.reactions[0].hit.killed);
    assert!(pos.piece(sq("c3")).is_none());
    assert_eq!(pos.reserve(Color::Black), [(Kind::Rook, 0, 1)]);
    // Замёрзший конь не перехватывает; невидимого коня соперника — тоже нет (свой ход).
    let mut pos = position("4k3/8/8/3n4/8/8/8/2R1K3 w - - 0 1");
    pos.set_frozen(sq("d5"), 1);
    assert!(pos.make_move(find(&pos, "c1", "c3")).reactions.is_empty());
    // Пешка на подошедшую фигуру не реагирует: ладья встала на c4 рядом с чёрной d5.
    let mut pos = position("4k3/8/8/3p4/8/8/8/2R1K3 w - - 0 1");
    assert!(pos.make_move(find(&pos, "c1", "c4")).reactions.is_empty());
}

#[test]
fn a_pawn_strikes_en_passant() {
    // Белая e2–e4 проходит через e3, которую бьёт чёрная d4: та сразу бьёт пешку на e4.
    let mut pos = position("4k3/8/8/8/3p4/8/4P3/4K3 w - - 0 1");
    let outcome = pos.make_move(find(&pos, "e2", "e4"));
    assert_eq!(outcome.reactions.len(), 1);
    let strike = outcome.reactions[0];
    assert_eq!((strike.from, strike.hit.square), (sq("d4"), sq("e4")));
    // Ход на одну клетку — не на проходе: e2–e3 встаёт прямо под удар d4, но пешка молчит.
    let mut pos = position("4k3/8/8/8/3p4/8/4P3/4K3 w - - 0 1");
    assert!(pos.make_move(find(&pos, "e2", "e3")).reactions.is_empty());
    // Двойной ход мимо пешки, которая e3 не достаёт (b4), — тоже нет.
    let mut pos = position("4k3/8/8/8/1p6/8/4P3/4K3 w - - 0 1");
    assert!(pos.make_move(find(&pos, "e2", "e4")).reactions.is_empty());
}

#[test]
fn a_piece_comes_back_from_the_reserve_at_most_twice() {
    // Белая пешка d4 бьёт чёрного коня, вернувшегося уже дважды: в запас он не уходит.
    let mut pos = position("4k3/8/8/4n3/3P4/8/8/4K3 w - - 0 1");
    if let Some(p) = &mut pos.board[sq("e5") as usize] {
        p.reborn = MAX_REBIRTHS;
        p.hp = 300;
    }
    pos.make_move(find(&pos, "e1", "f1"));
    let turn = pos.end_turn();
    assert!(turn.strikes[0].hit.killed);
    assert!(pos.reserve(Color::White).is_empty());
    // Вернувшийся однажды — уходит и при выставлении будет вернувшимся дважды.
    let mut pos = position("4k3/8/8/4n3/3P4/8/8/4K3 w - - 0 1");
    if let Some(p) = &mut pos.board[sq("e5") as usize] {
        p.reborn = 1;
        p.hp = 300;
    }
    pos.make_move(find(&pos, "e1", "f1"));
    pos.end_turn();
    pos.make_move(find(&pos, "e8", "d8"));
    pos.end_turn();
    pos.make_move(find(&pos, "f1", "e1"));
    pos.end_turn();
    pos.make_move(find(&pos, "d8", "e8"));
    pos.end_turn();
    let drop = Move::new(sq("a3"), sq("a3"), MoveKind::Drop(Kind::Knight));
    pos.make_move(drop);
    assert_eq!(pos.piece(sq("a3")).unwrap().reborn, 2);
}

#[test]
fn boosted_bishops_are_not_electric() {
    let mut pos = position("4k3/8/8/8/8/8/8/2B1K3 w - - 0 1");
    pos.boost(Color::White);
    assert!(!pos.piece(sq("c1")).unwrap().spark);
}

#[test]
fn strikes_reach_one_square_by_the_rules_of_the_move() {
    let neighbours = ["c3", "c4", "c5", "d3", "d5", "e3", "e4", "e5"];
    let jumps = ["b3", "b5", "c2", "c6", "e2", "e6", "f3", "f5"];
    for (letter, reach) in [
        ('P', &["c5", "e5"][..]),
        ('N', &jumps[..]),
        ('B', &["c3", "c5", "e3", "e5"][..]),
        ('R', &["c4", "d3", "d5", "e4"][..]),
        ('Q', &neighbours[..]),
        ('K', &neighbours[..]),
    ] {
        for target in neighbours.iter().chain(&jumps) {
            let mut pos = board(&[("d4", letter), (target, 'p')], 'w');
            let struck = !strikes(&mut pos).is_empty();
            assert_eq!(struck, reach.contains(target), "{letter} d4 → {target}");
        }
    }
    // Чёрная пешка бьёт вниз по диагонали, не прямо.
    let mut pos = board(&[("d5", 'p'), ("c4", 'P'), ("d4", 'P')], 'b');
    assert_eq!(strikes(&mut pos), ["d5-c4 300"]);
}

#[test]
fn strikes_go_from_a8_down_the_file_then_to_the_next_file() {
    // Ладья a4 бьёт раньше ладьи b3 и добивает пешку b4 — ладья b3 бьёт пешку c3.
    let mut pos = position("4k3/8/8/8/Rp6/1Rp5/8/4K3 w - - 0 1");
    pos.set_hp(sq("b4"), 300);
    assert_eq!(strikes(&mut pos), ["a4-b4 300†", "b3-c3 300"]);
    // На одной вертикали сверху вниз: ладья a5 раньше ладьи a3.
    let mut pos = position("4k3/8/8/R7/p7/Rp6/8/4K3 w - - 0 1");
    pos.set_hp(sq("a4"), 300);
    assert_eq!(strikes(&mut pos), ["a5-a4 300†", "a3-b3 300"]);
    assert_eq!(strike_order(sq("a8")), 0);
    assert_eq!(strike_order(sq("a1")), 7);
    assert_eq!(strike_order(sq("b8")), 8);
    assert_eq!(strike_order(sq("h1")), 63);
}

#[test]
fn frozen_pieces_do_not_strike_but_can_be_struck() {
    let fen = "4k3/8/8/8/8/2p5/8/1N2K3 w - - 0 1";
    let mut pos = position(fen);
    pos.set_frozen(sq("b1"), 1);
    assert!(strikes(&mut pos).is_empty());
    let mut pos = position(fen);
    pos.set_frozen(sq("c3"), 1);
    assert_eq!(strikes(&mut pos), ["b1-c3 300"]);
}

#[test]
fn the_game_ends_when_a_king_dies() {
    // Конь f3 бьёт раненого короля e1 (у ладьи h2 здоровья больше): у белых осталась ладья,
    // но без короля партия кончена (с 0.44).
    let mut pos = position("4k3/8/8/8/8/5n2/8/4K2R w - - 0 1");
    pos.set_hp(sq("e1"), 300);
    let mut game = Game::from_position(pos);
    assert!(game.position().any_card_playable());
    game.play(find(game.position(), "h1", "h2")).unwrap();
    assert!(game.end_turn());
    game.play(find(game.position(), "e8", "d8")).unwrap();
    assert!(game.end_turn());
    let pos = game.position();
    assert_eq!(pos.king_square(Color::White), None);
    assert!(pos.king_lost(Color::White));
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::Black
        }
    );
    assert_eq!(
        pos.points(Color::Black, Points::Victory),
        9,
        "король стоит 9 очков"
    );
    // Без короля карты не играют и шаха нет.
    assert!(!pos.can_play_card() && !pos.any_card_playable());
    assert!(!pos.in_check(Color::White));
    assert_eq!(pos.king_danger(Color::White), 0);
}

#[test]
fn check_is_only_a_warning_and_strikes_end_the_game() {
    // Ладья e2 достаёт короля e1 на 300.
    let mut pos = position("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1");
    assert_eq!(pos.king_danger(Color::White), 300);
    pos.set_hp(sq("e1"), 301);
    assert!(!pos.in_check(Color::White));
    pos.set_hp(sq("e1"), 300);
    assert!(pos.in_check(Color::White));
    // Король встаёт на d2 — по-прежнему рядом с ладьёй — и бьёт её сам.
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "e1", "d2")).unwrap();
    assert!(game.end_turn());
    assert_eq!(hp(game.position(), "e2"), 900);
    // Чёрные кончают ход — ладья убивает короля, последнюю фигуру белых.
    game.play(find(game.position(), "e8", "d8")).unwrap();
    assert!(game.end_turn());
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::Black
        }
    );
    assert!(game.legal_moves().is_empty());
    assert!(!game.end_turn());
    let record = game.history().last().unwrap();
    assert_eq!(record.loser, Some(Color::White));
    assert_eq!(record.notation().suffix, "#");
    assert!(record.description().contains("У белых не осталось фигур"));
    assert_eq!(game.kills_by(Color::Black).len(), 1);
    // Отмена возвращает партию.
    assert!(game.undo());
    assert_eq!(game.status(), Status::Ongoing);
}

#[test]
fn there_is_no_checkmate() {
    // Раненому королю h1 некуда деться от ладьи g1 — партия всё равно идёт.
    let mut pos = position("k7/8/8/8/8/8/8/6rK w - - 0 1");
    pos.set_hp(sq("h1"), 150);
    assert!(pos.in_check(Color::White));
    let game = Game::from_position(pos);
    assert_eq!(game.status(), Status::Ongoing);
    assert!(!game.legal_moves().is_empty());
}

#[test]
fn wounded_rook_keeps_castling_rights_killed_rook_loses_them() {
    let fen = "4k3/8/8/8/8/8/7r/4K2R b K - 0 1";
    let castles = |pos: &Position| {
        pos.legal_moves()
            .iter()
            .any(|m| m.kind == MoveKind::CastleShort)
    };
    let mut pos = position(fen);
    assert_eq!(strikes(&mut pos), ["h2-h1 300"]);
    assert!(castles(&pos));

    let mut pos = position(fen);
    pos.set_hp(sq("h1"), 300);
    assert_eq!(strikes(&mut pos), ["h2-h1 300†"]);
    assert!(!castles(&pos));
}

#[test]
fn castling_moves_both_pieces() {
    let mut pos = position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    let outcome = pos.make_turn(find(&pos, "e1", "g1"));
    assert_eq!(outcome.rook, Some((sq("h1"), sq("f1"))));
    assert_eq!(pos.piece(sq("g1")).unwrap().kind, Kind::King);
    assert_eq!(pos.piece(sq("f1")).unwrap().kind, Kind::Rook);
    let outcome = pos.make_move(find(&pos, "e8", "c8"));
    assert_eq!(outcome.rook, Some((sq("a8"), sq("d8"))));
}

#[test]
fn pawn_promotes_on_reaching_the_last_rank() {
    let fen = "4k3/1P6/8/8/8/8/8/4K3 w - - 0 1";
    let pos = position(fen);
    let pushes: Vec<Move> = pos
        .legal_moves()
        .into_iter()
        .filter(|m| m.from == sq("b7"))
        .collect();
    assert_eq!(pushes.len(), 4);
    assert!(
        pushes
            .iter()
            .all(|m| m.to == sq("b8") && m.promotion.is_some()),
        "только вперёд, с превращением"
    );
    assert!(
        pushes.iter().all(|m| m.promotion != Some(Kind::King)),
        "в короля пешка не превращается"
    );
    let queen = pushes
        .into_iter()
        .find(|m| m.promotion == Some(Kind::Queen))
        .unwrap();
    let king = Move {
        promotion: Some(Kind::King),
        ..queen
    };
    let mut game = Game::from_position(pos.clone());
    assert_eq!(game.play(king), None);
    let mut forged = pos.clone();
    forged.make_move(king);
    assert_eq!(forged.piece(sq("b8")).map(|p| p.kind), Some(Kind::Pawn));

    // Новая фигура — с полным здоровьем.
    let mut pos = pos;
    pos.set_hp(sq("b7"), 200);
    let outcome = pos.make_turn(queen);
    assert_eq!(outcome.promoted, Some(Kind::Queen));
    assert_eq!(
        pos.piece(sq("b8")),
        Some(Piece::new(Kind::Queen, Color::White))
    );
    // Заслонённая пешка стоит.
    let blocked = position("1n2k3/1P6/8/8/8/8/8/4K3 w - - 0 1");
    assert!(!blocked.legal_moves().iter().any(|m| m.from == sq("b7")));
}

/// Случайная карта из руки; огненная волна — на случайную подходящую свою фигуру,
/// обмен — на случайную пару.
fn random_card(game: &mut Game, pick: usize) -> Option<CardPlay> {
    let pos = game.position();
    if game.is_over() || !pos.can_play_card() {
        return None;
    }
    let hand = pos.cards(pos.side()).hand();
    let index = pick % hand.len();
    // Нельзя сыграть (не хватает очков, нет фигур) — карту не играем.
    if !pos.card_playable(index) {
        return None;
    }
    let mut targets = Vec::new();
    if hand[index].targeted() {
        let first = pos.card_targets(index);
        targets.push(*first.get(pick % first.len().max(1))?);
    }
    if hand[index] == CardKind::Swap {
        let partners = pos.swap_partners(targets[0]);
        targets.push(partners[pick / 7 % partners.len()]);
    }
    Some(
        game.play_card(index, &targets)
            .expect("карта разыгрывается"),
    )
}

/// Король стороны там, где его помнит позиция, а погибшего короля нет на доске.
fn assert_kings_tracked(game: &Game) {
    let pos = game.position();
    for color in [Color::White, Color::Black] {
        let on_board = pos
            .pieces()
            .find(|&(_, p)| p.kind == Kind::King && p.color == color)
            .map(|(sq, _)| sq);
        if on_board != pos.king_square(color) {
            let history = game.history();
            for record in &history[history.len().saturating_sub(6)..] {
                eprintln!("{}", record.description());
            }
        }
        assert_eq!(on_board, pos.king_square(color));
    }
}

#[test]
fn random_games_end_when_a_side_has_no_pieces() {
    // Случайные партии с перевесом ходов к врагу: много ударов и убийств, гибнут и короли.
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n as u64) as usize
    };
    let mut kills = 0;
    let mut strikes = 0;
    let mut waves = 0;
    let mut swaps = 0;
    let mut kings_lost = 0;
    let mut victories = 0;
    let mut freezes = 0;
    let mut passes = 0;
    let mut encirclements = 0;
    for _ in 0..60 {
        let mut game = Game::new([next(DECK_COUNT), next(DECK_COUNT)]);
        for _ in 0..300 {
            if game.is_over() {
                break;
            }
            // Иногда карты до хода фигурой (до трёх), огненная волна — на случайную свою
            // фигуру.
            let mut cards = Vec::new();
            while next(3) == 0
                && let Some(play) = random_card(&mut game, next(64))
            {
                cards.push(play);
            }
            assert!(game.position().cards_played() <= CARDS_PER_TURN);
            let mover = game.position().side();
            if !game.is_over() && game.legal_moves().is_empty() {
                // Ходить нечем (фигуры заморожены) — ход фигурой пропускается.
                assert!(game.can_end_turn());
                passes += 1;
            } else if !game.is_over() {
                // Чаще — ходы, после которых наши фигуры достают больше врагов.
                let pos = game.position();
                let reached = |p: &Position| {
                    p.pieces()
                        .filter(|&(sq, piece)| piece.color != mover && p.max_damage(sq, mover) > 0)
                        .count()
                };
                let before = reached(pos);
                let moves = game.legal_moves();
                let closer: Vec<Move> = moves
                    .iter()
                    .copied()
                    .filter(|&mv| {
                        let mut after = pos.clone();
                        after.make_move(mv);
                        reached(&after) > before
                    })
                    .collect();
                let pool = if !closer.is_empty() && next(4) != 0 {
                    closer
                } else {
                    moves.to_vec()
                };
                let mv = pool[next(pool.len())];
                assert!(mv.is_drop() || !game.position().piece(mv.from).unwrap().is_frozen());
                assert!(game.position().piece(mv.to).is_none(), "ходом не бьют");
                game.play(mv).unwrap();
            }
            // Ход кончается сам, когда карт не сыграть, иначе — ещё карты или кнопка.
            while !game.is_over() && game.position().side() == mover {
                let pos = game.position();
                assert!(pos.any_card_playable() || !pos.moved());
                let play = if next(2) == 0 {
                    random_card(&mut game, next(64))
                } else {
                    None
                };
                match play {
                    Some(play) => cards.push(play),
                    // Пропускали ход, а карта освободила фигуру (размножение, жертва) —
                    // теперь надо сходить.
                    None if !game.legal_moves().is_empty() => {
                        let moves = game.legal_moves();
                        let mv = moves[next(moves.len())];
                        game.play(mv).unwrap();
                    }
                    None => assert!(game.end_turn()),
                }
            }
            for play in &cards {
                waves += usize::from(play.kind == CardKind::FireWave);
                swaps += usize::from(play.kind == CardKind::Swap);
                freezes += usize::from(play.kind.area().is_some());
            }
            assert_kings_tracked(&game);
            let pos = game.position();
            // Обмен не ставит пешку на последнюю горизонталь.
            for (sq, piece) in pos.pieces() {
                assert!(piece.kind != Kind::Pawn || rank_of(sq) != piece.color.last_rank());
            }
            match game.status() {
                Status::Victory { winner } => {
                    // Без фигур обе стороны остаются только от карты, задевшей всех (волна,
                    // ледяной взрыв), — тогда проигрывает сыгравший её.
                    let lost = |c: Color| !pos.has_pieces(c) || pos.king_lost(c);
                    assert!(lost(winner.opponent()));
                    assert!(!lost(winner) || pos.side() == winner.opponent());
                    victories += 1;
                }
                Status::Encircled { winner } => {
                    assert!(pos.encircles(winner));
                    encirclements += 1;
                }
                _ => {
                    assert!(pos.has_pieces(Color::White) && pos.has_pieces(Color::Black));
                    assert_ne!(pos.side(), mover, "ход перешёл к сопернику");
                }
            }
            // Без короля карты не играют.
            if pos.king_square(pos.side()).is_none() {
                assert!(!pos.any_card_playable());
            }
        }
        // Удары: бьёт сторона, закончившая ход, по чужим фигурам; убийства — ударами и картами.
        for record in game.history() {
            // Перехват конём в записи хода — удар соперника.
            let striker = match record.event {
                Event::Move(_) => record.color().opponent(),
                _ => record.color(),
            };
            for strike in record.strikes() {
                assert_eq!(strike.attacker.color, striker);
                assert_ne!(strike.hit.target.color, striker);
                assert!(!strike.attacker.is_frozen());
                strikes += 1;
                kills += usize::from(strike.hit.killed);
            }
            if let Event::Card(play) = &record.event {
                kills += play.hits.iter().filter(|h| h.killed).count();
            }
        }
        kings_lost += [Color::White, Color::Black]
            .into_iter()
            .filter(|&c| game.position().king_square(c).is_none())
            .count();
    }
    eprintln!(
        "ударов {strikes}, убийств {kills}, волн {waves}, обменов {swaps}, заморозок {freezes}, \
         пропусков {passes}, погибших королей {kings_lost}, побед {victories}, окружений \
         {encirclements}"
    );
    assert!(freezes > 10, "заморозок всего {freezes}");
    assert!(strikes > 1000, "ударов всего {strikes}");
    assert!(kills > 200, "убийств всего {kills}");
    assert!(waves > 50, "огненных волн всего {waves}");
    assert!(swaps > 30, "обменов всего {swaps}");
    assert!(kings_lost > 10, "погибших королей всего {kings_lost}");
    assert!(victories > 10, "побед всего {victories}");
}

#[test]
fn threefold_repetition_is_a_draw() {
    let mut game = Game::new([1, 1]);
    for _ in 0..2 {
        for (from, to) in [("g1", "f3"), ("g8", "f6"), ("f3", "g1"), ("f6", "g8")] {
            let mv = find(game.position(), from, to);
            game.play(mv).unwrap();
            game.end_turn();
        }
    }
    assert_eq!(game.status(), Status::Repetition);
}

#[test]
fn undo_restores_the_position() {
    // Неперемешанные колоды: у белых в руке простые карты, ход ждёт кнопку.
    let mut game = Game::from_position(Position::start());
    let start = game.position().clone();
    let mv = find(game.position(), "e2", "e4");
    game.play(mv).unwrap();
    assert!(game.undo());
    assert_eq!(game.position(), &start);
    assert!(game.history().is_empty());
}

#[test]
fn notation_and_description_show_the_strikes() {
    // Конь b1 ранит пешку c3, слон d2 добивает её (из равных — раненую первой) и бьёт e3.
    let mut pos = position("4k3/8/8/8/8/2p1p3/3B4/1N2K3 w - - 0 1");
    pos.set_hp(sq("c3"), 400);
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "e1", "f1")).unwrap();
    assert!(game.end_turn());
    let record = &game.history()[1];
    assert_eq!(
        record.notation(),
        Notation {
            text: "Кb1⚔c3, Сd2×c3, Сd2⚔e3".to_owned(),
            damage: Some(900),
            suffix: "",
        }
    );
    let description = record.description();
    assert!(description.starts_with("Белые закончили ход."));
    assert!(description.contains("Конь b1 ударил пешку на c3: −1 HP, осталось 1/3."));
    assert!(description.contains("Слон d2 убил пешку на c3 (−1 HP)."));
    assert_eq!(game.kills_by(Color::White).len(), 1);
}

#[test]
fn each_side_draws_a_card_at_the_start_of_its_turn() {
    // Неперемешанные колоды: у белых простые карты, после хода ход ждёт кнопку.
    let mut game = Game::from_position(Position::start());
    // Белые ходят первыми и уже взяли карту, чёрные возьмут в начале своего хода.
    let white = game.position().cards(Color::White);
    let black = game.position().cards(Color::Black);
    assert_eq!(white.hand().len(), START_HAND as usize + 1);
    let size = UNSHUFFLED.len() as u8;
    assert_eq!(white.deck_len(), size - START_HAND - 1);
    assert_eq!(black.hand().len(), START_HAND as usize);
    assert_eq!(black.deck_len(), size - START_HAND);
    game.play(find(game.position(), "e2", "e4")).unwrap();
    let hand = |game: &Game, color| game.position().cards(color).hand().len();
    // Карту берут в начале своего хода — после конца хода соперника.
    assert_eq!(hand(&game, Color::Black), START_HAND as usize);
    assert!(game.end_turn());
    assert_eq!(hand(&game, Color::Black), START_HAND as usize + 1);
    assert_eq!(hand(&game, Color::White), START_HAND as usize + 1);
}

#[test]
fn decks_follow_the_recipes_of_the_menu() {
    use decks::{MAX_DECK, RECIPES};
    let count = |deck: &[CardKind], kind| deck.iter().filter(|&&c| c == kind).count();
    let family = |deck: &[CardKind], f: Family| deck.iter().filter(|c| c.family() == f).count();
    assert_eq!(
        DECK_COUNT,
        crate::assets::DECK_SKINS,
        "колода на каждую рубашку меню"
    );
    for index in 0..DECK_COUNT {
        let deck = deck_cards(index);
        assert!(deck.len() <= MAX_DECK, "колода {index} не помещается");
        assert_eq!(
            deck.len(),
            RECIPES[index]
                .iter()
                .map(|&(_, n)| n as usize)
                .sum::<usize>()
        );
    }
    // Лазурная — 30 карт, остальные — классические по 34: 15 обычных, 9 жертвы, 10 победы.
    let azure = deck_cards(0);
    assert_eq!(azure.len(), 30);
    assert_eq!(count(&azure, CardKind::FireWave), 2);
    assert_eq!(
        count(&azure, CardKind::Normal) + count(&azure, CardKind::Sacrifice),
        0
    );
    assert_eq!(count(&azure, CardKind::NorthWarrior), 2);
    for index in 1..DECK_COUNT {
        let deck = deck_cards(index);
        assert_eq!(deck.len(), 34);
        assert_eq!(
            [Family::Normal, Family::Sacrifice, Family::Victory].map(|f| family(&deck, f)),
            [15, 9, 10]
        );
        assert_eq!(count(&deck, CardKind::Normal), 2);
        assert_eq!(count(&deck, CardKind::FireWave), 4);
        assert_eq!(count(&deck, CardKind::Sacrifice), 2);
        assert_eq!(count(&deck, CardKind::Funding), 2);
        assert_eq!(count(&deck, CardKind::Swap), 2);
        assert_eq!(count(&deck, CardKind::Heal), 2);
        assert_eq!(count(&deck, CardKind::Victory), 0);
    }
    // Неперемешанная колода позиций — тот же состав, что у классической.
    let mut classic = deck_cards(1);
    let mut plain = UNSHUFFLED.to_vec();
    classic.sort_by_key(|k| format!("{k:?}"));
    plain.sort_by_key(|k| format!("{k:?}"));
    assert_eq!(classic, plain);
    // В коллекции — все 26 видов из всех колод.
    assert_eq!(collection().len(), 26);

    // Перемешанная колода — та же по составу, сданная рука взята сверху колоды.
    for index in 0..2 {
        let pos = Position::start().with_shuffled_cards(12345, [index; 2]);
        let white = pos.cards(Color::White);
        let mut all: Vec<CardKind> = white.hand().to_vec();
        all.extend_from_slice(&white.deck[..white.deck_len() as usize]);
        let mut sorted = all.clone();
        let mut recipe = deck_cards(index);
        sorted.sort_by_key(|k| format!("{k:?}"));
        recipe.sort_by_key(|k| format!("{k:?}"));
        assert_eq!(sorted, recipe, "колода {index}");
    }
    let pos = Position::start().with_shuffled_cards(12345, [1, 1]);
    let other = Position::start().with_shuffled_cards(999, [1, 1]);
    assert_ne!(
        pos.cards(Color::White).deck,
        other.cards(Color::White).deck,
        "разные партии — разный порядок карт"
    );
    // Стороны могут играть разными колодами.
    let mixed = Position::start().with_shuffled_cards(5, [0, 1]);
    assert_eq!(
        mixed.cards(Color::White).deck_len() as usize + START_HAND as usize + 1,
        30
    );
    assert_eq!(
        mixed.cards(Color::Black).deck_len() as usize + START_HAND as usize,
        34
    );
}
#[test]
fn only_one_card_per_turn_and_it_burns() {
    // Колоды не перемешаны: у белых в руке обычная, жертвы, обычная и жертвы.
    let mut game = Game::from_position(Position::start());
    let before = *game.position().cards(Color::White);
    assert!(before.hand().iter().all(|k| !k.targeted()));
    let second = before.hand()[1];
    let play = game.play_card(1, &[]).unwrap();
    assert_eq!(
        (play.kind, play.index, play.color),
        (second, 1, Color::White)
    );
    let after = game.position().cards(Color::White);
    assert_eq!(after.hand().len(), before.hand().len() - 1);
    // Карта ушла из середины руки, остальные сдвинулись.
    assert_eq!(after.hand()[0], before.hand()[0]);
    assert_eq!(after.hand()[1..], before.hand()[2..]);
    // Ещё две карты можно, четвёртую — нет.
    assert!(game.play_card(0, &[]).is_some());
    assert!(game.play_card(0, &[]).is_some());
    assert_eq!(game.position().cards_played(), CARDS_PER_TURN);
    assert!(!game.position().can_play_card());
    assert_eq!(game.play_card(0, &[]), None, "четвёртая карта за ход");
    // Карты не тратят ход фигурой; сыграв три карты и фигуру, белые заканчивают ход сами.
    game.play(find(game.position(), "e2", "e4")).unwrap();
    assert_eq!(game.position().side(), Color::Black);
    // После хода фигурой карты тоже можно: ход ждёт ещё карт или кнопку.
    game.play(find(game.position(), "e7", "e5")).unwrap();
    assert!(game.play_card(0, &[]).is_some());
    assert_eq!(game.position().side(), Color::Black);
    assert!(game.end_turn());
    // Новый ход белых: счёт карт заново, в руке прибавилась огненная волна.
    assert_eq!(game.position().cards_played(), 0);
    let hand = game.position().cards(Color::White).hand();
    assert_eq!(hand.len(), before.hand().len() - 3 + 1);
    assert_eq!(hand.last(), Some(&CardKind::FireWave));
    assert_eq!(game.play_card(99, &[]), None);
    assert!(game.play_card(0, &[]).is_some());
}

#[test]
fn third_card_after_a_move_ends_the_turn() {
    let mut game = Game::from_position(Position::start());
    game.play(find(game.position(), "e2", "e4")).unwrap();
    for _ in 0..2 {
        game.play_card(0, &[]).unwrap();
        assert_eq!(game.position().side(), Color::White, "можно ещё карту");
    }
    game.play_card(0, &[]).unwrap();
    assert_eq!(
        game.position().side(),
        Color::Black,
        "три карты сыграны — ход кончился"
    );
}

#[test]
fn turn_ends_with_the_button_after_a_move() {
    let mut game = Game::from_position(Position::start());
    assert!(!game.can_end_turn(), "без хода фигурой ход не закончить");
    assert!(!game.end_turn());
    game.play(find(game.position(), "e2", "e4")).unwrap();
    // Можно ещё сыграть карту: ход ждёт кнопку, фигурой больше не ходят.
    assert_eq!(game.position().side(), Color::White);
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.status(), Status::Ongoing);
    assert!(game.end_turn());
    assert_eq!(game.position().side(), Color::Black);
    assert!(!game.end_turn());
    // Конец хода — отдельная запись; ударов не было — без текста в списке ходов.
    assert!(matches!(
        &game.history()[1].event,
        Event::EndTurn(Color::White, turn) if turn.is_quiet()
    ));
    assert!(game.history()[1].notation().text.is_empty());
}

#[test]
fn turn_ends_by_itself_when_nothing_is_left() {
    // Карт нет — после хода фигурой делать нечего.
    let mut pos = position("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1");
    pos.cards[Color::White as usize].hand_len = 0;
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "e2", "e4")).unwrap();
    assert_eq!(game.position().side(), Color::Black);

    // Одна огненная волна в руке: её можно сыграть на свою фигуру — ход ждёт.
    let mut pos = position("4k3/8/8/8/8/8/8/4K2R w - - 0 1");
    pos.cards[Color::White as usize].hand_len = 0;
    pos.give_card(Color::White, CardKind::FireWave);
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "e1", "d1")).unwrap();
    assert_eq!(game.position().side(), Color::White);
    assert_eq!(game.position().card_targets(0), [sq("d1"), sq("h1")]);
}

/// Позиция с огненной волной в руке белых; номер волны в руке.
fn with_wave(fen: &str) -> (Position, usize) {
    let mut pos = position(fen);
    pos.give_card(Color::White, CardKind::FireWave);
    let index = pos.cards(Color::White).hand().len() - 1;
    (pos, index)
}

#[test]
fn fire_wave_burns_every_neighbour() {
    // Конь d4 окружён: три чёрные пешки сверху, своя пешка c3 и чёрный конь e3.
    let (pos, wave) = with_wave("4k3/8/8/2ppp3/3N4/2P1n3/8/4K3 w - - 0 1");
    let mut game = Game::from_position(pos);
    assert!(game.position().card_targets(wave).contains(&sq("d4")));
    let play = game.play_card(wave, &[sq("d4")]).unwrap();
    let mut hit: Vec<String> = play.hits.iter().map(|h| square_name(h.square)).collect();
    hit.sort();
    assert_eq!(hit, ["c3", "c5", "d5", "e3", "e5"]);
    assert!(
        play.hits
            .iter()
            .all(|h| h.damage == WAVE_DAMAGE && !h.killed)
    );
    let pos = game.position();
    assert_eq!(hp(pos, "d5"), 900 - 300);
    assert_eq!(hp(pos, "c3"), 900 - 300, "своих волна тоже задевает");
    assert_eq!(hp(pos, "e3"), 900 - 300);
    assert_eq!(hp(pos, "d4"), 900, "выбранную фигуру — нет");
    // Карта сыграна (можно ещё две), фигурой ещё можно сходить.
    assert_eq!(game.position().cards_played(), 1);
    assert!(game.position().can_play_card());
    assert!(!game.legal_moves().is_empty());
    assert_eq!(game.position().side(), Color::White);
}

#[test]
fn fire_wave_kills_and_counts_for_the_player() {
    let (mut pos, wave) = with_wave("4k3/8/8/3p4/3N4/8/8/4K3 w - - 0 1");
    pos.set_hp(sq("d5"), 300);
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "e1", "f1")).unwrap();
    let play = game.play_card(wave, &[sq("d4")]).unwrap();
    assert!(play.hits[0].killed);
    assert!(game.position().piece(sq("d5")).is_none());
    // В руке ещё карты — ход ждёт кнопку.
    assert_eq!(game.position().side(), Color::White);
    assert!(game.end_turn());
    assert_eq!(game.kills_by(Color::White)[0].kind, Kind::Pawn);
    let card = &game.history()[1];
    assert_eq!(card.notation().text, "Огненная Волна d4");
    assert!(card.description().contains("пешка d5 — погиб"));
}

#[test]
fn fire_wave_targets_only_own_pieces_and_burns_kings_too() {
    let (pos, wave) = with_wave("8/8/8/8/8/8/2nk4/1N1K4 w - - 0 1");
    let mut game = Game::from_position(pos);
    assert_eq!(game.play_card(wave, &[]), None, "волне нужна фигура");
    assert_eq!(game.play_card(wave, &[sq("c2")]), None, "чужая фигура");
    assert_eq!(game.play_card(wave, &[sq("e4")]), None, "пустая клетка");
    let play = game.play_card(wave, &[sq("b1")]).unwrap();
    // Конь c2 — сосед b1, король d2 — нет.
    assert_eq!(play.hits.len(), 1);
    assert_eq!(play.hits[0].square, sq("c2"));

    // Раненый чёрный король c3 рядом с ладьёй b2: волна вокруг ладьи его убивает, у
    // чёрных остаётся пешка — партия идёт.
    let (mut pos, wave) = with_wave("8/7p/8/8/8/2k5/1R6/7K w - - 0 1");
    pos.set_hp(sq("c3"), 200);
    let mut game = Game::from_position(pos);
    let play = game.play_card(wave, &[sq("b2")]).unwrap();
    let king = play.hits.iter().find(|h| h.square == sq("c3")).unwrap();
    assert_eq!((king.damage, king.killed), (WAVE_DAMAGE, true));
    let pos = game.position();
    assert_eq!(pos.king_square(Color::Black), None);
    assert_eq!(pos.points(Color::White, Points::Victory), 9);
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::White
        }
    );
}

#[test]
fn undo_takes_back_moves_cards_and_turn_ends() {
    let (pos, wave) = with_wave("4k3/8/8/2ppp3/3N4/8/8/4K3 w - - 0 1");
    let mut game = Game::from_position(pos.clone());
    game.play_card(wave, &[sq("d4")]).unwrap();
    game.play(find(game.position(), "e1", "f1")).unwrap();
    assert!(game.end_turn());
    assert_eq!(game.position().side(), Color::Black);
    assert_eq!(game.history().len(), 3);
    while game.undo() {}
    assert_eq!(game.position(), &pos);
}

#[test]
fn full_hand_and_empty_deck_stop_drawing() {
    let mut pos = Position::start();
    let white = &mut pos.cards[Color::White as usize];
    while white.hand().len() < MAX_HAND as usize {
        white.draw();
    }
    let full = *white;
    let black = &mut pos.cards[Color::Black as usize];
    black.deck_len = 0;
    let empty = *black;
    pos.make_turn(find(&pos, "e2", "e4"));
    assert_eq!(*pos.cards(Color::Black), empty);
    pos.make_turn(find(&pos, "e7", "e5"));
    assert_eq!(*pos.cards(Color::White), full);
}

#[test]
fn a_card_before_the_move_can_set_up_a_kill() {
    // Волна вокруг слона e5 оставит раненому королю f6 300 HP — и слон в конце хода его
    // добьёт: у чёрных больше нет фигур.
    let (mut pos, wave) = with_wave("8/8/5k2/4B3/8/8/8/4K3 w - - 0 1");
    pos.set_hp(sq("f6"), 600);
    assert!(pos.card_targets(wave).contains(&sq("e5")));
    let mut game = Game::from_position(pos);
    game.play_card(wave, &[sq("e5")]).unwrap();
    assert!(game.position().in_check(Color::Black));
    game.play(find(game.position(), "e1", "d1")).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::White
        }
    );

    // Обмен коня b1 с пешкой d6 ставит коня рядом с королём e8 (300 HP) — тоже можно.
    let (mut pos, _) = with_swap("4k3/8/3P4/8/8/8/8/1N2K3 w - - 0 1");
    pos.set_hp(sq("e8"), 300);
    assert!(pos.swap_partners(sq("b1")).contains(&sq("d6")));
}
#[test]
fn a_card_may_end_the_game() {
    // Волна вокруг коня d4 добивает последнюю фигуру чёрных — раненого короля d5.
    let (mut pos, wave) = with_wave("8/8/8/3k4/3N4/8/8/4K3 w - - 0 1");
    pos.set_hp(sq("d5"), 300);
    let mut game = Game::from_position(pos);
    game.play_card(wave, &[sq("d4")]).unwrap();
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::White
        }
    );
    let record = game.history().last().unwrap();
    assert_eq!(record.notation().suffix, "#");
    assert!(record.description().contains("У чёрных не осталось фигур"));
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.play_card(0, &[]), None);
}

#[test]
fn there_is_no_en_passant() {
    // Пешка e2 прошла на e4 мимо чёрной d4: брать на проходе нельзя — пешки ходом не бьют.
    let mut pos = position("4k3/8/8/8/3p4/8/4P3/4K3 w - - 0 1");
    pos.make_turn(find(&pos, "e2", "e4"));
    assert!(!has_move(&pos, "d4", "e3"));
    assert!(
        position("4k3/8/8/8/3pP3/8/8/4K3 b - e3 0 1")
            .legal_moves()
            .iter()
            .all(|m| m.to != sq("e3"))
    );
}
#[test]
fn funding_draws_two_cards() {
    let mut pos = position("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1");
    pos.give_card(Color::White, CardKind::Funding);
    pos.give_points(Color::White, Points::Sacrifice, 1);
    let before = *pos.cards(Color::White);
    let funding = before.hand().len() - 1;
    let mut game = Game::from_position(pos);
    let play = game.play_card(funding, &[]).unwrap();
    assert_eq!((play.kind, play.drawn), (CardKind::Funding, FUNDING_DRAW));
    let after = game.position().cards(Color::White);
    // Карта сгорела, две пришли сверху колоды.
    assert_eq!(after.hand().len(), before.hand().len() - 1 + 2);
    assert_eq!(after.deck_len(), before.deck_len() - 2);
    // Это одна из трёх карт хода, фигурой ходить ещё можно.
    assert_eq!(game.position().cards_played(), 1);
    assert!(!game.legal_moves().is_empty());
    let record = &game.history()[0];
    assert_eq!(record.notation().text, "Дополнительное финансирование");
    assert!(record.description().contains("взяли 2 карты"));
}

#[test]
fn funding_draws_only_what_fits() {
    // Рука почти полна: место только для одной карты.
    let mut pos = position("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1");
    while pos.cards(Color::White).hand().len() < MAX_HAND as usize - 1 {
        pos.give_card(Color::White, CardKind::Normal);
    }
    pos.give_card(Color::White, CardKind::Funding);
    pos.give_points(Color::White, Points::Sacrifice, 1);
    let funding = MAX_HAND as usize - 1;
    let mut full = pos.clone();
    let play = full.play_card(funding, &[]).unwrap();
    assert_eq!(play.drawn, 1);
    assert_eq!(full.cards(Color::White).hand().len(), MAX_HAND as usize);
    // Колода пуста — карт не взять, но сыграть финансирование можно.
    pos.cards[Color::White as usize].deck_len = 0;
    let mut game = Game::from_position(pos);
    let play = game.play_card(funding, &[]).unwrap();
    assert_eq!(play.drawn, 0);
    assert!(game.history()[0].description().contains("карт не взяли"));
}

/// Позиция с обменом в руке белых и очком победы на него; номер обмена в руке.
fn with_swap(fen: &str) -> (Position, usize) {
    let mut pos = position(fen);
    pos.give_card(Color::White, CardKind::Swap);
    pos.give_points(Color::White, Points::Victory, 1);
    let index = pos.cards(Color::White).hand().len() - 1;
    (pos, index)
}

#[test]
fn swap_exchanges_two_own_pieces_with_their_health() {
    let (mut pos, swap) = with_swap("4k3/8/8/8/8/8/8/RN2K2R w KQ - 0 1");
    pos.set_hp(sq("a1"), 1000);
    let mut game = Game::from_position(pos.clone());
    assert_eq!(game.position().swap_partners(sq("a1")).len(), 3);
    let play = game.play_card(swap, &[sq("a1"), sq("b1")]).unwrap();
    assert!(play.hits.is_empty());
    let after = game.position();
    assert_eq!(after.piece(sq("a1")).map(|p| p.kind), Some(Kind::Knight));
    let rook = after.piece(sq("b1")).unwrap();
    assert_eq!(
        (rook.kind, rook.hp),
        (Kind::Rook, 1000),
        "здоровье переходит с фигурой"
    );
    // Ладья ушла из угла — длинной рокировки больше нет, короткая осталась.
    assert_eq!(after.castling & WHITE_LONG, 0);
    assert_ne!(after.castling & WHITE_SHORT, 0);
    // Карта не тратит ход фигурой.
    assert_eq!(after.side(), Color::White);
    assert!(!game.legal_moves().is_empty());
    let record = &game.history()[0];
    assert_eq!(record.notation().text, "Обмен a1↔b1");
    assert!(
        record
            .description()
            .contains("ладья a1 и конь b1 поменялись местами")
    );
    assert!(game.undo());
    assert_eq!(game.position(), &pos);
}

#[test]
fn swapped_king_keeps_its_square_and_loses_castling() {
    let (pos, swap) = with_swap("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    let mut game = Game::from_position(pos);
    game.play_card(swap, &[sq("e1"), sq("h1")]).unwrap();
    let after = game.position();
    assert_eq!(after.king_square(Color::White), Some(sq("h1")));
    assert_eq!(after.piece(sq("e1")).map(|p| p.kind), Some(Kind::Rook));
    assert_eq!(after.castling & (WHITE_SHORT | WHITE_LONG), 0);
    assert!(
        !game
            .legal_moves()
            .iter()
            .any(|m| matches!(m.kind, MoveKind::CastleShort | MoveKind::CastleLong))
    );
}

#[test]
fn swap_keeps_pawns_off_the_last_rank() {
    let (pos, swap) = with_swap("R3k3/4P3/8/8/8/8/4P3/4K3 w - - 0 1");
    // Пешка e7 на a8 встала бы на последнюю горизонталь — нельзя; пешка e2 на e1 — можно.
    assert!(!pos.swap_partners(sq("e7")).contains(&sq("a8")));
    assert!(pos.swap_partners(sq("e2")).contains(&sq("e1")));
    let mut game = Game::from_position(pos);
    assert_eq!(game.play_card(swap, &[sq("e7"), sq("a8")]), None);
    assert_eq!(
        game.play_card(swap, &[sq("e2"), sq("e2")]),
        None,
        "фигура сама с собой"
    );
    assert_eq!(
        game.play_card(swap, &[sq("e2")]),
        None,
        "обмену нужны две фигуры"
    );
    assert!(game.play_card(swap, &[sq("e2"), sq("e1")]).is_some());
}

#[test]
fn swap_takes_only_own_pieces() {
    let (pos, swap) = with_swap("4k3/8/8/8/8/8/3p4/4K2N w - - 0 1");
    assert_eq!(pos.card_targets(swap), [sq("e1"), sq("h1")]);
    let mut game = Game::from_position(pos);
    assert_eq!(
        game.play_card(swap, &[sq("e1"), sq("d2")]),
        None,
        "чужая фигура"
    );
    assert_eq!(
        game.play_card(swap, &[sq("e1"), sq("e4")]),
        None,
        "пустая клетка"
    );
}

#[test]
fn swap_may_expose_own_king() {
    // Обмен ставит раненого короля e1 на b1 — рядом с ладьёй a1, которая его убьёт. Так
    // можно и после хода фигурой: шах только предупреждает.
    let (mut pos, swap) = with_swap("k7/8/8/8/8/8/8/rN2K2R w - - 0 1");
    pos.set_hp(sq("e1"), 300);
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "h1", "h2")).unwrap();
    assert!(!game.position().in_check(Color::White));
    assert!(game.position().swap_partners(sq("e1")).contains(&sq("b1")));
    game.play_card(swap, &[sq("e1"), sq("b1")]).unwrap();
    assert!(game.position().in_check(Color::White));
}
#[test]
fn kills_give_victory_and_sacrifice_points() {
    // Цена фигур в очках.
    let points = Kind::ALL.map(Kind::points);
    assert_eq!(
        points,
        [9, 7, 5, 3, 3, 1],
        "король, ферзь, ладья, слон, конь, пешка"
    );
    // Ладья a4 добивает коня a5: белым — 3 очка победы, чёрным — 3 очка жертвы.
    let mut pos = position("4k3/8/8/n7/R7/8/8/4K3 w - - 0 1");
    pos.set_hp(sq("a5"), 150);
    assert_eq!(strikes(&mut pos), ["a4-a5 300†"]);
    assert_eq!(pos.points(Color::White, Points::Victory), 3);
    assert_eq!(pos.points(Color::Black, Points::Sacrifice), 3);
    assert_eq!(pos.points(Color::White, Points::Sacrifice), 0);
    assert_eq!(pos.points(Color::Black, Points::Victory), 0);

    // Чёрная ладья a5 убивает белую a4: 5 очков победы — чёрным, 5 жертвы — белым.
    let mut pos = position("4k3/8/8/r7/R7/8/8/4K3 b - - 0 1");
    pos.set_hp(sq("a4"), 50);
    assert_eq!(strikes(&mut pos), ["a5-a4 300†"]);
    assert_eq!(pos.points(Color::Black, Points::Victory), 5);
    assert_eq!(pos.points(Color::White, Points::Sacrifice), 5);
    assert_eq!(pos.points(Color::White, Points::Victory), 0);
}
#[test]
fn wave_kills_score_for_both_sides() {
    // Волна вокруг коня d4 добивает чёрную пешку d5 и свою пешку c3.
    let (mut pos, wave) = with_wave("4k3/8/8/3p4/3N4/2P5/8/4K3 w - - 0 1");
    pos.set_hp(sq("d5"), 300);
    pos.set_hp(sq("c3"), 300);
    let play = pos.play_card(wave, &[sq("d4")]).unwrap();
    assert_eq!(play.hits.iter().filter(|h| h.killed).count(), 2);
    // Чужая пешка — очко победы белым и жертвы чёрным; своя — только очко жертвы белым.
    assert_eq!(pos.points(Color::White, Points::Victory), 1);
    assert_eq!(pos.points(Color::Black, Points::Sacrifice), 1);
    assert_eq!(pos.points(Color::White, Points::Sacrifice), 1);
    assert_eq!(pos.points(Color::Black, Points::Victory), 0);
}

#[test]
fn victory_and_sacrifice_cards_cost_points() {
    // Обмен стоит очко победы, финансирование — очко жертвы; простые карты бесплатны.
    assert_eq!(CardKind::Swap.cost(), Some((Points::Victory, 1)));
    assert_eq!(CardKind::Funding.cost(), Some((Points::Sacrifice, 1)));
    for kind in [CardKind::Normal, CardKind::Sacrifice, CardKind::Victory] {
        assert_eq!(kind.cost(), None);
    }
    let mut pos = position("4k3/8/8/8/8/8/4P3/R3K3 w - - 0 1");
    pos.give_card(Color::White, CardKind::Swap);
    pos.give_card(Color::White, CardKind::Funding);
    let hand = pos.cards(Color::White).hand().len();
    let (swap, funding) = (hand - 2, hand - 1);
    // Очков нет — сыграть нельзя.
    assert!(!pos.card_playable(swap) && !pos.card_playable(funding));
    assert!(pos.card_targets(swap).is_empty());
    assert_eq!(pos.clone().play_card(funding, &[]), None);
    assert_eq!(pos.clone().play_card(swap, &[sq("a1"), sq("e1")]), None);
    // С очками — можно, и очки тратятся.
    pos.give_points(Color::White, Points::Victory, 2);
    pos.give_points(Color::White, Points::Sacrifice, 1);
    let mut game = Game::from_position(pos);
    game.play_card(funding, &[]).unwrap();
    assert_eq!(game.position().points(Color::White, Points::Sacrifice), 0);
    game.play_card(swap, &[sq("a1"), sq("e1")]).unwrap();
    assert_eq!(game.position().points(Color::White, Points::Victory), 1);
    let record = game.history().last().unwrap();
    assert!(record.description().contains("«Обмен» за 1 очко победы"));
    // Отмена хода возвращает и очки.
    while game.undo() {}
    assert_eq!(game.position().points(Color::White, Points::Victory), 2);
}

/// Позиция с исцелением в руке белых и двумя очками победы на него; номер карты в руке.
fn with_heal(fen: &str) -> (Position, usize) {
    let mut pos = position(fen);
    pos.give_card(Color::White, CardKind::Heal);
    pos.give_points(Color::White, Points::Victory, 2);
    let index = pos.cards(Color::White).hand().len() - 1;
    (pos, index)
}

#[test]
fn heal_restores_own_neighbours_of_a_bishop() {
    // Слон d4: рядом раненые свои пешка c3 и конь e5, чужая пешка d5 и свой ферзь d3 —
    // целый; слон тоже ранен.
    let (mut pos, heal) = with_heal("4k3/8/8/3pN3/3B4/2PQ4/8/4K3 w - - 0 1");
    for (name, hp) in [("c3", 100), ("e5", 200), ("d5", 100), ("d4", 100)] {
        pos.set_hp(sq(name), hp);
    }
    assert_eq!(pos.card_targets(heal), [sq("d4")], "только свой слон");
    let mut game = Game::from_position(pos);
    let play = game.play_card(heal, &[sq("d4")]).unwrap();
    let after = game.position();
    // Пешке и коню — все 600, чужим и ферзю — ничего.
    assert_eq!(hp(after, "c3"), 700);
    assert_eq!(hp(after, "e5"), 800);
    assert_eq!(hp(after, "d5"), 100, "чужих не лечит");
    assert_eq!(hp(after, "d3"), 1200);
    assert_eq!(hp(after, "d4"), 100, "сам слон не лечится");
    let mut healed: Vec<(String, i32)> = play
        .heals
        .iter()
        .map(|h| (square_name(h.square), h.amount))
        .collect();
    healed.sort();
    assert_eq!(healed, [("c3".to_owned(), 600), ("e5".to_owned(), 600)]);
    // Стоит 2 очка победы.
    assert_eq!(after.points(Color::White, Points::Victory), 0);
    let record = &game.history()[0];
    assert_eq!(record.notation().text, "Исцеление d4");
    assert!(record.description().contains("пешка c3 — +2 HP"));
}

#[test]
fn heal_needs_an_own_bishop_and_two_points() {
    let (pos, heal) = with_heal("4k3/8/8/8/3b4/3N4/8/4K3 w - - 0 1");
    // Чужой слон и не слон — не цели.
    assert!(pos.card_targets(heal).is_empty());
    assert!(!pos.card_playable(heal));
    let mut game = Game::from_position(pos);
    assert_eq!(game.play_card(heal, &[sq("d4")]), None, "чужой слон");
    assert_eq!(game.play_card(heal, &[sq("d3")]), None, "не слон");
    // Одного очка победы мало.
    let mut pos = position("4k3/8/8/8/3B4/8/8/4K3 w - - 0 1");
    pos.give_card(Color::White, CardKind::Heal);
    pos.give_points(Color::White, Points::Victory, 1);
    let heal = pos.cards(Color::White).hand().len() - 1;
    assert!(!pos.card_playable(heal));
    assert_eq!(pos.play_card(heal, &[sq("d4")]), None);
}

#[test]
fn mana_comes_each_turn_and_pays_for_the_fire_wave() {
    // Перемешанные колоды сдают заново — мана в начале всё равно 2.
    assert_eq!(
        Game::new([1, 1])
            .position()
            .points(Color::White, Points::Mana),
        2
    );
    let mut game = Game::from_position(Position::start());
    let mana = |game: &Game, color| game.position().points(color, Points::Mana);
    assert_eq!(
        (mana(&game, Color::White), mana(&game, Color::Black)),
        (2, 0)
    );
    game.play(find(game.position(), "e2", "e4")).unwrap();
    game.end_turn();
    game.play(find(game.position(), "e7", "e5")).unwrap();
    game.end_turn();
    // Мана копится: у белых второй ход — 4, у чёрных — 2.
    assert_eq!(
        (mana(&game, Color::White), mana(&game, Color::Black)),
        (4, 2)
    );
    assert_eq!(CardKind::FireWave.cost(), Some((Points::Mana, 2)));
    let mut pos = game.position().clone();
    pos.give_card(Color::White, CardKind::FireWave);
    let wave = pos.cards(Color::White).hand().len() - 1;
    pos.play_card(wave, &[sq("e4")]).unwrap();
    assert_eq!(pos.points(Color::White, Points::Mana), 2);
}

/// Позиция с картой `kind` в руке белых и 10 маной на неё; номер карты в руке.
fn with_freeze(fen: &str, kind: CardKind) -> (Position, usize) {
    let mut pos = position(fen);
    pos.give_card(Color::White, kind);
    pos.give_points(Color::White, Points::Mana, 8);
    let index = pos.cards(Color::White).hand().len() - 1;
    (pos, index)
}

fn frozen(pos: &Position, name: &str) -> u8 {
    pos.piece(sq(name)).expect("фигура").frozen
}

#[test]
fn areas_cover_three_lines_or_a_square_and_stay_on_the_board() {
    let names =
        |squares: Vec<Square>| -> Vec<String> { squares.into_iter().map(square_name).collect() };
    let ranks = Area::Ranks.squares(sq("e5"));
    assert_eq!(ranks.len(), 24);
    assert!(ranks.iter().all(|&s| (3..=5).contains(&rank_of(s))));
    // У края полоса сдвигается внутрь: целиком на доске.
    assert!(
        Area::Files
            .squares(sq("a1"))
            .iter()
            .all(|&s| file_of(s) <= 2)
    );
    assert_eq!(
        names(Area::Square.squares(sq("h8"))),
        ["f6", "g6", "h6", "f7", "g7", "h7", "f8", "g8", "h8"]
    );
    assert_eq!(CardKind::Storm.cost(), Some((Points::Mana, 10)));
    assert_eq!(CardKind::IceBarrier.area(), Some(Area::Square));
}

#[test]
fn storm_freezes_three_ranks_except_kings_for_one_turn() {
    // Буря на e7: горизонтали 6–8. Чёрные пешки замерзают на ход чёрных, король — нет;
    // свой конь c6 тоже в полосе — он пропустит следующий ход белых.
    let (pos, storm) = with_freeze(
        "4k3/pppppppp/2N5/8/8/8/PPPPPPPP/4K3 w - - 0 1",
        CardKind::Storm,
    );
    assert_eq!(pos.card_targets(storm).len(), 64, "любая клетка");
    let mut game = Game::from_position(pos);
    let play = game.play_card(storm, &[sq("e7")]).unwrap();
    assert_eq!(play.frozen.len(), 9);
    let pos = game.position();
    assert_eq!(pos.points(Color::White, Points::Mana), 0);
    assert_eq!(frozen(pos, "a7"), 2);
    assert_eq!(frozen(pos, "c6"), 2);
    assert_eq!(frozen(pos, "e8"), 0, "король не замерзает");
    assert!(!has_move(pos, "c6", "b8"), "замёрзший конь не ходит");
    assert!(
        game.history()[0]
            .description()
            .contains("горизонтали 6–8 замёрзли")
    );
    game.play(find(game.position(), "e2", "e4")).unwrap();
    assert!(game.end_turn());
    // Ход чёрных: пешки стоят, ходит только король; замёрзшие пешки не бьют коня c6.
    let pos = game.position();
    assert!(pos.legal_moves().iter().all(|m| m.from == sq("e8")));
    assert_eq!(pos.max_damage(sq("c6"), Color::Black), 0);
    // Свой конь всё ещё во льду (оттает в начале своего хода).
    assert_eq!(frozen(pos, "c6"), 1);
    game.play(find(game.position(), "e8", "d8")).unwrap();
    assert!(game.end_turn());
    // Пешки чёрных оттаяли в конце своего хода, конь белых — в начале хода белых.
    let pos = game.position();
    assert_eq!(frozen(pos, "a7"), 0);
    assert_eq!(frozen(pos, "c6"), 0);
    assert!(has_move(pos, "c6", "b8"));
}

#[test]
fn avalanche_freezes_three_files() {
    let (pos, avalanche) = with_freeze("r3k3/p7/8/8/8/8/1P6/4K3 w - - 0 1", CardKind::Avalanche);
    let play = pos.clone().play_card(avalanche, &[sq("a1")]).unwrap();
    // Вертикали a–c: ладья a8, пешки a7 и b2.
    let mut squares: Vec<String> = play.frozen.iter().map(|&s| square_name(s)).collect();
    squares.sort();
    assert_eq!(squares, ["a7", "a8", "b2"]);
}

#[test]
fn ice_barrier_freezes_pieces_that_step_into_it() {
    // Барьер на d5: зона c4–e6. Конь чёрных встаёт в неё — и пропускает следующий ход.
    let (pos, barrier) = with_freeze("1n2k3/8/8/8/8/8/8/4K3 w - - 0 1", CardKind::IceBarrier);
    let mut game = Game::from_position(pos);
    let play = game.play_card(barrier, &[sq("d5")]).unwrap();
    assert!(play.frozen.is_empty());
    assert!(
        game.history()[0]
            .description()
            .contains("ледяная зона c4–e6")
    );
    assert_eq!(game.position().ice_zones().count_ones(), 9);
    game.play(find(game.position(), "e1", "e2")).unwrap();
    assert!(game.end_turn());
    let outcome = game.play(find(game.position(), "b8", "c6")).unwrap();
    assert!(outcome.frozen);
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("вмёрз")
    );
    assert!(game.end_turn());
    // Ход белых: зона растаяла, конь ещё во льду — он вмёрз в свой ход и оттает в начале
    // следующего хода чёрных.
    assert_eq!(game.position().ice_zones(), 0);
    assert_eq!(frozen(game.position(), "c6"), 1);
    game.play(find(game.position(), "e2", "e3")).unwrap();
    assert!(game.end_turn());
    assert_eq!(frozen(game.position(), "c6"), 0);
    assert!(has_move(game.position(), "c6", "b8"));
}

#[test]
fn a_side_with_nothing_to_move_passes() {
    // Буря на a1 замораживает пешки a2, b2 и коня b1 — королю a1 некуда ступить.
    let (pos, storm) = with_freeze("4k3/8/8/8/8/8/PP6/KN6 w - - 0 1", CardKind::Storm);
    let mut game = Game::from_position(pos);
    assert!(!game.can_end_turn(), "сначала надо сходить");
    game.play_card(storm, &[sq("a1")]).unwrap();
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.status(), Status::Ongoing, "пата нет");
    assert!(game.can_end_turn());
    assert!(game.end_turn());
    assert_eq!(game.position().side(), Color::Black);
}

/// Позиция с картой `kind` в руке белых и `n` очками нужного ей типа; номер карты в руке.
fn with_card(fen: &str, kind: CardKind, n: u8) -> (Position, usize) {
    let mut pos = position(fen);
    pos.give_card(Color::White, kind);
    if let Some((points, _)) = kind.cost() {
        pos.give_points(Color::White, points, n);
    }
    let index = pos.cards(Color::White).hand().len() - 1;
    (pos, index)
}

#[test]
fn offering_removes_own_piece_for_double_points() {
    let (pos, card) = with_card("4k3/8/8/8/8/8/8/R3K3 w - - 0 1", CardKind::Offering, 0);
    assert_eq!(CardKind::Offering.cost(), None, "бесплатна");
    assert_eq!(pos.card_targets(card), [sq("a1"), sq("e1")], "только свои");
    let mut game = Game::from_position(pos);
    game.play_card(card, &[sq("a1")]).unwrap();
    let pos = game.position();
    assert!(pos.piece(sq("a1")).is_none());
    assert_eq!(
        pos.points(Color::White, Points::Sacrifice),
        10,
        "ладья 5 × 2"
    );
    assert_eq!(pos.points(Color::Black, Points::Victory), 0);
}

#[test]
fn encircled_and_broken_wins_with_pieces_on_four_edges() {
    // Ладья a1 — нижний и левый края, ладья h8 — верхний и правый.
    let (pos, card) = with_card("4k2R/8/8/8/8/8/8/R3K3 w - - 0 1", CardKind::Encircle, 10);
    assert!(pos.card_playable(card));
    let mut game = Game::from_position(pos);
    game.play_card(card, &[]).unwrap();
    assert_eq!(
        game.status(),
        Status::Encircled {
            winner: Color::White
        }
    );
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("окружены и сломлены")
    );
    // Без правого края — нельзя.
    let (pos, card) = with_card("4k3/8/8/8/8/8/8/R3K3 w - - 0 1", CardKind::Encircle, 10);
    assert!(!pos.card_playable(card));
}

#[test]
fn they_multiply_splits_a_piece_and_returns_to_hand() {
    // Конь d4 делится: вторая половина — на соседнюю клетку, ближайшую к королю d1.
    let (mut pos, card) = with_card("4k3/8/8/8/3N4/8/8/3KQ3 w - - 0 1", CardKind::Multiply, 0);
    assert_eq!(CardKind::Multiply.cost(), Some((Points::Mana, 2)));
    assert_eq!(pos.card_targets(card), [sq("d4")], "не король и не ферзь");
    let mut game = Game::from_position(pos.clone());
    let play = game.play_card(card, &[sq("d4")]).unwrap();
    assert_eq!(play.spawned, Some(sq("d3")));
    let after = game.position();
    for name in ["d4", "d3"] {
        let knight = after.piece(sq(name)).unwrap();
        assert_eq!(
            (knight.kind, knight.hp, knight.attack()),
            (Kind::Knight, 450, 300)
        );
    }
    // После конца хода карта возвращается в руку.
    let hand = after.cards(Color::White).hand().len();
    game.play(find(game.position(), "e1", "e2")).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    let white = game.position().cards(Color::White).hand();
    assert_eq!(white.len(), hand + 1);
    assert_eq!(white.last(), Some(&CardKind::Multiply));
    // С одним здоровьем — обе по одному.
    pos.set_hp(sq("d4"), 1);
    pos.play_card(card, &[sq("d4")]).unwrap();
    assert_eq!((hp(&pos, "d4"), hp(&pos, "d3")), (1, 1));
}

#[test]
fn lone_warrior_grows_with_every_kill() {
    let (pos, card) = with_card("4k3/8/8/2p5/8/B7/8/4K3 w - - 0 1", CardKind::Champion, 10);
    assert_eq!(pos.card_targets(card), [sq("a3")], "не король");
    let mut game = Game::from_position(pos);
    game.play_card(card, &[sq("a3")]).unwrap();
    let bishop = game.position().piece(sq("a3")).unwrap();
    assert_eq!(
        (bishop.attack(), bishop.max_hp(), bishop.hp),
        (375, 1125, 1125)
    );
    // Слон a3 достаёт пешку c5 через b4 и в конце хода убивает её.
    let mut pos = game.position().clone();
    pos.set_hp(sq("a3"), 100);
    pos.set_hp(sq("c5"), 300);
    pos.make_move(find(&pos, "e1", "f1"));
    let strikes = pos.end_turn().strikes;
    assert!(strikes[0].hit.killed);
    assert_eq!(strikes[0].feast, 1025);
    let bishop = pos.piece(sq("a3")).unwrap();
    assert_eq!(
        (bishop.hp, bishop.attack()),
        (1125, 450),
        "всё здоровье и ещё +25%"
    );
}

#[test]
fn defensive_position_halves_damage_around_the_rook() {
    let (pos, card) = with_card("4k3/8/8/8/1n6/1R6/P7/4K3 w - - 0 1", CardKind::Bastion, 3);
    assert_eq!(pos.card_targets(card), [sq("b3")], "только ладья");
    let mut game = Game::from_position(pos);
    game.play_card(card, &[sq("b3")]).unwrap();
    let pos = game.position();
    assert_eq!(
        pos.damage_at(sq("a2"), 500),
        250,
        "рядом с ладьёй — вдвое меньше"
    );
    assert_eq!(pos.damage_at(sq("b3"), 500), 500, "сама ладья — нет");
    assert_eq!(pos.damage_at(sq("e1"), 500), 500);
    game.play(find(game.position(), "e1", "f1")).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    // Ладья бьёт коня b4: он чужой — ладья его не прикрывает.
    assert_eq!(hp(game.position(), "b4"), 900 - 300);
    game.play(find(game.position(), "e8", "d8")).unwrap();
    if game.position().side() == Color::Black {
        assert!(game.end_turn());
    }
    // Конь бьёт пешку a2: 300 → 150.
    assert_eq!(hp(game.position(), "a2"), 900 - 150);
}

#[test]
fn ice_wall_freezes_whoever_steps_on_the_rank_for_two_turns() {
    let (pos, card) = with_card("n3k3/8/8/8/8/8/8/4K3 w - - 0 1", CardKind::IceWall, 6);
    let mut game = Game::from_position(pos);
    game.play_card(card, &[sq("c6")]).unwrap();
    assert_eq!(game.position().ice_zones().count_ones(), 8);
    let turn = |game: &mut Game, from: &str, to: &str| {
        game.play(find(game.position(), from, to)).unwrap();
        if !game.is_over() && game.legal_moves().is_empty() && game.can_end_turn() {
            game.end_turn();
        }
    };
    turn(&mut game, "e1", "e2");
    // Ход чёрных, потом белых — стена держится; второй ход чёрных — ещё тоже.
    turn(&mut game, "e8", "d8");
    turn(&mut game, "e2", "e3");
    assert_eq!(game.position().ice_zones().count_ones(), 8);
    let outcome = game.play(find(game.position(), "a8", "b6")).unwrap();
    assert!(outcome.frozen, "конь встал на 6-ю горизонталь");
    if game.position().side() == Color::Black {
        game.end_turn();
    }
    // Начался ход белых после второго хода соперника — стена растаяла.
    assert_eq!(game.position().ice_zones(), 0);
}

#[test]
fn mass_sacrifice_halves_own_pieces_for_points() {
    let (pos, card) = with_card(
        "4k3/8/8/8/2pN4/2R5/8/4K3 w - - 0 1",
        CardKind::MassSacrifice,
        8,
    );
    let mut game = Game::from_position(pos);
    let play = game.play_card(card, &[sq("c3")]).unwrap();
    assert_eq!(play.hits.len(), 2, "конь и ладья, чужая пешка — нет");
    let pos = game.position();
    assert_eq!(
        (hp(pos, "d4"), hp(pos, "c3"), hp(pos, "c4")),
        (450, 600, 900)
    );
    assert_eq!(pos.points(Color::White, Points::Sacrifice), 3 + 5);
    assert_eq!(pos.points(Color::White, Points::Mana), 2);
}

#[test]
fn blood_god_turns_sacrifice_points_into_victory() {
    let (mut pos, card) = with_card("4k3/8/8/8/8/8/r7/R3K3 w - - 0 1", CardKind::BloodGod, 5);
    pos.set_hp(sq("a1"), 50);
    let mut game = Game::from_position(pos);
    game.play_card(card, &[]).unwrap();
    assert!(game.position().blood_god(Color::White));
    game.play(find(game.position(), "e1", "f1")).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    // Чёрная ладья убивает ладью a1: белым вместо 5 очков жертвы — 5 очков победы.
    game.play(find(game.position(), "e8", "d8")).unwrap();
    if game.position().side() == Color::Black {
        assert!(game.end_turn());
    }
    let pos = game.position();
    assert!(pos.piece(sq("a1")).is_none());
    assert_eq!(pos.points(Color::White, Points::Sacrifice), 0);
    assert_eq!(pos.points(Color::White, Points::Victory), 5);
}
#[test]
fn absolute_zero_freezes_half_forever_and_the_rest_decides() {
    // У чёрных 4 свободные фигуры: король и три коня. Замерзает половина (2), короли — нет.
    let (pos, card) = with_card(
        "4k3/8/8/8/n1n1n3/8/8/R3K3 w - - 0 1",
        CardKind::AbsoluteZero,
        30,
    );
    assert_eq!(CardKind::AbsoluteZero.cost(), Some((Points::Mana, 30)));
    let mut game = Game::from_position(pos);
    let play = game.play_card(card, &[]).unwrap();
    assert_eq!(play.frozen.len(), 2);
    let pos = game.position();
    assert!(
        play.frozen
            .iter()
            .all(|&sq| pos.piece(sq).unwrap().frozen == FOREVER)
    );
    assert!(
        !pos.piece(sq("e8")).unwrap().is_frozen(),
        "король не замерзает"
    );
    // Навсегда: ходы проходят, лёд не тает.
    game.play(find(game.position(), "e1", "d1")).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    game.play(find(game.position(), "e8", "f8")).unwrap();
    if game.position().side() == Color::Black {
        assert!(game.end_turn());
    }
    let mut pos = game.position().clone();
    assert!(
        play.frozen
            .iter()
            .all(|&sq| pos.piece(sq).unwrap().frozen == FOREVER)
    );
    // Свободных у чёрных — король и один конь; убрали их — чёрные проиграли, хотя
    // замёрзшие кони на доске.
    let free: Vec<Square> = pos
        .pieces()
        .filter(|&(_, p)| p.color == Color::Black && p.frozen != FOREVER)
        .map(|(s, _)| s)
        .collect();
    assert_eq!(free.len(), 2);
    for s in free {
        pos.board[s as usize] = None;
    }
    pos.kings[Color::Black as usize] = None;
    assert!(!pos.has_pieces(Color::Black));
    assert_eq!(
        Game::from_position(pos).status(),
        Status::Victory {
            winner: Color::White
        }
    );
}

#[test]
fn ice_shard_wounds_and_freezes_an_enemy() {
    let fen = "4k3/8/8/3n4/8/8/8/4K3 w - - 0 1";
    let (pos, card) = with_card(fen, CardKind::IceShard, 2);
    assert_eq!(CardKind::IceShard.cost(), Some((Points::Mana, 2)));
    assert_eq!(pos.card_targets(card), [sq("d5"), sq("e8")], "только чужие");
    let mut game = Game::from_position(pos.clone());
    let play = game.play_card(card, &[sq("d5")]).unwrap();
    assert_eq!(play.frozen, [sq("d5")]);
    assert_eq!(hp(game.position(), "d5"), 400);
    assert_eq!(frozen(game.position(), "d5"), 2);
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("замёрз")
    );
    // Король получает урон, но не мёрзнет.
    let mut king = pos.clone();
    king.play_card(card, &[sq("e8")]).unwrap();
    assert_eq!((hp(&king, "e8"), frozen(&king, "e8")), (1000, 0));
    // Убитого — не морозят, за него очки победы.
    let mut kill = pos.clone();
    kill.set_hp(sq("d5"), 400);
    let play = kill.play_card(card, &[sq("d5")]).unwrap();
    assert!(play.hits[0].killed && play.frozen.is_empty());
    assert!(kill.piece(sq("d5")).is_none());
    assert_eq!(kill.points(Color::White, Points::Victory), 3);
    assert_eq!(kill.kills(Color::White), 1);
}

#[test]
fn barrier_keeps_enemies_out_until_the_next_turn() {
    // Ладьи a4, h4 и d8 упираются в барьер c3–e5 вокруг коня d4, конь b5 не может его бить.
    let fen = "3rk3/8/8/1n6/r2N3r/8/8/4K3 w - - 0 1";
    let (pos, card) = with_card(fen, CardKind::Barrier, 2);
    assert_eq!(CardKind::Barrier.cost(), Some((Points::Victory, 2)));
    assert_eq!(pos.card_targets(card).len(), 64);
    let mut open = pos.clone();
    open.end_turn();
    assert!(has_move(&open, "d8", "d5") && has_move(&open, "a4", "c4"));
    assert_eq!(open.max_damage(sq("d4"), Color::Black), 300);

    let mut walled = pos.clone();
    walled.play_card(card, &[sq("d4")]).unwrap();
    assert_eq!(walled.barrier(Color::White).count_ones(), 9);
    walled.end_turn();
    for (from, to) in [
        ("d8", "d5"),
        ("a4", "c4"),
        ("a4", "d4"),
        ("h4", "e4"),
        ("b5", "c3"),
    ] {
        assert!(!has_move(&walled, from, to), "{from}-{to}");
    }
    for (from, to) in [("d8", "d6"), ("a4", "b4"), ("h4", "f4"), ("b5", "d6")] {
        assert!(has_move(&walled, from, to), "{from}-{to}");
    }
    // В барьер не бьют: коню d4 ничто не грозит, и в конце хода чёрных его не бьют.
    assert_eq!(walled.max_damage(sq("d4"), Color::Black), 0);
    assert!(strikes(&mut walled).is_empty());
    // В начале следующего хода белых барьер исчез.
    assert_eq!(walled.barrier(Color::White), 0);
    walled.end_turn();
    assert!(has_move(&walled, "d8", "d5"));
}
#[test]
fn barrier_stops_swaps_and_castling_into_it() {
    // Барьер чёрных на e1–g2 (вокруг f2): белым не рокироваться коротко и не поставить
    // обменом коня c2 на клетку внутри.
    let mut pos = position("r3k3/8/8/8/8/8/2N5/R3K2R b KQ - 0 1");
    pos.give_card(Color::Black, CardKind::Barrier);
    pos.give_points(Color::Black, Points::Victory, 2);
    let card = pos.cards(Color::Black).hand().len() - 1;
    pos.play_card(card, &[sq("f2")]).unwrap();
    pos.end_turn();
    assert!(!has_move(&pos, "e1", "g1"), "король проходит через барьер");
    assert!(has_move(&pos, "e1", "c1"));
    assert!(
        !pos.swap_partners(sq("b1")).contains(&sq("e1")),
        "король в барьере"
    );
}

#[test]
fn cold_ruler_pays_two_mana_per_frozen_enemy() {
    let fen = "4k3/8/8/3nn3/8/8/8/4K3 w - - 0 1";
    let (mut pos, card) = with_card(fen, CardKind::ColdRuler, 4);
    assert_eq!(CardKind::ColdRuler.cost(), Some((Points::Sacrifice, 4)));
    assert!(!pos.card_playable(card), "замёрзших нет");
    pos.set_frozen(sq("d5"), 1);
    pos.set_frozen(sq("e5"), FOREVER);
    assert!(pos.card_playable(card));
    let mana = pos.points(Color::White, Points::Mana);
    pos.play_card(card, &[]).unwrap();
    assert_eq!(pos.points(Color::White, Points::Mana), mana + 4);
    assert_eq!(pos.points(Color::White, Points::Sacrifice), 0);
}

#[test]
fn ice_blast_hits_everyone_within_two_squares() {
    // Замёрзший конь d5 (атака 300) бьёт пешки c4, d3 и f7; ладья a1 и короли далеко.
    let fen = "4k3/5p2/8/3n4/2P5/3P4/8/R3K3 w - - 0 1";
    let (mut pos, card) = with_card(fen, CardKind::IceBlast, 4);
    assert_eq!(CardKind::IceBlast.cost(), Some((Points::Mana, 4)));
    assert!(!pos.card_playable(card), "замёрзших чужих нет");
    pos.set_frozen(sq("d5"), 1);
    assert_eq!(pos.card_targets(card), [sq("d5")]);
    pos.set_hp(sq("f7"), 300);
    let play = pos.play_card(card, &[sq("d5")]).unwrap();
    assert_eq!(play.hits.len(), 3);
    assert_eq!((hp(&pos, "c4"), hp(&pos, "d3")), (600, 600));
    assert!(pos.piece(sq("f7")).is_none());
    assert_eq!(hp(&pos, "d5"), 900, "сам не страдает");
    assert_eq!(
        (hp(&pos, "a1"), hp(&pos, "e1"), hp(&pos, "e8")),
        (1200, 1500, 1500)
    );
    assert_eq!(pos.points(Color::White, Points::Victory), 1);
}

#[test]
fn victory_tales_pays_two_mana_per_killed_enemy() {
    let (mut pos, card) = with_card("4k3/8/8/8/8/8/p7/R3K3 w - - 0 1", CardKind::VictoryTales, 3);
    assert_eq!(CardKind::VictoryTales.cost(), Some((Points::Victory, 3)));
    assert!(!pos.card_playable(card), "побеждённых нет");
    pos.set_hp(sq("a2"), 100);
    // Ладья a1 добивает пешку a2 в конце хода белых.
    assert_eq!(strikes(&mut pos), ["a1-a2 300†"]);
    assert_eq!(pos.kills(Color::White), 1);
    pos.end_turn();
    assert!(pos.card_playable(card));
    let mana = pos.points(Color::White, Points::Mana);
    pos.play_card(card, &[]).unwrap();
    assert_eq!(pos.points(Color::White, Points::Mana), mana + 2);
}
#[test]
fn hard_winter_hits_only_frozen_enemies() {
    let fen = "4k3/8/8/3nn3/8/8/3N4/4K3 w - - 0 1";
    let (mut pos, card) = with_card(fen, CardKind::HardWinter, 5);
    assert_eq!(CardKind::HardWinter.cost(), Some((Points::Sacrifice, 5)));
    pos.set_frozen(sq("d2"), 1);
    assert!(!pos.card_playable(card), "замёрзла только своя");
    pos.set_frozen(sq("d5"), 1);
    let play = pos.play_card(card, &[]).unwrap();
    assert_eq!(play.hits.len(), 1);
    assert_eq!(
        (hp(&pos, "d5"), hp(&pos, "e5"), hp(&pos, "d2")),
        (400, 900, 900)
    );
}

#[test]
fn ice_warrior_freezes_the_survivors_of_its_strikes() {
    let fen = "4k3/8/8/8/3n4/8/8/3RK3 w - - 0 1";
    let (pos, card) = with_card(fen, CardKind::IceWarrior, 3);
    assert_eq!(CardKind::IceWarrior.cost(), Some((Points::Victory, 3)));
    assert_eq!(pos.card_targets(card), [sq("d1"), sq("e1")], "только свои");
    let mut game = Game::from_position(pos);
    game.play_card(card, &[sq("d1")]).unwrap();
    assert!(game.position().piece(sq("d1")).unwrap().frost_touch);
    // Ладья встаёт рядом с конём и в конце хода бьёт его — конь замерзает.
    game.play(find(game.position(), "d1", "d3")).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    let pos = game.position();
    assert_eq!(hp(pos, "d4"), 600);
    assert_eq!(frozen(pos, "d4"), 1);
    assert!(game.history().last().unwrap().strikes()[0].chilled);
    assert!(
        !pos.legal_moves().iter().any(|m| m.from == sq("d4")),
        "конь пропускает ход"
    );
}
#[test]
fn north_warrior_grows_with_frozen_enemies() {
    let fen = "4k3/8/8/3nn3/8/8/8/R3K3 w - - 0 1";
    let (mut pos, card) = with_card(fen, CardKind::NorthWarrior, 3);
    assert_eq!(CardKind::NorthWarrior.cost(), Some((Points::Victory, 3)));
    assert!(!pos.card_playable(card), "замёрзших нет");
    pos.set_frozen(sq("d5"), 1);
    pos.set_frozen(sq("e5"), 1);
    assert_eq!(pos.card_targets(card), [sq("a1"), sq("e1")]);
    pos.play_card(card, &[sq("a1")]).unwrap();
    let rook = pos.piece(sq("a1")).unwrap();
    assert_eq!((rook.attack(), rook.max_hp(), rook.hp), (400, 1400, 1400));
}

#[test]
fn deck_codes_round_trip_and_catch_damage() {
    // Порядок видов карт закреплён: от него зависят старые коды.
    let names: Vec<String> = CardKind::ALL.iter().map(|k| format!("{k:?}")).collect();
    assert_eq!(
        names.join(","),
        "Normal,FireWave,Sacrifice,Victory,Swap,Funding,Heal,Storm,Avalanche,IceBarrier,Offering,\
         Encircle,Multiply,Champion,Bastion,IceWall,MassSacrifice,BloodGod,AbsoluteZero,IceShard,\
         Barrier,ColdRuler,IceBlast,VictoryTales,HardWinter,IceWarrior,NorthWarrior"
    );
    // Известный код: две огненные волны и одна буря.
    let known = encode_deck_code(&[(CardKind::Storm, 1), (CardKind::FireWave, 2)]);
    assert_eq!(known, GOLDEN_CODE);
    assert_eq!(
        decode_deck_code(GOLDEN_CODE).unwrap(),
        [(CardKind::FireWave, 2), (CardKind::Storm, 1)]
    );
    // Коды стандартных колод — те же, что даёт страница tools/deck_builder.html.
    assert_eq!(
        encode_deck_code(balance().deck(0)),
        "BC1-FAECBAEFAgYCBwEIAQkBCgEOAQ8CEAESARMCFAEVAhYCFwIYARkCGgIo"
    );
    assert_eq!(
        encode_deck_code(balance().deck(1)),
        "BC1-GgACAQQCAgQCBQIGAgcBCAEJAQoBCwEMAQ0BDgEPARABEQESARMBFAEVARYBFwEYARkBGgEb"
    );
    // Все стандартные колоды туда и обратно; пробелы и переносы строк не мешают.
    for index in 0..DECK_COUNT {
        let mut recipe = balance().deck(index).to_vec();
        recipe.sort_by_key(|&(k, _)| k as usize);
        let code = encode_deck_code(&recipe);
        assert!(code.starts_with("BC1-") && code.is_ascii());
        assert_eq!(decode_deck_code(&code).unwrap(), recipe);
        let split = format!("  {}\r\n{}\n ", &code[..20], &code[20..]);
        assert_eq!(decode_deck_code(&split).unwrap(), recipe);
    }
    // Испорченное, обрезанное и чужое.
    let code = encode_deck_code(balance().deck(1));
    assert!(
        decode_deck_code(&code[..code.len() - 3]).is_err(),
        "обрезан"
    );
    let mut flipped: Vec<char> = code.chars().collect();
    flipped[10] = if flipped[10] == 'A' { 'B' } else { 'A' };
    assert!(
        decode_deck_code(&flipped.iter().collect::<String>()).is_err(),
        "знак изменён"
    );
    assert!(decode_deck_code("hello").unwrap_err().contains("BC1-"));
    assert!(
        decode_deck_code("BC1-а")
            .unwrap_err()
            .contains("лишний знак")
    );
    assert!(decode_deck_code("BC1-").is_err() && decode_deck_code("").is_err());
    // Неизвестная карта, пустая и слишком большая колода (с верным контрольным байтом).
    let forge = |pairs: &[(u8, u8)]| {
        let mut bytes = vec![pairs.len() as u8];
        for &(k, n) in pairs {
            bytes.extend([k, n]);
        }
        let sum = bytes
            .iter()
            .fold(0xA5u8, |s, &b| s.wrapping_mul(31).wrapping_add(b));
        bytes.push(sum);
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut code = String::from("BC1-");
        let mut bits = 0u32;
        let mut have = 0;
        for b in bytes {
            bits = bits << 8 | u32::from(b);
            have += 8;
            while have >= 6 {
                have -= 6;
                code.push(alphabet[(bits >> have & 63) as usize] as char);
            }
            bits &= (1 << have) - 1;
        }
        if have > 0 {
            code.push(alphabet[(bits << (6 - have) & 63) as usize] as char);
        }
        code
    };
    assert_eq!(
        decode_deck_code(&forge(&[(1, 2)])).unwrap(),
        [(CardKind::FireWave, 2)]
    );
    assert!(
        decode_deck_code(&forge(&[(27, 1)]))
            .unwrap_err()
            .contains("неизвестная карта")
    );
    assert!(
        decode_deck_code(&forge(&[]))
            .unwrap_err()
            .contains("от 1 до 64")
    );
    assert!(
        decode_deck_code(&forge(&[(1, 40), (7, 40)]))
            .unwrap_err()
            .contains("от 1 до 64")
    );
}

const GOLDEN_CODE: &str = "BC1-AgECBwE4";
/// Переписывает данные в `tools/deck_builder.html` — странице, где пользователь собирает колоды
/// и правит баланс: все виды карт (название, тип, стандартные цена и числа, описание, число
/// копий в каждой колоде), правила и фигуры. Берётся стандартный баланс. Запускать после
/// любой правки карт, чисел или колод: `cargo test --release refresh_deck_builder -- --ignored`.
#[test]
#[ignore = "переписывает tools/deck_builder.html"]
fn refresh_deck_builder_data() {
    use crate::balance::{Balance, PARAMS, RULES, ordered_kinds, rule_in_units, to_file};
    fn js(text: &str) -> String {
        let mut out = String::from("\"");
        for c in text.chars() {
            match c {
                '"' => out += "\\\"",
                '\\' => out += "\\\\",
                '\n' => out += "\\n",
                c => out.push(c),
            }
        }
        out + "\""
    }
    let defaults = Balance::default();
    let cards: Vec<String> = ordered_kinds()
        .into_iter()
        .map(|kind| {
            let (family, unit) = match kind.family() {
                Family::Normal => ("normal", "mana"),
                Family::Sacrifice => ("sacrifice", "sacrifice"),
                Family::Victory => ("victory", "victory"),
            };
            let params: Vec<String> = PARAMS
                .iter()
                .filter(|p| p.card == kind)
                .map(|p| {
                    format!(
                        "{{ key: {}, value: {}, min: {}, max: {}, note: {} }}",
                        js(p.key),
                        defaults.param(p.param),
                        p.min,
                        p.max,
                        js(p.note)
                    )
                })
                .collect();
            let copies: Vec<String> = (0..DECK_COUNT)
                .map(|i| {
                    let n = defaults.deck(i).iter().find(|&&(k, _)| k == kind);
                    n.map_or(0, |&(_, n)| n).to_string()
                })
                .collect();
            format!(
                "  {{ id: {}, index: {}, title: {}, family: \"{family}\", unit: \"{unit}\", cost: {}, \
                 params: [{}], inGame: [{}], description: {} }},",
                js(&format!("{kind:?}")),
                kind as usize,
                js(kind.title()),
                defaults.cost(kind),
                params.join(", "),
                copies.join(", "),
                js(&kind.description())
            )
        })
        .collect();
    let rules: Vec<String> = RULES
        .iter()
        .map(|r| {
            format!(
                "{{ key: {}, value: {}, min: {}, max: {}, note: {} }}",
                js(r.key),
                if rule_in_units(r.key) {
                    to_file(r.default)
                } else {
                    r.default
                },
                r.min,
                r.max,
                js(r.note)
            )
        })
        .collect();
    let pieces: Vec<String> = Kind::ALL
        .into_iter()
        .map(|kind| {
            let p = defaults.piece(kind);
            format!(
                "{{ id: \"{kind:?}\", title: {}, attack: {}, hp: {}, points: {} }}",
                js(kind.title()),
                to_file(p.attack),
                to_file(p.hp),
                p.points
            )
        })
        .collect();
    let names: Vec<String> = DECK_NAMES.iter().map(|n| js(n)).collect();
    let block = format!(
        "/* CARDS:BEGIN — не править руками: cargo test --release refresh_deck_builder -- --ignored */\n\
         const CARDS = [\n{}\n];\n\
         const BALANCE = {{\n  rules: [\n    {}\n  ],\n  pieces: [\n    {}\n  ],\n  decks: [{}],\n}};\n\
         /* CARDS:END */",
        cards.join("\n"),
        rules.join(",\n    "),
        pieces.join(",\n    "),
        names.join(", ")
    );
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tools/deck_builder.html");
    let html = std::fs::read_to_string(path).expect("tools/deck_builder.html");
    let begin = html.find("/* CARDS:BEGIN").expect("метка CARDS:BEGIN");
    let end_mark = "/* CARDS:END */";
    let end = html.find(end_mark).expect("метка CARDS:END") + end_mark.len();
    std::fs::write(path, format!("{}{block}{}", &html[..begin], &html[end..])).expect("запись");
}

#[test]
fn a_boosted_side_gets_three_lava_pawns() {
    let mut pos = Position::start();
    pos.boost(Color::Black);
    assert_eq!(pos.boosted(), Some(Color::Black));
    let lava: Vec<Square> = pos
        .pieces()
        .filter(|(_, p)| p.lava)
        .map(|(sq, _)| sq)
        .collect();
    assert_eq!(lava.len(), 3, "ход белых — лава сразу");
    assert!(
        lava.iter()
            .all(|&sq| pos.piece(sq).unwrap().kind == Kind::Pawn
                && pos.piece(sq).unwrap().color == Color::Black)
    );
    let sparks: Vec<Square> = pos
        .pieces()
        .filter(|(_, p)| p.spark)
        .map(|(sq, _)| sq)
        .collect();
    assert!(sparks.is_empty(), "с 0.41 электричества нет");
    // Выбор пешек один и тот же — для отмены и перебора.
    let mut again = Position::start();
    again.boost(Color::Black);
    assert_eq!(again, pos);
    // Ход компьютера: лавы в начале нет — пешки насытятся к ходу человека.
    let mut black = position(START_FEN.replace(" w ", " b ").as_str());
    black.boost(Color::Black);
    assert!(black.pieces().all(|(_, p)| !p.lava));
    let turn = black.make_turn(find(&black, "e7", "e5"));
    let _ = turn;
    assert_eq!(black.pieces().filter(|(_, p)| p.lava).count(), 3);
}

#[test]
fn lava_pawns_explode_at_the_end_of_their_turn_and_spare_their_own() {
    // Пешка d4 с лавой: вокруг белые конь c4, пешка d3, слон e5 и свой конь e4.
    let mut pos = position("4k3/8/8/4B3/2Npn3/3P4/8/4K3 b - - 0 1");
    pos.boost(Color::Black);
    pos.set_lava(sq("d4"));
    let full = |kind| Piece::new(kind, Color::White).hp;
    let turn = pos.end_turn();
    let damage = balance().rule(Rule::LavaDamage);
    assert_eq!(turn.blasts.len(), 1);
    assert_eq!(turn.blasts[0].center, sq("d4"));
    assert_eq!(turn.blasts[0].hits.len(), 3, "свой конь e4 не задет");
    assert_eq!(hp(&pos, "c4"), full(Kind::Knight) - damage);
    assert_eq!(hp(&pos, "d3"), full(Kind::Pawn) - damage);
    assert_eq!(hp(&pos, "e5"), full(Kind::Bishop) - damage);
    assert_eq!(hp(&pos, "e4"), Piece::new(Kind::Knight, Color::Black).hp);
    assert_eq!(
        hp(&pos, "d4"),
        Piece::new(Kind::Pawn, Color::Black).hp,
        "пешка цела"
    );
    // В начале хода белых пешка насытилась снова (других пешек нет).
    assert_eq!(turn.charged, [sq("d4")]);
    assert!(pos.piece(sq("d4")).unwrap().lava);
    // Замёрзшая пешка не взрывается и лаву теряет.
    let mut frozen_pawn = position("4k3/8/8/4B3/2Npn3/3P4/8/4K3 b - - 0 1");
    frozen_pawn.set_lava(sq("d4"));
    frozen_pawn.set_frozen(sq("d4"), 2);
    let turn = frozen_pawn.end_turn();
    assert!(turn.blasts.is_empty());
    assert!(!frozen_pawn.piece(sq("d4")).unwrap().lava);
    assert_eq!(hp(&frozen_pawn, "c4"), full(Kind::Knight));
}

#[test]
fn an_electric_bishop_spreads_its_strike_diagonally_around_the_target() {
    // Слон e3 бьёт коня d4; молния — на пешки c5 и e5 (по диагонали от коня), своя ладья c3
    // не задета.
    let mut pos = position("4k3/8/8/2p1p3/3n4/2R1B3/8/4K3 w - - 0 1");
    pos.set_spark(sq("e3"));
    let attack = pos.piece(sq("e3")).unwrap().attack();
    let spread = attack * balance().rule(Rule::SparkPercent) / 100;
    let pawn = Piece::new(Kind::Pawn, Color::Black).hp;
    let turn = pos.end_turn();
    // Ладья c3 бьёт раньше по очереди — берём удары слона.
    let bishop: Vec<&Strike> = turn.strikes.iter().filter(|s| s.from == sq("e3")).collect();
    assert_eq!(bishop[0].hit.square, sq("d4"));
    assert_eq!(bishop[0].hits().count(), 3);
    // Пешка c5 — за конём на той же линии: слон её не бьёт.
    assert_eq!(bishop.len(), 1);
    assert_eq!(hp(&pos, "e5"), pawn - spread);
    assert_eq!(hp(&pos, "c3"), Piece::new(Kind::Rook, Color::White).hp);
    let mut sparked = position("4k3/8/8/2p1p3/3n4/2R1B3/8/4K3 w - - 0 1");
    sparked.set_spark(sq("e3"));
    let mut game = Game::from_position(sparked);
    game.play(find(game.position(), "e1", "f1")).unwrap();
    assert!(game.end_turn());
    let record = game.history().last().unwrap();
    assert!(record.notation().text.contains('⚡'));
    assert!(record.description().contains("Молния задела"));
}

#[test]
fn boosted_games_against_random_moves_keep_the_rules() {
    let mut seed = 0x0DDB_1A5E_5BAD_5EEDu64;
    let mut next = move |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n as u64) as usize
    };
    let (mut blasts, mut sparks) = (0, 0);
    for _ in 0..30 {
        let mut game = Game::new([0; 2]);
        game.boost(Color::Black);
        for _ in 0..200 {
            if game.is_over() {
                break;
            }
            let mover = game.position().side();
            let moves = game.legal_moves().to_vec();
            if !moves.is_empty() {
                game.play(moves[next(moves.len())]).unwrap();
            }
            if !game.is_over() && game.position().side() == mover {
                assert!(game.end_turn());
            }
            let pos = game.position();
            for (_, piece) in pos.pieces() {
                assert!(!piece.lava || piece.kind == Kind::Pawn && piece.color == Color::Black);
                assert!(!piece.spark || piece.kind == Kind::Bishop && piece.color == Color::Black);
            }
            if !game.is_over() && pos.side() == Color::White {
                let lava = pos.pieces().filter(|(_, p)| p.lava).count();
                assert!(lava <= balance().rule(Rule::LavaPawns) as usize);
            }
            assert_kings_tracked(&game);
        }
        for record in game.history() {
            let Some(turn) = record.turn_end() else {
                continue;
            };
            for blast in &turn.blasts {
                assert_eq!(record.color(), Color::Black);
                assert!(blast.hits.iter().all(|h| h.target.color == Color::White));
                blasts += usize::from(!blast.hits.is_empty());
            }
            for strike in &turn.strikes {
                assert!(strike.hits().all(|h| h.target.color != record.color()));
                sparks += strike.splash.iter().flatten().count();
            }
        }
    }
    eprintln!("взрывов с уроном {blasts}, задетых молнией {sparks}");
    assert!(blasts > 20, "взрывов {blasts}");
    assert_eq!(sparks, 0, "с 0.41 электричества нет");
}
