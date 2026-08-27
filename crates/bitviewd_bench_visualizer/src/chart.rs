use crate::data::{DataPoint, DualRun, Run};
use crate::format;
use derive_more::{Deref, DerefMut};
use plotters::{coord::types::RangedCoordf64, prelude::*};
use std::{error::Error, path::Path};

const FONT: &str = "monospace";
const FONT_SIZE: i32 = 20;
const FONT_SIZE_BIG: i32 = 30;
const SIZE: (u32, u32) = (2000, 1000);
const TIME_BUFFER_MS: u64 = 10_000;

const BG_COLOR: RGBColor = RGBColor(18, 18, 24);
const TEXT_COLOR: RGBColor = RGBColor(230, 230, 240);
const COLORS: [RGBColor; 6] = [
    RGBColor(255, 99, 132),  // Pink/Red
    RGBColor(54, 162, 235),  // Blue
    RGBColor(75, 192, 192),  // Teal
    RGBColor(255, 206, 86),  // Yellow
    RGBColor(153, 102, 255), // Purple
    RGBColor(255, 159, 64),  // Orange
];

pub enum YAxisFormat {
    Bytes,
    Number,
}

pub struct ChartConfig<'a> {
    pub output_path: &'a Path,
    pub title: String,
    pub y_label: String,
    pub y_format: YAxisFormat,
}

/// Generate a simple line chart from runs
pub fn generate(config: ChartConfig, runs: &[Run]) -> Result<(), Box<dyn Error>> {
    if runs.is_empty() {
        return Ok(());
    }

    let max_time_ms = runs.iter().map(|r| r.max_timestamp()).max().unwrap_or(1000) + TIME_BUFFER_MS;
    let max_time_s = max_time_ms as f64 / 1000.0;
    let max_value = runs.iter().map(|r| r.max_value()).fold(0.0, f64::max);

    let (time_scaled, time_divisor, time_label) = format::time(max_time_s);
    let (value_scaled, scale_factor, y_label) =
        scale_y_axis(max_value, &config.y_label, &config.y_format);
    let x_labels = label_count(time_scaled);

    let root = SVGBackend::new(config.output_path, SIZE).into_drawing_area();
    root.fill(&BG_COLOR)?;

    let mut chart = Chart(
        ChartBuilder::on(&root)
            .caption(
                &config.title,
                (FONT, FONT_SIZE_BIG).into_font().color(&TEXT_COLOR),
            )
            .margin(20)
            .margin_right(40)
            .x_label_area_size(50)
            .margin_left(50)
            .right_y_label_area_size(75)
            .build_cartesian_2d(0.0..time_scaled * 1.025, 0.0..value_scaled * 1.1)?,
    );

    configure_mesh(&mut chart, time_label, &y_label, &config.y_format, x_labels)?;

    for (idx, run) in runs.iter().enumerate() {
        let color = COLORS[idx % COLORS.len()];
        draw_series(
            &mut chart,
            &run.data,
            &run.id,
            color,
            time_divisor,
            scale_factor,
        )?;
    }

    configure_legend(&mut chart)?;
    root.present()?;
    println!("Generated: {}", config.output_path.display());
    Ok(())
}

/// Generate a chart with dual series per run (e.g., current + peak memory)
pub fn generate_dual(
    config: ChartConfig,
    runs: &[DualRun],
    primary_suffix: &str,
    secondary_suffix: &str,
) -> Result<(), Box<dyn Error>> {
    if runs.is_empty() {
        return Ok(());
    }

    let max_time_ms = runs
        .iter()
        .flat_map(|r| r.primary.iter().chain(r.secondary.iter()))
        .map(|d| d.timestamp_ms)
        .max()
        .unwrap_or(1000)
        + TIME_BUFFER_MS;
    let max_time_s = max_time_ms as f64 / 1000.0;
    let max_value = runs.iter().map(|r| r.max_value()).fold(0.0, f64::max);

    let (time_scaled, time_divisor, time_label) = format::time(max_time_s);
    let (value_scaled, scale_factor, y_label) =
        scale_y_axis(max_value, &config.y_label, &config.y_format);
    let x_labels = label_count(time_scaled);

    let root = SVGBackend::new(config.output_path, SIZE).into_drawing_area();
    root.fill(&BG_COLOR)?;

    let mut chart = Chart(
        ChartBuilder::on(&root)
            .caption(
                &config.title,
                (FONT, FONT_SIZE_BIG).into_font().color(&TEXT_COLOR),
            )
            .margin(20)
            .margin_right(40)
            .x_label_area_size(50)
            .margin_left(50)
            .right_y_label_area_size(75)
            .build_cartesian_2d(0.0..time_scaled * 1.025, 0.0..value_scaled * 1.1)?,
    );

    configure_mesh(&mut chart, time_label, &y_label, &config.y_format, x_labels)?;

    for (idx, run) in runs.iter().enumerate() {
        let color = COLORS[idx % COLORS.len()];

        // Primary series (solid)
        draw_series(
            &mut chart,
            &run.primary,
            &format!("{} {}", run.id, primary_suffix),
            color,
            time_divisor,
            scale_factor,
        )?;

        // Secondary series (dashed)
        draw_dashed_series(
            &mut chart,
            &run.secondary,
            &format!("{} {}", run.id, secondary_suffix),
            color.mix(0.5),
            time_divisor,
            scale_factor,
        )?;
    }

    configure_legend(&mut chart)?;
    root.present()?;
    println!("Generated: {}", config.output_path.display());
    Ok(())
}

fn scale_y_axis(max_value: f64, base_label: &str, y_format: &YAxisFormat) -> (f64, f64, String) {
    match y_format {
        YAxisFormat::Bytes => {
            let (scaled, unit) = format::bytes(max_value);
            let factor = max_value / scaled;
            (scaled, factor, format!("{} ({})", base_label, unit))
        }
        YAxisFormat::Number => (max_value, 1.0, base_label.to_string()),
    }
}

/// Calculate appropriate label count to avoid duplicates when rounding to integers
fn label_count(max_value: f64) -> usize {
    let max_int = max_value.ceil() as usize;
    // Don't exceed the range, cap at 12 for readability
    max_int.clamp(2, 12)
}

#[derive(Deref, DerefMut)]
struct Chart<'a, 'b>(ChartContext<'a, SVGBackend<'b>, Cartesian2d<RangedCoordf64, RangedCoordf64>>);

fn configure_mesh(
    chart: &mut Chart,
    x_label: &str,
    y_label: &str,
    y_format: &YAxisFormat,
    x_labels: usize,
) -> Result<(), Box<dyn Error>> {
    let y_formatter = |y: &f64| match y_format {
        YAxisFormat::Bytes => {
            if y.fract() == 0.0 {
                format!("{:.0}", y)
            } else {
                format!("{:.1}", y)
            }
        }
        YAxisFormat::Number => format::axis_number(*y),
    };

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc(x_label)
        .y_desc(y_label)
        .x_label_formatter(&|x| format!("{:.0}", x))
        .y_label_formatter(&y_formatter)
        .x_labels(x_labels)
        .y_labels(10)
        .x_label_style((FONT, FONT_SIZE).into_font().color(&TEXT_COLOR.mix(0.7)))
        .y_label_style((FONT, FONT_SIZE).into_font().color(&TEXT_COLOR.mix(0.7)))
        .axis_style(TEXT_COLOR.mix(0.3))
        .draw()?;
    Ok(())
}

fn draw_series(
    chart: &mut Chart,
    data: &[DataPoint],
    label: &str,
    color: RGBColor,
    time_divisor: f64,
    scale_factor: f64,
) -> Result<(), Box<dyn Error>> {
    let points = data.iter().map(|d| {
        (
            d.timestamp_ms as f64 / 1000.0 / time_divisor,
            d.value / scale_factor,
        )
    });

    chart
        .draw_series(LineSeries::new(points, color.stroke_width(1)))?
        .label(label)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(1)));
    Ok(())
}

fn draw_dashed_series(
    chart: &mut Chart,
    data: &[DataPoint],
    label: &str,
    color: RGBAColor,
    time_divisor: f64,
    scale_factor: f64,
) -> Result<(), Box<dyn Error>> {
    let points: Vec<_> = data
        .iter()
        .map(|d| {
            (
                d.timestamp_ms as f64 / 1000.0 / time_divisor,
                d.value / scale_factor,
            )
        })
        .collect();

    // Draw dashed line by skipping every other segment
    chart
        .draw_series(
            points
                .windows(2)
                .enumerate()
                .filter(|(i, _)| i % 2 == 0)
                .map(|(_, w)| PathElement::new(vec![w[0], w[1]], color.stroke_width(2))),
        )?
        .label(label)
        .legend(move |(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 10, y), (x + 20, y)],
                color.stroke_width(2),
            )
        });
    Ok(())
}

fn configure_legend<'a>(chart: &mut Chart<'a, 'a>) -> Result<(), Box<dyn Error>> {
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .label_font((FONT, FONT_SIZE).into_font().color(&TEXT_COLOR.mix(0.9)))
        .background_style(BG_COLOR.mix(0.98))
        .border_style(BG_COLOR)
        .margin(10)
        .draw()?;
    Ok(())
}
