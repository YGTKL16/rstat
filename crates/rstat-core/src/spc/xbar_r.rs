use crate::error::StatError;
use crate::spc::rules::{SpcViolation, check_rules};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ControlLimits {
    pub lcl: f64,
    pub cl: f64,
    pub ucl: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SubgroupStat {
    pub index: usize,
    pub values: Vec<f64>,
    pub mean: f64,
    pub range: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SpcResult {
    pub subgroup_size: usize,
    pub subgroups: Vec<SubgroupStat>,
    pub xbar_limits: ControlLimits,
    pub r_limits: ControlLimits,
    pub grand_mean: f64,
    pub mean_range: f64,
    pub estimated_sigma: f64,
    pub violations: Vec<SpcViolation>,
}

struct SpcConstants {
    a2: f64,
    d3: f64,
    d4: f64,
    d2: f64,
}

const CONSTANTS: &[SpcConstants] = &[
    SpcConstants {
        a2: 1.880,
        d3: 0.000,
        d4: 3.267,
        d2: 1.128,
    }, // n = 2
    SpcConstants {
        a2: 1.023,
        d3: 0.000,
        d4: 2.574,
        d2: 1.693,
    }, // n = 3
    SpcConstants {
        a2: 0.729,
        d3: 0.000,
        d4: 2.282,
        d2: 2.059,
    }, // n = 4
    SpcConstants {
        a2: 0.577,
        d3: 0.000,
        d4: 2.114,
        d2: 2.326,
    }, // n = 5
    SpcConstants {
        a2: 0.483,
        d3: 0.000,
        d4: 2.004,
        d2: 2.534,
    }, // n = 6
    SpcConstants {
        a2: 0.419,
        d3: 0.076,
        d4: 1.924,
        d2: 2.704,
    }, // n = 7
    SpcConstants {
        a2: 0.373,
        d3: 0.136,
        d4: 1.864,
        d2: 2.847,
    }, // n = 8
    SpcConstants {
        a2: 0.337,
        d3: 0.184,
        d4: 1.816,
        d2: 2.970,
    }, // n = 9
    SpcConstants {
        a2: 0.308,
        d3: 0.223,
        d4: 1.777,
        d2: 3.078,
    }, // n = 10
];

fn get_constants(n: usize) -> Option<&'static SpcConstants> {
    if (2..=10).contains(&n) {
        Some(&CONSTANTS[n - 2])
    } else {
        None
    }
}

pub fn calculate_xbar_r(data: &[f64], subgroup_size: usize) -> Result<SpcResult, StatError> {
    // Check license
    crate::license::check_license()?;

    if !(2..=10).contains(&subgroup_size) {
        return Err(StatError::InvalidParameter(format!(
            "Grup boyutu (subgroup size) 2 ile 10 arasında olmalıdır. Verilen: {}",
            subgroup_size
        )));
    }

    for &x in data {
        if x.is_nan() {
            return Err(StatError::InvalidParameter(
                "Veride NaN değer bulunamaz".into(),
            ));
        }
    }

    let chunks: Vec<&[f64]> = data.chunks_exact(subgroup_size).collect();
    if chunks.is_empty() {
        return Err(StatError::InsufficientData {
            required: subgroup_size,
            got: data.len(),
        });
    }

    let c = get_constants(subgroup_size).ok_or_else(|| {
        StatError::InvalidParameter(format!("Geçersiz grup boyutu: {}", subgroup_size))
    })?;

    let mut subgroups = Vec::with_capacity(chunks.len());
    let mut sum_means = 0.0;
    let mut sum_ranges = 0.0;

    for (index, chunk) in chunks.iter().enumerate() {
        let mean = chunk.iter().sum::<f64>() / subgroup_size as f64;
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for &val in *chunk {
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }
        let range = max - min;

        subgroups.push(SubgroupStat {
            index,
            values: chunk.to_vec(),
            mean,
            range,
        });

        sum_means += mean;
        sum_ranges += range;
    }

    let k = subgroups.len() as f64;
    let grand_mean = sum_means / k;
    let mean_range = sum_ranges / k;
    let estimated_sigma = mean_range / c.d2;

    let xbar_lcl = grand_mean - c.a2 * mean_range;
    let xbar_ucl = grand_mean + c.a2 * mean_range;

    let r_lcl = c.d3 * mean_range;
    let r_ucl = c.d4 * mean_range;

    let means: Vec<f64> = subgroups.iter().map(|s| s.mean).collect();
    let violations = check_rules(&means, xbar_lcl, xbar_ucl, grand_mean);

    Ok(SpcResult {
        subgroup_size,
        subgroups,
        xbar_limits: ControlLimits {
            lcl: xbar_lcl,
            cl: grand_mean,
            ucl: xbar_ucl,
        },
        r_limits: ControlLimits {
            lcl: r_lcl,
            cl: mean_range,
            ucl: r_ucl,
        },
        grand_mean,
        mean_range,
        estimated_sigma,
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xbar_r_calculation_without_license_fails() {
        let _guard = crate::license::ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::remove_var("RSTAT_PRO");
        }
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let res = calculate_xbar_r(&data, 2);
        assert!(res.is_err());
        match res.unwrap_err() {
            StatError::LicenseRequired(_) => {}
            _ => panic!("Expected LicenseRequired error"),
        }
    }

    #[test]
    fn test_xbar_r_calculation_success() {
        let _guard = crate::license::ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("RSTAT_PRO", "1");
        }
        // Subgroup size = 3, n = 9 observations -> 3 subgroups
        let data = vec![
            10.0, 11.0, 12.0, // Subgroup 1: mean = 11.0, range = 2.0
            20.0, 21.0, 22.0, // Subgroup 2: mean = 21.0, range = 2.0
            30.0, 31.0, 32.0, // Subgroup 3: mean = 31.0, range = 2.0
        ];
        let res = calculate_xbar_r(&data, 3).unwrap();
        assert_eq!(res.subgroup_size, 3);
        assert_eq!(res.subgroups.len(), 3);
        assert_eq!(res.grand_mean, 21.0);
        assert_eq!(res.mean_range, 2.0);

        // n = 3 constants: a2 = 1.023, d2 = 1.693, d3 = 0.0, d4 = 2.574
        assert!((res.xbar_limits.lcl - (21.0 - 1.023 * 2.0)).abs() < 1e-9);
        assert!((res.xbar_limits.ucl - (21.0 + 1.023 * 2.0)).abs() < 1e-9);
        assert!((res.estimated_sigma - (2.0 / 1.693)).abs() < 1e-9);

        assert_eq!(res.r_limits.lcl, 0.0);
        assert!((res.r_limits.ucl - (2.574 * 2.0)).abs() < 1e-9);

        // Violations: Subgroup 1 and Subgroup 3 are way out of limits (LCL=18.954, UCL=23.046)
        // Subgroup 1 mean is 11.0, Subgroup 3 mean is 31.0
        assert_eq!(res.violations.len(), 2);
        assert_eq!(res.violations[0].rule_id, 1);
        assert_eq!(res.violations[0].subgroup_index, 0);
        assert_eq!(res.violations[1].rule_id, 1);
        assert_eq!(res.violations[1].subgroup_index, 2);
    }
}
