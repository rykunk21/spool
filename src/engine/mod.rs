// src/engine.rs
use crate::app::cell::Cell;
use crate::app::cell::CellOutput;
use crate::app::cell::PlotSpec;
use rhai::{Dynamic, Engine as RhaiEngine, EvalAltResult};
use std::sync::{Arc, Mutex};
mod plot;
mod polars;

pub struct Engine {
    pub rengine: RhaiEngine,
    pub plot_out: Arc<Mutex<Option<PlotSpec>>>,
}

impl Engine {
    pub fn new() -> Self {
        let plot_out: Arc<Mutex<Option<PlotSpec>>> = Arc::new(Mutex::new(None));
        let mut rengine = RhaiEngine::new();
        polars::register_polars(&mut rengine);
        plot::register_plot(&mut rengine, plot_out.clone());
        register_os(&mut rengine);
        Engine { rengine, plot_out }
    }

    /// Should this be modified to return a cell output? Only consumed by app_run.
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

        // check plot buffer first
        let plot = self.plot_out.lock().unwrap().take();
        if let Some(spec) = plot {
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
