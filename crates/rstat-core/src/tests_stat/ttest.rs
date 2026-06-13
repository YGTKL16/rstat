use crate::data::summary::{mean, variance};
use crate::dist::pvalue::{Alternative, ci_bounds, p_value};
use crate::error::StatError;
use crate::result::{GroupStats, TTestResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    #[default]
    Welch,
    Pooled,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Welch => "welch",
            Self::Pooled => "pooled",
        }
    }
}

fn describe(data: &[f64]) -> Result<(usize, f64, f64, f64), StatError> {
    let m = mean(data)?;
    let v = variance(data)?;
    Ok((data.len(), m, v, v.sqrt()))
}

pub fn one_sample(
    data: &[f64],
    mu0: f64,
    alt: Alternative,
    ci_level: f64,
    alpha: f64,
) -> Result<TTestResult, StatError> {
    if mu0.is_nan() {
        return Err(StatError::InvalidParameter("mu0 NaN olamaz".into()));
    }
    let (n, xbar, _, s) = describe(data)?;
    if s == 0.0 {
        return Err(StatError::Numerical(
            "standart sapma sıfır, t tanımsız".into(),
        ));
    }
    let se = s / (n as f64).sqrt();
    let t = (xbar - mu0) / se;
    let df = (n - 1) as f64;
    let pv = p_value(t, df, alt)?;
    let ci = ci_bounds(xbar, se, df, ci_level, alt)?;
    let d = (xbar - mu0) / s;

    Ok(TTestResult {
        test: "one-sample",
        method: "one-sample",
        alternative: alt.as_str().to_string(),
        groups: vec![GroupStats { name: "x".into(), n, mean: xbar, std: s }],
        statistic: t,
        df,
        p_value: pv,
        mean_diff: Some(xbar - mu0),
        ci: Some(ci),
        ci_level,
        cohens_d: Some(d),
        alpha,
        reject_null: pv < alpha,
    })
}

pub fn two_sample(
    a: &[f64],
    b: &[f64],
    method: Method,
    alt: Alternative,
    ci_level: f64,
    alpha: f64,
) -> Result<TTestResult, StatError> {
    let (n1, m1, v1, s1) = describe(a)?;
    let (n2, m2, v2, s2) = describe(b)?;
    let mean_diff = m1 - m2;

    let (t, df, se) = match method {
        Method::Welch => {
            let e1 = v1 / n1 as f64;
            let e2 = v2 / n2 as f64;
            let se = (e1 + e2).sqrt();
            if se == 0.0 {
                return Err(StatError::Numerical("standart hata sıfır".into()));
            }
            let t = mean_diff / se;
            let df = (e1 + e2).powi(2)
                / (e1.powi(2) / (n1 as f64 - 1.0) + e2.powi(2) / (n2 as f64 - 1.0));
            if !df.is_finite() || df <= 0.0 {
                return Err(StatError::Numerical(format!("Welch df hesaplanamadı: {df}")));
            }
            (t, df, se)
        }
        Method::Pooled => {
            let df = (n1 + n2 - 2) as f64;
            let sp2 = ((n1 - 1) as f64 * v1 + (n2 - 1) as f64 * v2) / df;
            let se = (sp2 * (1.0 / n1 as f64 + 1.0 / n2 as f64)).sqrt();
            if se == 0.0 {
                return Err(StatError::Numerical("standart hata sıfır".into()));
            }
            (mean_diff / se, df, se)
        }
    };

    let pv = p_value(t, df, alt)?;
    let ci = ci_bounds(mean_diff, se, df, ci_level, alt)?;

    // Cohen's d: her iki yöntemde pooled std ile (yaygın konvansiyon)
    let sp = (((n1 - 1) as f64 * v1 + (n2 - 1) as f64 * v2) / (n1 + n2 - 2) as f64).sqrt();
    let d = if sp > 0.0 { Some(mean_diff / sp) } else { None };

    Ok(TTestResult {
        test: "two-sample",
        method: method.as_str(),
        alternative: alt.as_str().to_string(),
        groups: vec![
            GroupStats { name: "a".into(), n: n1, mean: m1, std: s1 },
            GroupStats { name: "b".into(), n: n2, mean: m2, std: s2 },
        ],
        statistic: t,
        df,
        p_value: pv,
        mean_diff: Some(mean_diff),
        ci: Some(ci),
        ci_level,
        cohens_d: d,
        alpha,
        reject_null: pv < alpha,
    })
}

pub fn paired(
    a: &[f64],
    b: &[f64],
    alt: Alternative,
    ci_level: f64,
    alpha: f64,
) -> Result<TTestResult, StatError> {
    if a.len() != b.len() {
        return Err(StatError::LengthMismatch { a: a.len(), b: b.len() });
    }
    let diff: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
    let (n, dbar, _, sd) = describe(&diff)?;
    if sd == 0.0 {
        return Err(StatError::Numerical("farkların standart sapması sıfır".into()));
    }
    let se = sd / (n as f64).sqrt();
    let t = dbar / se;
    let df = (n - 1) as f64;
    let pv = p_value(t, df, alt)?;
    let ci = ci_bounds(dbar, se, df, ci_level, alt)?;
    let d = dbar / sd;

    let (_, m1, _, s1) = describe(a)?;
    let (_, m2, _, s2) = describe(b)?;

    Ok(TTestResult {
        test: "paired",
        method: "paired",
        alternative: alt.as_str().to_string(),
        groups: vec![
            GroupStats { name: "a".into(), n, mean: m1, std: s1 },
            GroupStats { name: "b".into(), n, mean: m2, std: s2 },
        ],
        statistic: t,
        df,
        p_value: pv,
        mean_diff: Some(dbar),
        ci: Some(ci),
        ci_level,
        cohens_d: Some(d),
        alpha,
        reject_null: pv < alpha,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sabit veri setleri — crossvalidate_ttest.py ile aynı
    const X: &[f64] = &[5.1, 4.9, 6.2, 5.5, 5.0, 4.8, 6.1, 5.3, 5.7, 4.95];
    const A: &[f64] = &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    const B: &[f64] = &[1.0, 2.0, 3.0, 3.5, 4.0, 4.0, 5.0, 8.0];
    const P1: &[f64] = &[10.2, 11.5, 9.8, 12.1, 10.9, 11.3];
    const P2: &[f64] = &[9.9, 11.0, 9.5, 11.8, 10.5, 11.1];

    // Referans değerler: scripts/crossvalidate_ttest.py çıktısından

    // Referans: scripts/crossvalidate_ttest.py çıktısından (scipy)

    #[test]
    fn test_one_sample_two_sided() {
        let r = one_sample(X, 5.0, Alternative::TwoSided, 0.95, 0.05).unwrap();
        assert!((r.statistic - 2.2292973506512723).abs() < 1e-10, "t={}", r.statistic);
        assert!((r.p_value - 0.05275727446494313).abs() < 1e-10, "p={}", r.p_value);
        assert!((r.df - 9.0).abs() < 1e-10);
        assert!(!r.reject_null);
    }

    #[test]
    fn test_two_sample_welch_two_sided() {
        let r = two_sample(A, B, Method::Welch, Alternative::TwoSided, 0.95, 0.05).unwrap();
        assert!((r.statistic - 1.1198635152570917).abs() < 1e-10, "t={}", r.statistic);
        assert!((r.p_value - 0.28162172376376).abs() < 1e-10, "p={}", r.p_value);
        assert!((r.df - 13.996246042536976).abs() < 1e-10, "df={}", r.df);
    }

    #[test]
    fn test_two_sample_pooled_two_sided() {
        let r = two_sample(A, B, Method::Pooled, Alternative::TwoSided, 0.95, 0.05).unwrap();
        assert!((r.statistic - 1.1198635152570917).abs() < 1e-10, "t={}", r.statistic);
        assert!((r.p_value - 0.2816167684249695).abs() < 1e-10, "p={}", r.p_value);
        assert!((r.df - 14.0).abs() < 1e-10, "df={}", r.df);
    }

    #[test]
    fn test_paired_two_sided() {
        let r = paired(P1, P2, Alternative::TwoSided, 0.95, 0.05).unwrap();
        assert!((r.statistic - 7.905694150420959).abs() < 1e-10, "t={}", r.statistic);
        assert!((r.p_value - 0.0005210669895035266).abs() < 1e-10, "p={}", r.p_value);
        assert!((r.df - 5.0).abs() < 1e-10);
        assert!(r.reject_null);
    }

    #[test]
    fn test_paired_length_mismatch() {
        assert!(matches!(
            paired(&[1.0, 2.0], &[1.0], Alternative::TwoSided, 0.95, 0.05),
            Err(StatError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn test_one_sample_zero_std() {
        assert!(matches!(
            one_sample(&[5.0, 5.0, 5.0], 5.0, Alternative::TwoSided, 0.95, 0.05),
            Err(StatError::Numerical(_))
        ));
    }

    #[test]
    fn test_one_sample_less() {
        let r = one_sample(X, 5.0, Alternative::Less, 0.95, 0.05).unwrap();
        assert!((r.p_value - 0.9736213627675284).abs() < 1e-10, "p={}", r.p_value);
        // Tek-yönlü CI: [-∞, ub]
        let ci = r.ci.unwrap();
        assert_eq!(ci[0], f64::NEG_INFINITY);
        assert!(ci[1].is_finite());
    }

    #[test]
    fn test_one_sample_greater() {
        let r = one_sample(X, 5.0, Alternative::Greater, 0.95, 0.05).unwrap();
        assert!((r.p_value - 0.026378637232471566).abs() < 1e-10, "p={}", r.p_value);
        // Tek-yönlü CI: [lb, +∞]
        let ci = r.ci.unwrap();
        assert!(ci[0].is_finite());
        assert_eq!(ci[1], f64::INFINITY);
    }
}
