use crate::data::summary::{mean, std_dev};
use crate::error::StatError;
use crate::spc::calculate_xbar_r;
use statrs::distribution::{ContinuousCDF, Normal};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CapabilityResult {
    pub cp: Option<f64>,
    pub cpk: f64,
    pub pp: Option<f64>,
    pub ppk: f64,
    pub ppm_lcl: f64,
    pub ppm_usl: f64,
    pub ppm_total: f64,
    pub within_sigma: f64,
    pub overall_sigma: f64,
    pub mean: f64,
    pub skewness: f64,
    pub normality_warning: bool,
}

pub fn skewness(data: &[f64]) -> Result<f64, StatError> {
    if data.len() < 3 {
        return Err(StatError::InsufficientData {
            required: 3,
            got: data.len(),
        });
    }
    let m = mean(data)?;
    let mut sum2 = 0.0;
    let mut sum3 = 0.0;
    for &x in data {
        let diff = x - m;
        sum2 += diff.powi(2);
        sum3 += diff.powi(3);
    }
    let n = data.len() as f64;
    let m2 = sum2 / n;
    let m3 = sum3 / n;
    if m2 == 0.0 {
        return Err(StatError::Numerical(
            "Varyans sıfır olduğundan skewness hesaplanamaz".into(),
        ));
    }
    Ok(m3 / m2.powf(1.5))
}

pub fn calculate_capability(
    data: &[f64],
    subgroup_size: usize,
    lsl: Option<f64>,
    usl: Option<f64>,
    _target: Option<f64>, // target variable reserved/optional
) -> Result<CapabilityResult, StatError> {
    // License check
    crate::license::check_license()?;

    if lsl.is_none() && usl.is_none() {
        return Err(StatError::InvalidParameter(
            "LSL (alt spesifikasyon limiti) veya USL (üst spesifikasyon limiti) değerlerinden en az biri tanımlanmalıdır.".into()
        ));
    }

    for &x in data {
        if x.is_nan() {
            return Err(StatError::InvalidParameter(
                "Veride NaN değer bulunamaz".into(),
            ));
        }
    }

    let m = mean(data)?;
    let s_overall = std_dev(data)?;
    if s_overall == 0.0 {
        return Err(StatError::Numerical(
            "Genel standart sapma sıfır, indeksler hesaplanamaz".into(),
        ));
    }

    // Reuse SPC calculations for within-subgroup variation
    let spc_res = calculate_xbar_r(data, subgroup_size)?;
    let s_within = spc_res.estimated_sigma;
    if s_within == 0.0 {
        return Err(StatError::Numerical(
            "Grup içi standart sapma sıfır, indeksler hesaplanamaz".into(),
        ));
    }

    // Cp and Pp (only defined if both specification limits are present)
    let cp = if let (Some(l), Some(u)) = (lsl, usl) {
        if u <= l {
            return Err(StatError::InvalidParameter(
                "USL değeri LSL değerinden büyük olmalıdır".into(),
            ));
        }
        Some((u - l) / (6.0 * s_within))
    } else {
        None
    };

    let pp = if let (Some(l), Some(u)) = (lsl, usl) {
        Some((u - l) / (6.0 * s_overall))
    } else {
        None
    };

    // Cpk
    let cpk = match (lsl, usl) {
        (Some(l), Some(u)) => {
            let cpu = (u - m) / (3.0 * s_within);
            let cpl = (m - l) / (3.0 * s_within);
            cpu.min(cpl)
        }
        (Some(l), None) => (m - l) / (3.0 * s_within),
        (None, Some(u)) => (u - m) / (3.0 * s_within),
        (None, None) => unreachable!(),
    };

    // Ppk
    let ppk = match (lsl, usl) {
        (Some(l), Some(u)) => {
            let ppu = (u - m) / (3.0 * s_overall);
            let ppl = (m - l) / (3.0 * s_overall);
            ppu.min(ppl)
        }
        (Some(l), None) => (m - l) / (3.0 * s_overall),
        (None, Some(u)) => (u - m) / (3.0 * s_overall),
        (None, None) => unreachable!(),
    };

    // PPM out-of-spec estimates using overall normal distribution
    let norm = Normal::new(m, s_overall)
        .map_err(|e| StatError::Numerical(format!("Normal dağılım oluşturulamadı: {}", e)))?;

    let ppm_lcl = if let Some(l) = lsl {
        norm.cdf(l) * 1_000_000.0
    } else {
        0.0
    };

    let ppm_usl = if let Some(u) = usl {
        (1.0 - norm.cdf(u)) * 1_000_000.0
    } else {
        0.0
    };

    let ppm_total = ppm_lcl + ppm_usl;

    // Skewness and normality warning
    let skew = skewness(data)?;
    let normality_warning = skew.abs() > 1.0;

    Ok(CapabilityResult {
        cp,
        cpk,
        pp,
        ppk,
        ppm_lcl,
        ppm_usl,
        ppm_total,
        within_sigma: s_within,
        overall_sigma: s_overall,
        mean: m,
        skewness: skew,
        normality_warning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_without_license_fails() {
        let _guard = crate::license::ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::remove_var("RSTAT_PRO");
        }
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let res = calculate_capability(&data, 2, Some(1.0), Some(5.0), None);
        assert!(res.is_err());
        match res.unwrap_err() {
            StatError::LicenseRequired(_) => {}
            _ => panic!("Expected LicenseRequired error"),
        }
    }

    #[test]
    fn test_capability_calculation_success() {
        let _guard = crate::license::ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("RSTAT_PRO", "1");
        }
        // Let's create a dataset of 10 values, subgroup size = 2
        // Subgroups: [2.0, 3.0] (r=1.0), [4.0, 5.0] (r=1.0), [6.0, 7.0] (r=1.0), [8.0, 9.0] (r=1.0), [10.0, 11.0] (r=1.0)
        // Mean range R-bar = 1.0
        // d2 for n=2 is 1.128 -> s_within = 1.0 / 1.128 = 0.8865257
        let data = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
        // overall mean = 6.5
        // overall std_dev = sqrt( variance(data) ) = 3.02765
        let res = calculate_capability(&data, 2, Some(0.0), Some(13.0), None).unwrap();

        assert!(res.cp.is_some());
        let cp = res.cp.unwrap();
        // Cp = (13 - 0) / (6 * 0.8865257) = 13 / 5.319154 = 2.4439977
        assert!((cp - 2.4439977).abs() < 1e-5);

        // Cpk = min( (13-6.5)/(3*s_within), (6.5-0)/(3*s_within) ) = 6.5 / (3 * 0.8865257) = 6.5 / 2.659577 = 2.4439977
        assert!((res.cpk - 2.4439977).abs() < 1e-5);

        assert!(res.pp.is_some());
        let pp = res.pp.unwrap();
        // Pp = 13 / (6 * 3.02765) = 13 / 18.1659 = 0.7156
        assert!((pp - 0.7156).abs() < 1e-3);

        // Ppk = 6.5 / (3 * 3.02765) = 0.7156
        assert!((res.ppk - 0.7156).abs() < 1e-3);

        // Skewness of symmetric uniform-like data is 0.0
        assert!(res.skewness.abs() < 1e-9);
        assert!(!res.normality_warning);
    }
}
