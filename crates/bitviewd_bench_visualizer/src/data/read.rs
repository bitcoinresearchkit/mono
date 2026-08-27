use std::{error::Error, fs, path::Path};

use super::{DataPoint, DualRun, Run};

pub fn read_runs(crate_path: &Path, filename: &str) -> Result<Vec<Run>, Box<dyn Error>> {
    let mut runs = Vec::new();
    for entry in fs::read_dir(crate_path)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }

        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid run ID")?
            .to_owned();
        if id.starts_with('_') || id.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }

        let csv = path.join(filename);
        if csv.exists()
            && let Ok(data) = read_csv(&csv)
        {
            runs.push(Run { id, data });
        }
    }
    Ok(runs)
}

pub fn read_dual_runs(crate_path: &Path, filename: &str) -> Result<Vec<DualRun>, Box<dyn Error>> {
    let mut runs = Vec::new();
    for entry in fs::read_dir(crate_path)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }

        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid run ID")?
            .to_owned();
        if id.starts_with('_') || id.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }

        let csv = path.join(filename);
        if csv.exists()
            && let Ok((primary, secondary)) = read_dual_csv(&csv)
        {
            runs.push(DualRun {
                id,
                primary,
                secondary,
            });
        }
    }
    Ok(runs)
}

fn read_csv(path: &Path) -> Result<Vec<DataPoint>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.split(',');
            Some(DataPoint {
                timestamp_ms: fields.next()?.parse().ok()?,
                value: fields.next()?.parse().ok()?,
            })
        })
        .collect())
}

fn read_dual_csv(path: &Path) -> Result<(Vec<DataPoint>, Vec<DataPoint>), Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut primary = Vec::new();
    let mut secondary = Vec::new();

    for line in content.lines().skip(1) {
        let mut fields = line.split(',');
        if let (Some(timestamp), Some(first), Some(second)) =
            (fields.next(), fields.next(), fields.next())
            && let (Ok(timestamp_ms), Ok(first), Ok(second)) =
                (timestamp.parse(), first.parse(), second.parse())
        {
            primary.push(DataPoint {
                timestamp_ms,
                value: first,
            });
            secondary.push(DataPoint {
                timestamp_ms,
                value: second,
            });
        }
    }

    Ok((primary, secondary))
}
