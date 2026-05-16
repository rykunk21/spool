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
    pub output_map: HashMap<String, Dynamic>,
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
    pub fn run_script(
        &mut self,
        script: &str,
    ) -> Result<(Dynamic, String), Box<rhai::EvalAltResult>> {
        let output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let out = output.clone();
        self.rengine.on_print(move |text| {
            let mut s = out.lock().unwrap();
            s.push_str(text);
            s.push('\n');
        });
        let result = self
            .rengine
            .eval_with_scope::<Dynamic>(&mut self.scope, script)?;
        let printed = output.lock().unwrap().trim_end().to_string();
        Ok((result, printed))
    } // converts Dynamic to CellOutput

    pub fn format_output(&self, result: Dynamic, printed: String) -> CellOutput {
        if let Some(spec) = result.clone().try_cast::<PlotSpec>() {
            return CellOutput::Plot(spec);
        }
        if !printed.is_empty() {
            CellOutput::Text(printed)
        } else if result.is_unit() {
            CellOutput::Empty
        } else {
            CellOutput::Text(result.to_string())
        }
    }
    // Hash the cell with the script
    // logging
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
