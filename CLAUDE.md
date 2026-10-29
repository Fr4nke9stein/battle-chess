# Боевые шахматы

Шахматы на Rust, у фигур есть атака, защита и здоровье. Правила и характеристики —
в README.md. Пользователь играет через ярлык на рабочем столе, общение — по-русски.

## Как обновлять игру

1. Изменить код, поднять `version` в Cargo.toml — версия видна в заголовке окна.
2. `cargo test --release` (debug-кэш удалён, release-зависимости уже собраны).
3. `powershell -NoProfile -ExecutionPolicy Bypass -File install.ps1` — собирает
   release, кладёт exe в `%LOCALAPPDATA%\Programs\BattleChess` и обновляет ярлык
   «Боевые шахматы» на рабочем столе. Открытую игру закрывать не нужно: запущенный
   exe переименовывается в `*.old`, новая версия стартует со следующего запуска.

В PATH старых терминалов может не быть cargo: добавлять `$env:USERPROFILE\.cargo\bin`.

Код — в приватном репозитории https://github.com/Fr4nke9stein/battle-chess (с 2026-10-16,
ветка `main`). Git и GitHub CLI поставлены через winget; в старых терминалах их может не быть в
PATH (`C:\Program Files\Git\cmd`, `C:\Program Files\GitHub CLI`). В репозитории
`core.autocrlf = false` (файлы — с LF), коммиты — от `Fr4nke9stein` с адресом GitHub
`66310481+Fr4nke9stein@users.noreply.github.com`. Не входят: `target` и старая копия
`battle-chess/` внутри проекта.

## Тулчейн: rustup stable-x86_64-pc-windows-gnu без Visual Studio и MinGW

- В Cargo.toml закреплены `parking_lot_core =0.9.10` и `libloading =0.8.8`: новые
  версии линкуются через raw-dylib, а для этого нужны dlltool и ассемблер.
- `build.rs` добавляет System32 в конец путей линкера (нет `libshlwapi.a`).
- egui/eframe закреплены на 0.31: в 0.34+ другое API (`App::ui`, `Panel`).

## Код

- **0.40 — поворот к рогалику (просьба пользователя, 2026-10-16):** каждая фигура в конце хода бьёт
  **всех** чужих в досягаемости (`strike_all` → `strike` на каждую цель) и **остаётся на месте**
  (убийца больше не встаёт на клетку цели, превращения через удар нет; `StrikeAnim::moving` всегда
  `false`). Слон бьёт по диагонали на 2 клетки (`BISHOP_STRIKES`, без преграды). Невидимость коней:
  кнопка «Кони: инвиз» под «Конец хода» (`cloak_rect`/`paint_cloak`/`over_cloak` в `table.rs`),
  `Position::{cloak, cloak_cooldown, cloak_knights, can_cloak, cloak_state, cloaked, ambush}` —
  `CLOAK_TURNS` = 3 начала своих ходов невидимы, потом `CLOAK_COOLDOWN` = 3 перезарядки
  (`tick_cloak` в `end_turn`). Невидимых коней не бьют удары, молния, лава, `any_attacker`; их
  клетка для соперника — пустая: ход на неё — засада (`Outcome::ambush`: удар атакой ходящей;
  убил — встаёт туда, нет — остаётся на месте), сквозь коня не пройти. Компьютер на такие клетки
  не ходит (фильтр в `ai.rs`). В игре невидимые кони полупрозрачные (`dressed`). Карты убраны из
  интерфейса: партия — `Position::start().without_cards()`, `SHOW_CARDS = false` в `table.rs`,
  в меню нет «Выбрать колоду»/«Коллекция карт», в панели сторон вместо карт — невидимость. Код
  карт остался (тесты его проверяют). **0.41** (просьба пользователя): ладья бьёт по прямой на 3
  клетки (`ROOK_STRIKES`), ферзь — на 3 во все 8 сторон (`QUEEN_STRIKES`; `any_attacker` перебирает
  его шаги и прыжки коня); электричества слонов у усиленного компьютера нет («слишком op»:
  `boost` даёт только лаву; `Piece::spark` и молния остались, в тестах — `set_spark`). **0.42**:
  слон, ладья и ферзь бьют только первую фигуру на каждой линии (`line_clear`: своя фигура и барьер
  закрывают линию, невидимый конь соперника — нет; в `strike_all` и `any_attacker`). **0.43**: ладья
  и ферзь бьют на 2 клетки (`ROOK_STRIKES` 8, `QUEEN_STRIKES` 16). «Шведка»: убитая фигура (кроме
  короля) уходит в запас убийце (`score_kill` → `Position::reserve` [сторона][`Kind`][ждать 0..=2],
  `RESERVE_WAIT` = 2, `tick_reserve` в начале своего хода), готовую выставляют вместо хода
  (`MoveKind::Drop(kind)`, `from` = `to`, `Move::is_drop`; `drop_squares` — пустые клетки своей
  половины вне барьера, пешку не на 1-ю горизонталь); выставленная `Piece::fresh` в конце этого
  хода не бьёт. Компьютер выставляет сам (ходы в `pseudo_moves`, запас — полцены в `evaluate`).
  В игре: запас — ячейки внизу слева (соперника — отражение вверху справа; `RESERVE_*`,
  `reserve_slots`/`paint_reserve`/`reserve_under`/`reserve_usable` в `table.rs`), щелчок выбирает
  (`drop_pick`), подсвечены клетки, щелчок по клетке выставляет; «×N» — готово, «+N» — ждут.
  **0.44** (просьбы пользователя): гибель короля — поражение (`Position::king_lost`, ставит `fall`;
  `refresh` и `ai::decided`); фигура возвращается из запаса не больше `MAX_REBIRTHS` = 2 раз
  (`Piece::reborn`, запас — ещё и по числу возвращений); перехват — чужие кони и пешки сразу бьют
  фигуру, вставшую ходом или выставлением в их досягаемость (`intercept` в `make_move`,
  `Outcome::reactions`, `Record::strikes` отдаёт их у хода — анимация та же); конь — 900 HP.
  Невидимость коней убрана целиком (кнопка, `cloak*`, засада). Единица `balance::UNIT` = 300:
  правила считают прежними числами, игрок видит делённые (`shown` — на экране и в описаниях),
  в `balance.ini` атака, здоровье и `lava_damage` — в единицах (`to_file`/`from_file`; число от
  100 — старый файл, читается как есть). Числа карт в файле — прежние. Файл пользователя на Mac
  переведён в единицы, копия — `balance.ini.before-0.44`.
  **0.45**: пешка перехватывает только на проходе — чужую пешку, прошедшую двойным ходом через
  клетку её удара (`intercept(sq, passed)`); конь — как раньше, любую вставшую в досягаемость.
  **0.46** (просьба пользователя, «вариант В»): партия «с перемещением при взятии» —
  `Position::with_advance`/`advance`, `Settings::advance` (кнопка «Взятие: …» в меню и на экране
  мультиплеера, «Взятие» в «Новой партии»). В `strike_all` ближник (пешка, конь, король), убивший
  в этот ход, встаёт на клетку самой дорогой из убитых им (`advance_after`: равенство — первая по
  очереди, пешка на последней — ферзь, лёд замораживает, маска `done` — второй раз не бьёт);
  дальнобойные стоят. Удар с перемещением — `Strike::advanced`, он последний среди ударов фигуры;
  в `app.rs` — `StrikeAnim::moving`/`killer_at` (до удара фигура рисуется на своей клетке, с
  выпадами остальных ударов). В сети правила шлют белые (первый зашедший) сразу после START:
  `RULES advance=0|1` (`net::rules_line`/`parse_rules`); чёрные ждут их (`net_pending`).
  **0.47** (просьба пользователя): цели фигура бьёт по порядку — дороже (`Kind::points`), из
  равных — с меньшим здоровьем, потом по очереди клеток. В партии «с перемещением» первое
  убийство вблизи (`close_strike`: у пешки, коня, короля — любое, у слона, ладьи, ферзя — по
  соседней клетке) ставит фигуру на клетку убитой (`advance_to`), и больше она в этот ход не
  бьёт (`advance_after` и выбор «самой дорогой из убитых» убраны).
  **0.48**: в окно вписывается не стол с картами (`TABLE_MIN`/`TABLE_MAX` — только для тестов
  раскладки), а `VIEW_MIN`…`VIEW_MAX` (доска, кнопка, запас у углов: `RESERVE_AT` = (−62, 310),
  ячейка 40×58, фигура ×0,6); целый масштаб — только если мельчит не больше чем на 8%, иначе
  дробный. Свой масштаб доски — `Settings::zoom` (50–250%, `zoom=` в settings.ini): ползунок в
  «Настройках» и Ctrl/⌘ + колесо над доской. Подписи «Взятие» короткие («на месте» / «с
  перемещением»): длинные раздували левую панель — появлялась чёрная полоса.
  **0.49**: пульс мультиплеера — игра шлёт `PING` раз в 15 с (`net::heartbeat`), сервер — раз
  в 20 с (`pulse`) и не пересылает чужой `PING`; нет ничего от сервера 60 с (`SILENCE`, таймаут
  чтения) — «сервер не отвечает». Причина: роутер забывал молчащее TCP-соединение, пока
  соперник думал, — ход висел неподтверждённым на сервере (`ss -tni`: `unacked`, `backoff`), и
  никто не знал о разрыве. Игра берёт в очередь только `M …` и `E`. Сервер пользователя:
  reemoat = `root@188.245.69.74` (ключ `~/.ssh/reemoat`, в ssh-config у хоста записан `rends` —
  не пускает), порт 14888, только `/root/mp_server.py` и процесс `setsid -f nohup python3
  /root/mp_server.py 14888` без лога и службы (просьба пользователя: ничего лишнего на VPS);
  после перезагрузки VPS не поднимается. Останавливать — `pkill -f "^python3 /root/mp_server"`
  (шаблон с портом убивает и саму ssh-команду).
  **0.50** (просьба пользователя): rogue-like режим — кнопка в меню (`Choice::Rogue` →
  `new_rogue_game`), `Position::rogue_start(seed)` (белые — обычная армия, в запасе 2 пешки,
  слон, конь; у чёрных фигур нет), `Position::rogue: Option<Rogue>` (`player`, `wave`/`waves`,
  `since` — ходов соперника с прошлой волны, `seed`). Волны — `spawn_wave` (число и веса —
  `Rule::Rogue*` в балансе, случайность — зерно партии и ключ позиции; пешки — на вторую с края
  горизонталь, остальные — на крайнюю, нет места — ближе), в `end_turn`: соперник начинает ход
  без фигур — волна сразу (`fresh`, в этот ход не бьёт), игрок начинает ход, а `since` ≥
  `RogueWaveTurns` — тоже; `TurnEnd::spawned` (клетка, вид), в описании «Вышла волна N из M», в
  нотации «Волна N». Несколько ходов за ход: `moves_made`/`used` (маска клеток ходивших фигур),
  `moves_per_turn` (`RogueMoves` у игрока, иначе 1), `moves_left`, `used(sq)`; `pseudo_moves`
  пропускает ходивших; `Game::play` кончает ход сам, только когда `legal` пуст;
  `Game::can_end_turn` — ещё и после первого хода. «Шведки» нет (`score_kill`). Поражение —
  `Position::defeated` (соперник в rogue-like — когда все волны вышли и фигур нет; так же в
  `ai::decided`); ничьих нет. В приложении: проявление волны (`appears`, `APPEAR_TIME`,
  `appear_alpha`), роль соперника «волны из тумана · N из M», итог — `result_text(status, rogue)`,
  строка хода «ещё ходов: N». Туман войны — на будущее (не сделан). **0.51** (просьба
  пользователя): фигуры волн сами не ходят (`pseudo_moves` пуст у соперника в rogue-like —
  компьютер просто кончает ход), а в `end_turn` до ударов `march`: все свободные шагают на
  клетку вперёд, передние первыми (колонна идёт целиком), занято/барьер/край — стоят, лёд
  замораживает; `TurnEnd::marched`, в описании «Волны шагнули вперёд», в нотации «Волна ↓N»;
  в приложении `marches` + `march_shift`, удары ждут `SLIDE_TIME`. **0.52** (просьба
  пользователя): волна выходит в конце каждого хода соперника (в `end_turn` после ударов и
  взрывов), только на его крайнюю горизонталь (`home_rank`), сколько поместится; не
  поместилась ни одна — волна не считается. Таймера (`Rogue::since`, `RogueWaveTurns`) и выхода
  «в пустой ход» больше нет; строка `rogue_wave_turns` старого `balance.ini` молча
  пропускается (`RETIRED_RULES` в `balance.rs`). **0.53** (просьба пользователя): в конце хода
  соперника сначала удары (и лава), потом `march`, потом волна; в приложении шаг — после
  ударов (`march_origin` — выпад с прежней клетки), волна проявляется после шага. Фигура волны
  на первой горизонтали игрока — его поражение (`Position::breached` в `defeated`; в описании
  «Волна дошла до первой горизонтали», в итоге — «Поражение»). **0.54** (просьба пользователя):
  уровни — `Rogue::{level, levels}` (`Rule::RogueLevels` = 10), волн на уровне — `waves_at`
  (`RogueWaves` + уровень − 1). В `end_turn` игрока: все волны уровня вышли и чужих фигур нет,
  уровень не последний — следующий уровень, его первая волна сразу (`TurnEnd::level_up`, в
  описании «уровень N!», в нотации «Уровень N»); армия и запас игрока остаются. Вес пешки —
  `RogueWeightPawn` − `RoguePawnDrop` × (уровень − 1). Победа (`defeated`) — только на
  последнем уровне. Роль соперника — «уровень L из 10 · волна N из M». **0.55** (просьба
  пользователя, только rogue-like): кони не перехватывают (`intercept` в rogue-like — только
  пешки на проходе); вес пешки 15 (×1,5), `RoguePawnDrop` = 2 (с 9-го уровня пешек нет; в
  `balance.ini` пользователя стояли прежние стандартные 10/1 — заменены, копия
  `balance.ini.before-0.55`); крайняя горизонталь занята — фигура волны выходит на ближайшую
  горизонталь со свободной клеткой (`spawn_wave`: 8-я, 7-я, … 2-я, на 1-ю игрока — нет).
  **0.59** (просьба пользователя): расстановка между уровнями. `Rogue::planning` и
  `Rogue::origin` (клетка начала уровня у каждой фигуры игрока, `NO_ORIGIN` — выставлена из
  запаса; `relocate` переносит метку, выставление её стирает, `origins` — снимок). В `end_turn`
  игрока уровень пройден → `begin_planning` (выжившие — на клетки начала уровня с полным
  здоровьем, `Piece::new` + `reborn`; без метки — в запас; `TurnEnd::{planning, returned}`),
  ход остаётся у игрока. В расстановке `pseudo_moves` → `arrange_moves` (`arrange_fits`: своя
  половина, пешка — только 2-я горизонталь): `Quiet` на пустую клетку или на свою фигуру —
  обмен; `Drop` — запас; `make_move` → `arrange` (ходов не тратит, без перехвата; обмен — в
  `Outcome::rook`, анимация как у ладьи рокировки). `Game::can_end_turn` в расстановке — всегда,
  `Game::play` её сам не кончает; `end_turn` в расстановке → `start_round` (снимок `origin`,
  уровень +1, права рокировки снова, первая волна, ходит игрок; `TurnEnd::level_up`). Нотация
  «Кg1→d4», «Кg1⇄Сf1», описание «Расстановка: …». В приложении: кнопка «Начать раунд» —
  `start_round_*.png` из `tools/breach_ui.py` (`START_LABEL`, тот же размер 53×35; буквы Р, Т,
  У, Ч, Ь дорисованы), `UiTextures::button(look, start)`; возвращение фигур — те же `marches`;
  строка хода и роль соперника — «расстановка перед уровнем N»; подсветка последнего хода до
  расстановки не показывается.
  **0.60** (просьба пользователя): золото и магазин. `Rogue::gold` — в `score_kill` за чужую
  фигуру `rogue_price(kind)` (`Rule::RogueGold*`: 1/2/2/3/4, он же цена в магазине);
  `TurnEnd::gold` — за удары этого конца хода («Золото: +N (всего M)»). `begin_planning`
  открывает магазин (`Rogue::shop`, `offers` — `Rule::RogueShopSlots` = 3 из `SHOP_MAX` = 5,
  `roll_offers`: виды без короля поровну, случайность — зерно и счёт `rolls`); пока он открыт,
  ходов нет и `can_end_turn` = false. Действия — `Event::Rogue(RogueAction)` через
  `Game::rogue_action` (в истории, «Ход назад» — по одному): `Game::buy(slot)` (в запас,
  готовой), `reroll` (`Rule::RogueRerollCost` = 1), `leave_shop` (→ расстановка), `win_round`
  (кнопка «Выиграть раунд» в правой панели, `rogue_controls`: фигуры волн убираются за золото,
  `wave = waves`, потом `finish_turn`; на последнем уровне — сразу победа; в `undo` приложения
  отменяется вместе с концом хода). Интерфейс (`table.rs`): золото — `coin_32.png` из
  арт-проекта (`tools/breach_coin.py`, просьба пользователя: самая большая) в `GOLD_AT`, число
  ×`POINTS_FONT` над ней (`paint_gold`, `theme::GOLD`); магазин — панель посреди доски
  (`shop_layout`/`shop_under`/`shop_usable`/`paint_shop`, `ShopSpot`, цены — `coin_16.png`,
  `paint_price`), доска под ним притушена, запас и золото — нет; щелчки в магазине —
  в начале `handle_input` (`shop_hover`).
  **0.61** (просьба пользователя): цена в магазине — `shop_price` = `rogue_price` ×
  `Rule::RogueShopPercent` (200%), обновление — 2 (в `balance.ini` пользователя стояло 1 —
  заменено, копия `balance.ini.before-0.61`); расстановка — только на две первые горизонтали
  (`arrange_fits`: `row < 2`, пешка — `row == 1`).
  **0.62** (просьба пользователя): магазин показывается и ловит щелчки, только когда доиграли
  удары, шаги и ход (`shop_ready`). Исправлена анимация: если на клетку убитой фигуры в тот же
  конец хода шагнула фигура волны (или вернулась к расстановке фигура игрока), до касания
  удара на клетке рисовалась убитая вместо пришедшей, а сама убитая пропадала — теперь
  пришедшую (`march_origin(sq) != sq`) не подменяет `struck_piece`, а убитая стоит до
  касания.
  **0.63** (просьба пользователя): прилавок — по весам волн первого уровня (`shop_weights`), и
  огненные фигуры (`Piece::fire`): у каждого вида огненная — вес ÷ `FIRE_RARITY` = 6, цена ×
  `Rule::RogueFirePercent` (200%). `Stock { kind, fire }` — фигура запаса и прилавка
  (`offers: [Option<Stock>; 5]`, `RogueAction::Buy { stock, .. }`); огненные — в
  `Position::fire_reserve` (готовы всегда), `reserve()` отдаёт `(Stock, готово, ждёт)` (огненные
  первыми), `take_stock`/`give_stock`, выставление — `MoveKind::FireDrop(kind)` (`Stock::drop_kind`,
  `MoveKind::stock`; в сети — код 20 + вид). `lava_blasts`: взрываются и `fire` (огонь не уходит,
  только что выставленная — нет); `begin_planning` сохраняет огонь (`Piece::with`). Анимация:
  полосы лавы всех видов из арт-проекта (`LAVA_PNG` — 12, `UiTextures::lava(color, kind, frame)`),
  огненная — кадры ожидания, после взрыва снова насыщение; в запасе и магазине —
  `paint_stock`; `drop_pick`/`reserve_hover` — по `Stock`; `Piece::title` — «Белый огненный
  конь».
  **0.64** (просьба пользователя): электрические фигуры в магазине — `Power { Plain, Fire, Spark }`
  (`Stock { kind, power }`, `Stock::of`/`piece`, `Power::adjective`), вес как у огненной,
  цена × `Rule::RogueSparkPercent` (200%), запас — `Position::spark_reserve`
  (`charged`/`charged_mut`), выставление — `MoveKind::SparkDrop` (в сети код 30 + вид). Работают
  через `Piece::spark` (молния в `strike` — у любого вида). Анимация: `electric_hold_*` всех
  видов из арт-проекта (`ELECTRIC_PNG` — 12, `UiTextures::electric(color, kind, frame)`),
  `spark_frame` — без проверки на слона; в запасе и магазине — `paint_stock` (обрезано по
  ячейке).
  **0.65** (просьба пользователя): электричество — спокойный вариант `electric_calm_*` из
  арт-проекта (`ELECTRIC_FRAMES` = 62, `ELECTRIC_LOOP` = [14, 62]; повтор сдвинут по клетке —
  `spark_frame`, `phase`). Продажа: `RogueAction::Sell { stock, price }`, `Position::sell`/
  `Game::sell` — готовая фигура запаса за `shop_price`, только пока открыт магазин; в
  интерфейсе — зона `ShopSpot::Sell` справа от панели (`SELL_WIDTH`), фигуру тянут из запаса
  (`sell_drag`, `reserve_sellable`; во время магазина ячейки своего запаса подсвечены),
  отпущенная над зоной продаётся. **0.66** (просьба пользователя): электричество — снова
  `electric_hold_*` (последняя версия, без кольца во вспышке), зона продажи — слева от
  магазина.
  **0.67** (просьба пользователя): воздушные фигуры — `Power::Wind`/`Piece::wind`, вес и цена как
  у огненных (`Rule::RogueWindPercent` = 200%), запас `wind_reserve`, `MoveKind::WindDrop` (в сети
  40 + вид). Два хода за один: `Position::airborne` — маска воздушных, сходивших раз; в
  `make_move` первый ход тратит `moves_made` и ставит бит вместо `used`, второй (с клетки из
  маски) — бесплатный, потом `used`; `pseudo_moves` при исчерпанных ходах даёт ходы только
  фигурам из маски (выставлений — нет); `can_move(sq)` — для выбора фигуры в приложении; маска
  сбрасывается там же, где `used`. Анимация — `wind_*` из арт-проекта (`tools/breach_wind.py`,
  40 кадров, `WIND_LOOP` = [8, 40], кадр и опора как у электричества): `Figure::wind`,
  `wind_frame` (повтор сдвинут по клетке), в запасе и магазине — `paint_stock`.
  **0.68** (просьба пользователя): золотые фигуры — `Power::Gold`/`Piece::gold`, вес как у
  остальных особых, цена × `Rule::RogueGoldPiecePercent` (400% — вдвое дороже огненной), запас
  `gold_reserve`, `MoveKind::GoldDrop` (в сети 50 + вид). В `strike`: убила золотая фигура
  игрока — ещё `Rule::RogueGoldBonus` (2) золота (входит в `TurnEnd::gold`). Спрайты — вариант
  `gold` из арт-проекта (`tools/breach_gilded.py`, целиком золотые; `pieces/gold/`):
  `GOLD_BODIES`/`GOLD_SHADOWS` в `assets.rs`, `Textures::piece_body`/`piece_shadow` (по
  `Piece::gold`) — на доске, при перетаскивании, во льду и портале, в запасе и магазине;
  хитбокс — по обычному силуэту. **0.69** (уточнение пользователя): золотая в магазине стоит как
  другие особые (`RogueGoldPiecePercent` = 200; в `balance.ini` пользователя стояло 400 — заменено,
  копия `balance.ini.before-0.69`), зато продаётся дороже: `sell_price` — цена обычной такой же в
  магазине, у золотой × `Rule::RogueGoldSellPercent` (200%); огненные, электрические и воздушные
  продаются как обычные (до 0.69 особые продавались по своей цене).
  **0.70** (просьба пользователя): теневые фигуры — `Power::Shade`/`Piece::shade`, вес и цена как
  у остальных особых (`Rule::RogueShadePercent` = 200%), запас `shade_reserve`,
  `MoveKind::ShadeDrop` (в сети 60 + вид). Ходы — `shade_moves` (в `pseudo_moves` вместо обычных,
  кроме короля): сквозь фигуры на пустые клетки — слон/ладья/ферзь по всей линии, конь — 16
  клеток на расстоянии 2, пешка — вперёд и через одну фигуру перед собой (с начальной — на две).
  Анимация из арт-проекта: аура `dark_fire_*` (`tools/breach_darkfire.py`, `DARK_LOOP` = [8, 40]),
  ход — теневой шаг (`tools/breach_shadowstep.py`): `MoveAnim::step` (длится
  (`STEP_IN_AT` + `STEP_IN`) кадров), исчезновение `shadowstep_out_*` (`STEP_OUT` = 18) на исходной
  клетке и с кадра `STEP_IN_AT` = 7 появление `shadowstep_in_*` (`STEP_IN` = 13) на целевой, без
  теней; `Figure::dark` (`Dark::{Aura, Out, In}`), `UiTextures::dark(phase, …)`; в запасе и
  магазине — аура в `paint_stock`. **0.71** (просьба пользователя): теневой шаг вдвое быстрее — кадр
  `STEP_FRAME_TIME` = 30 мс (аура — по-прежнему 60).
  **0.72** (просьба пользователя): ледяные фигуры — `Power::Ice`/`Piece::icy`, вес и цена как у
  остальных особых (`Rule::RogueIcePercent` = 200%), запас `ice_reserve`, `MoveKind::IceDrop` (в
  сети 70 + вид). В `strike`: цель ледяная — ударивший замерзает (`frozen` = 3: конец этого хода,
  начало и конец следующего — пропускает весь следующий ход), `Strike::rebound`; в `strike_all`
  замёрзший больше в этот ход не бьёт, `march` его не двигает; в описании «… замёрз о лёд», в
  приложении — заморозка ударившего в момент касания (`chilled` в `queue_strikes`). Спрайты —
  `tools/breach_ice_pieces.py` (`pieces/ice/`): `ICE_BODIES`/`ICE_SHADOWS`, `piece_body`/
  `piece_shadow` по `Piece::icy`.
  **0.73** (просьба пользователя): король-толстосум — `Piece::boss` (вид `King`), выходит первым в
  последней волне первого уровня (`spawn_wave`: `level == 1 && wave + 1 == waves`, клетка ближе к
  середине крайней горизонтали). `max_hp`/`attack` — `Rule::RogueBossHp`/`RogueBossAttack` (10 и 1
  единиц, × `UNIT`), удары — `BOSS_STRIKES` (16 клеток без преград, `strike_steps`), золото —
  `kill_gold` (`Rule::RogueBossGold` = 10). Не король стороны: `march` и `fall` не трогают
  `kings`/`king_lost`. Спрайт — `tools/breach_fat_king.py` (`pieces/fat_king/`, `FAT_KING`,
  `piece_body`/`piece_shadow`); `Piece::title` — «Чёрный король-толстосум», в описании волны —
  «король-толстосум»; карточка фигуры в панели — по `attack()`/`max_hp()`.
  **0.74** — главное меню из арт-проекта (книга при свече, закладки-кнопки, открытие книги
  перед партией, окно «Настройки»); **0.75** (просьба пользователя: «убери новое меню, верни как
  было») — откатено целиком, меню снова прежнее (`title_screen`).
  **0.76** (просьба пользователя): огненная фигура взрывается, только если волна заденет чужую
  фигуру (`blast_reaches` в `lava_blasts`); нет — не взрывается и остаётся огненной.
  **0.77** (просьба пользователя): гербы — `Crest` (`ALL`, `title`, `description` с числами
  баланса), у игрока — `Rogue::crests` (в порядке покупки, до конца забега), прилавок —
  `Rogue::crest_offers` (`Rule::RogueCrestSlots` = 3, `roll_crests` в `begin_planning`: разные, из
  некупленных; случайность — зерно и `crest_rolls`), `RogueAction::{BuyCrest, RerollCrests}`
  (`Game::buy_crest`/`reroll_crests`; цена `crest_price` = `Rule::RogueCrestPrice` 15, обновление
  `crest_reroll_cost` = 5). Действие — `crest_for(color, crest)` (только сторона игрока): `Heart` —
  `crested` даёт ледяной `bonus_hp` (`RogueCrestHeartHp` = 2 единицы; в `take_stock`, при
  возвращении к расстановке и при покупке — фигурам на доске); `Coin` — вес ледяных в
  `roll_offers` × `RogueCrestCoinPercent` (веса там ×100); `Snow2` — отскок замораживает на
  `1 + 2 × RogueCrestFreezeTurns` границ; `Pawn` — `Position::shop_price` (ледяная пешка — как
  обычная; свободная `shop_price` — без гербов); `Spikes` — `Strike::thorns` (урон ударившему ×
  `RogueCrestSpikesPercent`, погиб — золото игроку, в `strike_all` погибший больше не бьёт;
  `Strike::wounded` = `hits` + шипы — для анимации: цифра урона и погибший о шипы стоит с выпадом
  до касания); `Crystal` — в `strike`: ледяная во льду урона не получает, удар до
  `RogueCrestCrystalHp` (1 ед.) и ниже оставляет ровно столько и замораживает (`frozen` = 3,
  `chilled`). Интерфейс (`table.rs`): панель «ГЕРБЫ» справа от магазина (`shop_layout` →
  `[панель, гербы]`, вся группа с продажей — по центру доски; `ShopSpot::{Crest, CrestReroll}`,
  `CREST_SLOT`), купленные — в правом верхнем углу поля (`crest_corner`, `paint_crests`),
  подсказка — `crest_hover` (`crest_under`) → `paint_crest_tip`. Картинки — `tools/breach_crest.py`
  (`ui/crest_<heart|coin|snow2|pawn|spikes|crystal>.png`, `CREST` 49×45, `UiTextures::crest`).
  **0.78** (просьба пользователя): в rogue-like у коней обеих сторон своё здоровье —
  `rogue_knight_hp` (`Rule::RogueKnightHp` = 3 единицы; у пользователя в `[piece.Knight]` стоит 2):
  `crested` теперь и разница с обычным здоровьем коня в `bonus_hp` (вызывается в `rogue_start`,
  `spawn_wave`, `take_stock` обоих видов запаса, `begin_planning`, превращении пешки), справка и
  «Ударов до убийства» — по `Position::kind_max_hp` (с 0.114 — `rogue_max_hp` в `app.rs`). Кристалл замораживает на `frozen` = 1:
  фигура во льду до конца хода соперника, к своему ходу оттаивает и ходит.
  **0.79** (просьба пользователя): ядовитые фигуры волн — `Piece::venom`, выходят в `spawn_wave` на
  уровне `Rule::RoguePoisonLevel` (2) с шансом `RoguePoisonPercent` (50). Удар ядовитой по выжившей
  цели без яда ставит `Piece::poison` (концов ходов её стороны до урона: бьют в ход соперника — 1,
  в её ход — 2; `Strike::poisoned`), `poison_tick` в `end_turn` после ударов и взрывов —
  `RoguePoisonDamage` (1 ед.), `TurnEnd::poison` (`Hit`, цель — до урона), в описании «Яд: …».
  Расстановка лечит и яд (`Piece::new`). Арт — `tools/breach_poison.py`: спрайты
  `pieces/poison/` (`POISON_BODIES`/`POISON_SHADOWS`, `piece_body`/`piece_shadow`), лужа с каплями
  `effects/poison_*` (`POISON_LOOP` = [8, 40], `Figure::poison`, `poison_frame`, рисуется как
  вихрь), след шага `effects/poison_trail_<ne|sw|nw|se>` (96×80, середина между центрами клеток,
  30 кадров по 60 мс; `trails` в `queue_strikes` для шагнувших ядовитых, `paint_trails` — до
  фигур, направление — по экранной разнице клеток). Череп — `Glyph::Skull` цвета `POISON_SKULL`
  над каждой рисуемой фигурой с ядом (`paint_skulls`, после фигур). Урон яда — `Effect::hit` после
  ударов и взрывов, до него фигура прежняя (`poison_due`), погибшая стоит.
  **0.80** (просьба пользователя): варвары — `Power::Barbarian`/`Piece::barbarian`, вес и цена как у
  остальных особых (`Rule::RogueBarbarianPercent` = 200%; с ними у каждой особой силы доля
  прилавка 1/13), запас `barbarian_reserve`, `MoveKind::BarbarianDrop` (в сети 80 + вид). В
  `strike_all` удары фигуры идут раундами (`'rounds`): варвар, убивший кого-то за раунд (и молнией
  тоже), бьёт ещё раунд по оставшимся в досягаемости (`Strike::again`, «Снова —» в описании), не
  больше `BARBARIAN_ROUNDS` = 16. Спрайты — `tools/breach_barbarian.py` (`pieces/barbarian/`,
  `BARBARIAN_BODIES`/`BARBARIAN_SHADOWS`, `piece_body`/`piece_shadow`), анимации нет.
  **0.81** (просьба пользователя): гербы варваров (`Crest::{AxeSword, AxeFangs, AxeQueen, Axes,
  Whirl, FireAxe}`, `CRESTS` = 12, картинки `ui/crest_<axe_sword|axe_fangs|axe_queen|axes|whirl|
  fire_axe>.png` из `tools/breach_crest.py`; в углу — по `CREST_ROW` = 6 в ряд). Повтор варвара
  теперь ограничен `Rule::RogueBarbarianRepeats` = 1 (`barbarian_repeats`; был до 16 подряд —
  иначе огненному топору нечего добавить). `AxeSword` — `crested`: `bonus_attack` +
  `RogueCrestAxeAttack` (2 ед.), `bonus_hp` − `RogueCrestAxeHp` (1 ед., полное — не меньше
  единицы); `AxeFangs` — в `strike_all` первое убийство варвара за ход лечит
  `RogueCrestFangsHeal` (`Strike::feast`; цифра лечения теперь на клетке ударившего); `AxeQueen` —
  `Position::shop_price` − `RogueCrestQueenDiscount` (6); `Axes` — вес варваров в `roll_offers` ×
  `RogueCrestAxesPercent`; `Whirl` — повторный раунд бьёт всех чужих в квадрате радиуса
  `RogueCrestWhirlRadius` (2) без проверки линий (`Strike::whirl`, «Круговой удар —», анимации нет —
  обычные выпады); `FireAxe` — при здоровье ≤ `RogueCrestFireAxeHp` (1 ед.) ещё
  `RogueCrestFireAxeRepeats` (1) повтор.
  **0.82** (просьба пользователя): круговой удар — урон `Rule::RogueCrestWhirlDamage` (1 ед.)
  каждой фигуре в радиусе (`strike_raw`, а не атака), в приложении без выпадов
  (`StrikeAnim::lunge` — ноль у `whirl`) и разом: удары одного кругового — с одного `start`, звук
  один (`queue_strikes`, `whirl_from`). Своя анимация кругового удара — потом.
  **0.83** (просьба пользователя): маги — `Power::Wizard`/`Piece::wizard` (вес в `roll_offers` у пешки
  0, короля в магазине нет), цена × `Rule::RogueWizardPercent` (200%), запас `wizard_reserve`,
  `MoveKind::WizardDrop` (в сети 90 + вид), «Белый конь-маг». В `end_turn` после взрывов —
  `meteors`: каждый свободный маг стороны бьёт случайную клетку с чужой фигурой (случайность —
  зерно, `key()` и клетка мага) уроном `Rule::RogueWizardDamage` (1 ед.), `TurnEnd::meteors`
  (клетка мага, `Hit`), «Метеорит мага с …». Спрайты — `tools/breach_wizard.py` (`pieces/wizard/`,
  `WIZARD_BODIES`/`WIZARD_SHADOWS`). Метеорит — наш `tools/breach_meteor.py`
  (`effects/meteor.png`, 24 кадра 112×200, центр клетки (56, 172), 50 мс, касание в кадре 9: летит
  сверху-справа с огненным хвостом, вспышка, огненный шар, кольцо огня и подпалина в ромбе клетки,
  осколки, дым); в приложении `meteors` (по очереди через `METEOR_GAP`, звук в миг касания,
  `paint_meteors` поверх фигур), урон — `Effect::hit` в миг касания, до него фигура прежняя
  (`due_hits` — бывший `poison_due`, теперь и для метеоритов; яд — после метеоритов). Компьютер и
  магазин ждут `due_hits`.
  **0.84** (просьба пользователя, по образцу-картинке): метеорит крупнее и «метеоритнее», анимация
  плавнее — `tools/breach_meteor.py` переделан в арт-сессии: бугристый камень с кратерами (`ROCK_R` =
  12, `ROCK`, медленно вращается, спереди раскалённая кромка) в огненной оболочке, широкий хвост из
  языков (`TAIL` 60→84, за камнем шире, у камня бледно-жёлтое, к краям и концу красное, узор течёт
  назад — `value_noise`), искры; первые 3 кадра проявляется. Полоса — 44 кадра 200×280, центр клетки
  (72, 250), 30 мс, касание в кадре 16 (`METEOR*` в `assets/ui.rs`); взрыв — прежний, растянут на 28
  кадров (огненный шар до 22).
  **0.85** (просьба пользователя): метеорит падает вдвое быстрее — кадры полёта по
  `METEOR_FALL_TIME` = `METEOR_FRAME_TIME` / 2 (15 мс), взрыв — по-прежнему 30 мс (`meteor_impact`,
  `meteor_frame` в `app.rs`).
  **0.86** (просьба пользователя): гербы огненных фигур (`Crest::{FlamePawn, Flames, FlameSkull,
  FlameSword, FlameCoin, Bomb}`, `CRESTS` = 18, картинки `ui/crest_<flame_pawn|flames|flame_skull|
  flame_sword|flame_coin|bomb>.png` из арт-проекта). Взрыв вынесен в `explode(center, us, raw)`:
  `raw` = `None` — огненная фигура с гербами (`fire_damage`: `LavaDamage` или `RogueCrestFlameSwordDamage`
  2 ед., + `RogueCrestBombDamage` 2 ед.; клетки — `blast_area`: 8 вокруг, с `Flames` ещё крест до
  `RogueCrestFlamesReach` = 2; с `Bomb` задевает и своих, кроме себя), `Some` — пешка с лавой
  усиленного компьютера (как раньше). `blast_reaches` — по той же области. `FlameSkull` — в `end_turn`
  после ударов, взрывов, метеоритов и яда `death_blasts` (погибшие огненные игрока взрываются на
  своей клетке, цепочкой — погибшие от них свои огненные; в `TurnEnd::blasts`, анимация — обычная
  огненная волна 3×3, на дальних клетках креста — только цифры). `FlamePawn` —
  `Position::shop_price` − `RogueCrestFlamePawnDiscount` (2); `FlameCoin` — вес огненных ×
  `RogueCrestFlameCoinPercent`.
  **0.87** (просьба пользователя): взрыв с гербом двух огоньков — толстый крест (`blast_area`: в
  квадрате ±`RogueCrestFlamesReach` клетки, где |df| ≤ 1 или |dr| ≤ 1 — 3×3 и по три клетки с каждой
  стороны, 20 клеток), `Blast::wide` (`wide_blast`). Анимация — наш `tools/breach_firewave_plus.py`
  (функции и палитры `breach_firewave.py` на своём холсте: `fire_wave_plus_<under|over>.png`, 26
  кадров 304×248, центр клетки (152, 132), 60 мс; кромка бежит квадратными кольцами до 2.62 клетки
  только внутри креста, огоньки догорают по всему кресту) — `BIG_WAVE_*` в `assets/ui.rs`
  (`UiTextures::big_wave`, `Strip`), `Wave::wide` в `table.rs` (`impact` = `BIG_WAVE_IMPACT` 0.36 с,
  `length`); анимация рассчитана на крест в 2 клетки.
  **0.88** (просьба пользователя): гербы теневых фигур (`Crest::{DarkRook, DarkSkull, DarkFlames,
  DarkPortal, DarkCoin, DarkEye}`, `CRESTS` = 24, картинки `ui/crest_dark_<rook|skull|flames|portal|
  coin|eye>.png` из арт-проекта). `DarkRook` — `shop_price` − `RogueCrestDarkRookDiscount` (4);
  `DarkCoin` — вес теневых × `RogueCrestDarkCoinPercent`; `DarkEye` — в `strike_all` теневая
  игрока бьёт без `line_clear`; `DarkSkull` — в `make_move` после `relocate` удар атакой по чужим на
  `shade_passed` (по прямой — клетки между, ход 2×1 — две клетки рядом с обеими), `Outcome::passed`
  (`WaveHit`), в приложении — `Effect::hit` и `due_hits` на середине шага, «Тень бьёт насквозь».
  В начале `strike_raw`: `DarkFlames` — счётчик `Piece::dodge`, каждый `RogueCrestDarkDodgeEvery`-й
  (3) удар — урон 0 (`Strike::dodged`, без цифры); затем `DarkPortal` — теневая с ≤
  `RogueCrestDarkPortalHp` (1 ед.) от смертельного удара меняется местами (`board.swap`, метки
  `origin` тоже) с `portal_partner` (ближайшая своя без силы, не король/босс), удар принимает та
  (`Strike::swap` = (новая клетка теневой, теневая), `hit.target` — обычная); тот же атакующий
  клетку ушедшей больше не бьёт (`skip` в `strike_all`). В приложении перед таким ударом —
  `Portal::swap` (портал карты обмена с запомненными фигурами `Portal::pieces`), удары после него
  сдвигаются на `Portal::length()`; в портале фигуры — `dressed`.
  **0.89** (просьба пользователя): бронированные фигуры и вампиры волн — `Piece::{armored, vampire}`.
  Пользователь сказал «с третьей волны» / «с четвёртой волны»; понято как уровни (продолжение: яд —
  2-й уровень), но есть и правила волны уровня: `Rule::RogueArmor{Level, Wave, Percent, Hp}` (3, 1,
  30%, 3 ед.) и `RogueVampire{Level, Wave, Percent, Hp, Heal}` (4, 1, 30%, 1 ед., 1 ед.). В
  `spawn_wave` один бросок `next(100)` (только когда кто-то из них возможен; ядовитая — ни то ни
  другое): меньше шанса брони — броня, дальше в пределах суммы — вампир. Здоровье — в `crested`
  (`bonus_hp`). Вампир в `strike_raw`: удар с уроном лечит его на `RogueVampireHeal` (не выше полного,
  `Strike::feast`, «Ладья восстановила 1 HP»). Спрайты: броня — `tools/breach_armor.py`
  (`pieces/armor/`, `ARMOR_*`), вампиры — «Дракула» из `tools/breach_vampire.py`
  (`pieces/vampire_dracula/`, `VAMPIRE_*`); `Piece::title` — «Чёрная бронированная ладья», «Чёрный
  конь-вампир».
  **0.90** (просьба пользователя): гербы воздушных фигур (`Crest::{Winds, WindCoin, WindKnight,
  WindSword}`, `CRESTS` = 28, картинки `ui/crest_<winds|wind_coin|wind_knight|wind_sword>.png` из
  арт-проекта). `Winds` — в `make_move` маска `Position::twice` (сбрасывается вместе с `airborne`):
  после бесплатного второго хода фигура снова в `airborne` и в `twice`, третий ход — бесплатно, потом
  `used`; `WindCoin` — вес воздушных × `RogueCrestWindCoinPercent`; `WindKnight` — `shop_price` −
  `RogueCrestWindKnightDiscount` (4); `WindSword` — в `strike_all` после первого раунда ударов воздушная
  бьёт ещё раз первую живую из своих целей (`order`, `Strike::again`, «Снова —»).
  **0.91** (просьба пользователя): гербы электрических фигур (`Crest::{BoltBishop, BoltCoin, BoltPlus,
  Bolts, RedBolt, BoltOne}`, `CRESTS` = 34, картинки `ui/crest_<bolt_bishop|bolt_coin|bolt_plus|bolts|
  red_bolt|bolt_one>.png` из арт-проекта). Молния в `strike_raw`: по каждому направлению (`BISHOP_RAYS`,
  с `BoltPlus` — `KING_STEPS`) от цели до `spark_reach` клеток через пустые — первая фигура, чужая
  задета, своя или барьер останавливают; `Strike::splash` — `[Option<Hit>; SPLASH_MAX]` (8).
  `spark_reach`: 1; `Bolts` — 7; `RedBolt` — 1 + `Piece::charge` / `RogueCrestRedBoltStep` (10), не больше
  `RogueCrestRedBoltMax` (3). `Piece::charge` — убийства электрической (удар и молния) при гербе красной
  молнии, переживает расстановку (`Piece::with`), в карточке фигуры — «Заряд N · молния летит на M
  клеток». `BoltOne` — в `strike_all` очередь `strike_queue` переставлена: электрические первыми.
  `BoltBishop` — `shop_price` − `RogueCrestBoltBishopDiscount` (4); `BoltCoin` — вес электрических ×
  `RogueCrestBoltCoinPercent`.
  **0.92** (просьба пользователя): `Bolts` — не «на любое расстояние», а цепь: в `strike_raw` обход в
  ширину от цели (`sources`), с каждой задетой — те же направления и `spark_reach`, каждая клетка раз
  (`seen`). `Strike::splash` — `Vec<Jump>` (`Jump { from, hit }`: откуда прыгнула молния), `Strike`
  больше не `Copy`. В приложении — `Zap` (разряд от фигуры к фигуре в миг касания: ломаная в пикселях
  арта, голубое свечение 3 пикселя `ZAP_GLOW` и белое ядро `ZAP_CORE`, перескакивает каждые
  `ZAP_FLICKER`, гаснет за `ZAP_TIME`; `paint_zaps` после фигур). Заряд при наведении — в `paint_stats`
  табличка над головой: знак шрифта `Glyph::Bolt` (`z`, жёлтый `PIXEL_BOLT`) и число `Piece::charge`
  (только электрическим при гербе красной молнии).
  **0.93** (просьба пользователя): гербы магазина `Crest::{Pawns, Bishops, Knights, Rooks, Queens,
  RerollCoin}` (`CRESTS` = 40, картинки `ui/crest_<pawns2|bishops2|knights2|rooks2|queens2|
  reroll_coin>.png` из арт-проекта). Пары — в `roll_offers` вес вида (всех сил) × `RogueCrestPairPercent`
  (`paired`); `RerollCoin` — `Position::reroll_price` (`RogueCrestRerollCoinCost` = 1 вместо
  `RogueRerollCost`), им пользуются `reroll` и магазин в `table.rs` (свободная `reroll_cost` убрана).
  **0.94** (просьба пользователя): гербы магов `Crest::{HatCoin, Hats, HatBishop, Meteor, HatMeteor,
  HatPlus}` (`CRESTS` = 46, картинки `ui/crest_<hat_coin|hats|hat_bishop|meteor|hat_meteor|
  hat_plus>.png` из арт-проекта). `TurnEnd::meteors` — `Vec<Meteor>` (`from`, `at`, `hits`, `wide`,
  `crater`). В `meteors`: `Hats` — по два на мага (второй — своим броском); урон —
  `RogueCrestMeteorDamage` (2) с `Meteor`; `HatPlus` — ещё чужие на соседних по прямым (`ROOK_RAYS`);
  `HatMeteor` — `Position::craters` (маска) на клетке падения, гаснет, когда ход переходит к игроку
  (`end_turn`). `crater_burns` после `march`: шагнувшие в кратер — `RogueCrestCraterDamage` (1),
  `TurnEnd::burned`, золото за убитых — игроку. Арт — наши `tools/breach_crater.py` (кратер: тёмная
  воронка с тлеющими трещинами, угли и низкое пламя по краю ромба; `effects/crater_<under|over>.png`,
  12 кадров 80×112, центр (40, 84), 80 мс по кругу; задние языки — под фигурой, передние — над) и
  `tools/breach_meteor_wave.py` (огненная волна узким крестом на функциях `breach_firewave_plus.py`:
  `effects/meteor_wave_<under|over>.png`, 18 кадров 184×168 как у огненной волны). В приложении:
  `craters` (клетка, загорелся в миг касания, гаснет — после шагов следующего хода соперника +
  `CRATER_FADE`; кратер позиции без записи — всегда), `paint_craters` (до и после фигур),
  `meteor_waves`/`paint_meteor_waves`; урон горящих — `Effect::hit`/`due_hits` к концу шагов.
  **0.95** (просьба пользователя): гербы золотых фигур `Crest::{GoldKing, KingCoin, Kings2, KingQueen,
  KingSkull, Piggy}` (`CRESTS` = 52, картинки `ui/crest_<gold_king|king_coin|kings2|king_queen|
  king_skull|piggy>.png` из арт-проекта). В `strike`: победа золотой фигурой игрока —
  `RogueGoldBonus` + `RogueCrestGoldKingBonus` (1) с `GoldKing` + `RogueCrestKingQueenBonus` (2) у
  ферзя с `KingQueen`. `KingCoin` — `Position::sell_price` (не меньше `shop_price`; «полная стоимость»
  понята для всех фигур запаса — золотые и так продаются за свою цену), им пользуются `sell` и зона
  продажи. `Kings2` — вес золотых × `RogueCrestKings2Percent`. `KingSkull` — в `fall`: погибшая
  золотая игрока возвращает `shop_price`. `Piggy` — `piggy_bank` в `end_turn` перед
  `begin_planning`: (золото / `RogueCrestPiggyStep`) × золотых на доске и в `gold_reserve`,
  `TurnEnd::interest`, «Свинья-копилка: +N». Числа гербов в описании через `coins` — со склонением.
  **0.96** (просьба пользователя): объединённые фигуры — варвар с другой силой. `Stock::with` (вторая
  сила, у одиночных `Power::Plain`), `Stock::combo(kind, with)` (`power` = `Barbarian`), `Stock::has`,
  `Stock::name` («огненный варварский конь»), `COMBOS` = [Fire, Spark, Wind, Gold, Shade, Ice, Wizard];
  у фигуры — оба флага (`Stock::piece`, `Stock::of`, `Piece::has_power`), поэтому правила и гербы
  обеих сил действуют сами (огненный варвар взрывается и бьёт повторно, на нём «топор и меч» и
  «огонь и меч»); скидки гербов в `Position::shop_price` — по `stock.kind` и `stock.has` (снежинка и
  пешка: ледяной варвар-пешка — как обычный варвар), продажа золотого — по `has(Gold)`, копилка
  считает и золотых варваров запаса (`reserve`). Открывают гербы `Crest::{AxeFlame, AxeBolt, AxeWind,
  AxeKing, AxeDark, AxeSnow, AxeHat}` (`CRESTS` = 59, `Crest::unlocks`, `Position::combo_open`;
  картинки `ui/crest_axe_<flame|bolt|wind|king|dark|snow|hat>.png` из арт-проекта). Прилавок
  (`roll_offers`): одиночные — прежним порядком (`power_percent` — множители гербов «… и монеты» и
  двух топоров), за ними открытые объединённые с весом × `Rule::RogueComboWeightPercent` (100%) ×
  множители обеих сил (маг-пешки нет); цена — обычная × `RogueShopPercent` × `RogueComboPercent`
  (300%). Запас — `combo_reserve` [номер в `COMBOS`][сторона][`Kind`], готовы всегда, в `reserve()`
  первыми (`charged_stock`/`charged_stock_mut`); выставление — `MoveKind::ComboDrop(kind, номер)` (в
  сети 100 + номер × 6 + вид). `Piece::title` — все силы прилагательными по порядку, маг — «-маг» в
  конце. Арт-проект: спрайты `pieces/barbarian_gold`, `pieces/barbarian_ice`, `pieces/wizard_horns`
  (`BARBARIAN_GOLD`/`BARBARIAN_ICE`/`WIZARD_HORNS`, `combo_set` в `piece_body`/`piece_shadow` —
  первыми), анимации на спрайте варвара `effects/barbarian/<lava_crack|electric_hold|wind|dark_fire|
  shadowstep_out|shadowstep_in>_<код>.png` — в полосах `UiTextures` следом за обычными (`BARBARIAN`
  = 12, `strip_index`); `lava`/`electric`/`wind`/`dark` теперь берут `Piece`. В `paint_stock`
  объединённая анимируется по второй силе.
  **0.97** (просьба пользователя: «то же самое для золотых»): `COMBOS` — пары (основа, сила), 13:
  прежние 7 варварских (номера и сетевые коды не менялись), за ними золотые с огнём, молнией,
  ветром, тенью, льдом, магом (золотой варвар — по-прежнему `(Barbarian, Gold)`, `Stock::of` ищет
  пару по порядку). `Stock::combo(kind, основа, сила)`, `Crest::unlocks` → `(основа, сила)`,
  `combo_open(пара)`; в `roll_offers` множители основы и силы (`power_percent`). Гербы
  `Crest::{KingFlame, KingBolt, KingWind, KingDark, KingSnow, KingHat}` (`CRESTS` = 65, картинки
  `ui/crest_king_<flame|bolt|wind|dark|snow|hat>.png` из арт-проекта). Снежинка и пешка: ледяная
  объединённая пешка — как её основа. `Piece::title`: основы ближе к имени — «огненный золотой
  конь», «ледяная золотая ладья», «золотой варварский конь», «золотой слон-маг». Арт —
  `tools/breach_gold_combo.py`: спрайты `pieces/gold_ice`, `pieces/gold_wizard` (`GOLD_ICE`,
  `GOLD_WIZARD`, `combo_set` 3 и 4), анимации `effects/gold/…` — в полосах `UiTextures` после
  варварских (`GOLD` = 24 в `strip_index`: варвар — свой набор, иначе золотая — свой).
  **0.98** (просьба пользователя: «то же самое для магов»): `COMBOS` — 18, в конце маги с огнём,
  молнией, ветром, тенью, льдом (золотой маг и варвар-маг — прежние пары). Гербы `Crest::{HatFlame,
  HatBolt, HatWind, HatDark, HatSnow}` (`CRESTS` = 70, картинки `ui/crest_hat_<flame|bolt|wind|dark|
  snow>.png` из арт-проекта). Маг-пешки нет ни у одной пары с магом (`roll_offers`). Названия —
  «огненный конь-маг», «ледяная ладья-маг». Арт — `tools/breach_wizard_combo.py`: спрайт
  `pieces/wizard_ice` (`WIZARD_ICE`, `combo_set` 5), анимации `effects/wizard/…` — в полосах после
  золотых (`WIZARD` = 36 в `strip_index`: варвар, иначе золотая, иначе маг).
  **0.99** (просьба пользователя: «то же самое для ледяных»): `COMBOS` — 22, в конце ледяные с огнём,
  молнией, ветром, тенью (ледяные варвар, золотая и маг — прежние пары). Гербы `Crest::{FlameSnow,
  SnowBolt, SnowWind, SnowDark}` (`CRESTS` = 74, картинки `ui/crest_flame_snow.png`,
  `ui/crest_snow_<bolt|wind|dark>.png` из арт-проекта). Снежинка и пешка: у ледяной основы цена — как у
  второй силы (огненная ледяная пешка — как огненная). Названия — «огненный ледяной конь», «теневая
  ледяная ладья». Спрайт — обычный ледяной; анимации `tools/breach_ice_combo.py` (`effects/ice/…`) — в
  полосах после магов (`ICE` = 48 в `strip_index`: варвар, золотая, маг, ледяная). Код хода в сети —
  `u16` (`encode_move`/`decode_move`): `100 + номер × 6 + вид` у следующих пар перевалит за 255.
  **0.100** (просьба пользователя): над «Конец хода» в rogue-like — табличка ходов фигурами
  (`paint_moves_left` в `table.rs`, `MOVES_GAP`): знак `Glyph::Pawn` (`p` в шрифте) и
  `Position::moves_left` (кончились — красным `MOVES_SPENT`); только в ход игрока, не в расстановке и
  не в магазине. Воздушная тратит ход первым шагом, даровые следующие счётчик не убавляют.
  **0.101** (просьба пользователя): огненные объединённые — `COMBOS` 25, в конце огонь с молнией,
  ветром, тенью; гербы `Crest::{FlameBolt, FlameWind, FlameDark}` (`CRESTS` = 77, картинки
  `ui/crest_flame_<bolt|wind|dark>.png` из арт-проекта). Названия — «огненный электрический конь».
  Анимации — `tools/breach_fire_combo.py` (`effects/fire/…`: кадры силы на теле с трещинами ожидания)
  в полосах после ледяных (`FIRE` = 60; `strip_index` — огонь, только если нет другой основы,
  `base_set`; у `lava` своего набора огня нет — обычная). В `dressed`: пока кадр лавы не в
  `LAVA_HOLD` (насыщение, взрыв), кадры другой силы (`spark`/`wind`/аура `dark`) не рисуются — видна
  лава.
  **0.102** (просьба пользователя): электрические объединённые — `COMBOS` 27, в конце молния с ветром и
  тенью; гербы `Crest::{BoltWind, BoltDark}` (`CRESTS` = 79, картинки `ui/crest_bolt_<wind|dark>.png`
  из арт-проекта). Названия — «электрический воздушный конь». Анимации — `tools/breach_spark_combo.py`
  (`effects/spark/…`: вихрь или тёмная аура вместе с молниями, 40 кадров) в полосах вихря и тени после
  огненных (`SPARK` = 72 в `strip_index`); у электричества своего набора нет — `electric_index`
  (основа или огонь). В `paint_figures` вихрь и аура рисуются раньше электричества, так что у
  объединённой видна полоса набора молнии. Коды выставления в сети теперь доходят до 261 (`u16`).
  **0.103** (просьба пользователя): воздушные теневые — последняя пара, `COMBOS` 28 (все пары особых
  сил, сеть — до 267); герб `Crest::WindDark` (`CRESTS` = 80, `ui/crest_wind_dark.png` из арт-проекта).
  Анимации — `tools/breach_wind_combo.py` (`effects/wind/<dark_fire|shadowstep_*>`) после электрических
  (`WIND` = 84). Выбор набора — `strip_index(piece, анимация)`: основа (варвар, золотая, маг, ледяная),
  иначе набор другой силы, если в нём есть эта анимация (огонь — всё, кроме лавы; молния — вихрь и
  тень; ветер — тень); тест `every_special_piece_finds_its_animation_strips` в `assets/ui.rs`
  проверяет, что у всех пар номера полос в границах.
  **0.104** (просьба пользователя): в начале rogue-like в запасе 2 пешки и вместо слона и коня одна
  особая фигура — `Position::starting_special(seed)` в `rogue_start`: вид по `shop_weights` без пешки
  (ферзь реже всех), сила — одна из восьми поровну, случайность — зерно партии; кладётся
  `give_stock`. Старые тесты, которым нужен обычный конь в запасе, кладут его сами.
  **0.105** (просьба пользователя): на уровне `Rule::RogueArmorSkipLevel` (4) брони нет — её шанс
  прибавляется вампирам (60%), с 5-го снова как было. Зомби волн — `Piece::{zombie, risen}`, с
  `Rule::RogueZombieLevel` (5) с шансом `RogueZombiePercent` (30, тот же бросок `next(100)` после брони
  и вампиров: на 5+ уровнях по 30% у каждой), здоровье + `RogueZombieHp` (1 ед., в `crested`).
  Гибель — в `fall`: зомби соперника без `risen` оставляет `Position::graves[sq]` (`Grave { kind,
  wait }`: погиб в ход игрока — 1, в свой — 2 конца хода своей стороны). `rise_zombies` в `end_turn`
  соперника после `march`/`crater_burns`, до волны: восстаёт с `RogueZombieRiseHp` (1 ед.), `risen`,
  на своей клетке или ближайшей свободной соседней (`KING_STEPS`), иначе ждёт; `TurnEnd::risen`
  («Восстали зомби: …», «Зомби ↑N»). Уровень не пройден и соперник не побеждён, пока есть могилы
  (`waves_left` в `end_turn` и `defeated`); `win_round` и `start_round` могилы стирают. Спрайты —
  `tools/breach_zombie.py` (`pieces/zombie/`, `ZOMBIE_BODIES`/`ZOMBIE_SHADOWS`, `piece_body`/
  `piece_shadow`); «Чёрный конь-зомби». В приложении: восставшие проявляются как волна (`appears`
  после шагов), могила — череп `Glyph::Skull` цвета `GRAVE_SKULL`, мерцает (`paint_graves`, до фигур;
  на клетке ещё стоящей убитой — нет).
  **0.105.1** (уточнение пользователя: «50% волны — обычные, 50% — те, что я скажу, других нет, пока
  не скажу обратное»): у каждого уровня один вид усиленных — `spawn_wave` проверяет уровень на
  равенство: `RoguePoisonLevel` 2 — ядовитые, `RogueArmorLevel` 3 — броня, `RogueVampireLevel` 4 —
  вампиры, `RogueZombieLevel` 5 — зомби, шансы `…Percent` = 50; на 1-м и с 6-го — только обычные.
  `rogue_armor_skip_level` убран (в `RETIRED_RULES`). В `balance.ini` пользователя проценты брони,
  вампиров, зомби 30 → 50, строка пропуска удалена, копия `balance.ini.before-0.105.1`.
  **0.106** (просьба пользователя): вампиры в броне — фигура волны с `armored` и `vampire` сразу, на
  уровне `Rule::RogueArmoredVampireLevel` (6) с шансом `RogueArmoredVampirePercent` (50); здоровье +
  `RogueArmoredVampireHp` (3 ед., в `crested` вместо брони и вампира); лечение ударом — один раз за
  ход (`Position::fed`, маска клеток, сбрасывается со сменой хода). «Чёрная бронированная ладья-вампир».
  Спрайты — `tools/breach_armor_vampire.py` (`pieces/armor_vampire/`, `ARMOR_VAMPIRE_*`, в
  `piece_body`/`piece_shadow` раньше брони).
  **0.107** (просьба пользователя): ядовитые зомби — `venom` и `zombie` сразу, на уровне
  `Rule::RogueVenomZombieLevel` (7) с шансом `RogueVenomZombiePercent` (50); здоровье — как у зомби;
  `Grave::venom` — восстаёт ядовитым. Спрайты — `tools/breach_poison_zombie.py`
  (`pieces/poison_zombie/`, `POISON_ZOMBIE_*`, в `piece_body`/`piece_shadow` раньше ядовитых); лужи-анимации
  у них нет (`poison_frame` — `None` у зомби), след шага остаётся.
  **0.108** (просьба пользователя): rogue-like — основной режим. Обычной партии против компьютера
  нет: «Начать играть» в меню → `new_rogue_game` (`Choice::Rogue`/`Choice::Boost` убраны); `new_game`,
  `Setup`/`Opponent`/`SideChoice`, `Thinking`, `coin_flip` и `src/ai.rs` удалены (`make_turn` —
  только для тестов); компьютер (волны) кончает ход сам через `COMPUTER_DELAY` (`computer_since` в
  `drive_computer`); в левой панели «Новая партия» — «Взятие» и «Начать заново»; правила панели —
  `rules()` (rogue-like, числа из баланса). Мультиплеер (классическая партия) остался. Усиленного
  компьютера нет: `Position::boost`/`boosted`/`charge_lava`, `Piece::lava`, `TurnEnd::charged`,
  `Game::boost`, `show_boost`, `Settings::boosted` (строка `boosted=` старого settings.ini молча
  пропускается) убраны; `lava_blasts` — только огненные, `explode(center, us)` без `raw`,
  `blast_area`/`wide_blast` без `crested`. Магазин: фигуры и гербы на одном прилавке —
  `Rogue::wares: [Option<Ware>; SHOP_MAX]` (`SHOP_MAX` = 8, `Ware::{Piece(Stock), Crest(Crest)}`,
  `Position::wares`, в тестах `offers()` — только фигуры), мест `Rule::RogueShopSlots` (6), на каждом
  фигура с шансом `RogueShopPiecePercent` (25), иначе герб (разные, из некупленных; кончились —
  фигура): `roll_shop` (xorshift64*: у простого xorshift младшие биты соседних чисел связаны — выбор
  «фигура или герб» сдвигал доли видов); `buy(slot)` — и фигура, и герб (`ware_price`); обновление —
  `reroll_price` = `RogueRerollCost` (5; с гербом монеты и обновления — `RogueCrestRerollCoinCost`) +
  `RogueRerollStep` (1) × `Rogue::rerolls` (обнуляется в `begin_planning`). Убраны `crest_offers`,
  `roll_crests`, `buy_crest`, `reroll_crests`, `RogueAction::{Sell, RerollCrests}`, продажа (`sell`,
  `sell_price`, зона, `sell_drag`, `reserve_sellable`), герб `Crest::KingCoin` (`CRESTS` = 79,
  картинка убрана из `CRESTS` в `assets/ui.rs`, файл остался), правила `lava_pawns`,
  `rogue_gold_sell_percent`, `rogue_crest_slots`, `rogue_crest_reroll_cost` (в `RETIRED_RULES`).
  Интерфейс — одна панель «МАГАЗИН» посреди доски (`shop_layout` → `(Rect, места)`, `SHOP_SLOT`
  57×72, по `SHOP_ROW` = 6 в ряд, `ShopSpot::{Offer, Reroll, Leave}`), подсказка герба — и на
  прилавке (`crest_under`). Выставление в rogue-like: не больше `Rule::RogueDropsPerTurn` (1) за ход
  (`Position::drops`, `drops_left`; расстановку не считает), пока своих фигур на доске меньше
  `RogueMaxPieces` (16, `board_room`; и в расстановке — полная армия выставлять не даёт), только на
  две первые горизонтали (`drop_squares` → `arrange_fits`, пешка — на вторую). Первый уровень: у
  фигур волн здоровья меньше на `RogueFirstLevelHpCut` (1 ед., не меньше единицы; в `crested`, не у
  босса). Маги — метеорит раз в `RogueWizardEvery` (2) своих ходов: `Piece::rest` (после метеорита —
  every − 1, каждый следующий свой конец хода −1; только что выставленный и замёрзший не копят), в
  карточке фигуры справа — «Метеорит — в конце хода / через ход». Особые фигуры, могилы и кратеры
  движутся и в покое — `idle_animation` (кадр раз в 30 мс; раньше в rogue-like их двигал только
  курсор). В `balance.ini` пользователя `rogue_shop_slots` 3 → 6, `rogue_reroll_cost` 2 → 5, строки
  удалённых правил убраны, новые дописаны; копия — `balance.ini.before-0.108`.
  **0.109** (просьбы пользователя): мест в магазине снова 3 (`RogueShopSlots`; в `balance.ini`
  пользователя 6 → 3, копия `balance.ini.before-0.109`). Особая пешка превращается в особую фигуру:
  `Piece::new(kind, us).with(pawn)` + `crested` в `make_move` и `advance_to` (раньше — обычная).
  Очередь ударов — по горизонталям сверху вниз, в каждой слева направо: a8, b8, …, h8, a7, …, h1
  (`strike_queue`, `strike_order`; ею же идут взрывы огненных и метеориты магов, она же —
  последний ключ выбора цели).
  **0.110** (просьба пользователя): сундуки магазина — `Chest::{Pieces, Crests}` (`ALL`, `title`),
  `chest_price` (`Rule::RogueChestPiecesPrice` 15, `RogueChestCrestsPrice` 20), на выбор
  `chest_choices` (`RogueChestChoices` 3, массивы `CHEST_MAX` = 5). В `Rogue`: `chest_wares`
  [сундук][место] (`Ware`), `chests: [ChestState; 2]` (`Closed`/`Open`/`Taken`). `roll_chests` в
  `begin_planning` (до `roll_shop`, зерно и уровень; обновление не трогает): фигуры — разные, по
  весам прилавка (`stock_weights`, `draw_stock`, случайность `shop_random` — xorshift64*), гербы —
  разные некупленные; `roll_shop` не кладёт на прилавок гербы сундука. `open_chest` (платить,
  `Open`; пока открыт — `buy`/`reroll`/`leave_shop`/второй сундук — нельзя), `take_from_chest(slot)`
  (`receive` — общая выдача фигуры или герба, сундук `Taken` и пуст); `RogueAction::{OpenChest,
  TakeFromChest}`, `Game::open_chest`/`take_from_chest`, `Position::chest_open`/`chest_state`/
  `chest_wares`/`chest_for_sale`. Интерфейс: под прилавком — два места сундуков
  (`ShopSpot::Chest`, картинка `CHEST` 46×40 из нашего `tools/breach_chest.py`:
  `ui/chest_pieces.png` — дерево, железо, золотая пешка; `ui/chest_crests.png` — бордо, золото,
  щит; `UiTextures::chest`), открытый сундук — панель «СУНДУК … · выберите одно» (`ShopSpot::Pick`,
  `paint_ware`), подсказка герба — и там.
  **0.111** (просьбы пользователя): конец хода без хода фигурой — `Game::can_end_turn` = не конец
  партии и не магазин (кнопка горит янтарным только после хода, `pos_ready` в `table.rs`). Гербов
  объединения нет (28 вариантов `Crest` с `AxeFlame` по `WindDark`, `unlocks`, `combo_open`; `CRESTS`
  = 51, их картинки убраны из `CRESTS` в `assets/ui.rs`): в `stock_weights` все пары `COMBOS` сами,
  вместе — `Rule::RogueComboWeightPercent` (25) от всех одиночных особых того же вида (поровну на пару:
  вес × доля × число одиночных / число пар, с множителями обеих сил; маг-пешки нет);
  `SPECIAL_POWERS` — восемь сил по порядку. Карты сил: `Ware::Card(Power)`, `Chest::Cards`
  (`Rule::RogueChestCardsPrice` 15; `chests`/`chest_wares` — на 3 сундука; в `roll_chests` — разные
  силы поровну; купить, пока в руке есть место), рука `Rogue::hand: [Option<Power>; ROGUE_HAND_MAX]`
  (5; держат `hand_size` = `Rule::RogueHandSize` 3; `receive` кладёт карту), `Position::hand`,
  `cards_playable` (ход игрока, не расстановка и не магазин), `power_card_targets(power)` (своя, не
  король/босс, не объединённая, без этой силы, маг — не пешка), `play_power_card(slot, sq)` →
  `Piece::with_power` + `crested`, рука сдвигается; `RogueAction::PlayCard { power, square, piece }`
  («Сыграна карта огня на g1: теперь это …»), `Game::play_power_card`; `Power::card_title`/
  `card_description`. Интерфейс (`table.rs`): рука — в левом верхнем углу поля (`POWER_HAND_AT`,
  `hand_slots`/`hand_under`/`hand_usable`/`paint_hand`, выбранная приподнята на `POWER_HAND_LIFT` и в
  золотой рамке), в `app.rs` — `power_pick`/`power_hover`/`picked_power`: щелчок по карте выбирает,
  цели — `Marker::Move` (в `is_target`), щелчок по цели играет (`press`); подсказка — `crest_hover`
  теперь `(Ware, Rect)` (гербы и карты, `paint_crest_tip`); в сундуке карта во всё место
  (`paint_ware`). Арт — наши `tools/breach_power_cards.py` (`ui/card_<fire|spark|wind|gold|shade|ice|
  barbarian|wizard>.png`, 51×70: рамка тёмного металла с золотыми уголками и ромбами, поле в цвет
  силы, знак из `breach_crest.crest(знак, bare=True)` — у `crest` для этого режим без щита; новый знак
  `axe` — один топор) и третий сундук `ui/chest_cards.png` (`breach_chest.py`: синий, серебро, на
  крышке — карта с красным ромбом); `POWER_CARD`, `UiTextures::power_card`. В `balance.ini`
  пользователя `rogue_combo_weight_percent` 100 → 25, копия `balance.ini.before-0.111`.
  **0.112** (просьбы пользователя): продажа снова — `Position::sell`/`Game::sell`, `sell_price`
  (обычная такая же в магазине, золотая × `Rule::RogueGoldSellPercent` 200, снова не в
  `RETIRED_RULES`), `can_sell` (магазин, сундук не открыт), `RogueAction::Sell`; зона `ShopSpot::Sell`
  слева от панели (`SELL_WIDTH`), `sell_drag` и `reserve_sellable` в приложении — как в 0.107. Продажа
  гербов — `sell_crest` (`crest_sell_price` = `Rule::RogueCrestSellPrice` 5, гербы сдвигаются, фигуры
  `crested` заново), `RogueAction::SellCrest`; в магазине щелчок по гербу в углу (`corner_crest_under`)
  выбирает его (`crest_sell`, золотая рамка), под ним кнопка «Продать» с ценой (`crest_sell_spot`,
  `CREST_SELL`, `crest_sell_hover`). Карты — `Card { Power(Power), Mask, Crest, Double, Coin, Mirror,
  Rank, Feather }` (`ALL` — 15, `title`/`description`/`picks`), `Ware::Card(Card)`, рука
  `[Option<Card>; …]`, `Rogue::{last_card, card_rolls}`. В сундуке карт — разные, поровну, зеркало —
  `Rule::RogueRareCardPercent` (30) от обычной (`draw_card`). `Position::card_ready` (карты на фигуры
  — `cards_playable` и есть цели, остальные — свой ход, сундук не открыт), `card_squares(card, first)`
  (силы — `power_card_targets`, ранг — свои не король/ферзь/босс, `next_rank`; зеркало — свои не
  король, у которых есть образец `mirror_templates`: другой вид или силы, пешкой — не на первой и
  последней), `use_card(slot, targets)` (маска → `last_card` на её место; остальные уходят из руки и
  становятся `last_card`; герб — случайный некупленный, сперва не с прилавка и не из сундука,
  `card_random`; две монеты — ×2; монета — `sell_total`; перо — две `draw_card` кроме пера),
  `CardUse` в `RogueAction::PlayCard`, описание — `card_text`; `Game::use_card`. `receive` герба
  убирает его с прилавка и из сундуков. Приложение: `click_card` (без целей — сразу, иначе выбор),
  зеркало — `mirror_from` (первая фигура в рамке, потом образец), `picked_card`/`card_squares`,
  подсказка маски — «Сейчас — …». Карты в магазине тоже кликаются (только готовые). Арт — наш
  `tools/breach_power_cards.py`: `ui/card_<mask|crest|double|coin|mirror|rank|feather>.png` (знаки
  `sym_*` нарисованы в генераторе: маска, щит со звездой, две `coin_24`, `coin_32`, ручное зеркало,
  зелёная стрелка, перо), `UiTextures::rogue_card(card)` по `Card::ALL`.
  **0.112.1** (просьба пользователя): десятичная база — атака 10, здоровье 30 и т. д. Внутренние числа
  прежние (`UNIT` = 300 — одна «старая» единица, тесты считают в ней), показ и файл — в очках
  `balance::POINT` = `UNIT` / 10: `shown`, `to_file`, `from_file` (старое правило «число от 100 —
  прежний файл» убрано). Правила здоровья и урона rogue-like в `balance.ini` — в очках: в коде
  `rule(…).clamp(0, 999) * POINT`, стандартные ×10 (`rogue_knight_hp` 30, `rogue_boss_hp` 100, …);
  «не меньше одной единицы» — по-прежнему `UNIT`. `balance.ini` пользователя переведён (атака и
  здоровье фигур, `lava_damage` и эти правила ×10), копия `balance.ini.before-0.112.1`; файл друга на
  Mac переводить вручную тем же способом.
  **0.113** (просьба пользователя): в магазине два сундука, бывают большие. `Rogue::chests:
  [Option<ChestOffer>; CHEST_SLOTS]` (3; в магазине — `chest_slots` = `Rule::RogueChestSlots` 2) вместо
  `chest_wares`/`chests` по видам; `ChestOffer { kind, big, state, taken, wares }` (`title` «Большой
  сундук фигур», `choices`, `picks`, `price`, `wares`). `roll_chests` → `roll_chests_with(forced)`:
  разные виды в случайном порядке, большой — с шансом `RogueBigChestPercent` (30); у большого
  `RogueBigChestChoices` (5) на выбор, берут `RogueBigChestPicks` (2), цена × `RogueBigChestPricePercent`
  (200). API по месту: `chests`, `chest(slot)`, `chest_wares(slot)`, `chest_open` → место,
  `chest_for_sale(slot)` (сундук карт — пока в руке есть место на все `picks`), `open_chest(slot)`,
  `take_from_chest(item)` (взятое — `None`, `taken` +1; набрали `picks` — `Taken`);
  `RogueAction::OpenChest { chest, big, price }`. В тестах — `set_chests(&[(вид, большой)])`.
  Интерфейс: `ShopSpot::Chest(место)`, ряд сундуков по их числу, большой — `BIG_CHEST` 55×46 (дно на
  той же линии), заголовок открытого — «… · выберите N». Арт — `tools/breach_chest.py`: `chest(kind,
  big)`, `ui/chest_<вид>_big.png` (шире и выше, золотые уголки, красные камни на полосах),
  `UiTextures::chest(kind, big)`.
  **0.114** (просьбы пользователя): левой панели партии нет (`show_reference`, «Справка»,
  `reference_panel`, `new_game_section` убраны). В главном меню — кнопки «Настройки» и «Правила»
  (`Screen::{Settings, Rules}`, `Choice::{Settings, Rules}`, `settings_screen`/`rules_screen` в
  `menu.rs`; содержимое egui в плашке — `panel`: рамка как у кнопок, внутри `ScrollArea`; Esc —
  назад): настройки — `settings_section`, правила — `rules_section` (список `rules()`) слева,
  `stats_section` и `damage_section` справа, здоровье — `rogue_max_hp` (rogue-like всегда;
  `Position::kind_max_hp` теперь только для тестов). На месте панели — купленные гербы: колонка
  слева от руки и запаса (`crest_column` в `table.rs`, `CRESTS_AT` = (−174, −32), по
  `CREST_COLUMNS` = 2 в ряд, не помещаются до `VIEW_MAX.y` — ряды сходятся, попадание — с
  последнего), в rogue-like вписывается `ROGUE_VIEW_MIN` = (−178, −34) (`board_area`); кнопка
  «Продать» — под гербом, не правее колонки; подсказка герба колонки — справа от него;
  `corner_crest_under` → `column_crest_under`. Фигуры волн уровня — все усиленные своего вида:
  `rogue_<poison|armor|vampire|zombie|armored_vampire|venom_zombie>_percent` = 100 (в
  `balance.ini` пользователя 50 → 100, копия `balance.ini.before-0.114`); на 1-м уровне и с 8-го
  вида нет — обычные.
  **0.115** (просьба пользователя): электричество фигуры в покое вдвое медленнее —
  `SPARK_FRAME_TIME` = 2 × `ELECTRIC_FRAME_TIME` (120 мс) в `spark_frame` и в запасе/магазине
  (`paint_stock`); вспышка удара и другие силы (вихрь, аура, в том числе их полосы у объединённых с
  молнией) — по-прежнему 60 мс.
  **0.116** (просьба пользователя: ещё в полтора раза медленнее, но так же плавно, как до замедлений):
  `SPARK_FRAME_TIME` убран, кадр снова 60 мс, а полосы — замедленные `electric_slow_*` (все 12 фигур и
  основы `barbarian`/`gold`/`wizard`/`ice`/`fire`) из нашего `tools/breach_spark_slow.py`:
  `breach_arcs.render(..., slow=3, length=72)` — у `render` новый `slow` (с 1 кадры байт-в-байт
  прежние): аура течёт по пикселю за кадр, рост ауры и вспышка тела плавные, молнии и искорки ауры
  выпадают раз в `slow` кадров теми же бросками зерна, искры летят медленнее; `bodies(f)`/`under(f)` —
  по настоящему кадру (у огненной трещины мерцают с прежней скоростью). В игре `ELECTRIC_SLOW` = 3:
  `ELECTRIC_FRAMES` 72, `ELECTRIC_FLASH` 15, `ELECTRIC_LOOP` [24, 72]; вспышка удара — кадры
  `ELECTRIC_FLASH + ELECTRIC_SLOW × (0..3)` по 60 мс (прежняя скорость), сдвиг повтора по клетке ×3.
  Старые `electric_hold_*` остаются (арт-проект и превью объединённых).
  **0.117** (пользователь: «молния всё равно не плавная» — молнии появлялись разом раз в 3 кадра и
  стояли): в `render` при `slow` > 1 каждая молния — `Bolt::born`/`span` — в свой кадр внутри шага
  (второе зерно `seed + 991`, вспышка — разом), первые `GROW` = 2 кадра прорастает от начала
  (`Bolt::prefix`), потом гаснет ступенями (`paint_bolts_slow`: яркая до 45% жизни, голубая до 75%,
  тёмно-голубая); вспышка на концах — у начала, пока растёт, у конца — когда доросла; искорки ауры
  тоже вразнобой. Повтор сшит: полоса считается дважды (`_wrap`, `render.timeline`) — молнии конца
  повтора догорают в его начале, молнии вступления в повтор не переходят. Полосы `electric_slow_*`
  перерисованы тем же `tools/breach_spark_slow.py`, код игры не менялся.
  **0.118** (просьба пользователя): убраны все гербы скидок и частоты фигур — `Crest::{Coin, Pawn,
  AxeQueen, Axes, FlamePawn, FlameCoin, DarkRook, DarkCoin, WindCoin, WindKnight, BoltBishop, BoltCoin,
  Pawns, Bishops, Knights, Rooks, Queens, HatCoin, HatBishop, Kings2}` (`CRESTS` = 31, их картинки
  убраны из `CRESTS` в `assets/ui.rs`, файлы остались), их правила — в `RETIRED_RULES` (из
  `balance.ini` пользователя строки удалены, копия `balance.ini.before-0.118`). `Position::shop_price`
  = свободная `shop_price`; `power_percent` и `paired` убраны — в `stock_weights` их множители
  стали постоянными ×100 (веса и броски прилавка без гербов прежние). Герб монеты и обновления
  (`RerollCoin`) остался — он об обновлении, не о фигурах.
  **0.119** (просьба пользователя): мульти. Урон удара фигуры игрока — `Position::strike_damage` =
  атака × `Position::mult()`, до целого очка (`POINT`), не больше `MAX_DAMAGE_POINTS`; у фигур волн —
  атака. `mult()` — свёртка гербов в порядке покупки от 1: `Crest::Mult` — `m + mult_add()`
  (`Rule::RogueCrestMultAddPercent` 200, в сотых), `Crest::KingMult` — `m × king_mult()^king_hits`
  (`Rule::RogueCrestKingMultPercent` 150); не больше `MAX_MULT`. `Rogue::king_hits` — удары короля
  игрока (не толстосума) в этом бою: `count_king_hit` после удара в `strike` и в раундах `strike_all`
  (круговой не считается); обнуляется в `begin_planning`. Мульти берут: обычный удар, повторы, удар
  тени насквозь (`DarkSkull`), разлёт молнии (`SparkPercent` от урона удара); круговой удар,
  взрывы, метеориты, яд — нет. Гербы — `CRESTS` = 33, картинки `ui/crest_<mult|king_mult>.png` из
  `tools/breach_crest.py` (меч / малый король вверху, низ пустой), число на них пишет игра —
  `Crest::badge` («+2», «×1.5», `mult_text`), `paint_crest_badge` (в колонке и на прилавке, строка
  `BADGE_TOP` = 26). Мульти игрока — над числом золота (`paint_mult`, `MULT_RED`), в карточке фигуры
  «Атака 10 × 2.25 = 23», на табличке атаки — урон. Шрифт: `Glyph::{Times, Dot}` (`*`, `.`),
  `Glyph::text`.
  **0.120** (просьба пользователя): **правило** — удары короля и все прибавки к мульти от ударов или
  временных эффектов действуют до конца хода, если пользователь не уточнит иначе: `end_turn_mult` в
  `end_turn` после ударов, взрывов, метеоритов, яда и `death_blasts` обнуляет `Rogue::king_hits`
  (новые такие счётчики сбрасывать там же). Предела урона нет: `MAX_MULT`/`MAX_DAMAGE_POINTS`
  убраны, `strike_damage` насыщается только на `i32::MAX` (~71 млн очков; `as` у f64), проценты от
  урона — `percent_of` (через i64: молния, шипы, оборона), суммы урона в нотации — `damage_sum`.
  **0.121** (просьба пользователя): гербы мульти и мульти у взрывов и метеоритов. Мульти — по фигуре
  и виду урона: `mult_for(attacker, from, Blow)` (`Blow::{Strike, Blast, Meteor}`), свёртка гербов
  по порядку покупки; `damage(from, attacker, blow)` — урон фигуры игрока (у волн — атака);
  `strike_damage(piece)` и `piece_mult(piece)` — для глаз (без первой бьющей), `mult()` — общее
  (над золотом, без гербов отдельных фигур и взрыва). Счёт хода — `Rogue::turn: TurnCounts`
  (`king_hits`, `pawn_hits`, `knight_hits`, `queen_hits`, `blasts`, `meteors`, `jumped`, маска
  `struck`, `first`), копит `note_blow` (удары — в `strike` и раундах `strike_all`, кроме кругового;
  взрывы — `explode`; метеориты — `meteors`; прыжки теней — `make_move`, `shade_passed`), сброс —
  `end_turn_mult` и `begin_planning`; `advance_to` переносит `struck`/`first`. Новые гербы
  (`CRESTS` = 41): `FewMult` (× (`RogueCrestFewBase` 16 − своих на доске), не меньше ×1 — выбор
  пользователя), `GoldMult` (золотым + золото), `PawnMult` (×1.5^удары пешек), `KnightMult` (коням
  +1 × удары коней), `SwordMult` (×1.2^ударившие раньше), `FirstStrike` (первой фигуре, нанёсшей
  урон, +10), `QueenMult` (+3 × удары ферзя), `FlameMult` (×1.5^взрывы), `SnowMult` (×2^замёрзшие,
  все на доске — выбор пользователя); числа — `Rule::RogueCrest*Percent` в сотых. Изменены: `AxeSword`
  — варварам ×2 (атаки сверху и `bonus_attack` в `crested` больше нет, −здоровье осталось),
  `FlameSword` — взрыву +2 (выбор пользователя: «сделать мульти-гербом»), `Bomb` — взрыву ×2 (и
  задевает своих), `Meteor` — первому метеориту в ходу +10, `DarkSkull` — +3 за каждую фигуру, через
  которую прошли теневые (удара насквозь, `Outcome::passed` и его анимации больше нет). Взрыв —
  `explode(center, piece)`, урон `damage(.., Blast)` (`fire_damage` убран), метеорит — урон мага
  (`Blow::Meteor`), кратер — `Position::crater_damage` (урон упавшего метеорита). Герб снежинки и
  шипов убран (`Strike::thorns`, `Strike::wounded` — теперь `hits`). Правила `lava_damage` (с ним —
  `rule_in_units`), `rogue_crest_spikes_percent`, `rogue_crest_axe_attack`, `rogue_wizard_damage`,
  `rogue_crest_flame_sword_damage`, `rogue_crest_bomb_damage`, `rogue_crest_meteor_damage`,
  `rogue_crest_crater_damage` — в `RETIRED_RULES` (из `balance.ini` пользователя удалены, копия
  `balance.ini.before-0.121`). Числа на гербах — `Position::crest_badge` (у `FewMult` и `GoldMult` —
  сколько дают сейчас). Картинки — `tools/breach_crest.py`: `few`, `gold_mult`, `pawn_mult`,
  `knight_mult`, `sword_mult`, `first`, `queen_mult`, `flame_mult`, `snow_mult` (вверху слева знак —
  король, пешка, конь, меч, ферзь, малый огонёк, снежинка; справа красный `TIMES` 9×9, `SIXTEEN` или
  `ONE`; низ — под число), превью `preview_crest_mults2.png`.
  **0.122** (просьбы пользователя): здоровье фигур волн растёт по уровням — `Piece::tough` (уровень −
  1, ставят `spawn_wave` и `rise_zombies`), `max_hp` и здоровье восставшего — через `Piece::scaled`
  (× `Rule::RogueLevelHpPercent` / 100 = 5 за уровень, насыщается на `i32::MAX`; уровень 10 — ×5⁹),
  «армия: N HP» в панели — сумма с насыщением. Цены: герб — 10 (`RogueCrestPrice`), объединённая — ×2
  от обычной, как одиночные особые (`RogueComboPercent` 200), сундуки фигур/гербов/карт — 10/15/10
  (большие ×200% — 20/30/20, на 10 дешевле прежних). В `balance.ini` пользователя заменено, копия
  `balance.ini.before-0.122`.
  **0.123** (просьбы пользователя): сундук гербов — 10 (`RogueChestCrestsPrice`, большой — 20);
  круговой удар варвара бьёт уроном самой фигуры (`damage(.., Strike)`, в счёт ударов хода не
  идёт), правило `rogue_crest_whirl_damage` — в `RETIRED_RULES` (в `balance.ini` пользователя
  удалено, цена заменена, копия `balance.ini.before-0.123`). **Список гербов для пользователя —
  `Гербы.txt` в корне** (с 0.134 — `Усиления.txt`) (название и описание каждого, числа стандартного баланса; пишет
  `crest_list_text` в `game/tests.rs`). После любой правки гербов (новый, другое описание или
  число) — `cargo test --release refresh_crest_list -- --ignored`; тест `crest_list_is_up_to_date`
  падает, если файл устарел.
  **0.124** (просьбы пользователя): гербы `Crest::{BoltMult, WindMult}` (`CRESTS` = 43; картинки
  `ui/crest_<bolt_mult|wind_mult>.png` — малая молния / малый смерч и красный «×»): молнии — мульти
  + `RogueCrestBoltAddPercent` (500) за каждую фигуру перед задетой в цепочке (`Blow::Spark(до неё)`;
  в `strike_raw` у источников молнии — глубина цепочки, цель — 1), воздушным — + `RogueCrestWindAddPercent`
  (500) × `TurnCounts::wind_moves` (ходы воздушных игрока за ход, в `make_move`). Здоровье волн — ×4
  за уровень (`RogueLevelHpPercent` 400). Гербы складываются (`Crest::stacks`: все мульти, сердце,
  двойная снежинка (+1 ход за копию), топор и меч, топор и клыки, огненный топор, огонь и меч, бомба,
  теневой огонь и череп, две шляпы (+1 метеорит за копию), метеорит, король, король и ферзь, копилка):
  `Rogue::crests` — `[Option<Crest>; CREST_SLOTS]` (64), `crest_count`/`crest_times` (числа × копии в
  `crested`, `barbarian_repeats`, `strike_raw`, `strike`, `meteors`, `piggy_bank`), мульти — каждая
  копия в свёртке. Доступность — `crest_available` (место есть и герб складывается или его нет):
  `roll_shop`, `roll_chests`, `buy`, `take_from_chest`, карта герба; `receive` убирает с прилавка и из
  сундуков только гербы-механики. Продажа — по месту: `sell_crest(index)`, в приложении
  `crest_sell: Option<usize>`, `column_crest_under`/`crest_sell_spot` — по месту в колонке.
  Подсказка складывающегося герба — «Складывается: можно купить ещё (у вас — N)», в `Гербы.txt` —
  «Складывается.». В `balance.ini` пользователя `rogue_level_hp_percent` 500 → 400, копия
  `balance.ini.before-0.124`.
  **0.125** (просьбы пользователя): герб коня и умножения — мульти всем фигурам (без проверки вида в
  `mult_for`); герб урона — +4 (`RogueCrestMultAddPercent` 400; в `balance.ini` пользователя 200 → 400,
  копия `balance.ini.before-0.125`).
  **0.126** (просьбы пользователя): прилавка нет — в магазине только сундуки (`Rogue::wares`, `roll_shop`,
  `buy`, `ware_price`, `crest_price`, `SHOP_MAX`, `RogueAction::{Buy, BuyCrest}`, `Game::buy`, `ShopSpot::Offer`
  убраны; правила `rogue_shop_slots`, `rogue_shop_piece_percent`, `rogue_crest_price` — в `RETIRED_RULES`).
  «Обновить» (`reroll`) — новые сундуки (`roll_chests`, случайность — `Rogue::rolls`), за 5: `RogueRerollStep`
  = 0 (обновления не дорожают). Сундук медалей — `Chest::Medals` (`CHEST_SLOTS` = 4, `RogueChestMedalsPrice`
  10, большой — ×200%): разные виды фигур (`Ware::Medal(Kind)`, из всех шести), выбранный — `Rogue::medals`
  [`Kind`] +1; в `mult_for` начальное мульти фигуры игрока = 1 + `RogueMedalMultPercent` / 100 × медали её
  вида — раньше гербов (общее `mult()` — без медалей); `medal_title`/`medal_description`, `Position::medals`.
  Карты играют в раунде, в магазине и в расстановке (`cards_playable` — только «ходит игрок»; `card_ready`
  — ещё не открыт сундук). Продажа карт — `sell_card(slot)` за `card_sell_price` (`RogueCardSellPrice` 3),
  `RogueAction::SellCard`; `sell_total` (карта монеты) считает и карты в руке. Интерфейс: панель магазина
  — только сундуки (не уже `SHOP_MIN_COLUMNS` = 3 мест), над её правым верхним углом «свернуть»
  (`ShopSpot::Fold`); свёрнутый (`shop_folded`; `shop_shown` = `shop_ready` и не свёрнут) — доска не
  притушена, ввод — как в ходу, наверху посреди поля кнопка «Магазин» (`unfold_rect`, `unfold_hover`). В
  развёрнутом магазине щелчок по карте открывает меню (`card_menu`, `CardMenu::{Play, Sell}`,
  `card_menu_spots`/`card_menu_under`/`card_menu_usable`/`paint_card_menu`: плашки под картой); «Сыграть»
  карты с целью сворачивает магазин сам (`fold_for_card`), сыграли или отменили — он снова развёрнут.
  Медали игрока — в правом верхнем углу поля (`medal_row`, `MEDALS_AT`, «×N» под каждой, подсказка — через
  `crest_under`/`paint_crest_tip`). Арт: `tools/breach_medal.py` (наш: `ui/medal_<p|n|b|r|q|k>.png` 32×44 —
  золотой диск на красной ленте с белой полосой, выбит золотой знак фигуры из `breach_crest`; `MEDAL`,
  `UiTextures::medal`), сундук медалей в `tools/breach_chest.py` (`ui/chest_medals[_big].png`: тёмно-зелёный,
  золотые полосы, на крышке медаль). В `balance.ini` пользователя строки прилавка удалены,
  `rogue_reroll_step` 1 → 0, копия `balance.ini.before-0.126`.
  **0.126.1** (просьба пользователя): в магазине три сундука (`RogueChestSlots` 3; в `balance.ini`
  пользователя 2 → 3, копия `balance.ini.before-0.126.1`).
  **0.127** (просьбы пользователя): ледяные фигуры — с `Rule::RogueIceHp` (10) здоровья сверху (в `crested`; ставшая
  ледяной картой силы получает их сразу — `crested` прибавляет разницу); взрывы для герба огня и умножения
  копятся до конца раунда (`end_turn_mult` сохраняет `TurnCounts::blasts`, сброс — `begin_planning`). Числа:
  конь и «×» +2, снежинка и «×» ×3, смерч и «×» +8, огонь и меч +5 (в `balance.ini` пользователя заменены, копия
  `balance.ini.before-0.127`).
  **0.128** (просьба пользователя): десять гербов (`CRESTS` = 53; картинки из `tools/breach_crest.py`: `bishops2`
  (прежний файл), `mirror`, `nine`, `card`, `pawn_card`, `queen_skull`, `pawn_cards`, `king_medal`, `medals2`,
  `queen_two`, превью `preview_crest_0128.png`; знаки `NINE`, `TWO`, `CARD_BIG`/`CARD_SMALL`, `MEDAL_SMALL`,
  многоцветные — `colored`). `BishopPair` — мульти + `RogueCrestBishopAddPercent` (400) × `TurnCounts::bishop_pairs`
  (слон игрока ранил за ход две разные фигуры: `note_hit` из `strike_raw` — цель и молния, `explode`, `meteors`;
  `bishop_first`/`bishop_done`; до конца раунда — `end_turn_mult` его сохраняет). `Mirror` — копия герба перед ним:
  `Position::active_crests` (зеркало → последний уже разобранный; первым — ничего), им пользуются `has_crest`,
  `crest_count`, свёртка `mult_for`; `crests()` — по покупке (колонка, продажа), `crest_available` — по купленным.
  `Nine` — `Rogue::attackers` (первый удар фигуры за ход, за забег, в `note_blow`), каждая `RogueCrestNineEvery`-я
  (9) — метка `TurnCounts::nine`, мульти × `RogueCrestNineMultPercent` (900). `CardMult` — + `RogueCrestCardAddPercent`
  (200) × `Rogue::cards_used` (в `use_card`). Карты — `gift_card` (случайная, если есть место): `PawnCard` —
  превращение пешки игрока не в ферзя (`make_move`), `QueenSkull` — гибель ферзя игрока (`fall`), `PawnCards` — каждые
  `RogueCrestPawnKills` (4) убийств пешками за ход (`note_hit`). `KingMedal` — золотая фигура убила
  `RogueCrestGoldKills` (2) за ход — медаль её вида; `Medals2` — в `receive` медали ещё одна случайного другого вида.
  `QueenTwo` — в `strike_all` первая ударившая за ход фигура бьёт ещё раунд (`twice`; круговой удар — только у
  варвара). Складываются: два слона, зеркало, девятка, карта, две медали, ферзь и двойка.
  **0.129** (просьба пользователя): двенадцать гербов, все складываются (`CRESTS` = 65; картинки из
  `tools/breach_crest.py`: `knife_crest`, `knife_king`, `knife_coin`, `flame_two`, `dice`, `blank` (пустой щит),
  `queen_coin`, `skull`, `silver_medals`, `bush`, `knight_coin`, `coin_clock`, превью `preview_crest_0129.png`; знаки
  `KNIFE`, `MINI_SHIELD`, `DIE`, `QUESTION`, `LEAF`; монеты — `coins_at`, точки и стрелки — `after`, поверх знаков).
  Счёт забега в `Rogue`: `crest_sale_gold` (`sell_crest`), `piece_sale_gold` и `pieces_sold` (`sell`), `dice`, `bush`,
  `round_kill`, `luck_rolls` (`Position::luck` — своя случайность: число, череп, серебряные медали). `KnifeCrest` /
  `KnifeKing` — мульти + золото за проданные гербы / фигуры, `KnifeCoin` — + `RogueCrestKnifeCoinPercent` (50)% золота
  за фигуры; `Dice` — + `Rogue::dice` (1…`RogueCrestDiceMax` 10, `roll_dice` в `rogue_start` и `end_turn_mult`);
  `Blank` — + `RogueCrestBlankAddPercent` (1000), если `reserve(игрок)` пуст; `QueenCoin` — фигуре + её `sell_price`
  (числа на гербе нет); `Skull` — + `RogueCrestSkullAddPercent` (1500), `crumble_skulls` в `end_turn` после
  `end_turn_mult` (ход игрока, каждый с шансом 1 к `RogueCrestSkullChance` 25) → `TurnEnd::crumbled`, «Герб черепа
  рассыпался»; `MedalChance` («двух серебряных медалей», герб двух медалей уже есть) — в `note_hit` убийство фигурой
  игрока, каждый с шансом 1 к `RogueCrestMedalChance` (25) — медаль её вида; `Bush` — + `RogueCrestBushAddPercent`
  (200) × `Rogue::bush` (удар фигуры игрока без убийства в `strike_raw` +1, любое убийство в `note_hit` — 0; через
  конец хода не сбрасывается — пользователь назвал только сброс убийством); `KnightCoin` — первое убийство раунда
  (`round_kill`, сброс в `begin_planning`) сделал конь игрока — `RogueCrestKnightGold` (10) за копию; `CoinClock` —
  мульти × (`RogueCrestClockBasePercent` 200 + `RogueCrestClockStepPercent` 25 × `pieces_sold`) / 100; `FlameTwo` — в
  `lava_blasts` первая взорвавшаяся по `strike_queue` огненная взрывается ещё раз за копию, пока жива и
  `blast_reaches`; в приложении повторный взрыв той же клетки ждёт конца прежнего (`busy` в `queue_strikes`),
  насыщение огнём — раз на клетку. Числа на гербах — в `crest_badge` (ножи, число, куст — сколько дают сейчас).
  Исправлено: сундук, из которого нечего взять (гербам нет места из 64), не открывается (`chest_for_sale`,
  `takeable`) и закрывается, если после взятого брать больше нечего (`take_from_chest`).
  **0.130** (просьба пользователя): в волне от 3 до 6 фигур (`rogue_wave_min`/`rogue_wave_max` 3/6; в
  `balance.ini` пользователя 2/4 → 3/6, копия `balance.ini.before-0.130`).
  **0.131** (просьбы пользователя): толстосум — здоровье 80 (`rogue_boss_hp`; пользователь сказал «8
  здоровья» — понято в старых единицах, при десятичной базе было 100, минимум правила 10), бьёт не сквозь
  фигуры (`line_clear` проверяет и `boss`). Золото за любую побеждённую фигуру волн и толстосума +
  `Rule::RogueKillGoldBonus` (1) в `kill_gold`; `rogue_price` (база цен магазина и продажи) не менялся.
  Волн на 2 меньше (`rogue_waves` 10 → 8; толстосум — по-прежнему в последней волне первого уровня).
  Здоровье волн ×2.5 за уровень (`rogue_level_hp_percent` 250). Базовый урон — `Position::base_damage`:
  атака, у фигур игрока + `Rule::RogueMedalAttack` (5 очков, `medal_attack`) за медаль вида; `damage`
  умножает на мульти его. Табличка атаки при наведении (`paint_stats`) — базовый урон; выбрана своя фигура —
  над монетой вместо общего мульти `paint_damage_mult` (`table.rs`): синяя табличка с базовым уроном (число у
  «×»), красный `Glyph::Times`, красная с `piece_mult` (по образцу пользователя, как в Balatro). В
  `rogue_fuzz` проверка «золото < 30 000» заменена на проверку перескока через ноль (копилка поднимала
  золото до 32 тысяч). В `balance.ini` пользователя: волны 8, толстосум 80, рост здоровья 250, новые строки
  `rogue_medal_attack`, `rogue_kill_gold_bonus`; копия `balance.ini.before-0.131`.
  **0.132** (просьба пользователя): король игрока тоже становится особым — карты сил играют и на него
  (`power_card_targets`: только не толстосум; зеркало и ранг короля по-прежнему не берут). Спрайты и анимации
  королей во всех наборах уже были (12 фигур везде), дорисовывать не пришлось. Теневой король — свой ход в
  `shade_moves` (`Kind::King`: на 1 или 2 клетки по `KING_STEPS`, сквозь фигуру) плюс `castling_moves` в
  `pseudo_moves`. Портал герба теневого огня и портала теперь может унести короля — в `strike_raw` после
  `board.swap` обновляется `kings` (ошибку нашёл `rogue_like_fuzz_long`). Тест
  `rogue_like_power_cards_make_the_king_special`.
  **0.133** (просьба пользователя): город вместо магазина и гербов, и файла баланса больше нет.
  **Город** — этап между раундами на своём поле 8×8 (вместо магазина; фигур на нём нет; `begin_planning`
  открывает его — `Rogue::shop`, «Уйти» → расстановка). Здания — `Building` (16, `ALL`; `SHOP` — 14: по
  виду фигур и по особой силе), `Placed { building, at, mirrored, upgrade, seq }` (`at` — угол у вертикали a
  и 1-й горизонтали; клетки — `Building::cells` по `FOOTPRINT`/`mirror_cells` арт-проекта:
  `tools/breach_city.py`; развернуть — только у кого есть спрайт `_m`, `can_mirror`), `Rogue::{city,
  stash, decrees, upgrades}`; главное здание (d1–e2) и набор новобранцев (a1–b2) стоят с начала. Набор —
  обычные пешки игрока в запасе без конца (`ENDLESS`, `Position::endless`: `reserve`, `stock_ready`,
  `take_stock`, `give_stock`; не продаются; 2 пешки в начале больше не кладутся). Сундуки — только
  `Chest::{Pieces, Buildings}` (гербов, карт, медалей — нет; при 3 местах первые два — разных видов);
  `Ware::Building` — в колонку купленных (`stash`, `takeable` — пока есть место, `STASH_MAX` 32). Постройка —
  `place_building` (`can_place`), продажа — `sell_building(BuildingSpot::{Stash, City})` за
  `building_sell_price` (`Rule::RogueBuildingPrice` 10 × `RogueBuildingSellPercent` 50%; главное и набор не
  продаются; `building_sale_gold` — для «ножа и герба»), указ — `buy_decree` (`decree_price` =
  `Rule::RogueDecreePrice` 10, сколько угодно) и `use_decree(клетка)` (`can_decree`: здание из `SHOP` без
  усиления): усиление — с шансом `Rule::RogueDecreeOwnPercent` (50) из `Building::own_upgrades`, иначе из
  `COMMON_UPGRADES` (урон, меч, первая, девятка, карта, ножи, кубик, пустой, череп, куст, часы); череп,
  рассыпавшись, снимает усиление (`crumble_skulls`). Здание — как медаль: `buildings_for(фигура)` — базовый
  урон + `Rule::RogueBuildingAttack` (5), мульти + `RogueBuildingMultPercent` (100) за каждое своё здание.
  Усиления — `Position::upgrades()` (по `seq`, с их зданием); в `mult_for` мульти-усиление действует только
  на фигуры своего здания (`Building::serves`); остальные действия гербов — как были (`has_crest`,
  `crest_count`). Медалей нет (`medals`, `medal_*`, сундук, ряд в углу); гербы зеркала, обновления и трёх
  медалей удалены (`CRESTS` 60, картинки убраны из `CRESTS` в `assets/ui.rs`); `Crest::stacks`,
  `crest_available`, `sell_crest`, `crest_sell_price` убраны. Гербы без зданий — только в тестах
  (`#[cfg(test)] Rogue::crests`, `give_crest`, `crest_position`): действуют на все фигуры. Карта герба —
  `Card::Decree` (+1 указ; картинка прежняя). Карты в городе не играют (`cards_playable`: не `shop`),
  только продаются. Действия — `RogueAction::{Place, SellBuilding, BuyDecree, Decree}`. Интерфейс
  (`table.rs`): `city_view` — поле города вместо доски с фигурами (`paint_board` → `paint_city`: здания по
  `city_order` — порт `draw_order` арт-проекта, тени, `building_rect` — точка начала осей на левый угол
  клетки `at`; клетки ставящегося здания — зелёные/красные, «призрак» `CITY_GHOST`; табличка «Указы: N» над
  главным); колонка купленных — на месте гербов (`stash_column`, `paint_stash`, `stash_under`,
  `stash_sell_spot` — «Продать» под выбранным); меню здания — `BuildingMenu::{BuyDecree, Decree, Sell}`
  (`building_menu_spots`/`_under`/`_usable`, `paint_building_menu`); подсказка — `Tip::{Card, Building,
  Placed}` (`tip_under`, `paint_tip`: у усиленного здания — картинка герба и его описание, «Мульти — только
  у …»). Ввод — `city_input`/`shop_input` в `app.rs`: щелчок по зданию колонки выбирает его (магазин
  сворачивается — виден город), щелчок по клетке ставит, правая кнопка разворачивает, Esc — отмена; щелчок
  по зданию в городе открывает меню. Мульти над монетой без выбранной фигуры не показывается (общего мульти
  нет). Арт: спрайты зданий `assets/breach/city/<имя>[_m].png`, `_shadow`, `anchor_*.txt` — из арт-проекта
  (`UiTextures::building`), значки колонки — наш `tools/breach_city_icons.py` (те же здания тем же движком
  мельче: `icon_<имя>.png`, `UiTextures::building_icon`), сундук зданий — `tools/breach_chest.py`
  (`chest_buildings[_big].png`: серый камень, золото, домик). `Гербы.txt` — теперь здания и их усиления.
  **Файла `balance.ini` нет** (просьба пользователя: «удали balance.ini и всю логику, кинь это в текстовый
  документ»): числа — только в таблицах `balance.rs` (`balance()` — стандартный баланс); чтение и запись
  файла, предупреждения, `src/editor.rs` (сервер редактора), кнопка «Править баланс», вставка кода колоды
  (Ctrl+V), `tools/deck_builder.html` и тест `refresh_deck_builder` удалены. Список всех чисел для
  пользователя — **`Баланс.txt`** в корне (`Balance::text`); после правки чисел —
  `cargo test --release refresh_balance_list -- --ignored` (тест `balance_list_is_up_to_date` падает, если
  файл устарел). Файлы `balance.ini`, его копии и `editor.key` у пользователя удалены (его числа совпадали со
  стандартными, кроме здоровья коня 20 — в rogue-like его и так заменяет `rogue_knight_hp` 30).

  **0.134** (просьбы пользователя): магазин — здание города `Building::Shop` (2×2 на g1–h2, придумано мной в
  стиле арт-проекта: «лавка мага» — `build_shop` в `tools/breach_city.py`: каменный низ с фиолетовыми
  витринами `MAGIC`, фахверк, щипец к фасаду, полосатый навес `AWNING`/`CANVAS` + `stripes`, вывеска с монетой,
  башенка со светящимся шаром, бочки и сундук; спрайт `city/shop.png`, значок `icon_shop.png`). После раунда
  город открывается с закрытым магазином (`shop_folded` по умолчанию `true`): щелчок по зданию магазина
  открывает панель сундуков, «закрыть» (бывш. «свернуть») — назад к городу; кнопки «Магазин» наверху нет
  (`unfold_*` убраны). Большая кнопка в городе («Начать раунд», `city_leavable`) — `leave_shop`, к
  расстановке. Пешки не бесконечные (`ENDLESS`/`endless` убраны, в начале снова 2 пешки в запасе): щелчок по
  набору новобранцев — меню «Купить пешку» (`BuildingMenu::BuyRecruit`, `Position::buy_recruit`,
  `RogueAction::Recruit`, цена `recruit_price` = `Rule::RogueRecruitPrice` 5, сколько угодно, пешка — в
  запас). Плашки меню зданий — `BUILDING_MENU` 96×18. Гербов игрок не видит: усиление здания — только описание
  (`paint_tip`: «Усиление указа: …», без картинки и названия; в истории — «Указ: Конюшня»), картинки гербов
  из `assets/ui.rs` убраны (`UiTextures::crest`), `Crest::title` — только для тестов; описания усилений
  без слова «герб». Список для пользователя — `Усиления.txt` (бывший `Гербы.txt`, без названий; тест
  `refresh_crest_list`). Пешки выставляются и расставляются и на первую горизонталь (`arrange_fits` — две
  первые для всех).
  **0.135** (просьбы пользователя): сундук карт снова в магазине — `Chest::Cards` (`ALL` — 3: фигуры, здания,
  карты; `Rule::RogueChestCardsPrice` 10, большой — ×200%; карты разные, зеркало реже — `draw_card`;
  `chest_for_sale` — пока в руке есть место на все `picks`; картинки `chest_cards[_big].png` снова в
  `CHESTS`). «Уйти» в магазине больше не ведёт к расстановке — только закрывает магазин, игрок снова в городе
  (`ShopSpot::Leave` → `shop_folded`); из города — «Начать раунд». Тест
  `rogue_like_card_chest_is_back_and_leave_keeps_the_city`.
  **0.136** (просьбы пользователя): в начале rogue-like особой фигуры в запасе нет (в запасе — 2 пешки), зато
  две случайные фигуры армии (не пешки, разные клетки первой горизонтали) сразу особые —
  `Position::starting_specials(seed)` → `[(Square, Power); 2]` (сила — одна из `SPECIAL_POWERS` поровну;
  `with_power` в `rogue_start`; `starting_special` убрана). Сундуки: вид — по весам
  `Rule::RogueChest{Pieces,Buildings,Cards}Weight` (50/100/100 — сундук фигур вдвое реже, виды могут
  повторяться); в сундуке фигур пешек нет (`stock_weights` без `Kind::Pawn`, ферзь и так реже всех); здания —
  по редкости `Building::weight` (`Rule::RogueBuildingWeight*`: дом ополчения 15, конюшня и церковь 5, башня 3,
  дворец короля 2, дворец ферзя 1, здания сил по 3). Тесты `rogue_like_starts_with_two_special_pieces_on_the_board`,
  частоты — в `rogue_like_chests_offer_one_of_three`.
  **0.137** (просьба пользователя): здоровье фигур волн растёт по прямой, а не степенью — 200% от первого
  уровня на втором, 250% на третьем, 300% на четвёртом… (`game::level_hp_percent(level)`:
  `Rule::RogueLevelHpPercent` 200 — второй уровень, + `Rule::RogueLevelHpStepPercent` 50 за каждый следующий;
  `Piece::scaled` — через i64, насыщается на `i32::MAX`). Атака волн не растёт.
- `src/net.rs` (с 0.45, просьба пользователя: «очень простой мультиплеер») — клиент ретранслятора
  `tools/mp_server.py` (Python 3, без библиотек, `python3 mp_server.py [порт]`, порт 47900):
  сервер сажает двух подключившихся за доску (первый — белые), шлёт `WAIT`/`START white|black`/
  `LEFT` и пересылает строки. Партия детерминирована (без карт, без усиления, баланс —
  `balance::use_default`), поэтому передаются только действия: `M откуда куда вид превращение`
  (`encode_move`/`decode_move`, ход проверяется по `legal_moves`) и `E` (кнопка «Конец хода»),
  плюс `VER` — предупреждение о разных версиях. `Link` — поток чтения (`NetEvent` в канал),
  запись в общий `TcpStream`, `Drop` закрывает. В приложении: `Screen::Lobby` (`lobby_screen` в
  `menu.rs`: поле адреса — egui `TextEdit`, Enter подключает), `net`/`remote`/`net_queue`/
  `net_note`; `poll_net` каждый кадр, `start_network_game`, `drive_remote` (ходы соперника —
  когда доиграли анимации), `send_own` в `play` и `end_turn` (своё уходит, чужое — нет);
  `human_to_move` учитывает `remote`, отмены нет, «Меню» и «Новая партия» рвут соединение.
  Адрес — `server.txt` в папке настроек. Проверка: `python3 tools/mp_server.py 47911` и
  `cargo test --release two_games_stay_in_sync -- --ignored`. Сейчас пользователь на macOS: `install.ps1` не работает,
  сборка — `cargo build --release`, exe — `target/release/battle-chess`.
- `src/game.rs` — правила, `Kind::stats`, нотация. С 0.32 (просьба пользователя) бить ходом
  нельзя: ходы только на пустые клетки (`MoveKind::{Quiet, CastleShort, CastleLong}`, пешка —
  только вперёд, взятия на проходе нет), а фигуры бьют сами в `end_turn` (`strike_all` →
  `Vec<Strike>`, в истории `Event::EndTurn(color, strikes)`, `Record::strikes`): каждая
  свободная фигура стороны, закончившей ход, по очереди `strike_queue` (до 0.109 — a8…a1, b8…h1, с 0.109 — a8…h8, a7…h1;
  `strike_order`) бьёт одну чужую в досягаемости `strike_steps` (одна клетка по правилам
  хода, пешка — по диагонали вперёд, конь — «Г»): цель — меньше HP, при равенстве дороже по
  `Kind::points`, потом раньше в очереди; барьер соперника недосягаем; убийца с 0.33 встаёт
  на клетку цели (маска `done` — второй раз не бьёт; пешка на последней — ферзь,
  `Strike::promoted`; ледяная зона — замерзает), раненую бьёт с места; ответа нет (`Counter`, `forecast`, `landing_square`, цепь пешек убраны);
  `any_attacker`/`can_kill`/`max_damage` (шах, рокировка) — по тем же `strike_steps`. Удар
  снимает ровно атаку (`damage_at`). Если карта оставила без фигур обе стороны, проигрывает
  сыгравший (`refresh` проверяет его первым). Конец хода — `TurnEnd { strikes, blasts, charged }`
  (`Event::EndTurn(color, TurnEnd)`, `Record::turn_end`, `is_quiet`). Усиленный компьютер (с
  0.39 до 0.108, убран — просьба пользователя; ниже — как было): `Position::boost(color)` / `Game::boost` (`boosted`) — слоны
  стороны насыщаются электричеством (`Piece::spark`: удар расходится тем же уроном ×
  `Rule::SparkPercent` на чужие фигуры по диагонали от цели — `Strike::splash`, не больше трёх,
  `Strike::hits`), пешки — лавой (`Piece::lava`, `charge_lava`: `Rule::LavaPawns` случайных по
  ключу позиции в начале каждого хода соперника, если ходит соперник — сразу; в конце своего
  хода после ударов `lava_blasts` — волна `Rule::LavaDamage` только по чужим на 8 клетках,
  кроме барьера; пешка цела, замёрзшая не взрывается, лава уходит у всех). Уровней,
  способностей и разброса нет с 0.11, брони и строя пешек — с 0.12 (убраны по просьбе
  пользователя; здоровье подняли до бывшего «эффективного», чтобы число ударов не
  изменилось). Пира с 0.31 нет (просьба пользователя): убийца не лечится, кроме «воина»
  (`Piece::champion`; `Strike::feast` — его лечение). С 0.24 (просьба пользователя) мата
  нет: король гибнет, как любая фигура (ходы не проверяются на шах, `legal_moves` =
  `pseudo_moves`; волна бьёт королей в полную силу), `kings: [Option<Square>; 2]`
  (`fall` снимает погибшего); без короля карты не играют (`can_play_card`); проигрывает
  сторона без фигур (`has_pieces`, `Status::Victory`) — сразу, даже посреди хода
  (`refresh`, `note_victory`, `Record::loser`, «#» в нотации); пешка не превращается в
  короля (`PROMOTIONS`, проверка в `make_move`). Шах (`in_check`) — только
  предупреждение и запрет рокировки. С 0.25 заморозка (просьба пользователя): `Piece::frozen`
  — сколько границ ходов своей стороны (начал и концов, `thaw` в `end_turn`) фигура ещё во
  льду; `freeze_at` = 2 (с 0.33, просьба пользователя): чужая пропускает свой ход и оттаивает
  в его конце, своя (заморожена в свой ход) — в начале своего следующего; короли не
  замерзают; замёрзшие не ходят
  (`pseudo_moves`), не бьют и не грозят (`any_attacker`), ладья не рокирует; с 0.31 (просьба
  пользователя) и не бьют в конце хода (`strike_all`). Карты на поле — `CardKind::area()` (`Area::{Ranks, Files, Square}`, `squares(aim)` —
  у края полоса сдвигается внутрь), цель — любая клетка (`picks` = 1, `card_targets` — все
  64): буря, лавина, ледяной барьер, по 10 маны (`freeze_area`, `CardPlay::frozen`). Барьер
  оставляет зону `Position::ice` (маска по сторонам, тает в `end_turn` в начале хода
  поставившей стороны): вставшая в неё фигура замерзает (`make_move` → `Outcome::frozen`,
  ладья рокировки, `swap_pieces`). Пата нет: кому ходить нечем, тот пропускает ход фигурой
  (`Game::can_end_turn` = нет ходов в `legal`). Правило «после карты должен остаться ход» (`sound_after_card`) убрано.
  С 0.26 (просьба пользователя, без графики): цена фигур в очках `Kind::points` (1/3/3/5/7/9,
  `score_kill(victim: Piece, …)`); `CardKind::family()` (`Family`) задаёт лицо и чем платят
  (`cost`). Новые карты: «Жертва» `Offering`, «Окружены и сломлены» `Encircle` (`encircles`,
  `encircled_by` → `Status::Encircled`), «Они множатся» `Multiply` (`spawn_square`,
  `multiply`, возврат в руку — `returning` → `Cards::give_back` в `end_turn`), «Один в Поле
  Воин» `Champion` (`Piece::champion`/`boost`: `attack()`, `max_hp()`; пир — всё здоровье),
  «Оборонительная позиция» `Bastion` (`Piece::bastion`, `damage_at` — только своим фигурам с 0.33, в ударе, волне,
  `can_kill`/`max_damage`), «Ледяная стена»
  `IceWall` (`Area::Rank`, зона в `ice_later` — на ход дольше барьера), «Массовое
  Жертвоприношение» `MassSacrifice` (`Area::Square`), «Кровь Богу Крови» `BloodGod`
  (`blood_god`, `give_sacrifice`). Цели одной фигуры — `piece_target`; `CardKind::freezes()`
  — карты со льдом (анимация `Freeze`). Лица — пустые лица своего типа с ценой
  (`PLAIN_CARDS` в `cards.rs`). Колода — 25 карт. С 0.28 «Абсолютный Ноль» `AbsoluteZero`
  (ОК, 30 маны): половина (вверх) свободных фигур соперника, кроме королей, замерзает навсегда
  (`Piece::frozen` = `FOREVER`, `end_turn` его не уменьшает; выбор — перемешивание по
  `key()`, `absolute_zero`); `has_pieces` считает только свободные фигуры — без них поражение.
  Колода — 26 карт. С 0.30 (просьба пользователя, без графики) ещё восемь: «Ледяной снаряд»
  `IceShard` (ОК 2: 500 урона чужой фигуре — `blast`, выжившая замерзает), «Барьер» `Barrier`
  (КП 2, `Area::Square`: `Position::barrier` — маска по сторонам, гаснет в `end_turn` в
  начале хода поставившей стороны; `wall(us)` — барьер соперника: ходы фигур не встают в него
  и не проходят сквозь, `any_attacker` не бьёт внутрь и сквозь, рокировка, `swap_is_safe` и
  `spawn_square` тоже; в `app.rs` — золотистая заливка), «Хладнокровный правитель»
  `ColdRuler` (КЖ 4: 2 маны за замёрзшую чужую — `frozen_enemies`), «Ледяной взрыв»
  `IceBlast` (ОК 4: замёрзшая чужая бьёт своей атакой всех в радиусе `Param::IceBlastRadius` (стандартно 2),
  `blast_squares`, `ice_blast`), «Истории Побед» `VictoryTales` (КП 3: 2 маны за убитую чужую
  — `Position::kills`, считает `score_kill`), «Холодная зима» `HardWinter` (КЖ 5: 500 урона
  замёрзшим чужим), «Воин из Льда» `IceWarrior` (КП 3: `Piece::frost_touch` — `strike_all`
  замораживает выжившую цель, `Strike::chilled`), «Северный воин»
  `NorthWarrior` (КП 3: `Piece::bonus_attack`/`bonus_hp` += 50/100 × замёрзших чужих, разово).
  Чужие цели (`piece_target`: снаряд — любая, взрыв — замёрзшая; `card_targets` перебирает
  все 64 клетки) — красные стрелка и метка (`Aim::Enemy`, `Marker::Attack`). Колода — 34
  карты.
  Карты — в `Position`:
  колода и рука каждой стороны (`Cards`, массивы `CardKind` без кучи — позиции
  копируются в переборе), до `cards_per_turn` из баланса (стандартно 3) карт за ход (с 0.19, `cards_played`;
  `play_card(index, targets)`, `picks()` —
  сколько своих фигур нужно). Типы карт — обычная, огненная волна (обычная с действием),
  жертвы, победы, обмен и исцеление (победы с действием; с 0.22 простых карт победы в
  колоде нет), дополнительное финансирование (жертвы с
  действием: `Param::FundingDraw` (стандартно 2) карты из колоды, `CardPlay::drawn` — сколько взято)
  (`CardKind`, у карт с действием
  `title`/`description` для лица карты); колода (стандартная; теперь см. `decks.rs`) (12 обычных, из них 4 волны, буря,
  лавина и ледяной барьер;
  4 победы — 2 обмена и 2 исцеления; 4 жертвы, из них 2 финансирования — просьбы пользователя:
  обмен только среди карт победы, финансирование — только среди карт жертвы)
  перемешивается в `Game::new`. Обмен с 0.18 (`swap_is_safe`, `swap_pieces`): две свои
  фигуры меняются местами с HP, король и ладьи теряют рокировку (`rights_lost_at`), пешку
  нельзя поставить на последнюю горизонталь; первая фигура — `card_targets`, вторая —
  `swap_partners`.
  Ход с 0.16: `make_move` ставит `moved` и сторону не меняет; `end_turn` передаёт ход
  и берёт карту (`draw_card`); `make_turn` = оба (перебор компьютера). `Game::play` сам
  кончает ход, если `!any_card_playable()`, `Game::play_card` — если уже сходили и
  сыграть больше нечего;
  `Game::end_turn` — кнопка (`can_end_turn`: сходили фигурой). История — `Record` с
  `Event::{Move, Card, EndTurn}`; ничьи (`refresh`) — только в начале хода, поражение —
  сразу; шах дописывается ходу фигурой в `finish_turn`. Огненная волна (`wave_hits`,
  `burn_wave`): `Param::WaveDamage` (стандартно 300) всем 8 соседям, королям тоже; цели —
  `card_targets` (свои фигуры). Исцеление с 0.22 (`heal_targets`, `heal_is_safe`,
  `heal_around`, `Param::HealAmount`, стандартно 600): цель — свой слон, лечатся свои фигуры на 8 соседних
  клетках не выше полного здоровья (слон и чужие — нет); `CardPlay::heals`. Очки с 0.21 (`Points::{Victory, Sacrifice}`,
  `Position::points`; исцеление — 2 очка победы; с 0.23 ещё `Points::Mana`: `mana_per_turn` из баланса (стандартно 2)
  в начале каждого своего хода (`gain_mana` в `deal`/`end_turn`, копится; `deal` обнуляет
  очки — колоды сдают дважды), огненная волна стоит 2 маны): `score_kill` в `strike_all` и картах (`burn_wave`, `blast`) — хозяину
  погибшей фигуры очко жертвы, убийце-сопернику очко победы; `CardKind::cost` (обмен — 1
  победы, финансирование — 1 жертвы), `can_afford` в `card_playable`/`card_targets`/
  `play_card`, оплата при розыгрыше. Очки — в `Position`, поэтому `undo` их возвращает.
- `src/balance.rs` (с 0.36; с 0.133 без файла) — все числа игры: `Balance` (правила `Rule`, числа карт
  `Param`, таблицы `RULES`/`PARAMS` с ключами и пояснениями, `DEFAULT_PIECES`, `default_cost`, колоды),
  `balance()` — баланс для правил (`&'static`, стандартный). Список для пользователя — `Баланс.txt`
  (`Balance::text`, `refresh_balance_list`). Новые числа — в эти таблицы, не константами (память
  «Баланс и колоды»). Файл `balance.ini`, его разбор, редактор и `tools/deck_builder.html` убраны в 0.133.
- Код колоды (с 0.37): `game/decks.rs` — `encode_deck_code` (`BC1-` + base64url; Ctrl+C на экране выбора
  колоды копирует код выбранной); `decode_deck_code` — с 0.133 только для тестов (вставки нет); порядок
  `CardKind::ALL` закреплён тестом.
- `src/game/decks.rs` (с 0.35, состав — из `tools/deck_builder.html`, присланный пользователем 2026-10-15) — пять колод по номерам меню: `RECIPES` (состав; `deck_cards` читает `balance().deck`; `DECK_NAMES` — названия, их же берёт `menu::DECKS`; список «карта — число копий»; Лазурная — 30 карт, ледяная, без простых; остальные четыре пока классические по 34), `deck_cards(i)`, `DECK_COUNT` = 5 (= `DECK_SKINS`), `MAX_DECK` = 64 (длина массива в `Cards`), `UNSHUFFLED` — колода позиций из FEN для тестов. `Position::with_shuffled_cards(seed, [белые, чёрные])`, `Game::new([белые, чёрные])`: игра даёт обеим сторонам выбранную колоду. Чтобы изменить колоды — править `RECIPES`.
- `src/game/tests.rs` — тесты правил, случайные партии (с картами, волнами и обменами). С 0.103.1
  (просьба пользователя: «прогони геймплейные тесты») — `rogue_fuzz`: случайные rogue-like партии со
  случайными гербами (каждая четвёртая — со всеми), объединёнными фигурами в запасе, магазином, продажей,
  гербами, расстановкой, «Выиграть раунд» и отменой; после каждого действия `assert_rogue_sane`
  (фигуры живы и не выше полного здоровья, короли на месте, уровни и волны в пределах, золото не
  перевалило через ноль, каждый ход переживает `encode_move`/`decode_move`, запас собирается обратно
  в те же `Stock`, описания строятся), пробная отмена возвращает позицию в точности.
  `rogue_like_fuzz_keeps_the_game_sane` — 40 партий (`ROGUE_FUZZ_GAMES`), `rogue_like_fuzz_long` — 600
  (`--ignored`). Проверка с переполнениями и `debug_assert` — в отдельной папке, чтобы не сбивать кэш:
  `CARGO_TARGET_DIR=target/checked CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true
  CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true cargo test --release -- --include-ignored --skip
  two_games_stay_in_sync` (~2 ГБ, после — удалить). Проверка в окне —
  временный демо-код автоигры (`BC_AUTOPLAY` = секунды, `BC_SEED`, `BC_AUTOPLAY_LOG`): случайные
  действия человека через те же `play`/`end_turn`/`undo`/магазин, когда доиграли анимации, обычные
  партии против усиленного компьютера через раз, паника — в лог (`panic::set_hook`), «застряли» — нет
  записей 30 с; для символов в трассе — `CARGO_PROFILE_RELEASE_STRIP=false` и
  `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` (в `[profile.release]` стоит `strip = true`). В 0.103.1
  34 сессии по 5–7 минут: падений и зависаний нет; дважды в сборке с `debug_assert` сработала проверка
  egui `gamma_multiply` (множитель < 0 или NaN) — не повторилась ни в одном из следующих 26 прогонов,
  наши вызовы с проверкой множителя чисты; в обычной сборке проверки нет.
- `src/ai.rs` — с 0.108 удалён (обычной партии против компьютера нет); был — компьютер (альфа-бета: ход в переборе — `make_turn`, ход и удары в конце хода;
  досчёта нет — ударов ходом нет; сила одна — `DEPTH` = 2, `THINK_TIME` 0,8 с, `NOISE` = 40,
  бывший уровень 2 «любитель»: с 0.31 уровней нет, просьба пользователя; `decided` —
  выигрыш/проигрыш, когда у стороны нет фигур; король стоит `KING_WORTH` = 850; нет ходов —
  перебор пропускает ход). Карт компьютер с 0.32 не играет (просьба пользователя: «пока что»;
  `choose_card` и его тесты убраны — при возврате смотреть историю 0.31).
- `src/app.rs` — окно на egui: изометрическая доска (`BoardGeom`: клетки ↔ точки
  экрана, своя сторона внизу слева), анимации, панели, ограничитель кадров без
  VSync (расписание по абсолютному времени), счётчик FPS. Фигуры собираются в
  список `Figure` и рисуются от дальних к ближним: тени, потом тела. Курсор сначала
  попадает в хитбоксы фигур (от ближних), потом в ромбы клеток (`square_under`). Хитбокс с
  0.56 (просьба пользователя) — `Art::hits`: силуэт, сжатый на `HIT_SHRINK` = 2 пикселя арта, у
  короля — только ниже строки `KING_HIT_TOP` = 38 (высокий король закрывал клетку позади по
  диагонали); рисование по-прежнему по `covers`. С 0.57 (просьба пользователя) подсвеченная клетка-цель
  выбранной фигуры или фигуры запаса (`is_target`) важнее загораживающих её фигур: в
  `square_under` она проверяется первой (ходят только на пустые клетки). С 0.58 (просьба пользователя,
  варианты 1, 4 и 6 из предложенных): `Marker::Target` — рамка 3 пикселя и светлая заливка;
  рамка «дышит» (`target_tint`, `TARGET_PULSE`/`TARGET_DIM`, пока курсор на цели — кадры идут,
  `hovered_target`); на клетке-цели — призрак фигуры (`ghost`: выбранная или из запаса,
  `GHOST_ALPHA`, без тени, при перетаскивании — нет; на клетке с фигурой — нет). У фигуры под курсором — таблички атаки и здоровья
  (`paint_stats`, выезжают за `STATS_TIME`); числа и всплывающий урон — пиксельным
  шрифтом (`pixel_text`), таблички — `paint_plate`. Над колодой под курсором
  (`deck_hover`, `deck_under`) — такая же табличка с числом карт (`paint_deck_count`
  в `table.rs`, значок `Glyph::Card`). `undo` отменяет ход целиком (до начала хода, против
  компьютера — и его ход). `drive_computer` (с 0.108): компьютер — только волны rogue-like, сам не
  ходит; ждёт анимации, удары и карты соперника и через `COMPUTER_DELAY` кончает ход. `MoveAnim` — только скольжение. Удары в
  конце хода (с 0.32): `queue_strikes` берёт новые записи истории (`shown_history`) и ставит
  `StrikeAnim` в очередь — после скольжения и прошлых ударов, через `STRIKE_GAP` (за ударами —
  взрывы лавы: `LavaAnim { blast }` с кадра `LAVA_IGNITE` и `Wave::new` с отложенным стартом в
  кадре `LAVA_WAVE_AT`; потом новая лава — `LavaAnim` насыщения; молния слона — цифры
  `Effect::hit` на задетых в момент касания); кадры фигур — `dressed` (`lava_frame`: насыщение
  до `LAVA_SPREAD`, ожидание `LAVA_HOLD`, взрыв; `spark_frame`: вступление с `spark_since`, повтор
  `ELECTRIC_LOOP`, во время удара — вспышка `ELECTRIC_FLASH`), рисует `paint_figures` (лёд —
  важнее); усиленного компьютера (`Settings::boosted`, `show_boost`) с 0.108 нет. Выпад
  `LUNGE_*` (`strike_lunge`), цифры урона — `Effect::hit` в момент касания (в будущем), над ещё
  видными цифрами той же клетки (`Effect::stacked`); до касания цель прежняя, убитая стоит
  (`struck_piece`), убийца идёт на её клетку (`StrikeAnim::moving`/`killer_at`, на клетке
  его не рисуют до конца удара), звук — `advance_strikes`; цель «Воина из Льда» мёрзнет в
  момент касания.
- `src/app/table.rs` — стол по `table_preview` из `breach_cards.py` («вариант 12»):
  своя колода (`DECK_AT`) в левом нижнем углу, рука (`Seat::Near`) под правым краем
  доски, кнопка «Конец хода» (`BUTTON_AT`) справа по центру; соперника — отражение
  через центр доски. Раскладка в пикселях арта от угла картинки доски; поле стола
  `TABLE_MIN`…`TABLE_MAX` (648×480, с 0.28.1 — по краям содержимого) вписывается в окно
  вместе с доской; масштаб — целое число физических пикселей на пиксель арта (в
  развёрнутом окне 1920×1080 ровно 2), дробный — только если целый мельчит доску больше чем
  на четверть; движущиеся фигуры и карты ставятся на физический пиксель (`BoardGeom::snap`).
  Карта под курсором — ×3 с 0.29 (`ZOOM`, на верхнем слое — правая панель её не режет),
  поднята на `LIFT`/`LIFT_IDLE` и выезжает за `RISE_TIME`
  (`card_hover_since`); курсор ловят только карты веера — по узким полосам
  (`fan_slot_at`: ближайшая ось из накрывающих курсор карт), крупная карта его не
  держит (с 0.17.2, просьба пользователя — перебирать карты плавно), название и описание пишутся шрифтом egui поверх лица
  (`paint_card_text`, повёрнутый `TextShape`, кегль подбирается по полю; описание — Segoe UI Semibold `theme::semibold`, у карты без картинки — на всё лицо). Карту берут
  мышью, как в Hearthstone: `Held` догоняет курсор (`FOLLOW`) и качается от скорости
  (`SWAY`), рисуется на верхнем слое; отпущенная над `hand_zone` возвращается
  (`Returning`), иначе разыгрывается и горит кадрами `burn_strip*`. Отпущенная волна
  всегда (даже над целью — просьба пользователя) переходит в `Aiming` (цели кэшируются): стрелка `paint_aim` по
  `draw_arrow` из `breach_arrow.py` — с 0.16.1 от макушки своего короля (просьба
  пользователя, как в Hearthstone от героя), над целью — голубая и метка `target_ally`;
  если цель — сам король, только метка. Обмену — две цели: `Aiming::first` в метке, стрелка
  от неё ко второй (из `swap_partners`), повторный щелчок снимает выбор. `Wave`: слои
  `under`/`over` вокруг фигур, цифры урона в `WAVE_IMPACT`; до него задетые фигуры
  рисуются прежними (`wave_victims`). `Portal` (обмен): кадры портала на обеих клетках,
  фигуры — `portal_figures` с `Sink` (на исчезновении — прежние на своих клетках), срез
  по кромке рисует `paint_sunk` в `app.rs` столбцами пикселей. `Bless` (исцеление):
  `bless_under`/`bless_over` вокруг клетки слона, слон — кадр `bless_figure` вместо спрайта
  (`Figure::bless`, точка опоры `BLESS_ANCHOR`), цифры лечения в `HEAL_IMPACT` (12-й кадр). Текст карт —
  `Color32::PLACEHOLDER` в разметке, цвет задаёт `TextShape`: на лицах сторон белый, на
  лице победы багровый/тёмный; кегль описания не больше `DESCRIPTION_FONT`. Цена карты
  впечатана в лицо (`paste_cost` в `cards.rs`: цифра шрифта на венке — белая, на черепе —
  красная, `COST_AT`/`POINTS_TINT`), у сгорающей карты гаснет вместе с текстом. Очки
  стороны — `paint_points` у её колоды: венок и череп из `tools/breach_emblems.py` в
  `EMBLEM_ZOOM` = 3 раза крупнее, число — шрифтом ×`POINTS_FONT` над знаком (с 0.21.1, просьба
  пользователя; `POINTS_RAISE`); соперника — отражение, число под знаком. Карты, взятые
  финансированием, летят из колоды по одной (`Draw`, промежуток `DRAW_GAP`). Карта на поле
  (`aim_area`): в `Aiming` цели — все клетки, вместо меток своих фигур подсвечены клетки
  `Area::squares` вокруг клетки под курсором (`Marker::Ice`), стрелка от короля к ней.
  Заморозка (`Freeze`, с 0.25): кадры `breach_freeze.py` на клетках карты с задержкой
  `FREEZE_DELAY` кадров за клетку от прицела; вставшая в зону фигура замерзает так же, когда
  доедет (`freeze_arrivals`) или поднимется из портала. Снег (`snow_at`, `paint_snow` — до
  подсветок, от дальних клеток) держится под замёрзшими фигурами и на зонах, остальной тает
  за `MELT_TIME`; лёд на фигуре (`ice_on` → `Figure::ice`, `paint_iced` в `app.rs`): иней
  поднимается срезом спрайта по строкам (`FREEZE_FROST`, кромка — `frost_edge`), глыба —
  сразу после тела фигуры. Оттаивание и растаявшие зоны ловит `tick_ice` (маски `ice_seen`)
  и растворяет лёд (`thaws`).
- `src/assets.rs` — пиксель-арт: PNG из `assets/breach` (декодирует tiny-skia из
  resvg), картинка доски с подписями собирается из клеток при запуске, подсветки
  клеток (`Marker`) рисуются там же. Текстуры увеличиваются в целое число раз без
  сглаживания и выводятся с LINEAR — пиксели резкие при любом масштабе. Цвет
  умножается на альфу в sRGB (`from_rgba_unmultiplied` осветляет полупрозрачное).
  Пиксельный шрифт (`GLYPHS`, `Glyph`): буквы и цифры 6×8 и значки (меч, сердце,
  череп, стрелка, карта с ромбом), белые с тёмным контуром — цвет задаётся оттенком
  при выводе. Все числа и подписи на столе — этим шрифтом на табличках (просьба
  пользователя: всё новое — в стиле игры).
  `src/assets/cards.rs` — карты (с 0.29 — из `cards/hd`, `CARD_DENSITY` = 2 пикселя карты на
  пиксель арта: размеры в раскладке те же, `Sprite::size` — в пикселях арта; лицо каждого
  вида — `kind_index`; у всех карт одна раскладка лица своего типа, без картинки окно пустое
  (просьба пользователя: не менять облик отдельных карт; текстовые лица 0.29 убраны); крупная
  карта — `zoomed` по требованию, `dense_scale`; цена — шрифт ×2). Ниже — как было в 1x:
  лица из PNG, стопки колод и тени собираются в коде,
  повороты веера — RotSprite (scale2x ×3), кэшируются по углу; лица волны собираются
  из `front_w/b` и `art/fire_wave.png`, лица обмена — из `front_victory(_selected)` и
  `art/swap.png` (обмен — карта победы), лица финансирования — из
  `front_sacrifice(_selected)` и `art/contract_blood.png` (вариант 3 «в крови» из
  `breach_contract.py`, выбор пользователя); буря, лавина и ледяной барьер — обычные лица
  сторон с ценой (`FREEZE_CARDS`, картинок пока нет), двузначная цена центрируется
  (`cost_corner`). `src/assets/ui.rs` — кнопка, звенья стрелки
  (повороты RotSprite с шагом 5°, кэш) и метки цели, кадры огненной волны и портала
  (`PORTAL_SINK`/`PORTAL_SCALE` — копии `SINK`/`SCALE` генератора, округление как в Python),
  кадры заморозки (`Strip`: обрезаны по непрозрачному, пустые и повторы не хранятся — глыб 24
  полосы). Фигуры в инее — `FROZEN` в `assets.rs`.
- `src/app/menu.rs` (с 0.27) — меню при запуске (`Screen::{Menu, Decks, Game}`): «Начать
  играть» (`new_game(draft)`), «Выбрать колоду», «Выйти»; плашки-кнопки в стиле кнопки
  «Конец хода», поле `FIELD` 360×240 пикселей арта в целом масштабе. Выбор колоды — 5 стопок
  `DECKS` (рубашка белых с повёрнутым оттенком `SKIN_HUES`/`turn_hue` в `cards.rs`,
  `tex.cards.skin(i)`), выбранная — `Settings::deck` — с 0.35 выбранная колода определяет карты обеих сторон в новой партии (`Game::new([deck; 2])`). Из
  партии в меню — кнопка «Меню» в `controls`. «Коллекция карт» (с 0.31, `Screen::Collection`,
  `collection_page`): виды карт колоды — `game::collection()` (по типу, потом по цене), по
  `PAGE` = 12 на странице (2 ряда по `COLUMNS` = 6, шаг `CARD_STEP`, белые лица), по бокам
  стрелки `arrow` (треугольник ступеньками в пиксель арта, на крайней странице — тусклая),
  ←/→ листают, Esc — назад; карта под курсором — ×`COLLECTION_ZOOM` = 2, у карт без текста
  название на `paint_title`; текст лиц — `paint_card_text` из `table.rs`; плашки — `plate`.
- `src/settings.rs` — громкость, частота кадров, FPS, выбранная колода, взятие, масштаб;
  `%APPDATA%\BattleChess\settings.ini`. VSync задаётся в `main.rs` при запуске. `data_dir()` (с
  0.39.1) — папка настроек и баланса по системе: Windows `%APPDATA%`, macOS `~/Library/Application
  Support`, Linux `~/.config`. Для macOS (просьба пользователя: отправить игру другу) браузер и
  редактор открываются через `open`, шрифты — Arial и Apple Symbols (`theme.rs`). Пакет для Mac —
  исходники без `target`/`CLAUDE.md`/вложенной старой копии `battle-chess/`, плюс `Запустить.command`
  (ставит Command Line Tools и Rust, `cargo build --release`, запускает) и «КАК ЗАПУСТИТЬ.txt»;
  ZIP — через Python с Unix-правами (у `.command` — 755, иначе macOS не запустит). Собрать под
  macOS отсюда нельзя (нужен SDK Apple), проверить сборку на Mac тоже не было возможности.
- `tools/deck_builder.html` — страница-конструктор колод и баланса (с 2026-10-15) удалена в 0.133 вместе с
  файлом `balance.ini`.
- `tools/breach_art.py` — генератор своего пиксель-арта в стиле Into the Breach
  (изометрические клетки 56×42, фигуры-мехи и фигуры-веки) в `assets/breach/`;
  нужны `pillow` и `numpy`. Размеры холстов (фигура 64×84, точка опоры 32,66,
  блок клетки 56×74) повторены в `assets.rs` — менять вместе.
- `tools/breach_cards.py` — карты в том же стиле (пустое лицо 50×70, рубашка, колода,
  веер; повёрнутые карты — RotSprite) в `assets/breach/cards/`; палитры и доску берёт
  из `breach_art.py`. Встроены в игру (`src/assets/cards.rs`, `src/app/table.rs`).
  Три типа лиц (названия пользователя): **обычные карты** — `front_w/b(_selected).png`,
  кристалл стороны в углу; **карты жертвы** — `front_sacrifice(_selected).png`, чёрные
  с красным, череп; **карты победы** — `front_victory(_selected).png`, белые с желтизной
  и багровым, багровый лавровый венок (`emblem_wreath`, ветвь нарисована по пикселям). Рубашки общие, по сторонам. Все три
  типа встроены в игру. Раскладка `table_preview` с 2026-10-10 (выбор пользователя,
  «вариант 12»): своя колода — в левом нижнем углу экрана (от левого угла доски −76, +152),
  колода соперника — зеркально в правом верхнем, кнопка «Конец хода» — справа по центру
  экрана; встроена в `src/app/table.rs` с 0.16.
- `tools/breach_arrow.py` — стрелка прицеливания картой, как в Slay the Spire, в
  `assets/breach/ui/`: звенья `arrow_seg_<s|m|l>_<состояние>` и `arrow_head_<состояние>`
  (остриём вправо, без тени), метка цели `target_<состояние>` (4 шеврона вокруг ромба
  клетки, поля `marker_pad()` = 15, картинка 86×72). Состояния: `idle` (белая, цели нет), `enemy`
  (красная), `ally` (голубая). Сборка (`draw_arrow`): квадратичная Безье от верха карты
  к цели, контрольная точка — над картой на 0.28 длины выше верхней из точек; звенья
  через 8.5/10.5/12.5 px вдоль кривой (s до 30% длины, m до 62%, дальше l), наконечник у
  цели; каждое звено повёрнуто по касательной RotSprite (угол с шагом 5°, кэш), тень
  30% (2, 3). Встроена с 0.16 (`paint_aim` в `table.rs`); в игре только `idle` и `ally`.
- `tools/breach_ui.py` — кнопка «Конец хода» (53×35, тень уже в картинке) в
  `assets/breach/ui/`: `end_turn_normal`, `_hover`, `_pressed`, `_disabled`, `_ready`
  (янтарная — ходить нечем). Надпись — шрифтом игры 6×8, кириллица дорисована в
  `FONT`. Встроена с 0.16: `Ready` — сходили фигурой, `Normal` — ещё нет, `Disabled` —
  ход соперника.
- `tools/breach_burn.py` — анимация сжигания карты (32 кадра 74×110, 70 мс) в
  `assets/breach/cards/burn/`: `burn_strip.png`, `burn_strip_b.png`, `burn_strip_sacrifice.png`,
  `burn_strip_victory.png`, `burn_strip_fire_wave_w/b.png`, `burn_strip_swap_w/b.png`,
  `burn_strip_swap_victory.png` (`save_swap_victory` — обмен на лице победы, им горит игра),
  `burn_strip_funding_sacrifice.png` (`save_funding_sacrifice` — договор в крови на лице
  жертвы; оба через `save_on_face`) —
  кадры в ряд, плюс GIF-превью. numpy и Pillow с 2026-10-13 стоят в Python пользователя
  (`py -m pip install --user numpy pillow`, с его разрешения) — генераторы запускаются
  просто `py tools/<генератор>.py`.
- `tools/breach_cards_hd.py` (наш, с 0.29) — карты вдвое детальнее для игры в
  `assets/breach/cards/hd/`: лица 100×140 (`front_<w|b|sacrifice|victory>[_selected]`: шапка,
  окно под картинку, поле описания — одна раскладка у всех карт), рубашки, картинки окна 76×50
  (`art/`: огненная волна и обмен — со спрайтом пешки игры, исцеление — крылья
  `spread_wing` ×2, договор — scale2x картинки 1x), сжигание `burn/burn_strip_*` (карта 2x
  горит по карте прогорания 1x, огонь и дым — пиксели 1x вдвое). Рамки — линии в пиксель
  карты (`frame_hd`), знаки типа — scale2x знаков 1x. Генераторы 1x (`breach_cards.py`,
  `breach_burn.py`, картинки) остаются источником палитр и форм.
- `tools/breach_card_arts.py` (наш, 2026-10-16, просьба пользователя: картинки всем картам без
  них) — картинки окна HD 76×50 для 20 карт с действием в `assets/breach/cards/hd/art/<имя>.png`
  и сжигание `hd/burn/burn_strip_<имя>_<лицо>.png` (как у `breach_cards_hd.py`: обычным — `w` и
  `b`, остальным — `sacrifice`/`victory`), превью `hd/preview_card_arts.png`. Имена: обычные —
  `storm`, `avalanche`, `ice_barrier`, `multiply`, `ice_wall`, `mass_sacrifice`, `absolute_zero`,
  `ice_shard`, `ice_blast`; жертвы — `offering`, `encircle`, `blood_god`, `cold_ruler`,
  `hard_winter`; победы — `champion`, `bastion`, `barrier`, `victory_tales`, `ice_warrior`,
  `north_warrior`. Фигуры — спрайты игры или тот же `render_piece` мельче (`piece(code, scale)`,
  игра — 0.86), глыбы/иней/снег — палитра `breach_freeze.py` (`ice_block`, `block_h`, `shard`,
  `snowflake`), фигуры в глыбах — без инея (тёмные, видны сквозь лёд). Пустые карты без
  действия (обычная, жертвы, победы) остаются без картинки. В игру ещё не встроены: `ART`,
  `card_art`, `BURNS` и `CardArt::burn` в `src/assets/cards.rs`; 29 новых полос сжигания по
  ~4 МБ в распакованном виде — возможно, их стоит распаковывать по требованию.
  `py tools/breach_card_arts.py [папка] [--no-burn]`; `BC_PREVIEW=<png>` — только превью,
  `BC_PIECE_CACHE=<папка>` — кэш рендеров фигур.
- `tools/breach_lava.py` (арт-проект, 2026-10-16) — трещины лавы на фигуре перед огненной волной:
  `effects/lava_crack_<цвет><фигура>.png`, 28 кадров 64×84 (холст и точка опоры спрайта, 60 мс):
  растут до 13, с 17 разгораются, волна — с 21, к 27 гаснут. `tools/breach_arcs.py` (там же) —
  электричество слона «Супер Саян»: `effects/electric_<w|b>B.png` (30 кадров 120×120, опора 60,92 —
  как у `bless_figure`; вступление 0–7, вспышка 5, повтор 8–23, концовка) и
  `electric_hold_<w|b>B.png` (без концовки). Встроены с 0.39 (`LAVA_*`, `ELECTRIC_*` в
  `assets/ui.rs`, полосы `Strip`): пешки — `lava_crack_?P`, слоны — `electric_hold_?B`.
- `tools/breach_coin.py` (арт-проект, 2026-10-18, просьба пользователя) — иконка золотой монеты
  под углом: повёрнута вокруг вертикали (лицо — эллипс `SQUASH` = 0.7, справа ребро толщиной
  `THICK` с насечкой через строку), бортик со светом сверху-слева, чеканный король (с 2026-10-18 вместо пешки, по пикселям — `KING` для 24 и 32; на 16 px —
  выпуклая точка), блик-звёздочка; шесть тонов `GOLD`, контур #020201. `assets/breach/ui/coin_16.png`,
  `coin_24.png`, `coin_32.png` (тени нет), превью `preview_coin.png`. В игру не встроено.
- `tools/breach_cursor.py` — курсор в том же стиле (стрелка-дротик 22×24, точка нажатия
  `HOTSPOT` = (1, 1), тень уже в картинке) в `assets/breach/cursor/`: `cursor_default`,
  `_hover` (кромка цвета выделения), `_move`, `_attack` (с 0.32 не встроен — ходом не бьют), `_card`, `_off`. Встроен:
  `Pointer` в `assets.rs`; `pointer_kind` в `app.rs` выбирает вид по тому, что под
  курсором, `paint_cursor` прячет системный курсор и рисует стрелку на слое `Debug` в
  ближайшем к доске целом масштабе (NEAREST). Особые курсоры egui остаются системными.
- `tools/breach_firewave.py` — огненная волна на клетку вокруг фигуры (20 кадров 184×168,
  60 мс, центр верхней грани клетки фигуры — `CX, CY` = 92, 98) в `assets/breach/effects/`:
  `fire_wave_under.png` (земля и огонь на трёх клетках позади — до фигур) и
  `fire_wave_over.png` (остальное — после фигур), кадры в ряд. Арт карты, вызывающей
  волну, — `assets/breach/cards/art/fire_wave.png` (окно лица карты 38×25 в точке 6, 17).
  Встроены с 0.16.
- `tools/breach_lava.py` (наш, 2026-10-16, просьба пользователя) — лавовые трещины на фигуре
  перед огненной волной: из точки в теле по спрайту расходятся трещины (остриё бело-жёлтое,
  лава оранжевая, тёмная корка по бокам), потом разгораются (лава светлеет и расширяется,
  язычки пламени, искры, фигура раскаляется) и гаснут — фигура снова прежняя. Кадры — замена
  спрайта: `assets/breach/effects/lava_crack_<цвет><фигура>.png`, 28 кадров 64×84 в ряд, точка
  опоры и тень — как у обычного спрайта, 60 мс. Огненная волна стартует (её кадр 0) в кадре
  трещин `WAVE_AT` = 21, пока трещины гаснут (они гаснут к кадру 27). Превью —
  `preview_lava_crack.gif` (белая и чёрная пешки на доске, затем волна) и
  `preview_lava_crack_frames.png`. С 2026-10-18 (просьба пользователя: «как у взрывающейся пешки — для всех
  остальных фигур») у фигур выше пешки (`tall` в `crack_times`) второй очаг трещин в верхней трети
  тела (стволы стартуют на 2 кадра позже) и язычки пламени ещё на середине и внизу; пешки не
  изменились (байт в байт). Превью всех — `preview_lava_crack_all.gif`. В игре — только пешки
  (`LAVA_PNG` в `assets/ui.rs`), полосы остальных фигур готовы и с той же разметкой кадров.
- `tools/breach_arcs.py` (наш, 2026-10-16, просьба пользователя: «электрические дуги в духе
  Супер Саяна 2» вокруг слона) — слон заряжается (искорки, золотая аура растёт), вспышка в кадре
  `FLASH` = 5 (8 молний расходятся во все стороны; кольцо по земле убрано 2026-10-18 — просьба пользователя), потом
  держится: аура-пламя языками вверх (шум бежит вверх, период = длине повтора), голубые дуги
  вдоль тела, изредка молния наружу или в землю с искрами; в конце аура опадает. Кадры —
  замена спрайта слона на холсте 120×120, точка опоры (центр клетки) — (60, 92), как у
  `bless_figure`; тень — обычная. `assets/breach/effects/electric_<w|b>B.png`, 30 кадров в ряд,
  60 мс: вступление [0, 8), повтор [8, 24) — крутить сколько нужно, концовка [24, 30). Версия,
  которая не гаснет (просьба пользователя), — `electric_hold_<w|b>B.png`: те же кадры [0, 24) без
  концовки, после 23-го снова 8-й. Превью — `preview_electric.gif`, `preview_electric_hold.gif`
  (повтор 10 раз), `preview_electric_frames.png`. Молнии — `CYAN` (белое ядро, голубая
  кайма), аура — `GOLD`. С 2026-10-18 (просьба пользователя) — для всех 12 фигур:
  `electric_<цвет><фигура>.png` и `electric_hold_<цвет><фигура>.png`, зерно по виду `SEEDS` (у слона прежнее 7 —
  его полосы не изменились), превью остальных — `preview_electric_all.gif`. В игре — только слоны
  (`electric_hold_?B`), полосы остальных готовы с той же разметкой кадров. Спокойный вариант (2026-10-18, просьба
  пользователя: «слишком бросается в глаза», выбраны пункты «ритм вместо треска» и «ауру ужать») —
  `electric_calm_<цвет><фигура>.png`, 62 кадра: вступление [0, 8) как было, опадание [8, 14) — кадры концовки с
  каймой под ними, повтор [14, 62) (`CALM_LOOP` = 48, 2,9 с): аура — кайма 1–2 пикселя `GOLD`, дышит по синусу;
  искорки в кадрах повтора `CALM_SPARKS` = 8, 19; разряд `CALM_DISCHARGE` = 30 (три дуги вдоль тела и молния в
  землю, кайма ярче). Белые ядра и молнии при разряде — как в обычном. Сравнение — `preview_electric_calm.gif`.
  В игре фазу повтора стоит сдвигать у разных фигур, чтобы разряды не шли хором. Не встроено.
- `tools/breach_wind.py` (арт-проект, 2026-10-18, просьба пользователя: «как воздушный элементаль /
  Windrun», фигура не парит) — постоянный вихрь вокруг низа коня: столб струй через `RING_STEP` по
  высоте до `WRAP` = половины фигуры, радиус каждой — по ширине силуэта на её высоте + `GAP` (облегает
  фигуру), фаза закручена по высоте (`TWIST`, спираль), по две струи напротив; крутятся (`SPIN`) и
  непрерывно ползут вверх (`CLIMB` шагов за повтор): сверху тают, снизу появляются. Струя — белая голова,
  голубоватый тающий хвост, тёмная подводка; внутри столба — белёсая дымка (к краям гуще). У самой земли
  вихрь расширяется раструбом (`FLARE`/`FLARE_H`, функция `rx_at`), и по клетке от основания расходятся
  кольца ветра (`GROUND` = 3, проходят путь `GROUND_PASSES` раз за повтор до `GROUND_MAX` и тают к краю). Плотность по высоте (просьба пользователя): к верху столба редеет (`THIN` = 0.7, дымка
  тоже), ниже `DENSE_H` — четыре струи по кругу с толстыми головами (`arc(..., thick=True)`). Кадры — замена спрайта на холсте 120×120, опора (60, 92), как у `electric_*`, 60 мс:
  `assets/breach/effects/wind_<цвет><фигура>.png` (с 2026-10-18 — все 12 фигур, просьба пользователя; вихрь
  считается по силуэту каждой, кони не изменились), 40 кадров — вступление [0, 8), повтор [8, 40). Превью —
  `preview_wind.gif` (кони), `preview_wind_all.gif` (остальные), `preview_wind_frames.png`. Не встроено.
- `tools/breach_gilded.py` (арт-проект, 2026-10-19, просьба пользователя: «золотая или gilded фигура с
  роскошными украшениями») — те же 3D-фигуры `breach_art.py` с украшениями прямо на теле (новые материалы
  `FILI` — узор, `GEM` — камни, `PEARL` — жемчуг; палитры подставляются в `art.PALETTES` и рисует
  `art.render_piece`): нитка жемчуга по уступу основания, крупные камни на поясе и короне, пояс филиграни
  (два кольца + выпуклые бусины; у короля — одно кольцо), у ферзя шары короны — жемчуг и камень сверху, у
  короля камень в кресте, у ладьи золотой край зубцов и рамка бойницы с камнями на зубцах, у коня розетка с
  камнем на уздечке; самые светлые пиксели золота — белый блеск. Варианты: `gilded` (корпус в цвете
  стороны, отделка — золото; камни: белые — сапфиры, чёрные — рубины) и `gold` (белые — жёлтое золото +
  белая эмаль + сапфиры, чёрные — красное золото + чёрная эмаль + рубины). Спрайты как обычные (64×84,
  опора 32,66, отдельная тень): `assets/breach/pieces/<gilded|gold>/<цвет><фигура>.png` и `_shadow.png`;
  превью `pieces/preview_gilded.png` (обычные / позолоченные / золотые) и `preview_gilded_board.png`.
  Не встроено.
- `tools/breach_darkfire.py` (арт-проект, 2026-10-19, просьба пользователя: чёрно-фиолетовое пламя; первый вариант
  с отдельными языками и огнём по клетке пользователю не понравился, переделан «как аура электрической фигуры, чуть
  гуще») — аура по образцу `breach_arcs.py`: вокруг силуэта по бокам колышется (`SIDE` = 4.5 + 5.5·шум, у
  электрической 3.5 + 4.5), вверх — языки (`UP` = 7 + 25·шум², у электрической 5 + 22), узор бежит вверх (`RISE`,
  период = длине повтора). Цвета: нутро почти чёрное `FILL` (альфа 215), ближе к фигуре `DEEP`, вплотную —
  тёмно-фиолетовая кайма `LINER` (отделяет чёрный контур фигуры), под контуром `GLOW`, снаружи фиолетовый контур
  `CONTOUR`, кончики языков — сиреневые `GLINT`, искорки в ауре. Фигура чуть темнеет (`TINT`), по краю мерцает
  отсвет `RIM`. Кадры — замена спрайта на холсте 120×120, опора (60, 92), как у `electric_*`, 60 мс:
  `assets/breach/effects/dark_fire_<цвет><фигура>.png` (все 12), 40 кадров — вступление [0, 8), повтор [8, 40).
  Превью — `preview_dark_fire.gif`, `preview_dark_fire_frames.png`. Не встроено.
- `tools/breach_shadowstep.py` (арт-проект, 2026-10-19, просьба пользователя: фигуры с тёмной аурой прыгают с клетки
  на клетку, игнорируя фигуры, — «между warp-прыжком из Warhammer 40 000 и теневым шагом из аниме, с еле заметным
  силуэтом на пару мгновений») — две полосы на фигуру (все 12), холст 120×120, опора (60, 92), 60 мс.
  `shadowstep_out_<цвет><фигура>.png` (`OUT_LEN` = 18, на исходной клетке): кадры ауры `dark_fire_*` → фигура
  чернеет до силуэта с фиолетовым контуром (`darken`), по полу сходится кольцо (`floor_ring`), за ней раскрывается
  варп-разлом (`rift`: рваная линза, нутро `VOID` с завихрениями `SWIRL`, кромка `LIP`, контур и двойное свечение;
  ширина по кадрам `RIFT_OUT`; с 2026-10-19 по просьбе пользователя ниже — высота 0.5 × рост фигуры + 12 — и быстрее:
  раскрывается за 2 кадра, схлопывается за 1), фигуру втягивает — сжатие по ширине (`squash`, `SQUASH_OUT`; в рывке силуэт светлее
  нутра — `DASH_BODY`/`DASH_EDGE`), теневые полосы рывка (`speed_lines`), осколки в разлом, схлопывание со вспышкой;
  с кадра 5 — оставленный силуэт (`ghost`: тёмно-фиолетовый с контуром, тает построчно и мерцает до конца полосы).
  Фигуры (и её тени) нет с кадра `VANISH` = 7. `shadowstep_in_<цвет><фигура>.png` (`IN_LEN` = 13, на целевой
  клетке; начинается в кадре исчезновения `IN_AT` = 7): разлом (`RIFT_IN`), с кадра `EMERGE` = 2 фигура выходит
  (`SQUASH_IN`), кольцо по полу, с кадра 5 — кадры вступления ауры (`dark.render`), последний кадр = `dark_fire`
  кадр 7, дальше — повтор ауры с `darkfire.LOOP_START`. Следа между клетками нет. Превью — `preview_shadowstep.gif`
  (конь и ферзь прыгают через фигуры), `preview_shadowstep_frames.png`. Не встроено.
- `tools/breach_ice_pieces.py` (арт-проект, 2026-10-19, просьба пользователя: «иссиня-голубые фигуры с участками,
  обмёрзшими льдом») — те же 3D-фигуры `breach_art.py` с материалами `ICE` (прозрачный лёд) и `RIME` (иней):
  корпус и отделка синие (белые — иссиня-голубые, чёрные — глубокий тёмно-синий; огоньки — как у сторон), наледь в
  `iced`: крупные пятна льда по 3D-шуму `noise3` (ячейка 6.5) с инеем по краю, иней шапками на верхних поверхностях
  (по нормали вверх), лёд от основания; сосульки (`icicle`, цепочка шариков) из-под поясов и короны (`ICICLES`), у
  коня — из-под челюсти; три ледяных кристалла у основания (`shard`); белые искры на самом светлом льду. Спрайты как
  обычные: `assets/breach/pieces/ice/<цвет><фигура>.png` и `_shadow.png`; превью `pieces/preview_ice_pieces.png`
  (обычные / ледяные) и `preview_ice_pieces_board.png`. Не встроено.
- `tools/breach_fat_king.py` (арт-проект, 2026-10-19, просьба пользователя: «король-толстосум — на основе золотого
  короля, большое пузо и свисающие украшения») — палитры варианта `gold` из `breach_gilded.py` (белые — жёлтое золото,
  белая эмаль, сапфиры; чёрные — красное золото, чёрная эмаль, рубины); тело `BODY` = профиль `PROFILE` (круглое брюхо
  вместо ножки, снизу подворачивается к подставке) плавно слит (`smin`) с выпирающим вперёд животом-эллипсоидом
  (`BELLY_C`/`BELLY_R`); пояс эмали `BELT` под пузом с пряжкой-камнем; две нитки по пузу (`necklace`, бусины кладутся
  на поверхность через `surface_r`: крупный жемчуг; камни через жемчужину с медальоном-камнем в оправе); с боков
  короны — нитки жемчуга с каплями-камнями, с пояса — короткие подвески (`dangle`); крест и камни как у золотого
  короля. Спрайт как обычный: `assets/breach/pieces/fat_king/<w|b>K.png` и `_shadow.png`; превью
  `pieces/preview_fat_king.png` (золотой король и толстосум, ниже — на доске среди золотых фигур). Не встроено.
- `tools/breach_menu.py` (арт-проект, 2026-10-19, просьба пользователя: картинка главного меню — книга на деревянном
  столе при свече, кнопки — закладки справа; вторая версия по замечаниям: без названия, чернильницы, монет и фигур,
  свеча слева вверху, закладки тёмные, поменьше и извиваются, книга — узкий высокий классический том, как присланный
  образец) — сцена 480×270 (при 1920×1080 ровно ×4, как пиксели меню `FIELD`): стол из досок (`paint_table`), книга
  (`paint_book`): обложка `BOOK` = (120, 10, 300, 234) в бордовой коже (с 2026-10-19 шире — просьба пользователя), рамка-«плетёнка» — косая золотая сетка между
  двойными золотыми линиями, медальон с королём (`breach_coin.KING[24]` ×2) посередине обложки, веточки над и под ним;
  толщина `STRIP` = 30 со сдвигом `KX` = 0.3: снизу доски и страницы, слева круглый корешок с бинтами, золотыми
  линиями и тёмной плашкой. Свеча слева вверху (`paint_candle`, огонёк `flame_image`, свечение `glow`), свет —
  `light()` от `LIGHT_GROUND` = (46, 74), виньетка, тени вправо-вниз (`Scene.cast`). Файлы `assets/breach/menu/`:
  `menu_background.png` (с огоньком), `menu_background_noflame.png` + `menu_flame.png` (`FLAME_FRAMES` = 8 кадров
  18×32, 90 мс, верх фитиля — `FLAME_ANCHOR` (9, 29) в кадре, `FLAME_AT` (46, 33) на фоне). Закладки (`tab_image`):
  приглушённые тёмные ленты (`TAB_COLORS`: wine, plum, teal, ochre, forest, navy, umber, slate) с раздвоенным концом и
  прострочкой; от книги прямые, дальше извиваются (`tab_wave`: размах `TAB_WAVE` = 2, шаг `TAB_PERIOD` = 64, фаза —
  `tab_phase` по месту в `SLOTS`); буквы шрифта 5×7 (`FONT`) целиком сдвигаются по волне. Высота ленты `TAB_BODY_H` = 13,
  холст `TAB_W`×`TAB_H` = 148×20 (тень внутри); закладка k: холст в (`BOOK_RIGHT` − 1, `TAB_Y0` + k·`TAB_PITCH` −
  `TAB_WAVE`) = (299, 52 + 17k); состояния `normal` (длина `TAB_LEN` = 140), `hover` (+5, на тон светлее, надпись
  светлее), `pressed` (+2, темнее, надпись ниже). `tab_<имя>_<состояние>.png` для `TABS` (play «Начать игру», rogue,
  multiplayer, boost_off/on, advance_off «Взятие: на месте»/advance_on «Взятие: с перемещением», balance, settings
  «Настройки» — в меню пока нет, quit), пустые `tab_blank_<цвет>_<состояние>.png`; надпись не длиннее `TEXT_MAX` = 123.
  Превью — `preview_menu.png` (×2), `preview_menu.gif`. Открытие книги (2026-10-19, просьба пользователя: «книга
  легко приоткрывается, из неё идёт свет»; затем по просьбе закладки остаются на месте и свет экран не заливает) —
  `menu_open.png`: `OPEN_FRAMES` = 30 кадров 480×270 в ряд по `OPEN_MS` = 70 мс (огонёк свечи уже в кадрах, закладок
  нет — игра рисует их поверх на прежних местах): обложка поворачивается на корешке до `OPEN_MAX` = 9° к кадру ~16
  (`warped`: правый край выше и правее, снизу видна доска, подсвеченная светом), из щели под ней — лучи вниз-вправо по
  страницам и столу, они переливаются до конца полосы (`open_frame`); последний кадр можно держать. Обложка отдельно —
  `menu_open_cover.png` (те же кадры, RGBA: только обложка с доской и контуром): порядок рисования — кадр
  `menu_open`, закладки, поверх `menu_open_cover` (закладки выходят из-под приподнятой обложки, а не лежат на ней). Превью —
  `preview_menu_open.gif`, `preview_menu_open_frames.png`. Было встроено в 0.74, в 0.75 убрано (просьба пользователя).
- `tools/breach_crest.py` (арт-проект, 2026-10-20, по эскизу пользователя; маленький треугольный вариант 15×16
  заменён) — герб 49×45 с контуром (на 40% больше `coin_32`): симметричный щит (`shield`: верх вогнут на `TOP_DIP`,
  углы — рожки, бока почти прямые до `WAIST` = 0.55, дальше дугой к острию), тонкий обод из тёмного металла `IRON` с
  фаской, золотые уголки-треугольники на рожках с искрой, поле очень тёмное бордовое `WINE` с тенью под ободом,
  золотая молния-зигзаг (`BOLT`) с бликом и тенью. Файл `assets/breach/ui/crest.png` (тени нет), превью
  `preview_crest.png` (крупно и рядом с монетой в размере игры). Второй вариант (просьба пользователя) — тот же герб
  со снежинкой: `crest("snow")` → `crest_snow.png` (шесть лучей через 60° с двумя парами веточек — `snowflake`,
  лёд `ICE` бело-голубой), превью `preview_crest_snow.png`. Третий вариант (по эскизу пользователя) — молния и
  снежинка вместе: `crest("both")` → `crest_both.png` (золотая молния слева, ×0.72; снежинка справа, радиус 7.6, центр ровно в пикселе (29.5, 22.5) — симметрична; обе
  ближе к центру щита и ниже — просьба пользователя; по
  одной паре веточек), превью `preview_crest_both.png`. Четвёртый вариант (просьба пользователя) — снежинка и сердце: `crest("heart")` → `crest_heart.png` (снежинка как в `both`, слева красное сердце `heart` — кривая (x² + y² − 1)³ ≤ x²y³, s = 4.8, центр (16.5, 22.5), палитра `RED`), превью `preview_crest_heart.png`. Пятый — снежинка и монета: `crest("coin")` → `crest_coin.png` (слева вклеена `coin_16.png` со своим контуром, центр x = 15.5, тень на поле), превью `preview_crest_coin.png`. Шестой — две снежинки: `crest("snow2")` → `crest_snow2.png` (снежинки как в `both`, центры (15.5, 22.5) и (31.5, 22.5) — симметрично), превью `preview_crest_snow2.png`. Седьмой — пешка и снежинка: `crest("pawn")` → `crest_pawn.png` (золотая пешка `PAWN` по пикселям 9×14 со столбцом 16 посередине, снежинка — центр (30.5, 22.5)), превью `preview_crest_pawn.png`. Восьмой — шипы и снежинка: `crest("spikes")` → `crest_spikes.png` (стальные шипы `SPIKES` по пикселям 11×12, палитра `STEEL`), девятый — снежинка в ледяном кристалле: `crest("crystal")` → `crest_crystal.png` (шестигранник посередине щита, грани палитры `CRYSTAL` — верхние скаты светлее, левая сторона к свету, кромка с бликом; снежинка r = 7.6 почти белая, без каймы, с тенью на лёд), превью `preview_crest_spikes.png`, `preview_crest_crystal.png`. Встроены с 0.77 (гербы rogue-like): `heart`, `coin`, `snow2`, `pawn`, `spikes`, `crystal`. Топоры (2026-10-21, просьба
  пользователя): топор — `axe(w, h, hx, yc, size, side, haft_len)` — маски топорища (`WOOD`), золотых обмотки и навершия,
  стального лезвия (шейка у топорища, веер к выпуклой кромке ≈ 7.6·size); `axe_sword` (топор слева, лезвием наружу, ось
  x = 20; справа меч по пикселям: клинок `STEEL`, гарда и навершие `GOLD`, рукоять `LEATHER`), `axe_fangs` (справа два
  клыка `fang` цвета `BONE`, концы загнуты друг к другу), `axe_queen` (справа золотой ферзь `QUEEN` по пикселям 11×17),
  `axes` (два топора зеркально, оси 20 и 27), `whirl` (топор ×0.9 посередине в красном вихре `whirl`: три дуги-струи по
  кругу r = 12.5, голова толстая, хвост тонкий и доходит до следующей головы, палитра `WHIRL`; вихрь — слой `under`,
  под знаками, со своей каймой), `fire_axe` (топор посередине ниже обычного, топорище короче; за ним языки пламени
  `FLAMES` → `flames`: от круглого основания к острому изогнутому кончику, слои `FIRE` снаружи красные, к середине
  жёлтые, искры). Файлы `crest_<axe_sword|axe_fangs|axe_queen|axes|whirl|fire_axe>.png`, превью у каждого и все шесть
  рядом — `preview_crest_axes.png`. Огонёк (2026-10-22, просьба пользователя): `crest("flame")` → `crest_flame.png` — посередине пламя
  слоями `FLAME_LAYERS` (через `flames`: красный — главный язык и два боковых, оранжевый, жёлтый, бледное ядро), кайма и
  тень `WINE[4]`, кончики языков светлее, на поле два круга отсвета, четыре искры; превью `preview_crest_flame.png`. Ещё шесть (там же, просьба
  пользователя): общий помощник `draw_flame` (слои, кайма, тень, отсвет по желанию), малый огонёк `small_flame(cx, base,
  k = 0.62)` слева (x 16) — `flame_pawn` (справа пешка `PAWN`), `flame_skull` (череп `SKULL` по пикселям 11×11, `BONE`),
  `flame_sword` (меч, как у топора, ось x 31), `flame_coin` (`coin_16` с центром x 31.5); `flames` — два малых огонька
  (x 15.5 и 31.5); `bomb` — авиабомба (как ядерная, по просьбе пользователя вместо круглой «игровой» с фитилём) падает носом
  вниз посередине: корпус `hull` — овал с конусом к хвосту (`BOMB`, военный оливковый, блик), жёлтая полоса `HAZARD`,
  оперение `fins` — два стабилизатора, ребро и рамка поверху (`FINS`, светлая сталь — тёмная сливалась с полем), по бокам полосы падения. Маски
  бомбы нельзя звать `body` — это маска щита, иначе пропадает контур. Файлы `crest_<flame_pawn|flames|flame_skull|flame_sword|flame_coin|
  bomb>.png`, рядом — `preview_crest_fires.png`.
  Чёрно-фиолетовый огонёк (просьба пользователя: под цвет теневых фигур): `crest("dark_flame")` → `crest_dark_flame.png` —
  те же `FLAME_LAYERS`, `draw_flame` с цветами `DARK_FIRE` (снаружи внутрь: фиолетовая кромка, тёмно-фиолетовый, почти
  чёрный, чёрное нутро — как аура `breach_darkfire.py`), кончики `DARK_TIP`, отсвет `DARK_GLOW`, искры `DARK_SPARK`. С ним
  ещё шесть (просьба пользователя): малый тёмный огонёк слева (`small_flame(16, 27)` с `DARK_FIRE`) и справа —
  `dark_rook` (золотая ладья `ROOK` по пикселям 11×13), `dark_skull` (череп `SKULL`), `dark_portal` (овальный портал
  rx 6.2, ry 9.2 с центром x 31.5: кайма `DARK_FIRE`, нутро `VOID`, закрученные струи `cos(2·угол + 7.5·r)`, светлая
  кромка сверху-слева, искра в глубине), `dark_coin` (`coin_16`), `dark_eye` (демонический глаз: линза-миндаль по формуле
  13×8, светящийся край века, тёмное белко, радужка, зрачок-щель `VOID`, блик, три шипа над веком; пробовали глаз по
  пикселям — пользователю первый вариант понравился больше, возвращён); `dark_flames` — два тёмных
  огонька (x 15.5 и 31.5). Файлы `crest_dark_<rook|skull|flames|portal|coin|eye>.png`, рядом — `preview_crest_darks.png`.
  Воздушный смерч (2026-10-22, просьба пользователя): `crest("wind")` → `crest_wind.png` — посередине воронка из 13 витков
  (`tornado_strokes`: дуги эллипсов, сверху широкие, к низу уже и с изгибом, каждый виток повёрнут; голова струи белая,
  хвост голубой, задняя половина витка темнее — палитра `AIR`, как у вихря воздушных фигур), тёмная кайма, у основания
  облачко пыли из пяти клубов (`DUST`); превью `preview_crest_wind.png`. С ним ещё четыре (просьба
  пользователя): помощник `draw_tornado(mid, top, bottom, width, n, base_y, taper)` (`taper` < 1 — ровный конус; малый
  смерч с чашей сверху был похож на бокал), малый смерч слева (x 15.5, ширина 5.2, 12 витков, `taper` 0.8) и справа —
  `wind_coin` (`coin_16`), `wind_knight` (золотой конь `KNIGHT` по пикселям 12×15), `wind_sword` (меч, как у огонька);
  `winds` — два малых смерча (x 15 и 32). Файлы `crest_<winds|wind_coin|wind_knight|wind_sword>.png`, рядом —
  `preview_crest_winds.png`.
  Молнии (там же, просьба пользователя): `bolt_mask(k, ox, oy)` — молния `BOLT` в масштабе и сдвиге; слева малая (как в
  `both`: k 0.72, сдвиг −0.31, −0.07) и справа — `bolt_bishop` (золотой слон `BISHOP` по пикселям 9×15), `bolt_coin`
  (`coin_16`), `bolt_plus` (толстый плюс `PLUS` 11×11 голубой `CYAN`); `bolts` — две молнии (k 0.62, сдвиги −0.365 и
  0.315); `red_bolt` — большая молния первого герба палитрой `RED`. Файлы `crest_<bolt_bishop|bolt_coin|bolt_plus|bolts|
  red_bolt>.png`, рядом — `preview_crest_bolts.png`. `bolt_one` — малая молния и цифра 1 (`ONE` по пикселям 7×13,
  светлая сталь `STEEL`), `crest_bolt_one.png`. Пары фигур (просьба пользователя): две золотые одинаковые фигуры
  симметрично середине щита (`pairs`: сетка и левый/правый край) — `pawns2`, `bishops2`, `rooks2` (ладья `ROOK`),
  `queens2` (ферзь `QUEEN`), `knights2` (кони `KNIGHT` мордами друг к другу — левый отражён); `reroll_coin` — знак
  обновления `reroll_arrow` (кольцо r 5.4 с разрывом справа сверху и наконечником, голубой `CYAN`) слева и `coin_16`
  справа. Файлы `crest_<pawns2|bishops2|knights2|rooks2|queens2|reroll_coin>.png`, рядом — `preview_crest_pairs.png`.
  Магическая шляпа (2026-10-23, просьба пользователя): `crest("wizard_hat")` → `crest_wizard_hat.png` — посередине синяя
  (`HAT`, как у белых магов) шляпа `wizard_hat`: ровный конус, ось гнётся только у кончика (`4.8·t^4.5`; при раннем изгибе
  край выходил отвесным — «парус»), поля — эллипс, тень полей на тулье, золотая лента `GOLD`, две звёздочки-крестика и две
  точки на тулье, три искорки вокруг (`STARLIGHT`); превью `preview_crest_wizard_hat.png`. С ней ещё (просьба
  пользователя): `wizard_hat(x, y, cx, by, k)` — шляпа с серединой полей и масштабом, `draw_hat` (знаки, тень полей,
  звёздочка); малая шляпа слева (x 16, поля на y 29.5, k 0.62) и справа — `hat_coin` (`coin_16`), `hat_bishop` (слон
  `BISHOP`), `hat_plus` (плюс `PLUS`, `CYAN`), `hat_meteor` (малый метеорит); `hats` — две шляпы (x 15 и 32, k 0.6);
  `meteor` — метеорит посередине (`draw_meteor(cx, cy, r, length)`: бугристый камень `ROCK` как в игре, кратеры со светлой
  кромкой, спереди — вниз-влево — раскалённая кромка, вокруг ореол, хвост пламени вверх-вправо слоями `FIRE`, искры).
  Файлы `crest_<hat_coin|hats|hat_bishop|hat_meteor|hat_plus|meteor>.png`, рядом — `preview_crest_hats.png`.
  Золотые короли (2026-10-23, просьба пользователя): `gold_king` — большой король `KING_BIG` по пикселям 13×20 посередине
  (корона с тремя зубцами — круглая была похожа на пешку, — крест, рубины `RUBY` в обруче), малый `KING_SMALL` 9×16 слева
  (x 11) и справа — `king_coin` (`coin_16`), `king_queen` (малый ферзь `QUEEN_SMALL` 9×14 — ниже короля, основания на одной линии; обычный
  `QUEEN` был больше короля), `king_skull` (череп `SKULL`); `kings2` — два
  малых короля (x 11 и 27); `piggy` — розовая свинка-копилка мордой влево (`PIG`: пухлое тело, голова, ухо с тёмной серединкой,
  пятачок — светлый диск с обводкой и двумя ноздрями, хвостик-завиток, глаз, прорезь на спине; ножки `PIG_LEG` — только две,
  короткие, толстые, тоном темнее, с копытцами: четыре тонких под брюхом походили на вымя), сверху падает золотая монетка. Файлы `crest_<gold_king|
  king_coin|kings2|king_queen|king_skull|piggy>.png`, рядом — `preview_crest_kings.png`.
  Топор с другими силами (2026-10-23, просьба пользователя): топор слева, как в `axe_sword` (ось x 20), справа —
  `axe_flame` (малый огонёк x 31.5), `axe_snow` (снежинка x 30.5), `axe_bolt` (малая молния, сдвиг 0.29), `axe_hat`
  (малая шляпа x 31), `axe_dark` (малый чёрно-фиолетовый огонёк), `axe_wind` (малый смерч x 31.5), `axe_king` (малый
  король x 27). Файлы `crest_axe_<flame|snow|bolt|hat|dark|wind|king>.png`, рядом — `preview_crest_axe_mix.png`.
  Встроены с 0.96 (открывают варваров-объединённых).
  Золотой король с другими силами (там же): малый король `KING_SMALL` слева (x 11), справа те же малые знаки —
  `king_flame`, `king_bolt`, `king_snow`, `king_wind`, `king_dark`, `king_hat`; король с варваром — уже есть `axe_king`.
  Файлы `crest_king_<flame|bolt|snow|wind|dark|hat>.png`, рядом — `preview_crest_king_mix.png`. Встроены с 0.97
  (открывают золотые объединённые фигуры).
  Шляпа мага с другими силами (там же): малая шляпа слева (`draw_hat(16, 29.5, 0.62)`), справа — `hat_flame`, `hat_snow`,
  `hat_bolt`, `hat_wind`, `hat_dark`; шляпа с топором и с королём — уже есть `axe_hat`, `king_hat`. Файлы
  `crest_hat_<flame|snow|bolt|wind|dark>.png`, рядом — `preview_crest_hat_mix.png`. Встроены с 0.98 (открывают
  магов-объединённых).
  Огонёк с другими силами (там же): малый огонёк слева (`small_flame(16, 27)`), справа — `flame_snow`, `flame_bolt`,
  `flame_wind`, `flame_dark`; огонёк с топором, королём, шляпой — уже есть `axe_flame`, `king_flame`, `hat_flame`. Файлы
  `crest_flame_<snow|bolt|wind|dark>.png`, рядом — `preview_crest_flame_mix.png`. `flame_<bolt|wind|dark>` встроены
  с 0.101 (огненные объединённые).
  Снежинка с другими силами (там же): снежинка слева (центр (16.5, 22.5)), справа — `snow_bolt` (зеркально к `both`, где
  молния слева), `snow_wind`, `snow_dark`; снежинка с огоньком, топором, королём, шляпой — уже есть. Файлы
  `crest_snow_<bolt|wind|dark>.png`, рядом — `preview_crest_snow_mix.png`. Встроены с 0.99 (вместе с `flame_snow`
  открывают ледяных-объединённых).
  Молния с другими силами (2026-10-23, просьба пользователя): малая молния слева (`bolt_mask(0.72, -0.31, -0.07)`, как
  в `both`), справа — `bolt_wind` (малый смерч), `bolt_dark` (малый чёрно-фиолетовый огонёк); молния с огоньком,
  снежинкой, королём, топором, шляпой — уже есть. Файлы `crest_bolt_<wind|dark>.png`, рядом —
  `preview_crest_bolt_mix.png`. Встроены с 0.102 (электрические объединённые). Ветер и тень (там же): малый смерч
  слева (как у `wind_coin`), справа — малый чёрно-фиолетовый огонёк — `crest_wind_dark.png`, превью
  `preview_crest_wind_dark.png`; теперь гербы есть у всех пар сил. Встроен с 0.103.
- `tools/breach_poison.py` (арт-проект, 2026-10-21, просьба пользователя: ядовитые фигуры — частично облиты зелёной
  жидкостью, капают в лужу под собой, при шаге вперёд оставляют пропадающий след) — спрайты: 3D-фигуры `breach_art.py` с
  материалом `SLIME` (палитра `SLIME_RAMP`, у сторон корпус и огни прежние): шапка жидкости на макушке (`COAT` = 0.2 высоты,
  кромка по шуму), потёки вниз (`drips`: полуэллипс по азимуту, 6 к зрителю разной длины), на концах длинных — капли-шарики,
  на уступах — редкие лужицы; самые светлые пиксели — блики `GLINT`. `assets/breach/pieces/poison/<код>.png` и `_shadow.png`
  (как обычные: 64×84, опора 32,66). Анимация — замена спрайта на холсте 120×120, опора (60, 92), как у `electric_*`,
  60 мс: `assets/breach/effects/poison_<код>.png` (все 12), 40 кадров — вступление [0, 8) (лужа растекается), повтор
  [8, 40): лужа под фигурой (`blob`: неровный эллипс 4:3, `PUDDLE_RX` = 17, в ромбе клетки, с брызгами рядом, кромка
  колышется, блики мерцают), с концов потёков на теле (`drop_sources`, до трёх) набухают и падают капли шириной 2 пикселя
  (с тёмной обводкой), всплеск и круги в луже, по потёку сбегает блик, раз за повтор лопается пузырь. След шага —
  `poison_trail_<ne|sw|nw|se>.png` (ne — к восьмой горизонтали без переворота доски, сдвиг клетки на экране (±28, ±21)):
  30 кадров 96×80, середина между центрами клеток — (48, 40); рисовать до фигур. Форма — капля (по рисунку пользователя,
  2026-10-21): голова `TRAIL_HEAD` = 14 там, куда фигура встала (под её лужей), хвост `TRAIL_HEAD` · t^`TAIL_POW` (1.4)
  сужается к исходной клетке, остриё — за её центром на `TAIL_OVER` = 0.25 шага. Кадры [0, 4) — скольжение (SLIDE_TIME
  0,2 с), капля вытягивается за фигурой; [4, 30) — хвост втягивается от острия к голове, след темнеет, распадается на
  пятна, поднимается зелёный пар. Лужа самой фигуры едет вместе с ней (она в кадрах анимации). Превью —
  `pieces/preview_poison_pieces.png`, `effects/preview_poison.gif` (все фигуры), `effects/preview_poison_step.gif` (волна
  ядовитых шагает вперёд), `effects/preview_poison_trail.png`, `effects/preview_poison_frames.png`. Не встроено.
- `tools/breach_poison.py` (арт-проект) — ядовитые фигуры (облиты зелёной жидкостью, лужа, капли,
  след шага); встроено с 0.79 — см. выше.
- `tools/breach_barbarian.py` (арт-проект, 2026-10-21, просьба пользователя: «фигуры варваров с рогатым шлемом») — те же
  3D-фигуры `breach_art.py`, корпус и огоньки в цветах сторон; голова под шлемом тёмного железа (`HELM`) с бронзовым ободом
  (`BRONZE`) и заклёпками (`HELMETS`: высота купола, обод, центр и радиус головы), по бокам — рога-кость цепочкой шаров
  наружу и вверх, к концу тоньше и темнее (`horn`, `HORNS`, `HORN`/`TIP`); отделка корпуса и грива коня — мех (`FUR`,
  клоки `FUR_DARK` по шуму), основание — как было. Ферзь — бронзовые шипы вместо шаров короны, верхний камень светится;
  король — крест над шлемом; ладья — железные зубцы, рога по бокам башни; конь — налобник с бронзовой кромкой, рога у ушей
  назад и вверх. Спрайты как обычные (64×84, опора 32,66): `assets/breach/pieces/barbarian/<код>.png` и `_shadow.png`;
  превью `pieces/preview_barbarian_pieces.png` (обычные / варвары) и `preview_barbarian_board.png`. Встроено с 0.80 (варвары rogue-like).
- `tools/breach_barbarian_combo.py` (арт-проект, 2026-10-23, просьба пользователя: «объединённый спрайт и анимацию
  варвара со всеми остальными особыми фигурами») — основа — варвар `breach_barbarian.py`. Анимации — те же генераторы
  особых фигур на спрайте варвара: `from_barbarian` подменяет `Image.open` (путь `pieces/<код>.png` → `pieces/barbarian/`),
  генераторы не тронуты; полосы `effects/barbarian/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_
  <код>.png` (все 12) — разметка кадров и опоры как у обычных (`breach_lava`, `breach_arcs` без концовки — `LOOP_END`,
  `breach_wind`, `breach_darkfire`, `breach_shadowstep`). Спрайты — объёмные заново: `pieces/barbarian_gold/` (корпус и
  шлем — золото варианта `gold` из `breach_gilded`, у чёрных красное; обод и заклёпки — эмаль; рога — кость; мех) и
  `pieces/barbarian_ice/` (`iced_barbarian`: части через `ip.iced`, сосульки и кристаллы как у ледяных, шлем — заиндевелое
  железо `FROST_IRON`; лёд и иней — материалы 11 и 12: `ip.ICE`/`ip.RIME` подменяются в `main`, иначе спорят со шлемом и
  ободом варвара — 5 и 6). Маг с варварскими рогами — отдельный набор `breach_wizard_horns.py`. Превью
  `pieces/preview_barbarian_combo.png` (ряды: варвар, золотой, ледяной, кадр огненного, электрического, воздушного,
  теневого) и `effects/preview_barbarian_combo.gif`. Встроено с 0.96 (объединённые фигуры).
- `tools/breach_gold_combo.py` (арт-проект, 2026-10-23, просьба пользователя: золотые фигуры со всеми анимациями и
  спрайтами особых фигур — как `breach_barbarian_combo.py`) — основа — золотые фигуры игры (`breach_gilded`, вариант
  `gold`, `pieces/gold/`). Анимации — те же генераторы на золотом спрайте (`from_gold` подменяет `Image.open`: `pieces/
  <код>.png` → `pieces/gold/`): `effects/gold/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png`
  (все 12, разметка как у обычных). Спрайты — объёмные заново: `pieces/gold_ice/` (`iced_gold`: украшенная золотая фигура
  через `ip.iced`, сосульки и кристаллы как у ледяных; лёд и иней — материалы 11 и 12, у золотых 5–7 — узор, камни,
  жемчуг) и `pieces/gold_wizard/` (маг `breach_wizard.wizard` с золотым корпусом и эмалью, шляпа — в цвете стороны);
  белый блеск на самом светлом золоте (`shine`). Золотой варвар — уже есть, `pieces/barbarian_gold/`. Превью
  `pieces/preview_gold_combo.png` (ряды: золотые, ледяные, варвары, маги, кадр огненного, электрического, воздушного,
  теневого) и `effects/preview_gold_combo.gif`. Встроено с 0.97 (золотые объединённые фигуры).
- `tools/breach_wizard_combo.py` (арт-проект, 2026-10-23, просьба пользователя: маги со всеми анимациями «по такому же
  принципу», что варвары и золотые) — основа — маги `pieces/wizard/`. Анимации — те же генераторы на спрайте мага
  (`from_wizard` подменяет `Image.open`): `effects/wizard/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|
  shadowstep_in>_<код>.png` (все 12, разметка как у обычных). Спрайт — `pieces/wizard_ice/` (`iced_wizard`: маг через
  `ip.iced`, корпус иссиня-голубой, сосульки с полей шляпы и из-под поясов, у ферзя и короля верхний пояс под шляпой —
  без сосулек; кристаллы у основания; шляпа в цвете стороны; лёд и иней — материалы 11 и 12). Золотой маг —
  `pieces/gold_wizard/`, маг с рогами — `pieces/wizard_horns/`. Превью `pieces/preview_wizard_combo.png` (ряды: маги,
  ледяные, золотые, с рогами, кадр огненного, электрического, воздушного, теневого) и `effects/preview_wizard_combo.gif`.
  Встроено с 0.98 (маги-объединённые).
- `tools/breach_ice_combo.py` (арт-проект, 2026-10-23, просьба пользователя: ледяные фигуры со всеми анимациями — как
  у варваров, золотых и магов) — основа — ледяные фигуры игры `pieces/ice/`. Анимации — те же генераторы на ледяном
  спрайте (`from_ice` подменяет `Image.open`): `effects/ice/<lava_crack|electric_hold|wind|dark_fire|shadowstep_out|
  shadowstep_in>_<код>.png` (все 12, разметка как у обычных). Ледяные спрайты с другими силами уже есть: `gold_ice`,
  `barbarian_ice`, `wizard_ice`. Превью `pieces/preview_ice_combo.png` (ряды: ледяные, золотые, варвары, маги, кадр
  огненного, электрического, воздушного, теневого) и `effects/preview_ice_combo.gif`. Встроено с 0.99.
- `tools/breach_fire_combo.py` (арт-проект, 2026-10-23, просьба пользователя: огненные фигуры со всеми анимациями;
  единственное изменение — часть струй вихря огненно-оранжевая) — основа — огненная фигура в ожидании взрыва: тело —
  кадры трещин `lava_crack` 13–16 (`HOLD`, как `LAVA_HOLD` в игре). У генераторов сил для этого необязательный
  `bodies(f)` — своё тело фигуры в кадре f на холсте 120×120 (`arcs.render`, `wind.render`, `dark.render`,
  `step.render` передаёт в ауру): аура строится по обычному силуэту, тело — кадр трещин `13 + f % 4` (повторы сил
  кратны 4 — шва нет), искры лавы за силуэтом тоже рисуются; без `bodies` вывод прежний (проверено побайтно). У вихря
  ещё `hot` — палитра второй спирали (`arc(..., pal=…)`): огненные струи `k` = 0 (и π/2 у земли), по клетке —
  половина колец; `HOT` — `#ffe08a`, `#ff9a2e`, `#f0641e`, `#a8260c`, подводка `#3d0a04` (янтарные `FIRE[3]`
  сливались с трещинами). Полосы `effects/fire/<electric_hold|wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png`
  (все 12, разметка как у обычных); набухание и взрыв — обычная `lava_crack`. Огонь на других основах уже есть —
  `effects/<ice|gold|barbarian|wizard>/lava_crack_*`. Превью `pieces/preview_fire_combo.png` (ряды: огненные, огненные
  ледяные, золотые, варвары, маги, кадр электрического, воздушного, теневого) и `effects/preview_fire_combo.gif`.
  Встроено с 0.101.
- `tools/breach_spark_combo.py` (арт-проект, 2026-10-23, просьба пользователя: электрические фигуры со всеми
  анимациями; у электрической теневой золотую ауру заменить фиолетовой) — основа — `electric_hold`. У `arcs.render`
  для этого ещё `length` (столько кадров без концовки: после `LOOP_END` повтор продолжается, первые 24 кадра — как у
  `electric_hold`) и `under(f)` — свой слой вместо золотой ауры (контур фигуры тогда золотом не подсвечивается); что
  `bodies` выносит за силуэт, теперь накладывается по альфе (искры лавы непрозрачны — огненные полосы не изменились,
  проверено). Воздушная — `arcs.render(length=40, bodies=кадры wind.render)`: золотая аура позади, вихрь, передние
  молнии поверх; теневая — `under` — тёмная аура `dark.render` без фигуры, `bodies` — её фигура, голубые молнии те же;
  теневой шаг — `step.render(..., aura=эти кадры)` (молнии гаснут с аурой при втягивании, при появлении —
  вступление электричества со вспышкой). Полосы `effects/spark/<wind|dark_fire|shadowstep_out|shadowstep_in>_<код>.png`
  (все 12, по 40 кадров, разметка как у `wind`/`dark_fire`, шаг — как у обычного). Электричество на других основах
  уже есть — `effects/<ice|gold|barbarian|wizard|fire>/electric_hold_*`. Превью `pieces/preview_spark_combo.png` (ряды:
  электрические, ледяные, золотые, варвары, маги, огненные, воздушные, теневые) и `effects/preview_spark_combo.gif`.
  Встроено с 0.102.
- `tools/breach_wind_combo.py` (арт-проект, 2026-10-23, просьба пользователя: воздушные фигуры с теневыми;
  единственное изменение — часть струй вихря фиолетовая) — `wind.render(code, bodies=кадры dark.render, hot=SHADE)`:
  задние струи, тёмная аура (что `bodies` выносит за силуэт, у `wind.render` теперь ложится по альфе — огненные полосы
  не изменились, проверено), фигура, дымка, передние струи; вторая спираль — `SHADE` (`#e2c8ff`, `#b583ff`, `#8644f2`,
  `#5a2aa8`, подводка `#1e0f33` — тона бликов и контура тёмной ауры). Теневой шаг — `step.render(..., aura=эти
  кадры)`: вихрь втягивается с аурой и при появлении набирается заново. Полосы
  `effects/wind/<dark_fire|shadowstep_out|shadowstep_in>_<код>.png` (все 12, 40 кадров, разметка как у
  `dark_fire`/`wind`). Воздух на других основах уже есть — `effects/<ice|gold|barbarian|wizard|fire|spark>/wind_*`.
  Превью `pieces/preview_wind_combo.png` (ряды: воздушные, ледяные, золотые, варвары, маги, огненные, электрические,
  теневые) и `effects/preview_wind_combo.gif`. Встроено с 0.103.
- `tools/breach_wizard.py` (арт-проект, 2026-10-21, просьба пользователя: «фигуры с шляпой мага») — те же 3D-фигуры
  `breach_art.py` в цветах сторон, макушка срезана (`HATS`: срез, поля — высота и радиус, тулья — радиус и высота), сверху
  остроконечная шляпа: поля — скруглённый диск (`disc`), тулья — шары вдоль изогнутой оси, к концу тоньше, кончик заломлен
  влево по экрану и чуть назад (`crown`, `LEAN`, изгиб 0.5 высоты), у основания золотая лента (`GOLD`), на тулье золотые
  звёздочки (`STAR`, `stars`). Шляпа белых синяя, чёрных багровая (`HAT_COLORS`). Кончик: ферзь — светящийся камень (шары
  короны убраны), король — золотой крестик (крест фигуры убран), слон — звезда; у коня шляпа на макушке. Спрайты как
  обычные (64×84, опора 32,66): `assets/breach/pieces/wizard/<код>.png` и `_shadow.png`; превью
  `pieces/preview_wizard_pieces.png` (обычные / маги) и `preview_wizard_board.png`. Встроено с 0.83.
- `tools/breach_wizard_horns.py` (арт-проект, 2026-10-23, просьба пользователя: отдельный спрайт «шляпа мага с
  варварскими рогами», не в анимациях `breach_barbarian_combo.py`) — фигуры-маги `breach_wizard.wizard` (шляпа, лента,
  звёзды, корпус без изменений), из боков тульи над полями (`HATS`: высота полей + 3.6; у коня — над шляпой на макушке)
  растут рога варвара `bb.horn` (`HORN_SHAPE` — наружу 1.05, вверх 1.3, толщина 0.34 радиуса; `HORN_SCALE` = 1.3 радиуса
  тульи — меньше их прятали поля), материалы рогов — 13 и 14 (`HORN_M`, `TIP_M`: у мага 5–7 — шляпа, лента, звёзды). По
  ширине до 4 пикселей за обычный спрайт (ладья). Спрайты как обычные: `assets/breach/pieces/wizard_horns/<код>.png` и
  `_shadow.png`; превью `pieces/preview_wizard_horns.png` (маги / маги с рогами) и `preview_wizard_horns_board.png`.
  Встроено с 0.96 (варвары-маги).
- `tools/breach_armor.py` (арт-проект, 2026-10-21, просьба пользователя: фигуры в латах; два варианта отвергнуты —
  пластины и кольчуга рисунком на теле («латы незаметны») и объёмная броня «как у космодесантников» (наплечники-купола,
  нагрудник) — нужны просто латы, заметнее первого варианта) — те же 3D-фигуры `breach_art.py`: корпус и шлем — сталь
  `PLATE` (у белых светлая, у чёрных воронёная; `steel`), отделка — `BRASS` (у чёрных медь); от основания до шеи
  (`LAMES`) — объёмные полосы лат (`lames`: тело, раздутое от `FLARE` = 0.1 у верхнего края до 2.0 у нижнего, высота
  `LAME` = 5.2 — ниже похоже на пружину; нижняя кромка нависает над следующей и даёт тень, по ней латунная кайма `EDGE`);
  ступенчатые наплечники (`spaulder`: купол и две полосы ниже, заклёпка; `SPAULDERS`, у пешки, слона, ферзя, короля);
  основание и огоньки — как у сторон. Ферзь — латунные шары короны, верхний камень светится; король — латунный крест с
  камнем; конь — латы только на шее (морда цельная), грива — латунь. Спрайты как обычные (64×84, опора 32,66):
  `assets/breach/pieces/armor/<код>.png` и `_shadow.png`; превью `pieces/preview_armor_pieces.png` (обычные / в латах) и
  `preview_armor_board.png`. Встроено с 0.89 (бронированные фигуры волн).
- `tools/breach_shield.py` (арт-проект, 2026-10-21, просьба пользователя: фигуры с большим треугольным, немного
  выпуклым щитом в правой руке, фигура смотрит на противника — к середине доски, щит перед ней; соседей не загораживать) —
  те же 3D-фигуры `breach_art.py` + отдельная объёмная деталь `shield`: треугольник (`heater`: прямой верх, бока почти
  прямые до 0.42 высоты, дальше дугой к острию), выпуклое лицо (`BULGE` = 1.1, толщина `THICK`), развёрнут поперёк корпуса
  (`n = взгляд − 0.45 · правая рука`: плоскостью к зрителю, а не ребром), отклонён назад. Взгляд — `FACING`: белые вверх по
  доске (от зрителя), чёрные вниз (к зрителю); правая рука — `cross(взгляд, Y)`; `SHIELDS`: высота центра, вынос вперёд и
  вправо, высота, ширина. Белые: щит справа, к зрителю тыльной стороной (доски `WOOD`/`WOOD_DARK`, кожаные ремни
  `LEATHER`, кайма), вынесен правее (иначе закрыт корпусом); чёрные: щит слева впереди, лицом к зрителю (поле `FIELD`
  багровое, у белых синее, кайма `BRASS`/медь, светящийся ромб `GLOW`). По ширине спрайт выходит за обычный не больше чем
  на 2 пикселя у белых и 4 у чёрных. При перевёрнутой доске стороны смотрели бы не туда. Спрайты как обычные (64×84,
  опора 32,66): `assets/breach/pieces/shield/<код>.png` и `_shadow.png`; превью `pieces/preview_shield_pieces.png` и
  `preview_shield_board.png`. Не встроено.
- `tools/breach_vampire.py` (арт-проект, 2026-10-22, просьба пользователя: «несколько вариантов внешнего вида фигур в
  вампирском стиле»; выбран объединённый «Дракула» — см. ниже) — те же 3D-фигуры `breach_art.py`, у всех огоньки кроваво-
  красные (`BLOOD_GLOW`). Варианты: `count` «Граф» — стоячий воротник плаща за головой (`collar`, `COLLARS`: оболочка
  сзади, к верху шире, верх зубцами; снаружи `CAPE` чёрный, изнутри `LINING` алый), отделка `CRIMSON`, брошь `GLOW` на
  груди (`BROOCH`), у коня воротника нет (плащ на шее выходил бруском); `blood` «Кровь» — бледный корпус `PALE` (белые —
  кость, чёрные — чёрно-винные), облиты кровью — форма `breach_poison.poisoned`, жидкость `BLOOD`; `bat` «Нетопырь» —
  перепончатые крылья по бокам головы (`wing`: силуэт `WING` с фестонами, пальцы `FINGERS` и верхняя кромка — `RIB`,
  перепонка `MEMBRANE`, обращены к зрителю, кончик заведён назад; `HEADS`), выходят за обычный спрайт до 6 пикселей в
  каждую сторону. Спрайты как обычные: `assets/breach/pieces/vampire_<count|blood|bat>/<код>.png` и `_shadow.png`;
  превью `pieces/preview_vampire_pieces.png` (ряды: обычные, «Граф», «Кровь», «Нетопырь»). Выбор пользователя —
  `dracula` «Дракула» («Граф» и «Кровь» вместе, крови меньше): воротник, багровая отделка и брошь «Графа», корпус
  `DRACULA_SKIN` (белые — кость, чёрные — почти чёрный с винным отливом: чёрно-винный «Крови» сливался с кровью), кровь
  `bloody` — тонкая кайма `LITTLE_COAT` = 0.07 и четыре узких коротких потёка, без лужиц и капель. Спрайты
  `pieces/vampire_dracula/<код>.png` и `_shadow.png`, превью `preview_vampire_dracula.png` (обычные / «Дракула») и
  `preview_vampire_dracula_board.png`; `py tools/breach_vampire.py assets/breach/pieces dracula` — только он. «Дракула» встроен с 0.89 (вампиры волн).
- `tools/breach_armor_vampire.py` (арт-проект, 2026-10-23, просьба пользователя: объединённая версия бронированных и
  вампиров) — `armor.armour` (сталь `PLATE`, полосы лат, наплечники) + от «Дракулы» воротник `vp.collar`, брошь, огни
  `vp.BLOOD_GLOW` и кровь `vp.bloody` (на исходных материалах корпуса до `armor.steel`, те же зерно и потёки, что у
  «Дракулы»); латунь лат — багровый металл. Номера материалов генераторов совпадают — `recolor` переносит их: сталь 5,
  багровое 13, плащ 14, подкладка 15, кровь 16. У коня воротника нет. Спрайты как обычные (64×84, опора 32,66):
  `assets/breach/pieces/armor_vampire/<код>.png` и `_shadow.png`; превью `pieces/preview_armor_vampire.png` (ряды: в латах,
  «Дракула», вместе) и `preview_armor_vampire_board.png`. Встроено с 0.106 (вампиры в броне).
- `tools/breach_zombie.py` (арт-проект, 2026-10-23, просьба пользователя: «дизайн спрайта фигур в тематике зомби») — те же
  3D-фигуры `breach_art.py`: кожа трупная (`SKIN`: белые — бледная серо-зелёная, чёрные — тёмная болотная), пятна гнили
  `ROT` по `ip.noise3` (у белых синюшные, у чёрных тёмные синяки — светлая плесень на тёмном выглядела камуфляжем);
  отделка — тряпьё `RAG` с лохмотьями вниз от кромок поясов (`RAGS`, полосы по 11° азимута разной длины) и дырами до кожи,
  короны — позеленевшая бронза `RUST` (`CROWNS`); вырванные куски (`CHUNKS` → `chunk_spheres`/`bitten`: шар вычтен из
  тела, `wounds` красит срез — мясо `FLESH` с рёбрами `BONE`, на голове — кость, у ладьи обломанные зубцы — гниль); шрамы
  со стежками (`SCARS`, `scar_mask`, `SEAM`); основание в земле `EARTH` с мхом `MOSS`, ступня — камень цвета отделки
  (рук, тянущихся к противнику, по просьбе пользователя нет — спрайт по ширине как обычный); огни `GLOWS` — у белых
  гнилостно-зелёные, у чёрных янтарные. Пешка — светящийся глаз, пустая глазница и рот с зубами (`pawn_face`), ферзь —
  костяные шишки короны (двух нет), король — покосившийся костяной крест (`crooked_cross`), конь — рваное ухо, вырванный
  бок шеи с рёбрами, зубы, свалявшаяся грива `HAIR` с вырванными клоками (`zombie_knight`). Материалы 5–12. Спрайты как
  обычные (64×84, опора 32,66): `assets/breach/pieces/zombie/<код>.png` и `_shadow.png`; превью
  `pieces/preview_zombie_pieces.png` (обычные / зомби) и `preview_zombie_board.png`. Встроено с 0.105 (зомби волн).
- `tools/breach_poison_zombie.py` (арт-проект, 2026-10-23, просьба пользователя: объединённая версия ядовитых и зомби) —
  зомби `zb.zombie` целиком, сверху облит ядовитой жидкостью, как ядовитые фигуры (`slimed` — логика `bp.slimy`
  поверх материалов зомби: шапка `bp.COAT`, потёки и лужицы на уступах выше основания; потёки — `drips_of`, те же зерно и
  длины, что у `bp.poisoned`; набухшие капли на концах длинных потёков; блики `bp.GLINT`). Заливает `WET` — кожу, гниль,
  тряпьё, бронзу, гриву; огни, рот и глазница пешки, шрамы (`SEAM`), раны и земля — видны. Жидкость — материал 13
  (`SLIME`, палитра `bp.SLIME_RAMP`), у зомби заняты 5–12. Спрайты как обычные (64×84, опора 32,66):
  `assets/breach/pieces/poison_zombie/<код>.png` и `_shadow.png`; превью `pieces/preview_poison_zombie.png` (ряды:
  ядовитые, зомби, вместе) и `preview_poison_zombie_board.png`. Встроено с 0.107 (ядовитые зомби).
- `tools/breach_castle.py` (арт-проект, 2026-10-29, просьба пользователя: замок на клетках a3–a6 и b3–b6; превью с
  фигурами не нужно) — история: собор Святого Вита (`breach_cathedral.py` и `landmarks/st_vitus*` удалены: пользователю
  показался пластмассовым, игрушечным — гладкие плоскости одним тоном, чистая плита основания), замок со стенами и
  донжоном, дворец буквой П, дворец по образцу Букингемского; правки по ходу: камень бежевый с чёрными оттенками,
  никакой травы и скалы — стоит прямо на клетках. Сейчас — по рисунку-плану пользователя («вернёмся к замку до
  Букингемского, только в такой форме»: прямоугольник, кружки на передних углах — башни, середина переднего края —
  ворота, внутри — готическая крыша): один длинный корпус u ±78, v от −40 до 24 (`EAVES` 52) — три этажа стрельчатых
  окон, контрфорсы с пинаклями (`pinnacle`), крутая черепичная кровля (конёк `RIDGE_V`/`RIDGE_Y`, `roof_y`) со
  стрельчатыми слуховыми окнами (`gothic_dormer`), гребнем (`battlement` по коньку) и шпилем-фонарём (`lathe`,
  `lantern_slots`); западный щипец с окном с переплётом (`tracery`); надвратный ризалит (`gate`, `herald`, крутой
  щипец, угловые башенки); круглые башни на передних углах (`round_tower`: откос, машикули `ring_corbels`, зубцы
  `ring_battlement`, бойницы `slits`, шиферный конус с флажком); две трубы на заднем скате. Фактура против
  «пластмассы»: кладка блоками разного тона, копоть у земли и чёрные потёки (`masonry`), плиты (`flagstones`),
  черепица и шифер рядами (`tiles`) — материал детали `deco(основа, указания…)` даёт и материал, и сдвиг тона;
  затенение (`Scene.ambient`), тени (луч к `LIGHT_DIR`, тон −2), горящие окна `LIT`; палитра камня `STONE` бежевая,
  тоны 0–1 почти чёрные. Лучевой рендер (масштаб `K` = 0.86, клетка `L` = 46, свет `art.L_SHADE`, 5 тонов, контур и
  швы; оси u — с запада на восток (мир −z), v — с севера на юг (мир +x), начало — вершина a4/a5/b4/b5). Файлы
  `assets/breach/landmarks/castle.png` (198×242) и `castle_shadow.png` (тот же размер, альфа 77, свет `SHADOW_DIR`
  круче, чем у фигур); точка начала осей на спрайте (86, 155) — ставится на нижний угол ромба клетки a5; рисовать:
  тень, фигуры за замком (горизонтали 7–8 у a–b), замок, остальные. Превью `landmarks/preview_castle.png` (пустая
  доска). `--quick` — без сглаживания. Не встроено.
- `tools/breach_university.py` (арт-проект, 2026-10-29, просьба пользователя: вместо базара — здание университета; базар
  `breach_bazaar.py` и `landmarks/bazaar*` удалены) — на a1, a2, b1, b2, у западного торца замка `breach_castle.py`, в
  том же стиле (бежевый песчаник с копотью, черепица, шифер, тёплые окна). Главный корпус — библиотека: стрельчатые
  окна с переплётом (`tracery`), контрфорсы с пинаклями, черепичная кровля со слуховыми окнами и короткими трубами;
  часовая башня к доске (верх 92): стрельчатый портал с дубовыми дверями (`door`) и крыльцом, герб — синий щит с
  раскрытой книгой (`book_crest`), часы на двух видимых гранях (`clock`, палитра `CLOCK` = 20 в `bc.PALETTE`), окна
  звонницы, зубчатый парапет с угловыми пинаклями, фонарь с проёмами и шиферным куполом, золотое навершие с флюгером;
  западное крыло — щипец к доске с окном с переплётом и эркер на западной стене, как у пражского Каролинума
  (`frustum` n = 8: гранёная призма, консоль — перевёрнутая пирамида, шатёр). Движок, тела и фактура — из
  `breach_castle` (`import breach_castle as bc`). Оси — как у замка, начало — общая вершина a1, a2, b1, b2 (на 3
  клетки к 1-й горизонтали от начала замка). Файлы `assets/breach/landmarks/university.png` (130×174) и
  `university_shadow.png`; точка начала осей на спрайте (55, 114) — ставится на нижний угол ромба клетки a2;
  рисуется после замка (ближе). Превью `landmarks/preview_university.png` — пустая доска с замком и университетом.
  `--quick` — без сглаживания. Не встроено.
- `tools/breach_city.py` (арт-проект, 2026-10-29, просьба пользователя: город игрока — главное здание с королевскими
  указами, набор новобранцев с бесконечными пешками, магазин зданий; 14 зданий по видам фигур и силам усиливают свои
  фигуры, как медали) — **правила пользователя для всего города**: где здание, там фигур нет (здание занимает свои
  клетки целиком); главное здание, набор, король, ферзь, ладья — 2×2, конь — три клетки буквой «Г», слон — три клетки
  в линию (размеры пешки и сил не названы); цвета по зерну забега не меняются, анимаций сил на зданиях нет. Здания —
  в стиле замка (движок, тела и фактура `breach_castle.py`; палитры `STRAW` 21, `PLASTER` 22, `EARTH` 23 в
  `bc.PALETTE`; узоры `door`, `stalls` — ворота денников, `timber` — фахверк, `rose` — круглое окно, `ribs` — рёбра
  купола, `crenels` — зубцы по периметру, `flagpole`): `main` — донжон с шиферной пирамидой и флагом, низкие стены с
  зубцами, четыре круглые башни, ворота с гербом; `recruit` — казарма с фахверком сзади, двор за частоколом с чучелами,
  стойкой копий, бочками и знаменем; `king` — дворец с портиком, угловые павильоны, купол с золотыми рёбрами и золотой
  короной; `queen` — дворец под вальмовой кровлей, круглые башенки с золотыми шарами, восьмигранная башня со шпилем,
  кольцом шаров и шаром наверху, щипец с розой; `rook` — массивная круглая башня-ладья с машикулями и крупными зубцами
  в низкой стене; `knight` — конюшня «Г» (клетки (0,0), (1,0), (1,1): угол сзади, внутренний угол к зрителю) с
  фахверком, воротами денников внутрь угла, башенкой-флюгером; `bishop` — церковь вдоль горизонталей: неф с
  контрфорсами, фасад с порталом и розой к 1-й горизонтали, колокольня сзади со шпилем и крестом. Оси — как у замка
  (u — к 8-й горизонтали, v — к вертикали h), начало — угол клетки (0, 0) здания у вертикали a и 1-й горизонтали (левый
  угол ромба); видны грани −u и +v. Файлы `assets/breach/city/<имя>.png` и `<имя>_shadow.png`, точки начала осей —
  `anchor_<имя>.txt`/`anchors.txt` (могут быть отрицательными — начало левее холста); превью `city/preview_city.png`
  (пустая доска: набор a1–b2, главное d1–e2, ладья g1–h2, церковь c3–c5, конюшня e4, e5, f5, король a6–b7, ферзь
  f7–g8; порядок — по передней вершине). Остальные девять — с временными размерами (пользователь выбрал «дорисовать»,
  размеры ещё не назвал): 1×1 — `pawn` (дом ополчения, каменная пешка), `wind` (мельница, крылья `sail`), `spark`
  (башня с медной катушкой, шаром и громоотводом), `wizard` (башня мага с синей шляпой в звёздах); 2×1 — `forge`
  (кузня с навесом, горном, наковальней), `ice` (ледяной чертог: голубые блоки, ледяная черепица со снегом `snowcap`,
  сосульки, башня с хрустальным шпилем, кристаллы), `treasury` (казна: портик с монетой, решётки, золотой гребень,
  сундуки), `crypt` (склеп тёмного камня `DARK` с фиолетовыми окнами `VIOLET`, обелиски), `longhouse` (длинный дом
  варваров под соломой `THATCH`, доски-рога `rbox`, щиты, рогатые черепа). Палитры 24–33 в `bc.PALETTE`; светящиеся
  (`VIOLET`, `ARC`, `MAGIC`) — в `bc.FLAT`, светлые во всех тонах. Отражение через вертикаль экрана — `mirror`
  ((u, v) → (Wv − v, Wu − u), имя `<имя>_m`): фасады −u и +v меняются местами, 2×1 и линия встают вдоль горизонталей.
  Превью `city/preview_town.png` — поле, застроенное всеми 16 зданиями (`TOWN`: высокие — сзади, у 8-й горизонтали и
  вертикали a; низкие — спереди; 24 свободные клетки — улицы: вертикаль c, 4-я и 7-я горизонтали, площадь перед
  королём); `city/preview_full.png` — все 64 клетки (`FULL` = `TOWN` + повторы тех же зданий на свободных клетках,
  одинаковые не рядом; превью по просьбе пользователя «когда все клетки заполнены»); порядок рисования — `draw_order`
  (кто целиком дальше, раньше; иначе по передней вершине). «Г» с внутренним
  углом сбоку одним спрайтом в очередь рисования не встаёт — его придётся резать по клеткам. `py tools/breach_city.py
  [имя | имя_m …] [--quick] [--preview]`. Не встроено.
- `tools/breach_portal.py` — портал под фигурой для карты обмена (16 кадров 80×116,
  60 мс, центр верхней грани клетки — `CX, CY` = 40, 88) в `assets/breach/effects/`:
  `portal_vanish_*` и `portal_appear_*`, слои `_under` (вихрь, задняя половина кольца,
  свечение — до фигур) и `_over` (передняя половина кольца, искры, вспышка — после).
  Фигуру рисует игра: в кадре f сдвиг вниз `SINK[вид][f]` (до 72 px) и срез по передней
  кромке кольца — эллипс `RX, RY` = 17, 12.75 с масштабом `SCALE[f]`, на срезе пиксель
  `#a6f2ff`; тень фигуры — только пока сдвиг 0. Обмен: исчезновение на обеих клетках,
  затем появление с переставленными фигурами. Арт карты — `cards/art/swap.png` (две
  белые пешки на порталах, голубой знак ⇄), сжигание — `burn_strip_swap_w/b.png`. Встроены
  с 0.18; в игре обмен — карта победы: картинка на лице победы, сжигание —
  `burn_strip_swap_victory.png`.
- `tools/breach_bless.py` — благословение слона (только слон): светится, раскрывает белые
  ангельские крылья, волна света на клетку вокруг (24 кадра, 60 мс) в
  `assets/breach/effects/`: `bless_under` (свет под слоном и кольцо по соседним клеткам —
  до фигур; кадр 184×168, `CX, CY` = 92, 98, как у огненной волны), `bless_figure_w/b`
  (слон с крыльями и ореолом — вместо спрайта слона; кадр 120×112, точка опоры — центр
  клетки `FAX, FAY` = 60, 92; тень — обычная тень слона), `bless_over` (столбы света над
  соседями, искры — после фигур). Крыло — по образцу пользователя (классические
  раскрытые крылья: гладкая кромка от плеча вверх-наружу, под ней ряды округлых перьев —
  кроющие, средние, маховые — с фестончатым нижним краем, кончик заострён):
  `spread_wing(right, geom, scale, rot)` строит его прямо в пикселях в любом масштабе и
  наклоне; `WING_ANIM` — для анимации (раскрытое — `WING_OPEN_SCALE` = 0.62), `WING_CARD` —
  пропорции под окно карты (меньше перьев, мелко не слипаются). Арт карты —
  `cards/art/bless.png` (слон с нимбом и крыльями, кольцо света), сжигание —
  `burn_strip_bless_w/b.png`. Встроено с 0.22 как карта победы «Исцеление» (просьба
  пользователя): картинка на лице победы с ценой 2, сжигание — `burn_strip_heal_victory.png`.
- `tools/breach_contract.py` — арт карты «Договор» (окно 38×25), варианты в
  `assets/breach/cards/art/contract_<вариант>.png`: `empty` (свиток на пустом фоне),
  `table` (на столе, перо и чернильница), `blood` (в крови), `candle` (лист под наклоном,
  свеча), `pact` (тёмный «кровавый договор»: красные письмена, чёрная печать, отпечаток
  пальца), `table_blood` (стол и лужа крови). Свиток по образцу пользователя: валики,
  рваные края, красная лента, восковая печать. Пользователь выбрал вариант 3 (`blood`,
  нумерация как в превью: 1 `empty` … 6 `table_blood`) для карты «Дополнительное
  финансирование» (карта жертвы); встроено с 0.20, сжигание —
  `burn_strip_funding_sacrifice.png`.
- `tools/breach_emblems.py` (наш) — венок победы и череп жертвы с лиц карт отдельно
  (`emblem_wreath.png` 15×12, `emblem_skull.png` 11×12 в `assets/breach/cards/`) — для
  счётчиков очков, с 0.23 и ромб маны `emblem_gem_w/b.png` (с 0.23.1 — 12×14, вытянут
  вверх и заходит на рамку карты; цена на нём белая с контуром, `COST_TINT`); рисуют их
  функции `emblem_wreath`/`emblem_skull`/`emblem_gem` из `breach_cards.py`. Числа
  счётчиков (`paint_points`) стоят на одной линии — над самым высоким знаком.
- `tools/breach_freeze.py` — заморозка клетки и фигуры, повторяемая по клеткам (18 кадров
  80×116, 60 мс, центр верхней грани клетки — `CX, CY` = 40, 88) в `assets/breach/effects/`:
  `freeze_tile_<v>` (снег на клетке: наметает от середины, сугробики, искорки, спереди
  бортик слоя; заполняет весь ромб — соседние замёрзшие клетки сливаются; до фигур),
  `freeze_block_<P|N|B|R|Q|K>_<v>` (полупрозрачная глыба своей высоты — после фигуры, на
  пустой клетке не рисуется; формы: 0 брусок, 1 восьмигранник, 2 узкая с осколками,
  3 скошенная с осколком), v = `variant_of(file, rank)` = (3·file + 5·rank) % 4, `freeze_over`
  (снег, пар, искры — после фигур); фигуры в инее — `pieces/frozen/<цвет><фигура>.png`
  (как обычные спрайты), иней поднимается от основания: в кадре f замёрзшая часть ниже
  линии `FROST[f]` × высота `PIECE_TOP[фигура]`. Много клеток (полполя): на каждой те же
  кадры со сдвигом `DELAY_PER_TILE` = 2 кадра на клетку расстояния (Чебышёва) от точки
  заморозки; последний кадр — состояние «заморожено», держать. Встроено с 0.25 — буря,
  лавина и ледяной барьер (обычные карты пользователя, по 10 маны).
- `assets/` — `breach/` (доска и фигуры), звуки lichess, `piece/cburnett/wN.svg` —
  только значок окна; `assets/battle-chess.ico` — значок ярлыка
  (`cargo run --release --example make_icon -- assets/battle-chess.ico`).

## Проверка интерфейса

Снимок окна — `PrintWindow`; с 2026-10-13 он даёт белый кадр, надёжнее снимок изнутри
игры: демо-код по `BC_SHOT=<путь.png>` через ~90 кадров шлёт
`ViewportCommand::Screenshot`, ловит `Event::Screenshot` и пишет PNG
(`tiny_skia::Pixmap::save_png`), потом выходит. Клики через `PostMessage` не доходят: winit сверяет
курсор через `WindowFromPoint`. Мышь пользователя не двигать. Нужное состояние
(выбранная фигура, наведение, позиция) задавать временным демо-кодом по переменной
окружения, окно уводить за экран `ViewportCommand::OuterPosition`; после проверки
демо-код удалять. Демо-код не должен менять `self.settings`: `save_settings` сразу пишет их в
настоящий `settings.ini` пользователя (так в 0.39 случайно включился «усиленный компьютер»).

Частоту кадров за экраном не мерить: Windows душит VSync невидимых окон (~27 FPS).
Замер с VSync — окном на экране с `with_active(false)`; без VSync можно за экраном.
Монитор пользователя 144 Гц, видеокарта RX 9060 XT (есть ещё встроенная).
