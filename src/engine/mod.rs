// src/engine.rs
use crate::app::cell::Cell;
use crate::app::cell::CellOutput;
use rhai::{Dynamic, Engine as RhaiEngine, EvalAltResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub mod plot;
mod polars;
use plot::PlotSpec;

pub struct Engine {
    pub rengine: RhaiEngine,
    pub scope: rhai::Scope<'static>,
    output_map: HashMap<String, Dynamic>,
}

impl Engine {
    pub fn new() -> Self {
        let mut rengine = RhaiEngine::new();
        polars::register_polars(&mut rengine);
        plot::register_plot(&mut rengine);
        register_os(&mut rengine);
        Engine {
            rengine,
            scope: rhai::Scope::new(),
            output_map: HashMap::new(),
        }
    }

    pub fn run_cell(&mut self, cell: &mut Cell) -> Result<CellOutput, Box<rhai::EvalAltResult>> {
        let script = cell.textarea.lines().join("\n");

        // initialize the output for printing
        let output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

        // try to get against the cahce
        let result = match self.output_map.get(&cell.id.to_string()) {
            Some(result) => result,
            None => {
                let out = output.clone();
                self.rengine.on_print(move |text| {
                    let mut s = out.lock().unwrap();
                    s.push_str(text);
                    s.push('\n');
                });

                let result = self
                    .rengine
                    .eval_with_scope::<Dynamic>(&mut self.scope, &script)?;
                // cache the result
                self.output_map.insert(cell.id.to_string(), result.clone());
                // debugging
                self.log(&result);
                self.output_map.get(&cell.id.to_string()).unwrap()
            }
        };

        // Format the result as the cell output for app
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

    /*
     * Helpers for cell running
     */
    /// Resolve the

    /// log the
    fn log(&self, result: &Dynamic) {
        use std::fs::OpenOptions;
        use std::io::Write;

        let type_name = result.type_name();
        let data = result.to_string();
        let line = format!("[{}] {}\n", type_name, data);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("rhai_log.txt")
            .expect("Failed to open log file");

        file.write_all(line.as_bytes())
            .expect("Failed to write log");
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
