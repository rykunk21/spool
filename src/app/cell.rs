use crate::util::vim::{Mode, Vim};
use ratatui_textarea::TextArea;

pub enum CellOutput {
    Text(String),
    Plot(PlotSpec),
    Error(String),
    Empty,
}
// We can keep this for now. in the future we might want to refactor to
// https://github.com/resonant-jovian/ratatui-plt
pub struct PlotSpec {
    pub title: String,
    pub data: Vec<(f64, f64)>,
    pub x_label: String,
    pub y_label: String,
    pub kind: PlotKind,
}

pub enum PlotKind {
    Line,
    Scatter,
    Bar,
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
