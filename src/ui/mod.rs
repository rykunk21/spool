// external imports
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::cell::CellOutput;
use crate::app::App;
pub mod ui_fx;

pub fn ui(frame: &mut ratatui::Frame, app: &mut App) {
    let visible = app.viewport.visible_mut(&mut app.cells);
    let constraints = vec![Constraint::Length(6); visible.len()];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    let offset = app.viewport.offset;
    let elapsed = app.fx_state.tick();

    for (i, cell) in visible.iter_mut().enumerate() {
        let is_selected = i + offset == app.selected;

        let is_last = i + offset == app.last_selected && i + offset != app.selected;

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

        frame.render_widget(&cell.textarea, chunks[i]);

        if is_selected {
            if let Some(effect) = &mut app.fx_state.select_effect {
                let buf = frame.buffer_mut();
                effect.process(elapsed.into(), buf, chunks[i]);
            }
        }
        if is_last {
            if let Some(effect) = &mut app.fx_state.deselect_effect {
                let buf = frame.buffer_mut();
                effect.process(elapsed.into(), buf, chunks[i]);
            }
        }
    }

    if app.show_output {
        let area = centered_rect(80, 60, frame.area());
        frame.render_widget(ratatui::widgets::Clear, area);

        if app.fx_state.output_pending {
            app.fx_state.build_output_effect(area);
            app.fx_state.output_pending = false;
        }

        match &app.cells[app.selected].output {
            CellOutput::Text(text) => {
                print_text(frame, text, area);
            }
            CellOutput::Error(e) => {
                print_text(frame, e, area);
            }
            CellOutput::Empty => {
                frame.render_widget(
                    Paragraph::new("No output").block(
                        Block::default()
                            .title("Output (o to close)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)),
                    ),
                    area,
                );
            }

            CellOutput::Plot(spec) => {
                let block = Block::default()
                    .title("Output (o to close)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green));
                let inner = block.inner(area);
                frame.render_widget(block, area);
                frame.render_widget(spec, inner)
            }
            _ => {
                panic!("unreachable");
            }
        }

        if let Some(effect) = &mut app.fx_state.output_effect {
            let buf = frame.buffer_mut();
            effect.process(elapsed.into(), buf, area);
        }
    }
}

fn print_text(frame: &mut ratatui::Frame, text: &String, area: Rect) {
    frame.render_widget(
        Paragraph::new(text.to_owned())
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

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
