// external imports
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tachyonfx::Shader;

// user imports
use crate::app::App;
mod ui_fx;
pub use ui_fx::FxState;

pub fn ui(frame: &mut ratatui::Frame, app: &mut App, fx_state: &mut FxState) {
    let elapsed = fx_state.tick();
    let visible = app.viewport.visible_mut(&mut app.cells);
    let constraints = vec![Constraint::Length(6); visible.len()];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    let offset = app.viewport.offset;
    for (i, cell) in visible.iter_mut().enumerate() {
        let is_selected = i + offset == app.selected;
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

        cell.textarea.set_block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        cell.textarea
            .set_cursor_line_style(if is_selected && app.editing {
                ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED)
            } else {
                ratatui::style::Style::default()
            });
        cell.textarea
            .set_cursor_style(if is_selected && app.editing {
                ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::REVERSED)
            } else {
                ratatui::style::Style::default()
            });

        frame.render_widget(cell.textarea.widget(), chunks[i]);

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
    if app.show_output {
        let output = app.cells[app.selected]
            .output
            .as_deref()
            .unwrap_or("No output");

        // center a popup in the terminal
        let area = centered_rect(60, 40, frame.area());

        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(output)
                .block(
                    Block::default()
                        .title("Output (o to close)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
