// helioscope-collector/src/charts/renderer.rs

use plotters::prelude::*;
use std::io::Write;

use super::types::{ChartData, TimeSeriesChart};

/// Error type for chart rendering operations
#[derive(Debug)]
pub enum RenderError {
    DrawingError(String),
    DataError(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::DrawingError(msg) => write!(f, "Drawing error: {}", msg),
            RenderError::DataError(msg) => write!(f, "Data error: {}", msg),
        }
    }
}

impl std::error::Error for RenderError {}

/// Renders time-series data to SVG format
pub struct SvgRenderer {
    config: TimeSeriesChart,
}

impl SvgRenderer {
    pub fn new(config: TimeSeriesChart) -> Self {
        Self { config }
    }

    /// Render chart data to SVG string
    pub fn render_to_string(&self, data: &ChartData) -> Result<String, RenderError> {
        let mut buffer = Vec::new();
        self.render_to_writer(&mut buffer, data)?;
        String::from_utf8(buffer).map_err(|e| RenderError::DataError(format!("UTF-8 error: {}", e)))
    }

    /// Render chart data to a writer
    pub fn render_to_writer<W: Write>(
        &self,
        writer: &mut W,
        data: &ChartData,
    ) -> Result<(), RenderError> {
        if data.is_empty() {
            return Err(RenderError::DataError(
                "No data available to render".to_string(),
            ));
        }

        let mut svg_buffer = String::new();
        {
            let root =
                SVGBackend::with_string(&mut svg_buffer, (self.config.width, self.config.height))
                    .into_drawing_area();

            root.fill(&WHITE)
                .map_err(|e| RenderError::DrawingError(format!("Fill error: {:?}", e)))?;

            // Calculate data bounds
            let (x_min, x_max, y_min, y_max) = self.calculate_bounds(data)?;

            // Add small margin to y-axis
            let y_margin = (y_max - y_min) * 0.1;
            let y_min = (y_min - y_margin).max(0.0);
            let y_max = y_max + y_margin;

            // Build chart
            let mut chart = ChartBuilder::on(&root)
                .caption(&data.title, ("sans-serif", 30))
                .margin(10)
                .x_label_area_size(40)
                .y_label_area_size(60)
                .build_cartesian_2d(x_min..x_max, y_min..y_max)
                .map_err(|e| RenderError::DrawingError(format!("Chart build error: {:?}", e)))?;

            // Configure mesh (grid)
            let mut mesh = chart.configure_mesh();
            mesh.x_desc(&data.x_label)
                .y_desc(&data.y_label)
                .x_label_formatter(&|x| {
                    // Convert Unix timestamp to UTC date string
                    if let Ok(dt) = time::OffsetDateTime::from_unix_timestamp(*x) {
                        format!(
                            "{:04}-{:02}-{:02}\n{:02}:{:02}",
                            dt.year(),
                            dt.month() as u8,
                            dt.day(),
                            dt.hour(),
                            dt.minute()
                        )
                    } else {
                        x.to_string()
                    }
                });

            if self.config.show_grid {
                mesh.draw()
                    .map_err(|e| RenderError::DrawingError(format!("Mesh draw error: {:?}", e)))?;
            } else {
                mesh.disable_mesh()
                    .draw()
                    .map_err(|e| RenderError::DrawingError(format!("Mesh draw error: {:?}", e)))?;
            }

            // Draw series
            for (idx, series) in data.series.iter().enumerate() {
                if series.is_empty() {
                    continue;
                }

                let color = Palette99::pick(idx);

                chart
                    .draw_series(
                        series
                            .points
                            .iter()
                            .map(|p| (p.timestamp, p.value))
                            .collect::<Vec<_>>()
                            .windows(2)
                            .map(|w| {
                                PathElement::new(
                                    vec![(w[0].0, w[0].1), (w[1].0, w[1].1)],
                                    color.stroke_width(2),
                                )
                            }),
                    )
                    .map_err(|e| RenderError::DrawingError(format!("Series draw error: {:?}", e)))?
                    .label(&series.name)
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(3))
                    });
            }

            // Configure legend
            if self.config.show_legend && data.series.len() > 1 {
                chart
                    .configure_series_labels()
                    .background_style(WHITE.mix(0.8))
                    .border_style(BLACK)
                    .draw()
                    .map_err(|e| {
                        RenderError::DrawingError(format!("Legend draw error: {:?}", e))
                    })?;
            }

            root.present()
                .map_err(|e| RenderError::DrawingError(format!("Present error: {:?}", e)))?;
        }

        writer
            .write_all(svg_buffer.as_bytes())
            .map_err(|e| RenderError::DataError(format!("Write error: {}", e)))?;

        Ok(())
    }

    /// Calculate the bounds for x and y axes
    fn calculate_bounds(&self, data: &ChartData) -> Result<(i64, i64, f64, f64), RenderError> {
        let mut x_min = i64::MAX;
        let mut x_max = i64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for series in &data.series {
            for point in &series.points {
                x_min = x_min.min(point.timestamp);
                x_max = x_max.max(point.timestamp);
                y_min = y_min.min(point.value);
                y_max = y_max.max(point.value);
            }
        }

        if x_min == i64::MAX || y_min == f64::MAX {
            return Err(RenderError::DataError("No valid data points".to_string()));
        }

        // Ensure x range has at least some width
        if x_min == x_max {
            x_max = x_min + 1;
        }

        // Ensure y range has at least some height
        if (y_max - y_min).abs() < f64::EPSILON {
            y_max = y_min + 1.0;
        }

        Ok((x_min, x_max, y_min, y_max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::types::TimeSeries;

    #[test]
    fn test_render_simple_chart() {
        let mut series = TimeSeries::new("test_metric").with_unit("%");
        series.add_point(0, 10.0);
        series.add_point(1, 20.0);
        series.add_point(2, 15.0);

        let mut chart_data = ChartData::new("Test Chart");
        chart_data.add_series(series);

        let config = TimeSeriesChart::new(800, 400);
        let renderer = SvgRenderer::new(config);

        let result = renderer.render_to_string(&chart_data);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Chart"));
    }

    #[test]
    fn test_render_empty_data_fails() {
        let chart_data = ChartData::new("Empty Chart");
        let config = TimeSeriesChart::default();
        let renderer = SvgRenderer::new(config);

        let result = renderer.render_to_string(&chart_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_multiple_series() {
        let mut series1 = TimeSeries::new("cpu_core_0");
        series1.add_point(0, 30.0);
        series1.add_point(1, 45.0);

        let mut series2 = TimeSeries::new("cpu_core_1");
        series2.add_point(0, 25.0);
        series2.add_point(1, 40.0);

        let mut chart_data = ChartData::new("CPU Usage");
        chart_data.add_series(series1);
        chart_data.add_series(series2);

        let config = TimeSeriesChart::default();
        let renderer = SvgRenderer::new(config);

        let result = renderer.render_to_string(&chart_data);
        assert!(result.is_ok());
    }
}
