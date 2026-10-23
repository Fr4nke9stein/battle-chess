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
    assert_eq!(
        pos.reserve(Color::White),
        [(Stock::new(Kind::Pawn, Power::Plain), 0, 1)]
    );
    let drops = |pos: &Position| pos.legal_moves().into_iter().filter(Move::is_drop).count();
    pos.make_move(find(&pos, "e8", "d8"));
    pos.end_turn();
    assert_eq!(drops(&pos), 0, "ход белых 1 — ещё ждёт");
    pos.make_move(find(&pos, "f1", "e1"));
    pos.end_turn();
    pos.make_move(find(&pos, "d8", "e8"));
    pos.end_turn();
    assert_eq!(
        pos.reserve(Color::White),
        [(Stock::new(Kind::Pawn, Power::Plain), 1, 0)]
    );
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
    let strike = outcome.reactions[0].clone();
    assert_eq!((strike.from, strike.hit.square), (sq("d5"), sq("c3")));
    assert_eq!(hp(&pos, "c3"), 1200 - 300);
    // Перехват убил — фигура уходит в запас коня.
    let mut pos = position("4k3/8/8/3n4/8/8/8/2R1K3 w - - 0 1");
    pos.set_hp(sq("c1"), 300);
    let outcome = pos.make_move(find(&pos, "c1", "c3"));
    assert!(outcome.reactions[0].hit.killed);
    assert!(pos.piece(sq("c3")).is_none());
    assert_eq!(
        pos.reserve(Color::Black),
        [(Stock::new(Kind::Rook, Power::Plain), 0, 1)]
    );
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
    let strike = outcome.reactions[0].clone();
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
                sparks += strike.splash.len();
            }
        }
    }
    eprintln!("взрывов с уроном {blasts}, задетых молнией {sparks}");
    assert!(blasts > 20, "взрывов {blasts}");
    assert_eq!(sparks, 0, "с 0.41 электричества нет");
}

/// Фигуры стороны `color`: (клетка, вид).
fn army(pos: &Position, color: Color) -> Vec<(Square, Kind)> {
    pos.pieces()
        .filter(|(_, p)| p.color == color)
        .map(|(sq, p)| (sq, p.kind))
        .collect()
}

#[test]
fn rogue_like_starts_with_the_first_wave_and_a_reserve() {
    let pos = Position::rogue_start(7);
    let rogue = pos.rogue().unwrap();
    assert_eq!(
        (rogue.player, rogue.wave, rogue.waves),
        (Color::White, 1, 10)
    );
    assert_eq!(army(&pos, Color::White).len(), 16);
    let wave = army(&pos, Color::Black);
    assert!(
        (2..=4).contains(&wave.len()),
        "волна — от 2 до 4 фигур: {wave:?}"
    );
    for (sq, kind) in wave {
        assert_ne!(kind, Kind::King);
        assert_eq!(
            rank_of(sq),
            7,
            "волна выходит на крайнюю горизонталь соперника"
        );
    }
    // В запасе — 2 пешки и одна особая фигура (не пешка).
    let ready = |kind: Kind| pos.reserve[Color::White as usize][kind as usize][0][0];
    assert_eq!(
        [Kind::Pawn, Kind::Bishop, Kind::Knight, Kind::Rook].map(ready),
        [2, 0, 0, 0]
    );
    let special: Vec<(Stock, u8, u8)> = pos
        .reserve(Color::White)
        .into_iter()
        .filter(|(stock, ..)| stock.power != Power::Plain)
        .collect();
    assert_eq!(special.len(), 1, "{special:?}");
    let (stock, ready, _) = special[0];
    assert_eq!(ready, 1);
    assert_ne!(stock.kind, Kind::Pawn);
    assert_eq!(pos.moves_per_turn(), 4);
}

#[test]
fn rogue_like_starting_special_piece_is_weighted_like_the_waves() {
    use std::collections::HashMap;
    let mut kinds: HashMap<Kind, u32> = HashMap::new();
    let mut powers: HashMap<Power, u32> = HashMap::new();
    for seed in 0..4000u64 {
        let stock = Position::starting_special(seed * 7919 + 1);
        assert_ne!(stock.kind, Kind::Pawn);
        assert_ne!(stock.kind, Kind::King);
        assert!(stock.power != Power::Plain && stock.with == Power::Plain);
        *kinds.entry(stock.kind).or_default() += 1;
        *powers.entry(stock.power).or_default() += 1;
    }
    // Веса волн: конь и слон 5, ладья 3, ферзь 1 — из 14.
    let share = |kind: Kind| f64::from(kinds[&kind]) / 4000.0;
    for (kind, weight) in [
        (Kind::Knight, 5.0),
        (Kind::Bishop, 5.0),
        (Kind::Rook, 3.0),
        (Kind::Queen, 1.0),
    ] {
        assert!(
            (share(kind) - weight / 14.0).abs() < 0.03,
            "{kind:?}: {}",
            share(kind)
        );
    }
    // Все восемь сил — примерно поровну.
    assert_eq!(powers.len(), 8);
    for (power, n) in powers {
        assert!(
            (f64::from(n) / 4000.0 - 0.125).abs() < 0.03,
            "{power:?}: {n}"
        );
    }
    // У одного зерна — одна и та же фигура.
    assert_eq!(
        Position::starting_special(42),
        Position::starting_special(42)
    );
}

#[test]
fn rogue_like_player_moves_four_different_pieces() {
    let mut game = Game::from_position(Position::rogue_start(7));
    assert!(!game.can_end_turn());
    for (from, to) in [("e2", "e4"), ("d2", "d4"), ("g1", "f3"), ("b1", "c3")] {
        assert_eq!(game.position().side(), Color::White);
        game.play(find(game.position(), from, to)).unwrap();
        // Сходившая фигура больше не ходит.
        assert!(!has_move(game.position(), to, "a6"));
        assert!(game.legal_moves().iter().all(|m| m.from != sq(to)));
    }
    // Четыре хода сделаны — ход кончился сам.
    assert_eq!(game.position().side(), Color::Black);

    // Закончить ход можно и раньше — после первого хода.
    let mut early = Game::from_position(Position::rogue_start(7));
    early.play(find(early.position(), "e2", "e4")).unwrap();
    assert_eq!(early.position().side(), Color::White);
    assert!(early.can_end_turn());
    assert!(early.end_turn());
    assert_eq!(early.position().side(), Color::Black);
}

#[test]
fn rogue_like_has_no_reserve_for_kills_and_runs_ten_levels() {
    let mut game = Game::from_position(Position::rogue_start(11));
    let start = game.position().rogue().unwrap();
    assert_eq!((start.level, start.levels, start.waves), (1, 10, 10));
    let mut level_ups = 0;
    for _ in 0..1000 {
        // Волны убиты (без «шведки» — запас не растёт).
        let reserve = game.position.reserve;
        for (sq, kind) in army(&game.position, Color::Black) {
            game.position.board[sq as usize] = None;
            game.position
                .score_kill(Piece::new(kind, Color::Black), Color::White);
        }
        assert_eq!(game.position.reserve, reserve, "шведки нет");
        game.refresh();
        let r = game.position().rogue().unwrap();
        if game.is_over() {
            assert_eq!(
                game.status(),
                Status::Victory {
                    winner: Color::White
                }
            );
            assert_eq!(
                (r.level, r.wave),
                (r.levels, r.waves),
                "все уровни пройдены"
            );
            assert_eq!(level_ups, 9);
            return;
        }
        // Игрок ходит и кончает ход: волны уровня кончились — расстановка (ход остаётся у
        // игрока), «Начать раунд» — новый уровень, на волну длиннее, его первая волна выходит
        // сразу, и снова ходит игрок.
        game.play(game.legal_moves()[0]).unwrap();
        if game.position().side() == Color::White {
            assert!(game.end_turn());
        }
        let turn = game.history().last().unwrap().turn_end().unwrap().clone();
        if r.wave == r.waves {
            assert_eq!(turn.planning, Some(r.level + 1));
            assert!(game.position().planning());
            assert_eq!(game.position().side(), Color::White);
            assert!(
                game.history()
                    .last()
                    .unwrap()
                    .description()
                    .contains("Расстановка")
            );
            assert!(game.position().shop_open() && !game.can_end_turn());
            assert!(game.leave_shop());
            assert!(game.end_turn(), "«Начать раунд»");
            let start = game.history().last().unwrap().turn_end().unwrap().clone();
            let after = game.position().rogue().unwrap();
            assert_eq!(start.level_up, Some(r.level + 1));
            assert_eq!(
                (after.level, after.wave, after.waves, after.planning),
                (r.level + 1, 1, r.waves + 1, false)
            );
            assert!((2..=4).contains(&start.spawned.len()));
            assert_eq!(game.position().side(), Color::White);
            level_ups += 1;
            continue;
        }
        assert_eq!(turn.planning, None);
        assert!(turn.spawned.is_empty());
        // Соперник сам не ходит; в конце его хода выходит волна — на крайнюю горизонталь.
        assert!(game.legal_moves().is_empty());
        assert!(game.end_turn());
        let spawned = &game.history().last().unwrap().turn_end().unwrap().spawned;
        // Последняя волна первого уровня — ещё и король-толстосум.
        assert!((2..=5).contains(&spawned.len()));
        assert!(spawned.iter().all(|&(sq, _)| rank_of(sq) == 7));
    }
    unreachable!("победа после последнего уровня");
}

/// Rogue-like: все волны уровня вышли и отбиты (чужих фигур нет) — следующий конец хода
/// игрока начнёт расстановку.
fn clear_level(game: &mut Game) {
    let mut r = game.position.rogue.unwrap();
    r.wave = r.waves;
    game.position.rogue = Some(r);
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    game.refresh();
}

#[test]
fn rogue_like_planning_restores_the_army_and_lets_it_be_rearranged() {
    let mut game = Game::from_position(Position::rogue_start(7));
    game.position.reserve[Color::White as usize][Kind::Knight as usize][0][0] = 1;
    clear_level(&mut game);
    // За уровень: конь g1–f3, пешка e2–e4, конь из запаса на c3; пешка h2 погибла, ладья a1
    // ранена.
    game.play(find(game.position(), "g1", "f3")).unwrap();
    game.play(find(game.position(), "e2", "e4")).unwrap();
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::Drop(Kind::Knight) && m.to == sq("c3"))
        .unwrap();
    game.play(drop).unwrap();
    game.position.board[sq("h2") as usize] = None;
    game.position.board[sq("a1") as usize].as_mut().unwrap().hp = 100;
    let knights = |pos: &Position| pos.reserve[Color::White as usize][Kind::Knight as usize][0];
    assert_eq!(knights(game.position()).iter().sum::<u8>(), 0);
    assert!(game.end_turn());

    // Расстановка: фигуры — на местах начала уровня, здоровы; конь из запаса — снова в
    // запасе; погибшая пешка не вернулась. Ходит по-прежнему игрок.
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    assert_eq!(turn.planning, Some(2));
    // Сначала магазин: доска ждёт, пока из него не уйдут.
    assert!(game.position().shop_open());
    assert!(game.legal_moves().is_empty() && !game.can_end_turn());
    assert!(game.leave_shop());
    let mut returned = turn.returned.clone();
    returned.sort_unstable();
    assert_eq!(returned, [(sq("f3"), sq("g1")), (sq("e4"), sq("e2"))]);
    let pos = game.position();
    assert!(pos.planning());
    assert_eq!(pos.side(), Color::White);
    assert_eq!(pos.piece(sq("g1")).unwrap().kind, Kind::Knight);
    assert_eq!(pos.piece(sq("e2")).unwrap().kind, Kind::Pawn);
    for name in ["f3", "e4", "c3", "h2"] {
        assert!(pos.piece(sq(name)).is_none(), "{name}");
    }
    let rook = pos.piece(sq("a1")).unwrap();
    assert_eq!(rook.hp, rook.max_hp(), "вылечилась");
    assert_eq!(knights(pos).iter().sum::<u8>(), 1, "конь вернулся в запас");
    assert!(game.can_end_turn());

    // Пешка — только на вторую горизонталь, остальные — на две первые; обмен — если обе
    // фигуры встают на новые клетки.
    let can = |from: &str, to: &str| has_move(game.position(), from, to);
    assert!(can("e2", "h2") && !can("e2", "e3"));
    assert!(can("g1", "h2") && !can("g1", "d3"));
    assert!(can("g1", "f1"), "конь и слон меняются местами");
    assert!(!can("g1", "e2"), "пешка не встаёт на первую горизонталь");
    let drops = |kind: Kind, to: &str| {
        game.legal_moves()
            .iter()
            .any(|m| m.kind == MoveKind::Drop(kind) && m.to == sq(to))
    };
    assert!(drops(Kind::Pawn, "h2") && !drops(Kind::Pawn, "h3"));
    assert!(drops(Kind::Knight, "h2") && !drops(Kind::Knight, "d3"));

    // Обмен, ход на пустую клетку и выставление ходов не тратят.
    let swap = game.play(find(game.position(), "g1", "f1")).unwrap();
    assert_eq!(swap.rook, Some((sq("f1"), sq("g1"))));
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::Drop(Kind::Pawn) && m.to == sq("h2"))
        .unwrap();
    game.play(drop).unwrap();
    for _ in 0..4 {
        game.play(find(game.position(), "f1", "g1")).unwrap();
        game.play(find(game.position(), "g1", "f1")).unwrap();
    }
    let pos = game.position();
    assert!(pos.planning() && pos.moves_left() == pos.moves_per_turn());
    assert_eq!(pos.piece(sq("g1")).unwrap().kind, Kind::Bishop);
    assert_eq!(pos.piece(sq("f1")).unwrap().kind, Kind::Knight);
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .starts_with("Расстановка")
    );
    // Отмена — по одному действию.
    assert!(game.undo());
    assert_eq!(game.position().piece(sq("g1")).unwrap().kind, Kind::Knight);
    game.play(find(game.position(), "g1", "f1")).unwrap();

    // «Начать раунд»: уровень 2, первая волна, ходит игрок.
    assert!(game.end_turn());
    let start = game.history().last().unwrap().turn_end().unwrap().clone();
    assert_eq!(start.level_up, Some(2));
    assert!((2..=4).contains(&start.spawned.len()));
    let pos = game.position();
    let r = pos.rogue().unwrap();
    assert_eq!((r.level, r.wave, r.planning), (2, 1, false));
    assert_eq!(pos.side(), Color::White);
    assert!(!game.can_end_turn(), "новый уровень — обычный ход");

    // Следующая расстановка возвращает на места начала второго уровня.
    clear_level(&mut game);
    game.play(find(game.position(), "f1", "e3")).unwrap();
    assert!(game.end_turn());
    assert!(game.leave_shop());
    let pos = game.position();
    assert!(pos.planning());
    assert_eq!(pos.piece(sq("f1")).unwrap().kind, Kind::Knight);
    assert_eq!(pos.piece(sq("g1")).unwrap().kind, Kind::Bishop);
    assert_eq!(pos.piece(sq("h2")).unwrap().kind, Kind::Pawn);
}

#[test]
fn rogue_like_gives_gold_for_defeated_pieces() {
    // Пешка e2 бьёт по диагонали: чёрная пешка d3 с 1 HP погибает — 1 золото.
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    game.position.board[sq("d3") as usize] = Some(Piece {
        hp: 1,
        ..Piece::new(Kind::Pawn, Color::Black)
    });
    game.refresh();
    game.play(find(game.position(), "g1", "h3")).unwrap();
    assert!(game.end_turn());
    let record = game.history().last().unwrap();
    let turn = record.turn_end().unwrap();
    assert_eq!(turn.gold, 1);
    assert!(record.description().contains("Золото: +1 (всего 1)"));
    assert_eq!(game.position().rogue().unwrap().gold, 1);
    // Цены — как в балансе: пешка 1, конь и слон 2, ладья 3, ферзь 4.
    let mut pos = Position::rogue_start(7);
    for kind in [
        Kind::Pawn,
        Kind::Knight,
        Kind::Bishop,
        Kind::Rook,
        Kind::Queen,
    ] {
        pos.score_kill(Piece::new(kind, Color::Black), Color::White);
    }
    assert_eq!(pos.rogue().unwrap().gold, 1 + 2 + 2 + 3 + 4);
    // Свои потери золота не дают.
    pos.score_kill(Piece::new(Kind::Queen, Color::White), Color::Black);
    assert_eq!(pos.rogue().unwrap().gold, 12);
}

#[test]
fn rogue_like_shop_sells_pieces_into_the_reserve() {
    let mut game = Game::from_position(Position::rogue_start(7));
    let wave: u16 = army(&game.position, Color::Black)
        .iter()
        .map(|&(_, kind)| rogue_price(kind))
        .sum();
    // «Выиграть раунд»: волны побеждены, за фигуры на доске — золото; открылся магазин.
    assert!(game.position().can_win_round());
    assert!(game.win_round());
    let pos = game.position();
    assert_eq!(pos.rogue().unwrap().gold, wave);
    assert!(pos.shop_open() && pos.planning());
    assert_eq!(pos.side(), Color::White);
    let offers = pos.offers();
    assert_eq!(offers.len(), 3);
    assert!(
        offers
            .iter()
            .all(|o| o.is_some_and(|s| s.kind != Kind::King))
    );

    game.position.rogue.as_mut().unwrap().gold = 10;
    let stock = offers[0].unwrap();
    let kind = stock.kind;
    let price = shop_price(stock);
    let fire = if stock.power == Power::Plain { 1 } else { 2 };
    assert_eq!(
        price,
        2 * fire * rogue_price(kind),
        "в магазине — вдвое дороже"
    );
    let ready = |game: &Game| {
        game.position()
            .reserve(Color::White)
            .iter()
            .find(|s| s.0 == stock)
            .map_or(0, |s| s.1)
    };
    let before = ready(&game);
    assert!(game.buy(0));
    assert_eq!(game.position().rogue().unwrap().gold, 10 - price);
    assert_eq!(ready(&game), before + 1, "куплена — в запасе, готова");
    assert_eq!(game.position().offers()[0], None);
    assert!(!game.buy(0), "место пусто");
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .starts_with("Магазин:")
    );
    // Отмена возвращает и золото, и фигуру на прилавок.
    assert!(game.undo());
    assert_eq!(game.position().rogue().unwrap().gold, 10);
    assert_eq!(game.position().offers()[0], Some(stock));
    assert!(game.buy(0));

    // Обновить — за 2 золота, прилавок снова полон; без золота — нельзя ни купить, ни
    // обновить.
    let gold = game.position().rogue().unwrap().gold;
    assert!(game.reroll());
    assert_eq!(game.position().rogue().unwrap().gold, gold - 2);
    assert!(game.position().offers().iter().all(Option::is_some));
    game.position.rogue.as_mut().unwrap().gold = 0;
    assert!(!game.reroll());
    assert!(!(0..3).any(|i| game.buy(i)));

    // «Уйти» — расстановка: купленную фигуру можно выставить.
    assert!(game.leave_shop());
    assert!(!game.position().shop_open() && game.position().planning());
    // Две первые горизонтали заняты целиком — выставлять некуда; освободилась h2 — можно.
    let drop_h2 = |game: &Game| {
        game.legal_moves()
            .iter()
            .any(|m| m.kind == stock.drop_kind() && m.to == sq("h2"))
    };
    assert!(!game.legal_moves().iter().any(Move::is_drop));
    game.position.board[sq("h2") as usize] = None;
    game.refresh();
    assert!(drop_h2(&game));
    assert!(game.can_end_turn());
}

#[test]
fn rogue_like_shop_is_weighted_like_the_first_waves() {
    let mut pos = Position::rogue_start(3);
    pos.rogue.as_mut().unwrap().shop = true;
    let mut count = std::collections::HashMap::new();
    let mut all: f64 = 0.0;
    for _ in 0..1500 {
        pos.roll_offers();
        for stock in pos.offers().into_iter().flatten() {
            *count.entry(stock).or_insert(0.0f64) += 1.0;
            all += 1.0;
        }
    }
    let share = |kind: Kind, power: Power| {
        count.get(&Stock::new(kind, power)).copied().unwrap_or(0.0) / all
    };
    // Веса волн первого уровня: пешка 15, конь и слон 5, ладья 3, ферзь 1 (из 29); огненная и
    // электрическая — каждая в 6 раз реже обычной того же вида.
    let pawn = share(Kind::Pawn, Power::Plain);
    assert!((pawn - 15.0 / 29.0 * 6.0 / 13.0).abs() < 0.04, "{pawn}");
    assert!(share(Kind::Queen, Power::Plain) < share(Kind::Rook, Power::Plain));
    assert!(share(Kind::Rook, Power::Plain) < share(Kind::Knight, Power::Plain));
    for power in [
        Power::Fire,
        Power::Spark,
        Power::Wind,
        Power::Gold,
        Power::Shade,
        Power::Ice,
        Power::Barbarian,
    ] {
        let all: f64 = [
            Kind::Pawn,
            Kind::Knight,
            Kind::Bishop,
            Kind::Rook,
            Kind::Queen,
        ]
        .into_iter()
        .map(|k| share(k, power))
        .sum();
        assert!((all - 1.0 / 13.0).abs() < 0.03, "{power:?} {all}");
        assert!(share(Kind::Pawn, power) > share(Kind::Queen, power));
    }
}

#[test]
fn rogue_like_fire_pieces_explode_every_turn() {
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    // Огненный конь d3, рядом — чёрная ладья e4 (конь её ударом не достаёт).
    game.position.board[sq("d3") as usize] = Some(Piece {
        fire: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    game.position.board[sq("e4") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    let full = Kind::Rook.stats().max_hp;
    let lava = balance().rule(Rule::LavaDamage);
    // Огненный слон из магазина — в запасе, выставляется отдельно от обычных.
    game.position.fire_reserve[Color::White as usize][Kind::Bishop as usize] = 1;
    game.refresh();
    let fire_drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::FireDrop(Kind::Bishop) && m.to == sq("a3"))
        .unwrap();
    game.play(fire_drop).unwrap();
    assert!(game.position().piece(sq("a3")).unwrap().fire);
    assert_eq!(
        game.position().fire_reserve(Color::White)[Kind::Bishop as usize],
        0
    );
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    assert_eq!(
        turn.blasts.len(),
        1,
        "только что выставленный не взрывается"
    );
    assert_eq!(turn.blasts[0].center, sq("d3"));
    assert_eq!(hp(game.position(), "e4"), full - lava);
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("Конь d3 взорвался лавой")
    );
    // Огонь не уходит: на следующем ходу — снова взрыв; слону на a3 задеть некого — он
    // не взрывается и остаётся огненным.
    assert!(game.position().piece(sq("d3")).unwrap().fire);
    assert!(game.end_turn());
    game.play(find(game.position(), "b1", "c3")).unwrap();
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    let centers: Vec<Square> = turn.blasts.iter().map(|b| b.center).collect();
    assert!(centers.contains(&sq("d3")) && !centers.contains(&sq("a3")));
    assert!(game.position().piece(sq("a3")).unwrap().fire);
}

#[test]
fn rogue_like_fire_pieces_are_sold_and_kept_through_planning() {
    let mut game = Game::from_position(Position::rogue_start(7));
    assert!(game.win_round());
    let r = game.position.rogue.as_mut().unwrap();
    r.gold = 20;
    r.offers[0] = Some(Stock::new(Kind::Knight, Power::Fire));
    // Огненный конь: 2 золота за коня × 2 (магазин) × 2 (огонь) = 8.
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Fire)), 8);
    assert!(game.buy(0));
    assert_eq!(game.position().rogue().unwrap().gold, 12);
    assert_eq!(
        game.position().fire_reserve(Color::White)[Kind::Knight as usize],
        1
    );
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("огненный конь")
    );
    assert!(game.leave_shop());
    // Расставили: огненный конь — вместо обычного g1 (обычный ушёл с доски: место занято).
    game.position.board[sq("g1") as usize] = None;
    game.refresh();
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::FireDrop(Kind::Knight) && m.to == sq("g1"))
        .unwrap();
    game.play(drop).unwrap();
    assert!(game.end_turn(), "«Начать раунд»");
    // Уровень пройден снова — огненный конь вернулся на g1 огненным.
    assert!(game.win_round());
    assert!(game.leave_shop());
    let knight = game.position().piece(sq("g1")).unwrap();
    assert!(knight.fire && knight.kind == Kind::Knight);
}

#[test]
fn rogue_like_spark_pieces_shock_around_the_target() {
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    // Электрический конь из магазина — в запасе, выставляется отдельно от обычных.
    game.position.spark_reserve[Color::White as usize][Kind::Knight as usize] = 1;
    // Электрический конь c3 бьёт ладью d5, молния — по диагонали от неё: c6 и e6 задеты, d6 —
    // нет.
    game.position.board[sq("c3") as usize] = Some(Piece {
        spark: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    for name in ["d5", "c6", "e6", "d6"] {
        game.position.board[sq(name) as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    }
    game.refresh();
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::SparkDrop(Kind::Knight) && m.to == sq("a3"))
        .unwrap();
    game.play(drop).unwrap();
    let knight = game.position().piece(sq("a3")).unwrap();
    assert!(knight.spark && !knight.fire);
    assert_eq!(
        game.position().spark_reserve(Color::White)[Kind::Knight as usize],
        0
    );
    assert_eq!(knight.title(), "Белый электрический конь");
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    let strike = turn
        .strikes
        .iter()
        .find(|s| s.from == sq("c3") && s.hit.square == sq("d5"))
        .expect("конь бьёт d5");
    let shocked: Vec<Square> = strike.splash.iter().map(|j| j.hit.square).collect();
    assert!(shocked.contains(&sq("c6")) && shocked.contains(&sq("e6")));
    assert!(!shocked.contains(&sq("d6")));
    // Электрическая фигура продаётся и возвращается к расстановке электрической.
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Spark)), 8);
}

#[test]
fn rogue_like_shop_buys_pieces_from_the_reserve() {
    let mut game = Game::from_position(Position::rogue_start(7));
    game.position.reserve[Color::White as usize][Kind::Knight as usize][0][0] = 1;
    let knight = Stock::new(Kind::Knight, Power::Plain);
    // До магазина продавать нельзя.
    assert!(!game.sell(knight));
    assert!(game.win_round());
    game.position.rogue.as_mut().unwrap().gold = 0;
    game.position.fire_reserve[Color::White as usize][Kind::Rook as usize] = 1;
    let count = |game: &Game, stock: Stock| {
        game.position()
            .reserve(Color::White)
            .iter()
            .find(|s| s.0 == stock)
            .map_or(0, |s| s.1)
    };
    // Конь из запаса — за цену в магазине (4), огненная ладья — как обычная (6).
    assert_eq!(count(&game, knight), 1);
    assert!(game.sell(knight));
    assert_eq!(count(&game, knight), 0);
    assert_eq!(game.position().rogue().unwrap().gold, 4);
    assert!(!game.sell(knight), "в запасе коня больше нет");
    let rook = Stock::new(Kind::Rook, Power::Fire);
    assert!(game.sell(rook));
    assert_eq!(game.position().rogue().unwrap().gold, 10);
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("Продана огненная ладья из запаса за 6 монет")
    );
    // Отмена возвращает и фигуру, и золото.
    assert!(game.undo());
    assert_eq!(count(&game, rook), 1);
    assert_eq!(game.position().rogue().unwrap().gold, 4);
    // После магазина — нельзя.
    assert!(game.leave_shop());
    assert!(!game.sell(rook));
}

#[test]
fn rogue_like_wind_pieces_move_twice_for_one_move() {
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    game.position.board[sq("g1") as usize] = Some(Piece {
        wind: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    game.refresh();
    let per_turn = game.position().moves_per_turn();
    // Первый ход воздушного коня тратит ход, второй — нет; между ними ходят другие.
    game.play(find(game.position(), "g1", "f3")).unwrap();
    assert_eq!(game.position().moves_left(), per_turn - 1);
    assert!(game.position().can_move(sq("f3")));
    game.play(find(game.position(), "e2", "e4")).unwrap();
    game.play(find(game.position(), "f3", "g5")).unwrap();
    assert_eq!(game.position().moves_left(), per_turn - 2);
    assert!(!game.position().can_move(sq("g5")), "третьего хода нет");
    assert!(game.legal_moves().iter().all(|m| m.from != sq("g5")));
    // Ещё два хода — все четыре потрачены (конь — один), ход кончился сам.
    game.play(find(game.position(), "d2", "d4")).unwrap();
    game.play(find(game.position(), "b1", "c3")).unwrap();
    assert_eq!(game.position().side(), Color::Black);
}

#[test]
fn rogue_like_wind_piece_gets_a_free_second_move_after_the_last_one() {
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    game.position.wind_reserve[Color::White as usize][Kind::Knight as usize] = 1;
    game.position.board[sq("g1") as usize] = None;
    game.refresh();
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::WindDrop(Kind::Knight) && m.to == sq("g1"))
        .unwrap();
    // Выставление — обычный ход: выставленный в этот ход больше не ходит.
    game.play(drop).unwrap();
    let knight = game.position().piece(sq("g1")).unwrap();
    assert!(knight.wind);
    assert_eq!(knight.title(), "Белый воздушный конь");
    assert_eq!(
        game.position().wind_reserve(Color::White)[Kind::Knight as usize],
        0
    );
    assert!(game.end_turn());
    assert!(game.end_turn());
    // Три обычных хода, четвёртый — воздушный конь; его второй ход ещё есть, других — нет.
    for (from, to) in [("e2", "e4"), ("d2", "d4"), ("b1", "c3"), ("g1", "f3")] {
        game.play(find(game.position(), from, to)).unwrap();
    }
    assert_eq!(game.position().moves_left(), 0);
    assert_eq!(game.position().side(), Color::White);
    assert!(!game.legal_moves().is_empty());
    assert!(game.legal_moves().iter().all(|m| m.from == sq("f3")));
    game.play(find(game.position(), "f3", "g5")).unwrap();
    // Ходить больше нечем — ход кончился сам.
    assert_eq!(game.position().side(), Color::Black);
}

#[test]
fn rogue_like_gold_pieces_earn_more_gold() {
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    // Золотой конь c3 убивает пешку d5 (1 золото) — и ещё 2 сверху; обычный конь f3 убивает
    // пешку g5 — только 1.
    game.position.board[sq("c3") as usize] = Some(Piece {
        gold: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    game.position.board[sq("g1") as usize] = None;
    game.position.board[sq("f3") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    for name in ["d5", "g5"] {
        game.position.board[sq(name) as usize] = Some(Piece {
            hp: 1,
            ..Piece::new(Kind::Pawn, Color::Black)
        });
    }
    game.position.gold_reserve[Color::White as usize][Kind::Rook as usize] = 1;
    game.refresh();
    // Золотая ладья из запаса выставляется отдельно и выглядит золотой.
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::GoldDrop(Kind::Rook) && m.to == sq("a3"))
        .unwrap();
    game.play(drop).unwrap();
    let rook = game.position().piece(sq("a3")).unwrap();
    assert!(rook.gold);
    assert_eq!(rook.title(), "Белая золотая ладья");
    assert_eq!(
        game.position().gold_reserve(Color::White)[Kind::Rook as usize],
        0
    );
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    assert!(turn.strikes.iter().filter(|s| s.hit.killed).count() == 2);
    assert_eq!(turn.gold, 1 + 2 + 1);
    // В магазине — как другие особые; продаётся вдвое дороже обычной (слон — 8 вместо 4).
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Gold)), 8);
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Fire)), 8);
    assert_eq!(sell_price(Stock::new(Kind::Bishop, Power::Plain)), 4);
    assert_eq!(sell_price(Stock::new(Kind::Bishop, Power::Gold)), 8);
    assert_eq!(sell_price(Stock::new(Kind::Bishop, Power::Fire)), 4);
}

#[test]
fn rogue_like_shade_pieces_move_through_pieces() {
    let mut pos = Position::rogue_start(7);
    for sq in 16..64u8 {
        pos.board[sq as usize] = None;
    }
    let shade = |kind: Kind| {
        Some(Piece {
            shade: true,
            ..Piece::new(kind, Color::White)
        })
    };
    // Теневая ладья a1 проходит сквозь пешку a2 — по всей вертикали; вбок путь занят своими,
    // но сквозь них — тоже нельзя встать на занятую клетку, а пустых там нет.
    pos.board[sq("a1") as usize] = shade(Kind::Rook);
    let to = |pos: &Position, from: &str| -> Vec<String> {
        let mut v: Vec<String> = pos
            .legal_moves()
            .iter()
            .filter(|m| m.from == sq(from))
            .map(|m| square_name(m.to))
            .collect();
        v.sort();
        v.dedup();
        v
    };
    assert_eq!(to(&pos, "a1"), ["a3", "a4", "a5", "a6", "a7", "a8"]);
    // Теневой слон c1 — сквозь пешки b2 и d2 по диагоналям.
    pos.board[sq("c1") as usize] = shade(Kind::Bishop);
    assert_eq!(to(&pos, "c1"), ["a3", "e3", "f4", "g5", "h6"]);
    // Теневой конь g1 — на любую пустую клетку через одну: e3, f3, g3, h3 (e1 занят).
    pos.board[sq("g1") as usize] = shade(Kind::Knight);
    assert_eq!(to(&pos, "g1"), ["e3", "f3", "g3", "h3"]);
    // Теневая пешка e2: e3 и e4; перед пешкой e3 стоит фигура — пешка проходит сквозь неё.
    pos.board[sq("e2") as usize] = shade(Kind::Pawn);
    assert_eq!(to(&pos, "e2"), ["e3", "e4"]);
    pos.board[sq("e3") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    pos.board[sq("d2") as usize] = None;
    pos.board[sq("d3") as usize] = shade(Kind::Pawn);
    pos.board[sq("d4") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    assert_eq!(to(&pos, "d3"), ["d5"], "через одну фигуру перед собой");
    assert_eq!(to(&pos, "e2"), ["e4"]);
    // Обычная пешка через фигуру не ходит.
    pos.board[sq("d3") as usize] = Some(Piece::new(Kind::Pawn, Color::White));
    assert!(to(&pos, "d3").is_empty());
    // Ход теневой ладьи сквозь пешку — обычный ход.
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "a1", "a5")).unwrap();
    assert!(game.position().piece(sq("a5")).unwrap().shade);
    assert_eq!(
        game.position().piece(sq("a5")).unwrap().title(),
        "Белая теневая ладья"
    );
    // В магазине — как другие особые.
    assert_eq!(shop_price(Stock::new(Kind::Rook, Power::Shade)), 12);
    game.position.shade_reserve[Color::White as usize][Kind::Knight as usize] = 1;
    game.refresh();
    assert!(
        game.legal_moves()
            .iter()
            .any(|m| m.kind == MoveKind::ShadeDrop(Kind::Knight))
    );
    assert_eq!(
        game.position().shade_reserve(Color::White)[Kind::Knight as usize],
        1
    );
}

#[test]
fn rogue_like_ice_pieces_freeze_their_attackers() {
    let mut game = Game::from_position(Position::rogue_start(7));
    for (sq, _) in army(&game.position, Color::Black) {
        game.position.board[sq as usize] = None;
    }
    // Ледяной конь e4; чёрная пешка f5 бьёт его (и только его) — и замерзает; чёрная ладья h4
    // бьёт обычную пешку h2 — не замерзает.
    game.position.board[sq("e4") as usize] = Some(Piece {
        icy: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    game.position.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    game.position.board[sq("h4") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    game.position.ice_reserve[Color::White as usize][Kind::Pawn as usize] = 1;
    game.refresh();
    // Ледяной конь сам не замораживает: бьёт пешку f5? Нет — d6, f6, c5, g5… — не достаёт; пусть
    // ход — выставление ледяной пешки из запаса.
    game.position.board[sq("a2") as usize] = None;
    game.refresh();
    let drop = *game
        .legal_moves()
        .iter()
        .find(|m| m.kind == MoveKind::IceDrop(Kind::Pawn) && m.to == sq("a2"))
        .unwrap();
    game.play(drop).unwrap();
    assert!(game.position().piece(sq("a2")).unwrap().icy);
    assert_eq!(
        game.position().piece(sq("a2")).unwrap().title(),
        "Белая ледяная пешка"
    );
    assert_eq!(
        game.position().ice_reserve(Color::White)[Kind::Pawn as usize],
        0
    );
    assert!(game.end_turn());
    // Ход чёрных: пешка f5 бьёт ледяного коня e4 — замерзает и не шагает; ладья h4 бьёт h2 и
    // шагает.
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    let hit = turn
        .strikes
        .iter()
        .find(|s| s.from == sq("f5"))
        .expect("пешка бьёт коня");
    assert!(hit.rebound);
    assert!(
        turn.strikes
            .iter()
            .filter(|s| s.from == sq("h4"))
            .all(|s| !s.rebound)
    );
    assert!(game.position().piece(sq("f5")).unwrap().is_frozen());
    assert!(!turn.marched.iter().any(|&(from, _)| from == sq("f5")));
    assert!(turn.marched.contains(&(sq("h4"), sq("h3"))));
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("замёрзла о лёд")
    );
    // Следующий ход чёрных замёрзшая пешка не бьёт и не шагает, потом оттаивает.
    game.play(find(game.position(), "b1", "c3")).unwrap();
    assert!(game.end_turn());
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    assert!(turn.strikes.iter().all(|s| s.from != sq("f5")));
    assert!(!turn.marched.iter().any(|&(from, _)| from == sq("f5")));
    assert!(!game.position().piece(sq("f5")).unwrap().is_frozen());
    // В магазине — как другие особые.
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Ice)), 8);
}

#[test]
fn rogue_like_fat_king_leads_the_last_wave_of_the_first_level() {
    let mut pos = Position::rogue_start(7);
    let bosses = |pos: &Position| {
        pos.pieces()
            .filter(|(_, p)| p.boss)
            .map(|(sq, _)| sq)
            .collect::<Vec<_>>()
    };
    assert!(bosses(&pos).is_empty());
    // Предпоследняя волна — без босса, последняя — с ним, на крайней горизонтали.
    let mut r = pos.rogue.unwrap();
    r.wave = r.waves - 2;
    pos.rogue = Some(r);
    for sq in 48..64u8 {
        pos.board[sq as usize] = None;
    }
    pos.spawn_wave();
    assert!(bosses(&pos).is_empty());
    for sq in 48..64u8 {
        pos.board[sq as usize] = None;
    }
    let spawned = pos.spawn_wave();
    let boss = bosses(&pos);
    assert_eq!(boss.len(), 1);
    assert_eq!(boss[0], sq("e8"), "середина крайней горизонтали");
    assert!(spawned.contains(&(sq("e8"), Kind::King)));
    let king = pos.piece(sq("e8")).unwrap();
    assert_eq!((king.hp, king.max_hp(), king.attack()), (3000, 3000, 300));
    assert_eq!(king.title(), "Чёрный король-толстосум");
    assert_eq!(pos.king_square(Color::Black), None, "не король стороны");
    // На втором уровне босса нет.
    let mut pos2 = Position::rogue_start(7);
    let mut r = pos2.rogue.unwrap();
    r.level = 2;
    r.wave = r.waves - 1;
    pos2.rogue = Some(r);
    for sq in 48..64u8 {
        pos2.board[sq as usize] = None;
    }
    pos2.spawn_wave();
    assert!(bosses(&pos2).is_empty());

    // Бьёт 16 клеток: 8 вокруг и через клетку по диагоналям и прямым.
    let mut pos = Position::rogue_start(7);
    for sq in 16..64u8 {
        pos.board[sq as usize] = None;
    }
    pos.board[sq("e5") as usize] = Some(Piece {
        boss: true,
        hp: 3000,
        ..Piece::new(Kind::King, Color::Black)
    });
    let targets = [
        "d6", "e6", "f6", "d5", "f5", "d4", "e4", "f4", "c7", "g7", "c3", "g3", "e7", "e3", "c5",
        "g5",
    ];
    // Белые ладьи заморожены — сами не бьют; b5 и h8 — дальше 16 клеток.
    for name in targets.iter().chain(&["b5", "h8"]) {
        pos.board[sq(name) as usize] = Some(Piece {
            frozen: 5,
            ..Piece::new(Kind::Rook, Color::White)
        });
    }
    pos.end_turn();
    let turn = pos.end_turn();
    let mut hit: Vec<String> = turn
        .strikes
        .iter()
        .filter(|s| s.from == sq("e5"))
        .map(|s| square_name(s.hit.square))
        .collect();
    hit.sort();
    let mut want: Vec<String> = targets.iter().map(|s| s.to_string()).collect();
    want.sort();
    assert_eq!(hit, want);
    assert!(
        turn.strikes
            .iter()
            .filter(|s| s.from == sq("e5"))
            .all(|s| s.hit.damage == 300)
    );
    // Победа над ним — 10 золота.
    let mut pos = Position::rogue_start(7);
    pos.score_kill(king, Color::White);
    assert_eq!(pos.rogue().unwrap().gold, 10);
}

#[test]
fn rogue_like_win_round_on_the_last_level_wins_the_game() {
    let mut game = Game::from_position(Position::rogue_start(7));
    let mut r = game.position.rogue.unwrap();
    r.level = r.levels;
    game.position.rogue = Some(r);
    assert!(game.win_round());
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::White
        }
    );
    assert!(!game.position().shop_open());
}

#[test]
fn rogue_like_pawns_get_rarer_with_each_level() {
    let pawn_share = |level: u8| {
        let (mut pawns, mut all) = (0, 0);
        for seed in 0..300 {
            let mut pos = Position::rogue_start(seed);
            let mut r = pos.rogue.unwrap();
            r.level = level;
            pos.rogue = Some(r);
            for sq in 56..64u8 {
                pos.board[sq as usize] = None;
            }
            for (_, kind) in pos.spawn_wave() {
                all += 1;
                pawns += usize::from(kind == Kind::Pawn);
            }
        }
        pawns as f64 / all as f64
    };
    let (first, fifth, last) = (pawn_share(1), pawn_share(5), pawn_share(10));
    assert!(first > fifth && fifth > last, "{first} {fifth} {last}");
    assert!(first > 0.45, "на 1-м уровне вес пешки 15 из 29: {first}");
    assert_eq!(last, 0.0, "с 9-го уровня пешек нет");
}

#[test]
fn rogue_like_blocked_wave_comes_out_on_the_nearest_free_rank() {
    let blocked = |ranks: std::ops::Range<i8>| {
        let mut pos = Position::rogue_start(9);
        for sq in 16..64u8 {
            pos.board[sq as usize] = None;
        }
        for rank in ranks {
            for file in 0..8 {
                pos.board[square(file, rank).unwrap() as usize] =
                    Some(Piece::new(Kind::Knight, Color::White));
            }
        }
        pos
    };
    // Восьмая занята фигурами игрока — волна выходит на седьмую, занята и она — на шестую.
    for (ranks, rank) in [(7..8, 6), (6..8, 5)] {
        let mut pos = blocked(ranks);
        let spawned = pos.spawn_wave();
        assert!((2..=4).contains(&spawned.len()), "{spawned:?}");
        assert!(
            spawned.iter().all(|&(sq, _)| rank_of(sq) == rank),
            "{spawned:?}"
        );
        assert_eq!(pos.rogue().unwrap().wave, 2);
    }
    // На восьмой одна свободная клетка — первая фигура встаёт туда, остальные — на седьмую.
    let mut pos = blocked(7..8);
    pos.board[sq("c8") as usize] = None;
    let spawned = pos.spawn_wave();
    assert!(spawned.iter().any(|&(at, _)| at == sq("c8")), "{spawned:?}");
    assert!(
        spawned
            .iter()
            .all(|&(at, _)| at == sq("c8") || rank_of(at) == 6),
        "{spawned:?}"
    );
    // На первую горизонталь игрока волна не выходит: места нет — волна ждёт.
    let mut pos = blocked(1..8);
    assert!(pos.spawn_wave().is_empty());
    assert_eq!(pos.rogue().unwrap().wave, 1);
}

#[test]
fn rogue_like_knights_do_not_intercept() {
    let mut pos = Position::rogue_start(5);
    for sq in 16..64u8 {
        pos.board[sq as usize] = None;
    }
    pos.board[sq("f6") as usize] = Some(Piece::new(Kind::Knight, Color::Black));
    pos.board[sq("d4") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    // Конь f6 достаёт e4, но не бьёт; пешка d4 бьёт на проходе, как в обычной партии.
    let outcome = pos.make_move(find(&pos, "e2", "e4"));
    assert_eq!(outcome.reactions.len(), 1);
    assert_eq!(outcome.reactions[0].from, sq("d4"));
    let mut pos = Position::rogue_start(5);
    for sq in 16..64u8 {
        pos.board[sq as usize] = None;
    }
    pos.board[sq("f6") as usize] = Some(Piece::new(Kind::Knight, Color::Black));
    assert!(pos.make_move(find(&pos, "e2", "e4")).reactions.is_empty());
    assert!(pos.make_move(find(&pos, "g1", "h3")).reactions.is_empty());
    let pawn = pos.piece(sq("e4")).unwrap();
    assert_eq!(pawn.hp, pawn.max_hp(), "конь волны не перехватил");
}

#[test]
fn rogue_like_waves_keep_coming_while_the_last_ones_live() {
    let mut game = Game::from_position(Position::rogue_start(3));
    for wave in 1..=3 {
        assert_eq!(game.position().rogue().unwrap().wave, wave);
        let mv = game.legal_moves()[0];
        game.play(mv).unwrap();
        if game.position().side() == Color::White {
            assert!(game.end_turn());
        }
        if game.is_over() {
            return;
        }
        assert!(game.end_turn());
    }
    assert_eq!(
        game.position().rogue().unwrap().wave,
        4,
        "волна — каждый ход"
    );
}

#[test]
fn rogue_like_random_games_keep_the_rules() {
    let mut seed = 0x0B0B_5EED_1234_5678u64;
    let mut next = move |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n as u64) as usize
    };
    let (mut wins, mut losses) = (0, 0);
    for g in 0..40 {
        let mut game = Game::from_position(Position::rogue_start(g * 7 + 1));
        for _ in 0..600 {
            if game.is_over() {
                break;
            }
            let moves = game.legal_moves().to_vec();
            let pos = game.position();
            if pos.planning() {
                // Расстановка: фигуры целы, на своей половине, пешки — на второй горизонтали.
                for (sq, p) in pos.pieces().filter(|(_, p)| p.color == Color::White) {
                    assert_eq!(p.hp, p.max_hp());
                    if p.kind == Kind::Pawn {
                        assert_eq!(rank_of(sq), 1);
                    } else {
                        assert!(rank_of(sq) < 2);
                    }
                }
                assert!(pos.pieces().all(|(_, p)| p.color == Color::White));
            }
            if pos.shop_open() {
                assert!(moves.is_empty() && !game.can_end_turn());
                match next(4) {
                    0 => {
                        game.reroll();
                    }
                    1 => {
                        game.leave_shop();
                    }
                    _ => {
                        game.buy(next(3));
                    }
                }
                continue;
            }
            if moves.is_empty() || (pos.moved() || pos.planning()) && next(3) == 0 {
                assert!(game.end_turn());
                continue;
            }
            let mv = moves[next(moves.len())];
            assert!(!game.position().used(mv.from));
            game.play(mv).unwrap();
            let pos = game.position();
            assert!(
                pos.king_square(Color::Black).is_none(),
                "босс — не король стороны"
            );
            let r = pos.rogue().unwrap();
            assert!(r.wave <= r.waves);
        }
        match game.status() {
            Status::Victory {
                winner: Color::White,
            } => {
                wins += 1;
                let r = game.position().rogue().unwrap();
                assert_eq!(r.wave, r.waves);
            }
            Status::Victory { .. } => losses += 1,
            _ => {}
        }
    }
    eprintln!("rogue-like: побед {wins}, поражений {losses}");
    assert!(wins + losses > 0);
}

#[test]
fn rogue_like_waves_step_forward_and_free_the_back_ranks() {
    // Пешка g6 упирается в белого коня g5; ладья g8 и пешка g7 за ней — тоже стоят; конь b8
    // и пешка e7 шагают вперёд.
    let mut pos = Position::rogue_start(5);
    for sq in 32..64u8 {
        pos.board[sq as usize] = None;
    }
    for (name, piece) in [
        ("g5", Piece::new(Kind::Knight, Color::White)),
        ("g6", Piece::new(Kind::Pawn, Color::Black)),
        ("g7", Piece::new(Kind::Pawn, Color::Black)),
        ("g8", Piece::new(Kind::Rook, Color::Black)),
        ("b8", Piece::new(Kind::Knight, Color::Black)),
        ("e7", Piece::new(Kind::Pawn, Color::Black)),
    ] {
        pos.board[sq(name) as usize] = Some(piece);
    }
    pos.end_turn();
    assert!(pos.legal_moves().is_empty(), "фигуры волн сами не ходят");
    let turn = pos.end_turn();
    let mut steps: Vec<String> = turn
        .marched
        .iter()
        .map(|&(f, t)| format!("{}-{}", square_name(f), square_name(t)))
        .collect();
    steps.sort();
    assert_eq!(steps, ["b8-b7", "e7-e6"]);
    assert_eq!(pos.piece(sq("b7")).unwrap().kind, Kind::Knight);
    assert_eq!(
        pos.piece(sq("g8")).unwrap().kind,
        Kind::Rook,
        "колонна стоит за пешкой"
    );
    // Колонна шагает целиком, когда впереди пусто.
    pos.board[sq("g5") as usize] = None;
    pos.end_turn();
    let turn = pos.end_turn();
    assert!(turn.marched.contains(&(sq("g6"), sq("g5"))));
    assert!(turn.marched.contains(&(sq("g8"), sq("g7"))));
}

#[test]
fn rogue_like_waves_strike_before_they_step() {
    // Чёрная пешка e4 бьёт коня f3 со своей клетки и только потом шагает на e3 (иначе с e3
    // она ударила бы пешку f2).
    let mut pos = Position::rogue_start(5);
    for sq in 16..64u8 {
        pos.board[sq as usize] = None;
    }
    pos.board[sq("e4") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    pos.board[sq("f3") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    pos.end_turn();
    let turn = pos.end_turn();
    assert_eq!(turn.strikes.len(), 1);
    assert_eq!(turn.strikes[0].from, sq("e4"));
    assert_eq!(turn.strikes[0].hit.square, sq("f3"));
    assert!(turn.marched.contains(&(sq("e4"), sq("e3"))));
    // Новая волна выходит после ударов — её фигуры в этот ход не били.
    assert!(
        turn.spawned
            .iter()
            .all(|&(s, _)| turn.strikes.iter().all(|st| st.from != s))
    );
}

#[test]
fn rogue_like_is_lost_when_a_wave_reaches_the_first_rank() {
    let mut pos = Position::rogue_start(5);
    for sq in 16..64u8 {
        pos.board[sq as usize] = None;
    }
    pos.board[sq("a1") as usize] = None;
    pos.board[sq("a2") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    let mut game = Game::from_position(pos);
    assert!(!game.position().breached());
    game.play(find(game.position(), "h2", "h3")).unwrap();
    assert!(game.end_turn());
    assert!(game.legal_moves().is_empty());
    assert!(game.end_turn());
    assert!(game.position().breached());
    assert_eq!(
        game.status(),
        Status::Victory {
            winner: Color::Black
        }
    );
    let record = game.history().last().unwrap();
    assert!(record.description().contains("дошла до первой горизонтали"));
}

/// Rogue-like позиция без фигур волн, с гербами `crests`; ходит `side`.
fn crest_position(crests: &[Crest], side: Color) -> Position {
    let mut pos = Position::rogue_start(7);
    for (sq, _) in army(&pos, Color::Black) {
        pos.board[sq as usize] = None;
    }
    let r = pos.rogue.as_mut().unwrap();
    for (slot, &crest) in r.crests.iter_mut().zip(crests) {
        *slot = Some(crest);
    }
    pos.side = side;
    pos
}

fn ice(kind: Kind) -> Piece {
    Piece {
        icy: true,
        ..Piece::new(kind, Color::White)
    }
}

#[test]
fn rogue_like_crests_are_bought_in_the_shop() {
    let mut game = Game::from_position(Position::rogue_start(7));
    assert!(game.position().crest_offers().iter().all(Option::is_none));
    assert!(game.win_round());
    let offers: Vec<Crest> = game
        .position()
        .crest_offers()
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(offers.len(), 3, "три герба на выбор");
    assert!(offers[0] != offers[1] && offers[1] != offers[2] && offers[0] != offers[2]);

    game.position.rogue.as_mut().unwrap().gold = 40;
    let crest = offers[1];
    assert!(game.buy_crest(1));
    assert_eq!(game.position().rogue().unwrap().gold, 25, "герб — 15");
    assert_eq!(game.position().crests(), vec![crest]);
    assert!(game.position().has_crest(crest));
    assert_eq!(game.position().crest_offers()[1], None);
    assert!(!game.buy_crest(1), "место пусто");
    let text = game.history().last().unwrap().description();
    assert!(text.contains(&crest.description()), "{text}");
    // Отмена возвращает золото и герб на прилавок.
    assert!(game.undo());
    assert_eq!(game.position().rogue().unwrap().gold, 40);
    assert!(game.position().crests().is_empty());
    assert!(game.buy_crest(1));

    // Обновить — за 5; купленный герб на прилавок больше не выходит.
    assert!(game.reroll_crests());
    assert_eq!(game.position().rogue().unwrap().gold, 20);
    let fresh: Vec<Crest> = game
        .position()
        .crest_offers()
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(fresh.len(), 3);
    assert!(!fresh.contains(&crest));
    game.position.rogue.as_mut().unwrap().gold = 4;
    assert!(
        !game.reroll_crests() && !game.buy_crest(0),
        "не хватает золота"
    );

    // Гербы — до конца забега: после ухода из магазина и следующей расстановки они на месте.
    assert!(game.leave_shop());
    assert!(game.end_turn());
    assert!(game.win_round());
    assert_eq!(game.position().crests(), vec![crest]);

    // Не купленным остался один герб — на прилавке только он.
    let mut pos = crest_position(&Crest::ALL[..CRESTS - 1], Color::White);
    pos.roll_crests();
    assert_eq!(
        pos.crest_offers().into_iter().flatten().collect::<Vec<_>>(),
        vec![Crest::WindDark]
    );
}

#[test]
fn rogue_like_heart_crest_gives_ice_pieces_more_health() {
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(ice(Kind::Knight));
    pos.board[sq("d4") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    pos.rogue.as_mut().unwrap().origin = pos.origins(Color::White);
    let full = Kind::Knight.stats().max_hp;
    let bonus = 2 * crate::balance::UNIT;
    // Купленный герб — сразу и ледяным фигурам на доске.
    let r = pos.rogue.as_mut().unwrap();
    r.shop = true;
    r.gold = 15;
    r.crest_offers[0] = Some(Crest::Heart);
    assert!(pos.buy_crest(0).is_some());
    let knight = pos.piece(sq("e4")).unwrap();
    assert_eq!((knight.hp, knight.max_hp()), (full + bonus, full + bonus));
    assert_eq!(pos.piece(sq("d4")).unwrap().max_hp(), full);
    // Выставленная из запаса — тоже.
    pos.ice_reserve[Color::White as usize][Kind::Pawn as usize] = 1;
    let pawn = pos.take_stock(Color::White, Stock::new(Kind::Pawn, Power::Ice));
    assert_eq!(pawn.max_hp(), Kind::Pawn.stats().max_hp + bonus);
    // К расстановке — снова с полным здоровьем и прибавкой.
    pos.wound(sq("e4"), 300);
    pos.begin_planning();
    let knight = pos.piece(sq("e4")).unwrap();
    assert_eq!((knight.hp, knight.max_hp()), (full + bonus, full + bonus));
}

#[test]
fn rogue_like_coin_crest_doubles_ice_pieces_in_the_shop() {
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut ice, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Ice {
                    ice += 1.0;
                }
            }
        }
        ice / all
    };
    let plain = share(&[]);
    let coin = share(&[Crest::Coin]);
    // Без герба ледяных — 1/13, с гербом их вес вдвое больше: 2/14.
    assert!((plain - 1.0 / 13.0).abs() < 0.02, "{plain}");
    assert!((coin - 2.0 / 14.0).abs() < 0.03, "{coin}");
}

#[test]
fn rogue_like_pawn_crest_prices_ice_pawns_as_plain() {
    let ice_pawn = Stock::new(Kind::Pawn, Power::Ice);
    let plain = shop_price(Stock::new(Kind::Pawn, Power::Plain));
    let pos = crest_position(&[], Color::White);
    assert_eq!(pos.shop_price(ice_pawn), shop_price(ice_pawn));
    assert!(shop_price(ice_pawn) > plain);
    let pos = crest_position(&[Crest::Pawn], Color::White);
    assert_eq!(pos.shop_price(ice_pawn), plain);
    assert_eq!(
        pos.shop_price(Stock::new(Kind::Knight, Power::Ice)),
        shop_price(Stock::new(Kind::Knight, Power::Ice)),
        "только пешки"
    );
}

#[test]
fn rogue_like_snow_crest_freezes_attackers_for_two_turns() {
    for (crests, frozen) in [(&[][..], 3), (&[Crest::Snow2][..], 5)] {
        let mut pos = crest_position(crests, Color::Black);
        pos.board[sq("e4") as usize] = Some(ice(Kind::Knight));
        pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
        let strikes = pos.strike_all();
        assert!(strikes[0].rebound);
        assert_eq!(pos.piece(sq("f5")).unwrap().frozen, frozen);
    }
}

#[test]
fn rogue_like_spikes_crest_returns_damage() {
    let mut pos = crest_position(&[Crest::Spikes], Color::Black);
    pos.board[sq("e4") as usize] = Some(ice(Kind::Knight));
    pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    let attack = Kind::Pawn.stats().attack;
    let strikes = pos.strike_all();
    let thorns = strikes[0].thorns.expect("шипы");
    assert_eq!(
        (thorns.square, thorns.damage, thorns.killed),
        (sq("f5"), attack, false)
    );
    let pawn = pos.piece(sq("f5")).unwrap();
    assert_eq!(pawn.hp, Kind::Pawn.stats().max_hp - attack);
    assert!(pawn.is_frozen(), "и замёрзла о лёд");

    // Король-толстосум погибает о шипы — золото игроку, больше он не бьёт.
    let mut pos = crest_position(&[Crest::Spikes], Color::Black);
    pos.board[sq("e4") as usize] = Some(ice(Kind::Knight));
    pos.board[sq("c4") as usize] = Some(Piece::new(Kind::Pawn, Color::White));
    pos.board[sq("d5") as usize] = Some(Piece {
        boss: true,
        hp: 300,
        ..Piece::new(Kind::King, Color::Black)
    });
    let strikes = pos.strike_all();
    assert_eq!(strikes.len(), 1, "погибший больше не бьёт");
    assert_eq!(strikes[0].hit.square, sq("e4"));
    assert!(strikes[0].thorns.unwrap().killed);
    assert_eq!(pos.piece(sq("d5")), None);
    assert_eq!(pos.rogue().unwrap().gold, 10);
    assert!(!strikes[0].rebound);
    let text = strike_description(&strikes[0]);
    assert!(text.contains("Шипы льда"), "{text}");

    // Без герба шипов нет.
    let mut pos = crest_position(&[], Color::Black);
    pos.board[sq("e4") as usize] = Some(ice(Kind::Knight));
    pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    assert_eq!(pos.strike_all()[0].thorns, None);
}

#[test]
fn rogue_like_crystal_crest_encases_ice_pieces_at_one_health() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[Crest::Crystal], Color::Black);
    // Удар оставил бы коню половину единицы — остаётся ровно одна, конь вмерзает в лёд; удар
    // второй пешки по нему во льду урона не наносит.
    pos.board[sq("e4") as usize] = Some(Piece {
        hp: unit + unit / 2,
        ..ice(Kind::Knight)
    });
    pos.board[sq("d5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    let strikes = pos.strike_all();
    assert_eq!(strikes.len(), 2);
    assert_eq!(strikes[0].from, sq("d5"));
    assert_eq!(strikes[0].hit.damage, unit / 2);
    assert!(strikes[0].chilled && !strikes[0].hit.killed);
    assert_eq!(strikes[1].hit.damage, 0);
    let knight = pos.piece(sq("e4")).unwrap();
    assert_eq!(knight.hp, unit);
    assert_eq!(knight.frozen, 1, "оттает в конце этого хода");
    // По коню с одной единицей удар смертелен.
    let mut pos = crest_position(&[Crest::Crystal], Color::Black);
    pos.board[sq("e4") as usize] = Some(Piece {
        hp: unit,
        ..ice(Kind::Knight)
    });
    pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    assert!(pos.strike_all()[0].hit.killed);
}

#[test]
fn rogue_like_encased_ice_piece_moves_on_its_next_turn() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[Crest::Crystal], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        hp: unit + unit / 2,
        ..ice(Kind::Knight)
    });
    pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "b1", "c3")).unwrap();
    assert!(game.end_turn());
    // Ход чёрных: пешка f5 бьёт коня — он вмерзает, но к ходу белых оттаивает и может ходить.
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    assert!(
        turn.strikes
            .iter()
            .any(|s| s.hit.square == sq("e4") && s.chilled)
    );
    let knight = game.position().piece(sq("e4")).unwrap();
    assert_eq!(knight.hp, unit);
    assert!(!knight.is_frozen());
    assert!(game.legal_moves().iter().any(|m| m.from == sq("e4")));
}

#[test]
fn rogue_like_knights_have_their_own_health() {
    let pos = Position::rogue_start(7);
    assert_eq!(pos.kind_max_hp(Kind::Knight), rogue_knight_hp());
    assert_eq!(rogue_knight_hp(), 3 * crate::balance::UNIT);
    let knight = pos.piece(sq("b1")).unwrap();
    assert_eq!(
        (knight.hp, knight.max_hp()),
        (rogue_knight_hp(), rogue_knight_hp())
    );
    assert_eq!(
        Position::start().kind_max_hp(Kind::Knight),
        Kind::Knight.stats().max_hp
    );
}

fn venom_pawn() -> Piece {
    Piece {
        venom: true,
        ..Piece::new(Kind::Pawn, Color::Black)
    }
}

#[test]
fn rogue_like_second_level_waves_are_half_poisonous() {
    let count = |level: u8| {
        let mut pos = Position::rogue_start(11);
        let (mut venom, mut all) = (0, 0);
        for i in 0..200u64 {
            for sq in 0..64 {
                if pos.board[sq].is_some_and(|p| p.color == Color::Black) {
                    pos.board[sq] = None;
                }
            }
            let r = pos.rogue.as_mut().unwrap();
            r.level = level;
            r.wave = 0;
            r.waves = 9;
            r.seed = 2 * i + 1;
            for (sq, _) in pos.spawn_wave() {
                all += 1;
                if pos.piece(sq).unwrap().venom {
                    venom += 1;
                }
            }
        }
        (venom as f64 / all as f64, all)
    };
    let (share, all) = count(2);
    assert!(all > 200);
    assert!((share - 0.5).abs() < 0.08, "{share}");
    assert_eq!(count(1).0, 0.0);
    assert_eq!(count(3).0, 0.0);
    assert_eq!(venom_pawn().title(), "Чёрная ядовитая пешка");
}

#[test]
fn rogue_like_poison_hurts_at_the_end_of_the_next_turn() {
    let unit = crate::balance::UNIT;
    // Конь покрепче: два удара и яд он переживёт.
    let full = Kind::Knight.stats().max_hp + 3 * unit;
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        hp: full,
        bonus_hp: 3 * unit as i16,
        ..Piece::new(Kind::Knight, Color::White)
    });
    // Две ядовитые пешки бьют коня — яд не копится.
    pos.board[sq("d5") as usize] = Some(venom_pawn());
    pos.board[sq("f5") as usize] = Some(venom_pawn());
    let mut game = Game::from_position(pos);
    game.play(find(game.position(), "b1", "c3")).unwrap();
    assert!(game.end_turn());
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    let poisoned: Vec<bool> = turn
        .strikes
        .iter()
        .filter(|s| s.hit.square == sq("e4"))
        .map(|s| s.poisoned)
        .collect();
    assert_eq!(poisoned, vec![true, false]);
    assert!(
        game.history()
            .last()
            .unwrap()
            .description()
            .contains("отравлен")
    );
    let knight = game.position().piece(sq("e4")).unwrap();
    assert_eq!(
        (knight.hp, knight.poison),
        (full - 2 * Kind::Pawn.stats().attack, 1)
    );
    // Ход белых: в его конце — урон яда, яд уходит.
    game.play(find(game.position(), "g1", "f3")).unwrap();
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    assert_eq!(turn.poison.len(), 1);
    assert_eq!(
        (turn.poison[0].square, turn.poison[0].damage),
        (sq("e4"), unit)
    );
    let knight = game.position().piece(sq("e4")).unwrap();
    assert_eq!(
        (knight.hp, knight.poison),
        (full - 2 * Kind::Pawn.stats().attack - unit, 0)
    );
    assert!(game.history().last().unwrap().description().contains("Яд:"));

    // Яд может убить.
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        hp: unit,
        poison: 1,
        ..Piece::new(Kind::Knight, Color::White)
    });
    let turn = pos.end_turn();
    assert!(turn.poison[0].killed);
    assert_eq!(pos.piece(sq("e4")), None);
}

#[test]
fn rogue_like_barbarians_strike_again_after_a_kill() {
    let unit = crate::balance::UNIT;
    let strikes = |barbarian: bool, f6_hp: i32| {
        let mut pos = crest_position(&[], Color::White);
        pos.board[sq("e4") as usize] = Some(Piece {
            barbarian,
            ..Piece::new(Kind::Knight, Color::White)
        });
        for (name, hp) in [("d6", unit), ("f6", f6_hp)] {
            pos.board[sq(name) as usize] = Some(Piece {
                hp,
                ..Piece::new(Kind::Pawn, Color::Black)
            });
        }
        let strikes: Vec<(Square, bool, bool)> = pos
            .strike_all()
            .iter()
            .filter(|s| s.from == sq("e4"))
            .map(|s| (s.hit.square, s.hit.killed, s.again))
            .collect();
        strikes
    };
    // Обычный конь бьёт каждую по разу.
    assert_eq!(
        strikes(false, 3 * unit),
        vec![(sq("d6"), true, false), (sq("f6"), false, false)]
    );
    // Варвар убил d6 — бьёт ещё раз; второй удар не убил — хватит.
    assert_eq!(
        strikes(true, 3 * unit),
        vec![
            (sq("d6"), true, false),
            (sq("f6"), false, false),
            (sq("f6"), false, true)
        ]
    );
    // Второй удар убил — третий раунд, но бить больше некого.
    assert_eq!(
        strikes(true, 2 * unit),
        vec![
            (sq("d6"), true, false),
            (sq("f6"), false, false),
            (sq("f6"), true, true)
        ]
    );
    // Магазин, запас и вид.
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Barbarian)), 8);
    let knight = Stock::new(Kind::Knight, Power::Barbarian).piece(Color::White);
    assert!(knight.barbarian);
    assert_eq!(knight.title(), "Белый варварский конь");
    assert_eq!(
        Stock::of(knight).drop_kind(),
        MoveKind::BarbarianDrop(Kind::Knight)
    );
}

/// Варвар вида `kind` у белых с гербами `crests` (по правилам rogue-like: `crested`).
fn barbarian(pos: &Position, kind: Kind) -> Piece {
    pos.crested(Piece {
        barbarian: true,
        ..Piece::new(kind, Color::White)
    })
}

/// Чёрная пешка с `hp` здоровья.
fn pawn_with(hp: i32) -> Piece {
    Piece {
        hp,
        ..Piece::new(Kind::Pawn, Color::Black)
    }
}

#[test]
fn rogue_like_axe_sword_crest_trades_health_for_damage() {
    let unit = crate::balance::UNIT;
    let pos = crest_position(&[Crest::AxeSword], Color::White);
    let knight = barbarian(&pos, Kind::Knight);
    let plain = Piece::new(Kind::Knight, Color::White);
    assert_eq!(knight.attack(), plain.attack() + 2 * unit);
    assert_eq!(knight.max_hp(), rogue_knight_hp() - unit);
    assert_eq!(knight.hp, knight.max_hp());
    // Не варвара герб не трогает.
    assert_eq!(pos.crested(plain).attack(), plain.attack());
}

#[test]
fn rogue_like_axe_fangs_crest_heals_once_per_turn() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[Crest::AxeFangs], Color::White);
    let knight = Piece {
        hp: unit,
        ..barbarian(&pos, Kind::Knight)
    };
    pos.board[sq("e4") as usize] = Some(knight);
    // Убивает обеих за первый удар; лечится один раз.
    pos.board[sq("d6") as usize] = Some(pawn_with(unit));
    pos.board[sq("f6") as usize] = Some(pawn_with(unit));
    let strikes = pos.strike_all();
    let feast: i32 = strikes.iter().map(|s| s.feast).sum();
    assert_eq!(feast, unit);
    assert_eq!(pos.piece(sq("e4")).unwrap().hp, 2 * unit);
    let text = strike_description(strikes.iter().find(|s| s.feast > 0).unwrap());
    assert!(text.contains("восстановил"), "{text}");
}

#[test]
fn rogue_like_axe_queen_and_axes_crests_help_buy_barbarians() {
    let queen = Stock::new(Kind::Queen, Power::Barbarian);
    let pos = crest_position(&[Crest::AxeQueen], Color::White);
    assert_eq!(pos.shop_price(queen), shop_price(queen) - 6);
    assert_eq!(
        pos.shop_price(Stock::new(Kind::Rook, Power::Barbarian)),
        shop_price(Stock::new(Kind::Rook, Power::Barbarian))
    );
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Barbarian {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    assert!((share(&[]) - 1.0 / 13.0).abs() < 0.02);
    assert!((share(&[Crest::Axes]) - 2.0 / 14.0).abs() < 0.03);
}

#[test]
fn rogue_like_whirl_crest_turns_the_second_attack_into_a_circle() {
    let unit = crate::balance::UNIT;
    // С гербом топора и меча варвар бьёт на 3, а круговой удар — по 1.
    let mut pos = crest_position(&[Crest::Whirl, Crest::AxeSword], Color::White);
    pos.board[sq("e4") as usize] = Some(barbarian(&pos, Kind::Knight));
    // d6 конь убивает; круговой удар — по всем в двух клетках: e5 и c6 (конём их не достать),
    // h4 — в трёх, его нет.
    pos.board[sq("d6") as usize] = Some(pawn_with(unit));
    pos.board[sq("e5") as usize] = Some(pawn_with(3 * unit));
    pos.board[sq("c6") as usize] = Some(pawn_with(3 * unit));
    pos.board[sq("h4") as usize] = Some(pawn_with(3 * unit));
    let strikes: Vec<(Square, bool)> = pos
        .strike_all()
        .iter()
        .filter(|s| s.from == sq("e4"))
        .map(|s| (s.hit.square, s.whirl))
        .collect();
    assert_eq!(strikes[0], (sq("d6"), false));
    let mut whirl: Vec<Square> = strikes[1..].iter().map(|s| s.0).collect();
    whirl.sort();
    let mut expect = vec![sq("e5"), sq("c6")];
    expect.sort();
    assert_eq!(whirl, expect);
    assert!(strikes[1..].iter().all(|s| s.1));
    assert_eq!(hp(&pos, "h4"), 3 * unit);
    assert_eq!(hp(&pos, "e5"), 2 * unit);
    assert_eq!(hp(&pos, "c6"), 2 * unit);
}

#[test]
fn rogue_like_fire_axe_crest_repeats_once_more_at_one_health() {
    let unit = crate::balance::UNIT;
    // Конь-варвар с одной единицей здоровья: d6, потом f6, потом c5 — каждая убита с одного удара.
    let rounds = |crests: &[Crest], hp: i32| {
        let mut pos = crest_position(crests, Color::White);
        pos.board[sq("e4") as usize] = Some(Piece {
            hp,
            ..barbarian(&pos, Kind::Knight)
        });
        pos.board[sq("d6") as usize] = Some(pawn_with(unit));
        pos.board[sq("f6") as usize] = Some(pawn_with(2 * unit));
        pos.board[sq("c5") as usize] = Some(pawn_with(3 * unit));
        pos.strike_all()
            .iter()
            .filter(|s| s.from == sq("e4"))
            .count()
    };
    // Раунд 1: d6 убита, f6 и c5 ранены; раунд 2: f6 убита, c5 ранена; раунд 3 — только с гербом.
    assert_eq!(rounds(&[], unit), 5);
    assert_eq!(rounds(&[Crest::FireAxe], 3 * unit), 5, "здоровья много");
    assert_eq!(rounds(&[Crest::FireAxe], unit), 6);
}

#[test]
fn rogue_like_wizards_cast_meteors_on_enemy_pieces() {
    let unit = crate::balance::UNIT;
    let wizard = Piece {
        wizard: true,
        ..Piece::new(Kind::Knight, Color::White)
    };
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(wizard);
    pos.board[sq("h8") as usize] = Some(pawn_with(3 * unit));
    pos.board[sq("a7") as usize] = Some(pawn_with(unit));
    let before = pos.clone();
    let turn = pos.end_turn();
    assert_eq!(turn.meteors.len(), 1, "один маг — один метеорит");
    let (from, hit) = (turn.meteors[0].from, turn.meteors[0].hits[0]);
    assert_eq!(from, sq("e4"));
    assert!([sq("h8"), sq("a7")].contains(&hit.square));
    assert_eq!(hit.damage, unit);
    // Та же позиция — тот же метеорит.
    let mut again = before.clone();
    assert_eq!(again.end_turn().meteors, turn.meteors);
    let record = Record {
        before,
        event: Event::EndTurn(Color::White, turn.clone()),
        check: false,
        loser: None,
    };
    assert!(record.description().contains("Метеорит мага с e4"));

    // Только что выставленный маг не колдует; без чужих фигур — тоже.
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        fresh: true,
        ..wizard
    });
    pos.board[sq("h8") as usize] = Some(pawn_with(3 * unit));
    assert!(pos.end_turn().meteors.is_empty());
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(wizard);
    assert!(pos.end_turn().meteors.is_empty());

    // Магом бывают все, кроме пешки и короля.
    let mut pos = crest_position(&[], Color::White);
    pos.rogue.as_mut().unwrap().shop = true;
    let mut kinds = std::collections::HashSet::new();
    for _ in 0..1500 {
        pos.roll_offers();
        for stock in pos.offers().into_iter().flatten() {
            if stock.power == Power::Wizard {
                kinds.insert(stock.kind);
            }
        }
    }
    assert!(!kinds.contains(&Kind::Pawn) && !kinds.contains(&Kind::King));
    assert!(kinds.contains(&Kind::Knight) && kinds.contains(&Kind::Queen));
    assert_eq!(shop_price(Stock::new(Kind::Knight, Power::Wizard)), 8);
    assert_eq!(wizard.title(), "Белый конь-маг");
    assert_eq!(
        Stock::of(wizard).drop_kind(),
        MoveKind::WizardDrop(Kind::Knight)
    );
}

fn fire(kind: Kind) -> Piece {
    Piece {
        fire: true,
        ..Piece::new(kind, Color::White)
    }
}

/// Взрыв огненного коня с e4 (ход белых) с гербами `crests`: клетки и урон задетых.
fn fire_blast(crests: &[Crest], setup: &[(&str, Piece)]) -> (Position, Vec<(Square, i32)>) {
    let mut pos = crest_position(crests, Color::White);
    pos.board[sq("e4") as usize] = Some(fire(Kind::Knight));
    for &(name, piece) in setup {
        pos.board[sq(name) as usize] = Some(piece);
    }
    let blasts = pos.lava_blasts();
    let hits = blasts
        .iter()
        .flat_map(|b| b.hits.iter())
        .map(|h| (h.square, h.damage))
        .collect();
    (pos, hits)
}

#[test]
fn rogue_like_flame_crests_change_the_blast() {
    let unit = crate::balance::UNIT;
    let lava = balance().rule(Rule::LavaDamage);
    let far = pawn_with(5 * unit);
    let near = pawn_with(5 * unit);
    // Без гербов — только соседи.
    let (_, hits) = fire_blast(&[], &[("e6", far), ("e5", near)]);
    assert_eq!(hits, vec![(sq("e5"), lava)]);
    // Два огонька — толстый крест: 3×3 и по три клетки с каждой стороны (d6, e6, f6, c4…),
    // угол квадрата 5×5 (g6) — нет.
    let corner = pawn_with(5 * unit);
    let (_, mut hits) = fire_blast(
        &[Crest::Flames],
        &[
            ("e6", far),
            ("e5", near),
            ("d6", far),
            ("c3", far),
            ("g6", corner),
        ],
    );
    hits.sort();
    let mut expect = vec![
        (sq("e5"), lava),
        (sq("e6"), lava),
        (sq("d6"), lava),
        (sq("c3"), lava),
    ];
    expect.sort();
    assert_eq!(hits, expect);
    let mut pos = crest_position(&[Crest::Flames], Color::White);
    pos.board[sq("e4") as usize] = Some(fire(Kind::Knight));
    pos.board[sq("e6") as usize] = Some(far);
    let blasts = pos.lava_blasts();
    assert!(
        blasts.len() == 1 && blasts[0].wide,
        "только на дальней клетке — тоже взрыв"
    );
    // Огонь и меч — 2 урона; с бомбой — 4, и своим тоже.
    let (_, hits) = fire_blast(&[Crest::FlameSword], &[("e5", near)]);
    assert_eq!(hits, vec![(sq("e5"), 2 * unit)]);
    let ally = Piece::new(Kind::Rook, Color::White);
    let (pos, mut hits) = fire_blast(
        &[Crest::FlameSword, Crest::Bomb],
        &[("e5", near), ("d4", ally)],
    );
    hits.sort();
    let mut expect = vec![(sq("e5"), 4 * unit), (sq("d4"), 4 * unit)];
    expect.sort();
    assert_eq!(hits, expect);
    assert!(pos.piece(sq("e4")).unwrap().fire, "сама цела");
    // С бомбой, но без врагов рядом — не взрывается.
    let (_, hits) = fire_blast(&[Crest::Bomb], &[("d4", ally)]);
    assert!(hits.is_empty());
}

#[test]
fn rogue_like_flame_skull_crest_blows_up_the_fallen() {
    let unit = crate::balance::UNIT;
    // Ход чёрных: пешки d5 и f5 бьют огненного коня e4 с одной единицей здоровья — он гибнет
    // и взрывается, задевая обеих.
    let setup = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::Black);
        pos.board[sq("e4") as usize] = Some(Piece {
            hp: unit,
            ..fire(Kind::Knight)
        });
        pos.board[sq("d5") as usize] = Some(pawn_with(5 * unit));
        pos.board[sq("f5") as usize] = Some(pawn_with(5 * unit));
        pos
    };
    let mut pos = setup(&[Crest::FlameSkull]);
    let turn = pos.end_turn();
    assert_eq!(pos.piece(sq("e4")), None);
    let blast = turn
        .blasts
        .iter()
        .find(|b| b.center == sq("e4"))
        .expect("взрыв");
    let mut hit: Vec<Square> = blast.hits.iter().map(|h| h.square).collect();
    hit.sort();
    assert_eq!(hit, vec![sq("d5"), sq("f5")]);
    // Без герба — просто погиб.
    let mut pos = setup(&[]);
    assert!(pos.end_turn().blasts.iter().all(|b| b.center != sq("e4")));
}

#[test]
fn rogue_like_flame_pawn_and_coin_crests_help_buy_fire_pieces() {
    let pawn = Stock::new(Kind::Pawn, Power::Fire);
    let pos = crest_position(&[Crest::FlamePawn], Color::White);
    assert_eq!(pos.shop_price(pawn), shop_price(pawn) - 2);
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Fire {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    let plain = share(&[]);
    let coin = share(&[Crest::FlameCoin]);
    assert!(coin > plain * 1.6, "{plain} {coin}");
}

fn shade(kind: Kind, hp: i32) -> Piece {
    Piece {
        shade: true,
        hp,
        ..Piece::new(kind, Color::White)
    }
}

/// Ход чёрных: теневой конь e4 (здоровье `hp`) под ударами коней c3, g5 и пешек d5, f5 — по
/// очереди c3, d5, f5, g5; удары по клетке e4.
fn shade_under_fire(crests: &[Crest], hp: i32) -> (Position, Vec<Strike>) {
    let mut pos = crest_position(crests, Color::Black);
    pos.board[sq("e4") as usize] = Some(shade(Kind::Knight, hp));
    for (name, kind) in [
        ("c3", Kind::Knight),
        ("d5", Kind::Pawn),
        ("f5", Kind::Pawn),
        ("g5", Kind::Knight),
    ] {
        pos.board[sq(name) as usize] = Some(Piece::new(kind, Color::Black));
    }
    let strikes = pos
        .strike_all()
        .into_iter()
        .filter(|s| s.hit.square == sq("e4"))
        .collect();
    (pos, strikes)
}

#[test]
fn rogue_like_dark_flames_crest_dodges_every_third_hit() {
    let unit = crate::balance::UNIT;
    let (pos, strikes) = shade_under_fire(&[Crest::DarkFlames], 10 * unit);
    let dodged: Vec<bool> = strikes.iter().map(|s| s.dodged).collect();
    assert_eq!(dodged, vec![false, false, true, false]);
    assert_eq!(hp(&pos, "e4"), 7 * unit);
    assert!(strike_description(&strikes[2]).contains("не получил"));
    let (pos, _) = shade_under_fire(&[], 10 * unit);
    assert_eq!(hp(&pos, "e4"), 6 * unit);
}

#[test]
fn rogue_like_dark_portal_crest_swaps_before_death() {
    let unit = crate::balance::UNIT;
    // Пример пользователя: первый и второй удар проходят, третий — мимо, на четвёртом (у
    // теневой одна единица) она уходит порталом к ближайшей обычной фигуре — пешке c2.
    let (pos, strikes) = shade_under_fire(&[Crest::DarkFlames, Crest::DarkPortal], 3 * unit);
    assert_eq!(strikes.len(), 4);
    assert!(strikes[2].dodged);
    let (other, runner) = strikes[3].swap.expect("ушла порталом");
    assert_eq!(other, sq("c2"));
    assert!(runner.shade);
    assert_eq!(strikes[3].hit.target.kind, Kind::Pawn);
    let pawn = pos.piece(sq("e4")).unwrap();
    assert!(!pawn.shade && pawn.kind == Kind::Pawn);
    assert_eq!(
        pawn.hp,
        Kind::Pawn.stats().max_hp - unit,
        "удар приняла пешка"
    );
    let knight = pos.piece(sq("c2")).unwrap();
    assert!(knight.shade && knight.hp == unit);
    assert!(strike_description(&strikes[3]).contains("порталом на c2"));
    // Без герба — погибла бы.
    let (pos, _) = shade_under_fire(&[Crest::DarkFlames], 3 * unit);
    assert_eq!(pos.piece(sq("e4")), None);
    // С королём не меняется: остались только король и сама теневая — гибнет.
    let mut pos = crest_position(&[Crest::DarkPortal], Color::Black);
    for sq in 0..16 {
        if pos.board[sq].is_some_and(|p| p.kind != Kind::King) {
            pos.board[sq] = None;
        }
    }
    pos.board[sq("e4") as usize] = Some(shade(Kind::Knight, unit));
    pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
    let strikes = pos.strike_all();
    assert!(strikes[0].swap.is_none() && strikes[0].hit.killed);
}

#[test]
fn rogue_like_dark_eye_crest_strikes_through_pieces() {
    let unit = crate::balance::UNIT;
    let hits = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        pos.board[sq("e4") as usize] = Some(shade(Kind::Rook, 4 * unit));
        pos.board[sq("e5") as usize] = Some(pawn_with(5 * unit));
        pos.board[sq("e6") as usize] = Some(pawn_with(5 * unit));
        let mut squares: Vec<Square> = pos
            .strike_all()
            .iter()
            .filter(|s| s.from == sq("e4"))
            .map(|s| s.hit.square)
            .collect();
        squares.sort();
        squares
    };
    assert_eq!(hits(&[]), vec![sq("e5")]);
    let mut both = vec![sq("e5"), sq("e6")];
    both.sort();
    assert_eq!(hits(&[Crest::DarkEye]), both);
}

#[test]
fn rogue_like_dark_skull_crest_hits_jumped_pieces() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[Crest::DarkSkull], Color::White);
    pos.board[sq("a3") as usize] = Some(shade(Kind::Rook, 4 * unit));
    pos.board[sq("a5") as usize] = Some(pawn_with(5 * unit));
    pos.board[sq("a4") as usize] = Some(Piece::new(Kind::Pawn, Color::White));
    let outcome = pos.make_move(Move::new(sq("a3"), sq("a6"), MoveKind::Quiet));
    let attack = Kind::Rook.stats().attack;
    assert_eq!(outcome.passed.len(), 1, "только чужие");
    assert_eq!(outcome.passed[0].square, sq("a5"));
    assert_eq!(hp(&pos, "a5"), 5 * unit - attack);
    assert_eq!(hp(&pos, "a4"), Kind::Pawn.stats().max_hp);
    // Конь «через одну»: на две и одну — через обе клетки рядом.
    let mut passed = Position::shade_passed(sq("d4"), sq("f5"));
    passed.sort();
    let mut expect = vec![sq("e4"), sq("e5")];
    expect.sort();
    assert_eq!(passed, expect);
    assert_eq!(Position::shade_passed(sq("d4"), sq("f6")), vec![sq("e5")]);
    // Без герба — никого.
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("a3") as usize] = Some(shade(Kind::Rook, 4 * unit));
    pos.board[sq("a5") as usize] = Some(pawn_with(5 * unit));
    assert!(
        pos.make_move(Move::new(sq("a3"), sq("a6"), MoveKind::Quiet))
            .passed
            .is_empty()
    );
}

#[test]
fn rogue_like_dark_rook_and_coin_crests_help_buy_shades() {
    let rook = Stock::new(Kind::Rook, Power::Shade);
    let pos = crest_position(&[Crest::DarkRook], Color::White);
    assert_eq!(pos.shop_price(rook), shop_price(rook) - 4);
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Shade {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    let plain = share(&[]);
    assert!(share(&[Crest::DarkCoin]) > plain * 1.6, "{plain}");
}

#[test]
fn rogue_like_armored_and_vampire_waves_come_on_later_levels() {
    let unit = crate::balance::UNIT;
    // Сколько бронированных, вампиров и всего фигур в волнах уровня `level`.
    let count = |level: u8| {
        let mut pos = Position::rogue_start(11);
        let (mut armored, mut vampires, mut zombies, mut all) = (0, 0, 0, 0);
        let mut both = 0;
        for i in 0..200u64 {
            for sq in 0..64 {
                if pos.board[sq].is_some_and(|p| p.color == Color::Black) {
                    pos.board[sq] = None;
                }
            }
            let r = pos.rogue.as_mut().unwrap();
            r.level = level;
            r.wave = 0;
            r.waves = 20;
            r.seed = 2 * i + 1;
            for (sq, kind) in pos.spawn_wave() {
                let p = pos.piece(sq).unwrap();
                all += 1;
                if p.armored && p.vampire {
                    both += 1;
                    let base = pos.kind_max_hp(kind);
                    assert_eq!((p.hp, p.max_hp()), (base + 3 * unit, base + 3 * unit));
                    continue;
                }
                if p.armored {
                    armored += 1;
                    let base = pos.kind_max_hp(kind);
                    assert_eq!((p.hp, p.max_hp()), (base + 3 * unit, base + 3 * unit));
                    assert!(!p.vampire && !p.venom);
                }
                if p.vampire {
                    vampires += 1;
                    assert_eq!(p.max_hp(), pos.kind_max_hp(kind) + unit);
                }
                if p.zombie {
                    zombies += 1;
                    // Зомби 7-го уровня — ядовитые, 5-го — нет.
                    assert_eq!(p.venom, level == 7);
                    assert_eq!(p.max_hp(), pos.kind_max_hp(kind) + unit);
                    assert!(!p.vampire && !p.armored && !p.risen);
                }
            }
        }
        let share = |n: i32| n as f64 / all as f64;
        BOTH.with(|b| b.set(share(both)));
        (share(armored), share(vampires), share(zombies))
    };
    // У каждого уровня — свой вид, половина волны: 3 — броня, 4 — вампиры, 5 — зомби; на
    // остальных — никого.
    assert_eq!(count(1), (0.0, 0.0, 0.0));
    assert_eq!(count(2), (0.0, 0.0, 0.0));
    let near = |x: f64| (x - 0.5).abs() < 0.07;
    let (armored, vampires, zombies) = count(3);
    assert!(
        near(armored) && vampires == 0.0 && zombies == 0.0,
        "{armored}"
    );
    let (armored, vampires, zombies) = count(4);
    assert!(
        armored == 0.0 && near(vampires) && zombies == 0.0,
        "{vampires}"
    );
    let (armored, vampires, zombies) = count(5);
    assert!(
        armored == 0.0 && vampires == 0.0 && near(zombies),
        "{zombies}"
    );
    // 6-й — вампиры в броне (половина волны), отдельных брони и вампиров нет.
    assert_eq!(count(6), (0.0, 0.0, 0.0));
    let both = BOTH.with(|b| b.get());
    assert!((both - 0.5).abs() < 0.07, "{both}");
    // 7-й — ядовитые зомби (половина волны).
    let (armored, vampires, zombies) = count(7);
    assert!(armored == 0.0 && vampires == 0.0, "{armored}");
    assert!((zombies - 0.5).abs() < 0.07, "{zombies}");
    assert_eq!(BOTH.with(|b| b.get()), 0.0);
    assert_eq!(count(8), (0.0, 0.0, 0.0));
    let piece = |armored, vampire| Piece {
        armored,
        vampire,
        ..Piece::new(Kind::Rook, Color::Black)
    };
    assert_eq!(piece(true, false).title(), "Чёрная бронированная ладья");
    assert_eq!(piece(false, true).title(), "Чёрная ладья-вампир");
    let zombie = Piece {
        zombie: true,
        ..Piece::new(Kind::Knight, Color::Black)
    };
    assert_eq!(zombie.title(), "Чёрный конь-зомби");
}

/// Зомби волны на `sq` с полным здоровьем (rogue-like, по правилам `crested`).
fn zombie_at(pos: &mut Position, name: &str, kind: Kind) {
    let piece = pos.crested(Piece {
        zombie: true,
        ..Piece::new(kind, Color::Black)
    });
    pos.board[sq(name) as usize] = Some(piece);
}

#[test]
fn rogue_like_zombies_rise_a_turn_after_death() {
    let unit = crate::balance::UNIT;
    let mut game = Game::from_position(crest_position(&[], Color::White));
    zombie_at(&mut game.position, "d5", Kind::Pawn);
    let zombie = game.position().piece(sq("d5")).unwrap();
    assert_eq!(zombie.max_hp(), Kind::Pawn.stats().max_hp + unit);
    // Конь игрока бьёт зомби (3 удара-единицы на 2-единичную пешку+1 — добиваем с запасом).
    game.position.board[sq("d5") as usize].as_mut().unwrap().hp = 1;
    game.position.board[sq("c3") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    // Ход игрока: удары в конце — зомби погиб, на d5 могила.
    let mv = game.legal_moves()[0];
    game.play(mv).unwrap();
    if game.position().side() == Color::White {
        assert!(game.end_turn());
    }
    assert!(game.position().piece(sq("d5")).is_none(), "зомби погиб");
    assert_eq!(game.position().grave(sq("d5")), Some(Kind::Pawn));
    // Уровень не кончается, пока могила не опустела: волны не кончились — продолжение.
    // Ход соперника: в его конце зомби восстаёт с 1 единицей здоровья.
    assert_eq!(game.position().side(), Color::Black);
    assert!(game.end_turn());
    let turn = game.history().last().unwrap().turn_end().unwrap().clone();
    let (at, kind) = turn.risen[0];
    assert_eq!(kind, Kind::Pawn);
    let risen = game.position().piece(at).unwrap();
    assert!(risen.zombie && risen.risen);
    assert_eq!(risen.hp, unit);
    assert_eq!(game.position().grave(sq("d5")), None);
    let text = game.history().last().unwrap().description();
    assert!(text.contains("Восстали зомби"), "{text}");
    // Второй раз не восстаёт: убитый восставший могилы не оставляет.
    let mut pos = game.position().clone();
    pos.side = Color::White;
    let hit = pos.wound(at, unit);
    assert!(hit.1);
    assert_eq!(pos.grave(at), None);
}

#[test]
fn rogue_like_venom_zombies_rise_venomous() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[], Color::White);
    let zombie = pos.crested(Piece {
        zombie: true,
        venom: true,
        ..Piece::new(Kind::Knight, Color::Black)
    });
    assert_eq!(zombie.title(), "Чёрный ядовитый конь-зомби");
    assert_eq!(zombie.max_hp(), rogue_knight_hp() + unit);
    pos.board[sq("d5") as usize] = Some(zombie);
    pos.side = Color::White;
    assert!(pos.wound(sq("d5"), 99 * unit).1);
    pos.side = Color::Black;
    let risen = pos.rise_zombies();
    assert_eq!(risen, vec![(sq("d5"), Kind::Knight)]);
    let p = pos.piece(sq("d5")).unwrap();
    assert!(p.venom && p.zombie && p.risen);
    assert_eq!(p.hp, unit);
}

#[test]
fn rogue_like_zombie_grave_keeps_the_level_going_and_rises_beside_a_blocker() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[], Color::White);
    zombie_at(&mut pos, "e6", Kind::Knight);
    // Погиб в ход игрока: могила, ждать один конец хода соперника.
    pos.side = Color::White;
    let (_, killed) = pos.wound(sq("e6"), 99 * unit);
    assert!(killed);
    assert!(pos.waves_left(Color::Black), "могила — ещё не конец уровня");
    // На клетку могилы встала фигура игрока — зомби восстаёт рядом.
    pos.board[sq("e6") as usize] = Some(Piece::new(Kind::Rook, Color::White));
    pos.side = Color::Black;
    let risen = pos.rise_zombies();
    assert_eq!(risen.len(), 1);
    let (at, kind) = risen[0];
    assert_eq!(kind, Kind::Knight);
    assert_ne!(at, sq("e6"));
    assert!((file_of(at) - file_of(sq("e6"))).abs() <= 1);
    assert!((rank_of(at) - rank_of(sq("e6"))).abs() <= 1);
    assert_eq!(pos.grave(sq("e6")), None, "могила опустела");
    assert!(pos.piece(at).is_some_and(|p| p.zombie && p.risen));
    // Погиб в свой ход (о шипы, в кратере) — восстанет не в этом конце хода, а в следующем.
    let mut pos = crest_position(&[], Color::White);
    zombie_at(&mut pos, "c6", Kind::Bishop);
    pos.side = Color::Black;
    pos.wound(sq("c6"), 99 * unit);
    assert!(pos.rise_zombies().is_empty());
    assert_eq!(pos.grave(sq("c6")), Some(Kind::Bishop));
    assert_eq!(pos.rise_zombies().len(), 1);
}

thread_local! {
    /// Доля вампиров в броне в последнем подсчёте волн.
    static BOTH: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

#[test]
fn rogue_like_armored_vampires_heal_once_a_turn() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[], Color::Black);
    // Чёрная ладья-вампир в броне на e5 бьёт две белые фигуры по прямой.
    let vampire = pos.crested(Piece {
        armored: true,
        vampire: true,
        ..Piece::new(Kind::Rook, Color::Black)
    });
    assert_eq!(vampire.max_hp(), Kind::Rook.stats().max_hp + 3 * unit);
    assert_eq!(vampire.title(), "Чёрная бронированная ладья-вампир");
    pos.board[sq("e5") as usize] = Some(Piece {
        hp: unit,
        ..vampire
    });
    pos.board[sq("e4") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    pos.board[sq("d5") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    let strikes = pos.strike_all();
    let feasts: Vec<i32> = strikes
        .iter()
        .filter(|s| s.from == sq("e5"))
        .map(|s| s.feast)
        .collect();
    assert_eq!(feasts.len(), 2);
    assert_eq!(feasts.iter().sum::<i32>(), unit, "лечится один раз за ход");
    assert_eq!(hp(&pos, "e5"), 2 * unit);
    // В следующий ход — снова один раз.
    pos.side = Color::White;
    pos.end_turn();
    pos.side = Color::Black;
    pos.fed = 0;
    let strikes = pos.strike_all();
    let healed: i32 = strikes
        .iter()
        .filter(|s| s.from == sq("e5"))
        .map(|s| s.feast)
        .sum();
    assert_eq!(healed, unit);
}

#[test]
fn rogue_like_vampires_heal_by_striking() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[], Color::Black);
    pos.board[sq("e4") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    pos.board[sq("f5") as usize] = Some(Piece {
        vampire: true,
        hp: unit,
        ..Piece::new(Kind::Pawn, Color::Black)
    });
    let strikes = pos.strike_all();
    let bite = strikes.iter().find(|s| s.from == sq("f5")).unwrap();
    assert_eq!(bite.feast, unit);
    assert_eq!(hp(&pos, "f5"), 2 * unit);
    assert!(
        strike_description(bite).contains("восстановила 1 HP"),
        "{}",
        strike_description(bite)
    );
    // Полное здоровье выше не поднимается.
    let mut pos = crest_position(&[], Color::Black);
    pos.board[sq("e4") as usize] = Some(Piece::new(Kind::Knight, Color::White));
    let vampire = pos.crested(Piece {
        vampire: true,
        ..Piece::new(Kind::Pawn, Color::Black)
    });
    pos.board[sq("f5") as usize] = Some(vampire);
    let strikes = pos.strike_all();
    assert_eq!(
        strikes.iter().find(|s| s.from == sq("f5")).unwrap().feast,
        0
    );
    assert_eq!(hp(&pos, "f5"), vampire.max_hp());
}

#[test]
fn rogue_like_winds_crest_gives_a_third_hop() {
    // Сколько раз подряд воздушный конь может сходить за ход и сколько ходов это стоит.
    let hops = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        pos.board[sq("d4") as usize] = Some(Piece {
            wind: true,
            ..Piece::new(Kind::Knight, Color::White)
        });
        let mut game = Game::from_position(pos);
        let mut at = sq("d4");
        let mut hops = 0;
        while let Some(&mv) = game
            .legal_moves()
            .iter()
            .find(|m| m.from == at && rank_of(m.to) >= 2 && rank_of(m.to) <= 5)
        {
            game.play(mv).unwrap();
            at = mv.to;
            hops += 1;
            if hops > 5 {
                break;
            }
        }
        (hops, game.position().moves_made)
    };
    assert_eq!(hops(&[]), (2, 1));
    assert_eq!(hops(&[Crest::Winds]), (3, 1));
}

#[test]
fn rogue_like_wind_sword_crest_strikes_one_target_twice() {
    let unit = crate::balance::UNIT;
    let strikes = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        pos.board[sq("e4") as usize] = Some(Piece {
            wind: true,
            ..Piece::new(Kind::Knight, Color::White)
        });
        pos.board[sq("d6") as usize] = Some(pawn_with(5 * unit));
        pos.board[sq("f6") as usize] = Some(pawn_with(5 * unit));
        pos.strike_all()
            .iter()
            .filter(|s| s.from == sq("e4"))
            .map(|s| (s.hit.square, s.again))
            .collect::<Vec<_>>()
    };
    assert_eq!(strikes(&[]).len(), 2);
    let double = strikes(&[Crest::WindSword]);
    assert_eq!(double.len(), 3);
    assert_eq!(double[2], (double[0].0, true), "первая цель — ещё раз");
}

#[test]
fn rogue_like_wind_knight_and_coin_crests_help_buy_wind_pieces() {
    let knight = Stock::new(Kind::Knight, Power::Wind);
    let pos = crest_position(&[Crest::WindKnight], Color::White);
    assert_eq!(pos.shop_price(knight), shop_price(knight) - 4);
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Wind {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    let plain = share(&[]);
    assert!(share(&[Crest::WindCoin]) > plain * 1.6, "{plain}");
}

/// Белый электрический конь e4 бьёт пешку d6 (здоровья много); `setup` — ещё чужие фигуры.
/// Возвращает клетки, задетые молнией.
fn spark_splash(crests: &[Crest], charge: u8, setup: &[&str]) -> Vec<Square> {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(crests, Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        spark: true,
        charge,
        ..Piece::new(Kind::Knight, Color::White)
    });
    pos.board[sq("d6") as usize] = Some(pawn_with(9 * unit));
    for &name in setup {
        pos.board[sq(name) as usize] = Some(pawn_with(9 * unit));
    }
    let strikes = pos.strike_all();
    let strike = strikes
        .iter()
        .find(|s| s.from == sq("e4") && s.hit.square == sq("d6"))
        .expect("удар по d6");
    let mut squares: Vec<Square> = strike.splash.iter().map(|j| j.hit.square).collect();
    squares.sort();
    squares
}

#[test]
fn rogue_like_bolt_crests_widen_the_lightning() {
    let sorted = |names: &[&str]| {
        let mut v: Vec<Square> = names.iter().map(|&n| sq(n)).collect();
        v.sort();
        v
    };
    // Без гербов — только по диагонали и вплотную.
    assert_eq!(spark_splash(&[], 0, &["c6", "c7", "b8"]), sorted(&["c7"]));
    // Молния и плюс — ещё по прямым.
    assert_eq!(
        spark_splash(&[Crest::BoltPlus], 0, &["c6", "c7"]),
        sorted(&["c6", "c7"])
    );
    // Две молнии — цепью по связанным: d6 → c7 → b8 и d6 → e7 → f8; через пустую клетку — нет.
    assert_eq!(
        spark_splash(&[Crest::Bolts], 0, &["c7", "b8", "e7", "f8"]),
        sorted(&["c7", "b8", "e7", "f8"])
    );
    assert_eq!(spark_splash(&[], 0, &["c7", "b8"]), sorted(&["c7"]));
    assert_eq!(spark_splash(&[Crest::Bolts], 0, &["b8"]), sorted(&[]));
    // С плюсом цепь идёт и по прямым: d6 → d7 → d8 → e8.
    assert_eq!(
        spark_splash(&[Crest::Bolts, Crest::BoltPlus], 0, &["d7", "d8", "e8"]),
        sorted(&["d7", "d8", "e8"])
    );
    // Красная молния: 10 зарядов — на клетку дальше.
    assert_eq!(spark_splash(&[Crest::RedBolt], 9, &["b8"]), sorted(&[]));
    assert_eq!(
        spark_splash(&[Crest::RedBolt], 10, &["b8"]),
        sorted(&["b8"])
    );
    let pos = crest_position(&[Crest::RedBolt], Color::White);
    let reach = |charge| {
        pos.spark_reach(Piece {
            spark: true,
            charge,
            ..Piece::new(Kind::Knight, Color::White)
        })
    };
    assert_eq!(
        (reach(0), reach(19), reach(20), reach(30), reach(200)),
        (1, 2, 3, 4, 4)
    );
}

#[test]
fn rogue_like_red_bolt_crest_charges_on_kills() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[Crest::RedBolt], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        spark: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    // Конь убивает d6, молния — c7.
    pos.board[sq("d6") as usize] = Some(pawn_with(unit));
    pos.board[sq("c7") as usize] = Some(pawn_with(1));
    pos.strike_all();
    assert_eq!(pos.piece(sq("e4")).unwrap().charge, 2);
    // Без герба заряд не копится.
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        spark: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    pos.board[sq("d6") as usize] = Some(pawn_with(unit));
    pos.strike_all();
    assert_eq!(pos.piece(sq("e4")).unwrap().charge, 0);
}

#[test]
fn rogue_like_bolt_one_crest_strikes_first() {
    let unit = crate::balance::UNIT;
    let first = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        pos.board[sq("b4") as usize] = Some(Piece::new(Kind::Knight, Color::White));
        pos.board[sq("g4") as usize] = Some(Piece {
            spark: true,
            ..Piece::new(Kind::Knight, Color::White)
        });
        pos.board[sq("a6") as usize] = Some(pawn_with(9 * unit));
        pos.board[sq("h6") as usize] = Some(pawn_with(9 * unit));
        pos.strike_all()[0].from
    };
    assert_eq!(first(&[]), sq("b4"));
    assert_eq!(first(&[Crest::BoltOne]), sq("g4"));
}

#[test]
fn rogue_like_bolt_bishop_and_coin_crests_help_buy_sparks() {
    let bishop = Stock::new(Kind::Bishop, Power::Spark);
    let pos = crest_position(&[Crest::BoltBishop], Color::White);
    assert_eq!(pos.shop_price(bishop), shop_price(bishop) - 4);
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Spark {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    let plain = share(&[]);
    assert!(share(&[Crest::BoltCoin]) > plain * 1.6, "{plain}");
}

#[test]
fn rogue_like_chain_lightning_remembers_its_jumps() {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[Crest::Bolts], Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        spark: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    for name in ["d6", "c7", "b8"] {
        pos.board[sq(name) as usize] = Some(pawn_with(9 * unit));
    }
    let strikes = pos.strike_all();
    let strike = strikes.iter().find(|s| s.hit.square == sq("d6")).unwrap();
    let jumps: Vec<(Square, Square)> = strike
        .splash
        .iter()
        .map(|j| (j.from, j.hit.square))
        .collect();
    assert_eq!(jumps, vec![(sq("d6"), sq("c7")), (sq("c7"), sq("b8"))]);
}

#[test]
fn rogue_like_pair_crests_double_their_kind_in_the_shop() {
    let share = |crests: &[Crest], kind: Kind| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..2000 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.kind == kind {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    for (crest, kind) in [
        (Crest::Pawns, Kind::Pawn),
        (Crest::Bishops, Kind::Bishop),
        (Crest::Knights, Kind::Knight),
        (Crest::Rooks, Kind::Rook),
        (Crest::Queens, Kind::Queen),
    ] {
        let plain = share(&[], kind);
        let paired = share(&[crest], kind);
        // Вес вида вдвое больше: доля p → 2p / (1 + p).
        let expect = 2.0 * plain / (1.0 + plain);
        assert!(
            (paired - expect).abs() < 0.04,
            "{kind:?}: {plain} → {paired}, ждали {expect}"
        );
        assert!(crest.description().contains("вдвое чаще"));
    }
}

#[test]
fn rogue_like_reroll_coin_crest_makes_rerolls_cheap() {
    let mut game = Game::from_position(Position::rogue_start(7));
    assert!(game.win_round());
    assert_eq!(game.position().reroll_price(), 2);
    game.position.rogue.as_mut().unwrap().crests[0] = Some(Crest::RerollCoin);
    assert_eq!(game.position().reroll_price(), 1);
    game.position.rogue.as_mut().unwrap().gold = 3;
    assert!(game.reroll());
    assert_eq!(game.position().rogue().unwrap().gold, 2);
    assert!(
        Crest::RerollCoin
            .description()
            .contains("1 монета вместо 2")
    );
}

/// Ход белых: конь-маг e4 и чёрные пешки `enemies` (здоровья много); конец хода — метеориты.
fn cast(crests: &[Crest], enemies: &[&str]) -> (Position, TurnEnd) {
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(crests, Color::White);
    pos.board[sq("e4") as usize] = Some(Piece {
        wizard: true,
        ..Piece::new(Kind::Knight, Color::White)
    });
    for &name in enemies {
        pos.board[sq(name) as usize] = Some(pawn_with(9 * unit));
    }
    let turn = pos.end_turn();
    (pos, turn)
}

#[test]
fn rogue_like_hat_crests_power_up_meteors() {
    let unit = crate::balance::UNIT;
    // Две шляпы — два метеорита; метеорит — 2 урона.
    let (_, turn) = cast(&[Crest::Hats, Crest::Meteor], &["h8"]);
    assert_eq!(turn.meteors.len(), 2);
    assert!(
        turn.meteors
            .iter()
            .all(|m| m.at == sq("h8") && m.hits[0].damage == 2 * unit)
    );
    let (_, turn) = cast(&[], &["h8"]);
    assert_eq!(turn.meteors.len(), 1);
    assert_eq!(turn.meteors[0].hits[0].damage, unit);
    // Шляпа и плюс — крестом: h8 и g8, h7 (но не g7).
    // Клетка падения случайная: задеты она и чужие соседи по прямым, по диагонали — нет.
    let enemies = ["h8", "g8", "h7", "g7"].map(sq);
    let (_, turn) = cast(&[Crest::HatPlus], &["h8", "g8", "h7", "g7"]);
    let meteor = &turn.meteors[0];
    let at = meteor.at;
    let mut hit: Vec<Square> = meteor.hits.iter().map(|h| h.square).collect();
    hit.sort();
    let mut expect: Vec<Square> = enemies
        .into_iter()
        .filter(|&s| {
            let (df, dr) = (file_of(s) - file_of(at), rank_of(s) - rank_of(at));
            df.abs() + dr.abs() <= 1
        })
        .collect();
    expect.sort();
    assert_eq!(hit, expect);
    assert_eq!(hit.len(), 3, "в углу из четырёх — клетка и два соседа");
    assert!(meteor.wide);
}

#[test]
fn rogue_like_hat_meteor_crest_leaves_a_burning_crater() {
    let unit = crate::balance::UNIT;
    // Метеорит падает на пешку h5 (одна чужая фигура) — там кратер; ход чёрных: она шагает на h4
    // — не в кратер, а пешка h6 шагает в кратер h5 и горит.
    let (mut pos, turn) = cast(&[Crest::HatMeteor], &["h5"]);
    assert!(turn.meteors[0].crater);
    assert_eq!(pos.craters(), 1 << sq("h5"));
    pos.board[sq("h6") as usize] = Some(pawn_with(9 * unit));
    let turn = pos.end_turn();
    assert!(turn.marched.contains(&(sq("h6"), sq("h5"))));
    assert_eq!(turn.burned.len(), 1);
    assert_eq!(
        (turn.burned[0].square, turn.burned[0].damage),
        (sq("h5"), unit)
    );
    assert_eq!(hp(&pos, "h5"), 8 * unit);
    // К ходу белых кратер погас.
    assert_eq!(pos.craters(), 0);
    // Без герба кратера нет.
    let (pos, turn) = cast(&[], &["h5"]);
    assert!(!turn.meteors[0].crater && pos.craters() == 0);
}

#[test]
fn rogue_like_hat_coin_and_bishop_crests_help_buy_wizards() {
    let bishop = Stock::new(Kind::Bishop, Power::Wizard);
    let pos = crest_position(&[Crest::HatBishop], Color::White);
    assert_eq!(pos.shop_price(bishop), shop_price(bishop) - 4);
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..1500 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.power == Power::Wizard {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    let plain = share(&[]);
    assert!(share(&[Crest::HatCoin]) > plain * 1.6, "{plain}");
}

fn golden(kind: Kind, hp: i32) -> Piece {
    Piece {
        gold: true,
        hp,
        ..Piece::new(kind, Color::White)
    }
}

#[test]
fn rogue_like_king_crests_add_gold_for_kills() {
    let unit = crate::balance::UNIT;
    let earned = |crests: &[Crest], kind: Kind| {
        let mut pos = crest_position(crests, Color::White);
        pos.board[sq("e4") as usize] = Some(golden(kind, 5 * unit));
        pos.board[sq("d6") as usize] = Some(pawn_with(1));
        pos.board[sq("e5") as usize] = Some(pawn_with(1));
        let before = pos.rogue().unwrap().gold;
        let kills = pos.strike_all().iter().filter(|s| s.hit.killed).count() as u16;
        (pos.rogue().unwrap().gold - before, kills)
    };
    // Золотой конь бьёт d6: 1 за пешку + 2 за золотую фигуру; с гербом короля — ещё 1.
    assert_eq!(earned(&[], Kind::Knight), (3, 1));
    assert_eq!(earned(&[Crest::GoldKing], Kind::Knight), (4, 1));
    // Золотой ферзь с гербом короля и ферзя — ещё 2 (достаёт только e5).
    assert_eq!(earned(&[Crest::KingQueen], Kind::Queen), (1 + 2 + 2, 1));
    assert_eq!(
        earned(&[Crest::KingQueen], Kind::Knight),
        (3, 1),
        "только ферзь"
    );
}

#[test]
fn rogue_like_king_coin_crest_sells_at_full_price() {
    let fire = Stock::new(Kind::Knight, Power::Fire);
    let pos = crest_position(&[], Color::White);
    assert_eq!(
        pos.sell_price(fire),
        shop_price(Stock::new(Kind::Knight, Power::Plain))
    );
    let pos = crest_position(&[Crest::KingCoin], Color::White);
    assert_eq!(pos.sell_price(fire), shop_price(fire));
    let gold = Stock::new(Kind::Knight, Power::Gold);
    assert_eq!(pos.sell_price(gold), sell_price(gold).max(shop_price(gold)));
}

#[test]
fn rogue_like_king_skull_crest_refunds_fallen_gold() {
    let unit = crate::balance::UNIT;
    let refund = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::Black);
        pos.board[sq("e4") as usize] = Some(golden(Kind::Knight, unit));
        pos.board[sq("f5") as usize] = Some(Piece::new(Kind::Pawn, Color::Black));
        let before = pos.rogue().unwrap().gold;
        pos.strike_all();
        assert_eq!(pos.piece(sq("e4")), None);
        pos.rogue().unwrap().gold - before
    };
    assert_eq!(refund(&[]), 0);
    assert_eq!(
        refund(&[Crest::KingSkull]),
        shop_price(Stock::new(Kind::Knight, Power::Gold))
    );
}

#[test]
fn rogue_like_piggy_crest_pays_interest_on_gold_pieces() {
    let mut pos = crest_position(&[Crest::Piggy], Color::White);
    pos.board[sq("e4") as usize] = Some(golden(Kind::Knight, 900));
    pos.gold_reserve[Color::White as usize][Kind::Pawn as usize] = 1;
    pos.rogue.as_mut().unwrap().gold = 25;
    assert_eq!(
        pos.piggy_bank(),
        4,
        "25 золота — 2 по монете на 2 золотые фигуры"
    );
    let mut game = Game::from_position(Position::rogue_start(7));
    game.position.rogue.as_mut().unwrap().crests[0] = Some(Crest::Piggy);
    game.position.board[sq("e3") as usize] = Some(golden(Kind::Knight, 900));
    game.position.rogue.as_mut().unwrap().gold = 30;
    assert!(game.win_round());
    let record = game
        .history()
        .iter()
        .rev()
        .find(|r| r.turn_end().is_some())
        .unwrap();
    let turn = record.turn_end().unwrap();
    assert!(turn.interest >= 3, "{}", turn.interest);
    assert!(record.description().contains("Свинья-копилка"));
    let crests = crest_position(&[], Color::White);
    assert_eq!(crests.piggy_bank(), 0);
}

#[test]
fn rogue_like_axe_and_king_crests_open_combined_pieces_in_the_shop() {
    use std::collections::HashSet;
    let combos = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let mut seen = HashSet::new();
        for _ in 0..3000 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                if stock.with != Power::Plain {
                    assert!(
                        !(stock.has(Power::Wizard) && stock.kind == Kind::Pawn),
                        "магом пешка не бывает"
                    );
                    seen.insert((stock.power, stock.with));
                }
            }
        }
        seen
    };
    assert!(combos(&[]).is_empty(), "без гербов объединённых нет");
    assert_eq!(
        combos(&[Crest::AxeFlame]),
        HashSet::from([(Power::Barbarian, Power::Fire)])
    );
    assert_eq!(
        combos(&[Crest::AxeHat, Crest::KingFlame]),
        HashSet::from([
            (Power::Barbarian, Power::Wizard),
            (Power::Gold, Power::Fire)
        ])
    );
    let all: Vec<Crest> = Crest::ALL
        .into_iter()
        .filter(|c| c.unlocks().is_some())
        .collect();
    assert_eq!(all.len(), COMBOS.len());
    assert_eq!(combos(&all), HashSet::from(COMBOS));
    for crest in all {
        let text = crest.description();
        let (base, _) = crest.unlocks().unwrap();
        let word = match base {
            Power::Gold => "золот",
            Power::Wizard => "маг",
            Power::Ice => "ледян",
            Power::Fire => "огнен",
            Power::Spark => "электр",
            Power::Wind => "воздуш",
            _ => "варвар",
        };
        assert!(text.contains(word) && !text.contains("  "), "{text}");
    }
}

#[test]
fn rogue_like_wind_combined_piece_has_both_powers_and_their_crests() {
    let stock = Stock::combo(Kind::Rook, Power::Wind, Power::Shade);
    let rook = stock.piece(Color::White);
    assert!(rook.wind && rook.shade && !rook.spark);
    assert_eq!(rook.title(), "Белая воздушная теневая ладья");
    assert_eq!(Stock::of(rook), stock);
    // Пары, открытые раньше, — прежние: воздушные варвар, золотая, маг, ледяная, огненная,
    // электрическая.
    for base in [
        Power::Barbarian,
        Power::Gold,
        Power::Wizard,
        Power::Ice,
        Power::Fire,
        Power::Spark,
    ] {
        let old = Stock::of(Stock::combo(Kind::Rook, base, Power::Wind).piece(Color::White));
        assert_eq!((old.power, old.with), (base, Power::Wind));
    }
    // Скидки гербов обеих сил: ветер и конь (−4), тень и ладья (−4).
    let pos = crest_position(&[Crest::WindKnight, Crest::DarkRook], Color::White);
    let knight = Stock::combo(Kind::Knight, Power::Wind, Power::Shade);
    assert_eq!(pos.shop_price(knight), shop_price(knight) - 4);
    assert_eq!(pos.shop_price(stock), shop_price(stock) - 4);
    // Выставление — последним номером, и по сети тоже.
    let mut pos = crest_position(&[Crest::WindDark], Color::White);
    pos.give_stock(rook);
    let legal = pos.legal_moves();
    let mv = *legal
        .iter()
        .find(|m| m.kind == stock.drop_kind())
        .expect("выставление воздушной объединённой");
    assert_eq!(mv.kind, MoveKind::ComboDrop(Kind::Rook, 27));
    assert_eq!(
        crate::net::decode_move(&crate::net::encode_move(mv), &legal),
        Some(mv)
    );

    // Как теневая — сквозь пешку a2 на a3; как воздушная — второй ход даровой (и тоже сквозь).
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("a1") as usize] = Some(rook);
    let left = pos.moves_left();
    let first = *pos
        .legal_moves()
        .iter()
        .find(|m| m.from == sq("a1") && m.to == sq("a3"))
        .expect("сквозь пешку");
    pos.make_move(first);
    assert_eq!(pos.moves_left(), left - 1);
    assert!(pos.can_move(sq("a3")), "второй ход воздушной");
    pos.board[sq("a4") as usize] = Some(Piece::new(Kind::Pawn, Color::White));
    assert!(
        pos.legal_moves()
            .iter()
            .any(|m| m.from == sq("a3") && m.to == sq("a5")),
        "второй ход — тоже сквозь фигуры"
    );
}

#[test]
fn rogue_like_spark_combined_piece_has_both_powers_and_their_crests() {
    let stock = Stock::combo(Kind::Knight, Power::Spark, Power::Wind);
    let knight = stock.piece(Color::White);
    assert!(knight.spark && knight.wind && !knight.fire);
    assert_eq!(knight.title(), "Белый электрический воздушный конь");
    assert_eq!(Stock::of(knight), stock);
    // Пары, открытые раньше, — прежние: электрические варвар, золотая, маг, ледяная, огненная.
    for base in [
        Power::Barbarian,
        Power::Gold,
        Power::Wizard,
        Power::Ice,
        Power::Fire,
    ] {
        let old = Stock::of(Stock::combo(Kind::Rook, base, Power::Spark).piece(Color::White));
        assert_eq!((old.power, old.with), (base, Power::Spark));
    }
    // Скидки гербов обеих сил: молния и слон (−4), ветер и конь (−4).
    let pos = crest_position(&[Crest::BoltBishop, Crest::WindKnight], Color::White);
    let bishop = Stock::combo(Kind::Bishop, Power::Spark, Power::Shade);
    assert_eq!(pos.shop_price(bishop), shop_price(bishop) - 4);
    assert_eq!(pos.shop_price(stock), shop_price(stock) - 4);

    // Электрический теневой ферзь: код выставления в сети — больше 255.
    let queen = Stock::combo(Kind::Queen, Power::Spark, Power::Shade);
    let mut pos = crest_position(&[Crest::BoltDark], Color::White);
    pos.give_stock(queen.piece(Color::White));
    let legal = pos.legal_moves();
    let mv = *legal
        .iter()
        .find(|m| m.kind == queen.drop_kind())
        .expect("выставление электрической объединённой");
    assert_eq!(mv.kind, MoveKind::ComboDrop(Kind::Queen, 26));
    assert!(crate::net::encode_move(mv).contains(" 260 "));
    assert_eq!(
        crate::net::decode_move(&crate::net::encode_move(mv), &legal),
        Some(mv)
    );

    // Как теневая — ходит сквозь фигуры: ладья с a1 через пешку a2 на a3.
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("a1") as usize] =
        Some(Stock::combo(Kind::Rook, Power::Spark, Power::Shade).piece(Color::White));
    assert!(
        pos.legal_moves()
            .iter()
            .any(|m| m.from == sq("a1") && m.to == sq("a3"))
    );
    // Как воздушная — первый ход тратит ход, второй даровой.
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(knight);
    let left = pos.moves_left();
    let first = *pos
        .legal_moves()
        .iter()
        .find(|m| m.from == sq("e4") && m.to == sq("f6"))
        .expect("ход коня");
    pos.make_move(first);
    assert_eq!(pos.moves_left(), left - 1);
    assert!(pos.can_move(sq("f6")), "второй ход воздушной");
    // Как электрическая — молния с цели на соседнюю по диагонали.
    let unit = crate::balance::UNIT;
    let mut pos = crest_position(&[], Color::White);
    pos.board[sq("e4") as usize] = Some(knight);
    pos.board[sq("d6") as usize] = Some(pawn_with(9 * unit));
    pos.board[sq("c7") as usize] = Some(pawn_with(9 * unit));
    let strikes = pos.strike_all();
    assert!(strikes.iter().any(|s| s.from == sq("e4")
        && s.hit.square == sq("d6")
        && s.splash.iter().any(|j| j.hit.square == sq("c7"))));
}

#[test]
fn rogue_like_fire_combined_piece_has_both_powers_and_their_crests() {
    let unit = crate::balance::UNIT;
    let stock = Stock::combo(Kind::Knight, Power::Fire, Power::Spark);
    let knight = stock.piece(Color::White);
    assert!(knight.fire && knight.spark && !knight.icy);
    assert_eq!(knight.title(), "Белый огненный электрический конь");
    assert_eq!(Stock::of(knight), stock);
    assert_eq!(
        Stock::combo(Kind::Rook, Power::Fire, Power::Shade).name(),
        "огненная теневая ладья"
    );
    // Пары, открытые раньше, — прежние: огненные варвар, золотая, маг, ледяная.
    for base in [Power::Barbarian, Power::Gold, Power::Wizard, Power::Ice] {
        let old = Stock::of(Stock::combo(Kind::Rook, base, Power::Fire).piece(Color::White));
        assert_eq!((old.power, old.with), (base, Power::Fire));
    }
    // Скидки гербов обеих сил: огонь и пешка (−2), молния и слон (−4).
    let pos = crest_position(&[Crest::FlamePawn, Crest::BoltBishop], Color::White);
    let pawn = Stock::combo(Kind::Pawn, Power::Fire, Power::Wind);
    assert_eq!(pos.shop_price(pawn), shop_price(pawn) - 2);
    let bishop = Stock::combo(Kind::Bishop, Power::Fire, Power::Spark);
    assert_eq!(pos.shop_price(bishop), shop_price(bishop) - 4);
    // Выставление — своим номером, и по сети тоже.
    let mut pos = crest_position(&[Crest::FlameBolt], Color::White);
    pos.give_stock(knight);
    let legal = pos.legal_moves();
    let mv = *legal
        .iter()
        .find(|m| m.kind == stock.drop_kind())
        .expect("выставление огненной объединённой");
    assert_eq!(mv.kind, MoveKind::ComboDrop(Kind::Knight, 22));
    assert_eq!(
        crate::net::decode_move(&crate::net::encode_move(mv), &legal),
        Some(mv)
    );

    // Как электрическая — молния с цели на соседнюю по диагонали; как огненная — взрыв (огонь
    // и меч — на 2).
    let mut pos = crest_position(&[Crest::FlameSword], Color::White);
    pos.board[sq("e4") as usize] = Some(knight);
    pos.board[sq("d6") as usize] = Some(pawn_with(9 * unit));
    pos.board[sq("c7") as usize] = Some(pawn_with(9 * unit));
    pos.board[sq("e5") as usize] = Some(pawn_with(9 * unit));
    let strikes = pos.strike_all();
    let strike = strikes
        .iter()
        .find(|s| s.from == sq("e4") && s.hit.square == sq("d6"))
        .expect("удар по d6");
    assert!(
        strike.splash.iter().any(|j| j.hit.square == sq("c7")),
        "молния на c7"
    );
    let blasts: Vec<(Square, i32)> = pos
        .lava_blasts()
        .iter()
        .flat_map(|b| b.hits.iter())
        .map(|h| (h.square, h.damage))
        .collect();
    assert_eq!(blasts, vec![(sq("e5"), 2 * unit)]);
}

#[test]
fn rogue_like_ice_combined_piece_has_both_powers_and_their_crests() {
    let unit = crate::balance::UNIT;
    let stock = Stock::combo(Kind::Knight, Power::Ice, Power::Fire);
    let knight = stock.piece(Color::White);
    assert!(knight.icy && knight.fire && !knight.wizard);
    assert_eq!(knight.title(), "Белый огненный ледяной конь");
    assert_eq!(Stock::of(knight), stock);
    assert_eq!(
        Stock::combo(Kind::Rook, Power::Ice, Power::Shade).name(),
        "теневая ледяная ладья"
    );
    // Пары, открытые раньше, — прежние: ледяной варвар, ледяная золотая, ледяной маг.
    for base in [Power::Barbarian, Power::Gold, Power::Wizard] {
        let old = Stock::of(Stock::combo(Kind::Rook, base, Power::Ice).piece(Color::White));
        assert_eq!((old.power, old.with), (base, Power::Ice));
    }
    // Снежинка и пешка: огненная ледяная пешка — как огненная.
    let pos = crest_position(&[Crest::Pawn], Color::White);
    assert_eq!(
        pos.shop_price(Stock::combo(Kind::Pawn, Power::Ice, Power::Fire)),
        shop_price(Stock::new(Kind::Pawn, Power::Fire))
    );
    // Выставление — своим номером, и по сети тоже.
    let mut pos = crest_position(&[Crest::FlameSnow, Crest::Heart], Color::White);
    pos.give_stock(knight);
    let legal = pos.legal_moves();
    let mv = *legal
        .iter()
        .find(|m| m.kind == stock.drop_kind())
        .expect("выставление ледяной объединённой");
    assert_eq!(mv.kind, MoveKind::ComboDrop(Kind::Knight, 18));
    assert_eq!(
        crate::net::decode_move(&crate::net::encode_move(mv), &legal),
        Some(mv)
    );
    // Сердце и снежинка — здоровье ледяной.
    pos.make_move(mv);
    let placed = pos.piece(mv.to).unwrap();
    assert_eq!(placed.max_hp(), rogue_knight_hp() + 2 * unit);

    // Как ледяная — ударивший замерзает; как огненная — взрывается (огонь и меч — на 2).
    let mut pos = crest_position(&[Crest::FlameSword], Color::Black);
    pos.board[sq("e4") as usize] = Some(knight);
    pos.board[sq("e6") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    let strikes = pos.strike_all();
    assert!(
        strikes.iter().any(|s| s.from == sq("e6") && s.rebound),
        "ударивший ледяную замёрз"
    );
    pos.side = Color::White;
    pos.board[sq("e5") as usize] = Some(pawn_with(5 * unit));
    let blasts: Vec<(Square, i32)> = pos
        .lava_blasts()
        .iter()
        .flat_map(|b| b.hits.iter())
        .map(|h| (h.square, h.damage))
        .collect();
    assert_eq!(blasts, vec![(sq("e5"), 2 * unit)]);
}

#[test]
fn rogue_like_wizard_combined_piece_has_both_powers_and_their_crests() {
    let unit = crate::balance::UNIT;
    let stock = Stock::combo(Kind::Knight, Power::Wizard, Power::Fire);
    let knight = stock.piece(Color::White);
    assert!(knight.wizard && knight.fire && !knight.gold);
    assert_eq!(knight.title(), "Белый огненный конь-маг");
    assert_eq!(Stock::of(knight), stock);
    assert_eq!(stock.drop_kind(), MoveKind::ComboDrop(Kind::Knight, 13));
    assert_eq!(
        Stock::combo(Kind::Rook, Power::Wizard, Power::Ice).name(),
        "ледяная ладья-маг"
    );
    // Пары, открытые раньше, — прежние: золотой маг, варвар-маг.
    let gold = Stock::of(Stock::combo(Kind::Rook, Power::Gold, Power::Wizard).piece(Color::White));
    assert_eq!((gold.power, gold.with), (Power::Gold, Power::Wizard));
    // Скидка шляпы и слона — и магу-объединённому.
    let pos = crest_position(&[Crest::HatBishop], Color::White);
    let bishop = Stock::combo(Kind::Bishop, Power::Wizard, Power::Spark);
    assert_eq!(pos.shop_price(bishop), shop_price(bishop) - 4);

    // Как маг: метеорит (с гербом метеорита — на 2); как огненная — взрыв (с огнём и мечом — на 2).
    let mut pos = crest_position(
        &[Crest::HatFlame, Crest::Meteor, Crest::FlameSword],
        Color::White,
    );
    pos.board[sq("e4") as usize] = Some(knight);
    pos.board[sq("e5") as usize] = Some(pawn_with(9 * unit));
    let meteors = pos.meteors();
    assert_eq!(meteors.len(), 1);
    assert_eq!(
        meteors[0]
            .hits
            .iter()
            .map(|h| (h.square, h.damage))
            .collect::<Vec<_>>(),
        vec![(sq("e5"), 2 * unit)]
    );
    let blasts: Vec<(Square, i32)> = pos
        .lava_blasts()
        .iter()
        .flat_map(|b| b.hits.iter())
        .map(|h| (h.square, h.damage))
        .collect();
    assert_eq!(blasts, vec![(sq("e5"), 2 * unit)]);
}

#[test]
fn rogue_like_gold_combined_piece_has_both_powers_and_their_crests() {
    let unit = crate::balance::UNIT;
    let stock = Stock::combo(Kind::Knight, Power::Gold, Power::Fire);
    let knight = stock.piece(Color::White);
    assert!(knight.gold && knight.fire && !knight.barbarian);
    assert_eq!(knight.title(), "Белый огненный золотой конь");
    assert_eq!(Stock::of(knight), stock);
    assert_eq!(
        Stock::combo(Kind::Bishop, Power::Gold, Power::Wizard).name(),
        "золотой слон-маг"
    );
    assert_eq!(
        Stock::combo(Kind::Rook, Power::Gold, Power::Ice).name(),
        "ледяная золотая ладья"
    );
    // Золотой варвар — по-прежнему варвар с золотом.
    let both = Stock::combo(Kind::Knight, Power::Barbarian, Power::Gold);
    assert_eq!(Stock::of(both.piece(Color::White)), both);
    assert_eq!(both.name(), "золотой варварский конь");
    // Выставление — своим номером, и по сети тоже.
    let mut pos = crest_position(&[Crest::KingFlame], Color::White);
    pos.give_stock(knight);
    assert!(pos.reserve(Color::White).contains(&(stock, 1, 0)));
    let legal = pos.legal_moves();
    let mv = *legal
        .iter()
        .find(|m| m.kind == stock.drop_kind())
        .expect("выставление золотой объединённой");
    assert_eq!(mv.kind, MoveKind::ComboDrop(Kind::Knight, 7));
    assert_eq!(
        crate::net::decode_move(&crate::net::encode_move(mv), &legal),
        Some(mv)
    );
    // Продаётся как золотая.
    assert_eq!(
        sell_price(stock),
        sell_price(Stock::new(Kind::Knight, Power::Gold))
    );

    // Как золотая: победа приносит золото сверху (с гербом короля — ещё); как огненная —
    // взрывается (с гербом огня и меча — на 2).
    let gold_gain = |piece: Piece| {
        let mut pos = crest_position(&[Crest::GoldKing, Crest::FlameSword], Color::White);
        pos.board[sq("e4") as usize] = Some(piece);
        pos.board[sq("d6") as usize] = Some(pawn_with(unit));
        pos.board[sq("e5") as usize] = Some(pawn_with(5 * unit));
        let before = pos.rogue().unwrap().gold;
        pos.strike_all();
        let gain = pos.rogue().unwrap().gold - before;
        let hits: Vec<(Square, i32)> = pos
            .lava_blasts()
            .iter()
            .flat_map(|b| b.hits.iter())
            .map(|h| (h.square, h.damage))
            .collect();
        (gain, hits)
    };
    let (plain, plain_hits) = gold_gain(fire(Kind::Knight));
    let (golden, golden_hits) = gold_gain(knight);
    assert_eq!(golden - plain, 2 + 1, "бонус золотой и герб короля");
    assert_eq!(plain_hits, vec![(sq("e5"), 2 * unit)]);
    assert_eq!(golden_hits, plain_hits);

    // Ледяная золотая: ударивший замерзает, герб снежинки и пешки — пешка как золотая.
    let mut pos = crest_position(&[Crest::KingSnow, Crest::Pawn], Color::Black);
    let pawn = Stock::combo(Kind::Pawn, Power::Gold, Power::Ice);
    assert_eq!(
        pos.shop_price(pawn),
        shop_price(Stock::new(Kind::Pawn, Power::Gold))
    );
    pos.board[sq("e4") as usize] =
        Some(Stock::combo(Kind::Rook, Power::Gold, Power::Ice).piece(Color::White));
    pos.board[sq("e6") as usize] = Some(Piece::new(Kind::Rook, Color::Black));
    let strikes = pos.strike_all();
    assert!(
        strikes.iter().any(|s| s.from == sq("e6") && s.rebound),
        "ударивший ледяную замерз"
    );
}

#[test]
fn rogue_like_combined_piece_is_stored_bought_and_dropped() {
    let stock = Stock::combo(Kind::Knight, Power::Barbarian, Power::Fire);
    let knight = stock.piece(Color::White);
    assert!(knight.barbarian && knight.fire && !knight.spark);
    assert_eq!(knight.title(), "Белый огненный варварский конь");
    assert_eq!(stock.name(), "огненный варварский конь");
    assert_eq!(
        Stock::combo(Kind::Bishop, Power::Barbarian, Power::Wizard).name(),
        "варварский слон-маг"
    );
    assert_eq!(Stock::of(knight), stock);
    assert_eq!(stock.drop_kind(), MoveKind::ComboDrop(Kind::Knight, 0));
    assert_eq!(MoveKind::ComboDrop(Kind::Knight, 0).stock(), Some(stock));
    // Цена — 300% обычной такой же.
    assert_eq!(
        shop_price(stock),
        shop_price(Stock::new(Kind::Knight, Power::Plain)) * 3
    );

    // Покупка — в запас, готовой; выставление — ходом, и по сети тоже.
    let mut pos = crest_position(&[Crest::AxeFlame], Color::White);
    let r = pos.rogue.as_mut().unwrap();
    r.shop = true;
    r.gold = 50;
    r.offers[0] = Some(stock);
    assert!(pos.buy(0).is_some());
    assert_eq!(pos.rogue().unwrap().gold, 50 - pos.shop_price(stock));
    assert!(pos.reserve(Color::White).contains(&(stock, 1, 0)));
    pos.rogue.as_mut().unwrap().shop = false;
    let legal = pos.legal_moves();
    let mv = *legal
        .iter()
        .find(|m| m.kind == stock.drop_kind())
        .expect("выставление объединённой");
    assert_eq!(
        crate::net::decode_move(&crate::net::encode_move(mv), &legal),
        Some(mv)
    );
    pos.make_move(mv);
    let placed = pos.piece(mv.to).unwrap();
    assert!(placed.barbarian && placed.fire);
    assert!(pos.reserve(Color::White).iter().all(|&(s, ..)| s != stock));
}

#[test]
fn rogue_like_combined_piece_has_both_powers_and_their_crests() {
    let unit = crate::balance::UNIT;
    // Топор и меч — герб варваров, огонь и меч — герб огненных: оба действуют.
    let mut pos = crest_position(&[Crest::AxeSword, Crest::FlameSword], Color::White);
    let knight =
        pos.crested(Stock::combo(Kind::Knight, Power::Barbarian, Power::Fire).piece(Color::White));
    let plain = Piece::new(Kind::Knight, Color::White);
    assert_eq!(knight.attack(), plain.attack() + 2 * unit);
    assert_eq!(knight.max_hp(), rogue_knight_hp() - unit);
    pos.board[sq("e4") as usize] = Some(knight);
    // Как варвар: убил d6 — бьёт f6 ещё раз.
    pos.board[sq("d6") as usize] = Some(pawn_with(3 * unit));
    pos.board[sq("f6") as usize] = Some(pawn_with(6 * unit));
    pos.board[sq("e5") as usize] = Some(pawn_with(5 * unit));
    let strikes: Vec<(Square, bool, bool)> = pos
        .strike_all()
        .iter()
        .filter(|s| s.from == sq("e4"))
        .map(|s| (s.hit.square, s.hit.killed, s.again))
        .collect();
    assert_eq!(
        strikes,
        vec![
            (sq("d6"), true, false),
            (sq("f6"), false, false),
            (sq("f6"), true, true)
        ]
    );
    // Как огненная: взрывается, и с гербом огня и меча — на 2.
    let hits: Vec<(Square, i32)> = pos
        .lava_blasts()
        .iter()
        .flat_map(|b| b.hits.iter())
        .map(|h| (h.square, h.damage))
        .collect();
    assert_eq!(hits, vec![(sq("e5"), 2 * unit)]);

    // Скидки гербов — и на объединённые: огонь и пешка (−2), топор и ферзь (−6).
    let pos = crest_position(&[Crest::FlamePawn, Crest::AxeQueen], Color::White);
    let pawn = Stock::combo(Kind::Pawn, Power::Barbarian, Power::Fire);
    assert_eq!(pos.shop_price(pawn), shop_price(pawn) - 2);
    let queen = Stock::combo(Kind::Queen, Power::Barbarian, Power::Ice);
    assert_eq!(pos.shop_price(queen), shop_price(queen) - 6);
    // Золотой варвар продаётся как золотой, шанс ледяных растёт и у ледяных варваров.
    let gold = Stock::combo(Kind::Rook, Power::Barbarian, Power::Gold);
    assert_eq!(
        sell_price(gold),
        sell_price(Stock::new(Kind::Rook, Power::Gold))
    );
    let share = |crests: &[Crest]| {
        let mut pos = crest_position(crests, Color::White);
        let (mut found, mut all) = (0.0f64, 0.0f64);
        for _ in 0..2000 {
            pos.roll_offers();
            for stock in pos.offers().into_iter().flatten() {
                all += 1.0;
                if stock.with == Power::Ice {
                    found += 1.0;
                }
            }
        }
        found / all
    };
    assert!(share(&[Crest::AxeSnow, Crest::Coin]) > share(&[Crest::AxeSnow]) * 1.5);
}

/// Проверки rogue-like после каждого действия: фигуры живы и не выше полного здоровья, короли на
/// месте, уровни и волны в пределах, золото не перевалило через ноль, каждый ход переживает сеть,
/// запас собирается обратно в те же фигуры, описания последних записей строятся.
fn assert_rogue_sane(game: &Game, context: &str) {
    let pos = game.position();
    for (sq, p) in pos.pieces() {
        assert!(p.hp > 0, "{context}: {} на {sq} с {} HP", p.title(), p.hp);
        assert!(
            p.hp <= p.max_hp(),
            "{context}: {} на {sq} — {} HP из {}",
            p.title(),
            p.hp,
            p.max_hp()
        );
    }
    for color in [Color::White, Color::Black] {
        if let Some(king) = pos.king_square(color) {
            let piece = pos.piece(king).expect("король на своей клетке");
            assert!(
                piece.kind == Kind::King && piece.color == color && !piece.boss,
                "{context}: на клетке короля {}",
                piece.title()
            );
        }
    }
    let r = pos.rogue().expect("rogue-like");
    assert!(r.wave <= r.waves && r.level <= r.levels, "{context}: {r:?}");
    assert!(r.gold < 30_000, "{context}: золото {}", r.gold);
    let legal = game.legal_moves();
    for &mv in legal {
        assert_eq!(
            crate::net::decode_move(&crate::net::encode_move(mv), legal),
            Some(mv),
            "{context}: ход по сети"
        );
    }
    for color in [Color::White, Color::Black] {
        for (stock, ready, wait) in pos.reserve(color) {
            assert!(ready + wait > 0);
            assert_eq!(Stock::of(stock.piece(color)), stock, "{context}: запас");
        }
    }
    for record in game.history().iter().rev().take(2) {
        let _ = record.description();
        let _ = record.notation();
    }
}

/// Долгий прогон rogue-like: случайные гербы (иногда все), объединённые фигуры в запасе, магазин,
/// продажа, гербы, расстановка, «Выиграть раунд», отмена — после каждого действия
/// `assert_rogue_sane`, а отмена возвращает позицию в точности. `ROGUE_FUZZ_GAMES` — число партий
/// (по умолчанию 40); долгий прогон — `cargo test --release rogue_like_fuzz -- --ignored`.
fn rogue_fuzz(games: u64) {
    let mut seed = 0x5EED_F00D_CAFE_0001u64;
    let mut next = move |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n.max(1) as u64) as usize
    };
    let specials = [
        Power::Fire,
        Power::Spark,
        Power::Wind,
        Power::Gold,
        Power::Shade,
        Power::Ice,
        Power::Barbarian,
        Power::Wizard,
    ];
    let (mut levels, mut shops, mut undos, mut actions) = (0u32, 0u32, 0u32, 0u32);
    let (mut wins, mut losses, mut deepest) = (0u32, 0u32, 0u8);
    for g in 0..games {
        let mut pos = Position::rogue_start(g * 7919 + 3);
        let r = pos.rogue.as_mut().unwrap();
        // Гербы: в каждой четвёртой партии — все, иначе случайные.
        let mut crests: Vec<Crest> = Crest::ALL.to_vec();
        if g % 4 != 0 {
            crests.retain(|_| next(3) == 0);
        }
        for (slot, &crest) in r.crests.iter_mut().zip(&crests) {
            *slot = Some(crest);
        }
        r.gold = next(60) as u16;
        // В запасе — несколько объединённых и особых.
        for _ in 0..next(5) {
            let kind = [
                Kind::Pawn,
                Kind::Knight,
                Kind::Bishop,
                Kind::Rook,
                Kind::Queen,
            ][next(5)];
            let stock = if next(2) == 0 {
                let (base, with) = COMBOS[next(COMBOS.len())];
                Stock::combo(kind, base, with)
            } else {
                Stock::new(kind, specials[next(specials.len())])
            };
            if stock.has(Power::Wizard) && kind == Kind::Pawn {
                continue;
            }
            pos.give_stock(stock.piece(Color::White));
        }
        let mut game = Game::from_position(pos);
        assert_rogue_sane(&game, "начало");
        for step in 0..1500 {
            if game.is_over() {
                break;
            }
            actions += 1;
            let context = format!("партия {g}, шаг {step}");
            // Изредка: действие и сразу отмена — позиция возвращается в точности.
            let probe = next(12) == 0;
            let before = (game.position().clone(), game.history().len());
            let pos = game.position();
            let moves = game.legal_moves().to_vec();
            if pos.shop_open() {
                shops += 1;
                let ready: Vec<Stock> = pos
                    .reserve(Color::White)
                    .into_iter()
                    .filter(|&(_, ready, _)| ready > 0)
                    .map(|(stock, ..)| stock)
                    .collect();
                match next(9) {
                    0 | 1 => {
                        game.buy(next(5));
                    }
                    2 => {
                        game.buy_crest(next(3));
                    }
                    3 => {
                        game.reroll();
                    }
                    4 => {
                        game.reroll_crests();
                    }
                    5 if !ready.is_empty() => {
                        game.sell(ready[next(ready.len())]);
                    }
                    _ => {
                        assert!(game.leave_shop(), "{context}: из магазина не уйти");
                    }
                }
            } else if game.position().side() == Color::White && next(40) == 0 {
                if game.win_round() {
                    levels += 1;
                }
            } else if !game.history().is_empty() && next(25) == 0 {
                assert!(game.undo());
                undos += 1;
            } else if moves.is_empty() || (pos.moved() || pos.planning()) && next(3) == 0 {
                assert!(game.end_turn(), "{context}: ход не закончить");
            } else {
                let mv = moves[next(moves.len())];
                assert!(game.play(mv).is_some(), "{context}: ход {mv:?} не сыгран");
            }
            assert_rogue_sane(&game, &context);
            if probe && game.history().len() > before.1 {
                while game.history().len() > before.1 {
                    assert!(game.undo());
                }
                assert!(
                    *game.position() == before.0,
                    "{context}: отмена вернула не ту позицию"
                );
                undos += 1;
            }
        }
        let r = game.position().rogue().unwrap();
        deepest = deepest.max(r.level);
        match game.status() {
            Status::Victory {
                winner: Color::White,
            } => wins += 1,
            Status::Victory { .. } => losses += 1,
            _ => {}
        }
    }
    eprintln!(
        "rogue-like fuzz: партий {games}, действий {actions}, в магазине {shops}, отмен {undos}, \
         выигранных раундов {levels}, побед {wins}, поражений {losses}, дальше всех — уровень \
         {deepest}"
    );
}

#[test]
fn rogue_like_fuzz_keeps_the_game_sane() {
    let games = std::env::var("ROGUE_FUZZ_GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40);
    rogue_fuzz(games);
}

#[test]
#[ignore]
fn rogue_like_fuzz_long() {
    rogue_fuzz(600);
}
