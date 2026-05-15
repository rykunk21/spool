use ratatui::{
    layout::{Offset, Rect},
    style::{Color, Style},
};
use std::time::{Duration, Instant};
use tachyonfx::{
    fx::{self, EvolveSymbolSet},
    pattern::{CoalescePattern, RadialPattern},
    CellFilter, Duration as FxDuration, Effect, Interpolation,
};

pub struct FxState {
    pub select_effect: Option<Effect>,
    pub deselect_effect: Option<Effect>,
    pub output_effect: Option<Effect>,
    pub output_pending: bool,
    pub last_tick: Instant,
}

impl FxState {
    pub fn new() -> Self {
        Self {
            select_effect: None,
            deselect_effect: None,
            output_effect: None,
            output_pending: false,
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

    pub fn trigger_output(&mut self) {
        self.output_pending = true;
        self.output_effect = None;
    }
    pub fn build_output_effect(&mut self, area: Rect) {
        let screen_bg = Color::Black;
        let content_bg = Color::Reset;
        let style = Style::default().fg(content_bg).bg(screen_bg);

        let boot_timer = (150, Interpolation::CircIn);
        let timer = (500, Interpolation::QuadIn);

        let startup = fx::evolve((EvolveSymbolSet::Shaded, style), boot_timer)
            .with_pattern(RadialPattern::with_transition((0.5, 0.5), 10.0))
            .with_area(area);

        let inner_fire_fx = fx::evolve_from((EvolveSymbolSet::Quadrants, style), timer)
            .with_pattern(CoalescePattern::new())
            .with_area(area)
            .reversed();

        let fire = fx::translate(inner_fire_fx, Offset { x: 0, y: -22 }, timer).with_area(area);

        let fade_in_text = fx::fade_from(screen_bg, screen_bg, timer)
            .with_filter(CellFilter::Text)
            .with_area(area)
            .with_pattern(CoalescePattern::new());

        self.output_effect = Some(fx::prolong_start(
            300,
            fx::sequence(&[
                startup,
                fx::parallel(&[fx::fade_from(screen_bg, screen_bg, 300), fire, fade_in_text]),
            ]),
        ));
    }
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        elapsed
    }
}
