use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SummaryStats {
    pub n: usize,
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
}

#[derive(Debug, Serialize)]
pub struct TTestResult {
    pub test: &'static str,
    pub method: &'static str,
    pub alternative: String,
    pub groups: Vec<GroupStats>,
    pub statistic: f64,
    pub df: f64,
    pub p_value: f64,
    pub mean_diff: Option<f64>,
    pub ci: Option<[f64; 2]>,
    pub ci_level: f64,
    pub cohens_d: Option<f64>,
    pub alpha: f64,
    pub reject_null: bool,
}

#[derive(Debug, Serialize)]
pub struct GroupStats {
    pub name: String,
    pub n: usize,
    pub mean: f64,
    pub std: f64,
}
