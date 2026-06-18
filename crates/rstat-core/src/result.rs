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

#[derive(Debug, Serialize)]
pub struct AnovaResult {
    pub test: &'static str, // "one-way-anova"
    pub groups: Vec<GroupStats>,
    // SS decomposition
    pub ss_between: f64,
    pub ss_within: f64,
    pub ss_total: f64,
    // Degrees of freedom
    pub df_between: f64,
    pub df_within: f64,
    // Mean squares
    pub ms_between: f64,
    pub ms_within: f64,
    // Test istatistiği
    pub f_statistic: f64,
    pub p_value: f64,
    pub eta_squared: f64,
    pub alpha: f64,
    pub reject_null: bool,
}

#[derive(Debug, Serialize)]
pub struct MeanCiResult {
    pub n: usize,
    pub mean: f64,
    pub std: f64,
    pub se: f64,
    pub df: f64,
    pub ci_level: f64,
    pub ci: [f64; 2],
}

#[derive(Debug, Serialize)]
pub struct ProportionCiResult {
    pub successes: u64,
    pub trials: u64,
    pub p_hat: f64,
    pub ci_level: f64,
    pub method: String, // "wilson", "wald"
    pub ci: [f64; 2],
}

#[derive(Debug, Serialize)]
pub struct VarianceCiResult {
    pub n: usize,
    pub variance: f64,
    pub std_dev: f64,
    pub df: f64,
    pub ci_level: f64,
    pub ci_variance: [f64; 2],
    pub ci_std_dev: [f64; 2],
}

#[derive(Debug, Serialize)]
pub struct ChiSqResult {
    pub test: &'static str,
    pub statistic: f64,
    pub df: f64,
    pub p_value: f64,
    pub observed: Vec<Vec<f64>>,
    pub expected: Vec<Vec<f64>>,
    pub cramers_v: Option<f64>,
    pub yates_corrected: bool,
    pub warning_low_expected: bool,
    pub alpha: f64,
    pub reject_null: bool,
}
