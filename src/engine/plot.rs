use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
};
use ratatui_plt::prelude::*;
use rhai::{Array, Dynamic, Engine as RhaiEngine};

#[derive(Clone)]
pub struct PlotSpec {
    pub title: String,
    pub data: Vec<(f64, f64)>,
    pub x_label: String,
    pub y_label: String,
    pub kind: PlotKind,
}

#[derive(Clone)]
pub enum PlotKind {
    Line,
    Scatter,
    HeatMap,
}

impl Widget for &PlotSpec {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let series = Series::new(&self.title)
            .data(self.data.clone())
            .color(Color::Cyan);

        match self.kind {
            PlotKind::Line => {
                LinePlot::new()
                    .series(series)
                    .title(&self.title)
                    .x_axis(Axis::new().label(&self.x_label))
                    .y_axis(Axis::new().label(&self.y_label))
                    .render(area, buf);
            }
            PlotKind::Scatter => {
                ScatterPlot::new()
                    .series(series)
                    .title(&self.title)
                    .x_axis(Axis::new().label(&self.x_label))
                    .y_axis(Axis::new().label(&self.y_label))
                    .render(area, buf);
            }
            PlotKind::HeatMap => {
                Paragraph::new("HeatMap not yet implemented")
                    .block(Block::default().borders(Borders::ALL))
                    .render(area, buf);
            }
        }
    }
}

pub fn register_plot(engine: &mut RhaiEngine) {
    engine.register_fn("plot", |x: Array, y: Array, title: String, kind: String| {
        let data: Vec<(f64, f64)> = x
            .iter()
            .zip(y.iter())
            .filter_map(|(a, b)| Some((a.as_float().ok()?, b.as_float().ok()?)))
            .collect();

        let plot_kind = match kind.as_str() {
            "scatter" => PlotKind::Scatter,
            "heatmap" => PlotKind::HeatMap,
            "line" => PlotKind::Line,
            _ => PlotKind::Line,
        };

        Dynamic::from(PlotSpec {
            title,
            data,
            x_label: String::from("x"),
            y_label: String::from("y"),
            kind: plot_kind,
        })
    });
}
