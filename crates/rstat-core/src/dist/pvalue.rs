use statrs::distribution::{ChiSquared, ContinuousCDF, FisherSnedecor, StudentsT};

use crate::error::StatError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alternative {
    TwoSided,
    Less,
    Greater,
}

impl Alternative {
    pub fn parse(s: &str) -> Result<Self, StatError> {
        match s {
            "two-sided" | "two_sided" | "two" => Ok(Self::TwoSided),
            "less" => Ok(Self::Less),
            "greater" => Ok(Self::Greater),
            other => Err(StatError::InvalidParameter(format!(
                "bilinmeyen alternative: '{other}' (two-sided|less|greater)"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TwoSided => "two-sided",
            Self::Less => "less",
            Self::Greater => "greater",
        }
    }
}

/// t-istatistiği ve df'den p-değeri.
/// İki taraflı için 2*sf(|t|) + clamp — scipy ile birebir.
pub fn p_value(t: f64, df: f64, alt: Alternative) -> Result<f64, StatError> {
    if !df.is_finite() || df <= 0.0 {
        return Err(StatError::Numerical(format!("geçersiz df: {df}")));
    }
    if t.is_nan() {
        return Err(StatError::Numerical("t istatistiği NaN".into()));
    }
    let dist = StudentsT::new(0.0, 1.0, df)
        .map_err(|e| StatError::Numerical(format!("t-dağılımı kurulamadı: {e}")))?;

    let p = match alt {
        Alternative::TwoSided => 2.0 * dist.sf(t.abs()),
        Alternative::Greater => dist.sf(t),
        Alternative::Less => dist.cdf(t),
    };
    Ok(p.clamp(0.0, 1.0))
}

/// İki taraflı CI için kritik t değeri: t_{1 - alpha/2, df}.
pub fn critical_t(ci_level: f64, df: f64) -> Result<f64, StatError> {
    if !(0.0 < ci_level && ci_level < 1.0) {
        return Err(StatError::InvalidParameter(format!(
            "ci_level (0,1) aralığında olmalı: {ci_level}"
        )));
    }
    if !df.is_finite() || df <= 0.0 {
        return Err(StatError::Numerical(format!("geçersiz df: {df}")));
    }
    let dist = StudentsT::new(0.0, 1.0, df)
        .map_err(|e| StatError::Numerical(format!("t-dağılımı kurulamadı: {e}")))?;
    Ok(dist.inverse_cdf(1.0 - (1.0 - ci_level) / 2.0))
}

/// Alternative'a göre doğru CI hesaplar.
/// - TwoSided: [est - tc*se, est + tc*se] (tc = t_{1-α/2})
/// - Greater:  [est - tc*se, +∞]         (tc = t_{ci_level})
/// - Less:     [-∞, est + tc*se]          (tc = t_{ci_level})
pub fn ci_bounds(
    est: f64,
    se: f64,
    df: f64,
    ci_level: f64,
    alt: Alternative,
) -> Result<[f64; 2], StatError> {
    if !df.is_finite() || df <= 0.0 {
        return Err(StatError::Numerical(format!("geçersiz df: {df}")));
    }
    if !(0.0 < ci_level && ci_level < 1.0) {
        return Err(StatError::InvalidParameter(format!(
            "ci_level (0,1) aralığında olmalı: {ci_level}"
        )));
    }
    let dist = StudentsT::new(0.0, 1.0, df)
        .map_err(|e| StatError::Numerical(format!("t-dağılımı kurulamadı: {e}")))?;
    Ok(match alt {
        Alternative::TwoSided => {
            let tc = dist.inverse_cdf(1.0 - (1.0 - ci_level) / 2.0);
            [est - tc * se, est + tc * se]
        }
        Alternative::Greater => {
            let tc = dist.inverse_cdf(ci_level);
            [est - tc * se, f64::INFINITY]
        }
        Alternative::Less => {
            let tc = dist.inverse_cdf(ci_level);
            [f64::NEG_INFINITY, est + tc * se]
        }
    })
}

/// F istatistiğinden p-değeri (daima sağ kuyruk)
pub fn p_value_f(f: f64, df1: f64, df2: f64) -> Result<f64, StatError> {
    if !df1.is_finite() || df1 <= 0.0 || !df2.is_finite() || df2 <= 0.0 {
        return Err(StatError::InvalidParameter(format!(
            "geçersiz df: df1={df1}, df2={df2}"
        )));
    }
    if f.is_nan() {
        return Err(StatError::Numerical("F istatistiği NaN".into()));
    }
    let dist = FisherSnedecor::new(df1, df2)
        .map_err(|e| StatError::Numerical(format!("F-dağılımı kurulamadı: {e}")))?;
    Ok((1.0 - dist.cdf(f)).clamp(0.0, 1.0))
}

/// Chi-square istatistiğinden p-değeri
pub fn p_value_chi2(chi_sq: f64, df: f64) -> Result<f64, StatError> {
    if !df.is_finite() || df <= 0.0 {
        return Err(StatError::InvalidParameter(format!("geçersiz df: df={df}")));
    }
    if chi_sq.is_nan() {
        return Err(StatError::Numerical("chi_sq istatistiği NaN".into()));
    }
    let dist = ChiSquared::new(df)
        .map_err(|e| StatError::Numerical(format!("χ²-dağılımı kurulamadı: {e}")))?;
    Ok((1.0 - dist.cdf(chi_sq)).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Referans: scipy.stats.t — python3 -c "from scipy import stats; ..."
    // stats.t.sf(2.5, 10) * 2
    #[test]
    fn test_two_sided() {
        let p = p_value(2.5, 10.0, Alternative::TwoSided).unwrap();
        assert!((p - 0.03144684423660882).abs() < 1e-10, "p={p}");
    }

    // stats.t.sf(2.5, 10)
    #[test]
    fn test_greater() {
        let p = p_value(2.5, 10.0, Alternative::Greater).unwrap();
        assert!((p - 0.01572342211830441).abs() < 1e-10, "p={p}");
    }

    // stats.t.cdf(2.5, 10)
    #[test]
    fn test_less() {
        let p = p_value(2.5, 10.0, Alternative::Less).unwrap();
        assert!((p - 0.9842765778816955).abs() < 1e-10, "p={p}");
    }

    // stats.t.ppf(0.975, 10)
    #[test]
    fn test_critical_t() {
        let t = critical_t(0.95, 10.0).unwrap();
        assert!((t - 2.228138851986274).abs() < 1e-10, "t={t}");
    }

    #[test]
    fn test_invalid_df() {
        assert!(p_value(1.0, 0.0, Alternative::TwoSided).is_err());
        assert!(p_value(1.0, f64::INFINITY, Alternative::TwoSided).is_err());
    }

    #[test]
    fn test_invalid_ci_level() {
        assert!(critical_t(0.0, 10.0).is_err());
        assert!(critical_t(1.0, 10.0).is_err());
    }

    #[test]
    fn test_p_value_f() {
        // scipy: stats.f.sf(3.5, 2, 10) -> 0.07042962777237427
        let p = p_value_f(3.5, 2.0, 10.0).unwrap();
        assert!((p - 0.07042962777237427).abs() < 1e-10, "p={p}");

        // scipy: stats.f.sf(1.0, 1, 1) -> 0.5
        let p = p_value_f(1.0, 1.0, 1.0).unwrap();
        assert!((p - 0.5).abs() < 1e-10, "p={p}");

        // scipy: stats.f.sf(0.5, 4, 20) -> 0.7360371889109243
        let p = p_value_f(0.5, 4.0, 20.0).unwrap();
        assert!((p - 0.7360371889109243).abs() < 1e-10, "p={p}");

        // scipy: stats.f.sf(10.2, 5, 12) -> 0.0005361417572438391
        let p = p_value_f(10.2, 5.0, 12.0).unwrap();
        assert!((p - 0.0005361417572438391).abs() < 1e-10, "p={p}");
    }

    #[test]
    fn test_p_value_chi2() {
        // scipy: stats.chi2.sf(5.5, 3) -> 0.1386386173824151
        let p = p_value_chi2(5.5, 3.0).unwrap();
        assert!((p - 0.1386386173824151).abs() < 1e-10, "p={p}");

        // scipy: stats.chi2.sf(1.2, 1) -> 0.273321678292295
        let p = p_value_chi2(1.2, 1.0).unwrap();
        assert!((p - 0.273321678292295).abs() < 1e-10, "p={p}");

        // scipy: stats.chi2.sf(15.0, 5) -> 0.010362337915786429
        let p = p_value_chi2(15.0, 5.0).unwrap();
        assert!((p - 0.010362337915786429).abs() < 1e-10, "p={p}");

        // scipy: stats.chi2.sf(0.5, 2) -> 0.7788007830714049
        let p = p_value_chi2(0.5, 2.0).unwrap();
        assert!((p - 0.7788007830714049).abs() < 1e-10, "p={p}");
    }
}
