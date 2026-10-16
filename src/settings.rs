//! Настройки игры: громкость, частота кадров, счётчик FPS, колода, сила компьютера.
//! Хранятся в `%APPDATA%\BattleChess\settings.ini` строками «ключ=значение».

use std::path::PathBuf;

/// Ограничение частоты кадров.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameRate {
    /// Кадры синхронизированы с монитором: 144 кадра на мониторе 144 Гц.
    Vsync,
    /// Без синхронизации, не чаще заданного числа кадров в секунду.
    Cap(u32),
    /// Без синхронизации и без ограничения.
    Unlimited,
}

impl FrameRate {
    pub const ALL: [FrameRate; 4] = [
        FrameRate::Vsync,
        FrameRate::Cap(240),
        FrameRate::Cap(480),
        FrameRate::Unlimited,
    ];

    pub fn label(self) -> String {
        match self {
            FrameRate::Vsync => "как у монитора".to_owned(),
            FrameRate::Cap(fps) => fps.to_string(),
            FrameRate::Unlimited => "без ограничения".to_owned(),
        }
    }

    pub fn vsync(self) -> bool {
        self == FrameRate::Vsync
    }

    fn key(self) -> String {
        match self {
            FrameRate::Vsync => "vsync".to_owned(),
            FrameRate::Cap(fps) => fps.to_string(),
            FrameRate::Unlimited => "unlimited".to_owned(),
        }
    }

    fn parse(text: &str) -> Option<FrameRate> {
        match text {
            "vsync" => Some(FrameRate::Vsync),
            "unlimited" => Some(FrameRate::Unlimited),
            fps => fps
                .parse()
                .ok()
                .filter(|&fps| fps >= 30)
                .map(FrameRate::Cap),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Громкость звука в процентах.
    pub volume: u8,
    pub sound: bool,
    pub frame_rate: FrameRate,
    pub show_fps: bool,
    /// Выбранная в меню колода (номер в `DECKS` в app/menu.rs).
    pub deck: u8,
    /// Компьютер с усиленными фигурами: лава в пешках, электричество в слонах.
    pub boosted: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: 80,
            sound: true,
            frame_rate: FrameRate::Vsync,
            show_fps: false,
            deck: 0,
            boosted: false,
        }
    }
}

impl Settings {
    /// Настройки с диска; чего нет или не читается — по умолчанию.
    pub fn load() -> Settings {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map_or_else(Settings::default, |text| Settings::parse(&text))
    }

    /// Сохраняет настройки; если не вышло, игра просто работает дальше.
    pub fn save(&self) {
        let Some(path) = path() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, self.to_text());
    }

    /// Разбирает строки «ключ=значение»; незнакомое и испорченное пропускается.
    fn parse(text: &str) -> Settings {
        let mut settings = Settings::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "volume" => {
                    if let Ok(volume) = value.parse::<u32>() {
                        settings.volume = volume.min(100) as u8;
                    }
                }
                "sound" => settings.sound = value != "off",
                "frame_rate" => {
                    if let Some(rate) = FrameRate::parse(value) {
                        settings.frame_rate = rate;
                    }
                }
                "show_fps" => settings.show_fps = value == "on",
                "deck" => settings.deck = value.parse().unwrap_or(0),
                "boosted" => settings.boosted = value == "on",
                _ => {}
            }
        }
        settings
    }

    fn to_text(&self) -> String {
        let on_off = |on: bool| if on { "on" } else { "off" };
        format!(
            "volume={}\nsound={}\nframe_rate={}\nshow_fps={}\ndeck={}\nboosted={}\n",
            self.volume,
            on_off(self.sound),
            self.frame_rate.key(),
            on_off(self.show_fps),
            self.deck,
            on_off(self.boosted),
        )
    }
}
fn path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("settings.ini"))
}

/// Папка настроек и баланса игры: на Windows — `%APPDATA%\BattleChess`, на macOS —
/// `~/Library/Application Support/BattleChess`, на Linux — `~/.config/BattleChess`.
pub fn data_dir() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")))?
    };
    Some(base.join("BattleChess"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_survive_a_round_trip() {
        for frame_rate in FrameRate::ALL {
            let settings = Settings {
                volume: 35,
                sound: false,
                frame_rate,
                show_fps: true,
                deck: 3,
                boosted: true,
            };
            assert_eq!(Settings::parse(&settings.to_text()), settings);
        }
    }

    #[test]
    fn broken_settings_fall_back_to_defaults() {
        let settings = Settings::parse("volume=900\nframe_rate=5\nмусор\nshow_fps=on");
        assert_eq!(settings.volume, 100);
        assert_eq!(settings.frame_rate, FrameRate::Vsync);
        assert!(settings.show_fps);
        assert_eq!(Settings::parse(""), Settings::default());
    }
}
