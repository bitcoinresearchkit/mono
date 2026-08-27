use super::DataPoint;

#[derive(Debug, Clone)]
pub struct Run {
    pub id: String,
    pub data: Vec<DataPoint>,
}

impl Run {
    pub fn max_timestamp(&self) -> u64 {
        self.data
            .iter()
            .map(|point| point.timestamp_ms)
            .max()
            .unwrap_or(0)
    }

    pub fn max_value(&self) -> f64 {
        self.data
            .iter()
            .map(|point| point.value)
            .fold(0.0, f64::max)
    }

    pub fn trimmed(&self, max_timestamp_ms: u64) -> Self {
        Self {
            id: self.id.clone(),
            data: self
                .data
                .iter()
                .filter(|point| point.timestamp_ms <= max_timestamp_ms)
                .cloned()
                .collect(),
        }
    }
}
