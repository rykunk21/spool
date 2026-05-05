use rhai::{Array, Dynamic, Engine, Scope};
use serde::Deserialize;
use statrs::statistics::Statistics;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct Notebook {
    cells: Vec<Cell>,
}

#[derive(Debug, Deserialize)]
struct Cell {
    id: String,
    code: String,
    #[serde(default)]
    inputs: Vec<String>,
}

fn register_host_functions(engine: &mut Engine) {
    engine.register_fn("mean", |arr: Array| {
        let nums: Vec<f64> = arr.into_iter().map(|x| x.cast::<f64>()).collect();
        nums.mean()
    });

    engine.register_fn("variance", |arr: Array| {
        let nums: Vec<f64> = arr.into_iter().map(|x| x.cast::<f64>()).collect();
        nums.variance()
    });
}

// Long-lived notebook execution context
struct NotebookContext {
    engine: Engine,
    scope: Scope,
    outputs: HashMap<String, Dynamic>,
}

impl NotebookContext {
    fn new() -> Self {
        let mut engine = Engine::new();
        register_host_functions(&mut engine);

        Self {
            engine,
            scope: Scope::new(),
            outputs: HashMap::new(),
        }
    }

    fn execute_cell(&mut self, cell: &Cell) -> Result<Dynamic, Box<dyn std::error::Error>> {
        // Inject multiple input values into scope
        for input_id in &cell.inputs {
            let value = self
                .outputs
                .get(input_id)
                .ok_or_else(|| format!("Missing input cell '{}'", input_id))?;
            self.scope.push(input_id, value.clone());
        }

        // Evaluate Rhai code
        let result = self
            .engine
            .eval_with_scope::<Dynamic>(&mut self.scope, &cell.code)?;
        self.outputs.insert(cell.id.clone(), result.clone());

        Ok(result)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load notebook JSON
    let json = fs::read_to_string("notebook.json")?;
    let notebook: Notebook = serde_json::from_str(&json)?;

    // Create a persistent notebook context
    let mut ctx = NotebookContext::new();

    // Execute all cells in order
    for cell in &notebook.cells {
        let result = ctx.execute_cell(cell)?;
        println!("cell {} => {:?}", cell.id, result);
    }

    Ok(())
}
