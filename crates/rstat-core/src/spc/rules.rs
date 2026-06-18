#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SpcViolation {
    pub rule_id: u8,
    pub subgroup_index: usize,
    pub description: String,
}

pub fn check_rules(means: &[f64], lcl: f64, ucl: f64, center: f64) -> Vec<SpcViolation> {
    let mut violations = Vec::new();

    for i in 0..means.len() {
        // Rule 1: 1 point beyond 3-sigma limits
        if means[i] < lcl || means[i] > ucl {
            violations.push(SpcViolation {
                rule_id: 1,
                subgroup_index: i,
                description: format!(
                    "Kural 1 İhlali: Nokta ({:.6}) kontrol limitlerinin dışında [LCL: {:.6}, UCL: {:.6}]",
                    means[i], lcl, ucl
                ),
            });
        }

        // Rule 2: 9 consecutive points on one side of center line
        if i >= 8 {
            let slice = &means[i - 8..=i];
            let all_above = slice.iter().all(|&x| x > center);
            let all_below = slice.iter().all(|&x| x < center);
            if all_above || all_below {
                let side = if all_above { "üstünde" } else { "altında" };
                violations.push(SpcViolation {
                    rule_id: 2,
                    subgroup_index: i,
                    description: format!(
                        "Kural 2 İhlali: 9 ardışık nokta merkez çizgisinin aynı tarafında ({})",
                        side
                    ),
                });
            }
        }

        // Rule 3: 6 consecutive points increasing or decreasing
        if i >= 5 {
            let slice = &means[i - 5..=i];
            let mut increasing = true;
            let mut decreasing = true;
            for j in 0..5 {
                if slice[j + 1] <= slice[j] {
                    increasing = false;
                }
                if slice[j + 1] >= slice[j] {
                    decreasing = false;
                }
            }
            if increasing || decreasing {
                let dir = if increasing { "artıyor" } else { "azalıyor" };
                violations.push(SpcViolation {
                    rule_id: 3,
                    subgroup_index: i,
                    description: format!("Kural 3 İhlali: 6 ardışık nokta sürekli {}", dir),
                });
            }
        }

        // Rule 4: 14 points alternating up and down
        if i >= 13 {
            let mut alternating = true;
            let mut diffs = Vec::with_capacity(13);
            for j in 0..13 {
                let idx = i - 12 + j;
                let diff = means[idx] - means[idx - 1];
                diffs.push(diff);
            }
            for j in 1..13 {
                if diffs[j] * diffs[j - 1] >= 0.0 {
                    alternating = false;
                    break;
                }
            }
            // Also ensure no diff is zero (which is checked by diffs[j] * diffs[j-1] >= 0.0)
            if alternating {
                violations.push(SpcViolation {
                    rule_id: 4,
                    subgroup_index: i,
                    description: "Kural 4 İhlali: 14 ardışık nokta inişli çıkışlı dalgalanıyor"
                        .into(),
                });
            }
        }
    }

    violations
}
