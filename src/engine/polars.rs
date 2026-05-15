use polars::prelude::*;
use rhai::{Array, Dynamic, Engine as RhaiEngine, EvalAltResult};

#[derive(Clone)]
pub struct RhaiDataFrame(pub DataFrame);

impl RhaiDataFrame {
    fn read_csv(path: String) -> Result<Self, Box<EvalAltResult>> {
        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(path.into()))
            .map_err(|e| e.to_string())?
            .finish()
            .map_err(|e| e.to_string())?;
        Ok(RhaiDataFrame(df))
    }

    fn shape(&mut self) -> Array {
        let (rows, cols) = self.0.shape();
        vec![Dynamic::from(rows as i64), Dynamic::from(cols as i64)]
    }

    fn columns(&mut self) -> Array {
        self.0
            .get_column_names()
            .iter()
            .map(|s| Dynamic::from(s.to_string()))
            .collect()
    }

    fn select(&mut self, col: String) -> Result<Array, Box<EvalAltResult>> {
        let arr = self
            .0
            .column(&col)
            .map_err(|e| e.to_string())?
            .as_series()
            .unwrap()
            .f64()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|v| Dynamic::from(v.unwrap_or(f64::NAN)))
            .collect();
        Ok(arr)
    }
}

pub fn register_polars(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<RhaiDataFrame>("DataFrame");
    engine.register_fn("read_csv", RhaiDataFrame::read_csv);
    engine.register_fn("shape", RhaiDataFrame::shape);
    engine.register_fn("columns", RhaiDataFrame::columns);
    engine.register_fn("select", RhaiDataFrame::select);
}
