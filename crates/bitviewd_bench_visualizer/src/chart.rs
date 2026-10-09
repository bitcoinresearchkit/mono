use std::{error::Error, path::Path};

use plotters::{
    coord::Shift,
    prelude::*,
    style::text_anchor::{HPos, Pos, VPos},
};

use crate::{
    data::{Cycle, Run},
    format,
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
type Area<'a> = DrawingArea<SVGBackend<'a>, Shift>;

const WIDTH: u32 = 2400;
const LEGEND: i32 = 460;
const FONT: &str = "monospace";
const BG: RGBColor = RGBColor(18, 18, 24);
const TEXT: RGBColor = RGBColor(230, 230, 240);
const MAX_POINTS: usize = 2_500;
/// Trailing window for rates derived from cumulative counters.
const RATE_WINDOW_S: f64 = 60.0;
const PALETTE: [RGBColor; 12] = [
    RGBColor(78, 121, 167),
    RGBColor(242, 142, 43),
    RGBColor(225, 87, 89),
    RGBColor(118, 183, 178),
    RGBColor(89, 161, 79),
    RGBColor(237, 201, 72),
    RGBColor(176, 122, 161),
    RGBColor(255, 157, 167),
    RGBColor(156, 117, 95),
    RGBColor(188, 189, 34),
    RGBColor(102, 194, 255),
    RGBColor(190, 255, 120),
];

pub fn color(index: usize) -> RGBColor {
    PALETTE[index % PALETTE.len()]
}

fn phase_color(name: &str) -> RGBAColor {
    match name {
        "indexing" => RGBColor(78, 121, 167).mix(0.12),
        "compute" => RGBColor(242, 142, 43).mix(0.12),
        _ => RGBColor(176, 122, 161).mix(0.12),
    }
}

/// The width left of every time axis, so all panels share one horizontal scale.
const Y_LABELS: i32 = 220;

pub struct Series {
    pub label: String,
    pub color: RGBColor,
    pub dashed: bool,
    pub points: Vec<(f64, f64)>,
}

#[derive(Clone, Copy)]
pub enum Unit {
    Bytes,
    BytesPerSecond,
    Plain(&'static str),
}

impl Unit {
    fn scale(self, max: f64) -> (f64, String) {
        match self {
            Self::Bytes => {
                let (divisor, unit) = format::byte_unit(max);
                (divisor, unit.to_owned())
            }
            Self::BytesPerSecond => {
                let (divisor, unit) = format::byte_unit(max);
                (divisor, format!("{unit}/s"))
            }
            Self::Plain(label) => (1.0, label.to_owned()),
        }
    }
}

pub struct TimeAxis {
    max_s: f64,
    divisor: f64,
    label: &'static str,
}

impl TimeAxis {
    pub fn new(max_s: f64) -> Self {
        let max_s = max_s.max(1.0);
        let (divisor, label) = match max_s {
            s if s >= 7_200.0 => (3_600.0, "hours"),
            s if s >= 120.0 => (60.0, "minutes"),
            _ => (1.0, "seconds"),
        };
        Self {
            max_s,
            divisor,
            label,
        }
    }

    fn x(&self, seconds: f64) -> f64 {
        seconds / self.divisor
    }

    fn range(&self) -> std::ops::Range<f64> {
        0.0..self.x(self.max_s) * 1.01
    }
}

/// Rate of a cumulative counter over exactly the trailing window, its start interpolated; no
/// point before a full window exists, so start-up bursts are averaged like everything else.
pub fn rate(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let Some(&(first, _)) = points.first() else {
        return Vec::new();
    };
    let mut rates = Vec::with_capacity(points.len());
    let mut start = 0;
    for &(time, value) in points {
        let from = time - RATE_WINDOW_S;
        if from < first {
            continue;
        }
        while start + 1 < points.len() && points[start + 1].0 <= from {
            start += 1;
        }
        if start + 1 >= points.len() || points[start + 1].0 <= points[start].0 {
            continue;
        }
        let ((t0, v0), (t1, v1)) = (points[start], points[start + 1]);
        let from_value = v0 + (v1 - v0) * (from - t0) / (t1 - t0);
        rates.push((time, (value - from_value).max(0.0) / RATE_WINDOW_S));
    }
    rates
}

/// Trailing-window mean, to compare noisy samples across runs.
pub fn mean(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut means = Vec::with_capacity(points.len());
    let (mut start, mut sum) = (0, 0.0);
    for (index, &(time, value)) in points.iter().enumerate() {
        sum += value;
        while time - points[start].0 > RATE_WINDOW_S {
            sum -= points[start].1;
            start += 1;
        }
        means.push((time, sum / (index - start + 1) as f64));
    }
    means
}

fn tick(value: f64) -> String {
    if value.abs() >= 1_000.0 {
        format::count(value)
    } else if value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        let text = format!("{value:.2}");
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

fn font(size: i32) -> TextStyle<'static> {
    (FONT, size).into_font().color(&TEXT)
}

fn dim_font(size: i32) -> TextStyle<'static> {
    (FONT, size).into_font().color(&TEXT.mix(0.7))
}

pub fn header_height(lines: usize) -> u32 {
    100 + lines as u32 * 30
}

/// Text lines at the top-left of an area.
pub fn header(area: &Area, title: &str, lines: &[String]) -> Result {
    area.draw(&Text::new(title.to_owned(), (30, 22), font(34)))?;
    for (index, line) in lines.iter().enumerate() {
        area.draw(&Text::new(
            line.clone(),
            (30, 74 + index as i32 * 30),
            dim_font(21),
        ))?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Swatch {
    Line,
    Dashed,
    Box,
}

const LEGEND_ROW: i32 = 28;

/// Height a panel needs so its legend fits beside the plot.
pub fn lines_height(entries: usize) -> u32 {
    380.max(100 + entries as u32 * LEGEND_ROW as u32)
}

fn legend(area: &Area, entries: &[(String, RGBAColor, Swatch)]) -> Result {
    for (index, (label, color, swatch)) in entries.iter().enumerate() {
        let y = 70 + index as i32 * LEGEND_ROW;
        let style = color.stroke_width(3);
        match swatch {
            Swatch::Line => area.draw(&PathElement::new(vec![(10, y), (46, y)], style))?,
            Swatch::Dashed => {
                area.draw(&PathElement::new(vec![(10, y), (24, y)], style))?;
                area.draw(&PathElement::new(vec![(32, y), (46, y)], style))?;
            }
            Swatch::Box => {
                area.draw(&Rectangle::new([(10, y - 9), (46, y + 9)], color.filled()))?
            }
        }
        area.draw(&Text::new(
            label.clone(),
            (58, y),
            dim_font(19).pos(Pos::new(HPos::Left, VPos::Center)),
        ))?;
    }
    Ok(())
}

/// Lines over time, with the run's cycles shaded behind them and the legend beside the plot.
pub fn lines(
    area: &Area,
    title: &str,
    axis: &TimeAxis,
    cycles: &[Cycle],
    unit: Unit,
    series: &[Series],
) -> Result {
    let (plot, side) = area.split_horizontally(WIDTH as i32 - LEGEND);
    let max = series
        .iter()
        .flat_map(|series| series.points.iter().map(|(_, value)| *value))
        .fold(0.0, f64::max);
    let (divisor, unit_label) = unit.scale(max);
    let y_max = if max > 0.0 { max / divisor * 1.08 } else { 1.0 };
    let mut chart = ChartBuilder::on(&plot)
        .caption(title, font(26))
        .margin(14)
        .margin_left(30)
        .x_label_area_size(48)
        .y_label_area_size(Y_LABELS)
        .build_cartesian_2d(axis.range(), 0.0..y_max)?;
    chart.draw_series(cycles.iter().map(|cycle| {
        Rectangle::new(
            [(axis.x(cycle.start_s), 0.0), (axis.x(cycle.end_s), y_max)],
            phase_color(cycle.name).filled(),
        )
    }))?;
    chart
        .configure_mesh()
        .light_line_style(TEXT.mix(0.04))
        .bold_line_style(TEXT.mix(0.09))
        .axis_style(TEXT.mix(0.4))
        .x_desc(axis.label)
        .y_desc(unit_label)
        .x_labels(14)
        .y_labels(8)
        .x_label_formatter(&|x| tick(*x))
        .y_label_formatter(&|y| tick(*y))
        .label_style(dim_font(18))
        .axis_desc_style(dim_font(18))
        .draw()?;
    for series in series {
        // A plot is ~1,900 pixels wide: more points only grow the file.
        let stride = series.points.len().div_ceil(MAX_POINTS).max(1);
        let points = series
            .points
            .iter()
            .step_by(stride)
            .chain(series.points.last().filter(|_| stride > 1))
            .map(|(time, value)| (axis.x(*time), value / divisor));
        let style = series.color.stroke_width(2);
        if series.dashed {
            chart.draw_series(DashedLineSeries::new(points, 12, 8, style))?;
        } else {
            chart.draw_series(LineSeries::new(points, style))?;
        }
    }
    legend(
        &side,
        &series
            .iter()
            .map(|series| {
                let swatch = if series.dashed {
                    Swatch::Dashed
                } else {
                    Swatch::Line
                };
                (series.label.clone(), series.color.to_rgba(), swatch)
            })
            .collect::<Vec<_>>(),
    )
}

pub fn timeline_height(run: &Run) -> u32 {
    120 + plugins(run).len() as u32 * 26
}

fn plugins(run: &Run) -> Vec<&str> {
    let mut plugins = Vec::new();
    for timing in &run.timings {
        if !plugins.contains(&timing.plugin.as_str()) {
            plugins.push(timing.plugin.as_str());
        }
    }
    plugins
}

/// One row per plugin: imports in grey, computes in colour, cycles shaded behind.
pub fn timeline(area: &Area, axis: &TimeAxis, run: &Run) -> Result {
    let plugins = plugins(run);
    let cycles = run.cycles();
    let rows = plugins.len();
    let (plot, side) = area.split_horizontally(WIDTH as i32 - LEGEND);
    let mut chart = ChartBuilder::on(&plot)
        .caption("Plugin timeline", font(26))
        .margin(14)
        .margin_left(30)
        .x_label_area_size(48)
        .y_label_area_size(Y_LABELS)
        .build_cartesian_2d(axis.range(), 0.0..rows as f64)?;
    chart.draw_series(cycles.iter().map(|cycle| {
        Rectangle::new(
            [
                (axis.x(cycle.start_s), 0.0),
                (axis.x(cycle.end_s), rows as f64),
            ],
            phase_color(cycle.name).filled(),
        )
    }))?;
    chart
        .configure_mesh()
        .disable_y_mesh()
        .light_line_style(TEXT.mix(0.04))
        .bold_line_style(TEXT.mix(0.09))
        .axis_style(TEXT.mix(0.4))
        .x_desc(axis.label)
        .x_labels(14)
        .y_labels(0)
        .x_label_formatter(&|x| tick(*x))
        .label_style(dim_font(18))
        .axis_desc_style(dim_font(18))
        .draw()?;
    // Row `i` from the top spans `rows - i - 1 ..= rows - i`.
    let row_y = |plugin: &str| {
        let index = plugins.iter().position(|name| *name == plugin).unwrap_or(0);
        (rows - index - 1) as f64
    };
    chart.draw_series(plugins.iter().map(|plugin| {
        Text::new(
            format!("{plugin} "),
            (axis.x(0.0), row_y(plugin) + 0.5),
            dim_font(18).pos(Pos::new(HPos::Right, VPos::Center)),
        )
    }))?;
    chart.draw_series(run.timings.iter().map(|timing| {
        let y = row_y(&timing.plugin);
        let (inset, color) = if timing.phase == "compute" {
            (
                0.15,
                color(
                    plugins
                        .iter()
                        .position(|p| *p == timing.plugin)
                        .unwrap_or(0),
                )
                .to_rgba(),
            )
        } else {
            (0.32, TEXT.mix(0.45))
        };
        let start = axis.x(timing.start_s);
        // Keep sub-pixel work visible as a sliver.
        let end = axis
            .x(timing.start_s + timing.duration_s)
            .max(start + axis.x(axis.max_s) / 1_500.0);
        Rectangle::new([(start, y + inset), (end, y + 1.0 - inset)], color.filled())
    }))?;
    let mut entries = vec![
        ("bar: import".to_owned(), TEXT.mix(0.45), Swatch::Box),
        (
            "bar: compute (by plugin)".to_owned(),
            color(0).to_rgba(),
            Swatch::Line,
        ),
    ];
    entries.extend(cycles.iter().map(|cycle| {
        let RGBAColor(r, g, b, _) = phase_color(cycle.name);
        (
            format!(
                "cycle: {} {}",
                cycle.name,
                format::duration(cycle.end_s - cycle.start_s)
            ),
            RGBAColor(r, g, b, 0.45),
            Swatch::Box,
        )
    }));
    legend(&side, &entries)
}

pub fn bars_height(rows: usize) -> u32 {
    110 + rows.min(BAR_ROWS) as u32 * 30
}

const BAR_ROWS: usize = 18;

/// Horizontal bars, largest first; the tail beyond `BAR_ROWS` folds into "other".
pub fn bars(
    area: &Area,
    title: &str,
    unit: Unit,
    rows: &[(String, f64)],
    color: RGBColor,
) -> Result {
    let mut rows = rows.to_vec();
    if rows.len() > BAR_ROWS {
        let other = rows
            .split_off(BAR_ROWS - 1)
            .iter()
            .map(|(_, value)| value)
            .sum();
        rows.push(("other".to_owned(), other));
    }
    let max = rows.iter().map(|(_, value)| *value).fold(0.0, f64::max);
    let (divisor, unit_label) = unit.scale(max);
    let count = rows.len();
    let mut chart = ChartBuilder::on(area)
        .caption(title, font(26))
        .margin(14)
        .margin_left(30)
        .margin_right(30)
        .x_label_area_size(48)
        .y_label_area_size(260)
        .build_cartesian_2d(0.0..(max / divisor).max(1e-9) * 1.25, 0.0..count as f64)?;
    chart
        .configure_mesh()
        .disable_y_mesh()
        .light_line_style(TEXT.mix(0.04))
        .bold_line_style(TEXT.mix(0.09))
        .axis_style(TEXT.mix(0.4))
        .x_desc(unit_label)
        .x_labels(8)
        .y_labels(0)
        .x_label_formatter(&|x| tick(*x))
        .label_style(dim_font(18))
        .axis_desc_style(dim_font(18))
        .draw()?;
    let y = |index: usize| (count - index - 1) as f64;
    chart.draw_series(rows.iter().enumerate().map(|(index, (_, value))| {
        Rectangle::new(
            [(0.0, y(index) + 0.18), (value / divisor, y(index) + 0.82)],
            color.filled(),
        )
    }))?;
    chart.draw_series(rows.iter().enumerate().map(|(index, (name, _))| {
        Text::new(
            format!("{name} "),
            (0.0, y(index) + 0.5),
            dim_font(18).pos(Pos::new(HPos::Right, VPos::Center)),
        )
    }))?;
    chart.draw_series(rows.iter().enumerate().map(|(index, (_, value))| {
        let label = match unit {
            Unit::Bytes | Unit::BytesPerSecond => format::bytes(*value),
            Unit::Plain(_) => format::duration(*value),
        };
        Text::new(
            format!(" {label}"),
            (value / divisor, y(index) + 0.5),
            dim_font(17).pos(Pos::new(HPos::Left, VPos::Center)),
        )
    }))?;
    Ok(())
}

pub fn table_height(rows: usize) -> u32 {
    90 + rows as u32 * 30
}

/// A text table; `colors` tints each body row's first cell (the run swatch).
pub fn table(
    area: &Area,
    title: &str,
    header: &[String],
    body: &[Vec<String>],
    colors: &[RGBColor],
) -> Result {
    area.draw(&Text::new(title.to_owned(), (30, 14), font(26)))?;
    let columns = header.len();
    let first = 640;
    let rest = (WIDTH as i32 - first - 60) / (columns as i32 - 1).max(1);
    let x = |column: usize| {
        if column == 0 {
            30
        } else {
            first + (column as i32 - 1) * rest
        }
    };
    for (column, cell) in header.iter().enumerate() {
        area.draw(&Text::new(cell.clone(), (x(column), 60), dim_font(18)))?;
    }
    for (row, cells) in body.iter().enumerate() {
        let y = 92 + row as i32 * 30;
        if let Some(color) = colors.get(row) {
            area.draw(&Rectangle::new([(14, y + 4), (24, y + 18)], color.filled()))?;
        }
        for (column, cell) in cells.iter().enumerate() {
            area.draw(&Text::new(cell.clone(), (x(column), y), dim_font(18)))?;
        }
    }
    Ok(())
}

/// An SVG of `WIDTH` with stacked panels of the given heights.
pub fn render(
    output: &Path,
    heights: &[u32],
    mut draw: impl FnMut(usize, &Area) -> Result,
) -> Result {
    let height = heights.iter().sum::<u32>().max(1);
    let root = SVGBackend::new(output, (WIDTH, height)).into_drawing_area();
    root.fill(&BG)?;
    let mut rest = root.clone();
    for (index, panel_height) in heights.iter().enumerate() {
        let (panel, next) = rest.split_vertically(*panel_height as i32);
        draw(index, &panel)?;
        rest = next;
    }
    root.present()?;
    Ok(())
}
