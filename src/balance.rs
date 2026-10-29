//! Баланс игры — все числа правил: характеристики фигур, цены и числа карт, правила хода и
//! rogue-like, составы пяти колод. С 0.133 (просьба пользователя) файла `balance.ini` нет: числа —
//! здесь, в таблицах `RULES`, `PARAMS`, `DEFAULT_PIECES`; их список для пользователя — `Баланс.txt`
//! в корне проекта (пишет `Balance::text`, тест `refresh_balance_list`).
//!
//! Правила читают числа через [`balance`].

#[cfg(test)]
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::game::{CardKind, DECK_COUNT, DEFAULT_RECIPES, Kind};
#[cfg(test)]
use crate::game::{DECK_NAMES, Family};

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
    RogueLevels,
    RoguePawnDrop,
    RogueGoldPawn,
    RogueGoldKnight,
    RogueGoldBishop,
    RogueGoldRook,
    RogueGoldQueen,
    RogueRerollCost,
    RogueShopPercent,
    RogueFirePercent,
    RogueSparkPercent,
    RogueWindPercent,
    RogueGoldPiecePercent,
    RogueGoldBonus,
    RogueShadePercent,
    RogueIcePercent,
    RogueBossHp,
    RogueBossAttack,
    RogueBossGold,
    RogueCrestHeartHp,
    RogueCrestFreezeTurns,
    RogueCrestCrystalHp,
    RogueKnightHp,
    RoguePoisonLevel,
    RoguePoisonPercent,
    RoguePoisonDamage,
    RogueBarbarianPercent,
    RogueBarbarianRepeats,
    RogueCrestAxeHp,
    RogueCrestFangsHeal,
    RogueCrestWhirlRadius,
    RogueWizardPercent,
    RogueCrestFlamesReach,
    RogueCrestDarkPortalHp,
    RogueCrestDarkDodgeEvery,
    RogueArmorLevel,
    RogueArmorWave,
    RogueArmorPercent,
    RogueArmorHp,
    RogueVampireLevel,
    RogueVampireWave,
    RogueVampirePercent,
    RogueVampireHp,
    RogueVampireHeal,
    RogueCrestRedBoltStep,
    RogueCrestRedBoltMax,
    RogueCrestGoldKingBonus,
    RogueCrestKingQueenBonus,
    RogueCrestPiggyStep,
    RogueComboPercent,
    RogueComboWeightPercent,
    RogueCrestFireAxeHp,
    RogueCrestFireAxeRepeats,
    RogueZombieLevel,
    RogueZombieWave,
    RogueZombiePercent,
    RogueZombieHp,
    RogueZombieRiseHp,
    RogueArmoredVampireLevel,
    RogueArmoredVampireWave,
    RogueArmoredVampirePercent,
    RogueArmoredVampireHp,
    RogueVenomZombieLevel,
    RogueVenomZombieWave,
    RogueVenomZombiePercent,
    RogueRerollStep,
    RogueDropsPerTurn,
    RogueMaxPieces,
    RogueFirstLevelHpCut,
    RogueWizardEvery,
    RogueChestPiecesPrice,
    RogueChestChoices,
    RogueHandSize,
    RogueGoldSellPercent,
    RogueRareCardPercent,
    RogueChestSlots,
    RogueBigChestPercent,
    RogueBigChestChoices,
    RogueBigChestPicks,
    RogueBigChestPricePercent,
    RogueCrestMultAddPercent,
    RogueCrestKingMultPercent,
    RogueCrestFewBase,
    RogueCrestPawnMultPercent,
    RogueCrestKnightAddPercent,
    RogueCrestSwordMultPercent,
    RogueCrestFirstAddPercent,
    RogueCrestQueenAddPercent,
    RogueCrestFlameMultPercent,
    RogueCrestSnowMultPercent,
    RogueCrestMeteorAddPercent,
    RogueCrestJumpAddPercent,
    RogueCrestBombMultPercent,
    RogueCrestFlameSwordAddPercent,
    RogueCrestAxeMultPercent,
    RogueLevelHpPercent,
    RogueLevelHpStepPercent,
    RogueCrestBoltAddPercent,
    RogueCrestWindAddPercent,
    RogueCardSellPrice,
    RogueBuildingMultPercent,
    RogueIceHp,
    RogueCrestBishopAddPercent,
    RogueCrestNineEvery,
    RogueCrestNineMultPercent,
    RogueCrestCardAddPercent,
    RogueCrestPawnKills,
    RogueCrestKnifeCoinPercent,
    RogueCrestDiceMax,
    RogueCrestBlankAddPercent,
    RogueCrestSkullAddPercent,
    RogueCrestSkullChance,
    RogueCrestBushAddPercent,
    RogueCrestKnightGold,
    RogueCrestClockBasePercent,
    RogueCrestClockStepPercent,
    RogueBuildingAttack,
    RogueKillGoldBonus,
    RogueChestBuildingsPrice,
    RogueBuildingPrice,
    RogueBuildingSellPercent,
    RogueDecreePrice,
    RogueDecreeOwnPercent,
    RogueRecruitPrice,
    RogueChestCardsPrice,
    RogueChestPiecesWeight,
    RogueChestBuildingsWeight,
    RogueChestCardsWeight,
    RogueBuildingWeightPawn,
    RogueBuildingWeightKnight,
    RogueBuildingWeightBishop,
    RogueBuildingWeightRook,
    RogueBuildingWeightQueen,
    RogueBuildingWeightKing,
    RogueBuildingWeightPower,
}

#[cfg_attr(not(test), allow(dead_code))] // ключ и пояснение — для списка `Баланс.txt`
pub struct RuleDef {
    pub key: &'static str,
    pub default: i32,
    pub note: &'static str,
}

/// В порядке `Rule`.
pub const RULES: [RuleDef; 149] = [
    RuleDef {
        key: "mana_per_turn",
        default: 2,
        note: "сколько маны сторона получает в начале каждого своего хода (мана копится)",
    },
    RuleDef {
        key: "cards_per_turn",
        default: 3,
        note: "сколько карт можно сыграть за ход — до хода фигурой и после него вместе",
    },
    RuleDef {
        key: "start_hand",
        default: 3,
        note: "сколько карт сдаётся в руку перед первым ходом",
    },
    RuleDef {
        key: "max_hand",
        default: 7,
        note: "больше карт в руке не бывает: с полной рукой карта остаётся в колоде",
    },
    RuleDef {
        key: "spark_percent",
        default: 100,
        note: "rogue-like: сколько процентов атаки электрической фигуры достаётся каждой \
               чужой фигуре, задетой её молнией",
    },
    RuleDef {
        key: "rogue_waves",
        default: 8,
        note: "rogue-like: сколько волн на первом уровне (на каждом следующем — на одну больше)",
    },
    RuleDef {
        key: "rogue_wave_min",
        default: 3,
        note: "rogue-like: меньше всего фигур в волне",
    },
    RuleDef {
        key: "rogue_wave_max",
        default: 6,
        note: "rogue-like: больше всего фигур в волне (если меньше наименьшего — берётся наименьшее)",
    },
    RuleDef {
        key: "rogue_moves",
        default: 4,
        note: "rogue-like: сколько ходов подряд делает игрок за свой ход — каждый другой фигурой",
    },
    RuleDef {
        key: "rogue_weight_pawn",
        default: 15,
        note: "rogue-like: вес пешки при выборе фигур волны — чем больше, тем чаще",
    },
    RuleDef {
        key: "rogue_weight_knight",
        default: 5,
        note: "rogue-like: вес коня в волне",
    },
    RuleDef {
        key: "rogue_weight_bishop",
        default: 5,
        note: "rogue-like: вес слона в волне",
    },
    RuleDef {
        key: "rogue_weight_rook",
        default: 3,
        note: "rogue-like: вес ладьи в волне",
    },
    RuleDef {
        key: "rogue_weight_queen",
        default: 1,
        note: "rogue-like: вес ферзя в волне",
    },
    RuleDef {
        key: "rogue_levels",
        default: 10,
        note: "rogue-like: сколько уровней; на каждом следующем на одну волну больше (на первом — rogue_waves)",
    },
    RuleDef {
        key: "rogue_pawn_drop",
        default: 2,
        note: "rogue-like: на сколько вес пешки в волнах меньше с каждым следующим уровнем (не ниже 0)",
    },
    RuleDef {
        key: "rogue_gold_pawn",
        default: 1,
        note: "rogue-like: золото за побеждённую пешку, кроме rogue_kill_gold_bonus (цена в магазине — rogue_shop_percent от него)",
    },
    RuleDef {
        key: "rogue_gold_knight",
        default: 2,
        note: "rogue-like: золото за побеждённого коня",
    },
    RuleDef {
        key: "rogue_gold_bishop",
        default: 2,
        note: "rogue-like: золото за побеждённого слона",
    },
    RuleDef {
        key: "rogue_gold_rook",
        default: 3,
        note: "rogue-like: золото за побеждённую ладью",
    },
    RuleDef {
        key: "rogue_gold_queen",
        default: 4,
        note: "rogue-like: золото за побеждённого ферзя",
    },
    RuleDef {
        key: "rogue_reroll_cost",
        default: 5,
        note: "rogue-like: сколько золота стоит обновление сундуков в магазине",
    },
    RuleDef {
        key: "rogue_shop_percent",
        default: 200,
        note: "rogue-like: цена фигуры в магазине, % от золота за её победу (200 — вдвое дороже)",
    },
    RuleDef {
        key: "rogue_fire_percent",
        default: 200,
        note: "rogue-like: огненная фигура в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_spark_percent",
        default: 200,
        note: "rogue-like: электрическая фигура в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_wind_percent",
        default: 200,
        note: "rogue-like: воздушная фигура в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_gold_piece_percent",
        default: 200,
        note: "rogue-like: золотая фигура в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_gold_bonus",
        default: 2,
        note: "rogue-like: сколько золота сверху даёт победа, одержанная золотой фигурой",
    },
    RuleDef {
        key: "rogue_shade_percent",
        default: 200,
        note: "rogue-like: теневая фигура в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_ice_percent",
        default: 200,
        note: "rogue-like: ледяная фигура в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_boss_hp",
        default: 80,
        note: "rogue-like: здоровье короля-толстосума (босс последней волны первого уровня), в единицах",
    },
    RuleDef {
        key: "rogue_boss_attack",
        default: 10,
        note: "rogue-like: атака короля-толстосума, в единицах",
    },
    RuleDef {
        key: "rogue_boss_gold",
        default: 10,
        note: "rogue-like: золото за победу над королём-толстосумом",
    },
    RuleDef {
        key: "rogue_crest_heart_hp",
        default: 20,
        note: "rogue-like, герб снежинки и сердца: прибавка к здоровью ледяных фигур, в единицах",
    },
    RuleDef {
        key: "rogue_crest_freeze_turns",
        default: 2,
        note: "rogue-like, герб двойной снежинки: на сколько ходов ледяные фигуры замораживают ударившего",
    },
    RuleDef {
        key: "rogue_crest_crystal_hp",
        default: 10,
        note: "rogue-like, герб замороженной снежинки: с таким здоровьем (в единицах) ледяная фигура вмерзает в лёд",
    },
    RuleDef {
        key: "rogue_knight_hp",
        default: 30,
        note: "rogue-like: здоровье коня (обеих сторон), в единицах; в обычной партии — из [piece.Knight]",
    },
    RuleDef {
        key: "rogue_poison_level",
        default: 2,
        note: "rogue-like: на каком уровне фигуры волн бывают ядовитыми",
    },
    RuleDef {
        key: "rogue_poison_percent",
        default: 100,
        note: "rogue-like: шанс фигуры волны на этом уровне выйти ядовитой, %",
    },
    RuleDef {
        key: "rogue_poison_damage",
        default: 10,
        note: "rogue-like: урон яда в конце следующего хода отравленной фигуры, в единицах",
    },
    RuleDef {
        key: "rogue_barbarian_percent",
        default: 200,
        note: "rogue-like: фигура-варвар в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_barbarian_repeats",
        default: 1,
        note: "rogue-like: сколько раз подряд варвар бьёт ещё раз после убийства",
    },
    RuleDef {
        key: "rogue_crest_axe_hp",
        default: 10,
        note: "rogue-like, герб топора и меча: на столько меньше здоровья у варваров, в единицах",
    },
    RuleDef {
        key: "rogue_crest_fangs_heal",
        default: 10,
        note: "rogue-like, герб топора и клыков: лечение варвара за убийство (раз за ход), в единицах",
    },
    RuleDef {
        key: "rogue_crest_whirl_radius",
        default: 2,
        note: "rogue-like, герб топора и красного круга: радиус кругового удара варвара, клеток",
    },
    RuleDef {
        key: "rogue_wizard_percent",
        default: 200,
        note: "rogue-like: фигура-маг в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_crest_flames_reach",
        default: 2,
        note: "rogue-like, герб двух огоньков: взрыв огненной фигуры достаёт крестом на столько клеток",
    },
    RuleDef {
        key: "rogue_crest_dark_portal_hp",
        default: 10,
        note: "rogue-like, герб теневого огня и портала: с таким здоровьем (в единицах) и меньше теневая фигура уходит порталом от смертельного удара",
    },
    RuleDef {
        key: "rogue_crest_dark_dodge_every",
        default: 3,
        note: "rogue-like, герб двух теневых огней: теневая фигура не получает каждый N-й удар по ней",
    },
    RuleDef {
        key: "rogue_armor_level",
        default: 3,
        note: "rogue-like: на каком уровне в волнах бывают бронированные (только на нём)",
    },
    RuleDef {
        key: "rogue_armor_wave",
        default: 1,
        note: "rogue-like: с какой волны уровня бывают бронированные фигуры",
    },
    RuleDef {
        key: "rogue_armor_percent",
        default: 100,
        note: "rogue-like: шанс фигуры волны выйти бронированной, %",
    },
    RuleDef {
        key: "rogue_armor_hp",
        default: 30,
        note: "rogue-like: прибавка к здоровью бронированной фигуры, в единицах",
    },
    RuleDef {
        key: "rogue_vampire_level",
        default: 4,
        note: "rogue-like: на каком уровне в волнах бывают фигуры-вампиры (только на нём)",
    },
    RuleDef {
        key: "rogue_vampire_wave",
        default: 1,
        note: "rogue-like: с какой волны уровня бывают фигуры-вампиры",
    },
    RuleDef {
        key: "rogue_vampire_percent",
        default: 100,
        note: "rogue-like: шанс фигуры волны выйти вампиром, %",
    },
    RuleDef {
        key: "rogue_vampire_hp",
        default: 10,
        note: "rogue-like: прибавка к здоровью фигуры-вампира, в единицах",
    },
    RuleDef {
        key: "rogue_vampire_heal",
        default: 10,
        note: "rogue-like: сколько здоровья вампир восстанавливает за каждый удар по чужой фигуре, в единицах",
    },
    RuleDef {
        key: "rogue_crest_red_bolt_step",
        default: 10,
        note: "rogue-like, герб красной молнии: за столько убийств электрической фигуры молния летит на клетку дальше",
    },
    RuleDef {
        key: "rogue_crest_red_bolt_max",
        default: 3,
        note: "rogue-like, герб красной молнии: на сколько клеток самое большее удлиняется молния",
    },
    RuleDef {
        key: "rogue_crest_gold_king_bonus",
        default: 1,
        note: "rogue-like, герб короля: ещё столько золота за победу золотой фигурой",
    },
    RuleDef {
        key: "rogue_crest_king_queen_bonus",
        default: 2,
        note: "rogue-like, герб короля и ферзя: ещё столько золота за победу золотым ферзём",
    },
    RuleDef {
        key: "rogue_crest_piggy_step",
        default: 10,
        note: "rogue-like, герб свиньи-копилки: за каждые столько золота каждая золотая фигура приносит монету в конце уровня",
    },
    RuleDef {
        key: "rogue_combo_percent",
        default: 200,
        note: "rogue-like: объединённая фигура (варвар с другой силой) в магазине — % от цены обычной такой же",
    },
    RuleDef {
        key: "rogue_combo_weight_percent",
        default: 25,
        note: "rogue-like: все объединённые фигуры вместе выпадают в магазине и сундуке фигур с таким % \
               от шанса всех одиночных особых (25 — в 4 раза реже)",
    },
    RuleDef {
        key: "rogue_crest_fire_axe_hp",
        default: 10,
        note: "rogue-like, герб огненного топора: с таким здоровьем (в единицах) и меньше варвар бьёт после убийства лишний раз",
    },
    RuleDef {
        key: "rogue_crest_fire_axe_repeats",
        default: 1,
        note: "rogue-like, герб огненного топора: сколько лишних повторов удара",
    },
    RuleDef {
        key: "rogue_zombie_level",
        default: 5,
        note: "rogue-like: на каком уровне в волнах бывают фигуры-зомби (только на нём)",
    },
    RuleDef {
        key: "rogue_zombie_wave",
        default: 1,
        note: "rogue-like: с какой волны уровня бывают фигуры-зомби",
    },
    RuleDef {
        key: "rogue_zombie_percent",
        default: 100,
        note: "rogue-like: шанс фигуры волны выйти зомби, %",
    },
    RuleDef {
        key: "rogue_zombie_hp",
        default: 10,
        note: "rogue-like: прибавка к здоровью фигуры-зомби, в единицах",
    },
    RuleDef {
        key: "rogue_zombie_rise_hp",
        default: 10,
        note: "rogue-like: с каким здоровьем зомби восстаёт через ход после гибели (один раз), в единицах",
    },
    RuleDef {
        key: "rogue_armored_vampire_level",
        default: 6,
        note: "rogue-like: на каком уровне в волнах бывают вампиры в броне (только на нём)",
    },
    RuleDef {
        key: "rogue_armored_vampire_wave",
        default: 1,
        note: "rogue-like: с какой волны уровня бывают вампиры в броне",
    },
    RuleDef {
        key: "rogue_armored_vampire_percent",
        default: 100,
        note: "rogue-like: шанс фигуры волны выйти вампиром в броне, %",
    },
    RuleDef {
        key: "rogue_armored_vampire_hp",
        default: 30,
        note: "rogue-like: прибавка к здоровью вампира в броне, в единицах (лечится ударом раз за ход)",
    },
    RuleDef {
        key: "rogue_venom_zombie_level",
        default: 7,
        note: "rogue-like: на каком уровне в волнах бывают ядовитые зомби (только на нём)",
    },
    RuleDef {
        key: "rogue_venom_zombie_wave",
        default: 1,
        note: "rogue-like: с какой волны уровня бывают ядовитые зомби",
    },
    RuleDef {
        key: "rogue_venom_zombie_percent",
        default: 100,
        note: "rogue-like: шанс фигуры волны выйти ядовитым зомби, %",
    },
    RuleDef {
        key: "rogue_reroll_step",
        default: 0,
        note: "rogue-like: на сколько золота дорожает каждое следующее обновление сундуков (до ухода из магазина; 0 — не дорожает)",
    },
    RuleDef {
        key: "rogue_drops_per_turn",
        default: 1,
        note: "rogue-like: сколько фигур можно выставить из запаса за ход",
    },
    RuleDef {
        key: "rogue_max_pieces",
        default: 16,
        note: "rogue-like: больше стольких ваших фигур на доске не бывает (из запаса не выставить)",
    },
    RuleDef {
        key: "rogue_first_level_hp_cut",
        default: 10,
        note: "rogue-like: на первом уровне у фигур волн здоровья меньше на столько единиц (не \
               меньше одной)",
    },
    RuleDef {
        key: "rogue_wizard_every",
        default: 2,
        note: "rogue-like: маг запускает метеорит раз в столько своих ходов",
    },
    RuleDef {
        key: "rogue_chest_pieces_price",
        default: 10,
        note: "rogue-like: цена сундука фигур в магазине",
    },
    RuleDef {
        key: "rogue_chest_choices",
        default: 3,
        note: "rogue-like: из скольких вещей в сундуке выбирают одну",
    },
    RuleDef {
        key: "rogue_hand_size",
        default: 3,
        note: "rogue-like: сколько карт помещается в руку",
    },
    RuleDef {
        key: "rogue_gold_sell_percent",
        default: 200,
        note: "rogue-like: золотая фигура продаётся за столько % от продажи обычной такой же",
    },
    RuleDef {
        key: "rogue_rare_card_percent",
        default: 30,
        note: "rogue-like: шанс редкой карты (зеркала) в сундуке карт, % от шанса обычной",
    },
    RuleDef {
        key: "rogue_chest_slots",
        default: 3,
        note: "rogue-like: сколько сундуков в магазине (разных видов)",
    },
    RuleDef {
        key: "rogue_big_chest_percent",
        default: 30,
        note: "rogue-like: шанс, что сундук в магазине большой, %",
    },
    RuleDef {
        key: "rogue_big_chest_choices",
        default: 5,
        note: "rogue-like: из скольких вещей выбирают в большом сундуке",
    },
    RuleDef {
        key: "rogue_big_chest_picks",
        default: 2,
        note: "rogue-like: сколько вещей берут из большого сундука",
    },
    RuleDef {
        key: "rogue_big_chest_price_percent",
        default: 200,
        note: "rogue-like: цена большого сундука, % от обычного такого же",
    },
    RuleDef {
        key: "rogue_crest_mult_add_percent",
        default: 400,
        note: "rogue-like, герб урона: прибавка к мульти, в сотых (400 — мульти +4)",
    },
    RuleDef {
        key: "rogue_crest_king_mult_percent",
        default: 150,
        note: "rogue-like, герб урона короля: каждый удар вашего короля умножает мульти, в сотых (150 — ×1.5)",
    },
    RuleDef {
        key: "rogue_crest_few_base",
        default: 16,
        note: "rogue-like, герб короля и шестнадцати: мульти × (это число − ваших фигур на доске), не меньше ×1",
    },
    RuleDef {
        key: "rogue_crest_pawn_mult_percent",
        default: 150,
        note: "rogue-like, герб пешки и умножения: каждый удар вашей пешки в этом ходу умножает мульти, в сотых (150 — ×1.5)",
    },
    RuleDef {
        key: "rogue_crest_knight_add_percent",
        default: 200,
        note: "rogue-like, герб коня и умножения: мульти всех ваших фигур + это за каждый удар ваших коней в этом ходу, в сотых (200 — +2)",
    },
    RuleDef {
        key: "rogue_crest_sword_mult_percent",
        default: 120,
        note: "rogue-like, герб меча и умножения: мульти × это за каждую вашу фигуру, ударившую в этом ходу раньше, в сотых (120 — ×1.2)",
    },
    RuleDef {
        key: "rogue_crest_first_add_percent",
        default: 1000,
        note: "rogue-like, герб пешки и единицы: мульти первой бьющей в этом ходу фигуры + это, в сотых (1000 — +10)",
    },
    RuleDef {
        key: "rogue_crest_queen_add_percent",
        default: 300,
        note: "rogue-like, герб ферзя и умножения: мульти + это за каждый удар вашего ферзя в этом ходу, в сотых (300 — +3)",
    },
    RuleDef {
        key: "rogue_crest_flame_mult_percent",
        default: 150,
        note: "rogue-like, герб огня и умножения: каждый взрыв вашей огненной фигуры в этом раунде умножает мульти, в сотых (150 — ×1.5)",
    },
    RuleDef {
        key: "rogue_crest_snow_mult_percent",
        default: 300,
        note: "rogue-like, герб снежинки и умножения: мульти × это за каждую замороженную фигуру на доске, в сотых (300 — ×3)",
    },
    RuleDef {
        key: "rogue_crest_meteor_add_percent",
        default: 1000,
        note: "rogue-like, герб метеорита: мульти первого метеорита в этом ходу + это, в сотых (1000 — +10)",
    },
    RuleDef {
        key: "rogue_crest_jump_add_percent",
        default: 300,
        note: "rogue-like, герб теневого огня и черепа: мульти + это за каждую фигуру, через которую прошли ваши теневые в этом ходу, в сотых (300 — +3)",
    },
    RuleDef {
        key: "rogue_crest_bomb_mult_percent",
        default: 200,
        note: "rogue-like, герб падающей бомбы: мульти взрыва огненной фигуры × это, в сотых (200 — ×2)",
    },
    RuleDef {
        key: "rogue_crest_flame_sword_add_percent",
        default: 500,
        note: "rogue-like, герб огня и меча: мульти взрыва огненной фигуры + это, в сотых (500 — +5)",
    },
    RuleDef {
        key: "rogue_crest_axe_mult_percent",
        default: 200,
        note: "rogue-like, герб топора и меча: мульти варваров × это, в сотых (200 — ×2)",
    },
    RuleDef {
        key: "rogue_level_hp_percent",
        default: 200,
        note: "rogue-like: здоровье фигур волн на втором уровне, % от первого (200 — вдвое больше)",
    },
    RuleDef {
        key: "rogue_level_hp_step_percent",
        default: 50,
        note: "rogue-like: на сколько % от первого уровня здоровье волн растёт с каждым уровнем после второго (50: 200%, 250%, 300%…)",
    },
    RuleDef {
        key: "rogue_crest_bolt_add_percent",
        default: 500,
        note: "rogue-like, герб молнии и умножения: мульти молнии + это за каждую фигуру перед задетой в цепочке, в сотых (500 — +5)",
    },
    RuleDef {
        key: "rogue_crest_wind_add_percent",
        default: 800,
        note: "rogue-like, герб смерча и умножения: мульти воздушных фигур + это за каждый ход воздушных фигур в этом ходу, в сотых (800 — +8)",
    },
    RuleDef {
        key: "rogue_card_sell_price",
        default: 3,
        note: "rogue-like: за сколько продаётся карта из руки в магазине",
    },
    RuleDef {
        key: "rogue_building_mult_percent",
        default: 100,
        note: "rogue-like, здание вида фигур или силы в городе: мульти его фигур + это, раньше усилений, в сотых (100 — +1)",
    },
    RuleDef {
        key: "rogue_ice_hp",
        default: 10,
        note: "rogue-like: ледяные фигуры — с этим здоровьем сверху (и ставшие ледяными картой)",
    },
    RuleDef {
        key: "rogue_crest_bishop_add_percent",
        default: 400,
        note: "rogue-like, герб двух слонов: мульти + это до конца раунда каждый раз, когда ваш слон за ход ранит две разные фигуры (ударом, молнией, взрывом, метеоритом), в сотых (400 — +4)",
    },
    RuleDef {
        key: "rogue_crest_nine_every",
        default: 9,
        note: "rogue-like, герб девятки: каждая такая по счёту ваша атакующая фигура за забег получает мульти × `rogue_crest_nine_mult_percent`",
    },
    RuleDef {
        key: "rogue_crest_nine_mult_percent",
        default: 900,
        note: "rogue-like, герб девятки: во сколько раз мульти этой фигуры, в сотых (900 — ×9)",
    },
    RuleDef {
        key: "rogue_crest_card_add_percent",
        default: 200,
        note: "rogue-like, герб карты: мульти + это за каждую карту, сыгранную в этом забеге, в сотых (200 — +2)",
    },
    RuleDef {
        key: "rogue_crest_pawn_kills",
        default: 4,
        note: "rogue-like, герб пешки и синей карты: карта за каждые столько фигур, убитых вашими пешками за ход",
    },
    RuleDef {
        key: "rogue_crest_knife_coin_percent",
        default: 50,
        note: "rogue-like, герб ножа и монеты: мульти + столько процентов золота, вырученного за продажу фигур (50 — половина)",
    },
    RuleDef {
        key: "rogue_crest_dice_max",
        default: 10,
        note: "rogue-like, герб случайного числа: мульти + случайное от 1 до этого, новое каждый ход",
    },
    RuleDef {
        key: "rogue_crest_blank_add_percent",
        default: 1000,
        note: "rogue-like, пустой герб: мульти + это, если в запасе нет фигур, в сотых (1000 — +10)",
    },
    RuleDef {
        key: "rogue_crest_skull_add_percent",
        default: 1500,
        note: "rogue-like, герб черепа: мульти + это, в сотых (1500 — +15)",
    },
    RuleDef {
        key: "rogue_crest_skull_chance",
        default: 25,
        note: "rogue-like, герб черепа: шанс 1 к стольким, что он рассыплется в конце вашего хода",
    },
    RuleDef {
        key: "rogue_crest_bush_add_percent",
        default: 200,
        note: "rogue-like, герб куста: мульти + это за каждый удар ваших фигур без убийства подряд, в сотых (200 — +2)",
    },
    RuleDef {
        key: "rogue_crest_knight_gold",
        default: 10,
        note: "rogue-like, герб коня и монеты: столько золота, если первым в раунде убил ваш конь",
    },
    RuleDef {
        key: "rogue_crest_clock_base_percent",
        default: 200,
        note: "rogue-like, герб монеты и часов: мульти × (это + шаг × проданные фигуры), в сотых (200 — 2)",
    },
    RuleDef {
        key: "rogue_crest_clock_step_percent",
        default: 25,
        note: "rogue-like, герб монеты и часов: шаг за каждую проданную фигуру, в сотых (25 — 0.25)",
    },
    RuleDef {
        key: "rogue_building_attack",
        default: 5,
        note: "rogue-like, здание вида фигур или силы в городе: базовый урон его фигур + это, в единицах",
    },
    RuleDef {
        key: "rogue_kill_gold_bonus",
        default: 1,
        note: "rogue-like: сколько золота сверху даёт победа над любой фигурой волн (и над толстосумом)",
    },
    RuleDef {
        key: "rogue_chest_buildings_price",
        default: 10,
        note: "rogue-like: цена сундука зданий в городе (большой — rogue_big_chest_price_percent от неё)",
    },
    RuleDef {
        key: "rogue_building_price",
        default: 10,
        note: "rogue-like: цена здания — от неё считается продажа",
    },
    RuleDef {
        key: "rogue_building_sell_percent",
        default: 50,
        note: "rogue-like: здание продаётся за столько % своей цены (50 — за половину)",
    },
    RuleDef {
        key: "rogue_decree_price",
        default: 10,
        note: "rogue-like: цена королевского указа в главном здании",
    },
    RuleDef {
        key: "rogue_decree_own_percent",
        default: 50,
        note: "rogue-like: шанс, что указ даст зданию усиление его вида фигур или силы, %; иначе — общее (урон, кубик, череп…)",
    },
    RuleDef {
        key: "rogue_recruit_price",
        default: 5,
        note: "rogue-like: цена пешки в наборе новобранцев (сколько угодно, пешка — в запас)",
    },
    RuleDef {
        key: "rogue_chest_cards_price",
        default: 10,
        note: "rogue-like: цена сундука карт в магазине (большой — rogue_big_chest_price_percent от неё)",
    },
    RuleDef {
        key: "rogue_chest_pieces_weight",
        default: 50,
        note: "rogue-like: как часто в магазине сундук фигур (вес; 50 — вдвое реже остальных)",
    },
    RuleDef {
        key: "rogue_chest_buildings_weight",
        default: 100,
        note: "rogue-like: как часто в магазине сундук зданий (вес)",
    },
    RuleDef {
        key: "rogue_chest_cards_weight",
        default: 100,
        note: "rogue-like: как часто в магазине сундук карт (вес)",
    },
    RuleDef {
        key: "rogue_building_weight_pawn",
        default: 15,
        note: "rogue-like: редкость дома ополчения (пешки) в сундуке зданий — вес, чем больше, тем чаще",
    },
    RuleDef {
        key: "rogue_building_weight_knight",
        default: 5,
        note: "rogue-like: вес конюшни в сундуке зданий",
    },
    RuleDef {
        key: "rogue_building_weight_bishop",
        default: 5,
        note: "rogue-like: вес церкви (слоны) в сундуке зданий",
    },
    RuleDef {
        key: "rogue_building_weight_rook",
        default: 3,
        note: "rogue-like: вес башни (ладьи) в сундуке зданий",
    },
    RuleDef {
        key: "rogue_building_weight_queen",
        default: 1,
        note: "rogue-like: вес дворца ферзя в сундуке зданий (реже всех)",
    },
    RuleDef {
        key: "rogue_building_weight_king",
        default: 2,
        note: "rogue-like: вес дворца короля в сундуке зданий",
    },
    RuleDef {
        key: "rogue_building_weight_power",
        default: 3,
        note: "rogue-like: вес каждого здания особой силы (кузня, мельница, казна…) в сундуке зданий",
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

#[cfg_attr(not(test), allow(dead_code))] // ключ и пояснение — для списка `Баланс.txt`
pub struct ParamDef {
    pub param: Param,
    pub card: CardKind,
    pub key: &'static str,
    pub default: i32,
    pub note: &'static str,
}

const fn def(
    param: Param,
    card: CardKind,
    key: &'static str,
    default: i32,
    _limits: (i32, i32),
    note: &'static str,
) -> ParamDef {
    ParamDef {
        param,
        card,
        key,
        default,
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
/// `UNIT` — на экране и в списке баланса.
pub const UNIT: i32 = 300;

/// Одно очко на экране и в списке баланса (с 0.112.1, просьба пользователя: десятичная база —
/// атака 10, здоровье 30): `UNIT` / 10.
pub const POINT: i32 = UNIT / 10;

/// Здоровье или урон так, как их видит игрок: делённые на `POINT`, доля — вверх (раненая фигура
/// с 100 из 900 — «4 из 30»).
pub fn shown(value: i32) -> i32 {
    (value.max(0) + POINT - 1) / POINT
}

#[cfg(test)]
/// Атака и здоровье в списке баланса — в очках `POINT` (с 0.112.1; раньше — в `UNIT`).
pub fn to_file(value: i32) -> i32 {
    (value + POINT / 2).div_euclid(POINT)
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

    /// Список всех чисел для пользователя (`Баланс.txt`): правила, фигуры, карты, колоды.
    #[cfg(test)]
    pub fn text(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "Баланс «Боевых шахмат» — все числа игры.\n\
             С 0.133 файла balance.ini нет: числа зашиты в игру, а этот список — только для чтения.\n\
             Что поменять — скажите Claude. После «;» — что число значит. Атака и здоровье — в очках\n\
             (атака пешки — 10).\n\n",
        );
        out.push_str("[rules]\n");
        for (rule, value) in RULES.iter().zip(self.rules) {
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

#[cfg(test)]
/// Карты в порядке файла и конструктора: по типу (обычные, жертвы, победы), от дешёвых к
/// дорогим по стандартным ценам.
pub fn ordered_kinds() -> Vec<CardKind> {
    let mut kinds = CardKind::ALL.to_vec();
    kinds.sort_by_key(|&k| (k.family() as u8, default_cost(k)));
    kinds
}

#[cfg(test)]
fn family_name(family: Family) -> &'static str {
    match family {
        Family::Normal => "обычная карта",
        Family::Sacrifice => "карта жертвы",
        Family::Victory => "карта победы",
    }
}

#[cfg(test)]
fn payment(family: Family) -> &'static str {
    match family {
        Family::Normal => "мана",
        Family::Sacrifice => "очки жертвы",
        Family::Victory => "очки победы",
    }
}

// ---------- Общий доступ ----------

static DEFAULT: OnceLock<Balance> = OnceLock::new();

/// Баланс игры — стандартный, из таблиц этого файла (с 0.133 файла `balance.ini` нет).
pub fn balance() -> &'static Balance {
    DEFAULT.get_or_init(Balance::default)
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
    use crate::game::MAX_DECK;

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

    /// `Баланс.txt` в корне проекта — список всех чисел для пользователя (с 0.133, вместо
    /// `balance.ini`). После правки чисел: `cargo test --release refresh_balance_list -- --ignored`.
    #[test]
    #[ignore]
    fn refresh_balance_list() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Баланс.txt");
        std::fs::write(path, Balance::default().text()).expect("Баланс.txt");
    }

    #[test]
    fn balance_list_is_up_to_date() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Баланс.txt");
        let file = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            file == Balance::default().text(),
            "Баланс.txt устарел: cargo test --release refresh_balance_list -- --ignored"
        );
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
