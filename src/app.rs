use crate::cell::Cell;

pub struct App {
    pub cells: Vec<Cell>,
    pub selected: usize,
    pub editing: bool,
    pub last_selected: usize,
    next_id: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            cells: vec![],
            selected: 0,
            next_id: 1,
            editing: false,
            last_selected: 0,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.last_selected = self.selected;
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.cells.len() {
            self.last_selected = self.selected;
            self.selected += 1;
        }
    }

    pub fn add_cell(&mut self) {
        let idx = if self.cells.is_empty() {
            0
        } else {
            self.selected + 1
        };
        self.cells.insert(idx, Cell::new(self.next_id));
        self.next_id += 1;
        self.last_selected = self.selected;
        self.selected = idx;
    }

    pub fn delete_cell(&mut self) {
        self.cells.remove(self.selected);
        if self.selected >= self.cells.len() && !self.cells.is_empty() {
            self.selected = self.cells.len() - 1;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.editing = !self.editing;
    }

    pub fn append_char(&mut self, ch: char) {
        self.cells[self.selected].source.push(ch);
    }

    pub fn backspace(&mut self) {
        self.cells[self.selected].source.pop();
    }

    pub fn run_cell(&mut self) {
        self.cells[self.selected].run();
    }
}
