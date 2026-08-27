use super::DataPoint;

#[derive(Debug, Clone)]
pub struct DualRun {
    pub id: String,
    pub primary: Vec<DataPoint>,
    pub secondary: Vec<DataPoint>,
}

impl DualRun {
    pub fn max_value(&self) -> f64 {
        self.primary
            .iter()
            .chain(&self.secondary)
            .map(|point| point.value)
            .fold(0.0, f64::max)
    }

    pub fn trimmed(&self, max_timestamp_ms: u64) -> Self {
        Self {
            id: self.id.clone(),
            primary: self
                .primary
                .iter()
                .filter(|point| point.timestamp_ms <= max_timestamp_ms)
                .cloned()
                .collect(),
            secondary: self
                .secondary
                .iter()
                .filter(|point| point.timestamp_ms <= max_timestamp_ms)
                .cloned()
                .collect(),
        }
    }
}
