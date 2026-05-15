use crate::engine::plot::PlotSpec;
use crate::util::vim::{Mode, Vim};
use ratatui_textarea::TextArea;

pub enum CellOutput {
    Text(String),
    Plot(PlotSpec),
    Error(String),
    Empty,
}

pub struct Cell {
    pub id: usize,
    pub textarea: TextArea<'static>,
    pub vim: Vim,
    pub output: CellOutput,
}

impl Cell {
    pub fn new(id: usize) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_style(Mode::Normal.cursor_style());
        Self {
            id,
            textarea,
            vim: Vim::new(Mode::Normal),
            output: CellOutput::Empty,
        }
    }
}
