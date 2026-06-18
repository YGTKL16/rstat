use crate::error::StatError;
use crate::result::SummaryStats;

pub fn mean(data: &[f64]) -> Result<f64, StatError> {
    if data.is_empty() {
        return Err(StatError::EmptyData);
    }
    Ok(data.iter().sum::<f64>() / data.len() as f64)
}

/// Welford online algoritması — naif Σx² yöntemiyle karşılaştırıldığında
/// büyük değerlerde cancellation hatası olmaz.
pub fn variance(data: &[f64]) -> Result<f64, StatError> {
    if data.len() < 2 {
        return Err(StatError::InsufficientData {
            required: 2,
            got: data.len(),
        });
    }
    let mut mean = 0.0_f64;
    let mut m2 = 0.0_f64;
    for (i, &x) in data.iter().enumerate() {
        let delta = x - mean;
        mean += delta / (i + 1) as f64;
        m2 += delta * (x - mean);
    }
    Ok(m2 / (data.len() - 1) as f64)
}

pub fn std_dev(data: &[f64]) -> Result<f64, StatError> {
    variance(data).map(f64::sqrt)
}

pub fn quantile(data: &[f64], p: f64) -> Result<f64, StatError> {
    if data.is_empty() {
        return Err(StatError::EmptyData);
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let h = p * (n - 1) as f64;
    let lo = h.floor() as usize;
    let hi = h.ceil() as usize;
    if lo == hi {
        Ok(sorted[lo])
    } else {
        Ok(sorted[lo] + (h - lo as f64) * (sorted[hi] - sorted[lo]))
    }
}

pub fn summary(data: &[f64]) -> Result<SummaryStats, StatError> {
    if data.is_empty() {
        return Err(StatError::EmptyData);
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    Ok(SummaryStats {
        n: data.len(),
        mean: mean(data)?,
        std: std_dev(data)?,
        min: sorted[0],
        q1: quantile(data, 0.25)?,
        median: quantile(data, 0.50)?,
        q3: quantile(data, 0.75)?,
        max: *sorted.last().unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Referans: scipy.stats - elle hesaplanmış değerler
    const DATA: &[f64] = &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];

    #[test]
    fn test_mean() {
        let m = mean(DATA).unwrap();
        assert!((m - 5.0).abs() < 1e-10, "mean={m}");
    }

    #[test]
    fn test_variance() {
        // scipy: np.var([2,4,4,4,5,5,7,9], ddof=1) = 4.571428...
        let v = variance(DATA).unwrap();
        assert!((v - 4.571428571428571).abs() < 1e-10, "var={v}");
    }

    #[test]
    fn test_std() {
        let s = std_dev(DATA).unwrap();
        assert!((s - 2.138089935299395).abs() < 1e-10, "std={s}");
    }

    #[test]
    fn test_median() {
        let m = quantile(DATA, 0.5).unwrap();
        assert!((m - 4.5).abs() < 1e-10, "median={m}");
    }

    #[test]
    fn test_empty_mean() {
        assert!(matches!(mean(&[]), Err(StatError::EmptyData)));
    }

    #[test]
    fn test_single_variance() {
        assert!(matches!(
            variance(&[1.0]),
            Err(StatError::InsufficientData { .. })
        ));
    }
}
