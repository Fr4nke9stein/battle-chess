//! Меню при запуске: «Начать играть», «Выбрать колоду», «Коллекция карт», «Выйти» —
//! тёмно-синие плашки с кромкой, как панели Into the Breach. Выбор колоды: пять стопок
//! рубашкой вверх разных цветов (полная колода белых с повёрнутым оттенком); выбранная
//! поднята и обведена. Выбранная колода (Settings::deck) определяет состав карт обеих
//! сторон в новой партии. Коллекция: все карты игры по 12 на странице, стрелки по бокам листают.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind,
    Vec2, pos2, vec2,
};

use super::table::{paint_card_text, paint_title};
use super::{BattleChessApp, UV, paint_backdrop};
use crate::assets::{CARD, CardPic, DECK_SKINS, Pointer, SHADOW_SHIFT};
use crate::balance::{self, balance, plural};
use crate::game::{CardKind, Color, collection};
use crate::game::{DECK_COUNT, DECK_NAMES, decode_deck_code, encode_deck_code};
use crate::theme;

/// Названия колод в порядке `DECK_SKINS` (оттенки — в assets/cards.rs).
pub const DECKS: [&str; DECK_SKINS] = DECK_NAMES;

/// Поле меню в пикселях арта: оно целиком вписывается в окно.
const FIELD: Vec2 = vec2(360.0, 240.0);
/// Плашка кнопки, пиксели арта.
const BUTTON: Vec2 = vec2(132.0, 22.0);
/// Колоды в ряд: шаг и насколько поднимается колода под курсором и выбранная.
const DECK_STEP: f32 = 68.0;
const HOVER_LIFT: f32 = 3.0;
const PICK_LIFT: f32 = 7.0;
/// Коллекция: карт на странице, в ряду, шаг сетки и левый верхний угол первой карты; плашки
/// стрелок по бокам; карта под курсором — в `COLLECTION_ZOOM` раза крупнее.
const PAGE: usize = 12;
const COLUMNS: usize = 6;
const CARD_STEP: Vec2 = vec2(54.0, 76.0);
const GRID_MIN: Vec2 = vec2(20.0, 30.0);
const ARROW: Vec2 = vec2(13.0, 30.0);
const COLLECTION_ZOOM: usize = 2;

/// Какой экран показан.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    /// Мультиплеер: адрес сервера, подключение, ожидание соперника.
    Lobby,
    Decks,
    Collection,
    Game,
}

/// Что нажали в меню.
#[allow(dead_code)] // колоды и коллекция скрыты (0.40)
enum Choice {
    Play,
    /// Экран мультиплеера, подключиться, отменить подключение.
    Multiplayer,
    Connect,
    Disconnect,
    /// Компьютер с обычными или усиленными фигурами.
    Boost,
    /// Взятие с перемещением или без.
    Advance,
    Decks,
    Collection,
    Balance,
    Quit,
    Back,
    Deck(usize),
    /// Листать коллекцию: −1 — назад, +1 — вперёд.
    Page(isize),
}

/// Поле меню на экране: левый верхний угол и точек экрана на пиксель арта.
struct Field {
    origin: Pos2,
    scale: f32,
}

impl Field {
    fn at(&self, art: Vec2) -> Pos2 {
        self.origin + art * self.scale
    }

    fn rect(&self, min: Vec2, size: Vec2) -> Rect {
        Rect::from_min_size(self.at(min), size * self.scale)
    }
}

impl BattleChessApp {
    /// Меню и выбор колоды — вместо доски и панелей.
    pub(super) fn menu(&mut self, ctx: &egui::Context) {
        let mut choice = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::PAGE))
            .show(ctx, |ui| {
                let avail = ui.max_rect();
                paint_backdrop(ui.painter(), ui.clip_rect());
                let ppp = ctx.pixels_per_point();
                // Пиксель арта — целое число физических пикселей, как у доски.
                let physical = ((avail.width() / FIELD.x).min(avail.height() / FIELD.y) * ppp)
                    .floor()
                    .max(1.0);
                let scale = physical / ppp;
                let corner = avail.center() - FIELD * scale / 2.0;
                let origin = pos2(
                    (corner.x * ppp).round() / ppp,
                    (corner.y * ppp).round() / ppp,
                );
                let field = Field { origin, scale };
                let response = ui.allocate_rect(avail, Sense::click());
                let pointer = response.hover_pos();
                let painter = &ui.painter().clone();
                self.pointer = Pointer::Default;
                self.cursor_scale = physical as u32;
                choice = match self.screen {
                    Screen::Lobby => self.lobby_screen(ui, painter, &field, pointer),
                    Screen::Decks => self.deck_screen(ctx, painter, &field, pointer, physical),
                    Screen::Collection => {
                        self.collection_screen(ctx, painter, &field, pointer, physical)
                    }
                    _ => self.title_screen(painter, &field, pointer),
                }
                .filter(|_| response.clicked());
            });
        // Коллекцию листают и стрелками клавиатуры, Esc — назад в меню.
        if self.screen == Screen::Collection && choice.is_none() {
            choice = ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowLeft) {
                    Some(Choice::Page(-1))
                } else if i.key_pressed(egui::Key::ArrowRight) {
                    Some(Choice::Page(1))
                } else if i.key_pressed(egui::Key::Escape) {
                    Some(Choice::Back)
                } else {
                    None
                }
            });
        }
        // Выбор колоды: Ctrl+V вставляет код колоды в выбранную, Ctrl+C копирует её код.
        if self.screen == Screen::Decks && choice.is_none() {
            let (paste, copy) = ctx.input(|i| {
                let paste = i.events.iter().find_map(|e| match e {
                    egui::Event::Paste(text) => Some(text.clone()),
                    _ => None,
                });
                (
                    paste,
                    i.events.iter().any(|e| matches!(e, egui::Event::Copy)),
                )
            });
            if let Some(text) = paste {
                self.import_deck_code(&text);
            } else if copy {
                let deck = self.settings.deck as usize % DECK_COUNT;
                ctx.copy_text(encode_deck_code(balance().deck(deck)));
                self.deck_message = Some((
                    format!(
                        "Код колоды «{}» скопирован в буфер обмена",
                        DECK_NAMES[deck]
                    ),
                    true,
                ));
            }
        }
        match choice {
            Some(Choice::Play) => {
                self.new_game(self.draft);
                self.screen = Screen::Game;
            }
            Some(Choice::Multiplayer) => {
                self.net_status.clear();
                self.screen = Screen::Lobby;
            }
            Some(Choice::Connect) => self.connect(ctx),
            Some(Choice::Disconnect) => {
                self.leave_network();
                self.net_status = "Подключение отменено".to_owned();
            }
            Some(Choice::Decks) => self.screen = Screen::Decks,
            Some(Choice::Boost) => self.settings.boosted = !self.settings.boosted,
            Some(Choice::Advance) => self.settings.advance = !self.settings.advance,
            Some(Choice::Balance) => {
                if let Err(e) = crate::editor::open(ctx) {
                    self.balance_warnings.insert(0, e);
                    balance::open_in_editor();
                }
            }
            Some(Choice::Collection) => {
                self.reload_balance();
                self.collection_page = 0;
                self.screen = Screen::Collection;
            }
            Some(Choice::Page(step)) => {
                let last = collection().len().div_ceil(PAGE).max(1) - 1;
                self.collection_page = self.collection_page.saturating_add_signed(step).min(last);
            }
            Some(Choice::Back) => {
                if self.screen == Screen::Lobby {
                    self.leave_network();
                }
                self.reload_balance();
                self.deck_message = None;
                self.screen = Screen::Menu;
            }
            Some(Choice::Deck(i)) => {
                self.settings.deck = i as u8;
                self.deck_message = None;
            }
            Some(Choice::Quit) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            None => {}
        }
    }

    /// Мультиплеер: поле адреса сервера, «Подключиться» (Enter) или «Отменить», «Назад» и
    /// строка состояния. Партию начнёт сервер, когда подключится второй игрок.
    fn lobby_screen(
        &mut self,
        ui: &mut egui::Ui,
        painter: &Painter,
        field: &Field,
        pointer: Option<Pos2>,
    ) -> Option<Choice> {
        heading(painter, field, 34.0, "Мультиплеер", 22.0);
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 58.0)),
            Align2::CENTER_CENTER,
            "Адрес сервера (IP:порт)",
            FontId::proportional(7.0 * field.scale),
            theme::TEXT_DIM,
        );
        let connecting = self.net.is_some();
        let edit = ui.put(
            field.rect(vec2(80.0, 66.0), vec2(200.0, 18.0)),
            egui::TextEdit::singleline(&mut self.net_address)
                .hint_text("например, 203.0.113.5:47900")
                .font(FontId::proportional(8.0 * field.scale))
                .interactive(!connecting)
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center),
        );
        let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let mut picked = None;
        let x = (FIELD.x - BUTTON.x) / 2.0;
        // Правила сетевой партии выбирает тот, кто зашёл первым (играет белыми).
        let rule = advance_label(self.settings.advance);
        if self.plate(
            painter,
            field,
            vec2(x, 92.0),
            BUTTON,
            rule,
            pointer,
            !connecting,
        ) {
            picked = Some(Choice::Advance);
        }
        let (label, action) = if connecting {
            ("Отменить", Choice::Disconnect)
        } else {
            ("Подключиться", Choice::Connect)
        };
        if self.button(painter, field, vec2(x, 120.0), label, pointer) {
            picked = Some(action);
        }
        if self.button(painter, field, vec2(x, 146.0), "Назад", pointer) {
            picked = Some(Choice::Back);
        }
        if entered && !connecting {
            // Enter в поле — сразу подключиться, без щелчка.
            self.connect(&ui.ctx().clone());
        }
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 182.0)),
            Align2::CENTER_CENTER,
            &self.net_status,
            FontId::proportional(7.0 * field.scale),
            theme::TEXT_BRIGHT,
        );
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 200.0)),
            Align2::CENTER_CENTER,
            "Сервер — tools/mp_server.py на вашем VPS; оба игрока вводят один адрес.",
            FontId::proportional(5.5 * field.scale),
            theme::TEXT_DIM,
        );
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 210.0)),
            Align2::CENTER_CENTER,
            "Первый подключившийся играет белыми и выбирает правила взятия.",
            FontId::proportional(5.5 * field.scale),
            theme::TEXT_DIM,
        );
        picked
    }

    /// Заглавный экран: название, кнопки и предупреждения о `balance.ini`, если они есть.
    fn title_screen(
        &mut self,
        painter: &Painter,
        field: &Field,
        pointer: Option<Pos2>,
    ) -> Option<Choice> {
        heading(painter, field, 34.0, "Боевые шахматы", 22.0);
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 54.0)),
            Align2::CENTER_CENTER,
            "шахматы и пиксель-арт",
            FontId::proportional(6.0 * field.scale),
            theme::TEXT_DIM,
        );
        let strength = if self.settings.boosted {
            "Компьютер: усиленный"
        } else {
            "Компьютер: обычный"
        };
        let buttons = [
            ("Начать играть", Choice::Play),
            ("Мультиплеер", Choice::Multiplayer),
            (strength, Choice::Boost),
            (advance_label(self.settings.advance), Choice::Advance),
            ("Править баланс", Choice::Balance),
            ("Выйти", Choice::Quit),
        ];
        // С 0.40 карт в игре нет: выбор колоды и коллекция скрыты.
        let mut picked = None;
        for (k, (label, choice)) in buttons.into_iter().enumerate() {
            let min = vec2((FIELD.x - BUTTON.x) / 2.0, 62.0 + k as f32 * 23.0);
            if self.button(painter, field, min, label, pointer) {
                picked = Some(choice);
            }
        }
        // Что не так в balance.ini: первые две строки и сколько ещё.
        let lines: Vec<String> = self
            .balance_warnings
            .iter()
            .take(2)
            .map(|w| format!("balance.ini, {w}"))
            .chain(
                (self.balance_warnings.len() > 2)
                    .then(|| format!("…и ещё предупреждений: {}", self.balance_warnings.len() - 2)),
            )
            .collect();
        for (k, line) in lines.iter().enumerate() {
            painter.text(
                field.at(vec2(FIELD.x / 2.0, 228.0 + k as f32 * 7.0)),
                Align2::CENTER_CENTER,
                line,
                FontId::proportional(5.5 * field.scale),
                theme::ORANGE,
            );
        }
        picked
    }

    /// Вставленный код колоды становится составом выбранной колоды: пишется в `balance.ini`
    /// (прежний файл остаётся как `balance.ini.bak`), результат — в строке под колодами.
    fn import_deck_code(&mut self, text: &str) {
        let deck = self.settings.deck as usize % DECK_COUNT;
        let result = decode_deck_code(text)
            .and_then(|recipe| balance::set_deck_in_file(deck, &recipe).map(|()| recipe));
        self.deck_message = Some(match result {
            Ok(recipe) => {
                self.reload_balance();
                let total: u32 = recipe.iter().map(|&(_, n)| u32::from(n)).sum();
                (
                    format!(
                        "Колода «{}» заменена: {total} {} из кода (прежний файл — balance.ini.bak)",
                        DECK_NAMES[deck],
                        plural(total as i32, ["карта", "карты", "карт"])
                    ),
                    true,
                )
            }
            Err(e) => (format!("Код не принят: {e}"), false),
        });
    }

    /// Выбор колоды: пять стопок в ряд, под каждой — название и число карт; внизу кнопка
    /// «Назад», подсказка про Ctrl+V и строка о последней вставке кода.
    fn deck_screen(
        &mut self,
        ctx: &egui::Context,
        painter: &Painter,
        field: &Field,
        pointer: Option<Pos2>,
        physical: f32,
    ) -> Option<Choice> {
        heading(painter, field, 30.0, "Выбор колоды", 16.0);
        let textures = self.art.textures(ctx, physical as u32);
        let chosen = self.settings.deck as usize % DECK_SKINS;
        let mut picked = None;
        let left = (FIELD.x - DECK_STEP * (DECK_SKINS as f32 - 1.0)) / 2.0;
        for (i, name) in DECKS.iter().enumerate() {
            let sprite = textures.cards.skin(i);
            let center = vec2(left + DECK_STEP * i as f32, 112.0);
            let base = field.rect(center - sprite.size / 2.0, sprite.size);
            let hovered = pointer.is_some_and(|p| base.expand(2.0 * field.scale).contains(p));
            let lift = if i == chosen {
                PICK_LIFT
            } else if hovered {
                HOVER_LIFT
            } else {
                0.0
            };
            let body = base.translate(vec2(0.0, -lift * field.scale));
            let shadow = Rect::from_min_size(body.min, (sprite.size + SHADOW_SHIFT) * field.scale);
            painter.image(sprite.shadow.id(), shadow, UV, Color32::WHITE);
            painter.image(sprite.body.id(), body, UV, Color32::WHITE);
            if i == chosen {
                // Выбранная — в бледно-зелёной кромке, как карта, которую можно сыграть.
                let frame = body.expand(2.0 * field.scale);
                painter.rect_stroke(
                    frame,
                    CornerRadius::ZERO,
                    Stroke::new(field.scale, Color32::from_rgb(0xd8, 0xf0, 0xa0)),
                    StrokeKind::Inside,
                );
            }
            let color = if i == chosen {
                theme::TEXT_BRIGHT
            } else {
                theme::TEXT_DIM
            };
            painter.text(
                field.at(vec2(center.x, 166.0)),
                Align2::CENTER_CENTER,
                *name,
                FontId::new(7.0 * field.scale, theme::bold()),
                color,
            );
            let total: u32 = balance().deck(i).iter().map(|&(_, n)| u32::from(n)).sum();
            painter.text(
                field.at(vec2(center.x, 175.0)),
                Align2::CENTER_CENTER,
                format!(
                    "{total} {}",
                    plural(total as i32, ["карта", "карты", "карт"])
                ),
                FontId::proportional(5.5 * field.scale),
                theme::TEXT_DIM,
            );
            if hovered {
                self.pointer = Pointer::Hover;
                picked = Some(Choice::Deck(i));
            }
        }
        if let Some((text, ok)) = &self.deck_message {
            painter.text(
                field.at(vec2(FIELD.x / 2.0, 190.0)),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(5.5 * field.scale),
                if *ok { theme::GREEN } else { theme::ORANGE },
            );
        }
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 231.0)),
            Align2::CENTER_CENTER,
            "Ctrl+V — вставить код колоды в выбранную · Ctrl+C — скопировать её код",
            FontId::proportional(5.5 * field.scale),
            theme::TEXT_DIM,
        );
        let back = vec2((FIELD.x - BUTTON.x) / 2.0, 200.0);
        if self.button(painter, field, back, "Назад", pointer) {
            picked = Some(Choice::Back);
        }
        picked
    }

    /// Коллекция карт: страница из двух рядов по шесть, по бокам стрелки, внизу номер
    /// страницы и «Назад». Карта под курсором показана крупнее.
    fn collection_screen(
        &mut self,
        ctx: &egui::Context,
        painter: &Painter,
        field: &Field,
        pointer: Option<Pos2>,
        physical: f32,
    ) -> Option<Choice> {
        heading(painter, field, 15.0, "Коллекция карт", 14.0);
        let kinds = collection();
        let pages = kinds.len().div_ceil(PAGE).max(1);
        let page = self.collection_page.min(pages - 1);
        let shown: Vec<CardKind> = kinds.iter().copied().skip(page * PAGE).take(PAGE).collect();
        let face = |kind| CardPic::Front {
            kind,
            color: Color::White,
            selected: false,
        };
        let mut hovered = None;
        let textures = self.art.textures(ctx, physical as u32);
        for (k, &kind) in shown.iter().enumerate() {
            let cell = vec2((k % COLUMNS) as f32, (k / COLUMNS) as f32);
            let center = GRID_MIN + CARD / 2.0 + cell * CARD_STEP;
            let sprite = self.art.card(ctx, &textures, face(kind), 0.0);
            let body = field.rect(center - sprite.size / 2.0, sprite.size);
            let shadow = Rect::from_min_size(body.min, (sprite.size + SHADOW_SHIFT) * field.scale);
            painter.image(sprite.shadow.id(), shadow, UV, Color32::WHITE);
            painter.image(sprite.body.id(), body, UV, Color32::WHITE);
            paint_card_text(painter, field.at(center), field.scale, 0.0, kind, 1.0);
            if pointer.is_some_and(|p| body.contains(p)) {
                hovered = Some((kind, center, k / COLUMNS));
            }
        }
        let mut picked = None;
        let middle = GRID_MIN.y + (CARD_STEP.y + CARD.y) / 2.0 - ARROW.y / 2.0;
        let arrows = [
            (vec2(3.0, middle), -1, page > 0),
            (vec2(FIELD.x - 3.0 - ARROW.x, middle), 1, page + 1 < pages),
        ];
        for (min, step, enabled) in arrows {
            if self.arrow(painter, field, min, step, enabled, pointer) {
                picked = Some(Choice::Page(step));
            }
        }
        painter.text(
            field.at(vec2(FIELD.x / 2.0, 187.0)),
            Align2::CENTER_CENTER,
            format!("{} / {pages}", page + 1),
            FontId::new(7.0 * field.scale, theme::bold()),
            theme::TEXT_DIM,
        );
        let back = vec2((FIELD.x - BUTTON.x) / 2.0, 200.0);
        if self.button(painter, field, back, "Назад", pointer) {
            picked = Some(Choice::Back);
        }
        // Карта под курсором — крупнее, поверх соседей, целиком в поле меню.
        if let Some((kind, center, row)) = hovered {
            let zoom = COLLECTION_ZOOM as f32;
            let size = CARD * zoom;
            let center = center.clamp(size / 2.0, FIELD - size / 2.0);
            let sprite = self
                .art
                .card_zoomed(ctx, &textures, face(kind), COLLECTION_ZOOM);
            let body = field.rect(center - size / 2.0, size);
            let shadow = Rect::from_min_size(
                body.min + SHADOW_SHIFT * field.scale,
                (size + SHADOW_SHIFT * zoom) * field.scale,
            );
            painter.image(sprite.shadow.id(), shadow, UV, Color32::WHITE);
            painter.image(sprite.body.id(), body, UV, Color32::WHITE);
            paint_card_text(
                painter,
                field.at(center),
                field.scale * zoom,
                0.0,
                kind,
                1.0,
            );
            if kind.description().is_empty() {
                // У карт без текста название — на табличке у края крупной карты.
                let (edge, anchor) = if row == 0 {
                    (body.center_bottom() + vec2(0.0, 4.0), Align2::CENTER_TOP)
                } else {
                    (body.center_top() - vec2(0.0, 4.0), Align2::CENTER_BOTTOM)
                };
                paint_title(painter, edge, anchor, kind.title());
            }
        }
        picked
    }

    /// Плашка стрелки, листающей коллекцию (`step` < 0 — влево); `true` — курсор над ней и
    /// листать есть куда. Треугольник — ступеньками в пиксель арта.
    fn arrow(
        &mut self,
        painter: &Painter,
        field: &Field,
        min: Vec2,
        step: isize,
        enabled: bool,
        pointer: Option<Pos2>,
    ) -> bool {
        let hovered = self.plate(painter, field, min, ARROW, "", pointer, enabled);
        let color = if !enabled {
            Color32::from_rgb(0x49, 0x5c, 0x79)
        } else if hovered {
            Color32::WHITE
        } else {
            theme::TEXT_BRIGHT
        };
        // Ряды треугольника шириной 1…5…1: остриё смотрит туда, куда листает.
        let half = 4.0;
        let base = min + vec2(ARROW.x / 2.0, ARROW.y / 2.0 - half);
        for row in 0..=8 {
            let width = half + 1.0 - (row as f32 - half).abs();
            let left = if step < 0 {
                base.x + 3.0 - width
            } else {
                base.x - 2.0
            };
            let rect = field.rect(vec2(left, base.y + row as f32), vec2(width, 1.0));
            painter.rect_filled(rect, CornerRadius::ZERO, color);
        }
        hovered
    }

    /// Плашка кнопки; `true` — курсор над ней.
    fn button(
        &mut self,
        painter: &Painter,
        field: &Field,
        min: Vec2,
        label: &str,
        pointer: Option<Pos2>,
    ) -> bool {
        self.plate(painter, field, min, BUTTON, label, pointer, true)
    }

    /// Плашка размера `size` с надписью; неактивная не подсвечивается. `true` — курсор над
    /// активной плашкой.
    #[allow(clippy::too_many_arguments)]
    fn plate(
        &mut self,
        painter: &Painter,
        field: &Field,
        min: Vec2,
        size: Vec2,
        label: &str,
        pointer: Option<Pos2>,
        enabled: bool,
    ) -> bool {
        let rect = field.rect(min, size);
        let hovered = enabled && pointer.is_some_and(|p| rect.contains(p));
        let s = field.scale;
        // Тень, тёмный контур, заливка и светлая кромка внутри — как кнопка «Конец хода».
        let shadow = rect.translate(vec2(2.0, 3.0) * s);
        painter.rect_filled(shadow, CornerRadius::ZERO, Color32::from_black_alpha(77));
        painter.rect_filled(
            rect,
            CornerRadius::ZERO,
            Color32::from_rgb(0x0d, 0x0f, 0x17),
        );
        let fill = if hovered {
            Color32::from_rgb(0x2a, 0x35, 0x4d)
        } else {
            Color32::from_rgb(0x18, 0x1c, 0x29)
        };
        let inner = rect.shrink(s);
        painter.rect_filled(inner, CornerRadius::ZERO, fill);
        let edge = if hovered {
            Color32::from_rgb(0x9f, 0xb3, 0xd1)
        } else if enabled {
            Color32::from_rgb(0x49, 0x5c, 0x79)
        } else {
            Color32::from_rgb(0x2a, 0x33, 0x45)
        };
        painter.rect_stroke(
            inner,
            CornerRadius::ZERO,
            Stroke::new(s, edge),
            StrokeKind::Inside,
        );
        let text = if hovered {
            Color32::WHITE
        } else {
            theme::TEXT_BRIGHT
        };
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::new(9.0 * s, theme::bold()),
            text,
        );
        if hovered {
            self.pointer = Pointer::Hover;
        }
        hovered
    }
}

/// Подпись переключателя «перемещение при взятии».
fn advance_label(on: bool) -> &'static str {
    if on {
        "Взятие: с перемещением"
    } else {
        "Взятие: без перемещения"
    }
}

/// Заголовок с тёмной тенью на пиксель вниз-вправо.
fn heading(painter: &Painter, field: &Field, y: f32, text: &str, size: f32) {
    let at = field.at(vec2(FIELD.x / 2.0, y));
    let font = FontId::new(size * field.scale, theme::bold());
    let shadow = at + vec2(1.0, 1.0) * field.scale;
    painter.text(
        shadow,
        Align2::CENTER_CENTER,
        text,
        font.clone(),
        Color32::from_rgb(0x0d, 0x0f, 0x17),
    );
    painter.text(at, Align2::CENTER_CENTER, text, font, theme::TEXT_BRIGHT);
}
