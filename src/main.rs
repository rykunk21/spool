use crossterm::{
    event::{self, poll, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::style::Color;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use tachyonfx::{fx, Duration as FxDuration, Effect, EffectTimer, Shader};
mod cell;
use cell::Cell;

mod app;
use app::App;

struct FxState {
    select_effect: Option<Effect>,
    deselect_effect: Option<Effect>,
    last_tick: Instant,
}

impl FxState {
    fn new() -> Self {
        Self {
            select_effect: None,
            deselect_effect: None,
            last_tick: Instant::now(),
        }
    }

    fn trigger_selection(&mut self) {
        self.select_effect = Some(fx::fade_from_fg(
            Color::Yellow,
            FxDuration::from_millis(300),
        ));
        self.deselect_effect = Some(fx::fade_to_fg(
            Color::DarkGray,
            FxDuration::from_millis(200),
        ));
    }

    fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        elapsed
    }
}

fn ui(frame: &mut ratatui::Frame, app: &App, fx_state: &mut FxState) {
    let constraints = vec![Constraint::Length(6); app.cells.len()];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    let elapsed = fx_state.tick();

    for (i, cell) in app.cells.iter().enumerate() {
        let is_selected = i == app.selected;
        let is_last = i == app.last_selected && i != app.selected;

        let border_style = if is_selected {
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray)
        };

        let title = if is_selected {
            if app.editing {
                format!("Cell {} [editing]", cell.id)
            } else {
                format!("Cell {} [selected]", cell.id)
            }
        } else {
            format!("Cell {}", cell.id)
        };

        let widget = Paragraph::new(cell.source.clone()).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(widget, chunks[i]);

        // apply fade-in on newly selected cell
        if is_selected {
            if let Some(effect) = &mut fx_state.select_effect {
                let buf = frame.buffer_mut();
                effect.process(elapsed.into(), buf, chunks[i]);
            }
        }

        if is_last {
            if let Some(effect) = &mut fx_state.deselect_effect {
                let buf = frame.buffer_mut();
                effect.process(elapsed.into(), buf, chunks[i]);
            }
        }
    }

    // clean up finished effects
    if fx_state.select_effect.as_ref().map_or(false, |e| e.done()) {
        fx_state.select_effect = None;
    }
    if fx_state
        .deselect_effect
        .as_ref()
        .map_or(false, |e| e.done())
    {
        fx_state.deselect_effect = None;
    }
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut fx_state = FxState::new();

    loop {
        terminal.draw(|f| ui(f, &app, &mut fx_state))?;

        // use poll so we can redraw during animations
        if poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if !app.editing {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.move_down();
                            fx_state.trigger_selection();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.move_up();
                            fx_state.trigger_selection();
                        }
                        KeyCode::Char('a') => {
                            app.add_cell();
                            fx_state.trigger_selection();
                        }
                        KeyCode::Char('d') => app.delete_cell(),
                        KeyCode::Enter => app.toggle_focus(),
                        KeyCode::Tab => app.run_cell(),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Backspace => app.backspace(),
                        KeyCode::Char(c) => app.append_char(c),
                        KeyCode::Enter => app.toggle_focus(),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
