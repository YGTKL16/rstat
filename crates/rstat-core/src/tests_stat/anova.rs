use crate::data::summary::{mean, variance};
use crate::dist::pvalue::p_value_f;
use crate::error::StatError;
use crate::result::{AnovaResult, GroupStats};

/// One-way ANOVA (Tek Yönlü Varyans Analizi).
/// Farklı boyutlardaki grupları (unbalanced) destekler.
pub fn one_way(groups: &[&[f64]], alpha: f64) -> Result<AnovaResult, StatError> {
    let k = groups.len();
    if k < 2 {
        return Err(StatError::InvalidParameter(
            "ANOVA için en az 2 grup gerekli".into(),
        ));
    }
    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(StatError::InvalidParameter(
            "alpha (0, 1) aralığında olmalı".into(),
        ));
    }

    let mut group_stats = Vec::with_capacity(k);
    let mut total_n = 0;
    let mut sum_x = 0.0;
    let mut ss_within = 0.0;

    for (i, &group) in groups.iter().enumerate() {
        let n = group.len();
        if n < 2 {
            return Err(StatError::InsufficientData {
                required: 2,
                got: n,
            });
        }
        let m = mean(group)?;
        let v = variance(group)?;
        let s = v.sqrt();

        group_stats.push(GroupStats {
            name: format!("{}", i + 1),
            n,
            mean: m,
            std: s,
        });

        total_n += n;
        sum_x += group.iter().sum::<f64>();
        ss_within += (n - 1) as f64 * v;
    }

    let grand_mean = sum_x / total_n as f64;
    let mut ss_between = 0.0;
    for g in &group_stats {
        ss_between += g.n as f64 * (g.mean - grand_mean).powi(2);
    }

    let ss_total = ss_between + ss_within;
    let df_between = (k - 1) as f64;
    let df_within = (total_n - k) as f64;

    let ms_between = ss_between / df_between;
    let ms_within = ss_within / df_within;

    if ms_within == 0.0 {
        return Err(StatError::Numerical(
            "grup içi varyans sıfır, F istatistiği tanımsız".into(),
        ));
    }

    let f_statistic = ms_between / ms_within;
    let p_value = p_value_f(f_statistic, df_between, df_within)?;
    let eta_squared = ss_between / ss_total;
    let reject_null = p_value < alpha;

    Ok(AnovaResult {
        test: "one-way-anova",
        groups: group_stats,
        ss_between,
        ss_within,
        ss_total,
        df_between,
        df_within,
        ms_between,
        ms_within,
        f_statistic,
        p_value,
        eta_squared,
        alpha,
        reject_null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anova_balanced() {
        let g1 = [1.0, 2.0, 3.0, 4.0, 5.0];
        let g2 = [2.0, 3.0, 4.0, 5.0, 6.0];
        let g3 = [3.0, 4.0, 5.0, 6.0, 7.0];

        let res = one_way(&[&g1[..], &g2[..], &g3[..]], 0.05).unwrap();

        assert_eq!(res.ss_between, 10.0);
        assert_eq!(res.ss_within, 30.0);
        assert_eq!(res.ss_total, 40.0);
        assert_eq!(res.df_between, 2.0);
        assert_eq!(res.df_within, 12.0);
        assert_eq!(res.f_statistic, 2.0);
        assert!(
            (res.p_value - 0.17797851562500003).abs() < 1e-10,
            "p={}",
            res.p_value
        );
        assert_eq!(res.eta_squared, 0.25);
        assert!(!res.reject_null);
    }

    #[test]
    fn test_anova_unbalanced() {
        let g1 = [1.0, 2.0, 3.0];
        let g2 = [2.0, 3.0, 4.0, 5.0];
        let g3 = [5.0, 6.0, 7.0, 8.0, 9.0];

        let res = one_way(&[&g1[..], &g2[..], &g3[..]], 0.05).unwrap();

        assert!((res.f_statistic - 14.272058823529413).abs() < 1e-10);
        assert!((res.p_value - 0.0016167867190023655).abs() < 1e-10);
        assert!(res.reject_null);
    }

    #[test]
    fn test_anova_invalid_inputs() {
        let g1 = [1.0, 2.0];
        // En az 2 grup olmalı
        assert!(one_way(&[&g1[..]], 0.05).is_err());

        // Grupta en az 2 eleman olmalı
        let g2 = [3.0];
        assert!(one_way(&[&g1[..], &g2[..]], 0.05).is_err());

        // Geçersiz alpha
        assert!(one_way(&[&g1[..], &g1[..]], 0.0).is_err());
        assert!(one_way(&[&g1[..], &g1[..]], 1.0).is_err());
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    fn valid_group(min_len: usize) -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(
            -100.0..100.0, // overflow ve sayısal dengesizlikleri önlemek için aralığı sınırlayalım
            min_len..30,
        )
    }

    proptest! {
        #[test]
        fn prop_anova_valid(
            g1 in valid_group(2),
            g2 in valid_group(2),
            g3 in valid_group(2),
        ) {
            let groups = [&g1[..], &g2[..], &g3[..]];
            // ANOVA içi varyans sıfır olmadığı sürece sonuç dönmeli
            if let Ok(res) = one_way(&groups, 0.05) {
                prop_assert!((0.0..=1.0).contains(&res.p_value));
                prop_assert!((res.ss_total - (res.ss_between + res.ss_within)).abs() < 1e-8);
                prop_assert!(res.eta_squared >= 0.0 && res.eta_squared <= 1.0);
            }
        }
    }
}
