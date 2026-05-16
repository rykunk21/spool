use std::path::PathBuf;
mod viewport;
use crate::{
    app::viewport::Viewport,
    engine::Engine,
    ui::ui_fx::FxState,
    util::{Mode, Vim},
};
pub mod cell;
use cell::{Cell, CellOutput};

use ratatui_textarea::TextArea;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct App {
    pub cells: Vec<Cell>,
    pub viewport: Viewport,
    pub selected: usize,
    pub editing: bool,
    pub last_selected: usize,
    pub path: PathBuf,
    pub show_output: bool,
    pub fx_state: FxState,
    next_id: usize,
    engine: Engine,
}

impl App {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            std::fs::write(&path, "")?;
        }
        let mut app = Self {
            cells: Vec::new(),
            viewport: Viewport::new(5),
            selected: 0,
            editing: false,
            last_selected: 0,
            path,
            show_output: false,
            fx_state: FxState::new(),
            engine: Engine::new(),
            next_id: 1,
        };
        app.load_from_file()?;
        Ok(app)
    }

    pub fn load_from_file(&mut self) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&self.path)?;
        self.cells.clear();
        self.next_id = 1;
        let mut in_block = false;
        let mut current: Vec<String> = Vec::new();
        for line in content.lines() {
            if line.starts_with("```rust") && !in_block {
                in_block = true;
                current.clear();
            } else if line.starts_with("```") && in_block {
                in_block = false;
                let ta = TextArea::new(current.clone());
                self.cells.push(Cell {
                    id: self.next_id,
                    textarea: ta,
                    vim: Vim::new(Mode::Normal),
                    output: CellOutput::Empty,
                });
                self.next_id += 1;
            } else if in_block {
                current.push(line.to_string());
            }
        }
        Ok(())
    }

    pub fn save_to_file(&self) -> anyhow::Result<()> {
        let mut out = String::new();
        for cell in &self.cells {
            out.push_str("```rust\n");
            for line in cell.textarea.lines() {
                out.push_str(line);
                out.push('\n');
            }
            out.push_str("```\n\n");
        }
        std::fs::write(&self.path, out)?;
        Ok(())
    }
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.last_selected = self.selected;
            self.selected -= 1;
            // scroll viewport up when selected moves above the window
            if self.selected < self.viewport.offset {
                self.viewport.scroll_up();
            }
            self.fx_state.trigger_selection();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.cells.len() {
            self.last_selected = self.selected;
            self.selected += 1;
            // scroll viewport down when selected moves below the window
            if self.selected >= self.viewport.offset + self.viewport.height {
                self.viewport.scroll_down(self.cells.len());
            }
            self.fx_state.trigger_selection();
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
        // ensure new cell is visible
        if self.selected >= self.viewport.offset + self.viewport.height {
            self.viewport.scroll_down(self.cells.len());
        }
    }

    pub fn delete_cell(&mut self) {
        self.cells.remove(self.selected);
        if self.selected >= self.cells.len() && !self.cells.is_empty() {
            self.selected = self.cells.len() - 1;
        }
        // clamp viewport if it's now past the end
        let max_offset = self.cells.len().saturating_sub(self.viewport.height);
        self.viewport.offset = self.viewport.offset.min(max_offset);
    }

    pub fn toggle_focus(&mut self) {
        self.editing = !self.editing;
    }
    // on App
    pub fn run_from(&mut self, start: usize) -> Result<(), Box<rhai::EvalAltResult>> {
        // reset the scope when doing a rerun.
        self.engine.scope = rhai::Scope::new();
        let keys = self.compute_keys();
        for i in start..self.cells.len() {
            let key = keys[i].clone();
            if !self.engine.output_map.contains_key(&key) {
                let script = self.cells[i].textarea.lines().join("\n");
                match self.engine.run_script(&script) {
                    Ok((result, printed)) => {
                        self.engine.output_map.insert(key, result.clone());
                        self.cells[i].output = self.engine.format_output(result, printed);
                    }
                    Err(e) => {
                        self.cells[i].output = CellOutput::Error(e.to_string());
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
    pub fn run(&mut self) {
        // find the first dirty cell
        let keys = self.compute_keys();
        let start = keys
            .iter()
            .enumerate()
            .find(|(i, key)| !self.engine.output_map.contains_key(*key))
            .map(|(i, _)| i)
            .unwrap_or(self.selected);

        if let Err(e) = self.run_from(start) {
            self.cells[self.selected].output = CellOutput::Error(e.to_string());
        }
    }

    pub fn compute_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        let mut upstream = String::from("root");
        for cell in &self.cells {
            let script = cell.textarea.lines().join("\n");
            let key = App::cell_key(cell.id, &script, &upstream);
            upstream = key.clone();
            keys.push(key);
        }
        keys
    }
    fn cell_key(cell_id: usize, script: &str, upstream_key: &str) -> String {
        let mut hasher = DefaultHasher::new();
        cell_id.hash(&mut hasher);
        script.hash(&mut hasher);
        upstream_key.hash(&mut hasher);
        hasher.finish().to_string()
    }
}
