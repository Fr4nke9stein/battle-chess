//! Цвета и шрифты в духе тёмной темы lichess.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Theme, Visuals, vec2,
};

pub const PAGE: Color32 = Color32::from_rgb(0x16, 0x15, 0x12);
pub const PANEL: Color32 = Color32::from_rgb(0x26, 0x24, 0x21);
pub const PANEL_ALT: Color32 = Color32::from_rgb(0x30, 0x2e, 0x2c);
pub const BORDER: Color32 = Color32::from_rgb(0x40, 0x3e, 0x3b);
pub const TEXT: Color32 = Color32::from_rgb(0xba, 0xba, 0xba);
pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(0xe8, 0xe6, 0xe3);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8c, 0x8a, 0x88);
pub const BLUE: Color32 = Color32::from_rgb(0x36, 0x92, 0xe7);
pub const GREEN: Color32 = Color32::from_rgb(0x7f, 0xa6, 0x50);
pub const RED: Color32 = Color32::from_rgb(0xe0, 0x4a, 0x38);
pub const ORANGE: Color32 = Color32::from_rgb(0xe6, 0x8a, 0x2e);
/// Ответный удар.
pub const SALMON: Color32 = Color32::from_rgb(0xff, 0x8a, 0x6e);

/// Цвет как в CSS: `rgba(r, g, b, a)`.
pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Полужирное начертание (Segoe UI Semibold, если он есть в системе) — для мелкого текста
/// на картах: на тёмном лице тонкий обычный шрифт мелким кеглем расплывается.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

/// Жирное начертание (Segoe UI Bold, если он есть в системе).
pub fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_theme(Theme::Dark);

    let mut visuals = Visuals::dark();
    visuals.panel_fill = PAGE;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(0x1e, 0x1d, 0x1a);
    visuals.faint_bg_color = PANEL_ALT;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = BLUE;
    visuals.selection.stroke = Stroke {
        width: 1.0,
        color: Color32::WHITE,
    };
    visuals.hyperlink_color = BLUE;
    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_stroke = Stroke {
        width: 1.0,
        color: BORDER,
    };
    for (state, fill) in [
        (&mut widgets.inactive, Color32::from_rgb(0x3a, 0x38, 0x35)),
        (&mut widgets.hovered, Color32::from_rgb(0x4a, 0x47, 0x43)),
        (&mut widgets.active, Color32::from_rgb(0x56, 0x53, 0x4e)),
        (&mut widgets.open, Color32::from_rgb(0x4a, 0x47, 0x43)),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.corner_radius = CornerRadius::same(4);
    }
    ctx.set_visuals_of(Theme::Dark, visuals);

    ctx.style_mut_of(Theme::Dark, |style| {
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 5.0);
        style.interaction.tooltip_delay = 0.25;
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.5));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(14.5));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(20.0, bold()));
    });
}

/// На Windows берём Segoe UI (ближе к шрифту lichess) и Segoe UI Symbol (⚔, ☠, стрелки), на
/// macOS — Arial и Apple Symbols. Если шрифтов нет, остаются встроенные шрифты egui.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let windows = std::env::var_os("WINDIR").map(|dir| PathBuf::from(dir).join("Fonts"));
    let mac = PathBuf::from("/System/Library/Fonts");
    // Первый найденный файл из списка: шрифт Windows, потом macOS.
    let candidates = |win: &str, mac_files: &[&str]| -> Vec<PathBuf> {
        windows
            .iter()
            .map(|dir| dir.join(win))
            .chain(mac_files.iter().map(|file| mac.join(file)))
            .collect()
    };
    let mut load = |name: &str, files: Vec<PathBuf>| -> bool {
        let Some(bytes) = files.iter().find_map(|file| std::fs::read(file).ok()) else {
            return false;
        };
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
        true
    };
    let regular: Vec<String> = [
        (
            "Segoe UI",
            candidates("segoeui.ttf", &["Supplemental/Arial.ttf"]),
        ),
        (
            "Segoe UI Symbol",
            candidates("seguisym.ttf", &["Apple Symbols.ttf"]),
        ),
    ]
    .into_iter()
    .filter_map(|(name, files)| load(name, files).then(|| name.to_owned()))
    .collect();
    let bold_mac = ["Supplemental/Arial Bold.ttf"];
    let has_bold = load("Segoe UI Bold", candidates("segoeuib.ttf", &bold_mac));
    let has_semibold = load("Segoe UI Semibold", candidates("seguisb.ttf", &bold_mac));

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    for (i, name) in regular.into_iter().enumerate() {
        proportional.insert(i, name);
    }
    let mut bold_family = Vec::new();
    if has_bold {
        bold_family.push("Segoe UI Bold".to_owned());
    }
    bold_family.extend(fonts.families[&FontFamily::Proportional].iter().cloned());
    fonts.families.insert(bold(), bold_family);
    let mut semibold_family = Vec::new();
    if has_semibold {
        semibold_family.push("Segoe UI Semibold".to_owned());
    }
    semibold_family.extend(fonts.families[&FontFamily::Proportional].iter().cloned());
    fonts.families.insert(semibold(), semibold_family);
    ctx.set_fonts(fonts);
}
