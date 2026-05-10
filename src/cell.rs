use crate::util::vim::{Mode, Vim};
use tui_textarea::TextArea;

pub struct Cell {
    pub id: usize,
    pub textarea: TextArea<'static>,
    pub vim: Vim,
    pub output: Option<String>,
}

impl Cell {
    pub fn new(id: usize) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_style(Mode::Normal.cursor_style());
        Self {
            id,
            textarea,
            vim: Vim::new(Mode::Normal),
            output: None,
        }
    }
}
