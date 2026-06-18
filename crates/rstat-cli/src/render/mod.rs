pub mod anova;
pub mod capability;
pub mod chisq;
pub mod ci;
pub mod spc;
pub mod summary;
pub mod ttest;

pub use anova::print_anova;
pub use capability::print_capability;
pub use chisq::print_chisq;
pub use ci::{print_mean_ci, print_proportion_ci, print_variance_ci};
pub use spc::print_spc;
pub use summary::print_summary;
pub use ttest::print_ttest;

pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl OutputFormat {
    /// TTY'de tablo, pipe'ta JSON
    pub fn detect(flag: Option<&str>) -> Self {
        match flag {
            Some("json") => Self::Json,
            Some("csv") => Self::Csv,
            Some("table") => Self::Table,
            _ => {
                if atty::is(atty::Stream::Stdout) {
                    Self::Table
                } else {
                    Self::Json
                }
            }
        }
    }
}
