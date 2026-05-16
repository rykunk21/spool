// external imports
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use ratatui_textarea::Input;
use std::{io, path::PathBuf};

// crate modules
mod app;
mod engine;
mod ui;
mod util;
// crate ns resolution
use crate::util::vim::Mode;
use app::App;
use ui::ui;
use util::vim::{Transition, Vim};

fn main() -> anyhow::Result<()> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("notebook.md"));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(path)?;
    loop {
        // draw the ui every iteration
        terminal.draw(|f| ui(f, &mut app))?;

        // poll for an event?
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if !app.editing {
                    match key.code {
                        KeyCode::Char('o') => {
                            app.show_output = !app.show_output;
                        }
                        KeyCode::Char('q') => {
                            app.save_to_file()?;
                            break;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.move_down();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.move_up();
                        }
                        KeyCode::Char('a') => {
                            app.add_cell();
                        }
                        KeyCode::Char('d') => app.delete_cell(),
                        KeyCode::Enter => {
                            app.toggle_focus();
                            app.show_output = false;
                        }
                        KeyCode::Tab => {
                            app.run();
                            app.show_output = true;
                            app.fx_state.trigger_output();
                        }
                        _ => {}
                    }
                } else {
                    let input = Input::from(key);
                    if input.key == ratatui_textarea::Key::Esc
                        && app.cells[app.selected].vim.mode == Mode::Normal
                    {
                        app.toggle_focus();
                        app.save_to_file()?;
                    } else {
                        let cell = &mut app.cells[app.selected];
                        match cell.vim.transition(input, &mut cell.textarea) {
                            Transition::Mode(mode) if cell.vim.mode != mode => {
                                cell.textarea.set_block(mode.block());
                                cell.textarea.set_cursor_style(mode.cursor_style());
                                cell.vim = Vim::new(mode);
                            }
                            Transition::Pending(input) => {
                                cell.vim = Vim::new(cell.vim.mode).with_pending(input);
                            }
                            Transition::Quit => {
                                app.toggle_focus();
                                app.save_to_file()?;
                            }
                            Transition::Nop | Transition::Mode(_) => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
