use crate::data::summary::{mean, std_dev, variance};
use crate::dist::pvalue::Alternative;
use crate::dist::pvalue::ci_bounds;
use crate::error::StatError;
use crate::result::{MeanCiResult, ProportionCiResult, VarianceCiResult};
use statrs::distribution::{ChiSquared, ContinuousCDF, Normal};

/// Ortalama için Güven Aralığı (t-dağılımı bazlı).
pub fn mean_ci(data: &[f64], level: f64) -> Result<MeanCiResult, StatError> {
    let n = data.len();
    if n < 2 {
        return Err(StatError::InsufficientData {
            required: 2,
            got: n,
        });
    }
    if level <= 0.0 || level >= 1.0 {
        return Err(StatError::InvalidParameter(
            "level (0, 1) aralığında olmalı".into(),
        ));
    }

    let m = mean(data)?;
    let s = std_dev(data)?;
    let se = s / (n as f64).sqrt();
    let df = (n - 1) as f64;

    let bounds = ci_bounds(m, se, df, level, Alternative::TwoSided)?;

    Ok(MeanCiResult {
        n,
        mean: m,
        std: s,
        se,
        df,
        ci_level: level,
        ci: bounds,
    })
}

/// Oran için Güven Aralığı (Wald ve Wilson score yöntemleri).
pub fn proportion_ci(
    successes: u64,
    trials: u64,
    level: f64,
    method: &str,
) -> Result<ProportionCiResult, StatError> {
    if trials == 0 {
        return Err(StatError::InvalidParameter("deneme sayısı 0 olamaz".into()));
    }
    if successes > trials {
        return Err(StatError::InvalidParameter(
            "başarı sayısı deneme sayısından büyük olamaz".into(),
        ));
    }
    if level <= 0.0 || level >= 1.0 {
        return Err(StatError::InvalidParameter(
            "level (0, 1) aralığında olmalı".into(),
        ));
    }

    let p_hat = successes as f64 / trials as f64;
    let n = trials as f64;

    let z_dist = Normal::new(0.0, 1.0)
        .map_err(|e| StatError::Numerical(format!("Normal dağılım kurulamadı: {e}")))?;
    let z = z_dist.inverse_cdf(1.0 - (1.0 - level) / 2.0);

    let bounds = match method.to_ascii_lowercase().as_str() {
        "wald" => {
            let se = (p_hat * (1.0 - p_hat) / n).sqrt();
            let low = (p_hat - z * se).clamp(0.0, 1.0);
            let high = (p_hat + z * se).clamp(0.0, 1.0);
            [low, high]
        }
        "wilson" => {
            let denom = 1.0 + z.powi(2) / n;
            let center = (p_hat + z.powi(2) / (2.0 * n)) / denom;
            let spread =
                z * (p_hat * (1.0 - p_hat) / n + z.powi(2) / (4.0 * n.powi(2))).sqrt() / denom;
            let low = (center - spread).clamp(0.0, 1.0);
            let high = (center + spread).clamp(0.0, 1.0);
            [low, high]
        }
        other => {
            return Err(StatError::InvalidParameter(format!(
                "bilinmeyen yöntem: {other} (wilson|wald)"
            )));
        }
    };

    Ok(ProportionCiResult {
        successes,
        trials,
        p_hat,
        ci_level: level,
        method: method.to_string(),
        ci: bounds,
    })
}

/// Varyans ve Standart Sapma için Güven Aralığı (χ² dağılımı bazlı).
pub fn variance_ci(data: &[f64], level: f64) -> Result<VarianceCiResult, StatError> {
    let n = data.len();
    if n < 2 {
        return Err(StatError::InsufficientData {
            required: 2,
            got: n,
        });
    }
    if level <= 0.0 || level >= 1.0 {
        return Err(StatError::InvalidParameter(
            "level (0, 1) aralığında olmalı".into(),
        ));
    }

    let v = variance(data)?;
    let s = v.sqrt();
    let df = (n - 1) as f64;
    let alpha = 1.0 - level;

    let chi_dist = ChiSquared::new(df)
        .map_err(|e| StatError::Numerical(format!("χ²-dağılımı kurulamadı: {e}")))?;

    let chi2_lower = chi_dist.inverse_cdf(alpha / 2.0);
    let chi2_upper = chi_dist.inverse_cdf(1.0 - alpha / 2.0);

    if chi2_lower <= 0.0 || chi2_upper <= 0.0 {
        return Err(StatError::Numerical(
            "χ² kritik değerleri sıfır veya negatif".into(),
        ));
    }

    let ci_var_low = (df * v) / chi2_upper;
    let ci_var_high = (df * v) / chi2_lower;

    Ok(VarianceCiResult {
        n,
        variance: v,
        std_dev: s,
        df,
        ci_level: level,
        ci_variance: [ci_var_low, ci_var_high],
        ci_std_dev: [ci_var_low.sqrt(), ci_var_high.sqrt()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_ci() {
        let data = [1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let res = mean_ci(&data, 0.95).unwrap();
        assert_eq!(res.n, 6);
        assert!((res.ci[0] - 1.7683428465098379).abs() < 1e-10);
        assert!((res.ci[1] - 3.731657153490162).abs() < 1e-10);
    }

    #[test]
    fn test_proportion_ci_wald() {
        let res = proportion_ci(4, 10, 0.95, "wald").unwrap();
        assert!((res.ci[0] - 0.0963636851484016).abs() < 1e-10);
        assert!((res.ci[1] - 0.7036363148515985).abs() < 1e-10);
    }

    #[test]
    fn test_proportion_ci_wilson() {
        let res = proportion_ci(4, 10, 0.95, "wilson").unwrap();
        assert!((res.ci[0] - 0.16818032970623614).abs() < 1e-10);
        assert!((res.ci[1] - 0.6873262302663417).abs() < 1e-10);
    }

    #[test]
    fn test_variance_ci() {
        let data = [1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let res = variance_ci(&data, 0.95).unwrap();
        assert!((res.ci_variance[0] - 0.3409311763236312).abs() < 1e-10);
        assert!((res.ci_variance[1] - 5.2634009547199385).abs() < 1e-10);
        assert!((res.ci_std_dev[0] - 0.58389312063393).abs() < 1e-10);
        assert!((res.ci_std_dev[1] - 2.2942103117892097).abs() < 1e-10);
    }
}
