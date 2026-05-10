use ratatui::style::Color;
use std::time::{Duration, Instant};
use tachyonfx::{fx, Duration as FxDuration, Effect};

pub struct FxState {
    pub select_effect: Option<Effect>,
    pub deselect_effect: Option<Effect>,
    pub last_tick: Instant,
}

impl FxState {
    pub fn new() -> Self {
        Self {
            select_effect: None,
            deselect_effect: None,
            last_tick: Instant::now(),
        }
    }

    pub fn trigger_selection(&mut self) {
        self.select_effect = Some(fx::fade_from_fg(
            Color::Yellow,
            FxDuration::from_millis(300),
        ));
        self.deselect_effect = Some(fx::fade_to_fg(
            Color::DarkGray,
            FxDuration::from_millis(200),
        ));
    }

    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        elapsed
    }
}
