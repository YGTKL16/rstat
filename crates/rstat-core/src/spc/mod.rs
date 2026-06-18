pub mod rules;
pub mod xbar_r;

pub use rules::{SpcViolation, check_rules};
pub use xbar_r::{ControlLimits, SpcResult, SubgroupStat, calculate_xbar_r};
