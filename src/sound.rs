//! Звуки lichess (стандартный набор), вшитые в программу.

use std::io::Cursor;
use std::sync::mpsc::{self, Sender};

#[derive(Clone, Copy, Debug)]
pub enum Sound {
    /// Обычный ход.
    Move,
    /// Удар, после которого цель выжила.
    Hit,
    /// Цель убита.
    Kill,
    /// Начало и конец партии.
    Notify,
}

impl Sound {
    fn data(self) -> &'static [u8] {
        match self {
            Sound::Move => include_bytes!("../assets/sound/standard/Move.mp3"),
            Sound::Hit => include_bytes!("../assets/sound/standard/Berserk.mp3"),
            Sound::Kill => include_bytes!("../assets/sound/standard/Capture.mp3"),
            Sound::Notify => include_bytes!("../assets/sound/standard/GenericNotify.mp3"),
        }
    }
}

/// Проигрыватель. Аудиоустройство живёт в отдельном потоке, поэтому окно не
/// ждёт звука, а если устройства нет — игра просто остаётся без звука.
pub struct Sfx {
    queue: Option<Sender<(Sound, f32)>>,
    pub enabled: bool,
    /// Громкость в процентах.
    pub volume: u8,
}

impl Sfx {
    pub fn new() -> Sfx {
        let (queue, sounds) = mpsc::channel::<(Sound, f32)>();
        let player = std::thread::Builder::new()
            .name("звук".into())
            .spawn(move || {
                let Ok((_stream, output)) = rodio::OutputStream::try_default() else {
                    return;
                };
                for (sound, gain) in sounds {
                    if let Ok(source) = rodio::Decoder::new(Cursor::new(sound.data())) {
                        let source = rodio::Source::amplify(source, gain);
                        let _ = output.play_raw(rodio::Source::convert_samples(source));
                    }
                }
            });
        Sfx {
            queue: player.ok().map(|_| queue),
            enabled: true,
            volume: 100,
        }
    }

    pub fn play(&self, sound: Sound) {
        if self.enabled
            && self.volume > 0
            && let Some(queue) = &self.queue
        {
            // Слух воспринимает громкость логарифмически: квадрат делает ползунок
            // равномерным на слух (50% — примерно вдвое тише).
            let level = f32::from(self.volume) / 100.0;
            let _ = queue.send((sound, level * level));
        }
    }
}
