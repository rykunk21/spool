// src/engine.rs
use crate::app::cell::Cell;
use polars::prelude::*;
use rhai::Engine as RhaiEngine;
use std::sync::{Arc, Mutex};

pub struct Engine {
    pub rengine: RhaiEngine,
}

impl Engine {
    pub fn new() -> Self {
        let mut engine = RhaiEngine::new();
        Engine { rengine: engine }
    }

    pub fn run_cell(&mut self, cell: &mut Cell) -> Result<String, Box<rhai::EvalAltResult>> {
        let script = cell.textarea.lines().join("\n");
        let output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let out = output.clone();
        self.rengine.on_print(move |text| {
            let mut s = out.lock().unwrap();
            s.push_str(text);
            s.push('\n');
        });
        self.rengine.eval::<()>(&script)?;
        let result = output.lock().unwrap().trim_end().to_string();
        Ok(result)
    }
}

