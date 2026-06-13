use statrs::distribution::{ContinuousCDF, StudentsT};

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
}
