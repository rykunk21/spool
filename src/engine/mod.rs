// src/engine.rs
use crate::app::cell::{Cell, CellOutput};
use rhai::{Dynamic, Engine as RhaiEngine, EvalAltResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub mod dep;
pub mod plot;
mod polars;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use plot::PlotSpec;

pub struct Engine {
    pub rengine: RhaiEngine,
    pub scope: rhai::Scope<'static>,
    pub output_map: HashMap<u64, Dynamic>,
    // dependency graph — nodes are cell ids, edges are dependencies
    graph: DiGraph<usize, ()>,
    // map from cell_id to node index in the graph
    node_index: HashMap<usize, NodeIndex>,
    // map from cell_id to clean/dirty state
    pub dirty: HashMap<usize, bool>,
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
            graph: DiGraph::new(),
            node_index: HashMap::new(),
            dirty: HashMap::new(),
        }
    }

    // register a cell with the graph when it is created
    pub fn register_cell(&mut self, cell_id: usize) {
        let idx = self.graph.add_node(cell_id);
        self.node_index.insert(cell_id, idx);
        self.dirty.insert(cell_id, true);
    }

    // remove a cell from the graph when it is deleted
    pub fn deregister_cell(&mut self, cell_id: usize) {
        if let Some(idx) = self.node_index.remove(&cell_id) {
            self.graph.remove_node(idx);
        }
        self.dirty.remove(&cell_id);
    }

    // mark a cell dirty and propagate to all downstream dependents
    pub fn mark_dirty(&mut self, cell_id: usize) {
        self.dirty.insert(cell_id, true);
        // find all nodes that transitively depend on this cell
        let dependents = self.get_dependents(cell_id);
        for dep in dependents {
            self.dirty.insert(dep, true);
        }
    }

    pub fn is_dirty(&self, cell_id: usize) -> bool {
        *self.dirty.get(&cell_id).unwrap_or(&true)
    }

    // get all cells that directly or transitively depend on cell_id
    fn get_dependents(&self, cell_id: usize) -> Vec<usize> {
        let Some(&node) = self.node_index.get(&cell_id) else {
            return vec![];
        };
        // walk forward edges (dependents)
        let mut result = Vec::new();
        let mut stack = vec![node];
        let mut visited = std::collections::HashSet::new();
        while let Some(n) = stack.pop() {
            if !visited.insert(n) {
                continue;
            }
            for neighbor in self.graph.neighbors(n) {
                result.push(*self.graph.node_weight(neighbor).unwrap());
                stack.push(neighbor);
            }
        }
        result
    }

    // get all cells that this cell depends on, in topological order
    fn get_dependencies(&self, cell_id: usize) -> Vec<usize> {
        let Some(&node) = self.node_index.get(&cell_id) else {
            return vec![];
        };
        // walk reverse edges (dependencies)
        use petgraph::visit::EdgeRef;
        let mut result = Vec::new();
        let mut stack = vec![node];
        let mut visited = std::collections::HashSet::new();
        while let Some(n) = stack.pop() {
            if !visited.insert(n) {
                continue;
            }
            for edge in self.graph.edges_directed(n, petgraph::Direction::Incoming) {
                let dep_node = edge.source();
                result.push(*self.graph.node_weight(dep_node).unwrap());
                stack.push(dep_node);
            }
        }
        result
    }

    // update the dependency edges for a cell after analyzing its source
    pub fn update_deps(&mut self, cell_id: usize, dep_ids: Vec<usize>) {
        let Some(&node) = self.node_index.get(&cell_id) else {
            return;
        };
        // remove existing incoming edges for this cell
        let old_edges: Vec<_> = self
            .graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .map(|e| e.id())
            .collect();
        for edge in old_edges {
            self.graph.remove_edge(edge);
        }
        // add new edges: dep -> cell_id
        for dep_id in dep_ids {
            if let Some(&dep_node) = self.node_index.get(&dep_id) {
                self.graph.add_edge(dep_node, node, ());
            }
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
    }

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

    pub fn export_dot(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        )
    }

    pub fn run(&mut self, cell_id: usize, cells: &mut Vec<Cell>) -> Result<(), Box<EvalAltResult>> {
        // if clean, nothing to do
        if !self.is_dirty(cell_id) {
            return Ok(());
        }

        // get script source
        let idx = cells
            .iter()
            .position(|c| c.id == cell_id)
            .unwrap_or_else(|| panic!("cell_id {} not found", cell_id));
        let script = cells[idx].textarea.lines().join("\n");

        // analyze deps from AST
        let (defines, uses) = dep::get_defines_and_uses(&self.rengine, &script);

        // resolve which cell ids define the used variables
        let dep_ids: Vec<usize> = uses
            .iter()
            .filter_map(|var| {
                // walk backwards through cells before this one
                cells[..idx]
                    .iter()
                    .rev()
                    .find(|c| {
                        let s = c.textarea.lines().join("\n");
                        let (defs, _) = dep::get_defines_and_uses(&self.rengine, &s);
                        defs.contains(var)
                    })
                    .map(|c| c.id)
            })
            .collect();

        // update graph edges
        self.update_deps(cell_id, dep_ids.clone());

        // recursively run dirty dependencies first
        for dep_id in dep_ids {
            self.run(dep_id, cells)?;
        }

        // execute this cell
        let script = cells[idx].textarea.lines().join("\n");
        match self.run_script(&script) {
            Ok((result, printed)) => {
                cells[idx].output = self.format_output(result, printed);
                self.dirty.insert(cell_id, false);
            }
            Err(e) => {
                cells[idx].output = CellOutput::Error(e.to_string());
                return Err(e);
            }
        }
        Ok(())
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
