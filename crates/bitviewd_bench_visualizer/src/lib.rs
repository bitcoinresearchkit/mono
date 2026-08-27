mod chart;
mod data;
mod format;

use data::{Cutoffs, DualRun, Run, read_dual_runs, read_runs};
use std::{
    error::Error,
    path::{Path, PathBuf},
    slice,
};

pub struct Visualizer {
    workspace_root: PathBuf,
}

impl Visualizer {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }

    pub fn from_cargo_env() -> Result<Self, Box<dyn Error>> {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("Failed to find workspace root")?
            .to_path_buf();
        Ok(Self { workspace_root })
    }

    pub fn generate(&self) -> Result<(), Box<dyn Error>> {
        let path = self.workspace_root.join("benches").join("bitviewd");
        if !path.exists() {
            return Err("bitviewd benchmark directory does not exist".into());
        }
        self.generate_crate_charts(&path, "bitviewd")
    }

    fn generate_crate_charts(
        &self,
        crate_path: &Path,
        crate_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let disk_runs = read_runs(crate_path, "disk.csv")?;
        let memory_runs = read_dual_runs(crate_path, "memory.csv")?;
        let progress_runs = read_runs(crate_path, "progress.csv")?;
        let io_runs = read_dual_runs(crate_path, "io.csv")?;

        // Combined charts (all runs)
        self.generate_combined_charts(
            crate_path,
            crate_name,
            &disk_runs,
            &memory_runs,
            &progress_runs,
            &io_runs,
        )?;

        // Individual charts (one per run)
        self.generate_individual_charts(
            crate_path,
            crate_name,
            &disk_runs,
            &memory_runs,
            &progress_runs,
            &io_runs,
        )?;

        Ok(())
    }

    fn generate_combined_charts(
        &self,
        crate_path: &Path,
        crate_name: &str,
        disk_runs: &[Run],
        memory_runs: &[DualRun],
        progress_runs: &[Run],
        io_runs: &[DualRun],
    ) -> Result<(), Box<dyn Error>> {
        let cutoffs = Cutoffs::from_progress(progress_runs);

        // Trim data to per-run cutoffs for fair comparison
        let disk_trimmed = cutoffs.trim_runs(disk_runs);
        let memory_trimmed = cutoffs.trim_dual_runs(memory_runs);
        let io_trimmed = cutoffs.trim_dual_runs(io_runs);

        if !disk_trimmed.is_empty() {
            chart::generate(
                chart::ChartConfig {
                    output_path: &crate_path.join("disk.svg"),
                    title: format!("{} — Disk Usage", crate_name),
                    y_label: "Disk Usage".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                &disk_trimmed,
            )?;
        }

        if !memory_trimmed.is_empty() {
            chart::generate_dual(
                chart::ChartConfig {
                    output_path: &crate_path.join("memory.svg"),
                    title: format!("{} — Memory", crate_name),
                    y_label: "Memory".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                &memory_trimmed,
                "(current)",
                "(peak)",
            )?;
        }

        if !progress_runs.is_empty() {
            let progress_trimmed = cutoffs.trim_runs(progress_runs);
            chart::generate(
                chart::ChartConfig {
                    output_path: &crate_path.join("progress.svg"),
                    title: format!("{} — Progress", crate_name),
                    y_label: "Progress".to_string(),
                    y_format: chart::YAxisFormat::Number,
                },
                &progress_trimmed,
            )?;
        }

        if !io_trimmed.is_empty() {
            // I/O Read (primary column)
            let io_read: Vec<_> = io_trimmed
                .iter()
                .map(|r| Run {
                    id: r.id.clone(),
                    data: r.primary.clone(),
                })
                .collect();
            chart::generate(
                chart::ChartConfig {
                    output_path: &crate_path.join("io_read.svg"),
                    title: format!("{} — I/O Read", crate_name),
                    y_label: "Bytes Read".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                &io_read,
            )?;

            // I/O Write (secondary column)
            let io_write: Vec<_> = io_trimmed
                .iter()
                .map(|r| Run {
                    id: r.id.clone(),
                    data: r.secondary.clone(),
                })
                .collect();
            chart::generate(
                chart::ChartConfig {
                    output_path: &crate_path.join("io_write.svg"),
                    title: format!("{} — I/O Write", crate_name),
                    y_label: "Bytes Written".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                &io_write,
            )?;
        }

        Ok(())
    }

    fn generate_individual_charts(
        &self,
        crate_path: &Path,
        crate_name: &str,
        disk_runs: &[Run],
        memory_runs: &[DualRun],
        progress_runs: &[Run],
        io_runs: &[DualRun],
    ) -> Result<(), Box<dyn Error>> {
        for run in disk_runs {
            let run_path = crate_path.join(&run.id);
            chart::generate(
                chart::ChartConfig {
                    output_path: &run_path.join("disk.svg"),
                    title: format!("{} — Disk Usage", crate_name),
                    y_label: "Disk Usage".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                slice::from_ref(run),
            )?;
        }

        for run in memory_runs {
            let run_path = crate_path.join(&run.id);
            chart::generate_dual(
                chart::ChartConfig {
                    output_path: &run_path.join("memory.svg"),
                    title: format!("{} — Memory", crate_name),
                    y_label: "Memory".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                slice::from_ref(run),
                "(current)",
                "(peak)",
            )?;
        }

        for run in progress_runs {
            let run_path = crate_path.join(&run.id);
            chart::generate(
                chart::ChartConfig {
                    output_path: &run_path.join("progress.svg"),
                    title: format!("{} — Progress", crate_name),
                    y_label: "Progress".to_string(),
                    y_format: chart::YAxisFormat::Number,
                },
                slice::from_ref(run),
            )?;
        }

        for run in io_runs {
            let run_path = crate_path.join(&run.id);

            let read_run = Run {
                id: run.id.clone(),
                data: run.primary.clone(),
            };
            chart::generate(
                chart::ChartConfig {
                    output_path: &run_path.join("io_read.svg"),
                    title: format!("{} — I/O Read", crate_name),
                    y_label: "Bytes Read".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                slice::from_ref(&read_run),
            )?;

            let write_run = Run {
                id: run.id.clone(),
                data: run.secondary.clone(),
            };
            chart::generate(
                chart::ChartConfig {
                    output_path: &run_path.join("io_write.svg"),
                    title: format!("{} — I/O Write", crate_name),
                    y_label: "Bytes Written".to_string(),
                    y_format: chart::YAxisFormat::Bytes,
                },
                slice::from_ref(&write_run),
            )?;
        }

        Ok(())
    }
}
