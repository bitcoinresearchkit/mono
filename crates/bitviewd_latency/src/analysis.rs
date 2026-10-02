use crate::{Result, group::Group, record::Record, routes};
use jiff::civil::DateTime;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Default)]
pub struct Analysis {
    pub files: Vec<PathBuf>,
    pub groups: BTreeMap<(String, u16), Group>,
    pub lines: usize,
    pub ignored: usize,
    pub malformed: usize,
    pub unmatched: usize,
    pub count: usize,
    pub errors: usize,
    pub first: Option<DateTime>,
    pub last: Option<DateTime>,
}

impl Analysis {
    pub fn read(files: &[PathBuf]) -> Result<Self> {
        let catalog = routes::catalog();
        let mut analysis = Self {
            files: files.to_vec(),
            ..Self::default()
        };
        for path in files {
            for line in BufReader::new(File::open(path)?).lines() {
                let line = line?;
                analysis.lines += 1;
                let record = match Record::parse(&line) {
                    Ok(Some(record)) => record,
                    Ok(None) => {
                        analysis.ignored += 1;
                        continue;
                    }
                    Err(()) => {
                        analysis.malformed += 1;
                        continue;
                    }
                };
                analysis.first = Some(
                    analysis
                        .first
                        .map_or(record.timestamp, |t| t.min(record.timestamp)),
                );
                analysis.last = Some(
                    analysis
                        .last
                        .map_or(record.timestamp, |t| t.max(record.timestamp)),
                );
                analysis.count += 1;
                analysis.errors += usize::from(record.status >= 400);
                let endpoint = routes::endpoint(&record.uri, &catalog);
                let matched = catalog.contains(&endpoint);
                if !matched {
                    analysis.unmatched += 1;
                }
                let group = analysis
                    .groups
                    .entry((endpoint.to_string(), record.status))
                    .or_default();
                group.matched = matched;
                group.push(record);
            }
        }
        for group in analysis.groups.values_mut() {
            group.durations.sort_unstable();
        }
        Ok(analysis)
    }
}
