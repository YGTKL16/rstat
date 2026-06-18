use crate::dist::pvalue::p_value_chi2;
use crate::error::StatError;
use crate::result::ChiSqResult;

/// Ki-Kare Bağımsızlık Testi (Chi-Square Test of Independence).
/// İki kategorik değişken arasındaki bağımsızlığı test eder.
pub fn independence_test(
    table: &[Vec<f64>],
    yates: bool,
    alpha: f64,
) -> Result<ChiSqResult, StatError> {
    let r = table.len();
    if r < 2 {
        return Err(StatError::InvalidParameter(
            "tablo en az 2 satır içermeli".into(),
        ));
    }
    let c = table[0].len();
    if c < 2 {
        return Err(StatError::InvalidParameter(
            "tablo en az 2 sütun içermeli".into(),
        ));
    }
    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(StatError::InvalidParameter(
            "alpha (0, 1) aralığında olmalı".into(),
        ));
    }

    // Satır sütun kontrolü
    for row in table {
        if row.len() != c {
            return Err(StatError::LengthMismatch { a: c, b: row.len() });
        }
        for &val in row {
            if val < 0.0 {
                return Err(StatError::InvalidParameter(
                    "gözlenen değerler negatif olamaz".into(),
                ));
            }
        }
    }

    // Satır ve sütun toplamları
    let mut row_sums = vec![0.0; r];
    let mut col_sums = vec![0.0; c];
    let mut n_total = 0.0;

    for i in 0..r {
        for j in 0..c {
            let val = table[i][j];
            row_sums[i] += val;
            col_sums[j] += val;
            n_total += val;
        }
    }

    if n_total <= 0.0 {
        return Err(StatError::InvalidParameter(
            "toplam gözlem sayısı sıfır veya negatif olamaz".into(),
        ));
    }

    // Beklenen değerlerin (expected) hesaplanması
    let mut expected = vec![vec![0.0; c]; r];
    let mut warning_low_expected = false;

    for i in 0..r {
        for j in 0..c {
            let exp_val = (row_sums[i] * col_sums[j]) / n_total;
            if exp_val <= 0.0 {
                return Err(StatError::Numerical(format!(
                    "beklenen değer sıfır veya negatif (satır_toplamı={}, sütun_toplamı={})",
                    row_sums[i], col_sums[j]
                )));
            }
            if exp_val < 5.0 {
                warning_low_expected = true;
            }
            expected[i][j] = exp_val;
        }
    }

    // Yates correction check (sadece 2x2 ve yates=true için)
    let is_2x2 = r == 2 && c == 2;
    let apply_yates = is_2x2 && yates;

    let mut statistic = 0.0;
    let mut chi2_uncorrected = 0.0;

    for i in 0..r {
        for j in 0..c {
            let o = table[i][j];
            let e = expected[i][j];
            let diff = (o - e).abs();

            // Uncorrected chi2 (Cramér's V için daima uncorrected kullanılacak)
            chi2_uncorrected += (o - e).powi(2) / e;

            if apply_yates {
                let term = (diff - 0.5).max(0.0).powi(2) / e;
                statistic += term;
            } else {
                statistic += (o - e).powi(2) / e;
            }
        }
    }

    let df = ((r - 1) * (c - 1)) as f64;
    let p_value = p_value_chi2(statistic, df)?;
    let reject_null = p_value < alpha;

    // Cramér's V (uncorrected chi2 ile hesaplanır)
    let min_dim = (std::cmp::min(r, c) as f64).min(r as f64); // min(R, C)
    let cramers_v = if min_dim > 1.0 {
        let v_sq = chi2_uncorrected / (n_total * (min_dim - 1.0));
        Some(v_sq.max(0.0).sqrt())
    } else {
        None
    };

    Ok(ChiSqResult {
        test: "chi-square-independence",
        statistic,
        df,
        p_value,
        observed: table.to_vec(),
        expected,
        cramers_v,
        yates_corrected: apply_yates,
        warning_low_expected,
        alpha,
        reject_null,
    })
}

/// Ki-Kare Uyum İyiliği Testi (Chi-Square Goodness of Fit Test).
/// Gözlenen frekansların beklenen frekanslar ile uyumlu olup olmadığını test eder.
pub fn goodness_of_fit_test(
    observed: &[f64],
    expected: &[f64],
    alpha: f64,
) -> Result<ChiSqResult, StatError> {
    let k = observed.len();
    if k < 2 {
        return Err(StatError::InvalidParameter(
            "uyum iyiliği testi için en az 2 kategori gerekli".into(),
        ));
    }
    if expected.len() != k {
        return Err(StatError::LengthMismatch {
            a: k,
            b: expected.len(),
        });
    }
    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(StatError::InvalidParameter(
            "alpha (0, 1) aralığında olmalı".into(),
        ));
    }

    // Negatif kontrolü
    for &o in observed {
        if o < 0.0 {
            return Err(StatError::InvalidParameter(
                "gözlenen değerler negatif olamaz".into(),
            ));
        }
    }
    for &e in expected {
        if e < 0.0 {
            return Err(StatError::InvalidParameter(
                "beklenen değerler negatif olamaz".into(),
            ));
        }
    }

    let obs_sum: f64 = observed.iter().sum();
    let exp_sum: f64 = expected.iter().sum();

    if obs_sum <= 0.0 {
        return Err(StatError::InvalidParameter(
            "gözlenen değerlerin toplamı sıfırdan büyük olmalı".into(),
        ));
    }
    if exp_sum <= 0.0 {
        return Err(StatError::InvalidParameter(
            "beklenen değerlerin toplamı sıfırdan büyük olmalı".into(),
        ));
    }

    // Beklenen değerleri gözlenenlerin toplamına göre ölçekleme (scipy tarzı)
    let scale = obs_sum / exp_sum;
    let expected_scaled: Vec<f64> = expected.iter().map(|&e| e * scale).collect();

    let mut statistic = 0.0;
    let mut warning_low_expected = false;

    for i in 0..k {
        let o = observed[i];
        let e = expected_scaled[i];
        if e <= 0.0 {
            return Err(StatError::Numerical(
                "ölçeklenmiş beklenen değer sıfır veya negatif".into(),
            ));
        }
        if e < 5.0 {
            warning_low_expected = true;
        }
        statistic += (o - e).powi(2) / e;
    }

    let df = (k - 1) as f64;
    let p_value = p_value_chi2(statistic, df)?;
    let reject_null = p_value < alpha;

    Ok(ChiSqResult {
        test: "chi-square-goodness-of-fit",
        statistic,
        df,
        p_value,
        observed: vec![observed.to_vec()],
        expected: vec![expected_scaled],
        cramers_v: None,
        yates_corrected: false,
        warning_low_expected,
        alpha,
        reject_null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goodness_of_fit_simple() {
        let obs = [20.0, 30.0, 50.0];
        let exp = [25.0, 25.0, 50.0];
        // obs_sum = 100, exp_sum = 100
        // expected is scaled: [25, 25, 50]
        // statistic: (20-25)^2/25 + (30-25)^2/25 + (50-50)^2/50 = 25/25 + 25/25 + 0 = 2.0
        // df = 2
        let res = goodness_of_fit_test(&obs, &exp, 0.05).unwrap();
        assert_eq!(res.statistic, 2.0);
        assert_eq!(res.df, 2.0);
        // scipy stats.chi2.sf(2.0, 2) -> 0.36787944117144233
        assert!((res.p_value - 0.36787944117144233).abs() < 1e-10);
        assert!(!res.reject_null);
    }

    #[test]
    fn test_goodness_of_fit_scaling() {
        let obs = [20.0, 30.0, 50.0];
        let exp = [0.25, 0.25, 0.50]; // as probabilities
        // obs_sum = 100, exp_sum = 1.0
        // scale = 100/1 = 100 -> expected_scaled = [25, 25, 50]
        // statistic should be 2.0
        let res = goodness_of_fit_test(&obs, &exp, 0.05).unwrap();
        assert_eq!(res.statistic, 2.0);
        assert!((res.p_value - 0.36787944117144233).abs() < 1e-10);
    }

    #[test]
    fn test_independence_2x2_yates() {
        // scipy.stats.chi2_contingency([[10, 20], [20, 30]], correction=True)
        // observed:
        // [10, 20] -> sum = 30
        // [20, 30] -> sum = 50
        // col sums: [30, 50], row sums: [30, 50], grand total: 80
        // expected:
        // [[30*30/80, 30*50/80], [50*30/80, 50*50/80]] = [[11.25, 18.75], [18.75, 31.25]]
        // diffs: |O-E| = |10 - 11.25| = 1.25
        // with Yates: (|1.25| - 0.5)^2 / 11.25 = 0.75^2 / 11.25 = 0.5625 / 11.25 = 0.05
        // total Yates: 0.5625 * (1/11.25 + 1/18.75 + 1/18.75 + 1/31.25)
        // = 0.5625 * (0.08888... + 0.05333... + 0.05333... + 0.032) = 0.5625 * 0.227555... = 0.128
        // Let's verify with scipy values:
        // chi2_contingency result: stat = 0.128, p_value = 0.7205147871362552
        let table = vec![vec![10.0, 20.0], vec![20.0, 30.0]];
        let res_yates = independence_test(&table, true, 0.05).unwrap();
        assert!((res_yates.statistic - 0.128).abs() < 1e-9);
        assert!((res_yates.p_value - 0.7205147871362552).abs() < 1e-9);
        assert!(res_yates.yates_corrected);

        // Without Yates:
        // chi2_contingency result: stat = 0.355555..., p_value = 0.5509849875850935
        let res_no_yates = independence_test(&table, false, 0.05).unwrap();
        assert!((res_no_yates.statistic - 0.35555555555555557).abs() < 1e-9);
        assert!((res_no_yates.p_value - 0.5509849875850935).abs() < 1e-9);
        assert!(!res_no_yates.yates_corrected);

        // Cramér's V should be based on uncorrected chi2:
        // Cramér's V = sqrt(0.355555... / (80 * 1)) = sqrt(0.0044444...) = 0.066666...
        assert!((res_yates.cramers_v.unwrap() - 0.06666666666666667).abs() < 1e-9);
    }
}
