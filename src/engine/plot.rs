use std::sync::Mutex;

use crate::app::cell::{PlotKind, PlotSpec};
use polars::prelude::*;
use rhai::{Array, Engine as RhaiEngine};

pub fn register_plot(engine: &mut RhaiEngine, plot_out: Arc<Mutex<Option<PlotSpec>>>) {
    engine.register_fn("plot", move |x: Array, y: Array, title: String| {
        let data: Vec<(f64, f64)> = x
            .iter()
            .zip(y.iter())
            .filter_map(|(a, b)| Some((a.as_float().ok()?, b.as_float().ok()?)))
            .collect();
        *plot_out.lock().unwrap() = Some(PlotSpec {
            title,
            data,
            x_label: String::from("x"),
            y_label: String::from("y"),
            kind: PlotKind::Line,
        });
    });
}
