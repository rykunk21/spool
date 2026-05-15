// src/engine.rs
use crate::app::cell::Cell;
use crate::app::cell::CellOutput;
use rhai::{Dynamic, Engine as RhaiEngine, EvalAltResult};
use std::sync::{Arc, Mutex};
pub mod plot;
mod polars;
use plot::PlotSpec;

pub struct Engine {
    pub rengine: RhaiEngine,
}

impl Engine {
    pub fn new() -> Self {
        let mut rengine = RhaiEngine::new();
        polars::register_polars(&mut rengine);
        plot::register_plot(&mut rengine);
        register_os(&mut rengine);
        Engine { rengine }
    }

    pub fn run_cell(&mut self, cell: &mut Cell) -> Result<CellOutput, Box<rhai::EvalAltResult>> {
        let script = cell.textarea.lines().join("\n");
        let output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

        let out = output.clone();
        self.rengine.on_print(move |text| {
            let mut s = out.lock().unwrap();
            s.push_str(text);
            s.push('\n');
        });

        let mut scope = rhai::Scope::new();
        let result = self
            .rengine
            .eval_with_scope::<Dynamic>(&mut scope, &script)?;

        // Check if result is a PlotSpec
        if let Some(spec) = result.clone().try_cast::<PlotSpec>() {
            return Ok(CellOutput::Plot(spec));
        }

        let printed = output.lock().unwrap().trim_end().to_string();
        if !printed.is_empty() {
            Ok(CellOutput::Text(printed))
        } else if result.is_unit() {
            Ok(CellOutput::Empty)
        } else {
            Ok(CellOutput::Text(result.to_string()))
        }
    }
}

pub fn register_os(engine: &mut RhaiEngine) {
    engine.register_fn("cwd", || -> Result<String, Box<EvalAltResult>> {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| e.to_string().into())
    });
    engine.register_fn("ls", || -> Result<String, Box<EvalAltResult>> {
        let mut entries = std::fs::read_dir(".")
            .map_err(|e| -> Box<EvalAltResult> { e.to_string().into() })?
            .map(|entry| {
                entry
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .map_err(|e| -> Box<EvalAltResult> { e.to_string().into() })
            })
            .collect::<Result<Vec<String>, _>>()?;
        entries.sort();
        Ok(entries.join("\n"))
    });
}
